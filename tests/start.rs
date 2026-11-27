//! The daemon does not start with a setting in its file that names a runtime or
//! runner this build lacks, and says which ones it has.

#![cfg(all(
    feature = "runtime-claude",
    feature = "runtime-codex",
    feature = "runner-herdr"
))]

use std::process::Command;

fn start(name: &str, config: &str) -> std::process::Output {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&dir).expect("a folder");
    let file = dir.join("config.toml");
    std::fs::write(&file, config).expect("a config");
    Command::new(env!("CARGO_BIN_EXE_temet-workd"))
        .args(["--once", "--config"])
        .arg(&file)
        .arg("--log")
        .arg(dir.join("events.jsonl"))
        .env("TEMET_WORK_SANDBOX", "1")
        .output()
        .expect("runs")
}

#[test]
fn a_runtime_this_build_lacks_in_the_file_stops_the_daemon_listing_the_ones_it_has() {
    let out = start("start-runtime", "[settings]\nruntime = \"nonesuch\"\n");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("nonesuch"), "{err}");
    assert!(err.contains("this build has: claude, codex"), "{err}");
}

#[test]
fn a_runner_this_build_lacks_at_any_level_stops_the_daemon_too() {
    let config = "[[project]]\nid = \"radar\"\ntracker = \"github\"\nname = \"temet-ai/radar\"\n\
                  [project.settings.reviewer]\nrunner = \"nonesuch\"\n";
    let out = start("start-runner", config);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("nonesuch") && err.contains("herdr"), "{err}");
}
