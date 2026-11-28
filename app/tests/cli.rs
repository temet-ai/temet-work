//! The installed app's command line: it answers --version, --build and --desktop-entry and
//! exits, opening no window.

use std::process::Command;

fn run(arg: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_temet-work-app"))
        .arg(arg)
        .output()
        .expect("the app runs");
    assert!(out.status.success(), "{arg}: {out:?}");
    String::from_utf8(out.stdout).expect("text")
}

#[test]
fn version_names_the_build_and_build_prints_it_alone() {
    let build = run("--build").trim().to_string();
    assert!(!build.is_empty());
    assert_eq!(
        run("--version").trim(),
        format!("temet-work-app {} ({build})", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn the_desktop_entry_names_the_binary_it_came_from() {
    let entry = run("--desktop-entry");
    let exe = env!("CARGO_BIN_EXE_temet-work-app");
    assert!(entry.lines().any(|l| l == format!("Exec={exe}")), "{entry}");
}
