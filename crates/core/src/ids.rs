//! Typed ids, so a project id can never be passed where an agent id belongs.

use std::fmt;

macro_rules! id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $name(pub String);

        impl $name {
            /// Makes the id from any string.
            pub fn new(s: impl Into<String>) -> Self {
                Self(s.into())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

id!(
    /// A project: one repository and its tracker.
    ProjectId
);
id!(
    /// A milestone, as its tracker names it.
    MilestoneId
);
id!(
    /// A group of sessions working on a goal.
    GroupId
);
id!(
    /// One agent session.
    AgentId
);
id!(
    /// A machine agents run on.
    MachineId
);
id!(
    /// A work folder: a checkout on one machine.
    FolderId
);

/// An issue: its project and the key its tracker gives it (`516`, `ENG-41`).
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct IssueId {
    /// The project the issue belongs to.
    pub project: ProjectId,
    /// The tracker's key for it.
    pub key: String,
}

impl IssueId {
    /// Reads `project#key`, the form issue ids are shown in.
    pub fn parse(s: &str) -> Option<Self> {
        let (project, key) = s.rsplit_once('#')?;
        (!project.is_empty() && !key.is_empty()).then(|| Self::new(project, key))
    }

    /// Makes an issue id.
    pub fn new(project: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            project: ProjectId::new(project),
            key: key.into(),
        }
    }
}

impl fmt::Display for IssueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.project, self.key)
    }
}

/// A decision, numbered in the order decisions were made.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct DecisionId(pub u64);
