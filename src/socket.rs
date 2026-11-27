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
use temet_work_core::{Host, Peer, Request};

pub use temet_work_core::Reply;

/// A line that asks the socket something about the daemon itself, such as
/// `{"ask":"build"}`. The socket code answers it; it never reaches the loop.
#[derive(Serialize, Deserialize)]
#[serde(tag = "ask", rename_all = "snake_case")]
enum Question {
    /// The [`crate::BUILD`] of the process that holds the socket.
    Build,
}

/// Where the daemon listens: the core's rule
/// ([`temet_work_core::socket::path`]) over this process's environment.
pub fn path() -> PathBuf {
    let var = std::env::var_os;
    temet_work_core::socket::path(
        var("TEMET_WORK_SOCKET"),
        var("XDG_RUNTIME_DIR"),
        var("HOME"),
    )
}

/// A request waiting for the daemon's reply.
pub struct Pending {
    /// The request.
    pub request: Request,
    /// What the daemon learned about the process that sent it.
    pub peer: Peer,
    reply: Sender<Reply>,
}

impl Pending {
    /// Sends the reply back to whoever asked.
    pub fn answer(self, reply: Reply) {
        let _ = self.reply.send(reply);
    }
}

/// Listens at `path` and hands each request to the returned channel, from a
/// thread per connection. That thread answers `{"ask":"build"}` itself, with
/// this program's [`crate::BUILD`], without waiting for the daemon's loop.
/// Refuses when a daemon already answers there.
pub fn listen(path: &Path) -> Result<Receiver<Pending>, String> {
    listen_as(path, crate::BUILD)
}

/// [`listen`], answering the build question with `build`: for a test that needs
/// a daemon of another build.
pub fn listen_as(path: &Path, build: &str) -> Result<Receiver<Pending>, String> {
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
    let build: Arc<str> = build.into();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let (tx, build) = (tx.clone(), Arc::clone(&build));
            std::thread::spawn(move || serve(stream, &build, &tx));
        }
    });
    Ok(rx)
}

fn serve(stream: UnixStream, build: &str, tx: &Sender<Pending>) {
    // Who sent it, read once: the process at the other end of a connection
    // stays the same.
    let peer = crate::peer::of(&stream);
    let Ok(read) = stream.try_clone() else { return };
    let mut writer = stream;
    for line in BufReader::new(read).lines() {
        let Ok(line) = line else { return };
        let reply = match serde_json::from_str::<Request>(&line) {
            Err(e) => match serde_json::from_str::<Question>(&line) {
                Ok(Question::Build) => Reply::Done { text: build.into() },
                Err(_) => Reply::Refused {
                    why: format!("not a request: {e}"),
                },
            },
            Ok(request) => {
                let (reply, answered) = mpsc::channel();
                if tx
                    .send(Pending {
                        request,
                        peer: peer.clone(),
                        reply,
                    })
                    .is_err()
                {
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
/// -o ServerAliveCountMax=2 HOST temet-work relay --build BUILD --host NAME`
/// there, with this client's [`crate::BUILD`] and machine name. No listener and no
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
        cmd.args(["--build", &shell_word(crate::BUILD)]);
        if let Some(name) = machine_name() {
            cmd.args(["--host", &name]);
        }
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

/// `text` as one word the far machine's shell reads unchanged: ssh joins the
/// command into one line, so any other character becomes `_`.
fn shell_word(text: &str) -> String {
    let plain = |c: char| c.is_ascii_alphanumeric() || "._-+/".contains(c);
    text.chars()
        .map(|c| if plain(c) { c } else { '_' })
        .collect()
}

/// This machine's name, as `uname -n` says it on Linux and macOS.
fn machine_name() -> Option<String> {
    let out = Command::new("uname").arg("-n").output().ok()?;
    let name = String::from_utf8(out.stdout).ok()?;
    Some(shell_word(name.trim())).filter(|n| !n.is_empty())
}

/// What the client on the other end of a hop says about itself on the relay
/// call. A client from before builds were sent says nothing.
#[derive(Clone, Copy, Debug, Default)]
pub struct Caller<'a> {
    /// Its [`crate::BUILD`].
    pub build: Option<&'a str>,
    /// Its machine's name.
    pub host: Option<&'a str>,
}

/// `temet-work relay`: copies one request line from `input` to the daemon's
/// socket at `path` and its reply line to `out`, so the other end of an SSH hop
/// speaks the daemon's own wire format. The relay is only a pipe for that line,
/// so its own build does not matter: it asks the daemon which build it is
/// (`{"ask":"build"}`) and sends the request on the same connection, so one
/// process answers both. A `caller` at another build gets a refusal naming both
/// builds, and a daemon that does not say its build gets one saying to restart
/// it; neither is sent the request. The request is marked as relayed, so the
/// daemon knows it came from another machine.
///
/// The build is asked over the socket because the answer comes from the process
/// that holds it. A `<socket>.build` file can be left by an earlier daemon, or
/// stay behind after a rollback to a daemon that writes none. A path to the
/// daemon's own `temet-work` in an agent's [`DAEMON_VAR`] runs whatever is
/// installed there now, which after a reinstall without a restart is not what
/// the daemon runs.
pub fn relay(
    path: &Path,
    caller: Caller,
    input: impl BufRead,
    mut out: impl Write,
) -> Result<(), String> {
    let mut line = String::new();
    let mut input = input;
    input.read_line(&mut line).map_err(|e| e.to_string())?;
    let stream = connect(path)?;
    let mut replies = BufReader::new(&stream);
    let question = serde_json::to_string(&Question::Build).map_err(|e| e.to_string())?;
    let answer = exchange(&stream, &mut replies, &question)
        .ok()
        .filter(|a| !a.is_empty())
        .ok_or_else(|| {
            format!(
                "no temet-work daemon answers at {} (it closed the connection before it said its build); it may be restarting: run the command again in a minute",
                path.display()
            )
        })?;
    let refusal = match serde_json::from_str::<Reply>(&answer) {
        Ok(Reply::Done { text: build }) if caller.build == Some(shell_word(&build).as_str()) => {
            None
        }
        Ok(Reply::Done { text: build }) => Some(format!(
            "temet-work on {} is build {}, the daemon is build {build}: update it",
            caller.host.unwrap_or("the other machine"),
            caller.build.unwrap_or("unknown"),
        )),
        _ => Some(format!(
            "the daemon at {} does not say its build: it was started from an older temet-work; restart it",
            path.display()
        )),
    };
    let reply = match refusal {
        Some(why) => {
            let refusal = Reply::Refused { why };
            serde_json::to_string(&refusal).map_err(|e| e.to_string())? + "\n"
        }
        None => exchange(&stream, &mut replies, &relayed(line.trim_end()))?,
    };
    out.write_all(reply.as_bytes())
        .and_then(|()| out.flush())
        .map_err(|e| e.to_string())
}

/// Writes one line to the daemon and reads the line it replies with, which is
/// empty when it closed the connection instead.
fn exchange(
    mut stream: &UnixStream,
    replies: &mut impl BufRead,
    line: &str,
) -> Result<String, String> {
    stream
        .write_all(format!("{line}\n").as_bytes())
        .map_err(|e| e.to_string())?;
    let mut reply = String::new();
    replies.read_line(&mut reply).map_err(|e| e.to_string())?;
    Ok(reply)
}

/// A request line marked as relayed. A line that is not a JSON object goes on
/// as it is, and the daemon refuses it.
fn relayed(line: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(line) {
        Ok(serde_json::Value::Object(mut fields)) => {
            fields.insert("relayed".into(), true.into());
            serde_json::Value::Object(fields).to_string()
        }
        _ => line.to_string(),
    }
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

/// Answers one request against the host, from a sender the daemon learned
/// `peer` about. The sender is checked first ([`temet_work_core::World::admit`]).
/// The error stops the daemon: an event could not be logged.
pub fn answer(
    host: &mut Host,
    now_ms: u64,
    request: &Request,
    peer: &Peer,
) -> Result<Reply, String> {
    let sender = temet_work_core::Sender {
        by: request.by.clone(),
        proof: request
            .secret
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(crate::peer::digest),
        peer: peer.clone(),
        relayed: request.relayed,
    };
    if let Err(why) = host.world().admit(&sender, &request.command) {
        return Ok(Reply::Refused { why });
    }
    if let temet_work_core::Command::File { .. } = &request.command {
        return file(host, now_ms, request);
    }
    match host.world().events(request.by.as_ref(), &request.command) {
        Err(why) => Ok(Reply::Refused { why }),
        Ok(events) if events.is_empty() => Ok(match &request.command {
            temet_work_core::Command::Settings { target, role } => {
                match temet_work_core::text::settings(host.world(), target, *role) {
                    Ok(text) => Reply::Done { text },
                    Err(why) => Reply::Refused { why },
                }
            }
            temet_work_core::Command::View => Reply::View {
                view: temet_work_core::View::new(host.world(), host.problems()),
            },
            temet_work_core::Command::Issue { issue } => Reply::Done {
                text: temet_work_core::text::issue(host.world(), issue, &host.issue_text(issue))
                    .unwrap_or_default(),
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
            let changed = changed_setting(&request.command, events.first());
            for event in events {
                host.handle(now_ms, event)?;
            }
            if let Some(text) = changed {
                return Ok(Reply::Done { text });
            }
            match &request.command {
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
                temet_work_core::Command::Investigated { group } => {
                    let filed = host
                        .world()
                        .investigations
                        .get(group)
                        .map_or(0, |i| i.filed.len());
                    return Ok(Reply::Done {
                        text: format!(
                            "the investigation for {group} has ended with {filed} issue{} filed; \
                             your session is stopped",
                            if filed == 1 { "" } else { "s" }
                        ),
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

/// Files an issue for an investigator through the tracker, and answers with the
/// new issue's id and how a later issue names it in a "Depends on" line.
fn file(host: &mut Host, now_ms: u64, request: &Request) -> Result<Reply, String> {
    let effect = match host.world().filing(request.by.as_ref(), &request.command) {
        Ok(effect) => effect,
        Err(why) => return Ok(Reply::Refused { why }),
    };
    let events = match host.perform(now_ms, &effect)? {
        Ok(events) => events,
        Err(why) => return Ok(Reply::Refused { why }),
    };
    let Some((issue, milestone)) = events.iter().find_map(|e| match e {
        temet_work_core::Event::IssueFiled {
            issue, milestone, ..
        } => Some((issue, milestone)),
        _ => None,
    }) else {
        return Ok(Reply::Refused {
            why: "the tracker filed nothing".into(),
        });
    };
    // A number is written `#12` in a "Depends on" line; a key such as GLD-12 as it is.
    let named = if issue.key.chars().all(|c| c.is_ascii_digit()) {
        format!("#{}", issue.key)
    } else {
        issue.key.clone()
    };
    let milestone = milestone.as_ref().map_or_else(
        || " with no milestone".to_string(),
        |m| format!(" in milestone {m}"),
    );
    Ok(Reply::Done {
        text: format!(
            "filed {issue}{milestone}; an issue of {} that waits for it says: Depends on {named}",
            issue.project
        ),
    })
}

/// What a `set` or `unset` is answered with, read from the event it was recorded
/// as: an agent's value may be recorded for a role the command did not name.
fn changed_setting(
    command: &temet_work_core::Command,
    recorded: Option<&temet_work_core::Event>,
) -> Option<String> {
    use temet_work_core::{Command, Event};
    let for_role = |role: &Option<temet_work_core::model::Role>| {
        role.map_or_else(String::new, |r| format!(" for {}", r.name()))
    };
    match (command, recorded?) {
        (
            Command::Set { target, .. },
            Event::SettingSet {
                role, key, value, ..
            },
        ) => Some(format!("{key} is {value} at {target}{}", for_role(role))),
        (Command::Unset { target, .. }, Event::SettingUnset { role, key, .. }) => Some(format!(
            "{key} is no longer set at {target}{}",
            for_role(role)
        )),
        _ => None,
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
        Request::new(
            by.map(temet_work_core::ids::AgentId::new),
            temet_work_core::Command::Status,
        )
    }

    #[test]
    fn the_relay_marks_a_request_and_passes_anything_else_on() {
        let line = r#"{"by":"author-1","secret":"s","command":{"command":"status"}}"#;
        let marked: Request = serde_json::from_str(&relayed(line)).expect("a request");
        assert!(marked.relayed);
        assert_eq!(marked.secret.as_deref(), Some("s"));
        // A request that says it was not relayed is marked all the same.
        let lying = r#"{"by":null,"relayed":false,"command":{"command":"status"}}"#;
        assert!(
            serde_json::from_str::<Request>(&relayed(lying))
                .expect("request")
                .relayed
        );
        assert_eq!(relayed("not json"), "not json");
    }

    #[test]
    fn a_request_without_the_new_fields_still_parses_and_debug_hides_the_secret() {
        let old: Request =
            serde_json::from_str(r#"{"by":"author-1","command":{"command":"status"}}"#)
                .expect("an old request");
        assert_eq!(old.secret, None);
        assert!(!old.relayed);
        let mut new = old.clone();
        new.secret = Some("0123456789abcdef".into());
        let shown = format!("{new:?}");
        assert!(
            !shown.contains("0123456789abcdef") && shown.contains("<hidden>"),
            "{shown}"
        );
        // A request without a secret is sent as before.
        assert_eq!(
            serde_json::to_string(&old).expect("json"),
            r#"{"by":"author-1","command":{"command":"status"}}"#
        );
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
        let build = shell_word(crate::BUILD);
        let name = machine_name().expect("this machine's name");
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
                "--build",
                &build,
                "--host",
                &name,
            ]
        );
    }

    /// A socket path under this checkout's build folder, with nothing at it.
    fn socket_at(name: &str) -> PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/tmp");
        std::fs::create_dir_all(&dir).expect("scratch");
        let path = dir.join(format!("{}-{name}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        path
    }

    const STATUS: &str = r#"{"by":null,"command":{"command":"status"}}"#;

    /// What the relay writes back for a status request from `caller`.
    fn relayed_status(path: &Path, caller: Caller) -> Reply {
        let mut out = Vec::new();
        let input = format!("{STATUS}\n");
        assert_eq!(relay(path, caller, input.as_bytes(), &mut out), Ok(()));
        serde_json::from_slice(&out).expect("a reply")
    }

    fn at_octarine() -> Caller<'static> {
        Caller {
            build: Some("octarine"),
            host: Some("lancre"),
        }
    }

    #[test]
    fn the_socket_says_its_build_and_still_answers_the_next_request() {
        let path = socket_at("ask-build");
        let requests = listen(&path).expect("listen");
        let stream = UnixStream::connect(&path).expect("connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("timeout");
        let mut replies = BufReader::new(&stream);
        let mut ask = |line: &str| {
            let reply = exchange(&stream, &mut replies, line)?;
            serde_json::from_str::<Reply>(&reply).map_err(|e| e.to_string())
        };
        // Answered with nothing reading the channel: the loop plays no part.
        assert_eq!(
            ask(r#"{"ask":"build"}"#),
            Ok(Reply::Done {
                text: crate::BUILD.into()
            })
        );
        let daemon = std::thread::spawn(move || {
            let pending = requests
                .recv_timeout(Duration::from_secs(1))
                .expect("a request");
            assert_eq!(pending.request, status(None));
            pending.answer(Reply::Done {
                text: "all quiet".into(),
            });
        });
        let reply = ask(STATUS);
        daemon.join().expect("daemon");
        let _ = std::fs::remove_file(&path);
        assert_eq!(reply, all_quiet());
    }

    /// A daemon that takes one connection, closes its listener, and answers each
    /// line on that connection with what `reply` makes of it. Joining it gives
    /// the lines it got.
    fn one_connection(
        path: &Path,
        reply: impl Fn(&str) -> &'static str + Send + 'static,
    ) -> std::thread::JoinHandle<Vec<String>> {
        let listener = UnixListener::bind(path).expect("bind");
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            drop(listener);
            let lines = BufReader::new(stream.try_clone().expect("clone")).lines();
            lines
                .map(|line| {
                    let line = line.expect("a line");
                    writeln!(stream, "{}", reply(&line)).expect("reply");
                    line
                })
                .collect()
        })
    }

    #[test]
    fn the_relay_asks_and_sends_the_request_on_one_connection() {
        let path = socket_at("one-connection");
        let daemon = one_connection(&path, |line| {
            if line == r#"{"ask":"build"}"# {
                r#"{"reply":"done","text":"octarine"}"#
            } else {
                r#"{"reply":"done","text":"all quiet"}"#
            }
        });
        let reply = relayed_status(&path, at_octarine());
        let lines = daemon.join().expect("daemon");
        let again = UnixStream::connect(&path);
        let _ = std::fs::remove_file(&path);
        assert_eq!(Ok(reply), all_quiet());
        assert_eq!(lines, [r#"{"ask":"build"}"#.to_string(), relayed(STATUS)]);
        assert!(again.is_err(), "a second connection was taken");
    }

    #[test]
    fn a_daemon_that_does_not_say_its_build_is_sent_no_request() {
        let path = socket_at("older-daemon");
        // What a daemon from before the question says to a line it cannot read.
        let daemon = one_connection(
            &path,
            |_| r#"{"reply":"refused","why":"not a request: missing field `command`"}"#,
        );
        let reply = relayed_status(&path, at_octarine());
        let lines = daemon.join().expect("daemon");
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            reply,
            Reply::Refused {
                why: format!(
                    "the daemon at {} does not say its build: it was started from an older temet-work; restart it",
                    path.display()
                )
            }
        );
        assert_eq!(lines, [r#"{"ask":"build"}"#]);
    }

    #[test]
    fn relay_asks_the_daemon_its_build_to_refuse_a_caller_that_sends_none() {
        let path = socket_at("no-build");
        let requests = listen_as(&path, "octarine").expect("listen");
        let reply = relayed_status(&path, Caller::default());
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            reply,
            Reply::Refused {
                why: "temet-work on the other machine is build unknown, the daemon is build octarine: update it".into()
            }
        );
        assert!(requests.try_recv().is_err(), "the request was forwarded");
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
