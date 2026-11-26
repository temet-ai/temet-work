//! Wires the compiled-in parts together. A part is one feature in Cargo.toml,
//! one optional dependency and one `#[cfg]` block here; an adapter adds itself
//! to its family inside the family's block. `tests/parts.rs` checks all three.

use temet_work_core::Part;

/// The parts compiled into this build, in a fixed order: families first, then
/// services.
pub fn parts() -> Vec<Box<dyn Part>> {
    #[allow(unused_mut)]
    let mut parts: Vec<Box<dyn Part>> = Vec::new();
    #[cfg(feature = "tracker")]
    {
        #[allow(unused_mut)]
        let mut family = temet_work_tracker::Trackers::default();
        #[cfg(feature = "tracker-github")]
        family.add(Box::new(temet_work_tracker_github::GitHub));
        parts.push(Box::new(family));
    }
    #[cfg(feature = "runner")]
    {
        #[allow(unused_mut)]
        let mut family = temet_work_runner::Runners::default();
        #[cfg(feature = "runner-herdr")]
        family.add(Box::new(temet_work_runner_herdr::Herdr::default()));
        parts.push(Box::new(family));
    }
    #[cfg(feature = "runtime")]
    {
        #[allow(unused_mut)]
        let mut family = temet_work_runtime::Runtimes::default();
        #[cfg(feature = "runtime-claude")]
        family.add(Box::new(temet_work_runtime_claude::Claude));
        parts.push(Box::new(family));
    }
    #[cfg(feature = "store")]
    parts.push(Box::new(temet_work_store::Store));
    #[cfg(feature = "style")]
    parts.push(Box::new(temet_work_style::Style));
    #[cfg(feature = "mike")]
    parts.push(Box::new(temet_work_mike::Mike));
    parts
}
