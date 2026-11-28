//! The daemon entity against a Unix socket each test serves on a thread, in a temp folder of its
//! own. The waits between asks and an ask's deadline run on gpui's fake clock, and no test opens
//! a window. The core holds the real period and deadline; the durations written here are the
//! tests' own.
//!
//! The app reads the socket on a thread of its own, so a test waits in real time for an answer
//! that comes, at most [`WAIT`], and never on gpui's parking, which adds the real time it parks
//! to the fake clock. No assertion depends on a thread answering within a second.

use std::cell::Cell;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use gpui::{AppContext, Entity, TestAppContext};
use temet_work_app::daemon::{Daemon, Problem};
use temet_work_core::command::{VIEW_DEADLINE, VIEW_PERIOD};
use temet_work_core::event::Event;
use temet_work_core::ids::ProjectId;
use temet_work_core::{Command, Reply, Request, View, World};

/// A tick of the fake clock.
const TICK: Duration = Duration::from_millis(1);

/// The longest a test waits in real time for a thread, the socket's or the app's: far longer than
/// either takes on a loaded machine. A test waits this long only when it is about to fail.
const WAIT: Duration = Duration::from_secs(5);

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
    /// A message for each request read, sent once it is counted.
    heard: mpsc::Receiver<()>,
    seen: mpsc::Receiver<Seen>,
    _folder: Folder,
}

impl Socket {
    fn serve(answers: Vec<Answer>) -> Self {
        let folder = Folder::new();
        let path = folder.join("daemon.sock");
        let listener = UnixListener::bind(&path).expect("bind the socket");
        let asks = Arc::new(AtomicUsize::new(0));
        let (hear, heard) = mpsc::channel();
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
                let _ = hear.send(());
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
            heard,
            seen,
            _folder: folder,
        }
    }

    fn asks(&self) -> usize {
        self.asks.load(Ordering::SeqCst)
    }

    /// Waits until the socket has read `n` requests.
    fn heard(&self, n: usize) {
        while self.asks() < n {
            self.heard
                .recv_timeout(WAIT)
                .expect("the socket read a request");
        }
    }

    /// The next connection the socket finished with.
    fn next_seen(&self) -> Seen {
        self.seen
            .recv_timeout(WAIT)
            .expect("the socket saw a connection")
    }
}

/// Makes the entity, which starts its first ask. The app's reading thread wakes the test's
/// executor from outside it, which gpui's test executor allows only once parking is allowed;
/// nothing here parks it.
fn new(cx: &mut TestAppContext, path: PathBuf) -> Entity<Daemon> {
    cx.executor().allow_parking();
    cx.new(|cx| Daemon::at(path, cx))
}

fn start(cx: &mut TestAppContext, path: PathBuf) -> Entity<Daemon> {
    let daemon = new(cx, path);
    cx.run_until_parked();
    daemon
}

/// Waits for the socket to finish with its next connection, then runs what the answer woke. The
/// app hands its answer over before it closes the connection, so the entity has taken the answer
/// when this returns.
fn answered(cx: &mut TestAppContext, socket: &Socket) -> Seen {
    let seen = socket.next_seen();
    cx.run_until_parked();
    seen
}

/// Runs what is ready until `done` holds. Between tries the test's thread parks, at most [`WAIT`]
/// in all, and the app's reading thread unparks it when it wakes the executor. That parking is
/// the thread's own, not gpui's, so the fake clock does not move.
fn until(cx: &mut TestAppContext, done: impl Fn(&mut TestAppContext) -> bool) {
    let end = Instant::now() + WAIT;
    loop {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        let left = end.saturating_duration_since(Instant::now());
        assert!(!left.is_zero(), "not done within {WAIT:?}");
        std::thread::park_timeout(left);
    }
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
    let daemon = start(cx, socket.path.clone());

    let seen = answered(cx, &socket);
    assert_eq!(state(cx, &daemon), (Some(sent), None, false));
    assert!(seen.request.ends_with('\n'), "{:?}", seen.request);
    let request: Request = serde_json::from_str(&seen.request).expect("a request");
    assert_eq!(request, Request::new(None, Command::View));
    assert_eq!(seen.rest, "", "the ask sent more than one line");
}

/// The view a daemon from before milestones, checks and reviews (#130) sends, as the core's
/// tests keep it.
const OLDER_VIEW: &str = include_str!("../../crates/core/tests/fixtures/view-before-130.json");

#[gpui::test]
fn holds_the_view_an_older_daemon_sends(cx: &mut TestAppContext) {
    let sent: serde_json::Value = serde_json::from_str(OLDER_VIEW).expect("json");
    assert_eq!(sent.get("milestones"), None, "not an older view");
    let reply = serde_json::json!({"reply": "view", "view": sent});
    let socket = Socket::serve(vec![Answer::Line(reply.to_string())]);
    let daemon = start(cx, socket.path.clone());

    answered(cx, &socket);
    let want: View = serde_json::from_str(OLDER_VIEW).expect("a view");
    assert_eq!(state(cx, &daemon), (Some(want), None, false));
}

#[gpui::test]
fn no_socket(cx: &mut TestAppContext) {
    let folder = Folder::new();

    // No file at the path.
    let missing = folder.join("daemon.sock");
    let daemon = start(cx, missing.clone());
    until(cx, |cx| state(cx, &daemon).1.is_some());
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
    let daemon = start(cx, left.clone());
    until(cx, |cx| state(cx, &daemon).1.is_some());
    assert_eq!(
        state(cx, &daemon),
        (None, Some(Problem::NoSocket { path: left }), false)
    );
}

#[gpui::test]
fn a_refusal_gives_its_reason(cx: &mut TestAppContext) {
    let why = "no daemon of this user answers at Pseudopolis Yard".to_string();
    let socket = Socket::serve(vec![line(&Reply::Refused { why: why.clone() })]);
    let daemon = start(cx, socket.path.clone());

    answered(cx, &socket);
    assert_eq!(
        state(cx, &daemon),
        (None, Some(Problem::Refused { why }), false)
    );
}

#[gpui::test]
fn an_unreadable_reply_says_why(cx: &mut TestAppContext) {
    // A reply of a kind this build's core lacks, as a newer daemon sends: a problem, and the asks
    // go on.
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
    let daemon = start(cx, socket.path.clone());
    answered(cx, &socket);
    assert_eq!(
        state(cx, &daemon),
        (None, Some(Problem::Unreadable { why: parser }), false)
    );

    wait(cx, VIEW_PERIOD);
    answered(cx, &socket);
    let (view, problem, _) = state(cx, &daemon);
    assert_eq!(view, None);
    let Some(Problem::Unreadable { why }) = problem else {
        panic!("{problem:?}")
    };
    assert!(why.contains("watch#1 is ready"), "{why}");

    wait(cx, VIEW_PERIOD);
    answered(cx, &socket);
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
    let daemon = start(cx, socket.path.clone());
    answered(cx, &socket);
    assert_eq!(state(cx, &daemon), (Some(first.clone()), None, false));

    wait(cx, VIEW_PERIOD);
    answered(cx, &socket);
    assert_eq!(
        state(cx, &daemon),
        (Some(first), Some(Problem::Refused { why }), true)
    );

    wait(cx, VIEW_PERIOD);
    answered(cx, &socket);
    assert_eq!(state(cx, &daemon), (Some(second), None, false));
}

#[gpui::test]
fn asks_once_a_second_and_stops_when_dropped(cx: &mut TestAppContext) {
    let socket = Socket::serve(vec![view_reply(&view("the clacks are down"))]);
    let daemon = start(cx, socket.path.clone());
    answered(cx, &socket);
    assert_eq!(socket.asks(), 1, "it asks as soon as it is made");

    wait(cx, VIEW_PERIOD - TICK);
    assert_eq!(socket.asks(), 1, "before a second has passed");
    wait(cx, TICK);
    answered(cx, &socket);
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
    let daemon = new(cx, socket.path.clone());
    let notified = Rc::new(Cell::new(0));
    let _observing = cx.update(|cx| {
        let notified = notified.clone();
        cx.observe(&daemon, move |_, _| notified.set(notified.get() + 1))
    });

    cx.run_until_parked();
    answered(cx, &socket);
    wait(cx, VIEW_PERIOD);
    answered(cx, &socket);
    wait(cx, VIEW_PERIOD);
    answered(cx, &socket);
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
    let daemon = start(cx, socket.path.clone());
    answered(cx, &socket);
    assert_eq!(state(cx, &daemon), (Some(first.clone()), None, false));

    // The second ask: the socket reads it and never answers. Its deadline passes on the fake
    // clock alone; the read's own timeout, in real time, does not run out during the test. The
    // test waits until the socket has read it, so the third ask is the socket's third connection
    // and gets the third answer.
    wait(cx, VIEW_PERIOD);
    socket.heard(2);
    wait(cx, VIEW_DEADLINE - TICK);
    assert_eq!(
        state(cx, &daemon),
        (Some(first.clone()), None, false),
        "before the deadline"
    );
    wait(cx, TICK);
    assert_eq!(
        state(cx, &daemon),
        (
            Some(first),
            Some(Problem::NoAnswer {
                within: VIEW_DEADLINE
            }),
            true
        )
    );
    assert_eq!(
        Problem::NoAnswer {
            within: VIEW_DEADLINE
        }
        .to_string(),
        "no answer within 5 s"
    );

    wait(cx, VIEW_PERIOD);
    answered(cx, &socket);
    assert_eq!(state(cx, &daemon), (Some(second), None, false));
}

/// Runs `send` on the test's executor to its answer, waiting in real time for the socket's
/// thread.
fn sent(cx: &mut TestAppContext, path: PathBuf, command: Command) -> Result<String, Problem> {
    cx.executor().allow_parking();
    let answer: Rc<std::cell::RefCell<Option<Result<String, Problem>>>> = Rc::default();
    let into = answer.clone();
    let executor = cx.executor();
    cx.foreground_executor()
        .spawn(async move {
            let got = temet_work_app::daemon::send(path, command, &executor).await;
            *into.borrow_mut() = Some(got);
        })
        .detach();
    until(cx, |_| answer.borrow().is_some());
    answer.take().expect("an answer")
}

fn refine_command() -> Command {
    Command::Refine {
        issue: temet_work_core::ids::IssueId::new("watch", "WATCH-12"),
        title: Some("Sam Vimes keeps the night watch's keys".into()),
        checks: None,
        send: true,
    }
}

#[gpui::test]
fn a_command_is_sent_as_a_persons_request_and_done_is_its_answer(cx: &mut TestAppContext) {
    let socket = Socket::serve(vec![line(&Reply::Done {
        text: "refined watch#WATCH-12".into(),
    })]);
    let answer = sent(cx, socket.path.clone(), refine_command());
    assert_eq!(answer, Ok("refined watch#WATCH-12".into()));
    let seen = socket.next_seen();
    let request: Request = serde_json::from_str(&seen.request).expect("a request");
    assert_eq!(request, Request::new(None, refine_command()));
    assert_eq!(seen.rest, "", "the send wrote more than one line");
}

#[gpui::test]
fn a_refused_command_gives_the_reason_word_for_word(cx: &mut TestAppContext) {
    let why = "watch#WATCH-12 is merging: a merge is not undone".to_string();
    let socket = Socket::serve(vec![line(&Reply::Refused { why: why.clone() })]);
    let answer = sent(cx, socket.path.clone(), refine_command());
    assert_eq!(answer, Err(Problem::Refused { why: why.clone() }));
    assert_eq!(answer.unwrap_err().reason(), why);
}

#[gpui::test]
fn a_command_with_no_socket_says_where_it_looked(cx: &mut TestAppContext) {
    let folder = Folder::new();
    let missing = folder.join("daemon.sock");
    let answer = sent(cx, missing.clone(), refine_command());
    assert_eq!(answer, Err(Problem::NoSocket { path: missing }));
}
