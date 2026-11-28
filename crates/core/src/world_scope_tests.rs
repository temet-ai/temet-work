//! An issue that leaves its group's scope: nothing new starts on it until it comes
//! back, and then it resumes at the stage it reached. Work past the agents finishes.

use std::collections::BTreeMap;

use super::*;

/// Issue `key` synced again, now in `milestone`.
fn moved(key: &str, milestone: &str) -> Event {
    sync(key, milestone, key.parse().unwrap_or_default(), &[])
}

/// The night watch taking `projects` and `areas` in `milestones`.
fn watch(projects: &[&str], areas: &[&str], milestones: &[&str]) -> Event {
    Event::GroupCreated {
        group: GroupId::new("watch"),
        name: "Night watch".into(),
        projects: projects.iter().map(|p| pid(p)).collect(),
        areas: areas.iter().map(|a| pid(a)).collect(),
        milestones: milestones.iter().map(|m| MilestoneId::new(*m)).collect(),
        refining_at_once: 1,
        model: None,
        queue_low: None,
        investigate_every: None,
    }
}

/// `key` pinned by a person to `machine`.
fn pinned(key: &str, machine: &str) -> Event {
    Event::SettingSet {
        scope: Scope::Task(issue(key)),
        role: None,
        key: "machine".into(),
        value: Value::Text(machine.into()),
        by: "person".into(),
    }
}

fn holder(w: &World, folder: &str) -> Option<IssueId> {
    w.folders[&FolderId::new(folder)].holder.clone()
}

fn block(round: u8) -> Event {
    verdict(
        round,
        Verdict::Block {
            points: vec!["no test for a stopped clock".into()],
        },
    )
}

fn pull_ready() -> Event {
    Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
        replies: Vec::new(),
    }
}

#[test]
fn a_ready_issue_moved_out_of_the_groups_milestones_gets_no_author_until_it_moves_back() {
    let mut w = World::default();
    w.apply(&group(&["m1"], 1));
    w.apply(&sync("1", "m1", 1, &[]));
    w.apply(&planned("1"));
    assert_eq!(stage(&w, "1"), Stage::Ready, "no folder yet");
    assert_eq!(w.apply(&moved("1", "m9")), vec![]);
    assert_eq!(w.apply(&folder("f1")), vec![], "no author");
    assert_eq!(holder(&w, "f1"), None);
    assert_eq!(w.missing(), Missing::default());
    assert_eq!(
        w.apply(&moved("1", "m1")),
        vec![author("1", "f1")],
        "an author, and no refiner"
    );
    assert_eq!(stage(&w, "1"), Stage::Working);
    assert_eq!(holder(&w, "f1"), Some(issue("1")));

    // A pinned issue out of scope keeps no folder free for itself: issue 1 takes
    // the first free one, which issue 2 would want.
    let mut w = World::default();
    w.apply(&folder_on("f1", "ankh", "uberwald"));
    w.apply(&folder_on("f2", "ankh", "morpork"));
    w.apply(&group(&["m1"], 2));
    w.apply(&sync("1", "m1", 1, &[]));
    w.apply(&sync("2", "m1", 2, &[]));
    w.apply(&pinned("2", "uberwald"));
    w.apply(&started("refiner-2", Role::Refiner, "2"));
    w.apply(&moved("2", "m9"));
    assert_eq!(w.apply(&planned("2")).len(), 1, "only its refiner stops");
    assert_eq!(w.apply(&planned("1")), vec![author("1", "f1")]);
}

#[test]
fn a_group_narrowed_to_leave_out_a_ready_issue_starts_no_author_on_it() {
    let area = Event::AreaDefined {
        area: pid("guild"),
        name: "Thieves' Guild".into(),
        projects: vec![pid("ankh")],
        tracker: String::new(),
        tracker_name: String::new(),
    };
    for (wide, narrow) in [
        (
            watch(&["ankh"], &[], &["m1"]),
            watch(&["ankh"], &[], &["m2"]),
        ),
        (
            watch(&["ankh"], &[], &["m1"]),
            watch(&["klatch"], &[], &["m1"]),
        ),
        (
            watch(&[], &["guild"], &["m1"]),
            watch(&[], &["dolly"], &["m1"]),
        ),
    ] {
        let (mut w, _) =
            World::replay(&[area.clone(), wide, sync("1", "m1", 1, &[]), planned("1")]);
        assert_eq!(stage(&w, "1"), Stage::Ready);
        assert_eq!(w.apply(&narrow), vec![], "{narrow:?}");
        assert_eq!(w.apply(&folder("f1")), vec![], "{narrow:?}");
        assert_eq!(holder(&w, "f1"), None);
        assert!(w.out_of_scope(&w.issues[&issue("1")]));
    }
}

#[test]
fn a_refining_issue_moved_out_finishes_its_plan_and_then_waits() {
    let mut w = World::default();
    w.apply(&group(&["m1"], 1));
    w.apply(&folder("f1"));
    w.apply(&sync("1", "m1", 1, &[]));
    w.apply(&started("refiner-1", Role::Refiner, "1"));
    assert_eq!(w.apply(&moved("1", "m9")), vec![]);
    assert_eq!(stage(&w, "1"), Stage::Refining, "its refiner keeps going");
    assert!(waited_on(&w, "refiner-1"));
    assert_eq!(
        w.apply(&planned("1")),
        vec![Effect::StopAgent {
            agent: AgentId::new("refiner-1")
        }],
        "no author"
    );
    assert_eq!(stage(&w, "1"), Stage::Ready);
    assert_eq!(holder(&w, "f1"), None);
    assert_eq!(w.missing(), Missing::default());
}

#[test]
fn an_out_of_scope_refining_issue_frees_its_slot_once_its_refiner_stops() {
    let mut w = World::default();
    w.apply(&group(&["m1"], 1));
    w.apply(&sync("1", "m1", 1, &[]));
    w.apply(&started("refiner-1", Role::Refiner, "1"));
    w.apply(&moved("1", "m9"));
    assert_eq!(
        w.apply(&sync("2", "m1", 2, &[])),
        vec![],
        "issue 1's refiner still runs, so it holds the slot"
    );
    assert_eq!(w.apply(&ended("refiner-1")), vec![refiner("2")]);
    assert_eq!(stage(&w, "1"), Stage::New, "back in the pool, with no plan");
    assert_eq!(w.issues[&issue("1")].group, None);
    assert!(!w.missing().start.contains(&refiner("1")), "no new refiner");
    // Back in scope, it waits for a free slot like any new issue.
    assert_eq!(w.apply(&moved("1", "m1")), vec![]);
    w.apply(&started("refiner-2", Role::Refiner, "2"));
    assert_eq!(
        w.apply(&planned("2")),
        vec![
            Effect::StopAgent {
                agent: AgentId::new("refiner-2")
            },
            refiner("1")
        ]
    );
}

#[test]
fn an_issue_in_review_moved_out_starts_nothing_until_it_moves_back() {
    // No reviewer starts on the pull request its author opens.
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    w.apply(&moved("1", "m9"));
    assert!(waited_on(&w, "author-1"), "it finishes its step");
    assert_eq!(w.apply(&pull_ready()), vec![], "no reviewer");
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 1 });
    assert_eq!(w.missing(), Missing::default());
    assert_eq!(holder(&w, "f1"), Some(issue("1")), "it keeps its folder");
    w.apply(&moved("1", "m1"));
    assert_eq!(
        w.missing().start,
        vec![Effect::StartReview {
            issue: issue("1"),
            round: 1
        }]
    );

    // A running reviewer is not told of a new round.
    let mut w = in_review();
    w.apply(&block(1));
    w.apply(&Event::Told { issue: issue("1") });
    w.apply(&moved("1", "m9"));
    assert_eq!(w.apply(&pull_ready()), vec![]);
    assert!(!waited_on(&w, "reviewer-1"));
    assert_eq!(w.missing(), Missing::default());
    w.apply(&moved("1", "m1"));
    assert_eq!(
        w.missing().start,
        vec![Effect::StartReview {
            issue: issue("1"),
            round: 2
        }]
    );

    // A reviewer that has heard its round finishes it; no arbiter starts after.
    let mut w = in_review();
    for round in 1..3 {
        w.apply(&block(round));
        w.apply(&pull_ready());
    }
    w.apply(&Event::Told { issue: issue("1") });
    w.apply(&moved("1", "m9"));
    assert!(waited_on(&w, "reviewer-1"), "it finishes its review");
    assert_eq!(w.apply(&block(3)), vec![], "no arbiter");
    assert_eq!(stage(&w, "1"), Stage::Arbitration { round: 3 });
    assert_eq!(w.missing(), Missing::default());
    w.apply(&moved("1", "m1"));
    assert_eq!(
        w.missing().start,
        vec![Effect::StartAgent {
            group: GroupId::new("watch"),
            role: Role::Arbiter,
            issue: issue("1"),
            folder: None
        }]
    );
}

#[test]
fn an_issue_being_fixed_moved_out_is_not_told_or_nudged_until_it_moves_back() {
    let mut w = in_review();
    w.apply(&moved("1", "m9"));
    assert_eq!(w.apply(&block(1)), vec![], "no fix is sent");
    assert_eq!(stage(&w, "1"), Stage::Fixing { round: 1 });
    assert!(!waited_on(&w, "author-1"), "its step is done");
    assert_eq!(w.missing(), Missing::default());
    w.apply(&moved("1", "m1"));
    assert!(waited_on(&w, "author-1"));
    assert_eq!(
        w.missing().start,
        vec![Effect::RequestFix {
            issue: issue("1"),
            points: vec!["no test for a stopped clock".into()]
        }]
    );

    // An author that heard its fixes before the move finishes them.
    let mut w = in_review();
    w.apply(&block(1));
    w.apply(&Event::Told { issue: issue("1") });
    w.apply(&moved("1", "m9"));
    assert!(waited_on(&w, "author-1"));
}

#[test]
fn an_approved_issue_moved_out_is_not_merged_until_it_moves_back() {
    let mut w = in_review();
    w.apply(&moved("1", "m9"));
    assert_eq!(w.apply(&verdict(1, Verdict::Approve)), vec![]);
    assert_eq!(w.apply(&checks(true)), vec![], "no merge");
    assert_eq!(stage(&w, "1"), Stage::Merging);
    assert_eq!(w.missing(), Missing::default());
    assert_eq!(holder(&w, "f1"), Some(issue("1")));
    w.apply(&moved("1", "m1"));
    assert_eq!(w.missing().start, vec![Effect::Merge { issue: issue("1") }]);
}

#[test]
fn a_merge_under_way_finishes_and_deploys_after_its_issue_leaves_scope() {
    let mut w = merging_two();
    merged(&mut w, "ui");
    let narrowed = Event::GroupCreated {
        group: GroupId::new("watch"),
        name: "Night watch".into(),
        projects: vec![],
        areas: vec![pid("guild")],
        milestones: vec![MilestoneId::new("m1")],
        refining_at_once: 2,
        model: None,
        queue_low: None,
        investigate_every: None,
    };
    assert_eq!(w.apply(&narrowed), vec![]);
    assert!(w.out_of_scope(&w.issues[&gld()]));
    assert_eq!(
        w.missing().start,
        vec![Effect::Merge { issue: gld() }],
        "the next repository still merges"
    );
    assert!(merged(&mut w, "api").contains(&Effect::Deploy { issue: gld() }));
    assert_eq!(w.missing().start, vec![Effect::Deploy { issue: gld() }]);
    assert_eq!(
        w.apply(&Event::Deployed { issue: gld() }),
        vec![Effect::Verify { issue: gld() }]
    );
    assert_eq!(w.missing().start, vec![Effect::Verify { issue: gld() }]);
    assert_eq!(
        w.apply(&Event::Verified {
            issue: gld(),
            passed: true,
            output: String::new()
        }),
        vec![Effect::Close { issue: gld() }]
    );
}

#[test]
fn a_ready_issue_moved_into_another_groups_milestones_is_taken_by_that_group() {
    let guild = Event::GroupCreated {
        group: GroupId::new("guild"),
        name: "Thieves' Guild".into(),
        projects: vec![pid("ankh")],
        areas: vec![],
        milestones: vec![MilestoneId::new("m2")],
        refining_at_once: 1,
        model: None,
        queue_low: None,
        investigate_every: None,
    };
    let (mut w, _) = World::replay(&[
        group(&["m1"], 1),
        guild.clone(),
        sync("1", "m1", 1, &[]),
        planned("1"),
    ]);
    assert_eq!(w.issues[&issue("1")].group, Some(GroupId::new("watch")));
    assert_eq!(w.apply(&moved("1", "m2")), vec![], "no folder yet");
    assert_eq!(w.issues[&issue("1")].group, Some(GroupId::new("guild")));
    assert_eq!(
        w.apply(&folder("f1")),
        vec![Effect::StartAgent {
            group: GroupId::new("guild"),
            role: Role::Author,
            issue: issue("1"),
            folder: Some(FolderId::new("f1")),
        }],
        "an author, and no refiner"
    );

    // One past ready holds the watch's agent and folder, so it stays and waits.
    let mut w = in_review();
    w.apply(&guild);
    w.apply(&moved("1", "m2"));
    assert_eq!(w.issues[&issue("1")].group, Some(GroupId::new("watch")));
    assert_eq!(w.apply(&block(1)), vec![]);
    assert_eq!(w.missing(), Missing::default());
}

#[test]
fn an_issue_that_left_its_groups_scope_shows_its_stage_and_out_of_scope() {
    let mut w = World::default();
    w.apply(&group(&["m1"], 1));
    w.apply(&sync("2", "m1", 2, &[]));
    w.apply(&planned("2"));
    // Pinned to a machine with no work folder: in scope it waits for one there.
    w.apply(&pinned("2", "uberwald"));
    assert_eq!(w.machine_waits().len(), 1);
    w.apply(&moved("2", "m9"));
    let status = crate::text::text(&w, &BTreeMap::new());
    assert!(
        status.contains("\n  ankh#2  ready, out of scope  issue 2 [m9]\n"),
        "{status}"
    );
    // It waits on its scope, and not on the machine any more.
    assert!(
        status.contains(
            "Waiting\n  ankh#2 waiting, out of scope (m9 is not a milestone of watch)\nAgents\n"
        ),
        "{status}"
    );
    let unread = Err("not read here".into());
    let shown = crate::text::issue(&w, &issue("2"), &unread).unwrap_or_default();
    assert!(shown.contains("\nStage: ready, out of scope\n"), "{shown}");
}

#[test]
fn an_arbiter_that_ruled_is_stopped_and_not_waited_on_when_its_paused_issue_returns_to_arbitration()
{
    let mut w = arbitrating();
    w.apply(&started("arbiter-1", Role::Arbiter, "1"));
    let effects = w.apply(&ruling(false));
    assert!(
        effects.contains(&Effect::StopAgent {
            agent: AgentId::new("arbiter-1")
        }),
        "its step is done once its ruling is in: {effects:?}"
    );
    w.apply(&Event::Told { issue: issue("1") });
    w.apply(&pull_ready());
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 4 });
    w.apply(&moved("1", "m9"));
    assert_eq!(w.apply(&block(4)), vec![], "no arbiter");
    assert_eq!(stage(&w, "1"), Stage::Arbitration { round: 4 });
    assert!(!waited_on(&w, "arbiter-1"), "it ruled on the last dispute");
    // The runner reports the stop; back in scope a fresh arbiter hears the new dispute.
    w.apply(&ended("arbiter-1"));
    assert_eq!(w.missing(), Missing::default());
    w.apply(&moved("1", "m1"));
    assert_eq!(
        w.missing().start,
        vec![Effect::StartAgent {
            group: GroupId::new("watch"),
            role: Role::Arbiter,
            issue: issue("1"),
            folder: None
        }]
    );
}
