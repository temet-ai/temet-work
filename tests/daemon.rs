//! The daemon's loop with every family's fake: a configured project's issue is
//! synced, a group takes it, and a refiner starts in a labelled terminal.

#![cfg(all(feature = "tracker", feature = "runner", feature = "runtime"))]

use temet_work::config::Config;
use temet_work_core::host::Memory;
use temet_work_core::ids::{AgentId, IssueId};
use temet_work_core::model::Stage;
use temet_work_core::{Host, World};
use temet_work_runner::fake::FakeRunner;
use temet_work_runtime::FakeRuntime;
use temet_work_tracker::{FakeTracker, TrackedIssue};

const CONFIG: &str = r#"
[[project]]
id = "ankh"
tracker = "fake"
name = "city/ankh"

[[folder]]
id = "ankh-1"
project = "ankh"
path = "."

[[group]]
id = "watch"
name = "Night watch"
projects = ["ankh"]
"#;

fn host() -> Host {
    let mut trackers = temet_work_tracker::Trackers::default();
    trackers.add(Box::new(FakeTracker {
        issues: vec![(
            "city/ankh".into(),
            TrackedIssue {
                key: "1".into(),
                title: "Mend the clock".into(),
                body: String::new(),
                milestone: None,
                depends_on: vec![],
                order: 1,
                closed: false,
            },
        )],
        ..FakeTracker::default()
    }));
    let mut runtimes = temet_work_runtime::Runtimes::default();
    runtimes.add(Box::new(FakeRuntime));
    let mut runners = temet_work_runner::Runners::default();
    runners.add(Box::new(FakeRunner::default()));
    let mut host = Host::new(
        World::default(),
        vec![Box::new(trackers), Box::new(runtimes), Box::new(runners)],
        Box::new(Memory::default()),
    );
    for event in Config::parse(CONFIG).expect("config").events(host.world()) {
        host.handle(0, event).expect("config event");
    }
    host
}

#[test]
fn a_synced_issue_gets_a_refiner_in_a_labelled_terminal() {
    let mut host = host();
    assert_eq!(host.tick(0), Ok(1), "one issue synced");
    let issue = IssueId::new("ankh", "1");
    assert_eq!(host.world().issues[&issue].stage, Stage::Refining);
    let refiner = &host.world().agents[&AgentId::new("refiner-1")];
    assert_eq!(refiner.issue.as_ref(), Some(&issue));
    assert!(host.problems().is_empty(), "{:?}", host.problems());
    assert_eq!(
        host.tick(1),
        Ok(0),
        "the terminal's first update changes nothing"
    );
    let text = temet_work_core::text::text(host.world(), host.problems());
    assert!(text.contains("ankh#1  refining  Mend the clock"), "{text}");
    assert!(
        text.contains("refiner-1 on ankh#1: idle (on local)"),
        "{text}"
    );
}
