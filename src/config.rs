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
use temet_work_core::command::PERSON;
use temet_work_core::ids::{FolderId, GroupId, MachineId, MilestoneId, ProjectId};
use temet_work_core::model::{INVESTIGATE_EVERY, Role};
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
    // A setting at a level that never reads it is refused under the level's own
    // table, whether it is in that table or in a role's. A key that is no setting
    // is refused where it is read.
    let level_reads = |key: &str| match settings::spec(key) {
        Ok(_) => settings::check_level(key, scope.level())
            .map_err(|e| format!("{}: {e}", table_name(scope, None, ""))),
        Err(_) => Ok(()),
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
            let role_table = table_name(scope, Some(role), "");
            for (key, value) in inner {
                level_reads(key)?;
                settings::check_role(key, "role table")
                    .map_err(|e| format!("{role_table}: {e}"))?;
                read(Some(role), key, value)?;
            }
            continue;
        }
        level_reads(key)?;
        read(None, key, value)?;
    }
    Ok(())
}

/// The table of the file that holds `scope`'s settings for `role` (none: for
/// every role), with `sub` (`.model`) after it, as an error names it.
fn table_name(scope: &Scope, role: Option<Role>, sub: &str) -> String {
    let role = role.map_or_else(String::new, |r| format!(".{}", r.name()));
    match scope {
        Scope::Global => format!("[settings{role}{sub}]"),
        Scope::Project(id) => format!("[project.settings{role}{sub}] of {id}"),
        Scope::Area(id) => format!("[area.settings{role}{sub}] of {id}"),
        Scope::Milestone { .. } | Scope::Task(_) => {
            format!("[milestone.settings{role}{sub}] of {scope}")
        }
    }
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
    /// Below how many issues in its work queue it starts an investigator; twice
    /// the work folders of its projects (2 with none) when left out, and 0 for
    /// never.
    pub queue_low: Option<usize>,
    /// Minutes from the end of its last investigation before another may start;
    /// 120 when left out.
    pub investigate_every: Option<u64>,
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
            if g.investigate_every == Some(0) {
                return Err(format!(
                    "group {} has investigate_every 0: it takes at least 1 minute, so a group with \
                     nothing left to plan does not investigate in a loop",
                    g.id
                ));
            }
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
        self.group_models()?;
        self.wanted()?;
        Ok(())
    }

    /// A group's old `model`, as the plain `model` of each project and area it
    /// works on. It is kept apart from the tables' settings because it binds
    /// differently ([`Config::bound`]).
    fn group_models(&self) -> Result<Wanted, String> {
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
        Ok(out)
    }

    /// Every setting the file's tables give, with a project's old keys `ci`,
    /// `deploy`, `verify` and `run_in` read as settings on the project. A table
    /// written out wins over an old key at the same place.
    fn wanted(&self) -> Result<Wanted, String> {
        let mut out = Wanted::new();
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

    /// The file's settings with each plain `model` bound as it is written: to
    /// `model.<runtime>` of the file's global runtime (its `[settings]` runtime,
    /// else the build's default in `world`'s catalog). A table's plain `model` is
    /// refused where the file gives that scope and role another runtime; a
    /// group's old `model` never is, since it predates models per runtime. Only
    /// the file is read, never the live settings: a person's later `set global
    /// runtime` must not turn the file's model into another runtime's on the
    /// next start. A table's own model wins over a group's at the same place.
    fn bound(&self, world: &World) -> Result<Wanted, String> {
        let file = self.file_runtimes(world)?;
        let (plain, mut out): (Wanted, Wanted) = self
            .wanted()?
            .into_iter()
            .partition(|((_, _, key), _)| key == "model");
        for ((scope, role, key), value) in plain {
            let bound = file.model_key(&scope, role, &key).map_err(|e| {
                format!(
                    "{}: {e}, an entry of {}",
                    table_name(&scope, role, ""),
                    table_name(&scope, role, ".model")
                )
            })?;
            out.entry((scope, role, bound)).or_insert(value);
        }
        for ((scope, role, key), value) in self.group_models()? {
            out.entry((scope, role, file.bound_key(&key)))
                .or_insert(value);
        }
        Ok(out)
    }

    /// A world that holds only `world`'s catalog, the file's areas and the
    /// runtimes the file gives.
    fn file_runtimes(&self, world: &World) -> Result<World, String> {
        let mut file = World {
            catalog: world.catalog.clone(),
            ..World::default()
        };
        for a in &self.areas {
            file.apply(&Event::AreaDefined {
                area: ProjectId::new(&a.id),
                name: a.id.clone(),
                projects: a.projects.iter().map(ProjectId::new).collect(),
                tracker: String::new(),
                tracker_name: String::new(),
            });
        }
        for ((scope, role, key), value) in self.wanted()? {
            if key == "runtime" {
                file.apply(&Event::SettingSet {
                    scope,
                    role,
                    key,
                    value,
                    by: FILE.into(),
                });
            }
        }
        Ok(file)
    }

    /// Why this build cannot run with the file's settings: one that names a
    /// runtime or runner it does not have, or gives such a runtime a model, or a
    /// plain model in a table whose scope and role run a runtime other than the
    /// global one, by the file or by a person's command ([`Config::check_live`]).
    pub fn check_build(&self, world: &World) -> Result<(), String> {
        for ((scope, _, key), value) in self.bound(world)? {
            world
                .check_choice(&key, &value)
                .map_err(|e| format!("the settings of {scope}: {e}"))?;
        }
        self.check_live(world)
    }

    /// A plain `model` of the file's tables against the runtimes `world` holds
    /// once this file's events are applied, as the start will leave them, so a
    /// runtime only an earlier file set no longer counts. It is refused where
    /// its scope and role run neither the runtime it is bound to nor the live
    /// global runtime ([`World::runtime_unlike_model`]). A group's old `model`
    /// is never refused ([`Config::bound`]).
    fn check_live(&self, world: &World) -> Result<(), String> {
        let Some(bound) = self.file_runtimes(world)?.global_runtime() else {
            return Ok(());
        };
        let bound = bound.to_string();
        let (live, _) = world.clone().replay_onto(&self.events(world));
        for (scope, role, _) in self.wanted()?.keys().filter(|(_, _, k)| k == "model") {
            let Some(here) = live.runtime_unlike_model(scope, *role, &bound) else {
                continue;
            };
            let runs = here.value;
            let who = match here.by.as_deref() {
                Some(PERSON) => "a person",
                Some(by) => by,
                None => "the build",
            };
            let whose = role.map_or_else(
                || "every role's".to_string(),
                |r| format!("the {}s'", r.name()),
            );
            let at = match here.scope {
                Some(Scope::Global) | None => "globally".to_string(),
                Some(other) => format!("at {other}"),
            };
            return Err(format!(
                "{}: {who} set {whose} runtime to {runs} {at}, and a plain model in the file \
                 is the file's global runtime's ({bound}): name the runtime, as model.{runs}, \
                 an entry of {}",
                table_name(scope, *role, ""),
                table_name(scope, *role, ".model")
            ));
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
        // Settings the file cannot bind (`check_build` refuses them) change
        // nothing, rather than unset what the file set before.
        if let Ok(wanted) = self.bound(world) {
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
                    && w.queue_low == g.queue_low
                    && w.investigate_every == g.investigate_every.unwrap_or(INVESTIGATE_EVERY)
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
                    queue_low: g.queue_low,
                    investigate_every: g.investigate_every,
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

    #[test]
    fn a_group_reads_queue_low_and_investigate_every_or_leaves_them_to_the_defaults() {
        let group = |config: &Config| {
            let (world, _) = World::replay(&config.events(&World::default()));
            world.groups[&GroupId::new("watch")].clone()
        };
        let plain = Config::parse(EXAMPLE).expect("parse");
        let g = group(&plain);
        assert_eq!((g.queue_low, g.investigate_every), (None, 120));
        let set = Config::parse(&EXAMPLE.replace(
            "milestones = [\"Clocks\"]",
            "milestones = [\"Clocks\"]\nqueue_low = 5\ninvestigate_every = 30",
        ))
        .expect("parse");
        let g = group(&set);
        assert_eq!((g.queue_low, g.investigate_every), (Some(5), 30));
        // A change of either is sent again; the same values are not.
        let (world, _) = World::replay(&plain.events(&World::default()));
        assert!(matches!(
            set.events(&world).as_slice(),
            [Event::GroupCreated {
                queue_low: Some(5),
                investigate_every: Some(30),
                ..
            }]
        ));
        let (world, _) = World::replay(&set.events(&World::default()));
        assert_eq!(set.events(&world), vec![]);
        let never = EXAMPLE.replace("milestones = [\"Clocks\"]", "investigate_every = 0");
        let why = Config::parse(&never).expect_err("0 minutes");
        assert!(why.contains("investigate_every 0"), "{why}");
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
        // [settings] names alpha, so its plain model is alpha's.
        assert_eq!(
            get(Scope::Global, None, "model.alpha"),
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
                "model.alpha"
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

    /// A world in a build with the stand-in runtimes alpha, its default, and beta.
    fn two_runtimes() -> World {
        let mut world = World::default();
        world.catalog.insert(
            "runtime".into(),
            temet_work_core::settings::Choice {
                key: "runtime".into(),
                allowed: vec!["alpha".into(), "beta".into()],
                default: Some("alpha".into()),
            },
        );
        world
    }

    /// The model in the brief of `group`'s `role` on a new issue `project#44` of
    /// the milestone Sign-in rework.
    fn brief_model(world: &World, group: &str, project: &str, role: Role) -> Option<String> {
        let issue = temet_work_core::ids::IssueId::new(project, "44");
        let mut w = world.clone();
        w.apply(&Event::IssueSynced {
            issue: issue.clone(),
            title: "Sign in".into(),
            milestone: Some(MilestoneId::new("Sign-in rework")),
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        w.launch(&GroupId::new(group), role, &issue, None)
            .expect("known")
            .brief
            .model
    }

    /// A person's change of `scope`'s runtime for `role`.
    fn persons_runtime(scope: Scope, role: Option<Role>, runtime: &str) -> Event {
        Event::SettingSet {
            scope,
            role,
            key: "runtime".into(),
            value: Value::Text(runtime.into()),
            by: temet_work_core::command::PERSON.into(),
        }
    }

    #[test]
    fn a_plain_model_where_the_file_gives_another_runtime_stops_the_start_naming_it() {
        let world = two_runtimes();
        let reviewers_beta = "[settings.reviewer]\nruntime = \"beta\"\n";
        for text in [
            // The issue's example, with the stand-ins.
            format!("{reviewers_beta}model = \"gpt-x\"\n{TODAY}"),
            format!(
                "{reviewers_beta}{TODAY}\n[[project]]\nid = \"ledger\"\ntracker = \"github\"\n\
                 name = \"temet-ai/ledger\"\n\n[project.settings.reviewer]\nmodel = \"gpt-x\"\n"
            ),
        ] {
            let config = Config::parse(&text).expect("the file is well formed");
            let err = config.check_build(&world).expect_err("refused");
            assert!(err.contains("model.beta"), "{err}");
            assert!(err.contains("model]"), "names the model table: {err}");
        }
        // Named for its runtime, the model loads and reaches the beta reviewer.
        let text =
            format!("{reviewers_beta}\n[settings.reviewer.model]\nbeta = \"gpt-x\"\n{TODAY}");
        let config = Config::parse(&text).expect("parse");
        assert!(config.check_build(&world).is_ok());
        let (w, _) = two_runtimes().replay_onto(&config.events(&world));
        let brief = |role| brief_model(&w, "group-1", "radar", role);
        assert_eq!(brief(Role::Reviewer).as_deref(), Some("gpt-x"));
        // The group's old model is alpha's, the build's default.
        assert_eq!(brief(Role::Author).as_deref(), Some("haiku"));
    }

    #[test]
    fn a_plain_model_where_a_person_gave_another_runtime_stops_the_start_naming_it() {
        let ledger = Scope::Project(ProjectId::new("ledger"));
        for (person, text, table) in [
            (
                Scope::Global,
                format!("[settings.reviewer]\nmodel = \"gpt-x\"\n{TODAY}"),
                "[settings.reviewer.model]",
            ),
            (
                ledger.clone(),
                format!(
                    "{TODAY}\n[[project]]\nid = \"ledger\"\ntracker = \"github\"\n\
                     name = \"temet-ai/ledger\"\n\n[project.settings.reviewer]\nmodel = \"gpt-x\"\n"
                ),
                "[project.settings.reviewer.model] of ledger",
            ),
        ] {
            let config = Config::parse(&text).expect("the file is well formed");
            assert!(config.check_build(&two_runtimes()).is_ok(), "{table}");
            // `temet-work set global --role reviewer runtime beta`, or `set ledger`.
            let (world, _) = two_runtimes().replay_onto(&[persons_runtime(
                person,
                Some(Role::Reviewer),
                "beta",
            )]);
            let err = config.check_build(&world).expect_err("refused");
            assert!(
                err.contains("model.beta")
                    && err.contains(table)
                    && err.contains("a person set the reviewers' runtime to beta"),
                "{err}"
            );
        }
        // Named for its runtime, the model loads and reaches the beta reviewer.
        let (world, _) = two_runtimes().replay_onto(&[persons_runtime(
            Scope::Global,
            Some(Role::Reviewer),
            "beta",
        )]);
        let text = format!("[settings.reviewer.model]\nbeta = \"gpt-x\"\n{TODAY}");
        let config = Config::parse(&text).expect("parse");
        assert!(config.check_build(&world).is_ok());
        let (w, _) = world.clone().replay_onto(&config.events(&world));
        assert_eq!(
            brief_model(&w, "group-1", "radar", Role::Reviewer).as_deref(),
            Some("gpt-x")
        );
    }

    /// `TODAY` without its group's `model`, so a global model reaches radar's
    /// agents.
    fn today_without_a_group_model() -> String {
        TODAY.replace("model = \"haiku\"\n", "")
    }

    #[test]
    fn a_runtime_only_an_earlier_file_set_does_not_stop_the_start() {
        let today = today_without_a_group_model();
        let earlier = Config::parse(&format!("[settings.reviewer]\nruntime = \"beta\"\n{today}"))
            .expect("parse");
        let (world, _) = two_runtimes().replay_onto(&earlier.events(&two_runtimes()));
        let reviewers = |w: &World| w.text_at(&Scope::Global, Some(Role::Reviewer), "runtime");
        assert_eq!(reviewers(&world).as_deref(), Some("beta"));
        let now = Config::parse(&format!("[settings.reviewer]\nmodel = \"gpt-x\"\n{today}"))
            .expect("parse");
        now.check_build(&world)
            .expect("the earlier file's runtime is unset");
        let (w, _) = world.clone().replay_onto(&now.events(&world));
        assert_eq!(reviewers(&w).as_deref(), Some("alpha"));
        assert_eq!(
            w.settings[&(Scope::Global, Some(Role::Reviewer), "model.alpha".into())].value,
            Value::Text("gpt-x".into())
        );
        assert_eq!(
            brief_model(&w, "group-1", "radar", Role::Reviewer).as_deref(),
            Some("gpt-x")
        );
    }

    #[test]
    fn a_groups_old_model_over_a_project_on_another_runtime_starts_bound_to_the_global_one() {
        let text = r#"
[[project]]
id = "ledger"
tracker = "github"
name = "temet-ai/ledger"

[project.settings]
runtime = "beta"

[[folder]]
id = "ledger-1"
project = "ledger"
path = "/home/me/src/ledger-1"

[[group]]
id = "group-2"
projects = ["ledger"]
milestones = ["Sign-in rework"]
model = "sonnet"
"#;
        let ledger = || Scope::Project(ProjectId::new("ledger"));
        let by_person = || {
            two_runtimes()
                .replay_onto(&[persons_runtime(ledger(), None, "beta")])
                .0
        };
        // The file's runtime, the person's and the file's, and the person's alone.
        let without_runtime = text.replace("[project.settings]\nruntime = \"beta\"\n", "");
        for (text, world) in [
            (text.to_string(), two_runtimes()),
            (text.to_string(), by_person()),
            (without_runtime, by_person()),
        ] {
            let config = Config::parse(&text).expect("parse");
            config.check_build(&world).expect("starts");
            let (w, _) = world.clone().replay_onto(&config.events(&world));
            assert_eq!(
                w.settings[&(ledger(), None, "model.alpha".into())].value,
                Value::Text("sonnet".into())
            );
            assert_eq!(
                w.text_at(&ledger(), Some(Role::Author), "runtime")
                    .as_deref(),
                Some("beta")
            );
            assert_eq!(brief_model(&w, "group-2", "ledger", Role::Author), None);
        }
    }

    #[test]
    fn a_plain_model_in_the_file_is_bound_by_the_file_alone() {
        let alpha = format!(
            "[settings]\nmodel = \"sonnet\"\n\n[settings.reviewer]\nmodel = \"opus\"\n{}",
            today_without_a_group_model()
        );
        let config = Config::parse(&alpha).expect("parse");
        let (mut w, _) = two_runtimes().replay_onto(&config.events(&two_runtimes()));
        let global = |key: &str| (Scope::Global, None, key.to_string());
        let reviewers = |key: &str| (Scope::Global, Some(Role::Reviewer), key.to_string());
        assert_eq!(
            w.settings[&global("model.alpha")].value,
            Value::Text("sonnet".into())
        );
        // A person moves the global runtime, and the daemon restarts on the same
        // file: it starts, and the file's models stay alpha's.
        w.apply(&persons_runtime(Scope::Global, None, "beta"));
        config
            .check_build(&w)
            .expect("a moved global runtime starts");
        assert_eq!(config.events(&w), vec![], "no model.beta, nothing unset");
        assert!(w.settings.contains_key(&global("model.alpha")));
        assert!(!w.settings.contains_key(&global("model.beta")));
        assert_eq!(
            w.settings[&reviewers("model.alpha")].value,
            Value::Text("opus".into())
        );
        // The person puts the reviewers back on alpha: they read the file's opus.
        let mut back = w.clone();
        back.apply(&persons_runtime(
            Scope::Global,
            Some(Role::Reviewer),
            "alpha",
        ));
        config.check_build(&back).expect("reviewers on alpha start");
        assert_eq!(
            brief_model(&back, "group-1", "radar", Role::Reviewer).as_deref(),
            Some("opus")
        );
        // The file's own runtime becomes beta: the model is rebound.
        let beta =
            Config::parse(&alpha.replace("[settings]\n", "[settings]\nruntime = \"beta\"\n"))
                .expect("parse");
        let events = beta.events(&w);
        assert!(
            events.contains(&Event::SettingSet {
                scope: Scope::Global,
                role: None,
                key: "model.beta".into(),
                value: Value::Text("sonnet".into()),
                by: FILE.into(),
            }),
            "{events:?}"
        );
        assert!(
            events.contains(&Event::SettingUnset {
                scope: Scope::Global,
                role: None,
                key: "model.alpha".into(),
                by: FILE.into(),
            }),
            "{events:?}"
        );
        for event in &events {
            w.apply(event);
        }
        assert_eq!(
            w.settings[&global("model.beta")].value,
            Value::Text("sonnet".into())
        );
        assert!(!w.settings.contains_key(&global("model.alpha")));
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
            w.settings[&(Scope::Global, None, "model.alpha".to_string())].value,
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
    fn a_milestone_table_refuses_a_setting_no_milestone_reads() {
        let milestone = "[[milestone]]\nof = \"radar\"\nname = \"Clocks\"\n";
        let area = "[[area]]\nid = \"guild\"\nprojects = [\"radar\"]\n";
        for (key, line) in [
            ("sync_every", "sync_every = 7"),
            ("investigate", "investigate = \"the clock tower\""),
        ] {
            for table in ["[milestone.settings]", "[milestone.settings.reviewer]"] {
                let text = format!("{TODAY}\n{milestone}{table}\n{line}\n");
                let why = Config::parse(&text).expect_err(table);
                assert_eq!(
                    why,
                    format!(
                        "[milestone.settings] of radar:Clocks: {key} is not read at the \
                         milestone level, so a value there would have no effect; the levels \
                         that read it are: project, area, global"
                    )
                );
            }
            // The same key at each level of the file that reads it.
            let global = format!("[settings]\n{line}\n{TODAY}");
            let project = TODAY.replace(
                "ci = false\n",
                &format!("ci = false\n[project.settings]\n{line}\n"),
            );
            let in_area = format!("{TODAY}\n{area}[area.settings]\n{line}\n");
            for (scope, text) in [
                (Scope::Global, global),
                (Scope::Project(ProjectId::new("radar")), project),
                (Scope::Area(ProjectId::new("guild")), in_area),
            ] {
                let config = Config::parse(&text).expect("a level that reads it");
                let events = config.events(&World::default());
                assert!(
                    events.iter().any(|e| matches!(
                        e,
                        Event::SettingSet { scope: at, key: k, by, .. }
                            if *at == scope && k == key && by == FILE
                    )),
                    "{key} at {scope}: {events:?}"
                );
            }
        }
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
