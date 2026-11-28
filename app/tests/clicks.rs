//! Clicks on the Project page, drawn under GPUI's test support with the design system's faces
//! and no window: a tab, Group by, a row, Overturn with its confirm, and a long check's plus
//! and minus; and on parts drawn on their own: a DecisionRow's overturn confirm and
//! RefineDialog. Run with
//! `cargo test --features shots --test clicks`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AppContext, Entity, InteractiveElement, IntoElement, Modifiers, ParentElement, Styled,
    TestAppContext, TestDispatcher, VisualTestContext, div, px,
};
use gpui_component::Root;
use temet_work_app::components::status_tag::Status;
use temet_work_app::composites::check_list::{CheckList, PlanCheck};
use temet_work_app::composites::decision_row::{
    Decision, DecisionOption, DecisionRow, OverturnChoice, UndoCost,
};
use temet_work_app::composites::issue_pane::IssueTab;
use temet_work_app::composites::refine_dialog::{RefineCheck, RefineDialog};
use temet_work_app::pages::project::{Grouping, Project};
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
    click(vcx, "tab-pull");
    assert_eq!(vcx.read_entity(&page, |p, _| p.tab()), Some(IssueTab::Pull));
    last_click(vcx, "tab-decisions");
    assert_eq!(
        vcx.read_entity(&page, |p, _| p.tab()),
        Some(IssueTab::Decisions)
    );
}

#[test]
fn group_by_regroups_the_issues_on_click() {
    let (page, vcx) = page();
    click(vcx, "option-stage");
    assert_eq!(vcx.read_entity(&page, |p, _| p.grouping()), Grouping::Stage);
    // Grouped by stage the rows are still there to click.
    assert!(vcx.debug_bounds("issue-WATCH-51").is_some());
    last_click(vcx, "option-milestone");
    assert_eq!(
        vcx.read_entity(&page, |p, _| p.grouping()),
        Grouping::Milestone
    );
}

#[test]
fn a_row_is_selected_and_opened_on_click() {
    let (page, vcx) = page();
    click(vcx, "tab-review");
    last_click(vcx, "issue-WATCH-51");
    let (selected, tab) = vcx.read_entity(&page, |p, _| (p.selected().cloned(), p.tab()));
    assert_eq!(selected.as_deref(), Some("WATCH-51"));
    assert_eq!(tab, None, "a newly selected issue opens on its first tab");
}

#[test]
fn overturn_asks_first_then_overturns_and_says_so() {
    let (page, vcx) = page();
    let waiting = |vcx: &mut VisualTestContext| {
        vcx.read_entity(&page, |p, _| {
            p.areas()
                .iter()
                .flat_map(|a| a.projects.clone())
                .find(|p| p.id.as_ref() == "watch")
                .map(|p| p.waiting)
        })
    };
    assert_eq!(waiting(vcx), Some(2));

    click(vcx, "overturn-n1");
    let question = vcx.read_entity(&page, |p, _| p.asking().map(|a| a.question()));
    assert_eq!(
        question.as_deref(),
        Some(
            "Overturn the arbiter's decision on WATCH-51? Claude chose to page the tower list \
             200 at a time instead of loading all."
        )
    );
    // Cancelling leaves it; Overturn again, then confirm.
    click(vcx, "alert-cancel");
    assert!(vcx.read_entity(&page, |p, _| p.asking().is_none()));
    click(vcx, "overturn-n1");
    click(vcx, "alert-confirm");

    let (needs, toast) = vcx.read_entity(&page, |p, _| (p.needs().len(), p.toast().cloned()));
    assert_eq!(needs, 3);
    assert_eq!(
        toast.as_deref(),
        Some("Overturned on WATCH-51: Page the tower list 200 at a time instead of loading all")
    );
    assert_eq!(waiting(vcx), Some(1), "the project's count drops with it");
    // The toast and its close button lie inside the window.
    let close = vcx.debug_bounds("toast-close").expect("a close button");
    let window = vcx.update(|w, _| w.viewport_size());
    assert!(close.right() <= window.width && close.bottom() <= window.height);
    last_click(vcx, "toast-close");
    assert!(vcx.read_entity(&page, |p, _| p.toast().is_none()));
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
    assert!(vcx.debug_bounds("overturn-d1").is_none(), "the trigger gives way");
    assert!(vcx.debug_bounds("overturn-d1-reason").is_some());

    // Disabled until an option and a reason are given.
    click(vcx, "overturn-d1-confirm");
    click(vcx, "overturn-d1-option-3");
    click(vcx, "overturn-d1-confirm");
    assert!(choices.borrow().is_empty(), "an option alone is not enough");
    type_into(vcx, "overturn-d1-reason", "   ");
    click(vcx, "overturn-d1-confirm");
    assert!(choices.borrow().is_empty(), "a blank reason is not a reason");
    type_into(vcx, "overturn-d1-reason", "The \u{dc}berwald line has twelve.  ");
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
    assert!(vcx.debug_bounds("overturn-d2-option-1").is_none(), "no radio");
    type_into(vcx, "overturn-d2-reason", "A tower must be able to turn it off.");
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
    type_into(vcx, "refine-check-1-done-when", "Lamps light when the sensor reads dark.");
    type_into(vcx, "refine-check-1-proven-by", "cargo nextest run -p clacks-lamps sensor");
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
