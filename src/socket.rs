//! The daemon's socket: one JSON line per request and one per reply, over a
//! Unix socket. The command line uses it; the app and the MCP server will.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

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

/// `ConnectTimeout`, in seconds: long enough for a slow VPN or mobile link to
/// connect, and short enough that an agent's own command timeout does not fire
/// first. Without it an unreachable host takes the kernel's 75 to 127 s.
pub const SSH_CONNECT_TIMEOUT_S: u32 = 10;

/// `ServerAliveInterval`, in seconds: how long an open link may be silent before
/// ssh asks the other end whether it is still there. Without it a dropped link
/// takes TCP keepalive, about two hours.
pub const SSH_ALIVE_INTERVAL_S: u32 = 15;

/// `ServerAliveCountMax`: how many of those questions may go unanswered. ssh
/// gives up when the next one falls due, so a silent link ends after about
/// `SSH_ALIVE_INTERVAL_S * (SSH_ALIVE_COUNT_MAX + 1)`, 45 s: inside
/// [`SSH_DEADLINE`], so a dropped link is still reported as ssh's own failure.
pub const SSH_ALIVE_COUNT_MAX: u32 = 2;

/// The longest a whole hop may take: the connect limit and time for the daemon to
/// answer. Above any normal reply, below the 120 s an agent's tool call is
/// typically given, so the agent reads why instead of being killed.
pub const SSH_DEADLINE: Duration = Duration::from_secs(60);

/// How long the output of an ssh that has exited may stay open. A process it left
/// behind (a `ControlPersist` master) can hold the pipe long after the reply.
const SSH_OUTPUT_GRACE: Duration = Duration::from_millis(500);

/// The daemon's socket on another machine, reached by running
/// `ssh -o BatchMode=yes -o ConnectTimeout=10 -o ServerAliveInterval=15
/// -o ServerAliveCountMax=2 HOST temet-work relay` there. No listener and no
/// secret beyond the SSH access that already exists; BatchMode makes a missing key
/// fail with ssh's own error instead of waiting for a password nobody can type.
/// The other three options limit the connect and the silence on an open link
/// ([`SSH_CONNECT_TIMEOUT_S`], [`SSH_ALIVE_INTERVAL_S`], [`SSH_ALIVE_COUNT_MAX`]),
/// and `deadline` limits the whole hop.
pub struct Ssh {
    /// `HOST` or `USER@HOST`, as ssh takes it.
    pub target: String,
    /// The port, when not the default.
    pub port: Option<String>,
    /// The daemon's socket on that machine, when not its default.
    pub socket: Option<String>,
    /// The ssh program; `ssh`.
    pub program: String,
    /// How long the program may run before it is killed; [`SSH_DEADLINE`].
    pub deadline: Duration,
}

/// What an ssh that ran to its end left behind.
struct Ran {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Ssh {
    fn hop(&self, what: &str) -> String {
        format!("ssh {}: {what}", self.target)
    }

    /// Gives `child` the request line and waits for it to exit, at most
    /// `deadline`; after that it is killed and reaped. Its pipes are written and
    /// drained on threads, so a full one cannot block the wait.
    fn wait(&self, mut child: Child, line: String) -> Result<Ran, String> {
        let started = Instant::now();
        if let Some(mut stdin) = child.stdin.take() {
            std::thread::spawn(move || {
                // A hop that fails early closes its end before it reads: the exit
                // status says why, so a broken pipe here is not the answer.
                let _ = writeln!(stdin, "{line}");
            });
        }
        let stdout = drain(child.stdout.take());
        let stderr = drain(child.stderr.take());
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|e| self.hop(&e.to_string()))? {
                break status;
            }
            let left = self.deadline.saturating_sub(started.elapsed());
            if left.is_zero() {
                // It may have exited since the check: the kill then does nothing.
                let _ = child.kill();
                let _ = child.wait();
                let limit = self.deadline.as_secs_f64();
                return Err(self.hop(&format!("no answer within {limit} s")));
            }
            std::thread::sleep(left.min(Duration::from_millis(5)));
        };
        let until = Instant::now() + SSH_OUTPUT_GRACE;
        Ok(Ran {
            status,
            stdout: stdout.take(until),
            stderr: stderr.take(until),
        })
    }
}

/// One of a child's pipes, being read on a thread of its own.
struct Drain {
    bytes: Arc<Mutex<Vec<u8>>>,
    /// Closes when the pipe does.
    open: Receiver<()>,
}

impl Drain {
    /// What was read, once the pipe has closed or at `until`.
    fn take(self, until: Instant) -> Vec<u8> {
        let _ = self
            .open
            .recv_timeout(until.saturating_duration_since(Instant::now()));
        std::mem::take(&mut *self.bytes.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

fn drain(pipe: Option<impl Read + Send + 'static>) -> Drain {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let (closes, open) = mpsc::channel();
    let read = Arc::clone(&bytes);
    std::thread::spawn(move || {
        let _closes: Sender<()> = closes;
        let Some(mut pipe) = pipe else { return };
        let mut chunk = [0; 8192];
        loop {
            match pipe.read(&mut chunk) {
                Ok(0) => return,
                Ok(n) => read
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => return,
            }
        }
    });
    Drain { bytes, open }
}

impl Transport for Ssh {
    fn describe(&self) -> (String, Option<String>) {
        match &self.socket {
            Some(s) => (format!("{}{s}", self.target), self.port.clone()),
            None => (self.target.clone(), self.port.clone()),
        }
    }

    fn send(&self, request: &Request) -> Result<Reply, String> {
        let line = serde_json::to_string(request).map_err(|e| e.to_string())?;
        let mut cmd = Command::new(&self.program);
        cmd.args(["-o", "BatchMode=yes"]);
        for option in [
            format!("ConnectTimeout={SSH_CONNECT_TIMEOUT_S}"),
            format!("ServerAliveInterval={SSH_ALIVE_INTERVAL_S}"),
            format!("ServerAliveCountMax={SSH_ALIVE_COUNT_MAX}"),
        ] {
            cmd.args(["-o", &option]);
        }
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
        let child = cmd
            .spawn()
            .map_err(|e| self.hop(&format!("cannot run {}: {e}", self.program)))?;
        let out = self.wait(child, line)?;
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
        deadline: SSH_DEADLINE,
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
                temet_work_core::Command::Overturn {
                    decision, option, ..
                } => {
                    return Ok(Reply::Done {
                        text: format!("decision {} is overturned to option {option}", decision.0),
                    });
                }
                temet_work_core::Command::Proposal { rule, keep } => {
                    let state = if *keep { "kept" } else { "dropped" };
                    return Ok(Reply::Done {
                        text: format!("rule {} is {state}", rule.0),
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

    /// An `Ssh` to a host that does not exist, whose program is a shell script
    /// with this body. The second value is the script's path, removed with its
    /// folder by the caller.
    fn fake_ssh(name: &str, body: &str) -> (Ssh, PathBuf) {
        use std::os::unix::fs::PermissionsExt;
        // Under this checkout's build folder, never the shared temp folder.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("target/tmp/ssh-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch");
        let script = dir.join("ssh");
        std::fs::write(&script, format!("#!/bin/sh\n{body}\n")).expect("script");
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        let ssh = Ssh {
            target: "unseen-university.invalid".into(),
            port: None,
            socket: None,
            program: script.display().to_string(),
            deadline: SSH_DEADLINE,
        };
        (ssh, script)
    }

    fn remove(script: &Path) {
        let _ = std::fs::remove_dir_all(script.parent().expect("dir"));
    }

    fn status(by: Option<String>) -> Request {
        Request {
            by: by.map(temet_work_core::ids::AgentId::new),
            command: temet_work_core::Command::Status,
        }
    }

    const REPLY: &str = r#"printf '{"reply":"done","text":"all quiet"}\n'"#;

    fn all_quiet() -> Result<Reply, String> {
        Ok(Reply::Done {
            text: "all quiet".into(),
        })
    }

    #[test]
    fn every_hop_limits_the_connect_and_the_silence_before_the_target() {
        let (ssh, script) = fake_ssh(
            "options",
            &format!("printf '%s\\n' \"$@\" >\"$0.args\"\n{REPLY}"),
        );
        let reply = ssh.send(&status(None));
        let args = std::fs::read_to_string(script.with_extension("args")).expect("args");
        remove(&script);
        assert_eq!(reply, all_quiet());
        assert_eq!(
            args.lines().collect::<Vec<_>>(),
            [
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                "-o",
                "ServerAliveInterval=15",
                "-o",
                "ServerAliveCountMax=2",
                "--",
                "unseen-university.invalid",
                "temet-work",
                "relay",
            ]
        );
    }

    #[test]
    fn a_hop_that_never_answers_is_killed_at_the_deadline() {
        let (mut ssh, script) = fake_ssh("silent", "echo $$ >\"$0.pid\"\nexec sleep 30");
        ssh.deadline = Duration::from_millis(200);
        let started = Instant::now();
        // More than a pipe holds, and never read: writing it must not block either.
        let reply = ssh.send(&status(Some("x".repeat(1 << 20))));
        let took = started.elapsed();
        let pid = std::fs::read_to_string(script.with_extension("pid")).expect("pid");
        remove(&script);
        assert_eq!(
            reply,
            Err("ssh unseen-university.invalid: no answer within 0.2 s".to_string())
        );
        assert!(took < Duration::from_millis(900), "{took:?}");
        // `kill -0` still succeeds for a child that was killed and never reaped.
        let alive = Command::new("kill")
            .args(["-0", pid.trim()])
            .stderr(Stdio::null())
            .status()
            .expect("kill");
        assert!(!alive.success(), "process {pid} is still there");
    }

    #[test]
    fn a_chatty_hop_that_answers_in_time_returns_the_reply() {
        let (ssh, script) = fake_ssh(
            "chatty",
            &format!("head -c 1000000 /dev/zero >&2\ncat >/dev/null\n{REPLY}"),
        );
        let reply = ssh.send(&status(None));
        remove(&script);
        assert_eq!(reply, all_quiet());
    }

    #[test]
    fn a_reply_counts_when_a_process_left_behind_holds_the_pipe_open() {
        let (mut ssh, script) = fake_ssh(
            "lingering",
            &format!("sleep 5 >/dev/null &\necho $! >\"$0.pid\"\n{REPLY}"),
        );
        ssh.deadline = Duration::from_secs(3);
        let reply = ssh.send(&status(None));
        let pid = std::fs::read_to_string(script.with_extension("pid")).expect("pid");
        let _ = Command::new("kill").arg(pid.trim()).status();
        remove(&script);
        assert_eq!(reply, all_quiet());
    }

    #[test]
    fn an_unreachable_host_says_it_could_not_connect() {
        let (ssh, script) = fake_ssh(
            "unreachable",
            "echo 'ssh: Could not resolve hostname unseen-university.invalid' >&2\nexit 255",
        );
        let reply = ssh.send(&status(None));
        remove(&script);
        assert_eq!(
            reply,
            Err("ssh unseen-university.invalid: could not connect: ssh: Could not resolve hostname unseen-university.invalid".to_string())
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
