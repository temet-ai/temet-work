//! What `temet-work` answers itself, without a daemon, in every build.

use std::process::Command;

/// Runs the built `temet-work` with the daemon's socket pointing at nothing.
fn run(arg: &str) -> std::process::Output {
    let socket = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("no-daemon.sock");
    Command::new(env!("CARGO_BIN_EXE_temet-work"))
        .arg(arg)
        .env("TEMET_WORK_SOCKET", socket)
        .output()
        .expect("runs")
}

#[test]
fn version_prints_the_version_and_build_without_a_daemon() {
    let out = run("--version");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stderr), "");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!(
            "temet-work {} ({})\n",
            env!("CARGO_PKG_VERSION"),
            temet_work::BUILD
        )
    );
}

#[test]
fn help_lists_version() {
    let out = run("help");
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.lines().any(|l| l.starts_with("temet-work --version")),
        "{text}"
    );
}
