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
