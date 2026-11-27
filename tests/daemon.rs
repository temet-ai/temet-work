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
use temet_work_core::launch::SECRET_VAR;
use temet_work_core::model::Stage;
use temet_work_core::{Command, Host, Peer, Request, World};
use temet_work_runner::fake::{FakeRunner, Starts};
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

/// A host with every family's fake, and what its runner was asked to start.
fn host() -> (Host, Starts) {
    host_with(Box::new(Memory::default()))
}

fn host_with(journal: Box<dyn temet_work_core::Journal>) -> (Host, Starts) {
    let mut runtimes = temet_work_runtime::Runtimes::default();
    runtimes.add(Box::new(FakeRuntime));
    host_with_runtimes(journal, runtimes)
}

fn host_with_runtimes(
    journal: Box<dyn temet_work_core::Journal>,
    runtimes: temet_work_runtime::Runtimes,
) -> (Host, Starts) {
    host_on(journal, runtimes, CONFIG, None)
}

/// The secret the runner started `agent` with, from the environment it got.
fn secret(starts: &Starts, agent: &str) -> String {
    starts
        .lock()
        .expect("starts")
        .iter()
        .rev()
        .find(|s| s.labels.get("agent").map(String::as_str) == Some(agent))
        .and_then(|s| {
            s.program
                .env
                .iter()
                .find(|(k, _)| k == SECRET_VAR)
                .map(|(_, v)| v.clone())
        })
        .unwrap_or_else(|| panic!("{agent} was not started"))
}

/// A request from the agent `by`, carrying the secret it was started with, or
/// from a person.
fn from(starts: &Starts, by: Option<&str>, command: Command) -> Request {
    Request {
        by: by.map(AgentId::new),
        secret: by.map(|a| secret(starts, a)),
        relayed: false,
        command,
    }
}

fn host_on(
    journal: Box<dyn temet_work_core::Journal>,
    runtimes: temet_work_runtime::Runtimes,
    config: &str,
    daemon_host: Option<&str>,
) -> (Host, Starts) {
    host_from(
        World::default(),
        journal,
        runtimes,
        config,
        daemon_host,
        vec![tracked("city/ankh", "1", 1)],
    )
}

/// An open issue `key` of the tracker's project `name`, `order`th oldest.
fn tracked(name: &str, key: &str, order: u64) -> (String, TrackedIssue) {
    (
        name.into(),
        TrackedIssue {
            key: key.into(),
            title: "Mend the clock".into(),
            body: String::new(),
            milestone: None,
            depends_on: vec![],
            order,
            closed: false,
        },
    )
}

/// A host on `world`, with every family's fake and the tracker's `issues`.
fn host_from(
    world: World,
    journal: Box<dyn temet_work_core::Journal>,
    runtimes: temet_work_runtime::Runtimes,
    config: &str,
    daemon_host: Option<&str>,
    issues: Vec<(String, TrackedIssue)>,
) -> (Host, Starts) {
    host_tracking(
        world,
        journal,
        runtimes,
        config,
        daemon_host,
        FakeTracker {
            issues,
            ..FakeTracker::default()
        },
    )
}

/// A host on `world`, with every family's fake and `tracker` as its tracker.
fn host_tracking(
    world: World,
    journal: Box<dyn temet_work_core::Journal>,
    runtimes: temet_work_runtime::Runtimes,
    config: &str,
    daemon_host: Option<&str>,
    tracker: FakeTracker,
) -> (Host, Starts) {
    let mut trackers = temet_work_tracker::Trackers::default();
    trackers.add(Box::new(tracker));
    let mut runners = temet_work_runner::Runners::default();
    let fake = FakeRunner::default();
    let starts = fake.starts();
    runners.add(Box::new(fake));
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
        world,
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
    (host, starts)
}

#[test]
fn a_synced_issue_gets_a_refiner_in_a_labelled_terminal() {
    let (mut host, _) = host();
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

/// An older log holds plain models and sets no global runtime. The daemon puts
/// its build's runtimes in the world before it replays the log, so each plain
/// model is the build's default runtime's.
#[cfg(feature = "store")]
#[test]
fn a_start_over_a_log_with_no_global_runtime_binds_its_plain_models_to_the_builds_default() {
    use temet_work_core::event::Event;
    use temet_work_core::ids::{GroupId, ProjectId};
    use temet_work_core::model::Role;
    use temet_work_core::settings::{Scope, Value};
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("plain-model-start");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    let log = dir.join("events.jsonl");
    let ankh = ProjectId::new("ankh");
    let issue = IssueId::new("ankh", "1");
    let model = |scope: Scope, role: Option<Role>, value: &str, by: &str| Event::SettingSet {
        scope,
        role,
        key: "model".into(),
        value: Value::Text(value.into()),
        by: by.into(),
    };
    {
        let (mut written, _) = temet_work_store::Log::open(&log).expect("log");
        for event in [
            Event::ProjectAdded {
                project: ankh.clone(),
                tracker: "fake".into(),
                name: "city/ankh".into(),
                code_host: None,
                repo: None,
            },
            Event::GroupCreated {
                group: GroupId::new("watch"),
                name: "Night watch".into(),
                projects: vec![ankh.clone()],
                areas: Vec::new(),
                milestones: Vec::new(),
                refining_at_once: 1,
                model: None,
                queue_low: None,
                investigate_every: None,
            },
            Event::IssueSynced {
                issue: issue.clone(),
                title: "Mend the clock".into(),
                milestone: None,
                depends_on: vec![],
                order: 1,
                closed: false,
            },
            model(Scope::Task(issue.clone()), None, "opus", "refiner-1"),
            model(
                Scope::Project(ankh.clone()),
                Some(Role::Reviewer),
                "haiku",
                "person",
            ),
        ] {
            written.append(0, &event).expect("logged");
        }
    }
    let host = temet_work::start(&log, vec![Box::new(fake_runtimes())]).expect("start");
    let w = host.world();
    let kept: Vec<(Option<Role>, &str, String)> = w
        .settings
        .iter()
        .map(|((_, role, key), e)| (*role, key.as_str(), e.value.to_string()))
        .collect();
    assert_eq!(
        kept,
        [
            (Some(Role::Reviewer), "model.fake", "haiku".to_string()),
            (Some(Role::Author), "model.fake", "opus".to_string()),
        ]
    );
    let brief = |role| {
        w.launch(&GroupId::new("watch"), role, &issue, None)
            .expect("known")
            .brief
            .model
    };
    assert_eq!(brief(Role::Author).as_deref(), Some("opus"));
    assert_eq!(brief(Role::Reviewer).as_deref(), Some("haiku"));
}

/// A reviewer set to run Codex starts as the Codex program in a labelled
/// terminal; the author of the same issue keeps the build's default runtime.
#[cfg(feature = "runtime-codex")]
#[test]
fn a_reviewer_set_to_codex_starts_the_codex_program() {
    use temet_work::socket::{Reply, answer};
    use temet_work_core::model::{Check, Plan, Role};
    use temet_work_core::settings::Target;

    let mut runtimes = temet_work_runtime::Runtimes::default();
    runtimes.add(Box::new(FakeRuntime));
    runtimes.add(Box::new(temet_work_runtime_codex::Codex));
    let (mut host, starts) = host_with_runtimes(Box::new(Memory::default()), runtimes);
    let run = |host: &mut Host, by: Option<&str>, command: Command| {
        let request = from(&starts, by, command);
        match answer(host, 1, &request, &Peer::Unchecked).expect("answer") {
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
            pulls: Default::default(),
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

/// A person keeps the project's agents in plan mode. A refiner cannot move the
/// issue to Codex, which ignores `permission`, so the author runs Claude in it.
#[cfg(all(feature = "runtime-claude", feature = "runtime-codex"))]
#[test]
fn a_refiners_plan_naming_runtime_is_refused_and_the_author_runs_claude_in_plan_mode() {
    use temet_work::socket::{Reply, answer};
    use temet_work_core::model::{Check, Plan};
    use temet_work_core::settings::{Scope, Target, Value};
    use temet_work_core::{Event, text};

    // The Claude adapter marks the folder trusted in this home, never the real one.
    let home = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("runtime-plan-home");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).expect("home");
    let mut runtimes = temet_work_runtime::Runtimes::default().with_home(&home);
    runtimes.add(Box::new(temet_work_runtime_claude::Claude));
    runtimes.add(Box::new(temet_work_runtime_codex::Codex));
    let kept = Kept::default();
    let (mut host, starts) = host_with_runtimes(Box::new(kept.clone()), runtimes);
    let run = |host: &mut Host, by: Option<&str>, command: Command| {
        let request = from(&starts, by, command);
        answer(host, 1, &request, &Peer::Unchecked).expect("answer")
    };
    let done = |reply: Reply| match reply {
        Reply::Done { text } => text,
        Reply::Refused { why } => panic!("refused: {why}"),
    };
    done(run(
        &mut host,
        None,
        Command::Set {
            target: Target::parse("ankh"),
            role: None,
            key: "permission".into(),
            value: "plan".into(),
        },
    ));
    host.tick(0).expect("sync");
    let issue = IssueId::new("ankh", "1");
    let plan = |settings: &[(&str, &str)]| Command::Plan {
        issue: issue.clone(),
        plan: Plan {
            repos: Vec::new(),
            settings: settings
                .iter()
                .map(|(k, v)| (k.to_string(), Value::Text(v.to_string())))
                .collect(),
            checks: vec![Check {
                done_when: "the clock strikes thirteen".into(),
                proven_by: "listening at midnight".into(),
            }],
        },
    };

    let logged = kept.0.lock().expect("log").len();
    let refused = run(&mut host, Some("refiner-1"), plan(&[("runtime", "codex")]));
    assert!(
        matches!(&refused, Reply::Refused { why } if why.contains(
            "only a person sets runtime, because a runtime may not follow permission"
        )),
        "{refused:?}"
    );
    assert_eq!(host.world().issues[&issue].stage, Stage::Refining);
    assert_eq!(host.world().issues[&issue].plan, None);
    let task = Scope::Task(issue.clone());
    assert!(
        !host
            .world()
            .settings
            .keys()
            .any(|(at, _, key)| *at == task && key == "runtime"),
        "no task-level runtime"
    );
    let since: Vec<Event> = kept.0.lock().expect("log")[logged..].to_vec();
    assert!(
        !since
            .iter()
            .any(|e| matches!(e, Event::SettingSet { .. } | Event::PlanWritten { .. })),
        "{since:?}"
    );

    done(run(&mut host, Some("refiner-1"), plan(&[])));
    let author = done(run(
        &mut host,
        None,
        Command::Screen {
            agent: AgentId::new("author-1"),
        },
    ));
    assert!(
        author.contains("$ claude --permission-mode plan "),
        "{author}"
    );
    assert!(home.join(".claude.json").exists(), "trusted in the scratch");
    assert!(
        host.problems().is_empty(),
        "{}",
        text::text(host.world(), host.problems())
    );
}

#[test]
fn a_folder_on_another_machine_needs_daemon_host_before_an_agent_starts_there() {
    let config = CONFIG.replace("path = \".\"", "machine = \"mac\"\npath = \"/work/ankh\"");
    let (mut host, _) = host_on(Box::new(Memory::default()), fake_runtimes(), &config, None);
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
    let (mut host, _) = host_on(
        Box::new(Memory::default()),
        fake_runtimes(),
        &config,
        Some("omit"),
    );
    host.tick(0).expect("tick");
    // The refiner, and the investigator of a group short of work: both on mac.
    let machines: Vec<&str> = host
        .world()
        .agents
        .values()
        .map(|a| a.machine.0.as_str())
        .collect();
    assert_eq!(machines, ["mac", "mac"], "{:?}", host.problems());
}

#[test]
fn commands_reach_the_daemon_over_its_socket() {
    use temet_work::socket::{self, Reply};

    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("daemon-test.sock");
    let requests = socket::listen(&path).expect("listen");
    assert!(socket::listen(&path).is_err(), "a second daemon is refused");
    let (mut host, starts) = host();
    host.tick(0).expect("tick");
    let client = {
        let path = path.clone();
        std::thread::spawn(move || {
            let ask = |command| {
                socket::send(&path, &from(&starts, Some("refiner-1"), command)).expect("reply")
            };
            (
                ask(Command::Status),
                ask(Command::Ready {
                    issue: IssueId::new("ankh", "1"),
                    pull: Some("57".into()),
                    pulls: Default::default(),
                }),
            )
        })
    };
    for _ in 0..2 {
        let pending = requests
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("a request");
        let reply = socket::answer(&mut host, 1, &pending.request, &pending.peer).expect("answer");
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
            why: "ready for ankh#1 is the author's: no author is working on it; \
                  you are refiner-1"
                .into()
        }
    );
}

#[test]
fn an_issue_goes_from_plan_through_two_review_rounds_with_one_reviewer() {
    use temet_work::socket::{Reply, answer};
    use temet_work_core::Verdict;
    use temet_work_core::model::{Check, Plan};

    let kept = Kept::default();
    let (mut host, starts) = host_with(Box::new(kept.clone()));
    host.tick(0).expect("sync");
    let issue = IssueId::new("ankh", "1");
    let run = |host: &mut Host, by: &str, command: Command| {
        let request = from(&starts, Some(by), command);
        match answer(host, 1, &request, &Peer::Unchecked).expect("answer") {
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
        pulls: Default::default(),
    };
    // The refiner was stopped once its plan was in, so nothing may name it any
    // more; the author's own ready is accepted.
    let wrong = from(&starts, Some("refiner-1"), ready.clone());
    assert_eq!(
        answer(&mut host, 1, &wrong, &Peer::Unchecked).expect("answer"),
        Reply::Refused {
            why: "refiner-1 has ended, so no request may name it; an agent the issue \
                  still needs is started again with a new id"
                .into()
        }
    );
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
    use temet_work_core::event::Event;
    use temet_work_core::settings::{Scope, Target, Value};

    fn ask(host: &mut Host, by: Option<&str>, command: Command) -> Reply {
        let request = Request::new(by.map(AgentId::new), command);
        answer(host, 1, &request, &Peer::Unchecked).expect("answer")
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
        let (mut host, _) = host_with(Box::new(kept.clone()));
        host.tick(0).expect("sync");
        // A plain model is the global runtime's, the build's default here, and
        // the reply names the key it is recorded under.
        assert_eq!(
            done(ask(&mut host, None, set("ankh#1", None, "model", "opus"))),
            "model.fake is opus at ankh#1"
        );
        done(ask(
            &mut host,
            None,
            set("ankh", Some(Role::Reviewer), "starts", "1"),
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
        assert_eq!(row(&shown, "starts"), "3 built in");
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
        assert_eq!(row(&for_reviewer, "starts"), "1 project, set by person");
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
            status.contains("ankh#1: model.fake = opus, set by person"),
            "{status}"
        );

        let unset = Command::Unset {
            target: Target::parse("ankh#1"),
            role: None,
            key: "model".into(),
        };
        assert_eq!(
            done(ask(&mut host, None, unset.clone())),
            "model.fake is no longer set at ankh#1"
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
            Event::SettingSet { scope: Scope::Task(_), key, value: Value::Text(v), by, .. }
                if key == "model.fake" && v == "opus" && by == "person"
        ));
        assert!(matches!(
            settings[3],
            Event::SettingUnset { key, .. } if key == "model.fake"
        ));
    }

    #[test]
    fn a_refiners_plan_model_leaves_the_persons_reviewer_model() {
        use temet_work_core::model::{Check, Plan, Role};
        let (mut host, starts) = host();
        host.tick(0).expect("sync");
        for role in [Role::Reviewer, Role::Arbiter] {
            done(ask(
                &mut host,
                None,
                set("ankh", Some(role), "model", "opus"),
            ));
        }
        let refiner = |host: &mut Host, command: Command| {
            let request = from(&starts, Some("refiner-1"), command);
            answer(host, 1, &request, &Peer::Unchecked).expect("answer")
        };
        for role in [None, Some(Role::Author)] {
            assert_eq!(
                done(refiner(&mut host, set("ankh#1", role, "model", "haiku"))),
                "model.fake is haiku at ankh#1 for author"
            );
        }
        let why = refused(refiner(
            &mut host,
            set("ankh#1", Some(Role::Reviewer), "model", "haiku"),
        ));
        assert!(why.contains("are a person's to set"), "{why}");
        let unset = Command::Unset {
            target: Target::parse("ankh#1"),
            role: None,
            key: "model".into(),
        };
        assert_eq!(
            done(refiner(&mut host, unset)),
            "model.fake is no longer set at ankh#1 for author"
        );
        let plan = Command::Plan {
            issue: IssueId::new("ankh", "1"),
            plan: Plan {
                repos: Vec::new(),
                settings: [("model".to_string(), Value::Text("sonnet".into()))].into(),
                checks: vec![Check {
                    done_when: "the clock strikes thirteen".into(),
                    proven_by: "listening at midnight".into(),
                }],
            },
        };
        done(refiner(&mut host, plan));
        let mut shown = |role| {
            let command = Command::Settings {
                target: Target::parse("ankh#1"),
                role: Some(role),
            };
            row(&done(ask(&mut host, None, command)), "model")
        };
        assert_eq!(shown(Role::Reviewer), "opus project, set by person");
        assert_eq!(shown(Role::Author), "sonnet task, set by refiner-1");
        let status = done(ask(&mut host, None, Command::Status));
        assert!(
            status.contains("ankh#1, author: model.fake = sonnet, set by refiner-1\n"),
            "{status}"
        );
    }

    #[test]
    fn a_value_of_the_wrong_kind_an_unknown_key_or_an_adapter_the_build_lacks_is_refused_with_why()
    {
        let (mut host, _) = host();
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
                    .env_remove(SECRET_VAR)
                    .env_remove("TEMET_WORK_DAEMON")
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
        let (mut host, _) = host();
        host.tick(0).expect("sync");
        for _ in 0..4 {
            let pending = requests
                .recv_timeout(std::time::Duration::from_secs(5))
                .expect("a request");
            let reply = answer(&mut host, 1, &pending.request, &pending.peer).expect("answer");
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

    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("transports.sock");
    let requests = socket::listen(&path).expect("listen");
    let stub = stub_ssh("equal", &path, None);
    let (mut host, starts) = host();
    host.tick(0).expect("tick");
    let client = {
        let path = path.clone();
        std::thread::spawn(move || {
            let request = from(&starts, Some("refiner-1"), Command::Status);
            let local = Local { path }.send(&request);
            let ssh = Ssh {
                target: "stub".into(),
                port: None,
                socket: None,
                program: stub.display().to_string(),
                deadline: socket::SSH_DEADLINE,
            }
            .send(&request);
            (local, ssh)
        })
    };
    for _ in 0..2 {
        let pending = requests
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("a request");
        let reply = socket::answer(&mut host, 1, &pending.request, &pending.peer).expect("answer");
        pending.answer(reply);
    }
    let (local, ssh) = client.join().expect("client");
    assert!(
        matches!(&local, Ok(socket::Reply::Done { text }) if text.contains("refiner-1 on ankh#1")),
        "{local:?}"
    );
    assert_eq!(local, ssh);
}

#[test]
fn an_address_with_a_path_reaches_that_daemons_socket() {
    use temet_work::socket::{self, Ssh, Transport};

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
            deadline: socket::SSH_DEADLINE,
        }
        .send(&Request::new(None, Command::Status))
    });
    let (mut host, _) = host();
    host.tick(0).expect("tick");
    let pending = requests
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("a request");
    let reply = socket::answer(&mut host, 1, &pending.request, &pending.peer).expect("answer");
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
    let (mut host, starts) = host();
    host.tick(0).expect("tick");
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
                .env(SECRET_VAR, secret(&starts, by))
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
    let mut by = Vec::new();
    for _ in 0..3 {
        let pending = requests
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("a request");
        by.push(pending.request.by.clone().map(|a| a.0));
        assert!(pending.request.relayed, "marked by the relay");
        assert!(matches!(pending.peer, Peer::Read(_)), "{:?}", pending.peer);
        let reply = socket::answer(&mut host, 1, &pending.request, &pending.peer).expect("answer");
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
    use temet_work::socket::{self, Ssh, Transport};

    let nowhere = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("nobody.sock");
    let _ = std::fs::remove_file(&nowhere);
    let request = Request::new(None, Command::Status);
    let via = |name: &str, body: Option<&str>| {
        let stub = stub_ssh(name, &nowhere, body);
        Ssh {
            target: "stub".into(),
            port: None,
            socket: None,
            program: stub.display().to_string(),
            deadline: socket::SSH_DEADLINE,
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
fn a_hop_sends_the_clients_build_and_machine_name_on_the_relay_call() {
    use temet_work::socket::{self, Ssh, Transport};
    use temet_work_core::{Command, Request};

    let args = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("relay-call.args");
    let body = format!(
        r#"#!/bin/sh
printf '%s\n' "$@" >{}
cat >/dev/null
printf '{{"reply":"done","text":"all quiet"}}\n'
"#,
        args.display()
    );
    let stub = stub_ssh("relay-call", &args, Some(&body));
    Ssh {
        target: "stub".into(),
        port: None,
        socket: None,
        program: stub.display().to_string(),
        deadline: socket::SSH_DEADLINE,
    }
    .send(&Request::new(None, Command::Status))
    .expect("a reply");
    let args = std::fs::read_to_string(args).expect("args");
    let args: Vec<&str> = args.lines().collect();
    let uname = std::process::Command::new("uname")
        .arg("-n")
        .output()
        .expect("uname");
    let name = String::from_utf8_lossy(&uname.stdout);
    let call = [
        "temet-work",
        "relay",
        "--build",
        temet_work::BUILD,
        "--host",
        name.trim(),
    ];
    assert!(args.ends_with(&call), "{args:?}");
}

/// A stub `ssh` whose far end is the real `temet-work relay`, told that the
/// caller is build `build` on the machine `lancre`.
fn relay_stub(name: &str, socket: &std::path::Path, build: &str) -> std::path::PathBuf {
    let body = format!(
        "#!/bin/sh\nTEMET_WORK_SOCKET={} exec {} relay --build {build} --host lancre\n",
        socket.display(),
        env!("CARGO_BIN_EXE_temet-work")
    );
    stub_ssh(name, socket, Some(&body))
}

fn other_build_refusal() -> String {
    format!(
        "temet-work on lancre is build other, the daemon is build {}: update it",
        temet_work::BUILD
    )
}

#[test]
fn a_client_at_a_different_build_is_refused_and_nothing_reaches_the_daemon() {
    use temet_work::socket::{self, Reply, Ssh, Transport};
    use temet_work_core::{Command, Request};

    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("other-build.sock");
    let requests = socket::listen(&path).expect("listen");
    let stub = relay_stub("other-build", &path, "other");
    let reply = Ssh {
        target: "stub".into(),
        port: None,
        socket: None,
        program: stub.display().to_string(),
        deadline: socket::SSH_DEADLINE,
    }
    .send(&Request::new(None, Command::Status));
    assert_eq!(
        reply,
        Ok(Reply::Refused {
            why: other_build_refusal()
        })
    );
    assert!(requests.try_recv().is_err(), "the request was forwarded");
}

#[test]
fn a_person_at_a_different_build_reads_one_line_and_exit_code_1() {
    let sock = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("other-build-cli.sock");
    let _requests = temet_work::socket::listen(&sock).expect("listen");
    let stub = relay_stub("other-build-cli", &sock, "other");
    let path = format!(
        "{}:{}",
        stub.parent().expect("dir").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
        .arg("status")
        .env("PATH", path)
        .env("TEMET_WORK_DAEMON", "ssh://stub")
        .env_remove("TEMET_WORK_AGENT")
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        format!("refused: {}\n", other_build_refusal())
    );
    assert!(out.stdout.is_empty());
}

#[test]
fn the_relay_compares_the_client_with_the_daemons_build_not_its_own() {
    use temet_work::socket::{self, Reply, Ssh, Transport};

    assert_ne!("octarine", temet_work::BUILD);
    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("daemon-build.sock");
    let requests = socket::listen_as(&path, "octarine").expect("listen");
    let send_as = |name: &str, build: &str| {
        let stub = relay_stub(name, &path, build);
        std::thread::spawn(move || {
            Ssh {
                target: "stub".into(),
                port: None,
                socket: None,
                program: stub.display().to_string(),
                deadline: socket::SSH_DEADLINE,
            }
            .send(&Request::new(None, Command::Status))
        })
    };

    let client = send_as("at-the-daemons-build", "octarine");
    let pending = requests
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("a request");
    let (mut host, _) = host();
    host.tick(0).expect("tick");
    let answer = socket::answer(&mut host, 1, &pending.request, &pending.peer).expect("answer");
    pending.answer(answer.clone());
    assert!(matches!(answer, Reply::Done { .. }), "{answer:?}");
    assert_eq!(client.join().expect("client"), Ok(answer));

    let client = send_as("at-the-relays-build", temet_work::BUILD);
    assert_eq!(
        client.join().expect("client"),
        Ok(Reply::Refused {
            why: format!(
                "temet-work on lancre is build {}, the daemon is build octarine: update it",
                temet_work::BUILD
            )
        })
    );
    assert!(requests.try_recv().is_err(), "the request was forwarded");
}

#[test]
fn relay_with_no_daemon_exits_non_zero_saying_so() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
        .args(["relay", "--build", temet_work::BUILD])
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

/// A deploy that restarts the daemon: the old daemon hands the restart off and
/// records nothing; the new one, with the new build, records Deployed and runs
/// verify once.
#[test]
fn a_deploy_that_restarts_the_daemon_is_recorded_by_the_new_one() {
    use temet_work_core::event::Event;
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("handoff");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    let handoff = dir.join("deploy-handoff");
    let mut world = World::default();
    let events = Config::parse(CONFIG)
        .expect("config")
        .events(&World::default());
    for event in &events {
        world.apply(event);
    }
    world.apply(&Event::IssueSynced {
        issue: IssueId::new("ankh", "1"),
        title: "Mend the clock".into(),
        milestone: None,
        depends_on: vec![],
        order: 1,
        closed: false,
    });
    let issue = IssueId::new("ankh", "1");
    world.issues.get_mut(&issue).expect("issue").stage = Stage::Deploying;

    let verifies = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let shell = |file: std::path::PathBuf,
                 verifies: std::sync::Arc<std::sync::atomic::AtomicUsize>| {
        std::sync::Arc::new(
            move |command: &str, _: &std::path::PathBuf, _: &[(String, String)]| {
                if command == "make deploy" {
                    std::fs::write(&file, "ankh#1 new\n").map_err(|e| e.to_string())
                } else {
                    verifies.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                }
            },
        )
    };
    let part = |build: &str| {
        temet_work_deploy::Deploys::with(shell(handoff.clone(), verifies.clone()))
            .handoff(handoff.clone(), build)
    };
    let mut config = CONFIG.replace(
        "deploy = \"make deploy\"",
        "deploy = \"make deploy\"\nverify = \"make smoke\"",
    );
    config.push('\n');
    let settle = |host: &mut Host, rounds: u64, until: Stage| {
        for now in 0..rounds {
            host.tick(now).expect("tick");
            if host.world().issues[&issue].stage == until {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    };

    let mut old = Host::new(
        world.clone(),
        vec![Box::new(part("old"))],
        Box::new(Memory::default()),
    );
    for event in Config::parse(&config).expect("config").events(old.world()) {
        old.handle(0, event).expect("config event");
    }
    settle(&mut old, 100, Stage::Verifying);
    assert_eq!(
        old.world().issues[&issue].stage,
        Stage::Deploying,
        "not deployed by the old one"
    );
    assert_eq!(verifies.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert!(handoff.exists());

    // The restart: the old daemon is gone; the new one has the log's state.
    let world = old.world().clone();
    drop(old);
    let mut new = Host::new(
        world,
        vec![Box::new(part("new"))],
        Box::new(Memory::default()),
    );
    settle(&mut new, 200, Stage::Done);
    assert_eq!(
        new.world().issues[&issue].stage,
        Stage::Done,
        "{:?}",
        new.problems()
    );
    assert_eq!(
        verifies.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "verify ran once"
    );
    assert!(!handoff.exists());
}

#[test]
fn an_overturn_and_a_keep_go_through_the_daemon_and_reach_the_next_brief() {
    use temet_work::socket::{Reply, answer};
    use temet_work_core::Event;
    use temet_work_core::ids::GroupId;
    use temet_work_core::model::{Decision, DecisionKind, Role};

    let kept = Kept::default();
    let (mut host, starts) = host_with(Box::new(kept.clone()));
    host.tick(0).expect("sync");
    let issue = IssueId::new("ankh", "1");
    host.handle(
        0,
        Event::DecisionMade {
            decision: Decision {
                id: temet_work_core::ids::DecisionId(1),
                issue: issue.clone(),
                by: AgentId::new("refiner-1"),
                kind: DecisionKind::Choice,
                question: "Which connection?".into(),
                options: vec!["second".into(), "legacy".into()],
                chosen: 1,
                reason: "easiest to undo".into(),
                overturned_to: None,
                overturn_reason: String::new(),
            },
        },
    )
    .expect("decision");
    let mut ask = |by: Option<&str>, command| {
        let request = from(&starts, by, command);
        answer(&mut host, 1, &request, &Peer::Unchecked).expect("answer")
    };
    let overturn = Command::Overturn {
        decision: temet_work_core::ids::DecisionId(1),
        option: 1,
        reason: "legacy breaks channels".into(),
    };
    assert!(matches!(
        ask(Some("refiner-1"), overturn.clone()),
        Reply::Refused { why } if why.contains("a person's")
    ));
    assert!(matches!(ask(None, overturn), Reply::Done { .. }));
    let Reply::Done { text } = ask(None, Command::Status) else {
        panic!("status refused");
    };
    assert!(
        text.contains("overturned to option 1 (second): legacy breaks channels"),
        "{text}"
    );
    assert!(
        text.contains("1 proposed: Which connection? Prefer: second."),
        "{text}"
    );
    let keep = Command::Proposal {
        rule: temet_work_core::ids::RuleId(1),
        keep: true,
    };
    assert!(matches!(ask(None, keep), Reply::Done { .. }));
    assert!(host.problems().is_empty(), "{:?}", host.problems());
    let brief = host
        .world()
        .launch(&GroupId::new("watch"), Role::Author, &issue, None)
        .expect("launch")
        .brief
        .task;
    assert!(
        brief.contains("Rules from overturned decisions:\n- Which connection? Prefer: second."),
        "{brief}"
    );
    // The log alone rebuilds the same rules.
    let events = kept.0.lock().expect("log").clone();
    let (replayed, _) = temet_work_core::World::replay(&events);
    assert_eq!(replayed.rules, host.world().rules);
}

/// An area of two repositories, ui and svc, whose issue changes both.
mod two_repositories {
    use super::*;
    use std::sync::{Arc, Mutex};
    use temet_work::socket::{Reply, answer};
    use temet_work_core::Verdict;
    use temet_work_core::ids::{FolderId, ProjectId};
    use temet_work_core::model::{Check, Plan};

    const AREA: &str = r#"
[[area]]
id = "guild"
projects = ["ui", "svc"]
tracker = "fake"
tracker_project = "GLD"

[area.settings]
deploy = "make deploy"

[[project]]
id = "ui"
name = "group/ui"
code_host = "fake"

[[project]]
id = "svc"
name = "group/svc"
code_host = "fake"

[[folder]]
id = "ui-1"
project = "ui"
path = "."

[[folder]]
id = "svc-1"
project = "svc"
path = "."

[[group]]
id = "watch"
name = "Night watch"
areas = ["guild"]
"#;

    struct Fixture {
        host: Host,
        starts: Starts,
        merges: Arc<Mutex<Vec<String>>>,
        mains: Arc<Mutex<std::collections::BTreeMap<String, Checks>>>,
        deploys: Arc<Mutex<usize>>,
        kept: Kept,
    }

    fn pull(number: &str, checks: Checks) -> Pull {
        Pull {
            number: number.into(),
            state: PullState::Open,
            checks,
            conflicting: false,
        }
    }

    fn fixture(ui: Checks, svc: Checks) -> Fixture {
        let mut trackers = temet_work_tracker::Trackers::default();
        trackers.add(Box::new(FakeTracker {
            issues: vec![(
                "GLD".into(),
                TrackedIssue {
                    key: "GLD-1".into(),
                    title: "Show the guild on both".into(),
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
        let runner = FakeRunner::default();
        let starts = runner.starts();
        runners.add(Box::new(runner));
        let fake = FakeCodeHost {
            pulls: vec![
                ("group/ui".into(), pull("12", ui)),
                ("group/svc".into(), pull("34", svc)),
            ],
            ..FakeCodeHost::default()
        };
        let merges = fake.merged_repos.clone();
        let mains = fake.mains.clone();
        let mut code_hosts = temet_work_codehost::CodeHosts::default().every(0);
        code_hosts.add(Box::new(fake));
        let deploys = Arc::new(Mutex::new(0));
        let counted = deploys.clone();
        let deploy = temet_work_deploy::Deploys::with(Arc::new(
            move |_: &str, _: &std::path::PathBuf, _: &[(String, String)]| {
                *counted.lock().expect("lock") += 1;
                Ok(())
            },
        ));
        let kept = Kept::default();
        let mut host = Host::new(
            World::default(),
            vec![
                Box::new(trackers),
                Box::new(runners.daemon_host(None)),
                Box::new(fake_runtimes()),
                Box::new(code_hosts),
                Box::new(deploy),
            ],
            Box::new(kept.clone()),
        );
        for event in Config::parse(AREA).expect("config").events(host.world()) {
            host.handle(0, event).expect("config event");
        }
        Fixture {
            host,
            starts,
            merges,
            mains,
            deploys,
            kept,
        }
    }

    fn run(f: &mut Fixture, by: &str, command: Command) -> Reply {
        let request = from(&f.starts, Some(by), command);
        answer(&mut f.host, 1, &request, &Peer::Unchecked).expect("answer")
    }

    fn done(reply: Reply) -> String {
        match reply {
            Reply::Done { text } => text,
            Reply::Refused { why } => panic!("refused: {why}"),
        }
    }

    /// Takes GLD-1 to the author's `ready` naming both pull requests.
    fn in_review(f: &mut Fixture) -> IssueId {
        f.host.tick(0).expect("sync");
        let issue = IssueId::new("guild", "GLD-1");
        done(run(
            f,
            "refiner-1",
            Command::Plan {
                issue: issue.clone(),
                plan: Plan {
                    repos: vec![ProjectId::new("ui"), ProjectId::new("svc")],
                    settings: Default::default(),
                    checks: vec![Check {
                        done_when: "both show the guild".into(),
                        proven_by: "looking".into(),
                    }],
                },
            },
        ));
        let i = &f.host.world().issues[&issue];
        assert_eq!(i.folder, Some(FolderId::new("ui-1")));
        assert_eq!(i.other_folders, vec![FolderId::new("svc-1")]);
        let ready = Command::Ready {
            issue: issue.clone(),
            pull: None,
            pulls: [
                (ProjectId::new("ui"), "12".to_string()),
                (ProjectId::new("svc"), "34".to_string()),
            ]
            .into(),
        };
        done(run(f, "author-1", ready));
        issue
    }

    #[test]
    fn both_merge_in_order_after_uis_main_pipeline_and_deploy_runs_once() {
        let mut f = fixture(Checks::Passed, Checks::Passed);
        f.mains
            .lock()
            .expect("lock")
            .insert("group/ui".into(), Checks::Pending);
        let issue = in_review(&mut f);
        done(run(
            &mut f,
            "reviewer-1",
            Command::Review {
                issue: issue.clone(),
                verdict: Verdict::Approve,
            },
        ));
        f.host.tick(1).expect("CI read, then ui merges");
        f.host.tick(2).expect("svc waits for ui's main pipeline");
        assert_eq!(*f.merges.lock().expect("lock"), ["group/ui"]);
        assert_eq!(f.host.world().issues[&issue].stage, Stage::Merging);
        assert!(f.host.problems().is_empty(), "{:?}", f.host.problems());
        f.mains
            .lock()
            .expect("lock")
            .insert("group/ui".into(), Checks::Passed);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let mut now = 3;
        while f.host.world().issues[&issue].stage != Stage::Done
            && std::time::Instant::now() < deadline
        {
            f.host.tick(now).expect("tick");
            now += 1;
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(
            *f.merges.lock().expect("lock"),
            ["group/ui", "group/svc"],
            "the plan's order"
        );
        assert_eq!(f.host.world().issues[&issue].stage, Stage::Done);
        assert_eq!(*f.deploys.lock().expect("lock"), 1, "after the last merge");
        assert!(f.host.world().folders.values().all(|f| f.holder.is_none()));
        let logged = f.kept.0.lock().expect("journal").clone();
        let mut replayed = World::replay(&logged).0;
        replayed.catalog = f.host.world().catalog.clone();
        assert_eq!(&replayed, f.host.world());
    }

    #[test]
    fn ci_failing_on_the_second_repository_sends_the_author_back_naming_it() {
        let mut f = fixture(
            Checks::Passed,
            Checks::Failed {
                failing: vec!["rust".into()],
            },
        );
        let issue = in_review(&mut f);
        f.host.tick(1).expect("CI read");
        let i = &f.host.world().issues[&issue];
        assert_eq!(i.checks_ok, Some(false));
        assert!(i.fix.iter().any(|p| p.contains("svc")), "{:?}", i.fix);
        assert!(f.merges.lock().expect("lock").is_empty());
    }
}

/// Two issues of one project merge in the same round: their deploys run one
/// after the other, and both end verified and closed.
#[test]
fn two_merges_in_one_project_deploy_one_after_the_other() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use temet_work::socket::{Reply, answer};
    use temet_work_core::model::{Check, Plan, Role};
    use temet_work_core::{Command, Effect, Outcome, Part, Polled, Verdict};

    /// The deploy part, noting the most commands it has had running after an
    /// effect: it starts commands only when it carries one out.
    struct Counted {
        inner: temet_work_deploy::Deploys,
        most: Arc<AtomicUsize>,
    }

    impl Part for Counted {
        fn id(&self) -> &'static str {
            self.inner.id()
        }
        fn poll(&mut self, world: &World, now_ms: u64) -> Polled {
            self.inner.poll(world, now_ms)
        }
        fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
            let outcome = self.inner.carry_out(effect, world);
            self.most.fetch_max(self.inner.running(), Ordering::SeqCst);
            outcome
        }
    }

    const ONE_PROJECT: &str = r#"
[[project]]
id = "ankh"
tracker = "fake"
name = "city/ankh"
deploy = "make deploy"
verify = "make smoke"
run_in = "/srv/ankh"

[[folder]]
id = "ankh-1"
project = "ankh"
path = "."

[[folder]]
id = "ankh-2"
project = "ankh"
path = "."

[[group]]
id = "watch"
name = "Night watch"
projects = ["ankh"]
"#;

    let tracked = |key: &str, title: &str| {
        (
            "city/ankh".to_string(),
            TrackedIssue {
                key: key.into(),
                title: title.into(),
                body: String::new(),
                milestone: None,
                depends_on: vec![],
                order: key.parse().expect("number"),
                closed: false,
            },
        )
    };
    let mut trackers = temet_work_tracker::Trackers::default();
    trackers.add(Box::new(FakeTracker {
        issues: vec![
            tracked("1", "Mend the clock"),
            tracked("2", "Wind the clock"),
        ],
        ..FakeTracker::default()
    }));
    let mut runners = temet_work_runner::Runners::default();
    let runner = FakeRunner::default();
    let starts = runner.starts();
    runners.add(Box::new(runner));
    let pull = |number: &str| {
        (
            "city/ankh".to_string(),
            Pull {
                number: number.into(),
                state: PullState::Open,
                checks: Checks::Passed,
                conflicting: false,
            },
        )
    };
    let fake = FakeCodeHost {
        pulls: vec![pull("57"), pull("58")],
        ..FakeCodeHost::default()
    };
    let merged = fake.merged_repos.clone();
    let mut code_hosts = temet_work_codehost::CodeHosts::default().every(0);
    code_hosts.add(Box::new(fake));
    let deployed = Arc::new(Mutex::new(Vec::new()));
    let ran = deployed.clone();
    let most = Arc::new(AtomicUsize::new(0));
    let deploys = Counted {
        inner: temet_work_deploy::Deploys::with(Arc::new(
            move |command: &str, _: &std::path::PathBuf, env: &[(String, String)]| {
                if command == "make deploy" {
                    let issue = env
                        .iter()
                        .find_map(|(k, v)| (k == "TEMET_WORK_ISSUE").then(|| v.clone()))
                        .unwrap_or_default();
                    ran.lock().map_err(|e| e.to_string())?.push(issue);
                }
                Ok(())
            },
        )),
        most: most.clone(),
    };
    let mut host = Host::new(
        World::default(),
        vec![
            Box::new(trackers),
            Box::new(runners.daemon_host(None)),
            Box::new(fake_runtimes()),
            Box::new(code_hosts),
            Box::new(deploys),
        ],
        Box::new(Memory::default()),
    );
    for event in Config::parse(ONE_PROJECT)
        .expect("config")
        .events(host.world())
    {
        host.handle(0, event).expect("config event");
    }

    let run = |host: &mut Host, issue: &IssueId, role: Role, command: Command| {
        let agent = host
            .world()
            .agents
            .values()
            .find(|a| a.issue.as_ref() == Some(issue) && a.role == role)
            .map(|a| a.id.0.clone())
            .unwrap_or_else(|| panic!("no {role:?} on {issue}"));
        let request = from(&starts, Some(&agent), command);
        match answer(host, 1, &request, &Peer::Unchecked).expect("answer") {
            Reply::Done { text } => text,
            Reply::Refused { why } => panic!("refused: {why}"),
        }
    };
    host.tick(0).expect("sync");
    let issues = [IssueId::new("ankh", "1"), IssueId::new("ankh", "2")];
    for issue in &issues {
        let plan = Plan {
            repos: Vec::new(),
            settings: Default::default(),
            checks: vec![Check {
                done_when: "the clock strikes thirteen".into(),
                proven_by: "listening at midnight".into(),
            }],
        };
        let plan = Command::Plan {
            issue: issue.clone(),
            plan,
        };
        run(&mut host, issue, Role::Refiner, plan);
    }
    for (issue, pull) in issues.iter().zip(["57", "58"]) {
        let ready = Command::Ready {
            issue: issue.clone(),
            pull: Some(pull.into()),
            pulls: Default::default(),
        };
        run(&mut host, issue, Role::Author, ready);
    }
    for issue in &issues {
        let approve = Command::Review {
            issue: issue.clone(),
            verdict: Verdict::Approve,
        };
        run(&mut host, issue, Role::Reviewer, approve);
    }
    host.tick(1).expect("CI read, both merge");
    assert_eq!(
        *merged.lock().expect("lock"),
        ["city/ankh", "city/ankh"],
        "both merged in one round"
    );
    for issue in &issues {
        assert_eq!(host.world().issues[issue].stage, Stage::Deploying);
    }
    // The commands run on threads, each done within a millisecond or so; the
    // loop asks again every round, for at most two seconds.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while issues
        .iter()
        .any(|i| host.world().issues[i].stage != Stage::Done)
        && std::time::Instant::now() < deadline
    {
        host.tick(2).expect("tick");
        std::thread::yield_now();
    }
    for issue in &issues {
        let i = &host.world().issues[issue];
        assert_eq!(i.stage, Stage::Done, "{issue}: {:?}", host.problems());
        assert!(i.closed_by_loop, "{issue} closed");
    }
    assert!(host.problems().is_empty(), "{:?}", host.problems());
    assert_eq!(
        *deployed.lock().expect("lock"),
        ["ankh#1", "ankh#2"],
        "the deploy command ran once for each"
    );
    assert_eq!(
        most.load(Ordering::SeqCst),
        1,
        "never two commands of the project at once"
    );
}

/// Who may send what (issue 28): every agent's own secret, the process tree a
/// request comes from, and the relay from another machine.
mod senders {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::process::Output;
    use std::sync::mpsc::Receiver;
    use temet_work::socket::{self, Pending, Reply, answer};
    use temet_work_core::event::Event;
    use temet_work_core::ids::{DecisionId, GroupId, MachineId, RuleId};
    use temet_work_core::model::{Decision, DecisionKind, Role, Rule, RuleState};

    const PLAN: &str = "checks:\n  - done_when: the clock strikes thirteen\n    proven_by: listening at midnight\n";

    /// A fresh folder for one test, under cargo's folder for test files.
    fn scratch(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("senders-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch");
        dir
    }

    /// Answers what reaches `requests`, with the peer the socket read, until
    /// `client` ends; returns what it returned.
    fn serve<T>(
        host: &mut Host,
        requests: &Receiver<Pending>,
        client: std::thread::JoinHandle<T>,
    ) -> T {
        serve_seeing(host, requests, client).0
    }

    /// [`serve`], also returning the peer the socket read for each request.
    fn serve_seeing<T>(
        host: &mut Host,
        requests: &Receiver<Pending>,
        client: std::thread::JoinHandle<T>,
    ) -> (T, Vec<Peer>) {
        let mut seen = Vec::new();
        while !client.is_finished() {
            if let Ok(p) = requests.recv_timeout(std::time::Duration::from_millis(2)) {
                let reply = answer(host, 1, &p.request, &p.peer).expect("answer");
                seen.push(p.peer.clone());
                p.answer(reply);
            }
        }
        (client.join().expect("client"), seen)
    }

    /// The exit code and both outputs, trimmed.
    fn outcome(out: Output) -> (Option<i32>, String, String) {
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).trim().to_string(),
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        )
    }

    /// The built `temet-work` with `args`, reaching the daemon at `socket`, with
    /// neither agent variable set.
    fn temet_work(socket: &Path, args: &[&str]) -> std::process::Command {
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"));
        cmd.args(args)
            .env("TEMET_WORK_SOCKET", socket)
            .env_remove("TEMET_WORK_DAEMON")
            .env_remove("TEMET_WORK_AGENT")
            .env_remove(SECRET_VAR);
        cmd
    }

    /// `temet-work` run as the agent `by` with `secret`, as the daemon starts it.
    fn as_agent(
        socket: &Path,
        by: &str,
        secret: Option<&str>,
        args: &[&str],
    ) -> (Option<i32>, String, String) {
        let mut cmd = temet_work(socket, args);
        cmd.env("TEMET_WORK_AGENT", by);
        if let Some(s) = secret {
            cmd.env(SECRET_VAR, s);
        }
        outcome(cmd.output().expect("runs"))
    }

    /// `temet-work args` from a child of a shell that holds `parent` as its agent
    /// variables. The shell waits for the command instead of becoming it, as an
    /// agent's shell does, and the child runs it with `child` as its own
    /// variables: none at all when `child` is `None` (`env -u` both).
    #[cfg(target_os = "linux")]
    fn under(
        socket: &Path,
        parent: (&str, &str),
        child: Option<(&str, &str)>,
        args: &[&str],
    ) -> (Option<i32>, String, String) {
        under_with(socket, parent, child, args, "")
    }

    /// [`under`], with `input` on the command's standard input.
    #[cfg(target_os = "linux")]
    fn under_with(
        socket: &Path,
        parent: (&str, &str),
        child: Option<(&str, &str)>,
        args: &[&str],
        input: &str,
    ) -> (Option<i32>, String, String) {
        let built = Path::new(env!("CARGO_BIN_EXE_temet-work"));
        under_program(built, socket, parent, child, args, input)
    }

    /// [`under_with`], running `program` as the command.
    #[cfg(target_os = "linux")]
    fn under_program(
        program: &Path,
        socket: &Path,
        parent: (&str, &str),
        child: Option<(&str, &str)>,
        args: &[&str],
        input: &str,
    ) -> (Option<i32>, String, String) {
        use std::io::Write;
        let set: Vec<String> = match child {
            None => ["-u", "TEMET_WORK_AGENT", "-u", SECRET_VAR]
                .map(String::from)
                .to_vec(),
            Some((by, secret)) => vec![
                format!("TEMET_WORK_AGENT={by}"),
                format!("{SECRET_VAR}={secret}"),
            ],
        };
        // `; exit $?` keeps the shell alive as the command's parent.
        let mut child = std::process::Command::new("sh")
            .args(["-c", "env \"$@\"; exit $?", "sh"])
            .args(&set)
            .arg(program)
            .args(args)
            .env("TEMET_WORK_SOCKET", socket)
            .env_remove("TEMET_WORK_DAEMON")
            .env("TEMET_WORK_AGENT", parent.0)
            .env(SECRET_VAR, parent.1)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("sh runs");
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(input.as_bytes())
            .expect("input");
        outcome(child.wait_with_output().expect("sh ends"))
    }

    /// Two projects, ankh here and klatch on the machine mac, refined at once.
    const TWO: &str = r#"
daemon_host = "omit"

[[project]]
id = "ankh"
tracker = "fake"
name = "city/ankh"

[[project]]
id = "klatch"
tracker = "fake"
name = "city/klatch"

[[folder]]
id = "ankh-1"
project = "ankh"
path = "."

[[folder]]
id = "klatch-1"
project = "klatch"
machine = "mac"
path = "/work/klatch"

[[group]]
id = "watch"
name = "Night watch"
projects = ["ankh", "klatch"]
refining_at_once = 2
"#;

    #[cfg(feature = "store")]
    #[test]
    fn every_agent_gets_a_secret_of_its_own_and_none_is_logged_or_shown() {
        let log = scratch("log").join("events.jsonl");
        let (journal, world) = temet_work::journal(&log, Default::default()).expect("log");
        let (mut host, starts) = host_from(
            world,
            journal,
            fake_runtimes(),
            TWO,
            Some("omit"),
            vec![tracked("city/ankh", "1", 1), tracked("city/klatch", "1", 2)],
        );
        host.tick(0).expect("sync");
        let ankh = IssueId::new("ankh", "1");
        let plan = Command::Plan {
            issue: ankh.clone(),
            plan: temet_work::cli::plan(PLAN).expect("plan"),
        };
        let mut shown = Vec::new();
        let mut ask = |host: &mut Host, request: Request| {
            let reply = answer(host, 1, &request, &Peer::Unchecked).expect("answer");
            shown.push(format!("{reply:?}"));
            reply
        };
        let wrong = Request {
            secret: Some("0".repeat(64)),
            ..from(&starts, Some("refiner-1"), plan.clone())
        };
        assert!(matches!(ask(&mut host, wrong), Reply::Refused { .. }));
        assert!(matches!(
            ask(&mut host, from(&starts, Some("refiner-1"), plan)),
            Reply::Done { .. }
        ));
        for command in [
            Command::Status,
            Command::Issue {
                issue: ankh.clone(),
            },
            Command::Issue {
                issue: IssueId::new("klatch", "1"),
            },
            Command::Screen {
                agent: AgentId::new("author-1"),
            },
        ] {
            ask(&mut host, Request::new(None, command));
        }
        // Three agents here (the group's investigator among them) and one on mac,
        // each with its own secret.
        let started = starts.lock().expect("starts").clone();
        let machines: Vec<Option<&str>> = started.iter().map(|s| s.machine.as_deref()).collect();
        assert_eq!(machines.len(), 4, "{machines:?}");
        assert_eq!(machines.iter().filter(|m| m.is_none()).count(), 3);
        assert!(machines.contains(&Some("mac")), "{machines:?}");
        let secrets: Vec<String> = ["refiner-1", "refiner-2", "author-1", "investigator-1"]
            .iter()
            .map(|a| secret(&starts, a))
            .collect();
        let mut distinct = secrets.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(distinct.len(), 4);
        drop(host);
        let written = std::fs::read_to_string(&log).expect("log");
        shown.push(temet_work_core::text::text(
            &temet_work_store::replay(&temet_work_store::Log::read(&log).expect("read")),
            &Default::default(),
        ));
        for secret in &secrets {
            assert!(!written.contains(secret.as_str()), "the log holds a secret");
            for text in &shown {
                assert!(!text.contains(secret.as_str()), "{text}");
            }
            // Only its digest is kept.
            let digest = temet_work::peer::digest(secret);
            assert!(written.contains(&digest.0), "{written}");
        }
    }

    #[test]
    fn a_request_naming_an_agent_needs_that_agents_own_secret_through_the_command_line() {
        let dir = scratch("cli");
        let sock = dir.join("daemon.sock");
        let plan = dir.join("plan.yaml");
        std::fs::write(&plan, PLAN).expect("plan");
        let requests = socket::listen(&sock).expect("listen");
        let (mut host, starts) = host();
        host.tick(0).expect("sync");
        // One review round, so that the first block goes to the arbiter.
        let one_round = Command::Set {
            target: temet_work_core::settings::Target::parse("ankh"),
            role: None,
            key: "review_rounds".into(),
            value: "1".into(),
        };
        let set = answer(
            &mut host,
            1,
            &Request::new(None, one_round),
            &Peer::Unchecked,
        )
        .expect("answer");
        assert!(matches!(set, Reply::Done { .. }), "{set:?}");
        let client = std::thread::spawn({
            let sock = sock.clone();
            move || {
                let run = |by: &str, secret: Option<&str>, args: &[&str]| {
                    as_agent(&sock, by, secret, args)
                };
                let refiner = secret(&starts, "refiner-1");
                let plan = plan.display().to_string();
                let set = ["set", "ankh#1", "starts", "5"];
                let mut got = vec![
                    run("refiner-1", None, &set),
                    run("refiner-1", None, &["status"]),
                    run("refiner-1", Some(&"0".repeat(64)), &set),
                    run("refiner-9", Some(&refiner), &set),
                    run("refiner-1", Some(&refiner), &set),
                    run("refiner-1", Some(&refiner), &["plan", "ankh#1", &plan]),
                ];
                let author = secret(&starts, "author-1");
                got.push(run("author-1", Some(&refiner), &["ready", "ankh#1", "57"]));
                got.push(run("author-1", Some(&author), &["ready", "ankh#1", "57"]));
                let reviewer = secret(&starts, "reviewer-1");
                got.push(run(
                    "reviewer-1",
                    Some(&reviewer),
                    &["review", "ankh#1", "block", "no test for a stopped clock"],
                ));
                let arbiter = secret(&starts, "arbiter-1");
                got.push(run(
                    "arbiter-1",
                    Some(&arbiter),
                    &["rule", "ankh#1", "approve", "the", "test", "can", "follow"],
                ));
                got
            }
        });
        let got = serve(&mut host, &requests, client);
        let refused = |i: usize, what: &str| {
            assert_eq!(got[i].0, Some(1), "{i}: {:?}", got[i]);
            assert!(got[i].2.contains(what), "{i}: {:?}", got[i]);
        };
        refused(0, "names refiner-1 but carries no secret");
        refused(1, "names refiner-1 but carries no secret");
        refused(2, "names refiner-1 but its secret is wrong");
        refused(3, "refiner-9 is not an agent this daemon started");
        assert_eq!(got[4].0, Some(0), "{:?}", got[4]);
        assert_eq!(got[5].0, Some(0), "{:?}", got[5]);
        refused(
            6,
            "names author-1 but carries refiner-1's secret, not author-1's",
        );
        assert_eq!(got[7].1, "ankh#1 is in review, round 1", "{:?}", got[7]);
        assert!(got[8].1.contains("arbiter"), "{:?}", got[8]);
        assert_eq!(got[9].0, Some(0), "{:?}", got[9]);
        let decision = host.world().decisions.values().next().expect("a ruling");
        assert_eq!(decision.by, AgentId::new("arbiter-1"));
    }

    #[test]
    fn an_agent_acts_through_mcp_with_its_secret_and_is_refused_without_it() {
        use std::io::Write;
        let sock = scratch("mcp").join("daemon.sock");
        let requests = socket::listen(&sock).expect("listen");
        let (mut host, starts) = host();
        host.tick(0).expect("sync");
        let refiner = secret(&starts, "refiner-1");
        let client = std::thread::spawn({
            let sock = sock.clone();
            move || {
                let mcp = |secret: Option<&str>| {
                    let mut cmd = temet_work(&sock, &["mcp", "--allow", "act"]);
                    cmd.env("TEMET_WORK_AGENT", "refiner-1")
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped());
                    if let Some(s) = secret {
                        cmd.env(SECRET_VAR, s);
                    }
                    let mut child = cmd.spawn().expect("mcp runs");
                    let call = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"plan","arguments":{"issue":"ankh#1","checks":[{"done_when":"the clock strikes thirteen","proven_by":"listening at midnight"}]}}}"#;
                    writeln!(child.stdin.take().expect("stdin"), "{call}").expect("write");
                    let out = child.wait_with_output().expect("mcp ends");
                    let answer: serde_json::Value =
                        serde_json::from_slice(&out.stdout).expect("one answer");
                    (
                        answer["result"]["isError"].as_bool(),
                        answer["result"]["content"][0]["text"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                    )
                };
                (mcp(None), mcp(Some(&refiner)))
            }
        });
        let (without, with) = serve(&mut host, &requests, client);
        assert_eq!(without.0, Some(true), "{without:?}");
        assert!(without.1.contains("carries no secret"), "{without:?}");
        assert_eq!(with, (Some(false), "ankh#1 is working".to_string()));
    }

    #[cfg(feature = "store")]
    #[test]
    fn after_a_restart_running_agents_keep_their_secrets_and_older_ones_their_ids() {
        let kept = Kept::default();
        let config = CONFIG.replace(
            "projects = [\"ankh\"]",
            "projects = [\"ankh\"]\nrefining_at_once = 2",
        );
        let issues = || vec![tracked("city/ankh", "1", 1), tracked("city/ankh", "2", 2)];
        let (mut first, starts) = host_from(
            World::default(),
            Box::new(kept.clone()),
            fake_runtimes(),
            &config,
            None,
            issues(),
        );
        first.tick(0).expect("sync");
        let earlier = secret(&starts, "refiner-1");
        drop(first);
        // The log's lines as the store writes them, then an agent a daemon started
        // before agents had secrets, whose line has no digest.
        let mut lines: Vec<String> = kept
            .0
            .lock()
            .expect("log")
            .iter()
            .enumerate()
            .map(|(i, event)| {
                serde_json::to_string(&temet_work_store::Entry {
                    seq: i as u64 + 1,
                    at_ms: 0,
                    event: event.clone(),
                })
                .expect("line")
            })
            .collect();
        // Two of them: refiner-0 still running, refiner-9 ended.
        for agent in ["refiner-0", "refiner-9"] {
            lines.push(format!(
                r#"{{"seq":{},"at_ms":0,"event":{{"AgentStarted":{{"agent":"{agent}","group":"watch","role":"Refiner","issue":{{"project":"ankh","key":"2"}},"machine":"local","folder":null}}}}}}"#,
                lines.len() + 1
            ));
        }
        lines.push(format!(
            r#"{{"seq":{},"at_ms":0,"event":{{"AgentChanged":{{"agent":"refiner-9","state":"Ended"}}}}}}"#,
            lines.len() + 1
        ));
        let entries: Vec<temet_work_store::Entry> = lines
            .iter()
            .map(|l| serde_json::from_str(l).expect("a log line"))
            .collect();

        // The second daemon has only the log.
        let (mut second, _) = host_from(
            temet_work_store::replay(&entries),
            Box::new(Memory::default()),
            fake_runtimes(),
            &config,
            None,
            issues(),
        );
        assert_eq!(
            second.world().agents[&AgentId::new("refiner-1")].secret_digest,
            Some(temet_work::peer::digest(&earlier))
        );
        let plan = |key: &str| Command::Plan {
            issue: IssueId::new("ankh", key),
            plan: temet_work::cli::plan(PLAN).expect("plan"),
        };
        let mut ask = |by: &str, secret: Option<String>, command| {
            let request = Request {
                secret,
                ..Request::new(Some(AgentId::new(by)), command)
            };
            answer(&mut second, 1, &request, &Peer::Unchecked).expect("answer")
        };
        assert!(matches!(
            ask("refiner-1", Some("0".repeat(64)), plan("1")),
            Reply::Refused { why } if why.contains("secret is wrong")
        ));
        assert_eq!(
            ask("refiner-1", Some(earlier), plan("1")),
            Reply::Done {
                text: "ankh#1 is working".into()
            }
        );
        // The ended one is refused; the running one is taken by its id, and as
        // ankh#2 waits for the one folder, the plan counts.
        assert!(matches!(
            ask("refiner-9", None, plan("2")),
            Reply::Refused { why } if why.starts_with("refiner-9 has ended")
        ));
        assert_eq!(
            ask("refiner-0", None, plan("2")),
            Reply::Done {
                text: "ankh#2 is ready".into()
            }
        );
    }

    /// A world with ankh#1 at `stage`, ankh#2 deploying, decisions 1 and 2 with a
    /// rule proposed from each, and the agents `(id, role, secret)` on ankh#1. The host has
    /// no parts: what the commands lead to is not under test.
    fn tree_host(agents: &[(&str, Role, &str)], stage: Stage) -> Host {
        let mut w = World::default();
        for event in Config::parse(CONFIG).expect("config").events(&w) {
            w.apply(&event);
        }
        for key in ["1", "2"] {
            w.apply(&Event::IssueSynced {
                issue: IssueId::new("ankh", key),
                title: "Mend the clock".into(),
                milestone: None,
                depends_on: vec![],
                order: 1,
                closed: false,
            });
        }
        for (id, role, secret) in agents {
            w.apply(&Event::AgentStarted {
                agent: AgentId::new(*id),
                group: GroupId::new("watch"),
                role: *role,
                issue: Some(IssueId::new("ankh", "1")),
                machine: MachineId::new("local"),
                folder: None,
                secret_digest: Some(temet_work::peer::digest(secret)),
            });
        }
        // Decision 1 is the one to overturn; decision 2's rule is proposed too.
        for id in [1, 2] {
            let decision = Decision {
                id: DecisionId(id),
                issue: IssueId::new("ankh", "1"),
                by: AgentId::new("author-1"),
                kind: DecisionKind::Choice,
                question: "Which bell?".into(),
                options: vec!["the big one".into(), "the small one".into()],
                chosen: 1,
                reason: "quieter".into(),
                overturned_to: None,
                overturn_reason: String::new(),
            };
            w.apply(&Event::DecisionMade { decision });
            w.apply(&Event::RuleProposed {
                rule: Rule {
                    id: RuleId(id),
                    text: "Ring the big bell.".into(),
                    decision: DecisionId(id),
                    issue: IssueId::new("ankh", "1"),
                    decided_by: AgentId::new("author-1"),
                    overturned_by: "person".into(),
                    state: RuleState::Proposed,
                },
            });
        }
        let one = w
            .issues
            .get_mut(&IssueId::new("ankh", "1"))
            .expect("ankh#1");
        one.stage = stage;
        one.told = true;
        w.issues
            .get_mut(&IssueId::new("ankh", "2"))
            .expect("ankh#2")
            .stage = Stage::Deploying;
        Host::new(w, Vec::new(), Box::new(Memory::default()))
    }

    /// A random secret, as the runner makes one.
    #[cfg(target_os = "linux")]
    fn fresh() -> String {
        use std::io::Read;
        let mut bytes = [0u8; 32];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut bytes))
            .expect("random");
        temet_work_core::Digest::from_bytes(&bytes).0
    }

    /// A new folder in the temp dir that only this user can write, removed when
    /// dropped, so a failed test leaves nothing in it either.
    #[cfg(target_os = "linux")]
    struct Private(PathBuf);

    #[cfg(target_os = "linux")]
    impl Private {
        fn new() -> Self {
            use std::os::unix::fs::DirBuilderExt;
            // A random name, and `create` fails on a folder someone else made.
            let dir = std::env::temp_dir().join(format!("temet-work-test-{}", &fresh()[..16]));
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&dir)
                .expect("a new folder");
            Self(dir)
        }
    }

    #[cfg(target_os = "linux")]
    impl Drop for Private {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// The commands a person may run and an agent may not.
    const PERSONS: [&[&str]; 7] = [
        &["rule", "ankh#1", "approve", "the", "bell", "is", "fine"],
        &["overturn", "1", "1", "the", "big", "one", "is", "heard"],
        &["retry", "ankh#2"],
        &["proposal", "1", "keep"],
        &["proposal", "2", "drop"],
        &["set", "ankh", "review_rounds", "2"],
        &["unset", "ankh", "review_rounds"],
    ];

    #[cfg(target_os = "linux")]
    #[test]
    fn a_person_command_from_inside_an_agents_process_tree_is_refused_even_with_its_variables_unset()
     {
        let sock = scratch("tree").join("daemon.sock");
        let requests = socket::listen(&sock).expect("listen");
        let author = fresh();
        let mut host = tree_host(
            &[("author-1", Role::Author, &author)],
            Stage::Arbitration { round: 3 },
        );
        let client = std::thread::spawn({
            let sock = sock.clone();
            let author = author.clone();
            move || {
                let inside: Vec<_> = PERSONS
                    .iter()
                    .map(|args| under(&sock, ("author-1", &author), None, args))
                    .collect();
                let reads: Vec<_> = [&["status"][..], &["issue", "ankh#1"], &["settings", "ankh"]]
                    .iter()
                    .map(|args| under(&sock, ("author-1", &author), None, args))
                    .collect();
                // A person's own shell, then a shell holding a secret this daemon
                // never issued, as a second daemon's agent would.
                let outside: Vec<_> = PERSONS
                    .iter()
                    .map(|args| outcome(temet_work(&sock, args).output().expect("runs")))
                    .collect();
                let foreign = fresh();
                let other: Vec<_> = PERSONS[5..]
                    .iter()
                    .map(|args| under(&sock, ("author-1", &foreign), None, args))
                    .collect();
                (inside, reads, outside, other)
            }
        });
        let (inside, reads, outside, other) = serve(&mut host, &requests, client);
        for (args, got) in PERSONS.iter().zip(&inside) {
            assert_eq!(got.0, Some(1), "{args:?}: {got:?}");
            assert!(
                got.2
                    .contains("names no agent but comes from inside author-1's process tree"),
                "{args:?}: {got:?}"
            );
        }
        for got in &reads {
            assert_eq!(got.0, Some(0), "{got:?}");
        }
        for (args, got) in PERSONS.iter().zip(outside.iter().chain(&other)) {
            assert_eq!(got.0, Some(0), "{args:?}: {got:?}");
        }
        assert_eq!(other.len(), 2);
        assert!(
            host.world()
                .rules
                .values()
                .all(|r| r.state != RuleState::Proposed)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_stolen_secret_is_refused_by_the_secret_in_the_thiefs_own_process_tree() {
        let sock = scratch("stolen").join("daemon.sock");
        let requests = socket::listen(&sock).expect("listen");
        let (author, reviewer) = (fresh(), fresh());
        let mut host = tree_host(
            &[
                ("author-1", Role::Author, &author),
                ("reviewer-1", Role::Reviewer, &reviewer),
            ],
            Stage::InReview { round: 1 },
        );
        let approve = ["review", "ankh#1", "approve"];
        let client = std::thread::spawn({
            let sock = sock.clone();
            let (author, reviewer) = (author.clone(), reviewer.clone());
            move || {
                under(
                    &sock,
                    ("author-1", &author),
                    Some(("reviewer-1", &reviewer)),
                    &approve,
                )
            }
        });
        let stolen = serve(&mut host, &requests, client);
        assert_eq!(stolen.0, Some(1), "{stolen:?}");
        assert!(
            stolen.2.contains(
                "names reviewer-1 but comes from inside author-1's process tree, which holds author-1's secret"
            ),
            "{stolen:?}"
        );
        let issue = IssueId::new("ankh", "1");
        assert!(!host.world().issues[&issue].review_ok, "still in review");
        // The reviewer's own shell sends the same command, and it counts.
        let client = std::thread::spawn({
            let sock = sock.clone();
            move || {
                under(
                    &sock,
                    ("reviewer-1", &reviewer),
                    Some(("reviewer-1", &reviewer)),
                    &approve,
                )
            }
        });
        let own = serve(&mut host, &requests, client);
        assert_eq!(own.0, Some(0), "{own:?}");
        assert!(host.world().issues[&issue].review_ok, "approved");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_stolen_secret_from_a_sender_that_cannot_be_read_is_refused() {
        let sock = scratch("unreadable").join("daemon.sock");
        // A copy with no read bit runs non-dumpable, so its environ is root's.
        // It goes to the temp dir: on a busy project disk the copy alone can
        // take seconds. `install` writes it, so no thread here holds it open
        // when it runs.
        let copy = Private::new();
        let hidden = copy.0.join("temet-work");
        let installed = std::process::Command::new("install")
            .args(["-m", "111", env!("CARGO_BIN_EXE_temet-work")])
            .arg(&hidden)
            .status()
            .expect("install runs");
        assert!(installed.success(), "{installed}");
        let requests = socket::listen(&sock).expect("listen");
        let (author, reviewer) = (fresh(), fresh());
        let mut host = tree_host(
            &[
                ("author-1", Role::Author, &author),
                ("reviewer-1", Role::Reviewer, &reviewer),
            ],
            Stage::InReview { round: 1 },
        );
        let approve = ["review", "ankh#1", "approve"];
        let client = std::thread::spawn({
            let sock = sock.clone();
            let (author, reviewer) = (author.clone(), reviewer.clone());
            move || {
                under_program(
                    &hidden,
                    &sock,
                    ("author-1", &author),
                    Some(("reviewer-1", &reviewer)),
                    &approve,
                    "",
                )
            }
        });
        let (stolen, seen) = serve_seeing(&mut host, &requests, client);
        // Run as root the copy would be readable, and this test would prove nothing.
        assert!(
            matches!(&seen[..], [Peer::Unreadable { .. }]),
            "the daemon read the sender: {seen:?}"
        );
        assert_eq!(stolen.0, Some(1), "{stolen:?}");
        assert!(
            stolen.2.contains(
                "names reviewer-1 but comes from inside author-1's process tree, which holds author-1's secret"
            ),
            "{stolen:?}"
        );
        let issue = IssueId::new("ankh", "1");
        assert!(!host.world().issues[&issue].review_ok, "still in review");
        // The reviewer's own temet-work sends the same command, and it counts.
        let client = std::thread::spawn({
            let sock = sock.clone();
            move || as_agent(&sock, "reviewer-1", Some(&reviewer), &approve)
        });
        let own = serve(&mut host, &requests, client);
        assert_eq!(own.0, Some(0), "{own:?}");
        assert!(host.world().issues[&issue].review_ok, "approved");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_stolen_secret_sent_through_a_local_relay_is_refused_all_the_same() {
        let sock = scratch("stolen-relay").join("daemon.sock");
        let requests = socket::listen(&sock).expect("listen");
        let (author, reviewer) = (fresh(), fresh());
        let mut host = tree_host(
            &[
                ("author-1", Role::Author, &author),
                ("reviewer-1", Role::Reviewer, &reviewer),
            ],
            Stage::InReview { round: 1 },
        );
        // The request the reviewer would send, which the relay marks as relayed.
        let line = serde_json::to_string(&Request {
            by: Some(AgentId::new("reviewer-1")),
            secret: Some(reviewer.clone()),
            relayed: true,
            command: Command::Review {
                issue: IssueId::new("ankh", "1"),
                verdict: temet_work_core::Verdict::Approve,
            },
        })
        .expect("line");
        let client = std::thread::spawn({
            let sock = sock.clone();
            move || {
                under_with(
                    &sock,
                    ("author-1", &author),
                    None,
                    &[
                        "relay",
                        "--socket",
                        &sock.display().to_string(),
                        "--build",
                        temet_work::BUILD,
                        "--host",
                        "lancre",
                    ],
                    &format!("{line}\n"),
                )
            }
        });
        let got = serve(&mut host, &requests, client);
        assert_eq!(got.0, Some(0), "the relay ran: {got:?}");
        let reply: Reply = serde_json::from_str(&got.1).expect("a reply");
        assert!(
            matches!(&reply, Reply::Refused { why } if why.contains("inside author-1's process tree")),
            "{reply:?}"
        );
        assert!(!host.world().issues[&IssueId::new("ankh", "1")].review_ok);
    }

    #[test]
    fn a_person_command_through_the_relay_is_refused_and_reads_pass() {
        let sock = scratch("relay").join("daemon.sock");
        let requests = socket::listen(&sock).expect("listen");
        let stub = stub_ssh("senders-relay", &sock, None);
        let mut host = tree_host(&[], Stage::Arbitration { round: 3 });
        let client = std::thread::spawn(move || {
            let path = format!(
                "{}:{}",
                stub.parent().expect("dir").display(),
                std::env::var("PATH").unwrap_or_default()
            );
            let run = |args: &[&str]| {
                let mut cmd = temet_work(Path::new("/nonexistent.sock"), args);
                cmd.env_remove("TEMET_WORK_SOCKET")
                    .env("TEMET_WORK_DAEMON", "ssh://stub")
                    .env("PATH", &path);
                outcome(cmd.output().expect("runs"))
            };
            (run(PERSONS[0]), run(PERSONS[5]), run(&["status"]))
        });
        let (rule, set, status) = serve(&mut host, &requests, client);
        for got in [&rule, &set] {
            assert_eq!(got.0, Some(1), "{got:?}");
            assert!(
                got.2.contains("came through temet-work relay")
                    && got.2.contains("run it on the daemon's machine"),
                "{got:?}"
            );
        }
        assert_eq!(status.0, Some(0), "{status:?}");
    }
}

mod investigations {
    use super::*;
    use temet_work::socket::{Reply, answer};
    use temet_work_core::Event;
    use temet_work_core::ids::GroupId;
    use temet_work_tracker::TrackedMilestone;

    const SHORT: &str = r#"
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
milestones = ["First"]
refining_at_once = 2
"#;

    /// A host whose group, with one issue to refine and a free slot, is short of work.
    fn short() -> (Host, Starts, Kept) {
        let kept = Kept::default();
        let mut one = tracked("city/ankh", "1", 1);
        one.1.milestone = Some("First".into());
        let (host, starts) = host_tracking(
            World::default(),
            Box::new(kept.clone()),
            fake_runtimes(),
            SHORT,
            None,
            FakeTracker {
                issues: vec![one],
                milestones: vec![(
                    "city/ankh".into(),
                    TrackedMilestone {
                        title: "First".into(),
                        open: 1,
                    },
                )],
                ..FakeTracker::default()
            },
        );
        (host, starts, kept)
    }

    fn ask(host: &mut Host, request: Request) -> Reply {
        answer(host, 1_000, &request, &Peer::Unchecked).expect("answer")
    }

    fn file(title: &str) -> Command {
        Command::File {
            project: temet_work_core::ids::ProjectId::new("ankh"),
            title: title.into(),
            body: "Wind it nightly.\nDepends on #1".into(),
            milestone: None,
        }
    }

    fn starts_on(kept: &Kept, issue: &IssueId) -> usize {
        kept.0
            .lock()
            .expect("journal")
            .iter()
            .filter(|e| matches!(e, Event::AgentStarted { issue: Some(i), .. } if i == issue))
            .count()
    }

    #[test]
    fn an_issue_the_investigator_files_gets_a_refiner_at_once_and_the_next_sync_adds_nothing() {
        let (mut host, starts, kept) = short();
        host.tick(0).expect("tick");
        let investigator = &host.world().agents[&AgentId::new("investigator-1")];
        assert_eq!(
            (investigator.issue.as_ref(), investigator.folder.as_ref()),
            (None, None),
            "{:?}",
            host.problems()
        );
        // Cloned, so the lock is not held while the investigator's requests read it.
        let labels = starts.lock().expect("starts")[1].labels.clone();
        assert_eq!(labels.get("role").map(String::as_str), Some("investigator"));
        assert_eq!(labels.get("issue"), None);

        let reply = ask(
            &mut host,
            from(&starts, Some("investigator-1"), file("Wind the clock")),
        );
        let Reply::Done { text } = reply else {
            panic!("{reply:?}");
        };
        assert_eq!(
            text,
            "filed ankh#2 in milestone First; an issue of ankh that waits for it says: Depends on #2"
        );
        let filed = IssueId::new("ankh", "2");
        let issue = &host.world().issues[&filed];
        assert_eq!(issue.stage, Stage::Refining, "in the loop at once");
        assert_eq!(issue.depends_on, [IssueId::new("ankh", "1")]);
        assert_eq!(starts_on(&kept, &filed), 1, "a refiner on it");
        let g = host.world().groups[&GroupId::new("watch")].clone();
        assert_eq!(
            host.world().queue(&g),
            2,
            "it counts toward the queue at once"
        );
        assert!(
            kept.0
                .lock()
                .expect("journal")
                .contains(&Event::IssueFiled {
                    group: GroupId::new("watch"),
                    agent: AgentId::new("investigator-1"),
                    issue: filed.clone(),
                    title: "Wind the clock".into(),
                    milestone: Some(temet_work_core::ids::MilestoneId::new("First")),
                })
        );

        // The next sync finds the issue as filed: nothing changes, nobody else starts.
        let before = kept.0.lock().expect("journal").len();
        assert_eq!(host.tick(60_000), Ok(0), "no events from the sync");
        assert_eq!(starts_on(&kept, &filed), 1);
        assert_eq!(
            kept.0.lock().expect("journal").len(),
            before,
            "nothing logged"
        );

        // The same title again is the plainest duplicate.
        let again = ask(
            &mut host,
            from(&starts, Some("investigator-1"), file("Wind the clock")),
        );
        assert!(
            matches!(&again, Reply::Refused { why } if why.contains("ankh#2 is open with the title")),
            "{again:?}"
        );
        let person = ask(&mut host, Request::new(None, file("Ring the bell")));
        assert!(matches!(person, Reply::Refused { .. }), "{person:?}");

        // Its word ends the investigation and stops its session.
        let done = ask(
            &mut host,
            from(
                &starts,
                Some("investigator-1"),
                Command::Investigated {
                    group: GroupId::new("watch"),
                },
            ),
        );
        assert!(
            matches!(&done, Reply::Done { text } if text.contains("ended with 1 issue filed")),
            "{done:?}"
        );
        assert_eq!(
            host.world().agents[&AgentId::new("investigator-1")].state,
            temet_work_core::model::AgentState::Ended
        );
        assert_eq!(
            kept.0.lock().expect("journal").last(),
            Some(&Event::InvestigationEnded {
                group: GroupId::new("watch"),
                at_ms: 1_000,
                filed: 1,
            })
        );
        let Reply::Done { text } = ask(&mut host, Request::new(None, Command::Status)) else {
            panic!("status");
        };
        assert!(
            text.contains("; last investigated 1970-01-01 00:00 UTC, filed 1 issue"),
            "{text}"
        );
    }
}
