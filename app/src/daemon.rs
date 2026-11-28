//! Plumbing: the daemon's view, read over its socket and kept fresh in an entity the window
//! observes. It draws nothing.
//!
//! An ask is the CLI's `send` with a deadline: one `Request { by: None, command: View }` line
//! to the socket, one `Reply` line back, within [`VIEW_DEADLINE`] on the executor's clock. The
//! next ask starts [`VIEW_PERIOD`] after the last one ended, so asks never overlap. A failed ask
//! keeps the last view and marks it stale; the next good one clears the mark.
//!
//! A command the person gives in the window, such as moving an issue, goes the same way: one
//! `Request { by: None, command }` line, as a person's request, and one `Reply` line back within
//! [`VIEW_DEADLINE`] ([`send`]): the daemon's text when it did it, or why it was not done.
//!
//! The `Reply` match has no wildcard arm: a reply the core adds stops this file compiling until
//! the app says what it does with it. A reply line of a kind this build's core lacks, as a newer
//! daemon sends, is [`Problem::Unreadable`] with the parser's message.

use std::fmt;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use futures_channel::oneshot;
use gpui::{BackgroundExecutor, Context, FutureExt, Task, Timeout};
use temet_work_core::command::{VIEW_DEADLINE, VIEW_PERIOD};
use temet_work_core::{Command, Reply, Request, View};

/// Why the entity has no view, or why the one it holds is stale.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// Nothing answers at the socket's path: there is no file there, or there is a socket file
    /// that nothing listens on, as a daemon that stopped leaves behind.
    NoSocket {
        /// The path the ask tried.
        path: PathBuf,
    },
    /// The daemon refused the ask.
    Refused {
        /// The daemon's reason, word for word.
        why: String,
    },
    /// The answer is not a view: a line that is not a reply (a reply of a kind this build does
    /// not know among them), a reply of another kind, a socket that failed or closed before it
    /// sent a line, or no thread to read the socket on.
    Unreadable {
        /// The parser's or the socket's message.
        why: String,
    },
    /// The daemon took the connection and sent no answer in time.
    NoAnswer {
        /// How long the ask waited.
        within: Duration,
    },
}

impl Problem {
    /// Why it went wrong, for a person: the daemon's reason word for word when it refused, else
    /// what failed.
    pub fn reason(&self) -> String {
        match self {
            Self::Refused { why } => why.clone(),
            other => other.to_string(),
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSocket { path } => write!(f, "no socket at {}", path.display()),
            Self::Refused { why } => write!(f, "the daemon refused: {why}"),
            Self::Unreadable { why } => write!(f, "can't read the reply: {why}"),
            Self::NoAnswer { within } if within.subsec_nanos() == 0 => {
                write!(f, "no answer within {} s", within.as_secs())
            }
            Self::NoAnswer { within } => write!(f, "no answer within {} ms", within.as_millis()),
        }
    }
}

/// The daemon's view as the app last read it. Observers are notified only when what it holds
/// changes, so an idle window is not redrawn on every ask.
pub struct Daemon {
    path: PathBuf,
    view: Option<View>,
    problem: Option<Problem>,
    /// The loop of asks. Dropping the entity drops it, which stops the asks.
    _asks: Task<()>,
}

impl Daemon {
    /// Asks the daemon at the socket this process's environment names, by the core's rule
    /// ([`temet_work_core::socket::path`]).
    pub fn new(cx: &mut Context<Self>) -> Self {
        let var = std::env::var_os;
        let path = temet_work_core::socket::path(
            var("TEMET_WORK_SOCKET"),
            var("XDG_RUNTIME_DIR"),
            var("HOME"),
        );
        Self::at(path, cx)
    }

    /// Asks the daemon at `path` now and then [`VIEW_PERIOD`] after each ask ends. An ask with
    /// no answer within [`VIEW_DEADLINE`] on the executor's clock fails.
    pub fn at(path: PathBuf, cx: &mut Context<Self>) -> Self {
        let socket = path.clone();
        let asks = cx.spawn(async move |this, cx| {
            loop {
                let executor = cx.background_executor().clone();
                let answer = ask(socket.clone(), &executor).await;
                if this
                    .update(cx, |daemon, cx| daemon.take(answer, cx))
                    .is_err()
                {
                    return;
                }
                cx.background_executor().timer(VIEW_PERIOD).await;
            }
        });
        Self {
            path,
            view: None,
            problem: None,
            _asks: asks,
        }
    }

    /// The socket's path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The last view the daemon sent; `None` before the first good ask.
    pub fn view(&self) -> Option<&View> {
        self.view.as_ref()
    }

    /// Why the last ask failed; `None` after a good one.
    pub fn problem(&self) -> Option<&Problem> {
        self.problem.as_ref()
    }

    /// Whether the view is from an ask before the last, which failed.
    pub fn stale(&self) -> bool {
        self.view.is_some() && self.problem.is_some()
    }

    fn take(&mut self, answer: Result<View, Problem>, cx: &mut Context<Self>) {
        let changed = match answer {
            Ok(view) => {
                let changed = self.problem.is_some() || self.view.as_ref() != Some(&view);
                self.view = Some(view);
                self.problem = None;
                changed
            }
            Err(problem) => {
                let changed = self.problem.as_ref() != Some(&problem);
                self.problem = Some(problem);
                changed
            }
        };
        if changed {
            cx.notify();
        }
    }
}

/// Sends `command` to the daemon at `path` as a person's request, and gives the daemon's text
/// when it did it, or why it was not done: its refusal, no socket, or no answer once
/// [`VIEW_DEADLINE`] has passed on `executor`'s clock.
pub async fn send(
    path: PathBuf,
    command: Command,
    executor: &BackgroundExecutor,
) -> Result<String, Problem> {
    let reply = over_socket(path, command, executor).await?;
    match reply {
        Reply::Done { text } => Ok(text),
        Reply::Refused { why } => Err(Problem::Refused { why }),
        Reply::View { .. } => Err(Problem::Unreadable {
            why: "a view in place of the command's answer".into(),
        }),
    }
}

/// One ask: a `View` request to the socket at `path` and the view it answers, or
/// [`Problem::NoAnswer`] once [`VIEW_DEADLINE`] has passed on `executor`'s clock.
///
/// A socket read blocks, so it runs on a thread of its own, raced against a timer on the
/// executor. The timer decides the deadline, so under gpui's test executor the fake clock does.
/// The read's own timeout is the same deadline in real time: it gives the same answer and ends
/// the thread, which under a fake clock is all it does.
async fn ask(path: PathBuf, executor: &BackgroundExecutor) -> Result<View, Problem> {
    match over_socket(path, Command::View, executor).await? {
        Reply::View { view } => Ok(view),
        Reply::Refused { why } => Err(Problem::Refused { why }),
        Reply::Done { text } => Err(Problem::Unreadable {
            why: format!("a done reply in place of a view: {text}"),
        }),
    }
}

/// One request: `command` as a person's to the socket at `path`, and the reply line read, or
/// [`Problem::NoAnswer`] once [`VIEW_DEADLINE`] has passed on `executor`'s clock.
async fn over_socket(
    path: PathBuf,
    command: Command,
    executor: &BackgroundExecutor,
) -> Result<Reply, Problem> {
    let (answer, answered) = oneshot::channel();
    let reader = std::thread::Builder::new()
        .name("temet-work-view".into())
        .spawn(move || read(&path, &command, answer));
    if let Err(e) = reader {
        return Err(Problem::Unreadable { why: e.to_string() });
    }
    match answered.with_timeout(VIEW_DEADLINE, executor).await {
        Ok(Ok(answer)) => answer,
        Ok(Err(oneshot::Canceled)) => Err(Problem::Unreadable {
            why: "the thread reading the socket stopped without an answer".into(),
        }),
        Err(Timeout) => Err(Problem::NoAnswer {
            within: VIEW_DEADLINE,
        }),
    }
}

/// The reading thread: connects to the socket at `path`, hands the answer over, and only then
/// closes the connection, so whoever waits for the connection to end (the app's tests do) finds
/// the answer already handed over.
fn read(path: &Path, command: &Command, answer: oneshot::Sender<Result<Reply, Problem>>) {
    match UnixStream::connect(path) {
        Ok(stream) => {
            let _ = answer.send(exchange(&stream, command));
            drop(stream);
        }
        Err(_) => {
            let _ = answer.send(Err(Problem::NoSocket {
                path: path.to_path_buf(),
            }));
        }
    }
}

/// Writes `command` to `stream` as a person's request and reads the reply, waiting at most
/// [`VIEW_DEADLINE`] on each write and read.
fn exchange(stream: &UnixStream, command: &Command) -> Result<Reply, Problem> {
    let failed = |e: io::Error| match e.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => Problem::NoAnswer {
            within: VIEW_DEADLINE,
        },
        _ => Problem::Unreadable { why: e.to_string() },
    };
    stream
        .set_write_timeout(Some(VIEW_DEADLINE))
        .map_err(failed)?;
    stream
        .set_read_timeout(Some(VIEW_DEADLINE))
        .map_err(failed)?;
    let request = Request::new(None, command.clone());
    let line =
        serde_json::to_string(&request).map_err(|e| Problem::Unreadable { why: e.to_string() })?;
    writeln!(&*stream, "{line}").map_err(failed)?;
    let mut reply = String::new();
    let read = BufReader::new(stream)
        .read_line(&mut reply)
        .map_err(failed)?;
    if read == 0 {
        return Err(Problem::Unreadable {
            why: "the socket closed without a reply".into(),
        });
    }
    serde_json::from_str(&reply).map_err(|e| Problem::Unreadable { why: e.to_string() })
}
