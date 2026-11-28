//! One shared copy of each repository, for reading a change, a file or a search
//! without a checkout. A copy is a bare git repository: it has no working tree
//! and no build folder. The family keeps every copy in one folder beside the
//! daemon's event log, one per code host and repository, and every reader of a
//! repository uses its one copy: the first read clones it, later reads fetch
//! what they need. A read at a pull request's head or at the default branch's
//! head fetches that head first, so it shows a commit pushed just before it; a
//! read at a commit the copy holds fetches nothing.

use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How long a clone may take before it is stopped.
const CLONE_WITHIN: Duration = Duration::from_secs(600);

/// How long a fetch may take before it is stopped.
const FETCH_WITHIN: Duration = Duration::from_secs(120);

/// How long any other git command may take before it is stopped.
const READ_WITHIN: Duration = Duration::from_secs(60);

/// Where a read looks in a repository.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Point {
    /// The head of the default branch.
    Default,
    /// The head of a pull request: its number and the ref its code host keeps it
    /// under.
    Pull {
        /// Its number.
        number: String,
        /// Its ref, such as `refs/pull/57/head`.
        head: String,
    },
    /// A commit, by its id or the start of it.
    Commit(String),
}

/// The repositories' copies, shared by every read. Reads of one repository take
/// turns; reads of different ones run side by side.
#[derive(Debug)]
pub struct Copies {
    root: PathBuf,
    locks: Mutex<BTreeMap<PathBuf, Arc<Mutex<()>>>>,
    /// The arguments of every git command run, in order, so a test can count
    /// the clones and fetches.
    pub ran: Arc<Mutex<Vec<Vec<String>>>>,
}

/// One repository's copy, for one read ([`Copies::with`]).
pub struct Copy<'a> {
    copies: &'a Copies,
    dir: PathBuf,
    url: String,
}

/// How a git command ended: its exit code (none when a signal stopped it) and
/// what it printed.
pub struct Exit {
    /// Its exit code; 0 when it succeeded.
    pub code: Option<i32>,
    /// What it printed on standard output.
    pub out: Vec<u8>,
    /// What it printed on standard error.
    pub err: String,
}

/// What a git command printed, and whether it succeeded.
struct Ran {
    ok: bool,
    code: Option<i32>,
    out: Vec<u8>,
    err: String,
}

/// Whether `url` is a folder on this machine, which a test's repository is.
fn local(url: &str) -> bool {
    url.starts_with('/') || url.starts_with("file://")
}

impl Copies {
    /// Copies kept under `root`, made when the first read needs it.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            locks: Mutex::new(BTreeMap::new()),
            ran: Arc::default(),
        }
    }

    /// Where the copies are kept.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Runs `read` on the copy of `repo` on code host `host`, cloned from
    /// `url` first when there is none yet. No other read of that repository runs
    /// meanwhile.
    pub fn with<T>(
        &self,
        host: &str,
        repo: &str,
        url: &str,
        read: impl FnOnce(&Copy<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut dir = self.root.join(host);
        for part in repo.split('/') {
            if part.is_empty() || part == "." || part == ".." {
                return Err(format!(
                    "{repo} is not a repository name a copy can be kept under"
                ));
            }
            dir.push(part);
        }
        dir.set_extension("git");
        let lock = self
            .locks
            .lock()
            .map_err(|_| "the copies' lock is poisoned")?
            .entry(dir.clone())
            .or_default()
            .clone();
        let _turn = lock
            .lock()
            .map_err(|_| format!("the lock of {} is poisoned", dir.display()))?;
        let copy = Copy {
            copies: self,
            dir,
            url: url.to_string(),
        };
        if !copy.dir.join("HEAD").is_file() {
            copy.clone_it()?;
        }
        read(&copy)
    }

    /// Runs git with `args`, in `dir` when given, stopped after `within`.
    fn git(&self, dir: Option<&Path>, args: &[&str], within: Duration) -> Result<Ran, String> {
        if let Ok(mut ran) = self.ran.lock() {
            ran.push(args.iter().map(ToString::to_string).collect());
        }
        let mut cmd = Command::new("git");
        if let Some(dir) = dir {
            cmd.arg("-C").arg(dir);
        }
        // The same output whatever the person's own git settings say.
        for setting in [
            "core.quotePath=off",
            "color.ui=false",
            "diff.mnemonicPrefix=false",
            "diff.noprefix=false",
            "log.showSignature=false",
        ] {
            cmd.args(["-c", setting]);
        }
        cmd.args(args)
            // No prompt for a sign-in: the daemon has no terminal to answer it.
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| format!("cannot run git: {e}"))?;
        let drain = |pipe: Option<Box<dyn std::io::Read + Send>>| {
            std::thread::spawn(move || {
                let mut bytes = Vec::new();
                if let Some(mut p) = pipe {
                    let _ = p.read_to_end(&mut bytes);
                }
                bytes
            })
        };
        let out = drain(child.stdout.take().map(|p| Box::new(p) as _));
        let err = drain(child.stderr.take().map(|p| Box::new(p) as _));
        let until = Instant::now() + within;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < until => std::thread::sleep(Duration::from_millis(5)),
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "git {} took longer than {} s and was stopped",
                        args.first().copied().unwrap_or_default(),
                        within.as_secs()
                    ));
                }
                Err(e) => return Err(format!("git: {e}")),
            }
        };
        let out = out.join().unwrap_or_default();
        let err = String::from_utf8_lossy(&err.join().unwrap_or_default())
            .trim()
            .to_string();
        Ok(Ran {
            ok: status.success(),
            code: status.code(),
            out,
            err,
        })
    }
}

impl Copy<'_> {
    /// Where the copy is.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Refuses an address on the network while tests run, so no test reaches a
    /// code host.
    fn may_reach(&self) -> Result<(), String> {
        if !local(&self.url)
            && std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty())
        {
            return Err(format!(
                "TEMET_WORK_SANDBOX is set: {} is not fetched",
                self.url
            ));
        }
        Ok(())
    }

    fn clone_it(&self) -> Result<(), String> {
        self.may_reach()?;
        let parent = self.dir.parent().unwrap_or(&self.dir);
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        // Cloned beside it and moved into place, so a clone that stops halfway
        // leaves no copy that looks whole.
        let part = self.dir.with_extension("git.part");
        let _ = std::fs::remove_dir_all(&part);
        let ran = self.copies.git(
            None,
            &[
                "clone",
                "--bare",
                "--quiet",
                "--no-tags",
                "--",
                &self.url,
                &part.to_string_lossy(),
            ],
            CLONE_WITHIN,
        )?;
        if !ran.ok {
            let _ = std::fs::remove_dir_all(&part);
            return Err(format!("cannot clone {}: {}", self.url, ran.err));
        }
        std::fs::rename(&part, &self.dir).map_err(|e| format!("{}: {e}", self.dir.display()))
    }

    /// Runs git in the copy.
    fn git(&self, args: &[&str], within: Duration) -> Result<Ran, String> {
        self.copies.git(Some(&self.dir), args, within)
    }

    fn fetch(&self, refspec: &str) -> Result<(), String> {
        self.may_reach()?;
        let ran = self.git(
            &["fetch", "--quiet", "--no-tags", "--", &self.url, refspec],
            FETCH_WITHIN,
        )?;
        if ran.ok {
            Ok(())
        } else {
            Err(format!("cannot fetch from {}: {}", self.url, ran.err))
        }
    }

    /// The commit `name` names in the copy, or none.
    fn commit(&self, name: &str) -> Result<Option<String>, String> {
        let spec = format!("{name}^{{commit}}");
        let ran = self.git(
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                "--end-of-options",
                &spec,
            ],
            READ_WITHIN,
        )?;
        Ok(ran
            .ok
            .then(|| String::from_utf8_lossy(&ran.out).trim().to_string())
            .filter(|s| !s.is_empty()))
    }

    /// The commit at `point`, fetched first for a head. `what` names the
    /// repository in a refusal.
    pub fn resolve(&self, point: &Point, what: &str) -> Result<String, String> {
        match point {
            Point::Default => {
                self.fetch("+HEAD:refs/temet-work/default")?;
                self.commit("refs/temet-work/default")?
                    .ok_or_else(|| format!("{what} has no default branch"))
            }
            Point::Pull { number, head } => {
                let to = format!("refs/temet-work/pull/{number}");
                self.fetch(&format!("+{head}:{to}"))
                    .map_err(|e| format!("{what} has no pull request {number} to read ({e})"))?;
                self.commit(&to)?
                    .ok_or_else(|| format!("{what} has no pull request {number} to read"))
            }
            Point::Commit(id) => {
                if id.len() < 4 || id.len() > 64 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(format!(
                        "{id} is not a commit id: give its hexadecimal id or at least its first 4 \
                         characters, an issue as project#key for its pull request's head, or \
                         leave --at out for the default branch's head"
                    ));
                }
                if let Some(found) = self.commit(id)? {
                    return Ok(found);
                }
                // A commit made since the copy last fetched: every branch, then
                // look again.
                self.fetch("+refs/heads/*:refs/heads/*")?;
                self.commit(id)?
                    .ok_or_else(|| format!("{what} has no commit {id}"))
            }
        }
    }

    /// Runs git in the copy for a read, and gives what it printed; an error with
    /// git's reason when it fails.
    pub fn read(&self, args: &[&str]) -> Result<Vec<u8>, String> {
        let ran = self.git(args, READ_WITHIN)?;
        if ran.ok { Ok(ran.out) } else { Err(ran.err) }
    }

    /// Like [`Copy::read`], but a git that ran and failed is no error: what it
    /// printed comes back with its exit code, for a command whose code says
    /// something, such as `grep` finding nothing.
    pub fn read_with_code(&self, args: &[&str]) -> Result<Exit, String> {
        let ran = self.git(args, READ_WITHIN)?;
        Ok(Exit {
            code: if ran.ok { Some(0) } else { ran.code },
            out: ran.out,
            err: ran.err,
        })
    }
}
