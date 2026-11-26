//! The style guide: tokens, the generated palette, the ban list, and the check used when reviewing UI work.

use temet_work_core::Part;

/// The style part.
#[derive(Default)]
pub struct Style;

impl Part for Style {
    fn id(&self) -> &'static str {
        "style"
    }
}
