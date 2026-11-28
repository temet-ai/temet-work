//! The daemon's view mapped onto the Project page, from the core's checked-in view fixtures (the
//! current one, the one from before decisions had costs, and the one from before the plan and
//! reviews had states), and the page with no daemon. No window and no socket.

use std::path::Path;

use temet_work_app::components::Intent;
use temet_work_app::components::icon::IconName;
use temet_work_app::components::stage_tag::Stage;
use temet_work_app::components::status_tag::Status;
use temet_work_app::composites::decision_row::UndoCost;
use temet_work_app::daemon::Problem;
use temet_work_app::pages::project::{FactValue, Grouping, Project, ProjectView};
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
fn a_project_in_no_area_is_listed_at_the_top_level_with_its_open_decisions() {
    let page = watch(&parse(CURRENT));
    let areas: Vec<_> = page
        .areas
        .iter()
        .map(|a| {
            (
                a.name.to_string(),
                a.projects
                    .iter()
                    .map(|p| (p.name.to_string(), p.waiting))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        areas,
        [("Ankh-Morpork".to_string(), vec![("watch".to_string(), 0)])],
        "watch is on screen: Needs you counts its decisions, not a badge"
    );
    let loose: Vec<_> = page
        .projects
        .iter()
        .map(|p| (p.name.to_string(), p.icon, p.waiting))
        .collect();
    assert_eq!(
        loose,
        [("unseen".to_string(), Some(IconName::GitRepo), 0)],
        "unseen is in no area: at the top level, not under a No area row"
    );
    // Seen from unseen, watch counts the decisions that wait on a person: 3 and 1, as 2 is
    // overturned and 4 is past its merge.
    let unseen = project_view(&parse(CURRENT), None, NOW).expect("a project");
    assert_eq!(unseen.areas[0].projects[0].waiting, 2);
    assert_eq!(unseen.projects[0].waiting, 0, "unseen is on screen");
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
                "tracker.example.com ankh-morpork/watch".into()
            ),
            ("Code".into(), "git.example.com/ankh-morpork/watch".into()),
        ]
    );
    let unseen = project_view(&view, None, NOW).expect("a project");
    assert!(unseen.facts.is_empty(), "no address, so no fact");
    // A GitHub project's issues read as the radar page writes them.
    let mut github = view.clone();
    github.projects[1].issues_url = Some("https://github.com/ankh-morpork/watch/issues".into());
    assert_eq!(
        watch(&github).facts[0].value.as_ref(),
        "GitHub ankh-morpork/watch"
    );
}

#[test]
fn groups_list_their_running_agents_by_id_and_issue_and_count_the_ended_ones() {
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
                a.detail.clone().map(|d| d.to_string()),
                a.status,
            )
        })
        .collect();
    assert_eq!(
        agents,
        [
            (
                "author-2".into(),
                Some("watch#3 \u{b7} anthill".into()),
                Some("working: winding the clock \u{b7} pseudopolis \u{b7} anthill".into()),
                Status::Running
            ),
            (
                "author-4".into(),
                Some("watch#5 \u{b7} anthill".into()),
                Some("error: usage limit reached \u{b7} pseudopolis \u{b7} anthill".into()),
                Status::Failed
            ),
            (
                "refiner-1".into(),
                Some("watch#2".into()),
                Some("idle \u{b7} pseudopolis".into()),
                Status::Idle
            ),
            (
                "reviewer-3".into(),
                Some("watch#4 \u{b7} anthill".into()),
                Some("blocked: permission \u{b7} pseudopolis \u{b7} anthill".into()),
                Status::Blocked
            ),
        ],
        "the ended author-5 is not listed"
    );
    assert_eq!(
        group.empty.as_deref(),
        Some("No agents running \u{b7} 1 ended"),
        "shown only when no agent runs"
    );
    // With every agent ended, the group says how many have.
    let mut view = parse(CURRENT);
    for agent in &mut view.agents {
        agent.state = temet_work_core::view::StateWord::Ended;
    }
    let page = watch(&view);
    assert!(page.groups[0].agents.is_empty());
    assert_eq!(
        page.groups[0].empty.as_deref(),
        Some("No agents running \u{b7} 5 ended")
    );
}

#[test]
fn the_stage_counts_have_every_stage_with_done_from_the_view() {
    let page = watch(&parse(CURRENT));
    let counts: Vec<_> = page.stages.iter().map(|(s, n)| (s.name(), *n)).collect();
    assert_eq!(
        counts,
        [
            ("new", 1),
            ("refining", 1),
            ("ready", 0),
            ("working", 1),
            ("in review", 1),
            ("fixing", 1),
            ("with arbiter", 0),
            ("merging", 0),
            ("deploying", 0),
            ("verifying", 0),
            ("done", 2),
        ],
        "all 11 stages in the daemon's order; unseen#1 is another project's"
    );
}

#[test]
fn milestones_with_open_issues_carry_their_counts_due_dates_and_issues() {
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
                m.taken,
            )
        })
        .collect();
    assert_eq!(
        ms,
        [
            (
                "The Clacks".to_string(),
                2,
                6,
                Some("12 Dec".to_string()),
                4,
                true
            ),
            ("No milestone".to_string(), 0, 1, None, 1, true),
        ],
        "Snuff has nothing open, so it is not listed; watch#5 names no milestone"
    );
    let clacks = &page.milestones[0];
    let first = &clacks.issues[0];
    assert_eq!(
        first.key.as_ref(),
        "watch#1",
        "the whole id, as commands take it"
    );
    assert_eq!(first.stage, Stage::New);
    assert_eq!(first.group, None);
    assert_eq!(first.milestone.as_deref(), Some("The Clacks"));
    assert_eq!(first.agent, None);
    assert_eq!(first.updated.as_ref(), "12 min ago");
    let ringing = &clacks.issues[3];
    assert_eq!(ringing.key.as_ref(), "watch#4");
    assert_eq!(ringing.stage, Stage::InReview);
    assert_eq!(ringing.group.as_deref(), Some("Night Watch"));
    assert_eq!(ringing.agent.as_deref(), Some("reviewer-3"));
    assert_eq!(ringing.updated.as_ref(), "9 min ago");
}

/// The fixture with The Clacks' watch#1 and watch#4 moved to Snuff, which Night Watch does not
/// take; watch#4 is held by Night Watch.
fn with_snuff() -> View {
    let mut view = parse(CURRENT);
    for issue in &mut view.issues {
        if issue.id == "watch#1" || issue.id == "watch#4" {
            issue.milestone = Some(temet_work_core::ids::MilestoneId::new("Snuff"));
        }
    }
    view
}

#[test]
fn a_milestone_no_group_takes_says_so_and_how_many_a_group_holds_out_of_scope() {
    let page = watch(&with_snuff());
    let notes: Vec<_> = page
        .milestones
        .iter()
        .map(|m| (m.name.to_string(), m.note.clone().map(|n| n.to_string())))
        .collect();
    assert_eq!(
        notes,
        [
            (
                "Snuff".to_string(),
                Some("no group takes it \u{b7} 1 held by Night Watch, out of scope".to_string())
            ),
            ("The Clacks".to_string(), None),
            ("No milestone".to_string(), None),
        ]
    );
    // With no issue held, the note says only that no group takes it.
    let mut view = with_snuff();
    for issue in &mut view.issues {
        if issue.id == "watch#4" {
            issue.group = None;
        }
    }
    assert_eq!(
        watch(&view).milestones[0].note.as_deref(),
        Some("no group takes it")
    );
}

#[test]
fn the_board_has_a_lane_per_milestone_closed_when_no_group_takes_it() {
    let page = watch(&with_snuff());
    let lanes: Vec<_> = page
        .lanes
        .iter()
        .map(|l| {
            (
                l.title.to_string(),
                l.default_open,
                l.note.clone().map(|n| n.to_string()),
                l.issues
                    .iter()
                    .map(|i| i.key.to_string())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        lanes,
        [
            (
                "Snuff".to_string(),
                false,
                Some("no group takes it \u{b7} 1 held by Night Watch, out of scope".to_string()),
                vec!["watch#1".to_string(), "watch#4".to_string()]
            ),
            (
                "The Clacks".to_string(),
                true,
                None,
                vec!["watch#2".to_string(), "watch#3".to_string()]
            ),
            (
                "No milestone".to_string(),
                true,
                None,
                vec!["watch#5".to_string()]
            ),
        ]
    );
}

#[test]
fn each_card_has_its_open_decisions_and_running_agents() {
    let page = watch(&parse(CURRENT));
    let cards: Vec<_> = page
        .lanes
        .iter()
        .flat_map(|l| &l.issues)
        .map(|c| {
            (
                c.key.to_string(),
                c.stage,
                c.decisions,
                c.agents
                    .iter()
                    .map(|a| (a.label.to_string(), a.status))
                    .collect::<Vec<_>>(),
                c.updated.clone().map(|u| u.to_string()),
            )
        })
        .collect();
    assert_eq!(
        cards,
        [
            (
                "watch#1".to_string(),
                Stage::New,
                0,
                vec![],
                Some("12 min ago".to_string())
            ),
            (
                "watch#2".to_string(),
                Stage::Refining,
                0,
                vec![("refiner-1".to_string(), Status::Idle)],
                Some("11 min ago".to_string())
            ),
            (
                "watch#3".to_string(),
                Stage::Working,
                1,
                vec![("author-2".to_string(), Status::Running)],
                Some("10 min ago".to_string())
            ),
            (
                "watch#4".to_string(),
                Stage::InReview,
                1,
                vec![("reviewer-3".to_string(), Status::Blocked)],
                Some("9 min ago".to_string())
            ),
            (
                "watch#5".to_string(),
                Stage::Fixing,
                0,
                vec![("author-4".to_string(), Status::Failed)],
                Some("8 min ago".to_string())
            ),
        ],
        "watch#4's decision 2 is overturned, so one of its two is open"
    );
}

#[test]
fn needs_you_lists_open_decisions_costliest_first_as_on_main() {
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
    assert_eq!(first.issue.as_deref(), Some("watch#4"));
    assert_eq!(first.overturn_before.as_deref(), Some("its issue's merge"));
    assert!(
        first.question.is_none() && first.weighed.is_empty() && first.why.is_none(),
        "Needs you draws the short row; the pane has the question and why"
    );
    assert_eq!(page.needs[1].decision.cost, Some(UndoCost::Low));
    // Every project's open decisions, as on main.
    let unseen = project_view(&view, None, NOW).expect("a project");
    assert_eq!(unseen.needs.len(), 2);
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
fn each_decision_in_the_pane_has_its_question_and_the_other_options_numbered_from_1() {
    let page = watch(&parse(CURRENT));
    let decisions = page.details["watch#4"]
        .decisions
        .as_ref()
        .expect("decisions");
    let ids: Vec<_> = decisions.iter().map(|d| d.id.to_string()).collect();
    assert_eq!(ids, ["3", "2"], "the costliest to undo first");
    let merge = &decisions[0].decision;
    assert_eq!(
        merge.question.as_deref(),
        Some("Does watch#4 merge after 3 review rounds?")
    );
    assert_eq!(merge.took.as_ref(), "merge");
    assert_eq!(merge.by.as_ref(), "arbiter-6");
    assert_eq!(merge.cost, Some(UndoCost::High));
    assert_eq!(merge.overturn_before.as_deref(), Some("merge"));
    let others: Vec<_> = merge
        .options
        .iter()
        .map(|o| (o.id.to_string(), o.label.to_string()))
        .collect();
    assert_eq!(others, [("2".to_string(), "fix first".to_string())]);
    assert_eq!(merge.overturned, None);

    let bells = &decisions[1].decision;
    assert_eq!(bells.took.as_ref(), "two", "option 2 of 3 was chosen");
    assert_eq!(bells.why.as_deref(), Some("it is easier to undo"));
    let others: Vec<_> = bells
        .options
        .iter()
        .map(|o| (o.id.to_string(), o.label.to_string()))
        .collect();
    assert_eq!(
        others,
        [
            ("1".to_string(), "one bell".to_string()),
            ("3".to_string(), "all of them".to_string())
        ],
        "numbered as the daemon counts them, the chosen one left out"
    );
    assert_eq!(bells.overturned.as_deref(), Some("Overturned"));
    assert_eq!(bells.overturned_to.as_deref(), Some("all of them"));
    assert_eq!(
        bells.overturn_reason.as_deref(),
        Some("the guild pays by the bell")
    );
}

#[test]
fn the_panes_facts_are_group_waits_for_pull_request_and_tracker() {
    let page = project_view(&parse(CURRENT), None, NOW).expect("a project");
    let facts = |page: &ProjectView, key: &str| -> Vec<(String, FactValue)> {
        page.details[key]
            .facts
            .iter()
            .map(|f| (f.label.to_string(), f.value.clone()))
            .collect()
    };
    let text = |t: &str| FactValue::Text(t.to_string().into());
    assert_eq!(
        facts(&page, "unseen#1"),
        [
            ("Group".to_string(), text("Night Watch")),
            (
                "Waits for".to_string(),
                FactValue::Keys(vec!["watch#3".into()])
            ),
            (
                "Tracker".to_string(),
                FactValue::Link {
                    text: "unseen/university#1".into(),
                    url: None
                }
            ),
        ],
        "the milestone, agents and last change each have their own place on the screen"
    );
    let page = watch(&parse(CURRENT));
    assert_eq!(
        facts(&page, "watch#4"),
        [
            ("Group".to_string(), text("Night Watch")),
            (
                "Pull request".to_string(),
                FactValue::Pull {
                    number: 41,
                    checks: Some(false)
                }
            ),
            (
                "Tracker".to_string(),
                FactValue::Link {
                    text: "ankh-morpork/watch#4".into(),
                    url: Some("https://tracker.example.com/ankh-morpork/watch/issues/4".into())
                }
            ),
        ]
    );
    assert_eq!(
        page.details["watch#1"].facts[0].value,
        text("None yet"),
        "an issue no group has taken"
    );
}

#[test]
fn the_deadline_every_decision_in_needs_you_shares_is_the_panes_to_leave_out() {
    let page = watch(&parse(CURRENT));
    assert_eq!(page.shared_deadline.as_deref(), Some("merge"));
    // A ruling from an older log has no deadline, so none is shared.
    assert_eq!(watch(&parse(BEFORE_COSTS)).shared_deadline, None);
}

#[test]
fn an_issues_checks_keep_their_states() {
    let page = watch(&parse(CURRENT));
    let plan = page.details["watch#5"].plan.as_ref().expect("a plan");
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
    assert_eq!(plan.merge, None, "the view holds no merge rule");
    // An issue with no plan has no plan tab.
    assert!(page.details["watch#2"].plan.is_none());
}

#[test]
fn a_check_with_no_state_draws_no_status() {
    let page = watch(&parse(BEFORE_STATES));
    let plan = page.details["watch#4"].plan.as_ref().expect("a plan");
    assert_eq!(plan.checks[0].status, None);
}

#[test]
fn an_issues_review_rounds_keep_their_points_and_replies() {
    let page = watch(&parse(CURRENT));
    let detail = &page.details["watch#4"];
    assert_eq!(detail.round, 3);
    let rounds = detail.rounds.as_ref().expect("rounds");
    assert_eq!(rounds.len(), 2);
    assert_eq!(rounds[0].round, 1);
    assert_eq!(rounds[0].by.as_ref(), "reviewer-3");
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
    assert!(page.details["watch#1"].rounds.is_none());
}

#[test]
fn an_older_review_with_bare_points_maps_with_open_points_and_an_approval() {
    let page = watch(&parse(BEFORE_STATES));
    let rounds = page.details["watch#4"].rounds.as_ref().expect("rounds");
    assert_eq!(
        rounds[0].points[0].reply.as_deref(),
        Some("recast the bell")
    );
    assert!(rounds[0].points.iter().all(|p| !p.resolved));
    assert_eq!(rounds[1].status, Status::Passed);
    assert!(rounds[1].points.is_empty());
}

#[test]
fn an_issues_pull_request_comes_from_the_view() {
    let page = watch(&parse(CURRENT));
    let detail = &page.details["watch#4"];
    let pull = detail.pull.as_ref().expect("a pull request");
    assert_eq!(pull.number, 41);
    assert_eq!(pull.merged, Some(false));
    assert_eq!(pull.ci, Some(Status::Failed));
    assert!(pull.branch.is_none() && pull.lines.is_none() && pull.files.is_none());
    assert!(page.details["watch#1"].decisions.is_none());
    assert!(page.details["watch#1"].pull.is_none());
}

#[test]
fn the_daemons_problems_are_listed() {
    let page = watch(&parse(CURRENT));
    let problems: Vec<_> = page
        .problems
        .iter()
        .map(|p| (p.name.to_string(), p.text.to_string()))
        .collect();
    assert_eq!(
        problems,
        [("tracker".to_string(), "the clacks are down".to_string())]
    );
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
    assert_eq!(page.lanes.len(), 1);

    view.issues.clear();
    view.decisions.clear();
    let empty = watch(&view);
    assert!(empty.milestones.is_empty());
    assert!(empty.lanes.is_empty());
    assert!(empty.needs.is_empty());
    assert!(empty.details.is_empty());
    assert_eq!(empty.shared_deadline, None);
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

/// Every issue of `view`, by its id, with the stage the page shows for it and the stage word the
/// view gives it.
fn stages_shown(view: &View) -> Vec<(String, Stage, String)> {
    let mut out = Vec::new();
    for project in &view.projects {
        let page = project_view(view, Some(&project.id), NOW).expect("a project");
        for issue in page.milestones.iter().flat_map(|m| &m.issues) {
            let id = issue.key.to_string();
            let word = view
                .issues
                .iter()
                .find(|i| i.id == id)
                .map(|i| serde_json::to_value(i.stage).expect("a word"))
                .and_then(|w| w.as_str().map(str::to_string))
                .expect("the issue is in the view");
            out.push((id, issue.stage, word));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn each_issue_of_the_fixtures_shows_its_own_stage() {
    for (name, fixture) in [
        ("current", CURRENT),
        ("before costs", BEFORE_COSTS),
        ("before states", BEFORE_STATES),
    ] {
        let shown: Vec<_> = stages_shown(&parse(fixture))
            .into_iter()
            .map(|(id, stage, word)| (id, stage.name(), word))
            .collect();
        assert_eq!(
            shown,
            [
                ("unseen#1".to_string(), "ready", "ready".to_string()),
                ("watch#1".to_string(), "new", "new".to_string()),
                ("watch#2".to_string(), "refining", "refining".to_string()),
                ("watch#3".to_string(), "working", "working".to_string()),
                ("watch#4".to_string(), "in review", "in_review".to_string()),
                ("watch#5".to_string(), "fixing", "fixing".to_string()),
            ],
            "{name}"
        );
    }
}

#[test]
fn each_stage_word_of_the_view_has_its_own_stage() {
    use temet_work_core::view::StageWord;
    // The daemon's words, as the view writes them, and the stage of the same name.
    let words = [
        ("new", "new"),
        ("refining", "refining"),
        ("ready", "ready"),
        ("working", "working"),
        ("in_review", "in review"),
        ("fixing", "fixing"),
        ("arbitration", "with arbiter"),
        ("merging", "merging"),
        ("deploying", "deploying"),
        ("verifying", "verifying"),
        ("done", "done"),
    ];
    let mut stages = Vec::new();
    for (word, name) in words {
        let parsed: StageWord =
            serde_json::from_value(serde_json::Value::String(word.into())).expect("a stage word");
        assert_ne!(parsed, StageWord::Unknown, "{word} is a word of the view");
        let stage = view_map::stage(parsed);
        assert_eq!(stage.name(), name, "{word}");
        stages.push(stage);
    }
    assert_eq!(
        stages,
        Stage::ALL,
        "every stage, once each, in the daemon's order"
    );
    assert_eq!(
        view_map::stage(StageWord::Unknown),
        Stage::New,
        "a word this build lacks reads as the first stage"
    );
}

#[test]
fn every_state_word_of_the_view_has_a_status() {
    use temet_work_core::view::StateWord;
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
    page.select(&"unseen#1".into());
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
    assert_eq!(page.selected().map(|k| k.as_ref()), Some("unseen#1"));
    assert_eq!(page.grouping(), Grouping::Stage);

    // A new view replaces the old and clears the notice; the choices still stay.
    let mut newer = view.clone();
    newer.issues.pop();
    show(&mut page, Some(&newer), None, path, NOW);
    assert!(page.notice().is_none());
    assert_eq!(page.selected().map(|k| k.as_ref()), Some("unseen#1"));
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

#[test]
fn the_first_view_opens_its_first_issue_and_the_persons_choices_stay_through_refreshes() {
    use temet_work_app::composites::issue_pane::IssueTab;
    use temet_work_app::pages::project::ViewMode;
    let path = Path::new("/run/user/1000/temet-work.sock");
    let view = parse(CURRENT);
    let mut page = Project::blank(None);
    show(&mut page, None, None, path, NOW);
    assert_eq!(page.selected(), None);
    show(&mut page, Some(&view), None, path, NOW);
    assert_eq!(
        page.selected().map(|k| k.as_ref()),
        Some("unseen#1"),
        "the first issue listed"
    );
    assert_eq!(page.tab(), Some(IssueTab::Plan), "it has no open decision");
    assert_eq!(page.mode(), ViewMode::List, "the window opens on List");

    page.set_mode(ViewMode::Board);
    page.close();
    for _ in 0..3 {
        show(&mut page, Some(&view), None, path, NOW);
    }
    assert_eq!(page.selected(), None, "a closed pane stays closed");
    assert_eq!(page.mode(), ViewMode::Board);
}
