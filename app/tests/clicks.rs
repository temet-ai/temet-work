//! Clicks on the Project page, drawn under GPUI's test support with the design system's faces
//! and no window: a tab, Group by, a row, Overturn with its confirm, a long check's plus and
//! minus, and Refine with what its Save and "Save and send to refiner" send to a fake daemon.
//! Run with
//! `cargo test --features shots --test clicks`.

use std::sync::Arc;

use gpui::{
    AppContext, Entity, InteractiveElement, IntoElement, Modifiers, ParentElement, Styled,
    TestAppContext, TestDispatcher, VisualTestContext, div, px,
};
use gpui_component::Root;
use temet_work_app::components::status_tag::Status;
use temet_work_app::composites::check_list::{CheckList, PlanCheck};
use temet_work_app::composites::issue_pane::IssueTab;
use temet_work_app::composites::refine_dialog::{NotDone, Sending};
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

/// A daemon on a socket in a temp folder of the test's own: it reads each connection's request
/// line, hands it to the test, waits until the test lets it answer, then answers with `reply`.
struct FakeDaemon {
    path: std::path::PathBuf,
    requests: std::sync::mpsc::Receiver<String>,
    answer: std::sync::mpsc::Sender<()>,
    _folder: tempfile::TempDir,
}

impl FakeDaemon {
    fn serve(reply: temet_work_core::Reply) -> Self {
        use std::io::{BufRead, BufReader, Write};
        let folder = tempfile::Builder::new()
            .prefix("tw-clicks-")
            .tempdir()
            .expect("a temp folder");
        let path = folder.path().join("daemon.sock");
        let listener = std::os::unix::net::UnixListener::bind(&path).expect("bind the socket");
        let (heard, requests) = std::sync::mpsc::channel();
        let (answer, answers) = std::sync::mpsc::channel::<()>();
        let line = serde_json::to_string(&reply).expect("json");
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { return };
                let mut reader = BufReader::new(stream);
                let mut request = String::new();
                if reader.read_line(&mut request).is_err() {
                    continue;
                }
                let _ = heard.send(request);
                if answers.recv().is_err() {
                    return;
                }
                let _ = writeln!(reader.get_mut(), "{line}");
            }
        });
        Self {
            path,
            requests,
            answer,
            _folder: folder,
        }
    }

    /// The request the next connection sent, as a request.
    fn request(&self) -> temet_work_core::Request {
        let line = self
            .requests
            .recv_timeout(WAIT)
            .expect("the daemon got a request");
        serde_json::from_str(&line).expect("a request")
    }

    /// Lets the daemon answer the request it holds.
    fn answer(&self) {
        self.answer.send(()).expect("the daemon is serving");
    }
}

/// The longest a test waits in real time for the socket's thread.
const WAIT: std::time::Duration = std::time::Duration::from_secs(5);

/// The page with its sample data and the fake daemon's socket, its reading thread let wake the
/// test's executor.
fn page_with(daemon: &FakeDaemon) -> (Entity<Project>, &'static mut VisualTestContext) {
    let (page, vcx) = page();
    vcx.executor().allow_parking();
    let path = daemon.path.clone();
    vcx.update(|_, cx| page.update(cx, |p, _| p.set_socket(Some(path))));
    draw(vcx);
    (page, vcx)
}

/// Runs what is ready, drawing, until `done` holds, waiting in real time for the socket's
/// thread at most [`WAIT`].
fn until(vcx: &mut VisualTestContext, done: impl Fn(&mut VisualTestContext) -> bool) {
    let end = std::time::Instant::now() + WAIT;
    loop {
        draw(vcx);
        if done(vcx) {
            return;
        }
        let left = end.saturating_duration_since(std::time::Instant::now());
        assert!(!left.is_zero(), "not done within {WAIT:?}");
        std::thread::park_timeout(left.min(std::time::Duration::from_millis(20)));
    }
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

fn sending(vcx: &mut VisualTestContext, page: &Entity<Project>) -> Option<Sending> {
    vcx.read_entity(page, |p, _| p.refining().and_then(|r| r.sending))
}

fn refine(
    issue: &str,
    title: Option<&str>,
    checks: Option<Vec<(&str, &str)>>,
    send: bool,
) -> temet_work_core::Request {
    temet_work_core::Request::new(
        None,
        temet_work_core::Command::Refine {
            issue: temet_work_core::ids::IssueId::parse(issue).expect("an issue id"),
            title: title.map(str::to_string),
            checks: checks.map(|checks| {
                checks
                    .into_iter()
                    .map(|(d, p)| temet_work_core::model::Check::new(d, p))
                    .collect()
            }),
            send,
        },
    )
}

#[test]
fn save_sends_refine_with_the_edited_title_and_checks() {
    let daemon = FakeDaemon::serve(temet_work_core::Reply::Done {
        text: "refined watch#WATCH-44".into(),
    });
    let (page, vcx) = page_with(&daemon);
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

    // While the daemon holds its answer, the buttons are busy.
    assert_eq!(sending(vcx, &page), Some(Sending::Save));
    let request = daemon.request();
    assert_eq!(
        request,
        refine(
            "watch#WATCH-44",
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
    daemon.answer();
    until(vcx, |vcx| {
        vcx.read_entity(&page, |p, _| p.refining().is_none())
    });

    // Done: the dialog is closed and the focus is back on the actions button.
    let back = vcx.update(|window, cx| {
        let focus = page.update(cx, |p, cx| p.actions_focus(cx));
        focus.is_focused(window)
    });
    assert!(back, "the focus is back on the actions button");
}

#[test]
fn save_and_send_adds_send_and_a_title_alone_keeps_the_checks() {
    let daemon = FakeDaemon::serve(temet_work_core::Reply::Done {
        text: "sent watch#WATCH-44 to a refiner".into(),
    });
    let (page, vcx) = page_with(&daemon);
    open_refine(vcx);
    type_in(
        vcx,
        "refine-title",
        "Keep the constable's token in the keyring",
    );
    click(vcx, "refine-send");
    assert_eq!(sending(vcx, &page), Some(Sending::SaveAndSend));
    assert_eq!(
        daemon.request(),
        refine(
            "watch#WATCH-44",
            Some("Keep the constable's token in the keyring"),
            None,
            true,
        ),
        "a title edit alone sends no checks"
    );
    daemon.answer();
    until(vcx, |vcx| {
        vcx.read_entity(&page, |p, _| p.refining().is_none())
    });

    // With nothing changed, it still sends the issue to a refiner.
    open_refine(vcx);
    click(vcx, "refine-send");
    assert_eq!(daemon.request(), refine("watch#WATCH-44", None, None, true));
    daemon.answer();
    until(vcx, |vcx| {
        vcx.read_entity(&page, |p, _| p.refining().is_none())
    });
}

#[test]
fn a_refusal_shows_its_reason_and_keeps_the_draft() {
    let why = "the daemon is held: resume it, or Save without send";
    let daemon = FakeDaemon::serve(temet_work_core::Reply::Refused { why: why.into() });
    let (page, vcx) = page_with(&daemon);
    open_refine(vcx);
    type_in(
        vcx,
        "refine-title",
        "Keep the constable's token in the keyring",
    );
    click(vcx, "refine-send");
    daemon.request();
    daemon.answer();
    until(vcx, |vcx| sending(vcx, &page).is_none());

    let (not_done, title) = vcx.read_entity(&page, |p, cx| {
        let r = p.refining().expect("the dialog stays open");
        (r.not_done.clone(), r.draft.title(cx))
    });
    assert_eq!(not_done, Some(NotDone::Refused(why.into())));
    assert_eq!(title, "Keep the constable's token in the keyring");
    assert!(
        vcx.debug_bounds("refine-not-done").is_some(),
        "the reason is drawn in the dialog"
    );
}

#[test]
fn save_with_nothing_changed_or_no_title_sends_nothing() {
    let daemon = FakeDaemon::serve(temet_work_core::Reply::Done {
        text: String::new(),
    });
    let (page, vcx) = page_with(&daemon);
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
    assert!(
        daemon
            .requests
            .recv_timeout(std::time::Duration::from_millis(200))
            .is_err(),
        "nothing was sent"
    );
}
