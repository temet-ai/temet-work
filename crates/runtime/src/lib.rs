//! Agent runtimes: the programs that do the work. A runtime turns a brief into
//! the program a runner starts.
//!
//! This crate holds the [`Runtime`] contract, a [`FakeRuntime`] and the
//! [`contract`] every runtime adapter runs. Adapter crates depend on this crate only.

use std::path::{Path, PathBuf};

use temet_work_core::settings::{Choice, Scope};
use temet_work_core::world::LOCAL;
use temet_work_core::{Effect, Outcome, Part, World};

/// The core types this contract uses, re-exported so adapters need no other crate.
pub use temet_work_core::ids::IssueId;
pub use temet_work_core::launch::{Brief, Program};
pub use temet_work_core::model::Role;

/// One agent runtime, added by an adapter crate.
pub trait Runtime: Send {
    /// A short, stable id, the adapter's feature name after `runtime-`.
    fn id(&self) -> &'static str;

    /// Whether this is the runtime used when no setting names one. The build's
    /// first default one wins.
    fn is_default(&self) -> bool {
        false
    }

    /// The program that starts an agent with this brief: its task as the first
    /// prompt, and the model when the brief names one.
    fn program(&self, brief: &Brief) -> Program;

    /// Marks `folder` as one the person lets agents work in, before an agent
    /// starts there, in the runtime's own user files under `home` (the person's
    /// home folder, or the folder their settings are kept in when they name
    /// one). It changes nothing the runtime has already been told, and
    /// changes nothing else. The default does nothing.
    fn trust(&self, _folder: &Path, _home: &Path) -> Result<(), String> {
        Ok(())
    }

    /// The program that does what [`Runtime::trust`] does, run on the machine
    /// `folder` is on, for a folder on another machine: the daemon cannot reach
    /// the runtime's user files there. `None`, the default, when the runtime
    /// needs no folder trusted.
    fn trust_program(&self, _folder: &Path) -> Option<Program> {
        None
    }
}

/// The command line's hidden command that marks a folder trusted for a runtime,
/// `temet-work trust-folder RUNTIME PATH`, with `CLAUDE_CONFIG_DIR` from the
/// daemon's environment when it is set, as each agent gets it, so the folder is
/// trusted in the user files the agent reads. A runtime whose
/// [`Runtime::trust`] does something returns this from [`Runtime::trust_program`].
pub fn trust_folder_program(runtime: &str, folder: &Path) -> Program {
    Program {
        argv: vec![
            "temet-work".into(),
            "trust-folder".into(),
            runtime.into(),
            folder.to_string_lossy().into_owned(),
        ],
        env: passed_on_env()
            .into_iter()
            .filter(|(k, _)| k == "CLAUDE_CONFIG_DIR")
            .collect(),
    }
}

/// Where a runtime's user files are: the folder `CLAUDE_CONFIG_DIR` names when it
/// is set, else the person's home folder.
pub fn user_home() -> Option<PathBuf> {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// Variables every runtime passes on to its sessions from the daemon's
/// environment: a terminal gets its terminal service's environment, not the
/// daemon's. They are the proxy variables a capturing proxy needs and the code
/// host's sign-in, without which `gh` cannot read an issue or open a pull
/// request. `CLAUDE_CONFIG_DIR` goes too, so a session reads the same user files
/// the daemon marked its folder trusted in. Adapters use this one list, so a variable is added for every runtime
/// at once.
pub const PASSED_ON: [&str; 6] = [
    "CLAUDE_CONFIG_DIR",
    "HTTPS_PROXY",
    "NO_PROXY",
    "NODE_EXTRA_CA_CERTS",
    "GITHUB_TOKEN",
    "GH_TOKEN",
];

/// The [`PASSED_ON`] variables the daemon has, as `(name, value)` pairs.
pub fn passed_on_env() -> Vec<(String, String)> {
    passed_on_from(|k| std::env::var(k).ok())
}

/// The [`PASSED_ON`] variables `get` finds and that are not empty.
pub fn passed_on_from(get: impl Fn(&str) -> Option<String>) -> Vec<(String, String)> {
    PASSED_ON
        .iter()
        .filter_map(|k| get(k).filter(|v| !v.is_empty()).map(|v| (k.to_string(), v)))
        .collect()
}

/// A runtime for tests: `fake-agent [--model M] TASK`.
#[derive(Default)]
pub struct FakeRuntime;

impl Runtime for FakeRuntime {
    fn id(&self) -> &'static str {
        "fake"
    }

    fn is_default(&self) -> bool {
        true
    }

    fn program(&self, brief: &Brief) -> Program {
        let mut argv = vec!["fake-agent".to_string()];
        if let Some(model) = &brief.model {
            argv.push("--model".into());
            argv.push(model.clone());
        }
        argv.push("--permission".into());
        argv.push(brief.permission.clone());
        argv.push(brief.task.clone());
        Program {
            argv,
            env: Vec::new(),
        }
    }

    fn trust_program(&self, folder: &Path) -> Option<Program> {
        Some(Program {
            argv: vec!["fake-trust".into(), folder.to_string_lossy().into_owned()],
            env: Vec::new(),
        })
    }
}

/// The contract every runtime adapter runs.
pub mod contract {
    use super::Runtime;
    use temet_work_core::ids::IssueId;
    use temet_work_core::launch::Brief;
    use temet_work_core::model::Role;

    fn brief(model: Option<&str>) -> Brief {
        Brief {
            role: Role::Author,
            issue: Some(IssueId::new("contract", "1")),
            task: "Add the contract-check test, then stop.".into(),
            model: model.map(String::from),
            permission: "auto".into(),
            folder: None,
        }
    }

    /// Runs the contract. Returns the first step that failed, in words.
    pub fn check(runtime: &dyn Runtime) -> Result<(), String> {
        let program = runtime.program(&brief(None));
        if program.argv.is_empty() {
            return Err("the program has a name".into());
        }
        if !program
            .argv
            .iter()
            .any(|a| a.contains("Add the contract-check test"))
        {
            return Err("the brief's task reaches the program as its first prompt".into());
        }
        let with_model = runtime.program(&brief(Some("contract-model")));
        if !with_model.argv.iter().any(|a| a == "contract-model") {
            return Err("the brief's model reaches the program".into());
        }
        let folder = std::path::Path::new("/contract/folder");
        if let Some(trust) = runtime.trust_program(folder)
            && !trust.argv.iter().any(|a| a == "/contract/folder")
        {
            return Err("the program that trusts a folder on another machine names it".into());
        }
        Ok(())
    }
}

/// The runtime part: the runtimes compiled into this build, in the order they were added.
pub struct Runtimes {
    adapters: Vec<Box<dyn Runtime>>,
    /// Where the runtimes' user files are, for [`Runtime::trust`].
    home: Option<PathBuf>,
}

impl Default for Runtimes {
    fn default() -> Self {
        Self {
            adapters: Vec::new(),
            home: user_home(),
        }
    }
}

impl Runtimes {
    /// Uses `home` as where the runtimes' user files are, instead of the
    /// environment's.
    pub fn with_home(mut self, home: impl Into<PathBuf>) -> Self {
        self.home = Some(home.into());
        self
    }

    /// Adds a runtime. The root package calls this once per compiled-in adapter.
    pub fn add(&mut self, adapter: Box<dyn Runtime>) {
        self.adapters.push(adapter);
    }

    /// Marks `folder` trusted for the runtime `id`, in its user files under the
    /// home this family was given ([`user_home`] by default), as a start in a
    /// folder on this machine does. `temet-work trust-folder` runs this on the
    /// machine a folder is on.
    pub fn trust(&self, id: &str, folder: &Path) -> Result<(), String> {
        let runtime = self.adapters.iter().find(|a| a.id() == id).ok_or_else(|| {
            format!(
                "no runtime \"{id}\" in this build; this build has: {}",
                list(&self.ids())
            )
        })?;
        let home = self
            .home
            .as_ref()
            .ok_or("neither CLAUDE_CONFIG_DIR nor HOME is set, so there are no user files")?;
        runtime.trust(folder, home)
    }

    /// The ids of the runtimes in this build.
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }

    /// The id of the runtime used when no setting names one.
    pub fn default_id(&self) -> Option<&'static str> {
        self.adapters
            .iter()
            .find(|a| a.is_default())
            .map(|a| a.id())
    }

    /// The runtime an agent of `role` runs at `scope` (its issue's, or its
    /// group's for an agent with none): the one its `runtime` setting names, else
    /// this build's default.
    fn pick(&self, world: &World, scope: &Scope, role: Role) -> Result<&dyn Runtime, String> {
        let named = world.text_at(scope, Some(role), "runtime");
        let id = named
            .as_deref()
            .or_else(|| self.default_id())
            .ok_or_else(|| {
                format!(
                    "no runtime is set for {scope}'s {}s and this build has no default; \
                     set runtime globally. This build has: {}",
                    role.name(),
                    list(&self.ids())
                )
            })?;
        self.adapters
            .iter()
            .find(|a| a.id() == id)
            .map(|a| a.as_ref())
            .ok_or_else(|| {
                format!(
                    "no runtime \"{id}\" in this build; this build has: {}",
                    list(&self.ids())
                )
            })
    }
}

/// The shell script [`made_then`] runs: it makes the folder its first argument
/// names, then runs the rest as a program when there is one.
const MAKE_THEN: &str = r#"mkdir -p -- "$1" && shift && { [ $# -eq 0 ] || exec "$@"; }"#;

/// The program run on another machine before a reader starts there: it makes
/// the readers' folder `dir`, then runs `then`, the program that trusts the
/// folder, when the runtime has one, with that program's environment. A folder
/// already there is left as it is, since the readers of one project share it.
fn made_then(dir: &Path, then: Option<Program>) -> Program {
    let then = then.unwrap_or_default();
    let mut argv: Vec<String> = vec![
        "sh".into(),
        "-c".into(),
        MAKE_THEN.into(),
        "sh".into(),
        dir.to_string_lossy().into_owned(),
    ];
    argv.extend(then.argv);
    Program {
        argv,
        env: then.env,
    }
}

fn list(ids: &[&str]) -> String {
    if ids.is_empty() {
        "none".into()
    } else {
        ids.join(", ")
    }
}

impl Part for Runtimes {
    fn id(&self) -> &'static str {
        "runtime"
    }

    fn choices(&self) -> Vec<Choice> {
        vec![Choice {
            key: "runtime".into(),
            allowed: self.ids().into_iter().map(String::from).collect(),
            default: self.default_id().map(String::from),
        }]
    }

    /// Turns the loop's decision to start an agent into a program to run, with
    /// the runtime the `runtime` setting names for that issue and role.
    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let (group, role, issue, folder) = match effect {
            Effect::StartAgent {
                group,
                role,
                issue,
                folder,
            } => (group.clone(), *role, Some(issue), folder.as_ref()),
            // A review with no reviewer running starts one; a running one is
            // told of the next round elsewhere.
            Effect::StartReview { issue, .. } if world.running(issue, Role::Reviewer).is_none() => {
                let Some(group) = world.issues.get(issue).and_then(|i| i.group.clone()) else {
                    return Outcome::Failed(format!("{issue} has no group to review it"));
                };
                (group, Role::Reviewer, Some(issue), None)
            }
            Effect::Investigate { group } => (group.clone(), Role::Investigator, None, None),
            _ => return Outcome::NotMine,
        };
        let scope = world.agent_scope(&group, issue);
        let runtime = match self.pick(world, &scope, role) {
            Ok(r) => r,
            Err(e) => return Outcome::Failed(e),
        };
        let launch = match issue {
            Some(issue) => world.launch(&group, role, issue, folder),
            None => world.launch_investigator(&group),
        };
        let Some(launch) = launch else {
            return Outcome::Failed(match issue {
                Some(issue) => {
                    format!("cannot start a {} on {issue}: it is not known", role.name())
                }
                None => {
                    format!("cannot start an investigator for {group}: it runs no investigation")
                }
            });
        };
        // An agent that holds no work folder, a reader, runs in its readers'
        // folder, which is made before it starts. An author's work folder is a
        // checkout that is already there, and is never made.
        let readers = launch.folder.is_none();
        // A folder on this machine is made and trusted here before the agent
        // starts in it. One on another machine is made and trusted there, by the
        // program the runner runs on that machine before the agent's.
        let local = launch.machine.0 == LOCAL;
        if let (Some(dir), true, true) = (&launch.brief.folder, readers, local)
            && let Err(e) = std::fs::create_dir_all(dir)
        {
            return Outcome::Failed(format!("cannot make {}: {e}", dir.display()));
        }
        if let (Some(dir), Some(home), true) = (&launch.brief.folder, &self.home, local)
            && let Err(e) = runtime.trust(dir, home)
        {
            return Outcome::Failed(format!("cannot trust {}: {e}", dir.display()));
        }
        let before = match &launch.brief.folder {
            Some(dir) if !local => {
                let trust = runtime.trust_program(dir);
                if readers {
                    Some(made_then(dir, trust))
                } else {
                    trust
                }
            }
            _ => None,
        };
        let program = runtime.program(&launch.brief);
        Outcome::Done {
            events: Vec::new(),
            effects: vec![Effect::Run {
                launch: Box::new(launch),
                runtime: runtime.id().to_string(),
                program,
                before,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_a_start_into_a_program_to_run() {
        use temet_work_core::Event;
        use temet_work_core::ids::{GroupId, IssueId};
        use temet_work_core::model::Role;
        let mut world = World::default();
        world.apply(&Event::IssueSynced {
            issue: IssueId::new("ankh", "1"),
            title: "Mend the clock".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        let start = Effect::StartAgent {
            group: GroupId::new("watch"),
            role: Role::Refiner,
            issue: IssueId::new("ankh", "1"),
            folder: None,
        };
        let mut part = Runtimes::default();
        assert!(matches!(part.carry_out(&start, &world), Outcome::Failed(_)));
        part.add(Box::new(FakeRuntime));
        let Outcome::Done { effects, .. } = part.carry_out(&start, &world) else {
            panic!("not done");
        };
        let [
            Effect::Run {
                launch,
                runtime,
                program,
                before: None,
            },
        ] = effects.as_slice()
        else {
            panic!("{effects:?}");
        };
        assert_eq!(launch.agent.0, "refiner-1");
        assert_eq!(
            runtime, "fake",
            "it names the runtime that made the program"
        );
        assert_eq!(program.argv[0], "fake-agent");
        assert!(
            program
                .argv
                .last()
                .is_some_and(|t| t.contains("Mend the clock"))
        );
    }

    #[test]
    fn keeps_adapters_in_the_order_added() {
        let mut part = Runtimes::default();
        part.add(Box::new(FakeRuntime));
        assert_eq!(part.ids(), ["fake"]);
        assert_eq!(part.id(), "runtime");
    }

    #[test]
    fn passes_on_the_variables_that_are_set_and_not_empty() {
        let env = passed_on_from(|k| match k {
            "HTTPS_PROXY" => Some("http://p".into()),
            "GH_TOKEN" => Some(String::new()),
            "NO_PROXY" => Some("localhost".into()),
            _ => None,
        });
        assert_eq!(
            env,
            [
                ("HTTPS_PROXY".to_string(), "http://p".to_string()),
                ("NO_PROXY".to_string(), "localhost".to_string())
            ]
        );
        assert!(passed_on_from(|_| None).is_empty());
    }

    #[test]
    fn the_fake_passes_the_contract() {
        assert_eq!(contract::check(&FakeRuntime), Ok(()));
    }

    /// A second runtime, so the family always has two to choose between.
    struct Other;

    impl Runtime for Other {
        fn id(&self) -> &'static str {
            "other"
        }
        fn program(&self, brief: &Brief) -> Program {
            Program {
                argv: vec!["other-agent".into(), brief.task.clone()],
                env: Vec::new(),
            }
        }
    }

    fn world_with_issue() -> World {
        use temet_work_core::Event;
        let mut world = World::default();
        for key in ["1", "2"] {
            world.apply(&Event::IssueSynced {
                issue: IssueId::new("ankh", key),
                title: format!("issue {key}"),
                milestone: None,
                depends_on: vec![],
                order: 1,
                closed: false,
            });
        }
        world
    }

    fn start(role: Role, key: &str) -> Effect {
        Effect::StartAgent {
            group: temet_work_core::ids::GroupId::new("watch"),
            role,
            issue: IssueId::new("ankh", key),
            folder: None,
        }
    }

    fn program_for(part: &mut Runtimes, world: &World, effect: &Effect) -> String {
        let Outcome::Done { effects, .. } = part.carry_out(effect, world) else {
            panic!("not done");
        };
        let [Effect::Run { program, .. }] = effects.as_slice() else {
            panic!("{effects:?}");
        };
        program.argv[0].clone()
    }

    #[test]
    fn two_adapters_the_default_runs_until_a_setting_names_the_other() {
        use temet_work_core::Event;
        use temet_work_core::settings::{Scope, Value};
        let mut world = world_with_issue();
        let mut part = Runtimes::default();
        part.add(Box::new(Other));
        part.add(Box::new(FakeRuntime));
        assert_eq!(part.ids(), ["other", "fake"]);
        assert_eq!(part.default_id(), Some("fake"), "not the first one added");
        world.catalog = part
            .choices()
            .into_iter()
            .map(|c| (c.key.clone(), c))
            .collect();
        assert_eq!(
            program_for(&mut part, &world, &start(Role::Author, "1")),
            "fake-agent"
        );
        // Reviewers of ankh#1 run the other one; its authors and ankh#2 do not.
        world.apply(&Event::SettingSet {
            scope: Scope::Task(IssueId::new("ankh", "1")),
            role: Some(Role::Reviewer),
            key: "runtime".into(),
            value: Value::Text("other".into()),
            by: "person".into(),
        });
        assert_eq!(
            program_for(&mut part, &world, &start(Role::Reviewer, "1")),
            "other-agent"
        );
        assert_eq!(
            program_for(&mut part, &world, &start(Role::Author, "1")),
            "fake-agent"
        );
        assert_eq!(
            program_for(&mut part, &world, &start(Role::Reviewer, "2")),
            "fake-agent"
        );
    }

    #[test]
    fn two_adapters_a_setting_naming_one_the_build_lacks_is_a_problem_listing_the_ones_it_has() {
        use temet_work_core::Event;
        use temet_work_core::settings::{Scope, Value};
        let mut world = world_with_issue();
        let mut part = Runtimes::default();
        part.add(Box::new(FakeRuntime));
        part.add(Box::new(Other));
        world.apply(&Event::SettingSet {
            scope: Scope::Global,
            role: None,
            key: "runtime".into(),
            value: Value::Text("nonesuch".into()),
            by: "file".into(),
        });
        assert!(matches!(
            part.carry_out(&start(Role::Author, "1"), &world),
            Outcome::Failed(e) if e.contains("nonesuch") && e.contains("fake, other")
        ));
    }

    #[test]
    fn two_adapters_a_build_with_no_default_runtime_needs_one_set() {
        struct NoDefault;
        impl Runtime for NoDefault {
            fn id(&self) -> &'static str {
                "no-default"
            }
            fn program(&self, brief: &Brief) -> Program {
                Program {
                    argv: vec!["x".into(), brief.task.clone()],
                    env: Vec::new(),
                }
            }
        }
        let world = world_with_issue();
        let mut part = Runtimes::default();
        part.add(Box::new(NoDefault));
        assert_eq!(part.choices()[0].default, None);
        assert!(matches!(
            part.carry_out(&start(Role::Author, "1"), &world),
            Outcome::Failed(e) if e.contains("set runtime globally")
        ));
    }

    struct DropsTheModel;

    impl Runtime for DropsTheModel {
        fn id(&self) -> &'static str {
            "drops-the-model"
        }
        fn program(&self, brief: &Brief) -> Program {
            Program {
                argv: vec!["agent".into(), brief.task.clone()],
                env: Vec::new(),
            }
        }
    }

    #[test]
    fn the_contract_rejects_a_runtime_that_drops_the_model() {
        assert_eq!(
            contract::check(&DropsTheModel),
            Err("the brief's model reaches the program".into())
        );
    }

    /// A runtime that records the folders it was asked to trust, and what the
    /// terminal had started by then.
    struct Records(
        std::sync::Arc<std::sync::Mutex<Vec<(PathBuf, PathBuf)>>>,
        bool,
    );

    impl Runtime for Records {
        fn id(&self) -> &'static str {
            "records"
        }
        fn is_default(&self) -> bool {
            true
        }
        fn program(&self, brief: &Brief) -> Program {
            Program {
                argv: vec!["rec".into(), brief.task.clone()],
                env: Vec::new(),
            }
        }
        fn trust_program(&self, folder: &Path) -> Option<Program> {
            Some(Program {
                argv: vec!["rec-trust".into(), folder.to_string_lossy().into_owned()],
                env: Vec::new(),
            })
        }
        fn trust(&self, folder: &Path, home: &Path) -> Result<(), String> {
            if self.1 {
                return Err("no way".into());
            }
            self.0
                .lock()
                .expect("lock")
                .push((folder.to_path_buf(), home.to_path_buf()));
            Ok(())
        }
    }

    fn with_folder(machine: &str) -> World {
        use temet_work_core::Event;
        use temet_work_core::ids::{FolderId, MachineId, ProjectId};
        let mut world = world_with_issue();
        world.apply(&Event::FolderAdded {
            folder: FolderId::new("f1"),
            project: ProjectId::new("ankh"),
            machine: MachineId::new(machine),
            path: "/work/a".into(),
        });
        world
    }

    fn start_in(folder: Option<&str>) -> Effect {
        Effect::StartAgent {
            group: temet_work_core::ids::GroupId::new("watch"),
            role: Role::Author,
            issue: IssueId::new("ankh", "1"),
            folder: folder.map(temet_work_core::ids::FolderId::new),
        }
    }

    #[test]
    fn trusts_a_local_folder_before_the_agent_starts_and_only_then() {
        let seen = Default::default();
        let mut part = Runtimes::default().with_home("/h");
        part.add(Box::new(Records(std::sync::Arc::clone(&seen), false)));
        let world = with_folder(LOCAL);
        let Outcome::Done { effects, .. } = part.carry_out(&start_in(Some("f1")), &world) else {
            panic!("not done");
        };
        assert!(
            matches!(effects.as_slice(), [Effect::Run { before: None, .. }]),
            "a local folder needs nothing run before the agent: {effects:?}"
        );
        assert_eq!(
            *seen.lock().expect("lock"),
            [(PathBuf::from("/work/a"), PathBuf::from("/h"))]
        );
        // No folder, or a folder on another machine: nothing is trusted from here.
        let _ = part.carry_out(&start_in(None), &world_with_issue());
        let _ = part.carry_out(&start_in(Some("f1")), &with_folder("far"));
        assert_eq!(seen.lock().expect("lock").len(), 1);
    }

    #[test]
    fn a_folder_on_another_machine_is_trusted_there_by_the_runtimes_program_before_the_agent() {
        let seen = Default::default();
        let mut part = Runtimes::default().with_home("/h");
        part.add(Box::new(Records(std::sync::Arc::clone(&seen), false)));
        let Outcome::Done { effects, .. } =
            part.carry_out(&start_in(Some("f1")), &with_folder("far"))
        else {
            panic!("not done");
        };
        let [
            Effect::Run {
                launch,
                program,
                before: Some(before),
                ..
            },
        ] = effects.as_slice()
        else {
            panic!("{effects:?}");
        };
        assert_eq!(launch.machine.0, "far");
        assert_eq!(before.argv, ["rec-trust", "/work/a"]);
        assert_eq!(program.argv[0], "rec");
        assert!(
            seen.lock().expect("lock").is_empty(),
            "nothing is trusted on this machine"
        );
        // A runtime with no folder to trust runs nothing before its agent.
        struct Plain;
        impl Runtime for Plain {
            fn id(&self) -> &'static str {
                "plain"
            }
            fn is_default(&self) -> bool {
                true
            }
            fn program(&self, brief: &Brief) -> Program {
                Other.program(brief)
            }
        }
        let mut part = Runtimes::default().with_home("/h");
        part.add(Box::new(Plain));
        let world = with_folder("far");
        let Outcome::Done { effects, .. } = part.carry_out(&start_in(Some("f1")), &world) else {
            panic!("not done");
        };
        assert!(
            matches!(effects.as_slice(), [Effect::Run { before: None, .. }]),
            "{effects:?}"
        );
    }

    #[test]
    fn the_family_trusts_a_folder_for_the_runtime_it_is_asked_for() {
        let seen = Default::default();
        let mut part = Runtimes::default().with_home("/h");
        part.add(Box::new(Other));
        part.add(Box::new(Records(std::sync::Arc::clone(&seen), false)));
        assert_eq!(part.trust("records", Path::new("/work/a")), Ok(()));
        assert_eq!(
            *seen.lock().expect("lock"),
            [(PathBuf::from("/work/a"), PathBuf::from("/h"))]
        );
        assert_eq!(part.trust("other", Path::new("/work/a")), Ok(()));
        let why = part
            .trust("nonesuch", Path::new("/work/a"))
            .expect_err("unknown");
        assert!(
            why.contains("nonesuch") && why.contains("other, records"),
            "{why}"
        );
    }

    #[test]
    fn the_fake_trusts_a_far_folder_with_a_program_naming_it() {
        let program = FakeRuntime
            .trust_program(Path::new("/work/a"))
            .expect("a program");
        assert_eq!(program.argv, ["fake-trust", "/work/a"]);
        let cli = trust_folder_program("fake", Path::new("/work/a"));
        assert_eq!(cli.argv, ["temet-work", "trust-folder", "fake", "/work/a"]);
    }

    #[test]
    fn the_contract_rejects_a_trust_program_that_drops_the_folder() {
        struct Drops;
        impl Runtime for Drops {
            fn id(&self) -> &'static str {
                "drops"
            }
            fn program(&self, brief: &Brief) -> Program {
                FakeRuntime.program(brief)
            }
            fn trust_program(&self, _folder: &Path) -> Option<Program> {
                Some(Program {
                    argv: vec!["trust".into()],
                    env: Vec::new(),
                })
            }
        }
        assert_eq!(
            contract::check(&Drops),
            Err("the program that trusts a folder on another machine names it".into())
        );
    }

    #[test]
    fn a_folder_that_cannot_be_trusted_fails_the_start_naming_it() {
        let mut part = Runtimes::default().with_home("/h");
        part.add(Box::new(Records(Default::default(), true)));
        let Outcome::Failed(why) = part.carry_out(&start_in(Some("f1")), &with_folder(LOCAL))
        else {
            panic!("started");
        };
        assert!(why.contains("/work/a") && why.contains("no way"), "{why}");
    }

    #[test]
    fn an_investigation_starts_its_investigator_with_the_runtime_of_its_groups_project() {
        use temet_work_core::Event;
        use temet_work_core::ids::{AgentId, GroupId, ProjectId};
        use temet_work_core::settings::{Scope, Value};
        let mut world = World::default();
        let watch = GroupId::new("watch");
        for e in [
            Event::ProjectAdded {
                project: ProjectId::new("ankh"),
                tracker: "fake".into(),
                name: "city/ankh".into(),
                code_host: None,
                repo: None,
            },
            Event::GroupCreated {
                group: watch.clone(),
                name: "Night watch".into(),
                projects: vec![ProjectId::new("ankh")],
                areas: vec![],
                milestones: vec![],
                refining_at_once: 1,
                model: None,
                queue_low: None,
                investigate_every: None,
            },
            Event::SettingSet {
                scope: Scope::Project(ProjectId::new("ankh")),
                role: Some(Role::Investigator),
                key: "runtime".into(),
                value: Value::Text("other".into()),
                by: "person".into(),
            },
        ] {
            world.apply(&e);
        }
        let mut part = Runtimes::default();
        part.add(Box::new(FakeRuntime));
        part.add(Box::new(Other));
        let investigate = Effect::Investigate {
            group: watch.clone(),
        };
        assert!(
            matches!(part.carry_out(&investigate, &world), Outcome::Failed(why) if why.contains("runs no investigation")),
            "no investigation, no investigator"
        );
        world.apply(&Event::InvestigationStarted {
            group: watch,
            agent: AgentId::new("investigator-1"),
            at_ms: 0,
        });
        assert_eq!(program_for(&mut part, &world, &investigate), "other-agent");
        let Outcome::Done { effects, .. } = part.carry_out(&investigate, &world) else {
            panic!("not done");
        };
        let [Effect::Run { launch, .. }] = effects.as_slice() else {
            panic!("{effects:?}");
        };
        assert_eq!(
            (launch.agent.0.as_str(), launch.brief.issue.as_ref()),
            ("investigator-1", None)
        );
    }

    /// A path of its own for the test `name`, with nothing there yet, in the
    /// build folder beside the test program: the person's own folder, not the
    /// shared temporary one.
    fn temporary(name: &str) -> PathBuf {
        let build = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        let path = build.join(format!("temet-work-runtime-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        let _ = std::fs::remove_file(&path);
        path
    }

    /// ankh#1 and ankh#2 in a world whose readers' folder is `readers`.
    fn with_readers(readers: &Path) -> World {
        let mut world = world_with_issue();
        world.readers = Some(readers.to_path_buf());
        world
    }

    /// A runtime that records each folder it is asked to trust, and whether the
    /// folder was there by then. Its program that trusts a folder on another
    /// machine checks that the folder is there, and has a variable of its own.
    struct Checks(std::sync::Arc<std::sync::Mutex<Vec<(PathBuf, bool)>>>);

    impl Runtime for Checks {
        fn id(&self) -> &'static str {
            "checks"
        }
        fn is_default(&self) -> bool {
            true
        }
        fn program(&self, brief: &Brief) -> Program {
            Other.program(brief)
        }
        fn trust(&self, folder: &Path, _home: &Path) -> Result<(), String> {
            self.0
                .lock()
                .expect("lock")
                .push((folder.to_path_buf(), folder.is_dir()));
            Ok(())
        }
        fn trust_program(&self, folder: &Path) -> Option<Program> {
            Some(Program {
                argv: vec!["test".into(), "-d".into(), folder.display().to_string()],
                env: vec![("CLAUDE_CONFIG_DIR".into(), "/h/.claude".into())],
            })
        }
    }

    /// Runs `program` here, as a runner runs it on its machine.
    fn runs(program: &Program) -> bool {
        std::process::Command::new(&program.argv[0])
            .args(&program.argv[1..])
            .envs(program.env.iter().cloned())
            .status()
            .is_ok_and(|s| s.success())
    }

    #[test]
    fn a_reviewers_readers_folder_is_made_and_empty_before_it_starts() {
        let readers = temporary("made");
        let world = with_readers(&readers);
        let seen = Default::default();
        let mut part = Runtimes::default().with_home("/h");
        part.add(Box::new(Checks(std::sync::Arc::clone(&seen))));
        let Outcome::Done { effects, .. } = part.carry_out(&start(Role::Reviewer, "1"), &world)
        else {
            panic!("not done");
        };
        let [
            Effect::Run {
                launch,
                before: None,
                ..
            },
        ] = effects.as_slice()
        else {
            panic!("{effects:?}");
        };
        let dir = readers.join("ankh");
        assert_eq!(launch.brief.folder.as_deref(), Some(dir.as_path()));
        let made = std::fs::read_dir(&dir).map(Iterator::count);
        assert_eq!(made.ok(), Some(0), "made, and empty");
        assert_eq!(
            *seen.lock().expect("lock"),
            [(dir.clone(), true)],
            "made before it is trusted"
        );
        // The project's next reader shares it: it is not emptied.
        std::fs::write(dir.join("notes"), "Vimes was here").expect("write");
        let again = part.carry_out(&start(Role::Refiner, "2"), &world);
        assert!(matches!(again, Outcome::Done { .. }), "{again:?}");
        assert!(dir.join("notes").exists());
        let _ = std::fs::remove_dir_all(&readers);
    }

    #[test]
    fn a_readers_folder_that_cannot_be_made_fails_the_start_naming_it() {
        let readers = temporary("blocked");
        std::fs::write(&readers, "a file, not a folder").expect("write");
        let mut part = Runtimes::default().with_home("/h");
        part.add(Box::new(FakeRuntime));
        let outcome = part.carry_out(&start(Role::Reviewer, "1"), &with_readers(&readers));
        let _ = std::fs::remove_file(&readers);
        let Outcome::Failed(why) = outcome else {
            panic!("started: {outcome:?}");
        };
        assert!(
            why.starts_with("cannot make ") && why.contains(&readers.display().to_string()),
            "{why}"
        );
    }

    #[test]
    fn a_reader_on_another_machine_has_its_folder_made_there_then_trusted() {
        use temet_work_core::Event;
        use temet_work_core::settings::{Scope, Value};
        let readers = temporary("far");
        let mut world = with_readers(&readers);
        world.apply(&Event::SettingSet {
            scope: Scope::Task(IssueId::new("ankh", "1")),
            role: Some(Role::Reviewer),
            key: "machine".into(),
            value: Value::Text("far".into()),
            by: "person".into(),
        });
        let seen = Default::default();
        let mut part = Runtimes::default().with_home("/h");
        part.add(Box::new(Checks(std::sync::Arc::clone(&seen))));
        let Outcome::Done { effects, .. } = part.carry_out(&start(Role::Reviewer, "1"), &world)
        else {
            panic!("not done");
        };
        let [
            Effect::Run {
                launch,
                before: Some(before),
                ..
            },
        ] = effects.as_slice()
        else {
            panic!("{effects:?}");
        };
        let dir = readers.join("ankh");
        let path = dir.display().to_string();
        assert_eq!(launch.machine.0, "far");
        assert_eq!(
            before.argv,
            ["sh", "-c", MAKE_THEN, "sh", &path, "test", "-d", &path]
        );
        assert_eq!(
            before.env,
            [("CLAUDE_CONFIG_DIR".to_string(), "/h/.claude".to_string())],
            "the variables of the program that trusts it"
        );
        assert!(!readers.exists(), "nothing is made on this machine");
        assert!(seen.lock().expect("lock").is_empty(), "nor trusted here");
        // Run as the runner runs it there: it makes the folder, then trusts it.
        assert!(runs(before));
        assert!(dir.is_dir());
        // With no program to trust it, the folder is made all the same.
        std::fs::remove_dir_all(&readers).expect("remove");
        let made = made_then(&dir, None);
        assert_eq!(made.argv, ["sh", "-c", MAKE_THEN, "sh", &path]);
        assert!(runs(&made));
        assert!(dir.is_dir());
        let _ = std::fs::remove_dir_all(&readers);
    }

    #[test]
    fn an_authors_work_folder_is_never_made() {
        use temet_work_core::Event;
        use temet_work_core::ids::{FolderId, MachineId, ProjectId};
        let gone = temporary("author");
        for machine in [LOCAL, "far"] {
            let mut world = with_readers(&gone.join("readers"));
            world.apply(&Event::FolderAdded {
                folder: FolderId::new("f1"),
                project: ProjectId::new("ankh"),
                machine: MachineId::new(machine),
                path: gone.join("w1").display().to_string(),
            });
            let mut part = Runtimes::default().with_home("/h");
            part.add(Box::new(FakeRuntime));
            let Outcome::Done { effects, .. } = part.carry_out(&start_in(Some("f1")), &world)
            else {
                panic!("not done");
            };
            let [Effect::Run { before, .. }] = effects.as_slice() else {
                panic!("{effects:?}");
            };
            assert!(!gone.exists(), "nothing is made for an author on {machine}");
            // On another machine it is only trusted there.
            let trust = (machine != LOCAL).then(|| {
                vec![
                    "fake-trust".to_string(),
                    gone.join("w1").display().to_string(),
                ]
            });
            assert_eq!(before.as_ref().map(|b| b.argv.clone()), trust);
        }
    }
}
