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
        /// Where it is on that machine.
        path: String,
    },
    /// A new log was started. Its id labels the terminals its agents run in, so
    /// two daemons, or a daemon and an older log, never take each other's agents.
    LogStarted {
        /// The log's id.
        id: String,
    },
    /// A project was added, or its tracker changed.
    ProjectAdded {
        /// The project.
        project: ProjectId,
        /// The id of the tracker adapter that holds its issues.
        tracker: String,
        /// The tracker's own name for it.
        name: String,
        /// The id of the code-host adapter that holds its pull requests; the
        /// tracker's when left out.
        #[serde(default)]
        code_host: Option<String>,
        /// The code host's name for the repository; the tracker's name when left out.
        #[serde(default)]
        repo: Option<String>,
    },
    /// A project's deploy and verify commands were set.
    ProjectCommandsSet {
        /// The project.
        project: ProjectId,
        /// Run after a merge; none means merged is deployed.
        deploy: Option<String>,
        /// Run after a deploy, to check it works; none means deployed is verified.
        verify: Option<String>,
        /// The folder both run in.
        run_in: Option<String>,
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
        /// The model its agents run, when it names one.
        #[serde(default)]
        model: Option<String>,
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
        /// The pull request's number on the code host, when the author names it.
        #[serde(default)]
        pull: Option<String>,
    },
    /// What an issue's agent was to hear reached it: the fixes, or a new round.
    Told {
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
        /// The names of the checks that failed.
        #[serde(default)]
        failing: Vec<String>,
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
