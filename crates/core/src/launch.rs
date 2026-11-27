//! What runtimes and runners hand each other through the daemon: the program to
//! start, and the brief an agent starts with. Runtimes make a [`Program`] from a
//! [`Brief`]; runners start it. Neither names the other.

use std::path::PathBuf;

use crate::ids::IssueId;
use crate::model::Role;

/// The environment variable that names the agent sending a command. Every
/// agent is started with it, so the commands it runs are recorded as its own.
pub const AGENT_VAR: &str = "TEMET_WORK_AGENT";

/// A program to start: its arguments, first the program itself, and the
/// environment variables it adds.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Program {
    /// The program and its arguments.
    pub argv: Vec<String>,
    /// Environment variables to add, in order.
    pub env: Vec<(String, String)>,
}

/// What an agent is started with.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Brief {
    /// What the agent does.
    pub role: Role,
    /// The issue it works on.
    pub issue: IssueId,
    /// Its task, in full: what to do, what done means, what to read first.
    pub task: String,
    /// The model to run, when a setting names one.
    pub model: Option<String>,
    /// How the runtime asks about what the agent may do (`permission`).
    pub permission: String,
    /// The folder it works in.
    pub folder: Option<PathBuf>,
}

/// An agent about to start: who it is, where it runs, and its brief. The loop
/// makes one from its decision to start an agent ([`crate::World::launch`]).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Launch {
    /// Its id.
    pub agent: crate::ids::AgentId,
    /// Its group.
    pub group: crate::ids::GroupId,
    /// The machine it runs on.
    pub machine: crate::ids::MachineId,
    /// The work folder it holds, for an author.
    pub folder: Option<crate::ids::FolderId>,
    /// What it is told.
    pub brief: Brief,
}
