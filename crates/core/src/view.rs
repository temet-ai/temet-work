//! The loop's state as data, for a front end: what the text form lists, in the
//! same order, as plain fields that need none of the loop's logic to draw, and
//! what the desktop window shows beside it: when each issue last changed, its
//! plan's checks and its review rounds with the author's replies, each agent's
//! runtime and model, milestones with their progress and due dates, what
//! overturning a decision costs, and where each project's issues and code are.
//! Times are in milliseconds since 1970 and dates are `YYYY-MM-DD`; the window
//! works out "4 min ago" with its own clock.
//!
//! The desktop app reads it, and `temet-work status --json` prints it. It is
//! built from named fields, so what the log holds for the loop alone stays out:
//! an agent's secret digest and limit marks, folder paths, a plan's settings and
//! repositories, what an author is to fix, settings, rules and the waiting list. The problem lines and the
//! agents' state lines are free text that may name a work folder's path, so they
//! are taken without it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::host;
use crate::ids::{AgentId, FolderId, GroupId, MachineId, MilestoneId, ProjectId};
use crate::model::{self, AgentState, Check, Cost, Point, Stage};
use crate::text;
use crate::world::World;

/// The state, with what is wrong now. Done issues are counted, not listed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    /// The projects, by id.
    pub projects: Vec<Project>,
    /// The milestones of each project and area, by project or area and name.
    pub milestones: Vec<Milestone>,
    /// The groups, by id.
    pub groups: Vec<Group>,
    /// The issues that are not done, by project and key.
    pub issues: Vec<Issue>,
    /// How many issues are done.
    pub done: usize,
    /// The agents, by id, ended ones too.
    pub agents: Vec<Agent>,
    /// The decisions, by number.
    pub decisions: Vec<Decision>,
    /// What is wrong now, by where it came from.
    pub problems: BTreeMap<String, String>,
}

/// A project.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    /// Its id.
    pub id: ProjectId,
    /// Its tracker's name for it; none when its issues come from its area.
    pub tracker_name: Option<String>,
    /// The web address of its issues, from its tracker; none when its issues
    /// come from its area, or its tracker cannot say.
    pub issues_url: Option<String>,
    /// The web address of its code, from its code host; none when the code host
    /// cannot say.
    pub code_url: Option<String>,
    /// The area it is in; none for a project outside an area.
    pub area: Option<Area>,
}

/// The area a project is in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Area {
    /// Its id.
    pub id: ProjectId,
    /// Its name as shown.
    pub name: String,
    /// The web address of its own issues, from its tracker; none for an area
    /// whose projects keep their own, or whose tracker cannot say.
    pub issues_url: Option<String>,
}

/// A milestone of a project or area: one an issue names, or an open one its
/// tracker lists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Milestone {
    /// The project or area it is in.
    pub project: ProjectId,
    /// Its name.
    pub id: MilestoneId,
    /// How many of its issues are done.
    pub done: usize,
    /// How many issues it has, done ones too. An issue closed in the tracker
    /// before temet-work first synced it is not counted.
    pub total: usize,
    /// Its due date as `YYYY-MM-DD`; none when its tracker gives none or does
    /// not list it as open.
    pub due: Option<String>,
}

/// A group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    /// Its id.
    pub id: GroupId,
    /// A name a person gave it.
    pub name: String,
    /// The projects it names, as the text form lists them.
    pub projects: Vec<ProjectId>,
    /// The milestones it works on, most important first; empty for every one.
    pub milestones: Vec<MilestoneId>,
    /// How many issues it refines at once.
    pub refining_at_once: usize,
}

/// An issue that is not done.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    /// `project#key`, as commands take it.
    pub id: String,
    /// Its title.
    pub title: String,
    /// Where it is in the loop, in a fixed word.
    pub stage: StageWord,
    /// Where it is, in the text form's words: the round, and whether it left its
    /// group's scope.
    pub stage_text: String,
    /// Its milestone, if any.
    pub milestone: Option<MilestoneId>,
    /// The group that took it.
    pub group: Option<GroupId>,
    /// The issues that must be done first, each as `project#key`.
    pub depends_on: Vec<String>,
    /// Its latest review round; 0 before its first review.
    pub round: u8,
    /// Its pull requests, in the order their repositories merge.
    pub pulls: Vec<Pull>,
    /// Whether the review approved the current change.
    pub review_ok: bool,
    /// Whether CI and the checks passed on the current change; none until they
    /// finish. Its checks have no state of their own: this is theirs.
    pub checks_ok: Option<bool>,
    /// The agents on it, ended ones too.
    pub agents: Vec<AgentId>,
    /// When it last changed, in milliseconds since 1970; none when no event
    /// with a time has changed it since the daemon's log began holding them.
    pub updated_ms: Option<u64>,
    /// Its plan's checks, in the plan's order; none before it has a plan.
    pub checks: Vec<Check>,
    /// Its review rounds, in the order they finished.
    pub reviews: Vec<Review>,
}

/// One finished review round.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    /// The round, from 1.
    pub round: u8,
    /// The reviewer working on the issue when it finished; none when none was.
    pub reviewer: Option<AgentId>,
    /// What it said.
    pub verdict: VerdictWord,
    /// What a block asked to change; empty for an approval.
    pub points: Vec<String>,
    /// The author's replies, sent with the fix that answered this round; empty
    /// until then, or when the author sent none.
    pub replies: Vec<String>,
}

/// A review round's verdict, as one word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerdictWord {
    /// The change is good.
    Approve,
    /// The change needs work.
    Block,
}

/// One repository's pull request for an issue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pull {
    /// The repository.
    pub repo: ProjectId,
    /// Its number on the code host.
    pub number: String,
    /// Whether it has merged.
    pub merged: bool,
}

/// Where an issue is in the loop, as one word. They sort in the loop's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageWord {
    /// [`Stage::New`].
    New,
    /// [`Stage::Refining`].
    Refining,
    /// [`Stage::Ready`].
    Ready,
    /// [`Stage::Working`].
    Working,
    /// [`Stage::InReview`].
    InReview,
    /// [`Stage::Fixing`].
    Fixing,
    /// [`Stage::Arbitration`].
    Arbitration,
    /// [`Stage::Merging`].
    Merging,
    /// [`Stage::Deploying`].
    Deploying,
    /// [`Stage::Verifying`].
    Verifying,
    /// [`Stage::Done`].
    Done,
}

impl From<Stage> for StageWord {
    fn from(stage: Stage) -> Self {
        match stage {
            Stage::New => Self::New,
            Stage::Refining => Self::Refining,
            Stage::Ready => Self::Ready,
            Stage::Working => Self::Working,
            Stage::InReview { .. } => Self::InReview,
            Stage::Fixing { .. } => Self::Fixing,
            Stage::Arbitration { .. } => Self::Arbitration,
            Stage::Merging => Self::Merging,
            Stage::Deploying => Self::Deploying,
            Stage::Verifying => Self::Verifying,
            Stage::Done => Self::Done,
        }
    }
}

/// One agent session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agent {
    /// Its id.
    pub id: AgentId,
    /// The group it belongs to.
    pub group: GroupId,
    /// What it does, in one lower-case word ([`model::Role::name`]).
    pub role: String,
    /// The issue it works on, as `project#key`, if any.
    pub issue: Option<String>,
    /// The machine it runs on.
    pub machine: MachineId,
    /// The work folder it runs in, if any.
    pub folder: Option<FolderId>,
    /// What it is doing, in a fixed word.
    pub state: StateWord,
    /// What it is doing, in the text form's words.
    pub state_text: String,
    /// Its start number: a later agent has a larger one.
    pub started: u64,
    /// The runtime it was started with; none for an agent from an older log.
    pub runtime: Option<String>,
    /// The model it was started with; none when its settings named none, or for
    /// an agent from an older log.
    pub model: Option<String>,
}

/// What an agent is doing, as one word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateWord {
    /// [`AgentState::Working`].
    Working,
    /// [`AgentState::Idle`].
    Idle,
    /// [`AgentState::Blocked`].
    Blocked,
    /// [`AgentState::Errored`].
    Errored,
    /// [`AgentState::Ended`].
    Ended,
}

impl From<&AgentState> for StateWord {
    fn from(state: &AgentState) -> Self {
        match state {
            AgentState::Working { .. } => Self::Working,
            AgentState::Idle => Self::Idle,
            AgentState::Blocked { .. } => Self::Blocked,
            AgentState::Errored { .. } => Self::Errored,
            AgentState::Ended => Self::Ended,
        }
    }
}

/// A choice an agent made. Options are numbered from 1, as the text form lists
/// them and `overturn` takes them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    /// Its number.
    pub id: u64,
    /// The issue it is about, as `project#key`.
    pub issue: String,
    /// The agent that made it.
    pub by: AgentId,
    /// The question, in a line.
    pub question: String,
    /// The options weighed; the first is number 1.
    pub options: Vec<String>,
    /// The number of the option taken.
    pub chosen: usize,
    /// Why.
    pub reason: String,
    /// The number of the option a person chose instead, once overturned.
    pub overturned_to: Option<usize>,
    /// Why the person overturned it; empty until then.
    pub overturn_reason: String,
    /// What overturning it costs, as `low`, `medium` or `high`; none for a
    /// choice, and for a ruling from an older log.
    pub cost: Option<Cost>,
    /// The point after which overturning it costs more; none for a choice, and
    /// for a ruling from an older log. No time is given for it: nothing knows
    /// when a merge will happen.
    pub point: Option<DecisionPoint>,
}

/// The point after which overturning a decision costs more.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionPoint {
    /// Which point, as a word: `merge`, its issue's merge.
    pub at: Point,
    /// Whether its issue is past it.
    pub passed: bool,
}

impl View {
    /// The view of `world`, with what is wrong now.
    pub fn new(world: &World, problems: &BTreeMap<String, String>) -> Self {
        Self {
            projects: world.projects.values().map(|p| project(world, p)).collect(),
            milestones: milestones(world),
            groups: world.groups.values().map(group).collect(),
            issues: world
                .issues
                .values()
                .filter(|i| i.stage != Stage::Done)
                .map(|i| issue(world, i))
                .collect(),
            done: world
                .issues
                .values()
                .filter(|i| i.stage == Stage::Done)
                .count(),
            agents: world.agents.values().map(|a| agent(world, a)).collect(),
            decisions: world
                .decisions
                .values()
                .map(|d| decision(world, d))
                .collect(),
            problems: shown(world, problems),
        }
    }
}

/// Every milestone an issue names and every open one a tracker lists, by
/// project or area and name, with its issues counted.
fn milestones(world: &World) -> Vec<Milestone> {
    let mut counts: BTreeMap<(&ProjectId, &MilestoneId), (usize, usize)> = BTreeMap::new();
    for (source, listed) in &world.milestones {
        for m in listed.keys() {
            counts.entry((source, m)).or_default();
        }
    }
    for i in world.issues.values() {
        if let Some(m) = &i.milestone {
            let (done, total) = counts.entry((&i.id.project, m)).or_default();
            *total += 1;
            if i.stage == Stage::Done {
                *done += 1;
            }
        }
    }
    counts
        .into_iter()
        .map(|((project, id), (done, total))| Milestone {
            due: world
                .milestones
                .get(project)
                .and_then(|listed| listed.get(id))
                .cloned()
                .flatten(),
            project: project.clone(),
            id: id.clone(),
            done,
            total,
        })
        .collect()
}

/// The problem lines for the view, without work folders' paths. The host's lines
/// for stopped agents name the folder each runs in, so they are written again
/// without it; one for an agent that has since left that state is dropped.
fn shown(world: &World, problems: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let stopped: BTreeMap<String, String> = world
        .agents
        .values()
        .filter_map(|a| host::stopped(a, String::new))
        .collect();
    problems
        .iter()
        .filter_map(|(key, line)| {
            let line = if host::of_a_stopped_agent(key) {
                stopped.get(key)?
            } else {
                line
            };
            Some((key.clone(), without_paths(world, line)))
        })
        .collect()
}

/// `line` with each work folder's path in it written as `folder ID`. A path that
/// is a bare name, such as `work`, is a word like any other, so it is replaced
/// only where a part says it cannot trust it.
fn without_paths(world: &World, line: &str) -> String {
    let mut folders: Vec<&model::Folder> = world.folders.values().collect();
    // Longest first, so a folder inside another is replaced whole.
    folders.sort_by_key(|f| std::cmp::Reverse(f.path.len()));
    let mut line = line.to_string();
    for f in folders {
        let named = format!("folder {}", f.id);
        let path = f.path.trim_end_matches('/');
        if path.contains('/') {
            line = line.replace(path, &named);
        } else {
            for after in [":", " on "] {
                line = line.replace(
                    &format!("cannot trust {}{after}", f.path),
                    &format!("cannot trust {named}{after}"),
                );
            }
        }
    }
    line
}

fn project(world: &World, p: &model::Project) -> Project {
    let address = |id: &ProjectId| world.addresses.get(id).cloned().unwrap_or_default();
    let own = address(&p.id);
    Project {
        id: p.id.clone(),
        tracker_name: (!p.tracker.is_empty()).then(|| p.name.clone()),
        issues_url: own.issues.filter(|_| !p.tracker.is_empty()),
        code_url: own.code,
        area: world
            .areas
            .values()
            .find(|a| a.projects.contains(&p.id))
            .map(|a| Area {
                id: a.id.clone(),
                name: a.name.clone(),
                issues_url: address(&a.id).issues.filter(|_| !a.tracker.is_empty()),
            }),
    }
}

fn group(g: &model::Group) -> Group {
    Group {
        id: g.id.clone(),
        name: g.name.clone(),
        projects: g.projects.clone(),
        milestones: g.milestones.clone(),
        refining_at_once: g.refining_at_once,
    }
}

fn issue(world: &World, i: &model::Issue) -> Issue {
    Issue {
        id: i.id.to_string(),
        title: i.title.clone(),
        stage: i.stage.into(),
        stage_text: text::issue_stage(world, i),
        milestone: i.milestone.clone(),
        group: i.group.clone(),
        depends_on: i.depends_on.iter().map(ToString::to_string).collect(),
        round: i.round,
        pulls: world
            .pulls_of(i)
            .into_iter()
            .map(|(repo, number)| Pull {
                repo: repo.clone(),
                number: number.clone(),
                merged: i.merged.contains(repo),
            })
            .collect(),
        review_ok: i.review_ok,
        checks_ok: i.checks_ok,
        agents: world
            .agents
            .values()
            .filter(|a| a.issue.as_ref() == Some(&i.id))
            .map(|a| a.id.clone())
            .collect(),
        updated_ms: world.updated.at(&i.id),
        checks: i.plan.iter().flat_map(|p| p.checks.clone()).collect(),
        reviews: i
            .reviews
            .iter()
            .map(|r| Review {
                round: r.round,
                reviewer: r.reviewer.clone(),
                verdict: if r.approve {
                    VerdictWord::Approve
                } else {
                    VerdictWord::Block
                },
                points: r.points.clone(),
                replies: r.replies.clone(),
            })
            .collect(),
    }
}

fn agent(world: &World, a: &model::Agent) -> Agent {
    Agent {
        id: a.id.clone(),
        group: a.group.clone(),
        role: a.role.name().into(),
        issue: a.issue.as_ref().map(ToString::to_string),
        machine: a.machine.clone(),
        folder: a.folder.clone(),
        state: (&a.state).into(),
        state_text: without_paths(world, &text::agent_state(&a.state)),
        started: a.started,
        runtime: a.runtime.clone(),
        model: a.model.clone(),
    }
}

fn decision(world: &World, d: &model::Decision) -> Decision {
    Decision {
        id: d.id.0,
        issue: d.issue.to_string(),
        by: d.by.clone(),
        question: d.question.clone(),
        options: d.options.clone(),
        chosen: d.chosen + 1,
        reason: d.reason.clone(),
        overturned_to: d
            .overturned_to
            .filter(|o| *o < d.options.len())
            .map(|o| o + 1),
        overturn_reason: d.overturn_reason.clone(),
        cost: d.cost,
        point: d.point.map(|at| DecisionPoint {
            at,
            passed: world.issues.get(&d.issue).is_some_and(|i| at.passed(i)),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use crate::event::Verdict;
    use crate::host::{Host, Memory};
    use crate::ids::{DecisionId, IssueId};
    use crate::model::{Addresses, DecisionKind, Plan, Role};
    use crate::sender::Digest;
    use crate::settings::{Scope, Value};

    /// When the errored agent's limit resets.
    const RESETS_MS: u64 = 1_795_784_700_123;
    /// The reset time another agent was last continued after.
    const CONTINUED_MS: u64 = 1_795_700_000_456;
    /// The runtime the test's agents run, and the model most of them were
    /// started with.
    const RUNTIME: &str = "hex";
    const MODEL: &str = "anthill";

    /// The time of a test's `n`th event: one a minute.
    fn minute(n: usize) -> u64 {
        1_795_000_000_000 + 60_000 * n as u64
    }

    fn added(project: &str, tracker: &str, name: &str) -> Event {
        Event::ProjectAdded {
            project: ProjectId::new(project),
            tracker: tracker.into(),
            name: name.into(),
            code_host: None,
            repo: None,
        }
    }

    fn synced(project: &str, key: &str, title: &str, depends_on: Vec<IssueId>) -> Event {
        Event::IssueSynced {
            issue: IssueId::new(project, key),
            title: title.into(),
            milestone: Some(MilestoneId::new("The Clacks")),
            depends_on,
            order: key.parse().unwrap_or_default(),
            closed: false,
        }
    }

    /// The secret digest of the `n`th agent.
    fn digest(n: u8) -> Digest {
        Digest::from_bytes(&[0xd0 + n; 32])
    }

    /// An agent: its id, role, the key of its issue in `watch`, its state, and
    /// the runtime and model it was started with.
    type Started = (
        &'static str,
        Role,
        &'static str,
        AgentState,
        Option<&'static str>,
        Option<&'static str>,
    );

    /// The agents. The refiner's settings named no model; the ended author is
    /// from a log older than runtimes on its start.
    fn agents() -> [Started; 5] {
        [
            (
                "refiner-1",
                Role::Refiner,
                "2",
                AgentState::Idle,
                Some(RUNTIME),
                None,
            ),
            (
                "author-2",
                Role::Author,
                "3",
                AgentState::Working {
                    doing: "winding the clock".into(),
                },
                Some(RUNTIME),
                Some(MODEL),
            ),
            (
                "reviewer-3",
                Role::Reviewer,
                "4",
                AgentState::Blocked {
                    reason: "permission".into(),
                },
                Some(RUNTIME),
                Some(MODEL),
            ),
            (
                "author-4",
                Role::Author,
                "5",
                AgentState::Errored {
                    line: "usage limit reached".into(),
                    resets_ms: Some(RESETS_MS),
                },
                Some(RUNTIME),
                Some(MODEL),
            ),
            ("author-5", Role::Author, "6", AgentState::Ended, None, None),
        ]
    }

    /// An arbiter's ruling on `watch#key`, with the cost and point a ruling
    /// records.
    fn ruled(id: u64, key: &str, approve: bool) -> Event {
        let (cost, point) = model::Decision::ruling_cost(approve);
        Event::DecisionMade {
            decision: model::Decision {
                id: DecisionId(id),
                issue: IssueId::new("watch", key),
                by: AgentId::new("arbiter-6"),
                kind: DecisionKind::Ruling { approve },
                question: format!("Does watch#{key} merge after 3 review rounds?"),
                options: vec!["merge".into(), "fix first".into()],
                chosen: usize::from(!approve),
                reason: "the bells were rung by the book".into(),
                overturned_to: None,
                overturn_reason: String::new(),
                cost: Some(cost),
                point: Some(point),
            },
        }
    }

    fn listed(project: &str, milestone: &str, due: Option<&str>) -> Event {
        Event::MilestoneSynced {
            project: ProjectId::new(project),
            milestone: MilestoneId::new(milestone),
            due: due.map(String::from),
            open: true,
        }
    }

    fn decided(id: u64, key: &str, by: &str, options: &[&str], chosen: usize) -> Event {
        Event::DecisionMade {
            decision: model::Decision {
                id: DecisionId(id),
                issue: IssueId::new("watch", key),
                by: AgentId::new(by),
                kind: DecisionKind::Choice,
                question: format!("How is watch#{key} to be done?"),
                options: options.iter().map(ToString::to_string).collect(),
                chosen,
                reason: "it is easier to undo".into(),
                overturned_to: None,
                overturn_reason: String::new(),
                cost: None,
                point: None,
            },
        }
    }

    /// An area `ankh-morpork` holding `watch`, a project `unseen` outside it, a
    /// group, issues in most stages and agents in every state, each event a
    /// minute after the one before. `watch`'s tracker lists two open
    /// milestones, one with a due date.
    fn world() -> World {
        let watch = |key: &str| IssueId::new("watch", key);
        let mut events = vec![
            added("watch", "fake", "ankh-morpork/watch"),
            added("unseen", "fake", "unseen/university"),
            Event::AreaDefined {
                area: ProjectId::new("ankh-morpork"),
                name: "Ankh-Morpork".into(),
                projects: vec![ProjectId::new("watch")],
                tracker: "fake".into(),
                tracker_name: "AM".into(),
            },
            Event::FolderAdded {
                folder: FolderId::new("yard-1"),
                project: ProjectId::new("watch"),
                machine: MachineId::new("pseudopolis"),
                path: FOLDER_PATH.into(),
            },
            listed("watch", "The Clacks", Some("2026-12-12")),
            listed("watch", "Snuff", None),
            Event::SettingSet {
                scope: Scope::Project(ProjectId::new("watch")),
                role: None,
                key: "deploy".into(),
                value: Value::Text("make ring-the-bells".into()),
                by: "person".into(),
            },
            Event::GroupCreated {
                group: GroupId::new("night-watch"),
                name: "Night Watch".into(),
                projects: vec![ProjectId::new("watch"), ProjectId::new("unseen")],
                areas: Vec::new(),
                milestones: vec![MilestoneId::new("The Clacks")],
                refining_at_once: 2,
                model: None,
                queue_low: None,
                investigate_every: None,
            },
            synced("watch", "1", "Count the spoons", vec![]),
            synced("watch", "2", "Light the lamps", vec![]),
            synced("watch", "3", "Wind the clock", vec![]),
            synced("watch", "4", "Ring the bells", vec![]),
            synced("watch", "5", "Sweep the yard", vec![]),
            synced("watch", "6", "Feed the swamp dragon", vec![]),
            synced("watch", "7", "Oil the gate", vec![]),
            synced("unseen", "1", "Catalogue the library", vec![watch("3")]),
        ];
        for (n, (agent, role, key, state, runtime, model)) in (1..).zip(agents()) {
            events.push(Event::AgentStarted {
                agent: AgentId::new(agent),
                group: GroupId::new("night-watch"),
                role,
                issue: Some(watch(key)),
                machine: MachineId::new("pseudopolis"),
                folder: (role == Role::Author).then(|| FolderId::new("yard-1")),
                secret_digest: Some(digest(n)),
                runtime: runtime.map(String::from),
                model: model.map(String::from),
            });
            events.push(Event::AgentChanged {
                agent: AgentId::new(agent),
                state,
            });
        }
        events.extend([
            decided(
                1,
                "3",
                "author-2",
                &["wind it nightly", "wind it weekly"],
                0,
            ),
            decided(2, "4", "reviewer-3", &["one bell", "two", "all of them"], 1),
            Event::DecisionOverturned {
                decision: DecisionId(2),
                to: 2,
                reason: "the guild pays by the bell".into(),
                by: "person".into(),
            },
            ruled(3, "4", true),
            ruled(4, "6", false),
        ]);
        let (mut w, _) =
            World::default().replay_at(events.iter().enumerate().map(|(n, e)| (minute(n), e)));
        // Each issue is put where the view is to show it, whatever the loop did
        // with it on the way.
        let mut at = |id: IssueId, stage: Stage, round: u8| {
            let i = w.issues.get_mut(&id).expect("synced");
            i.stage = stage;
            i.round = round;
            i.group = (stage != Stage::New).then(|| GroupId::new("night-watch"));
        };
        at(watch("1"), Stage::New, 0);
        at(watch("2"), Stage::Refining, 0);
        at(watch("3"), Stage::Working, 0);
        at(watch("4"), Stage::InReview { round: 2 }, 2);
        at(watch("5"), Stage::Fixing { round: 1 }, 1);
        at(watch("6"), Stage::Done, 1);
        at(watch("7"), Stage::Done, 0);
        at(IssueId::new("unseen", "1"), Stage::Ready, 0);
        for (key, number) in [("4", "41"), ("5", "52")] {
            let i = w.issues.get_mut(&watch(key)).expect("synced");
            i.pulls.insert(ProjectId::new("watch"), number.into());
            i.plan = Some(Plan {
                checks: vec![model::Check {
                    done_when: "the clock strikes thirteen".into(),
                    proven_by: "listening at midnight".into(),
                }],
                settings: [("model".to_string(), Value::Text("the-luggage".into()))].into(),
                ..Plan::default()
            });
        }
        let reviewed = w.issues.get_mut(&watch("4")).expect("synced");
        reviewed.review_ok = true;
        reviewed.checks_ok = Some(false);
        reviewed.reviews = vec![
            model::Review {
                round: 1,
                reviewer: Some(AgentId::new("reviewer-3")),
                approve: false,
                points: vec!["the bell is cracked".into(), "the rope is frayed".into()],
                replies: vec![
                    "recast the bell".into(),
                    "the rope stays: it was checked".into(),
                ],
            },
            model::Review {
                round: 2,
                reviewer: Some(AgentId::new("reviewer-3")),
                approve: true,
                points: Vec::new(),
                replies: Vec::new(),
            },
        ];
        // The addresses the tracker and code host give, put in each round.
        let address = |issues: Option<&str>, code: Option<&str>| Addresses {
            issues: issues.map(String::from),
            code: code.map(String::from),
        };
        w.addresses = [
            (
                ProjectId::new("watch"),
                address(
                    Some("https://tracker.example.com/ankh-morpork/watch/issues"),
                    Some("https://git.example.com/ankh-morpork/watch"),
                ),
            ),
            (
                ProjectId::new("ankh-morpork"),
                address(Some("https://tracker.example.com/browse/AM"), None),
            ),
        ]
        .into();
        let fixing = w.issues.get_mut(&watch("5")).expect("synced");
        fixing.fix = vec!["the broom is missing".into()];
        // One that left its group's milestones after it was taken.
        w.issues.get_mut(&watch("5")).expect("synced").milestone = None;
        w.agents
            .get_mut(&AgentId::new("author-2"))
            .expect("started")
            .continued = Some(CONTINUED_MS);
        w
    }

    fn problems() -> BTreeMap<String, String> {
        [("tracker".to_string(), "the clacks are down".to_string())].into()
    }

    const SNAPSHOT: &str = include_str!("../tests/fixtures/view.json");

    #[test]
    fn the_view_as_json_is_the_snapshot() {
        let view = View::new(&world(), &problems());
        let json = serde_json::to_string_pretty(&view).expect("json") + "\n";
        assert_eq!(json, SNAPSHOT);
        assert_eq!(
            serde_json::from_str::<View>(&json).expect("a view"),
            view,
            "it reads back"
        );
    }

    /// A host over the snapshot world after one tick, so its problems are the
    /// ones the daemon reports: among them a line for the blocked agent and one
    /// for the errored agent, each naming the work folder's path.
    fn ticked() -> Host {
        let mut host = Host::new(world(), Vec::new(), Box::new(Memory::default()));
        host.tick(0).expect("tick");
        host
    }

    const FOLDER_PATH: &str = "/srv/pseudopolis-yard/watch";

    #[test]
    fn the_json_holds_no_digest_reset_time_path_fix_or_setting() {
        let host = ticked();
        for key in ["blocked: reviewer-3", "errored: author-4"] {
            let line = &host.problems()[key];
            assert!(line.contains(FOLDER_PATH), "the host names it: {line}");
        }
        let w = host.world();
        let view = View::new(w, host.problems());
        let json = serde_json::to_string(&view).expect("json");
        // The two lines are still there, without the folder.
        assert_eq!(
            view.problems["blocked: reviewer-3"],
            "reviewer-3 is blocked: permission; the daemon cannot answer it"
        );
        assert_eq!(
            view.problems["errored: author-4"],
            "author-4 stopped on an error: usage limit reached; \
             the daemon sends continue after it resets"
        );
        let digests: Vec<String> = w
            .agents
            .values()
            .map(|a| a.secret_digest.clone().expect("every agent has one").0)
            .collect();
        assert_eq!(digests.len(), 5);
        let reset = w.agents[&AgentId::new("author-4")]
            .limit_reset()
            .expect("the errored agent has a reset time");
        // A plan's checks are in it; its settings are not.
        for line in ["the clock strikes thirteen", "listening at midnight"] {
            assert!(json.contains(line), "{line} is not in {json}");
        }
        let left_out = [
            reset.to_string(),
            CONTINUED_MS.to_string(),
            FOLDER_PATH.into(),
            "the-luggage".into(),
            "the broom is missing".into(),
            "ring-the-bells".into(),
        ];
        for value in digests.iter().chain(&left_out) {
            assert!(!json.contains(value), "{value} is in {json}");
        }
    }

    /// The view's issue `id`.
    fn issue_of<'a>(view: &'a View, id: &str) -> &'a Issue {
        view.issues
            .iter()
            .find(|i| i.id == id)
            .unwrap_or_else(|| panic!("{id} is not in the view"))
    }

    /// The view's agent `id`.
    fn agent_of<'a>(view: &'a View, id: &str) -> &'a Agent {
        view.agents
            .iter()
            .find(|a| a.id.0 == id)
            .unwrap_or_else(|| panic!("{id} is not in the view"))
    }

    #[test]
    fn each_issue_has_the_time_of_the_last_event_that_changed_it() {
        let events = [
            added("watch", "fake", "ankh-morpork/watch"),
            synced("watch", "1", "Count the spoons", vec![]),
            synced("watch", "2", "Light the lamps", vec![]),
            // Takes watch#1 for refining.
            Event::GroupCreated {
                group: GroupId::new("night-watch"),
                name: "Night Watch".into(),
                projects: vec![ProjectId::new("watch")],
                areas: Vec::new(),
                milestones: Vec::new(),
                refining_at_once: 1,
                model: None,
                queue_low: None,
                investigate_every: None,
            },
            Event::AgentStarted {
                agent: AgentId::new("refiner-1"),
                group: GroupId::new("night-watch"),
                role: Role::Refiner,
                issue: Some(IssueId::new("watch", "1")),
                machine: MachineId::new("pseudopolis"),
                folder: None,
                secret_digest: None,
                runtime: Some(RUNTIME.into()),
                model: None,
            },
            // A state change that moves no stage changes no issue.
            Event::AgentChanged {
                agent: AgentId::new("refiner-1"),
                state: AgentState::Working {
                    doing: "counting".into(),
                },
            },
            synced("watch", "2", "Light every lamp", vec![]),
            // The same again: nothing changes.
            synced("watch", "2", "Light every lamp", vec![]),
        ];
        let (w, _) =
            World::default().replay_at(events.iter().enumerate().map(|(n, e)| (minute(n), e)));
        let view = View::new(&w, &BTreeMap::new());
        assert_eq!(issue_of(&view, "watch#1").updated_ms, Some(minute(3)));
        assert_eq!(issue_of(&view, "watch#2").updated_ms, Some(minute(6)));
        // A world replayed without times has none.
        let (w, _) = World::replay(&events);
        let view = View::new(&w, &BTreeMap::new());
        assert!(view.issues.iter().all(|i| i.updated_ms.is_none()));
    }

    #[test]
    fn an_agent_shows_the_runtime_and_model_it_was_started_with() {
        let mut w = world();
        // A person changes the model while the author runs: it still runs its own.
        w.apply(&Event::SettingSet {
            scope: Scope::Project(ProjectId::new("watch")),
            role: None,
            key: "model".into(),
            value: Value::Text("octavo".into()),
            by: "person".into(),
        });
        // An agent from an older log, whose start names neither.
        let old: Event = serde_json::from_str(
            r#"{"AgentStarted":{"agent":"author-9","group":"night-watch","role":"Author","issue":{"project":"watch","key":"7"},"machine":"pseudopolis","folder":null}}"#,
        )
        .expect("an older line reads");
        w.apply(&old);
        let view = View::new(&w, &BTreeMap::new());
        let runs = |id: &str| {
            let a = agent_of(&view, id);
            (a.runtime.as_deref(), a.model.as_deref())
        };
        assert_eq!(runs("author-2"), (Some(RUNTIME), Some(MODEL)));
        assert_eq!(runs("refiner-1"), (Some(RUNTIME), None));
        assert_eq!(runs("author-9"), (None, None));
    }

    #[test]
    fn a_milestone_counts_its_issues_and_shows_the_due_date_its_tracker_gives() {
        let mut w = world();
        let of = |view: &View, project: &str, id: &str| {
            view.milestones
                .iter()
                .find(|m| m.project.0 == project && m.id.0 == id)
                .map(|m| (m.done, m.total, m.due.clone()))
        };
        let view = View::new(&w, &BTreeMap::new());
        // watch#1 to #7 but #5, which left it; #6 and #7 are done.
        assert_eq!(
            of(&view, "watch", "The Clacks"),
            Some((2, 6, Some("2026-12-12".into())))
        );
        // Listed open with no due date, and no issue in it yet.
        assert_eq!(of(&view, "watch", "Snuff"), Some((0, 0, None)));
        // Named by an issue; unseen's tracker has not listed it.
        assert_eq!(of(&view, "unseen", "The Clacks"), Some((0, 1, None)));
        // Closed in the tracker: its issues still name it, and it has no due date.
        w.apply(&Event::MilestoneSynced {
            project: ProjectId::new("watch"),
            milestone: MilestoneId::new("The Clacks"),
            due: None,
            open: false,
        });
        let view = View::new(&w, &BTreeMap::new());
        assert_eq!(of(&view, "watch", "The Clacks"), Some((2, 6, None)));
        assert_eq!(view.milestones.len(), 3);
    }

    #[test]
    fn a_ruling_has_its_cost_to_undo_and_its_point_at_the_merge() {
        let mut w = world();
        let of = |view: &View, id: u64| {
            let d = view.decisions.iter().find(|d| d.id == id).expect("listed");
            (d.cost, d.point.clone().map(|p| (p.at, p.passed)))
        };
        let view = View::new(&w, &BTreeMap::new());
        // An approval of watch#4, still in review.
        assert_eq!(
            of(&view, 3),
            (Some(Cost::High), Some((Point::Merge, false)))
        );
        // A send-back on watch#6, which is done.
        assert_eq!(of(&view, 4), (Some(Cost::Low), Some((Point::Merge, true))));
        // A choice has neither.
        assert_eq!(of(&view, 1), (None, None));
        w.issues
            .get_mut(&IssueId::new("watch", "4"))
            .expect("synced")
            .stage = Stage::Deploying;
        let view = View::new(&w, &BTreeMap::new());
        assert_eq!(of(&view, 3), (Some(Cost::High), Some((Point::Merge, true))));
        // A ruling from an older log has neither.
        let old: Event = serde_json::from_str(
            r#"{"DecisionMade":{"decision":{"id":9,"issue":{"project":"watch","key":"3"},"by":"arbiter-6","kind":{"Ruling":{"approve":true}},"question":"Does watch#3 merge?","options":["merge","fix first"],"chosen":0,"reason":"it rings true","overturned_to":null}}}"#,
        )
        .expect("an older line reads");
        w.apply(&old);
        let view = View::new(&w, &BTreeMap::new());
        assert_eq!(of(&view, 9), (None, None));
    }

    #[test]
    fn a_project_has_the_addresses_of_its_issues_and_code() {
        let view = View::new(&world(), &BTreeMap::new());
        let watch = &view
            .projects
            .iter()
            .find(|p| p.id.0 == "watch")
            .expect("watch");
        assert_eq!(
            watch.issues_url.as_deref(),
            Some("https://tracker.example.com/ankh-morpork/watch/issues")
        );
        assert_eq!(
            watch.code_url.as_deref(),
            Some("https://git.example.com/ankh-morpork/watch")
        );
        assert_eq!(
            watch.area.as_ref().and_then(|a| a.issues_url.as_deref()),
            Some("https://tracker.example.com/browse/AM")
        );
        // No part gave unseen's.
        let unseen = &view
            .projects
            .iter()
            .find(|p| p.id.0 == "unseen")
            .expect("unseen");
        assert_eq!((&unseen.issues_url, &unseen.code_url), (&None, &None));
    }

    #[test]
    fn an_issue_lists_its_plans_checks() {
        let view = View::new(&world(), &BTreeMap::new());
        assert_eq!(
            issue_of(&view, "watch#4").checks,
            [Check {
                done_when: "the clock strikes thirteen".into(),
                proven_by: "listening at midnight".into(),
            }]
        );
        // No plan yet.
        assert_eq!(issue_of(&view, "watch#1").checks, []);
    }

    #[test]
    fn an_issue_shows_its_review_rounds_from_the_review_events() {
        let watch4 = IssueId::new("watch", "4");
        let started = |agent: &str, role| Event::AgentStarted {
            agent: AgentId::new(agent),
            group: GroupId::new("night-watch"),
            role,
            issue: Some(watch4.clone()),
            machine: MachineId::new("pseudopolis"),
            folder: None,
            secret_digest: None,
            runtime: None,
            model: None,
        };
        let ready = |replies: &[&str]| Event::PullRequestReady {
            issue: watch4.clone(),
            pull: None,
            pulls: [(ProjectId::new("watch"), "41".to_string())].into(),
            replies: replies.iter().map(ToString::to_string).collect(),
        };
        let points = vec![
            "the bell is cracked".to_string(),
            "the rope is frayed".into(),
        ];
        let replies = vec![
            "recast the bell".to_string(),
            "the rope stays: it was checked".into(),
        ];
        let events = [
            added("watch", "fake", "ankh-morpork/watch"),
            Event::FolderAdded {
                folder: FolderId::new("yard-1"),
                project: ProjectId::new("watch"),
                machine: MachineId::new("pseudopolis"),
                path: FOLDER_PATH.into(),
            },
            Event::GroupCreated {
                group: GroupId::new("night-watch"),
                name: "Night Watch".into(),
                projects: vec![ProjectId::new("watch")],
                areas: Vec::new(),
                milestones: Vec::new(),
                refining_at_once: 2,
                model: None,
                queue_low: None,
                investigate_every: None,
            },
            synced("watch", "3", "Wind the clock", vec![]),
            synced("watch", "4", "Ring the bells", vec![]),
            Event::PlanWritten {
                issue: watch4.clone(),
                plan: Plan::default(),
            },
            started("author-1", Role::Author),
            // The first pull request answers no round: a reply has nowhere to go.
            ready(&["nothing to answer yet"]),
            started("reviewer-3", Role::Reviewer),
            Event::ReviewFinished {
                issue: watch4.clone(),
                round: 1,
                verdict: Verdict::Block {
                    points: points.clone(),
                },
            },
            ready(&["recast the bell", "the rope stays: it was checked"]),
            Event::ReviewFinished {
                issue: watch4.clone(),
                round: 2,
                verdict: Verdict::Approve,
            },
        ];
        let (w, _) = World::replay(&events);
        let view = View::new(&w, &BTreeMap::new());
        let round = |round, verdict, points, replies| Review {
            round,
            reviewer: Some(AgentId::new("reviewer-3")),
            verdict,
            points,
            replies,
        };
        // The block has the replies the fix that answered it sent; the approval,
        // which no fix answered yet, has none.
        assert_eq!(
            issue_of(&view, "watch#4").reviews,
            [
                round(1, VerdictWord::Block, points, replies),
                round(2, VerdictWord::Approve, Vec::new(), Vec::new())
            ]
        );
        // Never reviewed.
        assert_eq!(issue_of(&view, "watch#3").reviews, []);
    }

    #[test]
    fn a_folders_path_in_a_parts_line_or_an_agents_is_written_as_the_folders_id() {
        let mut w = world();
        // A bare name, as a folder's path may be, and a folder inside another.
        for (id, path) in [
            ("bare", "work"),
            ("inner", "/srv/pseudopolis-yard/watch/ui/"),
        ] {
            w.apply(&Event::FolderAdded {
                folder: FolderId::new(id),
                project: ProjectId::new("watch"),
                machine: MachineId::new("pseudopolis"),
                path: path.into(),
            });
        }
        w.apply(&Event::AgentChanged {
            agent: AgentId::new("author-2"),
            state: AgentState::Working {
                doing: format!("reading {FOLDER_PATH}/clock.rs and {FOLDER_PATH}/ui/face.rs"),
            },
        });
        let problems = [
            (
                "run author-2",
                format!("cannot trust {FOLDER_PATH} on pseudopolis: denied"),
            ),
            (
                "start a author on watch#3",
                "cannot trust work: no work folder".to_string(),
            ),
        ]
        .map(|(key, line)| (key.to_string(), line))
        .into();
        let view = View::new(&w, &problems);
        assert_eq!(
            view.problems["run author-2"],
            "cannot trust folder yard-1 on pseudopolis: denied"
        );
        assert_eq!(
            view.problems["start a author on watch#3"],
            "cannot trust folder bare: no work folder"
        );
        let author = view.agents.iter().find(|a| a.id.0 == "author-2");
        assert_eq!(
            author.map(|a| a.state_text.as_str()),
            Some("working: reading folder yard-1/clock.rs and folder inner/face.rs")
        );
    }

    #[test]
    fn a_stopped_agents_line_goes_when_the_agent_has_left_that_state() {
        let mut host = ticked();
        host.handle(
            1,
            Event::AgentChanged {
                agent: AgentId::new("reviewer-3"),
                state: AgentState::Ended,
            },
        )
        .expect("ended");
        // The host lists it until its next tick.
        assert!(host.problems().contains_key("blocked: reviewer-3"));
        let view = View::new(host.world(), host.problems());
        assert!(!view.problems.contains_key("blocked: reviewer-3"));
        assert!(view.problems.contains_key("errored: author-4"));
    }

    #[test]
    fn the_text_form_and_the_view_name_the_same_open_issues_and_agents() {
        let (w, problems) = (world(), problems());
        let view = View::new(&w, &problems);
        let text = text::text(&w, &problems);
        // The lines of the section under `head`, without their indent.
        let section = |head: &str| -> Vec<&str> {
            text.lines()
                .skip_while(|line| *line != head)
                .skip(1)
                .map_while(|line| line.strip_prefix("  "))
                .collect()
        };
        let first = |line: &str| {
            let word = line.split_whitespace().next().unwrap_or_default();
            word.trim_end_matches(':').to_string()
        };
        let mut issues = section("Issues");
        assert_eq!(
            issues.pop(),
            Some(format!("and {} done", view.done).as_str())
        );
        assert_eq!(view.done, 2);
        assert_eq!(
            issues.into_iter().map(first).collect::<Vec<_>>(),
            view.issues.iter().map(|i| i.id.clone()).collect::<Vec<_>>()
        );
        assert_eq!(
            section("Agents").into_iter().map(first).collect::<Vec<_>>(),
            view.agents
                .iter()
                .map(|a| a.id.0.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn every_stage_and_agent_state_has_a_word_of_its_own() {
        fn words<T: Serialize>(all: impl IntoIterator<Item = T>) -> Vec<String> {
            all.into_iter()
                .map(|w| serde_json::to_string(&w).expect("a word"))
                .map(|w| w.trim_matches('"').to_string())
                .collect()
        }
        let stages = [
            Stage::New,
            Stage::Refining,
            Stage::Ready,
            Stage::Working,
            Stage::InReview { round: 1 },
            Stage::Fixing { round: 1 },
            Stage::Arbitration { round: 1 },
            Stage::Merging,
            Stage::Deploying,
            Stage::Verifying,
            Stage::Done,
        ];
        assert_eq!(
            words(stages.map(StageWord::from)),
            [
                "new",
                "refining",
                "ready",
                "working",
                "in_review",
                "fixing",
                "arbitration",
                "merging",
                "deploying",
                "verifying",
                "done"
            ]
        );
        let states = [
            AgentState::Working {
                doing: String::new(),
            },
            AgentState::Idle,
            AgentState::Blocked {
                reason: "permission".into(),
            },
            AgentState::Errored {
                line: String::new(),
                resets_ms: None,
            },
            AgentState::Ended,
        ];
        assert_eq!(
            words(states.iter().map(StateWord::from)),
            ["working", "idle", "blocked", "errored", "ended"]
        );
    }

    #[test]
    fn an_areas_issue_lists_its_pull_requests_in_merge_order() {
        let repos = [ProjectId::new("ui"), ProjectId::new("api")];
        let mut events = vec![
            added("ui", "", "guild/ui"),
            added("api", "", "guild/api"),
            Event::AreaDefined {
                area: ProjectId::new("guild"),
                name: "The Guild".into(),
                projects: repos.to_vec(),
                tracker: "fake".into(),
                tracker_name: "GLD".into(),
            },
            synced("guild", "GLD-1", "Mend the sign", vec![]),
        ];
        let id = IssueId::new("guild", "GLD-1");
        events.push(Event::PlanWritten {
            issue: id.clone(),
            plan: Plan::default(),
        });
        let (mut w, _) = World::replay(&events);
        let i = w.issues.get_mut(&id).expect("synced");
        // `ui` merges before `api`, though `api` sorts first.
        i.plan = Some(Plan {
            repos: repos.to_vec(),
            ..Plan::default()
        });
        i.pulls = [
            (repos[1].clone(), "34".into()),
            (repos[0].clone(), "12".into()),
        ]
        .into();
        i.merged = [repos[0].clone()].into();
        let view = View::new(&w, &BTreeMap::new());
        let pull = |repo: &str, number: &str, merged| Pull {
            repo: ProjectId::new(repo),
            number: number.into(),
            merged,
        };
        assert_eq!(
            view.issues[0].pulls,
            [pull("ui", "12", true), pull("api", "34", false)]
        );
        // Its projects keep no tracker of their own.
        for p in &view.projects {
            assert_eq!(p.tracker_name, None);
            assert_eq!(p.area.as_ref().map(|a| a.name.as_str()), Some("The Guild"));
        }
    }
}
