//! Readers run in the daemon's readers' folder: a refiner, reviewer, arbiter or
//! investigator starts while an author holds every work folder of its project,
//! and its launch names the readers' folder, never the author's.

use super::*;
use crate::settings::{Scope, Value};

/// The daemon's readers' folder in these tests.
const READERS: &str = "/srv/temet-work/readers";

/// The readers' folder of the project watch.
fn readers_of_watch() -> Option<std::path::PathBuf> {
    Some(format!("{READERS}/watch").into())
}

/// author-1 working on watch#12 in w1, the only work folder of watch, in a
/// world with a readers' folder.
fn author_1_holds_the_only_folder() -> World {
    let mut w = watch_12_working();
    w.readers = Some(READERS.into());
    assert_eq!(w.folders.len(), 1);
    assert_eq!(w.folders[&FolderId::new("w1")].holder, Some(watch_12()));
    w
}

/// The launch of a `role` on `issue` now, holding no work folder.
fn reader(w: &World, role: Role, issue: &IssueId) -> Launch {
    w.launch(&GroupId::new("watch"), role, issue, None)
        .expect("launch")
}

/// Whether `launch` runs on this machine in the readers' folder of watch, holding
/// no work folder.
fn in_readers_folder(launch: &Launch) {
    assert_eq!(launch.machine, MachineId::new(LOCAL));
    assert_eq!(launch.folder, None);
    assert_eq!(launch.brief.folder, readers_of_watch());
    assert_ne!(launch.brief.folder, Some("/srv/watch/w1".into()));
}

#[test]
fn reviewer_1_on_watch_12_starts_while_author_1_holds_the_only_work_folder() {
    let mut w = author_1_holds_the_only_folder();
    let ready = ready_12(&w, "author-1", Some("57")).expect("ready 57");
    let effects = apply_all(&mut w, &ready);
    assert!(
        effects.contains(&Effect::StartReview {
            issue: watch_12(),
            round: 1,
        }),
        "{effects:?}"
    );
    let launch = reader(&w, Role::Reviewer, &watch_12());
    assert_eq!(launch.agent, AgentId::new("reviewer-1"));
    in_readers_folder(&launch);
    assert_eq!(w.machine_waits(), vec![]);
}

#[test]
fn a_refiner_starts_while_author_1_holds_the_only_work_folder() {
    let mut w = author_1_holds_the_only_folder();
    let watch_13 = IssueId::new("watch", "13");
    let effects = w.apply(&Event::IssueSynced {
        issue: watch_13.clone(),
        title: "A rota for the day shift".into(),
        milestone: None,
        depends_on: vec![],
        order: 2,
        closed: false,
    });
    assert_eq!(
        effects,
        vec![Effect::StartAgent {
            group: GroupId::new("watch"),
            role: Role::Refiner,
            issue: watch_13.clone(),
            folder: None,
        }]
    );
    let launch = reader(&w, Role::Refiner, &watch_13);
    assert_eq!(launch.agent, AgentId::new("refiner-1"));
    in_readers_folder(&launch);
}

#[test]
fn an_arbiter_starts_while_author_1_holds_the_only_work_folder() {
    let mut w = author_1_holds_the_only_folder();
    w.apply(&Event::SettingSet {
        scope: Scope::Task(watch_12()),
        role: None,
        key: "review_rounds".into(),
        value: Value::Number(1),
        by: "person".into(),
    });
    let ready = ready_12(&w, "author-1", Some("57")).expect("ready 57");
    apply_all(&mut w, &ready);
    w.apply(&on_watch_12(started("reviewer-1", Role::Reviewer, "12")));
    let effects = w.apply(&block_12(1));
    assert!(
        effects.contains(&Effect::StartAgent {
            group: GroupId::new("watch"),
            role: Role::Arbiter,
            issue: watch_12(),
            folder: None,
        }),
        "{effects:?}"
    );
    let launch = reader(&w, Role::Arbiter, &watch_12());
    assert_eq!(launch.agent, AgentId::new("arbiter-1"));
    in_readers_folder(&launch);
}

#[test]
fn an_investigator_starts_while_author_1_holds_the_only_work_folder() {
    let mut w = author_1_holds_the_only_folder();
    w.apply(&Event::InvestigationStarted {
        group: GroupId::new("watch"),
        agent: AgentId::new("investigator-1"),
        at_ms: 0,
    });
    let launch = w
        .launch_investigator(&GroupId::new("watch"))
        .expect("launch");
    assert_eq!(launch.agent, AgentId::new("investigator-1"));
    in_readers_folder(&launch);
}

#[test]
fn a_reader_on_another_machine_runs_in_the_readers_folder_there() {
    let mut w = author_1_holds_the_only_folder();
    w.apply(&Event::SettingSet {
        scope: Scope::Task(watch_12()),
        role: Some(Role::Reviewer),
        key: "machine".into(),
        value: Value::Text("uberwald".into()),
        by: "person".into(),
    });
    let ready = ready_12(&w, "author-1", Some("57")).expect("ready 57");
    let effects = apply_all(&mut w, &ready);
    assert!(
        effects.contains(&Effect::StartReview {
            issue: watch_12(),
            round: 1,
        }),
        "no folder of watch is on uberwald, and it starts all the same: {effects:?}"
    );
    let launch = reader(&w, Role::Reviewer, &watch_12());
    assert_eq!(launch.machine, MachineId::new("uberwald"));
    assert_eq!(launch.brief.folder, readers_of_watch());
    assert_eq!(w.machine_waits(), vec![]);
}

#[test]
fn without_a_readers_folder_a_reader_runs_in_no_folder() {
    let w = watch_12_working();
    assert_eq!(w.readers, None);
    let launch = reader(&w, Role::Reviewer, &watch_12());
    assert_eq!(launch.brief.folder, None);
}

#[test]
fn an_agents_folder_is_its_work_folder_or_else_its_readers_folder() {
    let mut w = author_1_holds_the_only_folder();
    w.apply(&Event::AgentStarted {
        agent: AgentId::new("author-2"),
        group: GroupId::new("watch"),
        role: Role::Author,
        issue: Some(watch_12()),
        machine: MachineId::new("morpork"),
        folder: Some(FolderId::new("w1")),
        secret_digest: None,
        runtime: None,
        model: None,
    });
    w.apply(&on_watch_12(started("reviewer-1", Role::Reviewer, "12")));
    w.apply(&Event::InvestigationStarted {
        group: GroupId::new("watch"),
        agent: AgentId::new("investigator-1"),
        at_ms: 0,
    });
    w.apply(&Event::AgentStarted {
        agent: AgentId::new("investigator-1"),
        group: GroupId::new("watch"),
        role: Role::Investigator,
        issue: None,
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    });
    let folder_of = |agent: &str| w.folder_of(&w.agents[&AgentId::new(agent)]);
    assert_eq!(folder_of("author-2"), Some("/srv/watch/w1".into()));
    assert_eq!(folder_of("reviewer-1"), readers_of_watch());
    assert_eq!(folder_of("investigator-1"), readers_of_watch());
}
