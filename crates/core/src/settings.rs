//! Settings at every level, from global down to one task (`docs/settings.md`).
//!
//! Each level's values are built from `SettingSet` and `SettingUnset` events, and
//! one resolver answers "what is this setting for this issue and role, and which
//! level set it". Nothing here reads a file or a clock.

use crate::command::PERSON;
use crate::ids::{IssueId, ProjectId};
use crate::model::Role;
use crate::world::World;

/// The value of a setting.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// A yes or no.
    Flag(bool),
    /// A count, or a time in the setting's own unit.
    Number(u64),
    /// A name or a command.
    Text(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flag(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Text(t) => f.write_str(t),
        }
    }
}

/// What a setting applies to: one place in the chain.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum Scope {
    /// Everything.
    Global,
    /// An area: its projects and its own issues.
    Area(ProjectId),
    /// One repository.
    Project(ProjectId),
    /// The issues of one milestone of a project or area (`of`).
    Milestone {
        /// The project or area whose tracker names the milestone.
        of: ProjectId,
        /// The milestone's name.
        name: String,
    },
    /// One issue.
    Task(IssueId),
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => f.write_str("global"),
            Self::Area(id) | Self::Project(id) => write!(f, "{id}"),
            Self::Milestone { of, name } => write!(f, "{of}:{name}"),
            Self::Task(issue) => write!(f, "{issue}"),
        }
    }
}

/// A place in the chain, from the nearest to the widest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// One issue.
    Task,
    /// A milestone.
    Milestone,
    /// A project.
    Project,
    /// An area.
    Area,
    /// Everything.
    Global,
    /// The value in the code, when no level sets one.
    BuiltIn,
}

impl Level {
    /// The level in words.
    pub fn name(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Milestone => "milestone",
            Self::Project => "project",
            Self::Area => "area",
            Self::Global => "global",
            Self::BuiltIn => "built in",
        }
    }
}

/// Who a setting's change is recorded as made by when it comes from the
/// configuration file.
pub const FILE: &str = "file";

/// A value kept at one scope, with who set it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The value.
    pub value: Value,
    /// Who set it: `file`, `person`, or an agent's id.
    pub by: String,
}

/// Where a setting's key is kept: its scope, the role it is for (none for every
/// role), and its name.
pub type Key = (Scope, Option<Role>, String);

/// A setting as the chain answers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved {
    /// Its value.
    pub value: Value,
    /// The level that set it.
    pub level: Level,
    /// The scope that set it, when a level did.
    pub scope: Option<Scope>,
    /// Who set it, when a level did.
    pub by: Option<String>,
}

impl Resolved {
    /// The level in words, with who set it: `task, set by refiner-3`, `global`,
    /// `built in`. A value from the file does not name the file.
    pub fn source(&self) -> String {
        match self.by.as_deref() {
            Some(by) if by != FILE => format!("{}, set by {by}", self.level.name()),
            _ => self.level.name().to_string(),
        }
    }
}

/// What a family of adapters offers for one setting, so a value naming an
/// adapter this build does not have is refused. Parts report it; the host puts
/// it in the world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    /// The setting (`runtime`, `runner`).
    pub key: String,
    /// The ids of the adapters in this build.
    pub allowed: Vec<String>,
    /// The one used when no level sets the setting; none when the build has no
    /// default one.
    pub default: Option<String>,
}

/// What kind of value a setting takes.
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    /// A word or a command.
    Text,
    /// One of these words.
    OneOf(&'static [&'static str]),
    /// A whole number, at least this.
    Count(u64),
    /// `true` or `false`.
    Flag,
}

/// Whether an agent may set and unset a setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agents {
    /// A refiner may, for the issue it is refining, while it is refining.
    May,
    /// A refiner may in the same way, for the author only: its value is
    /// recorded for the author, and the other roles' are a person's to set.
    ForAuthor,
    /// Only a person does; why, in words that follow "because".
    MayNot(&'static str),
}

/// One setting the daemon knows.
#[derive(Clone, Copy, Debug)]
pub struct Spec {
    /// Its name.
    pub key: &'static str,
    /// What it takes.
    pub kind: Kind,
    /// What it means, in a line, with its unit.
    pub about: &'static str,
    /// Whether an agent may set it.
    pub agents: Agents,
}

/// The modes an agent can be started in, which `permission` takes one of.
const PERMISSIONS: &[&str] = &[
    "default",
    "acceptEdits",
    "plan",
    "auto",
    "dontAsk",
    "bypassPermissions",
];

/// Every setting, in the order they are listed.
pub const SPECS: [Spec; 16] = [
    Spec {
        key: "runtime",
        kind: Kind::Text,
        about: "the agent runtime an agent runs",
        agents: Agents::MayNot(
            "a runtime may not follow permission, so the runtime decides what an agent \
             may do without asking",
        ),
    },
    Spec {
        key: "model",
        kind: Kind::Text,
        about: "the model an agent runs; the runtime's own default when none is set",
        agents: Agents::ForAuthor,
    },
    Spec {
        key: "runner",
        kind: Kind::Text,
        about: "where an agent's terminal runs",
        agents: Agents::MayNot(
            "it follows from the person's machines, not from the issue, and a value for \
             one issue replaces the one a person chose for a role",
        ),
    },
    Spec {
        key: "machine",
        kind: Kind::Text,
        about: "the machine of the work folder an agent is given; any machine when none is set",
        agents: Agents::May,
    },
    Spec {
        key: "permission",
        kind: Kind::OneOf(PERMISSIONS),
        about: "how an agent's runtime asks about what it may do",
        agents: Agents::MayNot(
            "it decides what every later agent on the issue may do without asking",
        ),
    },
    Spec {
        key: "review_rounds",
        kind: Kind::Count(1),
        about: "review rounds before the arbiter rules",
        agents: Agents::MayNot(
            "it decides how many rounds of review an issue gets before the arbiter rules, \
             so fewer rounds weaken its review",
        ),
    },
    Spec {
        key: "ci",
        kind: Kind::Flag,
        about: "whether pull requests run CI; false leaves the plan's checks to the reviewer",
        agents: Agents::MayNot(
            "ci false takes the CI gate off the code the issue's own author writes, \
             in every repository the issue changes",
        ),
    },
    Spec {
        key: "deploy",
        kind: Kind::Text,
        about: "the command run after a merge",
        agents: Agents::MayNot("the daemon runs it as a shell command after a merge"),
    },
    Spec {
        key: "verify",
        kind: Kind::Text,
        about: "the command run after a deploy",
        agents: Agents::MayNot("the daemon runs it after a deploy"),
    },
    Spec {
        key: "run_in",
        kind: Kind::Text,
        about: "the folder deploy and verify run in",
        agents: Agents::MayNot("it is the folder deploy and verify run in"),
    },
    Spec {
        key: "merge",
        kind: Kind::OneOf(&["squash", "merge", "rebase"]),
        about: "how the code host merges a pull request",
        agents: Agents::May,
    },
    Spec {
        key: "remind_after",
        kind: Kind::Count(1),
        about: "minutes an agent its issue waits on may sit idle before it is reminded",
        agents: Agents::May,
    },
    Spec {
        key: "reminders",
        kind: Kind::Count(0),
        about: "reminders before an idle agent is stopped",
        agents: Agents::May,
    },
    Spec {
        key: "starts",
        kind: Kind::Count(1),
        about: "agents of one role started on one issue before the loop gives up",
        agents: Agents::May,
    },
    Spec {
        key: "sync_every",
        kind: Kind::Count(1),
        about: "seconds between syncs of a project with its tracker",
        agents: Agents::May,
    },
    Spec {
        key: "ci_every",
        kind: Kind::Count(1),
        about: "seconds between reads of a pull request's CI",
        agents: Agents::May,
    },
];

/// Every setting the daemon knows.
pub fn specs() -> impl Iterator<Item = &'static Spec> {
    SPECS.iter()
}

/// The setting called `key`.
pub fn spec(key: &str) -> Result<&'static Spec, String> {
    specs().find(|s| s.key == key).ok_or_else(|| {
        let keys: Vec<&str> = specs().map(|s| s.key).collect();
        format!("no setting {key}; the settings are: {}", keys.join(", "))
    })
}

/// The value in the code for `key`, used when no level sets one. The runtime and
/// the runner have none here: the build's own default is in its [`Choice`].
pub fn builtin(key: &str) -> Option<Value> {
    Some(match key {
        "permission" => Value::Text("auto".into()),
        "review_rounds" => Value::Number(3),
        "ci" => Value::Flag(true),
        "merge" => Value::Text("squash".into()),
        "remind_after" => Value::Number(10),
        "reminders" => Value::Number(2),
        "starts" => Value::Number(3),
        "sync_every" => Value::Number(60),
        "ci_every" => Value::Number(30),
        _ => return None,
    })
}

/// Checks that `value` is the kind `key` takes.
pub fn check_value(key: &str, value: &Value) -> Result<(), String> {
    let spec = spec(key)?;
    let wrong = |wanted: &str| Err(format!("{key} takes {wanted}, not {value}"));
    match (spec.kind, value) {
        (Kind::Flag, Value::Flag(_)) => Ok(()),
        (Kind::Flag, _) => wrong("true or false"),
        (Kind::Count(min), Value::Number(n)) if *n >= min => Ok(()),
        (Kind::Count(min), _) => wrong(&format!("a whole number, at least {min}")),
        (Kind::OneOf(words), Value::Text(t)) if words.contains(&t.as_str()) => Ok(()),
        (Kind::OneOf(words), _) => wrong(&format!("one of {}", words.join(", "))),
        (Kind::Text, Value::Text(t)) if !t.trim().is_empty() => Ok(()),
        (Kind::Text, _) => wrong("a word or a command"),
    }
}

/// Whether `by`, as a change records it, is an agent.
fn is_agent(by: &str) -> bool {
    by != FILE && by != PERSON
}

/// Whether an agent may set `key` for the author only.
fn for_author(key: &str) -> bool {
    spec(key).is_ok_and(|s| s.agents == Agents::ForAuthor)
}

/// The role a change of `key` by `by` is recorded for, when it names `role`: an
/// agent's change of a key it may set for the author only is the author's.
pub(crate) fn recorded_role(role: Option<Role>, key: &str, by: &str) -> Option<Role> {
    if is_agent(by) && for_author(key) {
        Some(Role::Author)
    } else {
        role
    }
}

/// The role the loop keeps a logged change under, or `None` when it leaves the
/// change out. Logs from before agents were refused hold changes an agent made
/// for one issue that it may no longer make: of a setting only a person sets,
/// left out, and of a setting that is now the author's only, for every role
/// (read as the author's) or for another role (left out).
pub(crate) fn kept_role(
    scope: &Scope,
    role: Option<Role>,
    key: &str,
    by: &str,
) -> Option<Option<Role>> {
    if !matches!(scope, Scope::Task(_)) || !is_agent(by) {
        return Some(role);
    }
    match spec(key).map(|s| s.agents) {
        Ok(Agents::MayNot(_)) => None,
        Ok(Agents::ForAuthor) if !matches!(role, None | Some(Role::Author)) => None,
        _ => Some(recorded_role(role, key, by)),
    }
}

/// Reads what was typed as the value of `key`.
pub fn parse_value(key: &str, raw: &str) -> Result<Value, String> {
    let value = match spec(key)?.kind {
        Kind::Flag => match raw {
            "true" => Value::Flag(true),
            "false" => Value::Flag(false),
            _ => return check_value(key, &Value::Text(raw.into())).map(|()| Value::Flag(false)),
        },
        Kind::Count(_) => raw
            .parse::<u64>()
            .map(Value::Number)
            .unwrap_or_else(|_| Value::Text(raw.into())),
        Kind::OneOf(_) | Kind::Text => Value::Text(raw.into()),
    };
    check_value(key, &value)?;
    Ok(value)
}

impl World {
    /// Refuses a value that names an adapter this build does not have, and says
    /// which ones it has.
    pub fn check_choice(&self, key: &str, value: &Value) -> Result<(), String> {
        let Some(choice) = self.catalog.get(key) else {
            return Ok(());
        };
        match value {
            Value::Text(t) if choice.allowed.iter().any(|a| a == t) => Ok(()),
            _ => Err(format!(
                "{key} \"{value}\" is not in this build; this build has: {}",
                if choice.allowed.is_empty() {
                    "none".to_string()
                } else {
                    choice.allowed.join(", ")
                }
            )),
        }
    }

    /// What a build that cannot start says: a family with no default adapter and
    /// no global value for its setting. Empty when it can start.
    pub fn unready(&self) -> Vec<String> {
        self.catalog
            .values()
            .filter(|c| c.default.is_none())
            .filter(|c| {
                !self
                    .settings
                    .contains_key(&(Scope::Global, None, c.key.clone()))
            })
            .map(|c| {
                if c.allowed.is_empty() {
                    format!("this build has no {}", c.key)
                } else {
                    format!(
                        "this build has no default {key}: set one, in [settings] of the \
                         configuration or with temet-work set global {key} VALUE; \
                         this build has: {}",
                        c.allowed.join(", "),
                        key = c.key
                    )
                }
            })
            .collect()
    }

    /// The scopes a setting is looked up in for `scope`, nearest first. A task
    /// reads its milestone, the repository it is worked in, that repository's area
    /// (or its own, for an area's issue) and global.
    pub fn chain(&self, scope: &Scope) -> Vec<(Level, Scope)> {
        let mut out = Vec::new();
        let area_of = |project: &ProjectId| {
            self.areas
                .values()
                .find(|a| a.projects.contains(project))
                .map(|a| a.id.clone())
        };
        match scope {
            Scope::Task(id) => {
                out.push((Level::Task, scope.clone()));
                let issue = self.issues.get(id);
                if let Some(m) = issue.and_then(|i| i.milestone.as_ref()) {
                    out.push((
                        Level::Milestone,
                        Scope::Milestone {
                            of: id.project.clone(),
                            name: m.0.clone(),
                        },
                    ));
                }
                let repo = issue
                    .and_then(|i| self.repo_of(i))
                    .map(|p| p.id.clone())
                    .or_else(|| {
                        self.projects
                            .contains_key(&id.project)
                            .then(|| id.project.clone())
                    });
                if let Some(repo) = &repo {
                    out.push((Level::Project, Scope::Project(repo.clone())));
                }
                let area = if self.areas.contains_key(&id.project) {
                    Some(id.project.clone())
                } else {
                    repo.as_ref().and_then(area_of)
                };
                if let Some(a) = area {
                    out.push((Level::Area, Scope::Area(a)));
                }
            }
            Scope::Milestone { of, .. } => {
                out.push((Level::Milestone, scope.clone()));
                if self.areas.contains_key(of) {
                    out.push((Level::Area, Scope::Area(of.clone())));
                } else {
                    out.push((Level::Project, Scope::Project(of.clone())));
                    if let Some(a) = area_of(of) {
                        out.push((Level::Area, Scope::Area(a)));
                    }
                }
            }
            Scope::Project(p) => {
                out.push((Level::Project, scope.clone()));
                if let Some(a) = area_of(p) {
                    out.push((Level::Area, Scope::Area(a)));
                }
            }
            Scope::Area(_) => out.push((Level::Area, scope.clone())),
            Scope::Global => {}
        }
        out.push((Level::Global, Scope::Global));
        out
    }

    /// The setting `key` over `scope`'s chain for `role` (none: the values for
    /// every role only), and which level set it. Within a level the role's own
    /// value comes before the value for every role; across levels the nearer one
    /// comes first. The build's own value is the last resort.
    pub fn resolve(&self, scope: &Scope, role: Option<Role>, key: &str) -> Option<Resolved> {
        for (level, at) in self.chain(scope) {
            let own = role.and_then(|r| self.settings.get(&(at.clone(), Some(r), key.to_string())));
            let entry = own.or_else(|| self.settings.get(&(at.clone(), None, key.to_string())));
            if let Some(e) = entry {
                return Some(Resolved {
                    value: e.value.clone(),
                    level,
                    scope: Some(at),
                    by: Some(e.by.clone()),
                });
            }
        }
        let value = match self.catalog.get(key) {
            Some(c) => c.default.clone().map(Value::Text),
            None => builtin(key),
        }
        .or_else(|| builtin(key))?;
        Some(Resolved {
            value,
            level: Level::BuiltIn,
            scope: None,
            by: None,
        })
    }

    /// The setting for an issue and role.
    pub fn setting(&self, issue: &IssueId, role: Option<Role>, key: &str) -> Option<Resolved> {
        self.resolve(&Scope::Task(issue.clone()), role, key)
    }

    /// The setting as text, when it has a value.
    pub fn setting_text(&self, issue: &IssueId, role: Option<Role>, key: &str) -> Option<String> {
        self.setting(issue, role, key).map(|r| r.value.to_string())
    }

    /// The setting as a whole number; the built-in one when a level holds a value of
    /// another kind.
    pub fn setting_number(&self, issue: &IssueId, role: Option<Role>, key: &str) -> u64 {
        match self.setting(issue, role, key).map(|r| r.value) {
            Some(Value::Number(n)) => n,
            _ => match builtin(key) {
                Some(Value::Number(n)) => n,
                _ => 0,
            },
        }
    }

    /// The setting as a yes or no; the built-in one when a level holds a value of
    /// another kind.
    pub fn setting_flag(&self, issue: &IssueId, role: Option<Role>, key: &str) -> bool {
        match self.setting(issue, role, key).map(|r| r.value) {
            Some(Value::Flag(b)) => b,
            _ => matches!(builtin(key), Some(Value::Flag(true))),
        }
    }

    /// A yes-or-no setting for one of the repositories an issue changes: the
    /// task's and the milestone's values come first, then that repository's.
    pub fn repo_flag(&self, issue: &IssueId, repo: &ProjectId, key: &str) -> bool {
        let own = self
            .chain(&Scope::Task(issue.clone()))
            .into_iter()
            .filter(|(level, _)| matches!(level, Level::Task | Level::Milestone))
            .find_map(|(_, at)| self.settings.get(&(at, None, key.to_string())))
            .map(|e| e.value.clone());
        let value = own.or_else(|| {
            self.resolve(&Scope::Project(repo.clone()), None, key)
                .map(|r| r.value)
        });
        match value {
            Some(Value::Flag(b)) => b,
            _ => matches!(builtin(key), Some(Value::Flag(true))),
        }
    }

    /// The setting for a project or an area as a source of issues, such as how
    /// often it is synced.
    pub fn source_number(&self, source: &ProjectId, key: &str) -> u64 {
        let scope = if self.areas.contains_key(source) {
            Scope::Area(source.clone())
        } else {
            Scope::Project(source.clone())
        };
        match self.resolve(&scope, None, key).map(|r| r.value) {
            Some(Value::Number(n)) => n,
            _ => match builtin(key) {
                Some(Value::Number(n)) => n,
                _ => 0,
            },
        }
    }

    /// Every setting in effect over `scope` for `role`, in the order of [`specs`].
    pub fn in_effect(&self, scope: &Scope, role: Option<Role>) -> Vec<(&'static str, Resolved)> {
        specs()
            .filter_map(|s| self.resolve(scope, role, s.key).map(|r| (s.key, r)))
            .collect()
    }

    /// Whether the author's value of `key` at `scope` is an agent's, for a key an
    /// agent may set for the author only. A person's change for every role at that
    /// scope overturns it: within a level the author's value would win otherwise.
    pub(crate) fn agents_author_value(&self, scope: &Scope, key: &str) -> bool {
        matches!(scope, Scope::Task(_))
            && for_author(key)
            && self
                .settings
                .get(&(scope.clone(), Some(Role::Author), key.to_string()))
                .is_some_and(|e| is_agent(&e.by))
    }

    /// Drops an agent's value for the author when a change by a person or the
    /// file, for every role, overturns it.
    pub(crate) fn overturn_agents_value(
        &mut self,
        scope: &Scope,
        role: Option<Role>,
        key: &str,
        by: &str,
    ) {
        if role.is_none() && !is_agent(by) && self.agents_author_value(scope, key) {
            self.settings
                .remove(&(scope.clone(), Some(Role::Author), key.to_string()));
        }
    }

    /// Records a value as `by`, as `SettingSet` does.
    pub(crate) fn put(
        &mut self,
        scope: &Scope,
        role: Option<Role>,
        key: &str,
        value: &Value,
        by: &str,
    ) {
        self.settings.insert(
            (scope.clone(), role, key.to_string()),
            Entry {
                value: value.clone(),
                by: by.to_string(),
            },
        );
    }
}

/// The role an agent works as at each stage of an issue, for showing the
/// settings that apply to it; none once the issue is past its agents.
pub fn role_at(stage: crate::model::Stage) -> Option<Role> {
    use crate::model::Stage;
    match stage {
        Stage::New | Stage::Refining => Some(Role::Refiner),
        Stage::Ready | Stage::Working | Stage::Fixing { .. } => Some(Role::Author),
        Stage::InReview { .. } => Some(Role::Reviewer),
        Stage::Arbitration { .. } => Some(Role::Arbiter),
        Stage::Merging | Stage::Deploying | Stage::Verifying | Stage::Done => None,
    }
}

#[cfg(test)]
mod level {
    use super::*;
    use crate::event::Event;
    use crate::ids::MilestoneId;

    pub(super) fn set(scope: Scope, role: Option<Role>, key: &str, value: &str, by: &str) -> Event {
        Event::SettingSet {
            scope,
            role,
            key: key.into(),
            value: Value::Text(value.into()),
            by: by.into(),
        }
    }

    pub(super) fn world(events: &[Event]) -> World {
        let (mut w, _) = World::replay(events);
        w.catalog.clear();
        w
    }

    /// radar#44 in milestone "Sign-in" of radar, which is in the area guild.
    pub(super) fn base() -> Vec<Event> {
        vec![
            Event::ProjectAdded {
                project: ProjectId::new("radar"),
                tracker: "t".into(),
                name: "city/radar".into(),
                code_host: None,
                repo: None,
            },
            Event::AreaDefined {
                area: ProjectId::new("guild"),
                name: "Guild".into(),
                projects: vec![ProjectId::new("radar")],
                tracker: String::new(),
                tracker_name: String::new(),
            },
            Event::IssueSynced {
                issue: IssueId::new("radar", "44"),
                title: "t".into(),
                milestone: Some(MilestoneId::new("Sign-in")),
                depends_on: vec![],
                order: 1,
                closed: false,
            },
        ]
    }

    fn at(w: &World, role: Option<Role>) -> Option<Resolved> {
        w.setting(&IssueId::new("radar", "44"), role, "model")
    }

    #[test]
    fn nothing_set_falls_to_the_built_in_value_or_none() {
        let w = world(&base());
        assert_eq!(at(&w, None), None, "the model has no built-in value");
        let r = w
            .setting(&IssueId::new("radar", "44"), None, "review_rounds")
            .expect("built in");
        assert_eq!(
            (r.value.clone(), r.level),
            (Value::Number(3), Level::BuiltIn)
        );
        assert_eq!(r.source(), "built in");
    }

    #[test]
    fn machine_takes_a_name_and_refuses_an_empty_one() {
        for empty in ["", "  "] {
            let refused = parse_value("machine", empty);
            assert!(
                refused
                    .as_ref()
                    .is_err_and(|e| e.starts_with("machine takes")),
                "{refused:?}"
            );
            assert!(check_value("machine", &Value::Text(empty.into())).is_err());
        }
        assert!(check_value("machine", &Value::Number(2)).is_err());
        // Any name: a folder on it may be listed later.
        assert_eq!(
            parse_value("machine", "mac2"),
            Ok(Value::Text("mac2".into()))
        );
        assert_eq!(builtin("machine"), None);
    }

    #[test]
    fn each_level_answers_when_it_alone_sets_the_value() {
        let cases = [
            (Scope::Global, Level::Global),
            (Scope::Area(ProjectId::new("guild")), Level::Area),
            (Scope::Project(ProjectId::new("radar")), Level::Project),
            (
                Scope::Milestone {
                    of: ProjectId::new("radar"),
                    name: "Sign-in".into(),
                },
                Level::Milestone,
            ),
            (Scope::Task(IssueId::new("radar", "44")), Level::Task),
        ];
        for (scope, level) in cases {
            let mut events = base();
            events.push(set(scope, None, "model", "opus", "person"));
            let r = at(&world(&events), None).expect("set");
            assert_eq!(r.level, level);
            assert_eq!(r.value, Value::Text("opus".into()));
            assert_eq!(r.by.as_deref(), Some("person"));
        }
    }

    #[test]
    fn the_nearer_level_wins() {
        let mut events = base();
        for (scope, v) in [
            (Scope::Global, "global"),
            (Scope::Area(ProjectId::new("guild")), "area"),
            (Scope::Project(ProjectId::new("radar")), "project"),
            (
                Scope::Milestone {
                    of: ProjectId::new("radar"),
                    name: "Sign-in".into(),
                },
                "milestone",
            ),
            (Scope::Task(IssueId::new("radar", "44")), "task"),
        ] {
            events.push(set(scope, None, "model", v, "person"));
            assert_eq!(
                at(&world(&events), None).expect("set").value,
                Value::Text(v.into()),
                "{v} is nearer than what came before"
            );
        }
    }

    #[test]
    fn within_a_level_the_role_comes_before_every_role() {
        let mut events = base();
        events.push(set(Scope::Global, None, "model", "everyone", "p"));
        events.push(set(
            Scope::Global,
            Some(Role::Reviewer),
            "model",
            "reviewers",
            "p",
        ));
        let w = world(&events);
        let text = |role| at(&w, role).map(|r| r.value.to_string());
        assert_eq!(text(Some(Role::Reviewer)).as_deref(), Some("reviewers"));
        assert_eq!(text(Some(Role::Author)).as_deref(), Some("everyone"));
        assert_eq!(text(None).as_deref(), Some("everyone"));
    }

    #[test]
    fn a_project_value_for_everyone_beats_an_area_value_for_reviewers() {
        let mut events = base();
        events.push(set(
            Scope::Area(ProjectId::new("guild")),
            Some(Role::Reviewer),
            "runtime",
            "beta",
            "p",
        ));
        events.push(set(
            Scope::Project(ProjectId::new("radar")),
            None,
            "runtime",
            "alpha",
            "p",
        ));
        let w = world(&events);
        let r = w
            .setting(
                &IssueId::new("radar", "44"),
                Some(Role::Reviewer),
                "runtime",
            )
            .expect("set");
        assert_eq!(
            (r.value, r.level),
            (Value::Text("alpha".into()), Level::Project)
        );
        // The project names its reviewers too, and then they run the other one.
        events.push(set(
            Scope::Project(ProjectId::new("radar")),
            Some(Role::Reviewer),
            "runtime",
            "beta",
            "p",
        ));
        let r = world(&events)
            .setting(
                &IssueId::new("radar", "44"),
                Some(Role::Reviewer),
                "runtime",
            )
            .expect("set");
        assert_eq!(r.value, Value::Text("beta".into()));
    }

    #[test]
    fn only_a_person_sets_ci_deploy_permission_review_rounds_run_in_runner_runtime_and_verify() {
        let mut keys: Vec<&str> = specs()
            .filter(|s| matches!(s.agents, Agents::MayNot(_)))
            .map(|s| s.key)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "ci",
                "deploy",
                "permission",
                "review_rounds",
                "run_in",
                "runner",
                "runtime",
                "verify",
            ]
        );
    }

    #[test]
    fn permission_takes_one_of_the_modes() {
        let modes = [
            "default",
            "acceptEdits",
            "plan",
            "auto",
            "dontAsk",
            "bypassPermissions",
        ];
        for mode in modes {
            assert_eq!(
                parse_value("permission", mode),
                Ok(Value::Text(mode.into()))
            );
        }
        let set = crate::command::Command::Set {
            target: Target::parse("global"),
            role: None,
            key: "permission".into(),
            value: "yolo".into(),
        };
        assert_eq!(
            world(&base()).events(None, &set),
            Err(format!(
                "permission takes one of {}, not yolo",
                modes.join(", ")
            ))
        );
        assert_eq!(builtin("permission"), Some(Value::Text("auto".into())));
    }

    #[test]
    fn a_value_of_the_wrong_kind_is_refused_with_the_reason() {
        assert!(parse_value("review_rounds", "many").is_err_and(|e| e.contains("whole number")));
        assert!(parse_value("review_rounds", "0").is_err());
        assert!(parse_value("ci", "maybe").is_err_and(|e| e.contains("true or false")));
        assert!(parse_value("merge", "fast-forward").is_err_and(|e| e.contains("squash")));
        assert_eq!(parse_value("ci", "false"), Ok(Value::Flag(false)));
        assert!(parse_value("colour", "red").is_err_and(|e| e.contains("no setting colour")));
    }
}

#[cfg(test)]
mod milestone {
    use super::level::{base, set, world};
    use super::*;
    use crate::event::Event;
    use crate::ids::MilestoneId;

    #[test]
    fn an_issue_is_matched_to_its_milestone_by_the_trackers_field() {
        let milestone = |name: &str| Scope::Milestone {
            of: ProjectId::new("radar"),
            name: name.into(),
        };
        let mut events = base();
        events.push(set(milestone("Sign-in"), None, "model", "opus", "file"));
        events.push(set(milestone("Other"), None, "model", "haiku", "file"));
        let w = world(&events);
        let model = |key: &str| w.setting(&IssueId::new("radar", key), None, "model");
        assert_eq!(
            model("44").map(|r| r.value.to_string()).as_deref(),
            Some("opus")
        );
        // An issue with no milestone, or another one, is not touched by it.
        let mut events = events.clone();
        events.push(Event::IssueSynced {
            issue: IssueId::new("radar", "45"),
            title: "t".into(),
            milestone: None,
            depends_on: vec![],
            order: 2,
            closed: false,
        });
        events.push(Event::IssueSynced {
            issue: IssueId::new("radar", "46"),
            title: "t".into(),
            milestone: Some(MilestoneId::new("Other")),
            depends_on: vec![],
            order: 3,
            closed: false,
        });
        let w = world(&events);
        let model = |key: &str| w.setting(&IssueId::new("radar", key), None, "model");
        assert_eq!(model("45"), None);
        assert_eq!(
            model("46").map(|r| r.value.to_string()).as_deref(),
            Some("haiku")
        );
    }

    #[test]
    fn a_milestone_is_named_with_its_project_or_area() {
        let w = world(&base());
        let target = |s: &str| w.scope_of(&crate::settings::Target::parse(s));
        assert_eq!(
            target("radar:Sign-in rework"),
            Ok(Scope::Milestone {
                of: ProjectId::new("radar"),
                name: "Sign-in rework".into()
            })
        );
        assert_eq!(
            target("guild:Heist"),
            Ok(Scope::Milestone {
                of: ProjectId::new("guild"),
                name: "Heist".into()
            })
        );
        assert!(target("nowhere:Heist").is_err_and(|e| e.contains("nowhere")));
    }

    #[test]
    fn a_milestone_of_an_area_reaches_the_issues_the_area_tracks() {
        let mut events = base();
        events.push(Event::AreaDefined {
            area: ProjectId::new("guild"),
            name: "Guild".into(),
            projects: vec![ProjectId::new("radar")],
            tracker: "t".into(),
            tracker_name: "GLD".into(),
        });
        events.push(Event::IssueSynced {
            issue: IssueId::new("guild", "GLD-1"),
            title: "t".into(),
            milestone: Some(MilestoneId::new("Heist")),
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        events.push(set(
            Scope::Milestone {
                of: ProjectId::new("guild"),
                name: "Heist".into(),
            },
            None,
            "model",
            "opus",
            "file",
        ));
        let w = world(&events);
        let r = w
            .setting(&IssueId::new("guild", "GLD-1"), None, "model")
            .expect("set");
        assert_eq!(r.level, Level::Milestone);
    }
}

#[cfg(test)]
mod area {
    use super::level::{base, set, world};
    use super::*;
    use crate::event::Event;

    #[test]
    fn an_issue_of_an_area_reads_the_project_level_of_the_repository_it_is_worked_in() {
        let mut events = base();
        events.push(Event::AreaDefined {
            area: ProjectId::new("guild"),
            name: "Guild".into(),
            projects: vec![ProjectId::new("radar"), ProjectId::new("ledger")],
            tracker: "t".into(),
            tracker_name: "GLD".into(),
        });
        events.push(Event::ProjectAdded {
            project: ProjectId::new("ledger"),
            tracker: String::new(),
            name: "city/ledger".into(),
            code_host: Some("h".into()),
            repo: None,
        });
        events.push(Event::IssueSynced {
            issue: IssueId::new("guild", "GLD-1"),
            title: "t".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        events.push(set(
            Scope::Project(ProjectId::new("radar")),
            None,
            "model",
            "radar's",
            "file",
        ));
        events.push(set(
            Scope::Project(ProjectId::new("ledger")),
            None,
            "model",
            "ledger's",
            "file",
        ));
        events.push(set(
            Scope::Area(ProjectId::new("guild")),
            None,
            "model",
            "guild's",
            "file",
        ));
        let w = world(&events);
        let model = |w: &World| {
            w.setting(&IssueId::new("guild", "GLD-1"), None, "model")
                .map(|r| (r.value.to_string(), r.level))
        };
        // No plan: the area's first repository.
        assert_eq!(model(&w), Some(("radar's".into(), Level::Project)));
        // The plan names the repository the change is worked in.
        let mut w = world(&events);
        w.issues
            .get_mut(&IssueId::new("guild", "GLD-1"))
            .expect("synced")
            .plan = Some(crate::model::Plan {
            repos: vec![ProjectId::new("ledger")],
            ..Default::default()
        });
        assert_eq!(model(&w), Some(("ledger's".into(), Level::Project)));
    }

    #[test]
    fn a_project_in_an_area_falls_through_to_the_area_then_global() {
        let mut events = base();
        events.push(set(Scope::Global, None, "model", "global", "file"));
        let w = world(&events);
        let at = |w: &World| {
            w.setting(&IssueId::new("radar", "44"), None, "model")
                .map(|r| r.level)
        };
        assert_eq!(at(&w), Some(Level::Global));
        events.push(set(
            Scope::Area(ProjectId::new("guild")),
            None,
            "model",
            "guild",
            "file",
        ));
        assert_eq!(at(&world(&events)), Some(Level::Area));
    }
}

#[cfg(test)]
mod replay {
    use super::level::{base, set, world};
    use super::*;
    use crate::event::Event;

    #[test]
    fn a_later_event_overrides_an_earlier_one_at_the_same_level_role_and_key() {
        let mut events = base();
        events.push(set(Scope::Global, None, "model", "sonnet", "file"));
        events.push(set(Scope::Global, None, "model", "haiku", "person"));
        events.push(set(
            Scope::Global,
            Some(Role::Author),
            "model",
            "opus",
            "person",
        ));
        let w = world(&events);
        let g = w.settings[&(Scope::Global, None, "model".to_string())].clone();
        assert_eq!(
            (g.value, g.by.as_str()),
            (Value::Text("haiku".into()), "person")
        );
        assert_eq!(
            w.settings.len(),
            2,
            "the role's own value is a key of its own"
        );
    }

    #[test]
    fn an_older_log_with_an_agent_set_deploy_or_permission_ignores_it() {
        let radar = || Scope::Project(ProjectId::new("radar"));
        let task = || Scope::Task(IssueId::new("radar", "44"));
        let by_refiner = |key: &str, value: Value| Event::SettingSet {
            scope: task(),
            role: None,
            key: key.into(),
            value,
            by: "refiner-3".into(),
        };
        let text = |value: &str| Value::Text(value.into());
        let mut events = base();
        events.push(set(radar(), None, "deploy", "make deploy", FILE));
        events.push(set(radar(), None, "permission", "plan", "person"));
        events.push(set(radar(), None, "runtime", "alpha", "person"));
        events.push(set(task(), None, "verify", "make smoke", "person"));
        events.extend([
            by_refiner("deploy", text("make mine")),
            by_refiner("run_in", text("/tmp/mine")),
            by_refiner("permission", text("bypassPermissions")),
            by_refiner("verify", text("true")),
            by_refiner("ci", Value::Flag(false)),
            by_refiner("runtime", text("beta")),
            by_refiner("runner", text("elsewhere")),
            by_refiner("review_rounds", Value::Number(1)),
            by_refiner("model", text("opus")),
        ]);
        events.push(set(
            task(),
            Some(Role::Author),
            "permission",
            "bypassPermissions",
            "refiner-3",
        ));
        // A refiner's runtime was once recorded for the author.
        events.push(set(
            task(),
            Some(Role::Author),
            "runtime",
            "beta",
            "refiner-3",
        ));
        // Nor does an agent's unset take away what a person set.
        events.push(Event::SettingUnset {
            scope: task(),
            role: None,
            key: "verify".into(),
            by: "refiner-3".into(),
        });
        let w = world(&events);
        let at = |key| {
            w.setting(&IssueId::new("radar", "44"), Some(Role::Author), key)
                .map(|r| (r.value.to_string(), r.source()))
        };
        let is = |value: &str, source: &str| Some((value.to_string(), source.to_string()));
        assert_eq!(at("deploy"), is("make deploy", "project"));
        assert_eq!(at("permission"), is("plan", "project, set by person"));
        assert_eq!(at("verify"), is("make smoke", "task, set by person"));
        assert_eq!(at("run_in"), None);
        assert_eq!(at("ci"), is("true", "built in"));
        // The next level's value, or none, so the build's own default is used.
        assert_eq!(at("runtime"), is("alpha", "project, set by person"));
        assert_eq!(at("runner"), None);
        assert_eq!(at("review_rounds"), is("3", "built in"));
        assert_eq!(at("model"), is("opus", "task, set by refiner-3"));
        // `temet-work settings` and `status` read these, so neither lists the rest.
        let agents: Vec<&str> = w
            .settings
            .iter()
            .filter(|(_, e)| e.by == "refiner-3")
            .map(|((_, _, key), _)| key.as_str())
            .collect();
        assert_eq!(agents, ["model"]);
    }

    #[test]
    fn an_older_log_with_a_refiners_model_for_every_role_reads_it_as_the_authors() {
        let task = || Scope::Task(IssueId::new("radar", "44"));
        let project = || Scope::Project(ProjectId::new("radar"));
        let mut events = base();
        events.push(set(
            project(),
            Some(Role::Reviewer),
            "model",
            "opus",
            "person",
        ));
        events.push(set(task(), None, "model", "sonnet", "refiner-14"));
        for role in [
            Role::Reviewer,
            Role::Arbiter,
            Role::Refiner,
            Role::Investigator,
        ] {
            events.push(set(task(), Some(role), "model", "haiku", "refiner-14"));
        }
        let read = |w: &World, role| {
            w.setting(&IssueId::new("radar", "44"), Some(role), "model")
                .map(|r| (r.value.to_string(), r.source()))
        };
        let is = |value: &str, source: &str| Some((value.to_string(), source.to_string()));
        let w = world(&events);
        assert_eq!(
            read(&w, Role::Author),
            is("sonnet", "task, set by refiner-14")
        );
        assert_eq!(
            read(&w, Role::Reviewer),
            is("opus", "project, set by person")
        );
        for role in [Role::Arbiter, Role::Refiner, Role::Investigator] {
            assert_eq!(read(&w, role), None, "{role:?}");
        }
        assert_eq!(
            w.settings.keys().filter(|(at, ..)| *at == task()).count(),
            1,
            "one value, for the author"
        );
        // An agent's unset for every role takes the author's value away.
        let mut unset = events.clone();
        unset.push(Event::SettingUnset {
            scope: task(),
            role: None,
            key: "model".into(),
            by: "refiner-14".into(),
        });
        assert_eq!(read(&world(&unset), Role::Author), None);
        // A person's later value for every role overturns it for every role.
        events.push(set(task(), None, "model", "fable", "person"));
        let w = world(&events);
        for role in [Role::Author, Role::Reviewer, Role::Arbiter] {
            assert_eq!(
                read(&w, role),
                is("fable", "task, set by person"),
                "{role:?}"
            );
        }
    }

    #[test]
    fn an_unset_removes_the_value_and_replaying_the_log_gives_the_same_settings() {
        let mut events = base();
        events.push(set(Scope::Global, None, "model", "sonnet", "file"));
        events.push(set(
            Scope::Task(IssueId::new("radar", "44")),
            None,
            "model",
            "opus",
            "refiner-3",
        ));
        events.push(Event::SettingUnset {
            scope: Scope::Global,
            role: None,
            key: "model".into(),
            by: "person".into(),
        });
        let once = world(&events);
        assert_eq!(once.settings.len(), 1);
        // Through the log's own format, event by event.
        let lines: Vec<String> = events
            .iter()
            .map(|e| serde_json::to_string(e).expect("serialises"))
            .collect();
        let again: Vec<Event> = lines
            .iter()
            .map(|l| serde_json::from_str(l).expect("reads back"))
            .collect();
        assert_eq!(world(&again).settings, once.settings);
        let r = once
            .setting(&IssueId::new("radar", "44"), Some(Role::Author), "model")
            .expect("task value");
        assert_eq!(r.source(), "task, set by refiner-3");
    }
}

#[cfg(test)]
mod unknown_adapter {
    use super::level::{base, world};
    use super::*;
    use crate::command::Command;

    fn with_catalog() -> World {
        let mut w = world(&base());
        w.catalog.insert(
            "runtime".into(),
            Choice {
                key: "runtime".into(),
                allowed: vec!["alpha".into(), "beta".into()],
                default: Some("alpha".into()),
            },
        );
        w
    }

    #[test]
    fn a_command_naming_an_adapter_the_build_lacks_is_refused_with_the_ones_it_has() {
        let w = with_catalog();
        let set = |value: &str| Command::Set {
            target: Target::parse("global"),
            role: None,
            key: "runtime".into(),
            value: value.into(),
        };
        let why = w.check(None, &set("nonesuch")).expect_err("refused");
        assert!(
            why.contains("nonesuch") && why.contains("alpha, beta"),
            "{why}"
        );
        assert!(w.check(None, &set("beta")).is_ok());
    }

    #[test]
    fn a_build_without_a_default_and_no_global_value_cannot_start() {
        let mut w = with_catalog();
        assert!(w.unready().is_empty());
        w.catalog.get_mut("runtime").expect("there").default = None;
        let why = w.unready().join("; ");
        assert!(
            why.contains("set one") && why.contains("alpha, beta"),
            "{why}"
        );
        w.put(
            &Scope::Global,
            None,
            "runtime",
            &Value::Text("beta".into()),
            "file",
        );
        assert!(w.unready().is_empty());
    }

    #[test]
    fn the_builds_default_is_what_nothing_set_resolves_to() {
        let w = with_catalog();
        let r = w
            .setting(&IssueId::new("radar", "44"), Some(Role::Author), "runtime")
            .expect("default");
        assert_eq!(
            (r.value.to_string().as_str(), r.level),
            ("alpha", Level::BuiltIn)
        );
    }
}

/// What a person names to set or read a setting at.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Target {
    /// Everything.
    Global,
    /// A project or an area, by id.
    Id(String),
    /// A milestone: `project:Milestone name`, or `area:Milestone name`.
    Milestone {
        /// The project or area.
        of: String,
        /// The milestone's name.
        name: String,
    },
    /// An issue: `project#key`.
    Issue(IssueId),
}

impl Target {
    /// Reads `global`, an id, `id:Milestone name` or `project#key`.
    pub fn parse(s: &str) -> Self {
        if s == "global" {
            return Self::Global;
        }
        if let Some((of, name)) = s.split_once(':') {
            return Self::Milestone {
                of: of.to_string(),
                name: name.to_string(),
            };
        }
        match IssueId::parse(s) {
            Some(issue) => Self::Issue(issue),
            None => Self::Id(s.to_string()),
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => f.write_str("global"),
            Self::Id(id) => f.write_str(id),
            Self::Milestone { of, name } => write!(f, "{of}:{name}"),
            Self::Issue(issue) => write!(f, "{issue}"),
        }
    }
}

impl World {
    /// The scope a target names, or why it names none.
    pub fn scope_of(&self, target: &Target) -> Result<Scope, String> {
        let place = |id: &str| {
            let id = ProjectId::new(id);
            if self.projects.contains_key(&id) {
                Ok(Scope::Project(id))
            } else if self.areas.contains_key(&id) {
                Ok(Scope::Area(id))
            } else {
                Err(format!("there is no project or area {id}"))
            }
        };
        match target {
            Target::Global => Ok(Scope::Global),
            Target::Id(id) => place(id),
            Target::Milestone { of, name } => {
                if name.trim().is_empty() {
                    return Err(format!("{of}: needs a milestone name after the colon"));
                }
                match place(of)? {
                    Scope::Project(of) | Scope::Area(of) => Ok(Scope::Milestone {
                        of,
                        name: name.clone(),
                    }),
                    _ => unreachable!("place returns a project or an area"),
                }
            }
            Target::Issue(issue) => {
                if self.issues.contains_key(issue) {
                    Ok(Scope::Task(issue.clone()))
                } else {
                    Err(format!("there is no issue {issue}"))
                }
            }
        }
    }
}
