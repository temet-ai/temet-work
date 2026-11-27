//! Where the daemon listens, as a rule over the environment. Reading the
//! environment is left to the caller.

use std::ffi::OsString;
use std::path::PathBuf;

/// The daemon's socket, from the values of `TEMET_WORK_SOCKET`,
/// `XDG_RUNTIME_DIR` and `HOME`, each `None` when unset; an empty value counts
/// as unset. `TEMET_WORK_SOCKET` wins, else it is
/// `$XDG_RUNTIME_DIR/temet-work/daemon.sock`, else
/// `$HOME/.local/state/temet-work/daemon.sock`, under the user's own state
/// folder. Never the shared temp folder, where another user could make the path
/// first.
pub fn path(
    socket: Option<OsString>,
    runtime_dir: Option<OsString>,
    home: Option<OsString>,
) -> PathBuf {
    let set = |value: Option<OsString>| value.filter(|v| !v.is_empty()).map(PathBuf::from);
    if let Some(path) = set(socket) {
        return path;
    }
    let dir =
        set(runtime_dir).unwrap_or_else(|| set(home).unwrap_or_default().join(".local/state"));
    dir.join("temet-work").join("daemon.sock")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(socket: &str, runtime_dir: &str, home: &str) -> PathBuf {
        path(
            Some(socket.into()),
            Some(runtime_dir.into()),
            Some(home.into()),
        )
    }

    #[test]
    fn temet_work_socket_wins() {
        assert_eq!(
            at("/run/clacks.sock", "/run/user/7", "/home/vimes"),
            PathBuf::from("/run/clacks.sock")
        );
        // Empty, it is as if unset.
        assert_eq!(
            at("", "/run/user/7", "/home/vimes"),
            path(None, Some("/run/user/7".into()), Some("/home/vimes".into()))
        );
    }

    #[test]
    fn else_it_is_under_the_runtime_folder() {
        let want = PathBuf::from("/run/user/7/temet-work/daemon.sock");
        assert_eq!(
            path(None, Some("/run/user/7".into()), Some("/home/vimes".into())),
            want
        );
        // Empty, it is as if unset.
        assert_eq!(
            at("", "", "/home/vimes"),
            path(None, None, Some("/home/vimes".into()))
        );
    }

    #[test]
    fn else_it_is_under_the_users_state_folder() {
        assert_eq!(
            path(None, None, Some("/home/vimes".into())),
            PathBuf::from("/home/vimes/.local/state/temet-work/daemon.sock")
        );
        // With no home either, empty or unset, the path is relative.
        let relative = PathBuf::from(".local/state/temet-work/daemon.sock");
        assert_eq!(at("", "", ""), relative);
        assert_eq!(path(None, None, None), relative);
    }
}
