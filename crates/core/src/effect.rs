//! What the loop asks the daemon to do. The daemon does the I/O and reports
//! back with events.

use crate::ids::{DecisionId, FolderId, GroupId, IssueId};
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
}
