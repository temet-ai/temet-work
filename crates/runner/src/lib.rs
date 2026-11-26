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

    /// What stops the runner from reporting updates now, such as a terminal
    /// service it cannot reach.
    fn problem(&self) -> Option<String> {
        None
    }

    /// The terminals it has listed in full since the last call, if it has: on
    /// connecting, a runner lists every terminal it holds, and one that is not on
    /// the list is gone. Its updates come with the list, in `updates`.
    fn listed(&mut self) -> Option<Vec<TerminalId>> {
        None
    }
}

/// How long an agent its issue waits on may sit idle before it is reminded.
pub const NUDGE_AFTER_MS: u64 = 600_000;

/// Reminders before an idle agent is stopped, so a fresh one takes over.
pub const NUDGES: u32 = 2;

/// Text waiting for an agent to finish its turn.
struct Message {
    agent: String,
    issue: temet_work_core::ids::IssueId,
    text: String,
    /// Whether it is what the issue's stage needs the agent to hear, so its
    /// arrival counts as told. A reminder does not.
    tells: bool,
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
    /// Text waiting for its agent to finish a turn.
    outbox: Vec<Message>,
    /// Agents started since the runner last listed its terminals: a list taken
    /// before they started does not have them.
    started_here: std::collections::BTreeSet<String>,
    /// Idle agents their issues wait on: since when, and how many reminders.
    idle: BTreeMap<String, (u64, u32)>,
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
    /// Reminds an idle agent its issue waits on, twice, ten minutes apart; ten
    /// minutes after the second reminder it stops the agent's session, so the loop
    /// starts a fresh one. Returns the agents it stopped, as ended.
    fn remind(
        &mut self,
        world: &World,
        seen: &BTreeMap<AgentId, AgentState>,
        now_ms: u64,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let waiting: Vec<(String, Role, temet_work_core::ids::IssueId)> = world
            .agents
            .values()
            .filter(|a| {
                let state = seen.get(&a.id).unwrap_or(&a.state);
                *state == AgentState::Idle && world.waiting_on(a)
            })
            .filter_map(|a| a.issue.clone().map(|i| (a.id.0.clone(), a.role, i)))
            .collect();
        self.idle
            .retain(|agent, _| waiting.iter().any(|(a, _, _)| a == agent));
        for (agent, role, issue) in waiting {
            let (since, nudges) = *self.idle.entry(agent.clone()).or_insert((now_ms, 0));
            if now_ms < since + NUDGE_AFTER_MS {
                continue;
            }
            if nudges < NUDGES {
                self.idle.insert(agent.clone(), (now_ms, nudges + 1));
                events.extend(self.queue(Message {
                    agent,
                    issue: issue.clone(),
                    text: brief::nudge(role, &issue),
                    tells: false,
                }));
                continue;
            }
            self.idle.remove(&agent);
            if let Some((i, terminal)) = self.terminals.remove(&agent) {
                let _ = self.adapters[i].stop(&terminal);
            }
            events.push(Event::AgentChanged {
                agent: AgentId::new(agent),
                state: AgentState::Ended,
            });
        }
        events
    }

    /// Queues a message unless the same one is already waiting, then sends what
    /// can be sent. Returns a `Told` event for each issue whose message arrived.
    fn queue(&mut self, message: Message) -> Vec<Event> {
        let waiting = self
            .outbox
            .iter()
            .any(|m| m.agent == message.agent && m.text == message.text);
        if !waiting {
            self.outbox.push(message);
        }
        self.deliver()
    }

    /// Sends what is waiting and keeps what the agent cannot take yet. Returns a
    /// `Told` event for each issue whose message arrived.
    fn deliver(&mut self) -> Vec<Event> {
        let mut keep = Vec::new();
        let mut told = Vec::new();
        for m in std::mem::take(&mut self.outbox) {
            let Some((i, terminal)) = self.terminals.get(&m.agent).cloned() else {
                keep.push(m);
                continue;
            };
            match self.adapters[i].send(&terminal, &m.text, true) {
                Ok(()) if m.tells => told.push(Event::Told { issue: m.issue }),
                Ok(()) => {}
                // The terminal is gone, so its agent has ended; the loop starts and
                // tells the next one.
                Err(RunnerError::Gone | RunnerError::Unknown(_)) => {
                    self.terminals.remove(&m.agent);
                    told.push(Event::AgentChanged {
                        agent: AgentId::new(m.agent),
                        state: AgentState::Ended,
                    });
                }
                // Busy, at a prompt, or the runner could not be reached: try again.
                Err(_) => keep.push(m),
            }
        }
        self.outbox = keep;
        told
    }
}

/// What an agent needs to reach the daemon that started it: the folder of the
/// daemon's own programs first on `PATH`, so `temet-work` is the matching command
/// line, and the daemon's socket when it was given one.
fn daemon_env() -> Vec<(String, String)> {
    let mut env = Vec::new();
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
    {
        let path = std::env::var("PATH").unwrap_or_default();
        env.push(("PATH".to_string(), format!("{}:{path}", dir.display())));
    }
    if let Some(socket) = std::env::var_os("TEMET_WORK_SOCKET").filter(|s| !s.is_empty()) {
        env.push((
            "TEMET_WORK_SOCKET".to_string(),
            socket.to_string_lossy().into_owned(),
        ));
    }
    env
}

fn state_of(status: &Status, doing: Option<&str>) -> Option<AgentState> {
    match status {
        Status::Between => Some(AgentState::Idle),
        Status::Working => Some(AgentState::Working {
            doing: doing.unwrap_or_default().to_string(),
        }),
        // What it asks for says more than the kind of prompt.
        Status::Blocked { kind } => Some(AgentState::Blocked {
            reason: doing.map_or_else(|| kind.clone(), String::from),
        }),
        Status::Gone => Some(AgentState::Ended),
        Status::Unknown => None,
    }
}

impl Part for Runners {
    fn id(&self) -> &'static str {
        "runner"
    }

    fn poll(&mut self, world: &World, now_ms: u64) -> Polled {
        let mut events = Vec::new();
        // Each agent's state as this poll has seen it, so a quick turn (idle,
        // working, idle) ends on idle and not on working.
        let mut seen: BTreeMap<AgentId, AgentState> = BTreeMap::new();
        for (i, adapter) in self.adapters.iter_mut().enumerate() {
            let listed = adapter.listed();
            let mut alive = std::collections::BTreeSet::new();
            for u in adapter.updates() {
                let Some(agent) = u.labels.get("agent").cloned() else {
                    continue;
                };
                // Another log's terminal, or an agent this log never started.
                let other_log = world
                    .log
                    .as_ref()
                    .is_some_and(|log| u.labels.get("log") != Some(log));
                let id = AgentId::new(agent.clone());
                if other_log || !world.agents.contains_key(&id) {
                    continue;
                }
                alive.insert(agent.clone());
                self.terminals.insert(agent, (i, u.terminal.clone()));
                let Some(state) = state_of(&u.status, u.doing.as_deref()) else {
                    continue;
                };
                let before = seen
                    .get(&id)
                    .or_else(|| world.agents.get(&id).map(|a| &a.state));
                if before == Some(&state) {
                    continue;
                }
                seen.insert(id.clone(), state.clone());
                events.push(Event::AgentChanged { agent: id, state });
            }
            // A full list from the runner that starts agents: an agent of this log
            // whose terminal is not on it, and that was not started since, is gone.
            if listed.is_some() && i == 0 {
                for a in world.agents.values() {
                    let name = &a.id.0;
                    let elsewhere = self.terminals.get(name).is_some_and(|(r, _)| *r != 0);
                    if a.state != AgentState::Ended
                        && !alive.contains(name)
                        && !self.started_here.contains(name)
                        && !elsewhere
                        && seen.get(&a.id) != Some(&AgentState::Ended)
                    {
                        self.terminals.remove(name);
                        seen.insert(a.id.clone(), AgentState::Ended);
                        events.push(Event::AgentChanged {
                            agent: a.id.clone(),
                            state: AgentState::Ended,
                        });
                    }
                }
                self.started_here.clear();
            }
        }
        events.extend(self.deliver());
        events.extend(self.remind(world, &seen, now_ms));
        let problems: Vec<String> = self.adapters.iter().filter_map(|a| a.problem()).collect();
        Polled {
            events,
            problem: (!problems.is_empty()).then(|| problems.join("; ")),
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
                let mut labels = BTreeMap::from([
                    ("agent".to_string(), launch.agent.0.clone()),
                    ("group".to_string(), launch.group.0.clone()),
                    ("issue".to_string(), brief.issue.to_string()),
                    ("role".to_string(), brief.role.name().to_string()),
                ]);
                if let Some(log) = &world.log {
                    labels.insert("log".to_string(), log.clone());
                }
                let mut program = program.clone();
                // Commands the agent runs are recorded as its own, and reach this
                // daemon with this daemon's own command line.
                program
                    .env
                    .push((AGENT_VAR.to_string(), launch.agent.0.clone()));
                program.env.extend(daemon_env());
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
                        self.started_here.insert(launch.agent.0.clone());
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
                let told = self.queue(Message {
                    agent: author.id.0.clone(),
                    issue: issue.clone(),
                    text: brief::fixes(issue, points),
                    tells: true,
                });
                Outcome::Done {
                    events: told,
                    effects: Vec::new(),
                }
            }
            // A reviewer still running hears of the next round; with none, a new
            // one is started elsewhere.
            Effect::StartReview { issue, round } => {
                let Some(reviewer) = world.running(issue, Role::Reviewer) else {
                    return Outcome::NotMine;
                };
                let told = self.queue(Message {
                    agent: reviewer.id.0.clone(),
                    issue: issue.clone(),
                    text: brief::next_round(issue, *round),
                    tells: true,
                });
                Outcome::Done {
                    events: told,
                    effects: Vec::new(),
                }
            }
            Effect::StopAgent { agent } => {
                let Some((i, terminal)) = self.terminals.get(&agent.0).cloned() else {
                    return Outcome::Failed(format!("no terminal is known for {agent}"));
                };
                match self.adapters[i].stop(&terminal) {
                    Ok(()) | Err(RunnerError::Gone | RunnerError::Unknown(_)) => {
                        self.terminals.remove(&agent.0);
                        self.outbox.retain(|m| m.agent != agent.0);
                        Outcome::Done {
                            events: vec![Event::AgentChanged {
                                agent: agent.clone(),
                                state: AgentState::Ended,
                            }],
                            effects: Vec::new(),
                        }
                    }
                    Err(e) => Outcome::Failed(e.to_string()),
                }
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
        assert_eq!(
            part.carry_out(&fix, &w),
            Outcome::Done {
                events: vec![Event::Told {
                    issue: IssueId::new("ankh", "1")
                }],
                effects: vec![]
            },
            "delivered at once, so the issue's author has been told"
        );
        let screen = part.screen(&idle.0).expect("screen");
        assert!(screen.contains("- no test for a stopped clock"), "{screen}");
        assert!(screen.contains("temet-work ready ankh#1"));

        let mut w = world();
        let mut part = runners();
        let asking = run(&mut part, &mut w, &[fake::PROMPT_PROGRAM]);
        part.carry_out(&fix, &w);
        part.carry_out(&fix, &w);
        assert_eq!(part.waiting(), 1, "the same message waits once");
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
        assert_eq!(
            part.carry_out(&next, &w),
            Outcome::Done {
                events: vec![Event::Told {
                    issue: IssueId::new("ankh", "1")
                }],
                effects: vec![]
            }
        );
        let screen = part.screen(&reviewer.0).expect("screen");
        assert!(screen.contains("review round 2"), "{screen}");
    }

    #[test]
    fn a_stopped_agent_ends_and_another_logs_terminals_are_ignored() {
        let mut w = world();
        w.apply(&Event::LogStarted { id: "b".into() });
        let mut part = runners();
        let agent = run(&mut part, &mut w, &[fake::PROMPT_PROGRAM]);
        // The same agent id under another log: its terminal's updates are not this log's.
        let mut other = world();
        other.apply(&Event::LogStarted { id: "a".into() });
        other.apply(&Event::AgentStarted {
            agent: agent.clone(),
            group: GroupId::new("watch"),
            role: Role::Author,
            issue: Some(IssueId::new("ankh", "1")),
            machine: MachineId::new(LOCAL),
            folder: None,
        });
        assert_eq!(part.poll(&other, 0).events, vec![]);
        let stop = Effect::StopAgent {
            agent: agent.clone(),
        };
        assert_eq!(
            part.carry_out(&stop, &w),
            Outcome::Done {
                events: vec![Event::AgentChanged {
                    agent,
                    state: AgentState::Ended
                }],
                effects: vec![]
            }
        );
    }

    /// The fake runner, shared with the test so it can stop terminals behind the
    /// family's back and hand it a full list.
    #[derive(Clone, Default)]
    struct Shared {
        fake: std::sync::Arc<std::sync::Mutex<FakeRunner>>,
        list: std::sync::Arc<std::sync::Mutex<Option<Vec<TerminalId>>>>,
    }

    impl Runner for Shared {
        fn id(&self) -> &'static str {
            "shared"
        }
        fn start(&mut self, start: &Start) -> Result<TerminalId, RunnerError> {
            self.fake.lock().expect("fake").start(start)
        }
        fn send(
            &mut self,
            t: &TerminalId,
            text: &str,
            only_if_idle: bool,
        ) -> Result<(), RunnerError> {
            self.fake.lock().expect("fake").send(t, text, only_if_idle)
        }
        fn screen(&mut self, t: &TerminalId) -> Result<String, RunnerError> {
            self.fake.lock().expect("fake").screen(t)
        }
        fn stop(&mut self, t: &TerminalId) -> Result<(), RunnerError> {
            self.fake.lock().expect("fake").stop(t)
        }
        fn updates(&mut self) -> Vec<Update> {
            self.fake.lock().expect("fake").updates()
        }
        fn listed(&mut self) -> Option<Vec<TerminalId>> {
            self.list.lock().expect("list").take()
        }
    }

    fn shared() -> (Runners, Shared) {
        let handle = Shared::default();
        let mut part = Runners::default();
        part.add(Box::new(handle.clone()));
        (part, handle)
    }

    fn apply(w: &mut World, events: &[Event]) {
        for e in events {
            w.apply(e);
        }
    }

    #[test]
    fn after_a_restart_an_agent_missing_from_the_first_list_has_ended() {
        let mut w = world();
        // A daemon before this one started an author; its terminal is gone since.
        let gone = run(&mut runners(), &mut w, &["cat"]);
        // This daemon's runner lists what it holds on connecting: nothing. An agent
        // it starts after that is not on the list, and has not ended.
        let (mut part, handle) = shared();
        *handle.list.lock().expect("list") = Some(vec![]);
        let fresh = run(&mut part, &mut w, &["cat"]);
        assert_eq!(
            part.poll(&w, 0).events,
            vec![Event::AgentChanged {
                agent: gone,
                state: AgentState::Ended
            }],
            "{fresh} started after the list was taken"
        );
    }

    #[test]
    fn sending_to_a_terminal_that_is_gone_ends_its_agent() {
        let mut w = world();
        let (mut part, handle) = shared();
        let author = run(&mut part, &mut w, &["cat"]);
        let first = part.poll(&w, 0).events;
        apply(&mut w, &first);
        let terminal = TerminalId("t1".into());
        handle
            .fake
            .lock()
            .expect("fake")
            .stop(&terminal)
            .expect("stop");
        handle.fake.lock().expect("fake").updates();
        let fix = Effect::RequestFix {
            issue: IssueId::new("ankh", "1"),
            points: vec!["no test".into()],
        };
        let Outcome::Done { events, .. } = part.carry_out(&fix, &w) else {
            panic!("not done");
        };
        assert_eq!(
            events,
            vec![Event::AgentChanged {
                agent: author,
                state: AgentState::Ended
            }]
        );
    }

    #[test]
    fn a_quick_turn_ends_on_idle() {
        let mut w = world();
        let mut part = runners();
        let author = run(&mut part, &mut w, &["cat"]);
        let first = part.poll(&w, 0).events;
        apply(&mut w, &first);
        let fix = Effect::RequestFix {
            issue: IssueId::new("ankh", "1"),
            points: vec!["no test".into()],
        };
        part.carry_out(&fix, &w);
        // The fake takes the text in one quick turn: working, then idle.
        let states: Vec<AgentState> = part
            .poll(&w, 0)
            .events
            .into_iter()
            .filter_map(|e| match e {
                Event::AgentChanged { agent, state } if agent == author => Some(state),
                _ => None,
            })
            .collect();
        assert_eq!(states.last(), Some(&AgentState::Idle), "{states:?}");
    }

    #[test]
    fn an_idle_agent_its_issue_waits_on_is_reminded_twice_then_stopped() {
        let mut w = world();
        let mut part = runners();
        // ankh#1 is being worked on by its author, who sits idle.
        if let Some(i) = w.issues.get_mut(&IssueId::new("ankh", "1")) {
            i.stage = temet_work_core::model::Stage::Working;
        }
        let author = run(&mut part, &mut w, &["cat"]);
        let first = part.poll(&w, 0).events;
        apply(&mut w, &first);
        let at = |n: u64| n * NUDGE_AFTER_MS;
        let polled = part.poll(&w, at(1)).events;
        apply(&mut w, &polled);
        let screen = part.screen(&author.0).expect("screen");
        assert!(
            screen.contains("ankh#1 is still waiting on you"),
            "{screen}"
        );
        let polled = part.poll(&w, at(2)).events;
        apply(&mut w, &polled);
        let ended = part.poll(&w, at(3)).events;
        assert!(ended.contains(&Event::AgentChanged {
            agent: author,
            state: AgentState::Ended
        }));
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
