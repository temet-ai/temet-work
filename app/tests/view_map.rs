//! The daemon's view mapped onto the Project page, from the core's checked-in view fixtures (the
//! current one, the one from before decisions had costs, and the one from before the plan and
//! reviews had states), and the page with no daemon. No window and no socket.

use std::path::Path;

use temet_work_app::components::Intent;
use temet_work_app::components::stage_tag::Stage;
use temet_work_app::components::status_tag::Status;
use temet_work_app::composites::decision_row::UndoCost;
use temet_work_app::daemon::Problem;
use temet_work_app::pages::project::{Grouping, Project, ProjectView};
use temet_work_app::view_map::{self, ago, due, notice, project_view, show};
use temet_work_core::View;
use temet_work_core::ids::ProjectId;

const CURRENT: &str = include_str!("../../crates/core/tests/fixtures/view.json");
const BEFORE_COSTS: &str = include_str!("../../crates/core/tests/fixtures/view-156.json");
const BEFORE_STATES: &str = include_str!("../../crates/core/tests/fixtures/view-before-161.json");

/// Five minutes after the fixtures' latest change.
const NOW: u64 = 1_795_001_200_000;

fn parse(text: &str) -> View {
    serde_json::from_str(text).expect("the fixture parses")
}

fn watch(view: &View) -> ProjectView {
    project_view(view, Some(&ProjectId::new("watch")), NOW).expect("a project")
}

#[test]
fn the_first_project_is_shown_unless_another_is_named() {
    let view = parse(CURRENT);
    let first = project_view(&view, None, NOW).expect("a project");
    assert_eq!(first.project.as_ref(), "unseen");
    let missing = project_view(&view, Some(&ProjectId::new("nowhere")), NOW).expect("a project");
    assert_eq!(missing.project.as_ref(), "unseen");
    assert_eq!(watch(&view).project.as_ref(), "watch");
}

#[test]
fn the_side_nav_lists_every_project_under_its_area() {
    let page = watch(&parse(CURRENT));
    let names: Vec<_> = page
        .areas
        .iter()
        .map(|a| {
            (
                a.name.to_string(),
                a.projects
                    .iter()
                    .map(|p| p.name.to_string())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        names,
        [
            ("No area".to_string(), vec!["unseen".to_string()]),
            ("Ankh-Morpork".to_string(), vec!["watch".to_string()]),
        ]
    );
}

#[test]
fn the_title_and_facts_come_from_the_project() {
    let view = parse(CURRENT);
    let page = watch(&view);
    assert_eq!(page.title.as_ref(), "watch");
    let facts: Vec<_> = page
        .facts
        .iter()
        .map(|f| (f.label.to_string(), f.value.to_string()))
        .collect();
    assert_eq!(
        facts,
        [
            (
                "Issues".into(),
                "tracker.example.com/ankh-morpork/watch/issues".into()
            ),
            ("Code".into(), "git.example.com/ankh-morpork/watch".into()),
        ]
    );
    let unseen = project_view(&view, None, NOW).expect("a project");
    assert!(unseen.facts.is_empty(), "no address, so no fact");
}

#[test]
fn groups_list_their_agents_that_have_not_ended() {
    let page = watch(&parse(CURRENT));
    assert_eq!(page.groups.len(), 1);
    let group = &page.groups[0];
    assert_eq!(group.name.as_ref(), "Night Watch");
    assert_eq!(group.note.as_deref(), Some("The Clacks"));
    let agents: Vec<_> = group
        .agents
        .iter()
        .map(|a| {
            (
                a.role.to_string(),
                a.r#where.clone().map(|w| w.to_string()),
                a.status,
            )
        })
        .collect();
    assert_eq!(
        agents,
        [
            (
                "author".into(),
                Some("pseudopolis \u{b7} anthill".into()),
                Status::Running
            ),
            (
                "author".into(),
                Some("pseudopolis \u{b7} anthill".into()),
                Status::Failed
            ),
            ("refiner".into(), Some("pseudopolis".into()), Status::Idle),
            (
                "reviewer".into(),
                Some("pseudopolis \u{b7} anthill".into()),
                Status::Blocked
            ),
        ],
        "the ended author-5 is not listed"
    );
}

#[test]
fn what_the_view_does_not_hold_stays_empty() {
    let page = watch(&parse(CURRENT));
    assert!(page.machines.is_empty());
    assert!(page.limits.is_empty());
}

#[test]
fn milestones_carry_their_counts_due_dates_and_open_issues() {
    let page = watch(&parse(CURRENT));
    let ms: Vec<_> = page
        .milestones
        .iter()
        .map(|m| {
            (
                m.name.to_string(),
                m.done,
                m.total,
                m.due.clone().map(|d| d.to_string()),
                m.issues.len(),
            )
        })
        .collect();
    assert_eq!(
        ms,
        [
            ("Snuff".to_string(), 0, 0, None, 0),
            (
                "The Clacks".to_string(),
                2,
                6,
                Some("12 Dec".to_string()),
                4
            ),
            ("No milestone".to_string(), 0, 1, None, 1),
        ],
        "watch#5 names no milestone"
    );
    let clacks = &page.milestones[1];
    let first = &clacks.issues[0];
    assert_eq!(first.key.as_ref(), "1");
    assert_eq!(first.stage, Stage::Refining);
    assert_eq!(first.group, None);
    assert_eq!(first.agent, None);
    assert_eq!(first.updated.as_ref(), "12 min ago");
    let ringing = &clacks.issues[3];
    assert_eq!(ringing.key.as_ref(), "4");
    assert_eq!(ringing.stage, Stage::InReview);
    assert_eq!(ringing.group.as_deref(), Some("Night Watch"));
    assert_eq!(ringing.agent.as_deref(), Some("reviewer \u{b7} hex"));
    assert_eq!(ringing.updated.as_ref(), "9 min ago");
}

#[test]
fn needs_you_lists_open_decisions_costliest_first() {
    let view = parse(CURRENT);
    let page = watch(&view);
    let ids: Vec<_> = page.needs.iter().map(|n| n.id.to_string()).collect();
    assert_eq!(
        ids,
        ["3", "1"],
        "decision 2 is overturned and 4 is on an issue that is past its merge"
    );
    let first = &page.needs[0].decision;
    assert_eq!(first.cost, Some(UndoCost::High));
    assert_eq!(first.took.as_ref(), "merge");
    assert_eq!(first.by.as_ref(), "arbiter-6");
    assert_eq!(first.issue.as_deref(), Some("4"));
    assert_eq!(first.overturn_before.as_deref(), Some("its issue's merge"));
    assert!(
        first.weighed.is_empty() && first.why.is_none(),
        "Needs you draws the short row; the pane has what was weighed and why"
    );
    assert_eq!(page.needs[1].decision.cost, Some(UndoCost::Low));
    assert_eq!(
        page.need_project.get("3").map(|p| p.as_ref()),
        Some("watch")
    );
    assert_eq!(
        page.same_decision
            .get("3")
            .map(|(k, i)| (k.as_ref(), i.as_ref())),
        Some(("4", "3"))
    );
    // Seen from the other project, the same decisions name their project.
    let unseen = project_view(&view, None, NOW).expect("a project");
    assert_eq!(unseen.needs.len(), 2);
    assert_eq!(unseen.needs[0].decision.issue.as_deref(), Some("watch#4"));
    assert!(unseen.same_decision.is_empty());
}

#[test]
fn a_decision_with_no_cost_comes_after_those_with_one() {
    for text in [BEFORE_COSTS, BEFORE_STATES] {
        let page = watch(&parse(text));
        let ids: Vec<_> = page.needs.iter().map(|n| n.id.to_string()).collect();
        assert_eq!(
            ids,
            ["3", "1"],
            "1 has no cost; 2 is overturned; 4 is on a closed issue"
        );
        assert_eq!(page.needs[0].decision.cost, Some(UndoCost::High));
        assert_eq!(page.needs[1].decision.cost, None);
        assert_eq!(page.needs[1].decision.overturn_before, None);
    }
}

#[test]
fn an_issues_checks_keep_their_states() {
    let page = watch(&parse(CURRENT));
    let plan = page.details["5"].plan.as_ref().expect("a plan");
    let states: Vec<_> = plan.checks.iter().map(|c| c.status).collect();
    assert_eq!(
        states,
        [
            Some(Status::Passed),
            Some(Status::Running),
            Some(Status::Waiting),
            Some(Status::Failed)
        ]
    );
    assert_eq!(plan.checks[0].done_when.as_ref(), "the cobbles are clear");
    assert_eq!(
        plan.checks[0].proven_by.as_ref(),
        "walking the yard at dawn"
    );
    assert_eq!(plan.folder.as_deref(), Some("pseudopolis:yard-1"));
    assert_eq!(plan.merge, None, "the view holds no merge rule");
    // An issue with no plan has no plan tab.
    assert!(page.details["2"].plan.is_none());
}

#[test]
fn a_check_with_no_state_draws_no_status() {
    let page = watch(&parse(BEFORE_STATES));
    let plan = page.details["4"].plan.as_ref().expect("a plan");
    assert_eq!(plan.checks[0].status, None);
}

#[test]
fn an_issues_review_rounds_keep_their_points_and_replies() {
    let page = watch(&parse(CURRENT));
    let detail = &page.details["4"];
    assert_eq!(detail.round, 3);
    let rounds = detail.rounds.as_ref().expect("rounds");
    assert_eq!(rounds.len(), 2);
    assert_eq!(rounds[0].round, 1);
    assert_eq!(rounds[0].by.as_ref(), "reviewer \u{b7} hex");
    assert_eq!(rounds[0].status, Status::Blocked);
    let points: Vec<_> = rounds[0]
        .points
        .iter()
        .map(|p| {
            (
                p.text.to_string(),
                p.reply.clone().map(|r| r.to_string()),
                p.resolved,
            )
        })
        .collect();
    assert_eq!(
        points,
        [
            (
                "the bell is cracked".into(),
                Some("recast the bell".into()),
                true
            ),
            (
                "the rope is frayed".into(),
                Some("the rope stays: it was checked".into()),
                false
            ),
        ]
    );
    assert!(page.details["1"].rounds.is_none());
}

#[test]
fn an_older_review_with_bare_points_maps_with_open_points_and_an_approval() {
    let page = watch(&parse(BEFORE_STATES));
    let rounds = page.details["4"].rounds.as_ref().expect("rounds");
    assert_eq!(
        rounds[0].points[0].reply.as_deref(),
        Some("recast the bell")
    );
    assert!(rounds[0].points.iter().all(|p| !p.resolved));
    assert_eq!(rounds[1].status, Status::Passed);
    assert!(rounds[1].points.is_empty());
}

#[test]
fn an_issues_pull_request_and_decisions_come_from_the_view() {
    let page = watch(&parse(CURRENT));
    let detail = &page.details["4"];
    let pull = detail.pull.as_ref().expect("a pull request");
    assert_eq!(pull.number, 41);
    assert_eq!(pull.merged, Some(false));
    assert_eq!(pull.ci, Some(Status::Failed));
    assert!(pull.branch.is_none() && pull.lines.is_none() && pull.files.is_none());
    let decisions = detail.decisions.as_ref().expect("decisions");
    let ids: Vec<_> = decisions.iter().map(|d| d.id.to_string()).collect();
    assert_eq!(ids, ["2", "3"]);
    assert_eq!(
        decisions[0].decision.overturned.as_deref(),
        Some("Overturned to all of them: the guild pays by the bell")
    );
    assert!(decisions[1].decision.overturned.is_none());
    assert!(page.details["1"].decisions.is_none());
    assert!(page.details["1"].pull.is_none());
}

#[test]
fn a_project_with_no_milestones_maps_with_its_issues_under_none() {
    let mut view = parse(CURRENT);
    view.milestones.clear();
    for issue in &mut view.issues {
        issue.milestone = None;
    }
    let page = watch(&view);
    assert_eq!(page.milestones.len(), 1);
    assert_eq!(page.milestones[0].name.as_ref(), "No milestone");
    assert_eq!(page.milestones[0].issues.len(), 5);

    view.issues.clear();
    view.decisions.clear();
    let empty = watch(&view);
    assert!(empty.milestones.is_empty());
    assert!(empty.needs.is_empty());
    assert!(empty.details.is_empty());
}

#[test]
fn a_view_with_no_projects_maps_to_nothing() {
    let mut view = parse(CURRENT);
    view.projects.clear();
    assert!(project_view(&view, None, NOW).is_none());
}

#[test]
fn times_and_dates_read_as_the_window_writes_them() {
    let now = 10_000_000_000;
    assert_eq!(ago(now, now).as_ref(), "just now");
    assert_eq!(ago(now, now + 5_000).as_ref(), "just now");
    assert_eq!(ago(now, now - 60_000).as_ref(), "1 min ago");
    assert_eq!(ago(now, now - 3_600_000).as_ref(), "1 h ago");
    assert_eq!(ago(now, now - 2 * 86_400_000).as_ref(), "2 d ago");
    assert_eq!(due("2026-12-12").as_ref(), "12 Dec");
    assert_eq!(due("2027-01-05").as_ref(), "5 Jan");
    assert_eq!(due("next week").as_ref(), "next week");
}

#[test]
fn every_word_of_the_view_has_a_stage_and_a_status() {
    use temet_work_core::view::{StageWord, StateWord};
    assert_eq!(view_map::stage(StageWord::Fixing), Stage::Working);
    assert_eq!(view_map::stage(StageWord::Arbitration), Stage::WithArbiter);
    assert_eq!(view_map::stage(StageWord::Unknown), Stage::Refining);
    assert_eq!(view_map::agent_status(StateWord::Ended), None);
    assert_eq!(
        view_map::agent_status(StateWord::Unknown),
        Some(Status::Waiting)
    );
}

#[test]
fn with_no_daemon_the_page_is_blank_and_says_why_with_the_socket() {
    let path = Path::new("/run/user/1000/temet-work.sock");
    let mut page = Project::blank(None);
    // Before the first answer.
    show(&mut page, None, None, path, NOW);
    let waiting = page.notice().expect("a notice");
    assert_eq!(waiting.title.as_ref(), "Waiting for the daemon");
    assert!(page.is_blank());

    let problems = [
        Problem::NoSocket {
            path: path.to_path_buf(),
        },
        Problem::Refused {
            why: "not you".into(),
        },
        Problem::Unreadable { why: "eof".into() },
        Problem::NoAnswer {
            within: std::time::Duration::from_secs(5),
        },
    ];
    for problem in problems {
        show(&mut page, None, Some(&problem), path, NOW);
        assert!(page.is_blank(), "{problem}");
        assert!(page.view().milestones.is_empty() && page.view().areas.is_empty());
        let notice = page.notice().expect("a notice");
        assert_eq!(notice.intent, Intent::Danger);
        let lines: Vec<_> = notice.lines.iter().map(|l| l.to_string()).collect();
        assert_eq!(
            lines,
            [
                problem.to_string(),
                "Socket: /run/user/1000/temet-work.sock".to_string()
            ]
        );
    }
}

#[test]
fn a_stale_view_stays_on_the_page_with_a_notice_and_the_choices_stay() {
    let path = Path::new("/run/user/1000/temet-work.sock");
    let view = parse(CURRENT);
    let mut page = Project::blank(None);
    show(&mut page, Some(&view), None, path, NOW);
    assert!(!page.is_blank());
    assert!(page.notice().is_none());
    page.select(&"4".into());
    page.set_grouping(Grouping::Stage);

    let problem = Problem::NoAnswer {
        within: std::time::Duration::from_secs(5),
    };
    show(&mut page, Some(&view), Some(&problem), path, NOW);
    assert!(!page.is_blank());
    assert_eq!(page.view().project.as_ref(), "unseen");
    let notice = page.notice().expect("a notice");
    assert_eq!(notice.intent, Intent::Warning);
    assert_eq!(notice.lines[0].as_ref(), "no answer within 5 s");
    assert_eq!(page.selected().map(|k| k.as_ref()), Some("4"));
    assert_eq!(page.grouping(), Grouping::Stage);

    // A new view replaces the old and clears the notice; the choices still stay.
    let mut newer = view.clone();
    newer.issues.pop();
    show(&mut page, Some(&newer), None, path, NOW);
    assert!(page.notice().is_none());
    assert_eq!(page.selected().map(|k| k.as_ref()), Some("4"));
    assert_eq!(page.grouping(), Grouping::Stage);
}

#[test]
fn a_view_with_no_projects_says_so() {
    let mut view = parse(CURRENT);
    view.projects.clear();
    let path = Path::new("/x.sock");
    let n = notice(Some(&view), None, path).expect("a notice");
    assert_eq!(n.title.as_ref(), "The daemon has no projects");
    let mut page = Project::blank(None);
    show(&mut page, Some(&view), None, path, NOW);
    assert!(page.is_blank());
}
