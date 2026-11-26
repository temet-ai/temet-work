use super::*;
use crate::event::{Event, Verdict};
use crate::ids::{
    AgentId, DecisionId, FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId,
};
use crate::model::{Decision, DecisionKind, Plan, Role, Stage};

fn issue(key: &str) -> IssueId {
    IssueId::new("ankh", key)
}

fn group(milestones: &[&str], refining_at_once: usize) -> Event {
    Event::GroupCreated {
        group: GroupId::new("watch"),
        name: "Night watch".into(),
        projects: vec![ProjectId::new("ankh")],
        milestones: milestones.iter().map(|m| MilestoneId::new(*m)).collect(),
        refining_at_once,
    }
}

fn folder(id: &str) -> Event {
    Event::FolderAdded {
        folder: FolderId::new(id),
        project: ProjectId::new("ankh"),
        machine: MachineId::new("morpork"),
    }
}

fn sync(key: &str, milestone: &str, order: u64, deps: &[&str]) -> Event {
    Event::IssueSynced {
        issue: issue(key),
        title: format!("issue {key}"),
        milestone: Some(MilestoneId::new(milestone)),
        depends_on: deps.iter().map(|d| issue(d)).collect(),
        order,
        closed: false,
    }
}

fn planned(key: &str) -> Event {
    Event::PlanWritten {
        issue: issue(key),
        plan: Plan::default(),
    }
}

fn refiner(key: &str) -> Effect {
    Effect::StartAgent {
        group: GroupId::new("watch"),
        role: Role::Refiner,
        issue: issue(key),
        folder: None,
    }
}

fn author(key: &str, folder: &str) -> Effect {
    Effect::StartAgent {
        group: GroupId::new("watch"),
        role: Role::Author,
        issue: issue(key),
        folder: Some(FolderId::new(folder)),
    }
}

/// A world where issue 1 has a plan and an author working in folder `f1`.
fn working_on_one() -> World {
    let (world, _) = World::replay(&[
        group(&["m1"], 1),
        folder("f1"),
        sync("1", "m1", 1, &[]),
        planned("1"),
    ]);
    world
}

fn stage(world: &World, key: &str) -> Stage {
    world.issues[&issue(key)].stage
}

#[test]
fn refines_one_issue_at_a_time_by_default() {
    let mut w = World::default();
    assert_eq!(w.apply(&group(&["m1"], 1)), vec![]);
    assert_eq!(w.apply(&sync("1", "m1", 1, &[])), vec![refiner("1")]);
    assert_eq!(w.apply(&sync("2", "m1", 2, &[])), vec![]);
    assert_eq!(stage(&w, "2"), Stage::New);
    // The plan frees the refining slot; no folder exists, so no author starts.
    assert_eq!(w.apply(&planned("1")), vec![refiner("2")]);
    assert_eq!(stage(&w, "1"), Stage::Ready);
}

#[test]
fn takes_issues_in_the_groups_milestone_order_then_tracker_order() {
    let (_, effects) = World::replay(&[
        sync("1", "m1", 1, &[]),
        sync("2", "m2", 2, &[]),
        sync("3", "m2", 3, &[]),
        group(&["m2", "m1"], 3),
    ]);
    assert_eq!(effects, vec![refiner("2"), refiner("3"), refiner("1")]);
}

#[test]
fn leaves_issues_outside_the_groups_projects_and_milestones() {
    let other_project = Event::IssueSynced {
        issue: IssueId::new("unseen", "9"),
        title: "elsewhere".into(),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: 1,
        closed: false,
    };
    let (w, effects) = World::replay(&[group(&["m1"], 5), other_project, sync("2", "m9", 2, &[])]);
    assert_eq!(effects, vec![]);
    assert_eq!(stage(&w, "2"), Stage::New);
}

#[test]
fn starts_an_author_when_a_folder_is_free_and_its_dependencies_are_done() {
    let mut w = World::default();
    w.apply(&group(&["m1"], 2));
    w.apply(&sync("1", "m1", 1, &["2"]));
    w.apply(&sync("2", "m1", 2, &[]));
    w.apply(&planned("1"));
    w.apply(&planned("2"));
    // No folder yet: both wait.
    assert_eq!(stage(&w, "1"), Stage::Ready);
    // Issue 1 depends on issue 2, so issue 2 starts first although it comes later.
    assert_eq!(w.apply(&folder("f1")), vec![author("2", "f1")]);
    assert_eq!(w.apply(&folder("f2")), vec![]);
    assert_eq!(stage(&w, "1"), Stage::Ready);
}

#[test]
fn one_issue_holds_a_folder_at_a_time() {
    let (w, effects) = World::replay(&[
        group(&["m1"], 2),
        folder("f1"),
        sync("1", "m1", 1, &[]),
        sync("2", "m1", 2, &[]),
        planned("1"),
        planned("2"),
    ]);
    let authors: Vec<&Effect> = effects
        .iter()
        .filter(|e| {
            matches!(
                e,
                Effect::StartAgent {
                    role: Role::Author,
                    ..
                }
            )
        })
        .collect();
    assert_eq!(authors, vec![&author("1", "f1")]);
    assert_eq!(stage(&w, "2"), Stage::Ready);
}

#[test]
fn review_blocks_go_back_to_the_author_then_to_an_arbiter_after_three_rounds() {
    let mut w = working_on_one();
    let pr = Event::PullRequestReady { issue: issue("1") };
    let block = |round| Event::ReviewFinished {
        issue: issue("1"),
        round,
        verdict: Verdict::Block {
            points: vec!["no test for an invalid document".into()],
        },
    };
    for round in 1..REVIEW_ROUNDS {
        assert_eq!(
            w.apply(&pr),
            vec![Effect::StartReview {
                issue: issue("1"),
                round
            }]
        );
        assert_eq!(
            w.apply(&block(round)),
            vec![Effect::RequestFix {
                issue: issue("1"),
                points: vec!["no test for an invalid document".into()]
            }]
        );
    }
    w.apply(&pr);
    assert_eq!(
        w.apply(&block(REVIEW_ROUNDS)),
        vec![Effect::StartAgent {
            group: GroupId::new("watch"),
            role: Role::Arbiter,
            issue: issue("1"),
            folder: None
        }]
    );
    assert_eq!(
        stage(&w, "1"),
        Stage::Arbitration {
            round: REVIEW_ROUNDS
        }
    );
}

#[test]
fn a_verdict_for_an_old_round_changes_nothing() {
    let mut w = working_on_one();
    w.apply(&Event::PullRequestReady { issue: issue("1") });
    let stale = Event::ReviewFinished {
        issue: issue("1"),
        round: 2,
        verdict: Verdict::Approve,
    };
    assert_eq!(w.apply(&stale), vec![]);
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 1 });
}

#[test]
fn merges_once_review_and_checks_pass_in_either_order() {
    for checks_first in [false, true] {
        let mut w = working_on_one();
        w.apply(&Event::PullRequestReady { issue: issue("1") });
        let approve = Event::ReviewFinished {
            issue: issue("1"),
            round: 1,
            verdict: Verdict::Approve,
        };
        let checks = Event::ChecksFinished {
            issue: issue("1"),
            passed: true,
        };
        let (first, second) = if checks_first {
            (checks, approve)
        } else {
            (approve, checks)
        };
        assert_eq!(w.apply(&first), vec![]);
        assert_eq!(w.apply(&second), vec![Effect::Merge { issue: issue("1") }]);
    }
}

#[test]
fn failed_checks_go_back_to_the_author() {
    let mut w = working_on_one();
    w.apply(&Event::PullRequestReady { issue: issue("1") });
    let effects = w.apply(&Event::ChecksFinished {
        issue: issue("1"),
        passed: false,
    });
    assert!(matches!(effects.as_slice(), [Effect::RequestFix { .. }]));
    assert_eq!(stage(&w, "1"), Stage::Fixing { round: 1 });
}

#[test]
fn merged_deployed_and_checked_closes_the_issue_and_frees_the_folder_for_the_next() {
    let mut w = World::default();
    for e in [
        group(&["m1"], 2),
        folder("f1"),
        sync("1", "m1", 1, &[]),
        sync("2", "m1", 2, &[]),
        planned("1"),
        planned("2"),
        Event::PullRequestReady { issue: issue("1") },
        Event::ReviewFinished {
            issue: issue("1"),
            round: 1,
            verdict: Verdict::Approve,
        },
        Event::ChecksFinished {
            issue: issue("1"),
            passed: true,
        },
    ] {
        w.apply(&e);
    }
    assert_eq!(
        w.apply(&Event::Merged { issue: issue("1") }),
        vec![
            Effect::Deploy { issue: issue("1") },
            Effect::ReleaseFolder {
                folder: FolderId::new("f1")
            },
            author("2", "f1"),
        ]
    );
    assert_eq!(
        w.apply(&Event::Deployed { issue: issue("1") }),
        vec![Effect::Verify { issue: issue("1") }]
    );
    assert_eq!(
        w.apply(&Event::Verified {
            issue: issue("1"),
            passed: true
        }),
        vec![Effect::Close { issue: issue("1") }]
    );
    assert_eq!(stage(&w, "1"), Stage::Done);
}

#[test]
fn an_arbiters_ruling_approves_or_sends_the_change_back() {
    for approve in [true, false] {
        let mut w = working_on_one();
        w.apply(&Event::PullRequestReady { issue: issue("1") });
        for round in 1..=REVIEW_ROUNDS {
            w.apply(&Event::ReviewFinished {
                issue: issue("1"),
                round,
                verdict: Verdict::Block { points: vec![] },
            });
            if round < REVIEW_ROUNDS {
                w.apply(&Event::PullRequestReady { issue: issue("1") });
            }
        }
        w.apply(&Event::ChecksFinished {
            issue: issue("1"),
            passed: true,
        });
        let ruling = Event::DecisionMade {
            decision: Decision {
                id: DecisionId(1),
                issue: issue("1"),
                by: AgentId::new("arbiter-1"),
                kind: DecisionKind::Ruling { approve },
                question: "Must the elicitation fields be tested?".into(),
                options: vec!["approve".into(), "send back".into()],
                chosen: if approve { 0 } else { 1 },
                reason: "Scope names every elicitation field".into(),
                overturned_to: None,
            },
        };
        let effects = w.apply(&ruling);
        if approve {
            assert_eq!(effects, vec![Effect::Merge { issue: issue("1") }]);
        } else {
            assert_eq!(
                effects,
                vec![Effect::RequestFix {
                    issue: issue("1"),
                    points: vec!["Scope names every elicitation field".into()]
                }]
            );
        }
    }
}

#[test]
fn overturning_a_decision_applies_it_and_proposes_a_rule_once() {
    let mut w = working_on_one();
    w.apply(&Event::DecisionMade {
        decision: Decision {
            id: DecisionId(7),
            issue: issue("1"),
            by: AgentId::new("refiner-1"),
            kind: DecisionKind::Choice,
            question: "Which connection gives channel sessions the new protocol?".into(),
            options: vec!["a second connection".into(), "stay legacy".into()],
            chosen: 1,
            reason: "easiest to undo".into(),
            overturned_to: None,
        },
    });
    let overturn = Event::DecisionOverturned {
        decision: DecisionId(7),
        to: 0,
    };
    assert_eq!(
        w.apply(&overturn),
        vec![
            Effect::ApplyOverturn {
                decision: DecisionId(7)
            },
            Effect::ProposeRule {
                from: DecisionId(7)
            }
        ]
    );
    assert_eq!(w.apply(&overturn), vec![]);
    let to_the_same = Event::DecisionOverturned {
        decision: DecisionId(7),
        to: 1,
    };
    assert_eq!(w.apply(&to_the_same), vec![]);
}

#[test]
fn closing_an_issue_in_the_tracker_frees_its_folder() {
    let mut w = working_on_one();
    let closed = Event::IssueSynced {
        issue: issue("1"),
        title: "issue 1".into(),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: 1,
        closed: true,
    };
    assert_eq!(
        w.apply(&closed),
        vec![Effect::ReleaseFolder {
            folder: FolderId::new("f1")
        }]
    );
    assert_eq!(stage(&w, "1"), Stage::Done);
}

#[test]
fn the_same_events_give_the_same_state_and_effects() {
    let events = [
        group(&["m2", "m1"], 2),
        folder("f1"),
        folder("f2"),
        sync("1", "m1", 1, &[]),
        sync("2", "m2", 2, &["1"]),
        sync("3", "m2", 3, &[]),
        planned("2"),
        planned("3"),
        Event::PullRequestReady { issue: issue("3") },
    ];
    assert_eq!(World::replay(&events), World::replay(&events));
}
