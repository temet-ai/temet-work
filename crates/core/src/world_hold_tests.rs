//! A held daemon (`temet-work hold`): no issue is taken and no agent starts, no
//! fix, review round or merge goes out, and nothing waits on an agent, until a
//! person resumes it. Work past the agents finishes.

use std::collections::BTreeMap;

use super::*;

fn pull_ready() -> Event {
    Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
        replies: Vec::new(),
    }
}

fn holder(w: &World, folder: &str) -> Option<IssueId> {
    w.folders[&FolderId::new(folder)].holder.clone()
}

#[test]
fn a_held_daemon_takes_no_issue_and_places_no_author_until_it_is_resumed() {
    let mut w = World::default();
    w.apply(&group(&["m1"], 1));
    w.apply(&folder("f1"));
    assert_eq!(w.apply(&Event::Held), vec![]);
    assert!(w.held);
    assert_eq!(w.apply(&sync("1", "m1", 1, &[])), vec![], "not taken");
    assert_eq!(stage(&w, "1"), Stage::New);
    assert_eq!(w.missing(), Missing::default());
    assert_eq!(w.apply(&Event::Resumed), vec![refiner("1")]);
    assert_eq!(stage(&w, "1"), Stage::Refining);

    // A plan that comes in while held places no author, and its issue keeps
    // no folder.
    w.apply(&Event::Held);
    assert_eq!(w.missing(), Missing::default(), "no refiner either");
    assert_eq!(w.apply(&planned("1")), vec![], "no author");
    assert_eq!(stage(&w, "1"), Stage::Ready);
    assert_eq!(holder(&w, "f1"), None);
    assert_eq!(w.missing(), Missing::default());
    assert_eq!(w.apply(&Event::Resumed), vec![author("1", "f1")]);
    assert_eq!(stage(&w, "1"), Stage::Working);
    assert_eq!(holder(&w, "f1"), Some(issue("1")));
}

#[test]
fn an_agent_at_work_finishes_its_step_and_the_next_step_waits_for_the_resume() {
    // The author's ready starts no reviewer.
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    w.apply(&Event::Held);
    assert_eq!(w.apply(&pull_ready()), vec![], "no reviewer");
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 1 });
    assert_eq!(w.missing(), Missing::default());
    assert_eq!(holder(&w, "f1"), Some(issue("1")), "it keeps its folder");
    assert_eq!(w.apply(&Event::Resumed), vec![]);
    assert_eq!(
        w.missing().start,
        vec![Effect::StartReview {
            issue: issue("1"),
            round: 1
        }]
    );

    // The reviewer's block sends no fix.
    let mut w = in_review();
    w.apply(&Event::Told { issue: issue("1") });
    w.apply(&Event::Held);
    let block = verdict(
        1,
        Verdict::Block {
            points: vec!["no test for a stopped clock".into()],
        },
    );
    assert_eq!(w.apply(&block), vec![], "no fix");
    assert_eq!(stage(&w, "1"), Stage::Fixing { round: 1 });
    assert_eq!(w.missing(), Missing::default());
    w.apply(&Event::Resumed);
    assert_eq!(
        w.missing().start,
        vec![Effect::RequestFix {
            issue: issue("1"),
            points: w.issues[&issue("1")].fix.clone(),
        }]
    );

    // An approval with passing checks asks for no merge.
    let mut w = in_review();
    w.apply(&Event::Held);
    w.apply(&verdict(1, Verdict::Approve));
    let checked = w.apply(&Event::ChecksFinished {
        issue: issue("1"),
        passed: true,
        failing: vec![],
    });
    assert!(
        !checked.contains(&Effect::Merge { issue: issue("1") }),
        "{checked:?}"
    );
    assert_eq!(stage(&w, "1"), Stage::Merging);
    assert_eq!(w.missing(), Missing::default());
    w.apply(&Event::Resumed);
    assert_eq!(w.missing().start, vec![Effect::Merge { issue: issue("1") }]);
}

#[test]
fn an_issue_with_a_pull_request_merged_still_merges_deploys_and_verifies_while_held() {
    let mut w = merging_two();
    w.apply(&Event::Held);
    assert_eq!(w.missing(), Missing::default(), "nothing merged: it waits");
    merged(&mut w, "ui");
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
fn nothing_waits_on_an_agent_while_held_and_one_that_ends_uses_no_start() {
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    assert!(waited_on(&w, "author-1"));
    w.apply(&Event::Held);
    assert!(!waited_on(&w, "author-1"), "not reminded or stopped");
    w.apply(&ended("author-1"));
    assert!(!w.agents[&AgentId::new("author-1")].ended_waited_on);
    assert_eq!(w.missing(), Missing::default(), "no author in its place");
    w.apply(&Event::Resumed);
    assert_eq!(w.missing().start, vec![author("1", "f1")]);
}

#[test]
fn a_hold_and_the_time_it_began_outlast_a_replay_until_a_resume() {
    let at = 1_795_000_000_000;
    let log = [
        (at - 60_000, group(&["m1"], 1)),
        (at, Event::Held),
        (at + 60_000, sync("1", "m1", 1, &[])),
    ];
    let (w, effects) = World::default().replay_at(log.iter().map(|(t, e)| (*t, e)));
    assert_eq!(effects, vec![], "nothing started on the way");
    assert!(w.held);
    assert_eq!(w.held_since, Some(at));
    let since = crate::investigation::utc(at);
    let status = crate::text::text(&w, &BTreeMap::new());
    assert!(
        status.starts_with(&format!(
            "Held since {since}: no issue is taken and no agent or investigation starts; agents \
             at work finish their turn, and merged changes still deploy. temet-work resume lifts \
             it.\nProjects\n"
        )),
        "{status}"
    );
    let view = crate::View::new(&w, &BTreeMap::new());
    assert_eq!((view.held, view.held_since), (true, Some(at)));

    let (w, _) = w.replay_at([(at + 120_000, &Event::Resumed)]);
    assert!(!w.held);
    assert_eq!(w.held_since, None);
    assert!(crate::text::text(&w, &BTreeMap::new()).starts_with("Projects\n"));
    let view = crate::View::new(&w, &BTreeMap::new());
    assert_eq!((view.held, view.held_since), (false, None));
}
