//! What runtimes and runners hand each other through the daemon: the program to
//! start, and the brief an agent starts with. Runtimes make a [`Program`] from a
//! [`Brief`]; runners start it. Neither names the other.

use std::path::PathBuf;

use crate::ids::IssueId;
use crate::model::Role;

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
    /// The model to run, when the group sets one.
    pub model: Option<String>,
    /// The folder it works in.
    pub folder: Option<PathBuf>,
}
