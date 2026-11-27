//! `temet-work trust-folder RUNTIME PATH`, the command the daemon runs on the
//! machine a work folder on another machine is on, run here as the built binary
//! with its user files in a scratch folder.

#![cfg(feature = "runtime-claude")]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A fresh scratch folder under the build folder, with `home/`, `config/` and
/// the work folder `watch/` in it.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/tw-test")
        .join(format!("trust-folder-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for sub in ["home", "config", "watch"] {
        std::fs::create_dir_all(dir.join(sub)).expect("scratch folder");
    }
    dir
}

/// Runs `temet-work trust-folder` with `args`, `HOME` in `dir/home` and
/// `CLAUDE_CONFIG_DIR` in `dir/config` when `config` is set.
fn trust(dir: &Path, config: bool, args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_temet-work"));
    cmd.arg("trust-folder")
        .args(args)
        .env("HOME", dir.join("home"))
        .env_remove("CLAUDE_CONFIG_DIR");
    if config {
        cmd.env("CLAUDE_CONFIG_DIR", dir.join("config"));
    }
    cmd.output().expect("runs")
}

fn folder(dir: &Path) -> String {
    std::fs::canonicalize(dir.join("watch"))
        .expect("the folder")
        .to_string_lossy()
        .into_owned()
}

fn read(file: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(file).expect("file")).expect("json")
}

#[test]
fn a_fresh_folder_is_trusted_other_keys_survive_and_a_second_run_writes_nothing() {
    let dir = scratch("fresh");
    let file = dir.join("home/.claude.json");
    std::fs::write(
        &file,
        r#"{"theme":"dark","projects":{"/other":{"hasTrustDialogAccepted":false,"allowedTools":["x"]}}}"#,
    )
    .expect("write");
    let watch = folder(&dir);
    let out = trust(&dir, false, &["claude", &watch]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        read(&file),
        serde_json::json!({"theme": "dark", "projects": {
            "/other": {"hasTrustDialogAccepted": false, "allowedTools": ["x"]},
            watch.as_str(): {"hasTrustDialogAccepted": true}}})
    );
    let before = std::fs::read(&file).expect("read");
    let mtime = std::fs::metadata(&file)
        .and_then(|m| m.modified())
        .expect("mtime");
    let again = trust(&dir, false, &["claude", &watch]);
    assert!(again.status.success(), "already trusted is done too");
    assert_eq!(std::fs::read(&file).expect("read"), before, "byte for byte");
    assert_eq!(
        std::fs::metadata(&file)
            .and_then(|m| m.modified())
            .expect("mtime"),
        mtime,
        "not written again"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_folder_claude_config_dir_names_holds_the_file_when_it_is_set() {
    let dir = scratch("config");
    let watch = folder(&dir);
    let out = trust(&dir, true, &["claude", &watch]);
    assert!(out.status.success());
    assert_eq!(
        read(&dir.join("config/.claude.json"))["projects"][watch.as_str()]["hasTrustDialogAccepted"],
        true
    );
    assert!(!dir.join("home/.claude.json").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_malformed_file_exits_nonzero_naming_it_and_is_left_as_it_was() {
    let dir = scratch("bad");
    let file = dir.join("home/.claude.json");
    std::fs::write(&file, "{ not json").expect("write");
    let out = trust(&dir, false, &["claude", &folder(&dir)]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains(&file.display().to_string()), "{err}");
    assert_eq!(err.trim().lines().count(), 1, "one line: {err}");
    assert_eq!(std::fs::read_to_string(&file).expect("read"), "{ not json");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_runtime_the_build_lacks_or_a_missing_argument_exits_nonzero_saying_why() {
    let dir = scratch("args");
    let out = trust(&dir, false, &["nonesuch", &folder(&dir)]);
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("nonesuch") && err.contains("claude"), "{err}");
    let out = trust(&dir, false, &["claude"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("RUNTIME PATH"));
    assert!(!dir.join("home/.claude.json").exists());
    let _ = std::fs::remove_dir_all(&dir);
}
