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
use temet_work_core::launch::{AGENT_VAR, SECRET_VAR};
use temet_work_core::model::{AgentState, Role};
use temet_work_core::sender::Digest;
use temet_work_core::settings::{Choice, Scope};
use temet_work_core::world::LOCAL;
use temet_work_core::{Effect, Event, Outcome, Part, Polled, World};

/// The core type this contract uses, re-exported so adapters need no other crate.
pub use temet_work_core::launch::Program;

pub mod contract;
pub mod fake;
pub mod reset;

/// What the daemon types into an agent whose limit has reset.
pub const CONTINUE: &str = "continue";

/// How long after the reset time an error line gives the daemon sends
/// [`CONTINUE`]. The line gives minutes only and clocks drift, and a continue
/// sent too early comes straight back with the same error.
pub const CONTINUE_AFTER_MS: u64 = 60_000;

/// How long the runner family waits after a failed stop before it tries again:
/// a stop the loop asked for ([`Effect::StopAgent`]), the stop after an agent's
/// last reminder, or the stop of an ended agent's terminal. It tries once this
/// long (a minute) has passed since the last try, so a terminal service that
/// keeps refusing is asked once a minute, not at every poll. A stop of an agent
/// that has not ended is listed under Problems while it keeps failing.
pub const STOP_AGAIN_MS: u64 = 60_000;

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
    /// The turn ended in an error, such as a usage limit. This is not a prompt:
    /// typing starts a new turn.
    Error {
        /// What the runner says about the error, when it says anything.
        message: Option<String>,
    },
    /// The program ended by itself, or the terminal was stopped or removed.
    Gone,
    /// The program was lost, as to a signal or a reboot, and a shell has taken
    /// or is taking its place in the terminal.
    Lost,
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
    /// The runner could not be reached, so nothing was sent and the call
    /// changed nothing; the text says why. A call that may have reached the
    /// runner before it failed is [`RunnerError::Other`]. Callers treat it as
    /// they treat `Other`, except the stop after an agent's last reminder
    /// ([`Runners`]), which needs to know whether its kill may have landed.
    Unreached(String),
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
            Self::Unreached(e) | Self::Other(e) => f.write_str(e),
        }
    }
}

impl std::error::Error for RunnerError {}

/// One runner, added by an adapter crate.
pub trait Runner: Send {
    /// A short, stable id, the adapter's feature name after `runner-`.
    fn id(&self) -> &'static str;

    /// Whether this is the runner used when no setting names one. The build's
    /// first default one wins.
    fn is_default(&self) -> bool {
        false
    }

    /// Starts a program in a new labelled terminal.
    fn start(&mut self, start: &Start) -> Result<TerminalId, RunnerError>;

    /// Runs `program` to its end on `machine` (this one when `None`), with no
    /// terminal, its arguments as given and no shell. A program that cannot be
    /// started, ends with a nonzero exit or runs out of time is an error that
    /// says why in one line: what happened, with the first line it wrote to
    /// standard error, or what to do about it.
    fn exec(&mut self, machine: Option<&str>, program: &Program) -> Result<(), RunnerError> {
        let _ = (machine, program);
        Err(RunnerError::NotBuilt(
            "running a program to its end without a terminal".into(),
        ))
    }

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

    /// The terminals it has listed in full since the last call, each with its
    /// labels, if it has: on connecting, a runner lists every terminal it holds,
    /// and one that is not on the list is gone. The list stands on its own, so it
    /// does not matter which call to `updates` its terminals' states arrive in.
    fn listed(&mut self) -> Option<Vec<Update>> {
        None
    }

    /// The latest status of `terminal` among the updates the runner holds that
    /// `updates` has not handed back yet, read without taking them: the next
    /// call to `updates` still returns them. As [`latest_of`] picks it: the
    /// first that ends the terminal, gone or lost, else the newest. `None` when
    /// it holds none for the terminal, and for a runner that does not keep
    /// updates between calls.
    fn latest(&self, terminal: &TerminalId) -> Option<Status> {
        let _ = terminal;
        None
    }
}

/// The status [`Runner::latest`] answers for `terminal` from `updates`, oldest
/// first: the first that ends the terminal, gone or lost, as the next poll ends
/// its agent with that one; else the newest. `None` when none is `terminal`'s.
pub fn latest_of(updates: &[Update], terminal: &TerminalId) -> Option<Status> {
    let mut newest = None;
    for u in updates.iter().filter(|u| &u.terminal == terminal) {
        if matches!(u.status, Status::Gone | Status::Lost) {
            return Some(u.status.clone());
        }
        newest = Some(&u.status);
    }
    newest.cloned()
}

/// Text waiting for an agent to finish its turn.
struct Message {
    agent: String,
    /// The issue it is about; none for an investigator's.
    issue: Option<temet_work_core::ids::IssueId>,
    text: String,
    /// Whether it is what the issue's stage needs the agent to hear, so its
    /// arrival counts as told. A reminder does not.
    tells: bool,
}

/// The count of reminders to an agent its issue waits on. A reminder's turn
/// that ends in an error after it was seen working for at least `remind_after`
/// (ten minutes by default) has recovered: a new count starts at that error. A
/// shorter one counts toward the stop, timed from the last reminder.
#[derive(Clone)]
struct Count {
    /// When its clock last started: when the count began (at the error, after
    /// a retry that recovered), or the last reminder.
    since: u64,
    /// The reminders sent.
    nudges: u32,
    /// What the latest poll saw the agent doing.
    saw: Saw,
    /// When the stop after the last reminder last failed. It is tried again
    /// [`STOP_AGAIN_MS`] later.
    last_try_ms: Option<u64>,
    /// Why that try failed.
    refusal: Option<RunnerError>,
    /// Whether a refused try of that stop may have reached the runner, which
    /// may then have carried it out: any refusal but
    /// [`RunnerError::Unreached`]. A loss of the agent reported after it is
    /// the daemon's own doing, so it ends the agent as an ordinary end. Once
    /// set it stays set, whatever later tries answer.
    may_have_stopped: bool,
}

/// A stop the loop asked for ([`Effect::StopAgent`]) whose last try failed.
struct Refused {
    /// When that try was made.
    at_ms: u64,
    /// Why it failed.
    why: String,
    /// Whether any refused try of it may have reached the runner, which may
    /// then have carried it out: any refusal but [`RunnerError::Unreached`].
    /// As with [`Count::may_have_stopped`], a loss reported after it is the
    /// daemon's own doing and ends the agent as an ordinary end.
    reached: bool,
}

/// How a stop that ended an agent's session went.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ended {
    /// The runner stopped the terminal.
    Stopped,
    /// The runner answered that the terminal was gone or unknown: there was
    /// nothing for the stop to end.
    NotThere,
}

/// What a poll saw an agent with a [`Count`] doing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Saw {
    /// Between turns.
    Idle,
    /// Stopped on an error, with no limit reset to wait for.
    Errored,
    /// In a turn that followed a reminder and an error, first seen working by
    /// the poll at `from`. The count is kept until the turn shows how it ends.
    /// It has recovered if it ends between turns, or in an error `remind_after`
    /// or more after `from`; an earlier error counts toward the stop, timed
    /// from the last reminder.
    Retrying { from: u64 },
}

/// The runner part: the runners compiled into this build, in the order they were added.
///
/// It starts each agent's program in a labelled terminal with the runner its `runner`
/// setting names (the build's default when none does), turns runner updates into agent events, and delivers text to an
/// agent between its turns, holding it while the agent is working. A terminal
/// reported gone or lost ends its agent, and is stopped so nothing stays under
/// the agent's labels, such as the shell herdr-core puts in place of a lost
/// program. Only the end of a lost one says its program was lost, so that it
/// uses up no start, and only when the daemon did not cause the loss: a loss
/// reported after the daemon asked for the agent's stop, and the runner may
/// have carried it out, is an ordinary end ([`Runners::remind`]). A stop the
/// loop asks for that fails is tried again a minute after the last try, until
/// it works or the runner says the terminal is gone or unknown, and the agent
/// is then reported ended.
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
    /// The reminder count of each idle or errored agent its issue waits on.
    counts: BTreeMap<String, Count>,
    /// Agents whose terminal this daemon has stopped, so the update saying
    /// it is gone is not taken for a terminal left behind.
    stopped: std::collections::BTreeSet<String>,
    /// The terminals of ended agents whose stop failed: the runner that holds
    /// each and when it was last tried. Each is tried again
    /// [`STOP_AGAIN_MS`] later, until it is stopped or its runner says it is
    /// gone or unknown.
    unstopped: BTreeMap<String, (usize, TerminalId, u64)>,
    /// The agents whose stop the loop asked for ([`Effect::StopAgent`]) and
    /// whose last try failed. Each is tried again
    /// once [`STOP_AGAIN_MS`] has passed since that try, by a poll or when the
    /// loop asks again, until the stop works or its runner says the terminal is
    /// gone or unknown; an ask before then answers with that try's reason.
    /// One its issue waits on again is dropped; one a person stopped never is,
    /// since nothing waits on it.
    refused: BTreeMap<String, Refused>,
    /// When the latest poll ran. A stop the loop asks for is timed at it: in a
    /// round the effects are carried out right after the polls.
    polled_ms: u64,
    /// The daemon's address as another machine reaches it over SSH
    /// (`daemon_host` in the configuration).
    daemon_host: Option<String>,
}

impl Runners {
    /// Sets the daemon's address as other machines reach it, `daemon_host` in the
    /// configuration. Without it no agent starts on another machine.
    #[must_use]
    pub fn daemon_host(mut self, host: Option<String>) -> Self {
        self.daemon_host = host.filter(|h| !h.is_empty());
        self
    }

    /// Adds a runner. The root package calls this once per compiled-in adapter.
    pub fn add(&mut self, adapter: Box<dyn Runner>) {
        self.adapters.push(adapter);
    }

    /// The ids of the runners in this build.
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }

    /// The id of the runner used when no setting names one.
    pub fn default_id(&self) -> Option<&'static str> {
        self.adapters
            .iter()
            .find(|a| a.is_default())
            .map(|a| a.id())
    }

    /// The index of the runner an agent of `role` runs in at `scope` (its issue's,
    /// or its group's for an agent with none): the one its `runner` setting
    /// names, else this build's default.
    fn pick(&self, world: &World, scope: &Scope, role: Role) -> Result<usize, String> {
        let list = || {
            let ids = self.ids();
            if ids.is_empty() {
                "none".to_string()
            } else {
                ids.join(", ")
            }
        };
        let named = world.text_at(scope, Some(role), "runner");
        let id = named
            .as_deref()
            .or_else(|| self.default_id())
            .ok_or_else(|| {
                format!(
                    "no runner is set for {scope}'s {}s and this build has no default; set runner \
                 globally. This build has: {}",
                    role.name(),
                    list()
                )
            })?;
        self.adapters
            .iter()
            .position(|a| a.id() == id)
            .ok_or_else(|| {
                format!(
                    "no runner \"{id}\" in this build; this build has: {}",
                    list()
                )
            })
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

    /// Reminds an idle agent its issue waits on, `reminders` times (two by default),
    /// `remind_after` minutes (ten) apart; that long after the last reminder it stops
    /// the agent's session, so the loop starts a fresh one. An agent whose turn
    /// ended in an error is idle here, unless it waits for its limit to reset.
    /// A reminder to such an agent starts a turn that may end on the error again,
    /// so its count is kept while it works. The count starts again if that turn
    /// ends between turns. It also starts again, at the error, if the turn ends
    /// in one after it was seen working for at least `remind_after`. A shorter
    /// turn that ends in an error counts toward the stop, timed from the last
    /// reminder. Returns the agents it stopped, as ended.
    ///
    /// An agent ends here only once its stop works, or its runner says the
    /// terminal is gone or unknown. After any other error it may still be
    /// running: it is not ended, so no fresh agent starts beside it, it keeps
    /// its terminal and its queued messages, and the stop is tried again, with
    /// no further reminder, once [`STOP_AGAIN_MS`] (a minute) has passed since
    /// the last try. It is listed under Problems while it keeps failing, until
    /// the stop works or the count is dropped. Between tries an agent in a turn,
    /// at a prompt or waiting on a limit, or one its issue no longer waits on, is
    /// not stopped, and the rules above say whether its count is kept. An agent
    /// with no terminal here, as before its runner has reported any after a
    /// restart, is not ended either.
    ///
    /// Whether it ends as lost, which uses up no start, turns on whether the
    /// daemon may have caused the loss. A refused try may have reached the
    /// runner, which may have ended the program before the refusal (herdr-core
    /// carries out the kill, then its `rm` times out or finds the terminal
    /// still live): a loss reported after that, by the poll or here, is the
    /// daemon's doing, and the agent ends as an ordinary end that uses a
    /// start, even when a later try is refused unreached. Otherwise it ends as
    /// lost when its runner's latest status for the terminal
    /// ([`Runner::latest`]) says its program was lost: just before the stop,
    /// and again just after one answered with the terminal gone or unknown,
    /// as when herdr-core restarted and the stop's own call was the first to
    /// see it. A stop that worked is not read again: a loss after the daemon's
    /// own kill is the daemon's.
    ///
    /// Decision: a refusal that never reached the runner
    /// ([`RunnerError::Unreached`], such as herdr-core's socket refusing the
    /// connection) spares the start. Nothing was sent, so no kill of the
    /// daemon's can have ended the program, and its likely cause is
    /// herdr-core going down or a reboot, which is the restart case a lost
    /// end is for, and which must not use up starts.
    fn remind(
        &mut self,
        world: &World,
        seen: &BTreeMap<AgentId, AgentState>,
        now_ms: u64,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let mut counts = std::mem::take(&mut self.counts);
        for a in world.agents.values() {
            if !world.waiting_on(a) {
                continue;
            }
            let issue = a.issue.as_ref();
            let agent = &a.id.0;
            let before = counts.remove(agent);
            let state = seen.get(&a.id).unwrap_or(&a.state);
            let saw = match state {
                AgentState::Idle => Saw::Idle,
                AgentState::Errored { .. } if state.limit_reset(a.continued).is_none() => {
                    Saw::Errored
                }
                AgentState::Working { .. } => {
                    if let Some(count) = before.filter(|c| c.nudges > 0 && c.saw != Saw::Idle) {
                        let from = match count.saw {
                            Saw::Retrying { from } => from,
                            _ => now_ms,
                        };
                        let saw = Saw::Retrying { from };
                        self.counts.insert(agent.clone(), Count { saw, ..count });
                    }
                    continue;
                }
                _ => continue,
            };
            // An investigator works on no issue: its group's settings answer.
            let scope = world.agent_scope(&a.group, issue);
            let after_ms = world.number_at(&scope, Some(a.role), "remind_after") * 60_000;
            // A retry that ends between turns, or that worked for `remind_after`
            // before its error, has left the old error behind.
            let mut count = before
                .filter(|c| match c.saw {
                    Saw::Retrying { from } => saw != Saw::Idle && now_ms < from + after_ms,
                    Saw::Idle | Saw::Errored => true,
                })
                .unwrap_or(Count {
                    since: now_ms,
                    nudges: 0,
                    saw,
                    last_try_ms: None,
                    refusal: None,
                    may_have_stopped: false,
                });
            count.saw = saw;
            let reminders = world.number_at(&scope, Some(a.role), "reminders");
            if now_ms < count.since + after_ms {
                self.counts.insert(agent.clone(), count);
                continue;
            }
            if u64::from(count.nudges) < reminders {
                count.since = now_ms;
                count.nudges += 1;
                self.counts.insert(agent.clone(), count);
                events.extend(self.queue(
                    world,
                    seen,
                    Message {
                        agent: agent.clone(),
                        issue: issue.cloned(),
                        text: issue.map_or_else(
                            || brief::investigation_nudge(&a.group),
                            |issue| brief::nudge(a.role, issue),
                        ),
                        tells: false,
                    },
                ));
                continue;
            }
            let too_soon = count
                .last_try_ms
                .is_some_and(|at| now_ms < at.saturating_add(STOP_AGAIN_MS));
            let Some((i, terminal)) = self.terminals.get(agent).cloned().filter(|_| !too_soon)
            else {
                self.counts.insert(agent.clone(), count);
                continue;
            };
            // A loss its runner received after this poll read the updates, which
            // no stop of the daemon's may have caused: the agent ends as lost,
            // so it does not use up a start, and its terminal, where a shell may
            // now be, is still stopped.
            let spared = !count.may_have_stopped;
            let lost_since_read =
                |part: &Self| part.adapters[i].latest(&terminal) == Some(Status::Lost);
            let lost_before = spared && lost_since_read(self);
            let lost = match self.end_session(agent, i, &terminal) {
                Ok(Ended::Stopped) => lost_before,
                // Nothing was there to stop: a loss the stop's own call brought
                // in, as when it reached a restarted herdr-core, is not its doing.
                Ok(Ended::NotThere) => lost_before || (spared && lost_since_read(self)),
                Err(e) => {
                    count.may_have_stopped |= !matches!(e, RunnerError::Unreached(_));
                    count.last_try_ms = Some(now_ms);
                    count.refusal = Some(e);
                    self.counts.insert(agent.clone(), count);
                    continue;
                }
            };
            events.push(Event::AgentChanged {
                agent: a.id.clone(),
                state: AgentState::Ended,
                lost,
            });
        }
        events
    }

    /// A line for each agent whose stop after its last reminder keeps failing,
    /// in agent order. A count that is dropped takes its line with it, and an
    /// agent whose terminal is gone has none.
    fn stop_problems(&self) -> impl Iterator<Item = String> {
        self.counts.iter().filter_map(|(agent, count)| {
            let why = count.refusal.as_ref()?;
            let (_, terminal) = self.terminals.get(agent)?;
            Some(format!("cannot stop {agent}'s terminal {terminal}: {why}"))
        })
    }

    /// Stops the terminal of an agent that has not ended. Once it is stopped, or
    /// its runner says it is gone or unknown, the terminal and the agent's queued
    /// messages are dropped, and the caller reports the agent ended. Any other
    /// error changes nothing: the agent may still be running.
    fn end_session(
        &mut self,
        agent: &str,
        i: usize,
        terminal: &TerminalId,
    ) -> Result<Ended, RunnerError> {
        let ended = match self.adapters[i].stop(terminal) {
            Ok(()) => Ended::Stopped,
            Err(RunnerError::Gone | RunnerError::Unknown(_)) => Ended::NotThere,
            Err(e) => return Err(e),
        };
        self.terminals.remove(agent);
        self.outbox.retain(|m| m.agent != agent);
        self.stopped.insert(agent.to_string());
        Ok(ended)
    }

    /// Stops the terminal of an ended agent: its program is ended and the
    /// terminal removed. A stop that fails is tried again [`STOP_AGAIN_MS`]
    /// after `now_ms`; one the runner answers with the terminal gone or unknown
    /// has nothing left to stop.
    fn stop(&mut self, agent: &str, i: usize, terminal: TerminalId, now_ms: u64) {
        match self.adapters[i].stop(&terminal) {
            Ok(()) | Err(RunnerError::Gone | RunnerError::Unknown(_)) => {
                self.unstopped.remove(agent);
                self.stopped.insert(agent.to_string());
            }
            Err(e) => {
                eprintln!(
                    "cannot stop {agent}'s terminal {terminal}: {e}; trying again in a minute"
                );
                self.unstopped
                    .insert(agent.to_string(), (i, terminal, now_ms));
            }
        }
    }

    /// Tries again each stop the loop asked for that failed [`STOP_AGAIN_MS`] or
    /// more before `now_ms`, and returns the agents it stopped, as ended. A try
    /// that fails is the next one's start. An agent that has ended otherwise, as
    /// when its terminal was reported gone, has no terminal here and is dropped.
    /// So is one its issue waits on again, as a reviewer whose stop failed and
    /// that has since heard the next round: the loop put it back to work, and
    /// the reminders take over.
    fn stop_refused_again(
        &mut self,
        world: &World,
        seen: &mut BTreeMap<AgentId, AgentState>,
        now_ms: u64,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        for (agent, refused) in std::mem::take(&mut self.refused) {
            let id = AgentId::new(agent.clone());
            let dropped = seen.get(&id) == Some(&AgentState::Ended)
                || world
                    .agents
                    .get(&id)
                    .is_none_or(|a| a.state == AgentState::Ended || world.waiting_on(a));
            let Some((i, terminal)) = self.terminals.get(&agent).cloned().filter(|_| !dropped)
            else {
                continue;
            };
            if now_ms < refused.at_ms.saturating_add(STOP_AGAIN_MS) {
                self.refused.insert(agent, refused);
                continue;
            }
            // As at the stop after the last reminder: a loss its runner holds
            // and this poll has not read ends the agent as lost, unless a
            // refused try may have caused it.
            let spared = !refused.reached;
            let lost_since_read =
                |part: &Self| part.adapters[i].latest(&terminal) == Some(Status::Lost);
            let lost_before = spared && lost_since_read(self);
            let lost = match self.end_session(&agent, i, &terminal) {
                Ok(Ended::Stopped) => lost_before,
                Ok(Ended::NotThere) => lost_before || (spared && lost_since_read(self)),
                Err(e) => {
                    let reached = refused.reached || !matches!(e, RunnerError::Unreached(_));
                    self.refused.insert(
                        agent,
                        Refused {
                            at_ms: now_ms,
                            why: e.to_string(),
                            reached,
                        },
                    );
                    continue;
                }
            };
            seen.insert(id.clone(), AgentState::Ended);
            events.push(Event::AgentChanged {
                agent: id,
                state: AgentState::Ended,
                lost,
            });
        }
        events
    }

    /// Tries again each stop that failed [`STOP_AGAIN_MS`] or more before `now_ms`.
    fn stop_again(&mut self, now_ms: u64) {
        let due: Vec<(String, usize, TerminalId)> = self
            .unstopped
            .iter()
            .filter(|(_, (_, _, at))| now_ms >= at.saturating_add(STOP_AGAIN_MS))
            .map(|(agent, (i, terminal, _))| (agent.clone(), *i, terminal.clone()))
            .collect();
        for (agent, i, terminal) in due {
            self.stop(&agent, i, terminal, now_ms);
        }
    }

    /// Queues a message unless the same one is already waiting, then sends what
    /// can be sent. Returns a `Told` event for each issue whose message arrived.
    fn queue(
        &mut self,
        world: &World,
        seen: &BTreeMap<AgentId, AgentState>,
        message: Message,
    ) -> Vec<Event> {
        let waiting = self
            .outbox
            .iter()
            .any(|m| m.agent == message.agent && m.text == message.text);
        if !waiting {
            self.outbox.push(message);
        }
        self.deliver(world, seen)
    }

    /// Sends what is waiting and keeps what the agent cannot take yet, and what
    /// is about a paused issue ([`World::paused`]). What waits for an agent a
    /// person stopped is dropped: it is told nothing more, even while its stop
    /// keeps failing. Returns a `Told` event for each issue whose message
    /// arrived. `seen` holds the states this poll has seen that `world` does not
    /// have yet.
    fn deliver(&mut self, world: &World, seen: &BTreeMap<AgentId, AgentState>) -> Vec<Event> {
        let mut keep = Vec::new();
        let mut told = Vec::new();
        self.outbox.retain(|m| {
            world
                .person_stopped(&AgentId::new(m.agent.clone()))
                .is_none()
        });
        for m in std::mem::take(&mut self.outbox) {
            let Some((i, terminal)) = self.terminals.get(&m.agent).cloned() else {
                keep.push(m);
                continue;
            };
            // A message about a paused issue, as while the daemon is held, waits
            // until the issue is no longer paused: the loop sends no fix or
            // round to one, and one queued before the pause waits the same.
            if m.issue
                .as_ref()
                .and_then(|i| world.issues.get(i))
                .is_some_and(|i| world.paused(i))
            {
                keep.push(m);
                continue;
            }
            // A send to a session that waits on its limit is lost to the error;
            // it goes after continue.
            if world
                .agents
                .get(&AgentId::new(m.agent.clone()))
                .is_some_and(|a| limit_reset(a, seen).is_some())
            {
                keep.push(m);
                continue;
            }
            match self.adapters[i].send(&terminal, &m.text, true) {
                Ok(()) if m.tells => told.extend(m.issue.map(|issue| Event::Told { issue })),
                Ok(()) => {}
                // The terminal is gone, so its agent has ended; the loop starts and
                // tells the next one.
                Err(RunnerError::Gone | RunnerError::Unknown(_)) => {
                    self.terminals.remove(&m.agent);
                    told.push(Event::AgentChanged {
                        agent: AgentId::new(m.agent),
                        state: AgentState::Ended,
                        lost: false,
                    });
                }
                // Busy, at a prompt, or the runner could not be reached: try again.
                Err(_) => keep.push(m),
            }
        }
        self.outbox = keep;
        told
    }

    /// Types [`CONTINUE`] into each agent whose limit has reset, once
    /// [`CONTINUE_AFTER_MS`] has passed since the time its error line gave, once
    /// per limit; never into an agent a person stopped. Returns a `Continued`
    /// event for each, and ends an agent whose terminal is gone. An agent whose
    /// terminal is busy or cannot be reached is tried again at the next poll.
    fn continue_after_reset(
        &mut self,
        world: &World,
        seen: &mut BTreeMap<AgentId, AgentState>,
        now_ms: u64,
    ) -> Vec<Event> {
        let due: Vec<(AgentId, u64)> = world
            .agents
            .values()
            .filter(|a| world.person_stopped(&a.id).is_none())
            .filter_map(|a| {
                let at = limit_reset(a, seen)?;
                (now_ms >= at.saturating_add(CONTINUE_AFTER_MS)).then(|| (a.id.clone(), at))
            })
            .collect();
        let mut events = Vec::new();
        for (agent, at) in due {
            let Some((i, terminal)) = self.terminals.get(&agent.0).cloned() else {
                continue;
            };
            match self.adapters[i].send(&terminal, CONTINUE, true) {
                Ok(()) => {
                    seen.insert(
                        agent.clone(),
                        AgentState::Working {
                            doing: String::new(),
                        },
                    );
                    events.push(Event::Continued {
                        agent,
                        resets_ms: at,
                    });
                }
                Err(RunnerError::Gone | RunnerError::Unknown(_)) => {
                    self.terminals.remove(&agent.0);
                    seen.insert(agent.clone(), AgentState::Ended);
                    events.push(Event::AgentChanged {
                        agent,
                        state: AgentState::Ended,
                        lost: false,
                    });
                }
                Err(_) => {}
            }
        }
        events
    }
}

/// When the limit `agent` waits on resets, by the state this poll has seen for
/// it, else the state on record.
fn limit_reset(
    agent: &temet_work_core::model::Agent,
    seen: &BTreeMap<AgentId, AgentState>,
) -> Option<u64> {
    seen.get(&agent.id)
        .unwrap_or(&agent.state)
        .limit_reset(agent.continued)
}

/// The state of an agent whose terminal reports that its turn ended in an
/// error, seen at `now_ms`: the runner's message when it gives one, else the
/// line on the visible screen that names the error, with the reset time that
/// line gives in the daemon's time zone.
fn errored(
    runner: &mut dyn Runner,
    terminal: &TerminalId,
    message: Option<&str>,
    now_ms: u64,
) -> AgentState {
    let line = message
        .map(str::trim)
        .filter(|m| !m.is_empty())
        .map(String::from)
        .or_else(|| {
            runner
                .screen(terminal)
                .ok()
                .and_then(|screen| reset::error_line(&screen))
        })
        .unwrap_or_default();
    let resets_ms = reset::reset_time(&line, now_ms, &jiff::tz::TimeZone::system());
    AgentState::Errored { line, resets_ms }
}

/// What an agent needs to reach the daemon that started it. A local agent gets
/// the folder of the daemon's own programs first on `PATH`, so `temet-work` is the
/// matching command line, and the daemon's socket when it was given one. An agent
/// on another machine gets `TEMET_WORK_DAEMON=ssh://HOST` instead: a path on the
/// daemon's machine means nothing there, and the other machine needs `temet-work`
/// on its own `PATH`.
fn daemon_env(
    remote_daemon_host: Option<&str>,
    socket: Option<&str>,
) -> Result<Vec<(String, String)>, String> {
    if let Some(host) = remote_daemon_host {
        // The daemon's own socket rides in the address, so a second daemon on the
        // same machine is reached too; none means the default there.
        let socket = socket.unwrap_or_default();
        if !socket.is_empty() && !socket.starts_with('/') {
            return Err(format!(
                "this daemon's TEMET_WORK_SOCKET is the relative path {socket}: an agent on another \
                 machine cannot find it, so give the daemon an absolute path"
            ));
        }
        return Ok(vec![(
            DAEMON_VAR.to_string(),
            format!("ssh://{host}{socket}"),
        )]);
    }
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
    Ok(env)
}

/// The variable that tells `temet-work` how to reach the daemon.
const DAEMON_VAR: &str = "TEMET_WORK_DAEMON";

/// A new agent's secret: 256 random bits as 64 hex digits, read from the
/// system's random source. It goes into the agent's environment only; the log
/// keeps its [`digest`].
fn secret() -> Result<String, String> {
    use std::io::Read;
    let mut bytes = [0u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|e| format!("/dev/urandom: {e}"))?;
    Ok(Digest::from_bytes(&bytes).0)
}

/// The SHA-256 digest of a secret, the form the log and the core keep.
pub fn digest(secret: &str) -> Digest {
    use sha2::Digest as _;
    Digest::from_bytes(&sha2::Sha256::digest(secret.as_bytes()))
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
        Status::Gone | Status::Lost => Some(AgentState::Ended),
        // An error's state needs its screen and the time: `errored` makes it.
        Status::Error { .. } | Status::Unknown => None,
    }
}

impl Part for Runners {
    fn id(&self) -> &'static str {
        "runner"
    }

    fn choices(&self) -> Vec<Choice> {
        vec![Choice {
            key: "runner".into(),
            allowed: self.ids().into_iter().map(String::from).collect(),
            default: self.default_id().map(String::from),
        }]
    }

    fn screen(&mut self, agent: &AgentId) -> Option<Result<String, String>> {
        Some(Runners::screen(self, &agent.0).map_err(|e| e.to_string()))
    }

    fn poll(&mut self, world: &World, now_ms: u64) -> Polled {
        self.polled_ms = now_ms;
        let mut events = Vec::new();
        // Each agent's state as this poll has seen it, so a quick turn (idle,
        // working, idle) ends on idle and not on working.
        let mut seen: BTreeMap<AgentId, AgentState> = BTreeMap::new();
        // Every runner's updates first, so each agent's terminal is known to the
        // runner that holds it before any runner's full list is read.
        let mut lists: Vec<(usize, Vec<Update>)> = Vec::new();
        // The terminals to stop, by agent, once every runner's updates are in.
        let mut stops: BTreeMap<String, (usize, TerminalId)> = BTreeMap::new();
        for (i, adapter) in self.adapters.iter_mut().enumerate() {
            let updates = adapter.updates();
            let listed = adapter.listed();
            if let Some(listed) = listed {
                lists.push((i, listed));
            }
            for u in updates {
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
                let ended = world
                    .agents
                    .get(&id)
                    .is_some_and(|a| a.state == AgentState::Ended)
                    || seen.get(&id) == Some(&AgentState::Ended);
                if ended {
                    // A gone or lost terminal of an agent that has ended, which
                    // this daemon has not stopped, as when a restart lost the
                    // stop of a shell herdr-core put in place of its program.
                    if matches!(u.status, Status::Gone | Status::Lost)
                        && !self.stopped.contains(&agent)
                        && !self.unstopped.contains_key(&agent)
                    {
                        stops.entry(agent).or_insert((i, u.terminal.clone()));
                    }
                    continue;
                }
                if matches!(u.status, Status::Gone | Status::Lost) {
                    // Nothing more is sent to it: text typed now would go to
                    // whatever holds the terminal, such as a shell.
                    self.terminals.remove(&agent);
                    self.outbox.retain(|m| m.agent != agent);
                    seen.insert(id.clone(), AgentState::Ended);
                    // A loss after a refused stop that may have reached the
                    // runner is that stop's doing: an ordinary end.
                    let killed = self.counts.get(&agent).is_some_and(|c| c.may_have_stopped)
                        || self.refused.get(&agent).is_some_and(|r| r.reached);
                    events.push(Event::AgentChanged {
                        agent: id,
                        state: AgentState::Ended,
                        lost: u.status == Status::Lost && !killed,
                    });
                    stops.insert(agent, (i, u.terminal.clone()));
                    continue;
                }
                self.terminals.insert(agent, (i, u.terminal.clone()));
                let before = seen
                    .get(&id)
                    .or_else(|| world.agents.get(&id).map(|a| &a.state));
                let state = match &u.status {
                    // The line is read once, when the error appears: the same
                    // error listed again, as after a restart, keeps the line and
                    // reset time on record.
                    Status::Error { .. } if matches!(before, Some(AgentState::Errored { .. })) => {
                        continue;
                    }
                    Status::Error { message } => {
                        errored(adapter.as_mut(), &u.terminal, message.as_deref(), now_ms)
                    }
                    status => match state_of(status, u.doing.as_deref()) {
                        Some(state) => state,
                        None => continue,
                    },
                };
                if before == Some(&state) {
                    continue;
                }
                seen.insert(id.clone(), state.clone());
                events.push(Event::AgentChanged {
                    agent: id,
                    state,
                    lost: false,
                });
            }
        }
        // A full list from a runner: an agent of this log whose terminal that runner
        // holds, and which is not on its list and was not started since, is gone. An
        // agent no runner holds yet is judged by the first runner's list, the one
        // that starts agents when no setting names another.
        let listed_any = !lists.is_empty();
        for (i, listed) in lists {
            let alive: std::collections::BTreeSet<String> = listed
                .iter()
                .filter(|u| {
                    world
                        .log
                        .as_ref()
                        .is_none_or(|log| u.labels.get("log") == Some(log))
                })
                .filter_map(|u| u.labels.get("agent").cloned())
                .collect();
            for a in world.agents.values() {
                let name = &a.id.0;
                let judged_here = match self.terminals.get(name) {
                    Some((r, _)) => *r == i,
                    None => i == 0,
                };
                if judged_here
                    && a.state != AgentState::Ended
                    && !alive.contains(name)
                    && !self.started_here.contains(name)
                    && seen.get(&a.id) != Some(&AgentState::Ended)
                {
                    self.terminals.remove(name);
                    seen.insert(a.id.clone(), AgentState::Ended);
                    events.push(Event::AgentChanged {
                        agent: a.id.clone(),
                        state: AgentState::Ended,
                        lost: false,
                    });
                }
            }
        }
        if listed_any {
            self.started_here.clear();
        }
        for (agent, (i, terminal)) in stops {
            self.stop(&agent, i, terminal, now_ms);
        }
        self.stop_again(now_ms);
        events.extend(self.stop_refused_again(world, &mut seen, now_ms));
        events.extend(self.continue_after_reset(world, &mut seen, now_ms));
        events.extend(self.deliver(world, &seen));
        events.extend(self.remind(world, &seen, now_ms));
        let problems: Vec<String> = self
            .adapters
            .iter()
            .filter_map(|a| a.problem())
            .chain(self.stop_problems())
            .collect();
        Polled {
            events,
            problem: (!problems.is_empty()).then(|| problems.join("; ")),
        }
    }

    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        match effect {
            Effect::Run {
                launch,
                runtime,
                program,
                before,
            } => {
                let brief = &launch.brief;
                let scope = world.agent_scope(&launch.group, brief.issue.as_ref());
                let chosen = match self.pick(world, &scope, brief.role) {
                    Ok(i) => i,
                    Err(e) => return Outcome::Failed(e),
                };
                let runner = &mut self.adapters[chosen];
                let mut labels = BTreeMap::from([
                    ("agent".to_string(), launch.agent.0.clone()),
                    ("group".to_string(), launch.group.0.clone()),
                    ("role".to_string(), brief.role.name().to_string()),
                ]);
                if let Some(issue) = &brief.issue {
                    labels.insert("issue".to_string(), issue.to_string());
                }
                if let Some(log) = &world.log {
                    labels.insert("log".to_string(), log.clone());
                }
                let mut program = program.clone();
                // Commands the agent runs are recorded as its own, carry a secret
                // only it was given, and reach this daemon with this daemon's own
                // command line.
                let secret = match secret() {
                    Ok(s) => s,
                    Err(e) => {
                        return Outcome::Failed(format!(
                            "cannot make a secret for {}: {e}",
                            launch.agent
                        ));
                    }
                };
                let secret_digest = digest(&secret);
                program
                    .env
                    .push((AGENT_VAR.to_string(), launch.agent.0.clone()));
                program.env.push((SECRET_VAR.to_string(), secret));
                let machine = (launch.machine.0 != LOCAL).then(|| launch.machine.0.clone());
                let remote = match (&machine, &self.daemon_host) {
                    (None, _) => None,
                    (Some(_), Some(host)) => Some(host.as_str()),
                    (Some(m), None) => {
                        return Outcome::Failed(format!(
                            "{} would run on the machine {m} (`machine` of its [[folder]]), but \
                             `daemon_host` is not set in config.toml: set it to this machine's \
                             name as {m} reaches it over SSH, so the agent can report back",
                            launch.agent
                        ));
                    }
                };
                let socket = std::env::var("TEMET_WORK_SOCKET").ok();
                match daemon_env(remote, socket.as_deref()) {
                    Ok(env) => program.env.extend(env),
                    Err(why) => return Outcome::Failed(why),
                }
                let start = Start {
                    program,
                    cwd: brief
                        .folder
                        .clone()
                        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
                        .unwrap_or_else(|| PathBuf::from("/")),
                    labels,
                    machine,
                };
                let place = start.machine.as_deref().unwrap_or("this machine");
                // The folder is trusted on its own machine first; an agent started
                // in a folder that is not would stop at the runtime's prompt.
                if let Some(before) = before {
                    if let Err(e) = runner.exec(start.machine.as_deref(), before) {
                        return Outcome::Failed(format!(
                            "cannot trust {} on {place}: {e}",
                            start.cwd.display()
                        ));
                    }
                    eprintln!(
                        "trusted: {} on {place}, for {}",
                        start.cwd.display(),
                        launch.agent
                    );
                }
                match runner.start(&start) {
                    Ok(terminal) => {
                        eprintln!(
                            "started: {} on {place} in {} (terminal {terminal})",
                            launch.agent,
                            start.cwd.display()
                        );
                        self.terminals
                            .insert(launch.agent.0.clone(), (chosen, terminal));
                        self.started_here.insert(launch.agent.0.clone());
                        Outcome::Done {
                            events: vec![Event::AgentStarted {
                                agent: launch.agent.clone(),
                                group: launch.group.clone(),
                                role: brief.role,
                                issue: brief.issue.clone(),
                                machine: launch.machine.clone(),
                                folder: launch.folder.clone(),
                                secret_digest: Some(secret_digest),
                                runtime: Some(runtime.clone()),
                                model: brief.model.clone(),
                            }],
                            effects: Vec::new(),
                        }
                    }
                    Err(e) => Outcome::Failed(e.to_string()),
                }
            }
            Effect::RequestFix { issue, points } => {
                // With no author running there is nothing to send: the loop starts
                // one, and it hears the fixes in its brief.
                let (Some(author), Some(i)) =
                    (world.running(issue, Role::Author), world.issues.get(issue))
                else {
                    return Outcome::done();
                };
                let told = self.queue(
                    world,
                    &BTreeMap::new(),
                    Message {
                        agent: author.id.0.clone(),
                        issue: Some(issue.clone()),
                        text: brief::fixes(i, &world.repos_of(i), points),
                        tells: true,
                    },
                );
                Outcome::Done {
                    events: told,
                    effects: Vec::new(),
                }
            }
            // A reviewer still running hears of the next round, with the pull
            // requests to review now; with none, a new one is started elsewhere.
            Effect::StartReview { issue, round } => {
                let (Some(reviewer), Some(i)) = (
                    world.running(issue, Role::Reviewer),
                    world.issues.get(issue),
                ) else {
                    return Outcome::NotMine;
                };
                // A next-round message still waiting may name a pull request
                // since replaced, which the reviewer must not act on: this one
                // takes its place. A reminder stays.
                self.outbox
                    .retain(|m| !(m.agent == reviewer.id.0 && m.tells));
                let told = self.queue(
                    world,
                    &BTreeMap::new(),
                    Message {
                        agent: reviewer.id.0.clone(),
                        issue: Some(issue.clone()),
                        text: brief::next_round(i, *round, &world.pulled(i)),
                        tells: true,
                    },
                );
                Outcome::Done {
                    events: told,
                    effects: Vec::new(),
                }
            }
            // The loop asks again for a stop that failed, as `World::missing`
            // does at every round: within a minute of the last try the answer
            // is that try's reason, and a poll makes the next try.
            Effect::StopAgent { agent } => {
                if let Some(refused) = self.refused.get(&agent.0)
                    && self.polled_ms < refused.at_ms.saturating_add(STOP_AGAIN_MS)
                {
                    return Outcome::Failed(refused.why.clone());
                }
                let Some((i, terminal)) = self.terminals.get(&agent.0).cloned() else {
                    return Outcome::Failed(format!("no terminal is known for {agent}"));
                };
                match self.end_session(&agent.0, i, &terminal) {
                    Ok(_) => {
                        self.refused.remove(&agent.0);
                        Outcome::Done {
                            events: vec![Event::AgentChanged {
                                agent: agent.clone(),
                                state: AgentState::Ended,
                                lost: false,
                            }],
                            effects: Vec::new(),
                        }
                    }
                    Err(e) => {
                        let why = e.to_string();
                        let reached = !matches!(e, RunnerError::Unreached(_))
                            || self.refused.get(&agent.0).is_some_and(|r| r.reached);
                        self.refused.insert(
                            agent.0.clone(),
                            Refused {
                                at_ms: self.polled_ms,
                                why: why.clone(),
                                reached,
                            },
                        );
                        Outcome::Failed(why)
                    }
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

    use temet_work_core::ids::{FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId};
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
        match run_events(part, w, argv).as_slice() {
            [Event::AgentStarted { agent, .. }] => agent.clone(),
            other => panic!("{other:?}"),
        }
    }

    /// Runs `argv` as an author on ankh#1, applies what the part reports and
    /// returns it.
    fn run_events(part: &mut Runners, w: &mut World, argv: &[&str]) -> Vec<Event> {
        let launch = w
            .launch(
                &GroupId::new("watch"),
                Role::Author,
                &IssueId::new("ankh", "1"),
                Some(&FolderId::new("f1")),
            )
            .expect("launch");
        let run = Effect::Run {
            launch: Box::new(launch),
            before: None,
            program: Program {
                argv: argv.iter().map(|a| a.to_string()).collect(),
                env: vec![],
            },
            runtime: "fake".into(),
        };
        let Outcome::Done { events, .. } = part.carry_out(&run, w) else {
            panic!("not started");
        };
        for e in &events {
            w.apply(e);
        }
        events
    }

    fn runners() -> Runners {
        let mut part = Runners::default();
        part.add(Box::new(FakeRunner::default()));
        part
    }

    #[test]
    fn an_agent_starts_recorded_with_the_runtime_that_made_its_program_and_its_briefs_model() {
        let mut w = world();
        let mut part = runners();
        let mut launch = w
            .launch(
                &GroupId::new("watch"),
                Role::Author,
                &IssueId::new("ankh", "1"),
                Some(&FolderId::new("f1")),
            )
            .expect("launch");
        launch.brief.model = Some("octavo".into());
        let run = Effect::Run {
            launch: Box::new(launch),
            runtime: "hex".into(),
            program: Program {
                argv: vec!["cat".into()],
                env: vec![],
            },
            before: None,
        };
        let Outcome::Done { events, .. } = part.carry_out(&run, &w) else {
            panic!("not started");
        };
        let [Event::AgentStarted { runtime, model, .. }] = events.as_slice() else {
            panic!("{events:?}");
        };
        assert_eq!(
            (runtime.as_deref(), model.as_deref()),
            (Some("hex"), Some("octavo"))
        );
        for e in &events {
            w.apply(e);
        }
        let agent = w.agents.values().next().expect("started");
        assert_eq!(
            (agent.runtime.as_deref(), agent.model.as_deref()),
            (Some("hex"), Some("octavo"))
        );
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
                },
                lost: false,
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
        // Round 1 is repeated after a failed check, a conflict or a refused merge.
        for round in [1, 2] {
            let next = Effect::StartReview {
                issue: IssueId::new("ankh", "1"),
                round,
            };
            let mut w = world();
            w.apply(&Event::ProjectAdded {
                project: ProjectId::new("ankh"),
                tracker: "fake".into(),
                name: "ankh-morpork/clock".into(),
                code_host: None,
                repo: None,
            });
            w.apply(&pull_ready("7"));
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
                launch: Box::new(launch),
                before: None,
                program: Program {
                    argv: vec!["cat".into()],
                    env: vec![],
                },
                runtime: "fake".into(),
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
            let i = &w.issues[&IssueId::new("ankh", "1")];
            let text = brief::next_round(i, round, &w.pulled(i));
            assert!(screen.contains(&text), "{screen}");
            assert!(
                screen.contains("- ankh (ankh-morpork/clock): pull request 7\n"),
                "{screen}"
            );
        }
    }

    /// The author's ready on ankh#1, naming pull request `number`.
    fn pull_ready(number: &str) -> Event {
        Event::PullRequestReady {
            issue: IssueId::new("ankh", "1"),
            pull: Some(number.into()),
            pulls: Default::default(),
            replies: Vec::new(),
        }
    }

    #[test]
    fn a_next_round_message_still_waiting_gives_way_to_a_later_one() {
        let next = Effect::StartReview {
            issue: IssueId::new("ankh", "1"),
            round: 1,
        };
        let mut w = world();
        let mut part = runners();
        let launch = w
            .launch(
                &GroupId::new("watch"),
                Role::Reviewer,
                &IssueId::new("ankh", "1"),
                None,
            )
            .expect("launch");
        // At a prompt, so nothing reaches it yet.
        let run = Effect::Run {
            launch: Box::new(launch),
            before: None,
            program: Program {
                argv: vec![fake::PROMPT_PROGRAM.into()],
                env: vec![],
            },
            runtime: "fake".into(),
        };
        let Outcome::Done { events, .. } = part.carry_out(&run, &w) else {
            panic!("not started");
        };
        for e in &events {
            w.apply(e);
        }
        for number in ["7", "8"] {
            w.apply(&pull_ready(number));
            part.carry_out(&next, &w);
        }
        let waiting: Vec<&str> = part.outbox.iter().map(|m| m.text.as_str()).collect();
        let i = &w.issues[&IssueId::new("ankh", "1")];
        assert_eq!(waiting, [brief::next_round(i, 1, &w.pulled(i))]);
        assert!(waiting[0].contains("pull request 8\n"), "{}", waiting[0]);
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
            secret_digest: None,
            runtime: None,
            model: None,
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
                    state: AgentState::Ended,
                    lost: false,
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
        list: std::sync::Arc<std::sync::Mutex<Option<Vec<Update>>>>,
        problem: std::sync::Arc<std::sync::Mutex<Option<String>>>,
        /// A terminal whose program is lost to a shell once the next call to
        /// `updates` has handed back its updates, so the loss lands after the
        /// poll's read.
        lose_after_read: std::sync::Arc<std::sync::Mutex<Option<TerminalId>>>,
        /// A terminal whose program is lost to a shell at the next stop, just
        /// before the fake answers it, as when the stop's call is the first to
        /// reach a herdr-core that restarted.
        lose_on_stop: std::sync::Arc<std::sync::Mutex<Option<TerminalId>>>,
    }

    impl Runner for Shared {
        fn id(&self) -> &'static str {
            "shared"
        }
        fn is_default(&self) -> bool {
            true
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
            let mut fake = self.fake.lock().expect("fake");
            if let Some(lost) = self.lose_on_stop.lock().expect("lose").take() {
                fake.lose_to_shell(&lost);
            }
            fake.stop(t)
        }
        fn updates(&mut self) -> Vec<Update> {
            let mut fake = self.fake.lock().expect("fake");
            let updates = fake.updates();
            if let Some(t) = self.lose_after_read.lock().expect("lose").take() {
                fake.lose_to_shell(&t);
            }
            updates
        }
        fn listed(&mut self) -> Option<Vec<Update>> {
            self.list.lock().expect("list").take()
        }
        fn latest(&self, t: &TerminalId) -> Option<Status> {
            self.fake.lock().expect("fake").latest(t)
        }
        fn problem(&self) -> Option<String> {
            self.problem.lock().expect("problem").clone()
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
                state: AgentState::Ended,
                lost: false,
            }],
            "{fresh} started after the list was taken"
        );
    }

    #[test]
    fn after_a_restart_an_agent_on_the_list_stays_whatever_poll_its_updates_come_in() {
        let mut w = world();
        let live = run(&mut runners(), &mut w, &["cat"]);
        let (mut part, handle) = shared();
        // The list names the agent; no update for it comes in this poll.
        let mut labels = BTreeMap::new();
        labels.insert("agent".to_string(), live.0.clone());
        *handle.list.lock().expect("list") = Some(vec![Update {
            terminal: TerminalId("t9".into()),
            labels,
            status: Status::Between,
            doing: None,
        }]);
        assert_eq!(part.poll(&w, 0).events, vec![], "{live} is on the list");
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
                state: AgentState::Ended,
                lost: false,
            }]
        );
    }

    #[test]
    fn continuing_a_terminal_that_is_gone_ends_its_agent() {
        let t1 = TerminalId("t1".into());
        for answer in [RunnerError::Gone, RunnerError::Unknown(t1.clone())] {
            let mut w = world();
            at_stage(&mut w, temet_work_core::model::Stage::Working);
            let (mut part, handle) = shared();
            let author = run(&mut part, &mut w, &["claude"]);
            handle.fake.lock().expect("fake").end_in_error(&t1, LIMIT);
            step(&mut part, &mut w, nov27(16, 50));
            assert!(limit_reset(&w.agents[&author], &BTreeMap::new()).is_some());
            handle.fake.lock().expect("fake").fail_sends(answer.clone());
            assert_eq!(
                step(&mut part, &mut w, nov27(17, 41)),
                vec![Event::AgentChanged {
                    agent: author.clone(),
                    state: AgentState::Ended,
                    lost: false,
                }],
                "{answer:?}"
            );
            assert!(w.agents[&author].ended_waited_on, "{answer:?} counts");
        }
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
                Event::AgentChanged { agent, state, .. } if agent == author => Some(state),
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
        let at = |n: u64| n * 600_000;
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
            state: AgentState::Ended,
            lost: false,
        }));
    }

    #[test]
    fn an_idle_investigator_is_reminded_then_stopped() {
        let mut w = world();
        let watch = GroupId::new("watch");
        for e in [
            Event::ProjectAdded {
                project: ProjectId::new("ankh"),
                tracker: "fake".into(),
                name: "city/ankh".into(),
                code_host: None,
                repo: None,
            },
            Event::GroupCreated {
                group: watch.clone(),
                name: "Night watch".into(),
                projects: vec![ProjectId::new("ankh")],
                areas: vec![],
                milestones: vec![],
                refining_at_once: 1,
                model: None,
                queue_low: None,
                investigate_every: None,
            },
            // One reminder, a minute apart, read at the group's first project.
            Event::SettingSet {
                scope: temet_work_core::settings::Scope::Project(ProjectId::new("ankh")),
                role: Some(Role::Investigator),
                key: "reminders".into(),
                value: temet_work_core::settings::Value::Number(1),
                by: "person".into(),
            },
            Event::SettingSet {
                scope: temet_work_core::settings::Scope::Project(ProjectId::new("ankh")),
                role: None,
                key: "remind_after".into(),
                value: temet_work_core::settings::Value::Number(1),
                by: "person".into(),
            },
            Event::InvestigationStarted {
                group: watch.clone(),
                agent: AgentId::new("investigator-1"),
                at_ms: 0,
            },
        ] {
            w.apply(&e);
        }
        let mut part = runners();
        let launch = w.launch_investigator(&watch).expect("launch");
        let run = Effect::Run {
            launch: Box::new(launch),
            before: None,
            program: Program {
                argv: vec!["cat".into()],
                env: vec![],
            },
            runtime: "fake".into(),
        };
        let Outcome::Done { events, .. } = part.carry_out(&run, &w) else {
            panic!("not started");
        };
        let [
            Event::AgentStarted {
                agent, issue: None, ..
            },
        ] = events.as_slice()
        else {
            panic!("{events:?}");
        };
        let agent = agent.clone();
        apply(&mut w, &events);
        let first = part.poll(&w, 0).events;
        apply(&mut w, &first);
        let minute = 60_000;
        let polled = part.poll(&w, minute).events;
        apply(&mut w, &polled);
        let screen = part.screen(&agent.0).expect("screen");
        assert!(
            screen.contains(
                "The investigation for watch is still running. If you have filed every issue you \
                 found, run: temet-work investigated watch."
            ),
            "{screen}"
        );
        assert!(!polled.iter().any(|e| matches!(e, Event::Told { .. })));
        let ended = part.poll(&w, 2 * minute).events;
        assert!(
            ended.ends_with(&[Event::AgentChanged {
                agent: agent.clone(),
                state: AgentState::Ended,
                lost: false,
            }]),
            "stopped a minute after its one reminder: {ended:?}"
        );
        apply(&mut w, &ended);
        assert_eq!(
            w.investigation_events(2 * minute, false),
            [Event::InvestigationEnded {
                group: watch,
                at_ms: 2 * minute,
                filed: 0,
            }],
            "so it cannot hold its group for ever"
        );
    }

    #[test]
    fn an_idle_reviewer_that_approved_is_neither_reminded_nor_stopped() {
        let mut w = world();
        let mut part = runners();
        let id = IssueId::new("ankh", "1");
        // ankh#1 is in review; the reviewer started here has heard the round.
        w.issues.get_mut(&id).expect("there").stage =
            temet_work_core::model::Stage::InReview { round: 1 };
        let launch = w
            .launch(&GroupId::new("watch"), Role::Reviewer, &id, None)
            .expect("launch");
        let reviewer = launch.agent.clone();
        let run = Effect::Run {
            launch: Box::new(launch),
            before: None,
            program: Program {
                argv: vec!["cat".into()],
                env: vec![],
            },
            runtime: "fake".into(),
        };
        let Outcome::Done { events, .. } = part.carry_out(&run, &w) else {
            panic!("not started");
        };
        apply(&mut w, &events);
        // It approved, and the checks are still running.
        w.issues.get_mut(&id).expect("there").review_ok = true;
        for n in 0..=4 {
            let polled = part.poll(&w, n * 600_000).events;
            let ended = Event::AgentChanged {
                agent: reviewer.clone(),
                state: AgentState::Ended,
                lost: false,
            };
            assert!(!polled.contains(&ended), "stopped at {n}0 minutes");
            apply(&mut w, &polled);
            let screen = part.screen(&reviewer.0).expect("screen");
            assert!(!screen.contains("is still waiting on you"), "{screen}");
        }
    }

    #[test]
    fn a_fix_with_no_author_running_is_left_to_the_next_authors_brief() {
        let fix = Effect::RequestFix {
            issue: IssueId::new("ankh", "1"),
            points: vec![],
        };
        let mut part = runners();
        assert_eq!(part.carry_out(&fix, &world()), Outcome::done());
        assert_eq!(part.waiting(), 0);
    }

    fn two() -> Runners {
        let mut part = Runners::default();
        part.add(Box::new(FakeRunner::named("first")));
        part.add(Box::new(FakeRunner::named("second")));
        part
    }

    /// Which runner holds the agent, by its id.
    fn held_by(part: &Runners, agent: &AgentId) -> &'static str {
        part.adapters[part.terminals[&agent.0].0].id()
    }

    #[test]
    fn two_adapters_the_default_runs_until_a_setting_names_the_other() {
        let mut w = world();
        let mut part = two();
        let first = run(&mut part, &mut w, &["cat"]);
        assert_eq!(held_by(&part, &first), "first");
        w.apply(&Event::SettingSet {
            scope: temet_work_core::settings::Scope::Task(IssueId::new("ankh", "1")),
            role: Some(Role::Author),
            key: "runner".into(),
            value: temet_work_core::settings::Value::Text("second".into()),
            by: "person".into(),
        });
        let second = run(&mut part, &mut w, &["cat"]);
        assert_eq!(held_by(&part, &second), "second");
        // Each agent's terminal is read from the runner that holds it.
        assert!(part.screen(&second.0).is_ok());
        assert!(part.screen(&first.0).is_ok());
    }

    #[test]
    fn two_adapters_each_judge_only_the_agents_they_hold() {
        let mut w = world();
        let mut part = two();
        let first = run(&mut part, &mut w, &["cat"]);
        w.apply(&Event::SettingSet {
            scope: temet_work_core::settings::Scope::Global,
            role: None,
            key: "runner".into(),
            value: temet_work_core::settings::Value::Text("second".into()),
            by: "file".into(),
        });
        let second = run(&mut part, &mut w, &["cat"]);
        // Neither runner lists anything, and nothing is judged gone.
        assert_eq!(part.poll(&w, 0).events, vec![]);
        assert_eq!(w.agents[&first].state, AgentState::Idle);
        assert_eq!(w.agents[&second].state, AgentState::Idle);
    }

    #[test]
    fn two_adapters_a_setting_naming_one_the_build_lacks_is_a_problem_listing_the_ones_it_has() {
        let mut w = world();
        let mut part = two();
        w.apply(&Event::SettingSet {
            scope: temet_work_core::settings::Scope::Global,
            role: None,
            key: "runner".into(),
            value: temet_work_core::settings::Value::Text("nonesuch".into()),
            by: "file".into(),
        });
        let launch = w
            .launch(
                &GroupId::new("watch"),
                Role::Author,
                &IssueId::new("ankh", "1"),
                Some(&FolderId::new("f1")),
            )
            .expect("launch");
        let run = Effect::Run {
            launch: Box::new(launch),
            before: None,
            program: Program {
                argv: vec!["cat".into()],
                env: vec![],
            },
            runtime: "fake".into(),
        };
        assert!(matches!(
            part.carry_out(&run, &w),
            Outcome::Failed(e) if e.contains("nonesuch") && e.contains("first, second")
        ));
    }

    #[test]
    fn a_reminder_follows_the_settings_of_its_agents_issue() {
        // remind_after and reminders are read per issue: one minute, no reminders,
        // so a single idle minute stops the agent.
        let mut w = world();
        let mut part = runners();
        let agent = run(&mut part, &mut w, &["cat"]);
        w.apply(&Event::Told {
            issue: IssueId::new("ankh", "1"),
        });
        for (key, n) in [("remind_after", 1), ("reminders", 0)] {
            w.apply(&Event::SettingSet {
                scope: temet_work_core::settings::Scope::Task(IssueId::new("ankh", "1")),
                role: None,
                key: key.into(),
                value: temet_work_core::settings::Value::Number(n),
                by: "person".into(),
            });
        }
        w.issues
            .get_mut(&IssueId::new("ankh", "1"))
            .expect("there")
            .stage = temet_work_core::model::Stage::Working;
        assert_eq!(part.poll(&w, 0).events, vec![]);
        assert_eq!(
            part.poll(&w, 60_000).events,
            vec![Event::AgentChanged {
                agent,
                state: AgentState::Ended,
                lost: false,
            }]
        );
    }

    #[test]
    fn keeps_adapters_in_the_order_added() {
        let mut part = Runners::default();
        part.add(Box::new(FakeRunner::named("first")));
        part.add(Box::new(FakeRunner::named("second")));
        assert_eq!(part.ids(), ["first", "second"]);
        assert_eq!(part.id(), "runner");
    }

    /// A runner that ignores the machine of a start.
    struct IgnoresMachine(FakeRunner);

    impl Runner for IgnoresMachine {
        fn id(&self) -> &'static str {
            "ignores-machine"
        }
        fn start(&mut self, start: &Start) -> Result<TerminalId, RunnerError> {
            self.0.start(&Start {
                machine: None,
                ..start.clone()
            })
        }
        fn send(
            &mut self,
            terminal: &TerminalId,
            text: &str,
            only_if_idle: bool,
        ) -> Result<(), RunnerError> {
            self.0.send(terminal, text, only_if_idle)
        }
        fn screen(&mut self, terminal: &TerminalId) -> Result<String, RunnerError> {
            self.0.screen(terminal)
        }
        fn stop(&mut self, terminal: &TerminalId) -> Result<(), RunnerError> {
            self.0.stop(terminal)
        }
        fn exec(&mut self, _machine: Option<&str>, program: &Program) -> Result<(), RunnerError> {
            self.0.exec(None, program)
        }
        fn updates(&mut self) -> Vec<Update> {
            self.0.updates()
        }
    }

    #[test]
    fn the_contract_rejects_a_runner_that_ignores_the_machine() {
        let mut runner = IgnoresMachine(FakeRunner::default());
        let mut harness = fake::FakeHarness;
        let err = contract::check(&mut runner, &mut harness).expect_err("must fail");
        assert!(err.contains("machine"), "{err}");
    }

    /// Starts the `run` helper's author on a work folder on `machine`.
    fn start_on(machine: &str, daemon_host: Option<&str>) -> (Vec<Start>, Option<String>) {
        let mut w = world();
        w.apply(&Event::FolderAdded {
            folder: FolderId::new("f2"),
            project: ProjectId::new("ankh"),
            machine: MachineId::new(machine),
            path: "/work".into(),
        });
        let fake = FakeRunner::default();
        let starts = fake.starts();
        let mut part = Runners::default().daemon_host(daemon_host.map(String::from));
        part.add(Box::new(fake));
        let launch = w
            .launch(
                &GroupId::new("watch"),
                Role::Author,
                &IssueId::new("ankh", "1"),
                Some(&FolderId::new("f2")),
            )
            .expect("launch");
        let run = Effect::Run {
            launch: Box::new(launch),
            before: None,
            program: Program {
                argv: vec!["cat".into()],
                env: vec![],
            },
            runtime: "fake".into(),
        };
        let problem = match part.carry_out(&run, &w) {
            Outcome::Failed(why) => Some(why),
            _ => None,
        };
        let got = starts.lock().expect("starts").clone();
        (got, problem)
    }

    /// The program that trusts the `run` helper's folder on its machine.
    fn trust_far() -> Program {
        Program {
            argv: ["temet-work", "trust-folder", "fake", "/work"]
                .map(String::from)
                .to_vec(),
            env: vec![],
        }
    }

    /// Asks `part` to start an author on ankh#1 in a work folder `/work` on
    /// `machine`, with `before` run first.
    fn run_on(part: &mut Runners, machine: &str, before: Option<Program>) -> Outcome {
        let mut w = world();
        w.apply(&Event::FolderAdded {
            folder: FolderId::new("f2"),
            project: ProjectId::new("ankh"),
            machine: MachineId::new(machine),
            path: "/work".into(),
        });
        let launch = w
            .launch(
                &GroupId::new("watch"),
                Role::Author,
                &IssueId::new("ankh", "1"),
                Some(&FolderId::new("f2")),
            )
            .expect("launch");
        let run = Effect::Run {
            launch: Box::new(launch),
            before,
            program: Program {
                argv: vec!["cat".into()],
                env: vec![],
            },
            runtime: "fake".into(),
        };
        part.carry_out(&run, &w)
    }

    /// The calls a fake was given, in a few words each.
    fn calls_of(calls: &fake::Calls) -> Vec<String> {
        calls
            .lock()
            .expect("calls")
            .iter()
            .map(|c| match c {
                fake::Call::Exec { machine, program } => format!(
                    "exec on {}: {}",
                    machine.as_deref().unwrap_or("here"),
                    program.argv.join(" ")
                ),
                fake::Call::Start(s) => {
                    format!("start on {}", s.machine.as_deref().unwrap_or("here"))
                }
                fake::Call::Stop(t) => format!("stop {t}"),
            })
            .collect()
    }

    #[test]
    fn a_folder_on_another_machine_is_trusted_there_before_its_agent_starts_on_every_start() {
        let fake = FakeRunner::default();
        let calls = fake.calls();
        let mut part = Runners::default().daemon_host(Some("omit".into()));
        part.add(Box::new(fake));
        // Twice: a folder trusted by an earlier start is trusted again, and the
        // agent still starts.
        for round in 1..=2 {
            let outcome = run_on(&mut part, "far", Some(trust_far()));
            assert!(
                matches!(&outcome, Outcome::Done { events, .. }
                    if matches!(events.as_slice(), [Event::AgentStarted { .. }])),
                "start {round}: {outcome:?}"
            );
        }
        let trust = "exec on far: temet-work trust-folder fake /work";
        assert_eq!(
            calls_of(&calls),
            [trust, "start on far", trust, "start on far"]
        );
    }

    #[test]
    fn a_start_with_nothing_to_run_first_runs_nothing_first() {
        let fake = FakeRunner::default();
        let calls = fake.calls();
        let mut part = Runners::default().daemon_host(Some("omit".into()));
        part.add(Box::new(fake));
        run_on(&mut part, "far", None);
        run_on(&mut part, LOCAL, None);
        assert_eq!(calls_of(&calls), ["start on far", "start on here"]);
    }

    #[test]
    fn a_trust_that_fails_on_the_other_machine_fails_the_start_and_opens_no_terminal() {
        let fake = FakeRunner::default().exec_fails(RunnerError::Other(
            "temet-work exited 1: /Users/vimes/.claude.json is not a JSON object".into(),
        ));
        let calls = fake.calls();
        let mut part = Runners::default().daemon_host(Some("omit".into()));
        part.add(Box::new(fake));
        let Outcome::Failed(why) = run_on(&mut part, "far", Some(trust_far())) else {
            panic!("started");
        };
        assert_eq!(
            why,
            "cannot trust /work on far: temet-work exited 1: /Users/vimes/.claude.json is not \
             a JSON object"
        );
        assert_eq!(
            calls_of(&calls),
            ["exec on far: temet-work trust-folder fake /work"],
            "no terminal is started"
        );
    }

    fn env_of(start: &Start) -> BTreeMap<&str, &str> {
        start
            .program
            .env
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    #[test]
    fn an_agent_on_another_machine_reaches_the_daemon_over_ssh() {
        let (starts, problem) = start_on("mac", Some("omit"));
        assert_eq!(problem, None);
        let env = env_of(&starts[0]);
        assert_eq!(starts[0].machine.as_deref(), Some("mac"));
        assert_eq!(env.get("TEMET_WORK_DAEMON"), Some(&"ssh://omit"));
        assert!(!env.contains_key("TEMET_WORK_SOCKET"), "{env:?}");
        assert!(!env.contains_key("PATH"), "{env:?}");
        assert!(env.contains_key("TEMET_WORK_AGENT"));
    }

    #[test]
    fn the_daemons_own_socket_rides_in_the_address() {
        assert_eq!(
            daemon_env(Some("omit"), Some("/run/user/1000/temet-work/daemon.sock")),
            Ok(vec![(
                "TEMET_WORK_DAEMON".to_string(),
                "ssh://omit/run/user/1000/temet-work/daemon.sock".to_string()
            )])
        );
    }

    #[test]
    fn a_relative_daemon_socket_is_a_problem_for_a_remote_agent_not_a_silent_default() {
        let why = daemon_env(Some("omit"), Some("daemon.sock")).expect_err("refused");
        assert!(
            why.contains("TEMET_WORK_SOCKET") && why.contains("absolute"),
            "{why}"
        );
        assert!(
            daemon_env(None, Some("daemon.sock")).is_ok(),
            "a local agent is unaffected"
        );
    }

    #[test]
    fn a_local_agent_keeps_the_environment_it_had() {
        let (starts, problem) = start_on(LOCAL, Some("omit"));
        assert_eq!(problem, None);
        let env = env_of(&starts[0]);
        assert_eq!(starts[0].machine, None);
        assert!(!env.contains_key("TEMET_WORK_DAEMON"), "{env:?}");
        assert!(env["PATH"].contains(':'), "{env:?}");
        assert!(env.contains_key("TEMET_WORK_AGENT"));
    }

    #[test]
    fn every_agent_gets_a_secret_of_its_own_and_only_its_digest_is_recorded() {
        let w = world();
        let fake = FakeRunner::default();
        let starts = fake.starts();
        let mut part = Runners::default().daemon_host(Some("omit".into()));
        part.add(Box::new(fake));
        let first = w
            .launch(
                &GroupId::new("watch"),
                Role::Author,
                &IssueId::new("ankh", "1"),
                Some(&FolderId::new("f1")),
            )
            .expect("launch");
        let mut events = Vec::new();
        // Two agents on this machine and one on another.
        for (id, machine) in [
            ("author-1", LOCAL),
            ("reviewer-1", LOCAL),
            ("arbiter-1", "mac"),
        ] {
            let mut launch = first.clone();
            launch.agent = AgentId::new(id);
            launch.machine = MachineId::new(machine);
            let run = Effect::Run {
                launch: Box::new(launch),
                before: None,
                program: Program {
                    argv: vec!["cat".into()],
                    env: vec![],
                },
                runtime: "fake".into(),
            };
            match part.carry_out(&run, &w) {
                Outcome::Done { events: e, .. } => events.extend(e),
                other => panic!("{other:?}"),
            }
        }
        let starts = starts.lock().expect("starts").clone();
        let secrets: Vec<String> = starts
            .iter()
            .map(|s| env_of(s)[SECRET_VAR].to_string())
            .collect();
        assert_eq!(starts[2].machine.as_deref(), Some("mac"));
        assert!(
            secrets
                .iter()
                .all(|s| s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())),
            "256 bits each"
        );
        let mut distinct = secrets.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(distinct.len(), 3, "three different secrets");
        let digests: Vec<Option<Digest>> = events
            .iter()
            .map(|e| match e {
                Event::AgentStarted { secret_digest, .. } => secret_digest.clone(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            digests,
            secrets.iter().map(|s| Some(digest(s))).collect::<Vec<_>>()
        );
        // Neither the events nor any Debug form shows a secret.
        let shown = format!("{events:?} {starts:?}");
        for secret in &secrets {
            assert!(!shown.contains(secret.as_str()), "{shown}");
        }
        assert!(shown.contains("<hidden>"), "{shown}");
    }

    #[test]
    fn a_digest_is_sha256_in_lowercase_hex() {
        assert_eq!(
            digest("abc").0,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn no_agent_starts_on_another_machine_without_daemon_host() {
        let (starts, problem) = start_on("mac", None);
        assert!(starts.is_empty());
        let why = problem.expect("a problem");
        assert!(
            why.contains("daemon_host") && why.contains("machine"),
            "{why}"
        );
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
        fn exec(&mut self, machine: Option<&str>, program: &Program) -> Result<(), RunnerError> {
            self.0.exec(machine, program)
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

    /// The problem left once the author of ankh#1, which is being worked on,
    /// has ended: the issue needs a new one, and these tests build no part
    /// that starts one.
    const NEXT_AUTHOR: &str = "start a author on ankh#1";

    #[test]
    fn an_agent_at_a_prompt_is_a_problem_naming_agent_folder_and_prompt_until_it_goes() {
        use temet_work_core::Host;
        use temet_work_core::host::Memory;
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let fake = handle.fake;
        let agent = run(&mut part, &mut w, &["cat"]);
        let mut host = Host::new(w, vec![Box::new(part)], Box::new(Memory::default()));
        host.tick(1).expect("tick");
        assert!(host.problems().is_empty(), "{:?}", host.problems());

        fake.lock()
            .expect("lock")
            .block(&TerminalId("t1".into()), "trust this folder");
        host.tick(2).expect("tick");
        let said = host.problems().values().cloned().collect::<Vec<_>>();
        assert_eq!(
            said,
            ["author-1 in . is blocked: trust this folder; the daemon cannot answer it"]
        );
        let text = temet_work_core::text::text(host.world(), host.problems());
        assert!(text.contains(&said[0]), "{text}");
        assert_eq!(agent.0, "author-1");

        fake.lock().expect("lock").unblock(&TerminalId("t1".into()));
        host.tick(3).expect("tick");
        assert!(host.problems().is_empty(), "{:?}", host.problems());

        fake.lock()
            .expect("lock")
            .block(&TerminalId("t1".into()), "trust this folder");
        host.tick(4).expect("tick");
        assert_eq!(host.problems().len(), 1);
        fake.lock()
            .expect("lock")
            .stop(&TerminalId("t1".into()))
            .expect("stop");
        host.tick(5).expect("tick");
        assert_eq!(
            host.problems().keys().collect::<Vec<_>>(),
            [NEXT_AUTHOR],
            "ended: its line goes"
        );
    }

    /// The line a Claude session's turn ended on at its usage limit, as the
    /// issue saw it.
    const LIMIT: &str = "You've hit your 5-hour limit · resets 5:40pm";

    /// 27 November 2026 at `hh:mm` in UTC, the zone tests run in, in
    /// milliseconds since 1970.
    fn nov27(hh: i8, mm: i8) -> u64 {
        let ms = jiff::tz::TimeZone::UTC
            .to_zoned_datetime(jiff::civil::date(2026, 11, 27).at(hh, mm, 0, 0))
            .expect("a time")
            .timestamp()
            .as_millisecond();
        u64::try_from(ms).expect("after 1970")
    }

    /// Puts ankh#1 in `stage` for the group `watch`, so its author is waited on.
    fn at_stage(w: &mut World, stage: temet_work_core::model::Stage) {
        let i = w.issues.get_mut(&IssueId::new("ankh", "1")).expect("there");
        i.stage = stage;
        i.group = Some(GroupId::new("watch"));
    }

    /// Polls at `now_ms` and applies what the part reports, which it returns.
    fn step(part: &mut Runners, w: &mut World, now_ms: u64) -> Vec<Event> {
        let events = part.poll(w, now_ms).events;
        apply(w, &events);
        events
    }

    fn screen_of(fake: &std::sync::Arc<std::sync::Mutex<FakeRunner>>) -> String {
        fake.lock()
            .expect("fake")
            .screen(&TerminalId("t1".into()))
            .expect("screen")
    }

    /// The lines of the first terminal that are exactly `continue`.
    fn continues(fake: &std::sync::Arc<std::sync::Mutex<FakeRunner>>) -> usize {
        screen_of(fake).lines().filter(|l| *l == CONTINUE).count()
    }

    /// A journal a test can read after the host has it.
    #[derive(Clone, Default)]
    struct Kept(std::sync::Arc<std::sync::Mutex<Vec<Event>>>);

    impl temet_work_core::host::Journal for Kept {
        fn append(&mut self, _at_ms: u64, event: &Event) -> Result<(), String> {
            self.0.lock().expect("kept").push(event.clone());
            Ok(())
        }
    }

    #[test]
    fn an_agent_whose_turn_ends_in_an_error_is_a_problem_naming_agent_folder_and_line() {
        use temet_work_core::Host;
        use temet_work_core::host::Memory;
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let fake = handle.fake;
        let t1 = TerminalId("t1".into());
        run(&mut part, &mut w, &["claude"]);
        let mut host = Host::new(w, vec![Box::new(part)], Box::new(Memory::default()));
        host.tick(nov27(16, 49)).expect("tick");
        assert!(host.problems().is_empty(), "{:?}", host.problems());

        fake.lock().expect("lock").end_in_error(&t1, LIMIT);
        host.tick(nov27(16, 50)).expect("tick");
        let said = host.problems().values().cloned().collect::<Vec<_>>();
        assert_eq!(
            said,
            [
                "author-1 in . stopped on an error: You've hit your 5-hour limit · resets 5:40pm; \
                 the daemon sends continue after it resets"
            ]
        );
        let text = temet_work_core::text::text(host.world(), host.problems());
        assert!(
            text.contains(
                "author-1 on ankh#1: error: You've hit your 5-hour limit · resets 5:40pm (on local)"
            ),
            "{text}"
        );
        assert!(text.contains(&said[0]), "{text}");

        // A person typed into it and its turn ended: it has left the state.
        fake.lock().expect("lock").unblock(&t1);
        host.tick(nov27(16, 51)).expect("tick");
        assert!(host.problems().is_empty(), "{:?}", host.problems());

        // An error with no reset time is listed the same way, until the agent ends.
        fake.lock()
            .expect("lock")
            .end_in_error(&t1, "API Error: 529 overloaded");
        host.tick(nov27(16, 52)).expect("tick");
        assert_eq!(
            host.problems().values().cloned().collect::<Vec<_>>(),
            [
                "author-1 in . stopped on an error: API Error: 529 overloaded; the daemon reminds \
                 it and stops it as it does an idle agent"
            ]
        );
        fake.lock().expect("lock").stop(&t1).expect("stop");
        host.tick(nov27(16, 53)).expect("tick");
        assert_eq!(
            host.problems().keys().collect::<Vec<_>>(),
            [NEXT_AUTHOR],
            "ended: its line goes"
        );
    }

    #[test]
    fn a_usage_limit_error_is_continued_after_its_reset_without_a_new_start() {
        use temet_work_core::Host;
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let fake = handle.fake;
        let author = run(&mut part, &mut w, &["claude"]);
        let kept = Kept::default();
        let mut host = Host::new(w, vec![Box::new(part)], Box::new(kept.clone()));
        fake.lock()
            .expect("lock")
            .end_in_error(&TerminalId("t1".into()), LIMIT);
        host.tick(nov27(16, 50)).expect("tick");
        assert_eq!(
            host.world().agents[&author].state,
            AgentState::Errored {
                line: LIMIT.into(),
                resets_ms: Some(nov27(17, 40))
            }
        );
        let limited = screen_of(&fake);
        // Every minute up to 17:40 nothing is typed, not even a reminder,
        // although its issue waits on it.
        for minute in 51..=100 {
            host.tick(nov27(16, 0) + minute * 60_000).expect("tick");
            assert_eq!(screen_of(&fake), limited, "typed at 16:00 + {minute} min");
        }
        host.tick(nov27(17, 41)).expect("tick");
        assert_eq!(screen_of(&fake), format!("{limited}\ncontinue"));
        assert_eq!(
            host.world().agents[&author].state,
            AgentState::Working {
                doing: String::new()
            }
        );
        assert!(host.problems().is_empty(), "{:?}", host.problems());
        for minute in [42, 43, 45, 50] {
            host.tick(nov27(17, minute)).expect("tick");
        }
        assert_eq!(continues(&fake), 1, "once");
        // The same one author, never ended, and no start since: `starts` is untouched.
        let events = kept.0.lock().expect("kept").clone();
        assert!(
            !events.iter().any(|e| matches!(
                e,
                Event::AgentStarted { .. }
                    | Event::AgentChanged {
                        state: AgentState::Ended,
                        ..
                    }
            )),
            "{events:?}"
        );
        assert!(events.contains(&Event::Continued {
            agent: author.clone(),
            resets_ms: nov27(17, 40)
        }));
        let authors: Vec<&temet_work_core::model::Agent> = host
            .world()
            .agents
            .values()
            .filter(|a| a.role == Role::Author)
            .collect();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].id, author);
        assert_ne!(authors[0].state, AgentState::Ended);
    }

    #[test]
    fn a_limit_read_just_after_midnight_is_continued_at_once() {
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let fake = handle.fake;
        let author = run(&mut part, &mut w, &["claude"]);
        fake.lock().expect("lock").end_in_error(
            &TerminalId("t1".into()),
            "You've hit your 5-hour limit · resets 11:40pm",
        );
        let mut events = step(&mut part, &mut w, nov27(0, 20));
        events.extend(step(&mut part, &mut w, nov27(0, 21)));
        assert_eq!(continues(&fake), 1);
        assert!(
            events.contains(&Event::Continued {
                agent: author.clone(),
                resets_ms: nov27(23, 40) - 24 * 60 * 60 * 1000
            }),
            "{events:?}"
        );
        assert!(!ended(&events, &author), "{events:?}");
        assert!(w.missing().start.is_empty(), "{:?}", w.missing());
    }

    #[test]
    fn a_limit_more_than_6_hours_ahead_is_treated_as_idle() {
        use temet_work_core::Host;
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let fake = handle.fake.clone();
        run(&mut part, &mut w, &["claude"]);
        let mut host = Host::new(w, vec![Box::new(part)], Box::new(Kept::default()));
        fake.lock()
            .expect("lock")
            .end_in_error(&TerminalId("t1".into()), LIMIT);
        host.tick(nov27(11, 0)).expect("tick");
        assert_eq!(
            host.problems().values().cloned().collect::<Vec<_>>(),
            [
                "author-1 in . stopped on an error: You've hit your 5-hour limit · resets 5:40pm; \
                 the daemon reminds it and stops it as it does an idle agent"
            ]
        );
        host.tick(nov27(11, 9)).expect("tick");
        assert_eq!(reminders(&handle), 0);
        host.tick(nov27(11, 10)).expect("tick");
        assert_eq!(reminders(&handle), 1);
    }

    #[test]
    fn an_error_with_no_reset_time_is_reminded_twice_then_stopped() {
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let fake = handle.fake;
        let author = run(&mut part, &mut w, &["claude"]);
        fake.lock()
            .expect("lock")
            .end_in_error(&TerminalId("t1".into()), "API Error: 529 overloaded");
        assert_eq!(
            step(&mut part, &mut w, 0),
            vec![Event::AgentChanged {
                agent: author.clone(),
                state: AgentState::Errored {
                    line: "API Error: 529 overloaded".into(),
                    resets_ms: None
                },
                lost: false,
            }]
        );
        let minute = |n: u64| n * 60_000;
        let reminders = || screen_of(&fake).matches("still waiting on you").count();
        step(&mut part, &mut w, minute(9));
        assert_eq!(reminders(), 0, "not before remind_after");
        step(&mut part, &mut w, minute(10));
        assert_eq!(reminders(), 1);
        step(&mut part, &mut w, minute(20));
        assert_eq!(reminders(), 2);
        let ended = step(&mut part, &mut w, minute(30));
        assert!(
            ended.contains(&Event::AgentChanged {
                agent: author,
                state: AgentState::Ended,
                lost: false,
            }),
            "{ended:?}"
        );
        // The loop starts a fresh author, as after an idle one is stopped.
        assert!(
            w.missing().start.iter().any(|e| matches!(
                e,
                Effect::StartAgent {
                    role: Role::Author,
                    ..
                }
            )),
            "{:?}",
            w.missing()
        );
    }

    /// An idle author on ankh#1, waited on, reminded at 10 and 20 minutes, with
    /// a fix waiting for it that the runner now fails to send.
    fn reminded_twice_with_a_fix_waiting() -> (World, Runners, Shared, AgentId) {
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let author = run_and_remind_twice(&mut part, &mut w, 0);
        assert_eq!(reminders(&handle), 2);
        handle
            .fake
            .lock()
            .expect("lock")
            .fail_sends(RunnerError::Other("cannot reach the core".into()));
        request_fix(&mut part, &mut w);
        assert_eq!(part.waiting(), 1);
        (w, part, handle, author)
    }

    /// Starts an idle author on ankh#1 and polls at `from_ms` and 10 and 20
    /// minutes later, so it is reminded twice, its stop due at 30 minutes.
    fn run_and_remind_twice(part: &mut Runners, w: &mut World, from_ms: u64) -> AgentId {
        let author = run(part, w, &["claude"]);
        for minute in [0, 10, 20] {
            step(part, w, from_ms + minute * 60_000);
        }
        author
    }

    /// When the stop after the last reminder comes due for the agent of
    /// [`reminded_twice_with_a_fix_waiting`].
    const STOP_DUE: u64 = 30 * 60_000;

    /// What herdr-core's adapter answers when it cannot open its socket.
    const REFUSED: &str = "cannot reach herdr-core at /run/user/1000/herdr-core/default.sock: \
                           Connection refused (os error 111)";

    /// Makes the next stop fail without reaching the runner.
    fn refuse_stop_unreached(handle: &Shared) {
        handle
            .fake
            .lock()
            .expect("lock")
            .fail_next_stop(RunnerError::Unreached(REFUSED.into()));
    }

    const TIMED_OUT: &str = "the request timed out";
    const STILL_LIVE: &str = "t1 is still live after kill";

    /// Makes the next stop fail with `why`.
    fn refuse_stop(handle: &Shared, why: &str) {
        handle
            .fake
            .lock()
            .expect("lock")
            .fail_next_stop(RunnerError::Other(why.into()));
    }

    /// Polls `after_ms` after [`STOP_DUE`] and applies the events it returns.
    fn poll_after_due(part: &mut Runners, w: &mut World, after_ms: u64) -> Polled {
        let polled = part.poll(w, STOP_DUE + after_ms);
        apply(w, &polled.events);
        polled
    }

    /// The problem line for a stop of `agent`'s terminal that failed with `why`.
    fn cannot_stop(agent: &AgentId, terminal: &str, why: &str) -> String {
        format!("cannot stop {agent}'s terminal {terminal}: {why}")
    }

    #[test]
    fn a_failed_stop_after_the_last_reminder_is_tried_once_a_minute_later_and_not_sooner() {
        let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
        refuse_stop(&handle, TIMED_OUT);

        let polled = poll_after_due(&mut part, &mut w, 1_000);
        assert_eq!(stops(&handle), 1);
        assert!(!ended(&polled.events, &author), "{polled:?}");
        // Every 2 s up to a minute after the stop came due, and just before the
        // next try.
        for after in (2..=60).step_by(2).map(|s| s * 1_000).chain([60_999]) {
            let polled = poll_after_due(&mut part, &mut w, after);
            assert_eq!(stops(&handle), 1, "tried again at {after} ms");
            assert!(!ended(&polled.events, &author), "at {after} ms: {polled:?}");
            assert!(part.screen(&author.0).is_ok(), "at {after} ms");
            assert_eq!(part.waiting(), 1, "at {after} ms");
            assert!(!starts_an_author(&w), "at {after} ms: {:?}", w.missing());
        }

        let polled = poll_after_due(&mut part, &mut w, 61_000);
        assert_eq!(stops(&handle), 2, "a minute after the first");
        assert_eq!(reminders(&handle), 2);
        assert!(ended(&polled.events, &author), "{polled:?}");
        assert!(matches!(
            part.screen(&author.0),
            Err(RunnerError::Unknown(_))
        ));
        assert_eq!(part.waiting(), 0);
        assert!(starts_an_author(&w), "{:?}", w.missing());
    }

    #[test]
    fn a_failed_stop_after_the_last_reminder_is_tried_a_minute_after_each_refusal() {
        let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
        for why in [TIMED_OUT, STILL_LIVE, STILL_LIVE] {
            refuse_stop(&handle, why);
        }
        let tries = [1_000, 61_000, 121_000, 181_000];
        // A poll every 2 s for five minutes.
        for after in (1..=301).step_by(2).map(|s| s * 1_000) {
            let polled = poll_after_due(&mut part, &mut w, after);
            let tried = tries.iter().filter(|t| after >= **t).count();
            assert_eq!(stops(&handle), tried, "at {after} ms");
            assert_eq!(
                ended(&polled.events, &author),
                after == 181_000,
                "at {after} ms: {polled:?}"
            );
            let why = match after {
                ..61_000 => Some(TIMED_OUT),
                61_000..181_000 => Some(STILL_LIVE),
                _ => None,
            };
            assert_eq!(
                polled.problem,
                why.map(|why| cannot_stop(&author, "t1", why)),
                "at {after} ms"
            );
        }
        assert!(starts_an_author(&w), "{:?}", w.missing());
    }

    #[test]
    fn a_failed_stop_after_the_last_reminder_is_a_runner_problem_until_the_stop_works() {
        let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
        refuse_stop(&handle, TIMED_OUT);
        // The poll at 1 s makes the try that fails.
        for after in [1_000]
            .into_iter()
            .chain((2..=60).step_by(2).map(|s| s * 1_000))
        {
            let polled = poll_after_due(&mut part, &mut w, after);
            assert_eq!(
                polled.problem,
                Some(cannot_stop(&author, "t1", TIMED_OUT)),
                "at {after} ms"
            );
        }
        let polled = poll_after_due(&mut part, &mut w, 61_000);
        assert!(ended(&polled.events, &author), "{polled:?}");
        assert_eq!(polled.problem, None);

        // Two authors whose stops fail, beside a problem of the runner's own:
        // the runner's first, then a line for each agent.
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let first = run(&mut part, &mut w, &["claude"]);
        let second = run(&mut part, &mut w, &["claude"]);
        for minute in [0, 10, 20] {
            step(&mut part, &mut w, minute * 60_000);
        }
        refuse_stop(&handle, TIMED_OUT);
        refuse_stop(&handle, "t2 is still live after kill");
        *handle.problem.lock().expect("problem") = Some("cannot reach the core".into());
        assert_eq!(
            poll_after_due(&mut part, &mut w, 0).problem,
            Some(format!(
                "cannot reach the core; {}; {}",
                cannot_stop(&first, "t1", TIMED_OUT),
                cannot_stop(&second, "t2", "t2 is still live after kill")
            ))
        );
    }

    #[test]
    fn a_failed_stops_problem_clears_when_its_count_is_dropped() {
        use temet_work_core::model::Stage;
        let failed = || {
            let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
            refuse_stop(&handle, TIMED_OUT);
            let polled = poll_after_due(&mut part, &mut w, 1_000);
            assert_eq!(polled.problem, Some(cannot_stop(&author, "t1", TIMED_OUT)));
            (w, part, handle, author)
        };

        // The issue stops waiting on the agent.
        let (mut w, mut part, handle, _) = failed();
        at_stage(&mut w, Stage::InReview { round: 1 });
        assert_eq!(poll_after_due(&mut part, &mut w, 3_000).problem, None);
        assert_eq!(poll_after_due(&mut part, &mut w, 61_000).problem, None);
        assert_eq!(stops(&handle), 1);

        // The terminal is reported gone.
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = failed();
        handle.fake.lock().expect("lock").end_program(&t1);
        let polled = poll_after_due(&mut part, &mut w, 3_000);
        assert!(ended(&polled.events, &author), "{polled:?}");
        assert_eq!(polled.problem, None);

        // Its program is reported lost, to a shell: the refused stop may have
        // ended it, so it is an ordinary end that uses a start.
        let (mut w, mut part, handle, author) = failed();
        handle.fake.lock().expect("lock").lose_to_shell(&t1);
        let polled = poll_after_due(&mut part, &mut w, 3_000);
        assert!(ended(&polled.events, &author), "{polled:?}");
        assert!(!ended_lost(&polled.events, &author), "{polled:?}");
        assert!(w.agents[&author].ended_waited_on, "it uses a start");
        assert_eq!(polled.problem, None);

        // A send is answered with the terminal gone.
        let (mut w, mut part, handle, author) = failed();
        handle
            .fake
            .lock()
            .expect("lock")
            .fail_sends(RunnerError::Gone);
        let polled = poll_after_due(&mut part, &mut w, 3_000);
        assert!(ended(&polled.events, &author), "{polled:?}");
        assert_eq!(polled.problem, None);
    }

    #[test]
    fn a_stop_after_the_last_reminder_answered_gone_or_unknown_ends_the_agent() {
        let t1 = TerminalId("t1".into());
        for answer in [RunnerError::Gone, RunnerError::Unknown(t1)] {
            let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
            handle
                .fake
                .lock()
                .expect("lock")
                .fail_next_stop(answer.clone());
            let polled = step(&mut part, &mut w, 30 * 60_000);
            assert!(ended(&polled, &author), "{answer:?}: {polled:?}");
            assert!(
                !ended_lost(&polled, &author),
                "{answer:?}: no loss was reported, so lost is false"
            );
            assert!(w.agents[&author].ended_waited_on, "{answer:?} uses a start");
            assert!(
                matches!(part.screen(&author.0), Err(RunnerError::Unknown(_))),
                "{answer:?}"
            );
            assert_eq!(part.waiting(), 0, "{answer:?}");
        }
    }

    /// An author on ankh#1, which does not wait on it, polled once at 0, and
    /// the loop's stop of it, which the runner fails with each of `refusals`
    /// in turn: the first try is made at once and answered with the first.
    fn stop_refused(refusals: &[RunnerError]) -> (World, Runners, Shared, AgentId, Effect) {
        let mut w = world();
        let (mut part, handle) = shared();
        let author = run(&mut part, &mut w, &["claude"]);
        step(&mut part, &mut w, 0);
        for why in refusals {
            handle
                .fake
                .lock()
                .expect("lock")
                .fail_next_stop(why.clone());
        }
        let stop = Effect::StopAgent {
            agent: author.clone(),
        };
        assert_eq!(
            part.carry_out(&stop, &w),
            Outcome::Failed(refusals[0].to_string())
        );
        assert_eq!(stops(&handle), 1);
        (w, part, handle, author, stop)
    }

    fn timed_out() -> RunnerError {
        RunnerError::Other(TIMED_OUT.into())
    }

    #[test]
    fn stop_agent_that_fails_is_tried_again_after_a_minute() {
        let (mut w, mut part, handle, author, stop) = stop_refused(&[timed_out()]);
        // A poll at 30 s, then the same stop again, as `World::missing` asks
        // for it at every round: no try, and the first try's reason.
        assert!(!ended(&step(&mut part, &mut w, 30_000), &author));
        assert_eq!(part.carry_out(&stop, &w), Outcome::Failed(TIMED_OUT.into()));
        assert!(!ended(&step(&mut part, &mut w, 59_999), &author));
        assert_eq!(stops(&handle), 1, "not tried again within the minute");
        assert!(part.screen(&author.0).is_ok(), "it keeps its terminal");

        // The poll at the minute tries again; the stop works and ends author-1.
        let events = step(&mut part, &mut w, 60_000);
        assert_eq!(stops(&handle), 2);
        assert_eq!(
            events,
            [Event::AgentChanged {
                agent: author.clone(),
                state: AgentState::Ended,
                lost: false,
            }]
        );
        assert_eq!(w.agents[&author].state, AgentState::Ended);
        // Nothing tries to stop it after that.
        for at in [62_000, 120_000, 180_000, 240_000] {
            assert_eq!(step(&mut part, &mut w, at), [], "at {at} ms");
        }
        assert_eq!(stops(&handle), 2);
    }

    /// An author on ankh#1 while it is worked on, so the issue waits on it,
    /// stopped by a person at 0 ms: the stop the loop asks for at once fails
    /// with each of `refusals` in turn.
    fn persons_stop_refused(refusals: &[RunnerError]) -> (World, Runners, Shared, AgentId) {
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let author = run(&mut part, &mut w, &["claude"]);
        step(&mut part, &mut w, 0);
        assert!(w.waiting_on(&w.agents[&author]), "before the stop");
        for why in refusals {
            handle
                .fake
                .lock()
                .expect("lock")
                .fail_next_stop(why.clone());
        }
        let asked = w.apply(&Event::AgentStoppedByPerson {
            agent: author.clone(),
        });
        let stop = Effect::StopAgent {
            agent: author.clone(),
        };
        assert_eq!(asked, std::slice::from_ref(&stop));
        assert_eq!(
            part.carry_out(&stop, &w),
            Outcome::Failed(refusals[0].to_string())
        );
        (w, part, handle, author)
    }

    #[test]
    fn persons_stop_that_fails_is_tried_again_a_minute_after_the_last_try_until_it_works() {
        let (mut w, mut part, handle, author) = persons_stop_refused(&[timed_out(), timed_out()]);
        let stop = Effect::StopAgent {
            agent: author.clone(),
        };
        // The loop asks again each round: within the minute the answer is the
        // last try's reason, which Problems lists as `stop author-1`.
        assert_eq!(w.missing().start, std::slice::from_ref(&stop));
        assert_eq!(part.carry_out(&stop, &w), Outcome::Failed(TIMED_OUT.into()));
        assert_eq!(stop.describe(), "stop author-1");
        assert!(!ended(&step(&mut part, &mut w, 59_999), &author));
        assert_eq!(stops(&handle), 1, "not tried again within the minute");
        // Its issue would wait on an author, but not on one a person stopped:
        // the tries are not dropped.
        assert!(!ended(&step(&mut part, &mut w, 60_000), &author));
        assert_eq!(stops(&handle), 2, "tried at the minute, and refused");
        assert_eq!(part.carry_out(&stop, &w), Outcome::Failed(TIMED_OUT.into()));
        let events = step(&mut part, &mut w, 120_000);
        assert_eq!(stops(&handle), 3);
        assert!(ended(&events, &author), "{events:?}");
        assert!(
            matches!(part.screen(&author.0), Err(RunnerError::Unknown(_))),
            "its terminal is stopped and removed"
        );
        let a = &w.agents[&author];
        assert!(!a.ended_waited_on, "it did not give up: no start is used");
        for at in [180_000, 240_000] {
            assert_eq!(step(&mut part, &mut w, at), [], "at {at} ms");
        }
        assert_eq!(stops(&handle), 3);
    }

    #[test]
    fn persons_stop_answered_gone_or_unknown_ends_the_agent() {
        for answer in [
            RunnerError::Gone,
            RunnerError::Unknown(TerminalId("t1".into())),
        ] {
            let (mut w, mut part, handle, author) =
                persons_stop_refused(&[timed_out(), answer.clone()]);
            let events = step(&mut part, &mut w, 60_000);
            assert_eq!(stops(&handle), 2, "{answer:?}");
            assert!(ended(&events, &author), "{answer:?}: {events:?}");
        }
    }

    #[test]
    fn persons_stop_that_works_ends_the_agent_and_removes_its_terminal() {
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let author = run(&mut part, &mut w, &["claude"]);
        let asked = w.apply(&Event::AgentStoppedByPerson {
            agent: author.clone(),
        });
        let [stop] = asked.as_slice() else {
            panic!("{asked:?}");
        };
        let Outcome::Done { events, .. } = part.carry_out(stop, &w) else {
            panic!("not stopped");
        };
        assert!(ended(&events, &author), "{events:?}");
        assert_eq!(stops(&handle), 1);
        assert!(matches!(
            part.screen(&author.0),
            Err(RunnerError::Unknown(_))
        ));
    }

    #[test]
    fn persons_stop_types_nothing_more_into_the_agent_while_its_stop_keeps_failing() {
        let issue = IssueId::new("ankh", "1");
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Fixing { round: 1 });
        let (mut part, handle) = shared();
        let terminals = handle.fake.clone();
        let author = run(&mut part, &mut w, &[fake::PROMPT_PROGRAM]);
        w.issues.get_mut(&issue).expect("there").told = false;
        let fix = Effect::RequestFix {
            issue: issue.clone(),
            points: vec!["no test for a stopped clock".into()],
        };
        part.carry_out(&fix, &w);
        assert_eq!(part.waiting(), 1, "it waits while the author is at a prompt");
        handle
            .fake
            .lock()
            .expect("lock")
            .fail_next_stop(timed_out());
        w.apply(&Event::AgentStoppedByPerson {
            agent: author.clone(),
        });
        let stop = Effect::StopAgent {
            agent: author.clone(),
        };
        assert_eq!(part.carry_out(&stop, &w), Outcome::Failed(TIMED_OUT.into()));
        // The loop sends it no fix, as it is a person's stopped agent.
        assert!(!w.missing().start.contains(&fix), "{:?}", w.missing());
        terminals
            .lock()
            .expect("lock")
            .unblock(&TerminalId("t1".into()));
        // Between turns for half an hour, its stop failing at every try: no
        // fix, no reminder and no continue.
        for (minutes, refuse) in [(0, false), (11, true), (21, true), (31, true)] {
            if refuse {
                handle
                    .fake
                    .lock()
                    .expect("lock")
                    .fail_next_stop(timed_out());
            }
            let polled = step(&mut part, &mut w, minutes * 60_000);
            assert!(
                !polled.iter().any(|e| matches!(e, Event::Told { .. })),
                "{polled:?}"
            );
        }
        let screen = screen_of(&terminals);
        assert!(!screen.contains("stopped clock"), "{screen}");
        assert!(!screen.contains("still waiting on you"), "{screen}");
        assert_eq!(continues(&terminals), 0);
        assert_eq!(part.waiting(), 0, "what waited for it is dropped");
    }

    /// A stop of the loop's that fails, then is answered with `answer` at the
    /// minute: the retries end and the agent is reported ended.
    fn retry_answered(answer: RunnerError) {
        let (mut w, mut part, handle, author, _) = stop_refused(&[timed_out(), answer.clone()]);
        let events = step(&mut part, &mut w, 60_000);
        assert_eq!(stops(&handle), 2, "{answer:?}");
        assert!(ended(&events, &author), "{answer:?}: {events:?}");
        assert!(
            matches!(part.screen(&author.0), Err(RunnerError::Unknown(_))),
            "{answer:?}: its terminal is dropped"
        );
        for at in [62_000, 120_000, 180_000] {
            assert_eq!(step(&mut part, &mut w, at), [], "{answer:?} at {at} ms");
        }
        assert_eq!(stops(&handle), 2, "{answer:?}: no more tries");
    }

    #[test]
    fn stop_agent_retry_answered_gone_ends_the_agent() {
        retry_answered(RunnerError::Gone);
    }

    #[test]
    fn stop_agent_retry_answered_unknown_ends_the_agent() {
        retry_answered(RunnerError::Unknown(TerminalId("t1".into())));
    }

    #[test]
    fn stop_agent_retry_is_dropped_once_the_issue_waits_on_the_agent_again() {
        let (mut w, mut part, handle, author, _) = stop_refused(&[timed_out()]);
        // The loop puts author-1 back to work before the minute is up, as it
        // does a reviewer whose stop failed when it hears the next round.
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        for at in [60_000, 120_000, 180_000] {
            assert!(!ended(&step(&mut part, &mut w, at), &author), "at {at} ms");
        }
        assert_eq!(stops(&handle), 1, "no retry");
        assert!(part.screen(&author.0).is_ok());
    }

    #[test]
    fn stop_agent_loss_after_a_refused_try_is_lost_only_when_the_try_reached_nothing() {
        let t1 = TerminalId("t1".into());
        for (refusal, lost) in [
            (timed_out(), false),
            (RunnerError::Unreached(REFUSED.into()), true),
        ] {
            let (mut w, mut part, handle, author, _) = stop_refused(std::slice::from_ref(&refusal));
            handle.fake.lock().expect("lock").lose_to_shell(&t1);
            let events = step(&mut part, &mut w, 2_000);
            let end = Event::AgentChanged {
                agent: author.clone(),
                state: AgentState::Ended,
                lost,
            };
            assert!(events.contains(&end), "{refusal:?}: {events:?}");
        }
    }

    #[test]
    fn stop_agent_of_an_old_author_whose_issue_works_in_another_folder_is_tried_once_a_minute() {
        let mut w = world();
        w.apply(&Event::FolderAdded {
            folder: FolderId::new("f2"),
            project: ProjectId::new("ankh"),
            machine: MachineId::new(LOCAL),
            path: "./f2".into(),
        });
        let (mut part, handle) = shared();
        let author = run(&mut part, &mut w, &["claude"]);
        // Its issue merged, the stop failed, the verify failed, and the issue
        // works again in f2, since author-1 still holds f1.
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        if let Some(i) = w.issues.get_mut(&IssueId::new("ankh", "1")) {
            i.folder = Some(FolderId::new("f2"));
        }
        assert!(!w.waiting_on(&w.agents[&author]));
        step(&mut part, &mut w, 0);
        refuse_stop(&handle, TIMED_OUT);
        let stop = Effect::StopAgent {
            agent: author.clone(),
        };
        let asked = |w: &World| w.missing().start.contains(&stop);
        // Each 2 s for 58 s: a poll, then the stop `World::missing` asks for.
        for at in (0..=58).step_by(2).map(|s| s * 1_000) {
            if at > 0 {
                assert!(!ended(&step(&mut part, &mut w, at), &author), "at {at} ms");
            }
            assert!(asked(&w), "at {at} ms");
            assert_eq!(
                part.carry_out(&stop, &w),
                Outcome::Failed(TIMED_OUT.into()),
                "at {at} ms"
            );
        }
        assert_eq!(stops(&handle), 1, "tried once in the minute");
        assert!(ended(&step(&mut part, &mut w, 60_000), &author));
        assert_eq!(stops(&handle), 2);
        assert!(!asked(&w));
    }

    #[test]
    fn stop_agent_retry_that_fails_again_is_next_tried_a_minute_after_it() {
        let still_live = RunnerError::Other(STILL_LIVE.into());
        let (mut w, mut part, handle, author, stop) =
            stop_refused(&[timed_out(), still_live.clone()]);
        // The retry at the minute fails with another reason, which an ask
        // right after it and one at 90 s answer with, without a try.
        assert!(!ended(&step(&mut part, &mut w, 60_000), &author));
        assert_eq!(stops(&handle), 2);
        assert_eq!(
            part.carry_out(&stop, &w),
            Outcome::Failed(STILL_LIVE.into())
        );
        step(&mut part, &mut w, 90_000);
        assert_eq!(
            part.carry_out(&stop, &w),
            Outcome::Failed(STILL_LIVE.into())
        );
        // The next try is a minute after that retry, not after the first.
        assert!(!ended(&step(&mut part, &mut w, 119_999), &author));
        assert_eq!(stops(&handle), 2);
        assert!(ended(&step(&mut part, &mut w, 120_000), &author));
        assert_eq!(stops(&handle), 3);
    }

    #[test]
    fn a_loss_after_the_polls_update_read_ends_the_agent_lost_at_its_last_reminder_stop() {
        let t1 = TerminalId("t1".into());
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let author = run(&mut part, &mut w, &["claude"]);
        for minute in [0, 10, 20] {
            step(&mut part, &mut w, minute * 60_000);
        }
        assert_eq!(reminders(&handle), 2);
        // The poll at the stop reads the updates, and the loss comes in after.
        *handle.lose_after_read.lock().expect("lock") = Some(t1.clone());
        let polled = step(&mut part, &mut w, STOP_DUE);
        assert!(ended_lost(&polled, &author), "{polled:?}");
        let calls = handle.fake.lock().expect("fake").calls();
        assert_eq!(
            calls.lock().expect("calls").last(),
            Some(&fake::Call::Stop(t1)),
            "its terminal, the shell, is stopped"
        );
        assert_eq!(stops(&handle), 1);
        let a = &w.agents[&author];
        assert!(!a.ended_waited_on, "it does not use up a start");
        assert!(a.lost_waited_on, "it counts toward the lost limit");
        assert!(starts_an_author(&w), "{:?}", w.missing());
        // The next poll reads the loss and the stop, and changes nothing.
        assert_eq!(step(&mut part, &mut w, STOP_DUE + 2_000), vec![]);
        assert_eq!(stops(&handle), 1, "stopped once");
    }

    /// Whether `agent` has ended as an ordinary end that uses a start: by
    /// `events`, and as the world records it.
    fn ended_using_a_start(w: &World, events: &[Event], agent: &AgentId) -> bool {
        let a = &w.agents[agent];
        ended(events, agent) && !ended_lost(events, agent) && a.ended_waited_on && !a.lost_waited_on
    }

    /// Whether `agent` has ended as lost, using no start.
    fn ended_lost_using_no_start(w: &World, events: &[Event], agent: &AgentId) -> bool {
        let a = &w.agents[agent];
        ended_lost(events, agent) && !ended(events, agent) && !a.ended_waited_on && a.lost_waited_on
    }

    #[test]
    fn an_author_lost_after_its_refused_last_reminder_stop_three_times_leaves_no_fourth_to_start() {
        // herdr-core kills the idle author's program, then its `rm` times out
        // or finds the terminal still live, and it puts a shell in its place:
        // as the issue saw it every 31 minutes.
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        for n in 1..=3_u64 {
            let at = (n - 1) * 3_600_000;
            let author = run_and_remind_twice(&mut part, &mut w, at);
            let terminal = TerminalId(format!("t{n}"));
            let why = if n == 2 {
                format!("{terminal} is still live after kill")
            } else {
                TIMED_OUT.to_string()
            };
            refuse_stop(&handle, &why);
            let polled = part.poll(&w, at + STOP_DUE + 1_000);
            apply(&mut w, &polled.events);
            assert!(!ended(&polled.events, &author), "{polled:?}");
            assert_eq!(
                polled.problem,
                Some(cannot_stop(&author, &terminal.0, &why))
            );
            handle.fake.lock().expect("lock").lose_to_shell(&terminal);
            let polled = part.poll(&w, at + STOP_DUE + 3_000);
            apply(&mut w, &polled.events);
            assert!(
                ended_using_a_start(&w, &polled.events, &author),
                "author {n}: {polled:?}"
            );
            assert_eq!(polled.problem, None);
            assert_eq!(
                starts_an_author(&w),
                n < 3,
                "after author {n}: {:?}",
                w.missing()
            );
        }
        assert_eq!(
            w.missing().given_up,
            ["ankh#1: 3 authors ended before it moved on; no more are started"]
        );
    }

    #[test]
    fn a_loss_read_through_latest_at_the_retry_of_a_refused_stop_uses_a_start() {
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
        refuse_stop(&handle, TIMED_OUT);
        let polled = poll_after_due(&mut part, &mut w, 1_000);
        assert!(!ended(&polled.events, &author), "{polled:?}");
        // The retry's poll reads the updates, and the loss comes in after.
        *handle.lose_after_read.lock().expect("lock") = Some(t1.clone());
        let polled = poll_after_due(&mut part, &mut w, 61_000);
        assert!(
            ended_using_a_start(&w, &polled.events, &author),
            "{polled:?}"
        );
        assert_eq!(stops(&handle), 2);
        assert_eq!(polled.problem, None);
        assert_eq!(poll_after_due(&mut part, &mut w, 63_000).events, vec![]);
        assert_eq!(stops(&handle), 2, "stopped once more, not again");
    }

    #[test]
    fn a_loss_after_a_stop_refused_unreached_ends_the_agent_lost() {
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
        refuse_stop_unreached(&handle);
        let polled = poll_after_due(&mut part, &mut w, 1_000);
        assert!(!ended(&polled.events, &author), "{polled:?}");
        assert_eq!(polled.problem, Some(cannot_stop(&author, "t1", REFUSED)));
        handle.fake.lock().expect("lock").lose_to_shell(&t1);
        let polled = poll_after_due(&mut part, &mut w, 3_000);
        assert!(
            ended_lost_using_no_start(&w, &polled.events, &author),
            "nothing reached the runner, so the loss is not the daemon's: {polled:?}"
        );
        assert!(starts_an_author(&w), "{:?}", w.missing());
    }

    #[test]
    fn a_stop_refusal_that_may_have_reached_the_runner_is_not_undone_by_an_unreached_retry() {
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
        refuse_stop(&handle, TIMED_OUT);
        refuse_stop_unreached(&handle);
        poll_after_due(&mut part, &mut w, 1_000);
        let polled = poll_after_due(&mut part, &mut w, 61_000);
        assert_eq!(stops(&handle), 2);
        assert_eq!(polled.problem, Some(cannot_stop(&author, "t1", REFUSED)));
        handle.fake.lock().expect("lock").lose_to_shell(&t1);
        let polled = poll_after_due(&mut part, &mut w, 63_000);
        assert!(
            ended_using_a_start(&w, &polled.events, &author),
            "{polled:?}"
        );
    }

    #[test]
    fn a_stop_answered_unknown_after_its_call_brought_a_loss_in_ends_the_agent_lost() {
        let t1 = TerminalId("t1".into());
        // herdr-core restarted: the poll's update read found nothing, the stop's
        // call reconnects and reads the terminal lost, and the kill answers
        // that it has no such terminal.
        let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
        *handle.lose_on_stop.lock().expect("lock") = Some(t1.clone());
        handle
            .fake
            .lock()
            .expect("lock")
            .fail_next_stop(RunnerError::Unknown(t1.clone()));
        let polled = poll_after_due(&mut part, &mut w, 0);
        assert!(
            ended_lost_using_no_start(&w, &polled.events, &author),
            "{polled:?}"
        );
        assert_eq!(stops(&handle), 1);
        assert_eq!(polled.problem, None);
        // The next poll reads the loss and changes nothing.
        assert_eq!(poll_after_due(&mut part, &mut w, 2_000).events, vec![]);
        assert_eq!(stops(&handle), 1, "stopped once");

        // A stop that works is not read again: the loss is its own kill's.
        let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
        *handle.lose_on_stop.lock().expect("lock") = Some(t1.clone());
        let polled = poll_after_due(&mut part, &mut w, 0);
        assert!(
            ended_using_a_start(&w, &polled.events, &author),
            "{polled:?}"
        );

        // After a refusal that may have reached the runner, the loss is that
        // stop's.
        let (mut w, mut part, handle, author) = reminded_twice_with_a_fix_waiting();
        refuse_stop(&handle, TIMED_OUT);
        poll_after_due(&mut part, &mut w, 1_000);
        *handle.lose_on_stop.lock().expect("lock") = Some(t1.clone());
        handle
            .fake
            .lock()
            .expect("lock")
            .fail_next_stop(RunnerError::Unknown(t1.clone()));
        let polled = poll_after_due(&mut part, &mut w, 61_000);
        assert!(
            ended_using_a_start(&w, &polled.events, &author),
            "{polled:?}"
        );
    }

    #[test]
    fn after_a_restart_an_agent_whose_terminal_is_not_reported_yet_is_not_ended() {
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut before, handle) = shared();
        let author = run(&mut before, &mut w, &["claude"]);
        step(&mut before, &mut w, 0);
        drop(before);
        // This daemon's runner cannot reach its terminal service: it reports
        // nothing, past the time of the stop.
        let mut part = Runners::default();
        part.add(Box::new(handle.clone()));
        for minute in [0, 10, 20, 30, 40] {
            let polled = step(&mut part, &mut w, minute * 60_000);
            assert!(!ended(&polled, &author), "at {minute} min: {polled:?}");
        }
        assert!(!starts_an_author(&w), "{:?}", w.missing());
        // It connects: the session was running all along, and is stopped.
        handle.fake.lock().expect("lock").reconnect();
        let polled = step(&mut part, &mut w, 41 * 60_000);
        assert!(ended(&polled, &author), "{polled:?}");
        assert_eq!(stops(&handle), 1);
        assert_eq!(part.waiting(), 0);
    }

    #[test]
    fn a_limit_error_that_returns_after_continue_is_reminded_not_continued_again() {
        let t1 = TerminalId("t1".into());
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let fake = handle.fake;
        let author = run(&mut part, &mut w, &["claude"]);
        fake.lock().expect("lock").end_in_error(&t1, LIMIT);
        step(&mut part, &mut w, nov27(16, 50));
        assert!(
            step(&mut part, &mut w, nov27(17, 41)).contains(&Event::Continued {
                agent: author.clone(),
                resets_ms: nov27(17, 40)
            })
        );
        // The same limit comes straight back.
        fake.lock().expect("lock").end_in_error(&t1, LIMIT);
        step(&mut part, &mut w, nov27(17, 42));
        assert_eq!(
            w.agents[&author].state,
            AgentState::Errored {
                line: LIMIT.into(),
                resets_ms: Some(nov27(17, 40))
            }
        );
        assert_eq!(w.agents[&author].limit_reset(), None, "already acted on");
        let reminders = || screen_of(&fake).matches("still waiting on you").count();
        for minute in 43..=71 {
            let polled = step(&mut part, &mut w, nov27(17, 0) + minute * 60_000);
            assert!(
                !polled.iter().any(|e| matches!(e, Event::Continued { .. })),
                "continued again at 17:00 + {minute} min"
            );
        }
        assert_eq!(continues(&fake), 1, "not continued on every poll");
        assert_eq!(reminders(), 2, "at 17:52 and 18:02");
        assert!(
            step(&mut part, &mut w, nov27(18, 12)).contains(&Event::AgentChanged {
                agent: author,
                state: AgentState::Ended,
                lost: false,
            })
        );

        // A later reset, such as a weekly limit after a 5-hour one, is waited for again.
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let fake = handle.fake;
        let reminders = || screen_of(&fake).matches("still waiting on you").count();
        let author = run(&mut part, &mut w, &["claude"]);
        fake.lock().expect("lock").end_in_error(&t1, LIMIT);
        step(&mut part, &mut w, nov27(16, 50));
        step(&mut part, &mut w, nov27(17, 41));
        fake.lock()
            .expect("lock")
            .end_in_error(&t1, "You've hit your weekly limit · resets 11pm");
        step(&mut part, &mut w, nov27(17, 42));
        assert_eq!(w.agents[&author].limit_reset(), Some(nov27(23, 0)));
        for (hh, mm) in [(17, 52), (18, 2), (18, 12), (23, 0)] {
            assert_eq!(
                step(&mut part, &mut w, nov27(hh, mm)),
                vec![],
                "{hh}:{mm:02}"
            );
        }
        assert_eq!((continues(&fake), reminders()), (1, 0));
        assert_eq!(
            step(&mut part, &mut w, nov27(23, 1)),
            vec![Event::Continued {
                agent: author,
                resets_ms: nov27(23, 0)
            }]
        );
        assert_eq!(continues(&fake), 2);
    }

    #[test]
    fn a_held_daemon_types_nothing_into_an_agent_of_a_paused_issue_until_the_resume() {
        let issue = IssueId::new("ankh", "1");
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Fixing { round: 1 });
        let (mut part, handle) = shared();
        let terminals = handle.fake;
        run(&mut part, &mut w, &[fake::PROMPT_PROGRAM]);
        w.issues.get_mut(&issue).expect("there").told = false;
        let fix = Effect::RequestFix {
            issue: issue.clone(),
            points: vec!["no test for a stopped clock".into()],
        };
        part.carry_out(&fix, &w);
        assert_eq!(
            part.waiting(),
            1,
            "it waits while the author is at a prompt"
        );
        w.apply(&Event::Held);
        terminals
            .lock()
            .expect("lock")
            .unblock(&TerminalId("t1".into()));
        // Between turns for over half an hour: no fix, no reminder and no stop.
        for minutes in [0, 11, 21, 31] {
            let polled = step(&mut part, &mut w, minutes * 60_000);
            assert!(
                !polled.iter().any(|e| matches!(
                    e,
                    Event::Told { .. }
                        | Event::AgentChanged {
                            state: AgentState::Ended,
                            ..
                        }
                )),
                "{polled:?}"
            );
        }
        let screen = screen_of(&terminals);
        assert!(!screen.contains("stopped clock"), "{screen}");
        assert!(!screen.contains("still waiting on you"), "{screen}");
        assert_eq!(part.waiting(), 1);
        w.apply(&Event::Resumed);
        let polled = step(&mut part, &mut w, 32 * 60_000);
        assert!(polled.contains(&Event::Told { issue }), "{polled:?}");
        assert!(screen_of(&terminals).contains("- no test for a stopped clock"));
    }

    #[test]
    fn fixes_wait_while_the_author_waits_for_its_limit_to_reset() {
        let issue = IssueId::new("ankh", "1");
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Fixing { round: 1 });
        let (mut part, handle) = shared();
        let fake = handle.fake;
        run(&mut part, &mut w, &["claude"]);
        // The fixes came after it started, so its brief did not have them.
        w.issues.get_mut(&issue).expect("there").told = false;
        fake.lock()
            .expect("lock")
            .end_in_error(&TerminalId("t1".into()), LIMIT);
        step(&mut part, &mut w, nov27(16, 50));
        let fix = Effect::RequestFix {
            issue: issue.clone(),
            points: vec!["no test for a stopped clock".into()],
        };
        assert_eq!(part.carry_out(&fix, &w), Outcome::done(), "nobody told");
        assert_eq!(part.waiting(), 1);
        for (hh, mm) in [(17, 0), (17, 40)] {
            let polled = step(&mut part, &mut w, nov27(hh, mm));
            assert!(
                !polled.iter().any(|e| matches!(e, Event::Told { .. })),
                "{polled:?}"
            );
        }
        assert!(!screen_of(&fake).contains("stopped clock"));
        assert!(!w.issues[&issue].told);

        let polled = step(&mut part, &mut w, nov27(17, 41));
        let continued = polled
            .iter()
            .position(|e| matches!(e, Event::Continued { .. }));
        let told = polled.iter().position(|e| matches!(e, Event::Told { .. }));
        assert!(
            continued.is_some() && told.is_some() && continued < told,
            "{polled:?}"
        );
        let screen = screen_of(&fake);
        let at = |text: &str| screen.find(text).unwrap_or(usize::MAX);
        assert!(
            at("\ncontinue\n") < at("- no test for a stopped clock"),
            "{screen}"
        );
        assert!(w.issues[&issue].told);
        assert_eq!(part.waiting(), 0);
    }

    #[test]
    fn after_a_restart_an_agent_waiting_on_its_limit_is_still_continued() {
        let mut w = world();
        let base = w.clone();
        let (mut part, handle) = shared();
        let mut log = run_events(&mut part, &mut w, &["claude"]);
        handle
            .fake
            .lock()
            .expect("lock")
            .end_in_error(&TerminalId("t1".into()), LIMIT);
        log.extend(step(&mut part, &mut w, nov27(16, 50)));
        drop(part);

        // The daemon restarts: its state is the log replayed, and its runner
        // lists the terminals it holds again.
        let mut again = base;
        apply(&mut again, &log);
        assert_eq!(again, w, "replaying the log gives the same state");
        let author = AgentId::new("author-1");
        assert_eq!(again.agents[&author].limit_reset(), Some(nov27(17, 40)));
        let mut part = Runners::default();
        part.add(Box::new(handle.clone()));
        handle.fake.lock().expect("lock").reconnect();
        let limited = screen_of(&handle.fake);
        for (hh, mm) in [(17, 0), (17, 40)] {
            assert_eq!(part.poll(&again, nov27(hh, mm)).events, vec![]);
        }
        assert_eq!(
            screen_of(&handle.fake),
            limited,
            "nothing typed before 17:41"
        );
        assert_eq!(
            part.poll(&again, nov27(17, 41)).events,
            vec![Event::Continued {
                agent: author,
                resets_ms: nov27(17, 40)
            }]
        );
        assert_eq!(screen_of(&handle.fake), format!("{limited}\ncontinue"));
    }

    const OVERLOADED: &str = "API Error: 529 overloaded";

    /// An author on ankh#1, waited on, whose turn ended on [`OVERLOADED`] and was
    /// seen at `now_ms`.
    fn overloaded_author(now_ms: u64) -> (World, Runners, Shared, AgentId) {
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let author = run(&mut part, &mut w, &["claude"]);
        handle
            .fake
            .lock()
            .expect("lock")
            .end_in_error(&TerminalId("t1".into()), OVERLOADED);
        step(&mut part, &mut w, now_ms);
        (w, part, handle, author)
    }

    fn reminders(handle: &Shared) -> usize {
        screen_of(&handle.fake)
            .matches("still waiting on you")
            .count()
    }

    fn ended(events: &[Event], agent: &AgentId) -> bool {
        events.contains(&Event::AgentChanged {
            agent: agent.clone(),
            state: AgentState::Ended,
            lost: false,
        })
    }

    /// Whether `events` end `agent` as one whose program was lost.
    fn ended_lost(events: &[Event], agent: &AgentId) -> bool {
        events.contains(&Event::AgentChanged {
            agent: agent.clone(),
            state: AgentState::Ended,
            lost: true,
        })
    }

    fn working(state: &AgentState) -> bool {
        matches!(state, AgentState::Working { .. })
    }

    #[test]
    fn a_fake_terminal_that_keeps_failing_is_working_for_one_call_to_updates() {
        let t1 = TerminalId("t1".into());
        let mut fake = FakeRunner::default();
        fake.start(&Start::default()).expect("start");
        fake.end_in_error(&t1, OVERLOADED);
        fake.updates();
        let statuses = |fake: &mut FakeRunner| -> Vec<Status> {
            fake.updates().into_iter().map(|u| u.status).collect()
        };
        // Not marked: a send ends between turns at once.
        fake.send(&t1, "again", true).expect("send");
        assert_eq!(statuses(&mut fake), [Status::Working, Status::Between]);

        fake.end_in_error(&t1, OVERLOADED);
        fake.keep_failing(&t1);
        fake.updates();
        for text in ["once", "twice"] {
            fake.send(&t1, text, true).expect("send");
            assert_eq!(statuses(&mut fake), [Status::Working]);
            let screen = fake.screen(&t1).expect("screen");
            assert!(screen.ends_with(text), "{screen}");
            assert_eq!(fake.send(&t1, "more", true), Err(RunnerError::Busy));
            assert_eq!(statuses(&mut fake), [Status::Error { message: None }]);
            let screen = fake.screen(&t1).expect("screen");
            assert!(screen.ends_with(OVERLOADED), "{screen}");
            assert_eq!(statuses(&mut fake), []);
        }
    }

    #[test]
    fn a_fake_terminal_that_keeps_working_stays_in_its_turn_until_ended() {
        let t1 = TerminalId("t1".into());
        let mut fake = FakeRunner::default();
        fake.start(&Start::default()).expect("start");
        fake.keep_working(&t1);
        fake.updates();
        let statuses = |fake: &mut FakeRunner| -> Vec<Status> {
            fake.updates().into_iter().map(|u| u.status).collect()
        };
        fake.send(&t1, "once", true).expect("send");
        assert_eq!(statuses(&mut fake), [Status::Working]);
        assert_eq!(statuses(&mut fake), []);
        assert_eq!(statuses(&mut fake), []);
        assert_eq!(fake.send(&t1, "more", true), Err(RunnerError::Busy));
        fake.end_in_error(&t1, OVERLOADED);
        assert_eq!(statuses(&mut fake), [Status::Error { message: None }]);
        // Unmarked, a send ends between turns at once again.
        fake.unblock(&t1);
        fake.updates();
        fake.send(&t1, "again", true).expect("send");
        assert_eq!(statuses(&mut fake), [Status::Working, Status::Between]);
    }

    #[test]
    fn an_error_that_comes_back_after_each_reminder_is_still_stopped() {
        let (mut w, mut part, handle, author) = overloaded_author(0);
        handle
            .fake
            .lock()
            .expect("lock")
            .keep_failing(&TerminalId("t1".into()));
        let minute = |n: u64| n * 60_000;
        for (n, at) in [(1, minute(10)), (2, minute(20))] {
            step(&mut part, &mut w, at - 1);
            assert_eq!(reminders(&handle), n - 1, "not before {at}");
            step(&mut part, &mut w, at);
            assert_eq!(reminders(&handle), n);
            step(&mut part, &mut w, at + 2_000);
            assert!(working(&w.agents[&author].state), "{:?}", w.agents[&author]);
            // The turn retries for half a minute, which does not move the clock.
            step(&mut part, &mut w, at + 30_000);
            assert_eq!(
                w.agents[&author].state,
                AgentState::Errored {
                    line: OVERLOADED.into(),
                    resets_ms: None
                }
            );
        }
        let before = step(&mut part, &mut w, minute(30) - 1);
        assert!(!ended(&before, &author), "{before:?}");
        let stopped = step(&mut part, &mut w, minute(30));
        assert!(ended(&stopped, &author), "{stopped:?}");
        assert_eq!(reminders(&handle), 2);
        assert!(
            w.missing().start.iter().any(|e| matches!(
                e,
                Effect::StartAgent {
                    role: Role::Author,
                    ..
                }
            )),
            "{:?}",
            w.missing()
        );
    }

    #[test]
    fn an_errored_agent_whose_reminder_works_an_hour_then_errors_is_reminded_afresh() {
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = overloaded_author(0);
        handle.fake.lock().expect("lock").keep_failing(&t1);
        let minute = |n: u64| n * 60_000;
        // The first reminder's turn fails within 30 seconds.
        step(&mut part, &mut w, minute(10));
        assert_eq!(reminders(&handle), 1);
        step(&mut part, &mut w, minute(10) + 2_000);
        step(&mut part, &mut w, minute(10) + 30_000);
        assert!(matches!(
            w.agents[&author].state,
            AgentState::Errored { .. }
        ));
        // The second one's gets through and works for an hour.
        handle.fake.lock().expect("lock").keep_working(&t1);
        step(&mut part, &mut w, minute(20));
        assert_eq!(reminders(&handle), 2);
        for at in [minute(20) + 2_000, minute(30), minute(50), minute(80) - 1] {
            let polled = step(&mut part, &mut w, at);
            assert!(!ended(&polled, &author), "stopped at {at} ms");
            assert!(working(&w.agents[&author].state), "at {at} ms");
        }
        {
            let mut fake = handle.fake.lock().expect("lock");
            fake.end_in_error(&t1, OVERLOADED);
            fake.keep_failing(&t1);
        }
        let polled = step(&mut part, &mut w, minute(80));
        assert!(!ended(&polled, &author), "{polled:?}");
        assert_eq!(reminders(&handle), 2, "no reminder on the error itself");
        for (n, at) in [(3, minute(90)), (4, minute(100))] {
            step(&mut part, &mut w, at - 1);
            assert_eq!(reminders(&handle), n - 1, "not before {at}");
            let polled = step(&mut part, &mut w, at);
            assert!(!ended(&polled, &author), "{polled:?}");
            assert_eq!(reminders(&handle), n);
            step(&mut part, &mut w, at + 2_000);
            step(&mut part, &mut w, at + 30_000);
        }
        let before = step(&mut part, &mut w, minute(110) - 1_000);
        assert!(!ended(&before, &author), "{before:?}");
        let stopped = step(&mut part, &mut w, minute(110));
        assert!(ended(&stopped, &author), "{stopped:?}");
        assert_eq!(reminders(&handle), 4);
    }

    #[test]
    fn a_reminders_turn_counts_as_recovery_after_remind_after() {
        use temet_work_core::settings::{Scope, Value};
        let t1 = TerminalId("t1".into());
        let minute = |n: u64| n * 60_000;
        // The author's `remind_after` when set, how long the second reminder's
        // turn is seen working before its error, and whether that is recovery.
        for (set, worked, recovers) in [
            (None, minute(10) - 1_000, false),
            (None, minute(10), true),
            (Some(30), minute(30) - 1_000, false),
            (Some(30), minute(30), true),
        ] {
            let case = format!("remind_after {set:?}, worked {worked} ms");
            let (mut w, mut part, handle, author) = overloaded_author(0);
            if let Some(minutes) = set {
                w.apply(&Event::SettingSet {
                    scope: Scope::Task(IssueId::new("ankh", "1")),
                    role: Some(Role::Author),
                    key: "remind_after".into(),
                    value: Value::Number(minutes),
                    by: "person".into(),
                });
            }
            let after = minute(set.unwrap_or(10));
            handle.fake.lock().expect("lock").keep_failing(&t1);
            step(&mut part, &mut w, after);
            step(&mut part, &mut w, after + 2_000);
            step(&mut part, &mut w, after + 30_000);
            handle.fake.lock().expect("lock").keep_working(&t1);
            step(&mut part, &mut w, 2 * after);
            assert_eq!(reminders(&handle), 2, "{case}");
            // The time runs from here, not from the reminder a second before.
            let first_seen = 2 * after + 1_000;
            step(&mut part, &mut w, first_seen);
            assert!(working(&w.agents[&author].state), "{case}");
            handle
                .fake
                .lock()
                .expect("lock")
                .end_in_error(&t1, OVERLOADED);
            let error = first_seen + worked;
            let polled = step(&mut part, &mut w, error);
            assert_eq!(ended(&polled, &author), !recovers, "{case}: {polled:?}");
            if recovers {
                step(&mut part, &mut w, error + after - 1);
                assert_eq!(reminders(&handle), 2, "{case}");
                step(&mut part, &mut w, error + after);
                assert_eq!(reminders(&handle), 3, "{case}");
            }
        }
    }

    #[test]
    fn an_errored_agent_whose_reminder_ends_between_turns_starts_the_count_again() {
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = overloaded_author(0);
        handle.fake.lock().expect("lock").keep_failing(&t1);
        let minute = |n: u64| n * 60_000;
        step(&mut part, &mut w, minute(10));
        step(&mut part, &mut w, minute(11));
        assert!(working(&w.agents[&author].state));
        // This time the turn gets through and ends.
        handle.fake.lock().expect("lock").unblock(&t1);
        step(&mut part, &mut w, minute(15));
        assert_eq!(w.agents[&author].state, AgentState::Idle);
        for (at, n) in [(24, 1), (25, 2), (30, 2), (34, 2), (35, 3)] {
            let polled = step(&mut part, &mut w, minute(at));
            assert!(!ended(&polled, &author), "stopped at {at} min");
            assert_eq!(reminders(&handle), n, "at {at} min");
        }
        let polled = step(&mut part, &mut w, minute(44));
        assert!(!ended(&polled, &author), "{polled:?}");
        let stopped = step(&mut part, &mut w, minute(45));
        assert!(ended(&stopped, &author), "{stopped:?}");
    }

    #[test]
    fn an_errored_agent_whose_reminder_ends_on_a_limit_waits_for_the_reset() {
        let t1 = TerminalId("t1".into());
        let at = |hh, mm| nov27(hh, mm);
        let (mut w, mut part, handle, author) = overloaded_author(at(16, 0));
        handle.fake.lock().expect("lock").keep_failing(&t1);
        step(&mut part, &mut w, at(16, 10));
        assert_eq!(reminders(&handle), 1);
        step(&mut part, &mut w, at(16, 11));
        assert!(working(&w.agents[&author].state));
        handle.fake.lock().expect("lock").end_in_error(&t1, LIMIT);
        step(&mut part, &mut w, at(16, 12));
        for (hh, mm) in [(16, 20), (16, 30), (16, 42), (17, 40)] {
            assert_eq!(step(&mut part, &mut w, at(hh, mm)), vec![], "{hh}:{mm:02}");
        }
        assert_eq!((continues(&handle.fake), reminders(&handle)), (0, 1));
        assert_eq!(
            step(&mut part, &mut w, at(17, 41)),
            vec![Event::Continued {
                agent: author.clone(),
                resets_ms: at(17, 40)
            }]
        );
        step(&mut part, &mut w, at(17, 41) + 2_000);
        handle
            .fake
            .lock()
            .expect("lock")
            .end_in_error(&t1, OVERLOADED);
        step(&mut part, &mut w, at(17, 42));
        for (hh, mm, n) in [(17, 51, 1), (17, 52, 2), (18, 2, 3), (18, 11, 3)] {
            let polled = step(&mut part, &mut w, at(hh, mm));
            assert!(!ended(&polled, &author), "stopped at {hh}:{mm:02}");
            assert_eq!(reminders(&handle), n, "at {hh}:{mm:02}");
            // The reminder's turn: working, then the 529 again.
            step(&mut part, &mut w, at(hh, mm) + 2_000);
            step(&mut part, &mut w, at(hh, mm) + 4_000);
        }
        let stopped = step(&mut part, &mut w, at(18, 12));
        assert!(ended(&stopped, &author), "{stopped:?}");
    }

    #[test]
    fn an_errored_agent_that_stops_at_a_prompt_starts_the_count_again() {
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = overloaded_author(0);
        handle.fake.lock().expect("lock").keep_failing(&t1);
        let minute = |n: u64| n * 60_000;
        step(&mut part, &mut w, minute(10));
        step(&mut part, &mut w, minute(11));
        assert!(working(&w.agents[&author].state));
        handle.fake.lock().expect("lock").block(&t1, "permission");
        step(&mut part, &mut w, minute(12));
        assert!(matches!(
            w.agents[&author].state,
            AgentState::Blocked { .. }
        ));
        handle
            .fake
            .lock()
            .expect("lock")
            .end_in_error(&t1, OVERLOADED);
        step(&mut part, &mut w, minute(15));
        for (at, n) in [(20, 1), (24, 1), (25, 2), (35, 3), (44, 3)] {
            let polled = step(&mut part, &mut w, minute(at));
            assert!(!ended(&polled, &author), "stopped at {at} min");
            assert_eq!(reminders(&handle), n, "at {at} min");
            step(&mut part, &mut w, minute(at) + 2_000);
            step(&mut part, &mut w, minute(at) + 4_000);
        }
        let stopped = step(&mut part, &mut w, minute(45));
        assert!(ended(&stopped, &author), "{stopped:?}");
    }

    #[test]
    fn an_errored_agent_its_issue_stops_waiting_on_starts_the_count_again() {
        use temet_work_core::model::Stage;
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = overloaded_author(0);
        handle.fake.lock().expect("lock").keep_failing(&t1);
        let minute = |n: u64| n * 60_000;
        step(&mut part, &mut w, minute(10));
        step(&mut part, &mut w, minute(11));
        assert!(working(&w.agents[&author].state));
        // The issue goes to review while the turn runs, and comes back later.
        at_stage(&mut w, Stage::InReview { round: 1 });
        step(&mut part, &mut w, minute(12));
        at_stage(&mut w, Stage::Fixing { round: 1 });
        w.issues
            .get_mut(&IssueId::new("ankh", "1"))
            .expect("there")
            .told = true;
        assert!(w.waiting_on(&w.agents[&author]));
        step(&mut part, &mut w, minute(15));
        step(&mut part, &mut w, minute(20));
        assert_eq!(reminders(&handle), 1, "not on the old count");
        step(&mut part, &mut w, minute(25));
        assert_eq!(reminders(&handle), 2);
    }

    /// The stops the shared fake was given, failed or not.
    fn stops(handle: &Shared) -> usize {
        let calls = handle.fake.lock().expect("fake").calls();
        calls
            .lock()
            .expect("calls")
            .iter()
            .filter(|c| matches!(c, fake::Call::Stop(_)))
            .count()
    }

    /// An author on ankh#1, waited on, started at `now_ms` and seen working on
    /// its brief, in a turn that stays working.
    fn working_author(now_ms: u64) -> (World, Runners, Shared, AgentId) {
        let t1 = TerminalId("t1".into());
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let author = run(&mut part, &mut w, &["claude"]);
        {
            let mut fake = handle.fake.lock().expect("lock");
            fake.keep_working(&t1);
            fake.send(&t1, "Mend the clock", false).expect("send");
        }
        step(&mut part, &mut w, now_ms);
        assert!(working(&w.agents[&author].state));
        (w, part, handle, author)
    }

    const FIX: &str = "the clock still stops at midnight";

    /// Asks for a fix to ankh#1 and applies what the part reports.
    fn request_fix(part: &mut Runners, w: &mut World) {
        let fix = Effect::RequestFix {
            issue: IssueId::new("ankh", "1"),
            points: vec![FIX.into()],
        };
        let Outcome::Done { events, .. } = part.carry_out(&fix, w) else {
            panic!("not done");
        };
        apply(w, &events);
    }

    /// Whether the loop now starts a new author on ankh#1.
    fn starts_an_author(w: &World) -> bool {
        w.missing().start.iter().any(|e| {
            matches!(e, Effect::StartAgent { role: Role::Author, issue, .. }
                     if *issue == IssueId::new("ankh", "1"))
        })
    }

    #[test]
    fn an_agent_whose_terminal_reports_no_status_yet_keeps_its_state_and_its_terminal() {
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = working_author(0);
        let before = w.agents[&author].state.clone();
        handle.fake.lock().expect("lock").no_status(&t1);
        for minute in 1..=40 {
            let polled = step(&mut part, &mut w, minute * 60_000);
            assert_eq!(polled, vec![], "at {minute} min");
        }
        assert_eq!(w.agents[&author].state, before);
        assert_eq!(stops(&handle), 0);
        assert!(!starts_an_author(&w), "{:?}", w.missing());
    }

    /// Loses `author`'s terminal t1 to a shell, polls at `at_ms`, then every
    /// minute for 70 minutes, and checks the author ended in that first poll,
    /// a new author is due, the terminal was stopped once and nothing was
    /// typed into the shell.
    fn lose_to_a_shell_and_watch(
        mut w: World,
        mut part: Runners,
        handle: &Shared,
        author: &AgentId,
        at_ms: u64,
    ) {
        let t1 = TerminalId("t1".into());
        let before = screen_of(&handle.fake);
        handle.fake.lock().expect("lock").lose_to_shell(&t1);
        let polled = step(&mut part, &mut w, at_ms);
        assert_eq!(
            polled,
            vec![Event::AgentChanged {
                agent: author.clone(),
                state: AgentState::Ended,
                lost: true,
            }]
        );
        assert!(starts_an_author(&w), "{:?}", w.missing());
        assert_eq!(stops(handle), 1);
        assert_eq!(
            part.waiting(),
            0,
            "a new author hears the fixes in its brief"
        );
        for minute in 1..=70 {
            step(&mut part, &mut w, at_ms + minute * 60_000);
        }
        assert_eq!(stops(handle), 1, "stopped once");
        assert_eq!(screen_of(&handle.fake), before, "typed into the shell");
    }

    #[test]
    fn an_author_working_whose_terminal_comes_back_as_a_shell_is_ended_replaced_and_removed() {
        let (mut w, mut part, handle, author) = working_author(0);
        request_fix(&mut part, &mut w);
        assert_eq!(part.waiting(), 1, "the fix waits for the turn to end");
        lose_to_a_shell_and_watch(w, part, &handle, &author, 60_000);
    }

    #[test]
    fn an_idle_or_limited_author_whose_terminal_comes_back_as_a_shell_is_neither_reminded_nor_continued()
     {
        // Idle: it would be reminded at ten minutes.
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let author = run(&mut part, &mut w, &["claude"]);
        step(&mut part, &mut w, 0);
        assert_eq!(w.agents[&author].state, AgentState::Idle);
        lose_to_a_shell_and_watch(w, part, &handle, &author, 60_000);

        // At its usage limit: it would be continued at 17:41.
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        let author = run(&mut part, &mut w, &["claude"]);
        handle
            .fake
            .lock()
            .expect("lock")
            .end_in_error(&TerminalId("t1".into()), LIMIT);
        step(&mut part, &mut w, nov27(16, 50));
        assert!(limit_reset(&w.agents[&author], &BTreeMap::new()).is_some());
        lose_to_a_shell_and_watch(w, part, &handle, &author, nov27(16, 51));
    }

    #[test]
    fn the_shell_is_removed_a_minute_after_a_stop_that_failed_and_not_sooner() {
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = working_author(0);
        let before = screen_of(&handle.fake);
        {
            let mut fake = handle.fake.lock().expect("lock");
            fake.lose_to_shell(&t1);
            // herdr-core will not remove a terminal it is still recovering.
            fake.fail_next_stop(RunnerError::Other("t1 is still live after kill".into()));
        }
        let polled = step(&mut part, &mut w, 1_000);
        assert!(ended_lost(&polled, &author), "{polled:?}");
        assert_eq!(stops(&handle), 1);
        for at in [2_000, 30_000, 60_999] {
            step(&mut part, &mut w, at);
            assert_eq!(stops(&handle), 1, "tried again at {at} ms");
        }
        step(&mut part, &mut w, 61_000);
        assert_eq!(stops(&handle), 2, "a minute after the first");
        for at in [62_000, 121_000, 181_000, 600_000] {
            step(&mut part, &mut w, at);
        }
        assert_eq!(stops(&handle), 2, "stopped");
        assert_eq!(screen_of(&handle.fake), before, "typed into the shell");
    }

    #[test]
    fn the_shell_is_removed_after_refusals_each_tried_a_minute_after_the_last() {
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = working_author(0);
        let before = screen_of(&handle.fake);
        {
            let mut fake = handle.fake.lock().expect("lock");
            fake.lose_to_shell(&t1);
            // A core that refuses three times in a row, then removes it.
            for _ in 0..3 {
                fake.fail_next_stop(RunnerError::Other("t1 is still live after kill".into()));
            }
        }
        // The first try at 1 s, each later one STOP_AGAIN_MS after the last.
        let tries = [1_000, 61_000, 121_000, 181_000];
        assert!(
            tries
                .windows(2)
                .all(|pair| pair[1] - pair[0] == STOP_AGAIN_MS)
        );
        // A poll every 10 s for five minutes, and one just before each retry.
        let mut polls: Vec<u64> = (0..30).map(|k| 1_000 + k * 10_000).collect();
        polls.extend(tries[1..].iter().map(|t| t - 1));
        polls.sort_unstable();
        for at in polls {
            let polled = step(&mut part, &mut w, at);
            if at == tries[0] {
                assert!(ended_lost(&polled, &author), "{polled:?}");
            } else {
                assert_eq!(polled, vec![], "at {at} ms");
            }
            let tried = tries.iter().filter(|t| at >= **t).count();
            assert_eq!(stops(&handle), tried, "at {at} ms");
        }
        assert_eq!(screen_of(&handle.fake), before, "typed into the shell");
    }

    #[test]
    fn the_shell_is_removed_once_a_stop_is_answered_with_the_terminal_gone_or_unknown() {
        let t1 = TerminalId("t1".into());
        for answer in [RunnerError::Gone, RunnerError::Unknown(t1.clone())] {
            let (mut w, mut part, handle, _) = working_author(0);
            {
                let mut fake = handle.fake.lock().expect("lock");
                fake.lose_to_shell(&t1);
                fake.fail_next_stop(answer.clone());
            }
            for at in [1_000, 30_000, 61_000, 121_000, 600_000] {
                step(&mut part, &mut w, at);
            }
            assert_eq!(stops(&handle), 1, "{answer:?} is not tried again");
        }
    }

    #[test]
    fn the_shell_is_removed_once_for_an_author_that_ended_before_a_restart() {
        let t1 = TerminalId("t1".into());
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut before, handle) = shared();
        let author = run(&mut before, &mut w, &["claude"]);
        // The daemon before this one saw the program lost and ended the author,
        // then restarted before it stopped the shell.
        handle.fake.lock().expect("lock").lose_to_shell(&t1);
        handle.fake.lock().expect("lock").updates();
        w.apply(&Event::AgentChanged {
            agent: author.clone(),
            state: AgentState::Ended,
            lost: true,
        });
        drop(before);
        // This daemon connects, and the runner lists the terminal, lost.
        let mut part = Runners::default();
        part.add(Box::new(handle.clone()));
        handle.fake.lock().expect("lock").reconnect();
        assert_eq!(step(&mut part, &mut w, 0), vec![], "it has ended already");
        assert_eq!(stops(&handle), 1);
        for minute in 1..=5 {
            handle.fake.lock().expect("lock").reconnect();
            assert_eq!(step(&mut part, &mut w, minute * 60_000), vec![]);
        }
        assert_eq!(stops(&handle), 1, "stopped once");
    }

    #[test]
    fn an_author_whose_program_ends_by_itself_is_ended_not_lost_and_its_terminal_removed() {
        let t1 = TerminalId("t1".into());
        let (mut w, mut part, handle, author) = working_author(0);
        handle.fake.lock().expect("lock").end_program(&t1);
        assert_eq!(
            step(&mut part, &mut w, 60_000),
            vec![Event::AgentChanged {
                agent: author.clone(),
                state: AgentState::Ended,
                lost: false,
            }]
        );
        assert!(w.agents[&author].ended_waited_on, "it counts");
        assert!(starts_an_author(&w), "{:?}", w.missing());
        assert_eq!(stops(&handle), 1);
    }

    #[test]
    fn an_author_lost_to_a_shell_three_times_still_leaves_a_fourth_to_start() {
        let mut w = world();
        at_stage(&mut w, temet_work_core::model::Stage::Working);
        let (mut part, handle) = shared();
        for n in 1..=3 {
            let author = run(&mut part, &mut w, &["claude"]);
            step(&mut part, &mut w, n * 60_000);
            let terminal = TerminalId(format!("t{n}"));
            handle.fake.lock().expect("lock").lose_to_shell(&terminal);
            let polled = step(&mut part, &mut w, n * 60_000 + 1_000);
            assert!(ended_lost(&polled, &author), "{polled:?}");
        }
        assert!(starts_an_author(&w), "{:?}", w.missing());
        assert_eq!(w.missing().given_up, Vec::<String>::new());
    }

    #[test]
    fn an_author_lost_to_a_shell_three_times_with_the_daemon_restarting_each_time_leaves_a_fourth()
    {
        // As on a reboot: herdr-core and the daemon both come back, and the
        // daemon has only its log.
        let mut log = vec![
            Event::GroupCreated {
                group: GroupId::new("watch"),
                name: "Night watch".into(),
                projects: vec![ProjectId::new("ankh")],
                areas: vec![],
                milestones: vec![MilestoneId::new("m1")],
                refining_at_once: 1,
                model: None,
                queue_low: None,
                investigate_every: None,
            },
            Event::FolderAdded {
                folder: FolderId::new("f1"),
                project: ProjectId::new("ankh"),
                machine: MachineId::new(LOCAL),
                path: ".".into(),
            },
            Event::IssueSynced {
                issue: IssueId::new("ankh", "1"),
                title: "Mend the clock".into(),
                milestone: Some(MilestoneId::new("m1")),
                depends_on: vec![],
                order: 1,
                closed: false,
            },
            Event::PlanWritten {
                issue: IssueId::new("ankh", "1"),
                plan: temet_work_core::model::Plan::default(),
            },
        ];
        let handle = Shared::default();
        let daemon = || {
            let mut part = Runners::default();
            part.add(Box::new(handle.clone()));
            part
        };
        for n in 1..=3 {
            let mut w = World::replay(&log).0;
            let mut part = daemon();
            log.extend(run_events(&mut part, &mut w, &["claude"]));
            log.extend(step(&mut part, &mut w, 0));
            drop(part);
            let author = AgentId::new(format!("author-{n}"));
            {
                // The reboot: nothing reaches the daemon that was running.
                let mut fake = handle.fake.lock().expect("lock");
                fake.lose_to_shell(&TerminalId(format!("t{n}")));
                fake.updates();
                fake.reconnect();
            }
            let mut w = World::replay(&log).0;
            assert!(w.waiting_on(&w.agents[&author]), "running in the log");
            let polled = step(&mut daemon(), &mut w, 0);
            assert_eq!(
                polled,
                vec![Event::AgentChanged {
                    agent: author,
                    state: AgentState::Ended,
                    lost: true,
                }]
            );
            log.extend(polled);
        }
        let w = World::replay(&log).0;
        assert!(starts_an_author(&w), "{:?}", w.missing());
        assert_eq!(w.missing().given_up, Vec::<String>::new());
    }
}
