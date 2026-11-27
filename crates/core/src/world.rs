//! The work loop: state built from events, and the effects each event asks for.
//!
//! [`World::apply`] takes one event, changes the state and returns what the daemon
//! must do. After every event the loop places work: it takes the next issue for
//! refining and starts an author on a ready issue when a work folder of its
//! project is free and the issues it depends on are done. Nothing here reads a
//! clock, a file or the network, so the same events always give the same state.

use std::collections::BTreeMap;

use crate::brief;
use crate::effect::Effect;
use crate::event::{Event, Verdict};
use crate::ids::{AgentId, DecisionId, FolderId, GroupId, IssueId, MachineId, ProjectId, RuleId};
use crate::launch::Launch;
use crate::model::Decision;
use crate::model::{
    Agent, AgentState, Area, DecisionKind, Folder, Group, Issue, Project, Role, Rule, RuleState,
    Stage,
};
use crate::settings::{FILE, Scope, Value};

/// What [`World::missing`] finds.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Missing {
    /// The agents to start.
    pub start: Vec<Effect>,
    /// Issues it gave up starting agents for, in a line each.
    pub given_up: Vec<String>,
}

/// The machine an agent runs on when its project has no work folder.
pub const LOCAL: &str = "local";

/// Everything the loop knows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct World {
    /// The log's id, once the log has one.
    pub log: Option<String>,
    /// Projects by id.
    pub projects: BTreeMap<ProjectId, Project>,
    /// Areas by id.
    pub areas: BTreeMap<ProjectId, Area>,
    /// Issues by id.
    pub issues: BTreeMap<IssueId, Issue>,
    /// Groups by id.
    pub groups: BTreeMap<GroupId, Group>,
    /// Agents by id.
    pub agents: BTreeMap<AgentId, Agent>,
    /// Work folders by id.
    pub folders: BTreeMap<FolderId, Folder>,
    /// Decisions by id.
    pub decisions: BTreeMap<DecisionId, Decision>,
    /// Rules proposed from overturned decisions, by id.
    pub rules: BTreeMap<RuleId, Rule>,
    /// Settings by level, role and key (`docs/settings.md`).
    pub settings: BTreeMap<crate::settings::Key, crate::settings::Entry>,
    /// What the build's adapters offer for the settings that name one, put in by
    /// the host. It is not built from events.
    pub catalog: BTreeMap<String, crate::settings::Choice>,
    /// How many events have been applied.
    pub applied: u64,
}

impl World {
    /// Builds the state from events in order, with every effect they asked for.
    pub fn replay<'a>(events: impl IntoIterator<Item = &'a Event>) -> (Self, Vec<Effect>) {
        let mut world = Self::default();
        let mut effects = Vec::new();
        for event in events {
            effects.extend(world.apply(event));
        }
        (world, effects)
    }

    /// Applies one event and returns what the daemon must do.
    pub fn apply(&mut self, event: &Event) -> Vec<Effect> {
        self.applied += 1;
        let mut out = Vec::new();
        match event {
            Event::FolderAdded {
                folder,
                project,
                machine,
                path,
            } => {
                let f = self
                    .folders
                    .entry(folder.clone())
                    .or_insert_with(|| Folder {
                        id: folder.clone(),
                        project: project.clone(),
                        machine: machine.clone(),
                        path: path.clone(),
                        holder: None,
                    });
                // A folder that moved keeps the issue that holds it.
                f.project = project.clone();
                f.machine = machine.clone();
                f.path = path.clone();
            }
            Event::LogStarted { id } => {
                self.log.get_or_insert_with(|| id.clone());
            }
            Event::ProjectAdded {
                project,
                tracker,
                name,
                code_host,
                repo,
            } => {
                let p = self
                    .projects
                    .entry(project.clone())
                    .or_insert_with(|| Project {
                        id: project.clone(),
                        tracker: String::new(),
                        name: String::new(),
                        code_host: String::new(),
                        repo: String::new(),
                    });
                p.tracker = tracker.clone();
                p.name = name.clone();
                p.code_host = code_host.clone().unwrap_or_else(|| tracker.clone());
                p.repo = repo.clone().unwrap_or_else(|| name.clone());
            }
            Event::AreaDefined {
                area,
                name,
                projects,
                tracker,
                tracker_name,
            } => {
                self.areas.insert(
                    area.clone(),
                    Area {
                        id: area.clone(),
                        name: name.clone(),
                        projects: projects.clone(),
                        tracker: tracker.clone(),
                        tracker_name: tracker_name.clone(),
                    },
                );
            }
            Event::ProjectCommandsSet {
                project,
                deploy,
                verify,
                run_in,
                ci,
            } => {
                // Older logs: the same values, as project-level settings.
                if self.projects.contains_key(project) {
                    let scope = Scope::Project(project.clone());
                    for (key, value) in [
                        ("deploy", deploy.clone().map(Value::Text)),
                        ("verify", verify.clone().map(Value::Text)),
                        ("run_in", run_in.clone().map(Value::Text)),
                        ("ci", ci.filter(|c| !c).map(Value::Flag)),
                    ] {
                        match value {
                            Some(v) => self.put(&scope, None, key, &v, FILE),
                            None => {
                                self.settings
                                    .remove(&(scope.clone(), None, key.to_string()));
                            }
                        }
                    }
                }
            }
            Event::SettingSet {
                scope,
                role,
                key,
                value,
                by,
            } => self.put(scope, *role, key, value, by),
            Event::SettingUnset {
                scope, role, key, ..
            } => {
                self.settings.remove(&(scope.clone(), *role, key.clone()));
            }
            Event::GroupCreated {
                group,
                name,
                projects,
                areas,
                milestones,
                refining_at_once,
                model,
            } => {
                // Older logs: a group's model, as a setting on what it works on.
                if let Some(m) = model {
                    let scopes = projects
                        .iter()
                        .map(|p| Scope::Project(p.clone()))
                        .chain(areas.iter().map(|a| Scope::Area(a.clone())));
                    for scope in scopes {
                        let key = (scope.clone(), None, "model".to_string());
                        if !self.settings.contains_key(&key) {
                            self.put(&scope, None, "model", &Value::Text(m.clone()), FILE);
                        }
                    }
                }
                self.groups.insert(
                    group.clone(),
                    Group {
                        id: group.clone(),
                        name: name.clone(),
                        projects: projects.clone(),
                        areas: areas.clone(),
                        milestones: milestones.clone(),
                        refining_at_once: (*refining_at_once).max(1),
                    },
                );
            }
            Event::IssueSynced {
                issue,
                title,
                milestone,
                depends_on,
                order,
                closed,
            } => {
                let entry = self.issues.entry(issue.clone()).or_insert_with(|| Issue {
                    id: issue.clone(),
                    title: String::new(),
                    milestone: None,
                    depends_on: Vec::new(),
                    order: 0,
                    stage: Stage::New,
                    group: None,
                    folder: None,
                    plan: None,
                    review_ok: false,
                    checks_ok: None,
                    pull: None,
                    fix: Vec::new(),
                    told: false,
                    fix_keeps_round: false,
                    round: 0,
                    closed_by_loop: false,
                    review_waived: false,
                });
                entry.title = title.clone();
                entry.milestone = milestone.clone();
                entry.depends_on = depends_on.clone();
                entry.order = *order;
                if !*closed && entry.stage == Stage::Done && !entry.closed_by_loop {
                    // Closed in the tracker, now reopened there: work it again.
                    entry.stage = Stage::New;
                    entry.group = None;
                    entry.plan = None;
                    entry.review_ok = false;
                    entry.checks_ok = None;
                    entry.pull = None;
                    entry.fix.clear();
                    entry.review_waived = false;
                }
                // A pull request that closes its issue on merge must not skip deploy
                // and verify: the loop closes it itself when they pass.
                let finishing = matches!(
                    entry.stage,
                    Stage::Merging | Stage::Deploying | Stage::Verifying
                );
                if *closed && entry.stage != Stage::Done && !finishing {
                    entry.stage = Stage::Done;
                    let folder = entry.folder.take();
                    // Closed in the tracker while agents worked on it: stop them
                    // before the folder goes to the next issue.
                    out.extend(
                        self.agents
                            .values()
                            .filter(|a| {
                                a.issue.as_ref() == Some(issue) && a.state != AgentState::Ended
                            })
                            .map(|a| Effect::StopAgent {
                                agent: a.id.clone(),
                            }),
                    );
                    self.release(folder, &mut out);
                }
            }
            Event::AgentStarted {
                agent,
                group,
                role,
                issue,
                machine,
                folder,
            } => {
                self.agents.insert(
                    agent.clone(),
                    Agent {
                        id: agent.clone(),
                        group: group.clone(),
                        role: *role,
                        issue: issue.clone(),
                        machine: machine.clone(),
                        folder: folder.clone(),
                        state: AgentState::Idle,
                        started: self.applied,
                    },
                );
                // A new agent hears what its stage needs in its brief.
                if let Some(i) = issue.as_ref().and_then(|id| self.issues.get_mut(id)) {
                    let needed = matches!(
                        (i.stage, role),
                        (Stage::Fixing { .. }, Role::Author)
                            | (Stage::InReview { .. }, Role::Reviewer)
                    );
                    if needed {
                        i.told = true;
                    }
                }
            }
            Event::Told { issue } => {
                if let Some(i) = self.issues.get_mut(issue) {
                    i.told = true;
                }
            }
            Event::AgentChanged { agent, state } => {
                // An ended session stays ended: an update already on its way when it
                // was stopped must not bring it back.
                if let Some(a) = self.agents.get_mut(agent)
                    && a.state != AgentState::Ended
                {
                    a.state = state.clone();
                }
            }
            Event::PlanWritten { issue, plan } => {
                // The refiner's work is done once its plan is in.
                let done = self.stop(issue, &[Role::Refiner]);
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == Stage::Refining
                {
                    i.plan = Some(plan.clone());
                    i.stage = Stage::Ready;
                    out.extend(done);
                }
            }
            Event::PullRequestReady { issue, pull } => {
                if let Some(i) = self.issues.get_mut(issue) {
                    if pull.is_some() {
                        i.pull = pull.clone();
                    }
                    let round = match i.stage {
                        Stage::Working => Some(1),
                        Stage::Fixing { round } if i.fix_keeps_round => Some(round),
                        Stage::Fixing { round } => Some(round.saturating_add(1)),
                        _ => None,
                    };
                    if let Some(round) = round {
                        i.stage = Stage::InReview { round };
                        i.round = round;
                        // A person's overturn approved this change: no review, only the checks.
                        i.review_ok = i.review_waived;
                        i.checks_ok = None;
                        i.fix.clear();
                        i.told = false;
                        i.fix_keeps_round = false;
                        if !i.review_waived {
                            out.push(Effect::StartReview {
                                issue: issue.clone(),
                                round,
                            });
                        }
                    }
                }
            }
            Event::ReviewFinished {
                issue,
                round,
                verdict,
            } => {
                let rounds = self.setting_number(issue, None, "review_rounds");
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == (Stage::InReview { round: *round })
                {
                    match verdict {
                        Verdict::Approve => {
                            i.review_ok = true;
                            Self::try_merge(i, &mut out);
                        }
                        Verdict::Block { points } if u64::from(*round) < rounds => {
                            Self::send_back(i, *round, points.clone(), false, &mut out);
                        }
                        Verdict::Block { .. } => {
                            // An issue in review was taken by a group, so it has one.
                            if let Some(group) = i.group.clone() {
                                i.stage = Stage::Arbitration { round: *round };
                                out.push(Effect::StartAgent {
                                    group,
                                    role: Role::Arbiter,
                                    issue: issue.clone(),
                                    folder: None,
                                });
                            }
                        }
                    }
                }
            }
            Event::ChecksFinished {
                issue,
                passed,
                failing,
            } => {
                if let Some(i) = self.issues.get_mut(issue) {
                    match i.stage {
                        Stage::InReview { round } | Stage::Arbitration { round } => {
                            i.checks_ok = Some(*passed);
                            if *passed {
                                Self::try_merge(i, &mut out);
                            } else {
                                i.review_ok = false;
                                let points = if failing.is_empty() {
                                    vec!["CI failed".into()]
                                } else {
                                    failing
                                        .iter()
                                        .map(|c| format!("CI check {c} failed"))
                                        .collect()
                                };
                                Self::send_back(i, round, points, true, &mut out);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::DecisionMade { decision } => {
                self.decisions.insert(decision.id, decision.clone());
                if let DecisionKind::Ruling { approve } = decision.kind
                    && let Some(i) = self.issues.get_mut(&decision.issue)
                    && let Stage::Arbitration { round } = i.stage
                {
                    if approve {
                        i.review_ok = true;
                        Self::try_merge(i, &mut out);
                    } else {
                        Self::send_back(i, round, vec![decision.reason.clone()], false, &mut out);
                    }
                }
            }
            Event::DecisionOverturned {
                decision,
                to,
                reason,
                ..
            } => {
                if let Some(d) = self.decisions.get_mut(decision)
                    && *to < d.options.len()
                    && *to != d.chosen
                    && d.overturned_to != Some(*to)
                {
                    d.overturned_to = Some(*to);
                    d.overturn_reason.clone_from(reason);
                    if let DecisionKind::Ruling { approve } = d.kind {
                        let issue = d.issue.clone();
                        let reviewers = self.stop(&issue, &[Role::Reviewer]);
                        if let Some(i) = self.issues.get_mut(&issue) {
                            Self::overturn_ruling(i, approve, reason, reviewers, &mut out);
                        }
                    }
                }
            }
            Event::RuleProposed { rule } => {
                // One rule per decision, however often it is overturned.
                if !self.rules.values().any(|r| r.decision == rule.decision) {
                    self.rules.insert(rule.id, rule.clone());
                }
            }
            Event::RuleAnswered { rule, keep } => {
                if let Some(r) = self.rules.get_mut(rule)
                    && r.state == RuleState::Proposed
                {
                    r.state = if *keep {
                        RuleState::Kept
                    } else {
                        RuleState::Dropped
                    };
                }
            }
            Event::Merged { issue } => {
                // Merged: the author, reviewer and arbiter are done.
                let done = self.stop(issue, &[Role::Author, Role::Reviewer, Role::Arbiter]);
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == Stage::Merging
                {
                    i.stage = Stage::Deploying;
                    out.push(Effect::Deploy {
                        issue: issue.clone(),
                    });
                    out.extend(done);
                    let folder = i.folder.take();
                    self.release(folder, &mut out);
                }
            }
            Event::MergeRefused { issue, reason } => {
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == Stage::Merging
                {
                    let round = i.round.max(1);
                    let points = vec![format!("the pull request cannot merge: {reason}")];
                    Self::send_back(i, round, points, true, &mut out);
                }
            }
            Event::Retried { issue } => {
                if self
                    .issues
                    .get(issue)
                    .is_some_and(|i| matches!(i.stage, Stage::Deploying | Stage::Verifying))
                {
                    out.push(Effect::Retry {
                        issue: issue.clone(),
                    });
                }
            }
            Event::Deployed { issue } => {
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == Stage::Deploying
                {
                    i.stage = Stage::Verifying;
                    out.push(Effect::Verify {
                        issue: issue.clone(),
                    });
                }
            }
            Event::Verified {
                issue,
                passed,
                output,
            } => {
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == Stage::Verifying
                {
                    if *passed {
                        i.stage = Stage::Done;
                        i.closed_by_loop = true;
                        out.push(Effect::Close {
                            issue: issue.clone(),
                        });
                    } else {
                        // Back to work, with what failed; the merged pull request is history.
                        i.stage = Stage::Ready;
                        i.review_ok = false;
                        i.checks_ok = None;
                        i.pull = None;
                        i.review_waived = false;
                        i.fix = vec![format!(
                            "after the last change merged and deployed, the verify check failed: {output}"
                        )];
                    }
                }
            }
        }
        out.extend(self.place());
        out
    }

    /// The repository an issue is worked in: its own project, or for an issue of
    /// an area, the first repository its plan names, else the area's first.
    pub fn repo_of(&self, issue: &Issue) -> Option<&Project> {
        if let Some(p) = self.projects.get(&issue.id.project) {
            return Some(p);
        }
        let area = self.areas.get(&issue.id.project)?;
        issue
            .plan
            .as_ref()
            .and_then(|plan| plan.repos.first())
            .or_else(|| area.projects.first())
            .and_then(|id| self.projects.get(id))
    }

    /// The repositories of the area an issue comes from; empty for a project's issue.
    fn area_repos(&self, issue: &IssueId) -> Vec<ProjectId> {
        self.areas
            .get(&issue.project)
            .map(|a| a.projects.clone())
            .unwrap_or_default()
    }

    /// The tracker's name for where an issue comes from: its project's or its area's.
    pub fn source_name(&self, issue: &IssueId) -> Option<&str> {
        self.projects
            .get(&issue.project)
            .map(|p| p.name.as_str())
            .or_else(|| {
                self.areas
                    .get(&issue.project)
                    .map(|a| a.tracker_name.as_str())
            })
    }

    /// Whether `agent`'s issue is waiting on it to finish its step: a refiner while
    /// its issue is refining, an author while it is worked on, a reviewer that has
    /// heard its round, an arbiter while the issue is with the arbiter.
    pub fn waiting_on(&self, agent: &Agent) -> bool {
        let Some(i) = agent.issue.as_ref().and_then(|id| self.issues.get(id)) else {
            return false;
        };
        match (i.stage, agent.role) {
            (Stage::Refining, Role::Refiner)
            | (Stage::Working | Stage::Fixing { .. }, Role::Author)
            | (Stage::Arbitration { .. }, Role::Arbiter) => true,
            (Stage::InReview { .. }, Role::Reviewer) => i.told,
            _ => false,
        }
    }

    /// The agent of `role` working on `issue` whose session has not ended; the
    /// one started last when there are several.
    pub fn running(&self, issue: &IssueId, role: Role) -> Option<&Agent> {
        self.agents
            .values()
            .filter(|a| {
                a.issue.as_ref() == Some(issue) && a.role == role && a.state != AgentState::Ended
            })
            .max_by_key(|a| a.started)
    }

    /// The agents each issue needs now and does not have: a refiner while it is
    /// refining, an author while it is worked on, a reviewer in review, an arbiter
    /// with the arbiter. This brings back an agent whose session ended, or one
    /// the daemon decided on and did not start before it stopped. A running
    /// author or reviewer that has not heard its fixes or its new round is told
    /// again. After
    /// the `starts` setting's number of agents of one role on one issue it gives
    /// up, and says so.
    pub fn missing(&self) -> Missing {
        let mut out = Missing::default();
        for i in self.issues.values() {
            // A merge, deploy or verify lost to a restart or a failure is asked for
            // again; the parts that do them do not repeat work already done or running.
            let id = i.id.clone();
            let finishing = match i.stage {
                Stage::Merging => Some(Effect::Merge { issue: id }),
                Stage::Deploying => Some(Effect::Deploy { issue: id }),
                Stage::Verifying => Some(Effect::Verify { issue: id }),
                _ => None,
            };
            if let Some(effect) = finishing {
                out.start.push(effect);
                continue;
            }
            let Some(group) = i.group.clone() else {
                continue;
            };
            let (role, effect) = match i.stage {
                Stage::Refining => (
                    Role::Refiner,
                    Effect::StartAgent {
                        group,
                        role: Role::Refiner,
                        issue: i.id.clone(),
                        folder: None,
                    },
                ),
                Stage::Working | Stage::Fixing { .. } => (
                    Role::Author,
                    Effect::StartAgent {
                        group,
                        role: Role::Author,
                        issue: i.id.clone(),
                        folder: i.folder.clone(),
                    },
                ),
                Stage::InReview { .. } if i.review_waived => continue,
                Stage::InReview { round } => (
                    Role::Reviewer,
                    Effect::StartReview {
                        issue: i.id.clone(),
                        round,
                    },
                ),
                Stage::Arbitration { .. } => (
                    Role::Arbiter,
                    Effect::StartAgent {
                        group,
                        role: Role::Arbiter,
                        issue: i.id.clone(),
                        folder: None,
                    },
                ),
                _ => continue,
            };
            if self.running(&i.id, role).is_some() {
                // Running, but not yet told what its stage needs: say it again.
                // The part that delivers it drops a message already waiting.
                if !i.told {
                    match i.stage {
                        Stage::Fixing { .. } => out.start.push(Effect::RequestFix {
                            issue: i.id.clone(),
                            points: i.fix.clone(),
                        }),
                        Stage::InReview { round } if round > 1 => {
                            out.start.push(Effect::StartReview {
                                issue: i.id.clone(),
                                round,
                            })
                        }
                        _ => {}
                    }
                }
                continue;
            }
            let tried = self
                .agents
                .values()
                .filter(|a| a.issue.as_ref() == Some(&i.id) && a.role == role)
                .count();
            if tried as u64 >= self.setting_number(&i.id, Some(role), "starts") {
                out.given_up.push(format!(
                    "{}: {tried} {}s ended before it moved on; no more are started",
                    i.id,
                    role.name()
                ));
            } else {
                out.start.push(effect);
            }
        }
        out
    }

    /// What starting an agent takes: its id, the machine and folder it runs in,
    /// and its brief. The id is its role and a number, counting every agent of
    /// that role this world has started (`author-3`). An agent with no work
    /// folder of its own runs in any folder of its project without holding it.
    /// `None` when the issue is unknown.
    pub fn launch(
        &self,
        group: &GroupId,
        role: Role,
        issue: &IssueId,
        folder: Option<&FolderId>,
    ) -> Option<Launch> {
        let i = self.issues.get(issue)?;
        let n = self.agents.values().filter(|a| a.role == role).count() + 1;
        // An agent without a folder of its own reads in a free one when it can,
        // so it does not work in a folder an author is changing.
        let place = match folder {
            Some(f) => self.folders.get(f),
            None => {
                let repo = self.repo_of(i).map_or(&issue.project, |p| &p.id);
                self.folders
                    .values()
                    .find(|f| &f.project == repo && f.holder.is_none())
                    .or_else(|| self.folders.values().find(|f| &f.project == repo))
            }
        };
        let path = place.map(|f| std::path::PathBuf::from(&f.path));
        let shared = folder.is_none() && place.is_some_and(|f| f.holder.is_some());
        Some(Launch {
            agent: AgentId::new(format!("{}-{n}", role.name())),
            group: group.clone(),
            machine: place.map_or_else(|| MachineId::new(LOCAL), |f| f.machine.clone()),
            folder: folder.cloned(),
            brief: crate::launch::Brief {
                model: self.setting_text(issue, Some(role), "model"),
                permission: self
                    .setting_text(issue, Some(role), "permission")
                    .unwrap_or_default(),
                ..brief::write(
                    role,
                    i,
                    self.source_name(issue),
                    &self.area_repos(issue),
                    path,
                    shared,
                    &self.kept_rules(),
                )
            },
        })
    }

    /// The text of every kept rule, in the order they were proposed.
    pub fn kept_rules(&self) -> Vec<String> {
        self.rules
            .values()
            .filter(|r| r.state == RuleState::Kept)
            .map(|r| r.text.clone())
            .collect()
    }

    /// Stops the running agents of `roles` on `issue`.
    fn stop(&self, issue: &IssueId, roles: &[Role]) -> Vec<Effect> {
        self.agents
            .values()
            .filter(|a| {
                a.issue.as_ref() == Some(issue)
                    && roles.contains(&a.role)
                    && a.state != AgentState::Ended
            })
            .map(|a| Effect::StopAgent {
                agent: a.id.clone(),
            })
            .collect()
    }

    /// Sends an issue back to its author with what to fix.
    fn send_back(
        i: &mut Issue,
        round: u8,
        points: Vec<String>,
        keeps_round: bool,
        out: &mut Vec<Effect>,
    ) {
        i.stage = Stage::Fixing { round };
        i.fix.clone_from(&points);
        i.told = false;
        i.fix_keeps_round = keeps_round;
        out.push(Effect::RequestFix {
            issue: i.id.clone(),
            points,
        });
    }

    /// What a person's overturn of an arbiter's ruling does to its issue. Once the
    /// issue has merged nothing changes: the loop cannot undo a merge.
    fn overturn_ruling(
        i: &mut Issue,
        was_approve: bool,
        reason: &str,
        reviewers: Vec<Effect>,
        out: &mut Vec<Effect>,
    ) {
        if was_approve {
            // Approved, now to be fixed: back to the author, with the person's reason.
            match i.stage {
                Stage::Arbitration { round } | Stage::InReview { round } => {
                    if matches!(i.stage, Stage::InReview { .. }) {
                        out.extend(reviewers);
                    }
                    i.review_ok = false;
                    Self::send_back(i, round, vec![reason.to_string()], false, out);
                }
                Stage::Merging => {
                    i.review_ok = false;
                    let round = i.round.max(1);
                    Self::send_back(i, round, vec![reason.to_string()], false, out);
                }
                Stage::Fixing { .. } => {
                    i.fix.push(reason.to_string());
                    i.told = false;
                    out.push(Effect::RequestFix {
                        issue: i.id.clone(),
                        points: i.fix.clone(),
                    });
                }
                _ => {}
            }
        } else {
            // Sent back, now to merge: the fix finishes, then the checks decide.
            match i.stage {
                Stage::Fixing { .. } => i.review_waived = true,
                Stage::InReview { .. } => {
                    out.extend(reviewers);
                    i.review_waived = true;
                    i.review_ok = true;
                    Self::try_merge(i, out);
                }
                _ => {}
            }
        }
    }

    /// Merges when the review and the checks have both passed.
    fn try_merge(issue: &mut Issue, out: &mut Vec<Effect>) {
        if issue.review_ok && issue.checks_ok == Some(true) {
            issue.stage = Stage::Merging;
            out.push(Effect::Merge {
                issue: issue.id.clone(),
            });
        }
    }

    /// Frees a work folder for the next issue.
    fn release(&mut self, folder: Option<FolderId>, out: &mut Vec<Effect>) {
        if let Some(id) = folder
            && let Some(f) = self.folders.get_mut(&id)
        {
            f.holder = None;
            out.push(Effect::ReleaseFolder { folder: id });
        }
    }

    /// Whether every issue `issue` depends on is done. An issue the loop has not
    /// seen counts as not done.
    fn ready_to_start(&self, issue: &Issue) -> bool {
        issue
            .depends_on
            .iter()
            .all(|d| self.issues.get(d).is_some_and(|i| i.stage == Stage::Done))
    }

    /// The order a group takes issues in: its milestones first to last, then the
    /// tracker's order, then the key.
    fn rank(group: &Group, issue: &Issue) -> (usize, u64, String) {
        let milestone = issue
            .milestone
            .as_ref()
            .and_then(|m| group.milestones.iter().position(|g| g == m))
            .unwrap_or(usize::MAX);
        (milestone, issue.order, issue.id.key.clone())
    }

    /// Whether a group works on an issue: one of its projects, and one of its
    /// milestones when it names any.
    fn in_scope(&self, group: &Group, issue: &Issue) -> bool {
        let from = &issue.id.project;
        let taken = group.projects.contains(from)
            || group.areas.contains(from)
            || group
                .areas
                .iter()
                .filter_map(|a| self.areas.get(a))
                .any(|a| a.projects.contains(from));
        taken
            && (group.milestones.is_empty()
                || issue
                    .milestone
                    .as_ref()
                    .is_some_and(|m| group.milestones.contains(m)))
    }

    /// Takes new issues for refining and starts authors on ready ones.
    fn place(&mut self) -> Vec<Effect> {
        let mut out = Vec::new();
        let groups: Vec<Group> = self.groups.values().cloned().collect();
        for group in &groups {
            let refining = self
                .issues
                .values()
                .filter(|i| i.group.as_ref() == Some(&group.id) && i.stage == Stage::Refining)
                .count();
            let mut new: Vec<&Issue> = self
                .issues
                .values()
                .filter(|i| i.stage == Stage::New && i.group.is_none() && self.in_scope(group, i))
                .collect();
            new.sort_by_key(|i| Self::rank(group, i));
            let take: Vec<IssueId> = new
                .iter()
                .take(group.refining_at_once.saturating_sub(refining))
                .map(|i| i.id.clone())
                .collect();
            for id in take {
                if let Some(i) = self.issues.get_mut(&id) {
                    i.group = Some(group.id.clone());
                    i.stage = Stage::Refining;
                    out.push(Effect::StartAgent {
                        group: group.id.clone(),
                        role: Role::Refiner,
                        issue: id,
                        folder: None,
                    });
                }
            }

            let mut ready: Vec<&Issue> = self
                .issues
                .values()
                .filter(|i| {
                    i.group.as_ref() == Some(&group.id)
                        && i.stage == Stage::Ready
                        && self.ready_to_start(i)
                })
                .collect();
            ready.sort_by_key(|i| Self::rank(group, i));
            let ready: Vec<IssueId> = ready.iter().map(|i| i.id.clone()).collect();
            for id in ready {
                let repo = self
                    .issues
                    .get(&id)
                    .and_then(|i| self.repo_of(i))
                    .map_or_else(|| id.project.clone(), |p| p.id.clone());
                let free = self
                    .folders
                    .values()
                    .find(|f| f.project == repo && f.holder.is_none())
                    .map(|f| f.id.clone());
                let Some(folder) = free else { continue };
                if let (Some(i), Some(f)) =
                    (self.issues.get_mut(&id), self.folders.get_mut(&folder))
                {
                    f.holder = Some(id.clone());
                    i.folder = Some(folder.clone());
                    i.stage = Stage::Working;
                    out.push(Effect::StartAgent {
                        group: group.id.clone(),
                        role: Role::Author,
                        issue: id,
                        folder: Some(folder),
                    });
                }
            }
        }
        out
    }
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "world_settings_tests.rs"]
mod settings_scope;
