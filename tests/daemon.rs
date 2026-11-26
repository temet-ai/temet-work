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

#[test]
fn commands_reach_the_daemon_over_its_socket() {
    use temet_work::socket::{self, Reply};
    use temet_work_core::{Command, Request};

    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("daemon-test.sock");
    let requests = socket::listen(&path).expect("listen");
    assert!(socket::listen(&path).is_err(), "a second daemon is refused");
    let client = {
        let path = path.clone();
        std::thread::spawn(move || {
            let ask = |command| {
                socket::send(
                    &path,
                    &Request {
                        by: Some(AgentId::new("author-9")),
                        command,
                    },
                )
                .expect("reply")
            };
            (
                ask(Command::Status),
                ask(Command::Ready {
                    issue: IssueId::new("ankh", "1"),
                }),
            )
        })
    };
    let mut host = host();
    host.tick(0).expect("tick");
    for _ in 0..2 {
        let pending = requests
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("a request");
        let reply = socket::answer(&mut host, 1, &pending.request).expect("answer");
        pending.answer(reply);
    }
    let (status, ready) = client.join().expect("client");
    assert!(
        matches!(&status, Reply::Done { text } if text.contains("refiner-1 on ankh#1")),
        "{status:?}"
    );
    assert_eq!(
        ready,
        Reply::Refused {
            why: "ankh#1 is refining, not being worked on".into()
        }
    );
}
