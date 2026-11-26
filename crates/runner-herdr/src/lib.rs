//! herdr-core terminals, reached over its JSON-RPC socket: one JSON message per
//! line, `hello` first.
//!
//! What the client relies on comes from `docs/findings/herdr-core.md`: labels
//! set when a terminal starts, so its first event already carries them; one
//! subscription filtered by temet-work's own label, on its own connection; `idle`
//! and `done` both meaning a turn ended; a send refused while the terminal is at
//! a prompt; the visible screen as the cheap read. Terminals are started with no
//! parent, so agents outlive a restart of the daemon.
//!
//! While `TEMET_WORK_SANDBOX` is set to anything but empty, as it is in every
//! test run, the client opens nothing and every call fails.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use temet_work_runner::{Runner, RunnerError, Start, Status, TerminalId, Update};

/// The label every terminal this client starts carries, and the subscription's filter.
const OWNER: (&str, &str) = ("owner", "temet-work");

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
    fn open(path: &PathBuf) -> Result<Self, RunnerError> {
        let stream = UnixStream::connect(path).map_err(|e| {
            RunnerError::Other(format!(
                "cannot reach herdr-core at {}: {e}",
                path.display()
            ))
        })?;
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

fn status_of(state: Option<&str>, kind: Option<&str>) -> Status {
    match state {
        Some("idle" | "done") => Status::Between,
        Some("working") => Status::Working,
        Some("blocked") => Status::Blocked {
            kind: kind.unwrap_or("prompt").to_string(),
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

/// What the event reader shares with the client.
#[derive(Default)]
struct Shared {
    updates: Vec<Update>,
    labels: BTreeMap<String, BTreeMap<String, String>>,
    alive: bool,
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
            Some("event") => {
                let p = &msg["params"];
                let Some(t) = p.get("terminal").and_then(Value::as_str) else {
                    continue;
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
                    )),
                    "finished" | "lost" | "removed" => Some(Status::Gone),
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
            Some("missed") => {
                // The reader fell behind: the core sends a fresh snapshot of terminals.
                for t in msg
                    .pointer("/params/terminals")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    let Some(addr) = t.get("address").and_then(Value::as_str) else {
                        continue;
                    };
                    let labels = labels_of(t.get("labels"));
                    s.labels.insert(addr.to_string(), labels.clone());
                    s.updates.push(Update {
                        terminal: TerminalId(addr.to_string()),
                        labels,
                        status: status_of(
                            t.pointer("/status/state").and_then(Value::as_str),
                            t.pointer("/status/kind").and_then(Value::as_str),
                        ),
                        doing: None,
                    });
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
}

impl Herdr {
    fn conn(&mut self) -> Result<&mut Conn, RunnerError> {
        if sandboxed() {
            return Err(RunnerError::Other(
                "TEMET_WORK_SANDBOX is set: herdr-core is not reached".into(),
            ));
        }
        let path = socket_path().ok_or_else(|| {
            RunnerError::Other(
                "no herdr-core socket: set HC_TERMINAL_SOCKET or XDG_RUNTIME_DIR".into(),
            )
        })?;
        let following = self.shared.lock().map(|s| s.alive).unwrap_or(false);
        if !following {
            let mut events = Conn::open(&path)?;
            events
                .call("events.subscribe", json!({"labels": {OWNER.0: OWNER.1}}))
                .map_err(rpc)?;
            if let Ok(mut s) = self.shared.lock() {
                s.alive = true;
            }
            let shared = Arc::clone(&self.shared);
            std::thread::spawn(move || follow(events, shared));
        }
        if self.control.is_none() {
            self.control = Some(Conn::open(&path)?);
        }
        self.control
            .as_mut()
            .ok_or_else(|| RunnerError::Other("no connection".into()))
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

    fn start(&mut self, start: &Start) -> Result<TerminalId, RunnerError> {
        let mut labels = start.labels.clone();
        labels.insert(OWNER.0.into(), OWNER.1.into());
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
        let result = self.conn()?.call("terminal.run", params).map_err(rpc)?;
        result
            .get("address")
            .and_then(Value::as_str)
            .map(|a| TerminalId(a.to_string()))
            .ok_or_else(|| {
                RunnerError::Other("herdr-core gave no address for the new terminal".into())
            })
    }

    fn send(
        &mut self,
        terminal: &TerminalId,
        text: &str,
        only_if_idle: bool,
    ) -> Result<(), RunnerError> {
        let conn = self.conn()?;
        if only_if_idle {
            // herdr-core's `if` matches one word, and a turn that ended reads `done`,
            // not `idle`: check the status first, then send without `if`.
            let facts = conn
                .call("terminal.get", json!({"terminal": terminal.0}))
                .map_err(|e| mapped(terminal, e))?;
            if facts.pointer("/status/state").and_then(Value::as_str) == Some("working") {
                return Err(RunnerError::Busy);
            }
        }
        conn.call(
            "terminal.send",
            json!({"terminal": terminal.0, "text": text}),
        )
        .map(|_| ())
        .map_err(|e| mapped(terminal, e))
    }

    fn screen(&mut self, terminal: &TerminalId) -> Result<String, RunnerError> {
        let result = self
            .conn()?
            .call("terminal.read", json!({"terminal": terminal.0, "lines": 0}))
            .map_err(|e| mapped(terminal, e))?;
        Ok(result
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string())
    }

    fn stop(&mut self, terminal: &TerminalId) -> Result<(), RunnerError> {
        let conn = self.conn()?;
        match conn.call("terminal.kill", json!({"terminal": terminal.0})) {
            Ok(_) => {}
            Err(e) if e.kind == "finished" => {}
            Err(e) => return Err(mapped(terminal, e)),
        }
        // `rm` refuses a terminal that is still live; kill reports once it has ended,
        // but give the core a moment if it has not caught up.
        for _ in 0..20 {
            match conn.call("terminal.rm", json!({"terminal": terminal.0})) {
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

    fn updates(&mut self) -> Vec<Update> {
        self.shared
            .lock()
            .map(|mut s| std::mem::take(&mut s.updates))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_idle_and_done_to_between_turns() {
        assert_eq!(status_of(Some("idle"), None), Status::Between);
        assert_eq!(status_of(Some("done"), None), Status::Between);
        assert_eq!(status_of(Some("working"), None), Status::Working);
        assert_eq!(
            status_of(Some("blocked"), Some("question")),
            Status::Blocked {
                kind: "question".into()
            }
        );
        assert_eq!(status_of(None, None), Status::Unknown);
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
