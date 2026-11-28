//! What `temet-work` answers itself, without a daemon, in every build.

use std::process::Command;

/// The built `temet-work` with the daemon's socket pointing at nothing.
fn temet_work(args: &[&str]) -> Command {
    let socket = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("no-daemon.sock");
    let mut command = Command::new(env!("CARGO_BIN_EXE_temet-work"));
    command.args(args).env("TEMET_WORK_SOCKET", socket);
    command
}

fn run(arg: &str) -> std::process::Output {
    temet_work(&[arg]).output().expect("runs")
}

/// Every way to make `temet-work` print without a daemon.
const PRINTING: [&[&str]; 4] = [&[], &["help"], &["parts"], &["--version"]];

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

#[test]
fn a_gone_reader_is_a_quiet_success() {
    for args in PRINTING {
        let (reader, writer) = std::io::pipe().expect("pipe");
        drop(reader);
        let out = temet_work(args).stdout(writer).output().expect("runs");
        assert_eq!(out.status.code(), Some(0), "{args:?}: {out:?}");
        assert_eq!(String::from_utf8_lossy(&out.stderr), "", "{args:?}");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn a_full_device_is_reported_and_exits_2() {
    for args in PRINTING {
        let full = std::fs::File::options()
            .write(true)
            .open("/dev/full")
            .expect("/dev/full");
        let out = temet_work(args).stdout(full).output().expect("runs");
        assert_eq!(out.status.code(), Some(2), "{args:?}: {out:?}");
        assert_eq!(
            String::from_utf8_lossy(&out.stderr),
            "standard output: No space left on device (os error 28)\n",
            "{args:?}"
        );
    }
}
