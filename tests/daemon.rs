//! The daemon's loop with every family's fake: a configured project's issue is
//! synced, a group takes it, and a refiner starts in a labelled terminal.

#![cfg(all(
    feature = "tracker",
    feature = "runner",
    feature = "runtime",
    feature = "codehost",
    feature = "deploy"
))]

use temet_work::config::Config;
use temet_work_codehost::{Checks, FakeCodeHost, Pull, PullState};
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
deploy = "make deploy"

[[folder]]
id = "ankh-1"
project = "ankh"
path = "."

[[group]]
id = "watch"
name = "Night watch"
projects = ["ankh"]
"#;

/// A journal the test reads after the host has taken it.
#[derive(Clone, Default)]
struct Kept(std::sync::Arc<std::sync::Mutex<Vec<temet_work_core::Event>>>);

impl temet_work_core::Journal for Kept {
    fn append(&mut self, _at_ms: u64, event: &temet_work_core::Event) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .push(event.clone());
        Ok(())
    }
}

fn host() -> Host {
    host_with(Box::new(Memory::default()))
}

fn host_with(journal: Box<dyn temet_work_core::Journal>) -> Host {
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
    let mut code_hosts = temet_work_codehost::CodeHosts::default().every(0);
    code_hosts.add(Box::new(FakeCodeHost {
        pulls: vec![(
            "city/ankh".into(),
            Pull {
                number: "57".into(),
                state: PullState::Open,
                checks: Checks::Passed,
                conflicting: false,
            },
        )],
    }));
    let deploys = temet_work_deploy::Deploys::with(std::sync::Arc::new(
        |_: &str, _: &std::path::PathBuf, _: &[(String, String)]| Ok(()),
    ));
    let mut host = Host::new(
        World::default(),
        vec![
            Box::new(trackers),
            Box::new(runners),
            Box::new(runtimes),
            Box::new(code_hosts),
            Box::new(deploys),
        ],
        journal,
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
                    pull: Some("57".into()),
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

#[test]
fn an_issue_goes_from_plan_through_two_review_rounds_with_one_reviewer() {
    use temet_work::socket::{Reply, answer};
    use temet_work_core::model::{Check, Plan};
    use temet_work_core::{Command, Request, Verdict};

    let kept = Kept::default();
    let mut host = host_with(Box::new(kept.clone()));
    host.tick(0).expect("sync");
    let issue = IssueId::new("ankh", "1");
    let run = |host: &mut Host, by: &str, command: Command| {
        let request = Request {
            by: Some(AgentId::new(by)),
            command,
        };
        match answer(host, 1, &request).expect("answer") {
            Reply::Done { text } => text,
            Reply::Refused { why } => panic!("{by}: refused: {why}"),
        }
    };
    let plan = Plan {
        checks: vec![Check {
            done_when: "the clock strikes thirteen".into(),
            proven_by: "listening at midnight".into(),
        }],
    };
    run(
        &mut host,
        "refiner-1",
        Command::Plan {
            issue: issue.clone(),
            plan,
        },
    );
    assert!(host.world().agents.contains_key(&AgentId::new("author-1")));
    let ready = Command::Ready {
        issue: issue.clone(),
        pull: Some("57".into()),
    };
    assert_eq!(
        run(&mut host, "author-1", ready.clone()),
        "ankh#1 is in review, round 1"
    );
    assert!(
        host.world()
            .agents
            .contains_key(&AgentId::new("reviewer-1"))
    );
    let block = Command::Review {
        issue: issue.clone(),
        verdict: Verdict::Block {
            points: vec!["no test for a stopped clock".into()],
        },
    };
    assert_eq!(
        run(&mut host, "reviewer-1", block),
        "ankh#1 is fixing, round 1"
    );
    assert_eq!(
        run(&mut host, "author-1", ready),
        "ankh#1 is in review, round 2"
    );
    let reviewers: Vec<&str> = host
        .world()
        .agents
        .keys()
        .map(|a| a.0.as_str())
        .filter(|a| a.starts_with("reviewer"))
        .collect();
    assert_eq!(
        reviewers,
        ["reviewer-1"],
        "the running reviewer takes round 2"
    );
    assert!(host.world().issues[&issue].told, "and it heard of round 2");
    assert!(host.problems().is_empty(), "{:?}", host.problems());

    let approve = Command::Review {
        issue: issue.clone(),
        verdict: Verdict::Approve,
    };
    run(&mut host, "reviewer-1", approve);
    host.tick(2).expect("CI read");
    assert_eq!(
        host.world().issues[&issue].stage,
        Stage::Deploying,
        "approved, CI passed, merged, deploy command running"
    );
    let folder = &host.world().folders[&temet_work_core::ids::FolderId::new("ankh-1")];
    assert_eq!(folder.holder, None, "the folder is free for the next issue");
    for now in 3..100 {
        host.tick(now).expect("tick");
        if host.world().issues[&issue].stage == Stage::Done {
            break;
        }
        std::thread::yield_now();
    }
    assert_eq!(
        host.world().issues[&issue].stage,
        Stage::Done,
        "deployed, verified, closed"
    );
    assert!(host.problems().is_empty(), "{:?}", host.problems());
    // The log of a live run, with parts carrying out effects, replays to the same state.
    let logged = kept.0.lock().expect("journal").clone();
    assert_eq!(&World::replay(&logged).0, host.world());
}
