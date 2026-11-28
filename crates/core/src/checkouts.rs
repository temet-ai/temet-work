//! Readers' own checkouts: which ones a reviewer, arbiter, refiner or
//! investigator starts with, where each goes, and when each is removed
//! (`docs/architecture.md`, "Who coordinates"). The loop asks for a checkout and
//! for its removal; a part does the git work and reports what it did.

use std::path::Path;

use crate::brief;
use crate::effect::Effect;
use crate::ids::{AgentId, MachineId, ProjectId};
use crate::launch::Launch;
use crate::model::{Agent, AgentState, Checkout, Issue, Role};
use crate::world::{LOCAL, World};

/// The folder beside a work folder that holds the checkouts made from it, one
/// folder per event log inside it, so a second daemon's never land among the
/// first's.
pub const CHECKOUTS_DIR: &str = "temet-work-checkouts";

/// What a world with no log id yet names its log in a checkout's path. The
/// daemon starts every log with an id, so only a test meets it.
const NO_LOG: &str = "no-log";

/// One repository a reader reads, and the branch whose head its checkout is
/// made at; none for the head of the default branch.
pub(crate) type Read = (ProjectId, Option<String>);

impl World {
    /// The repositories a reader of `role` on `issue` reads, each with the
    /// branch its checkout is made at: for a reviewer or an arbiter, each
    /// repository with an open pull request, in the order they merge, at that
    /// pull request's branch (the first repository at its default branch when
    /// none is on record, as in older logs); for a refiner, each of the area's
    /// repositories for an area's issue, else the issue's own, at the default
    /// branch. An author reads nothing here: it works in its work folders.
    pub(crate) fn reads(&self, issue: &Issue, role: Role) -> Vec<Read> {
        match role {
            Role::Reviewer | Role::Arbiter => {
                let open: Vec<Read> = self
                    .pulls_of(issue)
                    .into_iter()
                    .filter(|(r, _)| !issue.merged.contains(*r))
                    .map(|(r, _)| (r.clone(), issue.branches.get(r).cloned()))
                    .collect();
                if open.is_empty() {
                    self.repos_of(issue)
                        .into_iter()
                        .take(1)
                        .map(|r| (r, None))
                        .collect()
                } else {
                    open
                }
            }
            Role::Refiner => {
                let area = self.area_repos(&issue.id);
                let repos = if area.is_empty() {
                    self.repos_of(issue)
                } else {
                    area
                };
                repos.into_iter().map(|r| (r, None)).collect()
            }
            Role::Author | Role::Investigator => Vec::new(),
        }
    }

    /// The checkouts `agent` gets of `reads`, all on one machine: `machine`
    /// when a setting names one, else that of the first work folder of the
    /// first repository read that has one. A repository with no work folder
    /// there gets none. Each is added from the first work folder of its
    /// repository on that machine, and goes in
    /// `temet-work-checkouts/<log id>/<name>` beside it. A checkout made for
    /// this agent's earlier try at starting, of the same repository at the same
    /// branch, is given again, so a start that keeps failing does not make one
    /// every round.
    pub(crate) fn plan_checkouts(
        &self,
        agent: &AgentId,
        machine: Option<MachineId>,
        reads: &[Read],
    ) -> Vec<Checkout> {
        let Some(machine) = machine.or_else(|| {
            reads.iter().find_map(|(r, _)| {
                self.folders
                    .values()
                    .find(|f| &f.project == r)
                    .map(|f| f.machine.clone())
            })
        }) else {
            return Vec::new();
        };
        let log = self.log.as_deref().unwrap_or(NO_LOG);
        let mut out: Vec<Checkout> = Vec::new();
        for (repo, branch) in reads {
            if out.iter().any(|c| &c.project == repo) {
                continue;
            }
            let Some(from) = self
                .folders
                .values()
                .find(|f| &f.project == repo && f.machine == machine)
            else {
                continue;
            };
            let again = self.checkouts.values().find(|c| {
                &c.agent == agent
                    && &c.project == repo
                    && &c.branch == branch
                    && c.machine == machine
                    && c.kept.is_none()
            });
            if let Some(c) = again {
                out.push(c.clone());
                continue;
            }
            let name = (1u32..)
                .map(|n| format!("{repo}-{n}"))
                .find(|n| !self.checkouts.contains_key(n) && !out.iter().any(|c| &c.name == n))
                .unwrap_or_else(|| format!("{repo}-{agent}"));
            out.push(Checkout {
                path: beside(&from.path, log, &name),
                name,
                agent: agent.clone(),
                project: repo.clone(),
                machine: machine.clone(),
                from: from.path.clone(),
                branch: branch.clone(),
                kept: None,
            });
        }
        out
    }

    /// The machine an agent with `checkouts` runs on: theirs, else this one.
    pub(crate) fn machine_of(checkouts: &[Checkout]) -> MachineId {
        checkouts
            .first()
            .map_or_else(|| MachineId::new(LOCAL), |c| c.machine.clone())
    }

    /// How a reader's brief names `checkouts`, and the repositories of `reads`
    /// it has none of, which it reads on their code hosts.
    pub(crate) fn place_of(&self, checkouts: &[Checkout], reads: &[Read]) -> brief::Workplace {
        let own: Vec<brief::Own> = checkouts.iter().map(|c| self.own(c)).collect();
        brief::Workplace {
            folder: checkouts.first().map(|c| c.path.clone().into()),
            held: Vec::new(),
            without: reads
                .iter()
                .map(|(r, _)| r)
                .filter(|r| !checkouts.iter().any(|c| &c.project == *r))
                .cloned()
                .collect(),
            checkouts: own,
        }
    }

    fn own(&self, c: &Checkout) -> brief::Own {
        brief::Own {
            project: c.project.clone(),
            repo: self
                .projects
                .get(&c.project)
                .map_or_else(String::new, |p| p.repo.clone()),
            path: c.path.clone(),
            branch: c.branch.clone(),
        }
    }

    /// `agent`'s own checkouts as its next-round message names them, each at
    /// the branch of its repository's pull request now on record.
    pub fn own_checkouts(&self, agent: &Agent) -> Vec<brief::Own> {
        let issue = agent.issue.as_ref().and_then(|i| self.issues.get(i));
        agent
            .checkouts
            .iter()
            .map(|c| brief::Own {
                project: c.project.clone(),
                repo: self
                    .projects
                    .get(&c.project)
                    .map_or_else(String::new, |p| p.repo.clone()),
                path: c.path.clone(),
                branch: issue.and_then(|i| i.branches.get(&c.project).cloned()),
            })
            .collect()
    }

    /// The first of `launch`'s checkouts that is not made yet, as the effect
    /// that makes it and then offers `start` again; none once each is made, when
    /// the agent's program may run.
    pub fn checkout_to_make(&self, launch: &Launch, start: &Effect) -> Option<Effect> {
        let c = launch
            .checkouts
            .iter()
            .find(|c| self.checkouts.get(&c.name) != Some(*c))?;
        Some(Effect::AddCheckout {
            checkout: c.clone(),
            then: Box::new(start.clone()),
        })
    }

    /// The removal of each checkout of `agent` that is not kept.
    pub(crate) fn remove_checkouts_of(&self, agent: &AgentId) -> Vec<Effect> {
        self.checkouts
            .values()
            .filter(|c| &c.agent == agent && c.kept.is_none())
            .map(|c| Effect::RemoveCheckout {
                checkout: c.clone(),
            })
            .collect()
    }

    /// The removal of each checkout whose agent has ended and that is not kept:
    /// asked for every round, so one that failed is tried again. The host waits
    /// a minute between tries of one checkout.
    pub(crate) fn removals(&self) -> Vec<Effect> {
        self.checkouts
            .values()
            .filter(|c| c.kept.is_none() && self.agents.get(&c.agent).is_some_and(ended))
            .map(|c| Effect::RemoveCheckout {
                checkout: c.clone(),
            })
            .collect()
    }

    /// What the daemon asks for in its first round: the removal of each
    /// checkout whose agent has ended or never started, as after a crash
    /// between making the checkout and starting the agent or a restart that
    /// lost a removal, and of each kept one, which is checked again and removed
    /// once it is gone or clean.
    pub fn sweep(&self) -> Vec<Effect> {
        self.checkouts
            .values()
            .filter(|c| c.kept.is_some() || self.agents.get(&c.agent).is_none_or(ended))
            .map(|c| Effect::RemoveCheckout {
                checkout: c.clone(),
            })
            .collect()
    }

    /// Whether `effect` would start a reviewer or an arbiter before the code
    /// host has reported the branch of each open pull request it reviews: its
    /// checkouts are made at those branches' heads. The loop drops it, and asks
    /// again each round, as for a start that waits for a machine.
    pub(crate) fn waits_for_branch(&self, effect: &Effect) -> bool {
        let issue = match effect {
            Effect::StartReview { issue, .. } if self.running(issue, Role::Reviewer).is_none() => {
                issue
            }
            Effect::StartAgent {
                issue,
                role: Role::Arbiter,
                ..
            } => issue,
            _ => return false,
        };
        self.issues.get(issue).is_some_and(|i| {
            i.pulls
                .keys()
                .any(|r| !i.merged.contains(r) && !i.branches.contains_key(r))
        })
    }
}

/// Whether `agent` has ended.
fn ended(agent: &Agent) -> bool {
    agent.state == AgentState::Ended
}

/// Where the checkout `name` of log `log` goes: in `temet-work-checkouts/<log>/`
/// beside the work folder at `from`, on that folder's machine. A folder named
/// by a relative path has its checkouts beside it too.
fn beside(from: &str, log: &str, name: &str) -> String {
    let trimmed = from.trim_end_matches('/');
    let parent = match trimmed {
        "" => String::new(),
        "." | ".." => format!("{trimmed}/.."),
        _ => match Path::new(trimmed).parent().and_then(Path::to_str) {
            Some("") | None => ".".to_string(),
            Some(p) => p.trim_end_matches('/').to_string(),
        },
    };
    format!("{parent}/{CHECKOUTS_DIR}/{log}/{name}")
}

#[cfg(test)]
mod tests {
    use super::beside;

    #[test]
    fn a_checkout_goes_beside_its_work_folder_in_a_folder_of_its_log() {
        for (from, path) in [
            ("/srv/ankh/f1", "/srv/ankh/temet-work-checkouts/l1/watch-1"),
            ("/srv/ankh/f1/", "/srv/ankh/temet-work-checkouts/l1/watch-1"),
            ("/f1", "/temet-work-checkouts/l1/watch-1"),
            ("f1", "./temet-work-checkouts/l1/watch-1"),
            ("work/f1", "work/temet-work-checkouts/l1/watch-1"),
            (".", "./../temet-work-checkouts/l1/watch-1"),
        ] {
            assert_eq!(beside(from, "l1", "watch-1"), path, "{from}");
        }
    }
}
