//! The objects the work loop keeps. `docs/objects.md` describes each one.

use crate::ids::{
    AgentId, DecisionId, FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId,
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
    /// The work folder it holds while an author works on it.
    pub folder: Option<FolderId>,
    /// The refiner's plan.
    pub plan: Option<Plan>,
    /// Whether the reviewer approved the current change.
    pub review_ok: bool,
    /// Whether CI and the checks passed on the current change.
    pub checks_ok: Option<bool>,
    /// Its pull request's number on the code host, once the author names it.
    pub pull: Option<String>,
    /// What its author is to fix, while it is being fixed.
    pub fix: Vec<String>,
    /// Whether the agent its stage needs has heard what to do: the author what
    /// to fix, the reviewer that a new round is up. A new agent hears it in its
    /// brief.
    pub told: bool,
    /// Whether the loop closed it, after verifying it. One the tracker closed and
    /// later reopened is worked again; one the loop closed stays closed.
    pub closed_by_loop: bool,
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
    /// The milestones it works on, most important first. Empty means every issue.
    pub milestones: Vec<MilestoneId>,
    /// How many issues it refines at once.
    pub refining_at_once: usize,
    /// The model its agents run, when it names one.
    pub model: Option<String>,
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

/// A project: one repository and the tracker its issues come from.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Project {
    /// Its id.
    pub id: ProjectId,
    /// The id of the tracker adapter that holds its issues.
    pub tracker: String,
    /// The tracker's own name for it (`owner/repo`, a project key).
    pub name: String,
    /// The id of the code-host adapter that holds its pull requests.
    pub code_host: String,
    /// The code host's name for the repository.
    pub repo: String,
    /// Run after a merge; none means merged is deployed.
    pub deploy: Option<String>,
    /// Run after a deploy, to check it works; none means deployed is verified.
    pub verify: Option<String>,
    /// The folder both run in.
    pub run_in: Option<String>,
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
}
