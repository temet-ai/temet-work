//! A person's `assign` and `unassign` (`temet-work assign ISSUE AGENT`,
//! `temet-work unassign ISSUE AGENT`): which agent may be put on which issue,
//! what moves with it, and what an issue gets once the agent a person took off
//! it has ended.

use std::collections::BTreeMap;

use super::*;
use crate::model::{Agent, Check};

fn assign(key: &str, agent: &str) -> Command {
    Command::Assign {
        issue: issue(key),
        agent: AgentId::new(agent),
    }
}

fn unassign(key: &str, agent: &str) -> Command {
    Command::Unassign {
        issue: issue(key),
        agent: AgentId::new(agent),
    }
}

/// Runs a person's `command` on `w` and returns what its one event led to.
fn person(w: &mut World, command: &Command) -> Vec<Effect> {
    let events = w
        .events(None, command)
        .unwrap_or_else(|why| panic!("refused: {why}"));
    assert_eq!(events.len(), 1, "one event: {events:?}");
    apply_all(w, &events)
}

/// Why a person's `command` is refused. The world is left as it was.
fn refused(w: &World, command: &Command) -> String {
    let before = w.clone();
    let why = w.events(None, command).expect_err("refused");
    assert_eq!(*w, before, "a refusal changes nothing");
    why
}

fn agent<'a>(w: &'a World, id: &str) -> &'a Agent {
    &w.agents[&AgentId::new(id)]
}

fn stop(agent: &str) -> Effect {
    Effect::StopAgent {
        agent: AgentId::new(agent),
    }
}

fn brief(agent: &str, key: &str) -> Effect {
    Effect::Brief {
        agent: AgentId::new(agent),
        issue: issue(key),
    }
}

fn briefed(agent: &str, key: &str) -> Event {
    Event::Briefed {
        agent: AgentId::new(agent),
        issue: issue(key),
    }
}

fn round(key: &str, round: u8) -> Effect {
    Effect::StartReview {
        issue: issue(key),
        round,
    }
}

fn reader(role: Role, key: &str) -> Effect {
    Effect::StartAgent {
        group: GroupId::new("watch"),
        role,
        issue: issue(key),
        folder: None,
    }
}

fn approve(key: &str) -> Command {
    Command::Review {
        issue: issue(key),
        verdict: Verdict::Approve,
        failed: Vec::new(),
        open: Vec::new(),
    }
}

fn plan(key: &str) -> Command {
    Command::Plan {
        issue: issue(key),
        plan: Plan {
            checks: vec![Check::new("the clock strikes nine", "listen at the tower")],
            ..Plan::default()
        },
    }
}

fn ready(key: &str) -> Command {
    Command::Ready {
        issue: issue(key),
        pull: Some("57".into()),
        pulls: BTreeMap::new(),
        replies: Vec::new(),
    }
}

fn by(agent: &str) -> AgentId {
    AgentId::new(agent)
}

/// An agent started with no issue, as an investigator is.
fn investigator(agent: &str) -> Event {
    Event::AgentStarted {
        agent: AgentId::new(agent),
        group: GroupId::new("watch"),
        role: Role::Investigator,
        issue: None,
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    }
}

/// An agent of `group` started on `on`.
fn started_on(agent: &str, group: &str, role: Role, on: IssueId) -> Event {
    Event::AgentStarted {
        agent: AgentId::new(agent),
        group: GroupId::new(group),
        role,
        issue: Some(on),
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    }
}

/// Issues 1 and 2 of ankh being refined, refiner-1 on 1 and refiner-2 on 2.
fn two_refining_events() -> Vec<Event> {
    vec![
        group(&["m1"], 2),
        folder("f1"),
        folder("f2"),
        sync("1", "m1", 1, &[]),
        sync("2", "m1", 2, &[]),
        started("refiner-1", Role::Refiner, "1"),
        started("refiner-2", Role::Refiner, "2"),
    ]
}

fn two_refining() -> World {
    World::replay(&two_refining_events()).0
}

/// Issues 1 and 2 in review round 1 on pull requests 51 and 52: author-1 and
/// reviewer-1 on 1, author-2 and reviewer-2 on 2, each reviewer told.
fn two_in_review_events() -> Vec<Event> {
    let mut events = two_refining_events();
    for key in ["1", "2"] {
        events.push(planned(key));
        events.push(ended(&format!("refiner-{key}")));
    }
    for key in ["1", "2"] {
        events.push(started(&format!("author-{key}"), Role::Author, key));
        events.push(Event::PullRequestReady {
            issue: issue(key),
            pull: Some(format!("5{key}")),
            pulls: BTreeMap::new(),
            replies: Vec::new(),
        });
        events.push(started(&format!("reviewer-{key}"), Role::Reviewer, key));
    }
    events
}

fn two_in_review() -> World {
    World::replay(&two_in_review_events()).0
}

/// Issues 1 and 2 with the arbiter after three blocked rounds: arbiter-1 on 1
/// and arbiter-2 on 2.
fn two_with_the_arbiter() -> World {
    let mut w = two_in_review();
    for key in ["1", "2"] {
        for n in 1..=3 {
            w.apply(&Event::ReviewFinished {
                issue: issue(key),
                round: n,
                verdict: Verdict::Block {
                    points: vec!["the bell rings once too often".into()],
                },
                failed: None,
                open: None,
            });
            if n < 3 {
                w.apply(&Event::PullRequestReady {
                    issue: issue(key),
                    pull: None,
                    pulls: BTreeMap::new(),
                    replies: vec!["it rings nine times now".into()],
                });
            }
        }
        w.apply(&started(&format!("arbiter-{key}"), Role::Arbiter, key));
    }
    assert_eq!(stage(&w, "2"), Stage::Arbitration { round: 3 });
    w
}

#[test]
fn a_persons_assign_of_a_reviewer_moves_it_and_the_issue_it_left_gets_a_fresh_one_at_once() {
    let mut w = two_in_review();
    let events = w.events(None, &assign("2", "reviewer-1")).expect("taken");
    assert_eq!(
        events,
        vec![Event::Assigned {
            issue: issue("2"),
            agent: by("reviewer-1"),
            from: issue("1"),
        }]
    );
    assert_eq!(
        apply_all(&mut w, &events),
        vec![stop("reviewer-2"), brief("reviewer-1", "2"), round("1", 1)],
        "the reviewer ankh#2 had is unassigned and stopped, the moved one is sent its brief, \
         and ankh#1 gets a fresh reviewer"
    );
    let moved = agent(&w, "reviewer-1");
    assert_eq!(moved.issue, Some(issue("2")));
    assert_eq!(moved.moved_from, Some(issue("1")));
    assert!(moved.assigned() && !moved.unassigned && !moved.briefed);
    let had = agent(&w, "reviewer-2");
    assert!(had.unassigned && had.issue == Some(issue("2")));
    assert_eq!(
        w.running(&issue("2"), Role::Reviewer).map(|a| a.id.clone()),
        Some(by("reviewer-1"))
    );
    assert_eq!(w.running(&issue("1"), Role::Reviewer), None);
    // ankh#1's fresh reviewer uses none of its starts: reviewer-1 did not end.
    assert!(!w.agents.values().any(|a| a.ended_waited_on));
}

#[test]
fn a_moved_reviewers_verdict_waits_for_its_brief_and_its_old_issues_commands_are_refused() {
    let mut w = two_in_review();
    person(&mut w, &assign("2", "reviewer-1"));
    let why = w
        .events(Some(&by("reviewer-1")), &approve("2"))
        .expect_err("not briefed yet");
    assert_eq!(
        why,
        "a person moved reviewer-1 to ankh#2, and its brief for ankh#2 has not reached it yet; \
         review the whole change once it has"
    );
    assert_eq!(
        w.events(Some(&by("reviewer-1")), &approve("1")),
        Err(
            "review for ankh#1 is no longer reviewer-1's: a person moved reviewer-1 from it to \
             ankh#2; work on ankh#2"
                .into()
        )
    );
    assert_eq!(
        w.events(Some(&by("reviewer-2")), &approve("2")),
        Err(
            "review for ankh#2 is no longer reviewer-2's: a person unassigned reviewer-2 from \
             it, and its session is being stopped"
                .into()
        )
    );
    // Until the brief arrives the loop asks for it each round, and for no
    // next-round message, which would go to a reviewer that never heard the
    // issue.
    let missing = w.missing().start;
    assert!(missing.contains(&brief("reviewer-1", "2")), "{missing:?}");
    assert!(!missing.contains(&round("2", 1)), "{missing:?}");
    w.apply(&briefed("reviewer-1", "2"));
    assert!(agent(&w, "reviewer-1").briefed);
    assert!(w.issues[&issue("2")].told);
    assert!(!w.missing().start.contains(&brief("reviewer-1", "2")));
    let events = w
        .events(Some(&by("reviewer-1")), &approve("2"))
        .expect("taken after its brief");
    w.apply(&events[0]);
    assert!(w.issues[&issue("2")].review_ok);
    assert_eq!(
        w.issues[&issue("2")].reviews[0].reviewer,
        Some(by("reviewer-1"))
    );
}

#[test]
fn a_moved_agent_is_sent_the_brief_a_fresh_one_would_start_with_after_a_line_saying_so() {
    let mut w = two_in_review();
    person(&mut w, &assign("2", "reviewer-1"));
    let text = w.moved_brief(&by("reviewer-1")).expect("a brief");
    let fresh = w
        .launch(&GroupId::new("watch"), Role::Reviewer, &issue("2"), None)
        .expect("a fresh reviewer's launch")
        .brief
        .task;
    let line = "A person moved you from ankh#1 to ankh#2: you work on ankh#2 now, and your \
                commands for ankh#1 are refused from here on. Leave ankh#1 as it is. Your brief \
                for ankh#2 follows.\n\n";
    assert_eq!(text, format!("{line}{fresh}"));
    assert!(text.contains("pull request 52"), "{text}");
    assert_eq!(w.moved_brief(&by("reviewer-2")), None, "no person moved it");
}

#[test]
fn a_persons_assign_of_a_refiner_moves_it_and_its_plan_for_the_new_issue_is_taken_at_once() {
    let mut w = two_refining();
    assert_eq!(
        person(&mut w, &assign("2", "refiner-1")),
        vec![
            stop("refiner-2"),
            brief("refiner-1", "2"),
            reader(Role::Refiner, "1")
        ]
    );
    assert!(
        w.events(Some(&by("refiner-1")), &plan("1"))
            .expect_err("moved")
            .contains("a person moved refiner-1 from it to ankh#2")
    );
    assert!(
        w.events(Some(&by("refiner-2")), &plan("2"))
            .expect_err("unassigned")
            .contains("a person unassigned refiner-2 from it")
    );
    let events = w.events(Some(&by("refiner-1")), &plan("2")).expect("taken");
    apply_all(&mut w, &events);
    assert_eq!(stage(&w, "2"), Stage::Working, "planned and placed");
}

#[test]
fn a_persons_assign_of_an_arbiter_moves_it_and_its_ruling_on_the_new_issue_is_taken() {
    let mut w = two_with_the_arbiter();
    assert_eq!(
        person(&mut w, &assign("2", "arbiter-1")),
        vec![
            stop("arbiter-2"),
            brief("arbiter-1", "2"),
            reader(Role::Arbiter, "1")
        ]
    );
    let rule = |key: &str| Command::Rule {
        issue: issue(key),
        approve: false,
        reason: "nine strokes, not ten".into(),
    };
    assert!(
        w.events(Some(&by("arbiter-1")), &rule("1"))
            .expect_err("moved")
            .contains("a person moved arbiter-1 from it to ankh#2")
    );
    let events = w.events(Some(&by("arbiter-1")), &rule("2")).expect("taken");
    apply_all(&mut w, &events);
    assert_eq!(stage(&w, "2"), Stage::Fixing { round: 3 });
}

#[test]
fn an_assign_of_an_author_investigator_unknown_or_ended_agent_is_refused_with_its_reason() {
    let mut w = two_in_review();
    w.apply(&investigator("investigator-1"));
    assert_eq!(
        refused(&w, &assign("2", "author-1")),
        "author-1 is an author, and an author is not assigned: it holds its issue's work \
         folders and may be mid-edit in them, so moving it would leave two authors in one \
         folder. Unassigning an issue's author starts a fresh one in its folders: temet-work \
         unassign ankh#2 AUTHOR"
    );
    assert_eq!(
        refused(&w, &assign("2", "investigator-1")),
        "investigator-1 is an investigator, which works for its group on no issue, so it is \
         not assigned to one"
    );
    assert_eq!(
        refused(&w, &assign("2", "reviewer-9")),
        "reviewer-9 is not an agent this daemon started"
    );
    w.apply(&ended("reviewer-1"));
    assert_eq!(
        refused(&w, &assign("2", "reviewer-1")),
        "reviewer-1 has ended; only a running agent is assigned"
    );
}

#[test]
fn an_assign_of_an_agent_on_the_issue_or_of_another_group_or_project_is_refused() {
    let mut w = two_in_review();
    w.apply(&started_on(
        "reviewer-7",
        "guild",
        Role::Reviewer,
        issue("1"),
    ));
    w.apply(&started_on(
        "reviewer-8",
        "watch",
        Role::Reviewer,
        IssueId::new("unseen", "3"),
    ));
    assert_eq!(
        refused(&w, &assign("2", "reviewer-2")),
        "reviewer-2 is on ankh#2 already"
    );
    assert_eq!(
        refused(&w, &assign("2", "reviewer-7")),
        "reviewer-7 is of group guild, and ankh#2 is watch's: an agent is assigned only within \
         its own group"
    );
    assert_eq!(
        refused(&w, &assign("2", "reviewer-8")),
        "reviewer-8 works on unseen#3, of project unseen, and ankh#2 is of ankh: an agent is \
         assigned only within the project it works in"
    );
    assert_eq!(
        refused(&w, &assign("9", "reviewer-1")),
        "there is no issue ankh#9"
    );
}

#[test]
fn an_assign_of_a_role_the_issues_stage_does_not_need_now_names_the_role_it_needs() {
    let mut w = two_in_review();
    w.apply(&started("refiner-3", Role::Refiner, "1"));
    assert_eq!(
        refused(&w, &assign("2", "refiner-3")),
        format!(
            "ankh#2 is {}, so it needs a reviewer now, not a refiner",
            crate::text::stage(Stage::InReview { round: 1 })
        )
    );
    // With its approval in, ankh#2 waits on its checks and needs no agent.
    w.apply(&Event::ReviewFinished {
        issue: issue("2"),
        round: 1,
        verdict: Verdict::Approve,
        failed: None,
        open: None,
    });
    assert_eq!(
        refused(&w, &assign("2", "reviewer-1")),
        format!(
            "ankh#2 is {} and needs no agent now, so no reviewer is assigned to it",
            crate::text::stage(Stage::InReview { round: 1 })
        )
    );
    // Being refined, an issue needs a refiner.
    let mut w = two_refining();
    w.apply(&started("reviewer-3", Role::Reviewer, "1"));
    assert_eq!(
        refused(&w, &assign("2", "reviewer-3")),
        format!(
            "ankh#2 is {}, so it needs a refiner now, not a reviewer",
            crate::text::stage(Stage::Refining)
        )
    );
    // Being fixed, an issue needs its author, which is never assigned.
    let mut w = two_in_review();
    w.apply(&verdict(
        1,
        Verdict::Block {
            points: vec!["the bell rings once too often".into()],
        },
    ));
    assert_eq!(
        refused(&w, &assign("1", "reviewer-2")),
        format!(
            "ankh#1 is {}, so it needs an author now, not a reviewer",
            crate::text::stage(Stage::Fixing { round: 1 })
        )
    );
}

#[test]
fn an_assign_to_a_paused_issue_is_refused_whether_held_or_out_of_scope() {
    let mut w = two_in_review();
    w.apply(&Event::Held);
    assert_eq!(
        refused(&w, &assign("2", "reviewer-1")),
        "ankh#2 is paused while the daemon is held, and no work starts on it until a person \
         resumes the daemon"
    );
    let mut w = two_in_review();
    w.apply(&group(&["m2"], 2));
    assert_eq!(
        refused(&w, &assign("2", "reviewer-1")),
        "ankh#2 is paused: it left its group's scope, and no work starts on it until it is back"
    );
}

#[test]
fn an_assign_or_unassign_from_an_agent_is_refused_since_only_a_person_assigns() {
    let w = two_in_review();
    assert_eq!(
        w.events(Some(&by("author-1")), &assign("2", "reviewer-1")),
        Err(
            "assign is a person's: author-1 is an agent, and only a person assigns an agent \
             to an issue or unassigns one"
                .into()
        )
    );
    assert_eq!(
        w.events(Some(&by("reviewer-1")), &unassign("1", "reviewer-1")),
        Err(
            "unassign is a person's: reviewer-1 is an agent, and only a person assigns an \
             agent to an issue or unassigns one"
                .into()
        )
    );
}

#[test]
fn an_unassigned_author_is_stopped_at_once_and_a_fresh_one_starts_in_its_folders_once_it_has_ended()
{
    let mut w = working_on_one();
    w.apply(&started("author-1", Role::Author, "1"));
    assert_eq!(
        w.setting_number(&issue("1"), Some(Role::Author), "starts"),
        3
    );
    // Three unassigns, and a fourth fresh author still starts: an end a person
    // asked for uses up no start.
    for (old, next) in [
        ("author-1", "author-2"),
        ("author-2", "author-3"),
        ("author-3", "author-4"),
    ] {
        assert_eq!(person(&mut w, &unassign("1", old)), vec![stop(old)]);
        let why = w
            .events(Some(&by(old)), &ready("1"))
            .expect_err("taken off");
        assert_eq!(
            why,
            format!(
                "ready for ankh#1 is no longer {old}'s: a person unassigned {old} from it, and \
                 its session is being stopped"
            )
        );
        assert!(!w.waiting_on(agent(&w, old)), "nothing waits on it");
        assert_eq!(
            w.missing(),
            Missing {
                start: vec![stop(old)],
                given_up: Vec::new(),
            },
            "its stop is asked for again, and no author starts while it runs"
        );
        w.apply(&ended(old));
        assert!(!agent(&w, old).ended_waited_on, "its end uses no start");
        assert_eq!(
            w.missing(),
            Missing {
                start: vec![author("1", "f1")],
                given_up: Vec::new(),
            }
        );
        w.apply(&started(next, Role::Author, "1"));
    }
    assert_eq!(
        agent(&w, "author-4").held,
        vec![FolderId::new("f1")],
        "the fresh author holds the issue's folder"
    );
}

#[test]
fn an_unassigned_authors_replacement_works_on_its_branch_and_pull_request() {
    let mut w = two_in_review();
    w.apply(&verdict(
        1,
        Verdict::Block {
            points: vec!["the bell rings once too often".into()],
        },
    ));
    assert_eq!(
        person(&mut w, &unassign("1", "author-1")),
        vec![stop("author-1")]
    );
    w.apply(&ended("author-1"));
    assert_eq!(w.missing().start, vec![author("1", "f1")]);
    let launch = w
        .launch(
            &GroupId::new("watch"),
            Role::Author,
            &issue("1"),
            Some(&FolderId::new("f1")),
        )
        .expect("launch");
    assert_eq!(
        launch.brief.folder,
        Some(std::path::PathBuf::from("/srv/ankh/f1"))
    );
    assert!(
        launch.brief.task.contains("pull request 51"),
        "{}",
        launch.brief.task
    );
    assert!(
        launch.brief.task.contains("the bell rings once too often"),
        "{}",
        launch.brief.task
    );
}

#[test]
fn an_unassigned_reviewer_is_replaced_once_it_has_ended_unless_its_approval_is_in() {
    let mut w = two_in_review();
    assert_eq!(
        person(&mut w, &unassign("1", "reviewer-1")),
        vec![stop("reviewer-1")]
    );
    assert_eq!(
        w.missing().start,
        vec![stop("reviewer-1")],
        "no reviewer starts while it runs"
    );
    w.apply(&ended("reviewer-1"));
    assert_eq!(w.missing().start, vec![round("1", 1)]);

    // With its approval in, the issue waits on its checks: nothing starts.
    let mut w = two_in_review();
    w.apply(&verdict(1, Verdict::Approve));
    person(&mut w, &unassign("1", "reviewer-1"));
    w.apply(&ended("reviewer-1"));
    assert_eq!(w.missing(), Missing::default());
}

#[test]
fn an_unassigned_refiner_is_replaced_once_it_has_ended() {
    let mut w = two_refining();
    assert_eq!(
        person(&mut w, &unassign("1", "refiner-1")),
        vec![stop("refiner-1")]
    );
    assert!(
        w.events(Some(&by("refiner-1")), &plan("1"))
            .expect_err("taken off")
            .contains("a person unassigned refiner-1 from it")
    );
    assert_eq!(w.missing().start, vec![stop("refiner-1")]);
    w.apply(&ended("refiner-1"));
    assert_eq!(w.missing().start, vec![reader(Role::Refiner, "1")]);
}

#[test]
fn an_unassign_of_an_agent_not_on_the_issue_ended_or_unknown_is_refused_with_its_reason() {
    let mut w = two_in_review();
    w.apply(&investigator("investigator-1"));
    assert_eq!(
        refused(&w, &unassign("2", "reviewer-1")),
        "reviewer-1 is not on ankh#2; it works on ankh#1"
    );
    assert_eq!(
        refused(&w, &unassign("2", "reviewer-9")),
        "reviewer-9 is not an agent this daemon started"
    );
    assert_eq!(
        refused(&w, &unassign("2", "investigator-1")),
        "investigator-1 is an investigator, which works for its group on no issue, so it is not \
         unassigned from one"
    );
    assert_eq!(
        refused(&w, &unassign("9", "reviewer-2")),
        "there is no issue ankh#9"
    );
    person(&mut w, &unassign("1", "reviewer-1"));
    assert_eq!(
        refused(&w, &unassign("1", "reviewer-1")),
        "reviewer-1 was unassigned from ankh#1 already and is being stopped"
    );
    assert_eq!(
        refused(&w, &unassign("2", "reviewer-1")),
        "reviewer-1 is not on ankh#2: it was unassigned from ankh#1 and is being stopped"
    );
    assert_eq!(
        refused(&w, &assign("2", "reviewer-1")),
        "reviewer-1 was unassigned from ankh#1 by a person and is being stopped; assign a \
         running agent that works on an issue"
    );
    w.apply(&ended("reviewer-1"));
    assert_eq!(
        refused(&w, &unassign("1", "reviewer-1")),
        "reviewer-1 has ended; there is nothing to unassign"
    );
}

#[test]
fn an_unassign_works_on_a_paused_issue_since_a_stop_starts_no_work() {
    let mut w = two_in_review();
    w.apply(&Event::Held);
    assert_eq!(
        person(&mut w, &unassign("1", "author-1")),
        vec![stop("author-1")]
    );
    assert_eq!(w.missing().start, vec![stop("author-1")]);
}

#[test]
fn replaying_assigns_and_unassigns_gives_the_same_world_which_asks_again_for_stops_and_briefs() {
    let mut events = two_in_review_events();
    let mut w = World::replay(&events).0;
    for command in [assign("2", "reviewer-1"), unassign("1", "author-1")] {
        let logged = w.events(None, &command).expect("taken");
        apply_all(&mut w, &logged);
        events.extend(logged);
    }
    let (replayed, _) = World::replay(&events);
    assert_eq!(replayed, w);
    // A replay drops effects: the next round asks again for each unassigned
    // agent's stop, the moved reviewer's brief and ankh#1's fresh reviewer.
    let start = replayed.missing().start;
    for effect in [
        stop("author-1"),
        stop("reviewer-2"),
        brief("reviewer-1", "2"),
        round("1", 1),
    ] {
        assert!(start.contains(&effect), "{effect:?} in {start:?}");
    }
    assert!(replayed.events(Some(&by("author-1")), &ready("1")).is_err());
    assert!(
        replayed
            .events(Some(&by("reviewer-1")), &approve("2"))
            .is_err(),
        "still waiting for its brief"
    );
    assert!(
        replayed
            .events(Some(&by("reviewer-2")), &approve("2"))
            .is_err()
    );
}

#[test]
fn the_view_and_the_text_say_who_a_person_assigned_and_unassigned_and_who_may_be_assigned() {
    let mut w = two_in_review();
    let of = |v: &crate::View, key: &str| {
        v.issues
            .iter()
            .find(|i| i.id == format!("ankh#{key}"))
            .expect("listed")
            .assignable
            .clone()
    };
    let view = crate::View::new(&w, &BTreeMap::new());
    assert_eq!(of(&view, "1"), vec![by("reviewer-2")]);
    assert_eq!(of(&view, "2"), vec![by("reviewer-1")]);

    person(&mut w, &assign("2", "reviewer-1"));
    let view = crate::View::new(&w, &BTreeMap::new());
    let seen = |id: &str| {
        let a = view.agents.iter().find(|a| a.id == by(id)).expect("listed");
        (a.issue.clone(), a.assigned, a.unassigned)
    };
    assert_eq!(seen("reviewer-1"), (Some("ankh#2".into()), true, false));
    assert_eq!(seen("reviewer-2"), (Some("ankh#2".into()), false, true));
    assert_eq!(of(&view, "1"), vec![by("reviewer-1")], "it may go back");
    assert_eq!(of(&view, "2"), Vec::<AgentId>::new());

    let status = crate::text::text(&w, &BTreeMap::new());
    assert!(
        status.contains("  reviewer-1 on ankh#2, assigned by a person: idle (on morpork)\n"),
        "{status}"
    );
    assert!(
        status.contains(
            "  reviewer-2 unassigned from ankh#2 by a person, stopping: idle (on morpork)\n"
        ),
        "{status}"
    );
    let issue_text =
        crate::text::issue(&w, &issue("2"), &Err("not read".into())).expect("an issue");
    assert!(
        issue_text.contains("  reviewer-1 (reviewer) on ankh#2, assigned by a person: idle\n"),
        "{issue_text}"
    );
    assert!(
        issue_text.contains(
            "  reviewer-2 (reviewer) unassigned from ankh#2 by a person, stopping: idle\n"
        ),
        "{issue_text}"
    );
    assert!(
        issue_text.contains("  author-2 (author): idle\n"),
        "{issue_text}"
    );

    w.apply(&ended("reviewer-2"));
    let status = crate::text::text(&w, &BTreeMap::new());
    assert!(
        status.contains("  reviewer-2 unassigned from ankh#2 by a person: ended (on morpork)\n"),
        "{status}"
    );
    let view = crate::View::new(&w, &BTreeMap::new());
    assert!(
        !view
            .agents
            .iter()
            .any(|a| a.id == by("reviewer-2") && a.unassigned),
        "an ended one is no longer being stopped"
    );
}
