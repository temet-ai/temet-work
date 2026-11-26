//! What the loop asks the daemon to do. The daemon does the I/O and reports
//! back with events.

use crate::ids::{DecisionId, FolderId, GroupId, IssueId};
use crate::launch::{Launch, Program};
use crate::model::Role;

/// One thing for the daemon to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Start an agent for a group, on an issue, in a folder when it needs one.
    StartAgent {
        /// The group.
        group: GroupId,
        /// The role.
        role: Role,
        /// The issue.
        issue: IssueId,
        /// The work folder, for an author.
        folder: Option<FolderId>,
    },
    /// Ask the issue's author to fix what the review or the checks found.
    RequestFix {
        /// The issue.
        issue: IssueId,
        /// What to fix.
        points: Vec<String>,
    },
    /// Start review round `round` of the issue's pull request.
    StartReview {
        /// The issue.
        issue: IssueId,
        /// The round, from 1.
        round: u8,
    },
    /// Merge the issue's pull request.
    Merge {
        /// The issue.
        issue: IssueId,
    },
    /// Deploy what merged.
    Deploy {
        /// The issue.
        issue: IssueId,
    },
    /// Run the real-session check after deploy.
    Verify {
        /// The issue.
        issue: IssueId,
    },
    /// Close the issue in its tracker.
    Close {
        /// The issue.
        issue: IssueId,
    },
    /// The folder is free for the next issue.
    ReleaseFolder {
        /// The folder.
        folder: FolderId,
    },
    /// Carry out a person's overturn of a decision.
    ApplyOverturn {
        /// The decision.
        decision: DecisionId,
    },
    /// Propose a rule change from an overturned decision, to be measured.
    ProposeRule {
        /// The decision it comes from.
        from: DecisionId,
    },
    /// Stop an agent's session: its issue was closed while it worked.
    StopAgent {
        /// The agent.
        agent: crate::ids::AgentId,
    },
    /// Run an agent's program in a terminal. The part that turns a brief into
    /// a program asks for this; the part that runs terminals carries it out.
    Run {
        /// The agent.
        launch: Launch,
        /// Its program.
        program: Program,
    },
}

impl Effect {
    /// The effect in a few words, for a problem report.
    pub fn describe(&self) -> String {
        match self {
            Self::StartAgent { role, issue, .. } => format!("start a {} on {issue}", role.name()),
            Self::RequestFix { issue, .. } => format!("send the fixes for {issue} to its author"),
            Self::StartReview { issue, round } => format!("start review round {round} of {issue}"),
            Self::Merge { issue } => format!("merge {issue}"),
            Self::Deploy { issue } => format!("deploy {issue}"),
            Self::Verify { issue } => format!("verify {issue} after deploy"),
            Self::Close { issue } => format!("close {issue} in its tracker"),
            Self::ReleaseFolder { folder } => format!("release folder {folder}"),
            Self::ApplyOverturn { decision } => {
                format!("apply the overturn of decision {}", decision.0)
            }
            Self::ProposeRule { from } => format!("propose a rule from decision {}", from.0),
            Self::Run { launch, .. } => format!("run {}", launch.agent),
            Self::StopAgent { agent } => format!("stop {agent}"),
        }
    }

    /// Whether a part has to do something. A released folder is already free
    /// in the loop's state; nothing outside needs to hear of it.
    pub fn needs_a_part(&self) -> bool {
        !matches!(self, Self::ReleaseFolder { .. })
    }
}
