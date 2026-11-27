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

fn fake_runtimes() -> temet_work_runtime::Runtimes {
    let mut runtimes = temet_work_runtime::Runtimes::default();
    runtimes.add(Box::new(FakeRuntime));
    runtimes
}

fn host() -> Host {
    host_with(Box::new(Memory::default()))
}

fn host_with(journal: Box<dyn temet_work_core::Journal>) -> Host {
    let mut runtimes = temet_work_runtime::Runtimes::default();
    runtimes.add(Box::new(FakeRuntime));
    host_with_runtimes(journal, runtimes)
}

fn host_with_runtimes(
    journal: Box<dyn temet_work_core::Journal>,
    runtimes: temet_work_runtime::Runtimes,
) -> Host {
    host_on(journal, runtimes, CONFIG, None)
}

fn host_on(
    journal: Box<dyn temet_work_core::Journal>,
    runtimes: temet_work_runtime::Runtimes,
    config: &str,
    daemon_host: Option<&str>,
) -> Host {
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
    let mut runners = temet_work_runner::Runners::default();
    runners.add(Box::new(FakeRunner::default()));
    let runners = runners.daemon_host(daemon_host.map(String::from));
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
        ..FakeCodeHost::default()
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
    for event in Config::parse(config).expect("config").events(host.world()) {
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

/// A reviewer set to run Codex starts as the Codex program in a labelled
/// terminal; the author of the same issue keeps the build's default runtime.
#[cfg(feature = "runtime-codex")]
#[test]
fn a_reviewer_set_to_codex_starts_the_codex_program() {
    use temet_work::socket::{Reply, answer};
    use temet_work_core::model::{Check, Plan, Role};
    use temet_work_core::settings::Target;
    use temet_work_core::{Command, Request};

    let mut runtimes = temet_work_runtime::Runtimes::default();
    runtimes.add(Box::new(FakeRuntime));
    runtimes.add(Box::new(temet_work_runtime_codex::Codex));
    let mut host = host_with_runtimes(Box::new(Memory::default()), runtimes);
    let run = |host: &mut Host, by: Option<&str>, command: Command| {
        let request = Request {
            by: by.map(AgentId::new),
            command,
        };
        match answer(host, 1, &request).expect("answer") {
            Reply::Done { text } => text,
            Reply::Refused { why } => panic!("refused: {why}"),
        }
    };
    run(
        &mut host,
        None,
        Command::Set {
            target: Target::parse("ankh"),
            role: Some(Role::Reviewer),
            key: "runtime".into(),
            value: "codex".into(),
        },
    );
    host.tick(0).expect("sync");
    let issue = IssueId::new("ankh", "1");
    let plan = Plan {
        repos: Vec::new(),
        settings: Default::default(),
        checks: vec![Check {
            done_when: "the clock strikes thirteen".into(),
            proven_by: "listening at midnight".into(),
        }],
    };
    run(
        &mut host,
        Some("refiner-1"),
        Command::Plan {
            issue: issue.clone(),
            plan,
        },
    );
    run(
        &mut host,
        Some("author-1"),
        Command::Ready {
            issue,
            pull: Some("57".into()),
        },
    );
    assert!(host.problems().is_empty(), "{:?}", host.problems());
    let screen = |host: &mut Host, agent: &str| {
        run(
            host,
            None,
            Command::Screen {
                agent: AgentId::new(agent),
            },
        )
    };
    let reviewer = screen(&mut host, "reviewer-1");
    assert!(reviewer.contains("$ codex --approve-for-me "), "{reviewer}");
    let author = screen(&mut host, "author-1");
    assert!(author.contains("$ fake-agent "), "{author}");
}

#[test]
fn a_folder_on_another_machine_needs_daemon_host_before_an_agent_starts_there() {
    let config = CONFIG.replace("path = \".\"", "machine = \"mac\"\npath = \"/work/ankh\"");
    let mut host = host_on(Box::new(Memory::default()), fake_runtimes(), &config, None);
    host.tick(0).expect("tick");
    assert!(host.world().agents.is_empty(), "no agent started");
    let problems = host
        .problems()
        .values()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        problems.contains("machine")
            && problems.contains("daemon_host")
            && problems.contains("mac"),
        "{problems}"
    );
    let mut host = host_on(Box::new(Memory::default()), fake_runtimes(), &config, Some("omit"));
    host.tick(0).expect("tick");
    assert_eq!(host.world().agents.len(), 1, "{:?}", host.problems());
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
        repos: Vec::new(),
        settings: Default::default(),
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
    // The deploy and verify commands run on threads: usually done within a
    // millisecond, waited on for at most two seconds.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let mut now = 3;
    while host.world().issues[&issue].stage != Stage::Done && std::time::Instant::now() < deadline {
        host.tick(now).expect("tick");
        now += 1;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(
        host.world().issues[&issue].stage,
        Stage::Done,
        "deployed, verified, closed: {:?}",
        host.problems()
    );
    assert!(host.problems().is_empty(), "{:?}", host.problems());
    // The log of a live run, with parts carrying out effects, replays to the same state.
    let logged = kept.0.lock().expect("journal").clone();
    let mut replayed = World::replay(&logged).0;
    // What the build's parts offer is not in the log.
    replayed.catalog = host.world().catalog.clone();
    assert_eq!(&replayed, host.world());
}

mod settings_commands {
    use super::*;
    use temet_work::socket::{Reply, answer};
    use temet_work_core::Command;
    use temet_work_core::Request;
    use temet_work_core::event::Event;
    use temet_work_core::settings::{Scope, Target, Value};

    fn ask(host: &mut Host, by: Option<&str>, command: Command) -> Reply {
        let request = Request {
            by: by.map(AgentId::new),
            command,
        };
        answer(host, 1, &request).expect("answer")
    }

    fn set(
        target: &str,
        role: Option<temet_work_core::model::Role>,
        key: &str,
        value: &str,
    ) -> Command {
        Command::Set {
            target: Target::parse(target),
            role,
            key: key.into(),
            value: value.into(),
        }
    }

    /// The words of the line that lists `key`, after the key.
    fn row(text: &str, key: &str) -> String {
        let line = text
            .lines()
            .find(|l| l.split_whitespace().next() == Some(key))
            .unwrap_or_else(|| panic!("no {key} in {text}"));
        line.split_whitespace()
            .skip(1)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn done(reply: Reply) -> String {
        match reply {
            Reply::Done { text } => text,
            Reply::Refused { why } => panic!("refused: {why}"),
        }
    }

    fn refused(reply: Reply) -> String {
        match reply {
            Reply::Refused { why } => why,
            Reply::Done { text } => panic!("done: {text}"),
        }
    }

    #[test]
    fn set_settings_and_unset_go_through_the_daemon_and_are_events_with_who_made_them() {
        use temet_work_core::model::Role;
        let kept = Kept::default();
        let mut host = host_with(Box::new(kept.clone()));
        host.tick(0).expect("sync");
        assert_eq!(
            done(ask(&mut host, None, set("ankh#1", None, "model", "opus"))),
            "model is opus at ankh#1"
        );
        done(ask(
            &mut host,
            None,
            set("ankh", Some(Role::Reviewer), "review_rounds", "1"),
        ));
        let shown = done(ask(
            &mut host,
            None,
            Command::Settings {
                target: Target::parse("ankh#1"),
                role: Some(Role::Author),
            },
        ));
        assert!(shown.starts_with("ankh#1, author\n"), "{shown}");
        assert_eq!(row(&shown, "model"), "opus task, set by person");
        assert_eq!(row(&shown, "review_rounds"), "3 built in");
        assert_eq!(row(&shown, "runtime"), "fake built in");
        assert_eq!(
            row(&shown, "deploy"),
            "make deploy project",
            "from the file's project"
        );
        let for_reviewer = done(ask(
            &mut host,
            None,
            Command::Settings {
                target: Target::parse("ankh#1"),
                role: Some(Role::Reviewer),
            },
        ));
        assert_eq!(
            row(&for_reviewer, "review_rounds"),
            "1 project, set by person"
        );
        // `temet-work issue` shows the role at work on the issue: the refiner.
        let issue = done(ask(
            &mut host,
            None,
            Command::Issue {
                issue: IssueId::new("ankh", "1"),
            },
        ));
        assert!(issue.contains("Settings for the refiner:"), "{issue}");
        assert!(issue.contains("opus"), "{issue}");
        // The status lists the events' effect, with who made each.
        let status = done(ask(&mut host, None, Command::Status));
        assert!(
            status.contains("ankh#1: model = opus, set by person"),
            "{status}"
        );

        let unset = Command::Unset {
            target: Target::parse("ankh#1"),
            role: None,
            key: "model".into(),
        };
        assert_eq!(
            done(ask(&mut host, None, unset.clone())),
            "model is no longer set at ankh#1"
        );
        assert!(refused(ask(&mut host, None, unset)).contains("nothing to unset"));

        let logged = kept.0.lock().expect("journal").clone();
        let settings: Vec<&Event> = logged
            .iter()
            .filter(|e| matches!(e, Event::SettingSet { .. } | Event::SettingUnset { .. }))
            .collect();
        assert_eq!(settings.len(), 4, "{settings:?}");
        assert!(matches!(
            settings[0],
            Event::SettingSet { key, by, .. } if key == "deploy" && by == "file"
        ));
        assert!(matches!(
            settings[1],
            Event::SettingSet { scope: Scope::Task(_), value: Value::Text(v), by, .. } if v == "opus" && by == "person"
        ));
    }

    #[test]
    fn a_value_of_the_wrong_kind_an_unknown_key_or_an_adapter_the_build_lacks_is_refused_with_why()
    {
        let mut host = host();
        host.tick(0).expect("sync");
        assert!(
            refused(ask(
                &mut host,
                None,
                set("ankh", None, "review_rounds", "many")
            ))
            .contains("whole number")
        );
        assert!(
            refused(ask(&mut host, None, set("ankh", None, "colour", "red")))
                .contains("no setting colour")
        );
        let why = refused(ask(
            &mut host,
            None,
            set("global", None, "runtime", "nonesuch"),
        ));
        assert!(
            why.contains("nonesuch") && why.contains("this build has: fake"),
            "{why}"
        );
        let why = refused(ask(
            &mut host,
            None,
            set("global", None, "runner", "nonesuch"),
        ));
        assert!(why.contains("this build has: fake"), "{why}");
        assert!(
            refused(ask(&mut host, None, set("nowhere", None, "model", "x"))).contains("nowhere")
        );
        assert!(
            refused(ask(&mut host, None, set("ankh#9", None, "model", "x")))
                .contains("no issue ankh#9")
        );
        assert!(
            host.world().settings.values().all(|e| e.by == "file"),
            "only what the file gave"
        );
        // An agent that is not the issue's refiner may not change settings.
        let why = refused(ask(
            &mut host,
            Some("author-9"),
            set("ankh#1", None, "model", "x"),
        ));
        assert!(why.contains("not an agent"), "{why}");
    }

    #[test]
    fn a_refused_setting_exits_1_and_a_done_one_exits_0() {
        use temet_work::socket;
        let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("settings-exit.sock");
        let requests = socket::listen(&path).expect("listen");
        let run = {
            let path = path.clone();
            move |args: &[&str]| {
                std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
                    .args(args)
                    .env("TEMET_WORK_SOCKET", &path)
                    .env_remove("TEMET_WORK_AGENT")
                    .output()
                    .expect("runs")
            }
        };
        let client = std::thread::spawn({
            move || {
                (
                    run(&["set", "ankh", "review_rounds", "many"]),
                    run(&["set", "ankh", "review_rounds", "2"]),
                    run(&["settings", "ankh"]),
                    run(&["unset", "ankh", "review_rounds"]),
                )
            }
        });
        let mut host = host();
        host.tick(0).expect("sync");
        for _ in 0..4 {
            let pending = requests
                .recv_timeout(std::time::Duration::from_secs(5))
                .expect("a request");
            let reply = answer(&mut host, 1, &pending.request).expect("answer");
            pending.answer(reply);
        }
        let (bad, good, shown, unset) = client.join().expect("client");
        assert_eq!(bad.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&bad.stderr).contains("whole number"));
        assert_eq!(good.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&shown.stdout).contains("review_rounds  2"));
        assert_eq!(unset.status.code(), Some(0));
    }
}

/// A stub `ssh` in its own folder: it drops ssh's options and the host and runs
/// the rest here, with `temet-work` the one this build made and the daemon's
/// socket set, as the far machine's own environment would.
fn stub_ssh(name: &str, socket: &std::path::Path, body: Option<&str>) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("stub-{name}"));
    std::fs::create_dir_all(&dir).expect("dir");
    let script = body.map(String::from).unwrap_or_else(|| {
        format!(
            "#!/bin/sh\nwhile [ $# -gt 0 ]; do case \"$1\" in -o|-p) shift 2;; -*) shift;; *) break;; esac; done\nshift\nTEMET_WORK_SOCKET={} PATH={}:$PATH exec \"$@\"\n",
            socket.display(),
            std::path::Path::new(env!("CARGO_BIN_EXE_temet-work")).parent().expect("bin dir").display()
        )
    });
    let path = dir.join("ssh");
    std::fs::write(&path, script).expect("stub");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    path
}

#[test]
fn the_same_request_gets_equal_replies_through_the_socket_and_through_ssh() {
    use temet_work::socket::{self, Local, Ssh, Transport};
    use temet_work_core::{Command, Request};

    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("transports.sock");
    let requests = socket::listen(&path).expect("listen");
    let stub = stub_ssh("equal", &path, None);
    let client = {
        let path = path.clone();
        std::thread::spawn(move || {
            let request = Request {
                by: Some(AgentId::new("author-9")),
                command: Command::Status,
            };
            let local = Local { path }.send(&request);
            let ssh = Ssh {
                target: "stub".into(),
                port: None,
                socket: None,
                program: stub.display().to_string(),
            }
            .send(&request);
            (local, ssh)
        })
    };
    let mut host = host();
    host.tick(0).expect("tick");
    for _ in 0..2 {
        let pending = requests
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("a request");
        let reply = socket::answer(&mut host, 1, &pending.request).expect("answer");
        pending.answer(reply);
    }
    let (local, ssh) = client.join().expect("client");
    assert!(local.is_ok(), "{local:?}");
    assert_eq!(local, ssh);
}

#[test]
fn an_address_with_a_path_reaches_that_daemons_socket() {
    use temet_work::socket::{self, Ssh, Transport};
    use temet_work_core::{Command, Request};

    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("second.sock");
    let requests = socket::listen(&path).expect("listen");
    // The stub's own socket is another, dead one: only the path in the address can win.
    let dead = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("dead.sock");
    let stub = stub_ssh("second", &dead, None);
    let socket = path.display().to_string();
    let client = std::thread::spawn(move || {
        Ssh {
            target: "stub".into(),
            port: None,
            socket: Some(socket),
            program: stub.display().to_string(),
        }
        .send(&Request {
            by: None,
            command: Command::Status,
        })
    });
    let mut host = host();
    host.tick(0).expect("tick");
    let pending = requests
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("a request");
    let reply = socket::answer(&mut host, 1, &pending.request).expect("answer");
    pending.answer(reply);
    assert!(client.join().expect("client").is_ok());
}

#[test]
fn an_agent_commands_arrive_over_ssh_as_its_own() {
    use temet_work::socket;

    let sock = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ssh-agent.sock");
    let requests = socket::listen(&sock).expect("listen");
    let stub = stub_ssh("agent", &sock, None);
    let plan = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ssh-plan.yaml");
    std::fs::write(
        &plan,
        "checks:\n  - done_when: the clock strikes thirteen\n    proven_by: listening at midnight\n",
    )
    .expect("plan");
    let client = std::thread::spawn(move || {
        let run = |by: &str, args: &[&str]| {
            let path = format!(
                "{}:{}",
                stub.parent().expect("dir").display(),
                std::env::var("PATH").unwrap_or_default()
            );
            let out = std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
                .args(args)
                .env("PATH", path)
                .env("TEMET_WORK_DAEMON", "ssh://stub")
                .env_remove("TEMET_WORK_SOCKET")
                .env("TEMET_WORK_AGENT", by)
                .output()
                .expect("run");
            (
                out.status.code(),
                String::from_utf8_lossy(&out.stdout).trim().to_string(),
                String::from_utf8_lossy(&out.stderr).trim().to_string(),
            )
        };
        let plan_arg = plan.display().to_string();
        vec![
            run("refiner-1", &["plan", "ankh#1", &plan_arg]),
            run("author-1", &["ready", "ankh#1", "57"]),
            run("reviewer-1", &["review", "ankh#1", "approve"]),
        ]
    });
    let mut host = host();
    host.tick(0).expect("tick");
    let mut by = Vec::new();
    for _ in 0..3 {
        let pending = requests
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("a request");
        by.push(pending.request.by.clone().map(|a| a.0));
        let reply = socket::answer(&mut host, 1, &pending.request).expect("answer");
        pending.answer(reply);
    }
    assert_eq!(
        by,
        [
            Some("refiner-1".to_string()),
            Some("author-1".to_string()),
            Some("reviewer-1".to_string())
        ]
    );
    let got = client.join().expect("client");
    assert_eq!(got[0].0, Some(0), "{got:?}");
    assert_eq!(got[1].1, "ankh#1 is in review, round 1", "{got:?}");
    assert_eq!(got[2].0, Some(0), "{got:?}");
}

#[test]
fn a_failed_ssh_hop_says_which_step_failed() {
    use temet_work::socket::{Ssh, Transport};
    use temet_work_core::{Command, Request};

    let nowhere = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("nobody.sock");
    let _ = std::fs::remove_file(&nowhere);
    let request = Request {
        by: None,
        command: Command::Status,
    };
    let via = |name: &str, body: Option<&str>| {
        let stub = stub_ssh(name, &nowhere, body);
        Ssh {
            target: "stub".into(),
            port: None,
            socket: None,
            program: stub.display().to_string(),
        }
        .send(&request)
    };
    assert_eq!(
        via(
            "255",
            Some(
                "#!/bin/sh\necho 'ssh: connect to host stub port 22: Connection refused' >&2\nexit 255\n"
            )
        ),
        Err(
            "ssh stub: could not connect: ssh: connect to host stub port 22: Connection refused"
                .to_string()
        )
    );
    assert_eq!(
        via("127", Some("#!/bin/sh\necho 'sh: temet-work: not found' >&2\nexit 127\n")),
        Err("ssh stub: temet-work is not on the other machine's PATH: install it there, on the PATH a non-interactive ssh gets".to_string())
    );
    let e = via("nodaemon", None).expect_err("no daemon");
    assert!(
        e.starts_with("ssh stub: no temet-work daemon answers at ") && e.contains("nobody.sock"),
        "{e}"
    );
}

#[test]
fn relay_with_no_daemon_exits_non_zero_saying_so() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
        .arg("relay")
        .env(
            "TEMET_WORK_SOCKET",
            std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("none.sock"),
        )
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no temet-work daemon answers at "));
}
