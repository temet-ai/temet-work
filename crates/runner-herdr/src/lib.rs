//! herdr-core terminals, reached over its JSON-RPC socket.
//!
//! Not built yet: every call answers [`RunnerError::NotBuilt`]. What the client
//! will rely on, and what to avoid, is in `docs/findings/herdr-core.md`: one
//! subscription filtered by labels for status, labels set at start, `blocked`
//! refusing a send, a connection per wait.

use temet_work_runner::{Runner, RunnerError, Start, TerminalId, Update};

/// The herdr adapter.
#[derive(Default)]
pub struct Herdr;

fn not_built() -> RunnerError {
    RunnerError::NotBuilt("the herdr-core socket client".into())
}

impl Runner for Herdr {
    fn id(&self) -> &'static str {
        "herdr"
    }

    fn start(&mut self, _start: &Start) -> Result<TerminalId, RunnerError> {
        Err(not_built())
    }

    fn send(
        &mut self,
        _terminal: &TerminalId,
        _text: &str,
        _only_if_idle: bool,
    ) -> Result<(), RunnerError> {
        Err(not_built())
    }

    fn screen(&mut self, _terminal: &TerminalId) -> Result<String, RunnerError> {
        Err(not_built())
    }

    fn stop(&mut self, _terminal: &TerminalId) -> Result<(), RunnerError> {
        Err(not_built())
    }

    fn updates(&mut self) -> Vec<Update> {
        Vec::new()
    }
}
