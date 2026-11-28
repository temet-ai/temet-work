//! Clicks on the Project page, drawn under GPUI's test support with the design system's faces
//! and no window: a tab, Group by, a row, Overturn with its confirm, a long check's plus
//! and minus, and Assign and Unassign in the issue's actions menu, which send their command to a
//! socket the test serves. Run with
//! `cargo test --features shots --test clicks`.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use gpui::{
    AppContext, Entity, InteractiveElement, IntoElement, Modifiers, ParentElement, Styled,
    TestAppContext, TestDispatcher, VisualTestContext, div, px,
};
use gpui_component::Root;
use temet_work_app::components::Intent;
use temet_work_app::components::status_tag::Status;
use temet_work_app::composites::check_list::{CheckList, PlanCheck};
use temet_work_app::composites::issue_pane::IssueTab;
use temet_work_app::pages::project::{Grouping, Project};
use temet_work_app::{assets, sample, theme};
use temet_work_core::ids::{AgentId, IssueId};
use temet_work_core::{Command, Reply, Request};

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

/// The longest a test waits in real time for the socket's thread or the page's: far longer than
/// either takes on a loaded machine. A test waits this long only when it is about to fail.
const WAIT: Duration = Duration::from_secs(5);

/// A socket in a private temp folder that answers each request line with `reply` and hands the
/// request line over.
struct Socket {
    path: PathBuf,
    requests: mpsc::Receiver<String>,
    _folder: tempfile::TempDir,
}

impl Socket {
    fn serve(reply: Reply) -> Self {
        let folder = tempfile::Builder::new()
            .prefix("tw-clicks-")
            .tempdir()
            .expect("a temp folder");
        let path = folder.path().join("daemon.sock");
        let listener = UnixListener::bind(&path).expect("bind the socket");
        let line = serde_json::to_string(&reply).expect("json");
        let (tx, requests) = mpsc::channel();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { return };
                let mut reader = BufReader::new(stream);
                let mut request = String::new();
                if reader.read_line(&mut request).is_err() {
                    continue;
                }
                let _ = writeln!(reader.get_mut(), "{line}");
                let _ = tx.send(request);
            }
        });
        Self {
            path,
            requests,
            _folder: folder,
        }
    }

    /// The next request line the socket read, as a request.
    fn request(&self) -> Request {
        let line = self
            .requests
            .recv_timeout(WAIT)
            .expect("the page sent a request");
        serde_json::from_str(&line).unwrap_or_else(|e| panic!("{line:?} is not a request: {e}"))
    }
}

/// The sample page in a test window, sending to `socket`. The page's reading thread wakes the
/// test's executor from outside it, which gpui's test executor allows only once parking is.
fn page_sending_to(socket: &Socket) -> (Entity<Project>, &'static mut VisualTestContext) {
    let (page, vcx) = page();
    vcx.executor().allow_parking();
    let path = socket.path.clone();
    vcx.update(|_, cx| page.update(cx, |p, _| p.set_socket(Some(path))));
    (page, vcx)
}

/// Runs what is ready until `done` holds, parking the test's thread between tries, at most
/// [`WAIT`] in all; the page's reading thread unparks it when it wakes the executor. The parking
/// is the thread's own, not gpui's, so the fake clock does not move.
fn until(vcx: &mut VisualTestContext, done: impl Fn(&mut VisualTestContext) -> bool) {
    let end = Instant::now() + WAIT;
    loop {
        vcx.run_until_parked();
        if done(vcx) {
            return;
        }
        let left = end.saturating_duration_since(Instant::now());
        assert!(!left.is_zero(), "not done within {WAIT:?}");
        std::thread::park_timeout(left);
    }
}

fn person(command: Command) -> Request {
    Request::new(None, command)
}

fn watch_44() -> IssueId {
    IssueId::parse("watch#44").expect("an issue id")
}

#[test]
fn assign_sends_the_agent_picked_from_the_actions_menu() {
    let socket = Socket::serve(Reply::Done {
        text: "assigned py-1 to watch#44".into(),
    });
    let (page, vcx) = page_sending_to(&socket);
    click(vcx, "issue-actions");
    // The menu lists only the agents the view says a person may assign: not nw-1, a running
    // refiner of the group that the issue's review does not need.
    click(vcx, "issue-actions-assign");
    assert!(vcx.debug_bounds("assign-py-1").is_some());
    assert!(vcx.debug_bounds("assign-nw-1").is_none());
    click(vcx, "assign-py-1");
    assert_eq!(
        socket.request(),
        person(Command::Assign {
            issue: watch_44(),
            agent: AgentId::new("py-1"),
        })
    );
    // A done reply needs no word, and picking an agent closed the menu.
    vcx.run_until_parked();
    draw(vcx);
    assert!(vcx.read_entity(&page, |p, _| p.toast().is_none()));
    assert!(vcx.debug_bounds("assign-py-1").is_none());
}

#[test]
fn unassign_sends_the_agent_picked_from_the_actions_menu() {
    let socket = Socket::serve(Reply::Done {
        text: "unassigned nw-3 from watch#44".into(),
    });
    let (_, vcx) = page_sending_to(&socket);
    click(vcx, "issue-actions");
    click(vcx, "issue-actions-unassign");
    assert!(vcx.debug_bounds("unassign-nw-2").is_some());
    click(vcx, "unassign-nw-3");
    assert_eq!(
        socket.request(),
        person(Command::Unassign {
            issue: watch_44(),
            agent: AgentId::new("nw-3"),
        })
    );
}

#[test]
fn a_refused_assign_shows_the_daemon_reply_in_a_danger_toast() {
    let why = "py-1 is a reviewer, and watch#44 is with the author: it needs no reviewer now";
    let socket = Socket::serve(Reply::Refused { why: why.into() });
    let (page, vcx) = page_sending_to(&socket);
    click(vcx, "issue-actions");
    click(vcx, "issue-actions-assign");
    click(vcx, "assign-py-1");
    socket.request();
    until(vcx, |vcx| {
        vcx.read_entity(&page, |p, _| p.toast().is_some())
    });
    let (toast, intent) = vcx.read_entity(&page, |p, _| (p.toast().cloned(), p.toast_intent()));
    assert_eq!(
        toast.as_deref(),
        Some(format!("Assign py-1 to watch#44: {why}").as_str())
    );
    assert_eq!(intent, Some(Intent::Danger));
    draw(vcx);
    assert!(
        vcx.debug_bounds("toast-close").is_some(),
        "the toast is drawn"
    );
}
