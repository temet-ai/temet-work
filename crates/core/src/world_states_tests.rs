//! Each check's state and each review point's reply and state, as the loop
//! learns them: from the plan, the author's ready, the reviewer's verdict and a
//! change that goes back to its author before it was judged.

use std::collections::BTreeMap;

use super::*;
use crate::command::Command;
use crate::ids::{AgentId, DecisionId, FolderId, GroupId, IssueId, MachineId, ProjectId};
use crate::model::{Check, CheckState, Plan, PointState};
use crate::settings::{Scope, Value};
use crate::view::View;

use CheckState::{Failed, Passed, Running, Waiting};
use PointState::{Open, Resolved};

fn watch4() -> IssueId {
    IssueId::new("watch", "4")
}

fn started(agent: &str, role: Role) -> Event {
    Event::AgentStarted {
        agent: AgentId::new(agent),
        group: GroupId::new("night-watch"),
        role,
        issue: Some(watch4()),
        machine: MachineId::new("pseudopolis"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    }
}

/// `watch#4` being refined by refiner-1, with author-2, reviewer-3 and
/// arbiter-5 started, and `rounds` review rounds before an arbiter rules.
fn refining(rounds: u64) -> World {
    let events = [
        Event::ProjectAdded {
            project: ProjectId::new("watch"),
            tracker: "fake".into(),
            name: "ankh-morpork/watch".into(),
            code_host: None,
            repo: None,
        },
        Event::FolderAdded {
            folder: FolderId::new("yard-1"),
            project: ProjectId::new("watch"),
            machine: MachineId::new("pseudopolis"),
            path: "/srv/pseudopolis-yard/watch".into(),
        },
        Event::SettingSet {
            scope: Scope::Project(ProjectId::new("watch")),
            role: None,
            key: "review_rounds".into(),
            value: Value::Number(rounds),
            by: "person".into(),
        },
        Event::GroupCreated {
            group: GroupId::new("night-watch"),
            name: "Night Watch".into(),
            projects: vec![ProjectId::new("watch")],
            areas: Vec::new(),
            milestones: Vec::new(),
            refining_at_once: 1,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
        Event::IssueSynced {
            issue: watch4(),
            title: "Ring the bells".into(),
            milestone: None,
            depends_on: Vec::new(),
            order: 4,
            closed: false,
        },
        started("refiner-1", Role::Refiner),
    ];
    World::replay(&events).0
}

/// The events `command` from `by` becomes, or why it is refused. A reviewer
/// hears of its round first.
fn events(w: &mut World, by: &str, command: &Command) -> Result<Vec<Event>, String> {
    if matches!(command, Command::Review { .. }) {
        w.apply(&Event::Told { issue: watch4() });
    }
    w.events(Some(&AgentId::new(by)), command)
}

/// Runs `command` from `by`, applies what it becomes and returns what that asks
/// for.
fn run(w: &mut World, by: &str, command: &Command) -> Vec<Effect> {
    events(w, by, command)
        .unwrap_or_else(|e| panic!("{by} is refused: {e}"))
        .iter()
        .flat_map(|e| w.apply(e))
        .collect()
}

fn refused(w: &mut World, by: &str, command: &Command) -> String {
    events(w, by, command).expect_err("refused")
}

/// `watch#4` with refiner-1's plan of two checks, and author-2, reviewer-3 and
/// arbiter-5 started.
fn planned(rounds: u64) -> World {
    let mut w = refining(rounds);
    run(&mut w, "refiner-1", &plan());
    for (agent, role) in [
        ("author-2", Role::Author),
        ("reviewer-3", Role::Reviewer),
        ("arbiter-5", Role::Arbiter),
    ] {
        w.apply(&started(agent, role));
    }
    w
}

fn plan() -> Command {
    Command::Plan {
        issue: watch4(),
        plan: Plan {
            checks: vec![
                Check::new("the bell rings at midnight", "listening at the tower"),
                // A state the refiner sends is the loop's to set.
                Check {
                    state: Some(Passed),
                    ..Check::new("the rope holds", "hanging a troll from it")
                },
            ],
            ..Plan::default()
        },
    }
}

fn ready(replies: &[&str]) -> Command {
    Command::Ready {
        issue: watch4(),
        pull: Some("41".into()),
        pulls: BTreeMap::new(),
        replies: replies.iter().map(ToString::to_string).collect(),
    }
}

fn approve() -> Command {
    review(None, &[], &[])
}

fn block(points: &[&str], failed: &[usize], open: &[usize]) -> Command {
    review(Some(points), failed, open)
}

fn review(points: Option<&[&str]>, failed: &[usize], open: &[usize]) -> Command {
    Command::Review {
        issue: watch4(),
        verdict: match points {
            None => Verdict::Approve,
            Some(points) => Verdict::Block {
                points: points.iter().map(ToString::to_string).collect(),
            },
        },
        failed: failed.to_vec(),
        open: open.to_vec(),
    }
}

/// The state of each of `watch#4`'s checks, as the view shows them.
fn checks(w: &World) -> Vec<Option<CheckState>> {
    let view = View::new(w, &BTreeMap::new());
    let i = view
        .issues
        .iter()
        .find(|i| i.id == "watch#4")
        .expect("listed");
    i.checks.iter().map(|c| c.state).collect()
}

/// Each point of each of `watch#4`'s review rounds, as the view shows them:
/// its text, reply and state.
type Shown = Vec<Vec<(String, Option<String>, Option<PointState>)>>;

fn points(w: &World) -> Shown {
    let view = View::new(w, &BTreeMap::new());
    let i = view
        .issues
        .iter()
        .find(|i| i.id == "watch#4")
        .expect("listed");
    i.reviews
        .iter()
        .map(|r| {
            r.points
                .iter()
                .map(|p| (p.text.clone(), p.reply.clone(), p.state))
                .collect()
        })
        .collect()
}

fn point(
    text: &str,
    reply: Option<&str>,
    state: PointState,
) -> (String, Option<String>, Option<PointState>) {
    (text.into(), reply.map(String::from), Some(state))
}

#[test]
fn a_new_plan_shows_each_check_as_waiting_and_the_log_holds_it() {
    let mut w = refining(3);
    let written = events(&mut w, "refiner-1", &plan()).expect("a plan");
    let [Event::PlanWritten { plan, .. }] = written.as_slice() else {
        panic!("{written:?}");
    };
    let line = serde_json::to_string(plan).expect("json");
    assert_eq!(line.matches(r#""state":"waiting""#).count(), 2, "{line}");
    w.apply(&written[0]);
    assert_eq!(checks(&w), [Some(Waiting), Some(Waiting)]);
}

#[test]
fn the_authors_ready_sets_every_check_running_unless_the_review_is_waived() {
    let mut w = planned(1);
    run(&mut w, "author-2", &ready(&[]));
    assert_eq!(checks(&w), [Some(Running), Some(Running)]);
    // Round 1 is the last: the block goes to an arbiter, who sends it back.
    run(
        &mut w,
        "reviewer-3",
        &block(&["the bell is cracked"], &[2], &[]),
    );
    run(
        &mut w,
        "arbiter-5",
        &Command::Rule {
            issue: watch4(),
            approve: false,
            reason: "the rope must hold a troll".into(),
        },
    );
    // A person overturns the ruling: the review is waived and the fix merges
    // once CI passes, so no reviewer judges the checks again.
    run_as_person(
        &mut w,
        &Command::Overturn {
            decision: DecisionId(1),
            option: 1,
            reason: "the troll was heavier than any bell".into(),
        },
    );
    run(&mut w, "author-2", &ready(&["recast it"]));
    assert_eq!(w.issues[&watch4()].stage, Stage::InReview { round: 2 });
    assert_eq!(checks(&w), [Some(Passed), Some(Failed)]);
}

fn run_as_person(w: &mut World, command: &Command) {
    for e in w.events(None, command).expect("accepted") {
        w.apply(&e);
    }
}

#[test]
fn an_overturn_that_waives_the_review_mid_round_puts_the_running_checks_back_to_waiting() {
    let mut w = planned(1);
    run(&mut w, "author-2", &ready(&[]));
    run(
        &mut w,
        "reviewer-3",
        &block(&["the rope snapped"], &[2], &[]),
    );
    run(
        &mut w,
        "arbiter-5",
        &Command::Rule {
            issue: watch4(),
            approve: false,
            reason: "the rope must hold a troll".into(),
        },
    );
    run(&mut w, "author-2", &ready(&["a new rope"]));
    assert_eq!(w.issues[&watch4()].stage, Stage::InReview { round: 2 });
    assert_eq!(checks(&w), [Some(Running), Some(Running)]);
    // A person overturns the ruling while round 2 is in review: the reviewer is
    // stopped and no one judges the checks.
    run_as_person(
        &mut w,
        &Command::Overturn {
            decision: DecisionId(1),
            option: 1,
            reason: "the troll was heavier than any bell".into(),
        },
    );
    assert!(w.issues[&watch4()].review_waived);
    assert_eq!(checks(&w), [Some(Waiting), Some(Waiting)]);
    // CI passes, the change merges and deploys: still no one judged them.
    for e in [
        Event::ChecksFinished {
            issue: watch4(),
            passed: true,
            failing: Vec::new(),
        },
        Event::Merged {
            issue: watch4(),
            repo: None,
        },
    ] {
        w.apply(&e);
    }
    assert_eq!(w.issues[&watch4()].stage, Stage::Deploying);
    assert_eq!(checks(&w), [Some(Waiting), Some(Waiting)]);
    // A failed verify sends it back to work, and the checks still wait.
    w.apply(&Event::Deployed { issue: watch4() });
    w.apply(&Event::Verified {
        issue: watch4(),
        passed: false,
        output: "the bell fell".into(),
    });
    assert_eq!(w.issues[&watch4()].stage, Stage::Working);
    assert_eq!(checks(&w), [Some(Waiting), Some(Waiting)]);
}

#[test]
fn the_reviewers_verdict_marks_each_check_passed_or_failed() {
    let mut w = planned(3);
    run(&mut w, "author-2", &ready(&[]));
    // Check 2 does not hold; check 1, left unnamed, passed. The log holds which.
    let verdict = events(
        &mut w,
        "reviewer-3",
        &block(&["the rope snapped"], &[2], &[]),
    )
    .expect("a block");
    assert!(
        matches!(
            verdict.as_slice(),
            [Event::ReviewFinished { failed: Some(failed), .. }] if failed == &[2]
        ),
        "{verdict:?}"
    );
    for e in &verdict {
        w.apply(e);
    }
    assert_eq!(checks(&w), [Some(Passed), Some(Failed)]);
    run(&mut w, "author-2", &ready(&["a new rope"]));
    assert_eq!(checks(&w), [Some(Running), Some(Running)]);
    // The daemon refuses --failed on an approve, and a check the plan lacks.
    let why = refused(&mut w, "reviewer-3", &review(None, &[1], &[]));
    assert!(why.contains("an approve passes every check"), "{why}");
    for n in [0, 3] {
        let why = refused(&mut w, "reviewer-3", &block(&["x"], &[n], &[]));
        assert!(
            why.contains(&format!("watch#4's plan has checks 1 to 2, not {n}")),
            "{why}"
        );
    }
    run(&mut w, "reviewer-3", &approve());
    assert_eq!(checks(&w), [Some(Passed), Some(Passed)]);
}

#[test]
fn a_change_sent_back_before_it_was_judged_puts_its_running_checks_back_to_waiting() {
    let mut w = planned(1);
    run(&mut w, "author-2", &ready(&[]));
    // CI names its own jobs, not the plan's checks: it marks none of them.
    w.apply(&Event::ChecksFinished {
        issue: watch4(),
        passed: false,
        failing: vec!["the bell rings at midnight".into()],
    });
    assert_eq!(checks(&w), [Some(Waiting), Some(Waiting)]);
    run(&mut w, "author-2", &ready(&[]));
    assert_eq!(checks(&w), [Some(Running), Some(Running)]);
    w.apply(&Event::PullConflicts {
        issue: watch4(),
        repo: None,
    });
    assert_eq!(checks(&w), [Some(Waiting), Some(Waiting)]);
    run(&mut w, "author-2", &ready(&[]));
    // The last round's block goes to an arbiter; CI fails before the ruling.
    // The reviewer judged both checks, so each keeps its state.
    run(
        &mut w,
        "reviewer-3",
        &block(&["the rope snapped"], &[2], &[]),
    );
    assert!(matches!(
        w.issues[&watch4()].stage,
        Stage::Arbitration { .. }
    ));
    w.apply(&Event::ChecksFinished {
        issue: watch4(),
        passed: false,
        failing: Vec::new(),
    });
    assert_eq!(checks(&w), [Some(Passed), Some(Failed)]);
    // A passing CI run marks no check either.
    run(&mut w, "author-2", &ready(&["a new rope"]));
    run(&mut w, "reviewer-3", &approve());
    w.apply(&Event::ChecksFinished {
        issue: watch4(),
        passed: true,
        failing: Vec::new(),
    });
    assert_eq!(checks(&w), [Some(Passed), Some(Passed)]);
}

#[test]
fn the_first_ready_after_a_block_carries_one_reply_for_each_point() {
    let mut w = planned(3);
    // The first ready answers no block.
    let why = refused(&mut w, "author-2", &ready(&["early"]));
    assert!(why.contains("no review point to reply to"), "{why}");
    run(&mut w, "author-2", &ready(&[]));
    run(
        &mut w,
        "reviewer-3",
        &block(&["the bell is cracked", "the rope is frayed"], &[], &[]),
    );
    // One reply, or none, is refused, naming each point and the form to use.
    for replies in [&["recast the bell"][..], &[]] {
        let why = refused(&mut w, "author-2", &ready(replies));
        for line in [
            "this ready answers review round 1 of watch#4, so it needs exactly one reply for each \
             of the round's points below",
            "\n1. the bell is cracked\n2. the rope is frayed\n",
            "Run: temet-work ready watch#4 --reply \"<reply to point 1>\" --reply \"<reply to point 2>\"",
            "In the MCP ready tool, give replies with one entry for each point",
        ] {
            assert!(why.contains(line), "missing {line:?} in {why}");
        }
    }
    run(
        &mut w,
        "author-2",
        &ready(&["recast the bell", "the rope stays: it was checked"]),
    );
    assert_eq!(
        points(&w),
        [vec![
            point("the bell is cracked", Some("recast the bell"), Open),
            point(
                "the rope is frayed",
                Some("the rope stays: it was checked"),
                Open
            ),
        ]]
    );
    // A later ready answers no block: here, a fix for CI.
    w.apply(&Event::ChecksFinished {
        issue: watch4(),
        passed: false,
        failing: Vec::new(),
    });
    let why = refused(&mut w, "author-2", &ready(&["the lint is fixed"]));
    assert!(why.contains("no review point to reply to"), "{why}");
    run(&mut w, "author-2", &ready(&[]));
}

#[test]
fn after_a_fix_ruling_the_replies_answer_the_reviewers_last_block() {
    let mut w = planned(1);
    run(&mut w, "author-2", &ready(&[]));
    run(
        &mut w,
        "reviewer-3",
        &block(&["the bell is cracked"], &[], &[]),
    );
    run(
        &mut w,
        "arbiter-5",
        &Command::Rule {
            issue: watch4(),
            approve: false,
            reason: "a cracked bell does not ring true".into(),
        },
    );
    let why = refused(&mut w, "author-2", &ready(&[]));
    assert!(why.contains("\n1. the bell is cracked\n"), "{why}");
    run(&mut w, "author-2", &ready(&["recast the bell"]));
    assert_eq!(
        points(&w),
        [vec![point(
            "the bell is cracked",
            Some("recast the bell"),
            Open
        )]]
    );
}

#[test]
fn the_next_round_resolves_each_point_or_keeps_it_open() {
    let mut w = planned(3);
    run(&mut w, "author-2", &ready(&[]));
    // Round 1 has no round before it.
    let why = refused(&mut w, "reviewer-3", &block(&["x"], &[], &[1]));
    assert!(
        why.contains("review round 1 of watch#4 has no round before it"),
        "{why}"
    );
    run(
        &mut w,
        "reviewer-3",
        &block(&["the bell is cracked", "the rope is frayed"], &[], &[]),
    );
    run(
        &mut w,
        "author-2",
        &ready(&["recast the bell", "the rope stays: it was checked"]),
    );
    // Out of range, and with an approve.
    for n in [0, 3] {
        let why = refused(&mut w, "reviewer-3", &block(&["x"], &[], &[n]));
        assert!(
            why.contains(&format!(
                "review round 1 of watch#4 has points 1 to 2, not {n}"
            )),
            "{why}"
        );
    }
    let why = refused(&mut w, "reviewer-3", &review(None, &[], &[2]));
    assert!(why.contains("an approve resolves every point"), "{why}");
    // Round 2 keeps the rope open and adds a point of its own.
    run(
        &mut w,
        "reviewer-3",
        &block(&["the clapper is loose"], &[], &[2]),
    );
    let author = "the rope stays: it was checked";
    assert_eq!(
        points(&w),
        [
            vec![
                point("the bell is cracked", Some("recast the bell"), Resolved),
                point("the rope is frayed", Some(author), Open),
            ],
            vec![
                point("the clapper is loose", None, Open),
                point("the rope is frayed", None, Open),
            ],
        ]
    );
    // The author is told to fix the kept point again.
    assert_eq!(
        w.issues[&watch4()].fix,
        ["the clapper is loose", "the rope is frayed"]
    );
    run(
        &mut w,
        "author-2",
        &ready(&["tightened it", "a new rope is on order"]),
    );
    // A block may hold only points kept open.
    let kept = events(&mut w, "reviewer-3", &block(&[], &[], &[2])).expect("a block");
    assert!(
        matches!(
            kept.as_slice(),
            [Event::ReviewFinished { verdict: Verdict::Block { points }, open: Some(open), .. }]
                if points == &["the rope is frayed"] && open == &[2]
        ),
        "{kept:?}"
    );
    // An approve resolves every point of the round before.
    run(&mut w, "reviewer-3", &approve());
    let rounds = points(&w);
    assert_eq!(
        rounds[1],
        [
            point("the clapper is loose", Some("tightened it"), Resolved),
            point(
                "the rope is frayed",
                Some("a new rope is on order"),
                Resolved
            ),
        ]
    );
    assert!(rounds[2].is_empty(), "an approve raises no point");
}

/// The arbiter's brief as the world writes it when `effects` ask for one, which
/// must be for `agent`, who then starts.
fn arbiter_brief(w: &mut World, effects: &[Effect], agent: &str) -> String {
    let group = effects
        .iter()
        .find_map(|e| match e {
            Effect::StartAgent {
                group,
                role: Role::Arbiter,
                ..
            } => Some(group.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no arbiter is asked for: {effects:?}"));
    let launch = w
        .launch(&group, Role::Arbiter, &watch4(), None)
        .expect("a launch");
    assert_eq!(launch.agent, AgentId::new(agent));
    w.apply(&started(agent, Role::Arbiter));
    launch.brief.task
}

/// `watch#4` blocked in each of its three rounds, round 2 keeping a point of
/// round 1 open and round 3 one of round 2, with what the third block asks for.
fn blocked_three_times() -> (World, Vec<Effect>) {
    let mut w = refining(3);
    run(&mut w, "refiner-1", &plan());
    w.apply(&started("author-2", Role::Author));
    w.apply(&started("reviewer-3", Role::Reviewer));
    run(&mut w, "author-2", &ready(&[]));
    run(
        &mut w,
        "reviewer-3",
        &block(&["the bell is cracked", "the rope is frayed"], &[], &[]),
    );
    run(
        &mut w,
        "author-2",
        &ready(&["recast the bell", "the rope stays: it was checked"]),
    );
    run(
        &mut w,
        "reviewer-3",
        &block(&["the clapper is loose"], &[], &[2]),
    );
    run(
        &mut w,
        "author-2",
        &ready(&["tightened it", "a new rope is on order"]),
    );
    let effects = run(
        &mut w,
        "reviewer-3",
        &block(&["the tower leans"], &[], &[1]),
    );
    (w, effects)
}

/// Rounds 1 and 2 of [`blocked_three_times`] as an arbiter's brief lists them.
const FIRST_TWO_ROUNDS: &str = "Review rounds:\n  \
    Round 1: blocked\n    \
    1. the bell is cracked\n       The author's reply: recast the bell\n       \
    State: resolved by round 2\n    \
    2. the rope is frayed\n       The author's reply: the rope stays: it was checked\n       \
    State: kept open by round 2\n  \
    Round 2: blocked\n    \
    1. the clapper is loose\n       The author's reply: tightened it\n       \
    State: kept open by round 3\n    \
    2. the rope is frayed\n       The author's reply: a new rope is on order\n       \
    State: resolved by round 3\n";

#[test]
fn an_arbiter_is_given_every_review_round_with_each_reply_and_state() {
    let (mut w, effects) = blocked_three_times();
    let brief = arbiter_brief(&mut w, &effects, "arbiter-1");
    let third = "  Round 3: blocked\n    \
        1. the tower leans\n       The author's reply: none on record\n       \
        State: not judged by a later round\n    \
        2. the clapper is loose\n       The author's reply: none on record\n       \
        State: not judged by a later round\n  \
        No later round judged the points of round 3, the last block, and the author has not \
        replied to it because review rounds ran out before the author's next ready.\n";
    assert!(
        brief.contains(&format!("{FIRST_TWO_ROUNDS}{third}")),
        "{brief}"
    );
    let (_, last) = brief.split_once("Round 3: blocked").expect("round 3");
    assert!(!last.contains("kept open"), "{brief}");
    assert!(!last.contains("resolved by"), "{brief}");
    assert!(
        brief.contains(
            "Read the pull request and the review rounds listed above, with the author's \
             replies; temet-work issue watch#4 lists the rounds again. Decide whether it merges."
        ),
        "{brief}"
    );
    assert!(
        !brief.contains("the review points and the author's answers"),
        "{brief}"
    );
}

#[test]
fn a_second_arbiter_is_given_every_review_round_with_the_replies_to_the_ruled_one() {
    let (mut w, effects) = blocked_three_times();
    arbiter_brief(&mut w, &effects, "arbiter-1");
    run(
        &mut w,
        "arbiter-1",
        &Command::Rule {
            issue: watch4(),
            approve: false,
            reason: "a leaning tower drops its bell".into(),
        },
    );
    w.apply(&Event::AgentChanged {
        agent: AgentId::new("arbiter-1"),
        state: AgentState::Ended,
        lost: false,
    });
    run(
        &mut w,
        "author-2",
        &ready(&["propped the tower", "pinned the clapper"]),
    );
    // Round 4 keeps the clapper open.
    let effects = run(
        &mut w,
        "reviewer-3",
        &block(&["the prop is rotten"], &[], &[2]),
    );
    let brief = arbiter_brief(&mut w, &effects, "arbiter-2");
    let later = "  Round 3: blocked\n    \
        1. the tower leans\n       The author's reply: propped the tower\n       \
        State: resolved by round 4\n    \
        2. the clapper is loose\n       The author's reply: pinned the clapper\n       \
        State: kept open by round 4\n  \
        Round 4: blocked\n    \
        1. the prop is rotten\n       The author's reply: none on record\n       \
        State: not judged by a later round\n    \
        2. the clapper is loose\n       The author's reply: none on record\n       \
        State: not judged by a later round\n  \
        No later round judged the points of round 4, the last block, and the author has not \
        replied to it because review rounds ran out before the author's next ready.\n";
    assert!(
        brief.contains(&format!("{FIRST_TWO_ROUNDS}{later}")),
        "{brief}"
    );
}
