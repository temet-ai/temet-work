//! The daemon entity against a Unix socket each test serves on a thread, in a temp folder of its
//! own. The waits between asks run on gpui's fake clock, and no test opens a window. The core
//! holds the real period and deadline; the durations written here are the tests' own.

use std::cell::Cell;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use gpui::{AppContext, Entity, TestAppContext};
use temet_work_app::daemon::{Daemon, Problem};
use temet_work_core::command::{VIEW_DEADLINE, VIEW_PERIOD};
use temet_work_core::event::Event;
use temet_work_core::ids::ProjectId;
use temet_work_core::{Command, Reply, Request, View, World};

/// A deadline short enough for a test to wait it out in real time.
const SHORT: Duration = Duration::from_millis(20);

/// A tick of the fake clock.
const TICK: Duration = Duration::from_millis(1);

/// A folder of the test's own under the temp folder, made private and removed when the test
/// ends. Its path stays short, since a socket's path has a limit of about a hundred bytes.
struct Folder(tempfile::TempDir);

impl Folder {
    fn new() -> Self {
        Self(
            tempfile::Builder::new()
                .prefix("tw-app-")
                .tempdir()
                .expect("a temp folder"),
        )
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.path().join(name)
    }
}

/// What the socket does with a connection once it has read the request line.
#[derive(Clone)]
enum Answer {
    /// Writes this line.
    Line(String),
    /// Closes the connection without writing.
    Close,
    /// Holds the connection open and never writes.
    Silent,
}

fn line(reply: &Reply) -> Answer {
    Answer::Line(serde_json::to_string(reply).expect("json"))
}

fn view_reply(view: &View) -> Answer {
    line(&Reply::View { view: view.clone() })
}

/// A view of one project with one problem line, so two problems make two different views.
fn view(problem: &str) -> View {
    let (world, _) = World::replay(&[Event::ProjectAdded {
        project: ProjectId::new("watch"),
        tracker: "fake".into(),
        name: "ankh-morpork/watch".into(),
        code_host: None,
        repo: None,
    }]);
    let problems = [("tracker".to_string(), problem.to_string())].into();
    View::new(&world, &problems)
}

/// One connection as the socket saw it.
struct Seen {
    /// The first line, with its line end.
    request: String,
    /// What came after the first line, up to the end of the connection.
    rest: String,
}

/// A socket served on a thread. It answers the n-th connection with the n-th answer, and every
/// connection after the last with the last.
struct Socket {
    path: PathBuf,
    /// Requests read; counted before the answer is written, so an ask that has returned is
    /// always counted.
    asks: Arc<AtomicUsize>,
    seen: mpsc::Receiver<Seen>,
    _folder: Folder,
}

impl Socket {
    fn serve(answers: Vec<Answer>) -> Self {
        let folder = Folder::new();
        let path = folder.join("daemon.sock");
        let listener = UnixListener::bind(&path).expect("bind the socket");
        let asks = Arc::new(AtomicUsize::new(0));
        let (tx, seen) = mpsc::channel();
        let counted = asks.clone();
        std::thread::spawn(move || {
            let mut held = Vec::new();
            for (n, stream) in listener.incoming().enumerate() {
                let Ok(stream) = stream else { return };
                let mut reader = BufReader::new(stream);
                let mut request = String::new();
                if reader.read_line(&mut request).is_err() {
                    continue;
                }
                counted.fetch_add(1, Ordering::SeqCst);
                let answer = answers[n.min(answers.len() - 1)].clone();
                match answer {
                    Answer::Line(text) => {
                        let _ = writeln!(reader.get_mut(), "{text}");
                    }
                    Answer::Close => {
                        let _ = reader.get_ref().shutdown(Shutdown::Write);
                    }
                    Answer::Silent => {
                        held.push(reader);
                        continue;
                    }
                }
                let mut rest = String::new();
                let _ = reader.read_to_string(&mut rest);
                let _ = tx.send(Seen { request, rest });
            }
        });
        Self {
            path,
            asks,
            seen,
            _folder: folder,
        }
    }

    fn asks(&self) -> usize {
        self.asks.load(Ordering::SeqCst)
    }

    /// The next connection the socket finished with.
    fn next_seen(&self) -> Seen {
        self.seen
            .recv_timeout(Duration::from_secs(1))
            .expect("the socket saw a connection")
    }
}

fn start(cx: &mut TestAppContext, path: PathBuf, deadline: Duration) -> Entity<Daemon> {
    let daemon = cx.new(|cx| Daemon::at(path, deadline, cx));
    cx.run_until_parked();
    daemon
}

/// Moves the fake clock on and runs what that wakes.
fn wait(cx: &mut TestAppContext, time: Duration) {
    cx.executor().advance_clock(time);
    cx.run_until_parked();
}

/// What the entity holds: its view, its problem and its stale mark.
fn state(
    cx: &mut TestAppContext,
    daemon: &Entity<Daemon>,
) -> (Option<View>, Option<Problem>, bool) {
    daemon.read_with(cx, |d, _| {
        (d.view().cloned(), d.problem().cloned(), d.stale())
    })
}

#[gpui::test]
fn holds_the_view_the_socket_sends(cx: &mut TestAppContext) {
    let sent = view("the clacks are down");
    let socket = Socket::serve(vec![view_reply(&sent)]);
    let daemon = start(cx, socket.path.clone(), VIEW_DEADLINE);

    assert_eq!(state(cx, &daemon), (Some(sent), None, false));
    let seen = socket.next_seen();
    assert!(seen.request.ends_with('\n'), "{:?}", seen.request);
    let request: Request = serde_json::from_str(&seen.request).expect("a request");
    assert_eq!(request, Request::new(None, Command::View));
    assert_eq!(seen.rest, "", "the ask sent more than one line");
}

#[gpui::test]
fn no_socket(cx: &mut TestAppContext) {
    let folder = Folder::new();

    // No file at the path.
    let missing = folder.join("daemon.sock");
    let daemon = start(cx, missing.clone(), VIEW_DEADLINE);
    let want = Problem::NoSocket {
        path: missing.clone(),
    };
    assert_eq!(state(cx, &daemon), (None, Some(want.clone()), false));
    assert_eq!(
        want.to_string(),
        format!("no socket at {}", missing.display())
    );

    // A socket file nothing listens on, as a daemon that stopped leaves behind.
    let left = folder.join("left.sock");
    drop(UnixListener::bind(&left).expect("bind"));
    assert!(left.exists());
    let daemon = start(cx, left.clone(), VIEW_DEADLINE);
    assert_eq!(
        state(cx, &daemon),
        (None, Some(Problem::NoSocket { path: left }), false)
    );
}

#[gpui::test]
fn a_refusal_gives_its_reason(cx: &mut TestAppContext) {
    let why = "no daemon of this user answers at Pseudopolis Yard".to_string();
    let socket = Socket::serve(vec![line(&Reply::Refused { why: why.clone() })]);
    let daemon = start(cx, socket.path.clone(), VIEW_DEADLINE);

    assert_eq!(
        state(cx, &daemon),
        (None, Some(Problem::Refused { why }), false)
    );
}

#[gpui::test]
fn an_unreadable_reply_says_why(cx: &mut TestAppContext) {
    let not_a_reply = r#"{"reply":"shrug"}"#;
    let parser = serde_json::from_str::<Reply>(not_a_reply)
        .expect_err("not a reply")
        .to_string();
    let socket = Socket::serve(vec![
        Answer::Line(not_a_reply.into()),
        line(&Reply::Done {
            text: "watch#1 is ready".into(),
        }),
        Answer::Close,
    ]);
    let daemon = start(cx, socket.path.clone(), VIEW_DEADLINE);
    assert_eq!(
        state(cx, &daemon),
        (None, Some(Problem::Unreadable { why: parser }), false)
    );

    wait(cx, VIEW_PERIOD);
    let (view, problem, _) = state(cx, &daemon);
    assert_eq!(view, None);
    let Some(Problem::Unreadable { why }) = problem else {
        panic!("{problem:?}")
    };
    assert!(why.contains("watch#1 is ready"), "{why}");

    wait(cx, VIEW_PERIOD);
    assert_eq!(
        state(cx, &daemon),
        (
            None,
            Some(Problem::Unreadable {
                why: "the socket closed without a reply".into()
            }),
            false
        )
    );
    assert!(
        Problem::Unreadable { why: "x".into() }
            .to_string()
            .starts_with("can't read the reply")
    );
}

#[gpui::test]
fn a_good_reply_clears_the_stale_mark(cx: &mut TestAppContext) {
    let first = view("the clacks are down");
    let second = view("the clacks are slow");
    let why = "the daemon is shutting down".to_string();
    let socket = Socket::serve(vec![
        view_reply(&first),
        line(&Reply::Refused { why: why.clone() }),
        view_reply(&second),
    ]);
    let daemon = start(cx, socket.path.clone(), VIEW_DEADLINE);
    assert_eq!(state(cx, &daemon), (Some(first.clone()), None, false));

    wait(cx, VIEW_PERIOD);
    assert_eq!(
        state(cx, &daemon),
        (Some(first), Some(Problem::Refused { why }), true)
    );

    wait(cx, VIEW_PERIOD);
    assert_eq!(state(cx, &daemon), (Some(second), None, false));
}

#[gpui::test]
fn asks_once_a_second_and_stops_when_dropped(cx: &mut TestAppContext) {
    let socket = Socket::serve(vec![view_reply(&view("the clacks are down"))]);
    let daemon = start(cx, socket.path.clone(), VIEW_DEADLINE);
    assert_eq!(socket.asks(), 1, "it asks as soon as it is made");

    wait(cx, VIEW_PERIOD - TICK);
    assert_eq!(socket.asks(), 1, "before a second has passed");
    wait(cx, TICK);
    assert_eq!(socket.asks(), 2, "once a second has passed");

    let weak = daemon.downgrade();
    drop(daemon);
    cx.update(|_| {});
    assert!(weak.upgrade().is_none(), "the entity is released");
    wait(cx, VIEW_PERIOD * 5);
    assert_eq!(socket.asks(), 2, "after the entity is released");
}

#[gpui::test]
fn an_unchanged_view_does_not_notify(cx: &mut TestAppContext) {
    let socket = Socket::serve(vec![view_reply(&view("the clacks are down"))]);
    let daemon = cx.new(|cx| Daemon::at(socket.path.clone(), VIEW_DEADLINE, cx));
    let notified = Rc::new(Cell::new(0));
    let _observing = cx.update(|cx| {
        let notified = notified.clone();
        cx.observe(&daemon, move |_, _| notified.set(notified.get() + 1))
    });

    cx.run_until_parked();
    wait(cx, VIEW_PERIOD);
    wait(cx, VIEW_PERIOD);
    assert_eq!(socket.asks(), 3);
    assert_eq!(
        notified.get(),
        1,
        "only the first answer changed the entity"
    );
}

#[gpui::test]
fn no_answer_in_time_marks_it_stale(cx: &mut TestAppContext) {
    let first = view("the clacks are down");
    let second = view("the clacks are slow");
    let socket = Socket::serve(vec![
        view_reply(&first),
        Answer::Silent,
        view_reply(&second),
    ]);
    let daemon = start(cx, socket.path.clone(), SHORT);
    assert_eq!(state(cx, &daemon), (Some(first.clone()), None, false));

    wait(cx, VIEW_PERIOD);
    assert_eq!(socket.asks(), 2);
    assert_eq!(
        state(cx, &daemon),
        (Some(first), Some(Problem::NoAnswer { within: SHORT }), true)
    );
    assert_eq!(
        Problem::NoAnswer {
            within: VIEW_DEADLINE
        }
        .to_string(),
        "no answer within 5 s"
    );

    wait(cx, VIEW_PERIOD);
    assert_eq!(state(cx, &daemon), (Some(second), None, false));
}
