//! The app's logic, without a window: the sample data in the parts' props, the selected row,
//! the issue table's column choice, a milestone's subtitle, the icons the parts need, and the
//! `refine` a RefineDialog's Save sends.

use std::collections::HashSet;

use gpui::{SharedString, px};
use temet_work_app::assets::FILES;
use temet_work_app::components::icon::{IconName, IconSize};
use temet_work_app::components::stage_tag::{Stage, StageTag};
use temet_work_app::components::status_tag::Status;
use temet_work_app::composites::check_list::{PlanCheck, count_line};
use temet_work_app::composites::issue_pane::{IssueTab, tabs_shown};
use temet_work_app::composites::issue_table::{GroupBy, agent_fits, colgroup, count, grouped};
use temet_work_app::composites::milestone_panel::{progress, subtitle};
use temet_work_app::composites::refine_dialog::RefineCheck;
use temet_work_app::pages::project::{
    Grouping, Project, Selection, all_issues_summary, issue_facts, refine_command,
};
use temet_work_app::sample;
use temet_work_app::theme;
use temet_work_app::theme::size::IssueColumns;
use temet_work_core::Command;
use temet_work_core::ids::IssueId;
use temet_work_core::model::Check;

fn columns() -> IssueColumns {
    IssueColumns {
        key: px(76.),
        stage: px(120.),
        agent: px(124.),
        updated: px(88.),
        title_min: px(272.),
    }
}

#[test]
fn the_sample_has_every_state_the_screen_must_draw() {
    let view = sample::project_view();
    assert!(
        view.groups.iter().any(|g| g.agents.is_empty()),
        "a group with no agents"
    );
    assert!(
        view.milestones.iter().any(|m| m.issues.is_empty()),
        "a milestone with no issues"
    );
    assert!(
        view.milestones
            .iter()
            .flat_map(|m| &m.issues)
            .any(|i| i.title.len() > 120),
        "a title longer than two lines"
    );
    assert!(
        view.groups.iter().any(|g| g.name.len() > 40),
        "a group name that is cut short"
    );
    let projects: Vec<_> = view.areas.iter().flat_map(|a| &a.projects).collect();
    assert!(
        projects.iter().any(|p| p.id == view.project),
        "the project on screen is listed"
    );
}

#[test]
fn each_milestone_counts_its_issues_and_the_done_ones_as_done() {
    for m in sample::project_view().milestones {
        assert_eq!(m.issues.len(), m.total, "{}", m.name);
        let done = m.issues.iter().filter(|i| i.stage == Stage::Done).count();
        assert_eq!(m.done, done, "{}", m.name);
    }
}

#[test]
fn every_agent_folder_and_decision_names_an_issue_of_its_own_group() {
    let view = sample::project_view();
    let issues: Vec<_> = view
        .milestones
        .iter()
        .flat_map(|m| m.issues.clone())
        .collect();
    for group in &view.groups {
        for agent in &group.agents {
            let Some(place) = &agent.r#where else {
                continue;
            };
            for issue in &issues {
                let folder = issue.key.to_lowercase();
                if place.contains(&format!("{folder}-")) {
                    assert_eq!(issue.group.as_ref(), Some(&group.name), "{place}");
                }
            }
        }
    }
    // Each project's count of decisions that need the person matches the list.
    for need in &view.needs {
        assert!(view.need_project.contains_key(&need.id), "{}", need.id);
    }
    for (need, (key, id)) in &view.same_decision {
        let need = view.needs.iter().find(|n| &n.id == need).expect("the need");
        let decisions = view.details[key].decisions.as_ref().expect("decisions");
        let decision = decisions
            .iter()
            .find(|d| &d.id == id)
            .expect("the decision");
        assert_eq!(
            need.decision.took, decision.decision.took,
            "one decision, one wording"
        );
    }
}

#[test]
fn each_group_note_names_milestones_the_window_has() {
    let view = sample::project_view();
    let names: HashSet<String> = view.milestones.iter().map(|m| m.name.to_string()).collect();
    for group in &view.groups {
        let Some(note) = &group.note else { continue };
        let Some((project, versions)) = note.split_once(' ') else {
            continue;
        };
        if project != view.project.as_ref() {
            continue;
        }
        for version in versions.split(", ") {
            assert!(names.contains(&format!("{project} {version}")), "{note}");
        }
    }
}

#[test]
fn the_issues_and_code_links_are_written_alike() {
    for fact in sample::project_view().facts {
        let (host, path) = fact.value.split_once('/').expect("host and path");
        assert!(
            host.ends_with(".example.com") && !path.contains(' '),
            "{}",
            fact.value
        );
    }
}

#[test]
fn overturning_asks_then_marks_both_copies_of_a_decision_and_says_so() {
    let mut page = Project::new(sample::project_view(), sample::selected());
    page.ask_need(&"n3".into());
    let ask = page.asking().cloned().expect("a question");
    assert_eq!(
        ask.decision,
        Some(("WATCH-44".into(), "d2".into())),
        "the same decision"
    );
    assert_eq!(
        ask.question(),
        "Overturn the arbiter's decision on WATCH-44? Claude chose to refuse to store the \
         token when the keyring is locked."
    );
    page.cancel();
    assert!(page.asking().is_none() && page.needs().len() == 4);
    page.ask_decision(&"WATCH-44".into(), &"d2".into());
    assert_eq!(
        page.asking().and_then(|a| a.need.clone()).as_deref(),
        Some("n3")
    );
    page.confirm();
    assert_eq!(page.needs().len(), 3);
    assert_eq!(
        page.toast().map(|t| t.to_string()),
        Some("Overturned on WATCH-44: Refuse to store the token when the keyring is locked".into())
    );
}

#[test]
fn issue_keys_are_unique_and_the_first_selection_is_one_of_them() {
    let view = sample::project_view();
    let keys: Vec<&SharedString> = view
        .milestones
        .iter()
        .flat_map(|m| &m.issues)
        .map(|i| &i.key)
        .collect();
    let unique: HashSet<_> = keys.iter().collect();
    assert_eq!(unique.len(), keys.len());
    let selected = sample::selected().expect("an issue starts selected");
    assert!(keys.contains(&&selected));
}

#[test]
fn selecting_a_row_moves_the_one_selection() {
    let mut selection = Selection::new(Some("WATCH-44".into()));
    assert!(
        !selection.select(&"WATCH-44".into()),
        "the same row changes nothing"
    );
    assert!(selection.select(&"WATCH-51".into()));
    assert_eq!(selection.key().map(|k| k.as_ref()), Some("WATCH-51"));
    let mut none = Selection::default();
    assert_eq!(none.key(), None);
    assert!(none.select(&"WATCH-55".into()));
}

#[test]
fn the_agent_column_shows_only_when_the_title_keeps_its_room() {
    let c = columns();
    // 76 + 120 + 124 + 88 + 272 = 680.
    assert!(agent_fits(px(680.), &c, true));
    assert!(!agent_fits(px(679.), &c, true));
    // Without the Stage column the agent fits 120 px sooner.
    assert!(agent_fits(px(560.), &c, false));
    assert_eq!(
        colgroup(&c, true, true),
        vec![
            Some(px(76.)),
            None,
            Some(px(120.)),
            Some(px(124.)),
            Some(px(88.))
        ]
    );
    assert_eq!(colgroup(&c, true, false).len(), 4);
    // Grouped by stage, the table has no Stage column.
    assert_eq!(
        colgroup(&c, false, true),
        vec![Some(px(76.)), None, Some(px(124.)), Some(px(88.))]
    );
}

#[test]
fn a_milestone_says_what_is_done_and_when_it_is_due() {
    assert_eq!(
        subtitle(Some(7), Some(12), Some("12 Dec"), None).as_deref(),
        Some("7 / 12 done · due 12 Dec")
    );
    assert_eq!(
        subtitle(None, None, Some("16 Jan"), None).as_deref(),
        Some("due 16 Jan")
    );
    assert_eq!(
        subtitle(Some(1), Some(2), None, Some("3 issues")).as_deref(),
        Some("3 issues")
    );
    assert_eq!(subtitle(None, None, None, None), None);
    // A milestone with no issues says only when it is due, whatever summary it is given.
    assert_eq!(
        subtitle(Some(0), Some(0), Some("20 Feb"), Some("3 issues")).as_deref(),
        Some("due 20 Feb")
    );
    assert_eq!(subtitle(Some(0), Some(0), None, None), None);
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
fn grouping_by_stage_follows_the_stages_and_by_group_their_first_appearance() {
    let issues: Vec<_> = sample::project_view()
        .milestones
        .into_iter()
        .flat_map(|m| m.issues)
        .collect();
    let by_stage = grouped(issues.clone(), GroupBy::Stage);
    let stages: Vec<Stage> = by_stage.iter().filter_map(|(s, _, _)| *s).collect();
    let mut in_order = stages.clone();
    in_order.sort_by_key(|s| Stage::ALL.iter().position(|x| x == s));
    assert_eq!(stages, in_order, "groups follow the order of the stages");
    assert!(by_stage.iter().all(|(_, _, these)| !these.is_empty()));
    assert_eq!(
        by_stage
            .iter()
            .map(|(_, _, these)| these.len())
            .sum::<usize>(),
        issues.len()
    );
    let by_group = grouped(issues.clone(), GroupBy::Group);
    let names: Vec<&str> = by_group.iter().map(|(_, n, _)| n.as_ref()).collect();
    assert_eq!(names[0], issues[0].group.as_deref().unwrap_or("No group"));
    assert_eq!(count(1), "1 issue");
    assert_eq!(count(3), "3 issues");
}

#[test]
fn the_group_by_control_maps_to_the_tables_grouping() {
    assert_eq!(Grouping::Milestone.table(), None);
    assert_eq!(Grouping::Stage.table(), Some(GroupBy::Stage));
    assert_eq!(Grouping::from_value("group"), Some(Grouping::Group));
    assert_eq!(Grouping::from_value("week"), None);
    let view = sample::project_view();
    assert_eq!(
        all_issues_summary(&view.milestones),
        "7 issues across 3 milestones"
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
    let full = &view.details[&SharedString::from("WATCH-44")];
    assert!(full.plan.is_some() && full.pull.is_some());
    assert_eq!(full.rounds.as_ref().map(Vec::len), Some(2));
    assert_eq!(full.decisions.as_ref().map(Vec::len), Some(2));
    assert!(
        !view.details.contains_key(&SharedString::from("WATCH-47")),
        "facts only"
    );
}

#[test]
fn the_pane_states_each_issues_facts() {
    let view = sample::project_view();
    let m = &view.milestones[0];
    let issue = m
        .issues
        .iter()
        .find(|i| i.key.as_ref() == "WATCH-47")
        .unwrap();
    let facts = issue_facts(m, issue, None);
    let labels: Vec<&str> = facts.iter().map(|(l, _)| l.as_ref()).collect();
    assert_eq!(labels, ["Milestone", "Group", "Agent", "Round", "Updated"]);
    assert_eq!(facts[3].1.as_ref(), "Not reviewed yet");
}

#[test]
fn selecting_another_row_opens_it_on_its_first_tab() {
    let mut page = Project::new(sample::project_view(), sample::selected());
    page.set_tab(IssueTab::Review);
    assert!(page.select(&"WATCH-51".into()));
    assert!(!page.select(&"WATCH-51".into()));
}

#[test]
fn needs_you_is_sorted_costliest_first_and_can_be_emptied() {
    use temet_work_app::composites::decision_row::UndoCost;
    let view = sample::project_view();
    let rank = |c: Option<UndoCost>| match c {
        Some(UndoCost::High) => 0,
        Some(UndoCost::Medium) => 1,
        Some(UndoCost::Low) => 2,
        None => 3,
    };
    let costs: Vec<_> = view.needs.iter().map(|n| rank(n.decision.cost)).collect();
    let mut sorted = costs.clone();
    sorted.sort();
    assert_eq!(costs, sorted);
    assert_eq!(view.needs.len(), 4);
    let none = sample::without_needs(view);
    assert!(none.needs.is_empty());
    assert!(
        none.areas
            .iter()
            .flat_map(|a| &a.projects)
            .all(|p| p.waiting == 0)
    );
}

#[test]
fn each_machine_has_a_slot_in_use_for_each_agent_on_it_and_one_is_full() {
    let view = sample::project_view();
    assert_eq!(view.machines.len(), 3);
    for machine in &view.machines {
        let agents = view
            .groups
            .iter()
            .flat_map(|g| &g.agents)
            .filter_map(|a| a.r#where.as_ref())
            .filter(|w| w.split(" · ").next() == Some(machine.label.as_ref()))
            .count();
        let used = machine.reading.split(" / ").next().expect("in use");
        assert_eq!(used, agents.to_string(), "{}", machine.label);
    }
    assert!(view.machines.iter().any(|m| m.value >= 1.));
    assert_eq!(view.limits.len(), 3);
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

#[test]
fn the_count_line_counts_the_checks_by_status_in_order() {
    let check = |status| PlanCheck {
        done_when: "The clacks tower answers".into(),
        proven_by: "cargo nextest run -p clacks".into(),
        status,
        status_text: None,
    };
    assert_eq!(count_line(&[]), None);
    assert_eq!(
        count_line(&[check(Some(Status::Failed))]).as_deref(),
        Some("1 check: 1 failed")
    );
    assert_eq!(
        count_line(&[
            check(Some(Status::Passed)),
            check(Some(Status::Running)),
            check(Some(Status::Passed)),
        ])
        .as_deref(),
        Some("3 checks: 2 passed, 1 running")
    );
    assert_eq!(
        count_line(&[check(None), check(None)]).as_deref(),
        Some("2 checks: 2 not run yet")
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
        "watch#WATCH-44",
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
