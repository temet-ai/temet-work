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
    let (_, entries) = Log::open(&p).expect("reopen");
    assert_eq!(replay(&entries), World::replay(&events()).0);
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
