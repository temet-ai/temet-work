//! What happened, as the parts and people report it. The loop's state is built
//! from these alone, in order.

use crate::ids::{
    AgentId, DecisionId, FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId, RuleId,
};
use crate::model::{AgentState, Decision, Plan, Role, Rule};
use crate::settings::{Scope, Value};

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
    /// An area was defined, or changed.
    AreaDefined {
        /// The area.
        area: ProjectId,
        /// Its name as shown.
        name: String,
        /// The repositories in it.
        projects: Vec<ProjectId>,
        /// The tracker adapter that holds its own issues; empty for none.
        #[serde(default)]
        tracker: String,
        /// The tracker's own name for its project.
        #[serde(default)]
        tracker_name: String,
    },
    /// A setting was given a value at one level, for one role or for every role.
    SettingSet {
        /// The level it is set at.
        scope: Scope,
        /// The role it is for; none for every role.
        role: Option<Role>,
        /// The setting.
        key: String,
        /// Its value.
        value: Value,
        /// Who set it: `file`, `person`, or an agent's id.
        by: String,
    },
    /// A setting's value at one level was removed, so the next level answers.
    SettingUnset {
        /// The level.
        scope: Scope,
        /// The role it was for; none for every role.
        role: Option<Role>,
        /// The setting.
        key: String,
        /// Who removed it.
        by: String,
    },
    /// A project's deploy and verify commands were set. Older logs hold these;
    /// they are read as project-level settings.
    ProjectCommandsSet {
        /// The project.
        project: ProjectId,
        /// Run after a merge; none means merged is deployed.
        deploy: Option<String>,
        /// Run after a deploy, to check it works; none means deployed is verified.
        verify: Option<String>,
        /// The folder both run in.
        run_in: Option<String>,
        /// Whether its pull requests run CI; with none, the plan's checks are left
        /// to the reviewer. Runs CI when left out.
        #[serde(default)]
        ci: Option<bool>,
    },
    /// A group was made.
    GroupCreated {
        /// The group.
        group: GroupId,
        /// Its name.
        name: String,
        /// The projects it takes issues from.
        projects: Vec<ProjectId>,
        /// The areas it takes issues from.
        #[serde(default)]
        areas: Vec<ProjectId>,
        /// The milestones it works on, most important first; empty for all.
        milestones: Vec<MilestoneId>,
        /// How many issues it refines at once.
        refining_at_once: usize,
        /// The model its agents run, when it names one. Older logs hold this;
        /// it is read as a model setting on the group's projects and areas.
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
        /// The pull request's number on the code host, from older logs, where one
        /// issue had one pull request: it is read as the first repository's.
        #[serde(default)]
        pull: Option<String>,
        /// The pull requests' numbers by repository, when the author names them.
        #[serde(default)]
        pulls: std::collections::BTreeMap<ProjectId, String>,
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
        /// The option chosen instead, as an index into its options.
        to: usize,
        /// Why. Empty in logs from before overturns had reasons.
        #[serde(default)]
        reason: String,
        /// Who overturned it: `person`.
        #[serde(default)]
        by: String,
    },
    /// A rule was proposed from an overturned decision.
    RuleProposed {
        /// The rule, in state `proposed`.
        rule: Rule,
    },
    /// A person kept or dropped a proposed rule.
    RuleAnswered {
        /// The rule.
        rule: RuleId,
        /// True to keep it, false to drop it.
        keep: bool,
    },
    /// The change merged: one repository's pull request, or (from older logs, and
    /// when left out) all of them.
    Merged {
        /// The issue.
        issue: IssueId,
        /// The repository whose pull request merged.
        #[serde(default)]
        repo: Option<ProjectId>,
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
        /// What the check said when it failed.
        #[serde(default)]
        output: String,
    },
    /// A person asked for the issue's deploy or verify to run again, now.
    Retried {
        /// The issue.
        issue: IssueId,
    },
    /// The code host reports that the pull request under review conflicts with the
    /// branch it merges into. It goes back to its author at once.
    PullConflicts {
        /// The issue.
        issue: IssueId,
        /// The repository whose pull request conflicts.
        #[serde(default)]
        repo: Option<ProjectId>,
    },
    /// The code host would not merge the pull request as it stands, such as for a
    /// conflict with the target branch.
    MergeRefused {
        /// The issue.
        issue: IssueId,
        /// Why, as the code host said it.
        reason: String,
    },
}
