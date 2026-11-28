//! Plumbing for the installed app: its build id, the launcher entry that starts it, and the
//! build of the binary installed at the path it was started from. It draws nothing.

use std::path::Path;
use std::process::Command;

use gpui::SharedString;

use crate::components::Intent;
use crate::pages::project::Notice;

/// The app's version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The app's build id: its checkout's commit, as `git describe --always --dirty` gave it.
pub const BUILD: &str = env!("TEMET_WORK_APP_BUILD");

/// What `--version` prints.
pub fn version_line() -> String {
    format!("temet-work-app {VERSION} ({BUILD})")
}

/// The launcher entry for the app installed at `exe`, as `--desktop-entry` prints it. Its
/// class matches the window's app id, so the compositor and the launcher take them as one.
pub fn desktop_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=temet-work\n\
         Comment=Issues, agents and decisions from the temet-work daemon\n\
         Exec={}\n\
         Terminal=false\n\
         StartupWMClass=temet-work\n\
         Categories=Development;\n",
        exe.display()
    )
}

/// The build of the binary at `exe` now, as its `--build` prints it; `None` when it can't be
/// run or prints nothing.
pub fn installed_build(exe: &Path) -> Option<String> {
    let out = Command::new(exe).arg("--build").output().ok()?;
    let build = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (out.status.success() && !build.is_empty()).then_some(build)
}

/// The notice the window shows once a build other than its own is installed where it was
/// started from.
pub fn newer_build_notice(installed: &str, running: &str) -> Notice {
    Notice {
        intent: Intent::Primary,
        title: SharedString::from("A newer build is installed"),
        lines: vec![SharedString::from(format!(
            "Installed: {installed}. Running: {running}. Restart temet-work to use it."
        ))],
    }
}

/// The binary this process was started from. Once an install replaces it, Linux names the
/// running one with " (deleted)" at the end; the path it came from is the one to read.
pub fn started_from() -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let text = exe.to_string_lossy();
    Some(match text.strip_suffix(" (deleted)") {
        Some(path) => path.into(),
        None => exe,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newer_build_notice_names_both_builds() {
        let n = newer_build_notice("b3d4b13", "aa86166");
        assert_eq!(n.title.as_ref(), "A newer build is installed");
        assert_eq!(
            n.lines[0].as_ref(),
            "Installed: b3d4b13. Running: aa86166. Restart temet-work to use it."
        );
    }

    #[test]
    fn the_desktop_entry_starts_the_installed_app_and_matches_the_window() {
        let entry = desktop_entry(Path::new("/home/vimes/.local/bin/temet-work-app"));
        assert!(entry.starts_with("[Desktop Entry]\n"), "{entry}");
        for line in [
            "Type=Application",
            "Name=temet-work",
            "Exec=/home/vimes/.local/bin/temet-work-app",
            "Terminal=false",
            "StartupWMClass=temet-work",
        ] {
            assert!(entry.lines().any(|l| l == line), "no {line:?} in {entry}");
        }
    }
}
