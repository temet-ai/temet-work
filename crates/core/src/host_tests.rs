use std::sync::{Arc, Mutex};

use super::*;
use crate::ids::{AgentId, FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId};
use crate::launch::Program;
use crate::model::{AgentState, Plan, Role};

/// A journal the test can read after the host has taken it.
#[derive(Clone, Default)]
struct Shared(Arc<Mutex<Vec<Event>>>, bool);

impl Journal for Shared {
    fn append(&mut self, _at_ms: u64, event: &Event) -> Result<(), String> {
        if self.1 {
            return Err("disk full".into());
        }
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .push(event.clone());
        Ok(())
    }
}

/// Turns a decision to start an agent into a program, as an agent runtime does.
struct Brains;

impl Part for Brains {
    fn id(&self) -> &'static str {
        "brains"
    }
    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let launch = match effect {
            Effect::StartAgent {
                group,
                role,
                issue,
                folder,
            } => world.launch(group, *role, issue, folder.as_ref()),
            Effect::Investigate { group } => world.launch_investigator(group),
            _ => return Outcome::NotMine,
        };
        let Some(launch) = launch else {
            return Outcome::Failed("unknown issue".into());
        };
        let program = Program {
            argv: vec!["agent".into(), launch.brief.task.clone()],
            env: vec![],
        };
        Outcome::Done {
            events: vec![],
            effects: vec![Effect::Run {
                launch: Box::new(launch),
                runtime: "brains".into(),
                program,
                before: None,
            }],
        }
    }
}

/// Runs programs, as a terminal runner does, and reports what it was told to.
#[derive(Default)]
struct Terms {
    polls: Vec<Polled>,
}

impl Part for Terms {
    fn id(&self) -> &'static str {
        "terms"
    }
    fn poll(&mut self, _world: &World, _now_ms: u64) -> Polled {
        if self.polls.is_empty() {
            Polled::default()
        } else {
            self.polls.remove(0)
        }
    }
    fn carry_out(&mut self, effect: &Effect, _world: &World) -> Outcome {
        let Effect::Run { launch, .. } = effect else {
            return Outcome::NotMine;
        };
        Outcome::Done {
            events: vec![Event::AgentStarted {
                agent: launch.agent.clone(),
                group: launch.group.clone(),
                role: launch.brief.role,
                issue: launch.brief.issue.clone(),
                machine: launch.machine.clone(),
                folder: launch.folder.clone(),
                secret_digest: None,
                runtime: None,
                model: None,
            }],
            effects: vec![],
        }
    }
}

fn setup(refining_at_once: usize) -> Vec<Event> {
    vec![
        Event::ProjectAdded {
            project: ProjectId::new("ankh"),
            tracker: "fake".into(),
            name: "city/ankh".into(),
            code_host: None,
            repo: None,
        },
        Event::FolderAdded {
            folder: FolderId::new("f1"),
            project: ProjectId::new("ankh"),
            machine: MachineId::new("morpork"),
            path: "/srv/ankh/f1".into(),
        },
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![ProjectId::new("ankh")],
            areas: Vec::new(),
            milestones: vec![],
            refining_at_once,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
    ]
}

fn synced(key: &str) -> Event {
    Event::IssueSynced {
        issue: IssueId::new("ankh", key),
        title: format!("Mend clock {key}"),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: key.parse().unwrap_or(0),
        closed: false,
    }
}

fn host(parts: Vec<Box<dyn Part>>, journal: Shared) -> Host {
    let mut h = Host::new(World::default(), parts, Box::new(journal));
    for e in setup(2) {
        h.handle(0, e).expect("setup");
    }
    h
}

#[test]
fn each_event_is_applied_at_the_time_it_is_logged_with() {
    let mut h = host(Vec::new(), Shared::default());
    let issue = IssueId::new("ankh", "1");
    h.handle(5_000, synced("1")).expect("handle");
    assert_eq!(h.world().updated.at(&issue), Some(5_000));
    // The same again changes nothing, so the time stays.
    h.handle(7_000, synced("1")).expect("handle");
    assert_eq!(h.world().updated.at(&issue), Some(5_000));
    let renamed = Event::IssueSynced {
        issue: issue.clone(),
        title: "Mend the clock tower".into(),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: 1,
        closed: false,
    };
    h.handle(9_000, renamed).expect("handle");
    assert_eq!(h.world().updated.at(&issue), Some(9_000));
}

/// Gives the web addresses of every project, as a tracker or code-host family
/// does: `issues` from one part, `code` from another.
struct Places {
    issues: Option<&'static str>,
    code: Option<&'static str>,
}

impl Part for Places {
    fn id(&self) -> &'static str {
        "places"
    }

    fn addresses(&self, world: &World) -> BTreeMap<ProjectId, crate::model::Addresses> {
        world
            .projects
            .keys()
            .map(|id| {
                let at = |a: Option<&str>| a.map(|a| format!("{a}/{id}"));
                (
                    id.clone(),
                    crate::model::Addresses {
                        issues: at(self.issues),
                        code: at(self.code),
                    },
                )
            })
            .collect()
    }
}

#[test]
fn each_round_the_parts_addresses_are_put_in_the_world_and_the_first_part_wins() {
    let parts: Vec<Box<dyn Part>> = vec![
        Box::new(Places {
            issues: Some("https://tracker.example.com"),
            code: None,
        }),
        Box::new(Places {
            issues: Some("https://other.example.com"),
            code: Some("https://git.example.com"),
        }),
    ];
    let mut h = host(parts, Shared::default());
    let ankh = ProjectId::new("ankh");
    // The project came from the configuration after the host started: the next
    // round has it.
    assert!(!h.world().addresses.contains_key(&ankh));
    h.tick(0).expect("tick");
    assert_eq!(
        h.world().addresses[&ankh],
        crate::model::Addresses {
            issues: Some("https://tracker.example.com/ankh".into()),
            code: Some("https://git.example.com/ankh".into()),
        }
    );
}

#[test]
fn a_synced_issue_starts_a_refiner_through_the_parts_and_logs_it() {
    let journal = Shared::default();
    let mut h = host(
        vec![Box::new(Brains), Box::new(Terms::default())],
        journal.clone(),
    );
    h.handle(1, synced("1")).expect("handle");
    let agent = h
        .world()
        .agents
        .get(&AgentId::new("refiner-1"))
        .expect("refiner-1");
    assert_eq!(agent.issue, Some(IssueId::new("ankh", "1")));
    // A reader with no `machine` setting runs on this machine.
    assert_eq!(agent.machine, MachineId::new(crate::world::LOCAL));
    assert!(h.problems().is_empty(), "{:?}", h.problems());
    let logged = journal.0.lock().expect("journal").clone();
    assert_eq!(logged.len(), 5);
    assert!(matches!(logged[4], Event::AgentStarted { .. }));
}

#[test]
fn a_tick_starts_one_investigator_for_a_group_short_of_work_and_logs_its_start_and_end() {
    let journal = Shared::default();
    let mut h = host(
        vec![Box::new(Brains), Box::new(Terms::default())],
        journal.clone(),
    );
    h.tick(1_000).expect("tick");
    h.tick(2_000).expect("tick");
    let me = AgentId::new("investigator-1");
    let logged = journal.0.lock().expect("journal").clone();
    let since_setup = &logged[3..];
    assert_eq!(
        since_setup,
        [
            Event::InvestigationStarted {
                group: GroupId::new("watch"),
                agent: me.clone(),
                at_ms: 1_000,
            },
            Event::AgentStarted {
                agent: me.clone(),
                group: GroupId::new("watch"),
                role: Role::Investigator,
                issue: None,
                machine: MachineId::new(crate::world::LOCAL),
                folder: None,
                secret_digest: None,
                runtime: None,
                model: None,
            },
        ],
        "one start, the second tick none"
    );
    // Its word ends the investigation at once, stamped with the host's clock.
    h.handle(
        3_000,
        Event::Investigated {
            group: GroupId::new("watch"),
            agent: me,
        },
    )
    .expect("handle");
    let logged = journal.0.lock().expect("journal").clone();
    assert_eq!(
        logged.last(),
        Some(&Event::InvestigationEnded {
            group: GroupId::new("watch"),
            at_ms: 3_000,
            filed: 0,
        })
    );
}

#[test]
fn agents_are_numbered_by_role_in_the_order_they_start() {
    let mut h = host(
        vec![Box::new(Brains), Box::new(Terms::default())],
        Shared::default(),
    );
    h.handle(1, synced("1")).expect("one");
    h.handle(2, synced("2")).expect("two");
    let ids: Vec<&str> = h.world().agents.keys().map(|a| a.0.as_str()).collect();
    assert_eq!(ids, ["refiner-1", "refiner-2"]);
}

#[test]
fn an_effect_no_part_takes_is_a_problem() {
    let mut h = host(vec![], Shared::default());
    h.handle(1, synced("1")).expect("handle");
    assert_eq!(
        h.problems()
            .get("start a refiner on ankh#1")
            .map(String::as_str),
        Some("no part in this build does this")
    );
}

#[test]
fn issue_text_no_part_reads_is_refused_with_the_reason() {
    let mut h = host(vec![Box::new(Terms::default())], Shared::default());
    h.handle(1, synced("1")).expect("handle");
    assert_eq!(
        h.issue_text(&IssueId::new("ankh", "1")),
        Err("no part in this build reads issue text".into())
    );
}

#[test]
fn a_failed_poll_is_a_problem_until_a_poll_succeeds() {
    let terms = Terms {
        polls: vec![
            Polled {
                events: vec![],
                problem: Some("socket refused".into()),
            },
            Polled {
                events: vec![Event::AgentChanged {
                    agent: AgentId::new("nobody"),
                    state: AgentState::Idle,
                    lost: false,
                }],
                problem: None,
            },
        ],
    };
    let mut h = host(vec![Box::new(terms)], Shared::default());
    assert_eq!(h.tick(1), Ok(0));
    assert_eq!(
        h.problems().get("terms").map(String::as_str),
        Some("socket refused")
    );
    assert_eq!(h.tick(2), Ok(1));
    assert!(h.problems().get("terms").is_none());
}

#[test]
fn a_journal_that_cannot_write_stops_the_host() {
    let mut h = Host::new(
        World::default(),
        vec![],
        Box::new(Shared(Arc::default(), true)),
    );
    assert_eq!(h.handle(0, synced("1")), Err("disk full".into()));
    assert_eq!(
        h.world().applied,
        0,
        "an event that was not logged is not applied"
    );
}

#[test]
fn a_part_that_keeps_asking_for_more_is_stopped() {
    struct Echo;
    impl Part for Echo {
        fn id(&self) -> &'static str {
            "echo"
        }
        fn carry_out(&mut self, effect: &Effect, _world: &World) -> Outcome {
            Outcome::Done {
                events: vec![],
                effects: vec![effect.clone()],
            }
        }
    }
    let mut h = Host::new(
        World::default(),
        vec![Box::new(Echo)],
        Box::new(Memory::default()),
    );
    let mut fails = Vec::new();
    for e in setup(1) {
        fails.push(h.handle(0, e));
    }
    fails.push(h.handle(0, synced("1")));
    assert!(
        fails
            .last()
            .expect("result")
            .as_ref()
            .is_err_and(|e| e.contains("steps"))
    );
}

#[test]
fn an_author_runs_in_its_folder_and_a_refiner_in_the_readers_folder_of_the_project() {
    let mut w = World {
        readers: Some("/srv/readers".into()),
        ..World::default()
    };
    for e in setup(1) {
        w.apply(&e);
    }
    w.apply(&synced("1"));
    w.apply(&Event::PlanWritten {
        issue: IssueId::new("ankh", "1"),
        plan: Plan {
            repos: Vec::new(),
            settings: Default::default(),
            checks: vec![crate::model::Check {
                done_when: "the clock strikes thirteen".into(),
                proven_by: "listening at midnight".into(),
                state: None,
            }],
        },
    });
    let group = GroupId::new("watch");
    let issue = IssueId::new("ankh", "1");
    let author = w
        .launch(&group, Role::Author, &issue, Some(&FolderId::new("f1")))
        .expect("author");
    assert_eq!(author.agent, AgentId::new("author-1"));
    assert_eq!(author.folder, Some(FolderId::new("f1")));
    assert_eq!(
        author.brief.folder.as_deref(),
        Some(std::path::Path::new("/srv/ankh/f1"))
    );
    assert!(author.brief.task.contains("the clock strikes thirteen"));
    assert!(author.brief.task.contains("temet-work ready ankh#1"));
    let refiner = w
        .launch(&group, Role::Refiner, &issue, None)
        .expect("refiner");
    assert_eq!(refiner.folder, None, "a refiner holds no folder");
    assert_eq!(
        refiner.brief.folder.as_deref(),
        Some(std::path::Path::new("/srv/readers/ankh"))
    );
    assert!(refiner.brief.task.contains("issue 1 of city/ankh"));
    assert!(
        w.launch(&group, Role::Refiner, &IssueId::new("ankh", "9"), None)
            .is_none()
    );
}

#[test]
fn an_issue_a_failed_verify_sent_back_is_a_problem_until_it_is_done() {
    let mut h = host(vec![], Shared::default());
    let one = IssueId::new("ankh", "1");
    let closed = |closed: bool| Event::IssueSynced {
        issue: IssueId::new("ankh", "1"),
        title: "Mend clock 1".into(),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: 1,
        closed,
    };
    for e in [
        synced("1"),
        Event::PlanWritten {
            issue: one.clone(),
            plan: Plan::default(),
        },
        Event::PullRequestReady {
            issue: one.clone(),
            pull: Some("57".into()),
            pulls: Default::default(),
            replies: Vec::new(),
        },
        Event::ReviewFinished {
            issue: one.clone(),
            round: 1,
            verdict: crate::event::Verdict::Approve,
            failed: None,
            open: None,
        },
        Event::ChecksFinished {
            issue: one.clone(),
            passed: true,
            failing: Vec::new(),
        },
        Event::Merged {
            issue: one.clone(),
            repo: None,
        },
        Event::Deployed { issue: one.clone() },
    ] {
        h.handle(1, e).expect("handle");
    }
    let key = "verify failed: ankh#1";
    assert!(!h.problems().contains_key(key), "not before the verify");
    h.handle(
        2,
        Event::Verified {
            issue: one.clone(),
            passed: false,
            output: "the clock struck twelve".into(),
        },
    )
    .expect("handle");
    assert_eq!(
        h.problems().get(key).map(String::as_str),
        Some(
            "the merged change of ankh#1 failed the verify check and is being fixed; once the \
             daemon has reopened it in its tracker, closing it there finishes it"
        )
    );
    let status = crate::text::text(h.world(), h.problems());
    let problems = status.split_once("\nProblems\n").map(|(_, rest)| rest);
    assert!(
        problems.is_some_and(|p| p.contains(
            "  verify failed: ankh#1: the merged change of ankh#1 failed the verify check"
        )),
        "{status}"
    );
    let view = crate::View::new(h.world(), h.problems());
    assert_eq!(view.problems.get(key), h.problems().get(key));

    // Listed as open again: the reopen happened.
    h.handle(3, closed(false)).expect("handle");
    assert_eq!(
        h.problems().get(key).map(String::as_str),
        Some(
            "the merged change of ankh#1 failed the verify check and is being fixed; closing it \
             in its tracker finishes it"
        )
    );
    h.handle(4, closed(true)).expect("handle");
    assert_eq!(h.world().issues[&one].stage, crate::model::Stage::Done);
    assert!(!h.problems().contains_key(key));
    let status = crate::text::text(h.world(), h.problems());
    assert!(!status.contains("verify failed"), "{status}");
    let view = crate::View::new(h.world(), h.problems());
    assert!(!view.problems.contains_key(key));
}

fn ankh_one() -> IssueId {
    IssueId::new("ankh", "1")
}

/// A ready that names `pulls`, by repository; an area's issue has several.
fn ready(issue: &IssueId, pulls: &[(&str, &str)]) -> Event {
    Event::PullRequestReady {
        issue: issue.clone(),
        pull: None,
        pulls: pulls
            .iter()
            .map(|(repo, number)| (ProjectId::new(*repo), (*number).to_string()))
            .collect(),
        replies: Vec::new(),
    }
}

fn reviewed(issue: &IssueId, round: u8, verdict: crate::event::Verdict) -> Event {
    Event::ReviewFinished {
        issue: issue.clone(),
        round,
        verdict,
        failed: None,
        open: None,
    }
}

/// An approving review of `round` and passing checks.
fn approved(issue: &IssueId, round: u8) -> [Event; 2] {
    [
        reviewed(issue, round, crate::event::Verdict::Approve),
        Event::ChecksFinished {
            issue: issue.clone(),
            passed: true,
            failing: Vec::new(),
        },
    ]
}

fn merged(issue: &IssueId, repo: &str) -> Event {
    Event::Merged {
        issue: issue.clone(),
        repo: Some(ProjectId::new(repo)),
    }
}

fn failed_verify(issue: &IssueId) -> Event {
    Event::Verified {
        issue: issue.clone(),
        passed: false,
        output: "the clock struck twelve".into(),
    }
}

fn feed(h: &mut Host, events: impl IntoIterator<Item = Event>) {
    for e in events {
        h.handle(1, e).expect("handle");
    }
}

/// Takes a planned issue of ankh to a failed verify.
fn fail_the_verify(h: &mut Host, issue: &IssueId) {
    feed(h, [ready(issue, &[("ankh", "57")])]);
    feed(h, approved(issue, 1));
    feed(
        h,
        [
            merged(issue, "ankh"),
            Event::Deployed {
                issue: issue.clone(),
            },
            failed_verify(issue),
        ],
    );
}

fn planned(issue: &IssueId) -> Event {
    Event::PlanWritten {
        issue: issue.clone(),
        plan: Plan::default(),
    }
}

/// Asserts the whole failed-verify line of `issue`, in the host's problems, the
/// status text and the view.
fn assert_verify_line(h: &Host, issue: &IssueId, line: &str) {
    let key = format!("verify failed: {issue}");
    assert_eq!(h.problems().get(&key).map(String::as_str), Some(line));
    let status = crate::text::text(h.world(), h.problems());
    let listed = format!("  {key}: {line}");
    assert!(
        status
            .split_once("\nProblems\n")
            .is_some_and(|(_, rest)| rest.lines().any(|l| l == listed)),
        "{status}"
    );
    let view = crate::View::new(h.world(), h.problems());
    assert_eq!(view.problems.get(&key).map(String::as_str), Some(line));
}

#[test]
fn a_failed_verify_offers_no_close_while_its_fix_is_finishing() {
    use crate::model::Stage;
    let mut h = host(vec![], Shared::default());
    let one = ankh_one();
    let two = IssueId::new("ankh", "2");
    let stage = |h: &Host| h.world().issues[&ankh_one()].stage;
    let no_close = |at: &str| {
        format!(
            "the merged change of ankh#1 failed the verify check and is being fixed; it is {at}, \
             a stage where a close in its tracker is not applied"
        )
    };
    let close_finishes = "the merged change of ankh#1 failed the verify check and is being fixed; \
                          closing it in its tracker finishes it";

    // The second issue takes the only folder once the first has merged, so the
    // first waits in Ready after its verify fails.
    feed(&mut h, [synced("1"), planned(&one)]);
    feed(&mut h, [ready(&one, &[("ankh", "57")])]);
    feed(&mut h, approved(&one, 1));
    feed(
        &mut h,
        [
            merged(&one, "ankh"),
            Event::Deployed { issue: one.clone() },
            synced("2"),
            planned(&two),
            failed_verify(&one),
        ],
    );
    assert_eq!(stage(&h), Stage::Ready);
    assert_verify_line(
        &h,
        &one,
        "the merged change of ankh#1 failed the verify check and is being fixed; once the daemon \
         has reopened it in its tracker, closing it there finishes it",
    );

    // Listed as open again, and the folder comes free.
    let close = |key: &str| {
        let mut e = synced(key);
        if let Event::IssueSynced { closed, .. } = &mut e {
            *closed = true;
        }
        e
    };
    feed(&mut h, [synced("1"), close("2")]);
    assert_eq!(stage(&h), Stage::Working);
    assert_verify_line(&h, &one, close_finishes);

    let block = crate::event::Verdict::Block {
        points: vec!["the clock still strikes twelve".into()],
    };
    feed(
        &mut h,
        [ready(&one, &[("ankh", "58")]), reviewed(&one, 1, block)],
    );
    assert_eq!(stage(&h), Stage::Fixing { round: 1 });
    assert!(h.world().issues[&one].merged.is_empty());
    assert_verify_line(&h, &one, close_finishes);

    feed(&mut h, [ready(&one, &[])]);
    feed(&mut h, approved(&one, 2));
    assert_eq!(stage(&h), Stage::Merging);
    assert_verify_line(&h, &one, &no_close("merging"));

    feed(&mut h, [merged(&one, "ankh")]);
    assert_eq!(stage(&h), Stage::Deploying);
    assert_verify_line(&h, &one, &no_close("deploying"));
    feed(&mut h, [close("1")]);
    assert_eq!(stage(&h), Stage::Deploying);
    assert_verify_line(&h, &one, &no_close("deploying"));

    feed(&mut h, [Event::Deployed { issue: one.clone() }]);
    assert_eq!(stage(&h), Stage::Verifying);
    assert_verify_line(&h, &one, &no_close("verifying"));
}

#[test]
fn a_fix_that_merges_before_the_reopen_offers_no_close() {
    let mut h = host(vec![], Shared::default());
    let one = ankh_one();
    feed(&mut h, [synced("1"), planned(&one)]);
    fail_the_verify(&mut h, &one);
    feed(&mut h, [ready(&one, &[("ankh", "58")])]);
    feed(&mut h, approved(&one, 1));
    let issue = &h.world().issues[&one];
    assert_eq!(issue.stage, crate::model::Stage::Merging);
    assert!(issue.reopen_due);
    assert_verify_line(
        &h,
        &one,
        "the merged change of ankh#1 failed the verify check and is being fixed; it is merging, a \
         stage where a close in its tracker is not applied",
    );
}

#[test]
fn a_failed_verify_offers_no_close_once_one_repository_of_the_fix_has_merged() {
    let mut h = Host::new(World::default(), vec![], Box::new(Shared::default()));
    let (ui, api) = (ProjectId::new("ui"), ProjectId::new("api"));
    let issue = IssueId::new("guild", "GLD-12");
    let mut setup = vec![Event::AreaDefined {
        area: ProjectId::new("guild"),
        name: "Thieves' Guild".into(),
        projects: vec![ui.clone(), api.clone()],
        tracker: "fake".into(),
        tracker_name: "GLD".into(),
    }];
    for repo in [&ui, &api] {
        setup.push(Event::ProjectAdded {
            project: repo.clone(),
            tracker: String::new(),
            name: format!("guild/{repo}"),
            code_host: Some("fake".into()),
            repo: None,
        });
        setup.push(Event::FolderAdded {
            folder: FolderId::new(format!("{repo}-1")),
            project: repo.clone(),
            machine: MachineId::new("morpork"),
            path: format!("/srv/{repo}-1"),
        });
    }
    setup.extend([
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![],
            areas: vec![ProjectId::new("guild")],
            milestones: vec![],
            refining_at_once: 1,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
        Event::IssueSynced {
            issue: issue.clone(),
            title: "Mend the ledger".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        },
        Event::PlanWritten {
            issue: issue.clone(),
            plan: Plan {
                repos: vec![ui, api],
                ..Plan::default()
            },
        },
    ]);
    feed(&mut h, setup);
    feed(&mut h, [ready(&issue, &[("ui", "12"), ("api", "34")])]);
    feed(&mut h, approved(&issue, 1));
    feed(
        &mut h,
        [
            merged(&issue, "ui"),
            merged(&issue, "api"),
            Event::Deployed {
                issue: issue.clone(),
            },
            failed_verify(&issue),
        ],
    );
    feed(&mut h, [ready(&issue, &[("ui", "13"), ("api", "35")])]);
    feed(&mut h, approved(&issue, 1));
    feed(
        &mut h,
        [
            merged(&issue, "ui"),
            Event::MergeRefused {
                issue: issue.clone(),
                reason: "pull request 35 conflicts with the branch it merges into".into(),
            },
        ],
    );
    let entry = &h.world().issues[&issue];
    assert_eq!(entry.stage, crate::model::Stage::Fixing { round: 1 });
    assert!(entry.merged.contains(&ProjectId::new("ui")));
    assert_verify_line(
        &h,
        &issue,
        "the merged change of guild#GLD-12 failed the verify check and is being fixed; it is \
         fixing, round 1, a stage where a close in its tracker is not applied",
    );
}

/// Fails every stop, as a runner whose terminal service times out, and
/// reports what its polls hold.
#[derive(Default)]
struct Unstoppable {
    polls: Vec<Polled>,
    stops: Arc<Mutex<Vec<AgentId>>>,
}

impl Part for Unstoppable {
    fn id(&self) -> &'static str {
        "unstoppable"
    }
    fn poll(&mut self, _world: &World, _now_ms: u64) -> Polled {
        if self.polls.is_empty() {
            Polled::default()
        } else {
            self.polls.remove(0)
        }
    }
    fn carry_out(&mut self, effect: &Effect, _world: &World) -> Outcome {
        let Effect::StopAgent { agent } = effect else {
            return Outcome::NotMine;
        };
        self.stops.lock().expect("stops").push(agent.clone());
        Outcome::Failed("the request timed out".into())
    }
}

#[test]
fn stop_problem_goes_when_the_agent_ends() {
    let author_ends = Polled {
        events: vec![Event::AgentChanged {
            agent: AgentId::new("author-1"),
            state: AgentState::Ended,
            lost: false,
        }],
        problem: None,
    };
    let unstoppable = Unstoppable {
        polls: vec![Polled::default(), author_ends],
        stops: Arc::default(),
    };
    let stops = unstoppable.stops.clone();
    let mut h = host(
        vec![
            Box::new(Brains),
            Box::new(Terms::default()),
            Box::new(unstoppable),
        ],
        Shared::default(),
    );
    // refiner-1's stop after its plan fails, and author-1's after a person
    // closes the issue in its tracker.
    feed(&mut h, [synced("1"), planned(&ankh_one())]);
    feed(
        &mut h,
        [Event::IssueSynced {
            issue: ankh_one(),
            title: "Mend clock 1".into(),
            milestone: Some(MilestoneId::new("m1")),
            depends_on: vec![],
            order: 1,
            closed: true,
        }],
    );
    let line = |h: &Host, agent: &str| h.problems().get(&format!("stop {agent}")).cloned();
    let timed_out = Some("the request timed out".to_string());
    assert_eq!(line(&h, "author-1"), timed_out);
    assert_eq!(line(&h, "refiner-1"), timed_out);

    // A tick asks for both stops again; both fail, and both lines stay.
    h.tick(2).expect("tick");
    assert_eq!(line(&h, "author-1"), timed_out);
    assert_eq!(line(&h, "refiner-1"), timed_out);

    // The next poll reports author-1 ended, with no stop of it carried out:
    // its line goes, and refiner-1's, which still runs, stays.
    let tried = stops.lock().expect("stops").len();
    h.tick(3).expect("tick");
    assert_eq!(line(&h, "author-1"), None, "{:?}", h.problems());
    assert_eq!(line(&h, "refiner-1"), timed_out);
    let after: Vec<AgentId> = stops.lock().expect("stops")[tried..].to_vec();
    assert_eq!(
        after,
        [AgentId::new("refiner-1")],
        "author-1 is not asked again"
    );
    let status = crate::text::text(h.world(), h.problems());
    assert!(!status.contains("stop author-1"), "{status}");
    assert!(status.contains("stop refiner-1"), "{status}");
}
