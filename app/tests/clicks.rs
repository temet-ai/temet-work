//! Clicks and keys on the Project page, drawn under GPUI's test support with the design system's
//! faces and no window: a tab, View and Group by, a row, q and the close button, the pane's
//! scroll, a Waits for key, the pane over the board, and a long check's plus and minus. Run with
//! `cargo test --features shots --test clicks`.

use std::sync::Arc;

use gpui::{
    AppContext, Entity, InteractiveElement, IntoElement, Modifiers, ParentElement, ScrollDelta,
    ScrollWheelEvent, Styled, TestAppContext, TestDispatcher, VisualTestContext, div, point, px,
};
use gpui_component::Root;
use temet_work_app::components::status_tag::Status;
use temet_work_app::composites::check_list::{CheckList, PlanCheck};
use temet_work_app::composites::issue_pane::IssueTab;
use temet_work_app::pages::project::{Grouping, Project, ViewMode};
use temet_work_app::{assets, sample, theme};

/// The page in a test window, drawn.
fn page() -> (Entity<Project>, &'static mut VisualTestContext) {
    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new("Archivo"));
    // The app context outlives the test's window, as the window's own context does.
    let cx: &'static mut TestAppContext = Box::leak(Box::new(
        TestAppContext::build_with_text_system(TestDispatcher::new(0), None, text),
    ));
    cx.update(|cx| {
        gpui_component::init(cx);
        cx.text_system()
            .add_fonts(assets::faces())
            .expect("the faces load");
        theme::apply(cx, "dark").expect("the dark theme");
    });
    let page = cx.new(|_| Project::new(sample::project_view(), sample::selected()));
    let root = page.clone();
    // Opening the window draws its first frame. The issue tables settle their columns in the
    // next, but every target of a click is drawn in the first, and a click finds its target in
    // the frame drawn last.
    let (_, vcx) = cx.add_window_view(move |window, cx| Root::new(root, window, cx));
    (page, vcx)
}

/// One frame, after the work the last click started.
fn draw(vcx: &mut VisualTestContext) {
    vcx.run_until_parked();
    vcx.update(|window, cx| window.draw(cx).clear(cx));
}

/// Scrolls the main column until the element drawn with `selector` is in the window: Needs you
/// lists every open decision above the issues.
fn reveal(vcx: &mut VisualTestContext, selector: &'static str) {
    let window = vcx.update(|w, _| w.viewport_size());
    for _ in 0..40 {
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("nothing drawn as {selector}"));
        if bounds.bottom() <= window.height && bounds.top() >= px(0.) {
            return;
        }
        let main = vcx.debug_bounds("main").expect("the main column");
        let step = if bounds.top() < px(0.) { 300. } else { -300. };
        vcx.simulate_event(ScrollWheelEvent {
            position: main.center(),
            delta: ScrollDelta::Pixels(point(px(0.), px(step))),
            ..Default::default()
        });
        draw(vcx);
    }
    panic!("{selector} never came into the window");
}

/// Clicks the middle of the element drawn with `selector`, then draws the one frame the next
/// click looks for its target in.
fn click(vcx: &mut VisualTestContext, selector: &'static str) {
    let bounds = vcx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("nothing drawn as {selector}"));
    vcx.simulate_click(bounds.center(), Modifiers::none());
    draw(vcx);
}

/// Clicks the element drawn with `selector` and lets the page take the click, without drawing
/// again: for a last click, whose result the test reads from the page.
fn last_click(vcx: &mut VisualTestContext, selector: &'static str) {
    let bounds = vcx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("nothing drawn as {selector}"));
    vcx.simulate_click(bounds.center(), Modifiers::none());
    vcx.run_until_parked();
}

#[test]
fn a_tab_opens_on_click() {
    let (page, vcx) = page();
    assert_eq!(
        vcx.read_entity(&page, |p, _| p.tab()),
        Some(IssueTab::Decisions),
        "clacks#212 has open decisions"
    );
    click(vcx, "tab-plan");
    assert_eq!(vcx.read_entity(&page, |p, _| p.tab()), Some(IssueTab::Plan));
    last_click(vcx, "tab-review");
    assert_eq!(
        vcx.read_entity(&page, |p, _| p.tab()),
        Some(IssueTab::Review)
    );
}

#[test]
fn board_and_list_switch_what_is_drawn_and_group_by_shows_on_the_list_only() {
    let (page, vcx) = page();
    assert_eq!(vcx.read_entity(&page, |p, _| p.mode()), ViewMode::List);
    assert!(
        vcx.debug_bounds("option-milestone").is_some(),
        "Group by on the list"
    );
    assert!(vcx.debug_bounds("board").is_none());
    assert!(vcx.debug_bounds("issue-clacks#212").is_some());

    click(vcx, "option-board");
    assert_eq!(vcx.read_entity(&page, |p, _| p.mode()), ViewMode::Board);
    assert!(vcx.debug_bounds("board").is_some(), "the issues as a board");
    assert!(
        vcx.debug_bounds("option-milestone").is_none(),
        "no Group by on the board"
    );
    assert!(vcx.debug_bounds("issue-clacks#212").is_none(), "no table");

    click(vcx, "option-list");
    assert!(vcx.debug_bounds("board").is_none());
    assert!(
        vcx.debug_bounds("group-fixing").is_none(),
        "by milestone at first"
    );
    click(vcx, "option-stage");
    assert_eq!(vcx.read_entity(&page, |p, _| p.grouping()), Grouping::Stage);
    assert!(
        vcx.debug_bounds("group-fixing").is_some(),
        "one table under a row per stage"
    );
    assert!(vcx.debug_bounds("issue-clacks#212").is_some());
    last_click(vcx, "option-milestone");
    assert_eq!(
        vcx.read_entity(&page, |p, _| p.grouping()),
        Grouping::Milestone
    );
}

#[test]
fn a_click_or_enter_on_a_row_opens_its_issue_on_the_right_tab() {
    let (page, vcx) = page();
    reveal(vcx, "issue-clacks#210");
    click(vcx, "issue-clacks#208");
    let (selected, tab) = vcx.read_entity(&page, |p, _| (p.selected().cloned(), p.tab()));
    assert_eq!(selected.as_deref(), Some("clacks#208"));
    assert_eq!(
        tab,
        Some(IssueTab::Decisions),
        "clacks#208 has open decisions"
    );
    // The row after it has the focus by Tab; Enter opens it.
    vcx.simulate_keystrokes("tab enter");
    draw(vcx);
    let (selected, tab) = vcx.read_entity(&page, |p, _| (p.selected().cloned(), p.tab()));
    assert_eq!(selected.as_deref(), Some("clacks#210"));
    assert_eq!(tab, Some(IssueTab::Plan), "clacks#210 has no open decision");
}

#[test]
fn q_closes_the_pane_and_a_click_on_a_row_opens_it_again() {
    let (page, vcx) = page();
    assert!(vcx.debug_bounds("side").is_some());
    vcx.simulate_keystrokes("q");
    draw(vcx);
    assert_eq!(vcx.read_entity(&page, |p, _| p.selected().cloned()), None);
    assert!(vcx.debug_bounds("side").is_none(), "no pane");
    reveal(vcx, "issue-clacks#215");
    click(vcx, "issue-clacks#215");
    assert_eq!(
        vcx.read_entity(&page, |p, _| p.selected().cloned())
            .as_deref(),
        Some("clacks#215")
    );
    assert!(vcx.debug_bounds("side").is_some());
    // Its close button closes it too.
    last_click(vcx, "pane-close-clacks#215");
    assert_eq!(vcx.read_entity(&page, |p, _| p.selected().cloned()), None);
}

#[test]
fn the_panes_head_stays_at_its_top_while_the_pane_scrolls() {
    let (_, vcx) = page();
    draw(vcx);
    let head = vcx.debug_bounds("pane-head").expect("the head");
    let side = vcx.debug_bounds("side").expect("the pane");
    assert_eq!(head.top(), side.top());
    let tabs = vcx.debug_bounds("tab-plan").expect("the tabs");
    let scroll = vcx.debug_bounds("pane-scroll").expect("the scroll area");
    vcx.simulate_event(ScrollWheelEvent {
        position: scroll.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-400.))),
        ..Default::default()
    });
    draw(vcx);
    let moved = vcx.debug_bounds("tab-plan").expect("the tabs");
    assert!(moved.top() < tabs.top(), "the content scrolled");
    let head = vcx.debug_bounds("pane-head").expect("the head");
    assert_eq!(head.top(), side.top(), "the head stayed put");
    assert!(
        moved.top() >= head.bottom() || moved.bottom() <= head.bottom(),
        "nothing passes over the head: it clips its own area"
    );
}

#[test]
fn a_waits_for_key_opens_that_issue() {
    let (page, vcx) = page();
    reveal(vcx, "issue-clacks#219");
    click(vcx, "issue-clacks#219");
    last_click(vcx, "waits-for-clacks#215");
    assert_eq!(
        vcx.read_entity(&page, |p, _| p.selected().cloned())
            .as_deref(),
        Some("clacks#215")
    );
}

#[test]
fn on_the_board_the_pane_lies_over_it_and_on_the_list_it_takes_a_column() {
    let (page, vcx) = page();
    let main_with_pane = vcx
        .debug_bounds("main")
        .expect("the main column")
        .size
        .width;
    vcx.simulate_keystrokes("q");
    draw(vcx);
    let main_alone = vcx
        .debug_bounds("main")
        .expect("the main column")
        .size
        .width;
    assert!(
        main_with_pane < main_alone,
        "on the list the pane takes a column: {main_with_pane:?} < {main_alone:?}"
    );

    click(vcx, "option-board");
    let board_alone = vcx.debug_bounds("board").expect("the board").size.width;
    vcx.update(|_, cx| {
        page.update(cx, |p, cx| {
            p.select(&"clacks#219".into());
            cx.notify();
        })
    });
    draw(vcx);
    assert!(vcx.debug_bounds("side").is_some(), "the pane is open");
    let board_with_pane = vcx.debug_bounds("board").expect("the board").size.width;
    assert_eq!(board_with_pane, board_alone, "the board keeps its width");
    let main = vcx
        .debug_bounds("main")
        .expect("the main column")
        .size
        .width;
    assert_eq!(main, main_alone, "the pane lies over the main column");
}

/// A check list in a column as wide as the pane's, outlined for the test to measure: a check
/// whose done-when runs past three lines, then a short one.
struct LongChecks;

impl gpui::Render for LongChecks {
    fn render(&mut self, _: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let check = |done_when: &str, proven_by: &str| PlanCheck {
            done_when: done_when.to_string().into(),
            proven_by: proven_by.to_string().into(),
            status: Some(Status::Passed),
            status_text: None,
        };
        div()
            .w(theme::size::col_right(cx, px(1440.)))
            .debug_selector(|| "checks".into())
            .child(CheckList::new(
                "list",
                vec![
                    check(
                        "A message sent from Ankh-Morpork to Genua over the eight towers of the \
                         coast line arrives with the X-Clacks-Overhead field exactly as the \
                         sending tower set it, when the message is rerouted by the Hub, when it \
                         waits at a tower during fog, and when a tower on the way restarts.",
                        "cargo nextest run -p clacks-network overhead",
                    ),
                    check(
                        "No setting turns the copy off on any tower.",
                        "grep for overhead in the tower config schema finds no key",
                    ),
                ],
            ))
    }
}

#[test]
fn plus_opens_a_long_check_and_minus_closes_it() {
    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new("Archivo"));
    let cx: &'static mut TestAppContext = Box::leak(Box::new(
        TestAppContext::build_with_text_system(TestDispatcher::new(0), None, text),
    ));
    cx.update(|cx| {
        gpui_component::init(cx);
        cx.text_system()
            .add_fonts(assets::faces())
            .expect("the faces load");
        theme::apply(cx, "dark").expect("the dark theme");
    });
    let (_, vcx) = cx.add_window_view(|window, cx| {
        let checks = cx.new(|_| LongChecks);
        Root::new(checks, window, cx)
    });
    // The list measures its width in the first frame and clamps the long check in the next.
    draw(vcx);
    draw(vcx);
    let height = |vcx: &mut VisualTestContext| {
        vcx.debug_bounds("checks")
            .expect("the list is drawn")
            .size
            .height
    };
    let closed = height(vcx);
    click(vcx, "list-check-0-toggle");
    let open = height(vcx);
    assert!(open > closed, "open {open:?}, closed {closed:?}");
    click(vcx, "list-check-0-toggle");
    assert_eq!(height(vcx), closed);
    // The short check's button only holds its place: a click on it opens nothing.
    click(vcx, "list-check-1-toggle");
    assert_eq!(height(vcx), closed);
}
