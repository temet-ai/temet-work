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
    /// Areas: projects that work together.
    #[serde(default, rename = "area")]
    pub areas: Vec<AreaConf>,
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
    /// The tracker adapter that holds its issues (`github`); left out when its
    /// area's tracker holds them.
    #[serde(default)]
    pub tracker: Option<String>,
    /// The tracker's name for it (`owner/repo`).
    pub name: String,
    /// The code-host adapter that holds its pull requests; the tracker when
    /// left out.
    pub code_host: Option<String>,
    /// The code host's name for the repository; `name` when left out.
    pub repo: Option<String>,
    /// Run after a merge, through `sh -c`; merged counts as deployed when left out.
    pub deploy: Option<String>,
    /// Run after a deploy, to check it works; deployed counts as verified when
    /// left out.
    pub verify: Option<String>,
    /// The folder `deploy` and `verify` run in; the home folder when left out.
    pub run_in: Option<String>,
    /// Whether its pull requests run CI; `false` leaves the plan's checks to the
    /// reviewer. True when left out.
    pub ci: Option<bool>,
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

/// One `[[area]]`: projects that work together. `docs/areas.md` says why.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AreaConf {
    /// A short id, shared with projects, used in its own issues' ids (`guild#GLD-12`).
    pub id: String,
    /// Its name as shown; the id when left out.
    pub name: Option<String>,
    /// The projects in it.
    pub projects: Vec<String>,
    /// The tracker adapter that holds its own issues, when one tracker project
    /// serves all its projects (`jira`).
    pub tracker: Option<String>,
    /// The tracker's name for that project (`GLD`).
    pub tracker_project: Option<String>,
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
    #[serde(default)]
    pub projects: Vec<String>,
    /// The areas it works on: each area's own issues and its projects'.
    #[serde(default)]
    pub areas: Vec<String>,
    /// The milestones it works through, first to last; every issue of its
    /// projects when left out.
    #[serde(default)]
    pub milestones: Vec<String>,
    /// How many issues it refines at once; 1 when left out.
    #[serde(default = "one")]
    pub refining_at_once: usize,
    /// The model its agents run (`haiku`, `sonnet`); the runtime's default when
    /// left out.
    pub model: Option<String>,
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
        let mut areas = BTreeSet::new();
        let mut in_area = BTreeSet::new();
        for a in &self.areas {
            if projects.contains(a.id.as_str()) || !areas.insert(a.id.as_str()) {
                return Err(format!(
                    "{} names both an area and a project, or two areas",
                    a.id
                ));
            }
            for p in &a.projects {
                if !projects.contains(p.as_str()) {
                    return Err(format!(
                        "area {} names project {p}, which is not listed",
                        a.id
                    ));
                }
                if !in_area.insert(p.as_str()) {
                    return Err(format!("project {p} is in two areas"));
                }
            }
            if a.tracker.is_some() != a.tracker_project.is_some() {
                return Err(format!(
                    "area {} needs both tracker and tracker_project, or neither",
                    a.id
                ));
            }
        }
        for p in &self.projects {
            let area_tracks = self
                .areas
                .iter()
                .any(|a| a.tracker.is_some() && a.projects.contains(&p.id));
            if p.tracker.is_none() && !area_tracks {
                return Err(format!(
                    "project {} names no tracker, and no area tracks its issues",
                    p.id
                ));
            }
            if p.tracker.is_none() && p.code_host.is_none() {
                return Err(format!("project {} names no code_host", p.id));
            }
        }
        let mut groups = BTreeSet::new();
        for g in &self.groups {
            if let Some(a) = g.areas.iter().find(|a| !areas.contains(a.as_str())) {
                return Err(format!(
                    "group {} names area {a}, which is not listed",
                    g.id
                ));
            }
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
            let tracker = p.tracker.clone().unwrap_or_default();
            let code_host = p.code_host.clone().unwrap_or_else(|| tracker.clone());
            let repo = p.repo.clone().unwrap_or_else(|| p.name.clone());
            let same = world.projects.get(&id).is_some_and(|w| {
                w.tracker == tracker
                    && w.name == p.name
                    && w.code_host == code_host
                    && w.repo == repo
            });
            if !same {
                out.push(Event::ProjectAdded {
                    project: id,
                    tracker,
                    name: p.name.clone(),
                    code_host: Some(code_host),
                    repo: Some(repo),
                });
            }
        }
        for p in &self.projects {
            let id = ProjectId::new(&p.id);
            let now = world
                .projects
                .get(&id)
                .map(|w| (w.deploy.clone(), w.verify.clone(), w.run_in.clone(), w.ci))
                .unwrap_or((None, None, None, true));
            let want = (
                p.deploy.clone(),
                p.verify.clone(),
                p.run_in.clone(),
                p.ci.unwrap_or(true),
            );
            if now != want {
                out.push(Event::ProjectCommandsSet {
                    project: id,
                    deploy: want.0,
                    verify: want.1,
                    run_in: want.2,
                    ci: Some(want.3),
                });
            }
        }
        for a in &self.areas {
            let id = ProjectId::new(&a.id);
            let name = a.name.clone().unwrap_or_else(|| a.id.clone());
            let projects: Vec<ProjectId> = a.projects.iter().map(ProjectId::new).collect();
            let tracker = a.tracker.clone().unwrap_or_default();
            let tracker_name = a.tracker_project.clone().unwrap_or_default();
            let same = world.areas.get(&id).is_some_and(|w| {
                w.name == name
                    && w.projects == projects
                    && w.tracker == tracker
                    && w.tracker_name == tracker_name
            });
            if !same {
                out.push(Event::AreaDefined {
                    area: id,
                    name,
                    projects,
                    tracker,
                    tracker_name,
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
            let areas: Vec<ProjectId> = g.areas.iter().map(ProjectId::new).collect();
            let milestones: Vec<MilestoneId> = g.milestones.iter().map(MilestoneId::new).collect();
            let refining_at_once = g.refining_at_once.max(1);
            let same = world.groups.get(&id).is_some_and(|w| {
                w.name == name
                    && w.projects == projects
                    && w.areas == areas
                    && w.milestones == milestones
                    && w.refining_at_once == refining_at_once
                    && w.model == g.model
            });
            if !same {
                out.push(Event::GroupCreated {
                    group: id,
                    name,
                    projects,
                    areas,
                    milestones,
                    refining_at_once,
                    model: g.model.clone(),
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
deploy = "make deploy"

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
        assert_eq!(events.len(), 4);
        assert!(
            matches!(&events[1], Event::ProjectCommandsSet { deploy: Some(d), .. } if d == "make deploy")
        );
        assert!(matches!(&events[2], Event::FolderAdded { machine, .. } if machine.0 == LOCAL));
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

    const AREA: &str = r#"
[[project]]
id = "ui"
name = "group/ui"
code_host = "fake"

[[project]]
id = "api"
name = "group/api"
code_host = "fake"

[[area]]
id = "guild"
name = "Thieves' Guild"
projects = ["ui", "api"]
tracker = "fake"
tracker_project = "GLD"

[[group]]
id = "watch"
areas = ["guild"]
"#;

    #[test]
    fn an_area_with_its_own_tracker_serves_projects_that_name_none() {
        let config = Config::parse(AREA).expect("parse");
        let events = config.events(&World::default());
        assert!(events.iter().any(
            |e| matches!(e, Event::AreaDefined { tracker_name, .. } if tracker_name == "GLD")
        ));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::GroupCreated { areas, .. } if areas.len() == 1))
        );
        let (world, _) = World::replay(&events);
        assert_eq!(config.events(&world), vec![], "nothing new the second time");
        let untracked = AREA.replace("tracker = \"fake\"\ntracker_project = \"GLD\"\n", "");
        assert!(Config::parse(&untracked).is_err_and(|e| e.contains("names no tracker")));
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
