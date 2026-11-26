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

use temet_work_core::Part;

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
#[derive(Default)]
pub struct Runners {
    adapters: Vec<Box<dyn Runner>>,
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

impl Part for Runners {
    fn id(&self) -> &'static str {
        "runner"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeRunner;

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
