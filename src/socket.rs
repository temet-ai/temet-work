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

/// The variable that picks how a request reaches the daemon: unset or empty is
/// the local socket, `ssh://[USER@]HOST[:PORT][/SOCKET/PATH]` is SSH.
pub const DAEMON_VAR: &str = "TEMET_WORK_DAEMON";

/// How a request gets to the daemon and its reply back. The command line and the
/// MCP server send through this and nothing else, so another way to reach the
/// daemon (TCP with a token, say) needs no change in them.
pub trait Transport {
    /// Where it goes, for tests and messages: `local:PATH`, or the ssh target and port.
    fn describe(&self) -> (String, Option<String>);

    /// Sends one request and returns the daemon's reply. An `Err` is a failure to
    /// reach the daemon, in one line; a refusal is `Ok(Reply::Refused)`.
    fn send(&self, request: &Request) -> Result<Reply, String>;
}

/// The daemon's own Unix socket on this machine.
pub struct Local {
    /// The socket's path.
    pub path: PathBuf,
}

impl Transport for Local {
    fn describe(&self) -> (String, Option<String>) {
        (format!("local:{}", self.path.display()), None)
    }

    fn send(&self, request: &Request) -> Result<Reply, String> {
        send(&self.path, request)
    }
}

/// The daemon's socket on another machine, reached by running
/// `ssh -o BatchMode=yes HOST temet-work relay` there. No listener and no secret
/// beyond the SSH access that already exists; BatchMode makes a missing key fail
/// with ssh's own error instead of waiting for a password nobody can type.
pub struct Ssh {
    /// `HOST` or `USER@HOST`, as ssh takes it.
    pub target: String,
    /// The port, when not the default.
    pub port: Option<String>,
    /// The daemon's socket on that machine, when not its default.
    pub socket: Option<String>,
    /// The ssh program; `ssh`.
    pub program: String,
}

impl Ssh {
    fn hop(&self, what: &str) -> String {
        format!("ssh {}: {what}", self.target)
    }
}

impl Transport for Ssh {
    fn describe(&self) -> (String, Option<String>) {
        match &self.socket {
            Some(s) => (format!("{}{s}", self.target), self.port.clone()),
            None => (self.target.clone(), self.port.clone()),
        }
    }

    fn send(&self, request: &Request) -> Result<Reply, String> {
        use std::process::{Command, Stdio};
        let line = serde_json::to_string(request).map_err(|e| e.to_string())?;
        let mut cmd = Command::new(&self.program);
        cmd.args(["-o", "BatchMode=yes"]);
        if let Some(port) = &self.port {
            cmd.args(["-p", port]);
        }
        // `--` ends ssh's options, so the target is never read as one.
        cmd.args(["--", self.target.as_str(), "temet-work", "relay"]);
        if let Some(socket) = &self.socket {
            cmd.args(["--socket", socket]);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| self.hop(&format!("cannot run {}: {e}", self.program)))?;
        if let Some(mut stdin) = child.stdin.take() {
            // A hop that fails early closes its end before it reads: the exit
            // status below says why, so a broken pipe here is not the answer.
            let _ = writeln!(stdin, "{line}");
        }
        let out = child.wait_with_output().map_err(|e| e.to_string())?;
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stderr = stderr.trim().lines().last().unwrap_or("").to_string();
        match out.status.code() {
            Some(0) => {}
            Some(255) => return Err(self.hop(&format!("could not connect: {stderr}"))),
            Some(127) => {
                return Err(self.hop(
                    "temet-work is not on the other machine's PATH: install it there, on the PATH a non-interactive ssh gets",
                ));
            }
            Some(RELAY_NO_DAEMON) => return Err(self.hop(&stderr)),
            other => {
                return Err(self.hop(&format!(
                    "the relay failed ({}): {stderr}",
                    other.map_or("killed".to_string(), |c| format!("exit {c}"))
                )));
            }
        }
        serde_json::from_slice(&out.stdout)
            .map_err(|e| format!("the daemon's reply is not readable: {e}"))
    }
}

/// The exit code of `temet-work relay` when no daemon answers on its machine.
pub const RELAY_NO_DAEMON: i32 = 4;

/// Picks the transport from the value of `TEMET_WORK_DAEMON` (`None` or empty:
/// the local socket at `socket`).
pub fn transport_for(value: Option<&str>, socket: PathBuf) -> Result<Box<dyn Transport>, String> {
    let Some(value) = value.filter(|v| !v.is_empty()) else {
        return Ok(Box::new(Local { path: socket }));
    };
    let refused = || {
        format!(
            "{DAEMON_VAR}={value} is not understood: leave it unset for the local socket, or write ssh://[USER@]HOST[:PORT], optionally followed by the daemon's socket path there, as ssh://HOST/run/user/1000/temet-work/daemon.sock"
        )
    };
    let rest = value.strip_prefix("ssh://").ok_or_else(refused)?;
    // `HOST/ABSOLUTE/SOCKET/PATH`: the path names the daemon's socket there.
    let (rest, socket) = match rest.find('/') {
        Some(i) => (&rest[..i], Some(rest[i..].to_string()).filter(|p| p != "/")),
        None => (rest, None),
    };
    let (target, port) = match rest.rsplit_once(':') {
        Some((t, p)) if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => {
            (t, Some(p.to_string()))
        }
        _ => (rest, None),
    };
    // A leading `-` would be read by ssh as an option (`-oProxyCommand=...`).
    if target.is_empty() || target.ends_with('@') || target.starts_with('-') || target.contains('/')
    {
        return Err(refused());
    }
    Ok(Box::new(Ssh {
        target: target.to_string(),
        port,
        socket,
        program: "ssh".into(),
    }))
}

/// The transport the environment names: `TEMET_WORK_DAEMON`, else the local socket.
pub fn from_env() -> Result<Box<dyn Transport>, String> {
    transport_for(std::env::var(DAEMON_VAR).ok().as_deref(), path())
}

/// `temet-work relay`: copies one request line from `input` to the daemon's
/// socket at `path` and its reply line to `out`, so the other end of an SSH hop
/// speaks the daemon's own wire format.
pub fn relay(path: &Path, input: impl BufRead, mut out: impl Write) -> Result<(), String> {
    let mut line = String::new();
    let mut input = input;
    input.read_line(&mut line).map_err(|e| e.to_string())?;
    let mut stream = connect(path)?;
    stream
        .write_all(line.trim_end().as_bytes())
        .and_then(|()| stream.write_all(b"\n"))
        .map_err(|e| e.to_string())?;
    let mut reply = String::new();
    BufReader::new(stream)
        .read_line(&mut reply)
        .map_err(|e| e.to_string())?;
    out.write_all(reply.as_bytes())
        .and_then(|()| out.flush())
        .map_err(|e| e.to_string())
}

fn connect(path: &Path) -> Result<UnixStream, String> {
    UnixStream::connect(path).map_err(|e| {
        format!(
            "no temet-work daemon answers at {} ({e}); it may be restarting: run the command again in a minute",
            path.display()
        )
    })
}

/// Sends one request to the daemon at `path` and returns its reply.
pub fn send(path: &Path, request: &Request) -> Result<Reply, String> {
    let mut stream = connect(path)?;
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
    match host.world().events(request.by.as_ref(), &request.command) {
        Err(why) => Ok(Reply::Refused { why }),
        Ok(events) if events.is_empty() => Ok(match &request.command {
            temet_work_core::Command::Settings { target, role } => {
                match temet_work_core::text::settings(host.world(), target, *role) {
                    Ok(text) => Reply::Done { text },
                    Err(why) => Reply::Refused { why },
                }
            }
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
        Ok(events) => {
            for event in events {
                host.handle(now_ms, event)?;
            }
            match &request.command {
                temet_work_core::Command::Set {
                    target,
                    role,
                    key,
                    value,
                } => {
                    let for_role = role.map_or_else(String::new, |r| format!(" for {}", r.name()));
                    return Ok(Reply::Done {
                        text: format!("{key} is {value} at {target}{for_role}"),
                    });
                }
                temet_work_core::Command::Unset {
                    target, role, key, ..
                } => {
                    let for_role = role.map_or_else(String::new, |r| format!(" for {}", r.name()));
                    return Ok(Reply::Done {
                        text: format!("{key} is no longer set at {target}{for_role}"),
                    });
                }
                _ => (),
            };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn target(value: Option<&str>) -> Result<(String, Option<String>), String> {
        // Reads the choice back through the Ssh fields by sending nothing: the
        // parse is checked through `describe`.
        transport_for(value, PathBuf::from("/s")).map(|t| t.describe())
    }

    #[test]
    fn unset_or_empty_is_the_local_socket() {
        assert_eq!(target(None), Ok(("local:/s".into(), None)));
        assert_eq!(target(Some("")), Ok(("local:/s".into(), None)));
    }

    #[test]
    fn ssh_takes_a_host_a_user_and_a_port() {
        assert_eq!(target(Some("ssh://omit")), Ok(("omit".into(), None)));
        assert_eq!(
            target(Some("ssh://dawid@omit:2222")),
            Ok(("dawid@omit".into(), Some("2222".into())))
        );
    }

    #[test]
    fn any_other_scheme_is_refused_naming_the_variable_and_the_forms() {
        let e = target(Some("http://x")).expect_err("refused");
        assert!(
            e.contains("TEMET_WORK_DAEMON") && e.contains("ssh://HOST"),
            "{e}"
        );
        assert!(target(Some("ssh://")).is_err());
        assert!(target(Some("ssh://-oProxyCommand=touch%20x")).is_err());
        assert!(target(Some("ssh://-oProxyCommand=x/run/s.sock")).is_err());
        assert_eq!(
            target(Some("ssh://omit/run/user/1000/temet-work/daemon.sock")),
            Ok(("omit/run/user/1000/temet-work/daemon.sock".into(), None))
        );
        assert_eq!(target(Some("ssh://omit/")), Ok(("omit".into(), None)));
    }
}
