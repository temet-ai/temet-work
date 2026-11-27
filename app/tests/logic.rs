//! The app's logic, without a window: the sample data in the parts' props, the selected row,
//! the issue table's column choice, a milestone's subtitle, and the icons the parts need.

use std::collections::HashSet;

use gpui::{SharedString, px};
use temet_work_app::assets::FILES;
use temet_work_app::components::icon::{IconName, IconSize};
use temet_work_app::components::stage_tag::Stage;
use temet_work_app::components::status_tag::Status;
use temet_work_app::composites::issue_table::{agent_fits, colgroup};
use temet_work_app::composites::milestone_panel::{progress, subtitle};
use temet_work_app::pages::project::Selection;
use temet_work_app::sample;
use temet_work_app::theme;
use temet_work_app::theme::size::IssueColumns;

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
fn each_milestone_counts_its_open_issues_in_its_total() {
    for m in sample::project_view().milestones {
        assert_eq!(m.done + m.issues.len(), m.total, "{}", m.name);
    }
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
        colgroup(&c, true),
        vec![
            Some(px(76.)),
            None,
            Some(px(120.)),
            Some(px(124.)),
            Some(px(88.))
        ]
    );
    assert_eq!(colgroup(&c, false).len(), 4);
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
