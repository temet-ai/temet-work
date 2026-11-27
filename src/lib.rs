//! Wires the compiled-in parts together. A part is one feature in Cargo.toml,
//! one optional dependency and one `#[cfg]` block here; an adapter adds itself
//! to its family inside the family's block. `tests/parts.rs` checks all three.

pub mod cli;
pub mod config;
pub mod mcp;
pub mod peer;
pub mod socket;

use std::path::{Path, PathBuf};

use temet_work_core::{Journal, Part, World};

/// The parts compiled into this build, in a fixed order: families first, then
/// services.
pub fn parts() -> Vec<Box<dyn Part>> {
    parts_with(None, None)
}

/// The id of this build: the commit it was built from, `-dirty` when the
/// checkout had changes, or `unknown` outside a git checkout.
pub const BUILD: &str = env!("TEMET_WORK_BUILD");

/// The parts, with `daemon_host`: this machine as other machines reach it over
/// SSH; and `handoff`, the file a deploy that restarts the daemon records itself in.
#[cfg_attr(not(feature = "runner"), allow(unused_variables))]
#[cfg_attr(not(feature = "deploy"), allow(unused_variables))]
pub fn parts_with(daemon_host: Option<String>, handoff: Option<PathBuf>) -> Vec<Box<dyn Part>> {
    #[allow(unused_mut)]
    let mut parts: Vec<Box<dyn Part>> = Vec::new();
    #[cfg(feature = "tracker")]
    {
        #[allow(unused_mut)]
        let mut family = temet_work_tracker::Trackers::default();
        #[cfg(feature = "tracker-github")]
        family.add(Box::new(temet_work_tracker_github::GitHub::default()));
        #[cfg(feature = "tracker-jira")]
        family.add(Box::new(temet_work_tracker_jira::Jira::default()));
        parts.push(Box::new(family));
    }
    #[cfg(feature = "runner")]
    {
        #[allow(unused_mut)]
        let mut family = temet_work_runner::Runners::default().daemon_host(daemon_host);
        #[cfg(feature = "runner-herdr")]
        family.add(Box::new(temet_work_runner_herdr::Herdr::default()));
        parts.push(Box::new(family));
    }
    #[cfg(feature = "runtime")]
    parts.push(Box::new(runtimes()));
    #[cfg(feature = "codehost")]
    {
        #[allow(unused_mut)]
        let mut family = temet_work_codehost::CodeHosts::default();
        #[cfg(feature = "codehost-github")]
        family.add(Box::new(temet_work_codehost_github::GitHub::default()));
        #[cfg(feature = "codehost-gitlab")]
        family.add(Box::new(temet_work_codehost_gitlab::GitLab::default()));
        parts.push(Box::new(family));
    }
    #[cfg(feature = "deploy")]
    {
        let mut deploys = temet_work_deploy::Deploys::default();
        if let Some(file) = handoff {
            deploys = deploys.handoff(file, BUILD);
        }
        parts.push(Box::new(deploys));
    }
    #[cfg(feature = "store")]
    parts.push(Box::new(temet_work_store::Store));
    #[cfg(feature = "style")]
    parts.push(Box::new(temet_work_style::Style));
    #[cfg(feature = "mike")]
    parts.push(Box::new(temet_work_mike::Mike));
    parts
}

/// The runtime family, with each runtime compiled into this build.
#[cfg(feature = "runtime")]
fn runtimes() -> temet_work_runtime::Runtimes {
    #[allow(unused_mut)]
    let mut family = temet_work_runtime::Runtimes::default();
    #[cfg(feature = "runtime-claude")]
    family.add(Box::new(temet_work_runtime_claude::Claude));
    #[cfg(feature = "runtime-codex")]
    family.add(Box::new(temet_work_runtime_codex::Codex));
    family
}

/// Marks `folder` trusted for the runtime `runtime` in its user files on this
/// machine, as the daemon does before an agent starts in a folder on its own
/// machine. `temet-work trust-folder` runs this; the daemon runs that on the
/// machine a folder on another machine is on.
#[cfg_attr(not(feature = "runtime"), allow(unused_variables))]
pub fn trust_folder(runtime: &str, folder: &Path) -> Result<(), String> {
    #[cfg(feature = "runtime")]
    {
        runtimes().trust(runtime, folder)
    }
    #[cfg(not(feature = "runtime"))]
    {
        Err(format!(
            "no runtime \"{runtime}\" in this build: it was built without runtimes"
        ))
    }
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
