//! A person's stop of one agent (`temet-work stop AGENT`): its terminal is
//! stopped, nothing waits on it, and no agent starts on its issue until a person
//! starts the issue again (`temet-work start ISSUE`). Other agents of the issue
//! carry on, and its merge, deploy and verify are still asked for.

use super::*;
use crate::model::AgentState;

fn stop(agent: &str) -> Event {
    Event::AgentStoppedByPerson {
        agent: AgentId::new(agent),
    }
}

fn start() -> Event {
    Event::IssueStartedByPerson { issue: issue("1") }
}

fn stop_effect(agent: &str) -> Effect {
    Effect::StopAgent {
        agent: AgentId::new(agent),
    }
}

/// Whether anything in `effects` starts an agent on issue 1 or tells one.
fn starts_or_tells(effects: &[Effect]) -> bool {
    effects.iter().any(|e| match e {
        Effect::StartAgent { issue: i, .. }
        | Effect::StartReview { issue: i, .. }
        | Effect::RequestFix { issue: i, .. } => *i == issue("1"),
        _ => false,
    })
}

fn pull_ready() -> Event {
    Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
        replies: Vec::new(),
    }
}

/// Issue 1 refining with refiner-1 running.
fn refining() -> World {
    let mut w = World::default();
    w.apply(&group(&["m1"], 1));
    w.apply(&folder("f1"));
    w.apply(&sync("1", "m1", 1, &[]));
    w.apply(&started("refiner-1", Role::Refiner, "1"));
    w
}

/// Issue 1 being worked on by author-1 in f1.
fn working() -> World {
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    w
}

/// Issue 1 being fixed after a block in round 1, author-1 told of it.
fn fixing() -> World {
    let mut w = in_review();
    w.apply(&Event::Told { issue: issue("1") });
    w.apply(&verdict(
        1,
        Verdict::Block {
            points: vec!["no test for a stopped clock".into()],
        },
    ));
    w.apply(&Event::Told { issue: issue("1") });
    w
}

/// Issue 1 with the arbiter, arbiter-1 running.
fn with_the_arbiter() -> World {
    let mut w = arbitrating();
    w.apply(&started("arbiter-1", Role::Arbiter, "1"));
    w
}

#[test]
fn a_person_stop_ends_the_agent_and_its_issue_waits_on_it_no_more() {
    let mut w = working();
    assert!(waited_on(&w, "author-1"));
    let at = 1_795_000_000_000;
    assert_eq!(
        w.apply_at(at, &stop("author-1")),
        vec![stop_effect("author-1")]
    );
    assert!(
        !waited_on(&w, "author-1"),
        "not reminded or stopped for idling"
    );
    assert_eq!(w.person_stopped(&AgentId::new("author-1")), Some(Some(at)));
    assert_eq!(w.stopped_by(&issue("1")), Some(&AgentId::new("author-1")));
    // Asked for again each round until it ends, since a stop may fail.
    assert_eq!(w.missing().start, vec![stop_effect("author-1")]);

    // Its end, once the runner reports it, does not count toward `starts`: it
    // did not give up its step.
    w.apply(&ended("author-1"));
    let a = &w.agents[&AgentId::new("author-1")];
    assert!(!a.ended_waited_on && !a.lost_waited_on);
    assert_eq!(w.missing(), Missing::default());
    // Not even a lost program counts.
    let mut w = working();
    w.apply(&stop("author-1"));
    w.apply(&Event::AgentChanged {
        agent: AgentId::new("author-1"),
        state: AgentState::Ended,
        lost: true,
    });
    assert!(!w.agents[&AgentId::new("author-1")].lost_waited_on);
}

#[test]
fn a_person_stop_starts_no_fresh_agent_on_its_issue_whatever_the_role() {
    let cases: [(&str, World, &str); 5] = [
        ("a refiner while refining", refining(), "refiner-1"),
        ("an author while working", working(), "author-1"),
        ("an author while fixing", fixing(), "author-1"),
        ("a reviewer in review", in_review(), "reviewer-1"),
        (
            "an arbiter with the arbiter",
            with_the_arbiter(),
            "arbiter-1",
        ),
    ];
    for (what, mut w, agent) in cases {
        let before = w.issues[&issue("1")].clone();
        let held: Vec<_> = w.folders.values().map(|f| f.holder.clone()).collect();
        assert!(waited_on(&w, agent), "{what}: waited on before");
        w.apply(&stop(agent));
        w.apply(&ended(agent));
        for round in 0..3 {
            let missing = w.missing();
            assert!(
                !starts_or_tells(&missing.start),
                "{what}, round {round}: {missing:?}"
            );
            assert!(missing.given_up.is_empty(), "{what}: {missing:?}");
        }
        // The issue keeps its stage, folders, plan and rounds.
        let after = &w.issues[&issue("1")];
        assert_eq!(after.stage, before.stage, "{what}");
        assert_eq!(
            (
                &after.folder,
                &after.other_folders,
                &after.plan,
                after.round
            ),
            (
                &before.folder,
                &before.other_folders,
                &before.plan,
                before.round
            ),
            "{what}"
        );
        let still: Vec<_> = w.folders.values().map(|f| f.holder.clone()).collect();
        assert_eq!(still, held, "{what}: the folders stay held");
    }
}

#[test]
fn a_person_stop_lets_the_issues_other_agents_carry_on() {
    // The author is stopped in review: the reviewer still runs and is waited on,
    // and its block is heard, but no author starts for the fix.
    let mut w = in_review();
    w.apply(&Event::Told { issue: issue("1") });
    w.apply(&stop("author-1"));
    w.apply(&ended("author-1"));
    assert!(waited_on(&w, "reviewer-1"));
    let block = verdict(
        1,
        Verdict::Block {
            points: vec!["the bell rings twice".into()],
        },
    );
    let effects = w.apply(&block);
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::StartAgent { .. })),
        "{effects:?}"
    );
    assert_eq!(stage(&w, "1"), Stage::Fixing { round: 1 });
    assert!(
        !starts_or_tells(&w.missing().start),
        "no author for the fix"
    );

    // The reviewer is stopped while the author fixes: the author's ready is
    // taken, and starts no reviewer.
    let mut w = fixing();
    w.apply(&stop("reviewer-1"));
    w.apply(&ended("reviewer-1"));
    assert!(waited_on(&w, "author-1"));
    assert_eq!(w.apply(&pull_ready()), vec![], "no reviewer starts");
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 2 });
    assert!(!starts_or_tells(&w.missing().start));
}

#[test]
fn a_person_stop_still_asks_for_the_merge_deploy_and_verify() {
    // Stopping between repositories would leave main branches that do not match.
    let mut w = in_review();
    w.apply(&Event::Told { issue: issue("1") });
    w.apply(&stop("author-1"));
    w.apply(&verdict(1, Verdict::Approve));
    assert!(
        w.apply(&checks(true))
            .contains(&Effect::Merge { issue: issue("1") })
    );
    assert!(
        w.missing()
            .start
            .contains(&Effect::Merge { issue: issue("1") })
    );
    let merged = w.apply(&Event::Merged {
        issue: issue("1"),
        repo: None,
    });
    assert!(merged.contains(&Effect::Deploy { issue: issue("1") }));
    assert!(
        w.missing()
            .start
            .contains(&Effect::Deploy { issue: issue("1") })
    );
    w.apply(&Event::Deployed { issue: issue("1") });
    assert!(
        w.missing()
            .start
            .contains(&Effect::Verify { issue: issue("1") })
    );
}

#[test]
fn a_replay_keeps_a_person_stop_and_asks_again_for_the_stop_of_an_agent_still_running() {
    let mut w = working();
    w.apply(&stop("author-1"));
    // A replay drops effects, as a restart does: the stop may never have run.
    let (again, _) = World::default().replay_onto(
        [
            group(&["m1"], 1),
            folder("f1"),
            sync("1", "m1", 1, &[]),
            planned("1"),
            started("author-1", Role::Author, "1"),
            stop("author-1"),
        ]
        .iter(),
    );
    assert_eq!(again.stopped_by(&issue("1")), w.stopped_by(&issue("1")));
    assert_eq!(again.missing().start, vec![stop_effect("author-1")]);
    assert!(!again.waiting_on(&again.agents[&AgentId::new("author-1")]));
}

#[test]
fn a_person_start_lets_the_issue_go_on_with_a_fresh_agent_once_the_stopped_one_ended() {
    let mut w = working();
    w.apply(&stop("author-1"));
    // Started again while the stopped author still runs, as when its stop keeps
    // failing: its stop is still asked for, and no fresh one starts beside it.
    assert_eq!(w.apply(&start()), vec![]);
    assert_eq!(w.stopped_by(&issue("1")), None);
    assert!(
        w.person_stopped(&AgentId::new("author-1")).is_some(),
        "still stopped"
    );
    assert_eq!(w.missing().start, vec![stop_effect("author-1")]);
    w.apply(&ended("author-1"));
    // A fresh author, in the issue's folder.
    assert_eq!(w.missing().start, vec![author("1", "f1")]);

    // A stopped reviewer is not told the round once the issue starts again;
    // a fresh one starts after it has ended, and hears the round in its brief.
    let mut w = fixing();
    w.apply(&stop("reviewer-1"));
    w.apply(&pull_ready());
    w.apply(&start());
    assert!(
        !w.missing()
            .start
            .iter()
            .any(|e| matches!(e, Effect::StartReview { .. })),
        "{:?}",
        w.missing()
    );
    w.apply(&ended("reviewer-1"));
    assert!(w.missing().start.contains(&Effect::StartReview {
        issue: issue("1"),
        round: 2
    }));

    // A ready issue a person stopped is placed once started.
    let mut w = refining();
    w.apply(&stop("refiner-1"));
    // The plan stops the refiner, as ever, and places no author.
    assert_eq!(w.apply(&planned("1")), vec![stop_effect("refiner-1")]);
    assert_eq!(stage(&w, "1"), Stage::Ready);
    w.apply(&ended("refiner-1"));
    assert_eq!(w.apply(&start()), vec![author("1", "f1")]);
}

#[test]
fn a_person_stop_of_an_investigator_ends_its_investigation_and_the_next_waits() {
    let mut w = World::replay(&[
        Event::ProjectAdded {
            project: ProjectId::new("ankh"),
            tracker: "fake".into(),
            name: "city/ankh".into(),
            code_host: None,
            repo: None,
        },
        folder("f1"),
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![ProjectId::new("ankh")],
            areas: Vec::new(),
            milestones: vec![MilestoneId::new("m1")],
            refining_at_once: 1,
            model: None,
            queue_low: None,
            investigate_every: Some(30),
        },
    ])
    .0;
    let begin = w.investigation_events(0, true);
    let [Event::InvestigationStarted { agent, .. }] = begin.as_slice() else {
        panic!("{begin:?}");
    };
    let agent = agent.clone();
    w.apply(&begin[0]);
    w.apply(&Event::AgentStarted {
        agent: agent.clone(),
        group: GroupId::new("watch"),
        role: Role::Investigator,
        issue: None,
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    });
    assert!(w.waiting_on(&w.agents[&agent]));
    assert_eq!(
        w.apply(&Event::AgentStoppedByPerson {
            agent: agent.clone()
        }),
        vec![Effect::StopAgent {
            agent: agent.clone()
        }]
    );
    assert!(!w.waiting_on(&w.agents[&agent]));
    assert!(w.stopped_issues.is_empty(), "it works on no issue");
    // Running until its session ends, as when the stop keeps failing.
    assert!(w.investigating(&GroupId::new("watch")).is_some());
    assert_eq!(
        w.missing().start,
        vec![Effect::StopAgent {
            agent: agent.clone()
        }]
    );
    w.apply(&Event::AgentChanged {
        agent: agent.clone(),
        state: AgentState::Ended,
        lost: false,
    });
    let ended_at = 60_000;
    for e in w.investigation_events(ended_at, false) {
        w.apply(&e);
    }
    assert!(
        w.investigating(&GroupId::new("watch")).is_none(),
        "it ended"
    );
    let wait = 30 * 60_000;
    assert!(
        !w.investigation_events(ended_at + wait - 1, true)
            .iter()
            .any(|e| matches!(e, Event::InvestigationStarted { .. })),
        "the next waits investigate_every"
    );
    assert!(
        w.investigation_events(ended_at + wait, true)
            .iter()
            .any(|e| matches!(e, Event::InvestigationStarted { .. }))
    );
}
