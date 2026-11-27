//! Where an agent may run: the work folders its `machine` setting leaves it, and
//! the issues waiting because none of them can take it (`docs/settings.md`). A set
//! value is never passed over for a folder on another machine.

use crate::effect::Effect;
use crate::ids::{IssueId, MachineId, ProjectId};
use crate::model::{Folder, Issue, Role, Stage};
use crate::world::{LOCAL, World};

/// An issue whose next agent waits for a work folder on the machine its `machine`
/// setting names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Waiting {
    /// The issue.
    pub issue: IssueId,
    /// The agent that cannot start.
    pub role: Role,
    /// The machine the setting names for it.
    pub machine: MachineId,
    /// A repository the agent needs a folder of that has none on the machine at
    /// all.
    pub none_in: Option<ProjectId>,
}

impl World {
    /// The machine `issue`'s agents of `role` must run on, when a level sets
    /// `machine`; any machine when none does.
    pub fn machine_for(&self, issue: &IssueId, role: Role) -> Option<MachineId> {
        self.setting_text(issue, Some(role), "machine")
            .map(MachineId::new)
    }

    /// The id of the repository `issue` is worked in.
    fn folders_of(&self, issue: &Issue) -> ProjectId {
        self.repo_of(issue)
            .map_or_else(|| issue.id.project.clone(), |p| p.id.clone())
    }

    /// The work folders an agent of `role` on `issue` may be given: those of the
    /// repository it is worked in, on its `machine` when one is set.
    pub(crate) fn usable_folders<'a>(
        &'a self,
        issue: &Issue,
        role: Role,
    ) -> impl Iterator<Item = &'a Folder> + use<'a> {
        let project = self.folders_of(issue);
        let machine = self.machine_for(&issue.id, role);
        self.folders.values().filter(move |f| {
            f.project == project && machine.as_ref().is_none_or(|m| &f.machine == m)
        })
    }

    /// Whether an agent of `role` that holds no folder of its own has somewhere to
    /// run on `issue`. It may share a held folder. A project with no work folder
    /// runs it on this machine, which a `machine` of `local` allows.
    pub(crate) fn has_place(&self, issue: &Issue, role: Role) -> bool {
        let Some(machine) = self.machine_for(&issue.id, role) else {
            return true;
        };
        if self.usable_folders(issue, role).next().is_some() {
            return true;
        }
        let project = self.folders_of(issue);
        machine.0 == LOCAL && !self.folders.values().any(|f| f.project == project)
    }

    /// Whether `effect` would start an agent its `machine` setting leaves with
    /// nowhere to run. The loop drops it, and asks again once that changes.
    pub(crate) fn waits_for_machine(&self, effect: &Effect) -> bool {
        let (issue, role) = match effect {
            Effect::StartAgent {
                issue,
                role,
                folder: None,
                ..
            } => (issue, *role),
            // A running reviewer is told of the round; only a new one needs a place.
            Effect::StartReview { issue, .. } if self.running(issue, Role::Reviewer).is_none() => {
                (issue, Role::Reviewer)
            }
            _ => return false,
        };
        self.issues
            .get(issue)
            .is_some_and(|i| !self.has_place(i, role))
    }

    /// Every issue whose next agent waits for a work folder on a set machine, and
    /// for nothing else, such as an issue it depends on.
    pub fn machine_waits(&self) -> Vec<Waiting> {
        self.issues
            .values()
            .filter_map(|i| self.machine_wait(i))
            .collect()
    }

    fn machine_wait(&self, i: &Issue) -> Option<Waiting> {
        let role = match i.stage {
            Stage::New if self.groups.values().any(|g| self.in_scope(g, i)) => Role::Refiner,
            Stage::Refining => Role::Refiner,
            Stage::Ready if i.group.is_some() && self.ready_to_start(i) => Role::Author,
            Stage::InReview { .. } if !i.review_waived => Role::Reviewer,
            Stage::Arbitration { .. } => Role::Arbiter,
            _ => return None,
        };
        let machine = self.machine_for(&i.id, role)?;
        // An author holds a folder of every repository the issue changes; every
        // other agent may share one of the first.
        let (served, repos) = if role == Role::Author {
            let repos = self.repos_of(i);
            (self.free_folders(&repos, Some(&machine)).is_some(), repos)
        } else {
            let served = self.running(&i.id, role).is_some() || self.has_place(i, role);
            (served, vec![self.folders_of(i)])
        };
        let none_in = repos.into_iter().find(|repo| {
            !self
                .folders
                .values()
                .any(|f| &f.project == repo && f.machine == machine)
        });
        (!served).then(|| Waiting {
            issue: i.id.clone(),
            role,
            machine,
            none_in,
        })
    }
}
