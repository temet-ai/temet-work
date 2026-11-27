//! Agent runtimes: the programs that do the work. A runtime turns a brief into
//! the program a runner starts.
//!
//! This crate holds the [`Runtime`] contract, a [`FakeRuntime`] and the
//! [`contract`] every runtime adapter runs. Adapter crates depend on this crate only.

use std::path::{Path, PathBuf};

use temet_work_core::settings::Choice;
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
            issue: IssueId::new("contract", "1"),
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

    /// The runtime an agent of `role` on `issue` runs: the one its `runtime`
    /// setting names, else this build's default.
    fn pick(&self, world: &World, issue: &IssueId, role: Role) -> Result<&dyn Runtime, String> {
        let named = world.setting_text(issue, Some(role), "runtime");
        let id = named
            .as_deref()
            .or_else(|| self.default_id())
            .ok_or_else(|| {
                format!(
                    "no runtime is set for {issue}'s {}s and this build has no default; \
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
            } => (group.clone(), *role, issue, folder.as_ref()),
            // A review with no reviewer running starts one; a running one is
            // told of the next round elsewhere.
            Effect::StartReview { issue, .. } if world.running(issue, Role::Reviewer).is_none() => {
                let Some(group) = world.issues.get(issue).and_then(|i| i.group.clone()) else {
                    return Outcome::Failed(format!("{issue} has no group to review it"));
                };
                (group, Role::Reviewer, issue, None)
            }
            _ => return Outcome::NotMine,
        };
        let runtime = match self.pick(world, issue, role) {
            Ok(r) => r,
            Err(e) => return Outcome::Failed(e),
        };
        let Some(launch) = world.launch(&group, role, issue, folder) else {
            return Outcome::Failed(format!(
                "cannot start a {} on {issue}: it is not known, or no work folder is on the \
                 machine its `machine` setting names",
                role.name()
            ));
        };
        // A folder on this machine is trusted before the agent starts in it. One on
        // another machine cannot be from here: the terminal's blocked prompt
        // reports it.
        let local = launch.machine.0 == LOCAL;
        if let (Some(dir), Some(home), true) = (&launch.brief.folder, &self.home, local)
            && let Err(e) = runtime.trust(dir, home)
        {
            return Outcome::Failed(format!("cannot trust {}: {e}", dir.display()));
        }
        let program = runtime.program(&launch.brief);
        Outcome::Done {
            events: Vec::new(),
            effects: vec![Effect::Run { launch, program }],
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
        let [Effect::Run { launch, program }] = effects.as_slice() else {
            panic!("{effects:?}");
        };
        assert_eq!(launch.agent.0, "refiner-1");
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
        assert!(matches!(effects.as_slice(), [Effect::Run { .. }]));
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
    fn a_folder_that_cannot_be_trusted_fails_the_start_naming_it() {
        let mut part = Runtimes::default().with_home("/h");
        part.add(Box::new(Records(Default::default(), true)));
        let Outcome::Failed(why) = part.carry_out(&start_in(Some("f1")), &with_folder(LOCAL))
        else {
            panic!("started");
        };
        assert!(why.contains("/work/a") && why.contains("no way"), "{why}");
    }
}
