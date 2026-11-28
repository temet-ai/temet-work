//! Whether a stop that fails may have reached herdr-core, against a stand-in
//! core on a Unix socket in a temporary folder, or none at all.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use serde_json::{Value, json};
use temet_work_runner::{Runner, RunnerError, TerminalId};
use temet_work_runner_herdr::Herdr;

/// What the stand-in does on its control connection after it reads a request.
#[derive(Clone, Copy)]
enum After {
    /// Closes the connection without answering `terminal.kill`.
    ReadingKill,
    /// Answers `terminal.kill`, then closes the connection.
    AnsweringKill,
}

/// Answers one connection: `hello` and `events.subscribe`, and `terminal.kill`
/// as `after` says.
fn answer(stream: UnixStream, after: After) {
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
        if writeln!(writer, "{reply}").is_err() || method == Some("terminal.kill") {
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
    std::thread::spawn(move || {
        for stream in listener.incoming().map_while(Result::ok) {
            std::thread::spawn(move || answer(stream, after));
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
fn a_stop_whose_connection_closes_after_the_kill_may_have_reached_the_core() {
    let stopped = stop_against("answered", After::AnsweringKill);
    assert!(matches!(stopped, Err(RunnerError::Other(_))), "{stopped:?}");
}
