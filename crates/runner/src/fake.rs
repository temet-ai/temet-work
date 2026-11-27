//! A deterministic runner for tests: terminals live in memory, a send echoes its
//! text onto the screen and runs one turn, and tests can put a terminal at a
//! prompt or end it.

use std::collections::BTreeMap;

use crate::contract::Harness;
use crate::{Program, Runner, RunnerError, Start, Status, TerminalId, Update};

#[derive(Clone, Debug)]
struct Term {
    labels: BTreeMap<String, String>,
    status: Status,
    screen: Vec<String>,
}

/// An in-memory runner.
#[derive(Debug)]
pub struct FakeRunner {
    id: &'static str,
    next: u64,
    terms: BTreeMap<TerminalId, Term>,
    pending: Vec<Update>,
    starts: Starts,
    calls: Calls,
    exec_error: Option<RunnerError>,
}

/// Every [`Start`] a fake runner was given, shared so a test can read them after
/// the runner has been boxed into a part.
pub type Starts = std::sync::Arc<std::sync::Mutex<Vec<Start>>>;

/// A call a fake runner was given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    /// A program run to its end on a machine (this one when `None`).
    Exec {
        /// The machine.
        machine: Option<String>,
        /// The program.
        program: Program,
    },
    /// A terminal started.
    Start(Start),
}

/// Every `exec` and `start` a fake runner was given, in order, shared like [`Starts`].
pub type Calls = std::sync::Arc<std::sync::Mutex<Vec<Call>>>;

impl Default for FakeRunner {
    fn default() -> Self {
        Self::named("fake")
    }
}

impl FakeRunner {
    /// A fake runner with the given id.
    pub fn named(id: &'static str) -> Self {
        Self {
            id,
            next: 1,
            terms: BTreeMap::new(),
            pending: Vec::new(),
            starts: Starts::default(),
            calls: Calls::default(),
            exec_error: None,
        }
    }

    /// The starts this runner is given, in order.
    pub fn starts(&self) -> Starts {
        self.starts.clone()
    }

    /// The `exec` and `start` calls this runner is given, in order.
    pub fn calls(&self) -> Calls {
        self.calls.clone()
    }

    /// Makes every `exec` fail with `error`, as a program that exits nonzero
    /// or a machine that cannot be reached does.
    #[must_use]
    pub fn exec_fails(mut self, error: RunnerError) -> Self {
        self.exec_error = Some(error);
        self
    }

    fn update(&mut self, terminal: &TerminalId, doing: Option<String>) {
        if let Some(t) = self.terms.get(terminal) {
            self.pending.push(Update {
                terminal: terminal.clone(),
                labels: t.labels.clone(),
                status: t.status.clone(),
                doing,
            });
        }
    }

    /// Puts a terminal at a prompt of the given kind.
    pub fn block(&mut self, terminal: &TerminalId, kind: &str) {
        if let Some(t) = self.terms.get_mut(terminal) {
            t.status = Status::Blocked { kind: kind.into() };
            self.update(terminal, None);
        }
    }

    /// Lets a terminal at a prompt go on: its prompt was answered.
    pub fn unblock(&mut self, terminal: &TerminalId) {
        if let Some(t) = self.terms.get_mut(terminal) {
            t.status = Status::Between;
            self.update(terminal, None);
        }
    }

    /// Ends a terminal's turn in an error that shows `line` on its screen, as a
    /// session at its usage limit does. The status carries no message, so the
    /// line is read from the screen. Text sent after it starts a new turn.
    pub fn end_in_error(&mut self, terminal: &TerminalId, line: &str) {
        if let Some(t) = self.terms.get_mut(terminal) {
            t.screen.push(line.to_string());
            t.status = Status::Error { message: None };
            self.update(terminal, None);
        }
    }

    /// Reports every terminal that has not ended once more, with its labels and
    /// status, as a runner does when it connects again after a restart.
    pub fn reconnect(&mut self) {
        let live: Vec<TerminalId> = self
            .terms
            .iter()
            .filter(|(_, t)| t.status != Status::Gone)
            .map(|(id, _)| id.clone())
            .collect();
        for id in live {
            self.update(&id, None);
        }
    }
}

impl Runner for FakeRunner {
    fn id(&self) -> &'static str {
        self.id
    }

    fn is_default(&self) -> bool {
        true
    }

    fn start(&mut self, start: &Start) -> Result<TerminalId, RunnerError> {
        if let Ok(mut starts) = self.starts.lock() {
            starts.push(start.clone());
        }
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(Call::Start(start.clone()));
        }
        // A terminal on another machine is addressed `machine/t1`, as herdr-core does.
        let id = TerminalId(match &start.machine {
            Some(m) => format!("{m}/t{}", self.next),
            None => format!("t{}", self.next),
        });
        self.next += 1;
        let status = if start.program.argv.first().map(String::as_str) == Some(PROMPT_PROGRAM) {
            Status::Blocked {
                kind: "question".into(),
            }
        } else {
            Status::Between
        };
        self.terms.insert(
            id.clone(),
            Term {
                labels: start.labels.clone(),
                status,
                screen: vec![format!("$ {}", start.program.argv.join(" "))],
            },
        );
        self.update(&id, None);
        Ok(id)
    }

    /// Runs nothing: it records the call, and fails as told to by
    /// [`FakeRunner::exec_fails`], or when the program is [`FAIL_PROGRAM`].
    fn exec(&mut self, machine: Option<&str>, program: &Program) -> Result<(), RunnerError> {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(Call::Exec {
                machine: machine.map(String::from),
                program: program.clone(),
            });
        }
        if let Some(e) = &self.exec_error {
            return Err(e.clone());
        }
        if program.argv.first().map(String::as_str) == Some(FAIL_PROGRAM) {
            return Err(RunnerError::Other(format!("{FAIL_PROGRAM} exited 1")));
        }
        Ok(())
    }

    fn send(
        &mut self,
        terminal: &TerminalId,
        text: &str,
        only_if_idle: bool,
    ) -> Result<(), RunnerError> {
        let t = self
            .terms
            .get_mut(terminal)
            .ok_or_else(|| RunnerError::Unknown(terminal.clone()))?;
        match t.status {
            Status::Gone => return Err(RunnerError::Gone),
            Status::Blocked { .. } => return Err(RunnerError::Blocked),
            Status::Working if only_if_idle => return Err(RunnerError::Busy),
            _ => {}
        }
        t.screen.push(text.to_string());
        t.status = Status::Working;
        self.update(terminal, Some(format!("reading: {text}")));
        if let Some(t) = self.terms.get_mut(terminal) {
            t.status = Status::Between;
        }
        self.update(terminal, None);
        Ok(())
    }

    fn screen(&mut self, terminal: &TerminalId) -> Result<String, RunnerError> {
        self.terms
            .get(terminal)
            .map(|t| t.screen.join("\n"))
            .ok_or_else(|| RunnerError::Unknown(terminal.clone()))
    }

    fn stop(&mut self, terminal: &TerminalId) -> Result<(), RunnerError> {
        if !self.terms.contains_key(terminal) {
            return Err(RunnerError::Unknown(terminal.clone()));
        }
        if let Some(t) = self.terms.get_mut(terminal) {
            t.status = Status::Gone;
        }
        self.update(terminal, None);
        Ok(())
    }

    fn updates(&mut self) -> Vec<Update> {
        std::mem::take(&mut self.pending)
    }
}

/// A program name the fake starts already stopped at a prompt.
pub const PROMPT_PROGRAM: &str = "fake-prompt";

/// A program the fake runs to a nonzero exit, as `false` does everywhere.
pub const FAIL_PROGRAM: &str = "false";

/// The contract harness for [`FakeRunner`].
pub struct FakeHarness;

impl Harness for FakeHarness {
    fn folder(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(".")
    }

    fn echo_program(&self) -> Vec<String> {
        vec!["cat".into()]
    }

    fn prompt_program(&self) -> Option<Vec<String>> {
        Some(vec![PROMPT_PROGRAM.into()])
    }

    fn machine(&self) -> Option<String> {
        Some("elsewhere".into())
    }

    fn reached(&self, machine: &str, terminal: &TerminalId) -> bool {
        terminal.0.starts_with(&format!("{machine}/"))
    }

    fn settle(&mut self, _runner: &mut dyn Runner) {}
}
