//! The loop's state as data, for a front end: what the text form lists, in the
//! same order, as plain fields that need none of the loop's logic to draw.
//!
//! The desktop app reads it, and `temet-work status --json` prints it. It is
//! built from named fields, so what the log holds for the loop alone stays out:
//! an agent's secret digest and limit marks, folder paths, plans, what an author
//! is to fix, settings, rules and the waiting list.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::{AgentId, FolderId, GroupId, MachineId, MilestoneId, ProjectId};
use crate::model::{self, AgentState, Stage};
use crate::text;
use crate::world::World;

/// The state, with what is wrong now. Done issues are counted, not listed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    /// The projects, by id.
    pub projects: Vec<Project>,
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
    /// finish.
    pub checks_ok: Option<bool>,
    /// The agents on it, ended ones too.
    pub agents: Vec<AgentId>,
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
}

impl View {
    /// The view of `world`, with what is wrong now.
    pub fn new(world: &World, problems: &BTreeMap<String, String>) -> Self {
        Self {
            projects: world.projects.values().map(|p| project(world, p)).collect(),
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
            agents: world.agents.values().map(agent).collect(),
            decisions: world.decisions.values().map(decision).collect(),
            problems: problems.clone(),
        }
    }
}

fn project(world: &World, p: &model::Project) -> Project {
    Project {
        id: p.id.clone(),
        tracker_name: (!p.tracker.is_empty()).then(|| p.name.clone()),
        area: world
            .areas
            .values()
            .find(|a| a.projects.contains(&p.id))
            .map(|a| Area {
                id: a.id.clone(),
                name: a.name.clone(),
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
    }
}

fn agent(a: &model::Agent) -> Agent {
    Agent {
        id: a.id.clone(),
        group: a.group.clone(),
        role: a.role.name().into(),
        issue: a.issue.as_ref().map(ToString::to_string),
        machine: a.machine.clone(),
        folder: a.folder.clone(),
        state: (&a.state).into(),
        state_text: text::agent_state(&a.state),
        started: a.started,
    }
}

fn decision(d: &model::Decision) -> Decision {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use crate::ids::{DecisionId, IssueId};
    use crate::model::{DecisionKind, Plan, Role};
    use crate::sender::Digest;
    use crate::settings::{Scope, Value};

    /// When the errored agent's limit resets.
    const RESETS_MS: u64 = 1_795_784_700_123;
    /// The reset time another agent was last continued after.
    const CONTINUED_MS: u64 = 1_795_700_000_456;

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

    /// The agents: id, role, the key of its issue in `watch`, and its state.
    fn agents() -> [(&'static str, Role, &'static str, AgentState); 5] {
        [
            ("refiner-1", Role::Refiner, "2", AgentState::Idle),
            (
                "author-2",
                Role::Author,
                "3",
                AgentState::Working {
                    doing: "winding the clock".into(),
                },
            ),
            (
                "reviewer-3",
                Role::Reviewer,
                "4",
                AgentState::Blocked {
                    reason: "permission".into(),
                },
            ),
            (
                "author-4",
                Role::Author,
                "5",
                AgentState::Errored {
                    line: "usage limit reached".into(),
                    resets_ms: Some(RESETS_MS),
                },
            ),
            ("author-5", Role::Author, "6", AgentState::Ended),
        ]
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
            },
        }
    }

    /// An area `ankh-morpork` holding `watch`, a project `unseen` outside it, a
    /// group, issues in most stages and agents in every state.
    fn world() -> World {
        let watch = |key: &str| IssueId::new("watch", key);
        let mut events = vec![
            added("watch", "fake", "ankh-morpork/watch"),
            added("unseen", "fake", "unseen/university"),
            Event::AreaDefined {
                area: ProjectId::new("ankh-morpork"),
                name: "Ankh-Morpork".into(),
                projects: vec![ProjectId::new("watch")],
                tracker: String::new(),
                tracker_name: String::new(),
            },
            Event::FolderAdded {
                folder: FolderId::new("yard-1"),
                project: ProjectId::new("watch"),
                machine: MachineId::new("pseudopolis"),
                path: "/srv/pseudopolis-yard/watch".into(),
            },
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
        for (n, (agent, role, key, state)) in (1..).zip(agents()) {
            events.push(Event::AgentStarted {
                agent: AgentId::new(agent),
                group: GroupId::new("night-watch"),
                role,
                issue: Some(watch(key)),
                machine: MachineId::new("pseudopolis"),
                folder: (role == Role::Author).then(|| FolderId::new("yard-1")),
                secret_digest: Some(digest(n)),
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
        ]);
        let (mut w, _) = World::replay(&events);
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
                ..Plan::default()
            });
        }
        let reviewed = w.issues.get_mut(&watch("4")).expect("synced");
        reviewed.review_ok = true;
        reviewed.checks_ok = Some(false);
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

    #[test]
    fn the_json_holds_no_digest_reset_time_path_plan_fix_or_setting() {
        let w = world();
        let json = serde_json::to_string(&View::new(&w, &problems())).expect("json");
        let digests: Vec<String> = w
            .agents
            .values()
            .map(|a| a.secret_digest.clone().expect("every agent has one").0)
            .collect();
        assert_eq!(digests.len(), 5);
        let reset = w.agents[&AgentId::new("author-4")]
            .limit_reset()
            .expect("the errored agent has a reset time");
        let left_out = [
            reset.to_string(),
            CONTINUED_MS.to_string(),
            "/srv/pseudopolis-yard".into(),
            "the clock strikes thirteen".into(),
            "listening at midnight".into(),
            "the broom is missing".into(),
            "ring-the-bells".into(),
        ];
        for value in digests.iter().chain(&left_out) {
            assert!(!json.contains(value), "{value} is in {json}");
        }
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
