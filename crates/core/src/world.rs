//! The work loop: state built from events, and the effects each event asks for.
//!
//! [`World::apply`] takes one event, changes the state and returns what the daemon
//! must do. After every event the loop places work: it takes the next issue for
//! refining and starts an author on a ready issue when a work folder of its
//! project is free, on the machine its `machine` setting names, and the issues it
//! depends on are done. An issue that leaves its group's scope gets no new agent,
//! fix, review or merge until it comes back ([`World::paused`]). Nothing here
//! reads a clock, a file or the network, so the same events always give the same
//! state.

use std::collections::{BTreeMap, BTreeSet};

use crate::brief;
use crate::effect::Effect;
use crate::event::{Event, Verdict};
use crate::ids::{
    AgentId, DecisionId, FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId, RuleId,
};
use crate::launch::Launch;
use crate::model::Decision;
use crate::model::{
    Addresses, Agent, AgentState, Area, CheckState, ClosedPull, DecisionKind, Folder, Group,
    INVESTIGATE_EVERY, Investigation, Issue, PointState, Project, Review, Role, Rule, RuleState,
    Stage,
};
use crate::placement::Unlisted;
use crate::settings::{self, FILE, Scope, Value};

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

/// How many agents of one role whose programs were lost one issue may wait on,
/// for each start the `starts` setting allows, before [`World::missing`] gives
/// up on it. Such ends do not use up `starts`, so a reboot costs no start, but
/// a program lost at the same point every run, as to an out-of-memory kill, is
/// started at most three times `starts` (nine by default).
pub const LOST_PER_START: u64 = 3;

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
    /// Each group's investigation: the one running, else its last.
    pub investigations: BTreeMap<GroupId, Investigation>,
    /// Settings by level, role and key (`docs/settings.md`).
    pub settings: BTreeMap<crate::settings::Key, crate::settings::Entry>,
    /// The `model.<runtime>` an older log's last plain `model` at each scope and
    /// role was bound to, so its next plain change there takes that value away
    /// after the global runtime has changed. Built from the log as it replays.
    pub plain_models: BTreeMap<(Scope, Option<Role>), String>,
    /// What the build's adapters offer for the settings that name one, put in by
    /// the host. It is not built from events.
    pub catalog: BTreeMap<String, crate::settings::Choice>,
    /// The open milestones each project's or area's tracker last listed, with
    /// each one's due date as `YYYY-MM-DD` when the tracker gives one.
    pub milestones: BTreeMap<ProjectId, BTreeMap<MilestoneId, Option<String>>>,
    /// Each project's and area's web addresses, put in by the host each round
    /// from what the parts give. It is not built from events.
    pub addresses: BTreeMap<ProjectId, Addresses>,
    /// When each issue last changed, from the times events were applied at.
    pub updated: Updated,
    /// Whether a person has held the daemon (`temet-work hold`, or
    /// `temet-workd --hold`) and not resumed it. While held, no issue is taken
    /// for refining, no author is placed, no investigation starts, every issue is
    /// paused ([`World::paused`]) and nothing waits on an investigator or an
    /// agent of a paused issue ([`World::waiting_on`]). Agents at work finish
    /// their turn, and an issue with a pull request merged still merges the rest,
    /// deploys and is verified.
    pub held: bool,
    /// When the hold began, in milliseconds since 1970: the time its event was
    /// applied at ([`World::apply_at`]). None while the daemon is not held, and
    /// in a world replayed without times.
    pub held_since: Option<u64>,
    /// How many events have been applied.
    pub applied: u64,
}

/// When each issue last changed: the time of the last event after which its
/// record differed. Only events applied with a time ([`World::apply_at`]) set
/// one; a world replayed without times has none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Updated {
    /// The time, in milliseconds since 1970, by issue.
    at: BTreeMap<IssueId, u64>,
    /// A mark of each issue's record after the last event, to tell whether the
    /// next one changed it ([`mark`]).
    marks: BTreeMap<IssueId, u64>,
}

impl Updated {
    /// When `issue` last changed, in milliseconds since 1970; none when no event
    /// with a time changed it.
    pub fn at(&self, issue: &IssueId) -> Option<u64> {
        self.at.get(issue).copied()
    }

    /// Marks every issue as it is now, and stamps `at_ms` on each whose mark
    /// changed, when there is a time.
    fn note(&mut self, issues: &BTreeMap<IssueId, Issue>, at_ms: Option<u64>) {
        for (id, issue) in issues {
            let m = mark(issue);
            let changed = match self.marks.get_mut(id) {
                Some(old) if *old == m => false,
                Some(old) => {
                    *old = m;
                    true
                }
                None => {
                    self.marks.insert(id.clone(), m);
                    true
                }
            };
            if let (true, Some(at)) = (changed, at_ms) {
                self.at.insert(id.clone(), at);
            }
        }
    }
}

/// A mark of an issue's record: two records with different marks differ. Text
/// that changes only together with the issue's stage or with a count, its plan,
/// what it is to fix, its review points and the author's replies, is marked by
/// its count, so marking every issue after every event stays cheap. The checks'
/// and points' states are short, so they are marked whole.
fn mark(i: &Issue) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::hash::DefaultHasher::new();
    (&i.id, &i.title, &i.milestone, &i.depends_on, i.order).hash(&mut h);
    (i.stage, &i.group, &i.folder, &i.other_folders).hash(&mut h);
    i.plan
        .as_ref()
        .map(|p| (p.checks.len(), p.repos.len(), p.settings.len()))
        .hash(&mut h);
    (i.review_ok, i.checks_ok, &i.pulls, &i.merged, i.fix.len()).hash(&mut h);
    (&i.branches, &i.closed).hash(&mut h);
    (i.told, i.fix_keeps_round, i.round, i.closed_by_loop).hash(&mut h);
    let replies: usize = i.reviews.iter().map(|r| r.replies.len()).sum();
    (i.has_merged, i.review_waived, i.reviews.len(), replies).hash(&mut h);
    for c in i.plan.iter().flat_map(|p| &p.checks) {
        c.state.hash(&mut h);
    }
    for r in &i.reviews {
        r.states.hash(&mut h);
    }
    (i.verify_failed, i.reopen_due).hash(&mut h);
    h.finish()
}

impl World {
    /// Builds the state from events in order, with every effect they asked for.
    pub fn replay<'a>(events: impl IntoIterator<Item = &'a Event>) -> (Self, Vec<Effect>) {
        Self::default().replay_onto(events)
    }

    /// Builds the state from events in order onto this world, with every effect
    /// they asked for. A world that already holds the build's catalog binds an
    /// older log's plain model to the build's default runtime where the log had
    /// set no global one.
    pub fn replay_onto<'a>(
        mut self,
        events: impl IntoIterator<Item = &'a Event>,
    ) -> (Self, Vec<Effect>) {
        let mut effects = Vec::new();
        for event in events {
            effects.extend(self.apply(event));
        }
        (self, effects)
    }

    /// Builds the state from events in order onto this world, each applied at
    /// the time it was logged with ([`World::apply_at`]), with every effect they
    /// asked for. A log replayed this way gives each issue the time it last
    /// changed, the same as when its events were first applied.
    pub fn replay_at<'a>(
        mut self,
        events: impl IntoIterator<Item = (u64, &'a Event)>,
    ) -> (Self, Vec<Effect>) {
        let mut effects = Vec::new();
        for (at_ms, event) in events {
            effects.extend(self.apply_at(at_ms, event));
        }
        (self, effects)
    }

    /// Applies one event and returns what the daemon must do. It gives no issue
    /// a time it changed: [`World::apply_at`] does.
    pub fn apply(&mut self, event: &Event) -> Vec<Effect> {
        let out = self.change(event);
        self.updated.note(&self.issues, None);
        out
    }

    /// Applies one event that happened at `at_ms`, in milliseconds since 1970,
    /// and returns what the daemon must do. Each issue whose record it changed
    /// last changed at `at_ms`; an event that changes no issue leaves the times
    /// alone. A hold applied at `at_ms` began then ([`World::held_since`]).
    pub fn apply_at(&mut self, at_ms: u64, event: &Event) -> Vec<Effect> {
        let out = self.change(event);
        self.updated.note(&self.issues, Some(at_ms));
        if *event == Event::Held && self.held_since.is_none() {
            self.held_since = Some(at_ms);
        }
        out
    }

    /// Changes the state for one event and returns what the daemon must do.
    fn change(&mut self, event: &Event) -> Vec<Effect> {
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
            } => {
                // Older logs hold plain models: each is the global runtime's here,
                // and replaces the plain model set before at that place.
                let bound = self.bound_key(key);
                if let Some(role) = settings::kept_role(scope, *role, &bound, by) {
                    if key == "model" {
                        self.drop_plain_model(scope, role, by);
                        self.plain_models
                            .insert((scope.clone(), role), bound.clone());
                    }
                    self.overturn_agents_value(scope, role, &bound, by);
                    self.put(scope, role, &bound, value, by);
                }
            }
            Event::SettingUnset {
                scope,
                role,
                key,
                by,
            } => {
                // An older log's plain unset takes away the model its plain set
                // there was bound to; with none, the global runtime's here.
                let bound = self.bound_key(key);
                if let Some(role) = settings::kept_role(scope, *role, &bound, by) {
                    let plain = (key == "model")
                        .then(|| self.drop_plain_model(scope, role, by))
                        .flatten();
                    self.overturn_agents_value(scope, role, &bound, by);
                    self.settings
                        .remove(&(scope.clone(), role, plain.unwrap_or(bound)));
                }
            }
            Event::GroupCreated {
                group,
                name,
                projects,
                areas,
                milestones,
                refining_at_once,
                model,
                queue_low,
                investigate_every,
            } => {
                // Older logs: a group's model, as a plain model on what it works
                // on, for the runtime that is global here.
                if let Some(m) = model {
                    let key = self.bound_key("model");
                    let scopes = projects
                        .iter()
                        .map(|p| Scope::Project(p.clone()))
                        .chain(areas.iter().map(|a| Scope::Area(a.clone())));
                    for scope in scopes {
                        if !self
                            .settings
                            .contains_key(&(scope.clone(), None, key.clone()))
                        {
                            self.put(&scope, None, &key, &Value::Text(m.clone()), FILE);
                            self.plain_models.insert((scope, None), key.clone());
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
                        queue_low: *queue_low,
                        investigate_every: investigate_every.unwrap_or(INVESTIGATE_EVERY),
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
                    other_folders: Vec::new(),
                    plan: None,
                    review_ok: false,
                    checks_ok: None,
                    pulls: BTreeMap::new(),
                    merged: BTreeSet::new(),
                    branches: BTreeMap::new(),
                    closed: BTreeMap::new(),
                    fix: Vec::new(),
                    told: false,
                    fix_keeps_round: false,
                    round: 0,
                    closed_by_loop: false,
                    has_merged: false,
                    verify_failed: false,
                    reopen_due: false,
                    review_waived: false,
                    reviews: Vec::new(),
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
                    entry.pulls.clear();
                    entry.merged.clear();
                    entry.branches.clear();
                    entry.closed.clear();
                    entry.has_merged = false;
                    entry.fix.clear();
                    entry.review_waived = false;
                    entry.reviews.clear();
                } else if !*closed && entry.reopen_due {
                    // Open in the tracker after a failed verify: the reopen has
                    // happened, whoever made it, so a later close is a person's.
                    entry.reopen_due = false;
                    entry.has_merged = false;
                }
                // A pull request that closes its issue on merge must not skip deploy
                // and verify: the loop closes it itself when they pass.
                // Nor must one that closes it when only the first of several merged.
                if *closed && entry.stage != Stage::Done && !entry.finishing() {
                    entry.stage = Stage::Done;
                    entry.verify_failed = false;
                    entry.reopen_due = false;
                    let folders = entry.take_folders();
                    // Closed in the tracker while agents worked on it: stop them.
                    // No other author is given the folders until this issue's
                    // author has ended (`World::in_use`).
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
                    self.release(folders, &mut out);
                }
            }
            Event::MilestoneSynced {
                project,
                milestone,
                due,
                open,
            } => {
                let listed = self.milestones.entry(project.clone()).or_default();
                if *open {
                    listed.insert(milestone.clone(), due.clone());
                } else {
                    listed.remove(milestone);
                }
                if listed.is_empty() {
                    self.milestones.remove(project);
                }
            }
            Event::AgentStarted {
                agent,
                group,
                role,
                issue,
                machine,
                folder,
                secret_digest,
                runtime,
                model,
            } => {
                // An author holds its issue's folders, one per repository; an
                // older log's author may name none of them.
                let held = match issue.as_ref().and_then(|id| self.issues.get(id)) {
                    _ if *role != Role::Author => Vec::new(),
                    Some(i) if folder.is_none() || *folder == i.folder => {
                        i.folder.iter().chain(&i.other_folders).cloned().collect()
                    }
                    _ => folder.iter().cloned().collect(),
                };
                self.agents.insert(
                    agent.clone(),
                    Agent {
                        id: agent.clone(),
                        group: group.clone(),
                        role: *role,
                        issue: issue.clone(),
                        machine: machine.clone(),
                        folder: folder.clone(),
                        held,
                        state: AgentState::Idle,
                        started: self.applied,
                        secret_digest: secret_digest.clone(),
                        continued: None,
                        ended_waited_on: false,
                        lost_waited_on: false,
                        runtime: runtime.clone(),
                        model: model.clone(),
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
            Event::AgentChanged { agent, state, lost } => {
                let waited_on = self.agents.get(agent).is_some_and(|a| self.waiting_on(a));
                // An ended session stays ended: an update already on its way when it
                // was stopped must not bring it back.
                if let Some(a) = self.agents.get_mut(agent)
                    && a.state != AgentState::Ended
                {
                    a.state = state.clone();
                    let ended = *state == AgentState::Ended && waited_on;
                    a.ended_waited_on = ended && !lost;
                    a.lost_waited_on = ended && *lost;
                }
            }
            Event::Continued { agent, resets_ms } => {
                if let Some(a) = self.agents.get_mut(agent)
                    && a.state != AgentState::Ended
                {
                    a.state = AgentState::Working {
                        doing: String::new(),
                    };
                    a.continued = Some(*resets_ms);
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
            Event::PullRequestReady {
                issue,
                pull,
                pulls,
                replies,
            } => {
                let first = self
                    .issues
                    .get(issue)
                    .and_then(|i| self.repos_of(i).into_iter().next());
                if let Some(i) = self.issues.get_mut(issue) {
                    // The author's replies go on the block this ready answers.
                    if i.block_to_answer().is_some()
                        && let Some(r) = i.reviews.last_mut()
                    {
                        r.replies.extend(replies.iter().cloned());
                    }
                    // Older logs name one pull request: the first repository's.
                    let old = pull.clone().zip(first).map(|(n, repo)| (repo, n));
                    for (repo, number) in old
                        .into_iter()
                        .chain(pulls.iter().map(|(r, n)| (r.clone(), n.clone())))
                    {
                        // A new pull request where one merged is a fix that has yet
                        // to merge, on a branch the code host has yet to report.
                        if i.pulls.insert(repo.clone(), number.clone()) != Some(number) {
                            i.merged.remove(&repo);
                            i.branches.remove(&repo);
                            // A person's overturn approved the change on
                            // record, not one in another pull request.
                            i.review_waived = false;
                        }
                        // A reopened or new one takes the place of one a person closed.
                        i.closed.remove(&repo);
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
                            // The author reports that every check holds, all at
                            // once; CI and the reviewer check that now. With the
                            // review waived no one will judge them, so they keep
                            // their state.
                            i.set_checks(CheckState::Running);
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
                failed,
                open,
            } => {
                let rounds = self.setting_number(issue, None, "review_rounds");
                let arbiter_running = self.running(issue, Role::Arbiter).is_some();
                let reviewer = self.running(issue, Role::Reviewer).map(|a| a.id.clone());
                let arbiters = self.stop(issue, &[Role::Arbiter]);
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == (Stage::InReview { round: *round })
                {
                    Self::judge(i, *round, failed.as_deref(), open.as_deref());
                    let points = match verdict {
                        Verdict::Approve => Vec::new(),
                        Verdict::Block { points } => points.clone(),
                    };
                    i.reviews.push(Review {
                        round: *round,
                        reviewer,
                        approve: matches!(verdict, Verdict::Approve),
                        // A point is open until the next round judges it.
                        states: match open {
                            Some(_) => vec![PointState::Open; points.len()],
                            None => Vec::new(),
                        },
                        points,
                        replies: Vec::new(),
                    });
                    // The latest verdict counts: a block takes an approval back.
                    i.review_ok = matches!(verdict, Verdict::Approve);
                    match verdict {
                        Verdict::Approve => Self::try_merge(i, &mut out),
                        Verdict::Block { points } if u64::from(*round) < rounds => {
                            Self::send_back(i, *round, points.clone(), false, arbiters, &mut out);
                        }
                        Verdict::Block { .. } => {
                            // An issue in review was taken by a group, so it has one.
                            if let Some(group) = i.group.clone() {
                                i.stage = Stage::Arbitration { round: *round };
                                // One arbiter at a time. One still running, because
                                // a stop failed or an older log left it, is reminded
                                // and stopped like any idle agent, or, once it has
                                // ruled, asked again to stop by `World::missing`. A
                                // fresh one starts once it has ended.
                                if !arbiter_running {
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
            }
            Event::ChecksFinished {
                issue,
                passed,
                failing,
            } => {
                let arbiters = self.stop(issue, &[Role::Arbiter]);
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
                                Self::send_back(i, round, points, true, arbiters, &mut out);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::DecisionMade { decision } => {
                self.decisions.insert(decision.id, decision.clone());
                // A choice is only recorded: the agent that made it carries on,
                // and no stage or agent changes.
                if let DecisionKind::Ruling { approve } = decision.kind {
                    // The arbiter's work is done once its ruling is in; a later
                    // arbitration starts a fresh one, which hears that dispute.
                    let done = self.stop(&decision.issue, &[Role::Arbiter]);
                    if let Some(i) = self.issues.get_mut(&decision.issue)
                        && let Stage::Arbitration { round } = i.stage
                    {
                        if approve {
                            out.extend(done);
                            i.review_ok = true;
                            Self::try_merge(i, &mut out);
                        } else {
                            let points = vec![decision.reason.clone()];
                            Self::send_back(i, round, points, false, done, &mut out);
                        }
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
                        let arbiters = self.stop(&issue, &[Role::Arbiter]);
                        if let Some(i) = self.issues.get_mut(&issue) {
                            Self::overturn_ruling(
                                i, approve, reason, reviewers, arbiters, &mut out,
                            );
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
            Event::Merged { issue, repo } => {
                let repos = self
                    .issues
                    .get(issue)
                    .map(|i| self.repos_of(i))
                    .unwrap_or_default();
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == Stage::Merging
                {
                    i.has_merged = true;
                    match repo {
                        Some(r) => {
                            i.merged.insert(r.clone());
                        }
                        None => i.merged.extend(repos.iter().cloned()),
                    }
                    // An issue from an older log may name repositories it has no
                    // pull request in; only those it has one in merge.
                    if repos
                        .iter()
                        .all(|r| i.merged.contains(r) || !i.pulls.contains_key(r))
                    {
                        // Merged everywhere: the author, reviewer and arbiter are done.
                        // The folders go to no other author until the author has
                        // ended (`World::in_use`).
                        i.stage = Stage::Deploying;
                        let folders = i.take_folders();
                        out.push(Effect::Deploy {
                            issue: issue.clone(),
                        });
                        out.extend(
                            self.stop(issue, &[Role::Author, Role::Reviewer, Role::Arbiter]),
                        );
                        self.release(folders, &mut out);
                    } else {
                        // The next repository's pull request merges after this one's.
                        out.push(Effect::Merge {
                            issue: issue.clone(),
                        });
                    }
                }
            }
            Event::PullConflicts { issue, repo } => {
                let arbiters = self.stop(issue, &[Role::Arbiter]);
                if let Some(i) = self.issues.get_mut(issue)
                    && let Stage::InReview { round } | Stage::Arbitration { round } = i.stage
                {
                    i.review_ok = false;
                    let which = repo
                        .as_ref()
                        .filter(|_| i.pulls.len() > 1)
                        .map_or_else(String::new, |r| format!(" in {r}"));
                    // An older log names no repository: its one pull request's.
                    let branch = repo
                        .as_ref()
                        .or_else(|| i.pulls.keys().next().filter(|_| i.pulls.len() == 1))
                        .and_then(|r| i.branches.get(r))
                        .map_or_else(|| "your own branch".to_string(), Clone::clone);
                    let points = vec![format!(
                        "the pull request{which} conflicts with the branch it merges into: rebase \
                         onto that branch, then push {branch} with `git push --force-with-lease`"
                    )];
                    Self::send_back(i, round, points, true, arbiters, &mut out);
                }
            }
            Event::PullBranch {
                issue,
                repo,
                number,
                branch,
            } => {
                // Only the branch of the pull request on record: a late read of
                // one since closed or replaced is history.
                if let Some(i) = self.issues.get_mut(issue)
                    && i.pulls.get(repo) == Some(number)
                {
                    i.branches.insert(repo.clone(), branch.clone());
                }
            }
            Event::PullClosed {
                issue,
                repo,
                number,
            } => {
                let repos = self
                    .issues
                    .get(issue)
                    .map(|i| self.repos_of(i))
                    .unwrap_or_default();
                // A running reviewer or arbiter was judging the closed one: a
                // fresh one starts once ready names its replacement. One stopped
                // now ends while the issue is fixed, so it does not count against
                // `starts`.
                let reviewers = self.stop(issue, &[Role::Reviewer]);
                let arbiters = self.stop(issue, &[Role::Arbiter]);
                if let Some(i) = self.issues.get_mut(issue)
                    && i.pulls.get(repo) == Some(number)
                    && let Some(round) = match i.stage {
                        Stage::InReview { round }
                        | Stage::Arbitration { round }
                        | Stage::Fixing { round } => Some(round),
                        Stage::Merging => Some(i.round.max(1)),
                        _ => None,
                    }
                {
                    // Dropped, so that ready asks for the number of the one the
                    // author reopens or opens in its place.
                    i.pulls.remove(repo);
                    i.merged.remove(repo);
                    let branch = i.branches.remove(repo);
                    i.closed.insert(
                        repo.clone(),
                        ClosedPull {
                            number: number.clone(),
                            branch,
                        },
                    );
                    let merged: Vec<String> = repos
                        .iter()
                        .filter(|r| i.merged.contains(*r))
                        .map(ToString::to_string)
                        .collect();
                    let (which, there) = if repos.len() > 1 {
                        (format!(" in {repo}"), " there".to_string())
                    } else {
                        (String::new(), String::new())
                    };
                    let before = match merged.len() {
                        0 => String::new(),
                        1 => format!("{} is merged; ", merged[0]),
                        _ => format!("{} are merged; ", merged.join(", ")),
                    };
                    // It cannot be reopened once its branch is deleted, as when the
                    // code host closed it for that reason.
                    let point = format!(
                        "{before}pull request {number}{which} was closed without merging: reopen \
                         it if its branch still exists, or open a new one{there}"
                    );
                    i.review_ok = false;
                    // Even a reopen is reviewed: the closed branch can take new
                    // commits, and the record does not say which ones a person
                    // approved.
                    i.review_waived = false;
                    out.extend(reviewers);
                    if matches!(i.stage, Stage::Fixing { .. }) {
                        // Already back with its author, as after a failed check: one
                        // more point, in the same stage and round.
                        out.extend(arbiters);
                        i.fix.push(point);
                        i.told = false;
                        out.push(Effect::RequestFix {
                            issue: issue.clone(),
                            points: i.fix.clone(),
                        });
                    } else {
                        Self::send_back(i, round, vec![point], true, arbiters, &mut out);
                    }
                }
            }
            Event::MergeRefused { issue, reason } => {
                let repos = self
                    .issues
                    .get(issue)
                    .map(|i| self.repos_of(i))
                    .unwrap_or_default();
                let arbiters = self.stop(issue, &[Role::Arbiter]);
                if let Some(i) = self.issues.get_mut(issue)
                    && i.stage == Stage::Merging
                {
                    let round = i.round.max(1);
                    let merged: Vec<String> = repos
                        .iter()
                        .filter(|r| i.merged.contains(r))
                        .map(ToString::to_string)
                        .collect();
                    let next = repos.iter().find(|r| !i.merged.contains(r));
                    let point = match (next, repos.len()) {
                        (Some(next), n) if n > 1 && merged.is_empty() => {
                            format!("{next} cannot merge: {reason}")
                        }
                        (Some(next), n) if n > 1 => format!(
                            "{} {} merged, {next} cannot merge: {reason}",
                            merged.join(", "),
                            if merged.len() == 1 { "is" } else { "are" },
                        ),
                        _ => format!("the pull request cannot merge: {reason}"),
                    };
                    Self::send_back(i, round, vec![point], true, arbiters, &mut out);
                }
            }
            Event::InvestigationStarted {
                group,
                agent,
                at_ms,
            } => {
                let open = self
                    .investigations
                    .get(group)
                    .is_some_and(Investigation::running);
                if self.groups.contains_key(group) && !open {
                    let last = self.investigations.get(group);
                    // The window moves on only past an investigation that
                    // finished: one that did not may have read nothing.
                    let since_ms = last.and_then(|i| {
                        if i.done {
                            Some(i.started_ms)
                        } else {
                            i.since_ms
                        }
                    });
                    let first_started_ms = last.map_or(*at_ms, |i| i.first_started_ms);
                    self.investigations.insert(
                        group.clone(),
                        Investigation {
                            group: group.clone(),
                            agent: agent.clone(),
                            started_ms: *at_ms,
                            since_ms,
                            first_started_ms,
                            ended_ms: None,
                            filed: Vec::new(),
                            done: false,
                        },
                    );
                    out.push(Effect::Investigate {
                        group: group.clone(),
                    });
                }
            }
            Event::IssueFiled {
                group,
                agent,
                issue,
                ..
            } => {
                if let Some(i) = self.investigations.get_mut(group)
                    && i.running()
                    && &i.agent == agent
                    && !i.filed.contains(issue)
                {
                    i.filed.push(issue.clone());
                }
            }
            Event::Investigated { group, agent } => {
                if let Some(i) = self.investigations.get_mut(group)
                    && i.running()
                    && &i.agent == agent
                {
                    i.done = true;
                    out.extend(self.stop_agent(agent));
                }
            }
            Event::InvestigationEnded { group, at_ms, .. } => {
                if let Some(i) = self.investigations.get_mut(group)
                    && i.running()
                {
                    i.ended_ms = Some(*at_ms);
                    let agent = i.agent.clone();
                    out.extend(self.stop_agent(&agent));
                }
            }
            Event::Held => self.held = true,
            Event::Resumed => {
                self.held = false;
                self.held_since = None;
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
                        i.verify_failed = false;
                        i.reopen_due = false;
                        out.push(Effect::Close {
                            issue: issue.clone(),
                        });
                    } else {
                        // Back to work, with what failed; the merged pull request is
                        // history. The merge may have closed it in its tracker, where
                        // the tracker part reopens it.
                        i.stage = Stage::Ready;
                        i.review_ok = false;
                        i.checks_ok = None;
                        i.pulls.clear();
                        i.merged.clear();
                        i.branches.clear();
                        i.closed.clear();
                        i.review_waived = false;
                        i.verify_failed = true;
                        i.reopen_due = true;
                        i.fix = vec![format!(
                            "after the last change merged and deployed, the verify check failed: {output}"
                        )];
                    }
                }
            }
        }
        out.retain(|e| !self.waits_for_scope(e));
        self.follow_scope();
        out.extend(self.place());
        out.retain(|e| !self.waits_for_machine(e));
        out
    }

    /// The repositories an issue changes, in the order they merge: its own
    /// project; or for an issue of an area, the repositories its plan names, else
    /// the area's first.
    pub fn repos_of(&self, issue: &Issue) -> Vec<ProjectId> {
        if self.projects.contains_key(&issue.id.project) {
            return vec![issue.id.project.clone()];
        }
        let mut repos: Vec<ProjectId> = Vec::new();
        for r in issue.plan.iter().flat_map(|p| p.repos.iter()) {
            if !repos.contains(r) {
                repos.push(r.clone());
            }
        }
        if repos.is_empty() {
            repos.push(
                self.areas
                    .get(&issue.id.project)
                    .and_then(|a| a.projects.first())
                    .unwrap_or(&issue.id.project)
                    .clone(),
            );
        }
        repos
    }

    /// An issue's pull requests by repository, in the order they merge; one for a
    /// repository it no longer names comes after those.
    pub(crate) fn pulls_of<'a>(&self, issue: &'a Issue) -> Vec<(&'a ProjectId, &'a String)> {
        let repos = self.repos_of(issue);
        let mut pulls: Vec<_> = issue.pulls.iter().collect();
        // The sort keeps the order of those that come after.
        pulls.sort_by_key(|(r, _)| repos.iter().position(|x| x == *r).unwrap_or(repos.len()));
        pulls
    }

    /// An issue's pull requests as an agent is told of them, each with where to
    /// open it, in the order they merge. One a person closed is given in its
    /// repository's place, as closed.
    pub fn pulled(&self, issue: &Issue) -> Vec<brief::Pulled> {
        let order = self.repos_of(issue);
        order
            .iter()
            .chain(issue.pulls.keys().filter(|r| !order.contains(r)))
            .chain(
                issue
                    .closed
                    .keys()
                    .filter(|r| !order.contains(r) && !issue.pulls.contains_key(*r)),
            )
            .filter_map(|r| {
                let repo = self
                    .projects
                    .get(r)
                    .map_or_else(String::new, |p| p.repo.clone());
                if let Some(number) = issue.pulls.get(r) {
                    return Some(brief::Pulled {
                        project: r.clone(),
                        repo,
                        number: number.clone(),
                        merged: issue.merged.contains(r),
                        branch: issue.branches.get(r).cloned(),
                        closed: false,
                    });
                }
                let closed = issue.closed.get(r)?;
                Some(brief::Pulled {
                    project: r.clone(),
                    repo,
                    number: closed.number.clone(),
                    merged: false,
                    branch: closed.branch.clone(),
                    closed: true,
                })
            })
            .collect()
    }

    /// The repository an issue is worked in: its first.
    pub fn repo_of(&self, issue: &Issue) -> Option<&Project> {
        self.repos_of(issue)
            .first()
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
    /// heard its round, an arbiter while the issue is with the arbiter. Once a
    /// reviewer's or arbiter's approval is in, the issue waits on the checks, not
    /// on that agent. An arbiter that has ruled on the issue has finished its step,
    /// and so has an author on a paused issue that has not heard its fixes. A
    /// group's investigation waits on its investigator until it says it is done.
    /// Nothing waits on an agent whose step is over for good
    /// ([`Self::over_for_good`]), such as an old author whose issue works again
    /// in another folder: it is not reminded, and its stop is asked for again.
    /// While the daemon is held ([`World::held`]), nothing waits on an
    /// investigator or on an agent of a paused issue.
    pub fn waiting_on(&self, agent: &Agent) -> bool {
        // A held daemon pushes no agent of a paused issue, nor an investigator:
        // one at work finishes its turn, and none is reminded into another or
        // stopped for sitting idle, since a person may be talking to it in its
        // terminal. An issue with a pull request merged is not paused, and its
        // agents are waited on as ever, so its arbiter is not stopped mid-ruling.
        if self.held
            && agent
                .issue
                .as_ref()
                .and_then(|id| self.issues.get(id))
                .is_none_or(|i| self.paused(i))
        {
            return false;
        }
        if self.investigates(agent) {
            return true;
        }
        let Some(i) = agent.issue.as_ref().and_then(|id| self.issues.get(id)) else {
            return false;
        };
        if self.over_for_good(agent) {
            return false;
        }
        match (i.stage, agent.role) {
            (Stage::Refining, Role::Refiner) | (Stage::Working, Role::Author) => true,
            (Stage::Fixing { .. }, Role::Author) => i.told || !self.paused(i),
            (Stage::Arbitration { .. }, Role::Arbiter) => {
                !i.review_ok
                    && !self.decisions.values().any(|d| {
                        d.issue == i.id
                            && d.by == agent.id
                            && matches!(d.kind, DecisionKind::Ruling { .. })
                    })
            }
            (Stage::InReview { .. }, Role::Reviewer) => i.told && !i.review_ok,
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
    /// with the arbiter. An issue in review or with the arbiter whose approval is in
    /// needs neither while its checks run. This brings back an agent whose session
    /// ended, or one the daemon decided on and did not start before it stopped. A
    /// running author or reviewer that has not heard its fixes or its round is
    /// told again, and an arbiter that ruled in an earlier arbitration and still
    /// runs is asked again to stop. So is every agent that runs although its step
    /// is over for good ([`Self::over_for_good`]), first, since its stop may have
    /// failed or been lost to a restart. After the `starts` setting's number of agents of one role have
    /// ended while one issue waited on them it gives up, and says so. Agents whose programs were
    /// lost do not count toward `starts`; after [`LOST_PER_START`] times `starts` of them it gives
    /// up too, with a line of its own. A paused
    /// issue gets nothing until it is back in its group's scope and the daemon is
    /// not held, and then what its stage needs. A running investigation whose
    /// investigator never started gets one, once the daemon is not held.
    pub fn missing(&self) -> Missing {
        let mut out = Missing {
            start: self.stops_again(),
            ..Missing::default()
        };
        out.start.extend(self.missing_investigators());
        for i in self.issues.values().filter(|i| !self.paused(i)) {
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
                // Approved, by a verdict, a ruling or a person's overturn: only the
                // checks are left.
                Stage::InReview { .. } | Stage::Arbitration { .. } if i.review_ok => continue,
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
                // An arbiter whose ruling is in has finished its step, so nothing
                // reminds or stops it. One still running when the issue waits on an
                // arbiter again is one whose stop failed, or that an older log left
                // running: ask for its stop again, and a fresh one starts once it
                // has ended.
                if role == Role::Arbiter {
                    out.start.extend(
                        self.agents
                            .values()
                            .filter(|a| {
                                a.issue.as_ref() == Some(&i.id)
                                    && a.role == Role::Arbiter
                                    && a.state != AgentState::Ended
                                    && !self.waiting_on(a)
                            })
                            .map(|a| Effect::StopAgent {
                                agent: a.id.clone(),
                            }),
                    );
                }
                // Running, but not yet told what its stage needs: say it again.
                // The part that delivers it drops a message already waiting.
                if !i.told {
                    match i.stage {
                        Stage::Fixing { .. } => out.start.push(Effect::RequestFix {
                            issue: i.id.clone(),
                            points: i.fix.clone(),
                        }),
                        Stage::InReview { round } => out.start.push(Effect::StartReview {
                            issue: i.id.clone(),
                            round,
                        }),
                        _ => {}
                    }
                }
                continue;
            }
            // Only agents that ended while the issue waited on them count: not one
            // the loop stopped because the issue went back to its author. One
            // whose program was lost, as to a reboot, counts toward a limit of
            // its own, so a program lost at the same point every run is not
            // started forever.
            let of_role: Vec<&Agent> = self
                .agents
                .values()
                .filter(|a| a.issue.as_ref() == Some(&i.id) && a.role == role)
                .collect();
            let tried = of_role.iter().filter(|a| a.ended_waited_on).count();
            let lost = of_role.iter().filter(|a| a.lost_waited_on).count();
            let starts = self.setting_number(&i.id, Some(role), "starts");
            if tried as u64 >= starts {
                out.given_up.push(format!(
                    "{}: {tried} {}s ended before it moved on; no more are started",
                    i.id,
                    role.name()
                ));
            } else if lost as u64 >= starts.saturating_mul(LOST_PER_START) {
                out.given_up.push(format!(
                    "{}: the programs of {lost} {}s were lost before it moved on; no more are started",
                    i.id,
                    role.name()
                ));
            } else {
                out.start.push(effect);
            }
        }
        out.start.retain(|e| !self.waits_for_machine(e));
        out
    }

    /// A stop for each agent that runs although its step is over for good
    /// ([`Self::over_for_good`]), in agent order. The loop asked for it when the
    /// step ended, but that stop may have failed, or been lost to a restart,
    /// since a replay drops effects. It is asked for on a paused issue too: a
    /// stop starts no work.
    fn stops_again(&self) -> Vec<Effect> {
        self.agents
            .values()
            .filter(|a| a.state != AgentState::Ended && self.over_for_good(a))
            .map(|a| Effect::StopAgent {
                agent: a.id.clone(),
            })
            .collect()
    }

    /// Whether nothing will need `agent` again: an agent on an issue that is
    /// done, deploying or verifying, or back to new or ready, as after a reopen
    /// in the tracker or a failed verify; any but the refiner on an issue being
    /// refined; a refiner whose issue's plan is in; an author whose issue holds
    /// other folders than the ones it held; an investigator whose investigation
    /// has ended. An author or reviewer on an issue in review, being fixed or
    /// merging, a refiner while its issue is refining and the investigator of a
    /// running investigation are still needed. An arbiter that has ruled is
    /// asked to stop by [`Self::missing`] once its issue waits on an arbiter
    /// again.
    fn over_for_good(&self, agent: &Agent) -> bool {
        if agent.role == Role::Investigator && agent.issue.is_none() {
            return self
                .investigating(&agent.group)
                .is_none_or(|i| i.agent != agent.id);
        }
        let Some(i) = agent.issue.as_ref().and_then(|id| self.issues.get(id)) else {
            return false;
        };
        match i.stage {
            // Back to new or ready, every agent on it is from before: a reopen
            // in the tracker or a failed verify ended their steps.
            Stage::Done | Stage::Deploying | Stage::Verifying | Stage::New | Stage::Ready => true,
            Stage::Refining => agent.role != Role::Refiner,
            _ => match agent.role {
                Role::Refiner => i.plan.is_some(),
                // Its issue let them go, as at a close before a reopen, and
                // holds others now, for a later author.
                Role::Author => {
                    let holds =
                        |f: &FolderId| i.folder.as_ref() == Some(f) || i.other_folders.contains(f);
                    i.folder.is_some() && !agent.held.is_empty() && !agent.held.iter().any(holds)
                }
                _ => false,
            },
        }
    }

    /// The folder an agent of `role` on `issue` runs in: its own, else (for a
    /// reader) a free one of the issue's project when there is one, so it does not
    /// work in a folder an author is changing. Only folders on the role's
    /// `machine` count, when one is set.
    pub fn folder_for(
        &self,
        issue: &Issue,
        role: Role,
        folder: Option<&FolderId>,
    ) -> Option<&Folder> {
        match folder {
            Some(f) => self.folders.get(f),
            None => self
                .usable_folders(issue, role)
                .find(|f| f.holder.is_none())
                .or_else(|| self.usable_folders(issue, role).next()),
        }
    }

    /// What starting an agent takes: its id, the machine and folder it runs in,
    /// and its brief. The id is its role and a number, counting every agent of
    /// that role this world has started (`author-3`). An agent with no work
    /// folder of its own runs in a folder of its project without holding it.
    /// `None` when the issue is unknown, or when such an agent has no folder on
    /// the machine its `machine` setting names. An investigator works for its
    /// group, not on an issue: it starts through [`World::launch_investigator`].
    pub fn launch(
        &self,
        group: &GroupId,
        role: Role,
        issue: &IssueId,
        folder: Option<&FolderId>,
    ) -> Option<Launch> {
        if role == Role::Investigator {
            return None;
        }
        let i = self.issues.get(issue)?;
        if folder.is_none() && !self.has_place(i, role) {
            return None;
        }
        let n = self.agents.values().filter(|a| a.role == role).count() + 1;
        let place = self.folder_for(i, role, folder);
        let path = place.map(|f| std::path::PathBuf::from(&f.path));
        let shared = folder.is_none() && place.is_some_and(|f| f.holder.is_some());
        // An author holding folders in several repositories is told each one.
        let held: Vec<brief::Held> = if role == Role::Author {
            i.folder
                .iter()
                .chain(&i.other_folders)
                .filter_map(|id| self.folders.get(id))
                .map(|f| brief::Held {
                    project: f.project.clone(),
                    repo: self
                        .projects
                        .get(&f.project)
                        .map_or_else(String::new, |p| p.repo.clone()),
                    path: f.path.clone(),
                })
                .collect()
        } else {
            Vec::new()
        };
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
                    brief::Workplace {
                        folder: path,
                        shared,
                        held,
                    },
                    &self.pulled(i),
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

    /// Sends an issue back to its author with what to fix, and stops its running
    /// arbiters (`arbiters`, from [`Self::stop`]). An arbiter's work ends with its
    /// ruling, and its brief names the round and pull requests it was started for,
    /// so the next final-round block starts a fresh one.
    fn send_back(
        i: &mut Issue,
        round: u8,
        points: Vec<String>,
        keeps_round: bool,
        arbiters: Vec<Effect>,
        out: &mut Vec<Effect>,
    ) {
        out.extend(arbiters);
        i.checks_wait();
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
        arbiters: Vec<Effect>,
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
                    Self::send_back(i, round, vec![reason.to_string()], false, arbiters, out);
                }
                Stage::Merging => {
                    i.review_ok = false;
                    let round = i.round.max(1);
                    Self::send_back(i, round, vec![reason.to_string()], false, arbiters, out);
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
                    // The stopped reviewer judges no check, and no one else
                    // will: as with a send-back, the running ones wait.
                    i.checks_wait();
                    Self::try_merge(i, out);
                }
                _ => {}
            }
        }
    }

    /// What a reviewer's verdict on round `round` of `i` says beside approve or
    /// block, from a verdict line that gives it: each of the plan's checks is
    /// failed when `failed` names it and passed otherwise, and each point of the
    /// round before stays open when `open` names it and is resolved otherwise.
    /// Unlike a ready, such a line marks a check from an older plan too, since
    /// it names every check.
    fn judge(i: &mut Issue, round: u8, failed: Option<&[usize]>, open: Option<&[usize]>) {
        if let Some(failed) = failed {
            for (n, c) in (1..).zip(i.plan.iter_mut().flat_map(|p| p.checks.iter_mut())) {
                c.state = Some(if failed.contains(&n) {
                    CheckState::Failed
                } else {
                    CheckState::Passed
                });
            }
        }
        if let Some(open) = open
            && i.round_before(round).is_some()
            && let Some(before) = i.reviews.last_mut()
        {
            before.states = (1..=before.points.len())
                .map(|n| {
                    if open.contains(&n) {
                        PointState::Open
                    } else {
                        PointState::Resolved
                    }
                })
                .collect();
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

    /// Frees work folders for the next issue.
    fn release(&mut self, folders: Vec<FolderId>, out: &mut Vec<Effect>) {
        for id in folders {
            if let Some(f) = self.folders.get_mut(&id) {
                f.holder = None;
                out.push(Effect::ReleaseFolder { folder: id });
            }
        }
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

    /// Whether `issue` has left the scope of the group that took it: the group no
    /// longer lists its milestone, its project or its area. It is worked out each
    /// time from the issue and the group, so nothing about it is stored. A done
    /// issue has not left.
    pub fn out_of_scope(&self, issue: &Issue) -> bool {
        issue.stage != Stage::Done
            && issue
                .group
                .as_ref()
                .and_then(|g| self.groups.get(g))
                .is_some_and(|g| !self.in_scope(g, issue))
    }

    /// Whether nothing new starts on `issue` because it left its group's scope,
    /// or the daemon is held ([`World::held`]), before any of its pull requests
    /// merged. One with a pull request merged, or deploying or verifying, is past
    /// the agents and finishes, and a done one has nothing left to start.
    pub fn paused(&self, issue: &Issue) -> bool {
        (self.held || self.out_of_scope(issue))
            && issue.merged.is_empty()
            && !matches!(
                issue.stage,
                Stage::Deploying | Stage::Verifying | Stage::Done
            )
    }

    /// Whether `effect` starts or tells an agent, or asks for a merge, on a paused
    /// issue. The loop drops it; [`World::missing`] asks again once the issue is
    /// back in scope and the daemon is not held.
    fn waits_for_scope(&self, effect: &Effect) -> bool {
        match effect {
            Effect::StartAgent { issue, .. }
            | Effect::RequestFix { issue, .. }
            | Effect::StartReview { issue, .. }
            | Effect::Merge { issue } => self.issues.get(issue).is_some_and(|i| self.paused(i)),
            _ => false,
        }
    }

    /// Follows the issues whose group no longer covers them. A refining one
    /// whose refiner has stopped goes back to new: it has no plan, and a refiner
    /// started when it comes back must wait for a free slot like any other. A
    /// ready one another group covers goes to that group: it has a plan and holds
    /// no agent or folder, so nothing ties it to the group that refined it.
    fn follow_scope(&mut self) {
        let mut back = Vec::new();
        let mut moved = Vec::new();
        for i in self.issues.values().filter(|i| self.out_of_scope(i)) {
            match i.stage {
                Stage::Refining if self.running(&i.id, Role::Refiner).is_none() => {
                    back.push(i.id.clone());
                }
                Stage::Ready => {
                    if let Some(g) = self.groups.values().find(|g| self.in_scope(g, i)) {
                        moved.push((i.id.clone(), g.id.clone()));
                    }
                }
                _ => {}
            }
        }
        for id in back {
            if let Some(i) = self.issues.get_mut(&id) {
                i.stage = Stage::New;
                i.group = None;
            }
        }
        for (id, group) in moved {
            if let Some(i) = self.issues.get_mut(&id) {
                i.group = Some(group);
            }
        }
    }

    /// Whether a group works on an issue: one of its projects, and one of its
    /// milestones when it names any.
    pub(crate) fn in_scope(&self, group: &Group, issue: &Issue) -> bool {
        self.unlisted(group, issue).is_none()
    }

    /// What `group` does not list that it would need to for `issue` to be in its
    /// scope; `None` when it is. Its project or area comes first, since without
    /// it the milestone does not matter.
    pub(crate) fn unlisted(&self, group: &Group, issue: &Issue) -> Option<Unlisted> {
        let from = &issue.id.project;
        let taken = group.projects.contains(from)
            || group.areas.contains(from)
            || group
                .areas
                .iter()
                .filter_map(|a| self.areas.get(a))
                .any(|a| a.projects.contains(from));
        if !taken {
            return Some(if self.areas.contains_key(from) {
                Unlisted::Area(from.clone())
            } else {
                Unlisted::Project(from.clone())
            });
        }
        match &issue.milestone {
            _ if group.milestones.is_empty() => None,
            Some(m) if group.milestones.contains(m) => None,
            Some(m) => Some(Unlisted::Milestone(m.clone())),
            None => Some(Unlisted::NoMilestone),
        }
    }

    /// A free work folder `may` allows in each of `repos`, in their order and all
    /// on one machine, since one author has to reach every one: the machine of the
    /// first repository's folder. `None` when no such machine has them all, so two
    /// issues cannot each hold half of what the other needs. A folder no issue
    /// holds is not free while an agent that held it runs ([`Self::in_use`]).
    pub(crate) fn free_folders(
        &self,
        repos: &[ProjectId],
        may: impl Fn(&Folder) -> bool,
    ) -> Option<Vec<FolderId>> {
        let (first, others) = repos.split_first()?;
        let in_use = self.in_use();
        let free = || {
            self.folders
                .values()
                .filter(|f| f.holder.is_none() && !in_use.contains(&f.id) && may(f))
        };
        free().filter(|f| &f.project == first).find_map(|start| {
            std::iter::once(Some(start.id.clone()))
                .chain(others.iter().map(|repo| {
                    free()
                        .find(|f| &f.project == repo && f.machine == start.machine)
                        .map(|f| f.id.clone())
                }))
                .collect()
        })
    }

    /// The work folders held by an agent that has not ended: an author's stop
    /// may have failed after its issue merged or was closed, and its session may
    /// still change them.
    pub(crate) fn in_use(&self) -> BTreeSet<&FolderId> {
        self.agents
            .values()
            .filter(|a| a.state != AgentState::Ended)
            .flat_map(|a| &a.held)
            .collect()
    }

    /// Takes new issues for refining and starts authors on ready ones; none
    /// while the daemon is held.
    fn place(&mut self) -> Vec<Effect> {
        let mut out = Vec::new();
        if self.held {
            return out;
        }
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
                // One whose refiner has no folder on its machine takes no slot.
                .filter(|i| self.has_place(i, Role::Refiner))
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
                        && self.start_waits(group, i).is_empty()
                })
                .collect();
            ready.sort_by_key(|i| Self::rank(group, i));
            let ready: Vec<IssueId> = ready.iter().map(|i| i.id.clone()).collect();
            for id in ready {
                let Some(mut picked) = self.issues.get(&id).and_then(|i| self.author_folders(i))
                else {
                    continue;
                };
                for f in &picked {
                    if let Some(f) = self.folders.get_mut(f) {
                        f.holder = Some(id.clone());
                    }
                }
                if let Some(i) = self.issues.get_mut(&id) {
                    let first = picked.remove(0);
                    i.folder = Some(first.clone());
                    i.other_folders = picked;
                    i.stage = Stage::Working;
                    out.push(Effect::StartAgent {
                        group: group.id.clone(),
                        role: Role::Author,
                        issue: id,
                        folder: Some(first),
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

#[cfg(test)]
#[path = "world_states_tests.rs"]
mod states;

#[cfg(test)]
#[path = "world_stops_tests.rs"]
mod stops;
