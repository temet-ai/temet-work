//! The daemon's configuration: which projects it works on and where their
//! issues come from, the work folders agents use, and the groups of agents.
//! It is TOML, read at start; `docs/configuration.md` has an example.
//!
//! The configuration becomes events, so the log records when a project, folder
//! or group was added or changed. Removing one from the file does not remove it
//! from the loop yet. Settings (`docs/settings.md`) are the exception: the tables
//! `[settings]`, `[project.settings]`, `[area.settings]` and `[[milestone]]` become
//! `SettingSet` events recorded as made by the file, and a value that is no longer
//! in the file is unset again, unless a person or an agent has set it since.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use temet_work_core::ids::{FolderId, GroupId, MachineId, MilestoneId, ProjectId};
use temet_work_core::model::Role;
use temet_work_core::settings::{self, FILE, Key, Scope, Value};
use temet_work_core::world::LOCAL;
use temet_work_core::{Event, World};

/// The configuration file.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// This machine as other machines reach it over SSH (`omit`, or a name from
    /// their `~/.ssh/config`). Needed when a `[[folder]]` is on another machine.
    #[serde(default)]
    pub daemon_host: Option<String>,
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
    /// Milestones with settings of their own.
    #[serde(default, rename = "milestone")]
    pub milestones: Vec<MilestoneConf>,
    /// The global settings; a table per role holds that role's.
    pub settings: Option<toml::Table>,
}

/// One `[[milestone]]`: settings for the issues of one milestone of a project or
/// an area.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MilestoneConf {
    /// The project or area whose tracker names the milestone.
    pub of: String,
    /// The milestone's name, as its tracker has it.
    pub name: String,
    /// Its settings.
    #[serde(default)]
    pub settings: toml::Table,
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
    /// reviewer. True when left out. Read as the project's `ci` setting.
    pub ci: Option<bool>,
    /// Its settings; a table per role holds that role's.
    pub settings: Option<toml::Table>,
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

/// What the file says a setting is, by where, for which role and which setting.
type Wanted = BTreeMap<Key, Value>;

/// Reads a `[settings]`-style table at `scope`: scalars are the settings for every
/// role, a table named for a role holds that role's, of the settings read per role.
/// In either, a table `model` holds a model for each runtime it names.
fn read_settings(table: &toml::Table, scope: &Scope, out: &mut Wanted) -> Result<(), String> {
    let at = |scope: &Scope| match scope {
        Scope::Global => "[settings]".to_string(),
        other => format!("the settings of {other}"),
    };
    let mut read = |role: Option<Role>, key: &str, value: &toml::Value| {
        // TOML cannot hold `model = "x"` and `model.y` in one table, so a
        // runtime's model is an entry of a `model` table: the key `model.y`.
        let entries = match value {
            toml::Value::Table(models) if key == "model" => models
                .iter()
                .map(|(runtime, model)| (format!("model.{runtime}"), model))
                .collect(),
            _ => vec![(key.to_string(), value)],
        };
        for (key, value) in entries {
            let value = value_of(&key, value).map_err(|e| format!("{}: {e}", at(scope)))?;
            out.insert((scope.clone(), role, key), value);
        }
        Ok::<(), String>(())
    };
    for (key, value) in table {
        if let toml::Value::Table(inner) = value
            && key != "model"
        {
            let role = Role::parse(key).map_err(|_| {
                format!(
                    "{}: {key} is not a setting, and is not a role either; \
                     the roles are refiner, author, reviewer, arbiter, investigator",
                    at(scope)
                )
            })?;
            let name = role.name();
            let role_table = match scope {
                Scope::Global => format!("[settings.{name}]"),
                Scope::Project(id) => format!("[project.settings.{name}] of {id}"),
                Scope::Area(id) => format!("[area.settings.{name}] of {id}"),
                Scope::Milestone { .. } | Scope::Task(_) => {
                    format!("[milestone.settings.{name}] of {scope}")
                }
            };
            for (key, value) in inner {
                settings::check_role(key, "role table")
                    .map_err(|e| format!("{role_table}: {e}"))?;
                read(Some(role), key, value)?;
            }
            continue;
        }
        read(None, key, value)?;
    }
    Ok(())
}

/// A setting's value from TOML, checked against what the setting takes.
fn value_of(key: &str, value: &toml::Value) -> Result<Value, String> {
    settings::spec(key)?;
    let value = match value {
        toml::Value::Boolean(b) => Value::Flag(*b),
        toml::Value::Integer(n) => {
            Value::Number(u64::try_from(*n).map_err(|_| format!("{key} cannot be {n}"))?)
        }
        toml::Value::String(t) => Value::Text(t.clone()),
        other => return Err(format!("{key} takes a single value, not {other}")),
    };
    settings::check_value(key, &value)?;
    Ok(value)
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
    /// Its settings; a table per role holds that role's.
    pub settings: Option<toml::Table>,
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
    /// left out. Read as the `model` setting of the projects and areas it works on.
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
        for m in &self.milestones {
            if !projects.contains(m.of.as_str()) && !areas.contains(m.of.as_str()) {
                return Err(format!(
                    "a milestone names {}, which is not a listed project or area",
                    m.of
                ));
            }
            if m.name.trim().is_empty() {
                return Err(format!("a milestone of {} has no name", m.of));
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
        self.wanted()?;
        Ok(())
    }

    /// Every setting the file gives, with the legacy keys read as settings at
    /// their level: a group's `model` on the projects and areas it works on, a
    /// project's `ci`, `deploy`, `verify` and `run_in` on the project. A table
    /// written out wins over a legacy key at the same place.
    fn wanted(&self) -> Result<Wanted, String> {
        let mut out = Wanted::new();
        let mut from_group: BTreeMap<Scope, (String, String)> = BTreeMap::new();
        for g in &self.groups {
            let Some(model) = &g.model else { continue };
            let scopes = g
                .projects
                .iter()
                .map(|p| Scope::Project(ProjectId::new(p)))
                .chain(g.areas.iter().map(|a| Scope::Area(ProjectId::new(a))));
            for scope in scopes {
                if let Some((other, was)) = from_group.get(&scope)
                    && was != model
                {
                    return Err(format!(
                        "groups {other} and {} give {scope} different models ({was}, {model}); \
                         set model on {scope} instead",
                        g.id
                    ));
                }
                from_group.insert(scope.clone(), (g.id.clone(), model.clone()));
                out.insert(
                    (scope, None, "model".into()),
                    value_of("model", &toml::Value::String(model.clone()))?,
                );
            }
        }
        for p in &self.projects {
            let scope = Scope::Project(ProjectId::new(&p.id));
            for (key, value) in [
                ("deploy", p.deploy.clone().map(toml::Value::String)),
                ("verify", p.verify.clone().map(toml::Value::String)),
                ("run_in", p.run_in.clone().map(toml::Value::String)),
                ("ci", p.ci.filter(|ci| !ci).map(toml::Value::Boolean)),
            ] {
                if let Some(v) = value {
                    out.insert((scope.clone(), None, key.into()), value_of(key, &v)?);
                }
            }
        }
        if let Some(t) = &self.settings {
            read_settings(t, &Scope::Global, &mut out)?;
        }
        for p in &self.projects {
            if let Some(t) = &p.settings {
                read_settings(t, &Scope::Project(ProjectId::new(&p.id)), &mut out)?;
            }
        }
        for a in &self.areas {
            if let Some(t) = &a.settings {
                read_settings(t, &Scope::Area(ProjectId::new(&a.id)), &mut out)?;
            }
        }
        let mut seen = BTreeSet::new();
        for m in &self.milestones {
            if !seen.insert((&m.of, &m.name)) {
                return Err(format!("milestone {}:{} is listed twice", m.of, m.name));
            }
            let scope = Scope::Milestone {
                of: ProjectId::new(&m.of),
                name: m.name.clone(),
            };
            read_settings(&m.settings, &scope, &mut out)?;
        }
        Ok(out)
    }

    /// Why this build cannot run with the file's settings: one that names a
    /// runtime or runner it does not have, or gives such a runtime a model.
    pub fn check_build(&self, world: &World) -> Result<(), String> {
        for ((scope, _, key), value) in self.wanted()? {
            world
                .check_choice(&key, &value)
                .map_err(|e| format!("the settings of {scope}: {e}"))?;
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
        // Before the groups, so what a group takes on already has its settings.
        let wanted = self.wanted().unwrap_or_default();
        for (key, value) in &wanted {
            let current = world.settings.get(key);
            // Not over a value a person or an agent has set since.
            if current.is_none_or(|e| e.by == FILE && &e.value != value) {
                out.push(Event::SettingSet {
                    scope: key.0.clone(),
                    role: key.1,
                    key: key.2.clone(),
                    value: value.clone(),
                    by: FILE.into(),
                });
            }
        }
        for (key, entry) in &world.settings {
            if entry.by == FILE && !wanted.contains_key(key) {
                out.push(Event::SettingUnset {
                    scope: key.0.clone(),
                    role: key.1,
                    key: key.2.clone(),
                    by: FILE.into(),
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
            });
            if !same {
                out.push(Event::GroupCreated {
                    group: id,
                    name,
                    projects,
                    areas,
                    milestones,
                    refining_at_once,
                    model: None,
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
    fn reads_the_daemon_host() {
        let c = Config::parse("daemon_host = \"omit\"\n").expect("config");
        assert_eq!(c.daemon_host.as_deref(), Some("omit"));
        assert_eq!(Config::parse("").expect("config").daemon_host, None);
    }

    #[test]
    fn becomes_events_once() {
        let config = Config::parse(EXAMPLE).expect("parse");
        let events = config.events(&World::default());
        assert_eq!(events.len(), 4);
        assert!(matches!(&events[1], Event::FolderAdded { machine, .. } if machine.0 == LOCAL));
        assert!(matches!(
            &events[2],
            Event::SettingSet { key, value: Value::Text(d), by, .. }
                if key == "deploy" && d == "make deploy" && by == FILE
        ));
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

    /// Today's configuration file, as the daemon read it before settings.
    const TODAY: &str = r#"
[[project]]
id = "radar"
tracker = "github"
name = "temet-ai/radar"
deploy = "make deploy"
verify = "make smoke"
run_in = "/home/me/src/radar-deploy"
ci = false

[[folder]]
id = "radar-1"
project = "radar"
path = "/home/me/src/radar-1"

[[group]]
id = "group-1"
name = "Group 1"
projects = ["radar"]
milestones = ["Sign-in rework"]
refining_at_once = 1
model = "haiku"
"#;

    fn world_of(text: &str) -> World {
        let config = Config::parse(text).expect("parse");
        let (world, _) = World::replay(&config.events(&World::default()));
        assert_eq!(config.events(&world), vec![], "nothing new the second time");
        world
    }

    #[test]
    fn todays_file_loads_unchanged_and_its_old_keys_are_settings_at_their_level() {
        let w = world_of(TODAY);
        let issue = temet_work_core::ids::IssueId::new("radar", "44");
        let text = |key| w.setting_text(&issue, None, key);
        // Before the issue is synced the chain still starts at the project.
        let at = |key| w.resolve(&Scope::Project(ProjectId::new("radar")), None, key);
        assert_eq!(text("deploy").as_deref(), Some("make deploy"));
        assert_eq!(
            at("verify").map(|r| r.value),
            Some(Value::Text("make smoke".into()))
        );
        assert_eq!(
            at("run_in").map(|r| r.value),
            Some(Value::Text("/home/me/src/radar-deploy".into()))
        );
        assert_eq!(at("ci").map(|r| r.value), Some(Value::Flag(false)));
        assert_eq!(
            at("model").map(|r| r.value),
            Some(Value::Text("haiku".into()))
        );
        assert_eq!(
            at("model").map(|r| r.level),
            Some(temet_work_core::settings::Level::Project)
        );
        assert!(w.settings.values().all(|e| e.by == FILE));
        // The group is as it was.
        assert_eq!(w.groups[&GroupId::new("group-1")].milestones.len(), 1);
    }

    /// Every table the settings add.
    const NEW: &str = r#"
[settings]
runtime = "alpha"
model = "sonnet"
review_rounds = 4

[settings.reviewer]
runtime = "beta"

[[project]]
id = "radar"
tracker = "github"
name = "temet-ai/radar"

[project.settings]
model = "haiku"
ci = false

[project.settings.author]
permission = "plan"

[[project]]
id = "ledger"
name = "temet-ai/ledger"
code_host = "github"

[[area]]
id = "guild"
projects = ["radar", "ledger"]
tracker = "jira"
tracker_project = "GLD"

[area.settings]
merge = "rebase"

[[milestone]]
of = "radar"
name = "Sign-in rework"

[milestone.settings]
model = "opus"

[[milestone]]
of = "guild"
name = "Heist"

[milestone.settings]
starts = 1
"#;

    #[test]
    fn every_new_table_becomes_a_setting_event_by_the_file() {
        let config = Config::parse(NEW).expect("parse");
        let events = config.events(&World::default());
        let sets: Vec<&Event> = events
            .iter()
            .filter(|e| matches!(e, Event::SettingSet { .. }))
            .collect();
        assert_eq!(sets.len(), 10, "{sets:?}");
        assert!(
            sets.iter()
                .all(|e| matches!(e, Event::SettingSet { by, .. } if by == FILE))
        );
        let w = world_of(NEW);
        let get = |scope: Scope, role: Option<Role>, key: &str| {
            w.settings
                .get(&(scope, role, key.to_string()))
                .map(|e| e.value.clone())
        };
        let radar = || Scope::Project(ProjectId::new("radar"));
        assert_eq!(
            get(Scope::Global, None, "model"),
            Some(Value::Text("sonnet".into()))
        );
        assert_eq!(
            get(Scope::Global, None, "review_rounds"),
            Some(Value::Number(4))
        );
        assert_eq!(
            get(Scope::Global, Some(Role::Reviewer), "runtime"),
            Some(Value::Text("beta".into()))
        );
        assert_eq!(get(radar(), None, "ci"), Some(Value::Flag(false)));
        assert_eq!(
            get(radar(), Some(Role::Author), "permission"),
            Some(Value::Text("plan".into()))
        );
        assert_eq!(
            get(Scope::Area(ProjectId::new("guild")), None, "merge"),
            Some(Value::Text("rebase".into()))
        );
        assert_eq!(
            get(
                Scope::Milestone {
                    of: ProjectId::new("radar"),
                    name: "Sign-in rework".into()
                },
                None,
                "model"
            ),
            Some(Value::Text("opus".into()))
        );
        assert_eq!(
            get(
                Scope::Milestone {
                    of: ProjectId::new("guild"),
                    name: "Heist".into()
                },
                None,
                "starts"
            ),
            Some(Value::Number(1))
        );
    }

    #[test]
    fn a_machine_is_read_from_every_settings_table() {
        let text =
            NEW.replace(
                "review_rounds = 4\n",
                "review_rounds = 4\nmachine = \"local\"\n",
            )
            .replace(
                "[settings.reviewer]\n",
                "[settings.author]\nmachine = \"mac\"\n\n[settings.reviewer]\n",
            )
            .replace("ci = false\n", "ci = false\nmachine = \"infra\"\n")
            .replace(
                "model = \"opus\"\n",
                "model = \"opus\"\nmachine = \"mac\"\n",
            ) + "\n[milestone.settings.reviewer]\nmachine = \"deus\"\n";
        let w = world_of(&text);
        let get = |scope: Scope, role: Option<Role>| {
            w.settings
                .get(&(scope, role, "machine".to_string()))
                .map(|e| e.value.to_string())
        };
        let milestone = |of: &str, name: &str| Scope::Milestone {
            of: ProjectId::new(of),
            name: name.into(),
        };
        let radar = Scope::Project(ProjectId::new("radar"));
        assert_eq!(get(Scope::Global, None).as_deref(), Some("local"));
        assert_eq!(
            get(Scope::Global, Some(Role::Author)).as_deref(),
            Some("mac")
        );
        assert_eq!(get(radar, None).as_deref(), Some("infra"));
        assert_eq!(
            get(milestone("radar", "Sign-in rework"), None).as_deref(),
            Some("mac")
        );
        assert_eq!(
            get(milestone("guild", "Heist"), Some(Role::Reviewer)).as_deref(),
            Some("deus")
        );
        let empty = format!("[settings]\nmachine = \"\"\n{TODAY}");
        assert!(Config::parse(&empty).is_err_and(|e| e.contains("machine")));
    }

    #[test]
    fn a_model_table_gives_each_runtime_its_model_at_every_level_and_role() {
        let text = r#"
[settings.model]
alpha = "sonnet"
beta = "gpt-x"

[settings.reviewer.model]
beta = "gpt-y"

[[project]]
id = "ankh"
tracker = "fake"
name = "city/ankh"

[project.settings.model]
alpha = "haiku"

[project.settings.author.model]
beta = "gpt-z"

[[area]]
id = "guild"
projects = ["ankh"]

[area.settings.model]
alpha = "guild's"

[[milestone]]
of = "ankh"
name = "Clocks"

[milestone.settings.model]
alpha = "opus"

[milestone.settings.arbiter]
model = { beta = "gpt-w" }
"#;
        let w = world_of(text);
        let ankh = || Scope::Project(ProjectId::new("ankh"));
        let milestone = || Scope::Milestone {
            of: ProjectId::new("ankh"),
            name: "Clocks".into(),
        };
        let want = [
            (Scope::Global, None, "model.alpha", "sonnet"),
            (Scope::Global, None, "model.beta", "gpt-x"),
            (Scope::Global, Some(Role::Reviewer), "model.beta", "gpt-y"),
            (ankh(), None, "model.alpha", "haiku"),
            (ankh(), Some(Role::Author), "model.beta", "gpt-z"),
            (
                Scope::Area(ProjectId::new("guild")),
                None,
                "model.alpha",
                "guild's",
            ),
            (milestone(), None, "model.alpha", "opus"),
            (milestone(), Some(Role::Arbiter), "model.beta", "gpt-w"),
        ];
        assert_eq!(w.settings.len(), want.len(), "{:?}", w.settings);
        for (scope, role, key, value) in want {
            let entry = &w.settings[&(scope, role, key.to_string())];
            assert_eq!(
                (entry.value.to_string().as_str(), entry.by.as_str()),
                (value, FILE)
            );
        }
        for (bad, named) in [
            ("[settings.model]\nalpha = 4\n", "model.alpha"),
            (
                "[settings.model.reviewer]\nalpha = \"x\"\n",
                "model.reviewer",
            ),
            ("[settings.model]\nalpha = \"\"\n", "model.alpha"),
        ] {
            let why = Config::parse(bad).expect_err(bad);
            assert!(why.contains(named), "{why}");
        }
    }

    #[test]
    fn a_model_for_a_runtime_the_build_lacks_stops_the_start_naming_it() {
        use temet_work_core::settings::Choice;
        let mut world = World::default();
        world.catalog.insert(
            "runtime".into(),
            Choice {
                key: "runtime".into(),
                allowed: vec!["alpha".into(), "beta".into()],
                default: Some("alpha".into()),
            },
        );
        for table in ["[settings.model]", "[settings.reviewer.model]"] {
            let text = format!("{table}\nnonesuch = \"x\"\n{TODAY}");
            let config = Config::parse(&text).expect("the file is well formed");
            let err = config.check_build(&world).expect_err("refused");
            assert!(
                err.contains("model.nonesuch") && err.contains("alpha, beta"),
                "{err}"
            );
        }
        let had = format!("[settings.model]\nbeta = \"x\"\n{TODAY}");
        let config = Config::parse(&had).expect("parse");
        assert!(config.check_build(&world).is_ok());
    }

    /// The first TOML block under `heading` in `docs/configuration.md`.
    fn documented(heading: &str) -> &'static str {
        include_str!("../docs/configuration.md")
            .split_once(&format!("\n## {heading}\n"))
            .and_then(|(_, after)| after.split_once("```toml\n"))
            .and_then(|(_, block)| block.split_once("```"))
            .map(|(block, _)| block)
            .expect("the section has a toml block")
    }

    #[test]
    fn the_documented_examples_load() {
        Config::parse(documented("Example")).expect("the example");
        let mut w = world_of(documented("Sending work to one machine"));
        let issue = temet_work_core::ids::IssueId::new("radar", "44");
        w.apply(&Event::IssueSynced {
            issue: issue.clone(),
            title: "Sign in".into(),
            milestone: Some(MilestoneId::new("Slack sign-in rework")),
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        let machine = |role| w.machine_for(&issue, role).map(|m| m.0);
        assert_eq!(machine(Role::Author).as_deref(), Some("mac"));
        assert_eq!(machine(Role::Reviewer).as_deref(), Some("local"));
    }

    #[test]
    fn a_table_written_out_wins_over_an_old_key_and_a_changed_file_changes_the_setting() {
        let text = TODAY.replace(
            "ci = false\n",
            "ci = false\n\n[project.settings]\nverify = \"make check\"\nmodel = \"opus\"\n",
        );
        let w = world_of(&text);
        let at = |key| w.resolve(&Scope::Project(ProjectId::new("radar")), None, key);
        assert_eq!(
            at("verify").map(|r| r.value),
            Some(Value::Text("make check".into()))
        );
        assert_eq!(
            at("model").map(|r| r.value),
            Some(Value::Text("opus".into()))
        );
        // The file changes: the value is set again; a table taken out leaves the old
        // key's value, here the group's model.
        let again = Config::parse(
            &text
                .replace("make check", "make verify")
                .replace("model = \"opus\"\n", ""),
        )
        .expect("parse");
        let events = again.events(&w);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::SettingSet { key, value: Value::Text(v), .. } if key == "verify" && v == "make verify"
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            Event::SettingSet { key, value: Value::Text(v), .. } if key == "model" && v == "haiku"
        )));
        // A key taken out with nothing under it is unset.
        let bare = Config::parse(
            &text
                .replace("verify = \"make check\"\n", "")
                .replace("verify = \"make smoke\"\n", ""),
        )
        .expect("parse");
        assert!(bare.events(&w).iter().any(|e| matches!(
            e,
            Event::SettingUnset { key, by, .. } if key == "verify" && by == FILE
        )));
    }

    #[test]
    fn what_a_person_set_is_not_overwritten_by_the_file_on_the_next_read() {
        let config = Config::parse(NEW).expect("parse");
        let (mut w, _) = World::replay(&config.events(&World::default()));
        w.apply(&Event::SettingSet {
            scope: Scope::Global,
            role: None,
            key: "model".into(),
            value: Value::Text("fable".into()),
            by: "person".into(),
        });
        assert_eq!(config.events(&w), vec![]);
        assert_eq!(
            w.settings[&(Scope::Global, None, "model".to_string())].value,
            Value::Text("fable".into())
        );
    }

    #[test]
    fn a_key_the_daemon_does_not_know_is_an_error_naming_the_key() {
        for (where_, text) in [
            ("[settings]\ncolour = \"red\"\n", "colour"),
            ("[settings.reviewer]\nreview_roundz = 2\n", "review_roundz"),
            ("[settings.janitor]\nmodel = \"x\"\n", "janitor"),
        ] {
            let err = Config::parse(&format!("{where_}{TODAY}")).expect_err(text);
            assert!(err.contains(text), "{err}");
        }
        let in_project = TODAY.replace("ci = false", "ci = false\n[project.settings]\nflavour = 1");
        assert!(Config::parse(&in_project).is_err_and(|e| e.contains("flavour")));
        let milestone = format!(
            "{TODAY}\n[[milestone]]\nof = \"radar\"\nname = \"M\"\n[milestone.settings]\nspeed = 3\n"
        );
        assert!(Config::parse(&milestone).is_err_and(|e| e.contains("speed")));
        let unlisted = format!("{TODAY}\n[[milestone]]\nof = \"nowhere\"\nname = \"M\"\n");
        assert!(Config::parse(&unlisted).is_err_and(|e| e.contains("nowhere")));
    }

    #[test]
    fn a_role_table_refuses_a_setting_not_read_per_role() {
        let area = "[[area]]\nid = \"guild\"\nprojects = [\"radar\"]\n";
        let milestone = "[[milestone]]\nof = \"radar\"\nname = \"M\"\n";
        for (table, text) in [
            (
                "[settings.reviewer]",
                format!("[settings]\n[settings.reviewer]\nci = false\n{TODAY}"),
            ),
            (
                "[project.settings.reviewer]",
                TODAY.replace(
                    "ci = false\n",
                    "ci = false\n[project.settings.reviewer]\nci = false\n",
                ),
            ),
            (
                "[area.settings.reviewer]",
                format!("{TODAY}\n{area}[area.settings.reviewer]\nci = false\n"),
            ),
            (
                "[milestone.settings.reviewer]",
                format!("{TODAY}\n{milestone}[milestone.settings.reviewer]\nci = false\n"),
            ),
        ] {
            let why = Config::parse(&text).expect_err(table);
            assert!(
                why.starts_with(table)
                    && why.contains("ci is the same for every role, so it takes no role table"),
                "{why}"
            );
        }
        Config::parse(&format!("[settings.reviewer]\nmodel = \"opus\"\n{TODAY}"))
            .expect("the model is read per role");
    }

    #[test]
    fn a_value_of_the_wrong_kind_is_an_error_and_so_is_a_model_two_groups_disagree_on() {
        let bad = format!("[settings]\nreview_rounds = \"many\"\n{TODAY}");
        assert!(
            Config::parse(&bad)
                .is_err_and(|e| e.contains("review_rounds") && e.contains("whole number"))
        );
        let two = format!(
            "{TODAY}\n[[group]]\nid = \"group-2\"\nprojects = [\"radar\"]\nmodel = \"opus\"\n"
        );
        assert!(Config::parse(&two).is_err_and(|e| e.contains("group-1") && e.contains("group-2")));
    }

    #[test]
    fn a_configuration_with_an_unknown_permission_mode_is_refused() {
        for table in ["[settings]", "[settings.author]"] {
            let text = format!("{table}\npermission = \"yolo\"\n{TODAY}");
            let why = Config::parse(&text).expect_err("refused");
            assert!(
                why.contains(
                    "permission takes one of default, acceptEdits, plan, auto, dontAsk, \
                     bypassPermissions, not yolo"
                ),
                "{why}"
            );
        }
    }

    #[test]
    fn a_runtime_this_build_lacks_stops_the_start_listing_the_ones_it_has() {
        use temet_work_core::settings::Choice;
        let text = format!("[settings]\nruntime = \"nonesuch\"\n{TODAY}");
        let config = Config::parse(&text).expect("the file is well formed");
        let mut world = World::default();
        world.catalog.insert(
            "runtime".into(),
            Choice {
                key: "runtime".into(),
                allowed: vec!["alpha".into(), "beta".into()],
                default: Some("alpha".into()),
            },
        );
        let err = config.check_build(&world).expect_err("refused");
        assert!(
            err.contains("nonesuch") && err.contains("alpha, beta"),
            "{err}"
        );
        assert!(
            Config::parse(TODAY)
                .expect("parse")
                .check_build(&world)
                .is_ok()
        );
    }
}
