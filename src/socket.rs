//! The daemon's socket: one JSON line per request and one per reply, over a
//! Unix socket. The command line uses it; the app and the MCP server will.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

use serde::{Deserialize, Serialize};
use temet_work_core::{Host, Request};

/// The daemon's answer to one request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum Reply {
    /// Done; the text says what changed, or holds the state for `status`.
    Done {
        /// What to show.
        text: String,
    },
    /// Refused, and why.
    Refused {
        /// Why, in a line.
        why: String,
    },
}

/// Where the daemon listens: `TEMET_WORK_SOCKET`, else
/// `$XDG_RUNTIME_DIR/temet-work/daemon.sock`, else under the user's own state
/// folder. Never the shared temp folder, where another user could make the path
/// first.
pub fn path() -> PathBuf {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    if let Some(p) = var("TEMET_WORK_SOCKET") {
        return PathBuf::from(p);
    }
    let dir = var("XDG_RUNTIME_DIR").map_or_else(
        || {
            var("HOME")
                .map(PathBuf::from)
                .unwrap_or_default()
                .join(".local/state")
        },
        PathBuf::from,
    );
    dir.join("temet-work").join("daemon.sock")
}

/// A request waiting for the daemon's reply.
pub struct Pending {
    /// The request.
    pub request: Request,
    reply: Sender<Reply>,
}

impl Pending {
    /// Sends the reply back to whoever asked.
    pub fn answer(self, reply: Reply) {
        let _ = self.reply.send(reply);
    }
}

/// Listens at `path` and hands each request to the returned channel, from a
/// thread per connection. Refuses when a daemon already answers there.
pub fn listen(path: &Path) -> Result<Receiver<Pending>, String> {
    if UnixStream::connect(path).is_ok() {
        return Err(format!(
            "a daemon is already listening at {}",
            path.display()
        ));
    }
    // Nothing answers: what is there is left from a daemon that stopped.
    let _ = std::fs::remove_file(path);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let listener = UnixListener::bind(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let tx = tx.clone();
            std::thread::spawn(move || serve(stream, &tx));
        }
    });
    Ok(rx)
}

fn serve(stream: UnixStream, tx: &Sender<Pending>) {
    let Ok(read) = stream.try_clone() else { return };
    let mut writer = stream;
    for line in BufReader::new(read).lines() {
        let Ok(line) = line else { return };
        let reply = match serde_json::from_str::<Request>(&line) {
            Err(e) => Reply::Refused {
                why: format!("not a request: {e}"),
            },
            Ok(request) => {
                let (reply, answered) = mpsc::channel();
                if tx.send(Pending { request, reply }).is_err() {
                    return;
                }
                answered.recv().unwrap_or(Reply::Refused {
                    why: "the daemon stopped before it answered".into(),
                })
            }
        };
        let Ok(text) = serde_json::to_string(&reply) else {
            return;
        };
        if writeln!(writer, "{text}").is_err() {
            return;
        }
    }
}

/// Sends one request to the daemon at `path` and returns its reply.
pub fn send(path: &Path, request: &Request) -> Result<Reply, String> {
    let mut stream = UnixStream::connect(path).map_err(|e| {
        format!(
            "no temet-work daemon answers at {} ({e}); it may be restarting: run the command again in a minute",
            path.display()
        )
    })?;
    let line = serde_json::to_string(request).map_err(|e| e.to_string())?;
    writeln!(stream, "{line}").map_err(|e| e.to_string())?;
    let mut reply = String::new();
    BufReader::new(stream)
        .read_line(&mut reply)
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&reply).map_err(|e| format!("the daemon's reply is not readable: {e}"))
}

/// Answers one request against the host. The error stops the daemon: an event
/// could not be logged.
pub fn answer(host: &mut Host, now_ms: u64, request: &Request) -> Result<Reply, String> {
    match host.world().check(request.by.as_ref(), &request.command) {
        Err(why) => Ok(Reply::Refused { why }),
        Ok(None) => Ok(match &request.command {
            temet_work_core::Command::Issue { issue } => Reply::Done {
                text: temet_work_core::text::issue(host.world(), issue).unwrap_or_default(),
            },
            temet_work_core::Command::Screen { agent } => match host.screen(agent) {
                Ok(text) => Reply::Done { text },
                Err(why) => Reply::Refused { why },
            },
            _ => Reply::Done {
                text: temet_work_core::text::text(host.world(), host.problems()),
            },
        }),
        Ok(Some(event)) => {
            host.handle(now_ms, event)?;
            let text = request
                .command
                .issue()
                .and_then(|i| host.world().issues.get(i))
                .map_or_else(
                    || "done".to_string(),
                    |i| format!("{} is {}", i.id, temet_work_core::text::stage(i.stage)),
                );
            Ok(Reply::Done { text })
        }
    }
}
