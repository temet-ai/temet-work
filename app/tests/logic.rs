//! The app's logic, without a window: the sample in the shape of the radar page's data, the
//! page's choices, the issue table's columns, a milestone's subtitle, the pane's count line, and
//! the icons and faces the parts need, and the `refine` a RefineDialog's Save sends.

use std::collections::HashSet;

use gpui::{SharedString, px};
use temet_work_app::assets::FILES;
use temet_work_app::components::icon::{IconName, IconSize};
use temet_work_app::components::stage_tag::{Stage, StageTag};
use temet_work_app::components::status_tag::Status;

use temet_work_app::composites::decision_row::{Decision, UndoCost};
use temet_work_app::composites::issue_pane::{IssueTab, closes, one_deadline, tabs_shown};
use temet_work_app::composites::issue_table::{
    Columns, GroupBy, Room, colgroup, count, grouped, optional_columns, room,
};
use temet_work_app::composites::milestone_panel::{progress, subtitle};
use temet_work_app::composites::refine_dialog::RefineCheck;
use temet_work_app::pages::project::{
    Grouping, Project, ViewMode, all_issues_summary, first_tab, refine_command,
};
use temet_work_app::sample;
use temet_work_app::theme;
use temet_work_app::theme::size::IssueColumns;
use temet_work_core::Command;
use temet_work_core::ids::IssueId;
use temet_work_core::model::Check;

fn columns() -> IssueColumns {
    IssueColumns {
        key: px(96.),
        stage: px(120.),
        milestone: px(88.),
        agent: px(124.),
        updated: px(88.),
        title_min: px(272.),
        title_narrow: px(180.),
    }
}

#[test]
fn the_sample_has_the_shape_of_the_radar_pages_data() {
    let view = sample::project_view();
    let issues: Vec<_> = view.milestones.iter().flat_map(|m| &m.issues).collect();
    assert_eq!(issues.len(), 53, "open issues");
    let backlog = view
        .milestones
        .iter()
        .find(|m| m.name.as_ref() == "Backlog")
        .expect("Backlog");
    assert_eq!(backlog.issues.len(), 43);
    assert!(!backlog.taken, "no group takes Backlog");
    assert_eq!(
        backlog.note.as_deref(),
        Some("no group takes it \u{b7} 18 held by grand-trunk, out of scope")
    );
    let names: Vec<&str> = view.milestones.iter().map(|m| m.name.as_ref()).collect();
    assert_eq!(names, ["0.4", "0.5", "Backlog"], "0.3 has nothing open");
    // Needs you lists only decisions high to undo, and the sample's 33 are low or medium.
    assert!(
        view.needs.is_empty(),
        "no decision in the sample is high to undo"
    );
    assert_eq!(view.stages.last(), Some(&(Stage::Done, 91)));
    assert_eq!(view.areas.len(), 0, "no areas");
    assert_eq!(view.projects.len(), 1);
    assert_eq!(
        view.projects[0].waiting, 0,
        "the project on screen: Needs you counts its decisions"
    );
    assert_eq!(
        view.groups[0].empty.as_deref(),
        Some("No agents running \u{b7} 324 ended")
    );
    assert!(view.groups[0].agents.is_empty());
    assert_eq!(
        view.shared_deadline, None,
        "Needs you is empty, so no deadline is shared"
    );
    assert!(view.problems.is_empty(), "no daemon answers, so none");
    let keys: HashSet<_> = issues.iter().map(|i| &i.key).collect();
    assert_eq!(keys.len(), issues.len(), "keys are unique");
    let selected = sample::selected().expect("an issue starts open");
    assert!(keys.contains(&selected));
}

#[test]
fn with_the_daemon_running_seven_agents_work_and_two_new_issues_are_refining() {
    let view = sample::running_view();
    let agents: Vec<_> = view.groups[0]
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
                "author-71".into(),
                Some("clacks#212 \u{b7} opus".into()),
                Status::Running
            ),
            (
                "investigator-6".into(),
                Some("clacks#212 \u{b7} sonnet".into()),
                Status::Running
            ),
            (
                "author-69".into(),
                Some("clacks#215 \u{b7} opus".into()),
                Status::Running
            ),
            (
                "reviewer-74".into(),
                Some("clacks#201 \u{b7} opus".into()),
                Status::Running
            ),
            (
                "reviewer-75".into(),
                Some("clacks#203 \u{b7} sonnet".into()),
                Status::Idle
            ),
            (
                "refiner-82".into(),
                Some("clacks#224 \u{b7} opus".into()),
                Status::Running
            ),
            (
                "refiner-83".into(),
                Some("clacks#226 \u{b7} sonnet".into()),
                Status::Blocked
            ),
        ]
    );
    let refining: Vec<_> = view
        .milestones
        .iter()
        .flat_map(|m| &m.issues)
        .filter(|i| i.stage == Stage::Refining)
        .map(|i| (i.key.to_string(), i.updated.to_string()))
        .collect();
    assert_eq!(
        refining,
        [
            ("clacks#224".to_string(), "just now".to_string()),
            ("clacks#226".to_string(), "just now".to_string())
        ]
    );
    assert_eq!(view.problems.len(), 2);
}

#[test]
fn each_milestone_lists_only_its_open_issues() {
    for m in sample::project_view().milestones {
        assert!(m.issues.len() <= m.total - m.done, "{}", m.name);
    }
}

#[test]
fn the_issues_and_code_facts_are_written_as_the_radar_page_writes_them() {
    let facts: Vec<_> = sample::project_view()
        .facts
        .iter()
        .map(|f| (f.label.to_string(), f.value.to_string()))
        .collect();
    assert_eq!(
        facts,
        [
            (
                "Issues".to_string(),
                "GitHub grand-trunk/clacks".to_string()
            ),
            (
                "Code".to_string(),
                "github.com/grand-trunk/clacks".to_string()
            )
        ]
    );
}

#[test]
fn opening_an_issue_picks_decisions_when_it_has_open_ones_and_plan_otherwise() {
    let view = sample::project_view();
    assert_eq!(
        first_tab(view.details.get(&SharedString::from("clacks#212"))),
        IssueTab::Decisions
    );
    assert_eq!(
        first_tab(view.details.get(&SharedString::from("clacks#210"))),
        IssueTab::Plan
    );
    let mut page = Project::new(view, sample::selected());
    assert_eq!(page.tab(), Some(IssueTab::Decisions));
    page.set_tab(IssueTab::Plan);
    assert!(page.select(&"clacks#210".into()));
    assert_eq!(page.tab(), Some(IssueTab::Plan));
    assert!(
        !page.select(&"clacks#210".into()),
        "the same issue changes nothing"
    );
    page.close();
    assert_eq!(page.selected(), None);
    assert_eq!(page.mode(), ViewMode::List, "the window opens on List");
}

#[test]
fn a_table_has_room_for_updated_then_agent_then_neither() {
    let c = columns();
    // 96 + 124 + 120 = 340 fixed, with the Stage column.
    assert_eq!(room(px(340. + 88. + 272.), &c, true, false), Room::Both);
    assert_eq!(room(px(340. + 88. + 271.), &c, true, false), Room::Agent);
    assert_eq!(room(px(340. + 180.), &c, true, false), Room::Agent);
    assert_eq!(room(px(340. + 179.), &c, true, false), Room::Neither);
    // The Milestone column takes its room too.
    assert_eq!(room(px(340. + 88. + 272.), &c, true, true), Room::Agent);
    assert_eq!(optional_columns(None, Room::Both), (true, true));
    assert_eq!(optional_columns(None, Room::Agent), (true, false));
    assert_eq!(optional_columns(None, Room::Neither), (false, false));
    // Without the Agent column by request, Updated has its room.
    assert_eq!(optional_columns(Some(false), Room::Neither), (false, true));
    let all = Columns {
        stage: true,
        milestone: true,
        agent: true,
        updated: true,
    };
    assert_eq!(
        colgroup(&c, all),
        vec![
            Some(px(96.)),
            None,
            Some(px(120.)),
            Some(px(88.)),
            Some(px(124.)),
            Some(px(88.))
        ]
    );
    // Grouped by stage, the table has no Stage column.
    assert_eq!(
        colgroup(
            &c,
            Columns {
                stage: false,
                milestone: false,
                ..all
            }
        ),
        vec![Some(px(96.)), None, Some(px(124.)), Some(px(88.))]
    );
}

#[test]
fn a_milestone_says_what_is_done_when_it_is_due_and_its_note() {
    assert_eq!(
        subtitle(Some(7), Some(12), Some("12 Dec"), None, None).as_deref(),
        Some("7 / 12 done · due 12 Dec")
    );
    assert_eq!(
        subtitle(Some(1), Some(44), None, None, Some("no group takes it")).as_deref(),
        Some("1 / 44 done · no group takes it")
    );
    assert_eq!(
        subtitle(None, None, Some("16 Jan"), None, None).as_deref(),
        Some("due 16 Jan")
    );
    assert_eq!(
        subtitle(Some(1), Some(2), None, Some("3 issues"), None).as_deref(),
        Some("3 issues")
    );
    assert_eq!(subtitle(None, None, None, None, None), None);
    // A milestone with no issues says only when it is due, whatever summary it is given.
    assert_eq!(
        subtitle(Some(0), Some(0), Some("20 Feb"), Some("3 issues"), None).as_deref(),
        Some("due 20 Feb")
    );
    assert_eq!(subtitle(Some(0), Some(0), None, None, None), None);
    assert_eq!(progress(0, 0), 0.);
    assert_eq!(progress(7, 14), 0.5);
}

#[test]
fn every_status_and_stage_has_the_design_systems_view_and_an_embedded_icon() {
    let names: Vec<&str> = Status::ALL.iter().map(|s| s.name()).collect();
    let map: Vec<&str> = theme::statuses().iter().map(|s| s.name).collect();
    assert_eq!(names, map, "the statuses, in the design system's order");
    let stages: Vec<&str> = Stage::ALL.iter().map(|s| s.name()).collect();
    let map: Vec<&str> = theme::stages().iter().map(|s| s.name).collect();
    assert_eq!(stages, map, "the stages, in order");
    for status in Status::ALL {
        let _ = status.icon();
    }
    for stage in Stage::ALL {
        let _ = stage.icon();
    }
}

#[test]
fn every_icon_is_embedded_on_both_grids() {
    let paths: HashSet<&str> = FILES.iter().map(|(p, _)| *p).collect();
    for icon in IconName::ALL {
        for size in [IconSize::Standard, IconSize::Large] {
            let path = icon.path(size);
            assert!(paths.contains(path.as_ref()), "{path} is not embedded");
        }
        assert_eq!(IconName::from_name(icon.name()), Some(icon));
    }
    assert_eq!(paths.len(), IconName::ALL.len() * 2);
}

#[test]
fn every_font_file_the_design_system_lists_is_embedded() {
    let embedded: HashSet<&str> = temet_work_app::assets::FONTS
        .iter()
        .map(|(p, _)| *p)
        .collect();
    let listed: HashSet<&str> = theme::font_files().collect();
    assert_eq!(embedded, listed);
}

#[test]
fn grouping_by_stage_runs_from_the_furthest_stage_and_by_group_by_first_appearance() {
    let issues: Vec<_> = sample::project_view()
        .milestones
        .into_iter()
        .flat_map(|m| m.issues)
        .collect();
    let by_stage = grouped(issues.clone(), GroupBy::Stage);
    let stages: Vec<(Stage, usize)> = by_stage
        .iter()
        .filter_map(|(s, _, these)| s.map(|s| (s, these.len())))
        .collect();
    assert_eq!(
        stages,
        [
            (Stage::Fixing, 1),
            (Stage::InReview, 2),
            (Stage::Working, 1),
            (Stage::Ready, 22),
            (Stage::New, 27)
        ],
        "the issues closest to shipping first"
    );
    let by_group = grouped(issues.clone(), GroupBy::Group);
    let names: Vec<(&str, usize)> = by_group
        .iter()
        .map(|(_, n, these)| (n.as_ref(), these.len()))
        .collect();
    assert_eq!(names, [("grand-trunk", 26), ("No group", 27)]);
    assert_eq!(count(1), "1 issue");
    assert_eq!(count(3), "3 issues");
}

#[test]
fn the_view_and_group_by_controls_map_to_what_the_middle_draws() {
    assert_eq!(ViewMode::from_value("board"), Some(ViewMode::Board));
    assert_eq!(ViewMode::from_value("list"), Some(ViewMode::List));
    assert_eq!(ViewMode::from_value("grid"), None);
    assert_eq!(Grouping::Milestone.table(), None);
    assert_eq!(Grouping::Stage.table(), Some(GroupBy::Stage));
    assert_eq!(Grouping::from_value("group"), Some(Grouping::Group));
    assert_eq!(Grouping::from_value("week"), None);
    let view = sample::project_view();
    assert_eq!(
        all_issues_summary(&view.milestones),
        "53 open in 0.4, 0.5, Backlog"
    );
}

#[test]
fn the_pane_shows_a_tab_for_each_part_an_issue_has() {
    assert_eq!(
        tabs_shown(true, true, true, true),
        [
            IssueTab::Plan,
            IssueTab::Pull,
            IssueTab::Review,
            IssueTab::Decisions
        ]
    );
    assert_eq!(tabs_shown(false, false, true, false), [IssueTab::Review]);
    assert!(tabs_shown(false, false, false, false).is_empty());
    let view = sample::project_view();
    let full = &view.details[&SharedString::from("clacks#212")];
    assert_eq!(full.plan.as_ref().map(|p| p.checks.len()), Some(12));
    assert_eq!(full.rounds.as_ref().map(Vec::len), Some(1));
    assert_eq!(full.decisions.as_ref().map(Vec::len), Some(11));
    let quiet = &view.details[&SharedString::from("clacks#228")];
    assert!(
        quiet.plan.is_none() && quiet.rounds.is_none() && quiet.decisions.is_none(),
        "a new issue has its facts only"
    );
}

/// A decision with a cost and a deadline, for the count line.
fn decision(cost: UndoCost, before: &str, overturned: bool) -> Decision {
    Decision {
        question: Some("How many towers?".into()),
        took: "8".into(),
        by: "refiner-66".into(),
        issue: None,
        at: None,
        cost: Some(cost),
        weighed: Vec::new(),
        why: None,
        options: Vec::new(),
        overturn_before: Some(before.to_string().into()),
        overturned: overturned.then(|| "Overturned".into()),
        overturned_to: None,
        overturn_reason: None,
    }
}

#[test]
fn a_deadline_every_open_decision_shares_is_said_by_none_of_them() {
    let mut ds = vec![decision(UndoCost::Medium, "merge", false)];
    ds.extend((0..2).map(|_| decision(UndoCost::Low, "merge", false)));
    assert_eq!(one_deadline(&ds).as_deref(), Some("merge"));
    ds.push(decision(UndoCost::Low, "deploy", false));
    ds.push(decision(UndoCost::High, "merge", true));
    assert_eq!(one_deadline(&ds), None);
    assert_eq!(
        one_deadline(&[decision(UndoCost::Low, "merge", false)]),
        None,
        "one decision keeps its own"
    );
}

#[test]
fn q_closes_the_pane_unless_a_text_field_has_the_focus_or_a_modifier_is_held() {
    assert!(closes("q", false, false));
    assert!(!closes("q", false, true), "typed into the field");
    assert!(
        !closes("q", true, false),
        "ctrl-q and the like are commands"
    );
    assert!(!closes("w", false, false));
}

#[test]
fn a_tag_shows_the_round_only_on_the_stages_that_count_rounds() {
    let text = |stage, round| StageTag::new(stage).round(round).text().to_string();
    assert_eq!(text(Stage::Fixing, Some(1)), "fixing, round 1");
    assert_eq!(text(Stage::InReview, Some(2)), "in review, round 2");
    assert_eq!(text(Stage::WithArbiter, Some(3)), "with arbiter, round 3");
    assert_eq!(
        text(Stage::InReview, Some(0)),
        "in review",
        "before the first round"
    );
    assert_eq!(text(Stage::InReview, None), "in review");
    assert_eq!(
        text(Stage::Working, Some(2)),
        "working",
        "working counts no rounds"
    );
}

fn check(done_when: &str, proven_by: &str) -> RefineCheck {
    RefineCheck::new(done_when.to_string(), proven_by.to_string())
}

/// The plan's checks a RefineDialog opens with.
fn plan() -> Vec<RefineCheck> {
    vec![
        check(
            "Signing in stores a token in the keyring",
            "cargo nextest run -p watch-house sign_in",
        ),
        check("A revoked token sends the constable back to sign-in", ""),
    ]
}

#[test]
fn a_title_edit_alone_sends_the_title_and_no_checks() {
    let title = "Store the constable's token";
    let command = refine_command(
        IssueId::new("watch", "WATCH-44"),
        (title, &plan()),
        ("Keep the token  ", &plan()),
        false,
    );
    assert_eq!(
        command,
        Some(Command::Refine {
            issue: IssueId::new("watch", "WATCH-44"),
            title: Some("Keep the token".into()),
            checks: None,
            send: false,
        })
    );
}

#[test]
fn changed_checks_are_sent_whole_and_empty_rows_are_left_out() {
    let title = "Store the constable's token";
    let mut typed = plan();
    typed[1].proven_by = "cargo nextest run -p watch-house revoked\n".into();
    typed.push(check("  ", "left empty"));
    let command = refine_command(
        IssueId::new("watch", "WATCH-44"),
        (title, &plan()),
        (title, &typed),
        false,
    );
    assert_eq!(
        command,
        Some(Command::Refine {
            issue: IssueId::new("watch", "WATCH-44"),
            title: None,
            checks: Some(vec![
                Check::new(
                    "Signing in stores a token in the keyring",
                    "cargo nextest run -p watch-house sign_in"
                ),
                Check::new(
                    "A revoked token sends the constable back to sign-in",
                    "cargo nextest run -p watch-house revoked"
                ),
            ]),
            send: false,
        })
    );
    // Deleting every check sends an empty list, which removes the person's checks.
    let command = refine_command(
        IssueId::new("watch", "WATCH-44"),
        (title, &plan()),
        (title, &[]),
        false,
    );
    assert!(matches!(command, Some(Command::Refine { checks: Some(c), .. }) if c.is_empty()));
}

#[test]
fn nothing_changed_sends_nothing_unless_it_is_sent_to_a_refiner() {
    let title = "Store the constable's token";
    let padded = vec![
        check(
            " Signing in stores a token in the keyring ",
            "cargo nextest run -p watch-house sign_in",
        ),
        plan()[1].clone(),
    ];
    assert_eq!(
        refine_command(
            IssueId::new("watch", "WATCH-44"),
            (title, &plan()),
            (title, &padded),
            false
        ),
        None
    );
    assert_eq!(
        refine_command(
            IssueId::new("watch", "WATCH-44"),
            (title, &plan()),
            (title, &plan()),
            true
        ),
        Some(Command::Refine {
            issue: IssueId::new("watch", "WATCH-44"),
            title: None,
            checks: None,
            send: true,
        })
    );
}
