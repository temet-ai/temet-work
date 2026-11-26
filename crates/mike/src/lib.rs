//! Optional: reads capture, corrected turns and lessons from a mike proxy, as input to the improvement loop.

use temet_work_core::Part;

/// The mike part.
#[derive(Default)]
pub struct Mike;

impl Part for Mike {
    fn id(&self) -> &'static str {
        "mike"
    }
}
