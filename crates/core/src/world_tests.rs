use super::*;
use crate::command::Command;
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
        replies: Vec::new(),
    };
    let block = |round| Event::ReviewFinished {
        issue: issue("1"),
        round,
        verdict: Verdict::Block {
            points: vec!["no test for an invalid document".into()],
        },
        failed: None,
        open: None,
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
        replies: Vec::new(),
    });
    let stale = Event::ReviewFinished {
        issue: issue("1"),
        round: 2,
        verdict: Verdict::Approve,
        failed: None,
        open: None,
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
            replies: Vec::new(),
        });
        let approve = Event::ReviewFinished {
            issue: issue("1"),
            round: 1,
            verdict: Verdict::Approve,
            failed: None,
            open: None,
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
        replies: Vec::new(),
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
            replies: Vec::new(),
        },
        Event::ReviewFinished {
            issue: issue("1"),
            round: 1,
            verdict: Verdict::Approve,
            failed: None,
            open: None,
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
            replies: Vec::new(),
        });
        for round in 1..=3 {
            w.apply(&Event::ReviewFinished {
                issue: issue("1"),
                round,
                verdict: Verdict::Block { points: vec![] },
                failed: None,
                open: None,
            });
            if round < 3 {
                w.apply(&Event::PullRequestReady {
                    issue: issue("1"),
                    pull: None,
                    pulls: Default::default(),
                    replies: Vec::new(),
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
                cost: None,
                point: None,
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
        replies: Vec::new(),
    });
    for round in 1..=3 {
        w.apply(&Event::ReviewFinished {
            issue: issue("1"),
            round,
            verdict: Verdict::Block { points: vec![] },
            failed: None,
            open: None,
        });
        if round < 3 {
            w.apply(&Event::PullRequestReady {
                issue: issue("1"),
                pull: None,
                pulls: Default::default(),
                replies: Vec::new(),
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
            cost: None,
            point: None,
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
            replies: Vec::new(),
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
        replies: Vec::new(),
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
                replies: Vec::new(),
            });
            w.apply(&Event::ReviewFinished {
                issue: issue("1"),
                round: 4,
                verdict: Verdict::Approve,
                failed: None,
                open: None,
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
            cost: None,
            point: None,
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
fn a_person_closing_an_issue_that_merged_nothing_finishes_it() {
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    assert!(!w.issues[&issue("1")].has_merged);
    let effects = w.apply(&Event::IssueSynced {
        issue: issue("1"),
        title: "issue 1".into(),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: 1,
        closed: true,
    });
    assert_eq!(
        effects,
        [
            Effect::StopAgent {
                agent: AgentId::new("author-1")
            },
            Effect::ReleaseFolder {
                folder: FolderId::new("f1")
            }
        ]
    );
    assert_eq!(stage(&w, "1"), Stage::Done);
    assert_eq!(w.folders[&FolderId::new("f1")].holder, None);
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
            replies: Vec::new(),
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
        runtime: None,
        model: None,
    }
}

fn ended(agent: &str) -> Event {
    Event::AgentChanged {
        agent: AgentId::new(agent),
        state: crate::model::AgentState::Ended,
        lost: false,
    }
}

/// Ends `agent`, takes the author `missing` asks for in its place, and returns
/// that author's task as it starts.
fn replace_author(w: &mut World, agent: &str, next: &str) -> String {
    w.apply(&ended(agent));
    assert_eq!(w.missing().start, vec![author("1", "f1")]);
    let launch = w
        .launch(
            &GroupId::new("watch"),
            Role::Author,
            &issue("1"),
            Some(&FolderId::new("f1")),
        )
        .expect("launch");
    assert_eq!(launch.agent, AgentId::new(next));
    w.apply(&started(next, Role::Author, "1"));
    launch.brief.task
}

/// Runs `temet-work ready` as `agent`, naming `pull` when there is one, with
/// `replies` to the points of a block.
fn run_ready(w: &mut World, agent: &str, pull: Option<&str>, replies: &[&str]) {
    let ready = Command::Ready {
        issue: issue("1"),
        pull: pull.map(Into::into),
        pulls: Default::default(),
        replies: replies.iter().map(ToString::to_string).collect(),
    };
    for event in w.events(Some(&AgentId::new(agent)), &ready).expect("ready") {
        w.apply(&event);
    }
}

/// A world where author-1 works on issue 1 of a project whose code is
/// ankh-morpork/watch, and has said pull request 57 is ready.
fn pull_57_ready() -> World {
    let mut w = World::default();
    w.apply(&Event::ProjectAdded {
        project: ProjectId::new("ankh"),
        tracker: "fake".into(),
        name: "city/ankh".into(),
        code_host: Some("fake".into()),
        repo: Some("ankh-morpork/watch".into()),
    });
    for event in [
        group(&["m1"], 1),
        folder("f1"),
        sync("1", "m1", 1, &[]),
        planned("1"),
        started("author-1", Role::Author, "1"),
    ] {
        w.apply(&event);
    }
    run_ready(&mut w, "author-1", Some("57"), &[]);
    w
}

/// What an author on issue 1 is told of its pull request 57.
const GIVEN_57: &str = "- ankh (ankh-morpork/watch): pull request 57\n\
                        Push your fixes to that pull request's branch. Do not open another pull \
                        request.\n";

#[test]
fn an_author_replacing_one_that_ended_is_given_the_pull_request_to_fix() {
    let mut w = pull_57_ready();
    w.apply(&Event::ChecksFinished {
        issue: issue("1"),
        passed: false,
        failing: vec!["tests".into()],
    });

    let task = replace_author(&mut w, "author-1", "author-2");
    assert!(task.contains(GIVEN_57), "{task}");
    assert!(task.contains("- CI check tests failed\n"), "{task}");
    assert!(!task.contains("open a pull request"), "{task}");
    assert!(
        crate::brief::before_stop(&task).ends_with("run: temet-work ready ankh#1"),
        "the ready line takes no pull request number: {task}"
    );
    run_ready(&mut w, "author-2", None, &[]);
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 1 });
    assert_eq!(w.issues[&issue("1")].pulls[&ProjectId::new("ankh")], "57");

    w.apply(&Event::PullConflicts {
        issue: issue("1"),
        repo: Some(ProjectId::new("ankh")),
    });
    let task = replace_author(&mut w, "author-2", "author-3");
    assert!(task.contains(GIVEN_57), "{task}");
    assert!(
        task.contains("push your own branch with `git push --force-with-lease`"),
        "{task}"
    );
    assert!(
        crate::brief::before_stop(&task).ends_with("run: temet-work ready ankh#1"),
        "{task}"
    );
    run_ready(&mut w, "author-3", None, &[]);
    assert_eq!(w.issues[&issue("1")].pulls[&ProjectId::new("ankh")], "57");
}

#[test]
fn an_author_replacing_one_after_a_block_replies_and_names_no_pull_request() {
    let mut w = pull_57_ready();
    w.apply(&verdict(
        1,
        Verdict::Block {
            points: vec!["no test for a stopped clock".into()],
        },
    ));
    let task = replace_author(&mut w, "author-1", "author-2");
    assert!(task.contains(GIVEN_57), "{task}");
    assert!(
        crate::brief::before_stop(&task)
            .ends_with("run: temet-work ready ankh#1 --reply \"<reply to point 1>\""),
        "{task}"
    );
    run_ready(&mut w, "author-2", None, &["tested it"]);
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 2 });
    assert_eq!(w.issues[&issue("1")].pulls[&ProjectId::new("ankh")], "57");
}

fn watch_12() -> IssueId {
    IssueId::new("watch", "12")
}

fn on_watch_12(event: Event) -> Event {
    match event {
        Event::AgentStarted {
            agent,
            group,
            role,
            machine,
            folder,
            secret_digest,
            runtime,
            model,
            ..
        } => Event::AgentStarted {
            agent,
            group,
            role,
            issue: Some(watch_12()),
            machine,
            folder,
            secret_digest,
            runtime,
            model,
        },
        other => other,
    }
}

/// A world where author-1 works on watch#12, in folder w1 of the project watch,
/// whose code is ankh-morpork/watch.
fn watch_12_working() -> World {
    let mut w = World::default();
    for event in [
        Event::ProjectAdded {
            project: pid("watch"),
            tracker: "fake".into(),
            name: "ankh-morpork/watch".into(),
            code_host: Some("fake".into()),
            repo: None,
        },
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![pid("watch")],
            areas: Vec::new(),
            milestones: Vec::new(),
            refining_at_once: 1,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
        Event::FolderAdded {
            folder: FolderId::new("w1"),
            project: pid("watch"),
            machine: MachineId::new("morpork"),
            path: "/srv/watch/w1".into(),
        },
        Event::IssueSynced {
            issue: watch_12(),
            title: "A night shift rota".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        },
        Event::PlanWritten {
            issue: watch_12(),
            plan: Plan::default(),
        },
        on_watch_12(started("author-1", Role::Author, "12")),
    ] {
        w.apply(&event);
    }
    assert_eq!(w.issues[&watch_12()].stage, Stage::Working);
    w
}

/// `temet-work ready` on watch#12 as `agent`, naming `pull` when there is one.
fn ready_12(w: &World, agent: &str, pull: Option<&str>) -> Result<Vec<Event>, String> {
    let ready = Command::Ready {
        issue: watch_12(),
        pull: pull.map(Into::into),
        pulls: Default::default(),
        replies: Vec::new(),
    };
    w.events(Some(&AgentId::new(agent)), &ready)
}

fn apply_all(w: &mut World, events: &[Event]) -> Vec<Effect> {
    events.iter().flat_map(|e| w.apply(e)).collect()
}

/// Approves round `round` of watch#12 and passes its checks.
fn approve_12(w: &mut World, round: u8) -> Vec<Effect> {
    let mut effects = w.apply(&Event::ReviewFinished {
        issue: watch_12(),
        round,
        verdict: Verdict::Approve,
        failed: None,
        open: None,
    });
    effects.extend(w.apply(&Event::ChecksFinished {
        issue: watch_12(),
        passed: true,
        failing: vec![],
    }));
    effects
}

/// Ends `agent` on watch#12 and returns the task of the author started in its
/// place, `next`.
fn next_author_12(w: &mut World, agent: &str, next: &str) -> String {
    w.apply(&ended(agent));
    let start = Effect::StartAgent {
        group: GroupId::new("watch"),
        role: Role::Author,
        issue: watch_12(),
        folder: Some(FolderId::new("w1")),
    };
    assert_eq!(w.missing().start, vec![start]);
    let launch = w
        .launch(
            &GroupId::new("watch"),
            Role::Author,
            &watch_12(),
            Some(&FolderId::new("w1")),
        )
        .expect("launch");
    assert_eq!(launch.agent, AgentId::new(next));
    w.apply(&on_watch_12(started(next, Role::Author, "12")));
    launch.brief.task
}

/// watch#12 merging pull request 57, on branch watch-12-night-shift, which
/// review approved and whose checks passed.
fn watch_12_merging_57() -> World {
    let mut w = watch_12_working();
    let ready = ready_12(&w, "author-1", Some("57")).expect("ready 57");
    apply_all(&mut w, &ready);
    w.apply(&Event::PullBranch {
        issue: watch_12(),
        repo: pid("watch"),
        number: "57".into(),
        branch: "watch-12-night-shift".into(),
    });
    let effects = approve_12(&mut w, 1);
    assert!(effects.contains(&Effect::Merge { issue: watch_12() }));
    w
}

fn closed_57() -> Event {
    Event::PullClosed {
        issue: watch_12(),
        repo: pid("watch"),
        number: "57".into(),
    }
}

#[test]
fn a_pull_request_a_person_closed_is_dropped_and_the_next_author_opens_another() {
    let mut w = watch_12_merging_57();
    w.apply(&closed_57());
    let i = &w.issues[&watch_12()];
    assert_eq!(i.stage, Stage::Fixing { round: 1 });
    assert_eq!(
        i.fix,
        [
            "pull request 57 was closed without merging: reopen it if its branch still exists, or \
          open a new one"
        ]
    );
    assert!(i.pulls.is_empty(), "57 is dropped: {:?}", i.pulls);

    let task = next_author_12(&mut w, "author-1", "author-2");
    for line in [
        "- watch (ankh-morpork/watch): pull request 57, branch watch-12-night-shift (closed \
         without merging)\n",
        "Pull request 57 was closed without merging: reopen it if its branch still exists, or \
         open a new pull request. If its branch still exists, a new one can start from it: git \
         fetch, then git switch watch-12-night-shift.\n",
        "- pull request 57 was closed without merging: reopen it if its branch still exists, or \
         open a new one\n",
    ] {
        assert!(task.contains(line), "missing {line:?} in:\n{task}");
    }
    for not in [
        "Push your fixes to that pull request's branch",
        "Do not open another pull request",
    ] {
        assert!(!task.contains(not), "{not:?} in:\n{task}");
    }
    assert!(
        crate::brief::before_stop(&task)
            .ends_with("temet-work ready watch#12 <pull request number>"),
        "{task}"
    );

    // 57 is not on record: a ready must name the one that replaces it.
    assert!(
        ready_12(&w, "author-2", None)
            .is_err_and(|e| e.contains("say which pull request: temet-work ready watch#12")),
    );
    let ready = ready_12(&w, "author-2", Some("58")).expect("ready 58");
    apply_all(&mut w, &ready);
    let i = &w.issues[&watch_12()];
    assert_eq!(i.stage, Stage::InReview { round: 1 });
    assert_eq!(i.pulls, [(pid("watch"), "58".to_string())].into());
    assert!(i.closed.is_empty() && i.branches.is_empty(), "{i:?}");
    let view = crate::view::View::new(&w, &BTreeMap::new());
    let pulls: Vec<&str> = view.issues[0]
        .pulls
        .iter()
        .map(|p| p.number.as_str())
        .collect();
    assert_eq!(pulls, ["58"]);

    // The loop follows 58: approved and passing, it merges.
    let effects = approve_12(&mut w, 1);
    assert!(
        effects.contains(&Effect::Merge { issue: watch_12() }),
        "{effects:?}"
    );
    assert_eq!(w.issues[&watch_12()].stage, Stage::Merging);

    // A later author is given 58 and its branch, and never hears of 57.
    w.apply(&Event::PullBranch {
        issue: watch_12(),
        repo: pid("watch"),
        number: "58".into(),
        branch: "watch-12-rota".into(),
    });
    w.apply(&Event::MergeRefused {
        issue: watch_12(),
        reason: "pull request 58 is not mergeable: the base branch moved".into(),
    });
    let task = next_author_12(&mut w, "author-2", "author-3");
    assert!(
        task.contains(
            "- watch (ankh-morpork/watch): pull request 58, branch watch-12-rota\n\
             Check out its branch first: git fetch, then git switch watch-12-rota. Push your fixes \
             to that pull request's branch. Do not open another pull request.\n"
        ),
        "{task}"
    );
    assert!(!task.contains("57"), "{task}");
    assert!(
        crate::brief::before_stop(&task).ends_with("run: temet-work ready watch#12"),
        "{task}"
    );
}

#[test]
fn an_author_told_of_a_closed_pull_request_is_asked_for_a_number() {
    let mut w = watch_12_merging_57();
    let effects = w.apply(&closed_57());
    let fixes: Vec<&Vec<String>> = effects
        .iter()
        .filter_map(|e| match e {
            Effect::RequestFix { issue, points } if *issue == watch_12() => Some(points),
            _ => None,
        })
        .collect();
    let [points] = fixes.as_slice() else {
        panic!("{effects:?}");
    };
    let i = &w.issues[&watch_12()];
    let told = crate::brief::fixes(i, &w.repos_of(i), points);
    assert_eq!(
        told,
        "Fix these, from the review or the checks:\n\
         - pull request 57 was closed without merging: reopen it if its branch still exists, or \
         open a new one\n\
         Then push and run: temet-work ready watch#12 <pull request number>"
    );
    // A closed pull request that is not the one on record changes nothing.
    let mut w = watch_12_merging_57();
    let before = w.issues[&watch_12()].clone();
    w.apply(&Event::PullClosed {
        issue: watch_12(),
        repo: pid("watch"),
        number: "56".into(),
    });
    assert_eq!(w.issues[&watch_12()], before);
}

/// The text the running reviewer of `issue` is sent for review round `round`,
/// as the runner makes it, and the brief of a reviewer newly launched on it.
fn told_a_reviewer(w: &World, issue: &IssueId, round: u8) -> [String; 2] {
    let i = &w.issues[issue];
    let again = crate::brief::next_round(i, round, &w.pulled(i));
    [again, launched_task(w, Role::Reviewer, issue)]
}

fn launched_task(w: &World, role: Role, issue: &IssueId) -> String {
    w.launch(&GroupId::new("watch"), role, issue, None)
        .expect("launch")
        .brief
        .task
}

#[test]
fn a_reviewer_is_given_the_pull_request_that_replaced_a_closed_one() {
    let mut w = watch_12_merging_57();
    w.apply(&closed_57());
    let ready = ready_12(&w, "author-1", Some("58")).expect("ready 58");
    let effects = apply_all(&mut w, &ready);
    assert!(
        effects.contains(&Effect::StartReview {
            issue: watch_12(),
            round: 1
        }),
        "{effects:?}"
    );

    let listed = "Review the pull request listed here. A pull request not listed is not part of \
                  the change:\n- watch (ankh-morpork/watch): pull request 58\n";
    let [again, brief] = told_a_reviewer(&w, &watch_12(), 1);
    let arbiter = launched_task(&w, Role::Arbiter, &watch_12());
    for text in [again, brief, arbiter] {
        assert!(text.contains(listed), "missing {listed:?} in:\n{text}");
        assert!(!text.contains("57"), "{text}");
    }
}

#[test]
fn a_reviewer_of_several_repositories_is_given_the_replacement_of_a_closed_one() {
    let mut w = merging_two();
    merged(&mut w, "ui");
    w.apply(&Event::PullClosed {
        issue: gld(),
        repo: pid("api"),
        number: "34".into(),
    });
    w.apply(&Event::AgentStarted {
        agent: AgentId::new("author-1"),
        group: GroupId::new("watch"),
        role: Role::Author,
        issue: Some(gld()),
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    });
    let ready = Command::Ready {
        issue: gld(),
        pull: None,
        pulls: [(pid("api"), "35".to_string())].into(),
        replies: Vec::new(),
    };
    let ready = w
        .events(Some(&AgentId::new("author-1")), &ready)
        .expect("ready");
    apply_all(&mut w, &ready);
    let Stage::InReview { round } = w.issues[&gld()].stage else {
        panic!("{:?}", w.issues[&gld()].stage);
    };

    let listed = "It changes several repositories; review every pull request listed here. A pull \
                  request not listed is not part of the change:\n\
                  - ui (group/ui): pull request 12 (merged already)\n\
                  - api (group/api): pull request 35\n";
    for text in told_a_reviewer(&w, &gld(), round) {
        assert!(text.contains(listed), "missing {listed:?} in:\n{text}");
        assert!(!text.contains("34"), "{text}");
    }
}

#[test]
fn a_pull_requests_branch_is_kept_by_repository() {
    let mut w = merging_two();
    let branch = |w: &mut World, repo: &str, number: &str, branch: &str| {
        w.apply(&Event::PullBranch {
            issue: gld(),
            repo: pid(repo),
            number: number.into(),
            branch: branch.into(),
        });
    };
    branch(&mut w, "ui", "12", "gld-12-locks");
    branch(&mut w, "api", "34", "gld-12-keys");
    // One for a pull request not on record is history.
    branch(&mut w, "api", "33", "gld-12-old");
    let i = &w.issues[&gld()];
    assert_eq!(
        i.branches,
        [
            (pid("ui"), "gld-12-locks".to_string()),
            (pid("api"), "gld-12-keys".to_string()),
        ]
        .into()
    );
    // A later read that finds another branch replaces it.
    branch(&mut w, "api", "34", "gld-12-keys-2");
    assert_eq!(w.issues[&gld()].branches[&pid("api")], "gld-12-keys-2");

    // An issue from an older log has neither branches nor closed pull requests.
    let mut old = serde_json::to_value(&w.issues[&gld()]).expect("serializes");
    let fields = old.as_object_mut().expect("an object");
    fields.remove("branches");
    fields.remove("closed");
    let read: crate::model::Issue = serde_json::from_value(old).expect("reads as before");
    assert!(read.branches.is_empty() && read.closed.is_empty());
    assert_eq!(read.pulls, w.issues[&gld()].pulls);
}

#[test]
fn a_conflict_names_the_branch_to_push() {
    let mut w = watch_12_working();
    let ready = ready_12(&w, "author-1", Some("57")).expect("ready 57");
    apply_all(&mut w, &ready);
    w.apply(&Event::PullBranch {
        issue: watch_12(),
        repo: pid("watch"),
        number: "57".into(),
        branch: "watch-12-night-shift".into(),
    });
    for repo in [Some(pid("watch")), None] {
        let mut w = w.clone();
        w.apply(&Event::PullConflicts {
            issue: watch_12(),
            repo,
        });
        assert_eq!(
            w.issues[&watch_12()].fix,
            [
                "the pull request conflicts with the branch it merges into: rebase onto that \
                 branch, then push watch-12-night-shift with `git push --force-with-lease`"
            ]
        );
    }
}

#[test]
fn a_closed_pull_request_in_one_repository_is_reopened_or_replaced_there() {
    let mut w = merging_two();
    for (repo, number) in [("ui", "12"), ("api", "34")] {
        w.apply(&Event::PullBranch {
            issue: gld(),
            repo: pid(repo),
            number: number.into(),
            branch: "gld-12-locks".into(),
        });
    }
    merged(&mut w, "ui");
    w.apply(&Event::PullClosed {
        issue: gld(),
        repo: pid("api"),
        number: "34".into(),
    });
    let i = &w.issues[&gld()];
    assert!(matches!(i.stage, Stage::Fixing { .. }));
    assert_eq!(
        i.pulls,
        [(pid("ui"), "12".to_string())].into(),
        "only api dropped"
    );
    assert!(i.merged.contains(&pid("ui")), "ui stays merged");
    assert_eq!(
        i.fix,
        [
            "ui is merged; pull request 34 in api was closed without merging: reopen it if its \
             branch still exists, or open a new one there"
        ]
    );

    w.apply(&Event::AgentStarted {
        agent: AgentId::new("author-1"),
        group: GroupId::new("watch"),
        role: Role::Author,
        issue: Some(gld()),
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    });
    let task = w
        .launch(
            &GroupId::new("watch"),
            Role::Author,
            &gld(),
            Some(&FolderId::new("ui-1")),
        )
        .expect("launch")
        .brief
        .task;
    assert!(
        task.contains(
            "- ui (group/ui): pull request 12, branch gld-12-locks (merged already)\n\
             - api (group/api): pull request 34, branch gld-12-locks (closed without merging)\n\
             Do not open another pull request in ui.\n\
             Pull request 34 in api was closed without merging: reopen it if its branch still \
             exists, or open a new pull request in api. If its branch still exists, a new one can \
             start from it: in the folder of api, git fetch, then git switch gld-12-locks.\n"
        ),
        "{task}"
    );
    assert!(!task.contains("Push your fixes"), "{task}");
    assert!(
        crate::brief::before_stop(&task)
            .ends_with("run: temet-work ready guild#GLD-12 api=<pull request number>"),
        "{task}"
    );

    // A ready with no number for api is refused; one with a number is followed.
    let ready = |pulls: &[(&str, &str)]| Command::Ready {
        issue: gld(),
        pull: None,
        pulls: pulls.iter().map(|(r, n)| (pid(r), n.to_string())).collect(),
        replies: Vec::new(),
    };
    let author = AgentId::new("author-1");
    for refused in [ready(&[]), ready(&[("ui", "12")])] {
        assert!(
            w.events(Some(&author), &refused)
                .is_err_and(|e| e.contains("temet-work ready guild#GLD-12 api=<number>")),
        );
    }
    let events = w
        .events(Some(&author), &ready(&[("api", "35")]))
        .expect("ready");
    apply_all(&mut w, &events);
    let effects = w.apply(&Event::ReviewFinished {
        issue: gld(),
        round: 1,
        verdict: Verdict::Approve,
        failed: None,
        open: None,
    });
    assert!(effects.is_empty(), "{effects:?}");
    let effects = w.apply(&Event::ChecksFinished {
        issue: gld(),
        passed: true,
        failing: vec![],
    });
    assert_eq!(effects, vec![Effect::Merge { issue: gld() }]);
    let i = &w.issues[&gld()];
    assert_eq!(i.stage, Stage::Merging, "api is not skipped");
    assert_eq!(i.pulls[&pid("api")], "35");
    assert!(i.merged.contains(&pid("ui")) && !i.merged.contains(&pid("api")));
    // Deploying only once api's new pull request merged.
    let effects = merged(&mut w, "api");
    assert!(
        effects.contains(&Effect::Deploy { issue: gld() }),
        "{effects:?}"
    );
    assert_eq!(w.issues[&gld()].stage, Stage::Deploying);
}

/// What the author of watch#12 is told when a person closed pull request 57.
const CLOSED_57: &str = "pull request 57 was closed without merging: reopen it if its branch \
                         still exists, or open a new one";

/// watch#12 in review round 1 with pull request 57, on branch
/// watch-12-night-shift, and its reviewer-1 running, told of the round.
fn watch_12_reviewing_57() -> World {
    let mut w = watch_12_working();
    let ready = ready_12(&w, "author-1", Some("57")).expect("ready 57");
    apply_all(&mut w, &ready);
    w.apply(&Event::PullBranch {
        issue: watch_12(),
        repo: pid("watch"),
        number: "57".into(),
        branch: "watch-12-night-shift".into(),
    });
    w.apply(&on_watch_12(started("reviewer-1", Role::Reviewer, "12")));
    let i = &w.issues[&watch_12()];
    assert_eq!(i.stage, Stage::InReview { round: 1 });
    assert!(i.told);
    w
}

/// Pull request 57, with its branch, as the closed one on record.
fn closed_57_on_record() -> BTreeMap<ProjectId, crate::model::ClosedPull> {
    [(
        pid("watch"),
        crate::model::ClosedPull {
            number: "57".into(),
            branch: Some("watch-12-night-shift".into()),
        },
    )]
    .into()
}

fn fix_12(points: &[&str]) -> Effect {
    Effect::RequestFix {
        issue: watch_12(),
        points: points.iter().map(ToString::to_string).collect(),
    }
}

/// The review's verdict on watch#12: a block with no points.
fn block_12(round: u8) -> Event {
    Event::ReviewFinished {
        issue: watch_12(),
        round,
        verdict: Verdict::Block { points: vec![] },
        failed: None,
        open: None,
    }
}

fn failed_rust_12() -> Event {
    Event::ChecksFinished {
        issue: watch_12(),
        passed: false,
        failing: vec!["rust".into()],
    }
}

/// The task of a `role` started on watch#12 now, and the text of `temet-work
/// issue watch#12`, which the task sends the agent to. With one pull request the
/// brief lists it with its number.
fn reader_of_12(w: &World, role: Role) -> (String, String) {
    let task = w
        .launch(&GroupId::new("watch"), role, &watch_12(), None)
        .expect("launch")
        .brief
        .task;
    let text = crate::text::issue(w, &watch_12(), &Err("not read here".into())).expect("text");
    (task, text)
}

/// Whether a reader started now is sent to 58 and never hears of 57.
fn sent_to_58(w: &World, role: Role) {
    let (task, text) = reader_of_12(w, role);
    assert!(task.contains("temet-work issue watch#12"), "{task}");
    assert!(!task.contains("57"), "{task}");
    assert!(text.contains("Pull request: watch #58\n"), "{text}");
    assert!(!text.contains("57"), "{text}");
}

#[test]
fn a_pull_request_closed_in_review_goes_back_to_its_author_and_stops_the_reviewer() {
    let mut w = watch_12_reviewing_57();
    // One that is not on record changes nothing.
    let before = w.issues[&watch_12()].clone();
    let other = Event::PullClosed {
        issue: watch_12(),
        repo: pid("watch"),
        number: "56".into(),
    };
    assert_eq!(w.apply(&other), vec![]);
    assert_eq!(w.issues[&watch_12()], before);

    let effects = w.apply(&closed_57());
    assert_eq!(effects, vec![stop("reviewer-1"), fix_12(&[CLOSED_57])]);
    let i = &w.issues[&watch_12()];
    assert_eq!(i.stage, Stage::Fixing { round: 1 });
    assert_eq!(i.round, 1);
    assert!(i.fix_keeps_round, "no review round is used up");
    assert_eq!(i.fix, [CLOSED_57]);
    assert!(i.pulls.is_empty() && i.branches.is_empty(), "{i:?}");
    assert_eq!(i.closed, closed_57_on_record());
}

#[test]
fn a_pull_request_closed_with_the_arbiter_goes_back_to_its_author_and_stops_the_arbiter() {
    let mut w = watch_12_reviewing_57();
    for round in 1..=3 {
        w.apply(&block_12(round));
        if round < 3 {
            let ready = ready_12(&w, "author-1", None).expect("ready");
            apply_all(&mut w, &ready);
        }
    }
    assert_eq!(w.issues[&watch_12()].stage, Stage::Arbitration { round: 3 });
    w.apply(&on_watch_12(started("arbiter-1", Role::Arbiter, "12")));

    let effects = w.apply(&closed_57());
    assert_eq!(
        effects,
        vec![stop("reviewer-1"), stop("arbiter-1"), fix_12(&[CLOSED_57])]
    );
    let i = &w.issues[&watch_12()];
    assert_eq!(i.stage, Stage::Fixing { round: 3 }, "the round is kept");
    assert_eq!(i.fix, [CLOSED_57]);
    assert!(i.pulls.is_empty());
    assert_eq!(i.closed, closed_57_on_record());

    // Once both have ended and 58 is named, a final-round block starts a fresh
    // arbiter, which is sent to 58.
    w.apply(&ended("reviewer-1"));
    w.apply(&ended("arbiter-1"));
    let ready = ready_12(&w, "author-1", Some("58")).expect("ready 58");
    apply_all(&mut w, &ready);
    assert_eq!(w.issues[&watch_12()].stage, Stage::InReview { round: 3 });
    let effects = w.apply(&block_12(3));
    assert!(starts_an_arbiter(&effects), "{effects:?}");
    sent_to_58(&w, Role::Arbiter);
}

#[test]
fn a_pull_request_closed_while_fixing_adds_its_point_and_keeps_the_stage() {
    // Back with its author for a failed check, which keeps the round, or for a
    // block, which uses one up.
    for (back, keeps) in [(failed_rust_12(), true), (block_12(1), false)] {
        let mut w = watch_12_reviewing_57();
        w.apply(&back);
        w.apply(&Event::Told { issue: watch_12() });
        let had = w.issues[&watch_12()].fix.clone();
        assert_eq!(w.issues[&watch_12()].fix_keeps_round, keeps);

        let effects = w.apply(&closed_57());
        let mut points: Vec<&str> = had.iter().map(String::as_str).collect();
        points.push(CLOSED_57);
        assert_eq!(effects, vec![stop("reviewer-1"), fix_12(&points)]);
        let i = &w.issues[&watch_12()];
        assert_eq!(i.stage, Stage::Fixing { round: 1 });
        assert_eq!(i.round, 1);
        assert_eq!(i.fix_keeps_round, keeps);
        assert_eq!(i.fix, points);
        assert!(!i.told, "the author hears every point again");
        assert!(i.pulls.is_empty());
        assert_eq!(i.closed, closed_57_on_record());
    }
}

/// watch#12 back with its author in round 3 with pull request 57, on an
/// arbiter's fix ruling that a person has overturned.
fn watch_12_overturned_on_57() -> World {
    let mut w = watch_12_reviewing_57();
    for round in 1..=3 {
        w.apply(&block_12(round));
        if round < 3 {
            let ready = ready_12(&w, "author-1", None).expect("ready");
            apply_all(&mut w, &ready);
        }
    }
    let mut sent_back = ruling(false);
    if let Event::DecisionMade { decision } = &mut sent_back {
        decision.issue = watch_12();
    }
    w.apply(&sent_back);
    w.apply(&overturned(0));
    let i = &w.issues[&watch_12()];
    assert_eq!(i.stage, Stage::Fixing { round: 3 });
    assert!(i.review_waived);
    w
}

/// That a ready naming `pull` on watch#12 asks for a review.
fn ready_asks_for_a_review(w: &mut World, pull: &str) {
    let ready = ready_12(w, "author-1", Some(pull)).expect("ready");
    let effects = apply_all(w, &ready);
    let i = &w.issues[&watch_12()];
    let Stage::InReview { round } = i.stage else {
        panic!("{i:?}");
    };
    let start = Effect::StartReview {
        issue: watch_12(),
        round,
    };
    assert!(effects.contains(&start), "{effects:?}");
    assert!(!i.review_ok && !i.review_waived, "{i:?}");
}

#[test]
fn a_closed_pull_request_ends_a_persons_review_waiver() {
    let fixing = watch_12_overturned_on_57();
    let mut in_review = fixing.clone();
    let ready = ready_12(&in_review, "author-1", None).expect("ready");
    apply_all(&mut in_review, &ready);
    assert_eq!(
        in_review.issues[&watch_12()].stage,
        Stage::InReview { round: 4 }
    );
    let mut merging = in_review.clone();
    merging.apply(&Event::ChecksFinished {
        issue: watch_12(),
        passed: true,
        failing: vec![],
    });
    assert_eq!(merging.issues[&watch_12()].stage, Stage::Merging);

    for mut held in [fixing, in_review, merging] {
        // One that is not on record leaves the waiver.
        held.apply(&Event::PullClosed {
            issue: watch_12(),
            repo: pid("watch"),
            number: "56".into(),
        });
        assert!(held.issues[&watch_12()].review_waived);

        held.apply(&closed_57());
        let i = &held.issues[&watch_12()];
        assert!(!i.review_ok && !i.review_waived, "{i:?}");
        // A replacement is reviewed, and so is 57 reopened: its branch can
        // have taken new commits.
        for pull in ["58", "57"] {
            ready_asks_for_a_review(&mut held.clone(), pull);
        }
    }
}

#[test]
fn a_replacement_pull_request_ends_a_persons_review_waiver() {
    // 57 is still on record, never found closed.
    ready_asks_for_a_review(&mut watch_12_overturned_on_57(), "58");

    // The fix the person approved, in 57, merges once the checks pass.
    for pull in [Some("57"), None] {
        let mut w = watch_12_overturned_on_57();
        let ready = ready_12(&w, "author-1", pull).expect("ready");
        assert_eq!(apply_all(&mut w, &ready), vec![], "{pull:?}");
        assert!(w.issues[&watch_12()].review_waived, "{pull:?}");
        let effects = w.apply(&Event::ChecksFinished {
            issue: watch_12(),
            passed: true,
            failing: vec![],
        });
        assert_eq!(
            effects,
            vec![Effect::Merge { issue: watch_12() }],
            "{pull:?}"
        );
    }
}

#[test]
fn a_reviewer_stopped_for_a_closed_pull_request_does_not_count_against_starts() {
    let one_start = |w: &mut World| {
        w.apply(&Event::SettingSet {
            scope: crate::settings::Scope::Global,
            role: Some(Role::Reviewer),
            key: "starts".into(),
            value: crate::settings::Value::Number(1),
            by: "person".into(),
        });
    };
    // With one start, a reviewer that ends while it is waited on is the last.
    let mut w = watch_12_reviewing_57();
    one_start(&mut w);
    w.apply(&ended("reviewer-1"));
    assert_eq!(w.missing().start, vec![]);
    assert_eq!(w.missing().given_up.len(), 1);

    // Closed in review, with the arbiter, after a failed check, or while merging:
    // the reviewer and the arbiter are stopped, and the issue gets a new reviewer
    // once ready names 58. Outside arbitration the arbiter is one whose stop
    // failed, so it still runs.
    let in_review = |_: &mut World| {};
    let with_the_arbiter = |w: &mut World| {
        for round in 1..=3 {
            w.apply(&block_12(round));
            if round < 3 {
                let ready = ready_12(w, "author-1", None).expect("ready");
                apply_all(w, &ready);
            }
        }
        assert_eq!(w.issues[&watch_12()].stage, Stage::Arbitration { round: 3 });
    };
    let fixing = |w: &mut World| {
        w.apply(&failed_rust_12());
        assert_eq!(w.issues[&watch_12()].stage, Stage::Fixing { round: 1 });
    };
    let merging = |w: &mut World| {
        approve_12(w, 1);
        assert_eq!(w.issues[&watch_12()].stage, Stage::Merging);
    };
    let stages: [&dyn Fn(&mut World); 4] = [&in_review, &with_the_arbiter, &fixing, &merging];
    for to in stages {
        let mut w = watch_12_reviewing_57();
        one_start(&mut w);
        to(&mut w);
        // The round goes on: 3 after arbitration, else 1.
        let (stage, round) = (w.issues[&watch_12()].stage, w.issues[&watch_12()].round);
        w.apply(&on_watch_12(started("arbiter-1", Role::Arbiter, "12")));
        let effects = w.apply(&closed_57());
        for agent in ["reviewer-1", "arbiter-1"] {
            assert!(
                effects.contains(&stop(agent)),
                "{agent} in {stage:?}: {effects:?}"
            );
        }
        w.apply(&ended("reviewer-1"));
        w.apply(&ended("arbiter-1"));

        let ready = ready_12(&w, "author-1", Some("58")).expect("ready 58");
        apply_all(&mut w, &ready);
        assert_eq!(w.issues[&watch_12()].stage, Stage::InReview { round });
        let missing = w.missing();
        assert_eq!(missing.given_up, Vec::<String>::new(), "{stage:?}");
        assert_eq!(
            missing.start,
            vec![Effect::StartReview {
                issue: watch_12(),
                round,
            }],
            "{stage:?}"
        );
        sent_to_58(&w, Role::Reviewer);
    }
}

#[test]
fn a_pull_request_closed_after_ci_failed_is_dropped_and_ready_58_is_followed() {
    let mut w = watch_12_reviewing_57();
    w.apply(&failed_rust_12());
    assert_eq!(w.issues[&watch_12()].stage, Stage::Fixing { round: 1 });
    // A person closes 57. No one has heard, and 57 is still on record.
    let ready = ready_12(&w, "author-1", None).expect("57 is on record");
    apply_all(&mut w, &ready);
    assert_eq!(w.issues[&watch_12()].stage, Stage::InReview { round: 1 });

    // The code host's next read finds 57 closed.
    let effects = w.apply(&closed_57());
    let fixes: Vec<&Vec<String>> = effects
        .iter()
        .filter_map(|e| match e {
            Effect::RequestFix { points, .. } => Some(points),
            _ => None,
        })
        .collect();
    assert_eq!(
        fixes,
        [&vec![CLOSED_57.to_string()]],
        "not the failed check"
    );
    let i = &w.issues[&watch_12()];
    assert_eq!(i.stage, Stage::Fixing { round: 1 });
    let told = crate::brief::fixes(i, &w.repos_of(i), fixes[0]);
    assert!(
        told.ends_with("temet-work ready watch#12 <pull request number>"),
        "{told}"
    );
    assert!(
        ready_12(&w, "author-1", None)
            .is_err_and(|e| e.contains("say which pull request: temet-work ready watch#12")),
    );

    let ready = ready_12(&w, "author-1", Some("58")).expect("ready 58");
    apply_all(&mut w, &ready);
    let i = &w.issues[&watch_12()];
    assert_eq!(i.stage, Stage::InReview { round: 1 });
    assert_eq!(i.pulls, [(pid("watch"), "58".to_string())].into());
    assert!(i.closed.is_empty(), "{:?}", i.closed);
    let effects = approve_12(&mut w, 1);
    assert!(
        effects.contains(&Effect::Merge { issue: watch_12() }),
        "{effects:?}"
    );
    assert_eq!(w.issues[&watch_12()].stage, Stage::Merging);
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

/// Which ends of an issue's agents count toward its `starts` limit.
mod lost_to_a_shell {
    use super::*;

    /// An end of `agent` whose program was lost.
    fn lost(agent: &str) -> Event {
        Event::AgentChanged {
            agent: AgentId::new(agent),
            state: crate::model::AgentState::Ended,
            lost: true,
        }
    }

    /// A world whose issue 1 is worked on by author-1, which it waits on, after
    /// each of `ends` in turn has ended the author running and the one `missing`
    /// asks for in its place has started: author-2, author-3 and so on. The last
    /// end is followed by no start.
    fn authors_ending(ends: &[fn(&str) -> Event]) -> World {
        let mut w = working_on_one();
        w.apply(&started("author-1", Role::Author, "1"));
        for (n, end) in (1..).zip(ends) {
            if n > 1 {
                assert_eq!(w.missing().start, vec![author("1", "f1")]);
                w.apply(&started(&format!("author-{n}"), Role::Author, "1"));
            }
            let agent = AgentId::new(format!("author-{n}"));
            assert!(w.waiting_on(&w.agents[&agent]), "{agent}");
            w.apply(&end(&agent.0));
        }
        w
    }

    /// What `missing` says once three authors of issue 1 have given up on it.
    const THREE_AUTHORS: &str = "ankh#1: 3 authors ended before it moved on; no more are started";

    #[test]
    fn an_author_lost_to_a_shell_three_times_still_gets_a_fourth_start() {
        let w = authors_ending(&[lost, lost, lost]);
        for n in 1..=3 {
            let a = &w.agents[&AgentId::new(format!("author-{n}"))];
            assert_eq!(a.state, crate::model::AgentState::Ended);
            assert!(!a.ended_waited_on, "{}", a.id);
        }
        let missing = w.missing();
        assert_eq!(missing.start, vec![author("1", "f1")]);
        assert_eq!(missing.given_up, Vec::<String>::new());
    }

    #[test]
    fn authors_that_end_by_themselves_still_use_up_the_starts() {
        let missing = authors_ending(&[ended, ended, ended]).missing();
        assert_eq!(missing.start, vec![]);
        assert_eq!(missing.given_up, [THREE_AUTHORS]);
    }

    #[test]
    fn only_the_authors_not_lost_to_a_shell_use_up_the_starts() {
        let missing = authors_ending(&[lost, ended, lost, ended]).missing();
        assert_eq!(missing.start, vec![author("1", "f1")], "a fifth");
        assert_eq!(missing.given_up, Vec::<String>::new());

        let missing = authors_ending(&[lost, ended, lost, ended, ended]).missing();
        assert_eq!(missing.start, vec![]);
        assert_eq!(missing.given_up, [THREE_AUTHORS]);
    }

    /// What `missing` says once the programs of `n` authors of issue 1 were
    /// lost while it waited on them.
    fn lost_authors(n: u32) -> String {
        format!(
            "ankh#1: the programs of {n} authors were lost before it moved on; no more are started"
        )
    }

    /// Starts an author on issue 1 and loses its program, `ends` times, as
    /// when an out-of-memory kill ends every run at the same point. After each
    /// end it returns what `missing` says, in order.
    fn lost_one_at_a_time(w: &mut World, ends: u32) -> Vec<Missing> {
        (1..=ends)
            .map(|n| {
                let agent = format!("author-{n}");
                w.apply(&started(&agent, Role::Author, "1"));
                assert!(w.waiting_on(&w.agents[&AgentId::new(&agent)]), "{agent}");
                w.apply(&lost(&agent));
                w.missing()
            })
            .collect()
    }

    #[test]
    fn an_author_lost_on_every_start_is_given_up_after_three_times_starts() {
        let mut w = working_on_one();
        let after = lost_one_at_a_time(&mut w, 9);
        for n in [3, 8] {
            let missing = &after[n - 1];
            assert_eq!(missing.start, vec![author("1", "f1")], "after {n} lost");
            assert_eq!(missing.given_up, Vec::<String>::new(), "after {n} lost");
        }
        let missing = &after[8];
        assert_eq!(missing.start, vec![], "after 9 lost");
        assert_eq!(missing.given_up, [lost_authors(9)]);
        assert!(
            w.agents.values().all(|a| !a.ended_waited_on),
            "none used up a start"
        );
    }

    #[test]
    fn the_lost_limit_follows_the_starts_the_issue_sets_for_its_role() {
        let mut w = working_on_one();
        w.apply(&Event::SettingSet {
            scope: Scope::Task(issue("1")),
            role: Some(Role::Author),
            key: "starts".into(),
            value: Value::Number(2),
            by: "person".into(),
        });
        let after = lost_one_at_a_time(&mut w, 6);
        let missing = &after[4];
        assert_eq!(missing.start, vec![author("1", "f1")], "after 5 lost");
        assert_eq!(missing.given_up, Vec::<String>::new(), "after 5 lost");
        let missing = &after[5];
        assert_eq!(missing.start, vec![], "after 6 lost");
        assert_eq!(missing.given_up, [lost_authors(6)]);
    }

    #[test]
    fn authors_lost_after_their_issue_moved_on_add_nothing_to_the_lost_limit() {
        let mut w = working_on_one();
        for n in 1..=9 {
            w.apply(&started(&format!("author-{n}"), Role::Author, "1"));
        }
        w.apply(&Event::PullRequestReady {
            issue: issue("1"),
            pull: Some("57".into()),
            pulls: Default::default(),
            replies: Vec::new(),
        });
        // In review the issue waits on its reviewer, not on its authors.
        for n in 1..=9 {
            w.apply(&lost(&format!("author-{n}")));
        }
        assert!(w.agents.values().all(|a| !a.lost_waited_on));
        // The reviewer blocks, and the issue waits on an author again.
        w.apply(&started("reviewer-1", Role::Reviewer, "1"));
        w.apply(&Event::ReviewFinished {
            issue: issue("1"),
            round: 1,
            verdict: Verdict::Block {
                points: vec!["no test for a stopped clock".into()],
            },
            failed: None,
            open: None,
        });
        assert!(matches!(stage(&w, "1"), Stage::Fixing { .. }));
        let missing = w.missing();
        assert_eq!(missing.given_up, Vec::<String>::new());
        assert_eq!(missing.start, vec![author("1", "f1")]);
    }
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
fn closing_an_issue_mid_work_stops_its_agents_and_frees_its_folder_once_they_end() {
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
    assert_eq!(effects.len(), 2, "{effects:?}");
    assert_eq!(stage(&w, "2"), Stage::Ready, "author-1 may still change f1");
    assert_eq!(
        w.apply(&ended("author-1")),
        [author("2", "f1")],
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
        replies: Vec::new(),
    });
    w.apply(&started("reviewer-1", Role::Reviewer, "1"));
    let points = vec!["no test for a stopped clock".to_string()];
    w.apply(&Event::ReviewFinished {
        issue: issue("1"),
        round: 1,
        verdict: Verdict::Block {
            points: points.clone(),
        },
        failed: None,
        open: None,
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
        replies: Vec::new(),
    });
    w.apply(&Event::ReviewFinished {
        issue: issue("1"),
        round: 1,
        verdict: Verdict::Block {
            points: vec!["no test for a stopped clock".into()],
        },
        failed: None,
        open: None,
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
        replies: Vec::new(),
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
        replies: Vec::new(),
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
        failed: None,
        open: None,
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
    let i = &w.issues[&issue("1")];
    let told = crate::brief::fixes(i, &w.repos_of(i), points);
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
        failed: None,
        open: None,
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
        replies: Vec::new(),
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
        replies: Vec::new(),
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
fn a_repeated_round_1_lost_to_a_restart_is_sent_to_its_running_reviewer_again() {
    let approved = || verdict(1, Verdict::Approve);
    let conflict = Event::PullConflicts {
        issue: issue("1"),
        repo: None,
    };
    let refused = Event::MergeRefused {
        issue: issue("1"),
        reason: "it conflicts with main".into(),
    };
    let paths = [
        ("approved, then CI failed", vec![approved(), checks(false)]),
        ("still reviewing, then CI failed", vec![checks(false)]),
        ("still reviewing, then a conflict", vec![conflict]),
        (
            "approved and checks passed, then the merge refused",
            vec![approved(), checks(true), refused],
        ),
    ];
    let reviewer = AgentId::new("reviewer-1");
    let approve = Command::Review {
        issue: issue("1"),
        verdict: Verdict::Approve,
        failed: Vec::new(),
        open: Vec::new(),
    };
    for (path, events) in paths {
        let mut w = in_review();
        for event in &events {
            w.apply(event);
        }
        // The effect `ready` returns is dropped, as a restart drops the queued message.
        w.apply(&pull_ready());
        assert_eq!(stage(&w, "1"), Stage::InReview { round: 1 }, "{path}");
        assert!(!w.issues[&issue("1")].told, "{path}");
        let running = w.running(&issue("1"), Role::Reviewer).map(|a| &a.id);
        assert_eq!(running, Some(&reviewer), "{path}");
        let again = Effect::StartReview {
            issue: issue("1"),
            round: 1,
        };
        assert_eq!(w.missing().start, vec![again], "{path}");
        let early = w.check(Some(&reviewer), &approve);
        assert!(
            early
                .as_ref()
                .is_err_and(|e| e.contains("started after your review began")),
            "{path}: {early:?}"
        );

        w.apply(&Event::Told { issue: issue("1") });
        assert!(waited_on(&w, "reviewer-1"), "{path}");
        assert_eq!(w.missing().start, vec![], "{path}: not sent a third time");
        let heard = w.check(Some(&reviewer), &approve);
        assert!(heard.is_ok(), "{path}: {heard:?}");
    }
}

#[test]
fn a_repeated_round_1_approved_before_its_running_reviewer_heard_of_it_is_left_alone() {
    let mut w = in_review();
    w.apply(&checks(false));
    w.apply(&pull_ready());
    // A person's verdict does not wait for the reviewer to have heard the round.
    w.apply(&verdict(1, Verdict::Approve));
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 1 });
    assert!(w.issues[&issue("1")].review_ok);
    assert!(!w.issues[&issue("1")].told);
    assert!(w.running(&issue("1"), Role::Reviewer).is_some());
    assert_eq!(w.missing().start, vec![]);
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
            replies: Vec::new(),
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

fn pull_ready() -> Event {
    Event::PullRequestReady {
        issue: issue("1"),
        pull: None,
        pulls: Default::default(),
        replies: Vec::new(),
    }
}

fn block(round: u8) -> Event {
    verdict(round, Verdict::Block { points: vec![] })
}

fn arbiter() -> Effect {
    Effect::StartAgent {
        group: GroupId::new("watch"),
        role: Role::Arbiter,
        issue: issue("1"),
        folder: None,
    }
}

fn starts_an_arbiter(effects: &[Effect]) -> bool {
    effects.iter().any(|e| {
        matches!(
            e,
            Effect::StartAgent {
                role: Role::Arbiter,
                ..
            }
        )
    })
}

fn stop(agent: &str) -> Effect {
    Effect::StopAgent {
        agent: AgentId::new(agent),
    }
}

/// The arbiters on issue 1 whose sessions have not ended.
fn arbiters_running(w: &World) -> Vec<&str> {
    w.agents
        .values()
        .filter(|a| {
            a.issue == Some(issue("1"))
                && a.role == Role::Arbiter
                && a.state != crate::model::AgentState::Ended
        })
        .map(|a| a.id.0.as_str())
        .collect()
}

/// Arbiter `n`'s "fix first" ruling on issue 1, as decision `n`.
fn fix_first(n: u64) -> Event {
    let Event::DecisionMade { mut decision } = ruling(false) else {
        unreachable!("a ruling is a decision")
    };
    decision.id = DecisionId(n);
    decision.by = AgentId::new(format!("arbiter-{n}"));
    Event::DecisionMade { decision }
}

#[test]
fn a_conflict_in_arbitration_stops_the_arbiter_so_the_next_block_has_one() {
    let mut w = arbitrating();
    w.apply(&started("arbiter-1", Role::Arbiter, "1"));
    assert_eq!(stage(&w, "1"), Stage::Arbitration { round: 3 });
    let effects = w.apply(&Event::PullConflicts {
        issue: issue("1"),
        repo: None,
    });
    assert!(effects.contains(&stop("arbiter-1")), "{effects:?}");
    w.apply(&ended("arbiter-1"));
    w.apply(&pull_ready());
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 3 }, "same round");
    let effects = w.apply(&block(3));
    assert!(effects.contains(&arbiter()), "{effects:?}");
    w.apply(&started("arbiter-2", Role::Arbiter, "1"));
    assert_eq!(arbiters_running(&w), ["arbiter-2"]);
    let running = w.running(&issue("1"), Role::Arbiter).map(|a| &a.id);
    assert_eq!(running, Some(&AgentId::new("arbiter-2")));
}

#[test]
fn every_send_back_from_the_arbiter_stops_it_and_the_next_block_starts_one() {
    let refused = Event::MergeRefused {
        issue: issue("1"),
        reason: "it conflicts with main".into(),
    };
    let conflict = Event::PullConflicts {
        issue: issue("1"),
        repo: None,
    };
    // What comes before the send-back, the send-back, and the round of the next
    // arbitration: a fix that names a new problem takes a new round.
    let rows: [(&str, Vec<Event>, Event, u8); 6] = [
        ("a conflict", vec![], conflict, 3),
        ("a failed check", vec![], checks(false), 3),
        ("a fix-first ruling", vec![], ruling(false), 4),
        (
            "an approval overturned in arbitration",
            vec![ruling(true)],
            overturned(1),
            4,
        ),
        (
            "an approval overturned while merging",
            vec![checks(true), ruling(true)],
            overturned(1),
            4,
        ),
        (
            "a refused merge after an approval",
            vec![checks(true), ruling(true)],
            refused,
            3,
        ),
    ];
    for (path, before, send_back, next) in rows {
        let mut w = arbitrating();
        w.apply(&started("arbiter-1", Role::Arbiter, "1"));
        for e in &before {
            w.apply(e);
        }
        let effects = w.apply(&send_back);
        assert!(effects.contains(&stop("arbiter-1")), "{path}: {effects:?}");
        assert!(
            matches!(stage(&w, "1"), Stage::Fixing { .. }),
            "{path}: {:?}",
            stage(&w, "1")
        );
        w.apply(&ended("arbiter-1"));
        w.apply(&pull_ready());
        assert_eq!(stage(&w, "1"), Stage::InReview { round: next }, "{path}");
        let effects = w.apply(&block(next));
        assert!(effects.contains(&arbiter()), "{path}: {effects:?}");
        w.apply(&started("arbiter-2", Role::Arbiter, "1"));
        assert_eq!(arbiters_running(&w), ["arbiter-2"], "{path}");
    }
}

#[test]
fn a_last_round_block_while_an_arbiter_still_runs_starts_no_second_one() {
    let conflict = Event::PullConflicts {
        issue: issue("1"),
        repo: None,
    };
    // The send-back whose stop fails, as an older log that never stopped
    // arbiter-1 would leave it; the round of the next arbitration; whether the
    // issue waits on arbiter-1 then; and what `World::missing` asks for while it
    // runs. One that has not ruled is reminded and stopped like any idle agent.
    // One that ruled is not, so the loop asks again for its stop.
    let rows: [(&str, Event, u8, bool, Vec<Effect>); 2] = [
        ("a conflict", conflict, 3, true, vec![]),
        (
            "a fix-first ruling",
            ruling(false),
            4,
            false,
            vec![stop("arbiter-1")],
        ),
    ];
    for (path, send_back, round, waits, asked) in rows {
        let mut w = arbitrating();
        w.apply(&started("arbiter-1", Role::Arbiter, "1"));
        w.apply(&send_back);
        w.apply(&pull_ready());
        let effects = w.apply(&block(round));
        assert!(!starts_an_arbiter(&effects), "{path}: {effects:?}");
        assert_eq!(stage(&w, "1"), Stage::Arbitration { round }, "{path}");
        assert_eq!(waited_on(&w, "arbiter-1"), waits, "{path}");
        let missing = w.missing();
        assert_eq!(missing.start, asked, "{path}");
        // Once it has ended, a fresh one takes over.
        w.apply(&ended("arbiter-1"));
        let missing = w.missing();
        assert_eq!(missing.start, vec![arbiter()], "{path}");
        assert_eq!(missing.given_up, Vec::<String>::new(), "{path}");
    }
}

#[test]
fn arbiters_stopped_by_a_send_back_do_not_count_toward_the_starts_limit() {
    let mut w = arbitrating();
    for (n, round) in [(1, 3), (2, 4)] {
        let id = format!("arbiter-{n}");
        w.apply(&started(&id, Role::Arbiter, "1"));
        let effects = w.apply(&fix_first(n));
        assert!(effects.contains(&stop(&id)), "{effects:?}");
        w.apply(&ended(&id));
        w.apply(&pull_ready());
        w.apply(&block(round + 1));
    }
    assert_eq!(stage(&w, "1"), Stage::Arbitration { round: 5 });
    w.apply(&started("arbiter-3", Role::Arbiter, "1"));
    // The third ends on its own before it rules.
    w.apply(&ended("arbiter-3"));
    let missing = w.missing();
    assert_eq!(missing.start, vec![arbiter()]);
    assert_eq!(missing.given_up, Vec::<String>::new());
}

#[test]
fn a_lost_merge_deploy_or_verify_is_asked_for_again() {
    let mut w = in_review();
    // Past the merge, the author and reviewer still running are asked to stop
    // first.
    let stops = [stop("author-1"), stop("reviewer-1")];
    for (s, want, stops) in [
        (Stage::Merging, Effect::Merge { issue: issue("1") }, &[][..]),
        (
            Stage::Deploying,
            Effect::Deploy { issue: issue("1") },
            &stops,
        ),
        (
            Stage::Verifying,
            Effect::Verify { issue: issue("1") },
            &stops,
        ),
    ] {
        if let Some(i) = w.issues.get_mut(&issue("1")) {
            i.stage = s;
        }
        let mut all = stops.to_vec();
        all.push(want);
        assert_eq!(w.missing().start, all);
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
fn an_older_log_with_a_close_after_a_failed_verify_replays_as_done() {
    // The tracker part no longer writes this close; a log that holds one keeps
    // the state it gave, so what came after it in the log still fits.
    let (w, _) = World::replay(&[
        group(&["m1"], 1),
        folder("f1"),
        sync("1", "m1", 1, &[]),
        planned("1"),
        pull_ready(),
        verdict(1, Verdict::Approve),
        checks(true),
        Event::Merged {
            issue: issue("1"),
            repo: None,
        },
        Event::Deployed { issue: issue("1") },
        Event::Verified {
            issue: issue("1"),
            passed: false,
            output: "the clock struck twelve".into(),
        },
        Event::IssueSynced {
            issue: issue("1"),
            title: "issue 1".into(),
            milestone: Some(MilestoneId::new("m1")),
            depends_on: vec![],
            order: 1,
            closed: true,
        },
    ]);
    assert_eq!(stage(&w, "1"), Stage::Done);
    assert!(w.issues[&issue("1")].has_merged);
    assert_eq!(w.folders[&FolderId::new("f1")].holder, None);
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
        lost: false,
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
            replies: Vec::new(),
        },
        Event::ReviewFinished {
            issue: issue("1"),
            round: 1,
            verdict: Verdict::Approve,
            failed: None,
            open: None,
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
        replies: Vec::new(),
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
        failed: None,
        open: None,
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
        replies: Vec::new(),
    });
    w.apply(&Event::ReviewFinished {
        issue: gld(),
        round: 1,
        verdict: Verdict::Approve,
        failed: None,
        open: None,
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
            replies: Vec::new(),
        },
        Event::ReviewFinished {
            issue: gld(),
            round: 1,
            verdict: Verdict::Approve,
            failed: None,
            open: None,
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
        replies: Vec::new(),
    });
    assert_eq!(
        w.issues[&gld()].pulls.get(&pid("ui")).map(String::as_str),
        Some("12")
    );
    w.apply(&Event::ReviewFinished {
        issue: gld(),
        round: 1,
        verdict: Verdict::Approve,
        failed: None,
        open: None,
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
    let unread = Err("not read here".into());
    let text = crate::text::issue(&w, &gld(), &unread).expect("text");
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
            replies: Vec::new(),
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

/// A world with folders `f1` and `f2` where issue 1 is in review: its author
/// holds `f1` and its reviewer runs, recorded with no folder.
fn in_review_with_a_second_folder() -> World {
    let (mut w, _) = World::replay(&[
        group(&["m1"], 2),
        folder("f1"),
        folder("f2"),
        sync("1", "m1", 1, &[]),
        planned("1"),
        started("author-1", Role::Author, "1"),
        Event::PullRequestReady {
            issue: issue("1"),
            pull: Some("57".into()),
            pulls: Default::default(),
            replies: Vec::new(),
        },
        started("reviewer-1", Role::Reviewer, "1"),
    ]);
    assert_eq!(w.folders[&FolderId::new("f2")].holder, None);
    w.apply(&sync("2", "m1", 2, &[]));
    w
}

fn reader_launch(w: &World, role: Role, key: &str) -> crate::launch::Brief {
    w.launch(&GroupId::new("watch"), role, &issue(key), None)
        .expect("launch")
        .brief
}

#[test]
fn a_reviewer_in_a_folder_an_author_is_then_given_was_told_to_leave_it_alone() {
    let mut w = in_review_with_a_second_folder();
    let leave = "Do not switch its branch or change any file in it.";
    let before = reader_launch(&w, Role::Reviewer, "1");
    assert_eq!(before.folder, Some("/srv/ankh/f2".into()));
    assert!(before.task.contains(leave), "{}", before.task);

    assert_eq!(w.apply(&planned("2")), vec![author("2", "f2")]);
    let after = reader_launch(&w, Role::Reviewer, "1");
    assert!(after.task.contains(leave), "{}", after.task);
}

#[test]
fn readers_in_every_free_folder_do_not_keep_a_ready_issue_from_its_author() {
    let mut w = in_review_with_a_second_folder();
    w.apply(&sync("3", "m1", 3, &[]));
    w.apply(&started("refiner-3", Role::Refiner, "3"));
    // f2 is the only free folder, and both running readers are in it.
    for (role, key) in [(Role::Reviewer, "1"), (Role::Refiner, "3")] {
        assert_eq!(
            reader_launch(&w, role, key).folder,
            Some("/srv/ankh/f2".into())
        );
        assert!(w.running(&issue(key), role).is_some());
    }
    assert_eq!(w.apply(&planned("2")), vec![author("2", "f2")]);
}

#[path = "world_scope_tests.rs"]
mod scope;

#[path = "world_hold_tests.rs"]
mod hold;

#[path = "world_assign_tests.rs"]
mod assign;
