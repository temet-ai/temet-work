//! Who is at the other end of the daemon's socket. On Linux the daemon reads the
//! sending process's id with `SO_PEERCRED`, then reads that process and each of
//! its parents in `/proc`, collecting the agent secrets in their environments. A
//! request from inside an agent's terminal is then known as one even when the
//! agent unset its variables in the process that sent it. Elsewhere (macOS)
//! nothing is read: `SO_PEERCRED` and `/proc` are Linux only.

use std::os::unix::net::UnixStream;
use std::path::Path;

use temet_work_core::launch::SECRET_VAR;
use temet_work_core::sender::{Digest, Peer};

/// The SHA-256 digest of a secret, the form the core compares.
pub fn digest(secret: &str) -> Digest {
    use sha2::Digest as _;
    Digest::from_bytes(&sha2::Sha256::digest(secret.as_bytes()))
}

/// The process at the other end of `stream`, and its parents.
#[cfg(target_os = "linux")]
pub fn of(stream: &UnixStream) -> Peer {
    match rustix::net::sockopt::socket_peercred(stream) {
        Ok(cred) => tree(cred.pid.as_raw_nonzero().get(), Path::new("/proc")),
        Err(e) => Peer::Unreadable(format!("SO_PEERCRED: {e}")),
    }
}

/// Not read: this platform has no `SO_PEERCRED` and no `/proc`.
#[cfg(not(target_os = "linux"))]
pub fn of(stream: &UnixStream) -> Peer {
    let _ = stream;
    Peer::Unchecked
}

/// How many parents are read at most; a real tree is a dozen deep.
const DEEPEST: usize = 256;

/// Reads process `pid` and its parents under `proc` (`/proc`): the digests of
/// every agent secret in their environments. The process itself must be
/// readable; a parent that is not, such as `sshd` or init running as another
/// user, is passed over, since agents run as the daemon's own user.
pub fn tree(pid: i32, proc: &Path) -> Peer {
    let mut found = Vec::new();
    let mut pid = pid;
    for depth in 0..DEEPEST {
        let dir = proc.join(pid.to_string());
        let parent = parent(&dir);
        match std::fs::read(dir.join("environ")) {
            Ok(env) => found.extend(secrets(&env).map(|s| digest(&s))),
            Err(e) if depth == 0 => return Peer::Unreadable(format!("process {pid}: {e}")),
            Err(_) => {}
        }
        match parent {
            Some(p) if p > 0 => pid = p,
            None if depth == 0 => {
                return Peer::Unreadable(format!("process {pid} has ended"));
            }
            _ => break,
        }
    }
    Peer::Read(found)
}

/// The values of [`SECRET_VAR`] in an environment as `/proc/PID/environ` holds it:
/// `NAME=value` entries, each ended by a zero byte.
fn secrets(environ: &[u8]) -> impl Iterator<Item = String> + '_ {
    let prefix = format!("{SECRET_VAR}=");
    environ.split(|b| *b == 0).filter_map(move |entry| {
        entry
            .strip_prefix(prefix.as_bytes())
            .filter(|v| !v.is_empty())
            .map(|v| String::from_utf8_lossy(v).into_owned())
    })
}

/// The parent of the process whose `/proc` folder is `dir`, from its `PPid:` line.
fn parent(dir: &Path) -> Option<i32> {
    let status = std::fs::read_to_string(dir.join("status")).ok()?;
    status
        .lines()
        .find_map(|l| l.strip_prefix("PPid:"))
        .and_then(|p| p.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A made-up `/proc` of processes `(pid, parent, environment)`.
    fn proc(name: &str, processes: &[(i32, i32, &str)]) -> PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/tmp")
            .join(format!("proc-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (pid, parent, env) in processes {
            let dir = root.join(pid.to_string());
            std::fs::create_dir_all(&dir).expect("dir");
            std::fs::write(dir.join("status"), format!("Name:\tsh\nPPid:\t{parent}\n"))
                .expect("status");
            if !env.is_empty() {
                std::fs::write(dir.join("environ"), env.replace(' ', "\0")).expect("environ");
            }
        }
        root
    }

    #[test]
    fn collects_every_secret_up_the_tree_and_passes_over_an_unreadable_parent() {
        let root = proc(
            "tree",
            &[
                (40, 30, "PATH=/bin HOME=/home/vimes"),
                (
                    30,
                    20,
                    "TEMET_WORK_AGENT_TOKEN=s1 TEMET_WORK_AGENT=author-1",
                ),
                // 20 has no environ file: another user's process.
                (20, 10, ""),
                (10, 0, "TEMET_WORK_AGENT_TOKEN=s2"),
            ],
        );
        let got = tree(40, &root);
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(got, Peer::Read(vec![digest("s1"), digest("s2")]));
    }

    #[test]
    fn a_sender_that_cannot_be_read_is_unreadable() {
        let root = proc("gone", &[(30, 20, "")]);
        let unreadable = tree(30, &root);
        let gone = tree(99, &root);
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            matches!(&unreadable, Peer::Unreadable(why) if why.starts_with("process 30:")),
            "{unreadable:?}"
        );
        assert!(matches!(gone, Peer::Unreadable(_)), "{gone:?}");
    }

    #[test]
    fn an_empty_secret_is_no_secret() {
        let found: Vec<String> =
            secrets(b"TEMET_WORK_AGENT_TOKEN=\0TEMET_WORK_AGENT_TOKENX=a\0").collect();
        assert!(found.is_empty(), "{found:?}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn this_test_reads_itself_through_a_socket_pair() {
        let (a, _b) = UnixStream::pair().expect("pair");
        assert!(matches!(of(&a), Peer::Read(_)), "{:?}", of(&a));
    }
}
