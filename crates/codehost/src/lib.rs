//! Code hosts: where an issue's pull request lives, what its CI says, and where
//! it is merged. A project's tracker and its code host can differ: a Jira
//! project tracks issues and hosts no code.
//!
//! This crate holds the [`CodeHost`] contract, a [`FakeCodeHost`] and the
//! [`contract`] every code-host adapter runs. Adapter crates depend on this crate
//! only.

use std::collections::BTreeMap;
use std::fmt;
use std::time::{Duration, Instant};

use temet_work_core::ids::{IssueId, ProjectId};
use temet_work_core::model::{Addresses, Issue, Stage};
use temet_work_core::{Effect, Event, Outcome, Part, Polled, World};

/// What a pull request's CI says about its latest commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Checks {
    /// Some are still running.
    Pending,
    /// Every one passed, or was skipped.
    Passed,
    /// These failed.
    Failed {
        /// Their names.
        failing: Vec<String>,
    },
    /// The repository runs none.
    None,
}

/// Whether a pull request is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PullState {
    /// Open.
    Open,
    /// Merged.
    Merged,
    /// Closed without merging.
    Closed,
}

/// A pull request as its code host reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pull {
    /// Its number.
    pub number: String,
    /// Whether it is open.
    pub state: PullState,
    /// Its CI.
    pub checks: Checks,
    /// Whether it conflicts with the branch it merges into.
    pub conflicting: bool,
    /// Whether it is a draft, which its code host will not merge until its author
    /// marks it ready for review.
    pub draft: bool,
    /// The branch it merges from; empty when the code host gave none.
    pub branch: String,
}

/// Why a code-host call failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodeHostError {
    /// No such repository or pull request.
    NotFound(String),
    /// Signed out, or not allowed.
    Denied(String),
    /// Anything else, as the code host reported it.
    Other(String),
}

impl fmt::Display for CodeHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(what) => write!(f, "not found: {what}"),
            Self::Denied(what) => write!(f, "not allowed: {what}"),
            Self::Other(e) => f.write_str(e),
        }
    }
}

impl std::error::Error for CodeHostError {}

/// One code host, added by an adapter crate. `repo` is the code host's name for
/// the repository (`owner/repo`).
pub trait CodeHost: Send {
    /// A short, stable id, the adapter's feature name after `codehost-`.
    fn id(&self) -> &'static str;

    /// The pull request `number`.
    fn pull(&mut self, repo: &str, number: &str) -> Result<Pull, CodeHostError>;

    /// Merges the pull request `number` the way `how` says: `squash`, `merge` or
    /// `rebase`.
    fn merge(&mut self, repo: &str, number: &str, how: &str) -> Result<(), CodeHostError>;

    /// The CI of the repository's default branch at the commit the merged pull
    /// request `number` made; `None` when there is no run for it. An error when
    /// that pull request has not merged, or when its run waits for a person, who
    /// reads the error.
    fn main_pipeline(&mut self, repo: &str, number: &str) -> Result<Checks, CodeHostError>;

    /// The web address of the repository's code, made from its name and the
    /// adapter's own settings, such as the host it reaches; none when the
    /// adapter cannot say. It asks the code host nothing.
    fn address(&self, repo: &str) -> Option<String>;
}

/// A code host in memory, for tests.
pub struct FakeCodeHost {
    /// Its id.
    pub id: &'static str,
    /// Pull requests, as (repo, pull).
    pub pulls: Vec<(String, Pull)>,
    /// How each merge was asked for (`squash`, `merge` or `rebase`), in order.
    pub merges: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    /// What each repository's default-branch CI says after a merge; a repository
    /// not named here passes. Shared, so a test can change it while the part runs.
    pub mains: std::sync::Arc<std::sync::Mutex<BTreeMap<String, Checks>>>,
    /// The repository of each merge, in order.
    pub merged_repos: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    /// The pull request number of each merge, in order.
    pub merged_numbers: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl Default for FakeCodeHost {
    fn default() -> Self {
        Self::named("fake")
    }
}

impl FakeCodeHost {
    /// A fake code host with the given id.
    pub fn named(id: &'static str) -> Self {
        Self {
            id,
            pulls: Vec::new(),
            merges: Default::default(),
            mains: Default::default(),
            merged_repos: Default::default(),
            merged_numbers: Default::default(),
        }
    }
}

impl CodeHost for FakeCodeHost {
    fn id(&self) -> &'static str {
        self.id
    }

    fn pull(&mut self, repo: &str, number: &str) -> Result<Pull, CodeHostError> {
        self.pulls
            .iter()
            .find(|(r, p)| r == repo && p.number == number)
            .map(|(_, p)| p.clone())
            .ok_or_else(|| CodeHostError::NotFound(format!("{repo} pull request {number}")))
    }

    fn merge(&mut self, repo: &str, number: &str, how: &str) -> Result<(), CodeHostError> {
        let (_, pull) = self
            .pulls
            .iter_mut()
            .find(|(r, p)| r == repo && p.number == number)
            .ok_or_else(|| CodeHostError::NotFound(format!("{repo} pull request {number}")))?;
        if pull.state != PullState::Open {
            return Err(CodeHostError::Other(format!(
                "pull request {number} is not open"
            )));
        }
        pull.state = PullState::Merged;
        if let Ok(mut merges) = self.merges.lock() {
            merges.push(how.to_string());
        }
        if let Ok(mut repos) = self.merged_repos.lock() {
            repos.push(repo.to_string());
        }
        if let Ok(mut numbers) = self.merged_numbers.lock() {
            numbers.push(number.to_string());
        }
        Ok(())
    }

    fn main_pipeline(&mut self, repo: &str, number: &str) -> Result<Checks, CodeHostError> {
        let pull = self.pull(repo, number)?;
        if pull.state != PullState::Merged {
            return Err(CodeHostError::Other(format!(
                "pull request {number} has not merged"
            )));
        }
        Ok(self
            .mains
            .lock()
            .ok()
            .and_then(|m| m.get(repo).cloned())
            .unwrap_or(Checks::Passed))
    }

    fn address(&self, repo: &str) -> Option<String> {
        Some(format!("https://{}.example.com/{repo}", self.id))
    }
}

/// The contract every code-host adapter runs.
pub mod contract {
    use super::{CodeHost, PullState};

    /// What a code host's tests supply.
    pub trait Harness {
        /// A repository with at least one pull request.
        fn repo(&self) -> String;

        /// The number of a pull request in it.
        fn pull(&self) -> String;

        /// An open pull request the contract may merge, when the harness allows
        /// writes. A live code host's harness answers `None`: the contract never
        /// changes a real repository.
        fn mergeable(&self) -> Option<String>;
    }

    fn ensure(ok: bool, what: &str) -> Result<(), String> {
        if ok { Ok(()) } else { Err(what.to_string()) }
    }

    /// Runs the contract. Returns the first step that failed, in words.
    pub fn check(host: &mut dyn CodeHost, harness: &dyn Harness) -> Result<(), String> {
        let repo = harness.repo();
        let number = harness.pull();
        ensure(
            host.address(&repo)
                .is_some_and(|a| a.starts_with("https://") && a.contains(&repo)),
            "the repository's code has an https address that holds its name",
        )?;
        let pull = host
            .pull(&repo, &number)
            .map_err(|e| format!("pull: {e}"))?;
        ensure(
            pull.number == number,
            "a pull request reports its own number",
        )?;
        ensure(!pull.branch.is_empty(), "a pull request reports its branch")?;
        ensure(
            host.pull(&repo, "999999999").is_err(),
            "a pull request the code host does not have is an error",
        )?;
        ensure(
            host.pull("no-such/repository", &number).is_err(),
            "an unknown repository is an error",
        )?;
        ensure(
            host.main_pipeline(&repo, "999999999").is_err(),
            "the main pipeline of a pull request the code host does not have is an error",
        )?;
        if pull.state == PullState::Merged {
            host.main_pipeline(&repo, &number)
                .map_err(|e| format!("main pipeline of a merged pull request: {e}"))?;
        } else {
            ensure(
                host.main_pipeline(&repo, &number).is_err(),
                "the main pipeline of a pull request that has not merged is an error",
            )?;
        }
        if let Some(n) = harness.mergeable() {
            host.merge(&repo, &n, "squash")
                .map_err(|e| format!("merge: {e}"))?;
            let after = host
                .pull(&repo, &n)
                .map_err(|e| format!("pull after merge: {e}"))?;
            ensure(
                after.state == PullState::Merged,
                "a merged pull request reads as merged",
            )?;
            host.main_pipeline(&repo, &n)
                .map_err(|e| format!("main pipeline after merge: {e}"))?;
            ensure(
                host.merge(&repo, &n, "squash").is_err(),
                "merging twice is an error",
            )?;
        }
        Ok(())
    }
}

/// How long after a failed merge it is tried again.
pub const MERGE_RETRY_AFTER: Duration = Duration::from_secs(60);

/// The code-host part: the code hosts compiled into this build, in the order
/// they were added.
///
/// It reads the CI of each pull request under review, as often as its `ci_every`
/// setting says (30 seconds by default), and reports when it has finished. An issue
/// that changes several repositories passes only when CI passed on every pull
/// request. A pull request that still shows no run at a later read, while its
/// repository runs CI, is a problem until it has one. One a person closed without
/// merging is reported closed at the read that finds it, in review or at the
/// merge, and its CI no longer counts. It merges an issue's pull requests when
/// the loop asks, one repository at a time in the plan's order, each after the
/// previous one's main pipeline passed.
#[derive(Default)]
pub struct CodeHosts {
    adapters: Vec<Box<dyn CodeHost>>,
    /// Overrides the `ci_every` setting, for tests.
    every_ms: Option<u64>,
    read_at: BTreeMap<IssueId, u64>,
    errors: BTreeMap<IssueId, String>,
    /// The pull requests under review that showed no run when last read, though
    /// their repository runs CI, by issue, repository and pull request: true once
    /// the read before had found none either.
    no_run: BTreeMap<(IssueId, ProjectId, String), bool>,
    merge_failed: BTreeMap<IssueId, (String, Instant)>,
    /// When the main pipeline after a merge was last read, and what it said,
    /// by issue, repository and pull request.
    main_read: BTreeMap<(IssueId, ProjectId, String), (Instant, Wait)>,
}

/// Why the next repository does not merge yet.
enum Wait {
    /// The main pipeline is running.
    Running,
    /// No run for the merge: `again` once a later read still found none.
    NoRun { again: bool },
    /// Something a person may have to look at.
    Stuck(String),
}

impl Wait {
    fn outcome(&self, issue: &IssueId, repo: &ProjectId) -> Outcome {
        match self {
            Self::Running | Self::NoRun { again: false } => Outcome::done(),
            Self::NoRun { again: true } => Outcome::Failed(format!(
                "{issue} waits to merge after {repo}: still no CI run on the main branch of \
                 {repo} for its merge; set ci to false for {repo} if it runs none there"
            )),
            Self::Stuck(why) => {
                Outcome::Failed(format!("{issue} waits to merge after {repo}: {why}"))
            }
        }
    }
}

/// What a try to merge one pull request found.
enum Tried {
    /// It merged, now or before.
    Merged,
    /// A person closed it without merging.
    Closed,
    /// The code host would not merge it, for this reason.
    Refused(String),
}

/// The code host would not let it merge: its author has to hear why.
fn refused(issue: &IssueId, reason: String) -> Outcome {
    Outcome::Done {
        events: vec![Event::MergeRefused {
            issue: issue.clone(),
            reason,
        }],
        effects: Vec::new(),
    }
}

impl CodeHosts {
    /// Adds a code host. The root package calls this once per compiled-in adapter.
    pub fn add(&mut self, adapter: Box<dyn CodeHost>) {
        self.adapters.push(adapter);
    }

    /// The ids of the code hosts in this build.
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }

    /// Reads CI every `ms` whatever the `ci_every` setting says.
    pub fn every(mut self, ms: u64) -> Self {
        self.every_ms = Some(ms);
        self
    }

    /// The adapter and the code host's name for the repository of project `repo`.
    fn host_for(
        &mut self,
        world: &World,
        repo: &ProjectId,
    ) -> Result<(&mut Box<dyn CodeHost>, String), String> {
        let project = world
            .projects
            .get(repo)
            .ok_or_else(|| format!("{repo} has no repository configured"))?;
        let host = self
            .adapters
            .iter_mut()
            .find(|a| a.id() == project.code_host)
            .ok_or_else(|| format!("no code host \"{}\" in this build", project.code_host))?;
        Ok((host, project.repo.clone()))
    }

    /// Reads every unmerged pull request of an issue, and says what to report:
    /// each branch the world does not hold yet; then a conflict in the first one
    /// that has it, and each one a person closed without merging; else, while CI
    /// has not been reported, a failure in any at once, with the repository named
    /// when the issue has several, or a pass once every one has passed. A branch
    /// is compared with the world's, so a pull request read every `ci_every`
    /// reports its branch once, and again only when it changes.
    fn read_pulls(&mut self, world: &World, issue: &Issue) -> Result<Vec<Event>, String> {
        let several = issue.pulls.len() > 1;
        let (mut failed, mut pending) = (Vec::new(), false);
        let mut events = Vec::new();
        let (mut conflict, mut closed) = (None, Vec::new());
        for repo in world.repos_of(issue) {
            let Some(number) = issue
                .pulls
                .get(&repo)
                .filter(|_| !issue.merged.contains(&repo))
            else {
                continue;
            };
            let (host, name) = self.host_for(world, &repo)?;
            let pull = host.pull(&name, number).map_err(|e| e.to_string())?;
            if !pull.branch.is_empty() && issue.branches.get(&repo) != Some(&pull.branch) {
                events.push(Event::PullBranch {
                    issue: issue.id.clone(),
                    repo: repo.clone(),
                    number: number.clone(),
                    branch: pull.branch.clone(),
                });
            }
            let key = (issue.id.clone(), repo.clone(), number.clone());
            // A person closed it: no CI runs on it any more, and whatever its last
            // run said no longer counts. Its author has to replace it.
            if pull.state == PullState::Closed {
                self.no_run.remove(&key);
                closed.push(Event::PullClosed {
                    issue: issue.id.clone(),
                    repo,
                    number: number.clone(),
                });
                continue;
            }
            // After a conflict only closed ones are looked for, so the author
            // hears of every pull request to replace with it.
            if conflict.is_some() {
                continue;
            }
            // A conflict is caught on every read, even after CI finished.
            if pull.state == PullState::Open && pull.conflicting {
                conflict = Some(Event::PullConflicts {
                    issue: issue.id.clone(),
                    repo: Some(repo),
                });
                continue;
            }
            // No checks yet usually means CI has not started on the new commit; only
            // a repository that runs no CI leaves the plan's checks to the reviewer.
            let awaited = pull.checks == Checks::None && world.repo_flag(&issue.id, &repo, "ci");
            if awaited {
                // A second read with none may mean no run is coming.
                self.no_run
                    .entry(key)
                    .and_modify(|again| *again = true)
                    .or_insert(false);
            } else {
                self.no_run.remove(&key);
            }
            match pull.checks {
                Checks::Pending => pending = true,
                Checks::None if awaited => pending = true,
                Checks::Passed | Checks::None => {}
                Checks::Failed { failing } => failed.extend(
                    failing
                        .into_iter()
                        .map(|c| if several { format!("{c} ({repo})") } else { c }),
                ),
            }
        }
        // The conflict first: the loop sends the issue back for it, and then adds
        // each closed one to the same fixes. Neither leaves a CI result to report,
        // since a pass would count for a set with a pull request gone.
        if conflict.is_some() || !closed.is_empty() {
            events.extend(conflict);
            events.extend(closed);
            return Ok(events);
        }
        let passed = failed.is_empty();
        events.extend(
            (issue.checks_ok.is_none() && (!passed || !pending)).then(|| Event::ChecksFinished {
                issue: issue.id.clone(),
                passed,
                failing: failed,
            }),
        );
        Ok(events)
    }

    /// Whether the main pipeline of `repo`, after its pull request `number` merged,
    /// lets the next repository merge: `None` when it does, else what to do instead.
    /// It is read at most once each `ci_every`; in between the last answer stands.
    fn wait_for_main(
        &mut self,
        world: &World,
        issue: &IssueId,
        repo: &ProjectId,
        number: &str,
    ) -> Option<Outcome> {
        let every = self
            .every_ms
            .unwrap_or_else(|| world.setting_number(issue, None, "ci_every") * 1000);
        let key = (issue.clone(), repo.clone(), number.to_string());
        if let Some((at, last)) = self.main_read.get(&key)
            && at.elapsed() < Duration::from_millis(every)
        {
            return Some(last.outcome(issue, repo));
        }
        let read = self
            .host_for(world, repo)
            .and_then(|(host, name)| host.main_pipeline(&name, number).map_err(|e| e.to_string()));
        let wait = match read {
            Ok(Checks::Passed) => {
                self.main_read.remove(&key);
                return None;
            }
            Ok(Checks::Pending) => Wait::Running,
            // Read just after a merge, the run may not exist yet.
            Ok(Checks::None) => Wait::NoRun {
                again: matches!(self.main_read.get(&key), Some((_, Wait::NoRun { .. }))),
            },
            Ok(Checks::Failed { failing }) => {
                self.main_read.remove(&key);
                return Some(refused(
                    issue,
                    format!(
                        "the main pipeline of {repo} failed ({}); run it again, or fix it with \
                         a new pull request in {repo} and name that with ready",
                        failing.join(", ")
                    ),
                ));
            }
            Err(e) => Wait::Stuck(e),
        };
        let outcome = wait.outcome(issue, repo);
        self.main_read.insert(key, (Instant::now(), wait));
        Some(outcome)
    }

    /// Merges the next unmerged pull request of the issue, in the plan's order,
    /// once the one before it has a passing main pipeline.
    fn merge_next(&mut self, world: &World, issue: &IssueId) -> Outcome {
        let Some(i) = world.issues.get(issue) else {
            return Outcome::Failed(format!("{issue} is not known"));
        };
        // An issue from an older log may name repositories it has no pull request
        // in; only those it has one in merge.
        let repos: Vec<ProjectId> = world
            .repos_of(i)
            .into_iter()
            .filter(|r| i.pulls.contains_key(r))
            .collect();
        if repos.is_empty() {
            return Outcome::Failed(format!("{issue} has no pull request on record"));
        }
        let Some(at) = repos.iter().position(|r| !i.merged.contains(r)) else {
            // Every pull request merged; the loop only needs to hear it.
            return Outcome::Done {
                events: vec![Event::Merged {
                    issue: issue.clone(),
                    repo: None,
                }],
                effects: Vec::new(),
            };
        };
        let repo = repos[at].clone();
        let Some(number) = i.pulls.get(&repo).cloned() else {
            return Outcome::Failed(format!("{issue} has no pull request on record for {repo}"));
        };
        if let Some((why, at)) = self.merge_failed.get(issue)
            && at.elapsed() < MERGE_RETRY_AFTER
        {
            return Outcome::Failed(format!("{why}; tried again a minute after it failed"));
        }
        // The repository before this one must be passing on its main branch first.
        if let Some(before) = at.checked_sub(1).map(|b| &repos[b])
            && world.repo_flag(issue, before, "ci")
            && let Some(prev) = i.pulls.get(before)
            && let Some(wait) = self.wait_for_main(world, issue, before, prev)
        {
            return wait;
        }
        let how = world
            .setting_text(issue, None, "merge")
            .unwrap_or_else(|| "squash".into());
        let (host, name) = match self.host_for(world, &repo) {
            Ok(found) => found,
            Err(e) => return Outcome::Failed(e),
        };
        let tried = host
            .pull(&name, &number)
            .map_err(|e| e.to_string())
            .and_then(|pull| {
                match pull.state {
                    // Merged already, by a person or an earlier try.
                    PullState::Merged => Ok(Tried::Merged),
                    // A person closed it: no merge is asked for, and the loop drops it.
                    PullState::Closed => Ok(Tried::Closed),
                    PullState::Open if pull.conflicting => Ok(Tried::Refused(format!(
                        "pull request {number} conflicts with the branch it merges into"
                    ))),
                    // Asking to merge it would only fail, every time. The command is
                    // `gh`'s: no other adapter reports drafts yet.
                    PullState::Open if pull.draft => Ok(Tried::Refused(format!(
                        "pull request {number} is a draft: mark it ready for review with \
                         `gh pr ready {number} -R {name}`"
                    ))),
                    PullState::Open => match host.merge(&name, &number, &how) {
                        Ok(()) => Ok(Tried::Merged),
                        Err(e)
                            if ["not mergeable", "merge conflict", "conflicts"]
                                .iter()
                                .any(|w| e.to_string().to_lowercase().contains(w)) =>
                        {
                            Ok(Tried::Refused(format!(
                                "pull request {number} is not mergeable: {e}"
                            )))
                        }
                        Err(e) => Err(e.to_string()),
                    },
                }
            });
        match tried {
            Ok(Tried::Merged) => {
                self.merge_failed.remove(issue);
                Outcome::Done {
                    events: vec![Event::Merged {
                        issue: issue.clone(),
                        repo: Some(repo),
                    }],
                    effects: Vec::new(),
                }
            }
            // Its author reopens it or opens another, and names that with ready.
            Ok(Tried::Closed) => {
                self.merge_failed.remove(issue);
                Outcome::Done {
                    events: vec![Event::PullClosed {
                        issue: issue.clone(),
                        repo,
                        number,
                    }],
                    effects: Vec::new(),
                }
            }
            // Only its author can fix it: back to work.
            Ok(Tried::Refused(reason)) => refused(issue, reason),
            Err(e) => {
                self.merge_failed
                    .insert(issue.clone(), (e.clone(), Instant::now()));
                Outcome::Failed(e)
            }
        }
    }
}

impl Part for CodeHosts {
    fn id(&self) -> &'static str {
        "codehost"
    }

    fn poll(&mut self, world: &World, now_ms: u64) -> Polled {
        let mut events = Vec::new();
        let waiting: Vec<Issue> = world
            .issues
            .values()
            .filter(|i| {
                matches!(i.stage, Stage::InReview { .. } | Stage::Arbitration { .. })
                    && !i.pulls.is_empty()
            })
            .cloned()
            .collect();
        let still_waiting = |id: &IssueId| waiting.iter().any(|w| &w.id == id);
        self.errors.retain(|id, _| still_waiting(id));
        self.no_run.retain(|(id, ..), _| still_waiting(id));
        for i in waiting {
            let issue = i.id.clone();
            let every = self
                .every_ms
                .unwrap_or_else(|| world.setting_number(&issue, None, "ci_every") * 1000);
            let due = self
                .read_at
                .get(&issue)
                .is_none_or(|at| now_ms >= at + every);
            if !due {
                continue;
            }
            self.read_at.insert(issue.clone(), now_ms);
            match self.read_pulls(world, &i) {
                Ok(found) => {
                    self.errors.remove(&issue);
                    events.extend(found);
                }
                Err(e) => {
                    self.errors.insert(issue.clone(), format!("{issue}: {e}"));
                }
            }
        }
        // Made from what the last reads found, so it stays listed between reads.
        let no_run = self.no_run.iter().filter(|(_, again)| **again);
        let problems: Vec<String> = self
            .errors
            .values()
            .cloned()
            .chain(no_run.map(|((issue, repo, number), _)| {
                format!(
                    "{issue}: still no CI run on pull request {number} in {repo}; set ci to \
                     false for {repo} if it runs none there"
                )
            }))
            .collect();
        let problem = (!problems.is_empty()).then(|| problems.join("; "));
        Polled { events, problem }
    }

    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let Effect::Merge { issue } = effect else {
            return Outcome::NotMine;
        };
        self.merge_next(world, issue)
    }

    /// Where each project's code is, from the code host it names.
    fn addresses(&self, world: &World) -> BTreeMap<ProjectId, Addresses> {
        world
            .projects
            .values()
            .filter_map(|p| {
                let host = self.adapters.iter().find(|a| a.id() == p.code_host)?;
                let code = host.address(&p.repo)?;
                Some((
                    p.id.clone(),
                    Addresses {
                        issues: None,
                        code: Some(code),
                    },
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temet_work_core::ids::ProjectId;
    use temet_work_core::settings::{Scope, Value};

    fn pull(number: &str, checks: Checks) -> Pull {
        Pull {
            number: number.into(),
            state: PullState::Open,
            checks,
            conflicting: false,
            draft: false,
            // None, so that only a test about branches sees them reported.
            branch: String::new(),
        }
    }

    fn fake() -> FakeCodeHost {
        FakeCodeHost {
            pulls: vec![
                ("city/ankh".into(), pull("57", Checks::Passed)),
                ("city/ankh".into(), pull("58", Checks::Pending)),
                (
                    "city/ankh".into(),
                    pull(
                        "59",
                        Checks::Failed {
                            failing: vec!["rust".into()],
                        },
                    ),
                ),
            ],
            ..Default::default()
        }
    }

    struct Writable;

    impl contract::Harness for Writable {
        fn repo(&self) -> String {
            "city/ankh".into()
        }
        fn pull(&self) -> String {
            "58".into()
        }
        fn mergeable(&self) -> Option<String> {
            Some("57".into())
        }
    }

    /// A world where ankh#1, #2 and #3 are in review with pull requests 57, 58 and 59.
    fn in_review() -> World {
        let mut w = World::default();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("ankh"),
            tracker: "fake".into(),
            name: "city/ankh".into(),
            code_host: None,
            repo: None,
        });
        for (key, number) in [("1", "57"), ("2", "58"), ("3", "59")] {
            let issue = IssueId::new("ankh", key);
            w.apply(&Event::IssueSynced {
                issue: issue.clone(),
                title: format!("issue {key}"),
                milestone: None,
                depends_on: vec![],
                order: 1,
                closed: false,
            });
            if let Some(i) = w.issues.get_mut(&issue) {
                i.stage = Stage::InReview { round: 1 };
                i.pulls.insert(ProjectId::new("ankh"), number.into());
            }
        }
        w
    }

    #[test]
    fn the_fake_passes_the_contract() {
        let mut host = fake();
        for (_, p) in &mut host.pulls {
            p.branch = format!("ankh-{}", p.number);
        }
        assert_eq!(contract::check(&mut host, &Writable), Ok(()));
        // A code host that reports no branch fails it.
        assert_eq!(
            contract::check(&mut fake(), &Writable),
            Err("a pull request reports its branch".into())
        );
    }

    #[test]
    fn reports_finished_ci_once_per_read_and_waits_on_running_ci() {
        let w = in_review();
        let mut part = CodeHosts::default();
        part.add(Box::new(fake()));
        let polled = part.poll(&w, 0);
        assert_eq!(polled.problem, None);
        assert_eq!(
            polled.events,
            vec![
                Event::ChecksFinished {
                    issue: IssueId::new("ankh", "1"),
                    passed: true,
                    failing: vec![],
                },
                Event::ChecksFinished {
                    issue: IssueId::new("ankh", "3"),
                    passed: false,
                    failing: vec!["rust".into()],
                },
            ]
        );
        assert_eq!(part.poll(&w, 30_000 - 1).events, vec![], "not due yet");
    }

    #[test]
    fn a_conflict_is_reported_on_every_read_even_after_ci_finished() {
        let mut w = in_review();
        if let Some(i) = w.issues.get_mut(&IssueId::new("ankh", "1")) {
            i.checks_ok = Some(true);
        }
        let mut host = fake();
        host.pulls[0].1.conflicting = true;
        let mut part = CodeHosts::default();
        part.add(Box::new(host));
        let conflict = Event::PullConflicts {
            issue: IssueId::new("ankh", "1"),
            repo: Some(ProjectId::new("ankh")),
        };
        assert!(part.poll(&w, 0).events.contains(&conflict));
        let mut clean = CodeHosts::default();
        clean.add(Box::new(fake()));
        assert!(
            !clean
                .poll(&w, 0)
                .events
                .iter()
                .any(|e| matches!(e, Event::PullConflicts { .. }))
        );
    }

    #[test]
    fn merges_and_reports_it_and_a_merged_pull_request_counts() {
        let w = in_review();
        let mut part = CodeHosts::default();
        part.add(Box::new(fake()));
        let merge = Effect::Merge {
            issue: IssueId::new("ankh", "1"),
        };
        let merged = Outcome::Done {
            events: vec![Event::Merged {
                issue: IssueId::new("ankh", "1"),
                repo: Some(ProjectId::new("ankh")),
            }],
            effects: vec![],
        };
        assert_eq!(part.carry_out(&merge, &w), merged);
        assert_eq!(part.carry_out(&merge, &w), merged, "already merged");
    }

    #[test]
    fn a_conflicting_pull_request_goes_back_to_its_author() {
        let w = in_review();
        let mut host = fake();
        host.pulls[0].1.conflicting = true;
        let mut part = CodeHosts::default();
        part.add(Box::new(host));
        let merge = Effect::Merge {
            issue: IssueId::new("ankh", "1"),
        };
        assert!(matches!(
            part.carry_out(&merge, &w),
            Outcome::Done { events, .. } if matches!(events.as_slice(), [Event::MergeRefused { reason, .. }] if reason.contains("conflicts"))
        ));
    }

    #[test]
    fn a_draft_pull_request_goes_back_to_its_author_with_gh_pr_ready() {
        let mut w = in_review();
        let issue = IssueId::new("ankh", "1");
        if let Some(i) = w.issues.get_mut(&issue) {
            i.stage = Stage::Merging;
        }
        let mut host = fake();
        host.pulls[0].1.draft = true;
        let merges = host.merges.clone();
        let mut part = CodeHosts::default();
        part.add(Box::new(host));
        let merge = Effect::Merge {
            issue: issue.clone(),
        };
        // The second try comes at once: a draft is not held back like a failed merge.
        let refusals = [(); 2].map(|()| part.carry_out(&merge, &w));
        for outcome in &refusals {
            let Outcome::Done { events, .. } = outcome else {
                panic!("expected a refusal, got {outcome:?}");
            };
            assert!(
                matches!(events.as_slice(), [Event::MergeRefused { reason, .. }] if reason.contains("57") && reason.contains("gh pr ready")),
                "{events:?}"
            );
            for e in events {
                w.apply(e);
            }
        }
        assert!(merges.lock().expect("lock").is_empty(), "never asked");
        let i = &w.issues[&issue];
        assert!(matches!(i.stage, Stage::Fixing { .. }));
        assert!(
            i.fix[0].starts_with("the pull request cannot merge:")
                && i.fix[0].contains("gh pr ready"),
            "{:?}",
            i.fix
        );
    }

    fn watch_12() -> IssueId {
        IssueId::new("watch", "12")
    }

    /// A world where watch#12, whose code is ankh-morpork/watch, is in `stage`
    /// with pull request `number` on record.
    fn watch_12_in(stage: Stage, number: &str) -> World {
        let mut w = World::default();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("watch"),
            tracker: "fake".into(),
            name: "ankh-morpork/watch".into(),
            code_host: None,
            repo: None,
        });
        w.apply(&Event::IssueSynced {
            issue: watch_12(),
            title: "A night shift rota".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        let i = w.issues.get_mut(&watch_12()).expect("issue");
        i.stage = stage;
        i.round = 1;
        i.pulls.insert(ProjectId::new("watch"), number.into());
        w
    }

    /// Pull request `number` of ankh-morpork/watch, passing, in `state`, on
    /// branch `branch`.
    fn watch_pull(number: &str, state: PullState, branch: &str) -> (String, Pull) {
        let p = Pull {
            state,
            branch: branch.into(),
            ..pull(number, Checks::Passed)
        };
        ("ankh-morpork/watch".into(), p)
    }

    #[test]
    fn a_closed_pull_request_is_reported_closed_and_not_merged() {
        let w = watch_12_in(Stage::Merging, "57");
        let host = FakeCodeHost {
            pulls: vec![watch_pull("57", PullState::Closed, "watch-12-night-shift")],
            ..Default::default()
        };
        let merges = host.merges.clone();
        let mut part = CodeHosts::default();
        part.add(Box::new(host));
        let merge = Effect::Merge { issue: watch_12() };
        let closed = Outcome::Done {
            events: vec![Event::PullClosed {
                issue: watch_12(),
                repo: ProjectId::new("watch"),
                number: "57".into(),
            }],
            effects: vec![],
        };
        assert_eq!(part.carry_out(&merge, &w), closed);
        assert!(
            merges.lock().expect("lock").is_empty(),
            "no merge asked for"
        );
    }

    #[test]
    fn after_a_closed_pull_request_the_merge_follows_the_one_ready_named() {
        let mut w = watch_12_in(Stage::Merging, "57");
        let host = FakeCodeHost {
            pulls: vec![
                watch_pull("57", PullState::Closed, "watch-12-night-shift"),
                watch_pull("58", PullState::Open, "watch-12-rota"),
            ],
            ..Default::default()
        };
        let numbers = host.merged_numbers.clone();
        let mut part = CodeHosts::default().every(0);
        part.add(Box::new(host));
        let merge = Effect::Merge { issue: watch_12() };
        let apply = |w: &mut World, outcome: Outcome| -> Vec<Effect> {
            let Outcome::Done { events, .. } = outcome else {
                panic!("expected events, got {outcome:?}");
            };
            events.iter().flat_map(|e| w.apply(e)).collect()
        };
        let closed = part.carry_out(&merge, &w);
        apply(&mut w, closed);
        assert_eq!(w.issues[&watch_12()].stage, Stage::Fixing { round: 1 });

        // The author names 58; review approves and its checks pass.
        w.apply(&Event::PullRequestReady {
            issue: watch_12(),
            pull: None,
            pulls: [(ProjectId::new("watch"), "58".to_string())].into(),
            replies: Vec::new(),
        });
        let read = part.poll(&w, 0);
        assert!(
            read.events.contains(&Event::PullBranch {
                issue: watch_12(),
                repo: ProjectId::new("watch"),
                number: "58".into(),
                branch: "watch-12-rota".into(),
            }),
            "{:?}",
            read.events
        );
        let mut effects = Vec::new();
        for e in read.events.iter().chain([&Event::ReviewFinished {
            issue: watch_12(),
            round: 1,
            verdict: temet_work_core::Verdict::Approve,
            failed: None,
            open: None,
        }]) {
            effects.extend(w.apply(e));
        }
        assert_eq!(effects, vec![merge.clone()]);

        let merged = part.carry_out(&merge, &w);
        assert_eq!(
            merged,
            Outcome::Done {
                events: vec![Event::Merged {
                    issue: watch_12(),
                    repo: Some(ProjectId::new("watch")),
                }],
                effects: vec![],
            }
        );
        let effects = apply(&mut w, merged);
        assert!(effects.contains(&Effect::Deploy { issue: watch_12() }));
        assert_eq!(w.issues[&watch_12()].stage, Stage::Deploying);
        assert_eq!(*numbers.lock().expect("lock"), ["58"], "57 is never merged");
    }

    #[test]
    fn a_pull_requests_branch_is_reported_once() {
        let mut w = watch_12_in(Stage::InReview { round: 1 }, "57");
        let mut part = CodeHosts::default().every(0);
        // Pull request 57 on `branch`, with CI still running so only the branch shows.
        let on = |part: &mut CodeHosts, branch: &str| {
            let mut host = FakeCodeHost {
                pulls: vec![watch_pull("57", PullState::Open, branch)],
                ..Default::default()
            };
            host.pulls[0].1.checks = Checks::Pending;
            part.adapters = vec![Box::new(host)];
        };
        let reported = |branch: &str| {
            vec![Event::PullBranch {
                issue: watch_12(),
                repo: ProjectId::new("watch"),
                number: "57".into(),
                branch: branch.into(),
            }]
        };
        let held = |w: &World| w.issues[&watch_12()].branches[&ProjectId::new("watch")].clone();
        on(&mut part, "watch-12-night-shift");
        let first = part.poll(&w, 0).events;
        assert_eq!(first, reported("watch-12-night-shift"));
        for e in &first {
            w.apply(e);
        }
        assert_eq!(held(&w), "watch-12-night-shift");
        assert_eq!(part.poll(&w, 1).events, vec![], "the world holds it");

        // The branch changes: it is reported again, once.
        on(&mut part, "watch-12-night-shift-2");
        let changed = part.poll(&w, 2).events;
        assert_eq!(changed, reported("watch-12-night-shift-2"));
        for e in &changed {
            w.apply(e);
        }
        assert_eq!(held(&w), "watch-12-night-shift-2");
        assert_eq!(
            part.poll(&w, 3).events,
            vec![],
            "the world holds the new one"
        );
    }

    fn closed_57() -> Event {
        Event::PullClosed {
            issue: watch_12(),
            repo: ProjectId::new("watch"),
            number: "57".into(),
        }
    }

    #[test]
    fn a_pull_request_closed_in_review_is_reported_closed_and_its_failed_checks_are_not() {
        let mut w = watch_12_in(Stage::InReview { round: 1 }, "57");
        let (repo, mut p) = watch_pull("57", PullState::Closed, "watch-12-night-shift");
        p.checks = Checks::Failed {
            failing: vec!["rust".into()],
        };
        // What a code host may still say of a closed one: no conflict follows.
        p.conflicting = true;
        let mut part = CodeHosts::default().every(0);
        part.add(Box::new(FakeCodeHost {
            pulls: vec![(repo, p)],
            ..Default::default()
        }));
        let read = part.poll(&w, 0);
        assert_eq!(read.problem, None);
        assert_eq!(
            read.events,
            vec![
                Event::PullBranch {
                    issue: watch_12(),
                    repo: ProjectId::new("watch"),
                    number: "57".into(),
                    branch: "watch-12-night-shift".into(),
                },
                closed_57(),
            ]
        );
        for e in &read.events {
            w.apply(e);
        }
        let i = &w.issues[&watch_12()];
        assert_eq!(i.stage, Stage::Fixing { round: 1 });
        assert_eq!(
            i.closed[&ProjectId::new("watch")].branch.as_deref(),
            Some("watch-12-night-shift"),
            "the branch read first is kept with the closed one"
        );
    }

    #[test]
    fn a_pull_request_closed_before_any_ci_run_is_reported_closed_not_waited_on() {
        let w = watch_12_in(Stage::InReview { round: 1 }, "57");
        let host = |state| FakeCodeHost {
            pulls: vec![(
                "ankh-morpork/watch".into(),
                Pull {
                    state,
                    ..pull("57", Checks::None)
                },
            )],
            ..Default::default()
        };
        let mut part = CodeHosts::default().every(0);
        part.add(Box::new(host(PullState::Closed)));
        for now in [0, 1] {
            let read = part.poll(&w, now);
            assert_eq!(read.events, vec![closed_57()], "read {now}");
            assert_eq!(read.problem, None, "read {now}");
        }
        // Open, the same pull request is waited on, and a second read lists it.
        let mut open = CodeHosts::default().every(0);
        open.add(Box::new(host(PullState::Open)));
        assert_eq!(open.poll(&w, 0).events, vec![]);
        assert!(
            open.poll(&w, 1)
                .problem
                .is_some_and(|p| p.contains("still no CI run on pull request 57"))
        );
    }

    fn set(w: &mut World, scope: Scope, key: &str, value: Value) {
        w.apply(&Event::SettingSet {
            scope,
            role: None,
            key: key.into(),
            value,
            by: "person".into(),
        });
    }

    #[test]
    fn the_merge_setting_reaches_the_code_host_for_that_issue_only() {
        let mut w = in_review();
        set(
            &mut w,
            Scope::Task(IssueId::new("ankh", "1")),
            "merge",
            Value::Text("rebase".into()),
        );
        let host = fake();
        let merges = host.merges.clone();
        let mut part = CodeHosts::default();
        part.add(Box::new(host));
        for key in ["1", "2"] {
            let merge = Effect::Merge {
                issue: IssueId::new("ankh", key),
            };
            assert!(matches!(part.carry_out(&merge, &w), Outcome::Done { .. }));
        }
        assert_eq!(*merges.lock().expect("lock"), ["rebase", "squash"]);
    }

    #[test]
    fn ci_is_read_as_often_as_the_ci_every_setting_says_for_that_issue() {
        let mut w = in_review();
        set(
            &mut w,
            Scope::Task(IssueId::new("ankh", "1")),
            "ci_every",
            Value::Number(300),
        );
        let mut part = CodeHosts::default();
        part.add(Box::new(fake()));
        let finished = |events: &[Event], key: &str| {
            events
                .iter()
                .any(|e| matches!(e, Event::ChecksFinished { issue, .. } if issue.key == key))
        };
        assert!(finished(&part.poll(&w, 0).events, "1"));
        // ankh#3 is read again after the default 30 seconds; ankh#1 after 300.
        let at_31s = part.poll(&w, 31_000).events;
        assert!(finished(&at_31s, "3"));
        assert!(!finished(&at_31s, "1"));
        assert!(finished(&part.poll(&w, 300_000).events, "1"));
    }

    #[test]
    fn no_ci_reading_waits_unless_the_project_runs_no_ci() {
        let mut w = in_review();
        let mut host = fake();
        host.pulls[0].1.checks = Checks::None;
        let mut part = CodeHosts::default().every(0);
        part.add(Box::new(host));
        let first = |events: Vec<Event>| {
            events
                .into_iter()
                .find(|e| matches!(e, Event::ChecksFinished { issue, .. } if issue.key == "1"))
        };
        assert_eq!(
            first(part.poll(&w, 0).events),
            None,
            "CI may not have started"
        );
        set(
            &mut w,
            Scope::Project(ProjectId::new("ankh")),
            "ci",
            Value::Flag(false),
        );
        assert!(matches!(
            first(part.poll(&w, 1).events),
            Some(Event::ChecksFinished { passed: true, .. })
        ));
    }

    /// The problem line for ankh#`key`, whose pull request `number` has no run.
    fn no_run(key: &str, number: &str) -> String {
        format!(
            "ankh#{key}: still no CI run on pull request {number} in ankh; set ci to false for \
             ankh if it runs none there"
        )
    }

    /// A part whose every poll reads again, and where pull request 57 has no run.
    fn watching_57() -> CodeHosts {
        let mut part = CodeHosts::default().every(0);
        set_57(&mut part, Checks::None);
        part
    }

    /// Makes pull request 57 read `checks` from now on.
    fn set_57(part: &mut CodeHosts, checks: Checks) {
        let mut host = fake();
        host.pulls[0].1.checks = checks;
        part.adapters = vec![Box::new(host)];
    }

    /// Whether the checks of ankh#1 were reported, and as what.
    fn reported_for_1(events: &[Event]) -> Option<bool> {
        events.iter().find_map(|e| match e {
            Event::ChecksFinished { issue, passed, .. } if issue.key == "1" => Some(*passed),
            _ => None,
        })
    }

    #[test]
    fn a_pull_request_still_without_checks_at_a_later_read_is_a_problem_until_it_has_some() {
        let w = in_review();
        let mut part = watching_57();
        let mut read = |now, found: Checks| {
            set_57(&mut part, found);
            let polled = part.poll(&w, now);
            (polled.problem, reported_for_1(&polled.events))
        };
        let listed = (Some(no_run("1", "57")), None);
        assert_eq!(
            read(0, Checks::None),
            (None, None),
            "the run may not exist yet"
        );
        assert_eq!(read(1, Checks::None), listed);
        assert_eq!(read(2, Checks::Pending), (None, None));
        assert_eq!(read(3, Checks::None), (None, None), "a first read again");
        assert_eq!(read(4, Checks::None), listed);
        assert_eq!(read(5, Checks::Passed), (None, Some(true)));
    }

    #[test]
    fn a_pull_request_without_checks_stays_listed_between_reads() {
        let w = in_review();
        let mut host = fake();
        host.pulls[0].1.checks = Checks::None;
        let mut part = CodeHosts::default().every(60_000);
        part.add(Box::new(host));
        assert_eq!(part.poll(&w, 0).problem, None);
        let listed = Some(no_run("1", "57"));
        assert_eq!(part.poll(&w, 60_000).problem, listed);
        assert_eq!(part.poll(&w, 60_001).problem, listed, "not due");
    }

    #[test]
    fn setting_ci_to_false_clears_a_pull_request_without_checks() {
        let mut w = in_review();
        let mut part = watching_57();
        part.poll(&w, 0);
        assert_eq!(part.poll(&w, 1).problem, Some(no_run("1", "57")));
        set(
            &mut w,
            Scope::Project(ProjectId::new("ankh")),
            "ci",
            Value::Flag(false),
        );
        let polled = part.poll(&w, 2);
        assert_eq!(polled.problem, None);
        assert_eq!(reported_for_1(&polled.events), Some(true));
    }

    #[test]
    fn an_issue_that_leaves_review_drops_its_no_run_problem() {
        for left in [Stage::Fixing { round: 1 }, Stage::Merging] {
            let mut w = in_review();
            let mut part = watching_57();
            let stage = |w: &mut World, stage| {
                let issue = IssueId::new("ankh", "1");
                w.issues.get_mut(&issue).expect("issue").stage = stage;
            };
            part.poll(&w, 0);
            assert_eq!(part.poll(&w, 1).problem, Some(no_run("1", "57")));
            stage(&mut w, left);
            assert_eq!(part.poll(&w, 2).problem, None);
            stage(&mut w, Stage::InReview { round: 2 });
            assert_eq!(
                part.poll(&w, 3).problem,
                None,
                "the new commit's first read"
            );
            assert_eq!(part.poll(&w, 4).problem, Some(no_run("1", "57")));
        }
    }

    #[test]
    fn a_no_run_problem_and_a_read_error_are_listed_together() {
        let mut w = in_review();
        add_in_review(&mut w, "lancre", "hills/lancre", Some("nonesuch"));
        let mut part = watching_57();
        let error = "lancre#1: no code host \"nonesuch\" in this build";
        assert_eq!(part.poll(&w, 0).problem.as_deref(), Some(error));
        assert_eq!(
            part.poll(&w, 1).problem,
            Some(format!("{error}; {}", no_run("1", "57")))
        );
        set_57(&mut part, Checks::Pending);
        assert_eq!(part.poll(&w, 2).problem.as_deref(), Some(error));
        set_57(&mut part, Checks::None);
        part.poll(&w, 3);
        part.poll(&w, 4);
        w.issues.remove(&IssueId::new("lancre", "1"));
        assert_eq!(part.poll(&w, 5).problem, Some(no_run("1", "57")));
    }

    #[test]
    fn an_issue_without_a_pull_request_cannot_merge() {
        let mut w = in_review();
        if let Some(i) = w.issues.get_mut(&IssueId::new("ankh", "1")) {
            i.pulls.clear();
        }
        let mut part = CodeHosts::default();
        part.add(Box::new(fake()));
        let merge = Effect::Merge {
            issue: IssueId::new("ankh", "1"),
        };
        assert!(matches!(part.carry_out(&merge, &w), Outcome::Failed(_)));
    }

    /// Code hosts "fake" and "other", in that order. Each has pull request 1 in
    /// every one of `repos`, failing a check named after the host, so a read from
    /// the wrong one shows.
    fn two(repos: &[&str]) -> [FakeCodeHost; 2] {
        ["fake", "other"].map(|id| {
            let failing = Checks::Failed {
                failing: vec![id.into()],
            };
            FakeCodeHost {
                pulls: repos
                    .iter()
                    .map(|r| (r.to_string(), pull("1", failing.clone())))
                    .collect(),
                ..FakeCodeHost::named(id)
            }
        })
    }

    fn part_of(hosts: [FakeCodeHost; 2]) -> CodeHosts {
        let mut part = CodeHosts::default();
        for host in hosts {
            part.add(Box::new(host));
        }
        part
    }

    const REPOS: [&str; 3] = ["city/ankh", "desert/klatch", "hills/lancre"];

    /// ankh#1 in review on code host "fake", which its tracker names, and klatch#1
    /// on "other" with tracker "fake".
    fn on_two_hosts() -> World {
        let mut w = World::default();
        add_in_review(&mut w, "ankh", "city/ankh", None);
        add_in_review(&mut w, "klatch", "desert/klatch", Some("other"));
        w
    }

    #[test]
    fn two_adapters_each_projects_code_has_the_address_its_own_code_host_gives() {
        let mut w = on_two_hosts();
        add_in_review(&mut w, "lancre", "hills/lancre", Some("gitea"));
        let addresses = part_of(two(&REPOS)).addresses(&w);
        let code = |id: &str| {
            addresses
                .get(&ProjectId::new(id))
                .and_then(|a| a.code.clone())
        };
        assert_eq!(
            code("ankh").as_deref(),
            Some("https://fake.example.com/city/ankh")
        );
        assert_eq!(
            code("klatch").as_deref(),
            Some("https://other.example.com/desert/klatch")
        );
        // A code host this build lacks gives none.
        assert_eq!(code("lancre"), None);
        assert!(addresses.values().all(|a| a.issues.is_none()));
    }

    /// A project on tracker "fake" whose issue 1 is in review with pull request 1.
    fn add_in_review(w: &mut World, id: &str, name: &str, code_host: Option<&str>) {
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new(id),
            tracker: "fake".into(),
            name: name.into(),
            code_host: code_host.map(String::from),
            repo: None,
        });
        let issue = IssueId::new(id, "1");
        w.apply(&Event::IssueSynced {
            issue: issue.clone(),
            title: "t".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        let i = w.issues.get_mut(&issue).expect("issue");
        i.stage = Stage::InReview { round: 1 };
        i.pulls.insert(ProjectId::new(id), "1".into());
    }

    fn failed_on(project: &str, host: &str) -> Event {
        Event::ChecksFinished {
            issue: IssueId::new(project, "1"),
            passed: false,
            failing: vec![host.into()],
        }
    }

    #[test]
    fn two_adapters_each_project_has_its_ci_read_from_its_own() {
        let polled = part_of(two(&REPOS)).poll(&on_two_hosts(), 0);
        assert_eq!(polled.problem, None);
        assert_eq!(
            polled.events,
            vec![failed_on("ankh", "fake"), failed_on("klatch", "other")]
        );
    }

    #[test]
    fn two_adapters_each_merge_goes_through_the_projects_own_only() {
        let w = on_two_hosts();
        for (project, repo, own) in [("ankh", "city/ankh", 0), ("klatch", "desert/klatch", 1)] {
            let hosts = two(&REPOS);
            let merges = hosts.each_ref().map(|h| h.merges.clone());
            let repos = hosts[own].merged_repos.clone();
            let merge = Effect::Merge {
                issue: IssueId::new(project, "1"),
            };
            assert!(matches!(
                part_of(hosts).carry_out(&merge, &w),
                Outcome::Done { events, .. } if matches!(events.as_slice(), [Event::Merged { .. }])
            ));
            assert_eq!(*merges[own].lock().expect("lock"), ["squash"], "{project}");
            assert!(
                merges[1 - own].lock().expect("lock").is_empty(),
                "{project}"
            );
            assert_eq!(*repos.lock().expect("lock"), [repo]);
        }
    }

    #[test]
    fn two_adapters_a_code_host_the_build_lacks_is_a_problem_and_the_others_are_still_read() {
        let mut w = on_two_hosts();
        add_in_review(&mut w, "lancre", "hills/lancre", Some("nonesuch"));
        let hosts = two(&REPOS);
        let merges = hosts.each_ref().map(|h| h.merges.clone());
        let mut part = part_of(hosts);
        let polled = part.poll(&w, 0);
        assert_eq!(
            polled.problem.as_deref(),
            Some("lancre#1: no code host \"nonesuch\" in this build")
        );
        assert_eq!(
            polled.events,
            vec![failed_on("ankh", "fake"), failed_on("klatch", "other")]
        );
        let merge = Effect::Merge {
            issue: IssueId::new("lancre", "1"),
        };
        assert_eq!(
            part.carry_out(&merge, &w),
            Outcome::Failed("no code host \"nonesuch\" in this build".into())
        );
        for m in merges {
            assert!(m.lock().expect("lock").is_empty());
        }
    }

    #[test]
    fn keeps_adapters_in_the_order_added() {
        let mut part = CodeHosts::default();
        part.add(Box::new(FakeCodeHost::default()));
        assert_eq!(part.ids(), ["fake"]);
        assert_eq!(part.id(), "codehost");
    }

    // An issue that changes two repositories, in the order [ui, svc].
    mod two_repositories {
        use super::*;

        fn pid(s: &str) -> ProjectId {
            ProjectId::new(s)
        }

        fn gld() -> IssueId {
            IssueId::new("guild", "GLD-1")
        }

        fn world(stage: Stage) -> World {
            let mut w = World::default();
            w.apply(&Event::AreaDefined {
                area: pid("guild"),
                name: "Guild".into(),
                projects: vec![pid("ui"), pid("svc")],
                tracker: "fake".into(),
                tracker_name: "GLD".into(),
            });
            for p in ["ui", "svc"] {
                w.apply(&Event::ProjectAdded {
                    project: pid(p),
                    tracker: String::new(),
                    name: format!("group/{p}"),
                    code_host: Some("fake".into()),
                    repo: None,
                });
            }
            w.apply(&Event::IssueSynced {
                issue: gld(),
                title: "t".into(),
                milestone: None,
                depends_on: vec![],
                order: 1,
                closed: false,
            });
            let i = w.issues.get_mut(&gld()).expect("issue");
            i.plan = Some(temet_work_core::model::Plan {
                repos: vec![pid("ui"), pid("svc")],
                ..Default::default()
            });
            i.stage = stage;
            i.pulls = [(pid("ui"), "1".to_string()), (pid("svc"), "2".to_string())].into();
            w
        }

        fn host(ui: Checks, svc: Checks) -> FakeCodeHost {
            FakeCodeHost {
                pulls: vec![
                    ("group/ui".into(), pull("1", ui)),
                    ("group/svc".into(), pull("2", svc)),
                ],
                ..Default::default()
            }
        }

        fn part(host: FakeCodeHost) -> CodeHosts {
            let mut part = CodeHosts::default().every(0);
            part.add(Box::new(host));
            part
        }

        fn failed(name: &str) -> Checks {
            Checks::Failed {
                failing: vec![name.into()],
            }
        }

        #[test]
        fn ci_must_pass_on_every_pull_request_and_a_failure_names_its_repository() {
            let w = world(Stage::InReview { round: 1 });
            let polled = part(host(Checks::Passed, failed("rust"))).poll(&w, 0);
            assert_eq!(
                polled.events,
                vec![Event::ChecksFinished {
                    issue: gld(),
                    passed: false,
                    failing: vec!["rust (svc)".into()],
                }]
            );
            let polled = part(host(Checks::Passed, Checks::Passed)).poll(&w, 0);
            assert!(matches!(
                polled.events.as_slice(),
                [Event::ChecksFinished { passed: true, .. }]
            ));
            let polled = part(host(Checks::Passed, Checks::Pending)).poll(&w, 0);
            assert_eq!(polled.events, vec![], "svc is still running");
        }

        #[test]
        fn the_loop_sends_the_author_back_with_the_failing_repository_named() {
            let mut w = world(Stage::InReview { round: 1 });
            let polled = part(host(Checks::Passed, failed("rust"))).poll(&w, 0);
            for e in &polled.events {
                w.apply(e);
            }
            let i = &w.issues[&gld()];
            assert_eq!(i.checks_ok, Some(false));
            assert_eq!(i.fix, vec!["CI check rust (svc) failed".to_string()]);
        }

        fn merge() -> Effect {
            Effect::Merge { issue: gld() }
        }

        fn merged(repo: &str) -> Outcome {
            Outcome::Done {
                events: vec![Event::Merged {
                    issue: gld(),
                    repo: Some(pid(repo)),
                }],
                effects: vec![],
            }
        }

        #[test]
        fn svc_merges_only_after_uis_main_pipeline_passed_and_deploy_fires_once() {
            let mut w = world(Stage::Merging);
            let fake = host(Checks::Passed, Checks::Passed);
            let repos = fake.merged_repos.clone();
            let mains = fake.mains.clone();
            let mut part = part(fake);
            assert_eq!(part.carry_out(&merge(), &w), merged("ui"));
            w.apply(&Event::Merged {
                issue: gld(),
                repo: Some(pid("ui")),
            });
            // ui's main pipeline is running: nothing merges, nothing is reported.
            mains
                .lock()
                .expect("lock")
                .insert("group/ui".into(), Checks::Pending);
            assert_eq!(part.carry_out(&merge(), &w), Outcome::done());
            assert_eq!(*repos.lock().expect("lock"), ["group/ui"]);
            mains
                .lock()
                .expect("lock")
                .insert("group/ui".into(), Checks::Passed);
            assert_eq!(part.carry_out(&merge(), &w), merged("svc"));
            assert_eq!(*repos.lock().expect("lock"), ["group/ui", "group/svc"]);
            let effects = w.apply(&Event::Merged {
                issue: gld(),
                repo: Some(pid("svc")),
            });
            let deploys = effects
                .iter()
                .filter(|e| matches!(e, Effect::Deploy { .. }))
                .count();
            assert_eq!(deploys, 1);
        }

        #[test]
        fn a_failed_main_pipeline_merges_nothing_further_and_goes_back_to_the_author() {
            let mut w = world(Stage::Merging);
            let fake = host(Checks::Passed, Checks::Passed);
            let repos = fake.merged_repos.clone();
            fake.mains
                .lock()
                .expect("lock")
                .insert("group/ui".into(), failed("deploy-tests"));
            let mut part = part(fake);
            assert_eq!(part.carry_out(&merge(), &w), merged("ui"));
            w.apply(&Event::Merged {
                issue: gld(),
                repo: Some(pid("ui")),
            });
            let Outcome::Done { events, .. } = part.carry_out(&merge(), &w) else {
                panic!("expected the loop to hear it");
            };
            assert!(
                matches!(events.as_slice(), [Event::MergeRefused { reason, .. }] if reason.contains("main pipeline of ui failed (deploy-tests)")),
                "{events:?}"
            );
            for e in &events {
                w.apply(e);
            }
            assert_eq!(*repos.lock().expect("lock"), ["group/ui"], "svc not merged");
            let i = &w.issues[&gld()];
            assert!(matches!(i.stage, Stage::Fixing { .. }));
            assert!(
                i.fix[0].starts_with("ui is merged, svc cannot merge: "),
                "{:?}",
                i.fix
            );
        }

        #[test]
        fn without_ci_it_does_not_wait_for_the_main_pipeline() {
            let mut w = world(Stage::Merging);
            set(&mut w, Scope::Area(pid("guild")), "ci", Value::Flag(false));
            let fake = host(Checks::Passed, Checks::Passed);
            fake.mains
                .lock()
                .expect("lock")
                .insert("group/ui".into(), Checks::Pending);
            let mut part = part(fake);
            assert_eq!(part.carry_out(&merge(), &w), merged("ui"));
            w.apply(&Event::Merged {
                issue: gld(),
                repo: Some(pid("ui")),
            });
            assert_eq!(part.carry_out(&merge(), &w), merged("svc"));
        }

        #[test]
        fn two_adapters_each_pull_request_of_one_issue_is_read_and_merged_by_its_own() {
            let mut w = world(Stage::InReview { round: 1 });
            w.apply(&Event::ProjectAdded {
                project: pid("svc"),
                tracker: String::new(),
                name: "group/svc".into(),
                code_host: Some("other".into()),
                repo: None,
            });
            let read = part_of(two_with_pulls()).poll(&w, 0);
            assert_eq!(read.problem, None);
            assert_eq!(
                read.events,
                vec![Event::ChecksFinished {
                    issue: gld(),
                    passed: false,
                    failing: vec!["fake (ui)".into(), "other (svc)".into()],
                }]
            );

            w.issues.get_mut(&gld()).expect("issue").stage = Stage::Merging;
            let hosts = two_with_pulls();
            let repos = hosts.each_ref().map(|h| h.merged_repos.clone());
            // ui's main pipeline passes on its own host and fails on the other.
            hosts[1]
                .mains
                .lock()
                .expect("lock")
                .insert("group/ui".into(), failed("other"));
            let mut part = part_of(hosts);
            assert_eq!(part.carry_out(&merge(), &w), merged("ui"));
            w.apply(&Event::Merged {
                issue: gld(),
                repo: Some(pid("ui")),
            });
            assert_eq!(part.carry_out(&merge(), &w), merged("svc"));
            assert_eq!(*repos[0].lock().expect("lock"), ["group/ui"]);
            assert_eq!(*repos[1].lock().expect("lock"), ["group/svc"]);
        }

        /// The two code hosts, each with both of the issue's pull requests.
        fn two_with_pulls() -> [FakeCodeHost; 2] {
            let mut hosts = two(&[]);
            for host in &mut hosts {
                let failing = failed(host.id);
                host.pulls = vec![
                    ("group/ui".into(), pull("1", failing.clone())),
                    ("group/svc".into(), pull("2", failing)),
                ];
            }
            hosts
        }

        fn closed(repo: &str, number: &str) -> Event {
            Event::PullClosed {
                issue: gld(),
                repo: pid(repo),
                number: number.into(),
            }
        }

        #[test]
        fn a_closed_pull_request_in_either_repository_is_reported_and_no_ci_result_is() {
            use PullState::{Closed, Open};
            for (ui, svc, reported) in [
                (Closed, Open, vec![closed("ui", "1")]),
                (Open, Closed, vec![closed("svc", "2")]),
                (Closed, Closed, vec![closed("ui", "1"), closed("svc", "2")]),
            ] {
                let mut fake = host(Checks::Passed, Checks::Passed);
                fake.pulls[0].1.state = ui;
                fake.pulls[1].1.state = svc;
                let read = part(fake).poll(&world(Stage::InReview { round: 1 }), 0);
                assert_eq!(read.events, reported, "ui {ui:?}, svc {svc:?}");
            }
            // Both closed: the author is told of both, in the round it had.
            let mut w = world(Stage::InReview { round: 1 });
            for e in [closed("ui", "1"), closed("svc", "2")] {
                w.apply(&e);
            }
            let i = &w.issues[&gld()];
            assert_eq!(i.stage, Stage::Fixing { round: 1 });
            assert!(i.pulls.is_empty(), "{:?}", i.pulls);
            assert_eq!(
                i.fix,
                ["ui", "svc"].map(|r| format!(
                    "pull request {} in {r} was closed without merging: reopen it if its branch \
                     still exists, or open a new one there",
                    if r == "ui" { 1 } else { 2 }
                ))
            );
        }

        #[test]
        fn a_conflict_and_a_closed_pull_request_go_back_in_one_fix() {
            let mut w = world(Stage::InReview { round: 1 });
            let mut fake = host(Checks::Passed, Checks::Passed);
            fake.pulls[0].1.conflicting = true;
            fake.pulls[1].1.state = PullState::Closed;
            let read = part(fake).poll(&w, 0);
            assert_eq!(
                read.events,
                vec![
                    Event::PullConflicts {
                        issue: gld(),
                        repo: Some(pid("ui")),
                    },
                    closed("svc", "2"),
                ]
            );
            let effects: Vec<Effect> = read.events.iter().flat_map(|e| w.apply(e)).collect();
            let i = &w.issues[&gld()];
            assert!(
                matches!(i.fix.as_slice(), [conflict, gone]
                    if conflict.starts_with("the pull request in ui conflicts")
                        && gone.starts_with("pull request 2 in svc was closed")),
                "{:?}",
                i.fix
            );
            let last = effects.iter().rev().find_map(|e| match e {
                Effect::RequestFix { points, .. } => Some(points),
                _ => None,
            });
            assert_eq!(last, Some(&i.fix), "the last fix message holds both");
        }

        #[test]
        fn a_conflict_in_the_second_repository_names_it() {
            let w = world(Stage::InReview { round: 1 });
            let mut fake = host(Checks::Passed, Checks::Passed);
            fake.pulls[1].1.conflicting = true;
            let events = part(fake).poll(&w, 0).events;
            let conflict = Event::PullConflicts {
                issue: gld(),
                repo: Some(pid("svc")),
            };
            assert!(events.contains(&conflict), "{events:?}");
        }

        #[test]
        fn each_repository_says_whether_it_runs_ci() {
            let mut w = world(Stage::InReview { round: 1 });
            set(&mut w, Scope::Project(pid("svc")), "ci", Value::Flag(false));
            let polled = part(host(Checks::Passed, Checks::None)).poll(&w, 0);
            assert!(matches!(
                polled.events.as_slice(),
                [Event::ChecksFinished { passed: true, .. }]
            ));
        }

        #[test]
        fn only_a_repository_that_runs_ci_is_named_for_a_pull_request_without_checks() {
            let mut w = world(Stage::InReview { round: 1 });
            set(&mut w, Scope::Project(pid("svc")), "ci", Value::Flag(false));
            let mut part = part(host(Checks::None, Checks::None));
            assert_eq!(part.poll(&w, 0).problem, None);
            assert_eq!(
                part.poll(&w, 1).problem.as_deref(),
                Some(
                    "guild#GLD-1: still no CI run on pull request 1 in ui; set ci to false for ui \
                     if it runs none there"
                )
            );
        }

        #[test]
        fn a_main_branch_still_without_a_run_at_a_later_read_is_a_problem_until_it_has_one() {
            let mut w = world(Stage::Merging);
            let fake = host(Checks::Passed, Checks::Passed);
            let mains = fake.mains.clone();
            mains
                .lock()
                .expect("lock")
                .insert("group/ui".into(), Checks::None);
            let mut part = part(fake);
            assert_eq!(part.carry_out(&merge(), &w), merged("ui"));
            w.apply(&Event::Merged {
                issue: gld(),
                repo: Some(pid("ui")),
            });
            assert_eq!(
                part.carry_out(&merge(), &w),
                Outcome::done(),
                "just merged: the run may not exist yet"
            );
            for _ in 0..2 {
                assert!(matches!(
                    part.carry_out(&merge(), &w),
                    Outcome::Failed(why) if why.contains("no CI run on the main branch of ui")
                ));
            }
            mains.lock().expect("lock").clear();
            assert_eq!(part.carry_out(&merge(), &w), merged("svc"));
        }

        #[test]
        fn within_ci_every_a_first_read_with_no_run_lists_nothing() {
            let mut w = world(Stage::Merging);
            let fake = host(Checks::Passed, Checks::Passed);
            fake.mains
                .lock()
                .expect("lock")
                .insert("group/ui".into(), Checks::None);
            let mut part = part(fake).every(60_000);
            assert_eq!(part.carry_out(&merge(), &w), merged("ui"));
            w.apply(&Event::Merged {
                issue: gld(),
                repo: Some(pid("ui")),
            });
            for _ in 0..2 {
                assert_eq!(part.carry_out(&merge(), &w), Outcome::done());
            }
        }

        #[test]
        fn an_older_issue_with_one_pull_request_merges_it_and_deploys() {
            let mut w = world(Stage::Merging);
            w.issues
                .get_mut(&gld())
                .expect("issue")
                .pulls
                .remove(&pid("svc"));
            let mut part = part(host(Checks::Passed, Checks::Passed));
            assert_eq!(part.carry_out(&merge(), &w), merged("ui"));
            let effects = w.apply(&Event::Merged {
                issue: gld(),
                repo: Some(pid("ui")),
            });
            assert!(effects.contains(&Effect::Deploy { issue: gld() }));
        }

        #[test]
        fn a_restart_asks_again_only_for_the_unmerged_pull_request() {
            let mut w = world(Stage::Merging);
            w.issues
                .get_mut(&gld())
                .expect("issue")
                .merged
                .insert(pid("ui"));
            let mut fake = host(Checks::Passed, Checks::Passed);
            fake.pulls[0].1.state = PullState::Merged;
            let repos = fake.merged_repos.clone();
            let mut part = part(fake);
            assert_eq!(part.carry_out(&merge(), &w), merged("svc"));
            assert_eq!(*repos.lock().expect("lock"), ["group/svc"]);
        }
    }
}
