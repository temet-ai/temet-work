//! Wires the compiled-in parts together. A part is one feature in Cargo.toml,
//! one optional dependency and one `#[cfg]` block here; an adapter adds itself
//! to its family inside the family's block. `tests/parts.rs` checks all three.

pub mod cli;
pub mod config;
pub mod mcp;
pub mod peer;
pub mod socket;

use std::path::Path;

use temet_work_core::host::Host;
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
/// SSH; and `log`, the daemon's event log, beside which a deploy that restarts
/// the daemon records itself (`deploy-handoff`) and the code-host family keeps
/// its one copy of each repository (`copies`).
#[cfg_attr(not(feature = "runner"), allow(unused_variables))]
#[cfg_attr(
    not(any(feature = "deploy", feature = "codehost")),
    allow(unused_variables)
)]
pub fn parts_with(daemon_host: Option<String>, log: Option<&Path>) -> Vec<Box<dyn Part>> {
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
        family.add(Box::new(
            temet_work_runner_herdr::Herdr::default().build(BUILD),
        ));
        parts.push(Box::new(family));
    }
    #[cfg(feature = "runtime")]
    parts.push(Box::new(runtimes()));
    #[cfg(feature = "codehost")]
    {
        #[allow(unused_mut)]
        let mut family = temet_work_codehost::CodeHosts::default();
        if let Some(log) = log {
            family = family.copies(log.with_file_name("copies"));
        }
        #[cfg(feature = "codehost-github")]
        family.add(Box::new(temet_work_codehost_github::GitHub::default()));
        #[cfg(feature = "codehost-gitlab")]
        family.add(Box::new(temet_work_codehost_gitlab::GitLab::default()));
        parts.push(Box::new(family));
    }
    #[cfg(feature = "deploy")]
    {
        let mut deploys = temet_work_deploy::Deploys::default();
        if let Some(log) = log {
            deploys = deploys.handoff(log.with_file_name("deploy-handoff"), BUILD);
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

/// The daemon's host: the event log at `path` replayed onto a world that
/// already holds what `parts` offer, so an older log's plain model is bound to
/// the build's default runtime where the log set no global one.
pub fn start(path: &Path, parts: Vec<Box<dyn Part>>) -> Result<Host, String> {
    let (journal, world) = journal(path, temet_work_core::host::catalog(&parts))?;
    Ok(Host::new(world, parts, journal))
}

/// Opens the event log at `path` and rebuilds the state from it, onto a world
/// that holds `catalog`. A build without the store keeps events in memory, so
/// nothing survives a restart.
#[cfg(feature = "store")]
pub fn journal(path: &Path, catalog: Catalog) -> Result<(Box<dyn Journal>, World), String> {
    let (log, entries) = temet_work_store::Log::open(path).map_err(|e| e.to_string())?;
    // Each entry at the time it was logged with, so a restart gives each issue
    // the same time it last changed.
    let world = temet_work_store::replay_onto(
        World {
            catalog,
            ..World::default()
        },
        &entries,
    );
    Ok((Box::new(log), world))
}

/// Opens the event log at `path` and rebuilds the state from it, onto a world
/// that holds `catalog`. A build without the store keeps events in memory, so
/// nothing survives a restart.
#[cfg(not(feature = "store"))]
pub fn journal(path: &Path, catalog: Catalog) -> Result<(Box<dyn Journal>, World), String> {
    let _ = path;
    Ok((
        Box::new(temet_work_core::host::Memory::default()),
        World {
            catalog,
            ..World::default()
        },
    ))
}

/// What a build's families offer for the settings that name an adapter.
pub type Catalog = std::collections::BTreeMap<String, temet_work_core::settings::Choice>;
