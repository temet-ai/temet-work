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
use crate::ids::{AgentId, DecisionId, FolderId, GroupId, IssueId, MachineId, ProjectId};
use crate::launch::Launch;
use crate::model::Decision;
use crate::model::{Agent, AgentState, DecisionKind, Folder, Group, Issue, Project, Role, Stage};

/// How many agents of one role the loop starts on one issue before it gives up.
pub const MAX_STARTS: usize = 3;

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

/// How many review rounds run before an arbiter rules.
pub const REVIEW_ROUNDS: u8 = 3;

/// Everything the loop knows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct World {
    /// The log's id, once the log has one.
    pub log: Option<String>,
    /// Projects by id.
    pub projects: BTreeMap<ProjectId, Project>,
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
                self.projects.insert(
                    project.clone(),
                    Project {
                        id: project.clone(),
                        tracker: tracker.clone(),
                        name: name.clone(),
                        code_host: code_host.clone().unwrap_or_else(|| tracker.clone()),
                        repo: repo.clone().unwrap_or_else(|| name.clone()),
                    },
                );
            }
            Event::GroupCreated {
                group,
                name,
                projects,
                milestones,
                refining_at_once,
            } => {
                self.groups.insert(
                    group.clone(),
                    Group {
                        id: group.clone(),
                        name: name.clone(),
                        projects: projects.clone(),
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
                });
                entry.title = title.clone();
                entry.milestone = milestone.clone();
                entry.depends_on = depends_on.clone();
                entry.order = *order;
                if *closed && entry.stage != Stage::Done {
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
            }
            Event::AgentChanged { agent, state } => {
                if let Some(a) = self.agents.get_mut(agent) {
                    a.state = state.clone();
                }
            }
            Event::PlanWritten { issue, plan } => {
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == Stage::Refining
                {
                    i.plan = Some(plan.clone());
                    i.stage = Stage::Ready;
                }
            }
            Event::PullRequestReady { issue, pull } => {
                if let Some(i) = self.issues.get_mut(issue) {
                    if pull.is_some() {
                        i.pull = pull.clone();
                    }
                    let round = match i.stage {
                        Stage::Working => Some(1),
                        Stage::Fixing { round } => Some(round.saturating_add(1)),
                        _ => None,
                    };
                    if let Some(round) = round {
                        i.stage = Stage::InReview { round };
                        i.review_ok = false;
                        i.checks_ok = None;
                        out.push(Effect::StartReview {
                            issue: issue.clone(),
                            round,
                        });
                    }
                }
            }
            Event::ReviewFinished {
                issue,
                round,
                verdict,
            } => {
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == (Stage::InReview { round: *round })
                {
                    match verdict {
                        Verdict::Approve => {
                            i.review_ok = true;
                            Self::try_merge(i, &mut out);
                        }
                        Verdict::Block { points } if *round < REVIEW_ROUNDS => {
                            i.stage = Stage::Fixing { round: *round };
                            out.push(Effect::RequestFix {
                                issue: issue.clone(),
                                points: points.clone(),
                            });
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
                                i.stage = Stage::Fixing { round };
                                i.review_ok = false;
                                let points = if failing.is_empty() {
                                    vec!["CI failed".into()]
                                } else {
                                    failing
                                        .iter()
                                        .map(|c| format!("CI check {c} failed"))
                                        .collect()
                                };
                                out.push(Effect::RequestFix {
                                    issue: issue.clone(),
                                    points,
                                });
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
                        i.stage = Stage::Fixing { round };
                        out.push(Effect::RequestFix {
                            issue: decision.issue.clone(),
                            points: vec![decision.reason.clone()],
                        });
                    }
                }
            }
            Event::DecisionOverturned { decision, to } => {
                if let Some(d) = self.decisions.get_mut(decision)
                    && *to < d.options.len()
                    && *to != d.chosen
                    && d.overturned_to != Some(*to)
                {
                    d.overturned_to = Some(*to);
                    out.push(Effect::ApplyOverturn {
                        decision: *decision,
                    });
                    out.push(Effect::ProposeRule { from: *decision });
                }
            }
            Event::Merged { issue } => {
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == Stage::Merging
                {
                    i.stage = Stage::Deploying;
                    out.push(Effect::Deploy {
                        issue: issue.clone(),
                    });
                    let folder = i.folder.take();
                    self.release(folder, &mut out);
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
            Event::Verified { issue, passed } => {
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == Stage::Verifying
                {
                    if *passed {
                        i.stage = Stage::Done;
                        out.push(Effect::Close {
                            issue: issue.clone(),
                        });
                    } else {
                        i.stage = Stage::Ready;
                        i.review_ok = false;
                        i.checks_ok = None;
                    }
                }
            }
        }
        out.extend(self.place());
        out
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
    /// the daemon decided on and did not start before it stopped. After
    /// [`MAX_STARTS`] agents of one role on one issue it gives up, and says so.
    pub fn missing(&self) -> Missing {
        let mut out = Missing::default();
        for i in self.issues.values() {
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
                continue;
            }
            let tried = self
                .agents
                .values()
                .filter(|a| a.issue.as_ref() == Some(&i.id) && a.role == role)
                .count();
            if tried >= MAX_STARTS {
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
        let place = match folder {
            Some(f) => self.folders.get(f),
            None => self.folders.values().find(|f| f.project == issue.project),
        };
        let path = place.map(|f| std::path::PathBuf::from(&f.path));
        Some(Launch {
            agent: AgentId::new(format!("{}-{n}", role.name())),
            group: group.clone(),
            machine: place.map_or_else(|| MachineId::new(LOCAL), |f| f.machine.clone()),
            folder: folder.cloned(),
            brief: brief::write(role, i, self.projects.get(&issue.project), path),
        })
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
    fn in_scope(group: &Group, issue: &Issue) -> bool {
        group.projects.contains(&issue.id.project)
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
                .filter(|i| i.stage == Stage::New && i.group.is_none() && Self::in_scope(group, i))
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
                let free = self
                    .folders
                    .values()
                    .find(|f| f.project == id.project && f.holder.is_none())
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
