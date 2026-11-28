//! The daemon does not start with a setting in its file that names a runtime or
//! runner this build lacks, and says which ones it has. `--hold` starts it held.

#![cfg(all(
    feature = "runtime-claude",
    feature = "runtime-codex",
    feature = "runner-herdr"
))]

use std::process::Command;

fn start(name: &str, config: &str) -> std::process::Output {
    start_with(name, config, &[])
}

/// One round of the daemon with `flags`, on the configuration `config` and the
/// log in the folder `name`, which a later call with the same name reuses.
fn start_with(name: &str, config: &str, flags: &[&str]) -> std::process::Output {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&dir).expect("a folder");
    let file = dir.join("config.toml");
    std::fs::write(&file, config).expect("a config");
    Command::new(env!("CARGO_BIN_EXE_temet-workd"))
        .args(["--once", "--config"])
        .arg(&file)
        .arg("--log")
        .arg(dir.join("events.jsonl"))
        .args(flags)
        .env("TEMET_WORK_SANDBOX", "1")
        .output()
        .expect("runs")
}

/// The line `status` prints for ankh#1 after one round on a log that holds it
/// as a new issue, with a configuration whose group would take it, and `flags`.
#[cfg(feature = "store")]
fn ankh_1_after_a_round(name: &str, flags: &[&str]) -> String {
    use temet_work_core::Event;
    use temet_work_core::ids::IssueId;

    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a folder");
    let (mut log, _) = temet_work_store::Log::open(&dir.join("events.jsonl")).expect("a log");
    for event in [
        Event::LogStarted { id: "lamp".into() },
        Event::IssueSynced {
            issue: IssueId::new("ankh", "1"),
            title: "Mend the clock".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        },
    ] {
        log.append(1, &event).expect("logged");
    }
    // The daemon takes the log's lock.
    drop(log);
    let config = "[[project]]\nid = \"ankh\"\ntracker = \"github\"\nname = \"ankh-morpork/watch\"\n\
                  [[group]]\nid = \"watch\"\nname = \"Night watch\"\nprojects = [\"ankh\"]\n";
    let out = start_with(name, config, flags);
    assert!(out.status.success(), "{out:?}");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find(|l| l.contains("ankh#1"))
        .unwrap_or_else(|| panic!("no ankh#1 in {out:?}"))
        .to_string()
}

#[cfg(feature = "store")]
#[test]
fn hold_is_logged_before_the_configuration_places_work() {
    // Without the hold, the group the configuration adds takes the issue at once.
    let line = ankh_1_after_a_round("start-unheld-order", &[]);
    assert!(line.contains("refining"), "{line}");
    let line = ankh_1_after_a_round("start-hold-order", &["--hold"]);
    assert!(line.contains("new") && !line.contains("refining"), "{line}");
}

#[test]
fn hold_starts_the_daemon_held_and_its_log_keeps_one_hold_through_restarts() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("start-hold");
    let _ = std::fs::remove_dir_all(&dir);
    for flags in [&["--hold"][..], &[], &["--hold"]] {
        let out = start_with("start-hold", "", flags);
        assert!(out.status.success(), "{out:?}");
        let status = String::from_utf8_lossy(&out.stdout);
        assert!(status.starts_with("Held since "), "{flags:?}: {status}");
    }
    let log = std::fs::read_to_string(dir.join("events.jsonl")).expect("the log");
    assert_eq!(log.matches(r#""event":"Held""#).count(), 1, "{log}");
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

#[cfg(feature = "tracker-jira")]
#[test]
fn a_project_on_the_jira_tracker_loads_and_the_daemon_polls_it() {
    let config = "[[project]]\nid = \"eng\"\ntracker = \"jira\"\nname = \"ENG\"\n";
    let out = start("start-jira", config);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The sandbox is on, so Jira is not called; the poll's failure shows it was made.
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("TEMET_WORK_SANDBOX"), "{text}");
}
