//! herdr-core terminals, reached over its JSON-RPC socket: one JSON message per
//! line, `hello` first.
//!
//! What the client relies on comes from `docs/findings/herdr-core.md`: labels
//! set when a terminal starts, so its first event already carries them; one
//! subscription filtered by temet-work's own label, on its own connection; `idle`
//! and `done` both meaning a turn ended; a send refused while the terminal is at
//! a prompt; the visible screen as the cheap read. Terminals are started with no
//! parent, so agents outlive a restart of the daemon. An agent's terminal also
//! gets the labels that put it on the herdr app's screens (`app_labels`), and
//! every terminal a `program` label, so a program herdr-core has replaced with
//! a shell can be told from it (`lost`).
//!
//! While `TEMET_WORK_SANDBOX` is set to anything but empty, as it is in every
//! test run, the client opens nothing and every call fails, unless it was given
//! a socket of its own ([`Herdr::socket`]).

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use temet_work_runner::{Program, Runner, RunnerError, Start, Status, TerminalId, Update};

/// The label every terminal this client starts carries, and the subscription's filter.
const OWNER: (&str, &str) = ("owner", "temet-work");

/// The label holding the first word of the argv a terminal was started with.
const PROGRAM: &str = "program";

/// The socket herdr-core listens on: `HC_TERMINAL_SOCKET`, else the default
/// under `XDG_RUNTIME_DIR`.
pub fn socket_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("HC_TERMINAL_SOCKET").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p));
    }
    std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|p| !p.is_empty())
        .map(|d| PathBuf::from(d).join("herdr-core").join("default.sock"))
}

fn sandboxed() -> bool {
    std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty())
}

/// An error the core reported, with its stable kind.
#[derive(Debug)]
struct RpcError {
    kind: String,
    message: String,
}

/// One connection to the core.
struct Conn {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
    next: u64,
}

impl Conn {
    /// Connects and says hello. A read waits at most `timeout`, so a core that
    /// stops answering cannot hang the daemon.
    fn open(path: &PathBuf, timeout: Duration) -> Result<Self, RunnerError> {
        let stream = UnixStream::connect(path).map_err(|e| {
            RunnerError::Other(format!(
                "cannot reach herdr-core at {}: {e}",
                path.display()
            ))
        })?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| RunnerError::Other(e.to_string()))?;
        let reader = BufReader::new(
            stream
                .try_clone()
                .map_err(|e| RunnerError::Other(e.to_string()))?,
        );
        let mut conn = Self {
            writer: stream,
            reader,
            next: 0,
        };
        conn.call("hello", json!({})).map_err(rpc)?;
        Ok(conn)
    }

    /// Sends one request and returns its result, skipping notifications.
    fn call(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        self.next += 1;
        let id = self.next;
        let line = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        let io = |e: std::io::Error| RpcError {
            kind: "io".into(),
            message: e.to_string(),
        };
        writeln!(self.writer, "{line}").map_err(io)?;
        let mut buf = String::new();
        loop {
            buf.clear();
            if self.reader.read_line(&mut buf).map_err(io)? == 0 {
                return Err(RpcError {
                    kind: "io".into(),
                    message: "herdr-core closed the connection".into(),
                });
            }
            let Ok(msg) = serde_json::from_str::<Value>(&buf) else {
                continue;
            };
            if msg.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(err) = msg.get("error") {
                return Err(RpcError {
                    kind: err
                        .pointer("/data/kind")
                        .and_then(Value::as_str)
                        .unwrap_or("error")
                        .to_string(),
                    message: err
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                });
            }
            return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
        }
    }
}

fn rpc(e: RpcError) -> RunnerError {
    RunnerError::Other(format!("herdr-core: {} ({})", e.message, e.kind))
}

/// `text` as part of a space or tab id: `-` for every character other than A-Z,
/// a-z, 0-9 and `-`, as mike makes the space ids of its fleet seats.
fn id_part(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// The labels that put an agent's terminal on the herdr app's screens: a space
/// per group, a tab in it per issue, and the agent's name. The app shows a
/// terminal that has `herdr.space` and `herdr.tab`, and makes the space or tab
/// when it does not know the id. The issue label names its project
/// (`ankh#WATCH-12`), so two projects' issues with one key get two tabs. An agent
/// on no issue, an investigator, gets a tab in its group's space named for its
/// role, and its group in its name where an issue would be. A start without a
/// `group` and an `agent` label, as the runner contract's are, gets none.
fn app_labels(labels: &BTreeMap<String, String>) -> Option<[(&'static str, String); 3]> {
    let (group, agent) = (labels.get("group")?, labels.get("agent")?);
    let (tab, about) = match labels.get("issue") {
        Some(issue) => (issue, issue),
        None => (labels.get("role")?, group),
    };
    let space = id_part(group);
    Some([
        ("herdr.space", format!("sptemet-work-{space}")),
        (
            "herdr.tab",
            format!("tbtemet-work-{space}-{}", id_part(tab)),
        ),
        ("herdr.name", format!("{agent} {about}")),
    ])
}

/// The `terminal.run` parameters for a start: `on` names the machine, and is
/// left out for a terminal on this one. The `program` label keeps the first
/// word of the argv, which herdr-core no longer reports once a shell has taken
/// the program's place.
fn run_params(start: &Start) -> Value {
    let mut labels = start.labels.clone();
    labels.insert(OWNER.0.into(), OWNER.1.into());
    if let Some(program) = start.program.argv.first() {
        labels.insert(PROGRAM.into(), program.clone());
    }
    for (key, value) in app_labels(&start.labels).into_iter().flatten() {
        labels.insert(key.into(), value);
    }
    let mut params = json!({
        "argv": start.program.argv,
        "cwd": start.cwd,
        "labels": labels,
        "no_notify": true,
        "no_parent": true,
    });
    if !start.program.env.is_empty() {
        params["env"] = Value::Object(
            start
                .program
                .env
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect(),
        );
    }
    if let Some(machine) = &start.machine {
        params["on"] = Value::String(machine.clone());
    }
    params
}

/// The `machine.exec` parameters for running `program` to its end: its
/// arguments as given, no shell, a timeout, and `on` for another machine.
fn exec_params(machine: Option<&str>, program: &Program) -> Value {
    let mut params = json!({
        "argv": program.argv,
        "timeout": EXEC_TIMEOUT.as_secs(),
    });
    if !program.env.is_empty() {
        params["env"] = Value::Object(
            program
                .env
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect(),
        );
    }
    if let Some(machine) = machine {
        params["on"] = Value::String(machine.to_string());
    }
    params
}

/// Where a program ran, in words.
fn place(machine: Option<&str>) -> String {
    machine.map_or_else(|| "this machine".to_string(), String::from)
}

/// Why a program could not run, when the shell's or the core's answer means
/// it is not on the `PATH` there. herdr-core runs it with that machine's core's
/// own environment, which can differ from an SSH session's.
fn not_on_path(machine: Option<&str>, program: &str) -> String {
    let (whose, on) = match machine {
        Some(m) => (format!("{m}'s"), format!(" --on {m}")),
        None => ("this machine's".to_string(), String::new()),
    };
    format!(
        "{program} is not on {whose} PATH for programs herdr-core runs there; \
         `hc exec{on} which {program}` checks it"
    )
}

/// What `machine.exec` answered for `argv`, as the runner reports it: an exit of
/// 0 is done; anything else is an error ending with the first line the program
/// wrote to standard error, where programs put the error ahead of any help. A
/// program without the command it was given is older than this daemon, whose
/// build is `build`: the error says to update it.
fn exec_outcome(
    build: Option<&str>,
    machine: Option<&str>,
    argv: &[String],
    result: Result<Value, RpcError>,
) -> Result<(), RunnerError> {
    let program = argv.first().map_or("the program", String::as_str);
    let result = match result {
        Ok(r) => r,
        Err(e) if e.kind == "spawn_failed" && e.message.contains("No such file") => {
            return Err(RunnerError::Other(not_on_path(machine, program)));
        }
        Err(e) => return Err(rpc(e)),
    };
    let stderr = String::from_utf8_lossy(&unbase64(
        result
            .get("stderr")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    ))
    .into_owned();
    let first = stderr.lines().map(str::trim).find(|l| !l.is_empty());
    // What every `temet-work` without the command prints, before its usage text.
    let no_command = argv.get(1).is_some_and(|command| {
        first.and_then(|l| l.strip_prefix("no command ")) == Some(command.as_str())
    });
    let said = first.map_or_else(String::new, |l| format!(": {l}"));
    if result.get("timed_out").and_then(Value::as_bool) == Some(true) {
        return Err(RunnerError::Other(format!(
            "{program} did not finish on {} within {} s{said}",
            place(machine),
            EXEC_TIMEOUT.as_secs()
        )));
    }
    match result.get("code").and_then(Value::as_i64) {
        Some(0) => Ok(()),
        // The shell's answer for a command it cannot find.
        Some(127) => Err(RunnerError::Other(format!(
            "{} (exit 127){said}",
            not_on_path(machine, program)
        ))),
        Some(2) if no_command => Err(RunnerError::Other(format!(
            "{program} on {} is older than this daemon{}: update it",
            place(machine),
            build.map_or_else(String::new, |b| format!(" (build {b})"))
        ))),
        Some(code) => Err(RunnerError::Other(format!(
            "{program} exited {code} on {}{said}",
            place(machine)
        ))),
        None => Err(RunnerError::Other(format!(
            "{program} was ended by signal {} on {}{said}",
            result
                .get("signal")
                .and_then(Value::as_i64)
                .map_or_else(|| "?".to_string(), |s| s.to_string()),
            place(machine)
        ))),
    }
}

/// Decodes standard base64, as herdr-core sends a program's output. Padding,
/// line breaks and anything else outside the alphabet are skipped.
fn unbase64(text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(u8::try_from((acc >> bits) & 0xff).unwrap_or_default());
            acc &= (1 << bits) - 1;
        }
    }
    out
}

/// A terminal's status from herdr-core's `state`, `kind` and `msg`. `error` is a
/// turn that ended in an error, such as a usage limit; its `msg`, when the core
/// sends one, says which.
fn status_of(state: Option<&str>, kind: Option<&str>, msg: Option<&str>) -> Status {
    match state {
        Some("idle" | "done") => Status::Between,
        Some("working") => Status::Working,
        Some("blocked") => Status::Blocked {
            kind: kind.unwrap_or("prompt").to_string(),
        },
        Some("error") => Status::Error {
            message: msg.filter(|m| !m.trim().is_empty()).map(String::from),
        },
        _ => Status::Unknown,
    }
}

fn labels_of(v: Option<&Value>) -> BTreeMap<String, String> {
    v.and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

/// Whether a terminal that has not finished lost its program, as to a signal
/// or a reboot. herdr-core keeps such a terminal, with its labels, and puts a
/// shell in the program's place. Until it has, the terminal carries the
/// `reason` the program ended for. After, no reason remains and the terminal's
/// `argv` is the shell's, which no longer starts with the `program` label. A
/// terminal started before that label is lost when it is an agent's and its
/// argv is one word: an agent is never started as one word, and a shell is. A
/// terminal listed without its argv is not read as lost.
fn lost(t: &Value, labels: &BTreeMap<String, String>) -> bool {
    if t.get("reason")
        .and_then(Value::as_str)
        .is_some_and(|r| !r.is_empty())
    {
        return true;
    }
    let Some(argv) = t
        .get("argv")
        .and_then(Value::as_array)
        .filter(|a| !a.is_empty())
    else {
        return false;
    };
    match labels.get(PROGRAM) {
        Some(program) => argv[0].as_str() != Some(program.as_str()),
        None => labels.contains_key("agent") && argv.len() == 1,
    }
}

/// What the event reader shares with the client.
#[derive(Default)]
struct Shared {
    updates: Vec<Update>,
    labels: BTreeMap<String, BTreeMap<String, String>>,
    alive: bool,
    /// The last full list of terminals, with their labels, until it is asked for.
    listed: Option<Vec<Update>>,
}

/// Turns a snapshot of terminals (from subscribing, or after a missed stretch)
/// into one update each, so terminals started before the daemon are known again.
fn take_snapshot(terminals: &Value, s: &mut Shared) {
    // A terminal this client knew that the new list lacks is gone: the core
    // restarted, or it was removed while the client was not listening.
    let listed: std::collections::BTreeSet<&str> = terminals
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|t| t.get("address").and_then(Value::as_str))
        .collect();
    let vanished: Vec<String> = s
        .labels
        .keys()
        .filter(|a| !listed.contains(a.as_str()))
        .cloned()
        .collect();
    for addr in vanished {
        let labels = s.labels.remove(&addr).unwrap_or_default();
        s.updates.push(Update {
            terminal: TerminalId(addr),
            labels,
            status: Status::Gone,
            doing: None,
        });
    }
    let first = s.updates.len();
    for t in terminals.as_array().into_iter().flatten() {
        let Some(addr) = t.get("address").and_then(Value::as_str) else {
            continue;
        };
        let labels = labels_of(t.get("labels"));
        s.labels.insert(addr.to_string(), labels.clone());
        // A finished terminal that was not removed is still listed; its session
        // is over. So is the session of one whose program was lost, although
        // herdr-core lists it live.
        let status = if t.get("state").and_then(Value::as_str) == Some("finished") {
            Status::Gone
        } else if lost(t, &labels) {
            Status::Lost
        } else {
            status_of(
                t.pointer("/status/state").and_then(Value::as_str),
                t.pointer("/status/kind").and_then(Value::as_str),
                t.pointer("/status/msg").and_then(Value::as_str),
            )
        };
        s.updates.push(Update {
            terminal: TerminalId(addr.to_string()),
            labels,
            status,
            doing: t
                .pointer("/status/msg")
                .and_then(Value::as_str)
                .map(String::from),
        });
    }
    s.listed = Some(s.updates[first..].to_vec());
}

/// Turns one event into its terminal's update, when it changes one's status,
/// and keeps the labels the later events of that terminal are read with.
fn take_event(p: &Value, s: &mut Shared) {
    let Some(t) = p.get("terminal").and_then(Value::as_str) else {
        return;
    };
    let data = &p["data"];
    let kind = p.get("type").and_then(Value::as_str).unwrap_or_default();
    if kind == "created" || kind == "labels" {
        let labels = labels_of(data.get("labels"));
        if !labels.is_empty() {
            s.labels.insert(t.to_string(), labels);
        }
    }
    let status = match kind {
        "created" => Some(Status::Unknown),
        "status" => Some(status_of(
            data.pointer("/status/state").and_then(Value::as_str),
            data.pointer("/status/kind").and_then(Value::as_str),
            data.pointer("/status/msg").and_then(Value::as_str),
        )),
        "finished" | "removed" => Some(Status::Gone),
        "lost" => Some(Status::Lost),
        _ => None,
    };
    if let Some(status) = status {
        let labels = s.labels.get(t).cloned().unwrap_or_default();
        s.updates.push(Update {
            terminal: TerminalId(t.to_string()),
            labels,
            status,
            doing: data
                .pointer("/status/msg")
                .and_then(Value::as_str)
                .map(String::from),
        });
    }
}

fn follow(mut conn: Conn, shared: Arc<Mutex<Shared>>) {
    let mut buf = String::new();
    loop {
        buf.clear();
        match conn.reader.read_line(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let Ok(msg) = serde_json::from_str::<Value>(&buf) else {
            continue;
        };
        let Ok(mut s) = shared.lock() else { break };
        match msg.get("method").and_then(Value::as_str) {
            Some("event") => take_event(&msg["params"], &mut s),
            Some("missed") => {
                // The reader fell behind: the core sends a fresh snapshot of terminals.
                if let Some(terminals) = msg.pointer("/params/terminals") {
                    take_snapshot(terminals, &mut s);
                }
            }
            _ => {}
        }
    }
    if let Ok(mut s) = shared.lock() {
        s.alive = false;
    }
}

/// The herdr adapter.
#[derive(Default)]
pub struct Herdr {
    control: Option<Conn>,
    shared: Arc<Mutex<Shared>>,
    /// Why the event subscription could not be opened, at the last try.
    unreachable: Option<String>,
    /// The build of the daemon this runner is part of.
    build: Option<String>,
    /// The socket to reach instead of [`socket_path`]'s.
    socket: Option<PathBuf>,
}

/// How long a request waits for its answer.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// How long a program run to its end may take. Under [`REQUEST_TIMEOUT`], so the
/// core ends it and answers before the request stops waiting.
const EXEC_TIMEOUT: Duration = Duration::from_secs(8);

/// How long the event connection may stay silent; the core sends a heartbeat
/// every 15 s, so a longer silence means the connection is dead.
const EVENTS_TIMEOUT: Duration = Duration::from_secs(45);

fn io_error(e: &RunnerError) -> RpcError {
    RpcError {
        kind: "io".into(),
        message: e.to_string(),
    }
}

impl Herdr {
    /// Names `build` as the daemon's, in the error for a program on a machine
    /// that is older than it.
    #[must_use]
    pub fn build(mut self, build: &str) -> Self {
        self.build = Some(build.to_string());
        self
    }

    /// Reaches the core listening at `path` instead of the one the environment
    /// names, sandbox or not: how a test puts a stand-in core in its place.
    #[must_use]
    pub fn socket(mut self, path: impl Into<PathBuf>) -> Self {
        self.socket = Some(path.into());
        self
    }

    fn path(&self) -> Result<PathBuf, RunnerError> {
        if let Some(path) = &self.socket {
            return Ok(path.clone());
        }
        if sandboxed() {
            return Err(RunnerError::Other(
                "TEMET_WORK_SANDBOX is set: herdr-core is not reached".into(),
            ));
        }
        socket_path().ok_or_else(|| {
            RunnerError::Other(
                "no herdr-core socket: set HC_TERMINAL_SOCKET or XDG_RUNTIME_DIR".into(),
            )
        })
    }

    /// Opens the event subscription when it is not open. The snapshot it
    /// starts with becomes one update per terminal, so terminals started before
    /// the daemon are known again.
    fn follow(&mut self) -> Result<(), RunnerError> {
        if self.shared.lock().map(|s| s.alive).unwrap_or(false) {
            return Ok(());
        }
        let mut events = Conn::open(&self.path()?, EVENTS_TIMEOUT)?;
        let snapshot = events
            .call("events.subscribe", json!({"labels": {OWNER.0: OWNER.1}}))
            .map_err(rpc)?;
        if let Ok(mut s) = self.shared.lock() {
            s.alive = true;
            if let Some(terminals) = snapshot.get("terminals") {
                take_snapshot(terminals, &mut s);
            }
        }
        let shared = Arc::clone(&self.shared);
        std::thread::spawn(move || follow(events, shared));
        Ok(())
    }

    /// Sends one request on the control connection, opening it when needed. A
    /// broken connection is dropped, so the next request opens a new one.
    fn request(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        self.follow().map_err(|e| io_error(&e))?;
        if self.control.is_none() {
            let path = self.path().map_err(|e| io_error(&e))?;
            self.control = Some(Conn::open(&path, REQUEST_TIMEOUT).map_err(|e| io_error(&e))?);
        }
        let Some(conn) = self.control.as_mut() else {
            return Err(io_error(&RunnerError::Other("no connection".into())));
        };
        let result = conn.call(method, params);
        if matches!(&result, Err(e) if e.kind == "io") {
            self.control = None;
        }
        result
    }
}

fn mapped(terminal: &TerminalId, e: RpcError) -> RunnerError {
    match e.kind.as_str() {
        "blocked" => RunnerError::Blocked,
        "status_mismatch" => RunnerError::Busy,
        "finished" => RunnerError::Gone,
        "not_found" => RunnerError::Unknown(terminal.clone()),
        _ => rpc(e),
    }
}

impl Runner for Herdr {
    fn id(&self) -> &'static str {
        "herdr"
    }

    fn is_default(&self) -> bool {
        true
    }

    fn start(&mut self, start: &Start) -> Result<TerminalId, RunnerError> {
        let params = run_params(start);
        let result = self.request("terminal.run", params).map_err(rpc)?;
        result
            .get("address")
            .and_then(Value::as_str)
            .map(|a| TerminalId(a.to_string()))
            .ok_or_else(|| {
                RunnerError::Other("herdr-core gave no address for the new terminal".into())
            })
    }

    fn exec(&mut self, machine: Option<&str>, program: &Program) -> Result<(), RunnerError> {
        let result = self.request("machine.exec", exec_params(machine, program));
        exec_outcome(self.build.as_deref(), machine, &program.argv, result)
    }

    fn send(
        &mut self,
        terminal: &TerminalId,
        text: &str,
        only_if_idle: bool,
    ) -> Result<(), RunnerError> {
        if only_if_idle {
            // herdr-core's `if` matches one word, and a turn that ended reads `done`,
            // not `idle`: check the status first, then send without `if`.
            let facts = self
                .request("terminal.get", json!({"terminal": terminal.0}))
                .map_err(|e| mapped(terminal, e))?;
            if facts.pointer("/status/state").and_then(Value::as_str) == Some("working") {
                return Err(RunnerError::Busy);
            }
        }
        self.request(
            "terminal.send",
            json!({"terminal": terminal.0, "text": text}),
        )
        .map(|_| ())
        .map_err(|e| mapped(terminal, e))
    }

    fn screen(&mut self, terminal: &TerminalId) -> Result<String, RunnerError> {
        let result = self
            .request("terminal.read", json!({"terminal": terminal.0, "lines": 0}))
            .map_err(|e| mapped(terminal, e))?;
        Ok(result
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string())
    }

    fn stop(&mut self, terminal: &TerminalId) -> Result<(), RunnerError> {
        match self.request("terminal.kill", json!({"terminal": terminal.0})) {
            Ok(_) => {}
            Err(e) if e.kind == "finished" => {}
            Err(e) => return Err(mapped(terminal, e)),
        }
        // `rm` refuses a terminal that is still live; kill reports once it has ended,
        // but give the core a moment if it has not caught up.
        for _ in 0..20 {
            match self.request("terminal.rm", json!({"terminal": terminal.0})) {
                Ok(_) => return Ok(()),
                Err(e) if e.kind == "live" => std::thread::sleep(Duration::from_millis(100)),
                Err(e) if e.kind == "not_found" => return Ok(()),
                Err(e) => return Err(mapped(terminal, e)),
            }
        }
        Err(RunnerError::Other(format!(
            "{terminal} is still live after kill"
        )))
    }

    fn listed(&mut self) -> Option<Vec<Update>> {
        self.shared.lock().ok().and_then(|mut s| s.listed.take())
    }

    fn problem(&self) -> Option<String> {
        self.unreachable
            .as_ref()
            .map(|e| format!("agents' states are not coming in: {e}"))
    }

    fn updates(&mut self) -> Vec<Update> {
        // Follow events even when nothing is being started or sent, so states
        // keep coming after a restart. A core that is down is tried again next
        // time; starting or sending reports it.
        if self.socket.is_some() || !sandboxed() {
            self.unreachable = self.follow().err().map(|e| e.to_string());
        }
        self.shared
            .lock()
            .map(|mut s| std::mem::take(&mut s.updates))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_on(machine: Option<&str>) -> Start {
        Start {
            machine: machine.map(String::from),
            ..Start::default()
        }
    }

    #[test]
    fn a_start_on_another_machine_sends_on_and_a_local_one_does_not() {
        assert_eq!(run_params(&start_on(Some("mac")))["on"], "mac");
        assert!(run_params(&start_on(None)).get("on").is_none());
    }

    /// A start as the runner family labels one, on this machine.
    fn start_of(group: &str, issue: &str, agent: &str, role: &str) -> Start {
        Start {
            labels: [
                ("agent", agent),
                ("group", group),
                ("issue", issue),
                ("role", role),
            ]
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .into(),
            ..Start::default()
        }
    }

    #[test]
    fn a_start_on_another_machine_carries_the_same_app_labels_as_one_here() {
        let here = start_of("watch", "ankh-morpork#12", "author-22", "author");
        let there = Start {
            machine: Some("lancre".into()),
            ..here.clone()
        };
        let (here, there) = (run_params(&here), run_params(&there));
        assert_eq!(there["on"], "lancre");
        assert_eq!(there["labels"], here["labels"]);
        assert_eq!(there["labels"]["herdr.space"], "sptemet-work-watch");
        assert_eq!(
            there["labels"]["herdr.tab"],
            "tbtemet-work-watch-ankh-morpork-12"
        );
        assert_eq!(there["labels"]["herdr.name"], "author-22 ankh-morpork#12");
    }

    #[test]
    fn a_group_shares_a_space_and_an_issues_agents_share_a_tab() {
        let labels = |group, issue, agent, role| {
            let params = run_params(&start_of(group, issue, agent, role));
            let label = |key: &str| params["labels"][key].as_str().map(String::from);
            (label("herdr.space"), label("herdr.tab"))
        };
        let author = labels("watch", "ankh-morpork#12", "author-22", "author");
        let reviewer = labels("watch", "ankh-morpork#12", "reviewer-3", "reviewer");
        let other_project = labels("watch", "pseudopolis#12", "author-23", "author");
        let other_group = labels("guild", "ankh-morpork#12", "author-24", "author");
        assert!(author.0.is_some() && author.1.is_some());
        assert_eq!(reviewer, author);
        assert_eq!(other_project.0, author.0);
        assert_ne!(other_project.1, author.1);
        assert_ne!(other_group.0, author.0);
        assert_ne!(other_group.1, author.1);
    }

    #[test]
    fn space_and_tab_ids_keep_to_letters_digits_and_hyphens() {
        let ids = |group, issue| {
            let params = run_params(&start_of(group, issue, "author-1", "author"));
            ["herdr.space", "herdr.tab", "herdr.name"].map(|key| {
                params["labels"][key]
                    .as_str()
                    .unwrap_or_default()
                    .to_string()
            })
        };
        assert_eq!(
            ids("watch", "ankh#WATCH-12"),
            [
                "sptemet-work-watch",
                "tbtemet-work-watch-ankh-WATCH-12",
                "author-1 ankh#WATCH-12"
            ]
        );
        assert_eq!(
            ids("watch", "ankh-morpork#12")[1],
            "tbtemet-work-watch-ankh-morpork-12"
        );
        assert_eq!(
            ids("watch", "ankh-morpork/watch#12")[1],
            "tbtemet-work-watch-ankh-morpork-watch-12"
        );
        assert_eq!(
            ids("night watch_2", "unseen.university#7"),
            [
                "sptemet-work-night-watch-2",
                "tbtemet-work-night-watch-2-unseen-university-7",
                "author-1 unseen.university#7"
            ]
        );
    }

    #[test]
    fn an_investigator_gets_its_groups_space_and_a_tab_of_its_own() {
        let labels = [
            ("agent", "investigator-1"),
            ("group", "watch"),
            ("role", "investigator"),
        ];
        let params = run_params(&Start {
            labels: labels.map(|(k, v)| (k.to_string(), v.to_string())).into(),
            ..Start::default()
        });
        assert_eq!(params["labels"]["herdr.space"], "sptemet-work-watch");
        assert_eq!(
            params["labels"]["herdr.tab"],
            "tbtemet-work-watch-investigator"
        );
        assert_eq!(params["labels"]["herdr.name"], "investigator-1 watch");
    }

    #[test]
    fn a_start_without_a_group_gets_no_app_labels() {
        let app_keys = |labels: &[(&str, &str)]| {
            let params = run_params(&Start {
                labels: labels
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                ..Start::default()
            });
            params["labels"]
                .as_object()
                .into_iter()
                .flatten()
                .filter(|(k, _)| k.starts_with("herdr."))
                .count()
        };
        // The runner contract's own terminals.
        assert_eq!(app_keys(&[("item", "contract-1"), ("role", "contract")]), 0);
        assert_eq!(app_keys(&[("agent", "author-1"), ("issue", "ankh#1")]), 0);
    }

    fn trust_on_mac() -> Program {
        Program {
            argv: ["temet-work", "trust-folder", "claude", "/Users/vimes/watch"]
                .map(String::from)
                .to_vec(),
            env: vec![(
                "CLAUDE_CONFIG_DIR".into(),
                "/Users/vimes/.claude-watch".into(),
            )],
        }
    }

    #[test]
    fn exec_sends_its_arguments_as_given_with_the_machine_and_the_environment() {
        let params = exec_params(Some("mac"), &trust_on_mac());
        assert_eq!(params["on"], "mac");
        assert_eq!(
            params["argv"],
            json!(["temet-work", "trust-folder", "claude", "/Users/vimes/watch"])
        );
        assert_eq!(
            params["env"],
            json!({"CLAUDE_CONFIG_DIR": "/Users/vimes/.claude-watch"})
        );
        assert!(params.get("line").is_none(), "no shell");
        assert!(
            params["timeout"]
                .as_u64()
                .is_some_and(|t| t < REQUEST_TIMEOUT.as_secs())
        );
        let here = exec_params(None, &Program::default());
        assert!(
            here.get("on").is_none() && here.get("env").is_none(),
            "{here}"
        );
    }

    /// `machine.exec`'s answer for a program that exited `code` having written
    /// `stderr`, base64 as the core sends it.
    fn exited(code: Option<i64>, stderr: &str) -> Result<Value, RpcError> {
        Ok(
            json!({"code": code, "signal": null, "stderr": stderr, "stdout": "",
                  "timed_out": false, "truncated": false, "ms": 9}),
        )
    }

    /// `/Users/vimes/.claude.json is not a JSON object` and a line break, in base64.
    const NOT_JSON: &str = "L1VzZXJzL3ZpbWVzLy5jbGF1ZGUuanNvbiBpcyBub3QgYSBKU09OIG9iamVjdAo=";

    /// `sh: 1: temet-work: not found` and a line break, in base64.
    const NOT_FOUND: &str = "c2g6IDE6IHRlbWV0LXdvcms6IG5vdCBmb3VuZAo=";

    /// `error: unexpected argument '--x'`, a blank line and a `Usage:` line, in
    /// base64.
    const BAD_ARGUMENT: &str =
        "ZXJyb3I6IHVuZXhwZWN0ZWQgYXJndW1lbnQgJy0teCcKClVzYWdlOiB0ZW1ldC13b3JrIFtPUFRJT05TXQo=";

    /// `no command trust-folder`, a blank line and the end of the usage text, in
    /// base64.
    const NO_COMMAND: &str = "bm8gY29tbWFuZCB0cnVzdC1mb2xkZXIKCnVzYWdlOiB0ZW1ldC13b3JrIENPTU1BTkQKCkFnZW50cyBhcmUga25vd24KdGhyb3VnaCBURU1FVF9XT1JLX0FHRU5ULCB3aGljaCB0aGUgZGFlbW9uIHNldHMgd2hlbiBpdCBzdGFydHMgdGhlbS4K";

    #[test]
    fn exec_maps_an_exit_of_zero_to_done_and_any_other_to_its_first_error_line() {
        let argv = trust_on_mac().argv;
        let run = |code, stderr| exec_outcome(None, Some("mac"), &argv, exited(Some(code), stderr));
        assert_eq!(run(0, ""), Ok(()));
        assert_eq!(
            run(1, NOT_JSON),
            Err(RunnerError::Other(
                "temet-work exited 1 on mac: /Users/vimes/.claude.json is not a JSON object".into()
            ))
        );
        assert_eq!(
            run(1, BAD_ARGUMENT),
            Err(RunnerError::Other(
                "temet-work exited 1 on mac: error: unexpected argument '--x'".into()
            ))
        );
    }

    #[test]
    fn exec_says_a_temet_work_without_the_command_is_older_than_the_daemon() {
        let argv = trust_on_mac().argv;
        let run = |build| exec_outcome(build, Some("mac"), &argv, exited(Some(2), NO_COMMAND));
        assert_eq!(
            run(Some("v0.4-1-gdeadbee")),
            Err(RunnerError::Other(
                "temet-work on mac is older than this daemon (build v0.4-1-gdeadbee): update it"
                    .into()
            ))
        );
        assert_eq!(
            run(None),
            Err(RunnerError::Other(
                "temet-work on mac is older than this daemon: update it".into()
            ))
        );
    }

    #[test]
    fn exec_maps_exit_127_and_a_missing_program_to_not_on_the_other_machines_path() {
        let argv = trust_on_mac().argv;
        let Err(RunnerError::Other(why)) =
            exec_outcome(None, Some("mac"), &argv, exited(Some(127), NOT_FOUND))
        else {
            panic!("not an error");
        };
        assert!(
            why.starts_with("temet-work is not on mac's PATH") && why.contains("exit 127"),
            "{why}"
        );
        // herdr-core runs argv with no shell: a missing program is its error.
        let missing = Err(RpcError {
            kind: "spawn_failed".into(),
            message: "cannot start temet-work: No such file or directory (os error 2)".into(),
        });
        let Err(RunnerError::Other(why)) = exec_outcome(None, Some("mac"), &argv, missing) else {
            panic!("not an error");
        };
        assert!(why.starts_with("temet-work is not on mac's PATH"), "{why}");
    }

    #[test]
    fn exec_reports_a_timeout_a_signal_and_an_unknown_machine() {
        let argv = trust_on_mac().argv;
        let timed_out = Ok(
            json!({"code": null, "signal": 9, "stderr": "", "stdout": "",
                                  "timed_out": true, "truncated": false, "ms": 8000}),
        );
        let Err(RunnerError::Other(why)) = exec_outcome(None, Some("mac"), &argv, timed_out) else {
            panic!("not an error");
        };
        assert!(why.contains("did not finish on mac within 8 s"), "{why}");
        let Err(RunnerError::Other(why)) = exec_outcome(None, Some("mac"), &argv, exited(None, ""))
        else {
            panic!("not an error");
        };
        assert!(why.contains("ended by signal"), "{why}");
        let unknown = Err(RpcError {
            kind: "machine_unknown".into(),
            message: "unknown machine \"lancre\"".into(),
        });
        let Err(RunnerError::Other(why)) = exec_outcome(None, Some("lancre"), &argv, unknown)
        else {
            panic!("not an error");
        };
        assert!(why.contains("unknown machine \"lancre\""), "{why}");
    }

    #[test]
    fn decodes_base64_with_and_without_padding() {
        assert_eq!(unbase64("Ygo="), b"b\n");
        assert_eq!(unbase64("aGVsbG8"), b"hello");
        assert_eq!(unbase64("aGVs\nbG8="), b"hello");
        assert_eq!(unbase64(""), b"");
    }

    #[test]
    fn maps_idle_and_done_to_between_turns() {
        assert_eq!(status_of(Some("idle"), None, None), Status::Between);
        assert_eq!(status_of(Some("done"), None, None), Status::Between);
        assert_eq!(status_of(Some("working"), None, None), Status::Working);
        assert_eq!(
            status_of(Some("blocked"), Some("question"), None),
            Status::Blocked {
                kind: "question".into()
            }
        );
        // A turn that ended in an error, with the core's message when it sends one.
        assert_eq!(
            status_of(
                Some("error"),
                None,
                Some("You've hit your 5-hour limit · resets 5:40pm")
            ),
            Status::Error {
                message: Some("You've hit your 5-hour limit · resets 5:40pm".into())
            }
        );
        assert_eq!(
            status_of(Some("error"), None, None),
            Status::Error { message: None }
        );
        assert_eq!(
            status_of(Some("error"), None, Some("  ")),
            Status::Error { message: None },
            "a blank message is none"
        );
        assert_eq!(status_of(None, None, None), Status::Unknown);
    }

    #[test]
    fn maps_core_errors_to_runner_errors() {
        let t = TerminalId("t7".into());
        let e = |kind: &str| RpcError {
            kind: kind.into(),
            message: String::new(),
        };
        assert_eq!(mapped(&t, e("blocked")), RunnerError::Blocked);
        assert_eq!(mapped(&t, e("status_mismatch")), RunnerError::Busy);
        assert_eq!(mapped(&t, e("finished")), RunnerError::Gone);
        assert_eq!(mapped(&t, e("not_found")), RunnerError::Unknown(t.clone()));
    }

    #[test]
    fn a_snapshot_becomes_one_update_per_terminal_with_its_labels() {
        let mut s = Shared::default();
        let snapshot = json!([
            {"address": "t3", "labels": {"agent": "author-1", "owner": "temet-work"},
             "status": {"state": "working", "msg": "running the tests"}},
            {"address": "t4", "labels": {"agent": "refiner-1"}, "status": {"state": "done"}},
            {"address": "t5", "labels": {"agent": "author-2"}, "state": "finished",
             "status": {"state": "done"}},
            {"address": "t6", "labels": {"agent": "author-3"}, "status": {"state": "error"}},
            {"labels": {"agent": "no-address"}}
        ]);
        take_snapshot(&snapshot, &mut s);
        assert_eq!(s.updates.len(), 4);
        assert_eq!(s.updates[3].labels["agent"], "author-3");
        assert_eq!(
            s.updates[3].status,
            Status::Error { message: None },
            "an agent whose turn ended in an error before the daemon started"
        );
        assert_eq!(s.updates[0].labels["agent"], "author-1");
        assert_eq!(s.updates[0].status, Status::Working);
        assert_eq!(s.updates[0].doing.as_deref(), Some("running the tests"));
        assert_eq!(s.updates[1].status, Status::Between);
        assert_eq!(s.updates[2].status, Status::Gone, "finished, not removed");
        // A later list without t3 reports it gone.
        s.updates.clear();
        take_snapshot(
            &json!([{"address": "t4", "labels": {"agent": "refiner-1"}}]),
            &mut s,
        );
        let gone: Vec<&str> = s
            .updates
            .iter()
            .filter(|u| u.status == Status::Gone)
            .map(|u| u.terminal.0.as_str())
            .collect();
        assert_eq!(gone, ["t3", "t5", "t6"]);
    }

    /// An author's start as the runner family makes it, with the program a
    /// Claude agent runs.
    fn claude_start() -> Start {
        Start {
            program: Program {
                argv: ["claude", "--permission-mode", "auto", "Mend the clock"]
                    .map(String::from)
                    .to_vec(),
                env: Vec::new(),
            },
            ..start_of("watch", "ankh#1", "author-1", "author")
        }
    }

    #[test]
    fn run_params_give_every_terminal_a_program_label_with_its_first_word() {
        let params = run_params(&claude_start());
        assert_eq!(params["labels"]["program"], "claude");
        assert_eq!(params["labels"]["agent"], "author-1");
        let contract = Start {
            program: Program {
                argv: vec!["cat".into()],
                env: Vec::new(),
            },
            ..Start::default()
        };
        assert_eq!(run_params(&contract)["labels"]["program"], "cat");
        assert!(
            run_params(&Start::default())["labels"]
                .get("program")
                .is_none(),
            "nothing to name"
        );
    }

    /// The labels an author's terminal carries in herdr-core, as `run_params`
    /// gives them and the herdr app adds to them.
    fn agent_labels(program: bool) -> Value {
        let mut labels = run_params(&claude_start())["labels"].clone();
        labels["log"] = json!("1a21e7fd7475c954");
        labels["where"] = json!("watch / 1");
        if !program {
            labels.as_object_mut().map(|m| m.remove("program"));
        }
        labels
    }

    /// The status `take_snapshot` gives the one terminal `terminal` lists.
    fn snapshot_status(terminal: Value) -> Status {
        let mut s = Shared::default();
        take_snapshot(&json!([terminal]), &mut s);
        assert_eq!(s.updates.len(), 1, "{:?}", s.updates);
        s.updates[0].status.clone()
    }

    /// The issue's first snapshot: a kept agent terminal whose program was
    /// killed, before herdr-core has put a shell in its place.
    fn recovering(labels: Value) -> Value {
        json!({"address": "t7", "state": "live", "reason": "signaled", "status": null,
               "pid": null, "keep": true, "labels": labels,
               "argv": ["claude", "--permission-mode", "auto", "Mend the clock"]})
    }

    /// The issue's second snapshot: the same terminal once the shell is in place.
    fn recovered(labels: Value) -> Value {
        json!({"address": "t7", "state": "live", "reason": null, "status": null,
               "pid": 3377947, "keep": true, "labels": labels, "argv": ["/usr/bin/zsh"]})
    }

    #[test]
    fn a_lost_program_is_lost_while_recovering_and_once_a_shell_has_replaced_it() {
        assert_eq!(
            snapshot_status(recovering(agent_labels(true))),
            Status::Lost
        );
        assert_eq!(snapshot_status(recovered(agent_labels(true))), Status::Lost);
        // A shell left by the runner contract's echo program, which is one word.
        let contract = json!({"item": "contract-1", "role": "contract", "owner": "temet-work",
                              "program": "cat"});
        assert_eq!(
            snapshot_status(recovered(contract.clone())),
            Status::Lost,
            "the program label does not need an agent"
        );
        assert_eq!(
            snapshot_status(json!({"address": "t8", "state": "live", "labels": contract,
                                   "argv": ["cat"], "status": null})),
            Status::Unknown,
            "the program still runs"
        );
    }

    #[test]
    fn a_lost_program_without_label_is_lost_once_a_shell_has_replaced_it() {
        // An agent terminal started by a build before the program label.
        assert_eq!(
            snapshot_status(recovered(agent_labels(false))),
            Status::Lost
        );
        assert_eq!(
            snapshot_status(recovering(agent_labels(false))),
            Status::Lost
        );
        // Listed without its argv, a terminal is not read as lost.
        let mut no_argv = recovered(agent_labels(false));
        no_argv.as_object_mut().map(|m| m.remove("argv"));
        assert_eq!(snapshot_status(no_argv), Status::Unknown);
        // A one-word program that is no agent's is not read as a shell.
        assert_eq!(
            snapshot_status(json!({"address": "t8", "state": "live", "status": null,
                                   "labels": {"item": "contract-1", "owner": "temet-work"},
                                   "argv": ["cat"]})),
            Status::Unknown
        );
    }

    #[test]
    fn a_lost_event_reports_lost_and_finished_and_removed_events_report_gone() {
        let mut s = Shared::default();
        take_event(
            &json!({"type": "created", "terminal": "t7",
                    "data": {"labels": agent_labels(true)}}),
            &mut s,
        );
        for kind in ["lost", "finished", "removed"] {
            take_event(&json!({"type": kind, "terminal": "t7", "data": {}}), &mut s);
        }
        let statuses: Vec<Status> = s.updates.iter().map(|u| u.status.clone()).collect();
        assert_eq!(
            statuses,
            [Status::Unknown, Status::Lost, Status::Gone, Status::Gone]
        );
        assert!(s.updates.iter().all(|u| u.labels["agent"] == "author-1"));
    }

    #[test]
    fn a_just_started_agent_terminal_with_no_status_yet_is_unknown() {
        let started = |labels| {
            json!({"address": "t7", "state": "live", "reason": null, "status": null,
                   "pid": 3369541, "keep": false, "labels": labels,
                   "argv": ["claude", "--permission-mode", "auto", "Mend the clock"]})
        };
        assert_eq!(
            snapshot_status(started(agent_labels(true))),
            Status::Unknown
        );
        assert_eq!(
            snapshot_status(started(agent_labels(false))),
            Status::Unknown,
            "started by a build before the program label"
        );
        // And once it reports, its status counts.
        let mut working = started(agent_labels(true));
        working["status"] = json!({"state": "working", "msg": "reading the brief"});
        assert_eq!(snapshot_status(working), Status::Working);
    }

    #[test]
    fn opens_nothing_in_the_sandbox() {
        // Every test runs with TEMET_WORK_SANDBOX=1 (.cargo/config.toml); a live run
        // clears it, and then this test has nothing to check.
        if !sandboxed() {
            return;
        }
        let mut h = Herdr::default();
        assert!(matches!(
            h.screen(&TerminalId("t1".into())),
            Err(RunnerError::Other(_))
        ));
    }
}
