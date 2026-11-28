//! What the loop asks the daemon to do. The daemon does the I/O and reports
//! back with events.

use crate::ids::{FolderId, GroupId, IssueId};
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
    /// Start a group's investigator, for the investigation the group began. It
    /// holds no folder and works on no issue ([`crate::World::launch_investigator`]).
    Investigate {
        /// The group.
        group: GroupId,
    },
    /// File an issue in the tracker of `source`, for `agent`, the investigator of
    /// `group`. The part that does it reports the new issue as synced and filed.
    File {
        /// The group.
        group: GroupId,
        /// The investigator.
        agent: crate::ids::AgentId,
        /// The project or area whose tracker it goes in.
        source: crate::ids::ProjectId,
        /// Its title.
        title: String,
        /// Its text.
        body: String,
        /// Its milestone, if any.
        milestone: Option<crate::ids::MilestoneId>,
    },
    /// Give an issue a new title in its tracker, for a person's refine. The
    /// daemon carries it out before the refine is logged, and refuses the refine
    /// when it fails.
    Retitle {
        /// The issue.
        issue: IssueId,
        /// Its new title.
        title: String,
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
    /// Run the issue's deploy or verify again at once, forgetting how often and
    /// how recently it failed.
    Retry {
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
    /// Stop an agent's session: its issue was closed while it worked.
    StopAgent {
        /// The agent.
        agent: crate::ids::AgentId,
    },
    /// Run an agent's program in a terminal. The part that turns a brief into
    /// a program asks for this; the part that runs terminals carries it out.
    Run {
        /// The agent. Boxed, so the effect stays small.
        launch: Box<Launch>,
        /// The id of the runtime that made its program.
        runtime: String,
        /// Its program.
        program: Program,
        /// A program run to its end on the agent's machine before its program
        /// starts: the one that marks its folder trusted there, for a folder on
        /// another machine. When it fails, the agent does not start.
        before: Option<Program>,
    },
}

impl Effect {
    /// The effect in a few words, for a problem report.
    pub fn describe(&self) -> String {
        match self {
            Self::StartAgent { role, issue, .. } => format!("start a {} on {issue}", role.name()),
            Self::Investigate { group } => format!("start an investigator for {group}"),
            Self::File { source, title, .. } => format!("file \"{title}\" in {source}"),
            Self::Retitle { issue, .. } => format!("retitle {issue} in its tracker"),
            Self::RequestFix { issue, .. } => format!("send the fixes for {issue} to its author"),
            Self::StartReview { issue, round } => format!("start review round {round} of {issue}"),
            Self::Merge { issue } => format!("merge {issue}"),
            Self::Deploy { issue } => format!("deploy {issue}"),
            Self::Verify { issue } => format!("verify {issue} after deploy"),
            Self::Retry { issue } => format!("retry the deploy or verify of {issue}"),
            Self::Close { issue } => format!("close {issue} in its tracker"),
            Self::ReleaseFolder { folder } => format!("release folder {folder}"),
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
