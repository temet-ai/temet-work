//! Clicks on the Project page, drawn under GPUI's test support with the design system's faces
//! and no window: a tab, Group by, a row, Overturn with its confirm, a long check's plus and
//! minus, the issue pane's actions menu moving an issue, and Refine with what its Save and "Save
//! and send to refiner" send. Run with
//! `cargo test --features shots --test clicks`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AppContext, Entity, InteractiveElement, IntoElement, Modifiers, ParentElement, Styled,
    TestAppContext, TestDispatcher, VisualTestContext, div, px,
};
use gpui_component::Root;
use temet_work_app::components::Intent;
use temet_work_app::components::status_tag::Status;
use temet_work_app::composites::check_list::{CheckList, PlanCheck};
use temet_work_app::composites::issue_pane::IssueTab;
use temet_work_app::composites::refine_dialog::Sending;
use temet_work_app::pages::project::{Grouping, Project};
use temet_work_app::{assets, sample, theme};
use temet_work_core::Command;
use temet_work_core::ids::IssueId;
use temet_work_core::view::StageWord;

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

/// What the page sent with its sender, and the answer the sender gives each time.
fn sending(
    page: &Entity<Project>,
    vcx: &mut VisualTestContext,
    answer: Result<&'static str, &'static str>,
) -> std::rc::Rc<std::cell::RefCell<Vec<Command>>> {
    let sent: std::rc::Rc<std::cell::RefCell<Vec<Command>>> = Default::default();
    let into = sent.clone();
    vcx.update(|_, cx| {
        page.update(cx, |p, _| {
            p.set_sender(move |command, _| {
                into.borrow_mut().push(command);
                gpui::Task::ready(answer.map(String::from).map_err(String::from))
            })
        })
    });
    sent
}

#[test]
fn move_to_ready_sends_the_move_and_shows_a_refusal_in_a_danger_toast() {
    let (page, vcx) = page();
    let why = "watch#WATCH-44 is in review, round 2: a reviewer is at work";
    let sent = sending(&page, vcx, Err(why));
    assert!(
        vcx.debug_bounds("move-to").is_none(),
        "the menu starts closed"
    );
    click(vcx, "issue-actions");
    assert!(vcx.read_entity(&page, |p, _| p.actions_open()));
    click(vcx, "move-to");
    // The submenu lists the moves the daemon takes for WATCH-44, and no other stage.
    assert!(vcx.debug_bounds("move-to-new").is_some());
    assert!(vcx.debug_bounds("move-to-working").is_none());
    last_click(vcx, "move-to-ready");
    assert_eq!(
        sent.borrow().as_slice(),
        [Command::Move {
            issue: IssueId::new("watch", "WATCH-44"),
            stage: StageWord::Ready,
        }]
    );
    let (toast, intent, open) = vcx.read_entity(&page, |p, _| {
        (p.toast().cloned(), p.toast_intent(), p.actions_open())
    });
    assert_eq!(toast.as_deref(), Some(why));
    assert_eq!(intent, Some(Intent::Danger));
    assert!(!open, "the menu closes once a stage is picked");
}

#[test]
fn an_accepted_move_says_what_the_daemon_said_in_the_plain_toast() {
    let (page, vcx) = page();
    let said = "watch#WATCH-44 is new; the move dropped its pull request 212, which is left open \
                on the code host for you to close or reuse";
    let sent = sending(&page, vcx, Ok(said));
    click(vcx, "issue-actions");
    click(vcx, "move-to");
    last_click(vcx, "move-to-new");
    assert_eq!(sent.borrow().len(), 1);
    let (toast, intent) = vcx.read_entity(&page, |p, _| (p.toast().cloned(), p.toast_intent()));
    assert_eq!(toast.as_deref(), Some(said));
    assert_eq!(intent, Some(Intent::None));
}

#[test]
fn a_press_outside_the_open_menu_closes_it_and_sends_nothing() {
    let (page, vcx) = page();
    let sent = sending(&page, vcx, Ok("unused"));
    click(vcx, "issue-actions");
    assert!(vcx.read_entity(&page, |p, _| p.actions_open()));
    last_click(vcx, "tab-pull");
    assert!(!vcx.read_entity(&page, |p, _| p.actions_open()));
    assert!(sent.borrow().is_empty());
}

/// One command the page sent, and where its answer goes.
type Held = (
    Command,
    futures_channel::oneshot::Sender<Result<String, String>>,
);

/// A sender that keeps each command and answers it only when the test does, so the test sees
/// the dialog busy while the daemon has not answered.
fn holding(page: &Entity<Project>, vcx: &mut VisualTestContext) -> Rc<RefCell<Vec<Held>>> {
    let held: Rc<RefCell<Vec<Held>>> = Rc::default();
    let into = held.clone();
    vcx.update(|_, cx| {
        page.update(cx, |p, _| {
            p.set_sender(move |command, cx| {
                let (answer, answered) = futures_channel::oneshot::channel();
                into.borrow_mut().push((command, answer));
                cx.spawn(async move |_| {
                    answered
                        .await
                        .unwrap_or_else(|_| Err("the test dropped the answer".into()))
                })
            })
        })
    });
    held
}

/// Answers the one command the page sent with `answer`, and gives the command.
fn answer(
    vcx: &mut VisualTestContext,
    held: &Rc<RefCell<Vec<Held>>>,
    answer: Result<&str, &str>,
) -> Command {
    let (command, to) = {
        let mut held = held.borrow_mut();
        assert_eq!(held.len(), 1, "one command sent");
        held.remove(0)
    };
    to.send(answer.map(String::from).map_err(String::from))
        .expect("the page waits for the answer");
    draw(vcx);
    command
}

/// Opens Refine on the selected issue from its pane's actions button.
fn open_refine(vcx: &mut VisualTestContext) {
    click(vcx, "issue-actions");
    click(vcx, "issue-actions-refine");
}

/// Puts `text` in the field drawn as `selector` in place of what it holds, as the person does:
/// a click in it, select all, then typing.
fn type_in(vcx: &mut VisualTestContext, selector: &'static str, text: &str) {
    click(vcx, selector);
    vcx.simulate_keystrokes("ctrl-a");
    vcx.simulate_input(text);
    draw(vcx);
}

fn refine_sending(vcx: &mut VisualTestContext, page: &Entity<Project>) -> Option<Sending> {
    vcx.read_entity(page, |p, _| p.refining().and_then(|r| r.sending))
}

fn refine(title: Option<&str>, checks: Option<Vec<(&str, &str)>>, send: bool) -> Command {
    Command::Refine {
        issue: IssueId::new("watch", "WATCH-44"),
        title: title.map(str::to_string),
        checks: checks.map(|checks| {
            checks
                .into_iter()
                .map(|(d, p)| temet_work_core::model::Check::new(d, p))
                .collect()
        }),
        send,
    }
}

#[test]
fn the_actions_menu_has_refine_before_move_to() {
    let (_, vcx) = page();
    click(vcx, "issue-actions");
    let refine = vcx.debug_bounds("issue-actions-refine").expect("Refine");
    let move_to = vcx.debug_bounds("move-to").expect("Move to");
    assert!(refine.bottom() <= move_to.top(), "Refine comes first");
}

#[test]
fn save_sends_refine_with_the_edited_title_and_checks() {
    let (page, vcx) = page();
    let held = holding(&page, vcx);
    open_refine(vcx);
    let opened = vcx.read_entity(&page, |p, _| p.refining().map(|r| r.opened.clone()));
    let (_, checks) = opened.expect("the dialog is open");
    assert_eq!(checks.len(), 3, "it opens with the plan's checks");

    type_in(
        vcx,
        "refine-title",
        "Keep the constable's token in the keyring",
    );
    // The last check goes; a new one comes in its place.
    click(vcx, "refine-check-2-delete");
    click(vcx, "refine-add");
    type_in(
        vcx,
        "refine-check-2-done-when",
        "A locked keyring refuses the token",
    );
    click(vcx, "refine-save");

    // While the daemon has not answered, the buttons are busy.
    assert_eq!(refine_sending(vcx, &page), Some(Sending::Save));
    let sent = answer(vcx, &held, Ok("refined watch#WATCH-44"));
    assert_eq!(
        sent,
        refine(
            Some("Keep the constable's token in the keyring"),
            Some(vec![
                (
                    "Signing in stores a token in the system keyring",
                    "cargo nextest run -p watch-house sign_in",
                ),
                (
                    "The token is read back after the Watch House restarts",
                    "cargo nextest run -p watch-house token_survives_restart",
                ),
                ("A locked keyring refuses the token", ""),
            ]),
            false,
        )
    );
    // Done: the dialog is closed and the focus is back on the actions button.
    assert!(vcx.read_entity(&page, |p, _| p.refining().is_none()));
    let back = vcx.update(|window, cx| {
        let focus = page.update(cx, |p, cx| p.actions_focus(cx));
        focus.is_focused(window)
    });
    assert!(back, "the focus is back on the actions button");
}

#[test]
fn save_and_send_adds_send_and_a_title_alone_keeps_the_checks() {
    let (page, vcx) = page();
    let held = holding(&page, vcx);
    open_refine(vcx);
    type_in(
        vcx,
        "refine-title",
        "Keep the constable's token in the keyring",
    );
    click(vcx, "refine-send");
    assert_eq!(refine_sending(vcx, &page), Some(Sending::SaveAndSend));
    assert_eq!(
        answer(vcx, &held, Ok("sent watch#WATCH-44 to a refiner")),
        refine(
            Some("Keep the constable's token in the keyring"),
            None,
            true
        ),
        "a title edit alone sends no checks"
    );
    assert!(vcx.read_entity(&page, |p, _| p.refining().is_none()));

    // With nothing changed, it still sends the issue to a refiner.
    open_refine(vcx);
    click(vcx, "refine-send");
    assert_eq!(answer(vcx, &held, Ok("sent")), refine(None, None, true));
    assert!(vcx.read_entity(&page, |p, _| p.refining().is_none()));
}

#[test]
fn a_refusal_shows_its_reason_and_keeps_the_draft() {
    let why = "the daemon is held: resume it, or Save without send";
    let (page, vcx) = page();
    let held = holding(&page, vcx);
    open_refine(vcx);
    type_in(
        vcx,
        "refine-title",
        "Keep the constable's token in the keyring",
    );
    click(vcx, "refine-send");
    answer(vcx, &held, Err(why));

    let (sending, not_done, title) = vcx.read_entity(&page, |p, cx| {
        let r = p.refining().expect("the dialog stays open");
        (r.sending, r.not_done.clone(), r.draft.title(cx))
    });
    assert_eq!(sending, None, "the buttons are free again");
    assert_eq!(not_done.as_deref(), Some(why));
    assert_eq!(title, "Keep the constable's token in the keyring");
    assert!(
        vcx.debug_bounds("refine-not-done").is_some(),
        "the reason is drawn in the dialog"
    );
}

#[test]
fn save_with_nothing_changed_or_no_title_sends_nothing() {
    let (page, vcx) = page();
    let held = holding(&page, vcx);
    open_refine(vcx);
    // No title: Save is disabled.
    type_in(vcx, "refine-title", " ");
    click(vcx, "refine-save");
    assert!(vcx.read_entity(&page, |p, _| {
        p.refining().is_some_and(|r| r.sending.is_none())
    }));
    // The title as it was: Save closes the dialog and sends nothing.
    let title = vcx.read_entity(&page, |p, _| p.refining().map(|r| r.opened.0.clone()));
    let title = title.expect("the dialog is open");
    vcx.update(|window, cx| {
        page.update(cx, |p, cx| {
            if let Some(r) = p.refining() {
                r.draft.title_field().set_value(title.clone(), window, cx);
            }
        })
    });
    draw(vcx);
    click(vcx, "refine-save");
    assert!(vcx.read_entity(&page, |p, _| p.refining().is_none()));
    assert!(held.borrow().is_empty(), "nothing was sent");
}
