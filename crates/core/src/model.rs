//! The objects the work loop keeps. `docs/objects.md` describes each one.

use crate::ids::{
    AgentId, DecisionId, FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId, RuleId,
};

/// Where an issue is in the loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Stage {
    /// Synced from the tracker, not taken by a group yet.
    New,
    /// A group took it; a refiner is writing its plan.
    Refining,
    /// It has a plan and waits for a work folder and for the issues it depends on.
    Ready,
    /// An author works on it in a work folder.
    Working,
    /// Its pull request is in review round `round` (from 1).
    InReview {
        /// The review round.
        round: u8,
    },
    /// The author is fixing what round `round` asked for, or a failed check.
    Fixing {
        /// The review round the fix answers.
        round: u8,
    },
    /// The rounds ran out with the reviewer still blocking; an arbiter rules.
    Arbitration {
        /// The last review round.
        round: u8,
    },
    /// Review and checks passed; it is being merged.
    Merging,
    /// Merged; it is being deployed.
    Deploying,
    /// Deployed; the real-session check is running.
    Verifying,
    /// Merged, deployed, checked and closed, or closed in the tracker.
    Done,
}

/// One "Done when" line and what proves it.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Check {
    /// The line from the issue.
    pub done_when: String,
    /// The test or command that proves it.
    pub proven_by: String,
}

/// The refiner's plan for an issue.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Plan {
    /// The checks the work must pass.
    pub checks: Vec<Check>,
    /// The repositories the change touches, in the order they merge. Empty means
    /// the issue's own project.
    #[serde(default)]
    pub repos: Vec<ProjectId>,
    /// Settings the refiner gives this issue, such as a stronger model for a
    /// hard one: task-level, recorded as set by the refiner.
    #[serde(default)]
    pub settings: std::collections::BTreeMap<String, crate::settings::Value>,
}

/// An issue as the loop sees it.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Issue {
    /// Its project and tracker key.
    pub id: IssueId,
    /// Its title.
    pub title: String,
    /// Its milestone, if any.
    pub milestone: Option<MilestoneId>,
    /// Issues that must be done first.
    pub depends_on: Vec<IssueId>,
    /// The tracker's order (oldest first), used after milestone order.
    pub order: u64,
    /// Where it is in the loop.
    pub stage: Stage,
    /// The group that took it.
    pub group: Option<GroupId>,
    /// The work folder it holds while an author works on it: the one of the
    /// first repository its plan names.
    pub folder: Option<FolderId>,
    /// The folders it holds in the plan's other repositories, in the plan's order.
    pub other_folders: Vec<FolderId>,
    /// The refiner's plan.
    pub plan: Option<Plan>,
    /// Whether the reviewer approved the current change.
    pub review_ok: bool,
    /// Whether CI and the checks passed on the current change.
    pub checks_ok: Option<bool>,
    /// Its pull requests' numbers on the code host, by repository, once the
    /// author names them.
    pub pulls: std::collections::BTreeMap<ProjectId, String>,
    /// The repositories whose pull request has merged.
    pub merged: std::collections::BTreeSet<ProjectId>,
    /// What its author is to fix, while it is being fixed.
    pub fix: Vec<String>,
    /// Whether the agent its stage needs has heard what to do: the author what
    /// to fix, the reviewer that a new round is up. A new agent hears it in its
    /// brief.
    pub told: bool,
    /// Whether it is being fixed for CI or a merge, not for a review: the next
    /// review keeps its round, so CI failures do not use up review rounds.
    pub fix_keeps_round: bool,
    /// Its latest review round; 0 before its first review.
    pub round: u8,
    /// Whether the loop closed it, after verifying it. One the tracker closed and
    /// later reopened is worked again; one the loop closed stays closed.
    pub closed_by_loop: bool,
    /// Whether a person's overturn of a ruling approved it: the review is not
    /// asked again, and it merges once the checks pass.
    #[serde(default)]
    pub review_waived: bool,
}

impl Issue {
    /// Takes every work folder the issue holds, the first one first.
    pub fn take_folders(&mut self) -> Vec<FolderId> {
        self.folder
            .take()
            .into_iter()
            .chain(std::mem::take(&mut self.other_folders))
            .collect()
    }
}

/// What an agent does.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Role {
    /// Writes an issue's plan.
    Refiner,
    /// Changes the code.
    Author,
    /// Reviews a pull request.
    Reviewer,
    /// Rules when review rounds run out or author and reviewer disagree.
    Arbiter,
    /// Answers questions for the others.
    Investigator,
}

impl Role {
    /// Reads a role from its lower-case word.
    pub fn parse(s: &str) -> Result<Self, String> {
        [
            Self::Refiner,
            Self::Author,
            Self::Reviewer,
            Self::Arbiter,
            Self::Investigator,
        ]
        .into_iter()
        .find(|r| r.name() == s)
        .ok_or_else(|| {
            format!("no role {s}; the roles are refiner, author, reviewer, arbiter, investigator")
        })
    }

    /// The role in one lower-case word, as agent ids and labels use it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Refiner => "refiner",
            Self::Author => "author",
            Self::Reviewer => "reviewer",
            Self::Arbiter => "arbiter",
            Self::Investigator => "investigator",
        }
    }
}

/// What an agent is doing, as its runner reports it.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AgentState {
    /// In a turn, with a line on what it is doing.
    Working {
        /// What it is doing, in its own words.
        doing: String,
    },
    /// Between turns.
    Idle,
    /// Stopped at a prompt it cannot answer itself, such as a permission.
    Blocked {
        /// What it is asking for.
        reason: String,
    },
    /// Its turn ended in an error, such as a usage limit. Typing starts a new
    /// turn, so this is not a prompt.
    Errored {
        /// The line its turn ended on; empty when no line named the error.
        line: String,
        /// When the limit the line names resets, in milliseconds since 1970,
        /// when the line gives a time that can be read.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        resets_ms: Option<u64>,
    },
    /// The session has ended.
    Ended,
}

/// One agent session.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    /// Its id.
    pub id: AgentId,
    /// The group it belongs to.
    pub group: GroupId,
    /// What it does.
    pub role: Role,
    /// The issue it works on, if any.
    pub issue: Option<IssueId>,
    /// The machine it runs on.
    pub machine: MachineId,
    /// The work folder it runs in, if any.
    pub folder: Option<FolderId>,
    /// What it is doing.
    pub state: AgentState,
    /// When it started: how many events the loop had applied by then. A later
    /// agent has a larger number.
    pub started: u64,
    /// The SHA-256 digest of the secret it was started with; none for an agent
    /// started before agents had secrets, which is taken by its id.
    pub secret_digest: Option<crate::sender::Digest>,
    /// The reset time of the last limit the daemon sent it `continue` after, so
    /// the same limit coming back is not continued again.
    #[serde(default)]
    pub continued: Option<u64>,
}

impl AgentState {
    /// When the limit an agent in this state waits on resets, for an agent last
    /// continued after the limit that reset at `continued`: its turn ended in an
    /// error whose line gives a reset time later than that. `None` for any other
    /// state, including an error with no reset time or one already acted on:
    /// such an agent is treated as idle.
    pub fn limit_reset(&self, continued: Option<u64>) -> Option<u64> {
        match self {
            Self::Errored {
                resets_ms: Some(at),
                ..
            } if continued.is_none_or(|done| *at > done) => Some(*at),
            _ => None,
        }
    }
}

impl Agent {
    /// When the limit this agent waits on resets ([`AgentState::limit_reset`]).
    pub fn limit_reset(&self) -> Option<u64> {
        self.state.limit_reset(self.continued)
    }
}

/// A group of sessions working on a goal.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Group {
    /// Its id.
    pub id: GroupId,
    /// A name a person gave it.
    pub name: String,
    /// The projects it takes issues from.
    pub projects: Vec<ProjectId>,
    /// The areas it takes issues from: the area's own and every project's in it.
    pub areas: Vec<ProjectId>,
    /// The milestones it works on, most important first. Empty means every issue.
    pub milestones: Vec<MilestoneId>,
    /// How many issues it refines at once.
    pub refining_at_once: usize,
    /// Below how many issues in its work queue it starts an investigator; none
    /// for the default, twice the work folders of its projects
    /// ([`crate::World::queue_low`]). 0 starts none.
    #[serde(default)]
    pub queue_low: Option<usize>,
    /// Minutes from the end of its last investigation before another may start.
    #[serde(default = "investigate_every")]
    pub investigate_every: u64,
}

/// How many minutes a group waits after an investigation ends before it starts
/// another, when its configuration names none.
pub const INVESTIGATE_EVERY: u64 = 120;

fn investigate_every() -> u64 {
    INVESTIGATE_EVERY
}

/// A group's investigation: an investigator reads the group's projects for
/// planned work that is not built and defects in merged work, and files an issue
/// for each. A group has at most one running at a time.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Investigation {
    /// The group.
    pub group: GroupId,
    /// Its investigator.
    pub agent: AgentId,
    /// When it started, in milliseconds since the Unix epoch.
    pub started_ms: u64,
    /// When the group's investigation before it started; none for its first.
    pub since_ms: Option<u64>,
    /// When it ended; none while it runs.
    pub ended_ms: Option<u64>,
    /// The issues its investigator filed, in order.
    pub filed: Vec<IssueId>,
    /// Whether its investigator said it is done.
    pub done: bool,
}

impl Investigation {
    /// Whether it is still running.
    pub fn running(&self) -> bool {
        self.ended_ms.is_none()
    }
}

/// A work folder: a checkout on one machine, held by one issue at a time.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Folder {
    /// Its id.
    pub id: FolderId,
    /// The project it is a checkout of.
    pub project: ProjectId,
    /// The machine it is on.
    pub machine: MachineId,
    /// Where it is on that machine.
    pub path: String,
    /// The issue that holds it, if any.
    pub holder: Option<IssueId>,
}

/// An area: repositories that work together, and the tracker its own issues come
/// from when one tracker project serves them all. `docs/areas.md` has why.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Area {
    /// Its id; projects and areas share one set of ids.
    pub id: ProjectId,
    /// Its name as shown.
    pub name: String,
    /// The repositories in it.
    pub projects: Vec<ProjectId>,
    /// The id of the tracker adapter that holds its issues; empty when its projects
    /// keep their own.
    pub tracker: String,
    /// The tracker's own name for its project.
    pub tracker_name: String,
}

/// A project: one repository and the tracker its issues come from.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Project {
    /// Its id.
    pub id: ProjectId,
    /// The id of the tracker adapter that holds its issues; empty when its area's
    /// tracker holds them.
    pub tracker: String,
    /// The tracker's own name for it (`owner/repo`, a project key).
    pub name: String,
    /// The id of the code-host adapter that holds its pull requests.
    pub code_host: String,
    /// The code host's name for the repository.
    pub repo: String,
}

/// What a decision settles.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DecisionKind {
    /// An arbiter's ruling on a review: approve the change, or send it back.
    Ruling {
        /// True to approve, false to send the change back to its author.
        approve: bool,
    },
    /// Any other choice an agent made.
    Choice,
}

/// A choice an agent made. Work goes on at once; a person can overturn it later.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Decision {
    /// Its number.
    pub id: DecisionId,
    /// The issue it is about.
    pub issue: IssueId,
    /// The agent that made it.
    pub by: AgentId,
    /// What was decided.
    pub kind: DecisionKind,
    /// The question, in a line.
    pub question: String,
    /// The options weighed.
    pub options: Vec<String>,
    /// The option taken, as an index into `options`.
    pub chosen: usize,
    /// Why.
    pub reason: String,
    /// The option a person chose instead, once overturned.
    pub overturned_to: Option<usize>,
    /// Why the person overturned it.
    #[serde(default)]
    pub overturn_reason: String,
}

/// Where a rule is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RuleState {
    /// Made from an overturn; a person has not answered yet. Not in any brief.
    Proposed,
    /// A person kept it: every brief written from now on holds it.
    Kept,
    /// A person dropped it.
    Dropped,
}

impl RuleState {
    /// The state in one lower-case word.
    pub fn name(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Kept => "kept",
            Self::Dropped => "dropped",
        }
    }
}

/// A line agents are to work by, proposed from a decision a person overturned.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Rule {
    /// Its number.
    pub id: RuleId,
    /// The rule, as an agent reads it.
    pub text: String,
    /// The decision it comes from.
    pub decision: DecisionId,
    /// The issue that decision was about.
    pub issue: IssueId,
    /// The agent that made the decision.
    pub decided_by: AgentId,
    /// Who overturned it.
    pub overturned_by: String,
    /// Where it is.
    pub state: RuleState,
}
