//! Claude Code sessions.

use temet_work_runtime::Runtime;

/// The claude adapter.
#[derive(Default)]
pub struct Claude;

impl Runtime for Claude {
    fn id(&self) -> &'static str {
        "claude"
    }
}
