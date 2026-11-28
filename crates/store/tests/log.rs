//! The event log on disk, in cargo's own folder for test files.

use std::io::Write;
use std::path::PathBuf;

use temet_work_core::ids::{FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId};
use temet_work_core::model::Plan;
use temet_work_core::{Event, World};
use temet_work_store::{Log, StoreError, replay};

/// A fresh log path for one test, under the target folder cargo gives tests.
fn path(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("store-tests");
    std::fs::create_dir_all(&dir).expect("test folder");
    let p = dir.join(format!("{name}.jsonl"));
    let _ = std::fs::remove_file(&p);
    p
}

/// The world `events` make when each is applied at `at_ms`, as the daemon
/// applies them as they come.
fn applied(at_ms: u64, events: &[Event]) -> World {
    World::default()
        .replay_at(events.iter().map(|e| (at_ms, e)))
        .0
}

fn events() -> Vec<Event> {
    vec![
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![ProjectId::new("ankh")],
            areas: Vec::new(),
            milestones: vec![MilestoneId::new("m1")],
            refining_at_once: 1,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
        Event::FolderAdded {
            folder: FolderId::new("f1"),
            project: ProjectId::new("ankh"),
            machine: MachineId::new("morpork"),
            path: "/srv/ankh/f1".into(),
        },
        Event::IssueSynced {
            issue: IssueId::new("ankh", "1"),
            title: "Mend the clock".into(),
            milestone: Some(MilestoneId::new("m1")),
            depends_on: vec![],
            order: 1,
            closed: false,
        },
        Event::PlanWritten {
            issue: IssueId::new("ankh", "1"),
            plan: Plan::default(),
        },
    ]
}

#[test]
fn appends_and_reads_back_in_order_with_numbers_from_one() {
    let p = path("round-trip");
    let (mut log, existing) = Log::open(&p).expect("open");
    assert!(existing.is_empty());
    for (i, e) in events().iter().enumerate() {
        let entry = log.append(1_000 + i as u64, e).expect("append");
        assert_eq!(entry.seq, i as u64 + 1);
    }
    drop(log);
    let (_, entries) = Log::open(&p).expect("reopen");
    assert_eq!(
        entries.iter().map(|e| e.seq).collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(
        entries.iter().map(|e| e.event.clone()).collect::<Vec<_>>(),
        events()
    );
}

#[test]
fn replaying_the_log_gives_the_same_state_and_change_times_as_applying_the_events() {
    let p = path("replay");
    let (mut log, _) = Log::open(&p).expect("open");
    let mut live = World::default();
    // A minute apart, as the daemon stamps them when it applies them.
    for (n, e) in (0..).zip(events()) {
        let at_ms = 1_795_000_000_000 + n * 60_000;
        log.append(at_ms, &e).expect("append");
        live.apply_at(at_ms, &e);
    }
    drop(log);
    let (_, entries) = Log::open(&p).expect("reopen");
    let replayed = replay(&entries);
    assert_eq!(replayed, live);
    // The plan, the fourth event, was the last change to ankh#1.
    let issue = IssueId::new("ankh", "1");
    assert_eq!(replayed.updated.at(&issue), Some(1_795_000_180_000));
    assert_eq!(replayed.updated.at(&issue), live.updated.at(&issue));
}

#[test]
fn replaying_the_log_gives_the_same_rules_section_after_a_restart() {
    use temet_work_core::ids::{AgentId, DecisionId, RuleId};
    use temet_work_core::model::{Decision, DecisionKind, Rule, RuleState};
    let rule = |state| Rule {
        id: RuleId(1),
        text: "Which connection? Prefer: second. Why: legacy breaks channels".into(),
        decision: DecisionId(1),
        issue: IssueId::new("ankh", "1"),
        decided_by: AgentId::new("refiner-1"),
        overturned_by: "person".into(),
        state,
    };
    let mut all = events();
    all.extend([
        Event::DecisionMade {
            decision: Decision {
                id: DecisionId(1),
                issue: IssueId::new("ankh", "1"),
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
        Event::DecisionOverturned {
            decision: DecisionId(1),
            to: 0,
            reason: "legacy breaks channels".into(),
            by: "person".into(),
        },
        Event::RuleProposed {
            rule: rule(RuleState::Proposed),
        },
        Event::RuleAnswered {
            rule: RuleId(1),
            keep: true,
        },
    ]);
    let p = path("rules");
    let (mut log, _) = Log::open(&p).expect("open");
    for e in &all {
        log.append(0, e).expect("append");
    }
    drop(log);
    let (_, entries) = Log::open(&p).expect("reopen");
    let replayed = replay(&entries);
    assert_eq!(replayed.rules[&RuleId(1)].state, RuleState::Kept);
    let none = Default::default();
    let live = temet_work_core::text::text(&World::replay(&all).0, &none);
    let after = temet_work_core::text::text(&replayed, &none);
    assert_eq!(live, after);
    assert!(
        after.contains("1 kept: Which connection? Prefer: second."),
        "{after}"
    );
}

#[test]
fn numbering_carries_on_after_reopening() {
    let p = path("carry-on");
    let (mut log, _) = Log::open(&p).expect("open");
    log.append(0, &events()[0]).expect("append");
    drop(log);
    let (mut log, _) = Log::open(&p).expect("reopen");
    assert_eq!(log.append(0, &events()[1]).expect("append").seq, 2);
}

#[test]
fn a_half_written_last_line_is_left_out() {
    let p = path("crash");
    let (mut log, _) = Log::open(&p).expect("open");
    log.append(0, &events()[0]).expect("append");
    drop(log);
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&p)
        .expect("file");
    write!(f, "{{\"seq\":2,\"at_ms\":0,\"event\":{{\"Folder").expect("write");
    drop(f);
    let (mut log, entries) = Log::open(&p).expect("reopen after a crash");
    assert_eq!(entries.len(), 1);
    // The next entry is numbered after the last complete one.
    assert_eq!(log.append(0, &events()[1]).expect("append").seq, 2);
    drop(log);
    let (_, entries) = Log::open(&p).expect("reopen after appending past the crash");
    assert_eq!(entries.iter().map(|e| e.seq).collect::<Vec<_>>(), [1, 2]);
}

#[test]
fn a_damaged_line_in_the_middle_is_an_error() {
    let p = path("damaged");
    std::fs::write(&p, "not json\n{\"seq\":1}\n").expect("write");
    assert!(matches!(
        Log::read(&p),
        Err(StoreError::Damaged { line: 1, .. })
    ));
}

#[test]
fn a_second_writer_is_refused_while_the_first_has_the_log() {
    let p = path("one-writer");
    let (first, _) = Log::open(&p).expect("open");
    assert!(matches!(Log::open(&p), Err(StoreError::Busy(_))));
    drop(first);
    assert!(Log::open(&p).is_ok(), "free again once the first closes it");
}

#[test]
fn an_old_pull_request_event_replays_as_the_first_repositorys_pull_request() {
    let p = path("old-pull");
    let lines = [
        r#"{"seq":1,"at_ms":0,"event":{"ProjectAdded":{"project":"ankh","tracker":"fake","name":"city/ankh"}}}"#,
        r#"{"seq":2,"at_ms":0,"event":{"IssueSynced":{"issue":{"project":"ankh","key":"1"},"title":"t","milestone":null,"depends_on":[],"order":1,"closed":false}}}"#,
        r#"{"seq":3,"at_ms":0,"event":{"PullRequestReady":{"issue":{"project":"ankh","key":"1"},"pull":"57"}}}"#,
        r#"{"seq":4,"at_ms":0,"event":{"Merged":{"issue":{"project":"ankh","key":"1"}}}}"#,
    ];
    std::fs::write(&p, lines.join("\n") + "\n").expect("write");
    let entries = Log::read(&p).expect("old entries read");
    let world = replay(&entries[..3]);
    let issue = &world.issues[&IssueId::new("ankh", "1")];
    assert_eq!(
        issue.pulls.get(&ProjectId::new("ankh")).map(String::as_str),
        Some("57")
    );
    assert_eq!(replay(&entries).issues.len(), 1, "an old Merged reads too");
}

#[test]
fn pull_requests_by_repository_survive_the_log() {
    let p = path("pulls-by-repo");
    let (mut log, _) = Log::open(&p).expect("open");
    let event = Event::PullRequestReady {
        issue: IssueId::new("guild", "GLD-1"),
        pull: None,
        pulls: [(ProjectId::new("ui"), "12".to_string())].into(),
        replies: Vec::new(),
    };
    log.append(0, &event).expect("append");
    drop(log);
    let entries = Log::read(&p).expect("read");
    assert_eq!(entries[0].event, event);
}

#[test]
fn an_agent_started_before_secrets_reads_with_no_digest_runtime_or_model_and_each_survives_the_log()
{
    use temet_work_core::ids::AgentId;
    use temet_work_core::sender::Digest;
    let p = path("agent-digest");
    // As a daemon wrote it before agents had secrets, runtimes or models on record.
    let old = r#"{"seq":1,"at_ms":0,"event":{"AgentStarted":{"agent":"author-1","group":"watch","role":"Author","issue":{"project":"ankh","key":"1"},"machine":"local","folder":"ankh-1"}}}"#;
    std::fs::write(&p, format!("{old}\n")).expect("write");
    let (mut log, entries) = Log::open(&p).expect("an old log opens");
    let author = &replay(&entries).agents[&AgentId::new("author-1")];
    assert_eq!(
        (&author.secret_digest, &author.runtime, &author.model),
        (&None, &None, &None)
    );
    let digest = Digest("ab".repeat(32));
    let event = Event::AgentStarted {
        agent: AgentId::new("reviewer-1"),
        group: GroupId::new("watch"),
        role: temet_work_core::model::Role::Reviewer,
        issue: Some(IssueId::new("ankh", "1")),
        machine: MachineId::new("local"),
        folder: None,
        secret_digest: Some(digest.clone()),
        runtime: Some("hex".into()),
        model: Some("anthill".into()),
    };
    log.append(1, &event).expect("append");
    drop(log);
    let entries = Log::read(&p).expect("read");
    assert_eq!(entries[1].event, event);
    let reviewer = &replay(&entries).agents[&AgentId::new("reviewer-1")];
    assert_eq!(reviewer.secret_digest, Some(digest));
    assert_eq!(
        (reviewer.runtime.as_deref(), reviewer.model.as_deref()),
        (Some("hex"), Some("anthill"))
    );
}

#[test]
fn replaying_the_log_keeps_whether_an_ended_agent_had_lost_its_program() {
    use temet_work_core::ids::AgentId;
    use temet_work_core::model::{AgentState, Role, Stage};
    let p = path("agent-lost");
    let author = AgentId::new("author-1");
    let mut events = events();
    events.push(Event::AgentStarted {
        agent: author.clone(),
        group: GroupId::new("watch"),
        role: Role::Author,
        issue: Some(IssueId::new("ankh", "1")),
        machine: MachineId::new("morpork"),
        folder: Some(FolderId::new("f1")),
        secret_digest: None,
        runtime: None,
        model: None,
    });
    let working = applied(0, &events);
    assert_eq!(
        working.issues[&IssueId::new("ankh", "1")].stage,
        Stage::Working
    );
    assert!(working.waiting_on(&working.agents[&author]));

    for lost in [true, false] {
        let end = Event::AgentChanged {
            agent: author.clone(),
            state: AgentState::Ended,
            lost,
        };
        let _ = std::fs::remove_file(&p);
        let (mut log, _) = Log::open(&p).expect("open");
        for e in events.iter().chain([&end]) {
            log.append(0, e).expect("append");
        }
        drop(log);
        // An end by the program itself is written as it was before the flag, so
        // a line from an older log reads as one.
        let text = std::fs::read_to_string(&p).expect("the log");
        let line = text.lines().last().expect("a line");
        assert_eq!(line.contains("\"lost\""), lost, "{line}");
        let entries = Log::read(&p).expect("read");
        assert_eq!(entries.last().expect("the end").event, end);
        let agent = &replay(&entries).agents[&author];
        assert_eq!(agent.state, AgentState::Ended);
        assert_eq!(agent.ended_waited_on, !lost, "lost {lost}");
        assert_eq!(agent.lost_waited_on, lost, "lost {lost}");
    }
}

#[test]
fn replaying_the_log_keeps_an_agents_limit_and_the_reset_it_was_continued_after() {
    use temet_work_core::ids::AgentId;
    use temet_work_core::model::{AgentState, Role};
    let p = path("agent-limit");
    let author = AgentId::new("author-1");
    let limited = Event::AgentChanged {
        agent: author.clone(),
        state: AgentState::Errored {
            line: "You've hit your 5-hour limit · resets 5:40pm".into(),
            resets_ms: Some(1_796_060_400_000),
        },
        lost: false,
    };
    let mut events = events();
    events.extend([
        Event::AgentStarted {
            agent: author.clone(),
            group: GroupId::new("watch"),
            role: Role::Author,
            issue: Some(IssueId::new("ankh", "1")),
            machine: MachineId::new("morpork"),
            folder: Some(FolderId::new("f1")),
            secret_digest: None,
            runtime: None,
            model: None,
        },
        limited.clone(),
    ]);
    let (mut log, _) = Log::open(&p).expect("open");
    for e in &events {
        log.append(0, e).expect("append");
    }
    drop(log);
    // A daemon restarted while the agent waits on its limit still knows when it resets.
    let (mut log, entries) = Log::open(&p).expect("reopen");
    let world = replay(&entries);
    assert_eq!(world, applied(0, &events));
    assert_eq!(world.agents[&author].limit_reset(), Some(1_796_060_400_000));

    // Continued, then the same limit again: it is not waited on a second time.
    let more = [
        Event::Continued {
            agent: author.clone(),
            resets_ms: 1_796_060_400_000,
        },
        limited,
    ];
    for e in &more {
        log.append(0, e).expect("append");
    }
    events.extend(more);
    drop(log);
    let world = replay(&Log::read(&p).expect("read"));
    assert_eq!(world, applied(0, &events));
    assert_eq!(world.agents[&author].continued, Some(1_796_060_400_000));
    assert_eq!(world.agents[&author].limit_reset(), None);
}

#[test]
fn replaying_an_investigation_gives_the_same_state_as_applying_its_events() {
    use temet_work_core::ids::AgentId;
    let watch = GroupId::new("watch");
    let me = AgentId::new("investigator-1");
    let mut all = events();
    all.extend([
        Event::InvestigationStarted {
            group: watch.clone(),
            agent: me.clone(),
            at_ms: 1_795_784_700_000,
        },
        Event::IssueSynced {
            issue: IssueId::new("ankh", "2"),
            title: "Wind the clock".into(),
            milestone: Some(MilestoneId::new("m1")),
            depends_on: vec![IssueId::new("ankh", "1")],
            order: 2,
            closed: false,
        },
        Event::IssueFiled {
            group: watch.clone(),
            agent: me,
            issue: IssueId::new("ankh", "2"),
            title: "Wind the clock".into(),
            milestone: Some(MilestoneId::new("m1")),
        },
        Event::InvestigationEnded {
            group: watch.clone(),
            at_ms: 1_795_784_760_000,
            filed: 1,
        },
    ]);
    let p = path("investigation");
    let (mut log, _) = Log::open(&p).expect("open");
    for e in &all {
        log.append(0, e).expect("append");
    }
    drop(log);
    let (_, entries) = Log::open(&p).expect("reopen");
    let replayed = replay(&entries);
    assert_eq!(replayed, applied(0, &all));
    let investigation = &replayed.investigations[&watch];
    assert_eq!(
        (
            investigation.started_ms,
            investigation.ended_ms,
            investigation.filed.as_slice()
        ),
        (
            1_795_784_700_000,
            Some(1_795_784_760_000),
            [IssueId::new("ankh", "2")].as_slice()
        )
    );
}

#[test]
fn an_older_logs_group_replays_with_the_default_queue_low_and_investigate_every() {
    let p = path("older-group");
    let line = r#"{"seq":1,"at_ms":0,"event":{"GroupCreated":{"group":"watch","name":"Night watch","projects":["ankh"],"milestones":[],"refining_at_once":1}}}"#;
    std::fs::write(&p, format!("{line}\n")).expect("write");
    let (_, entries) = Log::open(&p).expect("open");
    let world = replay(&entries);
    let group = &world.groups[&GroupId::new("watch")];
    assert_eq!(group.queue_low, None, "twice its work folders");
    assert_eq!(world.queue_low(group), 2, "it has none");
    assert_eq!(group.investigate_every, 120);
    // And a group written now leaves the defaults out, so older daemons read it.
    let (mut log, _) = Log::open(&path("new-group")).expect("open");
    let entry = log.append(0, &events()[0]).expect("append");
    let text = serde_json::to_string(&entry).expect("json");
    assert!(!text.contains("queue_low"), "{text}");
}

#[test]
fn an_older_log_replays_to_the_last_finished_investigations_window() {
    /// The brief's window line after replaying `events`, each one raw line.
    fn replayed(name: &str, events: &[&str]) -> (Option<u64>, String) {
        let p = path(name);
        let lines: String = events
            .iter()
            .enumerate()
            .map(|(i, e)| format!("{{\"seq\":{},\"at_ms\":0,\"event\":{e}}}\n", i + 1))
            .collect();
        std::fs::write(&p, lines).expect("write");
        let (_, entries) = Log::open(&p).expect("open");
        let world = replay(&entries);
        let watch = GroupId::new("watch");
        let launch = world.launch_investigator(&watch).expect("running");
        let line = launch.brief.task.lines().find(|l| l.starts_with("3. "));
        (
            world.investigations[&watch].since_ms,
            line.expect("line 3").to_string(),
        )
    }
    let group = r#"{"GroupCreated":{"group":"watch","name":"Night watch","projects":["ankh"],"milestones":[],"refining_at_once":1}}"#;

    let (since_ms, line) = replayed(
        "older-investigations",
        &[
            group,
            r#"{"InvestigationStarted":{"group":"watch","agent":"investigator-1","at_ms":0}}"#,
            r#"{"Investigated":{"group":"watch","agent":"investigator-1"}}"#,
            r#"{"InvestigationEnded":{"group":"watch","at_ms":600000,"filed":0}}"#,
            r#"{"InvestigationStarted":{"group":"watch","agent":"investigator-2","at_ms":7800000}}"#,
            r#"{"InvestigationEnded":{"group":"watch","at_ms":8400000,"filed":0}}"#,
            r#"{"InvestigationStarted":{"group":"watch","agent":"investigator-3","at_ms":15600000}}"#,
        ],
    );
    assert_eq!(since_ms, Some(0));
    assert!(
        line.contains(
            "since 1970-01-01 00:00 UTC, when the group's last finished investigation started"
        ),
        "{line}"
    );

    // 2026-11-27 13:05 UTC, and the group's first investigation ends unfinished.
    let (since_ms, line) = replayed(
        "older-unfinished-first",
        &[
            group,
            r#"{"InvestigationStarted":{"group":"watch","agent":"investigator-1","at_ms":1795784700000}}"#,
            r#"{"InvestigationEnded":{"group":"watch","at_ms":1795785300000,"filed":0}}"#,
            r#"{"InvestigationStarted":{"group":"watch","agent":"investigator-2","at_ms":1795792500000}}"#,
        ],
    );
    assert_eq!(since_ms, None);
    assert!(
        line.contains(
            "since 2026-11-20 13:05 UTC, 7 days before the group's first investigation started, \
             as none has finished yet"
        ),
        "{line}"
    );
}

#[test]
fn replaying_the_log_gives_the_same_check_states_replies_and_point_states_as_applying_them() {
    use temet_work_core::Command;
    use temet_work_core::Verdict;
    use temet_work_core::ids::AgentId;
    use temet_work_core::model::{Check, CheckState, PointState, Role};

    let issue = IssueId::new("ankh", "1");
    let started = |agent: &str, role| Event::AgentStarted {
        agent: AgentId::new(agent),
        group: GroupId::new("watch"),
        role,
        issue: Some(issue.clone()),
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    };
    let mut setup = events();
    // The issue is being refined; its plan comes from the refiner's command.
    setup.pop();
    setup.push(started("refiner-1", Role::Refiner));
    let ready = |replies: &[&str]| Command::Ready {
        issue: issue.clone(),
        pull: Some("57".into()),
        pulls: Default::default(),
        replies: replies.iter().map(ToString::to_string).collect(),
    };
    let block = |points: &[&str], failed: Vec<usize>, open: Vec<usize>| Command::Review {
        issue: issue.clone(),
        verdict: Verdict::Block {
            points: points.iter().map(ToString::to_string).collect(),
        },
        failed,
        open,
    };
    let steps = [
        (
            "refiner-1",
            Command::Plan {
                issue: issue.clone(),
                plan: Plan {
                    checks: vec![
                        Check::new("the clock strikes thirteen", "listening at midnight"),
                        Check::new("the hands point up", "looking at the face"),
                    ],
                    ..Plan::default()
                },
            },
        ),
        ("author-1", ready(&[])),
        (
            "reviewer-1",
            block(
                &["the spring is weak", "the face is dusty"],
                vec![2],
                vec![],
            ),
        ),
        ("author-1", ready(&["a new spring", "dusted it"])),
        ("reviewer-1", block(&[], vec![1], vec![2])),
    ];

    // The lines are written as the log writes them, in one write: syncing each
    // of them would take most of a second.
    let mut written = Vec::new();
    let mut live = World::default();
    let mut record = |live: &mut World, e: &Event| {
        let entry = temet_work_store::Entry {
            seq: written.len() as u64 + 1,
            at_ms: 1_795_000_000_000 + 60_000 * written.len() as u64,
            event: e.clone(),
        };
        live.apply_at(entry.at_ms, e);
        written.push(serde_json::to_string(&entry).expect("a line"));
    };
    for e in &setup {
        record(&mut live, e);
    }
    for (by, command) in steps {
        if by == "refiner-1" {
            record(&mut live, &started("author-1", Role::Author));
            record(&mut live, &started("reviewer-1", Role::Reviewer));
        }
        if matches!(command, Command::Review { .. }) {
            record(
                &mut live,
                &Event::Told {
                    issue: issue.clone(),
                },
            );
        }
        let made = live
            .events(Some(&AgentId::new(by)), &command)
            .unwrap_or_else(|e| panic!("{by} is refused: {e}"));
        for e in &made {
            record(&mut live, e);
        }
    }
    let p = path("states");
    std::fs::write(&p, written.join("\n") + "\n").expect("write");
    let (_, entries) = Log::open(&p).expect("open");
    let replayed = replay(&entries);
    assert_eq!(replayed, live);
    // What the loop learned is all there after the restart.
    let i = &replayed.issues[&issue];
    let states: Vec<_> = i
        .plan
        .iter()
        .flat_map(|p| &p.checks)
        .map(|c| c.state)
        .collect();
    assert_eq!(states, [Some(CheckState::Failed), Some(CheckState::Passed)]);
    assert_eq!(i.reviews[0].replies, ["a new spring", "dusted it"]);
    assert_eq!(
        i.reviews[0].states,
        [PointState::Resolved, PointState::Open]
    );
    assert_eq!(i.reviews[1].points, ["the face is dusty"]);
    assert_eq!(i.reviews[1].states, [PointState::Open]);
}

#[test]
fn a_persons_refine_survives_the_log_and_replays_to_the_same_state() {
    use temet_work_core::model::{Check, Stage};
    let p = path("refined");
    let (mut log, _) = Log::open(&p).expect("open");
    let mut events = events();
    // Sent back once its plan is in, with a new title and a check of the person's.
    events.push(Event::Refined {
        issue: IssueId::new("ankh", "1"),
        title: Some("Mend the clock before dusk".into()),
        checks: Some(vec![
            Check::new("the clock strikes six at dusk", "listening at dusk"),
            Check::new("Sam Vimes signs the log", ""),
        ]),
        send: true,
    });
    // A save that removes the person's checks.
    events.push(Event::Refined {
        issue: IssueId::new("ankh", "1"),
        title: None,
        checks: Some(vec![]),
        send: false,
    });
    for e in &events {
        log.append(5, e).expect("append");
    }
    drop(log);
    let entries = Log::read(&p).expect("read");
    let read: Vec<Event> = entries.iter().map(|e| e.event.clone()).collect();
    assert_eq!(read, events, "each event reads back as written");
    let replayed = replay(&entries);
    assert_eq!(replayed, applied(5, &events));
    let i = &replayed.issues[&IssueId::new("ankh", "1")];
    assert_eq!(i.title, "Mend the clock before dusk");
    assert_eq!(i.stage, Stage::Refining);
    assert!(i.person_checks.is_empty());
    // A save writes no `send`, nor a title or checks it does not change.
    let line = std::fs::read_to_string(&p).expect("log");
    let last = line.lines().last().expect("a line");
    assert!(!last.contains("send") && !last.contains("title"), "{last}");
}

#[test]
fn a_persons_assign_and_unassign_and_a_moved_agents_brief_survive_the_log_and_replay_the_same() {
    let p = path("assign");
    let (mut log, _) = Log::open(&p).expect("open");
    let issue = |key: &str| IssueId::new("ankh", key);
    let agent = |id: &str| temet_work_core::ids::AgentId::new(id);
    let reader = |id: &str, role, key: &str| Event::AgentStarted {
        agent: agent(id),
        group: GroupId::new("watch"),
        role,
        issue: Some(issue(key)),
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    };
    let mut events = events();
    events.extend([
        Event::IssueSynced {
            issue: issue("2"),
            title: "Ring the bell".into(),
            milestone: Some(MilestoneId::new("m1")),
            depends_on: vec![],
            order: 2,
            closed: false,
        },
        reader("refiner-1", temet_work_core::model::Role::Refiner, "2"),
        reader("refiner-2", temet_work_core::model::Role::Refiner, "3"),
    ]);
    let person = [
        Event::Assigned {
            issue: issue("2"),
            agent: agent("refiner-2"),
            from: issue("3"),
        },
        Event::Briefed {
            agent: agent("refiner-2"),
            issue: issue("2"),
        },
        Event::Unassigned {
            issue: issue("2"),
            agent: agent("refiner-2"),
        },
    ];
    events.extend(person.iter().cloned());
    let mut live = World::default();
    for e in &events {
        log.append(0, e).expect("append");
        live.apply_at(0, e);
    }
    drop(log);
    let (_, entries) = Log::open(&p).expect("reopen");
    let read: Vec<&Event> = entries.iter().map(|e| &e.event).collect();
    assert_eq!(
        read[read.len() - 3..],
        person.iter().collect::<Vec<_>>()[..]
    );
    assert_eq!(replay(&entries), live);
}
