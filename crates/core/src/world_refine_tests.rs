//! A person's refine (`temet-work refine`): the issue's title, the person's
//! checks kept apart from the plan, and a send that hands the issue to a
//! refiner at once, at each stage it may be sent from.

use super::*;
use crate::model::{AgentState, Check};
use crate::settings::{Scope, Value};

fn refine(title: Option<&str>, checks: Option<&[(&str, &str)]>, send: bool) -> Command {
    Command::Refine {
        issue: issue("1"),
        title: title.map(Into::into),
        checks: checks.map(|list| list.iter().map(|(d, p)| Check::new(*d, *p)).collect()),
        send,
    }
}

/// Runs a person's `command` and applies what it becomes, returning the effects.
fn run(w: &mut World, command: &Command) -> Vec<Effect> {
    let events = w.events(None, command).expect("accepted");
    events.iter().flat_map(|e| w.apply(e)).collect()
}

/// The reply the daemon gives a person's `command`, after applying it.
fn replied(w: &mut World, command: &Command) -> String {
    let before = w.clone();
    run(w, command);
    crate::text::refined(&before, w, &issue("1"))
}

fn refused(w: &World, command: &Command) -> String {
    w.events(None, command).expect_err("refused")
}

fn dusk() -> Vec<Check> {
    vec![Check::new("the gate shuts at dusk", "the dusk test")]
}

fn plan_with(done_when: &str) -> Event {
    Event::PlanWritten {
        issue: issue("1"),
        plan: Plan {
            checks: vec![Check {
                state: Some(crate::model::CheckState::Waiting),
                ..Check::new(done_when, "a test")
            }],
            ..Plan::default()
        },
    }
}

fn stop(agent: &str) -> Effect {
    Effect::StopAgent {
        agent: AgentId::new(agent),
    }
}

fn release(folder: &str) -> Effect {
    Effect::ReleaseFolder {
        folder: FolderId::new(folder),
    }
}

/// The task of the refiner a send starts on issue 1, which must be `agent`,
/// after it is recorded as started.
fn start_refiner(w: &mut World, agent: &str) -> String {
    let launch = w
        .launch(&GroupId::new("watch"), Role::Refiner, &issue("1"), None)
        .expect("launch");
    assert_eq!(launch.agent, AgentId::new(agent));
    w.apply(&started(agent, Role::Refiner, "1"));
    launch.brief.task
}

/// A world where issue 1 is new, and its group refines one issue at a time and
/// already refines issue 2.
fn new_with_the_slot_taken() -> World {
    let mut w = World::default();
    for event in [
        group(&["m1"], 1),
        folder("f1"),
        sync("2", "m1", 1, &[]),
        sync("1", "m1", 2, &[]),
    ] {
        w.apply(&event);
    }
    assert_eq!(stage(&w, "2"), Stage::Refining);
    assert_eq!(stage(&w, "1"), Stage::New, "the slot is taken");
    w
}

#[test]
fn a_refine_is_one_event_with_the_title_the_checks_and_the_send_and_replays_to_the_same_state() {
    let mut w = new_with_the_slot_taken();
    let command = refine(
        Some("The gate shuts at dusk"),
        Some(&[("the gate shuts at dusk", "the dusk test")]),
        true,
    );
    let events = w.events(None, &command).expect("accepted");
    assert_eq!(
        events,
        vec![Event::Refined {
            issue: issue("1"),
            title: Some("The gate shuts at dusk".into()),
            checks: Some(dusk()),
            send: true,
        }]
    );
    let mut log = vec![
        group(&["m1"], 1),
        folder("f1"),
        sync("2", "m1", 1, &[]),
        sync("1", "m1", 2, &[]),
    ];
    log.extend(events.iter().cloned());
    for e in &events {
        w.apply(e);
    }
    let (replayed, _) = World::replay(&log);
    assert_eq!(replayed, w);
    let i = &w.issues[&issue("1")];
    assert_eq!(i.title, "The gate shuts at dusk");
    assert_eq!(i.person_checks, dusk());
}

#[test]
fn the_persons_checks_are_kept_apart_from_the_plan_replaced_emptied_or_kept() {
    let mut w = World::replay(&[group(&["m1"], 1), sync("1", "m1", 1, &[])]).0;
    w.apply(&started("refiner-1", Role::Refiner, "1"));
    run(
        &mut w,
        &refine(
            None,
            Some(&[("the gate shuts at dusk", "the dusk test")]),
            false,
        ),
    );
    assert_eq!(w.issues[&issue("1")].person_checks, dusk());

    // A plan the refiner writes later leaves them as they are.
    w.apply(&plan_with("the gate shuts"));
    let i = &w.issues[&issue("1")];
    assert_eq!(i.person_checks, dusk());
    assert_eq!(i.plan.as_ref().map(|p| p.checks.len()), Some(1));

    // A refine with neither --check nor --no-checks keeps them.
    run(&mut w, &refine(Some("A gate at dusk"), None, false));
    assert_eq!(w.issues[&issue("1")].person_checks, dusk());

    // --check replaces the list, and --no-checks empties it.
    run(
        &mut w,
        &refine(
            None,
            Some(&[("the lamp is lit", ""), ("the bell rings", "")]),
            false,
        ),
    );
    assert_eq!(
        w.issues[&issue("1")].person_checks,
        vec![
            Check::new("the lamp is lit", ""),
            Check::new("the bell rings", "")
        ]
    );
    run(&mut w, &refine(None, Some(&[]), false));
    assert_eq!(w.issues[&issue("1")].person_checks, vec![]);
    assert_eq!(
        stage(&w, "1"),
        Stage::Ready,
        "a save without send moves nothing"
    );
}

#[test]
fn a_title_and_checks_are_stored_clean_and_an_empty_title_or_done_when_is_refused() {
    let mut w = World::replay(&[group(&["m1"], 1), sync("1", "m1", 1, &[])]).0;
    let messy = refine(
        Some("  The gate\r\n\tshuts \u{1b}at  dusk "),
        Some(&[("the gate\u{7} shuts\r\nat dusk", "the dusk\u{0} test")]),
        false,
    );
    let Ok(events) = w.events(None, &messy) else {
        panic!("refused");
    };
    assert_eq!(
        events,
        vec![Event::Refined {
            issue: issue("1"),
            title: Some("The gate shuts at dusk".into()),
            checks: Some(vec![Check::new("the gate shuts\nat dusk", "the dusk test")]),
            send: false,
        }]
    );
    for e in &events {
        w.apply(e);
    }

    // A title equal to the issue's is not written again; with nothing else to
    // change the refine becomes no event.
    assert_eq!(
        w.events(None, &refine(Some(" The gate shuts at dusk"), None, false)),
        Ok(vec![])
    );
    assert_eq!(
        w.events(
            None,
            &refine(
                Some("The gate shuts at dusk"),
                Some(&[("the lamp is lit", "")]),
                false
            )
        ),
        Ok(vec![Event::Refined {
            issue: issue("1"),
            title: None,
            checks: Some(vec![Check::new("the lamp is lit", "")]),
            send: false,
        }])
    );

    assert!(refused(&w, &refine(Some(" \n "), None, false)).contains("a title cannot be empty"));
    let why = refused(
        &w,
        &refine(None, Some(&[("ok", ""), (" \u{1b} ", "a test")]), false),
    );
    assert!(why.contains("check 2 has none"), "{why}");
    assert!(
        refused(
            &w,
            &Command::Refine {
                issue: issue("9"),
                title: Some("x".into()),
                checks: None,
                send: false,
            }
        )
        .contains("there is no issue ankh#9")
    );
}

#[test]
fn only_a_person_refines() {
    let mut w = World::replay(&[group(&["m1"], 1), sync("1", "m1", 1, &[])]).0;
    w.apply(&started("refiner-1", Role::Refiner, "1"));
    let why = w
        .events(
            Some(&AgentId::new("refiner-1")),
            &refine(Some("A gate"), None, false),
        )
        .expect_err("refused");
    assert!(
        why.starts_with("refine is a person's: refiner-1 is an agent"),
        "{why}"
    );
}

#[test]
fn a_send_on_a_new_issue_starts_a_refiner_past_refining_at_once_whose_brief_holds_the_checks() {
    let mut w = new_with_the_slot_taken();
    let effects = run(
        &mut w,
        &refine(
            None,
            Some(&[("the gate shuts at dusk", "the dusk test")]),
            true,
        ),
    );
    assert_eq!(effects, vec![refiner("1")]);
    assert_eq!(stage(&w, "1"), Stage::Refining);
    assert_eq!(w.issues[&issue("1")].group, Some(GroupId::new("watch")));
    assert_eq!(
        stage(&w, "2"),
        Stage::Refining,
        "the other keeps its refiner"
    );
    let task = start_refiner(&mut w, "refiner-1");
    assert!(
        task.contains(
            "The person's checks, each a point your plan must hold:\n\
             - the gate shuts at dusk (shown by: the dusk test)\n"
        ),
        "{task}"
    );
    assert!(!task.contains("It was refined before"), "{task}");
}

#[test]
fn a_send_on_a_new_issue_is_refused_when_no_group_covers_it_its_refiner_has_no_folder_or_held() {
    let mut w = new_with_the_slot_taken();
    w.apply(&sync("3", "m9", 3, &[]));
    let why = refused(
        &w,
        &Command::Refine {
            issue: issue("3"),
            title: None,
            checks: None,
            send: true,
        },
    );
    assert!(why.starts_with("no group's scope covers ankh#3"), "{why}");

    let mut pinned = w.clone();
    pinned.apply(&Event::SettingSet {
        scope: Scope::Global,
        role: Some(Role::Refiner),
        key: "machine".into(),
        value: Value::Text("klatch".into()),
        by: "person".into(),
    });
    let why = refused(&pinned, &refine(None, None, true));
    assert!(
        why.starts_with("ankh#1's refiner has no folder to run in"),
        "{why}"
    );
    assert!(why.contains("klatch"), "{why}");

    w.apply(&Event::Held);
    let why = refused(&w, &refine(Some("A gate"), None, true));
    assert!(why.contains("while the daemon is held"), "{why}");
    assert!(
        why.contains("temet-work resume") && why.contains("save without send"),
        "{why}"
    );
    // Saved without send while held: accepted.
    run(&mut w, &refine(Some("A gate"), None, false));
    assert_eq!(w.issues[&issue("1")].title, "A gate");
    assert_eq!(stage(&w, "1"), Stage::New);
}

#[test]
fn a_send_on_a_ready_issue_refines_it_again_with_the_plan_it_replaces_in_the_brief() {
    let mut w = World::replay(&[group(&["m1"], 1), sync("1", "m1", 1, &[])]).0;
    w.apply(&started("refiner-1", Role::Refiner, "1"));
    w.apply(&plan_with("the gate shuts"));
    w.apply(&ended("refiner-1"));
    assert_eq!(stage(&w, "1"), Stage::Ready, "no folder, so no author");
    let effects = run(
        &mut w,
        &refine(None, Some(&[("the lamp is lit at dusk", "")]), true),
    );
    assert_eq!(effects, vec![refiner("1")]);
    let i = &w.issues[&issue("1")];
    assert_eq!(i.stage, Stage::Refining);
    assert_eq!(
        i.plan.as_ref().map(|p| p.checks[0].done_when.as_str()),
        Some("the gate shuts"),
        "the old plan stays until the new one replaces it"
    );
    let task = start_refiner(&mut w, "refiner-2");
    assert!(
        task.contains(
            "It was refined before. The plan yours replaces had these checks:\n\
             - the gate shuts (shown by: a test)\n"
        ),
        "{task}"
    );
    assert!(
        task.contains(
            "The person's checks, each a point your plan must hold:\n- the lamp is lit at dusk\n"
        ),
        "{task}"
    );
    // The new plan replaces the old.
    w.apply(&plan_with("the lamp is lit"));
    let i = &w.issues[&issue("1")];
    assert_eq!(i.stage, Stage::Ready);
    assert_eq!(
        i.plan.as_ref().map(|p| p.checks[0].done_when.as_str()),
        Some("the lamp is lit")
    );
    assert_eq!(
        i.person_checks,
        vec![Check::new("the lamp is lit at dusk", "")]
    );
}

#[test]
fn a_send_while_refining_stops_the_refiner_refuses_its_plan_and_does_not_count_its_end() {
    let mut w = World::replay(&[group(&["m1"], 1), sync("1", "m1", 1, &[])]).0;
    w.apply(&started("refiner-1", Role::Refiner, "1"));
    let effects = run(
        &mut w,
        &refine(None, Some(&[("the lamp is lit", "")]), true),
    );
    assert_eq!(effects, vec![stop("refiner-1"), refiner("1")]);
    let task = start_refiner(&mut w, "refiner-2");
    assert!(task.contains("- the lamp is lit\n"), "{task}");

    // The stopped refiner's plan, sent before its stop lands, is refused.
    let plan = Command::Plan {
        issue: issue("1"),
        plan: Plan {
            checks: vec![Check::new("the gate shuts", "a test")],
            ..Plan::default()
        },
    };
    let why = w
        .events(Some(&AgentId::new("refiner-1")), &plan)
        .expect_err("refused");
    assert!(
        why.contains("a person sent ankh#1 to a refiner after refiner-1 started"),
        "{why}"
    );
    let old = &w.agents[&AgentId::new("refiner-1")];
    assert!(!w.waiting_on(old), "not reminded");
    assert_eq!(
        w.missing(),
        Missing {
            start: vec![stop("refiner-1")],
            given_up: vec![],
        }
    );

    // Its end counts toward no limit, and the fresh refiner's plan is taken.
    w.apply(&ended("refiner-1"));
    assert!(!w.agents[&AgentId::new("refiner-1")].ended_waited_on);
    assert_eq!(w.missing(), Missing::default());
    assert!(w.events(Some(&AgentId::new("refiner-2")), &plan).is_ok());
}

#[test]
fn a_send_starts_counting_starts_again() {
    let mut w = World::replay(&[group(&["m1"], 1), sync("1", "m1", 1, &[])]).0;
    for n in 1..=3 {
        let agent = format!("refiner-{n}");
        w.apply(&started(&agent, Role::Refiner, "1"));
        w.apply(&ended(&agent));
    }
    assert_eq!(w.missing().start, vec![], "three refiners ended: given up");
    assert_eq!(run(&mut w, &refine(None, None, true)), vec![refiner("1")]);
    w.apply(&started("refiner-4", Role::Refiner, "1"));
    w.apply(&ended("refiner-4"));
    assert_eq!(
        w.missing().start,
        vec![refiner("1")],
        "one end since the send"
    );
}

/// Issue 1 in folder f1 with author-1, its pull request 57 ready and reviewer-1
/// started, then taken on to `to`: in review, being fixed after a block, or
/// with arbiter-1 after three blocks.
fn reviewed_to(to: Stage) -> World {
    let mut w = working_on_one();
    w.apply(&plan_with("the gate shuts"));
    w.apply(&started("author-1", Role::Author, "1"));
    let ready = |w: &mut World, pull: Option<&str>| {
        let command = Command::Ready {
            issue: issue("1"),
            pull: pull.map(Into::into),
            pulls: Default::default(),
            replies: Vec::new(),
        };
        let events = w
            .events(Some(&AgentId::new("author-1")), &command)
            .expect("ready");
        apply_all(w, &events);
    };
    let block = |w: &mut World, round: u8| {
        w.apply(&Event::ReviewFinished {
            issue: issue("1"),
            round,
            verdict: Verdict::Block {
                points: vec![format!("round {round} wants a lamp")],
            },
            failed: Some(vec![]),
            open: Some(vec![]),
        });
    };
    ready(&mut w, Some("57"));
    w.apply(&started("reviewer-1", Role::Reviewer, "1"));
    match to {
        Stage::InReview { .. } => {}
        Stage::Fixing { .. } => block(&mut w, 1),
        _ => {
            for round in 1..=3 {
                block(&mut w, round);
                if round < 3 {
                    w.apply(&Event::PullRequestReady {
                        issue: issue("1"),
                        pull: None,
                        pulls: Default::default(),
                        replies: vec!["a lamp is added".into()],
                    });
                }
            }
            w.apply(&started("arbiter-1", Role::Arbiter, "1"));
        }
    }
    assert_eq!(stage(&w, "1"), to);
    w
}

/// Sends issue 1 back from `w`, where author-1 holds f1 and pull request 57 is
/// open, and checks what a send does at any stage an author works in.
fn sent_back(mut w: World, running: &[&str]) {
    let reply = replied(
        &mut w,
        &refine(None, Some(&[("the lamp is lit", "")]), true),
    );
    let i = &w.issues[&issue("1")];
    assert_eq!(i.stage, Stage::Refining);
    assert!(i.pulls.is_empty(), "the pull request is dropped");
    assert!(i.reviews.is_empty() && i.round == 0, "and the rounds");
    assert!(i.fix.is_empty() && !i.review_ok && i.checks_ok.is_none() && !i.review_waived);
    assert_eq!(i.folder, None, "its folder is taken from it");
    assert_eq!(w.folders[&FolderId::new("f1")].holder, None);
    let names: Vec<String> = running.iter().map(ToString::to_string).collect();
    assert_eq!(
        reply,
        format!(
            "ankh#1: it has 1 person's check; it is refining, and a refiner starts with a new \
             brief; {} stopped; pull request ankh #57 is left open on the code host and dropped \
             from the record; close it there if no longer wanted",
            match names.as_slice() {
                [one] => format!("{one} is"),
                [rest @ .., last] => format!("{} and {last} are", rest.join(", ")),
                [] => unreachable!(),
            }
        )
    );
    // None is reminded, and their ends count toward no limit.
    for agent in running {
        assert!(
            !w.waiting_on(&w.agents[&AgentId::new(*agent)]),
            "{agent} waited on"
        );
    }
    let refiner_task = start_refiner(&mut w, "refiner-1");
    assert!(
        refiner_task.contains("- the lamp is lit\n"),
        "{refiner_task}"
    );
    w.apply(&plan_with("the lamp is lit"));
    assert_eq!(stage(&w, "1"), Stage::Ready, "f1 waits for author-1 to end");
    for agent in running.iter().filter(|a| **a != "author-1") {
        w.apply(&ended(agent));
    }
    assert_eq!(stage(&w, "1"), Stage::Ready);
    let effects = w.apply(&ended("author-1"));
    assert_eq!(
        effects,
        vec![author("1", "f1")],
        "a new author once author-1 has ended"
    );
    for agent in running {
        let a = &w.agents[&AgentId::new(*agent)];
        assert!(
            a.state == AgentState::Ended && !a.ended_waited_on,
            "{agent} counted"
        );
    }
    // The next change starts at round 1.
    w.apply(&started("author-2", Role::Author, "1"));
    let command = Command::Ready {
        issue: issue("1"),
        pull: Some("58".into()),
        pulls: Default::default(),
        replies: Vec::new(),
    };
    let events = w
        .events(Some(&AgentId::new("author-2")), &command)
        .expect("ready");
    apply_all(&mut w, &events);
    assert_eq!(stage(&w, "1"), Stage::InReview { round: 1 });
}

#[test]
fn a_send_while_working_stops_the_author_and_drops_the_pull_request() {
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    w.issues
        .get_mut(&issue("1"))
        .expect("issue")
        .pulls
        .insert(ProjectId::new("ankh"), "57".into());
    let effects = w
        .events(None, &refine(None, None, true))
        .expect("accepted")
        .iter()
        .flat_map(|e| w.clone().apply(e))
        .collect::<Vec<_>>();
    assert_eq!(effects, vec![stop("author-1"), release("f1"), refiner("1")]);
    sent_back(w, &["author-1"]);
}

#[test]
fn a_send_in_review_stops_the_author_and_reviewer_and_drops_the_pull_request() {
    sent_back(
        reviewed_to(Stage::InReview { round: 1 }),
        &["author-1", "reviewer-1"],
    );
}

#[test]
fn a_send_while_fixing_stops_the_author_and_reviewer_and_drops_the_rounds() {
    let w = reviewed_to(Stage::Fixing { round: 1 });
    assert_eq!(w.issues[&issue("1")].reviews.len(), 1);
    sent_back(w, &["author-1", "reviewer-1"]);
}

#[test]
fn a_send_with_the_arbiter_stops_the_author_reviewer_and_arbiter() {
    let w = reviewed_to(Stage::Arbitration { round: 3 });
    assert_eq!(w.issues[&issue("1")].reviews.len(), 3);
    sent_back(w, &["arbiter-1", "author-1", "reviewer-1"]);
}

#[test]
fn a_send_is_refused_once_a_merge_has_begun_or_the_issue_is_done_and_a_save_still_works() {
    let mut w = reviewed_to(Stage::InReview { round: 1 });
    w.apply(&Event::ReviewFinished {
        issue: issue("1"),
        round: 1,
        verdict: Verdict::Approve,
        failed: Some(vec![]),
        open: Some(vec![]),
    });
    w.apply(&Event::ChecksFinished {
        issue: issue("1"),
        passed: true,
        failing: vec![],
    });
    let next = [
        Event::Merged {
            issue: issue("1"),
            repo: None,
        },
        Event::Deployed { issue: issue("1") },
        Event::Verified {
            issue: issue("1"),
            passed: true,
            output: String::new(),
        },
    ];
    for (n, why_word) in ["merging", "deploying", "verifying", "done"]
        .iter()
        .enumerate()
    {
        let why = refused(&w, &refine(Some("A gate"), None, true));
        if *why_word == "done" {
            assert!(
                why.contains("a done issue is worked again by reopening it"),
                "{why}"
            );
        } else {
            assert!(
                why.contains(why_word) && why.ends_with("a merge is not undone"),
                "{why}"
            );
        }
        let mut saved = w.clone();
        let title = format!("A gate {n}");
        run(
            &mut saved,
            &refine(Some(&title), Some(&[("the lamp is lit", "")]), false),
        );
        assert_eq!(
            saved.issues[&issue("1")].title,
            title,
            "saved at {why_word}"
        );
        assert_eq!(saved.issues[&issue("1")].stage, w.issues[&issue("1")].stage);
        if let Some(event) = next.get(n) {
            w.apply(event);
        }
    }

    // Being fixed with one repository's pull request merged, as after a person
    // closed the next one's mid-merge.
    let mut fixing = reviewed_to(Stage::Fixing { round: 1 });
    fixing
        .issues
        .get_mut(&issue("1"))
        .expect("issue")
        .merged
        .insert(ProjectId::new("ankh"));
    let why = refused(&fixing, &refine(None, None, true));
    assert!(why.contains("has its pull request in ankh merged"), "{why}");
}
