//! What temet-work keeps between runs: its state and the event log.

use temet_work_core::Part;

/// The store part.
#[derive(Default)]
pub struct Store;

impl Part for Store {
    fn id(&self) -> &'static str {
        "store"
    }
}
