//! The daemon's configuration: which projects it works on and where their
//! issues come from, the work folders agents use, and the groups of agents.
//! It is TOML, read at start; `docs/configuration.md` has an example.
//!
//! The configuration becomes events, so the log records when a project, folder
//! or group was added or changed. Removing one from the file does not remove it
//! from the loop yet.

use std::collections::BTreeSet;

use serde::Deserialize;
use temet_work_core::ids::{FolderId, GroupId, MachineId, MilestoneId, ProjectId};
use temet_work_core::world::LOCAL;
use temet_work_core::{Event, World};

/// The configuration file.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Projects.
    #[serde(default, rename = "project")]
    pub projects: Vec<ProjectConf>,
    /// Work folders.
    #[serde(default, rename = "folder")]
    pub folders: Vec<FolderConf>,
    /// Groups.
    #[serde(default, rename = "group")]
    pub groups: Vec<GroupConf>,
}

/// One `[[project]]`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConf {
    /// A short id, used in issue ids (`radar#44`).
    pub id: String,
    /// The tracker adapter that holds its issues (`github`).
    pub tracker: String,
    /// The tracker's name for it (`owner/repo`).
    pub name: String,
}

/// One `[[folder]]`: a checkout agents work in.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FolderConf {
    /// A short id.
    pub id: String,
    /// The project it is a checkout of.
    pub project: String,
    /// The machine it is on; `local` when left out.
    #[serde(default = "local")]
    pub machine: String,
    /// Where it is on that machine.
    pub path: String,
}

/// One `[[group]]`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupConf {
    /// A short id.
    pub id: String,
    /// Its name as shown; the id when left out.
    pub name: Option<String>,
    /// The projects it works on.
    pub projects: Vec<String>,
    /// The milestones it works through, first to last; every issue of its
    /// projects when left out.
    #[serde(default)]
    pub milestones: Vec<String>,
    /// How many issues it refines at once; 1 when left out.
    #[serde(default = "one")]
    pub refining_at_once: usize,
}

fn local() -> String {
    LOCAL.to_string()
}

fn one() -> usize {
    1
}

impl Config {
    /// Reads a configuration and checks that what it names exists.
    pub fn parse(text: &str) -> Result<Self, String> {
        let config: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        config.check()?;
        Ok(config)
    }

    fn check(&self) -> Result<(), String> {
        let mut projects = BTreeSet::new();
        for p in &self.projects {
            if !projects.insert(p.id.as_str()) {
                return Err(format!("project {} is listed twice", p.id));
            }
        }
        let mut folders = BTreeSet::new();
        for f in &self.folders {
            if !folders.insert(f.id.as_str()) {
                return Err(format!("folder {} is listed twice", f.id));
            }
            if !projects.contains(f.project.as_str()) {
                return Err(format!(
                    "folder {} names project {}, which is not listed",
                    f.id, f.project
                ));
            }
        }
        let mut groups = BTreeSet::new();
        for g in &self.groups {
            if !groups.insert(g.id.as_str()) {
                return Err(format!("group {} is listed twice", g.id));
            }
            if let Some(p) = g.projects.iter().find(|p| !projects.contains(p.as_str())) {
                return Err(format!(
                    "group {} names project {p}, which is not listed",
                    g.id
                ));
            }
        }
        Ok(())
    }

    /// The events that bring `world` in line with this configuration: one for
    /// each project, folder or group that is new or changed.
    pub fn events(&self, world: &World) -> Vec<Event> {
        let mut out = Vec::new();
        for p in &self.projects {
            let id = ProjectId::new(&p.id);
            let same = world
                .projects
                .get(&id)
                .is_some_and(|w| w.tracker == p.tracker && w.name == p.name);
            if !same {
                out.push(Event::ProjectAdded {
                    project: id,
                    tracker: p.tracker.clone(),
                    name: p.name.clone(),
                });
            }
        }
        for f in &self.folders {
            let id = FolderId::new(&f.id);
            let same = world.folders.get(&id).is_some_and(|w| {
                w.project.0 == f.project && w.machine.0 == f.machine && w.path == f.path
            });
            if !same {
                out.push(Event::FolderAdded {
                    folder: id,
                    project: ProjectId::new(&f.project),
                    machine: MachineId::new(&f.machine),
                    path: f.path.clone(),
                });
            }
        }
        for g in &self.groups {
            let id = GroupId::new(&g.id);
            let name = g.name.clone().unwrap_or_else(|| g.id.clone());
            let projects: Vec<ProjectId> = g.projects.iter().map(ProjectId::new).collect();
            let milestones: Vec<MilestoneId> = g.milestones.iter().map(MilestoneId::new).collect();
            let refining_at_once = g.refining_at_once.max(1);
            let same = world.groups.get(&id).is_some_and(|w| {
                w.name == name
                    && w.projects == projects
                    && w.milestones == milestones
                    && w.refining_at_once == refining_at_once
            });
            if !same {
                out.push(Event::GroupCreated {
                    group: id,
                    name,
                    projects,
                    milestones,
                    refining_at_once,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"
[[project]]
id = "ankh"
tracker = "fake"
name = "city/ankh"

[[folder]]
id = "ankh-1"
project = "ankh"
path = "/srv/ankh/1"

[[group]]
id = "watch"
name = "Night watch"
projects = ["ankh"]
milestones = ["Clocks"]
"#;

    #[test]
    fn becomes_events_once() {
        let config = Config::parse(EXAMPLE).expect("parse");
        let events = config.events(&World::default());
        assert_eq!(events.len(), 3);
        assert!(matches!(&events[1], Event::FolderAdded { machine, .. } if machine.0 == LOCAL));
        let (world, _) = World::replay(&events);
        assert_eq!(config.events(&world), vec![], "nothing new the second time");
    }

    #[test]
    fn a_changed_group_is_sent_again() {
        let (world, _) = World::replay(
            &Config::parse(EXAMPLE)
                .expect("parse")
                .events(&World::default()),
        );
        let changed =
            Config::parse(&EXAMPLE.replace("milestones = [\"Clocks\"]", "refining_at_once = 2"))
                .expect("parse");
        let events = changed.events(&world);
        assert!(matches!(
            events.as_slice(),
            [Event::GroupCreated {
                refining_at_once: 2,
                ..
            }]
        ));
    }

    #[test]
    fn refuses_what_names_an_unlisted_project_or_an_unknown_key() {
        let bad = EXAMPLE.replace("project = \"ankh\"", "project = \"klatch\"");
        assert!(Config::parse(&bad).is_err_and(|e| e.contains("klatch")));
        assert!(
            Config::parse("[[project]]\nid = \"a\"\ntracker = \"x\"\nname = \"y\"\ncolour = 1")
                .is_err()
        );
    }
}
