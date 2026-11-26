//! herdr-core terminals, reached over its JSON-RPC socket.

use temet_work_runner::Runner;

/// The herdr adapter.
#[derive(Default)]
pub struct Herdr;

impl Runner for Herdr {
    fn id(&self) -> &'static str {
        "herdr"
    }
}
