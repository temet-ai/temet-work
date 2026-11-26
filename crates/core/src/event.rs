//! What happened, as the parts and people report it. The loop's state is built
//! from these alone, in order.

use crate::ids::{
    AgentId, DecisionId, FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId,
};
use crate::model::{AgentState, Decision, Plan, Role};

/// The reviewer's answer for one round.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Verdict {
    /// The change is good.
    Approve,
    /// The change needs work; each point says what and where.
    Block {
        /// What must change.
        points: Vec<String>,
    },
}

/// One thing that happened.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Event {
    /// A work folder exists for a project on a machine.
    FolderAdded {
        /// The folder.
        folder: FolderId,
        /// Its project.
        project: ProjectId,
        /// Its machine.
        machine: MachineId,
    },
    /// A group was made.
    GroupCreated {
        /// The group.
        group: GroupId,
        /// Its name.
        name: String,
        /// The projects it takes issues from.
        projects: Vec<ProjectId>,
        /// The milestones it works on, most important first; empty for all.
        milestones: Vec<MilestoneId>,
        /// How many issues it refines at once.
        refining_at_once: usize,
    },
    /// The tracker reported an issue, new or changed.
    IssueSynced {
        /// The issue.
        issue: IssueId,
        /// Its title.
        title: String,
        /// Its milestone.
        milestone: Option<MilestoneId>,
        /// Issues that must be done first.
        depends_on: Vec<IssueId>,
        /// The tracker's order, oldest first.
        order: u64,
        /// Whether it is closed in the tracker.
        closed: bool,
    },
    /// An agent session started.
    AgentStarted {
        /// The agent.
        agent: AgentId,
        /// Its group.
        group: GroupId,
        /// Its role.
        role: Role,
        /// The issue it works on.
        issue: Option<IssueId>,
        /// Its machine.
        machine: MachineId,
        /// Its folder.
        folder: Option<FolderId>,
    },
    /// An agent's state changed.
    AgentChanged {
        /// The agent.
        agent: AgentId,
        /// Its new state.
        state: AgentState,
    },
    /// The refiner wrote an issue's plan.
    PlanWritten {
        /// The issue.
        issue: IssueId,
        /// The plan.
        plan: Plan,
    },
    /// The author opened or updated the pull request.
    PullRequestReady {
        /// The issue.
        issue: IssueId,
    },
    /// The reviewer finished a round.
    ReviewFinished {
        /// The issue.
        issue: IssueId,
        /// The round.
        round: u8,
        /// The verdict.
        verdict: Verdict,
    },
    /// CI and the plan's checks finished on the current change.
    ChecksFinished {
        /// The issue.
        issue: IssueId,
        /// Whether they passed.
        passed: bool,
    },
    /// An agent made a decision.
    DecisionMade {
        /// The decision.
        decision: Decision,
    },
    /// A person overturned a decision.
    DecisionOverturned {
        /// The decision.
        decision: DecisionId,
        /// The option chosen instead.
        to: usize,
    },
    /// The change merged.
    Merged {
        /// The issue.
        issue: IssueId,
    },
    /// The change deployed.
    Deployed {
        /// The issue.
        issue: IssueId,
    },
    /// The real-session check after deploy finished.
    Verified {
        /// The issue.
        issue: IssueId,
        /// Whether it passed.
        passed: bool,
    },
}
