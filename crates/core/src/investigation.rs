//! Investigations: when a group runs short of work, the loop starts one
//! investigator for it, which reads the group's projects and files the next
//! issues (`docs/configuration.md`, `docs/commands.md`).
//!
//! The clock comes in as the `now_ms` that [`crate::Host::tick`] takes, and the
//! times it sets are carried on the events, so nothing here reads one and
//! replaying the log gives the same state.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::brief;
use crate::effect::Effect;
use crate::event::Event;
use crate::ids::{AgentId, GroupId, IssueId, MachineId, MilestoneId, ProjectId};
use crate::launch::Launch;
use crate::model::{AgentState, Folder, Group, Investigation, Issue, Role, Stage};
use crate::settings::Scope;
use crate::world::{LOCAL, World};

/// The work queue length under which a group with no work folders starts an
/// investigator, when its configuration names none.
pub const QUEUE_LOW_WITHOUT_FOLDERS: usize = 2;

/// How far back a group's first investigation reads merged pull requests.
pub const FIRST_SINCE_DAYS: u64 = 7;

impl World {
    /// The group's work queue: its issues refining or ready, and the new issues
    /// it would take that no group has taken yet. Those count because the group
    /// refines them next; leaving them out would start investigations that file
    /// them again.
    pub fn queue(&self, group: &Group) -> usize {
        self.issues
            .values()
            .filter(|i| match i.stage {
                Stage::Refining | Stage::Ready => i.group.as_ref() == Some(&group.id),
                Stage::New => i.group.is_none() && self.in_scope(group, i),
                _ => false,
            })
            .count()
    }

    /// The repositories a group covers: its projects, then those of its areas,
    /// each once.
    pub fn group_repos(&self, group: &Group) -> Vec<ProjectId> {
        let mut out: Vec<ProjectId> = Vec::new();
        let in_areas = group
            .areas
            .iter()
            .filter_map(|a| self.areas.get(a))
            .flat_map(|a| a.projects.iter());
        for p in group.projects.iter().chain(in_areas) {
            if !out.contains(p) {
                out.push(p.clone());
            }
        }
        out
    }

    /// Where a group's issues are filed: each of its projects and areas whose
    /// tracker holds issues, and each project of its areas with a tracker of its
    /// own, in that order and each once.
    pub fn group_sources(&self, group: &Group) -> Vec<ProjectId> {
        let mut out: Vec<ProjectId> = Vec::new();
        let mut add = |id: &ProjectId| {
            if !out.contains(id) {
                out.push(id.clone());
            }
        };
        for p in &group.projects {
            match self.projects.get(p) {
                Some(project) if project.tracker.is_empty() => {
                    if let Some(area) = self
                        .areas
                        .values()
                        .find(|a| a.projects.contains(p) && !a.tracker.is_empty())
                    {
                        add(&area.id);
                    }
                }
                Some(_) => add(p),
                None => {}
            }
        }
        for a in group.areas.iter().filter_map(|a| self.areas.get(a)) {
            if !a.tracker.is_empty() {
                add(&a.id);
            }
            for p in &a.projects {
                if self.projects.get(p).is_some_and(|p| !p.tracker.is_empty()) {
                    add(p);
                }
            }
        }
        out
    }

    /// Below how many issues in its queue a group starts an investigator: its
    /// `queue_low`, else twice the work folders of the repositories it covers,
    /// else 2.
    pub fn queue_low(&self, group: &Group) -> usize {
        group.queue_low.unwrap_or_else(|| {
            let repos = self.group_repos(group);
            match self
                .folders
                .values()
                .filter(|f| repos.contains(&f.project))
                .count()
            {
                0 => QUEUE_LOW_WITHOUT_FOLDERS,
                n => n * 2,
            }
        })
    }

    /// The group's investigation while one runs.
    pub fn investigating(&self, group: &GroupId) -> Option<&Investigation> {
        self.investigations.get(group).filter(|i| i.running())
    }

    /// What the clock makes due at `now_ms`, as events: the end of each running
    /// investigation whose investigator said it is done or whose session ended,
    /// and, when `start`, the start of one for each group whose queue is below
    /// its `queue_low`, that runs none, and whose last investigation ended at
    /// least `investigate_every` minutes ago. A group never investigated may
    /// start at once.
    pub fn investigation_events(&self, now_ms: u64, start: bool) -> Vec<Event> {
        let mut out = Vec::new();
        for i in self.investigations.values().filter(|i| i.running()) {
            let ended = self
                .agents
                .get(&i.agent)
                .is_some_and(|a| a.state == AgentState::Ended);
            if i.done || ended {
                out.push(Event::InvestigationEnded {
                    group: i.group.clone(),
                    at_ms: now_ms,
                    filed: i.filed.len(),
                });
            }
        }
        if !start {
            return out;
        }
        let used: BTreeSet<&str> = self
            .agents
            .keys()
            .map(|a| a.0.as_str())
            .chain(self.investigations.values().map(|i| i.agent.0.as_str()))
            .collect();
        let mut names: Vec<String> = Vec::new();
        let mut n = self
            .agents
            .values()
            .filter(|a| a.role == Role::Investigator)
            .count()
            + 1;
        for g in self.groups.values() {
            if g.projects.is_empty() && g.areas.is_empty() {
                continue;
            }
            let last = self.investigations.get(&g.id);
            if last.is_some_and(Investigation::running) {
                continue;
            }
            let rested = last
                .and_then(|i| i.ended_ms)
                .is_none_or(|end| now_ms >= end.saturating_add(g.investigate_every * 60_000));
            if !rested || self.queue(g) >= self.queue_low(g) {
                continue;
            }
            let agent = loop {
                let name = format!("{}-{n}", Role::Investigator.name());
                n += 1;
                if !used.contains(name.as_str()) && !names.contains(&name) {
                    break name;
                }
            };
            names.push(agent.clone());
            out.push(Event::InvestigationStarted {
                group: g.id.clone(),
                agent: AgentId::new(agent),
                at_ms: now_ms,
            });
        }
        out
    }

    /// The investigations whose investigator has not started, such as one whose
    /// start failed or was lost to a restart: each needs its investigator.
    pub(crate) fn missing_investigators(&self) -> Vec<Effect> {
        self.investigations
            .values()
            .filter(|i| i.running() && !i.done && !self.agents.contains_key(&i.agent))
            .map(|i| Effect::Investigate {
                group: i.group.clone(),
            })
            .collect()
    }

    /// Whether `agent` is the investigator of its group's running investigation.
    pub(crate) fn investigates(&self, agent: &crate::model::Agent) -> bool {
        agent.role == Role::Investigator
            && agent.issue.is_none()
            && self
                .investigating(&agent.group)
                .is_some_and(|i| i.agent == agent.id && !i.done)
    }

    /// The level an agent's settings are read at: its issue's, or for an agent
    /// with none, such as an investigator, its group's first repository's (its
    /// first area's when it has no repository).
    pub fn agent_scope(&self, group: &GroupId, issue: Option<&IssueId>) -> Scope {
        if let Some(issue) = issue {
            return Scope::Task(issue.clone());
        }
        let Some(g) = self.groups.get(group) else {
            return Scope::Global;
        };
        self.group_repos(g)
            .into_iter()
            .next()
            .map(Scope::Project)
            .or_else(|| g.areas.first().cloned().map(Scope::Area))
            .unwrap_or(Scope::Global)
    }

    /// The folder a group's investigator runs in without holding it: a free one
    /// of the group's first repository, else one an author holds, on the machine
    /// its `machine` setting names when one does.
    fn investigator_folder(&self, group: &Group) -> Option<&Folder> {
        let first = self.group_repos(group).into_iter().next()?;
        let machine = self
            .text_at(
                &Scope::Project(first.clone()),
                Some(Role::Investigator),
                "machine",
            )
            .map(MachineId::new);
        let usable = || {
            self.folders
                .values()
                .filter(|f| f.project == first && machine.as_ref().is_none_or(|m| &f.machine == m))
        };
        usable()
            .find(|f| f.holder.is_none())
            .or_else(|| usable().next())
    }

    /// The milestone an investigator's issue gets when it names none: the last of
    /// its group's, none when the group names none.
    pub fn filing_milestone(&self, group: &Group) -> Option<MilestoneId> {
        group.milestones.last().cloned()
    }

    /// What starting `group`'s investigator takes: the id its investigation
    /// recorded, the machine and folder it runs in (holding none) and its brief.
    /// `None` when the group runs no investigation.
    pub fn launch_investigator(&self, group: &GroupId) -> Option<Launch> {
        let g = self.groups.get(group)?;
        let investigation = self.investigating(group)?;
        let scope = self.agent_scope(group, None);
        let place = self.investigator_folder(g);
        let sources = self.group_sources(g);
        let covered = sources.iter().map(|source| self.covered(source)).collect();
        let mut closed: Vec<&Issue> = self
            .issues
            .values()
            .filter(|i| sources.contains(&i.id.project) && i.closed_by_a_person())
            .collect();
        closed.sort_by_key(|i| i.order);
        let notes = self
            .group_repos(g)
            .into_iter()
            .filter_map(|p| {
                // Read for no role: only investigators read it.
                let text = self.text_at(&Scope::Project(p.clone()), None, "investigate")?;
                Some((p, text))
            })
            .collect();
        Some(Launch {
            agent: investigation.agent.clone(),
            group: group.clone(),
            machine: place.map_or_else(|| MachineId::new(LOCAL), |f| f.machine.clone()),
            folder: None,
            brief: crate::launch::Brief {
                model: self.text_at(&scope, Some(Role::Investigator), "model"),
                permission: self
                    .text_at(&scope, Some(Role::Investigator), "permission")
                    .unwrap_or_default(),
                ..brief::investigate(
                    &brief::Investigating {
                        group: g,
                        covered,
                        closed,
                        notes,
                        milestone: self.filing_milestone(g),
                        since: investigation.since_ms.map(utc),
                    },
                    brief::Workplace {
                        folder: place.map(|f| PathBuf::from(&f.path)),
                        shared: place.is_some_and(|f| f.holder.is_some()),
                        held: Vec::new(),
                    },
                    &self.kept_rules(),
                )
            },
        })
    }

    /// One place an investigator files issues in, with the repositories it reads.
    fn covered(&self, source: &ProjectId) -> brief::Covered {
        let folder_of = |p: &ProjectId| {
            self.folders
                .values()
                .find(|f| &f.project == p)
                .map(|f| f.path.clone())
        };
        let repos = match self.areas.get(source) {
            Some(a) => a.projects.clone(),
            None => vec![source.clone()],
        };
        brief::Covered {
            source: source.clone(),
            tracker: self.source_name_of(source).unwrap_or_default().to_string(),
            area: self.areas.contains_key(source),
            repos: repos
                .into_iter()
                .map(|r| {
                    let folder = folder_of(&r);
                    (r, folder)
                })
                .collect(),
        }
    }

    /// The tracker's name for a project or an area.
    fn source_name_of(&self, source: &ProjectId) -> Option<&str> {
        self.projects
            .get(source)
            .map(|p| p.name.as_str())
            .or_else(|| self.areas.get(source).map(|a| a.tracker_name.as_str()))
    }

    /// Stops `agent` when its session has not ended.
    pub(crate) fn stop_agent(&self, agent: &AgentId) -> Vec<Effect> {
        self.agents
            .get(agent)
            .filter(|a| a.state != AgentState::Ended)
            .map(|a| Effect::StopAgent {
                agent: a.id.clone(),
            })
            .into_iter()
            .collect()
    }
}

/// A time in milliseconds since the Unix epoch as `YYYY-MM-DD HH:MM UTC`.
pub fn utc(ms: u64) -> String {
    let secs = ms / 1000;
    let days = secs / 86_400;
    let rest = secs % 86_400;
    let (y, m, d) = civil(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02} UTC",
        rest / 3600,
        rest % 3600 / 60
    )
}

/// The calendar date `days` after 1970-01-01 (Howard Hinnant's `civil_from_days`).
fn civil(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + u64::from(m <= 2);
    (y, m, d)
}

#[cfg(test)]
#[path = "investigation_tests.rs"]
mod tests;
