//! The example in `docs/settings.md`, loaded as a configuration and run by the
//! two runtimes it names: what each program is started with.

#![cfg(all(feature = "runtime-claude", feature = "runtime-codex"))]

use temet_work::config::Config;
use temet_work_core::ids::{GroupId, IssueId, MilestoneId, ProjectId};
use temet_work_core::{Effect, Event, Outcome, World};

/// The first TOML block in `docs/settings.md`.
fn example() -> &'static str {
    include_str!("../docs/settings.md")
        .split_once("```toml\n")
        .and_then(|(_, block)| block.split_once("```"))
        .map(|(block, _)| block)
        .expect("the document has a toml block")
}

#[test]
fn the_documented_example_gives_each_runtime_only_its_own_model() {
    let config = Config::parse(example()).expect("the example loads");
    let mut runtime = temet_work::parts()
        .into_iter()
        .find(|p| p.id() == "runtime")
        .expect("the runtime family");
    let mut world = World::default();
    world
        .catalog
        .extend(runtime.choices().into_iter().map(|c| (c.key.clone(), c)));
    config.check_build(&world).expect("this build runs it");
    for event in config.events(&world) {
        world.apply(&event);
    }
    world.apply(&Event::GroupCreated {
        group: GroupId::new("watch"),
        name: "Night watch".into(),
        projects: vec![ProjectId::new("radar")],
        areas: Vec::new(),
        milestones: Vec::new(),
        refining_at_once: 2,
        model: None,
    });
    // radar#1 is in no milestone; radar#2 is in the example's.
    for (key, milestone) in [("1", None), ("2", Some("Slack sign-in rework"))] {
        world.apply(&Event::IssueSynced {
            issue: IssueId::new("radar", key),
            title: "Mend the clock".into(),
            milestone: milestone.map(MilestoneId::new),
            depends_on: vec![],
            order: key.parse().expect("a number"),
            closed: false,
        });
    }
    let mut argv = |effect: Effect| {
        let Outcome::Done { effects, .. } = runtime.carry_out(&effect, &world) else {
            panic!("{effect:?} was not carried out");
        };
        let [Effect::Run { program, .. }] = effects.as_slice() else {
            panic!("{effects:?}");
        };
        // The last argument is the task.
        program.argv[..program.argv.len() - 1].join(" ")
    };
    for (key, model) in [("1", "haiku"), ("2", "opus")] {
        let issue = IssueId::new("radar", key);
        let author = argv(Effect::StartAgent {
            group: GroupId::new("watch"),
            role: temet_work_core::model::Role::Author,
            issue: issue.clone(),
            folder: None,
        });
        assert_eq!(
            author,
            format!("claude --permission-mode auto --model {model}")
        );
        let reviewer = argv(Effect::StartReview { issue, round: 1 });
        assert_eq!(reviewer, "codex --approve-for-me");
    }
}
