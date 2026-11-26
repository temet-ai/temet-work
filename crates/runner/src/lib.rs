//! Runners: where agent sessions run. A runner starts a labelled terminal, sends
//! it text, reads its screen, stops it, and reports how each terminal's status
//! changes.
//!
//! This crate holds the [`Runner`] contract, a deterministic [`fake::FakeRunner`],
//! and the [`contract`] tests every runner adapter runs. Adapter crates depend on
//! this crate only.

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use temet_work_core::brief;
use temet_work_core::ids::AgentId;
use temet_work_core::launch::AGENT_VAR;
use temet_work_core::model::{AgentState, Role};
use temet_work_core::world::LOCAL;
use temet_work_core::{Effect, Event, Outcome, Part, Polled, World};

/// The core type this contract uses, re-exported so adapters need no other crate.
pub use temet_work_core::launch::Program;

pub mod contract;
pub mod fake;

/// A terminal, by the id its runner gave it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerminalId(pub String);

impl fmt::Display for TerminalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What to start, where, and how to label it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Start {
    /// The program.
    pub program: Program,
    /// The folder it starts in.
    pub cwd: PathBuf,
    /// Labels, such as `item` and `role`, so updates can be told apart. They are
    /// set when the terminal starts, so its first update already carries them.
    pub labels: BTreeMap<String, String>,
    /// The machine, when the runner reaches more than one.
    pub machine: Option<String>,
}

/// A terminal's status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Between turns. A runner that tells "idle" from "done" reports both as this:
    /// neither means the agent is ready for its next task, only that a turn ended.
    Between,
    /// In a turn.
    Working,
    /// Stopped at a prompt; typing into it would answer the prompt.
    Blocked {
        /// What kind of prompt, as the runner names it (`question`, `permission`).
        kind: String,
    },
    /// The program has ended.
    Gone,
    /// The program reports no status, as a plain shell does.
    Unknown,
}

/// A change in one terminal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Update {
    /// The terminal.
    pub terminal: TerminalId,
    /// Its labels.
    pub labels: BTreeMap<String, String>,
    /// Its status now.
    pub status: Status,
    /// What the agent says it is doing, when it says.
    pub doing: Option<String>,
}

/// Why a runner call failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunnerError {
    /// The terminal is at a prompt; nothing was typed.
    Blocked,
    /// The terminal is in a turn and the send asked for idle only; nothing was typed.
    Busy,
    /// The terminal's program has ended.
    Gone,
    /// No such terminal.
    Unknown(TerminalId),
    /// This adapter does not do this yet; the text says what is missing.
    NotBuilt(String),
    /// Anything else, as the runner reported it.
    Other(String),
}

impl fmt::Display for RunnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocked => f.write_str("the terminal is at a prompt; nothing was typed"),
            Self::Busy => f.write_str("the terminal is in a turn; nothing was typed"),
            Self::Gone => f.write_str("the terminal's program has ended"),
            Self::Unknown(t) => write!(f, "no terminal {t}"),
            Self::NotBuilt(what) => write!(f, "not built yet: {what}"),
            Self::Other(e) => f.write_str(e),
        }
    }
}

impl std::error::Error for RunnerError {}

/// One runner, added by an adapter crate.
pub trait Runner: Send {
    /// A short, stable id, the adapter's feature name after `runner-`.
    fn id(&self) -> &'static str;

    /// Starts a program in a new labelled terminal.
    fn start(&mut self, start: &Start) -> Result<TerminalId, RunnerError>;

    /// Types `text` and presses Enter. With `only_if_idle`, a terminal in a turn
    /// refuses with [`RunnerError::Busy`]; otherwise the text waits for the next
    /// turn. A terminal at a prompt always refuses with [`RunnerError::Blocked`].
    fn send(
        &mut self,
        terminal: &TerminalId,
        text: &str,
        only_if_idle: bool,
    ) -> Result<(), RunnerError>;

    /// The visible screen, as text. Cheap; history is not included.
    fn screen(&mut self, terminal: &TerminalId) -> Result<String, RunnerError>;

    /// Ends the terminal's program and removes the terminal.
    fn stop(&mut self, terminal: &TerminalId) -> Result<(), RunnerError>;

    /// The updates since the last call, oldest first.
    fn updates(&mut self) -> Vec<Update>;
}

/// The runner part: the runners compiled into this build, in the order they were added.
///
/// It starts each agent's program in a labelled terminal with the first runner in
/// this build, turns runner updates into agent events, and delivers text to an
/// agent between its turns, holding it while the agent is working.
#[derive(Default)]
pub struct Runners {
    adapters: Vec<Box<dyn Runner>>,
    /// Each agent's terminal: the runner that holds it and its id there.
    terminals: BTreeMap<String, (usize, TerminalId)>,
    /// Text waiting for its agent to finish a turn, as (agent, text).
    outbox: Vec<(String, String)>,
}

impl Runners {
    /// Adds a runner. The root package calls this once per compiled-in adapter.
    pub fn add(&mut self, adapter: Box<dyn Runner>) {
        self.adapters.push(adapter);
    }

    /// The ids of the runners in this build.
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }
}

impl Runners {
    /// What an agent's terminal shows now.
    pub fn screen(&mut self, agent: &str) -> Result<String, RunnerError> {
        let (i, terminal) = self
            .terminals
            .get(agent)
            .cloned()
            .ok_or_else(|| RunnerError::Unknown(TerminalId(agent.to_string())))?;
        self.adapters[i].screen(&terminal)
    }

    /// How many messages are waiting for their agent to finish a turn.
    pub fn waiting(&self) -> usize {
        self.outbox.len()
    }

    /// Sends what is waiting; keeps what the agent cannot take yet.
    fn deliver(&mut self) {
        let mut keep = Vec::new();
        for (agent, text) in std::mem::take(&mut self.outbox) {
            let Some((i, terminal)) = self.terminals.get(&agent).cloned() else {
                keep.push((agent, text));
                continue;
            };
            match self.adapters[i].send(&terminal, &text, true) {
                Ok(()) => {}
                Err(RunnerError::Busy | RunnerError::Blocked) => keep.push((agent, text)),
                // The terminal is gone: its updates report the agent ended.
                Err(_) => {}
            }
        }
        self.outbox = keep;
    }
}

fn state_of(status: &Status, doing: Option<&str>) -> Option<AgentState> {
    match status {
        Status::Between => Some(AgentState::Idle),
        Status::Working => Some(AgentState::Working {
            doing: doing.unwrap_or_default().to_string(),
        }),
        Status::Blocked { kind } => Some(AgentState::Blocked {
            reason: kind.clone(),
        }),
        Status::Gone => Some(AgentState::Ended),
        Status::Unknown => None,
    }
}

impl Part for Runners {
    fn id(&self) -> &'static str {
        "runner"
    }

    fn poll(&mut self, world: &World, _now_ms: u64) -> Polled {
        let mut events = Vec::new();
        for (i, adapter) in self.adapters.iter_mut().enumerate() {
            for u in adapter.updates() {
                let Some(agent) = u.labels.get("agent").cloned() else {
                    continue;
                };
                self.terminals
                    .insert(agent.clone(), (i, u.terminal.clone()));
                let Some(state) = state_of(&u.status, u.doing.as_deref()) else {
                    continue;
                };
                let id = AgentId::new(agent);
                if world.agents.get(&id).is_some_and(|a| a.state == state) {
                    continue;
                }
                events.push(Event::AgentChanged { agent: id, state });
            }
        }
        self.deliver();
        Polled {
            events,
            problem: None,
        }
    }

    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        match effect {
            Effect::Run { launch, program } => {
                let Some(runner) = self.adapters.first_mut() else {
                    return Outcome::Failed(
                        "no terminal runner is compiled into this build".into(),
                    );
                };
                let brief = &launch.brief;
                let labels = BTreeMap::from([
                    ("agent".to_string(), launch.agent.0.clone()),
                    ("group".to_string(), launch.group.0.clone()),
                    ("issue".to_string(), brief.issue.to_string()),
                    ("role".to_string(), brief.role.name().to_string()),
                ]);
                let mut program = program.clone();
                // Commands the agent runs are recorded as its own.
                program
                    .env
                    .push((AGENT_VAR.to_string(), launch.agent.0.clone()));
                let start = Start {
                    program,
                    cwd: brief
                        .folder
                        .clone()
                        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
                        .unwrap_or_else(|| PathBuf::from("/")),
                    labels,
                    machine: (launch.machine.0 != LOCAL).then(|| launch.machine.0.clone()),
                };
                match runner.start(&start) {
                    Ok(terminal) => {
                        self.terminals.insert(launch.agent.0.clone(), (0, terminal));
                        Outcome::Done {
                            events: vec![Event::AgentStarted {
                                agent: launch.agent.clone(),
                                group: launch.group.clone(),
                                role: brief.role,
                                issue: Some(brief.issue.clone()),
                                machine: launch.machine.clone(),
                                folder: launch.folder.clone(),
                            }],
                            effects: Vec::new(),
                        }
                    }
                    Err(e) => Outcome::Failed(e.to_string()),
                }
            }
            Effect::RequestFix { issue, points } => {
                let Some(author) = world.running(issue, Role::Author) else {
                    return Outcome::Failed(format!("no author is running for {issue}"));
                };
                self.outbox
                    .push((author.id.0.clone(), brief::fixes(issue, points)));
                self.deliver();
                Outcome::done()
            }
            // A reviewer still running hears of the next round; with none, a new
            // one is started elsewhere.
            Effect::StartReview { issue, round } => {
                let Some(reviewer) = world.running(issue, Role::Reviewer) else {
                    return Outcome::NotMine;
                };
                self.outbox
                    .push((reviewer.id.0.clone(), brief::next_round(issue, *round)));
                self.deliver();
                Outcome::done()
            }
            _ => Outcome::NotMine,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeRunner;

    use temet_work_core::ids::{FolderId, GroupId, IssueId, MachineId, ProjectId};
    use temet_work_core::launch::Program;

    /// A world with issue ankh#1 and a work folder for it.
    fn world() -> World {
        let mut w = World::default();
        w.apply(&Event::FolderAdded {
            folder: FolderId::new("f1"),
            project: ProjectId::new("ankh"),
            machine: MachineId::new(LOCAL),
            path: ".".into(),
        });
        w.apply(&Event::IssueSynced {
            issue: IssueId::new("ankh", "1"),
            title: "Mend the clock".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        w
    }

    /// Runs `argv` as an author on ankh#1 and applies what the part reports.
    fn run(part: &mut Runners, w: &mut World, argv: &[&str]) -> AgentId {
        let launch = w
            .launch(
                &GroupId::new("watch"),
                Role::Author,
                &IssueId::new("ankh", "1"),
                Some(&FolderId::new("f1")),
            )
            .expect("launch");
        let agent = launch.agent.clone();
        let run = Effect::Run {
            launch,
            program: Program {
                argv: argv.iter().map(|a| a.to_string()).collect(),
                env: vec![],
            },
        };
        let Outcome::Done { events, .. } = part.carry_out(&run, w) else {
            panic!("not started");
        };
        for e in &events {
            w.apply(e);
        }
        assert!(matches!(events.as_slice(), [Event::AgentStarted { .. }]));
        agent
    }

    fn runners() -> Runners {
        let mut part = Runners::default();
        part.add(Box::new(FakeRunner::default()));
        part
    }

    #[test]
    fn starts_a_labelled_terminal_and_reports_only_changes() {
        let mut w = world();
        let mut part = runners();
        let agent = run(&mut part, &mut w, &["cat"]);
        assert_eq!(agent.0, "author-1");
        assert_eq!(w.agents[&agent].state, AgentState::Idle);
        // The terminal's first update says it is between turns: the agent already is.
        assert_eq!(part.poll(&w, 0).events, vec![]);
    }

    #[test]
    fn a_terminal_at_a_prompt_makes_its_agent_blocked() {
        let mut w = world();
        let mut part = runners();
        let agent = run(&mut part, &mut w, &[fake::PROMPT_PROGRAM]);
        assert_eq!(
            part.poll(&w, 0).events,
            vec![Event::AgentChanged {
                agent,
                state: AgentState::Blocked {
                    reason: "question".into()
                }
            }]
        );
    }

    #[test]
    fn fixes_reach_the_author_between_turns_and_wait_while_it_is_at_a_prompt() {
        let fix = Effect::RequestFix {
            issue: IssueId::new("ankh", "1"),
            points: vec!["no test for a stopped clock".into()],
        };
        let mut w = world();
        let mut part = runners();
        let idle = run(&mut part, &mut w, &["cat"]);
        assert_eq!(part.carry_out(&fix, &w), Outcome::done());
        let screen = part.screen(&idle.0).expect("screen");
        assert!(screen.contains("- no test for a stopped clock"), "{screen}");
        assert!(screen.contains("temet-work ready ankh#1"));

        let mut w = world();
        let mut part = runners();
        let asking = run(&mut part, &mut w, &[fake::PROMPT_PROGRAM]);
        part.carry_out(&fix, &w);
        assert_eq!(part.waiting(), 1);
        assert!(
            !part
                .screen(&asking.0)
                .expect("screen")
                .contains("stopped clock")
        );
    }

    #[test]
    fn a_running_reviewer_hears_of_the_next_round_and_none_is_not_this_parts() {
        let next = Effect::StartReview {
            issue: IssueId::new("ankh", "1"),
            round: 2,
        };
        let mut w = world();
        let mut part = runners();
        assert_eq!(part.carry_out(&next, &w), Outcome::NotMine);
        let launch = w
            .launch(
                &GroupId::new("watch"),
                Role::Reviewer,
                &IssueId::new("ankh", "1"),
                None,
            )
            .expect("launch");
        let reviewer = launch.agent.clone();
        let run = Effect::Run {
            launch,
            program: Program {
                argv: vec!["cat".into()],
                env: vec![],
            },
        };
        let Outcome::Done { events, .. } = part.carry_out(&run, &w) else {
            panic!("not started");
        };
        for e in &events {
            w.apply(e);
        }
        assert_eq!(part.carry_out(&next, &w), Outcome::done());
        let screen = part.screen(&reviewer.0).expect("screen");
        assert!(screen.contains("review round 2"), "{screen}");
    }

    #[test]
    fn a_fix_with_no_author_running_fails() {
        let fix = Effect::RequestFix {
            issue: IssueId::new("ankh", "1"),
            points: vec![],
        };
        assert!(matches!(
            runners().carry_out(&fix, &world()),
            Outcome::Failed(_)
        ));
    }

    #[test]
    fn keeps_adapters_in_the_order_added() {
        let mut part = Runners::default();
        part.add(Box::new(FakeRunner::named("first")));
        part.add(Box::new(FakeRunner::named("second")));
        assert_eq!(part.ids(), ["first", "second"]);
        assert_eq!(part.id(), "runner");
    }

    #[test]
    fn the_fake_passes_the_contract() {
        let mut runner = FakeRunner::default();
        let mut harness = fake::FakeHarness;
        assert_eq!(contract::check(&mut runner, &mut harness), Ok(()));
    }

    /// A runner that types into a prompt: the contract must reject it.
    struct TypesIntoPrompts(FakeRunner);

    impl Runner for TypesIntoPrompts {
        fn id(&self) -> &'static str {
            "types-into-prompts"
        }
        fn start(&mut self, start: &Start) -> Result<TerminalId, RunnerError> {
            self.0.start(start)
        }
        fn send(
            &mut self,
            terminal: &TerminalId,
            text: &str,
            only_if_idle: bool,
        ) -> Result<(), RunnerError> {
            match self.0.send(terminal, text, only_if_idle) {
                Err(RunnerError::Blocked) => Ok(()),
                other => other,
            }
        }
        fn screen(&mut self, terminal: &TerminalId) -> Result<String, RunnerError> {
            self.0.screen(terminal)
        }
        fn stop(&mut self, terminal: &TerminalId) -> Result<(), RunnerError> {
            self.0.stop(terminal)
        }
        fn updates(&mut self) -> Vec<Update> {
            self.0.updates()
        }
    }

    #[test]
    fn the_contract_rejects_a_runner_that_types_into_a_prompt() {
        let mut runner = TypesIntoPrompts(FakeRunner::default());
        let mut harness = fake::FakeHarness;
        assert_eq!(
            contract::check(&mut runner, &mut harness),
            Err("a send to a terminal at a prompt is refused, and nothing is typed".into())
        );
    }
}
