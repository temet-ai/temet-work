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

/// A journal the test reads after the host has taken it: the events, and the
/// time each was logged with.
#[derive(Clone, Default)]
struct Kept(
    std::sync::Arc<std::sync::Mutex<Vec<temet_work_core::Event>>>,
    std::sync::Arc<std::sync::Mutex<Vec<u64>>>,
);

impl Kept {
    /// The world the log rebuilds, as the daemon rebuilds it at start: each
    /// event at its time, with what the build's parts put in and the log does
    /// not hold (`host`'s catalog and addresses).
    fn replayed(&self, host: &World) -> World {
        let events = self.0.lock().expect("journal").clone();
        let times = self.1.lock().expect("journal").clone();
        let mut world = World::default().replay_at(times.into_iter().zip(&events)).0;
        world.catalog = host.catalog.clone();
        world.addresses = host.addresses.clone();
        world
    }
}

impl temet_work_core::Journal for Kept {
    fn append(&mut self, at_ms: u64, event: &temet_work_core::Event) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .push(event.clone());
        self.1.lock().map_err(|e| e.to_string())?.push(at_ms);
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
    host_with_tracker(
        world,
        journal,
        runtimes,
        config,
        daemon_host,
        Box::new(tracker),
    )
}

/// A host on `world`, with every family's fake but the tracker, which is
/// `tracker`.
fn host_with_tracker(
    world: World,
    journal: Box<dyn temet_work_core::Journal>,
    runtimes: temet_work_runtime::Runtimes,
    config: &str,
    daemon_host: Option<&str>,
    tracker: Box<dyn temet_work_tracker::Tracker>,
) -> (Host, Starts) {
    let mut trackers = temet_work_tracker::Trackers::default();
    trackers.add(tracker);
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
                draft: false,
                branch: String::new(),
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
            Reply::View { view } => panic!("a view: {view:?}"),
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
            state: None,
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
            replies: Vec::new(),
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
        Reply::View { view } => panic!("a view: {view:?}"),
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
                state: None,
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
    // The project's agents are sent to mac, where its folder is.
    let config = CONFIG
        .replace("path = \".\"", "machine = \"mac\"\npath = \"/work/ankh\"")
        .replace(
            "deploy = \"make deploy\"\n",
            "deploy = \"make deploy\"\n\n[project.settings]\nmachine = \"mac\"\n",
        );
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
                    replies: Vec::new(),
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
fn a_persons_hold_starts_nothing_through_rounds_and_a_restart_until_a_resume() {
    use temet_work::socket::{self, Reply};

    let kept = Kept::default();
    let (mut host, starts) = host_with(Box::new(kept.clone()));
    let ask = |host: &mut Host, starts: &Starts, by: Option<&str>, command: Command| {
        socket::answer(host, 1, &from(starts, by, command), &Peer::Unchecked).expect("answer")
    };
    let held = ask(&mut host, &starts, None, Command::Hold);
    assert!(
        matches!(&held, Reply::Done { text } if text.starts_with("held: no issue is taken")),
        "{held:?}"
    );
    assert_eq!(host.tick(0), Ok(1), "the tracker still syncs");
    host.tick(60_000).expect("tick");
    let issue = IssueId::new("ankh", "1");
    assert_eq!(host.world().issues[&issue].stage, Stage::New, "not taken");
    assert!(starts.lock().expect("starts").is_empty(), "nothing started");
    let status = ask(&mut host, &starts, None, Command::Status);
    assert!(
        matches!(&status, Reply::Done { text } if text.starts_with("Held since ")),
        "{status:?}"
    );
    match ask(&mut host, &starts, None, Command::View) {
        Reply::View { view } => assert!(view.held && view.held_since.is_some()),
        other => panic!("{other:?}"),
    }

    // A restart replays the hold, and resuming starts the refiner.
    let (mut again, starts) = host_from(
        kept.replayed(host.world()),
        Box::new(kept.clone()),
        fake_runtimes(),
        CONFIG,
        None,
        vec![tracked("city/ankh", "1", 1)],
    );
    assert!(again.world().held, "the log keeps the hold");
    again.tick(120_000).expect("tick");
    assert!(starts.lock().expect("starts").is_empty(), "still nothing");
    let resumed = ask(&mut again, &starts, None, Command::Resume);
    assert_eq!(
        resumed,
        Reply::Done {
            text: "resumed: work starts again".into()
        }
    );
    assert_eq!(again.world().issues[&issue].stage, Stage::Refining);
    assert!(
        again
            .world()
            .agents
            .contains_key(&AgentId::new("refiner-1"))
    );
    // An agent may not hold the daemon, even with its own secret.
    assert!(matches!(
        ask(&mut again, &starts, Some("refiner-1"), Command::Hold),
        Reply::Refused { why } if why.starts_with("hold is a person's: refiner-1 is an agent")
    ));
}

#[test]
fn a_persons_stop_starts_no_agent_on_its_issue_through_rounds_and_a_restart_until_a_start() {
    use temet_work::socket::{self, Reply};

    let kept = Kept::default();
    let (mut host, starts) = host_with(Box::new(kept.clone()));
    let ask = |host: &mut Host, request: &Request, peer: &Peer| {
        socket::answer(host, 1, request, peer).expect("answer")
    };
    // The agents started on ankh#1; an investigator may start for the group.
    let on_issue = |starts: &temet_work_runner::fake::Starts| {
        starts
            .lock()
            .expect("starts")
            .iter()
            .filter(|s| s.labels.get("issue").map(String::as_str) == Some("ankh#1"))
            .count()
    };
    host.tick(0).expect("tick");
    let issue = IssueId::new("ankh", "1");
    let refiner = AgentId::new("refiner-1");
    assert_eq!(host.world().issues[&issue].stage, Stage::Refining);
    let stop = Command::Stop {
        agent: refiner.clone(),
    };

    // An agent may not stop an agent, even with its own secret; nor may a
    // person's command sent from inside its process tree with both variables
    // unset.
    let own = ask(
        &mut host,
        &from(&starts, Some("refiner-1"), stop.clone()),
        &Peer::Unchecked,
    );
    assert!(
        matches!(&own, Reply::Refused { why } if why.starts_with("stop is a person's: refiner-1 is an agent")),
        "{own:?}"
    );
    let inside = Peer::Read(vec![temet_work_runner::digest(&secret(
        &starts,
        "refiner-1",
    ))]);
    let tree = ask(&mut host, &Request::new(None, stop.clone()), &inside);
    assert!(
        matches!(&tree, Reply::Refused { why } if why.starts_with("stop names no agent but comes from inside refiner-1's process tree")),
        "{tree:?}"
    );
    assert!(
        host.world().person_stopped(&refiner).is_none(),
        "neither was taken"
    );

    // A person's stop reaches the runner, which stops the terminal: the agent ends.
    let stopped = ask(
        &mut host,
        &Request::new(None, stop.clone()),
        &Peer::Unchecked,
    );
    assert_eq!(
        stopped,
        Reply::Done {
            text: "refiner-1's terminal is being stopped: no agent starts on ankh#1 until \
                   temet-work start ankh#1; its other agents carry on, and its merge, deploy and \
                   verify still run"
                .into()
        }
    );
    assert_eq!(
        host.world().agents[&refiner].state,
        temet_work_core::model::AgentState::Ended
    );
    assert!(host.screen(&refiner).is_err(), "its terminal is gone");
    for at in [60_000, 120_000] {
        host.tick(at).expect("tick");
    }
    assert_eq!(on_issue(&starts), 1, "no fresh refiner");
    assert_eq!(
        host.world().issues[&issue].stage,
        Stage::Refining,
        "it keeps its stage"
    );
    let again = ask(
        &mut host,
        &Request::new(None, stop.clone()),
        &Peer::Unchecked,
    );
    assert!(
        matches!(&again, Reply::Refused { why } if why.starts_with("refiner-1 is already stopped by a person")),
        "{again:?}"
    );

    // Status and the view say so.
    let Reply::Done { text } = ask(
        &mut host,
        &Request::new(None, Command::Status),
        &Peer::Unchecked,
    ) else {
        panic!("no status");
    };
    assert!(
        text.contains("stopped by a person (refiner-1) since ")
            && text.contains("no agent starts on it until temet-work start ankh#1"),
        "{text}"
    );
    assert!(
        text.contains("refiner-1 on ankh#1: ended (on local); stopped by a person at "),
        "{text}"
    );
    match ask(
        &mut host,
        &Request::new(None, Command::View),
        &Peer::Unchecked,
    ) {
        Reply::View { view } => {
            let i = view
                .issues
                .iter()
                .find(|i| i.id == "ankh#1")
                .expect("listed");
            assert_eq!(i.stopped.as_ref(), Some(&refiner));
            assert!(i.stopped_since.is_some());
            let a = view
                .agents
                .iter()
                .find(|a| a.id == refiner)
                .expect("listed");
            assert!(a.stopped_by_person && a.stopped_at.is_some());
        }
        other => panic!("{other:?}"),
    }

    // A restart replays the stop: still nothing starts.
    let (mut again, starts) = host_from(
        kept.replayed(host.world()),
        Box::new(kept.clone()),
        fake_runtimes(),
        CONFIG,
        None,
        vec![tracked("city/ankh", "1", 1)],
    );
    assert_eq!(
        again.world().stopped_by(&issue),
        Some(&refiner),
        "the log keeps it"
    );
    again.tick(180_000).expect("tick");
    assert_eq!(on_issue(&starts), 0, "still nothing");

    // A person's start: the next round starts a fresh refiner.
    let started = ask(
        &mut again,
        &Request::new(
            None,
            Command::Start {
                issue: issue.clone(),
            },
        ),
        &Peer::Unchecked,
    );
    assert_eq!(
        started,
        Reply::Done {
            text: "started: the next round starts what ankh#1 needs".into()
        }
    );
    again.tick(240_000).expect("tick");
    assert!(
        again
            .world()
            .agents
            .contains_key(&AgentId::new("refiner-2"))
    );
    assert_eq!(on_issue(&starts), 1);
    let twice = ask(
        &mut again,
        &Request::new(None, Command::Start { issue }),
        &Peer::Unchecked,
    );
    assert!(
        matches!(&twice, Reply::Refused { why } if why.starts_with("ankh#1 is not stopped")),
        "{twice:?}"
    );
}

#[test]
fn a_persons_assign_and_unassign_reach_the_daemon_and_exit_0_and_an_agents_are_refused_with_1() {
    use temet_work::socket;

    let config = format!("{CONFIG}refining_at_once = 2\n");
    let (mut host, starts) = host_from(
        World::default(),
        Box::new(Memory::default()),
        fake_runtimes(),
        &config,
        None,
        vec![tracked("city/ankh", "1", 1), tracked("city/ankh", "2", 2)],
    );
    host.tick(0).expect("tick");
    let on = |host: &Host, id: &str| host.world().agents[&AgentId::new(id)].issue.clone();
    assert_eq!(on(&host, "refiner-1"), Some(IssueId::new("ankh", "1")));
    assert_eq!(on(&host, "refiner-2"), Some(IssueId::new("ankh", "2")));
    let token = secret(&starts, "refiner-1");
    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("assign-exit.sock");
    let requests = socket::listen(&path).expect("listen");
    let run = move |args: &[&str], agent: Option<&str>| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"));
        command
            .args(args)
            .env("TEMET_WORK_SOCKET", &path)
            .env_remove("TEMET_WORK_DAEMON");
        match agent {
            Some(secret) => command
                .env("TEMET_WORK_AGENT", "refiner-1")
                .env(SECRET_VAR, secret),
            None => command
                .env_remove("TEMET_WORK_AGENT")
                .env_remove(SECRET_VAR),
        };
        command.output().expect("runs")
    };
    let client = std::thread::spawn(move || {
        [
            run(&["assign", "ankh#2", "refiner-1"], None),
            run(&["assign", "ankh#1", "refiner-1"], Some(&token)),
            run(&["unassign", "ankh#2", "refiner-1"], Some(&token)),
            run(&["assign", "ankh#2", "author-9"], None),
            run(&["unassign", "ankh#2", "refiner-1"], None),
        ]
    });
    for n in 0..5 {
        let pending = requests
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("a request");
        let reply = socket::answer(&mut host, 1, &pending.request, &pending.peer).expect("answer");
        pending.answer(reply);
        if n == 0 {
            let w = host.world();
            assert_eq!(on(&host, "refiner-1"), Some(IssueId::new("ankh", "2")));
            assert!(
                w.agents[&AgentId::new("refiner-1")].briefed,
                "idle, so it heard at once"
            );
            assert!(w.agents[&AgentId::new("refiner-2")].unassigned);
            assert_eq!(
                w.running(
                    &IssueId::new("ankh", "1"),
                    temet_work_core::model::Role::Refiner
                )
                .map(|a| a.id.clone()),
                Some(AgentId::new("refiner-3")),
                "ankh#1 got a fresh refiner at once"
            );
        }
    }
    let [assigned, by_agent, unassigned_by_agent, unknown, unassigned] =
        client.join().expect("client");
    let out = |o: &std::process::Output| String::from_utf8_lossy(&o.stdout).trim().to_string();
    let err = |o: &std::process::Output| String::from_utf8_lossy(&o.stderr).trim().to_string();
    assert_eq!(assigned.status.code(), Some(0), "{}", err(&assigned));
    assert_eq!(
        out(&assigned),
        "refiner-1 works on ankh#2 now, moved from ankh#1; it hears its brief for ankh#2 when its \
         turn ends; refiner-2, its refiner until now, is unassigned and stopped; ankh#1 gets a \
         fresh refiner"
    );
    for refused in [&by_agent, &unassigned_by_agent] {
        assert_eq!(refused.status.code(), Some(1), "{}", out(refused));
        assert!(
            err(refused).contains("is a person's: refiner-1 is an agent, and only a person"),
            "{}",
            err(refused)
        );
    }
    assert_eq!(unknown.status.code(), Some(1));
    assert!(
        err(&unknown).contains("author-9 is not an agent this daemon started"),
        "{}",
        err(&unknown)
    );
    assert_eq!(unassigned.status.code(), Some(0), "{}", err(&unassigned));
    assert_eq!(
        out(&unassigned),
        "refiner-1 is unassigned from ankh#2 and its session is being stopped; once it has \
         ended, ankh#2 gets a fresh refiner"
    );
    let w = host.world();
    assert_eq!(
        w.agents[&AgentId::new("refiner-1")].state,
        temet_work_core::model::AgentState::Ended,
        "the fake runner stopped it at once"
    );
}

/// A plan of one check, as `temet-work plan` reads it.
const ONE_CHECK: &str =
    "checks:\n  - done_when: the clock strikes on time\n    proven_by: listening at noon\n";

/// Sends [`ONE_CHECK`] for ankh#1 as its refiner.
fn plan_one_check(host: &mut Host, starts: &Starts) {
    use temet_work::socket::{self, Reply};

    let command = Command::Plan {
        issue: IssueId::new("ankh", "1"),
        plan: temet_work::cli::plan(ONE_CHECK).expect("plan"),
    };
    let request = from(starts, Some("refiner-1"), command);
    let reply = socket::answer(host, 1, &request, &Peer::Unchecked).expect("answer");
    assert!(matches!(reply, Reply::Done { .. }), "{reply:?}");
}

/// The reply to a person's `temet-work issue ankh#1`.
fn issue_view(host: &mut Host) -> String {
    use temet_work::socket::{self, Reply};

    let command = Command::Issue {
        issue: IssueId::new("ankh", "1"),
    };
    let request = Request::new(None, command);
    match socket::answer(host, 1, &request, &Peer::Unchecked).expect("answer") {
        Reply::Done { text } => text,
        Reply::Refused { why } => panic!("refused: {why}"),
        Reply::View { view } => panic!("a view: {view:?}"),
    }
}

#[test]
fn the_issue_command_shows_the_text_the_last_sync_read_above_the_plan() {
    let body = "The clock in the square is slow.\n\nDone is a clock that strikes on time.";
    let (name, mut issue) = tracked("city/ankh", "1", 1);
    issue.body = body.into();
    let (mut host, starts) = host_tracking(
        World::default(),
        Box::new(Memory::default()),
        fake_runtimes(),
        CONFIG,
        None,
        FakeTracker {
            issues: vec![(name, issue)],
            ..FakeTracker::default()
        },
    );
    host.tick(0).expect("sync");
    plan_one_check(&mut host, &starts);
    let shown = issue_view(&mut host);
    let want = format!(
        "\nText of issue 1 of city/ankh, as the fake tracker returned it at the last sync that \
         worked:\n{body}\nEnd of the issue's text.\nPlan:\n"
    );
    assert!(shown.contains(&want), "{shown}");
    assert!(shown.starts_with("ankh#1  Mend the clock\n"), "{shown}");
}

#[test]
fn the_issue_command_before_the_first_sync_says_the_text_was_not_read() {
    // A daemon that restarted: its log holds the issue, and it has not synced.
    let (world, _) = World::replay(&[temet_work_core::Event::IssueSynced {
        issue: IssueId::new("ankh", "1"),
        title: "Mend the clock".into(),
        milestone: None,
        depends_on: vec![],
        order: 1,
        closed: false,
    }]);
    let (mut host, starts) = host_from(
        world,
        Box::new(Memory::default()),
        fake_runtimes(),
        CONFIG,
        None,
        vec![tracked("city/ankh", "1", 1)],
    );
    plan_one_check(&mut host, &starts);
    let shown = issue_view(&mut host);
    for line in [
        "ankh#1  Mend the clock\n",
        "\nStage: ",
        "\nGroup: watch\n",
        "\nThe issue's text was not read: ankh has not synced since the daemon started\nPlan:\n",
        "\n  - the clock strikes on time (shown by: listening at noon)\n",
        "\nAgents:\n",
    ] {
        assert!(shown.contains(line), "missing {line:?} in:\n{shown}");
    }
    assert!(!shown.contains("Text of"), "{shown}");
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
            Reply::View { view } => panic!("a view: {view:?}"),
        }
    };
    let plan = Plan {
        repos: Vec::new(),
        settings: Default::default(),
        checks: vec![Check {
            done_when: "the clock strikes thirteen".into(),
            proven_by: "listening at midnight".into(),
            state: None,
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
        replies: Vec::new(),
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
        failed: vec![1],
        open: Vec::new(),
    };
    assert_eq!(
        run(&mut host, "reviewer-1", block),
        "ankh#1 is fixing, round 1"
    );
    // The first ready after the block replies to its point.
    let replied = Command::Ready {
        issue: issue.clone(),
        pull: None,
        pulls: Default::default(),
        replies: vec!["added a test that stops the clock".into()],
    };
    assert_eq!(
        run(&mut host, "author-1", replied),
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
        failed: Vec::new(),
        open: Vec::new(),
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
    // The log of a live run, with parts carrying out effects, replays to the same
    // state, with the same time each issue last changed.
    assert_eq!(&kept.replayed(host.world()), host.world());
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
            Reply::View { view } => panic!("a view: {view:?}"),
        }
    }

    fn refused(reply: Reply) -> String {
        match reply {
            Reply::Refused { why } => why,
            Reply::Done { text } => panic!("done: {text}"),
            Reply::View { view } => panic!("a view: {view:?}"),
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
                    state: None,
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
            let local = Local { path: path.clone() }.send(&request);
            let ssh = Ssh {
                target: "stub".into(),
                port: None,
                socket: None,
                program: stub.display().to_string(),
                deadline: socket::SSH_DEADLINE,
            }
            .send(&request);
            let view = from(&starts, Some("refiner-1"), Command::View);
            let ssh_view = Ssh {
                target: "stub".into(),
                port: None,
                socket: None,
                program: stub.display().to_string(),
                deadline: socket::SSH_DEADLINE,
            }
            .send(&view);
            let local_view = Local { path }.send(&view);
            (local, ssh, local_view, ssh_view)
        })
    };
    for _ in 0..4 {
        let pending = requests
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("a request");
        let reply = socket::answer(&mut host, 1, &pending.request, &pending.peer).expect("answer");
        pending.answer(reply);
    }
    let (local, ssh, local_view, ssh_view) = client.join().expect("client");
    assert!(
        matches!(&local, Ok(socket::Reply::Done { text }) if text.contains("refiner-1 on ankh#1")),
        "{local:?}"
    );
    assert_eq!(local, ssh);
    assert!(
        matches!(&local_view, Ok(socket::Reply::View { view })
            if view.agents.iter().any(|a| a.id.0 == "refiner-1")),
        "{local_view:?}"
    );
    assert_eq!(local_view, ssh_view);
}

#[test]
fn status_json_prints_the_view() {
    use temet_work::socket;

    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("view.sock");
    let requests = socket::listen(&path).expect("listen");
    let client = std::thread::spawn(move || {
        std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
            .args(["status", "--json"])
            .env("TEMET_WORK_SOCKET", &path)
            .env_remove("TEMET_WORK_DAEMON")
            .env_remove("TEMET_WORK_AGENT")
            .env_remove(SECRET_VAR)
            .output()
            .expect("runs")
    });
    let (mut host, _) = host();
    host.tick(0).expect("tick");
    let pending = requests
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("a request");
    assert_eq!(pending.request.command, Command::View);
    let reply = socket::answer(&mut host, 1, &pending.request, &pending.peer).expect("answer");
    pending.answer(reply);
    let out = client.join().expect("client");
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let view: temet_work_core::View = serde_json::from_slice(&out.stdout).expect("a view");
    assert_eq!(
        view,
        temet_work_core::View::new(host.world(), host.problems())
    );
    assert_eq!(view.issues[0].id, "ankh#1");
}

/// Runs `status` and `status --json`, each with a standard output from
/// `stdout`, answers each through the socket and returns how each one ended.
fn status_writing_to(
    socket: &str,
    stdout: impl Fn() -> std::process::Stdio,
) -> [std::process::Output; 2] {
    use temet_work::socket;

    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(socket);
    let requests = socket::listen(&path).expect("listen");
    let (mut host, _) = host();
    host.tick(0).expect("tick");
    [
        (&["status"][..], Command::Status),
        (&["status", "--json"][..], Command::View),
    ]
    .map(|(args, command)| {
        let client = std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
            .args(args)
            .env("TEMET_WORK_SOCKET", &path)
            .env_remove("TEMET_WORK_DAEMON")
            .env_remove("TEMET_WORK_AGENT")
            .env_remove(SECRET_VAR)
            .stdout(stdout())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("runs");
        let pending = requests
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("a request");
        assert_eq!(pending.request.command, command);
        let reply = socket::answer(&mut host, 1, &pending.request, &pending.peer).expect("answer");
        pending.answer(reply);
        client.wait_with_output().expect("client")
    })
}

#[test]
fn status_to_a_gone_reader_is_a_quiet_success() {
    let ended = status_writing_to("gone-reader.sock", || {
        let (reader, writer) = std::io::pipe().expect("pipe");
        drop(reader);
        writer.into()
    });
    for out in ended {
        assert_eq!(out.status.code(), Some(0), "{out:?}");
        assert_eq!(String::from_utf8_lossy(&out.stderr), "");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn status_to_a_full_device_is_reported_and_exits_2() {
    let ended = status_writing_to("full-device.sock", || {
        std::fs::File::options()
            .write(true)
            .open("/dev/full")
            .expect("/dev/full")
            .into()
    });
    for out in ended {
        assert_eq!(out.status.code(), Some(2), "{out:?}");
        assert_eq!(
            String::from_utf8_lossy(&out.stderr),
            "standard output: No space left on device (os error 28)\n"
        );
    }
}

#[test]
fn json_on_a_command_other_than_status_is_refused_with_the_usage() {
    for args in [
        &["parts", "--json"][..],
        &["help", "--json"],
        &["mcp", "--json"],
        &["issue", "ankh#1", "--json"],
    ] {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
            .args(args)
            .env("TEMET_WORK_SOCKET", "/nonexistent.sock")
            .env_remove("TEMET_WORK_DAEMON")
            .output()
            .expect("runs");
        assert_eq!(out.status.code(), Some(2), "{args:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{args:?}: {out:?}");
        let why = String::from_utf8_lossy(&out.stderr);
        assert!(
            why.contains("temet-work status [--json]"),
            "{args:?}: {why}"
        );
    }
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
                cost: None,
                point: None,
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
            draft: false,
            branch: String::new(),
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
            Reply::View { view } => panic!("a view: {view:?}"),
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
                        state: None,
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
            replies: Vec::new(),
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
                failed: Vec::new(),
                open: Vec::new(),
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
        assert_eq!(&f.kept.replayed(f.host.world()), f.host.world());
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
/// author-1's choice on ankh#1, at `cost`: the clacks, the pigeon or the Post Office.
fn clacks_choice(cost: Option<&str>) -> Command {
    Command::Choose {
        issue: IssueId::new("ankh", "1"),
        question: "Which way does the clacks message go?".into(),
        options: vec!["clacks".into(), "pigeon".into(), "Post Office".into()],
        took: "pigeon".into(),
        why: "the towers are down past Sto Lat".into(),
        cost: cost.map(String::from),
    }
}

/// A host whose ankh#1 is being worked on by author-1, after author-1 has
/// recorded [`clacks_choice`] at medium cost through the socket's answer; the
/// log it kept, and the reply to the choice.
fn chosen() -> (Host, Starts, Kept, String) {
    use temet_work::socket::{Reply, answer};

    let kept = Kept::default();
    let (mut host, starts) = host_with(Box::new(kept.clone()));
    host.tick(0).expect("sync");
    plan_one_check(&mut host, &starts);
    let author = AgentId::new("author-1");
    let issue = IssueId::new("ankh", "1");
    assert_eq!(host.world().issues[&issue].stage, Stage::Working);
    let before = host.world().agents[&author].clone();
    let request = from(&starts, Some("author-1"), clacks_choice(Some("medium")));
    let reply = answer(&mut host, 1, &request, &Peer::Unchecked).expect("answer");
    let Reply::Done { text } = reply else {
        panic!("the choice is refused: {reply:?}");
    };
    // The author carries on: its state and its issue's stage are as they were.
    assert_eq!(host.world().agents[&author], before);
    assert_eq!(host.world().issues[&issue].stage, Stage::Working);
    (host, starts, kept, text)
}

#[test]
fn an_authors_choice_is_listed_by_status_and_the_view_and_kept_in_the_log() {
    use temet_work::socket::{Reply, answer};
    use temet_work_core::Event;
    use temet_work_core::model::{Cost, DecisionKind, Point};

    let (mut host, starts, kept, reply) = chosen();
    assert_eq!(reply, "decision 1 is recorded");
    let mut ask = |by: Option<&str>, command| {
        let request = from(&starts, by, command);
        answer(&mut host, 1, &request, &Peer::Unchecked).expect("answer")
    };
    let Reply::Done { text } = ask(None, Command::Status) else {
        panic!("status refused");
    };
    assert!(
        text.contains(
            "Decisions\n  1 ankh#1: Which way does the clacks message go? [1=clacks, 2=pigeon, \
             3=Post Office] chose pigeon, by author-1: the towers are down past Sto Lat; cost \
             medium, point merge\nRules\n"
        ),
        "{text}"
    );
    let Reply::View { view } = ask(None, Command::View) else {
        panic!("view refused");
    };
    // As `status --json` prints it.
    let json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string_pretty(&view).expect("json")).expect("reads");
    assert_eq!(
        json["decisions"],
        serde_json::json!([{
            "id": 1,
            "issue": "ankh#1",
            "by": "author-1",
            "question": "Which way does the clacks message go?",
            "options": ["clacks", "pigeon", "Post Office"],
            "chosen": 2,
            "reason": "the towers are down past Sto Lat",
            "overturned_to": null,
            "overturn_reason": "",
            "cost": "medium",
            "point": {"at": "merge", "passed": false}
        }])
    );
    // A choice with no cost, sent as a raw socket line, is refused by the loop.
    let raw = r#"{"by":"author-1","command":{"command":"choose","issue":{"project":"ankh","key":"1"},"question":"Which way?","options":["clacks","pigeon"],"took":"pigeon","why":"it flies"}}"#;
    let mut bare: Request = serde_json::from_str(raw).expect("a request without a cost reads");
    bare.secret = from(&starts, Some("author-1"), Command::Status).secret;
    assert!(matches!(
        answer(&mut host, 1, &bare, &Peer::Unchecked).expect("answer"),
        Reply::Refused { why } if why.contains("cost to undo: low, medium or high")
    ));
    assert_eq!(host.world().decisions.len(), 1);
    // One event in the log, and the log alone rebuilds the same decisions.
    let events = kept.0.lock().expect("log").clone();
    let made: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::DecisionMade { decision } => Some(decision),
            _ => None,
        })
        .collect();
    let [decision] = made.as_slice() else {
        panic!("{made:?}");
    };
    assert_eq!(
        (
            decision.id.0,
            &decision.kind,
            decision.by.0.as_str(),
            decision.chosen,
            decision.cost,
            decision.point
        ),
        (
            1,
            &DecisionKind::Choice,
            "author-1",
            1,
            Some(Cost::Medium),
            Some(Point::Merge)
        )
    );
    let (replayed, _) = World::replay(&events);
    assert_eq!(replayed.decisions, host.world().decisions);
    // Work carries on: the author's ready is accepted at once.
    let ready = Command::Ready {
        issue: IssueId::new("ankh", "1"),
        pull: Some("57".into()),
        pulls: Default::default(),
        replies: Vec::new(),
    };
    assert_eq!(
        answer(
            &mut host,
            1,
            &from(&starts, Some("author-1"), ready),
            &Peer::Unchecked
        )
        .expect("answer"),
        Reply::Done {
            text: "ankh#1 is in review, round 1".into()
        }
    );
}

#[test]
fn a_person_overturns_an_agents_recorded_choice_through_the_daemon() {
    use temet_work::socket::{Reply, answer};

    let (mut host, starts, _, _) = chosen();
    let issue = IssueId::new("ankh", "1");
    let mut ask = |by: Option<&str>, command| {
        let request = from(&starts, by, command);
        answer(&mut host, 1, &request, &Peer::Unchecked).expect("answer")
    };
    let overturn = Command::Overturn {
        decision: temet_work_core::ids::DecisionId(1),
        option: 3,
        reason: "the Post Office delivers in the rain".into(),
    };
    assert!(matches!(
        ask(Some("author-1"), overturn.clone()),
        Reply::Refused { why } if why.contains("overturn is a person's")
    ));
    assert_eq!(
        ask(None, overturn),
        Reply::Done {
            text: "decision 1 is overturned to option 3".into()
        }
    );
    let Reply::Done { text } = ask(None, Command::Status) else {
        panic!("status refused");
    };
    assert!(
        text.contains(
            "chose pigeon, by author-1: the towers are down past Sto Lat; cost medium, point \
             merge; overturned to option 3 (Post Office): the Post Office delivers in the rain\n"
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "1 proposed: Which way does the clacks message go? Prefer: Post Office. Why: the Post \
             Office delivers in the rain"
        ),
        "{text}"
    );
    assert_eq!(host.world().issues[&issue].stage, Stage::Working);
}

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
                draft: false,
                branch: String::new(),
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
            Reply::View { view } => panic!("a view: {view:?}"),
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
                state: None,
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
            replies: Vec::new(),
        };
        run(&mut host, issue, Role::Author, ready);
    }
    for issue in &issues {
        let approve = Command::Review {
            issue: issue.clone(),
            verdict: Verdict::Approve,
            failed: Vec::new(),
            open: Vec::new(),
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
    /// klatch's `machine` sends its refiner to mac too.
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

[project.settings]
machine = "mac"

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
                runtime: None,
                model: None,
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
                cost: None,
                point: None,
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
                failed: Vec::new(),
                open: Vec::new(),
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
                        due: None,
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

/// `change`, `show`, `search` and `ci` reach the daemon over its socket, as a
/// JSON request and through the command line, and read a repository the test
/// makes on disk as the code host's origin. The reads that use git come back as
/// work the daemon runs on a thread of its own.
mod reads {
    use std::io::{BufRead, BufReader, Write};
    use std::path::{Path, PathBuf};

    use temet_work::socket::{self, Answered, Reply};
    use temet_work_codehost::{
        CheckRun, Checks, CodeHosts, FakeCodeHost, Pull, PullState, RunState,
    };
    use temet_work_core::host::Memory;
    use temet_work_core::ids::{IssueId, ProjectId};
    use temet_work_core::launch::SECRET_VAR;
    use temet_work_core::model::Stage;
    use temet_work_core::{Event, Host, World};

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.name=Sam Vimes",
                "-c",
                "user.email=vimes@example.com",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// A bare origin for city/ankh: main with a README, and pull request 57
    /// adding a clock. Gives the origin and the first commit's id.
    fn origin(dir: &Path) -> (PathBuf, String) {
        let bare = dir.join("ankh.git");
        let work = dir.join("ankh-work");
        for d in [&bare, &work] {
            std::fs::create_dir_all(d).expect("dir");
        }
        git(&bare, &["init", "--bare", "--quiet", "-b", "main"]);
        git(&work, &["init", "--quiet", "-b", "main"]);
        std::fs::write(work.join("README.md"), "Ankh-Morpork\n").expect("write");
        git(&work, &["add", "README.md"]);
        git(&work, &["commit", "--quiet", "-m", "The city"]);
        let first = git(&work, &["rev-parse", "HEAD"]);
        git(
            &work,
            &[
                "push",
                "--quiet",
                &bare.to_string_lossy(),
                "HEAD:refs/heads/main",
            ],
        );
        std::fs::write(work.join("clock.rs"), "pub fn strike() -> u8 { 13 }\n").expect("write");
        git(&work, &["add", "clock.rs"]);
        git(&work, &["commit", "--quiet", "-m", "Mend the clock"]);
        git(
            &work,
            &[
                "push",
                "--quiet",
                &bare.to_string_lossy(),
                "HEAD:refs/pull/57/head",
            ],
        );
        (bare, first)
    }

    /// A host whose code-host family keeps its copies under `dir` and reads
    /// city/ankh from `bare`, with ankh#1 in review on pull request 57, whose
    /// rust check failed.
    fn host(dir: &Path, bare: &Path) -> Host {
        let mut w = World::default();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("ankh"),
            tracker: "fake".into(),
            name: "city/ankh".into(),
            code_host: None,
            repo: None,
        });
        w.apply(&Event::IssueSynced {
            issue: IssueId::new("ankh", "1"),
            title: "Mend the clock".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        let i = w.issues.get_mut(&IssueId::new("ankh", "1")).expect("issue");
        i.stage = Stage::InReview { round: 1 };
        i.pulls.insert(ProjectId::new("ankh"), "57".into());
        let mut fake = FakeCodeHost {
            pulls: vec![(
                "city/ankh".into(),
                Pull {
                    number: "57".into(),
                    state: PullState::Open,
                    checks: Checks::Failed {
                        failing: vec!["rust".into()],
                    },
                    conflicting: false,
                    draft: false,
                    branch: "ankh-1-clock".into(),
                },
            )],
            ..FakeCodeHost::default()
        };
        fake.clones
            .insert("city/ankh".into(), bare.to_string_lossy().into_owned());
        fake.runs.push((
            "city/ankh".into(),
            "57".into(),
            vec![CheckRun {
                name: "rust".into(),
                state: RunState::Failed,
                log: Some("test clock::strikes_twelve ... FAILED\n".into()),
                summary: String::new(),
                address: None,
            }],
        ));
        let mut family = CodeHosts::default().copies(dir.join("copies"));
        family.add(Box::new(fake));
        Host::new(w, vec![Box::new(family)], Box::new(Memory::default()))
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("daemon-reads-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch");
        dir
    }

    /// Answers `n` requests from `requests` as the daemon does: a read of code
    /// on a thread of its own.
    fn serve(host: &mut Host, requests: &std::sync::mpsc::Receiver<socket::Pending>, n: usize) {
        let mut threads = Vec::new();
        for _ in 0..n {
            let pending = requests
                .recv_timeout(std::time::Duration::from_secs(10))
                .expect("a request");
            match socket::answer_or_defer(host, 1, &pending.request, &pending.peer).expect("answer")
            {
                Answered::Now(reply) => pending.answer(reply),
                Answered::Later(work) => {
                    threads.push(std::thread::spawn(move || pending.answer(work())));
                }
            }
        }
        for t in threads {
            t.join().expect("read thread");
        }
    }

    #[test]
    fn each_read_answers_a_json_request_on_the_socket() {
        let dir = scratch("json");
        let (bare, first) = origin(&dir);
        let mut host = host(&dir, &bare);
        let path = dir.join("daemon.sock");
        let requests = socket::listen(&path).expect("listen");
        let lines = [
            r#"{"by":null,"command":{"command":"change","issue":{"project":"ankh","key":"1"}}}"#.to_string(),
            r#"{"by":null,"command":{"command":"show","project":"ankh","path":"clock.rs","at":{"issue":{"project":"ankh","key":"1"}}}}"#.into(),
            format!(
                r#"{{"by":null,"command":{{"command":"search","project":"ankh","pattern":"Ankh","at":{{"commit":"{first}"}}}}}}"#
            ),
            r#"{"by":null,"command":{"command":"ci","issue":{"project":"ankh","key":"1"}}}"#.into(),
            r#"{"by":null,"command":{"command":"show","project":"ankh","path":"clock.rs"}}"#.into(),
        ];
        let client = {
            let path = path.clone();
            std::thread::spawn(move || {
                lines
                    .iter()
                    .map(|line| {
                        let mut stream =
                            std::os::unix::net::UnixStream::connect(&path).expect("connect");
                        writeln!(stream, "{line}").expect("send");
                        let mut reply = String::new();
                        BufReader::new(stream).read_line(&mut reply).expect("reply");
                        serde_json::from_str::<Reply>(&reply).expect("a reply")
                    })
                    .collect::<Vec<_>>()
            })
        };
        serve(&mut host, &requests, 5);
        let replies = client.join().expect("client");
        let done = |r: &Reply| match r {
            Reply::Done { text } => text.clone(),
            other => panic!("not done: {other:?}"),
        };
        let change = done(&replies[0]);
        assert!(
            change.starts_with("ankh (city/ankh): pull request 57, 1 commit from"),
            "{change}"
        );
        assert!(change.contains("+pub fn strike() -> u8 { 13 }"), "{change}");
        assert_eq!(done(&replies[1]), "pub fn strike() -> u8 { 13 }\n");
        assert_eq!(done(&replies[2]), "README.md:1:Ankh-Morpork\n");
        let ci = done(&replies[3]);
        assert!(
            ci.contains("  rust: failed\n") && ci.contains("strikes_twelve ... FAILED"),
            "{ci}"
        );
        assert_eq!(
            replies[4],
            Reply::Refused {
                why: "ankh (city/ankh) has no clock.rs at the default branch's head".into()
            }
        );
        // The reads changed nothing: no event was logged.
        assert_eq!(host.world().applied, 2);
    }

    #[test]
    fn the_command_line_prints_each_read_and_exits_1_on_a_refusal() {
        let dir = scratch("cli");
        let (bare, _) = origin(&dir);
        let mut host = host(&dir, &bare);
        let path = dir.join("daemon.sock");
        let requests = socket::listen(&path).expect("listen");
        let cases: Vec<Vec<&str>> = vec![
            vec!["change", "ankh#1", "--by-commit"],
            vec!["show", "ankh", "", "--at", "ankh#1"],
            vec!["search", "ankh", "strike|Ankh", "--at", "ankh#1"],
            vec!["search", "ankh", "strike"],
            vec!["ci", "ankh#1"],
            vec!["show", "ankh", "clock.rs"],
            vec!["show", "ankh", "README.md", "--at", "deadbeef"],
            vec!["show", "lancre", "README.md"],
            vec!["change", "ankh#2"],
        ];
        let n = cases.len();
        let client = {
            let path = path.clone();
            std::thread::spawn(move || {
                cases
                    .into_iter()
                    .map(|args| {
                        std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
                            .args(&args)
                            .env("TEMET_WORK_SOCKET", &path)
                            .env_remove("TEMET_WORK_AGENT")
                            .env_remove(SECRET_VAR)
                            .env_remove("TEMET_WORK_DAEMON")
                            .output()
                            .expect("runs")
                    })
                    .collect::<Vec<_>>()
            })
        };
        serve(&mut host, &requests, n);
        let out = client.join().expect("client");
        let printed = |i: usize| {
            assert_eq!(
                out[i].status.code(),
                Some(0),
                "{i}: {}",
                String::from_utf8_lossy(&out[i].stderr)
            );
            String::from_utf8_lossy(&out[i].stdout).into_owned()
        };
        let refused = |i: usize| {
            assert_eq!(
                out[i].status.code(),
                Some(1),
                "{i}: {}",
                String::from_utf8_lossy(&out[i].stdout)
            );
            String::from_utf8_lossy(&out[i].stderr).into_owned()
        };
        let each = printed(0);
        assert!(
            each.contains("Mend the clock") && each.contains("each commit:"),
            "{each}"
        );
        assert_eq!(printed(1), "README.md\nclock.rs\n");
        assert_eq!(
            printed(2),
            "README.md:1:Ankh-Morpork\nclock.rs:1:pub fn strike() -> u8 { 13 }\n"
        );
        assert_eq!(
            printed(3),
            "no line matches strike in ankh (city/ankh) at the default branch's head\n"
        );
        assert!(printed(4).contains("The output of rust's failed steps:"));
        assert!(
            refused(5).contains("ankh (city/ankh) has no clock.rs at the default branch's head")
        );
        assert!(refused(6).contains("ankh (city/ankh) has no commit deadbeef"));
        assert!(refused(7).contains("there is no project lancre"));
        assert!(refused(8).contains("there is no issue ankh#2"));
    }
}

/// A person's move through the daemon: its reply, and a second `temet-workd`
/// that keeps it through a restart.
mod moves {
    use super::*;
    use temet_work::socket::{Reply, answer};
    use temet_work_core::view::StageWord;

    #[test]
    fn a_moves_reply_says_the_stage_and_names_the_pull_request_left_open() {
        let (mut host, starts) = host();
        host.tick(0).expect("sync");
        let issue = IssueId::new("ankh", "1");
        let ask = |host: &mut Host, by: Option<&str>, command: Command| match answer(
            host,
            1,
            &from(&starts, by, command),
            &Peer::Unchecked,
        )
        .expect("answer")
        {
            Reply::Done { text } => text,
            Reply::Refused { why } => panic!("refused: {why}"),
            Reply::View { view } => panic!("a view: {view:?}"),
        };
        plan_one_check(&mut host, &starts);
        let ready = Command::Ready {
            issue: issue.clone(),
            pull: Some("57".into()),
            pulls: Default::default(),
            replies: Vec::new(),
        };
        ask(&mut host, Some("author-1"), ready);
        let back = |stage| Command::Move {
            issue: issue.clone(),
            stage,
        };
        // author-1's stop works at once, so a new author takes ankh-1 in the
        // same round: the issue is working again, with no pull request.
        assert_eq!(
            ask(&mut host, None, back(StageWord::Ready)),
            "ankh#1 is working; the move dropped its pull request 57, which is left open on \
             the code host for you to close or reuse"
        );
        assert_eq!(
            host.world().agents[&AgentId::new("author-1")].state,
            temet_work_core::model::AgentState::Ended
        );
        assert!(host.world().agents.contains_key(&AgentId::new("author-2")));
        // Nothing dropped: the stage alone. Back to new it is taken for
        // refining again at once, by a fresh refiner.
        assert_eq!(
            ask(&mut host, None, back(StageWord::New)),
            "ankh#1 is refining"
        );
        assert!(host.world().agents.contains_key(&AgentId::new("refiner-2")));
        // A refused move says why, and an agent's is refused as overturn is.
        let refused = answer(
            &mut host,
            1,
            &from(&starts, None, back(StageWord::Merging)),
            &Peer::Unchecked,
        )
        .expect("answer");
        assert!(
            matches!(&refused, Reply::Refused { why } if why.contains("a step the loop or an agent owns")),
            "{refused:?}"
        );
        let refused = answer(
            &mut host,
            1,
            &from(&starts, Some("refiner-2"), back(StageWord::New)),
            &Peer::Unchecked,
        )
        .expect("answer");
        assert!(
            matches!(&refused, Reply::Refused { why } if why.starts_with("move is a person's: refiner-2 is an agent")),
            "{refused:?}"
        );
    }

    /// A folder of the test's own under the target folder, emptied first, for
    /// a second daemon's configuration, log and socket.
    fn real_run(name: &str) -> std::path::PathBuf {
        let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a folder");
        dir
    }

    /// A `temet-workd` on the configuration and log in `dir`, answering on
    /// its own socket there, once that socket answers.
    fn daemon(dir: &std::path::Path) -> std::process::Child {
        let socket = dir.join("d.sock");
        let child = std::process::Command::new(env!("CARGO_BIN_EXE_temet-workd"))
            .arg("--config")
            .arg(dir.join("config.toml"))
            .arg("--log")
            .arg(dir.join("events.jsonl"))
            .env("TEMET_WORK_SOCKET", &socket)
            .env("TEMET_WORK_SANDBOX", "1")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("the daemon runs");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while std::os::unix::net::UnixStream::connect(&socket).is_err() {
            assert!(
                std::time::Instant::now() < deadline,
                "the daemon did not answer"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        child
    }

    /// `temet-work ARGS` against the daemon in `dir`, as a person: its
    /// standard output, after a success.
    fn cli(dir: &std::path::Path, args: &[&str]) -> String {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
            .args(args)
            .env("TEMET_WORK_SOCKET", dir.join("d.sock"))
            .env_remove("TEMET_WORK_DAEMON")
            .env_remove("TEMET_WORK_AGENT")
            .env_remove(SECRET_VAR)
            .output()
            .expect("runs");
        assert!(out.status.success(), "{args:?}: {out:?}");
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    #[cfg(all(feature = "store", feature = "tracker-github"))]
    #[test]
    fn a_move_through_a_second_daemon_outlasts_its_restart() {
        use temet_work_core::Event;
        use temet_work_core::ids::{GroupId, ProjectId};
        use temet_work_core::model::{Check, Plan};

        let dir = real_run("daemon-move");
        std::fs::write(
            dir.join("config.toml"),
            "[[project]]\nid = \"ankh\"\ntracker = \"github\"\nname = \"ankh-morpork/watch\"\n\
             [[group]]\nid = \"watch\"\nname = \"Night watch\"\nprojects = [\"ankh\"]\n",
        )
        .expect("a config");
        // ankh#1 planned and ready, with no folder for an author; ankh#2 holds
        // the group's one refining slot, so ankh#1 stays at new once moved.
        let (mut log, _) = temet_work_store::Log::open(&dir.join("events.jsonl")).expect("a log");
        let synced = |key: &str, order| Event::IssueSynced {
            issue: IssueId::new("ankh", key),
            title: format!("Mend clock {key}"),
            milestone: None,
            depends_on: vec![],
            order,
            closed: false,
        };
        for event in [
            Event::LogStarted { id: "lamp".into() },
            Event::ProjectAdded {
                project: ProjectId::new("ankh"),
                tracker: "github".into(),
                name: "ankh-morpork/watch".into(),
                code_host: Some("github".into()),
                repo: Some("ankh-morpork/watch".into()),
            },
            Event::GroupCreated {
                group: GroupId::new("watch"),
                name: "Night watch".into(),
                projects: vec![ProjectId::new("ankh")],
                areas: vec![],
                milestones: vec![],
                refining_at_once: 1,
                model: None,
                queue_low: None,
                investigate_every: None,
            },
            synced("1", 1),
            Event::PlanWritten {
                issue: IssueId::new("ankh", "1"),
                plan: Plan {
                    checks: vec![Check::new("it strikes on time", "listening at noon")],
                    ..Plan::default()
                },
            },
            synced("2", 2),
        ] {
            log.append(1, &event).expect("logged");
        }
        // The daemon takes the log's lock.
        drop(log);

        let mut first = daemon(&dir);
        let moved = cli(&dir, &["move", "ankh#1", "new"]);
        let refused = std::process::Command::new(env!("CARGO_BIN_EXE_temet-work"))
            .args(["move", "ankh#1", "sideways"])
            .env("TEMET_WORK_SOCKET", dir.join("d.sock"))
            .output()
            .expect("runs");
        first.kill().expect("stopped");
        first.wait().expect("ended");
        assert_eq!(moved.trim_end(), "ankh#1 is new");
        assert!(!refused.status.success());
        assert!(
            String::from_utf8_lossy(&refused.stderr)
                .contains("no stage sideways; the stages are new"),
            "{refused:?}"
        );
        let log = std::fs::read_to_string(dir.join("events.jsonl")).expect("the log");
        assert_eq!(log.matches(r#""Moved""#).count(), 1, "{log}");

        // Restarted on the same log, the daemon has it at new.
        let mut again = daemon(&dir);
        let status = cli(&dir, &["status"]);
        let json = cli(&dir, &["status", "--json"]);
        again.kill().expect("stopped");
        again.wait().expect("ended");
        let line = status
            .lines()
            .find(|l| l.contains("ankh#1"))
            .unwrap_or_else(|| panic!("no ankh#1 in {status}"));
        assert!(line.contains("new") && !line.contains("ready"), "{line}");
        let view: temet_work_core::View = serde_json::from_str(&json).expect("a view");
        let shown = view
            .issues
            .iter()
            .find(|i| i.id == "ankh#1")
            .expect("listed");
        assert_eq!(
            (shown.stage, shown.moves.as_slice()),
            (StageWord::New, &[][..])
        );
    }
}

/// The fake tracker, still in the test's reach once the host has it, which
/// refuses every retitle with the reason in `refuse` while it holds one.
#[derive(Clone, Default)]
struct Titled {
    fake: std::sync::Arc<std::sync::Mutex<FakeTracker>>,
    refuse: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

impl Titled {
    fn title(&self, key: &str) -> String {
        let fake = self.fake.lock().expect("fake");
        let found = fake.issues.iter().find(|(_, i)| i.key == key);
        found.map(|(_, i)| i.title.clone()).unwrap_or_default()
    }
}

impl temet_work_tracker::Tracker for Titled {
    fn id(&self) -> &'static str {
        "fake"
    }
    fn issues(
        &mut self,
        project: &str,
    ) -> Result<Vec<TrackedIssue>, temet_work_tracker::TrackerError> {
        self.fake.lock().expect("fake").issues(project)
    }
    fn milestones(
        &mut self,
        project: &str,
    ) -> Result<Vec<temet_work_tracker::TrackedMilestone>, temet_work_tracker::TrackerError> {
        self.fake.lock().expect("fake").milestones(project)
    }
    fn address(&self, project: &str) -> Option<String> {
        self.fake.lock().expect("fake").address(project)
    }
    fn closed(
        &mut self,
        project: &str,
        key: &str,
    ) -> Result<bool, temet_work_tracker::TrackerError> {
        self.fake.lock().expect("fake").closed(project, key)
    }
    fn comment(
        &mut self,
        project: &str,
        key: &str,
        text: &str,
    ) -> Result<(), temet_work_tracker::TrackerError> {
        self.fake.lock().expect("fake").comment(project, key, text)
    }
    fn close(&mut self, project: &str, key: &str) -> Result<(), temet_work_tracker::TrackerError> {
        self.fake.lock().expect("fake").close(project, key)
    }
    fn reopen(&mut self, project: &str, key: &str) -> Result<(), temet_work_tracker::TrackerError> {
        self.fake.lock().expect("fake").reopen(project, key)
    }
    fn retitle(
        &mut self,
        project: &str,
        key: &str,
        title: &str,
    ) -> Result<(), temet_work_tracker::TrackerError> {
        if let Some(why) = self.refuse.lock().expect("refuse").clone() {
            return Err(temet_work_tracker::TrackerError::Denied(why));
        }
        self.fake.lock().expect("fake").retitle(project, key, title)
    }
    fn create(
        &mut self,
        project: &str,
        title: &str,
        body: &str,
        milestone: Option<&str>,
    ) -> Result<TrackedIssue, temet_work_tracker::TrackerError> {
        self.fake
            .lock()
            .expect("fake")
            .create(project, title, body, milestone)
    }
}

/// A host whose tracker is a [`Titled`] fake listing ankh#1, synced once so a
/// refiner works on ankh#1; the log it keeps, and the fake.
fn titled() -> (Host, Starts, Kept, Titled) {
    let kept = Kept::default();
    let titled = Titled::default();
    titled.fake.lock().expect("fake").issues = vec![tracked("city/ankh", "1", 1)];
    let (mut host, starts) = host_with_tracker(
        World::default(),
        Box::new(kept.clone()),
        fake_runtimes(),
        CONFIG,
        None,
        Box::new(titled.clone()),
    );
    host.tick(0).expect("sync");
    assert_eq!(
        host.world().issues[&IssueId::new("ankh", "1")].stage,
        Stage::Refining
    );
    (host, starts, kept, titled)
}

fn refine_ankh_1(title: Option<&str>, checks: &[(&str, &str)], send: bool) -> Command {
    Command::Refine {
        issue: IssueId::new("ankh", "1"),
        title: title.map(String::from),
        checks: (!checks.is_empty()).then(|| {
            checks
                .iter()
                .map(|(d, p)| temet_work_core::model::Check::new(*d, *p))
                .collect()
        }),
        send,
    }
}

#[test]
fn a_persons_refine_writes_the_title_to_the_tracker_first_and_status_and_the_view_show_it() {
    use temet_work::socket::{Reply, answer};

    let (mut host, starts, kept, titled) = titled();
    let ask = |host: &mut Host, by: Option<&str>, command| {
        answer(host, 1, &from(&starts, by, command), &Peer::Unchecked).expect("answer")
    };
    let reply = ask(
        &mut host,
        None,
        refine_ankh_1(
            Some("Mend the clock by dusk"),
            &[("the clock strikes six at dusk", "listening at dusk")],
            false,
        ),
    );
    assert_eq!(
        reply,
        Reply::Done {
            text: "ankh#1: its title is now \"Mend the clock by dusk\" in its tracker; it has 1 \
                   person's check"
                .into()
        }
    );
    assert_eq!(
        titled.title("1"),
        "Mend the clock by dusk",
        "written to the tracker"
    );
    let Reply::Done { text } = ask(&mut host, None, Command::Status) else {
        panic!("status refused");
    };
    assert!(
        text.contains("\n  ankh#1  refining  Mend the clock by dusk\n"),
        "{text}"
    );
    let Reply::Done { text } = ask(
        &mut host,
        None,
        Command::Issue {
            issue: IssueId::new("ankh", "1"),
        },
    ) else {
        panic!("issue refused");
    };
    assert!(
        text.starts_with("ankh#1  Mend the clock by dusk\n"),
        "{text}"
    );
    assert!(
        text.contains(
            "Person's checks:\n  - the clock strikes six at dusk (shown by: listening at dusk)\n"
        ),
        "{text}"
    );
    let Reply::View { view } = ask(&mut host, None, Command::View) else {
        panic!("view refused");
    };
    // As `status --json` prints it.
    let json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string_pretty(&view).expect("json")).expect("reads");
    assert_eq!(json["issues"][0]["title"], "Mend the clock by dusk");
    assert_eq!(
        json["issues"][0]["person_checks"],
        serde_json::json!([{
            "done_when": "the clock strikes six at dusk",
            "proven_by": "listening at dusk"
        }])
    );
    // The next sync lists the written title: nothing changes.
    host.tick(120_000).expect("sync");
    assert_eq!(
        host.world().issues[&IssueId::new("ankh", "1")].title,
        "Mend the clock by dusk"
    );
    assert_eq!(kept.replayed(host.world()).issues, host.world().issues);
}

#[test]
fn a_tracker_that_refuses_the_title_refuses_the_refine_and_nothing_is_logged() {
    use temet_work::socket::{Reply, answer};

    let (mut host, starts, kept, titled) = titled();
    *titled.refuse.lock().expect("refuse") = Some("HTTP 403: the guild says no".into());
    let logged = kept.0.lock().expect("log").len();
    let request = from(
        &starts,
        None,
        refine_ankh_1(Some("Mend the clock by dusk"), &[("a check", "")], true),
    );
    let reply = answer(&mut host, 1, &request, &Peer::Unchecked).expect("answer");
    let Reply::Refused { why } = reply else {
        panic!("taken: {reply:?}");
    };
    assert!(
        why.contains("did not take the title") && why.contains("HTTP 403: the guild says no"),
        "{why}"
    );
    assert_eq!(
        kept.0.lock().expect("log").len(),
        logged,
        "the log gets no new line"
    );
    let i = &host.world().issues[&IssueId::new("ankh", "1")];
    assert_eq!(i.title, "Mend the clock");
    assert!(i.person_checks.is_empty());
    assert_eq!(titled.title("1"), "Mend the clock");
}

#[test]
fn a_send_through_the_daemon_stops_the_refiner_and_starts_one_with_the_persons_checks() {
    use temet_work::socket::{Reply, answer};

    let (mut host, starts, _, _) = titled();
    let request = from(
        &starts,
        None,
        refine_ankh_1(None, &[("the clock strikes six at dusk", "")], true),
    );
    let reply = answer(&mut host, 1, &request, &Peer::Unchecked).expect("answer");
    assert_eq!(
        reply,
        Reply::Done {
            text: "ankh#1: it has 1 person's check; it is refining, and a refiner starts with a \
                   new brief; refiner-1 is stopped"
                .into()
        }
    );
    let started = starts
        .lock()
        .expect("starts")
        .last()
        .cloned()
        .expect("a start");
    assert_eq!(
        started.labels.get("agent").map(String::as_str),
        Some("refiner-2")
    );
    let brief = format!("{:?}", started.program);
    assert!(
        brief.contains("The person's checks, each a point your plan must hold:")
            && brief.contains("the clock strikes six at dusk"),
        "{brief}"
    );
    // A refine that changes nothing logs nothing and says so.
    let same = from(
        &starts,
        None,
        refine_ankh_1(Some("Mend the clock"), &[], false),
    );
    assert_eq!(
        answer(&mut host, 1, &same, &Peer::Unchecked).expect("answer"),
        Reply::Done {
            text: "nothing changed on ankh#1".into()
        }
    );
}
