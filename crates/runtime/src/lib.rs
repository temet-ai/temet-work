//! Agent runtimes: the programs that do the work. A runtime turns a brief into
//! the program a runner starts.
//!
//! This crate holds the [`Runtime`] contract, a [`FakeRuntime`] and the
//! [`contract`] every runtime adapter runs. Adapter crates depend on this crate only.

use temet_work_core::{Effect, Outcome, Part, World};

/// The core types this contract uses, re-exported so adapters need no other crate.
pub use temet_work_core::launch::{Brief, Program};

/// One agent runtime, added by an adapter crate.
pub trait Runtime: Send {
    /// A short, stable id, the adapter's feature name after `runtime-`.
    fn id(&self) -> &'static str;

    /// The program that starts an agent with this brief: its task as the first
    /// prompt, and the model when the brief names one.
    fn program(&self, brief: &Brief) -> Program;
}

/// A runtime for tests: `fake-agent [--model M] TASK`.
#[derive(Default)]
pub struct FakeRuntime;

impl Runtime for FakeRuntime {
    fn id(&self) -> &'static str {
        "fake"
    }

    fn program(&self, brief: &Brief) -> Program {
        let mut argv = vec!["fake-agent".to_string()];
        if let Some(model) = &brief.model {
            argv.push("--model".into());
            argv.push(model.clone());
        }
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
#[derive(Default)]
pub struct Runtimes {
    adapters: Vec<Box<dyn Runtime>>,
}

impl Runtimes {
    /// Adds a runtime. The root package calls this once per compiled-in adapter.
    pub fn add(&mut self, adapter: Box<dyn Runtime>) {
        self.adapters.push(adapter);
    }

    /// The ids of the runtimes in this build.
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }
}

impl Part for Runtimes {
    fn id(&self) -> &'static str {
        "runtime"
    }

    /// Turns the loop's decision to start an agent into a program to run, with
    /// the first runtime in this build.
    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let Effect::StartAgent {
            group,
            role,
            issue,
            folder,
        } = effect
        else {
            return Outcome::NotMine;
        };
        let Some(runtime) = self.adapters.first() else {
            return Outcome::Failed("no agent runtime is compiled into this build".into());
        };
        let Some(launch) = world.launch(group, *role, issue, folder.as_ref()) else {
            return Outcome::Failed(format!("{issue} is not known"));
        };
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
    fn the_fake_passes_the_contract() {
        assert_eq!(contract::check(&FakeRuntime), Ok(()));
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
}
