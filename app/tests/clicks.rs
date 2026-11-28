//! Clicks and keys on the Project page, drawn under GPUI's test support with the design system's
//! faces and no window: a tab, View and Group by, a row, q and the close button, the pane's
//! scroll, a Waits for key, the pane over the board, and a long check's plus and minus; and on
//! parts drawn on their own: a DecisionRow's overturn confirm and RefineDialog. Run with
//! `cargo test --features shots --test clicks`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AppContext, Entity, InteractiveElement, IntoElement, KeyUpEvent, Keystroke, Modifiers,
    ParentElement, ScrollDelta, ScrollWheelEvent, Styled, TestAppContext, TestDispatcher,
    VisualTestContext, div, point, px,
};
use gpui_component::Root;
use temet_work_app::components::status_tag::Status;
use temet_work_app::composites::check_list::{CheckList, PlanCheck};
use temet_work_app::composites::decision_row::{
    Decision, DecisionOption, DecisionRow, OverturnChoice, UndoCost,
};
use temet_work_app::components::input_group::InputGroup;
use temet_work_app::components::property_list::PropertyItem;
use temet_work_app::components::stage_tag::Stage;
use temet_work_app::composites::issue_pane::{IssuePane, IssueTab};
use temet_work_app::composites::refine_dialog::{RefineCheck, RefineDialog};
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
    // The click left the focus on its row; the next tab stop is the row after it, and Enter
    // opens it.
    vcx.update(|window, cx| window.focus_next(cx));
    draw(vcx);
    // A real key press ends with its key up, which is when GPUI clicks the focused element.
    vcx.simulate_keystrokes("enter");
    vcx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").expect("a key"),
    });
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

/// A test app with the faces and the dark theme, for a part drawn on its own.
fn part_cx() -> &'static mut TestAppContext {
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
    cx
}

/// Clicks a text field and types into it.
fn type_into(vcx: &mut VisualTestContext, selector: &'static str, text: &str) {
    click(vcx, selector);
    vcx.simulate_input(text);
    draw(vcx);
}

/// Two decisions in the pane's width: one with two other options, one with a single other
/// option. Each overturn it hands back is kept.
struct Decisions {
    choices: Rc<RefCell<Vec<(String, OverturnChoice)>>>,
}

impl gpui::Render for Decisions {
    fn render(&mut self, _: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let option = |id: &str, label: &str| DecisionOption {
            id: id.to_string().into(),
            label: label.to_string().into(),
        };
        let decision = |options: Vec<DecisionOption>| Decision {
            question: Some("How many towers should the test send a message through?".into()),
            took: "8".into(),
            by: "refiner-66".into(),
            issue: None,
            at: None,
            cost: Some(UndoCost::Low),
            weighed: Vec::new(),
            why: Some("Eight covers the longest line in use.".into()),
            options,
            overturn_before: Some("merge".into()),
            overturned: None,
            overturned_to: None,
            overturn_reason: None,
        };
        let row = |id: &'static str, options: Vec<DecisionOption>| {
            let choices = self.choices.clone();
            DecisionRow::new(id, decision(options)).on_overturn(move |choice, _, _| {
                choices.borrow_mut().push((id.to_string(), choice.clone()));
            })
        };
        div()
            .w(theme::size::col_right(cx, px(1440.)))
            .child(row("d1", vec![option("1", "4"), option("3", "12")]))
            .child(row("d2", vec![option("1", "Yes, on by default")]))
    }
}

#[test]
fn overturn_asks_for_an_option_and_a_reason_then_hands_them_back() {
    let cx = part_cx();
    let choices = Rc::new(RefCell::new(Vec::new()));
    let kept = choices.clone();
    let (_, vcx) = cx.add_window_view(move |window, cx| {
        let rows = cx.new(|_| Decisions { choices: kept });
        Root::new(rows, window, cx)
    });
    draw(vcx);
    assert!(vcx.debug_bounds("overturn-d1-confirm").is_none());
    click(vcx, "overturn-d1");
    assert!(
        vcx.debug_bounds("overturn-d1").is_none(),
        "the trigger gives way"
    );
    assert!(vcx.debug_bounds("overturn-d1-reason").is_some());

    // Disabled until an option and a reason are given.
    click(vcx, "overturn-d1-confirm");
    click(vcx, "overturn-d1-option-3");
    click(vcx, "overturn-d1-confirm");
    assert!(choices.borrow().is_empty(), "an option alone is not enough");
    type_into(vcx, "overturn-d1-reason", "   ");
    click(vcx, "overturn-d1-confirm");
    assert!(
        choices.borrow().is_empty(),
        "a blank reason is not a reason"
    );
    type_into(
        vcx,
        "overturn-d1-reason",
        "The \u{dc}berwald line has twelve.  ",
    );
    click(vcx, "overturn-d1-confirm");
    assert_eq!(
        choices.borrow().as_slice(),
        &[(
            "d1".to_string(),
            OverturnChoice {
                option: "3".into(),
                reason: "The \u{dc}berwald line has twelve.".into(),
            }
        )]
    );

    // Cancel closes the confirm; Escape in it does too.
    click(vcx, "overturn-d1-cancel");
    assert!(vcx.debug_bounds("overturn-d1-reason").is_none());
    assert!(vcx.debug_bounds("overturn-d1").is_some());
    click(vcx, "overturn-d1");
    click(vcx, "overturn-d1-reason");
    vcx.simulate_keystrokes("escape");
    draw(vcx);
    assert!(vcx.debug_bounds("overturn-d1-reason").is_none());
    assert!(vcx.debug_bounds("overturn-d1").is_some());

    // With one other option it starts chosen: a reason is all it asks for.
    click(vcx, "overturn-d2");
    assert!(
        vcx.debug_bounds("overturn-d2-option-1").is_none(),
        "no radio"
    );
    type_into(
        vcx,
        "overturn-d2-reason",
        "A tower must be able to turn it off.",
    );
    click(vcx, "overturn-d2-confirm");
    let last = choices.borrow().last().cloned();
    assert_eq!(
        last,
        Some((
            "d2".to_string(),
            OverturnChoice {
                option: "1".into(),
                reason: "A tower must be able to turn it off.".into(),
            }
        ))
    );
}

/// What RefineDialog handed back: each save's title, checks and send, and each close.
#[derive(Default)]
struct Refined {
    saves: Vec<(String, Vec<RefineCheck>, bool)>,
    closes: usize,
}

/// RefineDialog open over an empty window, for an issue with a blank title and one check.
struct Refining {
    open: bool,
    refined: Rc<RefCell<Refined>>,
}

impl gpui::Render for Refining {
    fn render(&mut self, _: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let saved = self.refined.clone();
        let closed = self.refined.clone();
        let view = cx.entity();
        div().size_full().child(
            RefineDialog::new(
                "refine",
                "clacks#219",
                "",
                vec![RefineCheck {
                    done_when: "Lamps follow the clock.".into(),
                    proven_by: "cargo nextest run -p clacks-lamps clock".into(),
                }],
            )
            .is_open(self.open)
            .on_save(move |title, checks, send, _, _| {
                saved
                    .borrow_mut()
                    .saves
                    .push((title.to_string(), checks.to_vec(), send));
            })
            .on_close(move |_, cx| {
                closed.borrow_mut().closes += 1;
                view.update(cx, |v, cx| {
                    v.open = false;
                    cx.notify();
                });
            }),
        )
    }
}

#[test]
fn refine_edits_the_title_and_checks_and_hands_them_back() {
    let cx = part_cx();
    let refined = Rc::new(RefCell::new(Refined::default()));
    let kept = refined.clone();
    let view = cx.new(|_| Refining {
        open: true,
        refined: kept,
    });
    let root = view.clone();
    let (_, vcx) = cx.add_window_view(move |window, cx| Root::new(root, window, cx));
    draw(vcx);

    // Both saves wait for a title.
    click(vcx, "refine-save");
    click(vcx, "refine-send");
    type_into(vcx, "refine-title", "   ");
    click(vcx, "refine-save");
    assert!(refined.borrow().saves.is_empty());

    type_into(vcx, "refine-title", "Night lamps follow the light  ");
    // Add a check and fill it in, add one left blank, then delete the first.
    click(vcx, "refine-add");
    type_into(
        vcx,
        "refine-check-1-done-when",
        "Lamps light when the sensor reads dark.",
    );
    type_into(
        vcx,
        "refine-check-1-proven-by",
        "cargo nextest run -p clacks-lamps sensor",
    );
    click(vcx, "refine-add");
    assert!(vcx.debug_bounds("refine-check-2-done-when").is_some());
    click(vcx, "refine-check-0-delete");
    assert!(vcx.debug_bounds("refine-check-2-done-when").is_none());

    let expected = vec![RefineCheck {
        done_when: "Lamps light when the sensor reads dark.".into(),
        proven_by: "cargo nextest run -p clacks-lamps sensor".into(),
    }];
    click(vcx, "refine-save");
    click(vcx, "refine-send");
    assert_eq!(
        refined.borrow().saves,
        vec![
            (
                "Night lamps follow the light".to_string(),
                expected.clone(),
                false
            ),
            ("Night lamps follow the light".to_string(), expected, true),
        ],
        "the title trimmed, the blank check left out"
    );

    // Escape closes it unsaved.
    click(vcx, "refine-title");
    vcx.simulate_keystrokes("escape");
    draw(vcx);
    assert_eq!(refined.borrow().closes, 1);
    assert_eq!(refined.borrow().saves.len(), 2);
    assert!(vcx.debug_bounds("refine-title").is_none());

    // Opened again, it starts from the issue's own title and checks.
    vcx.update(|_, cx| {
        view.update(cx, |v, cx| {
            v.open = true;
            cx.notify();
        })
    });
    draw(vcx);
    assert!(vcx.debug_bounds("refine-check-0-done-when").is_some());
    assert!(vcx.debug_bounds("refine-check-1-done-when").is_none());
    click(vcx, "refine-save");
    assert_eq!(refined.borrow().saves.len(), 2, "the title is blank again");
    click(vcx, "refine-close");
    assert_eq!(refined.borrow().closes, 2);
}

/// An issue's pane with a close handler, beside a text field: q typed in the field is text.
struct PaneBesideAField {
    closed: Rc<RefCell<bool>>,
}

impl gpui::Render for PaneBesideAField {
    fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        let closed = self.closed.clone();
        div()
            .flex()
            .size_full()
            .child(div().w(px(300.)).child(InputGroup::new("note").fill(true)))
            .child(
                div().w(px(440.)).child(
                    IssuePane::new("clacks#219", "Night lamps switch on by the clock", Stage::Ready)
                        .facts(vec![PropertyItem::new("Group", "grand-trunk")])
                        .on_close(move |_, _| *closed.borrow_mut() = true),
                ),
            )
    }
}

#[test]
fn q_typed_into_a_text_field_leaves_the_pane_open() {
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
    let closed = Rc::new(RefCell::new(false));
    let shown = closed.clone();
    let (_, vcx) = cx.add_window_view(move |window, cx| {
        let view = cx.new(|_| PaneBesideAField { closed: shown });
        Root::new(view, window, cx)
    });
    draw(vcx);
    click(vcx, "note");
    vcx.simulate_keystrokes("q");
    draw(vcx);
    assert!(!*closed.borrow(), "q went into the field");
    // Out of the field, q closes the pane.
    vcx.update(|window, cx| window.blur(cx));
    draw(vcx);
    vcx.simulate_keystrokes("q");
    draw(vcx);
    assert!(*closed.borrow());
}
