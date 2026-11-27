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
fn replaying_the_log_gives_the_same_state_as_applying_the_events() {
    let p = path("replay");
    let (mut log, _) = Log::open(&p).expect("open");
    for e in events() {
        log.append(0, &e).expect("append");
    }
    drop(log);
    let (_, entries) = Log::open(&p).expect("reopen");
    assert_eq!(replay(&entries), World::replay(&events()).0);
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
    };
    log.append(0, &event).expect("append");
    drop(log);
    let entries = Log::read(&p).expect("read");
    assert_eq!(entries[0].event, event);
}

#[test]
fn an_agent_started_before_secrets_reads_with_no_digest_and_a_digest_survives_the_log() {
    use temet_work_core::ids::AgentId;
    use temet_work_core::sender::Digest;
    let p = path("agent-digest");
    // As a daemon wrote it before agents had secrets.
    let old = r#"{"seq":1,"at_ms":0,"event":{"AgentStarted":{"agent":"author-1","group":"watch","role":"Author","issue":{"project":"ankh","key":"1"},"machine":"local","folder":"ankh-1"}}}"#;
    std::fs::write(&p, format!("{old}\n")).expect("write");
    let (mut log, entries) = Log::open(&p).expect("an old log opens");
    assert_eq!(
        replay(&entries).agents[&AgentId::new("author-1")].secret_digest,
        None
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
    };
    log.append(1, &event).expect("append");
    drop(log);
    let entries = Log::read(&p).expect("read");
    assert_eq!(entries[1].event, event);
    assert_eq!(
        replay(&entries).agents[&AgentId::new("reviewer-1")].secret_digest,
        Some(digest)
    );
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
    assert_eq!(world, World::replay(&events).0);
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
        log.append(1, e).expect("append");
    }
    events.extend(more);
    drop(log);
    let world = replay(&Log::read(&p).expect("read"));
    assert_eq!(world, World::replay(&events).0);
    assert_eq!(world.agents[&author].continued, Some(1_796_060_400_000));
    assert_eq!(world.agents[&author].limit_reset(), None);
}
