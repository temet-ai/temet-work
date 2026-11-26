//! Trackers: where issues and milestones come from, and where comments and
//! closes go back to. Pull requests and merges belong to the code host, not the
//! tracker: a Jira project tracks issues and hosts no code.
//!
//! This crate holds the [`Tracker`] contract, a [`FakeTracker`] and the
//! [`contract`] every tracker adapter runs. Adapter crates depend on this crate only.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use temet_work_core::ids::{IssueId, MilestoneId, ProjectId};
use temet_work_core::{Effect, Event, Outcome, Part, Polled, World};

/// An issue as a tracker reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackedIssue {
    /// The tracker's key for it (`516`, `ENG-41`).
    pub key: String,
    /// Its title.
    pub title: String,
    /// Its text.
    pub body: String,
    /// Its milestone's title, if any.
    pub milestone: Option<String>,
    /// Keys of the issues it says it depends on, read from its text.
    pub depends_on: Vec<String>,
    /// The tracker's order, oldest first.
    pub order: u64,
    /// Whether it is closed.
    pub closed: bool,
}

/// A milestone as a tracker reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackedMilestone {
    /// Its title.
    pub title: String,
    /// How many issues in it are open.
    pub open: u64,
}

/// Why a tracker call failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrackerError {
    /// No such project or issue.
    NotFound(String),
    /// Signed out, or not allowed.
    Denied(String),
    /// Anything else, as the tracker reported it.
    Other(String),
}

impl fmt::Display for TrackerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(what) => write!(f, "not found: {what}"),
            Self::Denied(what) => write!(f, "not allowed: {what}"),
            Self::Other(e) => f.write_str(e),
        }
    }
}

impl std::error::Error for TrackerError {}

/// One tracker, added by an adapter crate. `project` is the tracker's own name
/// for the project (`owner/repo`, a Jira key).
pub trait Tracker: Send {
    /// A short, stable id, the adapter's feature name after `tracker-`.
    fn id(&self) -> &'static str;

    /// The project's open issues.
    fn issues(&mut self, project: &str) -> Result<Vec<TrackedIssue>, TrackerError>;

    /// The project's open milestones.
    fn milestones(&mut self, project: &str) -> Result<Vec<TrackedMilestone>, TrackerError>;

    /// Adds a comment to an issue.
    fn comment(&mut self, project: &str, key: &str, text: &str) -> Result<(), TrackerError>;

    /// Closes an issue.
    fn close(&mut self, project: &str, key: &str) -> Result<(), TrackerError>;
}

/// The keys an issue's text says it depends on: on each line that starts with
/// "Depends on" (any case), every `#123` or `KEY-123`.
pub fn depends_on(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim_start_matches(|c: char| c.is_whitespace() || c == '-' || c == '*');
        if !trimmed.to_lowercase().starts_with("depends on") {
            continue;
        }
        for word in trimmed["depends on".len()..]
            .split(|c: char| c.is_whitespace() || c == ',' || c == '.' || c == ';')
        {
            let key = word.trim_start_matches('#');
            let is_number = !key.is_empty() && key.chars().all(|c| c.is_ascii_digit());
            let is_jira = key.split_once('-').is_some_and(|(p, n)| {
                !p.is_empty()
                    && p.chars().all(|c| c.is_ascii_uppercase())
                    && !n.is_empty()
                    && n.chars().all(|c| c.is_ascii_digit())
            });
            if (word.starts_with('#') && is_number) || is_jira {
                out.push(key.to_string());
            }
        }
    }
    out
}

/// A tracker in memory, for tests.
#[derive(Default)]
pub struct FakeTracker {
    /// Issues by project.
    pub issues: Vec<(String, TrackedIssue)>,
    /// Milestones by project.
    pub milestones: Vec<(String, TrackedMilestone)>,
    /// Comments added, as (project, key, text).
    pub comments: Vec<(String, String, String)>,
}

impl Tracker for FakeTracker {
    fn id(&self) -> &'static str {
        "fake"
    }

    fn issues(&mut self, project: &str) -> Result<Vec<TrackedIssue>, TrackerError> {
        let known = self.issues.iter().any(|(p, _)| p == project)
            || self.milestones.iter().any(|(p, _)| p == project);
        if !known {
            return Err(TrackerError::NotFound(project.to_string()));
        }
        Ok(self
            .issues
            .iter()
            .filter(|(p, i)| p == project && !i.closed)
            .map(|(_, i)| i.clone())
            .collect())
    }

    fn milestones(&mut self, project: &str) -> Result<Vec<TrackedMilestone>, TrackerError> {
        Ok(self
            .milestones
            .iter()
            .filter(|(p, _)| p == project)
            .map(|(_, m)| m.clone())
            .collect())
    }

    fn comment(&mut self, project: &str, key: &str, text: &str) -> Result<(), TrackerError> {
        if !self
            .issues
            .iter()
            .any(|(p, i)| p == project && i.key == key)
        {
            return Err(TrackerError::NotFound(format!("{project}#{key}")));
        }
        self.comments
            .push((project.into(), key.into(), text.into()));
        Ok(())
    }

    fn close(&mut self, project: &str, key: &str) -> Result<(), TrackerError> {
        let issue = self
            .issues
            .iter_mut()
            .find(|(p, i)| p == project && i.key == key)
            .ok_or_else(|| TrackerError::NotFound(format!("{project}#{key}")))?;
        issue.1.closed = true;
        Ok(())
    }
}

/// The contract every tracker adapter runs.
pub mod contract {
    use super::Tracker;

    /// What a tracker's tests supply.
    pub trait Harness {
        /// A project with open issues and at least one open milestone.
        fn project(&self) -> String;

        /// An issue the contract may comment on and close, when the harness allows
        /// writes. A live tracker's harness answers `None`: the contract never
        /// changes a real project.
        fn writable_issue(&self) -> Option<String>;
    }

    fn ensure(ok: bool, what: &str) -> Result<(), String> {
        if ok { Ok(()) } else { Err(what.to_string()) }
    }

    /// Runs the contract. Returns the first step that failed, in words.
    pub fn check(tracker: &mut dyn Tracker, harness: &dyn Harness) -> Result<(), String> {
        let project = harness.project();
        let issues = tracker
            .issues(&project)
            .map_err(|e| format!("issues: {e}"))?;
        ensure(!issues.is_empty(), "a project with open issues lists them")?;
        let mut keys: Vec<&str> = issues.iter().map(|i| i.key.as_str()).collect();
        keys.sort_unstable();
        keys.dedup();
        ensure(keys.len() == issues.len(), "each issue's key is unique")?;
        ensure(
            issues.iter().all(|i| !i.title.is_empty() && !i.closed),
            "every listed issue has a title and is open",
        )?;
        let milestones = tracker
            .milestones(&project)
            .map_err(|e| format!("milestones: {e}"))?;
        ensure(
            !milestones.is_empty(),
            "a project with open milestones lists them",
        )?;
        ensure(
            issues
                .iter()
                .filter_map(|i| i.milestone.as_deref())
                .all(|m| milestones.iter().any(|x| x.title == m)),
            "an open issue's milestone is one of the open milestones",
        )?;
        if let Some(key) = harness.writable_issue() {
            tracker
                .comment(&project, &key, "contract check")
                .map_err(|e| format!("comment: {e}"))?;
            tracker
                .close(&project, &key)
                .map_err(|e| format!("close: {e}"))?;
            let after = tracker
                .issues(&project)
                .map_err(|e| format!("issues after close: {e}"))?;
            ensure(
                !after.iter().any(|i| i.key == key),
                "a closed issue is no longer listed as open",
            )?;
        }
        ensure(
            tracker.issues("no-such/project").is_err(),
            "an unknown project is an error, not an empty list",
        )?;
        Ok(())
    }
}

/// How often a project is synced, by the daemon's clock.
pub const SYNC_EVERY_MS: u64 = 60_000;

/// The tracker part: the trackers compiled into this build, in the order they were added.
///
/// It syncs each project from the tracker its configuration names, once a
/// minute, and reports only what changed. It closes an issue in its tracker when
/// the loop has verified it.
pub struct Trackers {
    adapters: Vec<Box<dyn Tracker>>,
    every_ms: u64,
    synced_at: BTreeMap<ProjectId, u64>,
    errors: BTreeMap<ProjectId, String>,
}

impl Default for Trackers {
    fn default() -> Self {
        Self {
            adapters: Vec::new(),
            every_ms: SYNC_EVERY_MS,
            synced_at: BTreeMap::new(),
            errors: BTreeMap::new(),
        }
    }
}

/// The events that bring the loop's view of `project` up to `open`, the issues
/// its tracker lists as open now.
pub fn changes(world: &World, project: &ProjectId, open: &[TrackedIssue]) -> Vec<Event> {
    let id = |key: &str| IssueId::new(project.0.clone(), key);
    let mut out = Vec::new();
    let open_keys: BTreeSet<&str> = open.iter().map(|t| t.key.as_str()).collect();
    for t in open {
        let issue = id(&t.key);
        let milestone = t.milestone.clone().map(MilestoneId::new);
        let depends_on: Vec<IssueId> = t.depends_on.iter().map(|k| id(k)).collect();
        let same = world.issues.get(&issue).is_some_and(|i| {
            i.title == t.title
                && i.milestone == milestone
                && i.depends_on == depends_on
                && i.order == t.order
        });
        // A dependency that is neither open nor known was closed before syncing began.
        for d in &depends_on {
            if !open_keys.contains(d.key.as_str())
                && !world.issues.contains_key(d)
                && !out
                    .iter()
                    .any(|e| matches!(e, Event::IssueSynced { issue, .. } if issue == d))
            {
                out.push(Event::IssueSynced {
                    issue: d.clone(),
                    title: String::new(),
                    milestone: None,
                    depends_on: Vec::new(),
                    order: 0,
                    closed: true,
                });
            }
        }
        if !same {
            out.push(Event::IssueSynced {
                issue,
                title: t.title.clone(),
                milestone,
                depends_on,
                order: t.order,
                closed: false,
            });
        }
    }
    for i in world.issues.values() {
        if &i.id.project == project
            && i.stage != temet_work_core::model::Stage::Done
            && !open_keys.contains(i.id.key.as_str())
        {
            out.push(Event::IssueSynced {
                issue: i.id.clone(),
                title: i.title.clone(),
                milestone: i.milestone.clone(),
                depends_on: i.depends_on.clone(),
                order: i.order,
                closed: true,
            });
        }
    }
    out
}

impl Trackers {
    /// Adds a tracker. The root package calls this once per compiled-in adapter.
    pub fn add(&mut self, adapter: Box<dyn Tracker>) {
        self.adapters.push(adapter);
    }

    /// The ids of the trackers in this build.
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }

    /// Syncs every `ms` instead of every minute.
    pub fn every(mut self, ms: u64) -> Self {
        self.every_ms = ms;
        self
    }
}

impl Part for Trackers {
    fn id(&self) -> &'static str {
        "tracker"
    }

    fn poll(&mut self, world: &World, now_ms: u64) -> Polled {
        let mut events = Vec::new();
        for project in world.projects.values() {
            let due = self
                .synced_at
                .get(&project.id)
                .is_none_or(|at| now_ms >= at + self.every_ms);
            if !due {
                continue;
            }
            self.synced_at.insert(project.id.clone(), now_ms);
            let Some(tracker) = self.adapters.iter_mut().find(|a| a.id() == project.tracker) else {
                self.errors.insert(
                    project.id.clone(),
                    format!(
                        "{}: no tracker \"{}\" in this build",
                        project.id, project.tracker
                    ),
                );
                continue;
            };
            match tracker.issues(&project.name) {
                Ok(open) => {
                    self.errors.remove(&project.id);
                    events.extend(changes(world, &project.id, &open));
                }
                Err(e) => {
                    self.errors
                        .insert(project.id.clone(), format!("{}: {e}", project.name));
                }
            }
        }
        let problem = (!self.errors.is_empty())
            .then(|| self.errors.values().cloned().collect::<Vec<_>>().join("; "));
        Polled { events, problem }
    }

    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let Effect::Close { issue } = effect else {
            return Outcome::NotMine;
        };
        let Some(project) = world.projects.get(&issue.project) else {
            return Outcome::Failed(format!("project {} is not configured", issue.project));
        };
        let Some(tracker) = self.adapters.iter_mut().find(|a| a.id() == project.tracker) else {
            return Outcome::Failed(format!("no tracker \"{}\" in this build", project.tracker));
        };
        match tracker.close(&project.name, &issue.key) {
            Ok(()) => Outcome::done(),
            Err(e) => Outcome::Failed(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(key: &str, milestone: Option<&str>) -> TrackedIssue {
        TrackedIssue {
            key: key.into(),
            title: format!("issue {key}"),
            body: String::new(),
            milestone: milestone.map(String::from),
            depends_on: vec![],
            order: key.parse().unwrap_or(0),
            closed: false,
        }
    }

    fn fake() -> FakeTracker {
        FakeTracker {
            issues: vec![
                ("ankh/morpork".into(), issue("1", Some("Watch"))),
                ("ankh/morpork".into(), issue("2", None)),
            ],
            milestones: vec![(
                "ankh/morpork".into(),
                TrackedMilestone {
                    title: "Watch".into(),
                    open: 1,
                },
            )],
            comments: vec![],
        }
    }

    struct Writable;

    impl contract::Harness for Writable {
        fn project(&self) -> String {
            "ankh/morpork".into()
        }
        fn writable_issue(&self) -> Option<String> {
            Some("2".into())
        }
    }

    fn configured() -> World {
        let mut w = World::default();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("ankh"),
            tracker: "fake".into(),
            name: "ankh/morpork".into(),
        });
        w
    }

    fn apply(w: &mut World, events: &[Event]) {
        for e in events {
            w.apply(e);
        }
    }

    #[test]
    fn syncs_open_issues_once_then_only_changes() {
        let mut w = configured();
        let mut part = Trackers::default();
        part.add(Box::new(fake()));
        let first = part.poll(&w, 0);
        assert_eq!(first.problem, None);
        assert_eq!(first.events.len(), 2);
        apply(&mut w, &first.events);
        assert_eq!(
            part.poll(&w, SYNC_EVERY_MS - 1).events,
            vec![],
            "not due yet"
        );
        assert_eq!(
            part.poll(&w, SYNC_EVERY_MS).events,
            vec![],
            "nothing changed"
        );
    }

    #[test]
    fn an_issue_closed_in_the_tracker_is_synced_as_closed() {
        let mut w = configured();
        let mut part = Trackers::default().every(0);
        part.add(Box::new(fake()));
        let first = part.poll(&w, 0).events;
        apply(&mut w, &first);
        let mut gone = fake();
        gone.issues.retain(|(_, i)| i.key != "2");
        let mut part = Trackers::default().every(0);
        part.add(Box::new(gone));
        let events = part.poll(&w, 1).events;
        assert!(
            matches!(events.as_slice(), [Event::IssueSynced { issue, closed: true, .. }] if issue.key == "2"),
            "{events:?}"
        );
    }

    #[test]
    fn a_dependency_closed_before_syncing_began_counts_as_done() {
        let mut tracker = fake();
        tracker.issues[0].1.depends_on = vec!["40".into()];
        let mut part = Trackers::default();
        part.add(Box::new(tracker));
        let events = part.poll(&configured(), 0).events;
        assert!(events.iter().any(
            |e| matches!(e, Event::IssueSynced { issue, closed: true, .. } if issue.key == "40")
        ));
    }

    #[test]
    fn a_failing_project_stays_a_problem_until_it_syncs() {
        let mut w = configured();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("klatch"),
            tracker: "fake".into(),
            name: "no-such/project".into(),
        });
        let mut part = Trackers::default();
        part.add(Box::new(fake()));
        let first = part.poll(&w, 0);
        assert_eq!(first.events.len(), 2, "the project that works still syncs");
        assert!(
            first
                .problem
                .as_deref()
                .is_some_and(|p| p.contains("no-such/project"))
        );
        assert!(
            part.poll(&w, 1).problem.is_some(),
            "still shown between syncs"
        );
    }

    #[test]
    fn closes_a_verified_issue_in_its_tracker() {
        let w = configured();
        let mut part = Trackers::default();
        part.add(Box::new(fake()));
        let close = Effect::Close {
            issue: IssueId::new("ankh", "2"),
        };
        assert_eq!(part.carry_out(&close, &w), Outcome::done());
        let unknown = Effect::Close {
            issue: IssueId::new("klatch", "1"),
        };
        assert!(matches!(part.carry_out(&unknown, &w), Outcome::Failed(_)));
    }

    #[test]
    fn keeps_adapters_in_the_order_added() {
        let mut part = Trackers::default();
        part.add(Box::new(FakeTracker::default()));
        assert_eq!(part.ids(), ["fake"]);
        assert_eq!(part.id(), "tracker");
    }

    #[test]
    fn the_fake_passes_the_contract() {
        assert_eq!(contract::check(&mut fake(), &Writable), Ok(()));
    }

    #[test]
    fn reads_dependencies_from_depends_on_lines() {
        let body = "Change the codes.\n\nDepends on #491.\n- depends on #12, #13 and ENG-41\nSee #77 for context.";
        assert_eq!(depends_on(body), ["491", "12", "13", "ENG-41"]);
        assert_eq!(
            depends_on("No dependencies here, see #5."),
            Vec::<String>::new()
        );
    }
}
