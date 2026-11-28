//! Where an agent runs: the work folders an author is given on the machine its
//! `machine` setting names, the issues waiting because none is free there, and
//! the readers' folder a refiner, reviewer, arbiter or investigator runs in
//! (`docs/settings.md`). A set value is never passed over for a folder on another
//! machine. Also whether a ready issue starts at all once a folder is free, and
//! why not (`docs/configuration.md`).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crate::ids::{FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId};
use crate::model::{Group, Issue, Role, Stage};
use crate::world::{LOCAL, World};

/// A ready issue whose author waits for a work folder on the machine its
/// `machine` setting names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Waiting {
    /// The issue.
    pub issue: IssueId,
    /// The machine the setting names for its author.
    pub machine: MachineId,
    /// A repository the author needs a folder of that has none on the machine at
    /// all.
    pub none_in: Option<ProjectId>,
}

/// Why a ready issue would not start once a work folder is free
/// ([`World::start_waits`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartWait {
    /// The ready issue.
    pub issue: IssueId,
    /// The group that took it.
    pub group: GroupId,
    /// What holds it.
    pub why: Why,
}

/// What holds a ready issue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Why {
    /// Its group no longer covers it, and no other group does: the group no
    /// longer lists this.
    OutOfScope(Unlisted),
    /// An issue it depends on, directly or through other open issues in scope.
    On {
        /// The dependency.
        on: IssueId,
        /// The open issues in scope between the two, nearest to the ready issue
        /// first; none when it names `on` itself.
        through: Vec<IssueId>,
        /// Why `on` holds it.
        hold: Hold,
    },
}

/// Why a dependency holds a ready issue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hold {
    /// The world holds no issue by that id: the daemon never synced it, so
    /// nothing says it is done.
    NeverSynced,
    /// It is open and out of the group's scope: in a milestone the group does not
    /// take (`Some`), or in none any group covers (`None`), among them any in a
    /// repository nothing matches.
    OutOfScope(Option<MilestoneId>),
    /// It is open and in scope, at this stage, and is the far end of the walk.
    NotDone(Stage),
}

/// What a group does not list that it would need to for an issue to be in its
/// scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unlisted {
    /// The issue's project.
    Project(ProjectId),
    /// The issue's area, whose tracker holds it.
    Area(ProjectId),
    /// The issue's milestone.
    Milestone(MilestoneId),
    /// Any milestone: the issue has none, and the group names some.
    NoMilestone,
}

/// What a dependency is to the walk of [`World::start_waits`].
enum Step<'a> {
    /// It is done and holds nothing.
    Done,
    /// It holds the issue, and the walk goes no further.
    Holds(Hold),
    /// It is open and in scope, and the walk follows its own dependencies.
    Open(&'a Issue),
}

impl World {
    /// The machine `issue`'s agents of `role` must run on, when a level sets
    /// `machine`. When none does, an author takes a folder on any machine and
    /// any other agent runs on this one, in its readers' folder.
    pub fn machine_for(&self, issue: &IssueId, role: Role) -> Option<MachineId> {
        self.setting_text(issue, Some(role), "machine")
            .map(MachineId::new)
    }

    /// The id of the repository `issue` is worked in.
    fn folders_of(&self, issue: &Issue) -> ProjectId {
        self.repo_of(issue)
            .map_or_else(|| issue.id.project.clone(), |p| p.id.clone())
    }

    /// The folder readers of `project` run in: its subfolder of the daemon's
    /// readers' folder ([`World::readers`]). It is no checkout, no author is
    /// given it, and the readers of one project share it. None when the world
    /// has no readers' folder.
    pub fn readers_folder(&self, project: &ProjectId) -> Option<PathBuf> {
        self.readers.as_ref().map(|r| r.join(&project.0))
    }

    /// The machine and folder a reader of `role` on `issue` runs in: the
    /// readers' folder of the repository the issue is worked in, on the machine
    /// its `machine` setting names, else this one. It never waits for a work
    /// folder, so an author holding every one keeps no reader from starting.
    pub(crate) fn reader_place(&self, issue: &Issue, role: Role) -> (MachineId, Option<PathBuf>) {
        let machine = self
            .machine_for(&issue.id, role)
            .unwrap_or_else(|| MachineId::new(LOCAL));
        (machine, self.readers_folder(&self.folders_of(issue)))
    }

    /// The free work folders the author of `issue` is given, one in each
    /// repository it changes, on its `machine` when one is set. With none set it
    /// takes folders no ready issue pinned to a machine could use, and any free
    /// ones when that leaves it without.
    pub(crate) fn author_folders(&self, issue: &Issue) -> Option<Vec<FolderId>> {
        let repos = self.repos_of(issue);
        if let Some(machine) = self.machine_for(&issue.id, Role::Author) {
            return self.free_folders(&repos, |f| f.machine == machine);
        }
        let any = self.free_folders(&repos, |_| true)?;
        let wanted = self.pinned_wants();
        let spare = self.free_folders(&repos, |f| {
            !wanted.contains(&(f.project.clone(), f.machine.clone()))
        });
        Some(spare.unwrap_or(any))
    }

    /// The repository and machine of every folder a ready issue pinned to a
    /// machine could use, in any group: one whose author waits for a folder and
    /// for nothing else.
    fn pinned_wants(&self) -> BTreeSet<(ProjectId, MachineId)> {
        self.issues
            .values()
            .filter(|i| self.waits_for_author(i))
            .filter_map(|i| Some((i, self.machine_for(&i.id, Role::Author)?)))
            .flat_map(|(i, machine)| {
                self.repos_of(i)
                    .into_iter()
                    .map(move |repo| (repo, machine.clone()))
            })
            .collect()
    }

    /// Whether `issue` is ready for an author and waits for a work folder alone:
    /// refined, and nothing in [`World::start_waits`] holds it.
    fn waits_for_author(&self, issue: &Issue) -> bool {
        issue.stage == Stage::Ready
            && issue
                .group
                .as_ref()
                .and_then(|g| self.groups.get(g))
                .is_some_and(|g| self.start_waits(g, issue).is_empty())
    }

    /// Why `issue`, ready and taken by `group`, would not start once a work
    /// folder is free; nothing when it would. It is the one rule: the loop starts
    /// an author only on a ready issue it returns nothing for, the group's queue
    /// counts a ready issue only then, and `temet-work status` lists each wait
    /// under Waiting. Work folders and machines are left out
    /// ([`World::machine_waits`]).
    ///
    /// The issue waits when its group no longer covers it, and on each issue it
    /// depends on that is not done. Open dependencies in the group's scope are
    /// followed to their own, nearest first, each issue once. One the world never
    /// synced, and one out of scope (no group covers it, or the group does not
    /// take its milestone, whoever else does), holds the issue where the walk
    /// meets it. An open one in scope holds it where the walk ends: when every
    /// issue it leads to leads back to it, and none leads to a dependency that
    /// holds for one of the other reasons. A cycle is named once, by the first of
    /// its issues the walk meets, so in a cycle of two each issue names the other.
    pub fn start_waits(&self, group: &Group, issue: &Issue) -> Vec<StartWait> {
        let wait = |why| StartWait {
            issue: issue.id.clone(),
            group: group.id.clone(),
            why,
        };
        let not_done = |on: &Issue, through: Vec<IssueId>| Why::On {
            on: on.id.clone(),
            through,
            hold: Hold::NotDone(on.stage),
        };
        let mut out: Vec<StartWait> = self
            .unlisted(group, issue)
            .map(|u| wait(Why::OutOfScope(u)))
            .into_iter()
            .collect();
        // An issue that names itself waits on itself.
        if issue.depends_on.contains(&issue.id) {
            out.push(wait(not_done(issue, Vec::new())));
        }
        // What the walk met, in order: a wait, or an open dependency in scope with
        // the issues before it.
        let mut met: Vec<Result<StartWait, (&Issue, Vec<IssueId>)>> = Vec::new();
        let mut seen = BTreeSet::from([&issue.id]);
        let mut next = VecDeque::from([(issue, Vec::new())]);
        while let Some((from, through)) = next.pop_front() {
            for id in &from.depends_on {
                if !seen.insert(id) {
                    continue;
                }
                match self.step(group, id) {
                    Step::Done => {}
                    Step::Holds(hold) => met.push(Ok(wait(Why::On {
                        on: id.clone(),
                        through: through.clone(),
                        hold,
                    }))),
                    Step::Open(on) => {
                        met.push(Err((on, through.clone())));
                        let mut path = through.clone();
                        path.push(id.clone());
                        next.push_back((on, path));
                    }
                }
            }
        }
        let reach: BTreeMap<&IssueId, Option<BTreeSet<&IssueId>>> = met
            .iter()
            .filter_map(|m| m.as_ref().err().map(|(on, _)| *on))
            .chain([issue])
            .map(|i| (&i.id, self.reach(group, i)))
            .collect();
        // Every issue `id` leads to leads back to it, and none to another hold.
        let far_end = |id: &IssueId| {
            reach.get(id).and_then(Option::as_ref).is_some_and(|ahead| {
                ahead.iter().all(|a| {
                    reach
                        .get(a)
                        .and_then(Option::as_ref)
                        .is_some_and(|back| back.contains(id))
                })
            })
        };
        // The far ends named so far: a cycle is named once.
        let mut ends: Vec<&IssueId> = Vec::new();
        for m in met {
            match m {
                Ok(w) => out.push(w),
                Err((on, through)) if far_end(&on.id) => {
                    let named = ends.iter().any(|e| {
                        reach
                            .get(e)
                            .and_then(Option::as_ref)
                            .is_some_and(|r| r.contains(&on.id))
                    });
                    if !named {
                        ends.push(&on.id);
                        out.push(wait(not_done(on, through)));
                    }
                }
                Err(_) => {}
            }
        }
        out
    }

    /// What dependency `id` of a ready issue in `group` is to the walk. One the
    /// world never synced holds it, unless the name is one nothing matches, which
    /// no group can cover.
    fn step(&self, group: &Group, id: &IssueId) -> Step<'_> {
        let Some(on) = self.issues.get(id) else {
            return Step::Holds(if id.unmatched().is_some() {
                Hold::OutOfScope(None)
            } else {
                Hold::NeverSynced
            });
        };
        if on.stage == Stage::Done {
            return Step::Done;
        }
        let left_out = on
            .milestone
            .as_ref()
            .filter(|m| !group.milestones.is_empty() && !group.milestones.contains(m));
        if left_out.is_none() && self.groups.values().any(|g| self.in_scope(g, on)) {
            Step::Open(on)
        } else {
            Step::Holds(Hold::OutOfScope(left_out.cloned()))
        }
    }

    /// The open issues in scope that `issue` leads to through its dependencies,
    /// itself among them; `None` when one of them, or `issue`, depends on an
    /// issue that holds for another reason.
    fn reach<'a>(&'a self, group: &Group, issue: &'a Issue) -> Option<BTreeSet<&'a IssueId>> {
        let mut seen = BTreeSet::from([&issue.id]);
        let mut todo = vec![issue];
        while let Some(from) = todo.pop() {
            for id in &from.depends_on {
                match self.step(group, id) {
                    Step::Done => {}
                    Step::Holds(_) => return None,
                    Step::Open(on) => {
                        if seen.insert(&on.id) {
                            todo.push(on);
                        }
                    }
                }
            }
        }
        Some(seen)
    }

    /// Every ready issue whose author waits for a work folder on a set machine,
    /// and for nothing else, such as an issue it depends on or its group's scope.
    /// Only authors wait for a folder: a reader runs in its readers' folder.
    pub fn machine_waits(&self) -> Vec<Waiting> {
        self.issues
            .values()
            .filter_map(|i| self.machine_wait(i))
            .collect()
    }

    fn machine_wait(&self, i: &Issue) -> Option<Waiting> {
        if self.paused(i) || !self.waits_for_author(i) {
            return None;
        }
        let machine = self.machine_for(&i.id, Role::Author)?;
        // An author holds a folder of every repository the issue changes.
        let repos = self.repos_of(i);
        if self
            .free_folders(&repos, |f| f.machine == machine)
            .is_some()
        {
            return None;
        }
        let none_in = repos.into_iter().find(|repo| {
            !self
                .folders
                .values()
                .any(|f| &f.project == repo && f.machine == machine)
        });
        Some(Waiting {
            issue: i.id.clone(),
            machine,
            none_in,
        })
    }
}
