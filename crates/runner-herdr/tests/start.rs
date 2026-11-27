//! A start as herdr-core receives it, against a stand-in core on a Unix socket
//! in a temporary folder.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use serde_json::{Value, json};
use temet_work_runner::{Program, Runner, Start, TerminalId};
use temet_work_runner_herdr::Herdr;

/// Answers one client connection, and passes each `terminal.run` line on.
fn answer(stream: UnixStream, runs: &Sender<String>) {
    let Ok(mut writer) = stream.try_clone() else {
        return;
    };
    for line in BufReader::new(stream).lines().map_while(Result::ok) {
        let request: Value = serde_json::from_str(&line).unwrap_or_default();
        let result = match request["method"].as_str() {
            Some("events.subscribe") => json!({"terminals": []}),
            Some("terminal.run") => {
                let _ = runs.send(line.clone());
                json!({"address": "t7"})
            }
            _ => json!({}),
        };
        let reply = json!({"jsonrpc": "2.0", "id": request["id"], "result": result});
        if writeln!(writer, "{reply}").is_err() {
            return;
        }
    }
}

#[test]
fn an_agents_terminal_run_carries_its_herdr_space_tab_and_name() {
    // mike-keep: rust-temp-dir the folder is made new with mode 0700 below; a path that exists fails the test
    let folder = std::env::temp_dir().join(format!("temet-work-herdr-{}", std::process::id()));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&folder)
        .expect("a new temporary folder");
    let socket = folder.join("core.sock");
    let listener = UnixListener::bind(&socket).expect("the stand-in's socket");
    let (runs, received) = channel();
    std::thread::spawn(move || {
        for stream in listener.incoming().map_while(Result::ok) {
            let runs = runs.clone();
            std::thread::spawn(move || answer(stream, &runs));
        }
    });

    let given = [
        ("agent", "author-22"),
        ("group", "watch"),
        ("issue", "ankh-morpork#12"),
        ("role", "author"),
        ("log", "5f3a"),
    ];
    let start = Start {
        program: Program {
            argv: vec!["agent".into()],
            env: Vec::new(),
        },
        cwd: folder.clone(),
        labels: given.map(|(k, v)| (k.to_string(), v.to_string())).into(),
        machine: None,
    };
    let started = Herdr::default().socket(&socket).start(&start);
    let line = received.recv_timeout(Duration::from_secs(1));
    let _ = std::fs::remove_dir_all(&folder);

    assert_eq!(started, Ok(TerminalId("t7".into())));
    let request: Value =
        serde_json::from_str(&line.expect("a terminal.run line")).expect("a line of JSON");
    let labels: BTreeMap<String, String> =
        serde_json::from_value(request["params"]["labels"].clone()).expect("labels");
    let expected: BTreeMap<String, String> = given
        .into_iter()
        .chain([
            ("owner", "temet-work"),
            ("program", "agent"),
            ("herdr.space", "sptemet-work-watch"),
            ("herdr.tab", "tbtemet-work-watch-ankh-morpork-12"),
            ("herdr.name", "author-22 ankh-morpork#12"),
        ])
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    assert_eq!(labels, expected);
}
