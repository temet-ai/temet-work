//! Trackers: where issues and milestones come from, and where comments and
//! closes go back to. Pull requests and merges belong to the code host, not the
//! tracker: a Jira project tracks issues and hosts no code.
//!
//! This crate holds the [`Tracker`] contract, a [`FakeTracker`] and the
//! [`contract`] every tracker adapter runs. Adapter crates depend on this crate only.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use temet_work_core::ids::{IssueId, MilestoneId, ProjectId};
use temet_work_core::model::{Addresses, Issue, Stage};
use temet_work_core::{Effect, Event, IssueText, Outcome, Part, Polled, World};

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
    /// Its due date as `YYYY-MM-DD` ([`is_date`]); none when the tracker gives
    /// none.
    pub due: Option<String>,
}

/// Whether `s` is a date as `YYYY-MM-DD`, the form due dates are kept in.
pub fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        })
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

    /// The project's open milestones, each with its due date when it has one.
    fn milestones(&mut self, project: &str) -> Result<Vec<TrackedMilestone>, TrackerError>;

    /// The web address of the project's issues, made from its name and the
    /// adapter's own settings, such as the host it reaches; none when the
    /// adapter cannot say. It asks the tracker nothing.
    fn address(&self, project: &str) -> Option<String>;

    /// Whether the issue is closed. An error, `NotFound` among them, when the
    /// tracker cannot say.
    fn closed(&mut self, project: &str, key: &str) -> Result<bool, TrackerError>;

    /// Adds a comment to an issue.
    fn comment(&mut self, project: &str, key: &str, text: &str) -> Result<(), TrackerError>;

    /// Closes an issue. One already closed counts as closed.
    fn close(&mut self, project: &str, key: &str) -> Result<(), TrackerError>;

    /// Reopens a closed issue, so the tracker lists it as open again. One
    /// already open counts as reopened.
    fn reopen(&mut self, project: &str, key: &str) -> Result<(), TrackerError>;

    /// Files a new open issue in the project, in `milestone` when one is named,
    /// and returns it as the tracker will list it.
    fn create(
        &mut self,
        project: &str,
        title: &str,
        body: &str,
        milestone: Option<&str>,
    ) -> Result<TrackedIssue, TrackerError>;
}

/// The keys an issue's text says it depends on: on each line that starts with
/// "Depends on" (any case), every `#123` or `KEY-123`, and every `other#12` or
/// `owner/repo#12`, an issue in another project or area.
pub fn depends_on(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim_start_matches(|c: char| c.is_whitespace() || c == '-' || c == '*');
        if !trimmed.to_lowercase().starts_with("depends on") {
            continue;
        }
        for word in
            trimmed["depends on".len()..].split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        {
            // A full stop ends a sentence, and a repository's name may hold one.
            let word = word.trim_end_matches('.');
            if names_elsewhere(word) {
                out.push(word.to_string());
                continue;
            }
            for word in word.split('.') {
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
                } else if names_elsewhere(word) {
                    out.push(word.to_string());
                }
            }
        }
    }
    out
}

/// Whether `word` is `other#12` or `owner/repo#12`.
fn names_elsewhere(word: &str) -> bool {
    word.split_once('#').is_some_and(|(p, n)| {
        !p.is_empty()
            && !p.ends_with('.')
            && p.chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_./".contains(c))
            && !n.is_empty()
            && n.chars().all(|c| c.is_ascii_digit())
    })
}

/// A tracker in memory, for tests.
pub struct FakeTracker {
    /// Its id.
    pub id: &'static str,
    /// Issues by project.
    pub issues: Vec<(String, TrackedIssue)>,
    /// Milestones by project.
    pub milestones: Vec<(String, TrackedMilestone)>,
    /// Comments added, as (project, key, text).
    pub comments: Vec<(String, String, String)>,
    /// When set, every reopen fails with this error, as it does in a Jira
    /// workflow with no transition out of Done.
    pub reopen_refused: Option<TrackerError>,
    /// Open issues its list of a project leaves out, as (project, key), as
    /// GitHub's leaves out an open pull request; asked about, each is open.
    pub unlisted: Vec<(String, String)>,
}

impl Default for FakeTracker {
    fn default() -> Self {
        Self::named("fake")
    }
}

impl FakeTracker {
    /// A fake tracker with the given id.
    pub fn named(id: &'static str) -> Self {
        Self {
            id,
            issues: Vec::new(),
            milestones: Vec::new(),
            comments: Vec::new(),
            reopen_refused: None,
            unlisted: Vec::new(),
        }
    }
}

impl Tracker for FakeTracker {
    fn id(&self) -> &'static str {
        self.id
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
            .filter(|(p, i)| !self.unlisted.contains(&(p.clone(), i.key.clone())))
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

    fn address(&self, project: &str) -> Option<String> {
        Some(format!("https://{}.example.com/{project}/issues", self.id))
    }

    fn closed(&mut self, project: &str, key: &str) -> Result<bool, TrackerError> {
        self.issues
            .iter()
            .find(|(p, i)| p == project && i.key == key)
            .map(|(_, i)| i.closed)
            .ok_or_else(|| TrackerError::NotFound(format!("{project}#{key}")))
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

    fn reopen(&mut self, project: &str, key: &str) -> Result<(), TrackerError> {
        let issue = self
            .issues
            .iter_mut()
            .find(|(p, i)| p == project && i.key == key)
            .ok_or_else(|| TrackerError::NotFound(format!("{project}#{key}")))?;
        if let Some(e) = &self.reopen_refused {
            return Err(e.clone());
        }
        issue.1.closed = false;
        Ok(())
    }

    /// Numbers the new issue after the highest of the project's, as GitHub does.
    fn create(
        &mut self,
        project: &str,
        title: &str,
        body: &str,
        milestone: Option<&str>,
    ) -> Result<TrackedIssue, TrackerError> {
        let known = self.issues.iter().any(|(p, _)| p == project)
            || self.milestones.iter().any(|(p, _)| p == project);
        if !known {
            return Err(TrackerError::NotFound(project.to_string()));
        }
        if let Some(m) = milestone
            && !self
                .milestones
                .iter()
                .any(|(p, x)| p == project && x.title == m)
        {
            return Err(TrackerError::NotFound(format!(
                "milestone {m} in {project}"
            )));
        }
        let number = self
            .issues
            .iter()
            .filter(|(p, _)| p == project)
            .map(|(_, i)| i.order)
            .max()
            .unwrap_or(0)
            + 1;
        let issue = TrackedIssue {
            key: number.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            milestone: milestone.map(String::from),
            depends_on: depends_on(body),
            order: number,
            closed: false,
        };
        self.issues.push((project.to_string(), issue.clone()));
        Ok(issue)
    }
}

/// The contract every tracker adapter runs.
pub mod contract {
    use super::Tracker;

    /// What a tracker's tests supply.
    pub trait Harness {
        /// A project with open issues and at least one open milestone.
        fn project(&self) -> String;

        /// An issue the contract may comment on, close and reopen, when the
        /// harness allows writes. A live tracker's harness answers `None`: the
        /// contract never changes a real project.
        fn writable_issue(&self) -> Option<String>;

        /// Whether the contract may file a new issue in the project. A live
        /// tracker's harness answers false, as it does for writes.
        fn may_create(&self) -> bool {
            self.writable_issue().is_some()
        }
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
            milestones
                .iter()
                .all(|m| m.due.as_deref().is_none_or(super::is_date)),
            "a milestone's due date is YYYY-MM-DD",
        )?;
        ensure(
            tracker
                .address(&project)
                .is_some_and(|a| a.starts_with("https://") && a.contains(&project)),
            "the project's issues have an https address that holds its name",
        )?;
        ensure(
            !milestones.is_empty(),
            "a project with open milestones lists them",
        )?;
        let first = &issues[0].key;
        ensure(
            tracker.closed(&project, first) == Ok(false),
            "a listed open issue is not closed",
        )?;
        ensure(
            tracker.closed(&project, "999999999").is_err(),
            "an issue the tracker does not have is an error, not open or closed",
        )?;
        ensure(
            issues
                .iter()
                .filter_map(|i| i.milestone.as_deref())
                .all(|m| milestones.iter().any(|x| x.title == m)),
            "an open issue's milestone is one of the open milestones",
        )?;
        if harness.may_create() {
            let milestone = &milestones[0].title;
            let body = format!("Filed by the contract check.\nDepends on #{first}");
            let created = tracker
                .create(&project, "Contract check", &body, Some(milestone))
                .map_err(|e| format!("create: {e}"))?;
            ensure(
                created.title == "Contract check"
                    && created.milestone.as_deref() == Some(milestone.as_str())
                    && !created.closed,
                "a created issue is open, with its title and milestone",
            )?;
            ensure(
                !issues.iter().any(|i| i.key == created.key),
                "a created issue has a key of its own",
            )?;
            let listed = tracker
                .issues(&project)
                .map_err(|e| format!("issues after create: {e}"))?;
            ensure(
                listed.iter().any(|i| {
                    i.key == created.key
                        && i.title == created.title
                        && i.milestone == created.milestone
                        && i.order == created.order
                        && i.depends_on == created.depends_on
                }),
                "a created issue is then listed as it was returned, with its milestone",
            )?;
        }
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
            tracker
                .reopen(&project, &key)
                .map_err(|e| format!("reopen: {e}"))?;
            let reopened = tracker
                .issues(&project)
                .map_err(|e| format!("issues after reopen: {e}"))?;
            ensure(
                reopened.iter().any(|i| i.key == key && !i.closed),
                "a reopened issue is listed as open again",
            )?;
            ensure(
                tracker.reopen(&project, &key).is_ok(),
                "reopening an open issue counts as reopened",
            )?;
        }
        ensure(
            tracker.issues("no-such/project").is_err(),
            "an unknown project is an error, not an empty list",
        )?;
        Ok(())
    }
}

/// The tracker part: the trackers compiled into this build, in the order they were added.
///
/// It syncs each project from the tracker its configuration names, as often as
/// its `sync_every` setting says (a minute by default), and reports only what
/// changed: issues, and the open milestones with their due dates. It keeps each
/// open issue's text from the last sync in memory. It closes an issue in its
/// tracker when the loop has verified it, reopens one a failed verify sent back,
/// and gives the address of each project's and area's issues.
#[derive(Default)]
pub struct Trackers {
    adapters: Vec<Box<dyn Tracker>>,
    /// Overrides the `sync_every` setting, for tests.
    every_ms: Option<u64>,
    synced_at: BTreeMap<ProjectId, u64>,
    errors: BTreeMap<ProjectId, String>,
    /// For each source that has synced, the text of each open issue by key, as
    /// its last sync that worked listed it, and of each issue whose close is
    /// held ([`held`]) as the last sync that listed it did.
    texts: BTreeMap<ProjectId, BTreeMap<String, Kept>>,
    /// The issues waiting on their reopen after a failed verify that a sync has
    /// tried to reopen. A source with one not tried yet syncs at once.
    tried: BTreeSet<IssueId>,
    /// The issues waiting on their reopen that a reopen call has reopened, and
    /// that no sync has listed since. The sync after the first such call comes
    /// at once; later ones wait for `sync_every`, so a tracker slow to list a
    /// reopened issue is not asked again at every poll.
    reopened: BTreeSet<IssueId>,
}

/// An issue's text, as a sync read it.
#[derive(Clone)]
struct Kept {
    text: String,
    /// Whether the last sync that worked listed the issue; if not, an earlier one did.
    listed_last: bool,
}

/// Whether a close of `i` in its tracker is not passed to the loop. A pull
/// request can close its issue as it merges, so a close is held while the issue
/// is finishing ([`Issue::finishing`]), and after a failed verify until a sync
/// lists the issue as open again, which counts as its reopen whoever made it.
/// After that a close is a person's, and finishes the issue.
pub fn held(i: &Issue) -> bool {
    i.stage != Stage::Done && (i.reopen_due || i.finishing())
}

/// The events that bring the loop's view of `project` up to `open`, the issues
/// its tracker lists as open now.
pub fn changes(world: &World, project: &ProjectId, open: &[TrackedIssue]) -> Vec<Event> {
    let mut out = Vec::new();
    let open_keys: BTreeSet<&str> = open.iter().map(|t| t.key.as_str()).collect();
    // Oldest first, so a group placing issues as they arrive takes them in the
    // tracker's order and not in the order the tracker listed them.
    let mut open: Vec<&TrackedIssue> = open.iter().collect();
    open.sort_by_key(|t| t.order);
    for t in open {
        let synced = synced(world, project, t);
        let Event::IssueSynced {
            issue,
            milestone,
            depends_on,
            ..
        } = &synced
        else {
            continue;
        };
        // Unchanged, unless it was closed in the tracker and is open there again,
        // or it is listed for the first time since a failed verify: its reopen.
        let same = world.issues.get(issue).is_some_and(|i| {
            i.title == t.title
                && &i.milestone == milestone
                && &i.depends_on == depends_on
                && i.order == t.order
                && (i.stage != Stage::Done || i.closed_by_loop)
                && !i.reopen_due
        });
        if !same {
            out.push(synced);
        }
    }
    for i in world.issues.values() {
        if &i.id.project == project
            && i.stage != Stage::Done
            && !held(i)
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

/// The events that bring the loop's milestones of `project` up to `listed`, the
/// open ones its tracker lists now: one for each milestone newly listed or whose
/// due date changed, and one for each the loop has as open that the tracker no
/// longer lists. An unchanged milestone gives none, so a due date is logged only
/// when it changes.
pub fn milestone_changes(
    world: &World,
    project: &ProjectId,
    listed: &[TrackedMilestone],
) -> Vec<Event> {
    let known = world.milestones.get(project);
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for m in listed {
        let id = MilestoneId::new(m.title.as_str());
        if !seen.insert(id.clone()) {
            continue;
        }
        if known.and_then(|k| k.get(&id)) != Some(&m.due) {
            out.push(Event::MilestoneSynced {
                project: project.clone(),
                milestone: id,
                due: m.due.clone(),
                open: true,
            });
        }
    }
    for id in known.into_iter().flat_map(|k| k.keys()) {
        if !seen.contains(id) {
            out.push(Event::MilestoneSynced {
                project: project.clone(),
                milestone: id.clone(),
                due: None,
                open: false,
            });
        }
    }
    out
}

/// The open issue `t` of `project` as the loop takes it in.
pub fn synced(world: &World, project: &ProjectId, t: &TrackedIssue) -> Event {
    let depends_on: Vec<IssueId> = t
        .depends_on
        .iter()
        .map(|k| dependency(world, project, k))
        .collect();
    Event::IssueSynced {
        issue: IssueId::new(project.0.clone(), t.key.as_str()),
        title: t.title.clone(),
        milestone: t.milestone.clone().map(MilestoneId::new),
        depends_on,
        order: t.order,
        closed: false,
    }
}

/// The issue a dependency written `key` in an issue of `project` names:
/// `other#12` is in the project or area `other`, and any other key is in
/// `project`. A name nothing matches stays in the key, so the issue waits.
fn dependency(world: &World, project: &ProjectId, key: &str) -> IssueId {
    key.split_once('#')
        .and_then(|(source, n)| resolve(world, source).map(|s| IssueId::new(s.0, n)))
        .unwrap_or_else(|| IssueId::new(project.0.clone(), key))
}

/// The project or area a dependency names: by its id, or by its tracker's or code
/// host's name for it.
fn resolve(world: &World, name: &str) -> Option<ProjectId> {
    world
        .projects
        .values()
        .find(|p| p.id.0 == name || p.name == name || p.repo == name)
        .map(|p| p.id.clone())
        .or_else(|| {
            world
                .areas
                .values()
                .find(|a| a.id.0 == name || (!a.tracker_name.is_empty() && a.tracker_name == name))
                .map(|a| a.id.clone())
        })
}

/// Where issues come from: each project and area with a tracker of its own, as
/// (id, tracker adapter, tracker's name for it).
fn sources(world: &World) -> Vec<(ProjectId, String, String)> {
    let projects = world
        .projects
        .values()
        .filter(|p| !p.tracker.is_empty())
        .map(|p| (p.id.clone(), p.tracker.clone(), p.name.clone()));
    let areas = world
        .areas
        .values()
        .filter(|a| !a.tracker.is_empty())
        .map(|a| (a.id.clone(), a.tracker.clone(), a.tracker_name.clone()));
    projects.chain(areas).collect()
}

/// The issues that `open`, the open issues of `project`, depend on and that are
/// neither among them nor known to the loop, each with how a line writes it. Each
/// may have been closed before syncing began; only its tracker can say.
pub fn unknown_dependencies(
    world: &World,
    project: &ProjectId,
    open: &[TrackedIssue],
) -> BTreeMap<IssueId, String> {
    let open_keys: BTreeSet<&str> = open.iter().map(|t| t.key.as_str()).collect();
    open.iter()
        .flat_map(|t| &t.depends_on)
        .map(|written| (dependency(world, project, written), written.clone()))
        .filter(|(id, _)| {
            !(&id.project == project && open_keys.contains(id.key.as_str()))
                && !world.issues.contains_key(id)
        })
        .collect()
}

impl Trackers {
    /// Whether `issue` is closed, as the tracker of its project or area says, or
    /// why that cannot be known, to follow "an issue depends on KEY".
    fn ask(
        &mut self,
        sources: &[(ProjectId, String, String)],
        issue: &IssueId,
    ) -> Result<bool, String> {
        // A dependency on a name nothing matches keeps that name in its key.
        if let Some((name, _)) = issue.key.split_once('#') {
            return Err(format!(
                ", and no configured project or area is named {name}"
            ));
        }
        let source = &issue.project;
        let Some((_, tracker_id, name)) = sources.iter().find(|(id, _, _)| id == source) else {
            return Err(format!(", and {source} has no tracker configured"));
        };
        let Some(tracker) = self.adapters.iter_mut().find(|a| a.id() == tracker_id) else {
            return Err(format!(
                ", and no tracker \"{tracker_id}\" in this build reads {source}"
            ));
        };
        tracker
            .closed(name, &issue.key)
            .map_err(|e| format!(", which the {tracker_id} tracker cannot confirm closed: {e}"))
    }

    /// Adds a tracker. The root package calls this once per compiled-in adapter.
    pub fn add(&mut self, adapter: Box<dyn Tracker>) {
        self.adapters.push(adapter);
    }

    /// The ids of the trackers in this build.
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }

    /// Syncs every `ms` whatever the `sync_every` setting says.
    pub fn every(mut self, ms: u64) -> Self {
        self.every_ms = Some(ms);
        self
    }

    /// The text of `issue` from the last sync of its project or area, or why
    /// there is none.
    fn read(&self, world: &World, issue: &IssueId) -> Result<IssueText, String> {
        let source = &issue.project;
        let Some((_, tracker_id, name)) =
            sources(world).into_iter().find(|(id, _, _)| id == source)
        else {
            return Err(format!("{source} has no tracker configured"));
        };
        if !self.adapters.iter().any(|a| a.id() == tracker_id) {
            return Err(format!(
                "no tracker \"{tracker_id}\" in this build reads {source}"
            ));
        }
        let Some(texts) = self.texts.get(source) else {
            let failed = self
                .errors
                .get(source)
                .map_or_else(String::new, |e| format!(": {e}"));
            return Err(format!(
                "{source} has not synced since the daemon started{failed}"
            ));
        };
        let key = &issue.key;
        let Some(kept) = texts.get(key) else {
            return Err(format!(
                "the last sync of {name} that worked did not list issue {key}; a sync lists open \
                 issues only"
            ));
        };
        let at = if kept.listed_last {
            "worked"
        } else {
            "listed it"
        };
        Ok(IssueText {
            from: format!(
                "issue {key} of {name}, as the {tracker_id} tracker returned it at the last sync \
                 that {at}"
            ),
            text: kept.text.clone(),
        })
    }
}

impl Part for Trackers {
    fn id(&self) -> &'static str {
        "tracker"
    }

    fn poll(&mut self, world: &World, now_ms: u64) -> Polled {
        let mut events = Vec::new();
        let waiting = |id: &IssueId| world.issues.get(id).is_some_and(|i| i.reopen_due);
        self.tried.retain(waiting);
        self.reopened.retain(waiting);
        let sources = sources(world);
        for (source, tracker_id, name) in sources.iter().cloned() {
            let every = self
                .every_ms
                .unwrap_or_else(|| world.source_number(&source, "sync_every") * 1000);
            let reopen: Vec<&Issue> = world
                .issues
                .values()
                .filter(|i| i.id.project == source && i.reopen_due)
                .collect();
            // A failed verify syncs its issue's source at once, to reopen it.
            let due = reopen.iter().any(|i| !self.tried.contains(&i.id))
                || self
                    .synced_at
                    .get(&source)
                    .is_none_or(|at| now_ms >= at + every);
            if !due {
                continue;
            }
            self.synced_at.insert(source.clone(), now_ms);
            self.tried.extend(reopen.iter().map(|i| i.id.clone()));
            let Some(tracker) = self.adapters.iter_mut().find(|a| a.id() == tracker_id) else {
                self.errors.insert(
                    source.clone(),
                    format!("{source}: no tracker \"{tracker_id}\" in this build"),
                );
                continue;
            };
            match tracker.issues(&name) {
                Ok(open) => {
                    self.errors.remove(&source);
                    let mut unsure = Vec::new();
                    // Closed by the loop, still open in the tracker: the close failed; try again.
                    for t in &open {
                        let id = IssueId::new(source.0.clone(), t.key.as_str());
                        if world.issues.get(&id).is_some_and(|i| i.closed_by_loop)
                            && let Err(e) = tracker.close(&name, &t.key)
                        {
                            unsure.push(format!("closing {id} failed: {e}"));
                        }
                    }
                    // Sent back by a failed verify and not listed: the merge closed it,
                    // the last reopen failed, or the tracker does not list the reopen
                    // yet. The reopen counts once a sync lists the issue (`changes`
                    // passes that on); until then a close of it stays held. After that,
                    // its text is read at each sync, and a close there is a person's.
                    let listed: BTreeSet<&str> = open.iter().map(|t| t.key.as_str()).collect();
                    for i in reopen
                        .iter()
                        .filter(|i| !listed.contains(i.id.key.as_str()))
                    {
                        match tracker.reopen(&name, &i.id.key) {
                            Ok(()) => {
                                // Not news yet: a search such as Jira's may list the
                                // issue only seconds or minutes after the write. The
                                // first reopen brings the next sync forward.
                                if self.reopened.insert(i.id.clone()) {
                                    self.synced_at.remove(&source);
                                }
                            }
                            Err(e) => unsure.push(format!(
                                "{} failed its verify and cannot be reopened: {e}",
                                i.id
                            )),
                        }
                    }
                    let milestones = match tracker.milestones(&name) {
                        Ok(listed) => milestone_changes(world, &source, &listed),
                        Err(e) => {
                            unsure.push(format!("its milestones cannot be read: {e}"));
                            Vec::new()
                        }
                    };
                    // A dependency counts as done only once its tracker says it is
                    // closed; until then the issue waits.
                    for (issue, written) in unknown_dependencies(world, &source, &open) {
                        // Another project's open issue is listed by its own sync,
                        // unless that sync has worked and left it out, as it does an
                        // open pull request or an issue past the list's limit.
                        let listed_elsewhere = issue.project != source
                            && self
                                .texts
                                .get(&issue.project)
                                .is_none_or(|t| t.contains_key(&issue.key));
                        match self.ask(&sources, &issue) {
                            Ok(true) => events.push(Event::IssueSynced {
                                issue,
                                title: String::new(),
                                milestone: None,
                                depends_on: Vec::new(),
                                order: 0,
                                closed: true,
                            }),
                            Ok(false) if listed_elsewhere => {}
                            Ok(false) => unsure.push(format!(
                                "an issue depends on {written}, which is open but was not listed"
                            )),
                            Err(why) => unsure.push(format!("an issue depends on {written}{why}")),
                        }
                    }
                    if !unsure.is_empty() {
                        self.errors
                            .insert(source.clone(), format!("{}: {}", name, unsure.join("; ")));
                    }
                    events.extend(changes(world, &source, &open));
                    events.extend(milestones);
                    let mut texts: BTreeMap<String, Kept> = open
                        .into_iter()
                        .map(|t| {
                            let kept = Kept {
                                text: t.body,
                                listed_last: true,
                            };
                            (t.key, kept)
                        })
                        .collect();
                    // An issue whose close is held keeps the text the last sync that
                    // listed it read, so the fix after a failed verify can read it
                    // until the reopen happens.
                    if let Some(last) = self.texts.get(&source) {
                        for i in world
                            .issues
                            .values()
                            .filter(|i| i.id.project == source && held(i))
                        {
                            if let Some(kept) = last.get(&i.id.key) {
                                texts.entry(i.id.key.clone()).or_insert_with(|| Kept {
                                    listed_last: false,
                                    ..kept.clone()
                                });
                            }
                        }
                    }
                    self.texts.insert(source, texts);
                }
                Err(e) => {
                    self.errors.insert(source.clone(), format!("{}: {e}", name));
                }
            }
        }
        let problem = (!self.errors.is_empty())
            .then(|| self.errors.values().cloned().collect::<Vec<_>>().join("; "));
        Polled { events, problem }
    }

    fn issue_text(&self, world: &World, issue: &IssueId) -> Option<Result<IssueText, String>> {
        Some(self.read(world, issue))
    }

    fn addresses(&self, world: &World) -> BTreeMap<ProjectId, Addresses> {
        sources(world)
            .into_iter()
            .filter_map(|(id, tracker, name)| {
                let tracker = self.adapters.iter().find(|a| a.id() == tracker)?;
                let issues = tracker.address(&name)?;
                Some((
                    id,
                    Addresses {
                        issues: Some(issues),
                        code: None,
                    },
                ))
            })
            .collect()
    }

    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let source = match effect {
            Effect::Close { issue } => &issue.project,
            Effect::File { source, .. } => source,
            _ => return Outcome::NotMine,
        };
        let Some((_, tracker_id, name)) =
            sources(world).into_iter().find(|(id, _, _)| id == source)
        else {
            return Outcome::Failed(format!("{source} has no tracker configured"));
        };
        let Some(tracker) = self.adapters.iter_mut().find(|a| a.id() == tracker_id) else {
            return Outcome::Failed(format!("no tracker \"{tracker_id}\" in this build"));
        };
        match effect {
            Effect::File {
                group,
                agent,
                title,
                body,
                milestone,
                ..
            } => match tracker.create(&name, title, body, milestone.as_ref().map(|m| m.0.as_str()))
            {
                // Reported as synced at once, so the loop has it before the next
                // sync; that sync then finds nothing changed.
                Ok(created) => {
                    let synced = synced(world, source, &created);
                    let issue = IssueId::new(source.0.clone(), created.key.as_str());
                    Outcome::Done {
                        events: vec![
                            synced,
                            Event::IssueFiled {
                                group: group.clone(),
                                agent: agent.clone(),
                                issue,
                                title: created.title,
                                milestone: created.milestone.map(MilestoneId::new),
                            },
                        ],
                        effects: Vec::new(),
                    }
                }
                Err(e) => Outcome::Failed(format!("filing in {name}: {e}")),
            },
            Effect::Close { issue } => match tracker.close(&name, &issue.key) {
                Ok(()) => Outcome::done(),
                Err(e) => Outcome::Failed(e.to_string()),
            },
            _ => Outcome::NotMine,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temet_work_core::event::Verdict;
    use temet_work_core::ids::{AgentId, FolderId, GroupId, MachineId};
    use temet_work_core::model::{AgentState, Plan, Role};

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
            milestones: vec![
                (
                    "ankh/morpork".into(),
                    TrackedMilestone {
                        title: "Watch".into(),
                        open: 1,
                        due: Some("2026-12-12".into()),
                    },
                ),
                (
                    "ankh/morpork".into(),
                    TrackedMilestone {
                        title: "Snuff".into(),
                        open: 0,
                        due: None,
                    },
                ),
            ],
            ..FakeTracker::default()
        }
    }

    /// The milestone events among `events`, as (milestone, due date, open).
    fn milestones_in(events: &[Event]) -> Vec<(&str, Option<&str>, bool)> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::MilestoneSynced {
                    milestone,
                    due,
                    open,
                    ..
                } => Some((milestone.0.as_str(), due.as_deref(), *open)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_milestones_due_date_is_logged_once_and_again_only_when_it_changes() {
        let mut w = configured();
        let mut part = Trackers::default().every(0);
        part.add(Box::new(fake()));
        let first = part.poll(&w, 0).events;
        assert_eq!(
            milestones_in(&first),
            [("Watch", Some("2026-12-12"), true), ("Snuff", None, true)]
        );
        apply(&mut w, &first);
        assert_eq!(
            w.milestones[&ProjectId::new("ankh")][&MilestoneId::new("Watch")].as_deref(),
            Some("2026-12-12")
        );
        assert_eq!(
            part.poll(&w, 1).events,
            [],
            "an unchanged sync logs nothing"
        );
        // Moved a week, and Snuff closed.
        let mut moved = fake();
        moved.milestones[0].1.due = Some("2026-12-19".into());
        moved.milestones.pop();
        let mut part = Trackers::default().every(0);
        part.add(Box::new(moved));
        let next = part.poll(&w, 2).events;
        assert_eq!(
            milestones_in(&next),
            [("Watch", Some("2026-12-19"), true), ("Snuff", None, false)]
        );
        apply(&mut w, &next);
        assert_eq!(part.poll(&w, 3).events, []);
    }

    #[test]
    fn each_projects_issues_have_the_address_its_tracker_gives() {
        let mut w = configured();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("klatch"),
            tracker: "jira".into(),
            name: "KLA".into(),
            code_host: None,
            repo: None,
        });
        let mut part = Trackers::default();
        part.add(Box::new(fake()));
        let addresses = part.addresses(&w);
        assert_eq!(
            addresses[&ProjectId::new("ankh")].issues.as_deref(),
            Some("https://fake.example.com/ankh/morpork/issues")
        );
        // No tracker in this build gives klatch's.
        assert!(!addresses.contains_key(&ProjectId::new("klatch")));
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
            code_host: None,
            repo: None,
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
        // Two issues and two milestones.
        assert_eq!(first.events.len(), 4);
        apply(&mut w, &first.events);
        assert_eq!(part.poll(&w, 60_000 - 1).events, vec![], "not due yet");
        assert_eq!(part.poll(&w, 60_000).events, vec![], "nothing changed");
    }

    #[test]
    fn the_sync_every_setting_sets_how_often_a_project_is_synced() {
        use temet_work_core::settings::{Scope, Value};
        let mut w = configured();
        let source = w.projects.keys().next().expect("a project").clone();
        w.apply(&Event::SettingSet {
            scope: Scope::Project(source.clone()),
            role: None,
            key: "sync_every".into(),
            value: Value::Number(5),
            by: "person".into(),
        });
        let mut part = Trackers::default();
        part.add(Box::new(fake()));
        part.poll(&w, 0);
        part.poll(&w, 4_999);
        assert_eq!(part.synced_at[&source], 0, "five seconds have not passed");
        part.poll(&w, 5_000);
        assert_eq!(part.synced_at[&source], 5_000);
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

    /// A world that synced issue 2, took it to `stage` and then changed it with
    /// `change`.
    fn with_issue_two(stage: Stage, change: impl FnOnce(&mut Issue)) -> World {
        let mut w = configured();
        let t = issue("2", None);
        w.apply(&super::synced(&w, &ProjectId::new("ankh"), &t));
        if let Some(i) = w.issues.get_mut(&IssueId::new("ankh", "2")) {
            i.stage = stage;
            change(i);
        }
        w
    }

    const NOT_DONE: [Stage; 10] = [
        Stage::New,
        Stage::Refining,
        Stage::Ready,
        Stage::Working,
        Stage::InReview { round: 1 },
        Stage::Fixing { round: 1 },
        Stage::Arbitration { round: 3 },
        Stage::Merging,
        Stage::Deploying,
        Stage::Verifying,
    ];

    /// Whether `events` is one close of issue 2.
    fn closes_two(events: &[Event]) -> bool {
        matches!(events, [Event::IssueSynced { issue, closed: true, .. }] if issue.key == "2")
    }

    #[test]
    fn changes_leaves_out_a_close_only_while_the_issue_finishes_or_waits_on_its_reopen() {
        let ankh = ProjectId::new("ankh");
        for stage in NOT_DONE {
            let finishing = matches!(stage, Stage::Merging | Stage::Deploying | Stage::Verifying);
            // Merged work alone no longer holds a close back.
            let w = with_issue_two(stage, |i| i.has_merged = true);
            let events = changes(&w, &ankh, &[]);
            assert_eq!(closes_two(&events), !finishing, "{stage:?}: {events:?}");
            // A failed verify's reopen not yet seen does.
            let w = with_issue_two(stage, |i| {
                i.has_merged = true;
                i.reopen_due = true;
            });
            assert_eq!(changes(&w, &ankh, &[]), [], "{stage:?}");
        }
        // So does one of several pull requests merged before the rest.
        let w = with_issue_two(Stage::Fixing { round: 1 }, |i| {
            i.merged.insert(ProjectId::new("ankh"));
        });
        assert_eq!(changes(&w, &ankh, &[]), []);
        // Listed as open while its reopen is due: the reopen, whoever made it.
        let w = with_issue_two(Stage::Working, |i| i.reopen_due = true);
        let events = changes(&w, &ankh, &[issue("2", None)]);
        assert!(
            matches!(events.as_slice(), [Event::IssueSynced { issue, closed: false, .. }] if issue.key == "2"),
            "{events:?}"
        );
    }

    #[test]
    fn changes_reports_the_close_of_an_issue_that_merged_nothing() {
        let w = with_issue_two(Stage::Working, |_| ());
        let events = changes(&w, &ProjectId::new("ankh"), &[]);
        assert!(closes_two(&events), "{events:?}");
    }

    /// A fake the test still reaches after the part has taken it.
    #[derive(Clone)]
    struct Shared(std::sync::Arc<std::sync::Mutex<FakeTracker>>);

    impl Shared {
        fn fake(&self) -> std::sync::MutexGuard<'_, FakeTracker> {
            self.0.lock().expect("the fake")
        }

        /// Closes or opens issue `key` of ankh/morpork, as a merge or a person does.
        fn set_closed(&self, key: &str, closed: bool) {
            for (_, i) in self.fake().issues.iter_mut().filter(|(_, i)| i.key == key) {
                i.closed = closed;
            }
        }
    }

    impl Tracker for Shared {
        fn id(&self) -> &'static str {
            "fake"
        }
        fn issues(&mut self, project: &str) -> Result<Vec<TrackedIssue>, TrackerError> {
            self.fake().issues(project)
        }
        fn milestones(&mut self, project: &str) -> Result<Vec<TrackedMilestone>, TrackerError> {
            self.fake().milestones(project)
        }
        fn address(&self, project: &str) -> Option<String> {
            self.fake().address(project)
        }
        fn closed(&mut self, project: &str, key: &str) -> Result<bool, TrackerError> {
            self.fake().closed(project, key)
        }
        fn comment(&mut self, project: &str, key: &str, text: &str) -> Result<(), TrackerError> {
            self.fake().comment(project, key, text)
        }
        fn close(&mut self, project: &str, key: &str) -> Result<(), TrackerError> {
            self.fake().close(project, key)
        }
        fn reopen(&mut self, project: &str, key: &str) -> Result<(), TrackerError> {
            self.fake().reopen(project, key)
        }
        fn create(
            &mut self,
            project: &str,
            title: &str,
            body: &str,
            milestone: Option<&str>,
        ) -> Result<TrackedIssue, TrackerError> {
            self.fake().create(project, title, body, milestone)
        }
    }

    /// [`Shared`], except that the first listing after the first reopen still
    /// leaves the reopened issue out, as Jira's search can for seconds or
    /// minutes after a write.
    struct Lagging {
        shared: Shared,
        /// The issue the next listing leaves out.
        lag: Option<String>,
        /// Whether an issue has been reopened.
        reopened: bool,
    }

    impl Lagging {
        fn boxed(shared: Shared) -> Box<dyn Tracker> {
            Box::new(Self {
                shared,
                lag: None,
                reopened: false,
            })
        }
    }

    impl Tracker for Lagging {
        fn id(&self) -> &'static str {
            "fake"
        }
        fn issues(&mut self, project: &str) -> Result<Vec<TrackedIssue>, TrackerError> {
            let mut listed = self.shared.issues(project)?;
            if let Some(key) = self.lag.take() {
                listed.retain(|t| t.key != key);
            }
            Ok(listed)
        }
        fn milestones(&mut self, project: &str) -> Result<Vec<TrackedMilestone>, TrackerError> {
            self.shared.milestones(project)
        }
        fn address(&self, project: &str) -> Option<String> {
            self.shared.address(project)
        }
        fn closed(&mut self, project: &str, key: &str) -> Result<bool, TrackerError> {
            self.shared.closed(project, key)
        }
        fn comment(&mut self, project: &str, key: &str, text: &str) -> Result<(), TrackerError> {
            self.shared.comment(project, key, text)
        }
        fn close(&mut self, project: &str, key: &str) -> Result<(), TrackerError> {
            self.shared.close(project, key)
        }
        fn reopen(&mut self, project: &str, key: &str) -> Result<(), TrackerError> {
            self.shared.reopen(project, key)?;
            if !self.reopened {
                self.reopened = true;
                self.lag = Some(key.to_string());
            }
            Ok(())
        }
        fn create(
            &mut self,
            project: &str,
            title: &str,
            body: &str,
            milestone: Option<&str>,
        ) -> Result<TrackedIssue, TrackerError> {
            self.shared.create(project, title, body, milestone)
        }
    }

    const WIND: &str = "Wind the clock.\nDone is a clock that strikes thirteen.";

    /// Issue 1 of `tracker`, whose text is [`WIND`], taken through review, the
    /// merge (which closes it in the fake), a sync while it deploys, and a failed
    /// verify, with author-2 started on its fix. Issue 2 depends on it. Returns
    /// the world, the part, which has made no sync since the verify, and the fake.
    fn failed_verify(tracker: FakeTracker) -> (World, Trackers, Shared) {
        failed_verify_through(tracker, |shared| Box::new(shared))
    }

    /// [`failed_verify`], with the part reaching the fake through `wrap`.
    fn failed_verify_through(
        mut tracker: FakeTracker,
        wrap: impl FnOnce(Shared) -> Box<dyn Tracker>,
    ) -> (World, Trackers, Shared) {
        let ankh = ProjectId::new("ankh");
        let (one, two) = (IssueId::new("ankh", "1"), IssueId::new("ankh", "2"));
        let group = GroupId::new("watch");
        tracker.issues[0].1.body = WIND.into();
        tracker.issues[1].1.milestone = Some("Watch".into());
        tracker.issues[1].1.depends_on = vec!["1".into()];
        let shared = Shared(std::sync::Arc::new(std::sync::Mutex::new(tracker)));
        let mut part = Trackers::default().every(0);
        part.add(wrap(shared.clone()));
        let mut w = configured();
        for f in ["f1", "f2"] {
            w.apply(&Event::FolderAdded {
                folder: FolderId::new(f),
                project: ankh.clone(),
                machine: MachineId::new("morpork"),
                path: format!("/srv/ankh/{f}"),
            });
        }
        w.apply(&Event::GroupCreated {
            group: group.clone(),
            name: "Night watch".into(),
            projects: vec![ankh.clone()],
            areas: Vec::new(),
            milestones: vec![MilestoneId::new("Watch")],
            refining_at_once: 1,
            model: None,
            queue_low: None,
            investigate_every: None,
        });
        let first = part.poll(&w, 0).events;
        apply(&mut w, &first);
        for issue in [&one, &two] {
            w.apply(&Event::PlanWritten {
                issue: issue.clone(),
                plan: Plan::default(),
            });
        }
        apply(
            &mut w,
            &[
                Event::PullRequestReady {
                    issue: one.clone(),
                    pull: Some("57".into()),
                    pulls: Default::default(),
                    replies: Vec::new(),
                },
                Event::ReviewFinished {
                    issue: one.clone(),
                    round: 1,
                    verdict: Verdict::Approve,
                    failed: None,
                    open: None,
                },
                Event::ChecksFinished {
                    issue: one.clone(),
                    passed: true,
                    failing: Vec::new(),
                },
                Event::Merged {
                    issue: one.clone(),
                    repo: None,
                },
            ],
        );
        // The pull request said "Closes #1".
        shared.set_closed("1", true);
        assert_eq!(
            part.poll(&w, 1).events,
            [],
            "a close while deploying is held"
        );
        apply(
            &mut w,
            &[
                Event::Deployed { issue: one.clone() },
                Event::Verified {
                    issue: one.clone(),
                    passed: false,
                    output: "the clock struck twelve".into(),
                },
                Event::AgentStarted {
                    agent: AgentId::new("author-2"),
                    group,
                    role: Role::Author,
                    issue: Some(one.clone()),
                    machine: MachineId::new("morpork"),
                    folder: None,
                    secret_digest: None,
                    runtime: None,
                    model: None,
                    checkouts: Vec::new(),
                },
            ],
        );
        (w, part, shared)
    }

    /// Polls `part` at `now`, applies what it saw to `w`, and returns the events
    /// and the effects they led to.
    fn sync_and_apply(part: &mut Trackers, w: &mut World, now: u64) -> (Vec<Event>, Vec<Effect>) {
        let events = part.poll(w, now).events;
        let effects = events.iter().flat_map(|e| w.apply(e)).collect();
        (events, effects)
    }

    /// The world, part and fake of [`failed_verify`] on the plain fake, after the
    /// sync that reopens issue 1, the one that lists it, and one more.
    fn reopened() -> (World, Trackers, Shared) {
        let (mut w, mut part, shared) = failed_verify(fake());
        // The reopen is not news until a sync lists the issue.
        assert_eq!(sync_and_apply(&mut part, &mut w, 2), (vec![], vec![]));
        assert!(w.issues[&IssueId::new("ankh", "1")].reopen_due);
        let (events, effects) = sync_and_apply(&mut part, &mut w, 3);
        assert!(
            matches!(events.as_slice(), [Event::IssueSynced { issue, closed: false, .. }] if issue.key == "1"),
            "{events:?}"
        );
        assert_eq!(effects, []);
        assert_eq!(sync_and_apply(&mut part, &mut w, 4), (vec![], vec![]));
        (w, part, shared)
    }

    #[test]
    fn a_failed_verify_reopens_the_issue_in_its_tracker_with_no_comment_and_its_text_is_read() {
        let (one, two) = (IssueId::new("ankh", "1"), IssueId::new("ankh", "2"));
        let (w, part, shared) = reopened();
        assert_eq!(shared.fake().closed("ankh/morpork", "1"), Ok(false));
        assert_eq!(shared.fake().comments, []);

        let i = &w.issues[&one];
        assert!(!i.reopen_due && !i.has_merged && i.verify_failed);
        assert!(
            matches!(i.stage, Stage::Ready | Stage::Working),
            "{:?}",
            i.stage
        );
        assert!(i.fix[0].ends_with("the clock struck twelve"), "{:?}", i.fix);
        assert_ne!(w.agents[&AgentId::new("author-2")].state, AgentState::Ended);
        let folder = i.folder.clone().expect("a folder for the fix");
        assert_eq!(w.folders[&folder].holder.as_ref(), Some(&one));
        // Issue 2 waits, though a folder is free.
        assert_eq!(w.issues[&two].stage, Stage::Ready);
        assert!(w.folders.values().any(|f| f.holder.is_none()));

        let read = part.issue_text(&w, &one).expect("the tracker part answers");
        assert_eq!(read.as_ref().map(|t| t.text.as_str()), Ok(WIND));
        let shown = temet_work_core::text::issue(&w, &one, &read).expect("a known issue");
        assert!(
            shown.contains(&format!(
                "Text of issue 1 of ankh/morpork, as the fake tracker returned it at the last \
                 sync that worked:\n{WIND}\nEnd of the issue's text."
            )),
            "{shown}"
        );
        assert!(!shown.contains("a sync lists open issues only"), "{shown}");
        let to_fix = shown.split_once("To fix:").map(|(_, rest)| rest);
        assert!(
            to_fix.is_some_and(|t| t.contains("the clock struck twelve")),
            "{shown}"
        );
    }

    #[test]
    fn a_close_after_the_reopen_finishes_the_issue_as_a_persons() {
        let (one, two) = (IssueId::new("ankh", "1"), IssueId::new("ankh", "2"));
        let (mut w, mut part, shared) = reopened();
        let folder = w.issues[&one].folder.clone().expect("a folder for the fix");
        shared.set_closed("1", true);
        let (events, effects) = sync_and_apply(&mut part, &mut w, 5);
        assert!(
            matches!(events.as_slice(), [Event::IssueSynced { issue, closed: true, .. }] if issue == &one),
            "{events:?}"
        );
        let i = &w.issues[&one];
        assert_eq!(i.stage, Stage::Done);
        assert!(i.closed_by_a_person());
        assert_eq!(i.folder, None);
        assert!(effects.contains(&Effect::StopAgent {
            agent: AgentId::new("author-2")
        }));
        assert!(effects.contains(&Effect::ReleaseFolder {
            folder: folder.clone()
        }));
        assert!(w.folders.values().all(|f| f.holder.as_ref() != Some(&one)));
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::StartAgent { issue, role: Role::Author, .. } if issue == &two)),
            "{effects:?}"
        );
    }

    #[test]
    fn a_failed_reopen_holds_the_close_is_a_problem_keeps_the_text_and_is_tried_at_each_sync() {
        let one = IssueId::new("ankh", "1");
        let refused = TrackerError::Other(
            "1 has no transition out of a Done status; its transitions are: none".into(),
        );
        let tracker = FakeTracker {
            reopen_refused: Some(refused),
            ..fake()
        };
        let (mut w, mut part, shared) = failed_verify(tracker);
        for now in [2, 3] {
            let polled = part.poll(&w, now);
            assert!(
                !polled
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::IssueSynced { closed: true, .. })),
                "poll {now}: {:?}",
                polled.events
            );
            apply(&mut w, &polled.events);
            assert_eq!(
                polled.problem.as_deref(),
                Some(
                    "ankh/morpork: ankh#1 failed its verify and cannot be reopened: 1 has no \
                     transition out of a Done status; its transitions are: none"
                ),
                "poll {now}"
            );
            let read = part.issue_text(&w, &one).expect("the tracker part answers");
            assert_eq!(
                read,
                Ok(IssueText {
                    from: "issue 1 of ankh/morpork, as the fake tracker returned it at the last \
                           sync that listed it"
                        .into(),
                    text: WIND.into(),
                }),
                "poll {now}"
            );
        }
        assert!(w.issues[&one].reopen_due);
        assert_ne!(w.issues[&one].stage, Stage::Done);

        shared.fake().reopen_refused = None;
        let polled = part.poll(&w, 4);
        assert_eq!(polled.events, []);
        assert_eq!(polled.problem, None);
        assert_eq!(shared.fake().closed("ankh/morpork", "1"), Ok(false));
        // The next sync lists it: the reopen.
        let (events, _) = sync_and_apply(&mut part, &mut w, 5);
        assert!(
            matches!(events.as_slice(), [Event::IssueSynced { issue, closed: false, .. }] if issue == &one),
            "{events:?}"
        );
        assert!(!w.issues[&one].reopen_due);
    }

    #[test]
    fn a_close_in_a_listing_that_does_not_show_the_reopen_yet_stays_held() {
        let ankh = ProjectId::new("ankh");
        let (one, two) = (IssueId::new("ankh", "1"), IssueId::new("ankh", "2"));
        let (mut w, mut part, shared) = failed_verify_through(fake(), Lagging::boxed);
        // Synced every minute: the last sync was at 1.
        part.every_ms = Some(60_000);
        // Reopened at once after the failed verify, and not news yet.
        assert_eq!(sync_and_apply(&mut part, &mut w, 2), (vec![], vec![]));
        assert_eq!(shared.fake().closed("ankh/morpork", "1"), Ok(false));

        // The next sync comes at once, and its listing still leaves issue 1 out.
        let (events, effects) = sync_and_apply(&mut part, &mut w, 3);
        assert_eq!(part.synced_at[&ankh], 3);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::IssueSynced { closed: true, .. })),
            "{events:?}"
        );
        assert_eq!(effects, []);
        let i = &w.issues[&one];
        assert_ne!(i.stage, Stage::Done);
        assert!(i.reopen_due && i.has_merged);
        assert!(i.fix[0].ends_with("the clock struck twelve"), "{:?}", i.fix);
        assert_eq!(w.issues[&two].stage, Stage::Ready);
        let read = part.issue_text(&w, &one).expect("the tracker part answers");
        assert_eq!(read.map(|t| t.text), Ok(WIND.to_string()));

        // A tracker slow to list the reopen is not asked again before the next sync.
        part.poll(&w, 4);
        assert_eq!(part.synced_at[&ankh], 3);

        // The next listing shows it: the reopen.
        let (events, effects) = sync_and_apply(&mut part, &mut w, 60_003);
        assert!(
            matches!(events.as_slice(), [Event::IssueSynced { issue, closed: false, .. }] if issue == &one),
            "{events:?}"
        );
        assert_eq!(effects, []);
        let i = &w.issues[&one];
        assert!(!i.reopen_due && !i.has_merged);
        assert!(i.fix[0].ends_with("the clock struck twelve"), "{:?}", i.fix);
    }

    #[test]
    fn a_failed_verify_syncs_at_once_and_a_failed_reopen_waits_for_the_next_sync() {
        let tracker = FakeTracker {
            reopen_refused: Some(TrackerError::Denied("signed out".into())),
            ..fake()
        };
        let (w, mut part, _) = failed_verify(tracker);
        // Synced every minute: the last sync was at 1.
        part.every_ms = Some(60_000);
        let tried = |part: &mut Trackers, now| part.poll(&w, now).problem.is_some();
        assert!(tried(&mut part, 2), "at once after the failed verify");
        assert_eq!(part.synced_at[&ProjectId::new("ankh")], 2);
        part.poll(&w, 3);
        assert_eq!(
            part.synced_at[&ProjectId::new("ankh")],
            2,
            "not again before the next sync"
        );
        part.poll(&w, 60_002);
        assert_eq!(part.synced_at[&ProjectId::new("ankh")], 60_002);
    }

    #[test]
    fn a_dependency_counts_as_done_only_when_the_tracker_says_it_is_closed() {
        let mut tracker = fake();
        let mut closed = issue("40", None);
        closed.closed = true;
        tracker.issues.push(("ankh/morpork".into(), closed));
        tracker.issues[0].1.depends_on = vec!["40".into(), "ENG-41".into()];
        let mut part = Trackers::default();
        part.add(Box::new(tracker));
        let polled = part.poll(&configured(), 0);
        let closed: Vec<&str> = polled
            .events
            .iter()
            .filter_map(|e| match e {
                Event::IssueSynced {
                    issue,
                    closed: true,
                    ..
                } => Some(issue.key.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(closed, ["40"], "ENG-41 is not confirmed closed");
        assert!(polled.problem.is_some_and(|p| p.contains("ENG-41")));
    }

    /// Adds project `id`, whose issues `tracker` holds as `name`.
    fn add_project(w: &mut World, id: &str, tracker: &str, name: &str) {
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new(id),
            tracker: tracker.into(),
            name: name.into(),
            code_host: None,
            repo: None,
        });
    }

    /// The fake with `line` as what issue 1 of ankh depends on.
    fn depending(line: &str) -> FakeTracker {
        let mut tracker = fake();
        tracker.issues[0].1.depends_on = depends_on(line);
        tracker
    }

    /// Closed issue `key` of `name`: never listed, as GitHub has a merged pull
    /// request.
    fn closed_issue(name: &str, key: &str) -> (String, TrackedIssue) {
        let closed = TrackedIssue {
            closed: true,
            ..issue(key, None)
        };
        (name.into(), closed)
    }

    /// The issues `events` sync as closed.
    fn synced_closed(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::IssueSynced {
                    issue,
                    closed: true,
                    ..
                } => Some(issue.to_string()),
                _ => None,
            })
            .collect()
    }

    /// Asserts that ankh#1 depends on `on` alone, and that it is done.
    fn depends_on_done(w: &World, on: &IssueId) {
        assert_eq!(
            w.issues[&IssueId::new("ankh", "1")].depends_on,
            std::slice::from_ref(on)
        );
        assert_eq!(w.issues[on].stage, Stage::Done);
    }

    #[test]
    fn a_dependency_on_a_merged_pull_request_is_synced_as_closed() {
        for line in ["Depends on #57.", "Depends on ankh/morpork#57."] {
            let mut tracker = depending(line);
            tracker.issues.push(closed_issue("ankh/morpork", "57"));
            let mut part = Trackers::default().every(0);
            part.add(Box::new(tracker));
            let mut w = configured();
            let polled = part.poll(&w, 0);
            assert_eq!(polled.problem, None, "{line}");
            assert_eq!(synced_closed(&polled.events), ["ankh#57"], "{line}");
            apply(&mut w, &polled.events);
            depends_on_done(&w, &IssueId::new("ankh", "57"));
            assert_eq!(part.poll(&w, 1).events, [], "{line}");
        }
    }

    #[test]
    fn a_dependency_on_another_projects_closed_issue_is_asked_of_its_tracker() {
        for (line, closed) in [
            ("Depends on klatch#4", true),
            ("Depends on desert/klatch#4", true),
            ("Depends on klatch#4", false),
        ] {
            let mut w = configured();
            add_project(&mut w, "klatch", "other", "desert/klatch");
            // The depending issue's tracker has an open issue of the same name
            // and key, so asking it gives the wrong answer.
            let mut ankh = depending(line);
            ankh.issues.push(("desert/klatch".into(), issue("4", None)));
            let mut klatch = FakeTracker::named("other");
            let mut four = closed_issue("desert/klatch", "4");
            four.1.closed = closed;
            klatch.issues.push(four);
            let mut part = Trackers::default().every(0);
            part.add(Box::new(ankh));
            part.add(Box::new(klatch));
            let polled = part.poll(&w, 0);
            // An open one is listed by klatch's own sync, and is no problem.
            assert_eq!(polled.problem, None, "{line} {closed}");
            let expected: &[&str] = if closed { &["klatch#4"] } else { &[] };
            assert_eq!(synced_closed(&polled.events), expected, "{line} {closed}");
            apply(&mut w, &polled.events);
            if closed {
                depends_on_done(&w, &IssueId::new("klatch", "4"));
            } else {
                assert_ne!(w.issues[&IssueId::new("klatch", "4")].stage, Stage::Done);
            }
            assert_eq!(part.poll(&w, 1).events, [], "{line} {closed}");
        }
    }

    #[test]
    fn another_projects_open_issue_its_sync_does_not_list_is_a_problem() {
        let mut w = configured();
        add_project(&mut w, "klatch", "other", "desert/klatch");
        let mut klatch = FakeTracker::named("other");
        klatch
            .issues
            .push(("desert/klatch".into(), issue("4", None)));
        klatch.unlisted.push(("desert/klatch".into(), "4".into()));
        let mut part = Trackers::default().every(0);
        part.add(Box::new(depending("Depends on klatch#4")));
        part.add(Box::new(klatch));
        // ankh syncs before klatch has: klatch's own sync may list it.
        let first = part.poll(&w, 0);
        assert_eq!(first.problem, None);
        apply(&mut w, &first.events);
        assert!(!w.issues.contains_key(&IssueId::new("klatch", "4")));
        // klatch's sync worked and left it out.
        assert_eq!(
            part.poll(&w, 1).problem.as_deref(),
            Some("ankh/morpork: an issue depends on klatch#4, which is open but was not listed")
        );
    }

    #[test]
    fn a_dependency_no_tracker_can_be_asked_about_is_a_problem() {
        let mut w = configured();
        add_project(&mut w, "uberwald", "", "ankh/uberwald");
        add_project(&mut w, "klatch", "nonesuch", "desert/klatch");
        add_project(&mut w, "quirm", "other", "ankh/quirm");
        let mut ankh = depending(
            "Depends on nowhere#5, ankh/ramtops#6, uberwald#7, klatch#8 and ankh/quirm#9",
        );
        // The depending issue's tracker would call each of these closed.
        for (name, key) in [
            ("ankh/ramtops", "6"),
            ("ankh/uberwald", "7"),
            ("desert/klatch", "8"),
            ("ankh/quirm", "9"),
        ] {
            ankh.issues.push(closed_issue(name, key));
        }
        // Quirm's tracker has the project and no issue 9.
        let mut quirm = FakeTracker::named("other");
        quirm.issues.push(("ankh/quirm".into(), issue("1", None)));
        let mut part = Trackers::default();
        part.add(Box::new(ankh));
        part.add(Box::new(quirm));
        let polled = part.poll(&w, 0);
        assert_eq!(synced_closed(&polled.events), Vec::<String>::new());
        let problem = polled.problem.expect("problems");
        for line in [
            "nowhere#5, and no configured project or area is named nowhere",
            "ankh/ramtops#6, and no configured project or area is named ankh/ramtops",
            "uberwald#7, and uberwald has no tracker configured",
            "klatch#8, and no tracker \"nonesuch\" in this build reads klatch",
            "ankh/quirm#9, which the other tracker cannot confirm closed: not found: ankh/quirm#9",
        ] {
            let line = format!("an issue depends on {line}");
            assert!(problem.contains(&line), "{line}\nnot in: {problem}");
        }
        apply(&mut w, &polled.events);
        assert!(!w.issues.values().any(|i| i.stage == Stage::Done));
        assert!(part.poll(&w, 1).problem.is_some(), "still shown");
    }

    #[test]
    fn a_failing_project_stays_a_problem_until_it_syncs() {
        let mut w = configured();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("klatch"),
            tracker: "fake".into(),
            name: "no-such/project".into(),
            code_host: None,
            repo: None,
        });
        let mut part = Trackers::default();
        part.add(Box::new(fake()));
        let first = part.poll(&w, 0);
        assert_eq!(first.events.len(), 4, "the project that works still syncs");
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
    fn a_reopened_issue_is_a_change_and_a_failed_close_is_retried() {
        let mut w = configured();
        let mut part = Trackers::default().every(0);
        part.add(Box::new(fake()));
        let first = part.poll(&w, 0).events;
        apply(&mut w, &first);
        // Issue 1 closed in the tracker earlier (so not by the loop); issue 2 the loop closed.
        for (key, by_loop) in [("1", false), ("2", true)] {
            if let Some(i) = w.issues.get_mut(&IssueId::new("ankh", key)) {
                i.stage = Stage::Done;
                i.closed_by_loop = by_loop;
            }
        }
        let polled = part.poll(&w, 1);
        let synced: Vec<&str> = polled
            .events
            .iter()
            .filter_map(|e| match e {
                Event::IssueSynced {
                    issue,
                    closed: false,
                    ..
                } => Some(issue.key.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(synced, ["1"], "only the reopened one");
        assert_eq!(polled.problem, None, "the retried close worked");
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
    fn a_filed_issue_is_reported_at_once_and_the_next_sync_finds_nothing_changed() {
        let mut w = configured();
        let mut part = Trackers::default().every(0);
        part.add(Box::new(fake()));
        let first = part.poll(&w, 0).events;
        apply(&mut w, &first);
        let file = Effect::File {
            group: GroupId::new("watch"),
            agent: AgentId::new("investigator-1"),
            source: ProjectId::new("ankh"),
            title: "Wind the clock".into(),
            body: "Depends on #1".into(),
            milestone: Some(MilestoneId::new("Watch")),
        };
        let Outcome::Done { events, effects } = part.carry_out(&file, &w) else {
            panic!("not filed");
        };
        assert!(effects.is_empty());
        let issue = IssueId::new("ankh", "3");
        assert_eq!(
            events,
            [
                Event::IssueSynced {
                    issue: issue.clone(),
                    title: "Wind the clock".into(),
                    milestone: Some(MilestoneId::new("Watch")),
                    depends_on: vec![IssueId::new("ankh", "1")],
                    order: 3,
                    closed: false,
                },
                Event::IssueFiled {
                    group: GroupId::new("watch"),
                    agent: AgentId::new("investigator-1"),
                    issue,
                    title: "Wind the clock".into(),
                    milestone: Some(MilestoneId::new("Watch")),
                },
            ]
        );
        apply(&mut w, &events);
        assert_eq!(
            part.poll(&w, 1).events,
            vec![],
            "the next sync changes nothing"
        );
        let elsewhere = Effect::File {
            group: GroupId::new("watch"),
            agent: AgentId::new("investigator-1"),
            source: ProjectId::new("klatch"),
            title: "Wind the clock".into(),
            body: String::new(),
            milestone: None,
        };
        assert!(matches!(part.carry_out(&elsewhere, &w), Outcome::Failed(_)));
    }

    /// A part whose one tracker is the fake with `body` as issue 1's text.
    fn reading(body: &str) -> Trackers {
        let mut tracker = fake();
        tracker.issues[0].1.body = body.into();
        let mut part = Trackers::default();
        part.add(Box::new(tracker));
        part
    }

    /// What `part` gives for issue `key` of ankh: its text, or why not.
    fn text_of(part: &Trackers, w: &World, key: &str) -> Result<String, String> {
        part.issue_text(w, &IssueId::new("ankh", key))
            .expect("a tracker part reads issue text")
            .map(|t| t.text)
    }

    #[test]
    fn an_issues_text_is_the_one_the_last_sync_listed() {
        let w = configured();
        let mut part = reading("Wind it.\nDone is a clock that strikes.");
        part.poll(&w, 0);
        let read = part.issue_text(&w, &IssueId::new("ankh", "1"));
        assert_eq!(
            read,
            Some(Ok(IssueText {
                from: "issue 1 of ankh/morpork, as the fake tracker returned it at the last \
                       sync that worked"
                    .into(),
                text: "Wind it.\nDone is a clock that strikes.".into(),
            }))
        );
        // Edited in the tracker.
        part.adapters = reading("Wind it twice.").adapters;
        part.poll(&w, 60_000 - 1);
        assert_eq!(
            text_of(&part, &w, "1").as_deref(),
            Ok("Wind it.\nDone is a clock that strikes."),
            "not synced since the edit"
        );
        part.poll(&w, 60_000);
        assert_eq!(text_of(&part, &w, "1").as_deref(), Ok("Wind it twice."));
    }

    #[test]
    fn an_issue_asked_about_before_any_sync_has_not_been_read() {
        let w = configured();
        let part = reading("Wind it.");
        assert_eq!(
            text_of(&part, &w, "1"),
            Err("ankh has not synced since the daemon started".into())
        );
        // A first sync that fails says why.
        let mut part = Trackers::default();
        part.add(Box::new(FakeTracker::default()));
        part.poll(&w, 0);
        assert_eq!(
            text_of(&part, &w, "1"),
            Err(
                "ankh has not synced since the daemon started: ankh/morpork: not found: \
                 ankh/morpork"
                    .into()
            )
        );
    }

    #[test]
    fn an_issue_the_last_sync_did_not_list_has_no_text() {
        let w = configured();
        let mut part = reading("Wind it.");
        part.poll(&w, 0);
        assert_eq!(
            text_of(&part, &w, "9"),
            Err(
                "the last sync of ankh/morpork that worked did not list issue 9; a sync lists \
                 open issues only"
                    .into()
            )
        );
        // Closed in the tracker, so the next sync leaves it out.
        let mut gone = fake();
        gone.issues.retain(|(_, i)| i.key != "1");
        part.adapters = vec![Box::new(gone)];
        part.poll(&w, 60_000);
        assert!(text_of(&part, &w, "1").is_err_and(|why| why.contains("did not list issue 1")));
    }

    #[test]
    fn a_sync_that_fails_keeps_the_text_of_the_last_one_that_worked() {
        let w = configured();
        let mut part = reading("Wind it.");
        part.poll(&w, 0);
        part.adapters = vec![Box::new(FakeTracker::default())];
        assert!(part.poll(&w, 60_000).problem.is_some(), "the sync failed");
        assert_eq!(text_of(&part, &w, "1").as_deref(), Ok("Wind it."));
    }

    #[test]
    fn an_empty_text_is_read_as_empty() {
        let w = configured();
        let mut part = reading("");
        part.poll(&w, 0);
        assert_eq!(text_of(&part, &w, "1").as_deref(), Ok(""));
    }

    #[test]
    fn a_project_no_tracker_in_the_build_reads_has_no_text() {
        let mut w = two_sources();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("klatch"),
            tracker: "nonesuch".into(),
            name: "desert/klatch".into(),
            code_host: None,
            repo: None,
        });
        let mut part = two();
        part.poll(&w, 0);
        let read = |source: &str| part.issue_text(&w, &IssueId::new(source, "1"));
        assert_eq!(
            read("klatch"),
            Some(Err(
                "no tracker \"nonesuch\" in this build reads klatch".into()
            ))
        );
        assert_eq!(
            read("uberwald"),
            Some(Err("uberwald has no tracker configured".into()))
        );
        // An area's issue is read from the area's own tracker.
        assert!(read("guild").is_some_and(|r| {
            r.is_ok_and(|t| t.from.starts_with("issue 1 of GLD, as the other tracker"))
        }));
    }

    #[test]
    fn keeps_adapters_in_the_order_added() {
        let mut part = Trackers::default();
        part.add(Box::new(FakeTracker::default()));
        assert_eq!(part.ids(), ["fake"]);
        assert_eq!(part.id(), "tracker");
    }

    /// The tracker's names for the sources of [`two_sources`] and of a project
    /// whose tracker the build lacks.
    const NAMES: [&str; 3] = ["ankh/morpork", "GLD", "desert/klatch"];

    /// Project ankh on tracker "fake" and area guild on tracker "other".
    fn two_sources() -> World {
        let mut w = configured();
        w.apply(&Event::AreaDefined {
            area: ProjectId::new("guild"),
            name: "Guild".into(),
            projects: vec![],
            tracker: "other".into(),
            tracker_name: "GLD".into(),
        });
        w
    }

    /// Trackers "fake" and "other", in that order. Each has an issue 1, titled
    /// with its id, under every name, so reaching the wrong one shows.
    fn two() -> Trackers {
        let mut part = Trackers::default();
        for id in ["fake", "other"] {
            let mut tracker = FakeTracker::named(id);
            tracker.issues = NAMES
                .map(|name| {
                    let titled = TrackedIssue {
                        title: id.into(),
                        ..issue("1", None)
                    };
                    (name.into(), titled)
                })
                .into();
            part.add(Box::new(tracker));
        }
        part
    }

    /// Each synced issue, as (issue, title).
    fn synced(events: &[Event]) -> Vec<(String, &str)> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::IssueSynced { issue, title, .. } => {
                    Some((issue.to_string(), title.as_str()))
                }
                _ => None,
            })
            .collect()
    }

    /// Whether issue 1 under `name` is closed, in "fake" and in "other".
    fn closed_in(part: &mut Trackers, name: &str) -> Vec<bool> {
        part.adapters
            .iter_mut()
            .map(|a| a.closed(name, "1").expect("there"))
            .collect()
    }

    #[test]
    fn two_adapters_a_project_and_an_area_each_sync_from_their_own() {
        let polled = two().poll(&two_sources(), 0);
        assert_eq!(polled.problem, None);
        assert_eq!(
            synced(&polled.events),
            [("ankh#1".into(), "fake"), ("guild#1".into(), "other")]
        );
    }

    #[test]
    fn two_adapters_each_close_only_their_own_issue() {
        let w = two_sources();
        for (source, name, closed) in [
            ("ankh", "ankh/morpork", [true, false]),
            ("guild", "GLD", [false, true]),
        ] {
            let mut part = two();
            let close = Effect::Close {
                issue: IssueId::new(source, "1"),
            };
            assert_eq!(part.carry_out(&close, &w), Outcome::done());
            assert_eq!(closed_in(&mut part, name), closed, "{source}");
            for other in NAMES.into_iter().filter(|n| *n != name) {
                assert_eq!(closed_in(&mut part, other), [false, false], "{other}");
            }
        }
    }

    #[test]
    fn two_adapters_a_tracker_the_build_lacks_is_a_problem_and_the_others_still_sync() {
        let mut w = two_sources();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("klatch"),
            tracker: "nonesuch".into(),
            name: "desert/klatch".into(),
            code_host: None,
            repo: None,
        });
        let mut part = two();
        let polled = part.poll(&w, 0);
        assert_eq!(
            polled.problem.as_deref(),
            Some("klatch: no tracker \"nonesuch\" in this build")
        );
        assert_eq!(
            synced(&polled.events),
            [("ankh#1".into(), "fake"), ("guild#1".into(), "other")]
        );
        let close = Effect::Close {
            issue: IssueId::new("klatch", "1"),
        };
        assert_eq!(
            part.carry_out(&close, &w),
            Outcome::Failed("no tracker \"nonesuch\" in this build".into())
        );
        assert_eq!(closed_in(&mut part, "desert/klatch"), [false, false]);
    }

    #[test]
    fn the_fake_passes_the_contract() {
        assert_eq!(contract::check(&mut fake(), &Writable), Ok(()));
    }

    #[test]
    fn a_dependency_on_another_project_is_found_by_its_id_or_its_trackers_name() {
        let mut w = configured();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("klatch"),
            tracker: "fake".into(),
            name: "desert/klatch".into(),
            code_host: None,
            repo: None,
        });
        let mut tracker = fake();
        tracker.issues[0].1.depends_on =
            depends_on("Depends on klatch#3 and desert/klatch#4, and nowhere#5.");
        let events = changes(&w, &ProjectId::new("ankh"), &[tracker.issues[0].1.clone()]);
        let Some(Event::IssueSynced { depends_on, .. }) = events.first() else {
            panic!("{events:?}");
        };
        assert_eq!(
            depends_on,
            &[
                IssueId::new("klatch", "3"),
                IssueId::new("klatch", "4"),
                IssueId::new("ankh", "nowhere#5")
            ],
            "an unknown name stays unknown, so the issue waits"
        );
    }

    #[test]
    fn reads_dependencies_from_depends_on_lines() {
        let body = "Change the codes.\n\nDepends on #491.\n- depends on #12, #13 and ENG-41\nSee #77 for context.";
        assert_eq!(depends_on(body), ["491", "12", "13", "ENG-41"]);
        assert_eq!(
            depends_on("Depends on ankh/morpork.watch#4, klatch#5.#6 and e.g.#7."),
            ["ankh/morpork.watch#4", "klatch#5", "6", "7"]
        );
        assert_eq!(
            depends_on("No dependencies here, see #5."),
            Vec::<String>::new()
        );
    }
}
