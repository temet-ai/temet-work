//! Code hosts: where an issue's pull request lives, what its CI says, and where
//! it is merged. A project's tracker and its code host can differ: a Jira
//! project tracks issues and hosts no code.
//!
//! This crate holds the [`CodeHost`] contract, a [`FakeCodeHost`] and the
//! [`contract`] every code-host adapter runs. Adapter crates depend on this crate
//! only.

use std::collections::BTreeMap;
use std::fmt;

use temet_work_core::ids::IssueId;
use temet_work_core::model::Stage;
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

    /// Merges the pull request `number`.
    fn merge(&mut self, repo: &str, number: &str) -> Result<(), CodeHostError>;
}

/// A code host in memory, for tests.
#[derive(Default)]
pub struct FakeCodeHost {
    /// Pull requests, as (repo, pull).
    pub pulls: Vec<(String, Pull)>,
}

impl CodeHost for FakeCodeHost {
    fn id(&self) -> &'static str {
        "fake"
    }

    fn pull(&mut self, repo: &str, number: &str) -> Result<Pull, CodeHostError> {
        self.pulls
            .iter()
            .find(|(r, p)| r == repo && p.number == number)
            .map(|(_, p)| p.clone())
            .ok_or_else(|| CodeHostError::NotFound(format!("{repo} pull request {number}")))
    }

    fn merge(&mut self, repo: &str, number: &str) -> Result<(), CodeHostError> {
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
        Ok(())
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
        let pull = host
            .pull(&repo, &number)
            .map_err(|e| format!("pull: {e}"))?;
        ensure(
            pull.number == number,
            "a pull request reports its own number",
        )?;
        ensure(
            host.pull(&repo, "999999999").is_err(),
            "a pull request the code host does not have is an error",
        )?;
        ensure(
            host.pull("no-such/repository", &number).is_err(),
            "an unknown repository is an error",
        )?;
        if let Some(n) = harness.mergeable() {
            host.merge(&repo, &n).map_err(|e| format!("merge: {e}"))?;
            let after = host
                .pull(&repo, &n)
                .map_err(|e| format!("pull after merge: {e}"))?;
            ensure(
                after.state == PullState::Merged,
                "a merged pull request reads as merged",
            )?;
            ensure(host.merge(&repo, &n).is_err(), "merging twice is an error")?;
        }
        Ok(())
    }
}

/// How often an issue's CI is read, by the daemon's clock.
pub const POLL_EVERY_MS: u64 = 30_000;

/// The code-host part: the code hosts compiled into this build, in the order
/// they were added.
///
/// It reads the CI of each pull request under review and reports when it has
/// finished, and merges an issue's pull request when the loop asks.
pub struct CodeHosts {
    adapters: Vec<Box<dyn CodeHost>>,
    every_ms: u64,
    read_at: BTreeMap<IssueId, u64>,
    errors: BTreeMap<IssueId, String>,
}

impl Default for CodeHosts {
    fn default() -> Self {
        Self {
            adapters: Vec::new(),
            every_ms: POLL_EVERY_MS,
            read_at: BTreeMap::new(),
            errors: BTreeMap::new(),
        }
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

    /// Reads CI every `ms` instead of every 30 seconds.
    pub fn every(mut self, ms: u64) -> Self {
        self.every_ms = ms;
        self
    }

    /// The adapter and repository for an issue's project.
    fn host_for(
        &mut self,
        world: &World,
        issue: &IssueId,
    ) -> Result<(&mut Box<dyn CodeHost>, String), String> {
        let project = world
            .projects
            .get(&issue.project)
            .ok_or_else(|| format!("project {} is not configured", issue.project))?;
        let host = self
            .adapters
            .iter_mut()
            .find(|a| a.id() == project.code_host)
            .ok_or_else(|| format!("no code host \"{}\" in this build", project.code_host))?;
        Ok((host, project.repo.clone()))
    }
}

impl Part for CodeHosts {
    fn id(&self) -> &'static str {
        "codehost"
    }

    fn poll(&mut self, world: &World, now_ms: u64) -> Polled {
        let mut events = Vec::new();
        let waiting: Vec<(IssueId, String)> = world
            .issues
            .values()
            .filter(|i| {
                matches!(i.stage, Stage::InReview { .. } | Stage::Arbitration { .. })
                    && i.checks_ok.is_none()
            })
            .filter_map(|i| i.pull.clone().map(|p| (i.id.clone(), p)))
            .collect();
        self.errors
            .retain(|id, _| waiting.iter().any(|(w, _)| w == id));
        for (issue, number) in waiting {
            let due = self
                .read_at
                .get(&issue)
                .is_none_or(|at| now_ms >= at + self.every_ms);
            if !due {
                continue;
            }
            self.read_at.insert(issue.clone(), now_ms);
            let read = self
                .host_for(world, &issue)
                .and_then(|(host, repo)| host.pull(&repo, &number).map_err(|e| e.to_string()));
            let pull = match read {
                Ok(p) => {
                    self.errors.remove(&issue);
                    p
                }
                Err(e) => {
                    self.errors.insert(issue.clone(), format!("{issue}: {e}"));
                    continue;
                }
            };
            let finished = match pull.checks {
                Checks::Pending => None,
                // A repository without CI leaves the plan's checks to the reviewer.
                Checks::Passed | Checks::None => Some((true, Vec::new())),
                Checks::Failed { failing } => Some((false, failing)),
            };
            if let Some((passed, failing)) = finished {
                events.push(Event::ChecksFinished {
                    issue,
                    passed,
                    failing,
                });
            }
        }
        let problem = (!self.errors.is_empty())
            .then(|| self.errors.values().cloned().collect::<Vec<_>>().join("; "));
        Polled { events, problem }
    }

    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let Effect::Merge { issue } = effect else {
            return Outcome::NotMine;
        };
        let Some(number) = world.issues.get(issue).and_then(|i| i.pull.clone()) else {
            return Outcome::Failed(format!("{issue} has no pull request on record"));
        };
        let merged = self.host_for(world, issue).and_then(|(host, repo)| {
            match host.pull(&repo, &number).map_err(|e| e.to_string())?.state {
                // Merged already, by a person or an earlier try.
                PullState::Merged => Ok(()),
                PullState::Closed => {
                    Err(format!("pull request {number} was closed without merging"))
                }
                PullState::Open => host.merge(&repo, &number).map_err(|e| e.to_string()),
            }
        });
        match merged {
            Ok(()) => Outcome::Done {
                events: vec![Event::Merged {
                    issue: issue.clone(),
                }],
                effects: Vec::new(),
            },
            Err(e) => Outcome::Failed(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temet_work_core::ids::ProjectId;

    fn pull(number: &str, checks: Checks) -> Pull {
        Pull {
            number: number.into(),
            state: PullState::Open,
            checks,
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
                i.pull = Some(number.into());
            }
        }
        w
    }

    #[test]
    fn the_fake_passes_the_contract() {
        assert_eq!(contract::check(&mut fake(), &Writable), Ok(()));
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
        assert_eq!(
            part.poll(&w, POLL_EVERY_MS - 1).events,
            vec![],
            "not due yet"
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
            }],
            effects: vec![],
        };
        assert_eq!(part.carry_out(&merge, &w), merged);
        assert_eq!(part.carry_out(&merge, &w), merged, "already merged");
    }

    #[test]
    fn an_issue_without_a_pull_request_cannot_merge() {
        let mut w = in_review();
        if let Some(i) = w.issues.get_mut(&IssueId::new("ankh", "1")) {
            i.pull = None;
        }
        let mut part = CodeHosts::default();
        part.add(Box::new(fake()));
        let merge = Effect::Merge {
            issue: IssueId::new("ankh", "1"),
        };
        assert!(matches!(part.carry_out(&merge, &w), Outcome::Failed(_)));
    }

    #[test]
    fn keeps_adapters_in_the_order_added() {
        let mut part = CodeHosts::default();
        part.add(Box::new(FakeCodeHost::default()));
        assert_eq!(part.ids(), ["fake"]);
        assert_eq!(part.id(), "codehost");
    }
}
