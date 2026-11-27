//! Wires the compiled-in parts together. A part is one feature in Cargo.toml,
//! one optional dependency and one `#[cfg]` block here; an adapter adds itself
//! to its family inside the family's block. `tests/parts.rs` checks all three.

pub mod cli;
pub mod config;
pub mod mcp;
pub mod socket;

use std::path::Path;

use temet_work_core::{Journal, Part, World};

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
        family.add(Box::new(temet_work_tracker_github::GitHub::default()));
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
        #[cfg(feature = "runtime-codex")]
        family.add(Box::new(temet_work_runtime_codex::Codex));
        parts.push(Box::new(family));
    }
    #[cfg(feature = "codehost")]
    {
        #[allow(unused_mut)]
        let mut family = temet_work_codehost::CodeHosts::default();
        #[cfg(feature = "codehost-github")]
        family.add(Box::new(temet_work_codehost_github::GitHub::default()));
        parts.push(Box::new(family));
    }
    #[cfg(feature = "deploy")]
    parts.push(Box::new(temet_work_deploy::Deploys::default()));
    #[cfg(feature = "store")]
    parts.push(Box::new(temet_work_store::Store));
    #[cfg(feature = "style")]
    parts.push(Box::new(temet_work_style::Style));
    #[cfg(feature = "mike")]
    parts.push(Box::new(temet_work_mike::Mike));
    parts
}

/// Opens the event log at `path` and rebuilds the state from it. A build
/// without the store keeps events in memory, so nothing survives a restart.
#[cfg(feature = "store")]
pub fn journal(path: &Path) -> Result<(Box<dyn Journal>, World), String> {
    let (log, entries) = temet_work_store::Log::open(path).map_err(|e| e.to_string())?;
    Ok((Box::new(log), temet_work_store::replay(&entries)))
}

/// Opens the event log at `path` and rebuilds the state from it. A build
/// without the store keeps events in memory, so nothing survives a restart.
#[cfg(not(feature = "store"))]
pub fn journal(path: &Path) -> Result<(Box<dyn Journal>, World), String> {
    let _ = path;
    Ok((
        Box::new(temet_work_core::host::Memory::default()),
        World::default(),
    ))
}
