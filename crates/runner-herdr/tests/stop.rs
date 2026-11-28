//! Whether a stop that fails may have reached herdr-core, and a stop after the
//! core restarted, against a stand-in core on a Unix socket in a temporary
//! folder, or none at all.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use serde_json::{Value, json};
use temet_work_runner::{Runner, RunnerError, TerminalId};
use temet_work_runner_herdr::Herdr;

/// What the stand-in does on its control connection after it reads a request.
#[derive(Clone, Copy)]
enum After {
    /// Closes the connection without answering `terminal.kill`.
    ReadingKill,
    /// Answers `terminal.kill`, then removes its socket, so no new connection
    /// reaches it, and closes the connection: the core went away after the
    /// kill.
    AnsweringKill,
}

/// Answers one connection of the stand-in listening at `socket`: `hello` and
/// `events.subscribe`, and `terminal.kill` as `after` says.
fn answer(stream: UnixStream, after: After, socket: &Path) {
    let Ok(mut writer) = stream.try_clone() else {
        return;
    };
    for line in BufReader::new(stream).lines().map_while(Result::ok) {
        let request: Value = serde_json::from_str(&line).unwrap_or_default();
        let method = request["method"].as_str();
        if method == Some("terminal.kill") && matches!(after, After::ReadingKill) {
            return;
        }
        let result = match method {
            Some("events.subscribe") => json!({"seq": 0, "terminals": []}),
            Some("terminal.kill") => json!({"address": "t7", "state": "finished"}),
            _ => json!({}),
        };
        let reply = json!({"jsonrpc": "2.0", "id": request["id"], "result": result});
        if writeln!(writer, "{reply}").is_err() {
            return;
        }
        if method == Some("terminal.kill") {
            let _ = std::fs::remove_file(socket);
            return;
        }
    }
}

/// A new temporary folder for `case`, and the socket path in it.
fn folder(case: &str) -> (PathBuf, PathBuf) {
    // mike-keep: rust-temp-dir the folder is made new with mode 0700 below; a path that exists fails the test
    let folder = std::env::temp_dir().join(format!(
        "temet-work-herdr-stop-{case}-{}",
        std::process::id()
    ));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&folder)
        .expect("a new temporary folder");
    let socket = folder.join("core.sock");
    (folder, socket)
}

/// What a stop of t7 answers against a stand-in that does `after`.
fn stop_against(case: &str, after: After) -> Result<(), RunnerError> {
    let (folder, socket) = folder(case);
    let listener = UnixListener::bind(&socket).expect("the stand-in's socket");
    let path = socket.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming().map_while(Result::ok) {
            let path = path.clone();
            std::thread::spawn(move || answer(stream, after, &path));
        }
    });
    let stopped = Herdr::default()
        .socket(&socket)
        .stop(&TerminalId("t7".into()));
    let _ = std::fs::remove_dir_all(&folder);
    stopped
}

#[test]
fn a_stop_with_no_core_listening_sent_nothing() {
    let (folder, socket) = folder("none");
    let stopped = Herdr::default()
        .socket(&socket)
        .stop(&TerminalId("t7".into()));
    let _ = std::fs::remove_dir_all(&folder);
    let Err(RunnerError::Unreached(why)) = stopped else {
        panic!("{stopped:?}");
    };
    assert!(why.contains("cannot reach herdr-core"), "{why}");
}

#[test]
fn a_stop_whose_kill_was_read_and_not_answered_may_have_reached_the_core() {
    let stopped = stop_against("unanswered", After::ReadingKill);
    assert!(matches!(stopped, Err(RunnerError::Other(_))), "{stopped:?}");
}

#[test]
fn a_stop_whose_core_goes_away_after_the_kill_may_have_reached_the_core() {
    let stopped = stop_against("answered", After::AnsweringKill);
    assert!(matches!(stopped, Err(RunnerError::Other(_))), "{stopped:?}");
}

/// Answers one connection of a core that restarts after its first control
/// connection's first answer: that connection is closed and `closed` told.
/// A later control connection answers `terminal.kill` as the restarted core
/// does for a terminal it does not have.
fn answer_then_restart(stream: UnixStream, controls: &AtomicUsize, closed: &Sender<()>) {
    let Ok(mut writer) = stream.try_clone() else {
        return;
    };
    let mut first_control = None;
    for line in BufReader::new(stream).lines().map_while(Result::ok) {
        let request: Value = serde_json::from_str(&line).unwrap_or_default();
        let method = request["method"].as_str().unwrap_or_default().to_string();
        let reply = match method.as_str() {
            "hello" => json!({"jsonrpc": "2.0", "id": request["id"], "result": {}}),
            "events.subscribe" => {
                json!({"jsonrpc": "2.0", "id": request["id"], "result": {"seq": 0, "terminals": []}})
            }
            "terminal.kill" => json!({"jsonrpc": "2.0", "id": request["id"],
                "error": {"code": -32000, "message": "no terminal \"t7\"",
                          "data": {"kind": "not_found"}}}),
            _ => json!({"jsonrpc": "2.0", "id": request["id"], "result": {"text": ""}}),
        };
        // The first request after `hello` tells the event connection from a
        // control one.
        if method != "hello" && first_control.is_none() {
            first_control =
                Some(method != "events.subscribe" && controls.fetch_add(1, Ordering::SeqCst) == 0);
        }
        if writeln!(writer, "{reply}").is_err() {
            return;
        }
        if first_control == Some(true) {
            drop(writer);
            let _ = closed.send(());
            return;
        }
    }
}

#[test]
fn a_stop_after_the_core_restarted_is_sent_again_on_a_new_connection() {
    let (folder, socket) = folder("restarted");
    let listener = UnixListener::bind(&socket).expect("the stand-in's socket");
    let controls = Arc::new(AtomicUsize::new(0));
    let (closed, restarted) = channel();
    std::thread::spawn(move || {
        for stream in listener.incoming().map_while(Result::ok) {
            let (controls, closed) = (Arc::clone(&controls), closed.clone());
            std::thread::spawn(move || answer_then_restart(stream, &controls, &closed));
        }
    });
    let t7 = TerminalId("t7".into());
    let mut herdr = Herdr::default().socket(&socket);
    let read = herdr.screen(&t7);
    let gone = restarted.recv_timeout(Duration::from_secs(1));
    let stopped = herdr.stop(&t7);
    let _ = std::fs::remove_dir_all(&folder);
    assert_eq!(read, Ok(String::new()));
    assert!(gone.is_ok(), "the first control connection was closed");
    assert_eq!(stopped, Err(RunnerError::Unknown(t7)));
}
