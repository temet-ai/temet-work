//! An agent's kept terminal across a herdr-core restart, as the client reads it,
//! against a stand-in core on a Unix socket in a temporary folder. The stand-in
//! closes the first event subscription right after its list, as a core that
//! restarts does, and answers the next subscription with the list a restarted
//! core gives (`docs/findings/herdr-core.md`, "Across a core restart").

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use temet_work_runner::{Runner, Status, TerminalId, Update};
use temet_work_runner_herdr::Herdr;

/// The labels the runner family and the client give an author's terminal.
fn labels() -> Value {
    json!({"owner": "temet-work", "program": "claude", "agent": "author-1", "group": "watch",
           "issue": "ankh#1", "role": "author", "log": "5f3a"})
}

/// t7 as the first subscription lists it: the author idle in its program.
fn running() -> Value {
    json!({"address": "t7", "state": "live", "keep": true, "reason": null, "pid": 41,
           "argv": ["claude", "--permission-mode", "auto", "Mend the clock"],
           "status": {"state": "idle"}, "labels": labels()})
}

/// Answers one connection. The first `events.subscribe` of the stand-in lists
/// [`running`], and the connection is closed after the answer; a later one
/// lists `after`.
fn answer(stream: UnixStream, subscribes: &AtomicUsize, after: &Value) {
    let Ok(mut writer) = stream.try_clone() else {
        return;
    };
    for line in BufReader::new(stream).lines().map_while(Result::ok) {
        let request: Value = serde_json::from_str(&line).unwrap_or_default();
        let (result, close) = match request["method"].as_str() {
            Some("events.subscribe") if subscribes.fetch_add(1, Ordering::SeqCst) == 0 => {
                (json!({"seq": 0, "terminals": [running()]}), true)
            }
            Some("events.subscribe") => (json!({"seq": 0, "terminals": after}), false),
            _ => (json!({}), false),
        };
        let reply = json!({"jsonrpc": "2.0", "id": request["id"], "result": result});
        if writeln!(writer, "{reply}").is_err() || close {
            return;
        }
    }
}

/// The updates the client gives for t7 once it has subscribed again to a
/// stand-in whose second list is `after`.
fn after_a_restart(case: &str, after: Value) -> Vec<Update> {
    // mike-keep: rust-temp-dir the folder is made new with mode 0700 below; a path that exists fails the test
    let folder = std::env::temp_dir().join(format!(
        "temet-work-herdr-reconnect-{case}-{}",
        std::process::id()
    ));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&folder)
        .expect("a new temporary folder");
    let socket = folder.join("core.sock");
    let listener = UnixListener::bind(&socket).expect("the stand-in's socket");
    let subscribes = Arc::new(AtomicUsize::new(0));
    let after = Arc::new(after);
    std::thread::spawn(move || {
        for stream in listener.incoming().map_while(Result::ok) {
            let (subscribes, after) = (Arc::clone(&subscribes), Arc::clone(&after));
            std::thread::spawn(move || answer(stream, &subscribes, &after));
        }
    });

    let mut herdr = Herdr::default().socket(&socket);
    let first = herdr.updates();
    // The client sees the subscription end, then subscribes again at the
    // next call to `updates`.
    let deadline = Instant::now() + Duration::from_secs(2);
    let later = loop {
        let updates = herdr.updates();
        if !updates.is_empty() || Instant::now() > deadline {
            break updates;
        }
        std::thread::sleep(Duration::from_millis(1));
    };
    let _ = std::fs::remove_dir_all(&folder);

    assert_eq!(first.len(), 1, "{first:?}");
    assert_eq!(first[0].status, Status::Between);
    later
}

/// The one update in `updates`, which is t7's, with the agent's labels.
fn t7(updates: &[Update]) -> Status {
    assert_eq!(updates.len(), 1, "{updates:?}");
    assert_eq!(updates[0].terminal, TerminalId("t7".into()));
    assert_eq!(
        updates[0].labels.get("agent").map(String::as_str),
        Some("author-1")
    );
    assert_eq!(
        updates[0].labels.get("issue").map(String::as_str),
        Some("ankh#1")
    );
    updates[0].status.clone()
}

#[test]
fn an_agents_terminal_missing_from_the_list_after_a_reconnect_is_lost_with_its_labels() {
    assert_eq!(t7(&after_a_restart("missing", json!([]))), Status::Lost);
}

#[test]
fn an_agents_terminal_a_restarted_core_lists_with_a_shell_in_its_programs_place_is_lost() {
    // After `hc server stop` and `hc server start`.
    let shell = json!([{"address": "t7", "state": "live", "keep": true, "reason": null,
                        "pid": 3957249, "argv": ["/usr/bin/zsh"], "status": null,
                        "labels": labels()}]);
    assert_eq!(t7(&after_a_restart("shell", shell)), Status::Lost);
    // After the core and the program's holder were killed, before the shell.
    let stopped = json!([{"address": "t7", "state": "live", "keep": true,
                          "reason": "stopped_by_restart", "pid": null, "status": null,
                          "argv": ["claude", "--permission-mode", "auto", "Mend the clock"],
                          "labels": labels()}]);
    assert_eq!(t7(&after_a_restart("stopped", stopped)), Status::Lost);
}

#[test]
fn an_agents_terminal_whose_program_outlived_the_restart_is_not_lost() {
    // After `hc server restart`, or a SIGKILL of the core alone: the program
    // runs on, and the new core has no status for it yet.
    let kept = json!([{"address": "t7", "state": "live", "keep": true, "reason": null,
                       "pid": 41, "status": null, "labels": labels(),
                       "argv": ["claude", "--permission-mode", "auto", "Mend the clock"]}]);
    assert_eq!(t7(&after_a_restart("kept", kept)), Status::Unknown);
}
