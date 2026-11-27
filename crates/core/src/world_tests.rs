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
        areas: Vec::new(),
        milestones: milestones.iter().map(|m| MilestoneId::new(*m)).collect(),
        refining_at_once,
        model: None,
        queue_low: None,
        investigate_every: None,
    }
}

fn folder(id: &str) -> Event {
    Event::FolderAdded {
        folder: FolderId::new(id),
        project: ProjectId::new("ankh"),
        machine: MachineId::new("morpork"),
        path: format!("/srv/ankh/{id}"),
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
    let pr = Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
    };
    let block = |round| Event::ReviewFinished {
        issue: issue("1"),
        round,
        verdict: Verdict::Block {
            points: vec!["no test for an invalid document".into()],
        },
    };
    for round in 1..3 {
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
        w.apply(&block(3)),
        vec![Effect::StartAgent {
            group: GroupId::new("watch"),
            role: Role::Arbiter,
            issue: issue("1"),
            folder: None
        }]
    );
    assert_eq!(stage(&w, "1"), Stage::Arbitration { round: 3 });
}

#[test]
fn a_verdict_for_an_old_round_changes_nothing() {
    let mut w = working_on_one();
    w.apply(&Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
    });
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
        w.apply(&Event::PullRequestReady {
            issue: issue("1"),
            pull: None,
            pulls: Default::default(),
        });
        let approve = Event::ReviewFinished {
            issue: issue("1"),
            round: 1,
            verdict: Verdict::Approve,
        };
        let checks = Event::ChecksFinished {
            issue: issue("1"),
            passed: true,
            failing: vec![],
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
    w.apply(&Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
    });
    let effects = w.apply(&Event::ChecksFinished {
        issue: issue("1"),
        passed: false,
        failing: vec![],
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
        Event::PullRequestReady {
            issue: issue("1"),
            pull: None,
            pulls: Default::default(),
        },
        Event::ReviewFinished {
            issue: issue("1"),
            round: 1,
            verdict: Verdict::Approve,
        },
        Event::ChecksFinished {
            issue: issue("1"),
            passed: true,
            failing: vec![],
        },
    ] {
        w.apply(&e);
    }
    assert_eq!(
        w.apply(&Event::Merged {
            issue: issue("1"),
            repo: None
        }),
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
            passed: true,
            output: String::new(),
        }),
        vec![Effect::Close { issue: issue("1") }]
    );
    assert_eq!(stage(&w, "1"), Stage::Done);
}

#[test]
fn an_arbiters_ruling_approves_or_sends_the_change_back() {
    for approve in [true, false] {
        let mut w = working_on_one();
        w.apply(&Event::PullRequestReady {
            issue: issue("1"),
            pull: None,
            pulls: Default::default(),
        });
        for round in 1..=3 {
            w.apply(&Event::ReviewFinished {
                issue: issue("1"),
                round,
                verdict: Verdict::Block { points: vec![] },
            });
            if round < 3 {
                w.apply(&Event::PullRequestReady {
                    issue: issue("1"),
                    pull: None,
                    pulls: Default::default(),
                });
            }
        }
        w.apply(&Event::ChecksFinished {
            issue: issue("1"),
            passed: true,
            failing: vec![],
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
                overturn_reason: String::new(),
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

/// A world where issue 1 is with the arbiter after three review rounds, its
/// checks still running.
fn arbitrating() -> World {
    let mut w = working_on_one();
    w.apply(&Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
    });
    for round in 1..=3 {
        w.apply(&Event::ReviewFinished {
            issue: issue("1"),
            round,
            verdict: Verdict::Block { points: vec![] },
        });
        if round < 3 {
            w.apply(&Event::PullRequestReady {
                issue: issue("1"),
                pull: None,
                pulls: Default::default(),
            });
        }
    }
    w
}

/// The arbiter's ruling on issue 1.
fn ruling(approve: bool) -> Event {
    Event::DecisionMade {
        decision: Decision {
            id: DecisionId(1),
            issue: issue("1"),
            by: AgentId::new("arbiter-1"),
            kind: DecisionKind::Ruling { approve },
            question: "Must the fields be tested?".into(),
            options: vec!["merge".into(), "fix first".into()],
            chosen: usize::from(!approve),
            reason: "scope names every field".into(),
            overturned_to: None,
            overturn_reason: String::new(),
        },
    }
}

/// A world where the arbiter ruled on issue 1 after its checks passed.
fn ruled(approve: bool) -> World {
    let mut w = arbitrating();
    w.apply(&Event::ChecksFinished {
        issue: issue("1"),
        passed: true,
        failing: vec![],
    });
    w.apply(&ruling(approve));
    w
}

fn overturned(to: usize) -> Event {
    Event::DecisionOverturned {
        decision: DecisionId(1),
        to,
        reason: "the fields are tested elsewhere".into(),
        by: "person".into(),
    }
}

#[test]
fn overturning_an_approval_before_the_merge_sends_the_issue_back_to_its_author() {
    let mut w = ruled(true);
    assert_eq!(stage(&w, "1"), Stage::Merging);
    assert_eq!(
        w.apply(&overturned(1)),
        vec![Effect::RequestFix {
            issue: issue("1"),
            points: vec!["the fields are tested elsewhere".into()]
        }]
    );
    assert_eq!(stage(&w, "1"), Stage::Fixing { round: 3 });
    assert!(!w.issues[&issue("1")].review_ok);
    assert_eq!(w.decisions[&DecisionId(1)].overturned_to, Some(1));
}

#[test]
fn overturning_a_send_back_while_it_is_fixed_merges_after_the_checks_pass() {
    let mut w = ruled(false);
    assert_eq!(stage(&w, "1"), Stage::Fixing { round: 3 });
    assert_eq!(w.apply(&overturned(0)), vec![]);
    assert_eq!(
        stage(&w, "1"),
        Stage::Fixing { round: 3 },
        "the fix finishes"
    );
    // The author's fix arrives: no review is asked for, the checks decide.
    assert_eq!(
        w.apply(&Event::PullRequestReady {
            issue: issue("1"),
            pull: None,
            pulls: Default::default(),
        }),
        vec![]
    );
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 4 });
    assert_eq!(w.missing().start, vec![], "no reviewer is started");
    assert_eq!(
        w.apply(&Event::ChecksFinished {
            issue: issue("1"),
            passed: true,
            failing: vec![]
        }),
        vec![Effect::Merge { issue: issue("1") }]
    );
    assert_eq!(stage(&w, "1"), Stage::Merging);
}

#[test]
fn overturning_a_send_back_still_in_review_merges_when_the_checks_have_passed() {
    let mut w = ruled(false);
    w.apply(&Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
    });
    w.apply(&Event::ChecksFinished {
        issue: issue("1"),
        passed: true,
        failing: vec![],
    });
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 4 });
    assert_eq!(
        w.apply(&overturned(0)),
        vec![Effect::Merge { issue: issue("1") }]
    );
}

#[test]
fn overturning_after_the_merge_changes_no_stage() {
    for approve in [true, false] {
        let mut w = ruled(approve);
        if !approve {
            w.apply(&Event::PullRequestReady {
                issue: issue("1"),
                pull: None,
                pulls: Default::default(),
            });
            w.apply(&Event::ReviewFinished {
                issue: issue("1"),
                round: 4,
                verdict: Verdict::Approve,
            });
            w.apply(&Event::ChecksFinished {
                issue: issue("1"),
                passed: true,
                failing: vec![],
            });
        }
        w.apply(&Event::Merged {
            issue: issue("1"),
            repo: None,
        });
        let before = w.issues[&issue("1")].clone();
        assert_eq!(w.apply(&overturned(usize::from(approve))), vec![]);
        assert_eq!(w.issues[&issue("1")], before);
        assert_eq!(stage(&w, "1"), Stage::Deploying);
        assert!(w.decisions[&DecisionId(1)].overturned_to.is_some());
    }
}

#[test]
fn overturning_the_same_decision_to_the_same_option_twice_changes_nothing() {
    let mut w = ruled(true);
    w.apply(&overturned(1));
    let after_first = w.clone();
    let again = w.apply(&overturned(1));
    assert_eq!(again, vec![]);
    assert_eq!(w.issues, after_first.issues);
    assert_eq!(w.decisions, after_first.decisions);
}

#[test]
fn overturning_a_choice_changes_no_stage_and_an_unknown_option_nothing_at_all() {
    let mut w = working_on_one();
    w.apply(&Event::DecisionMade {
        decision: Decision {
            id: DecisionId(1),
            issue: issue("1"),
            by: AgentId::new("refiner-1"),
            kind: DecisionKind::Choice,
            question: "Which connection?".into(),
            options: vec!["a second connection".into(), "stay legacy".into()],
            chosen: 1,
            reason: "easiest to undo".into(),
            overturned_to: None,
            overturn_reason: String::new(),
        },
    });
    let before = w.issues.clone();
    assert_eq!(w.apply(&overturned(5)), vec![]);
    assert_eq!(w.decisions[&DecisionId(1)].overturned_to, None);
    assert_eq!(w.apply(&overturned(1)), vec![], "the option already chosen");
    assert_eq!(w.apply(&overturned(0)), vec![]);
    assert_eq!(w.decisions[&DecisionId(1)].overturned_to, Some(0));
    assert_eq!(w.issues, before);
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
        Event::PullRequestReady {
            issue: issue("3"),
            pull: None,
            pulls: Default::default(),
        },
    ];
    assert_eq!(World::replay(&events), World::replay(&events));
}

fn started(agent: &str, role: Role, key: &str) -> Event {
    Event::AgentStarted {
        agent: AgentId::new(agent),
        group: GroupId::new("watch"),
        role,
        issue: Some(issue(key)),
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
    }
}

fn ended(agent: &str) -> Event {
    Event::AgentChanged {
        agent: AgentId::new(agent),
        state: crate::model::AgentState::Ended,
    }
}

#[test]
fn an_issue_missing_its_agent_gets_one_until_three_have_ended() {
    let (mut w, _) = World::replay(&[group(&["m1"], 1), sync("1", "m1", 1, &[])]);
    assert_eq!(
        w.missing().start,
        vec![refiner("1")],
        "decided on, never started"
    );
    w.apply(&started("refiner-1", Role::Refiner, "1"));
    assert_eq!(w.missing(), crate::world::Missing::default());
    for n in 1..=3 {
        w.apply(&ended(&format!("refiner-{n}")));
        if n < 3 {
            assert_eq!(w.missing().start, vec![refiner("1")]);
            w.apply(&started(&format!("refiner-{}", n + 1), Role::Refiner, "1"));
        }
    }
    let gave_up = w.missing();
    assert_eq!(gave_up.start, vec![]);
    assert_eq!(
        gave_up.given_up,
        ["ankh#1: 3 refiners ended before it moved on; no more are started"]
    );
}

#[test]
fn the_newest_running_agent_is_the_one_that_hears() {
    let mut w = working_on_one();
    w.apply(&started("author-9", Role::Author, "1"));
    w.apply(&started("author-10", Role::Author, "1"));
    let running = w.running(&issue("1"), Role::Author).expect("an author");
    assert_eq!(
        running.id,
        AgentId::new("author-10"),
        "not the one that sorts last"
    );
}

#[test]
fn closing_an_issue_mid_work_stops_its_agents_before_freeing_its_folder() {
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    w.apply(&sync("2", "m1", 2, &[]));
    w.apply(&planned("2"));
    assert_eq!(stage(&w, "2"), Stage::Ready, "waiting for a folder");
    let close = Event::IssueSynced {
        issue: issue("1"),
        title: "issue 1".into(),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: 1,
        closed: true,
    };
    let effects = w.apply(&close);
    assert_eq!(
        &effects[..2],
        [
            Effect::StopAgent {
                agent: AgentId::new("author-1")
            },
            Effect::ReleaseFolder {
                folder: FolderId::new("f1")
            }
        ]
    );
    assert_eq!(
        effects[2],
        author("2", "f1"),
        "then the next issue takes the folder"
    );
}

#[test]
fn a_folder_moved_to_another_project_is_updated() {
    let mut w = World::default();
    w.apply(&folder("f1"));
    w.apply(&Event::FolderAdded {
        folder: FolderId::new("f1"),
        project: ProjectId::new("klatch"),
        machine: MachineId::new("morpork"),
        path: "/srv/klatch".into(),
    });
    assert_eq!(
        w.folders[&FolderId::new("f1")].project,
        ProjectId::new("klatch")
    );
}

#[test]
fn an_author_that_has_not_heard_its_fixes_is_told_again_until_it_has() {
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    w.apply(&Event::PullRequestReady {
        issue: issue("1"),
        pull: Some("57".into()),
        pulls: Default::default(),
    });
    w.apply(&started("reviewer-1", Role::Reviewer, "1"));
    let points = vec!["no test for a stopped clock".to_string()];
    w.apply(&Event::ReviewFinished {
        issue: issue("1"),
        round: 1,
        verdict: Verdict::Block {
            points: points.clone(),
        },
    });
    let fix = Effect::RequestFix {
        issue: issue("1"),
        points,
    };
    assert_eq!(
        w.missing().start,
        vec![fix.clone()],
        "after a restart the fix is sent again"
    );
    w.apply(&Event::Told { issue: issue("1") });
    assert_eq!(w.missing().start, vec![], "heard it");
}

#[test]
fn a_new_author_on_an_issue_being_fixed_hears_the_fixes_in_its_brief() {
    let mut w = working_on_one();
    w.apply(&Event::PullRequestReady {
        issue: issue("1"),
        pull: Some("57".into()),
        pulls: Default::default(),
    });
    w.apply(&Event::ReviewFinished {
        issue: issue("1"),
        round: 1,
        verdict: Verdict::Block {
            points: vec!["no test for a stopped clock".into()],
        },
    });
    let launch = w
        .launch(
            &GroupId::new("watch"),
            Role::Author,
            &issue("1"),
            Some(&FolderId::new("f1")),
        )
        .expect("launch");
    assert!(
        launch.brief.task.contains("- no test for a stopped clock"),
        "{}",
        launch.brief.task
    );
    w.apply(&started("author-1", Role::Author, "1"));
    assert!(w.issues[&issue("1")].told, "starting it told it");
}

#[test]
fn an_issue_reopened_in_the_tracker_is_worked_again_but_one_the_loop_closed_is_not() {
    let reopen = |closed| Event::IssueSynced {
        issue: issue("1"),
        title: "issue 1".into(),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: 1,
        closed,
    };
    let (mut w, _) = World::replay(&[group(&["m1"], 1), sync("1", "m1", 1, &[]), reopen(true)]);
    assert_eq!(stage(&w, "1"), Stage::Done);
    w.apply(&reopen(false));
    assert_eq!(stage(&w, "1"), Stage::Refining, "reopened, and taken again");

    let mut w = working_on_one();
    if let Some(i) = w.issues.get_mut(&issue("1")) {
        i.stage = Stage::Verifying;
    }
    w.apply(&Event::Verified {
        issue: issue("1"),
        passed: true,
        output: String::new(),
    });
    w.apply(&reopen(false));
    assert_eq!(
        stage(&w, "1"),
        Stage::Done,
        "the loop closed it; its close is retried instead"
    );
}

/// World `working_on_one` with issue 1's pull request 57 in review round 1.
fn in_review() -> World {
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    w.apply(&Event::PullRequestReady {
        issue: issue("1"),
        pull: Some("57".into()),
        pulls: Default::default(),
    });
    w.apply(&started("reviewer-1", Role::Reviewer, "1"));
    w
}

#[test]
fn a_ci_failure_or_a_refused_merge_does_not_use_up_a_review_round() {
    let mut w = in_review();
    w.apply(&Event::ChecksFinished {
        issue: issue("1"),
        passed: false,
        failing: vec!["tests".into()],
    });
    assert_eq!(w.issues[&issue("1")].fix, ["CI check tests failed"]);
    w.apply(&Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
    });
    assert_eq!(
        stage(&w, "1"),
        Stage::InReview { round: 1 },
        "still round 1"
    );
    w.apply(&Event::ReviewFinished {
        issue: issue("1"),
        round: 1,
        verdict: Verdict::Approve,
    });
    w.apply(&Event::ChecksFinished {
        issue: issue("1"),
        passed: true,
        failing: vec![],
    });
    assert_eq!(stage(&w, "1"), Stage::Merging);
    w.apply(&Event::MergeRefused {
        issue: issue("1"),
        reason: "it conflicts with main".into(),
    });
    assert_eq!(stage(&w, "1"), Stage::Fixing { round: 1 });
    assert_eq!(
        w.issues[&issue("1")].fix,
        ["the pull request cannot merge: it conflicts with main"]
    );
}

#[test]
fn a_conflict_seen_in_review_or_arbitration_goes_back_with_the_same_round() {
    let mut w = in_review();
    w.apply(&Event::PullConflicts {
        issue: issue("1"),
        repo: None,
    });
    assert_eq!(stage(&w, "1"), Stage::Fixing { round: 1 });
    assert_eq!(
        w.issues[&issue("1")].fix,
        [
            "the pull request conflicts with the branch it merges into: rebase onto that branch, \
             then push your own branch with `git push --force-with-lease`"
        ]
    );
    let mut w = in_review();
    w.issues.get_mut(&issue("1")).expect("issue").stage = Stage::Arbitration { round: 2 };
    w.apply(&Event::PullConflicts {
        issue: issue("1"),
        repo: None,
    });
    assert_eq!(stage(&w, "1"), Stage::Fixing { round: 2 });
    let mut w = in_review();
    w.issues.get_mut(&issue("1")).expect("issue").stage = Stage::Merging;
    w.apply(&Event::PullConflicts {
        issue: issue("1"),
        repo: None,
    });
    assert_eq!(stage(&w, "1"), Stage::Merging);
}

#[test]
fn a_conflict_fix_names_the_rebase_the_push_and_ready_in_that_order() {
    let mut w = in_review();
    let effects = w.apply(&Event::PullConflicts {
        issue: issue("1"),
        repo: None,
    });
    let [Effect::RequestFix { points, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let told = crate::brief::fixes(&issue("1"), points);
    let steps = [
        "rebase onto that branch",
        "git push --force-with-lease",
        "temet-work ready ankh#1",
    ];
    let at: Vec<usize> = steps
        .iter()
        .map(|s| told.find(s).unwrap_or_else(|| panic!("no {s:?} in {told}")))
        .collect();
    assert!(at.is_sorted(), "{told}");
}

/// The review's verdict on issue 1.
fn verdict(round: u8, verdict: Verdict) -> Event {
    Event::ReviewFinished {
        issue: issue("1"),
        round,
        verdict,
    }
}

fn checks(passed: bool) -> Event {
    Event::ChecksFinished {
        issue: issue("1"),
        passed,
        failing: vec![],
    }
}

fn waited_on(w: &World, agent: &str) -> bool {
    w.waiting_on(&w.agents[&AgentId::new(agent)])
}

#[test]
fn an_approving_reviewer_is_not_waited_on_while_checks_run() {
    let mut w = in_review();
    assert!(waited_on(&w, "reviewer-1"), "before its verdict");
    w.apply(&verdict(1, Verdict::Approve));
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 1 });
    assert_eq!(w.issues[&issue("1")].checks_ok, None);
    assert!(!waited_on(&w, "reviewer-1"));
}

#[test]
fn an_approving_reviewer_that_ends_is_not_replaced_while_checks_run() {
    let mut w = in_review();
    w.apply(&verdict(1, Verdict::Approve));
    w.apply(&ended("reviewer-1"));
    assert_eq!(w.missing(), crate::world::Missing::default());
}

#[test]
fn a_round_approved_before_its_running_reviewer_heard_of_it_is_not_told_again() {
    let mut w = in_review();
    w.apply(&verdict(
        1,
        Verdict::Block {
            points: vec!["no test for a stopped clock".into()],
        },
    ));
    w.apply(&Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
    });
    // A person's verdict does not wait for the reviewer to have heard the round.
    w.apply(&verdict(2, Verdict::Approve));
    assert!(!w.issues[&issue("1")].told);
    assert_eq!(w.missing().start, vec![]);
}

#[test]
fn after_a_ci_failure_the_next_round_waits_on_its_reviewer_again() {
    let mut w = in_review();
    w.apply(&verdict(1, Verdict::Approve));
    w.apply(&checks(false));
    let ready = Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
    };
    let next = Effect::StartReview {
        issue: issue("1"),
        round: 1,
    };
    assert_eq!(w.apply(&ready), vec![next.clone()], "the reviewer hears it");
    w.apply(&Event::Told { issue: issue("1") });
    assert!(waited_on(&w, "reviewer-1"));
    w.apply(&ended("reviewer-1"));
    assert_eq!(w.missing().start, vec![next]);
}

#[test]
fn an_approving_arbiter_is_not_waited_on_or_replaced_while_checks_run() {
    let mut w = arbitrating();
    w.apply(&started("arbiter-1", Role::Arbiter, "1"));
    assert!(waited_on(&w, "arbiter-1"), "before its ruling");
    w.apply(&ruling(true));
    assert_eq!(stage(&w, "1"), Stage::Arbitration { round: 3 });
    assert!(!waited_on(&w, "arbiter-1"));
    w.apply(&ended("arbiter-1"));
    assert_eq!(w.missing(), crate::world::Missing::default());
    assert_eq!(
        w.apply(&checks(true)),
        vec![Effect::Merge { issue: issue("1") }]
    );
}

#[test]
fn a_block_after_an_approval_in_the_last_round_leaves_the_arbiter_to_rule() {
    let block = |round| verdict(round, Verdict::Block { points: vec![] });
    let mut w = in_review();
    for round in 1..3 {
        w.apply(&block(round));
        w.apply(&Event::PullRequestReady {
            issue: issue("1"),
            pull: None,
            pulls: Default::default(),
        });
    }
    w.apply(&verdict(3, Verdict::Approve));
    w.apply(&block(3));
    w.apply(&started("arbiter-1", Role::Arbiter, "1"));
    assert!(waited_on(&w, "arbiter-1"));
    assert_eq!(
        w.apply(&checks(true)),
        vec![],
        "the approval was taken back"
    );
    assert_eq!(stage(&w, "1"), Stage::Arbitration { round: 3 });
}

#[test]
fn a_lost_merge_deploy_or_verify_is_asked_for_again() {
    let mut w = in_review();
    for (s, want) in [
        (Stage::Merging, Effect::Merge { issue: issue("1") }),
        (Stage::Deploying, Effect::Deploy { issue: issue("1") }),
        (Stage::Verifying, Effect::Verify { issue: issue("1") }),
    ] {
        if let Some(i) = w.issues.get_mut(&issue("1")) {
            i.stage = s;
        }
        assert_eq!(w.missing().start, vec![want]);
    }
}

#[test]
fn a_tracker_close_while_finishing_does_not_skip_verify() {
    let mut w = in_review();
    if let Some(i) = w.issues.get_mut(&issue("1")) {
        i.stage = Stage::Deploying;
    }
    w.apply(&Event::IssueSynced {
        issue: issue("1"),
        title: "issue 1".into(),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: 1,
        closed: true,
    });
    assert_eq!(
        stage(&w, "1"),
        Stage::Deploying,
        "closed on merge; still deploys and verifies"
    );
}

#[test]
fn a_failed_verify_sends_the_issue_back_with_why_and_without_the_merged_pull_request() {
    let mut w = in_review();
    if let Some(i) = w.issues.get_mut(&issue("1")) {
        i.stage = Stage::Verifying;
    }
    w.apply(&Event::Verified {
        issue: issue("1"),
        passed: false,
        output: "the clock struck twelve".into(),
    });
    let i = &w.issues[&issue("1")];
    assert!(i.pulls.is_empty());
    assert!(i.fix[0].ends_with("the clock struck twelve"), "{:?}", i.fix);
    let launch = w
        .launch(
            &GroupId::new("watch"),
            Role::Author,
            &issue("1"),
            Some(&FolderId::new("f1")),
        )
        .expect("launch");
    assert!(launch.brief.task.contains("the clock struck twelve"));
}

#[test]
fn an_ended_agent_stays_ended_when_a_late_update_arrives() {
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    w.apply(&ended("author-1"));
    w.apply(&Event::AgentChanged {
        agent: AgentId::new("author-1"),
        state: crate::model::AgentState::Working {
            doing: "sending the plan".into(),
        },
    });
    assert_eq!(
        w.agents[&AgentId::new("author-1")].state,
        crate::model::AgentState::Ended
    );
}

/// An area `sc` of projects `ui` and `api`, its issues from its own tracker, a
/// folder in each project, and a group that takes the whole area.
fn area_world() -> World {
    World::replay(&area_events()).0
}

fn area_events() -> Vec<Event> {
    vec![
        Event::AreaDefined {
            area: ProjectId::new("guild"),
            name: "Thieves' Guild".into(),
            projects: vec![ProjectId::new("ui"), ProjectId::new("api")],
            tracker: "fake".into(),
            tracker_name: "GLD".into(),
        },
        Event::ProjectAdded {
            project: ProjectId::new("ui"),
            tracker: String::new(),
            name: "group/ui".into(),
            code_host: Some("fake".into()),
            repo: None,
        },
        Event::ProjectAdded {
            project: ProjectId::new("api"),
            tracker: String::new(),
            name: "group/api".into(),
            code_host: Some("fake".into()),
            repo: None,
        },
        Event::FolderAdded {
            folder: FolderId::new("ui-1"),
            project: ProjectId::new("ui"),
            machine: MachineId::new("morpork"),
            path: "/srv/ui-1".into(),
        },
        Event::FolderAdded {
            folder: FolderId::new("api-1"),
            project: ProjectId::new("api"),
            machine: MachineId::new("morpork"),
            path: "/srv/api-1".into(),
        },
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![],
            areas: vec![ProjectId::new("guild")],
            milestones: vec![],
            refining_at_once: 2,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
    ]
}

fn area_issue(source: &str, key: &str) -> Event {
    Event::IssueSynced {
        issue: IssueId::new(source, key),
        title: format!("{source} {key}"),
        milestone: None,
        depends_on: vec![],
        order: 1,
        closed: false,
    }
}

#[test]
fn a_group_that_takes_an_area_takes_its_own_issues_and_its_projects() {
    let mut w = area_world();
    w.apply(&area_issue("guild", "GLD-12"));
    w.apply(&area_issue("api", "7"));
    w.apply(&area_issue("radar", "44"));
    assert_eq!(
        w.issues[&IssueId::new("guild", "GLD-12")].stage,
        Stage::Refining
    );
    assert_eq!(w.issues[&IssueId::new("api", "7")].stage, Stage::Refining);
    assert_eq!(
        w.issues[&IssueId::new("radar", "44")].stage,
        Stage::New,
        "not in the area"
    );
}

#[test]
fn an_area_issue_is_worked_in_the_first_repository_its_plan_names() {
    let mut w = area_world();
    let id = IssueId::new("guild", "GLD-12");
    w.apply(&area_issue("guild", "GLD-12"));
    let refiner = w
        .launch(&GroupId::new("watch"), Role::Refiner, &id, None)
        .expect("refiner");
    assert!(
        refiner.brief.task.contains("repositories are: ui, api"),
        "{}",
        refiner.brief.task
    );
    assert!(refiner.brief.task.contains("GLD-12 of GLD in its tracker"));
    let effects = w.apply(&Event::PlanWritten {
        issue: id.clone(),
        plan: Plan {
            repos: vec![ProjectId::new("api"), ProjectId::new("ui")],
            settings: Default::default(),
            checks: vec![],
        },
    });
    assert!(effects.contains(&Effect::StartAgent {
        group: GroupId::new("watch"),
        role: Role::Author,
        issue: id.clone(),
        folder: Some(FolderId::new("api-1")),
    }));
    assert_eq!(
        w.repo_of(&w.issues[&id]).map(|p| p.id.0.as_str()),
        Some("api")
    );
}

#[test]
fn a_retry_asks_again_for_the_deploy_or_verify_and_survives_a_restart() {
    let mut w = in_review();
    for (s, want) in [
        (Stage::Deploying, Effect::Retry { issue: issue("1") }),
        (Stage::Verifying, Effect::Retry { issue: issue("1") }),
    ] {
        if let Some(i) = w.issues.get_mut(&issue("1")) {
            i.stage = s;
        }
        assert!(
            w.apply(&Event::Retried { issue: issue("1") })
                .contains(&want),
            "{s:?}"
        );
    }
    // Replaying the log with the retry in it: same stage, deploy asked for again.
    let mut log = vec![
        group(&["m1"], 1),
        folder("f1"),
        sync("1", "m1", 1, &[]),
        planned("1"),
        Event::PullRequestReady {
            issue: issue("1"),
            pull: Some("57".into()),
            pulls: Default::default(),
        },
        Event::ReviewFinished {
            issue: issue("1"),
            round: 1,
            verdict: Verdict::Approve,
        },
        Event::ChecksFinished {
            issue: issue("1"),
            passed: true,
            failing: vec![],
        },
        Event::Merged {
            issue: issue("1"),
            repo: None,
        },
    ];
    let (before, _) = World::replay(&log);
    assert_eq!(stage(&before, "1"), Stage::Deploying);
    log.push(Event::Retried { issue: issue("1") });
    let (after, _) = World::replay(&log);
    assert_eq!(stage(&after, "1"), Stage::Deploying);
    assert_eq!(
        after.missing().start,
        vec![Effect::Deploy { issue: issue("1") }]
    );
}

#[test]
fn a_retry_of_an_issue_in_another_stage_changes_nothing() {
    let mut w = in_review();
    assert!(w.apply(&Event::Retried { issue: issue("1") }).is_empty());
}

// Stage 2: one issue across several repositories.

fn pid(s: &str) -> ProjectId {
    ProjectId::new(s)
}

fn two_repo_plan() -> Plan {
    Plan {
        repos: vec![pid("ui"), pid("api")],
        settings: Default::default(),
        checks: vec![],
    }
}

fn gld() -> IssueId {
    IssueId::new("guild", "GLD-12")
}

/// The area world with GLD-12 planned for [ui, api]; folders as `area_world` has.
fn two_repos_planned() -> World {
    let mut w = area_world();
    w.apply(&area_issue("guild", "GLD-12"));
    w
}

fn plan_two(w: &mut World) -> Vec<Effect> {
    w.apply(&Event::PlanWritten {
        issue: gld(),
        plan: two_repo_plan(),
    })
}

fn ready_both(w: &mut World) -> Vec<Effect> {
    w.apply(&Event::PullRequestReady {
        issue: gld(),
        pull: None,
        pulls: [
            (pid("ui"), "12".to_string()),
            (pid("api"), "34".to_string()),
        ]
        .into(),
    })
}

fn merged(w: &mut World, repo: &str) -> Vec<Effect> {
    w.apply(&Event::Merged {
        issue: gld(),
        repo: Some(pid(repo)),
    })
}

/// GLD-12 in the merging stage with both pull requests on record.
fn merging_two() -> World {
    let mut w = two_repos_planned();
    plan_two(&mut w);
    ready_both(&mut w);
    w.apply(&Event::ReviewFinished {
        issue: gld(),
        round: 1,
        verdict: Verdict::Approve,
    });
    w.apply(&Event::ChecksFinished {
        issue: gld(),
        passed: true,
        failing: vec![],
    });
    assert_eq!(w.issues[&gld()].stage, Stage::Merging);
    w
}

#[test]
fn an_issue_holds_a_folder_in_each_repository_its_plan_names_or_none() {
    let mut w = area_world();
    // Only ui has a free folder: take api's away by holding it for another issue.
    w.apply(&area_issue("guild", "GLD-12"));
    w.folders
        .get_mut(&FolderId::new("api-1"))
        .expect("folder")
        .holder = Some(IssueId::new("api", "1"));
    let effects = plan_two(&mut w);
    assert!(!effects.iter().any(|e| matches!(
        e,
        Effect::StartAgent {
            role: Role::Author,
            ..
        }
    )));
    let i = &w.issues[&gld()];
    assert_eq!(i.stage, Stage::Ready);
    assert_eq!(i.folder, None);
    assert!(i.other_folders.is_empty());
    assert_eq!(
        w.folders[&FolderId::new("ui-1")].holder,
        None,
        "holds nothing"
    );

    // api's folder comes free: the issue takes both, and the author starts in ui's.
    w.folders
        .get_mut(&FolderId::new("api-1"))
        .expect("folder")
        .holder = None;
    let effects = w.apply(&Event::FolderAdded {
        folder: FolderId::new("api-1"),
        project: pid("api"),
        machine: MachineId::new("morpork"),
        path: "/srv/api-1".into(),
    });
    assert_eq!(
        effects,
        vec![Effect::StartAgent {
            group: GroupId::new("watch"),
            role: Role::Author,
            issue: gld(),
            folder: Some(FolderId::new("ui-1")),
        }]
    );
    let i = &w.issues[&gld()];
    assert_eq!(i.other_folders, vec![FolderId::new("api-1")]);
    for f in ["ui-1", "api-1"] {
        assert_eq!(w.folders[&FolderId::new(f)].holder, Some(gld()));
    }
}

#[test]
fn the_authors_brief_lists_each_folder_and_asks_for_one_pull_request_per_repository() {
    let mut w = two_repos_planned();
    plan_two(&mut w);
    let launch = w
        .launch(
            &GroupId::new("watch"),
            Role::Author,
            &gld(),
            Some(&FolderId::new("ui-1")),
        )
        .expect("launch");
    let task = &launch.brief.task;
    assert!(task.contains("ui (group/ui): /srv/ui-1"), "{task}");
    assert!(task.contains("api (group/api): /srv/api-1"), "{task}");
    assert!(task.contains("same branch name"), "{task}");
    assert!(task.contains("one pull request in each"), "{task}");
    assert!(
        task.contains("ready guild#GLD-12 ui=<pull request number> api=<pull request number>"),
        "{task}"
    );
    assert_eq!(launch.brief.folder, Some("/srv/ui-1".into()));
}

#[test]
fn closing_the_issue_in_the_tracker_releases_every_folder() {
    let mut w = two_repos_planned();
    plan_two(&mut w);
    let effects = w.apply(&Event::IssueSynced {
        issue: gld(),
        title: "x".into(),
        milestone: None,
        depends_on: vec![],
        order: 1,
        closed: true,
    });
    let released: Vec<&Effect> = effects
        .iter()
        .filter(|e| matches!(e, Effect::ReleaseFolder { .. }))
        .collect();
    assert_eq!(released.len(), 2);
    assert!(w.folders.values().all(|f| f.holder.is_none()));
}

#[test]
fn the_reviewers_and_arbiters_briefs_name_every_pull_request_in_merge_order() {
    let mut w = two_repos_planned();
    // The plan's order is not the alphabet's: ui merges before api.
    plan_two(&mut w);
    ready_both(&mut w);
    for role in [Role::Reviewer, Role::Arbiter] {
        let task = w
            .launch(&GroupId::new("watch"), role, &gld(), None)
            .expect("launch")
            .brief
            .task;
        assert!(
            task.contains("- ui (group/ui): pull request 12\n- api (group/api): pull request 34\n"),
            "{role:?}: {task}"
        );
    }
}

#[test]
fn checks_pass_only_when_every_pull_request_passed_and_a_failure_names_its_repository() {
    let mut w = two_repos_planned();
    plan_two(&mut w);
    ready_both(&mut w);
    w.apply(&Event::ChecksFinished {
        issue: gld(),
        passed: false,
        failing: vec!["rust (api)".into()],
    });
    let i = &w.issues[&gld()];
    assert_eq!(i.checks_ok, Some(false));
    assert!(matches!(i.stage, Stage::Fixing { .. }));
    assert_eq!(i.fix, vec!["CI check rust (api) failed".to_string()]);
}

#[test]
fn a_conflict_in_one_of_several_pull_requests_names_its_repository() {
    let mut w = two_repos_planned();
    plan_two(&mut w);
    ready_both(&mut w);
    w.apply(&Event::PullConflicts {
        issue: gld(),
        repo: Some(pid("api")),
    });
    assert_eq!(
        w.issues[&gld()].fix,
        [
            "the pull request in api conflicts with the branch it merges into: rebase onto that \
             branch, then push your own branch with `git push --force-with-lease`"
        ]
    );
}

#[test]
fn merges_go_in_the_plans_order_and_deploy_waits_for_the_last() {
    let mut w = merging_two();
    let effects = merged(&mut w, "ui");
    assert_eq!(w.issues[&gld()].stage, Stage::Merging);
    assert_eq!(
        effects,
        vec![Effect::Merge { issue: gld() }],
        "next repository"
    );
    assert!(!effects.iter().any(|e| matches!(e, Effect::Deploy { .. })));
    assert!(
        w.folders.values().all(|f| f.holder.is_some()),
        "folders are held until the last merge"
    );
    let effects = merged(&mut w, "api");
    assert!(effects.contains(&Effect::Deploy { issue: gld() }));
    assert_eq!(
        effects
            .iter()
            .filter(|e| matches!(e, Effect::Deploy { .. }))
            .count(),
        1
    );
    assert_eq!(w.issues[&gld()].stage, Stage::Deploying);
    assert!(w.folders.values().all(|f| f.holder.is_none()));
}

#[test]
fn a_refused_merge_after_the_first_keeps_it_on_record_and_tells_the_author() {
    let mut w = merging_two();
    merged(&mut w, "ui");
    w.apply(&Event::MergeRefused {
        issue: gld(),
        reason: "pull request 34 conflicts with the branch it merges into".into(),
    });
    let i = &w.issues[&gld()];
    assert!(matches!(i.stage, Stage::Fixing { .. }));
    assert!(i.merged.contains(&pid("ui")));
    assert_eq!(
        i.fix,
        vec![
            "ui is merged, api cannot merge: pull request 34 conflicts with the branch it merges into"
                .to_string()
        ]
    );
    // The author fixes forward and names nothing new: the pull requests are on record.
    w.apply(&Event::PullRequestReady {
        issue: gld(),
        pull: None,
        pulls: Default::default(),
    });
    w.apply(&Event::ReviewFinished {
        issue: gld(),
        round: 1,
        verdict: Verdict::Approve,
    });
    let effects = w.apply(&Event::ChecksFinished {
        issue: gld(),
        passed: true,
        failing: vec![],
    });
    assert_eq!(effects, vec![Effect::Merge { issue: gld() }]);
    assert!(
        w.issues[&gld()].merged.contains(&pid("ui")),
        "never merged twice"
    );
}

#[test]
fn replaying_mid_sequence_resumes_at_the_second_repository() {
    let mut log = area_events();
    log.extend([
        area_issue("guild", "GLD-12"),
        Event::PlanWritten {
            issue: gld(),
            plan: two_repo_plan(),
        },
        Event::PullRequestReady {
            issue: gld(),
            pull: None,
            pulls: [
                (pid("ui"), "12".to_string()),
                (pid("api"), "34".to_string()),
            ]
            .into(),
        },
        Event::ReviewFinished {
            issue: gld(),
            round: 1,
            verdict: Verdict::Approve,
        },
        Event::ChecksFinished {
            issue: gld(),
            passed: true,
            failing: vec![],
        },
        Event::Merged {
            issue: gld(),
            repo: Some(pid("ui")),
        },
    ]);
    // A restarted daemon has the log only. It asks for the merge again, with ui's
    // on record, and keeps both folders.
    let (w, _) = World::replay(&log);
    let i = &w.issues[&gld()];
    assert_eq!(i.stage, Stage::Merging);
    assert_eq!(i.merged.iter().collect::<Vec<_>>(), [&pid("ui")]);
    assert_eq!(w.missing().start, vec![Effect::Merge { issue: gld() }]);
    assert!(w.folders.values().all(|f| f.holder == Some(gld())));
}

#[test]
fn a_failed_verify_clears_every_pull_request_and_merge() {
    let mut w = merging_two();
    merged(&mut w, "ui");
    merged(&mut w, "api");
    w.apply(&Event::Deployed { issue: gld() });
    w.apply(&Event::Verified {
        issue: gld(),
        passed: false,
        output: "no".into(),
    });
    let i = &w.issues[&gld()];
    assert!(i.pulls.is_empty() && i.merged.is_empty());
}

#[test]
fn an_old_single_pull_event_is_the_first_repositorys_and_an_old_merged_is_all_of_them() {
    let mut w = two_repos_planned();
    plan_two(&mut w);
    w.apply(&Event::PullRequestReady {
        issue: gld(),
        pull: Some("12".into()),
        pulls: Default::default(),
    });
    assert_eq!(
        w.issues[&gld()].pulls.get(&pid("ui")).map(String::as_str),
        Some("12")
    );
    w.apply(&Event::ReviewFinished {
        issue: gld(),
        round: 1,
        verdict: Verdict::Approve,
    });
    w.apply(&Event::ChecksFinished {
        issue: gld(),
        passed: true,
        failing: vec![],
    });
    w.apply(&Event::Merged {
        issue: gld(),
        repo: None,
    });
    assert_eq!(w.issues[&gld()].stage, Stage::Deploying);
}

#[test]
fn the_issue_text_lists_each_pull_request_with_its_repository_and_merge_state() {
    let mut w = merging_two();
    merged(&mut w, "ui");
    let text = crate::text::issue(&w, &gld()).expect("text");
    assert!(text.contains("Pull request: ui #12, merged"), "{text}");
    assert!(text.contains("Pull request: api #34\n"), "{text}");
    assert!(text.contains("Folders: ui-1, api-1"), "{text}");
}

#[test]
fn a_new_pull_request_in_a_merged_repository_merges_again_and_the_same_one_does_not() {
    let mut w = merging_two();
    merged(&mut w, "ui");
    w.apply(&Event::MergeRefused {
        issue: gld(),
        reason: "the main pipeline of ui failed".into(),
    });
    let ready = |w: &mut World, number: &str| {
        w.apply(&Event::PullRequestReady {
            issue: gld(),
            pull: None,
            pulls: [(pid("ui"), number.to_string())].into(),
        })
    };
    let mut same = w.clone();
    ready(&mut same, "12");
    assert!(same.issues[&gld()].merged.contains(&pid("ui")));
    ready(&mut w, "13");
    assert!(w.issues[&gld()].merged.is_empty());
}

#[test]
fn a_tracker_close_after_the_first_merge_does_not_drop_the_rest() {
    let mut w = merging_two();
    merged(&mut w, "ui");
    w.apply(&Event::MergeRefused {
        issue: gld(),
        reason: "conflicts".into(),
    });
    w.apply(&Event::IssueSynced {
        issue: gld(),
        title: "x".into(),
        milestone: None,
        depends_on: vec![],
        order: 1,
        closed: true,
    });
    assert!(matches!(w.issues[&gld()].stage, Stage::Fixing { .. }));
    assert!(w.folders.values().all(|f| f.holder.is_some()));
}

fn folder_on(id: &str, project: &str, machine: &str) -> Event {
    Event::FolderAdded {
        folder: FolderId::new(id),
        project: pid(project),
        machine: MachineId::new(machine),
        path: format!("/srv/{id}"),
    }
}

#[test]
fn an_issue_takes_every_folder_on_the_first_repositorys_machine_or_none() {
    // ui's folder is on morpork and api's only one on mac.
    let mut w = area_world();
    w.apply(&folder_on("api-1", "api", "mac"));
    w.apply(&area_issue("guild", "GLD-12"));
    plan_two(&mut w);
    assert_eq!(w.issues[&gld()].stage, Stage::Ready);
    assert!(w.folders.values().all(|f| f.holder.is_none()));

    // A ui folder beside api's: both come from that machine, though ui-1 comes first.
    let effects = w.apply(&folder_on("ui-2", "ui", "mac"));
    assert!(effects.contains(&Effect::StartAgent {
        group: GroupId::new("watch"),
        role: Role::Author,
        issue: gld(),
        folder: Some(FolderId::new("ui-2")),
    }));
    assert_eq!(w.issues[&gld()].other_folders, [FolderId::new("api-1")]);
    assert_eq!(w.folders[&FolderId::new("ui-1")].holder, None);
    let task = w
        .launch(
            &GroupId::new("watch"),
            Role::Author,
            &gld(),
            Some(&FolderId::new("ui-2")),
        )
        .expect("launch")
        .brief
        .task;
    assert!(
        task.contains("/srv/ui-2") && task.contains("/srv/api-1"),
        "{task}"
    );
}

#[path = "world_scope_tests.rs"]
mod scope;
