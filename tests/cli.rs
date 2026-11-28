//! What `temet-work` answers itself, without a daemon, in every build.

use std::process::Command;

/// The built `temet-work` with the daemon's socket pointing at nothing, and no
/// desktop, so `temet-work` alone lists the commands instead of opening a window.
fn temet_work(args: &[&str]) -> Command {
    let socket = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("no-daemon.sock");
    let mut command = Command::new(env!("CARGO_BIN_EXE_temet-work"));
    command
        .args(args)
        .env("TEMET_WORK_SOCKET", socket)
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("DISPLAY")
        .env("SSH_CONNECTION", "192.0.2.1 22 192.0.2.2 22");
    command
}

/// `temet-work` alone at a desktop, with `app` as the app's program, as a person.
fn at_a_desktop(app: &std::path::Path) -> Command {
    let mut command = temet_work(&[]);
    command
        .env("WAYLAND_DISPLAY", "wayland-1")
        .env_remove("SSH_CONNECTION")
        .env_remove("TEMET_WORK_AGENT")
        .env("TEMET_WORK_APP", app);
    command
}

/// A stand-in for the app that writes its own process group into `opened`.
fn stand_in_app(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a folder");
    let app = dir.join("app");
    let opened = dir.join("opened");
    std::fs::write(
        &app,
        format!(
            "#!/bin/sh\nps -o pgid= -p $$ > {}.part && mv {0}.part {0}\n",
            opened.display()
        ),
    )
    .expect("the stand-in");
    std::fs::set_permissions(&app, std::os::unix::fs::PermissionsExt::from_mode(0o755))
        .expect("executable");
    (app, opened)
}

/// What the stand-in wrote, once it has, within a few seconds.
fn read_when_written(path: &std::path::Path) -> String {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Ok(text) = std::fs::read_to_string(path) {
            return text;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the app was not opened"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn alone_at_a_desktop_it_opens_the_app_in_a_process_group_of_its_own() {
    let (app, opened) = stand_in_app("cli-window");
    let out = at_a_desktop(&app).output().expect("runs");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "opening the temet-work window; temet-work help lists the commands\n"
    );
    let group: u32 = read_when_written(&opened).trim().parse().expect("a group");
    let own: u32 = String::from_utf8_lossy(
        &Command::new("ps")
            .args(["-o", "pgid=", "-p", &std::process::id().to_string()])
            .output()
            .expect("ps")
            .stdout,
    )
    .trim()
    .parse()
    .expect("this test's group");
    assert_ne!(group, own, "the app left in this terminal's process group");
}

#[test]
fn alone_it_lists_the_commands_to_an_agent_and_without_a_desktop() {
    let (app, opened) = stand_in_app("cli-no-window");
    let agent = at_a_desktop(&app)
        .env("TEMET_WORK_AGENT", "refiner-1")
        .output()
        .expect("runs");
    let no_desktop = temet_work(&[])
        .env("TEMET_WORK_APP", &app)
        .output()
        .expect("runs");
    let usage = run("help").stdout;
    for out in [agent, no_desktop] {
        assert!(out.status.success(), "{out:?}");
        assert_eq!(out.stdout, usage);
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(!opened.exists(), "a window was opened");
}

#[test]
fn alone_at_a_desktop_without_the_app_it_lists_the_commands() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cli-no-app");
    let out = at_a_desktop(&dir.join("nonesuch")).output().expect("runs");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(out.stdout, run("help").stdout);
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

#[test]
fn help_lists_the_reads_and_a_read_it_cannot_send_exits_2() {
    let out = run("help");
    let text = String::from_utf8_lossy(&out.stdout);
    for form in [
        "temet-work change ISSUE [PROJECT] [--by-commit]",
        "temet-work show PROJECT PATH [--at ISSUE|COMMIT]",
        "temet-work search PROJECT PATTERN [--at ISSUE|COMMIT]",
        "temet-work ci ISSUE",
    ] {
        assert!(
            text.lines().any(|l| l.starts_with(form)),
            "{form} in:\n{text}"
        );
    }
    // Not understood, and understood with no daemon to answer: both exit 2.
    for args in [
        &["show"][..],
        &["ci", "watch"][..],
        &["change", "watch#12"][..],
    ] {
        let out = temet_work(args).output().expect("runs");
        assert_eq!(out.status.code(), Some(2), "{args:?}: {out:?}");
    }
}
