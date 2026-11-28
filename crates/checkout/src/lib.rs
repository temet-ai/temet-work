//! The checkout part: it carries out the loop's requests to make and remove a
//! reader's own checkout, and reports what it did as events.
//!
//! A checkout is a git worktree of a work folder's repository, detached at the
//! fetched head of a pull request's branch or of the default branch: detached,
//! since a branch already checked out in a work folder cannot be checked out in
//! a second worktree. Adding one writes only an entry under the work folder's
//! `.git` and a fetched remote-tracking branch there; it changes none of the
//! folder's files. A checkout is removed with the files git ignores in it, such
//! as a build folder under `target/`. One with an uncommitted change, a file git
//! does not ignore, or a commit that no remote-tracking branch holds is kept:
//! nothing in it is deleted, and it is reported kept with what was found.
//!
//! A checkout on another machine is worked on through `ssh MACHINE`, which
//! `docs/configuration.md` already requires to work from the daemon's machine.
//! It runs `git`, `test` and `rmdir`, and no other tool, so it has no adapters.

use std::path::Path;

use temet_work_core::model::Checkout;
use temet_work_core::world::LOCAL;
use temet_work_core::{Effect, Event, Outcome, Part, World};

/// The remote a checkout is fetched from: the one `git clone` names.
pub const REMOTE: &str = "origin";

/// What a command printed, and whether it succeeded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ran {
    /// Whether it exited with 0.
    pub ok: bool,
    /// What it wrote to standard output.
    pub out: String,
    /// What it wrote to standard error.
    pub err: String,
}

/// Runs a command on a machine: this one when `machine` is none, else the one
/// named. An error means the command could not be run there at all, such as
/// `ssh` failing to connect; a command that ran and failed is a [`Ran`] that is
/// not `ok`.
pub trait Shell: Send {
    /// Runs `argv` on `machine`.
    fn run(&mut self, machine: Option<&str>, argv: &[String]) -> Result<Ran, String>;
}

/// The command line that runs `argv` on `machine`: `argv` itself here, and
/// `ssh MACHINE` with `argv` quoted for the shell there on another machine.
pub fn command_line(machine: Option<&str>, argv: &[String]) -> Vec<String> {
    match machine {
        None => argv.to_vec(),
        Some(m) => {
            let quoted: Vec<String> = argv.iter().map(|a| quote(a)).collect();
            vec!["ssh".into(), m.into(), quoted.join(" ")]
        }
    }
}

/// `arg` quoted for a POSIX shell, unless it needs no quoting.
fn quote(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./=:+@%,".contains(c));
    if plain {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

/// The shell that runs commands for real: here, or through `ssh`. `ssh` exits
/// with 255 when it cannot reach the machine, which is an error, not a result.
#[derive(Clone, Debug, Default)]
pub struct Real {
    env: Vec<(String, String)>,
}

impl Real {
    /// A shell that adds `env` to each command's environment on this machine,
    /// such as the `GIT_CONFIG_*` variables a test sets.
    pub fn with_env(env: Vec<(String, String)>) -> Self {
        Self { env }
    }
}

impl Shell for Real {
    fn run(&mut self, machine: Option<&str>, argv: &[String]) -> Result<Ran, String> {
        let line = command_line(machine, argv);
        let (program, args) = line.split_first().ok_or("no command")?;
        let out = std::process::Command::new(program)
            .args(args)
            .envs(self.env.iter().map(|(k, v)| (k, v)))
            .stdin(std::process::Stdio::null())
            .output()
            .map_err(|e| format!("cannot run {program}: {e}"))?;
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if let Some(m) = machine
            && out.status.code() == Some(255)
        {
            return Err(format!("ssh {m} failed: {err}"));
        }
        Ok(Ran {
            ok: out.status.success(),
            out: String::from_utf8_lossy(&out.stdout).into_owned(),
            err,
        })
    }
}

/// What a checkout's folder holds when the part looks at it.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Found {
    /// Nothing is at its path.
    Gone,
    /// A git worktree with no change in it: it may be removed.
    Clean,
    /// Something a person has to look at, in a few words each: changes in a
    /// worktree, or a folder that is not one.
    Changes(Vec<String>),
}

/// The checkout part.
pub struct Checkouts {
    shell: Box<dyn Shell>,
}

impl Default for Checkouts {
    fn default() -> Self {
        Self::new(Box::new(Real::default()))
    }
}

impl Checkouts {
    /// The part, running its commands through `shell`.
    pub fn new(shell: Box<dyn Shell>) -> Self {
        Self { shell }
    }

    /// Runs `argv` where `c` is, and fails with what it printed when it fails.
    fn git(&mut self, c: &Checkout, argv: &[&str]) -> Result<String, String> {
        let argv: Vec<String> = argv.iter().map(ToString::to_string).collect();
        let ran = self.shell.run(machine(c), &argv)?;
        if ran.ok {
            Ok(ran.out)
        } else {
            let why = if ran.err.is_empty() {
                ran.out.trim().to_string()
            } else {
                ran.err
            };
            Err(format!("{} failed: {why}", argv.join(" ")))
        }
    }

    /// Whether `argv` succeeds where `c` is.
    fn holds(&mut self, c: &Checkout, argv: &[&str]) -> Result<bool, String> {
        let argv: Vec<String> = argv.iter().map(ToString::to_string).collect();
        Ok(self.shell.run(machine(c), &argv)?.ok)
    }

    /// Makes `c`: fetches the branch it is made at, or the default branch, into
    /// the work folder's repository, and adds a detached worktree at its head.
    /// A clean worktree of ours already at its path, left by a crash before it
    /// was recorded, is removed first; anything else there is kept for a person,
    /// and the next try takes another number.
    fn add(&mut self, c: &Checkout, then: &Effect) -> Result<Outcome, String> {
        let branch = match &c.branch {
            Some(b) => b.clone(),
            None => self.default_branch(c)?,
        };
        let refspec = format!("+refs/heads/{branch}:refs/remotes/{REMOTE}/{branch}");
        self.git(
            c,
            &["git", "-C", &c.from, "fetch", "--quiet", REMOTE, &refspec],
        )?;
        match self.look(c)? {
            Found::Gone => {}
            Found::Clean => self.remove_worktree(c)?,
            Found::Changes(changes) => {
                return Ok(Outcome::Done {
                    events: vec![added(c), kept(c, changes)],
                    effects: Vec::new(),
                });
            }
        }
        let head = format!("refs/remotes/{REMOTE}/{branch}");
        self.git(
            c,
            &[
                "git", "-C", &c.from, "worktree", "add", "--quiet", "--detach", &c.path, &head,
            ],
        )?;
        Ok(Outcome::Done {
            events: vec![added(c)],
            effects: vec![then.clone()],
        })
    }

    /// The default branch of the remote, as it names it now.
    fn default_branch(&mut self, c: &Checkout) -> Result<String, String> {
        let out = self.git(
            c,
            &[
                "git",
                "-C",
                &c.from,
                "ls-remote",
                "--symref",
                REMOTE,
                "HEAD",
            ],
        )?;
        out.lines()
            .find_map(|l| l.strip_prefix("ref: refs/heads/"))
            .and_then(|l| l.split_whitespace().next())
            .map(String::from)
            .ok_or_else(|| format!("{REMOTE} of {} names no default branch", c.from))
    }

    /// Removes `c` unless it has changes in it: then nothing in it is deleted,
    /// and it is reported kept. One already gone is reported removed.
    fn remove(&mut self, c: &Checkout) -> Result<Vec<Event>, String> {
        match self.look(c)? {
            Found::Gone => {
                // Its entry under the work folder's .git goes too; a work folder
                // that has gone as well has nothing to prune.
                let _ = self.holds(c, &["git", "-C", &c.from, "worktree", "prune"]);
            }
            Found::Clean => self.remove_worktree(c)?,
            Found::Changes(changes) => return Ok(vec![kept(c, changes)]),
        }
        Ok(vec![Event::CheckoutRemoved {
            name: c.name.clone(),
        }])
    }

    /// Removes the clean worktree at `c`'s path with the files git ignores in
    /// it, then the folders of its log and of the checkouts once they are
    /// empty. `git worktree remove` refuses a worktree with an uncommitted
    /// change or a file it does not ignore, so git checks that too.
    fn remove_worktree(&mut self, c: &Checkout) -> Result<(), String> {
        self.git(c, &["git", "-C", &c.path, "worktree", "remove", "."])?;
        let log = Path::new(&c.path).parent();
        for dir in [log, log.and_then(Path::parent)].into_iter().flatten() {
            if let Some(dir) = dir.to_str().filter(|d| !d.is_empty()) {
                // Only an empty folder goes; one that still holds another
                // checkout stays.
                let _ = self.holds(c, &["rmdir", dir]);
            }
        }
        Ok(())
    }

    /// What is at `c`'s path: nothing, a clean worktree, or something a person
    /// has to look at.
    fn look(&mut self, c: &Checkout) -> Result<Found, String> {
        let path = c.path.as_str();
        if !self.holds(c, &["test", "-e", path])? {
            return Ok(Found::Gone);
        }
        // A linked worktree has a .git file that points at its repository.
        let dot_git = format!("{path}/.git");
        if !self.holds(c, &["test", "-f", &dot_git])? {
            return Ok(Found::Changes(vec![
                "it is not a git worktree, so nothing in it was looked at".into(),
            ]));
        }
        let mut changes = Vec::new();
        let status = self.git(
            c,
            &[
                "git",
                "-C",
                path,
                "status",
                "--porcelain",
                "--untracked-files=all",
            ],
        )?;
        let paths: Vec<&str> = status
            .lines()
            .filter_map(|l| l.get(3..))
            .filter(|p| !p.is_empty())
            .collect();
        if !paths.is_empty() {
            let first: Vec<&str> = paths.iter().take(3).copied().collect();
            let more = if paths.len() > first.len() {
                ", ..."
            } else {
                ""
            };
            changes.push(format!(
                "{} {} changed, or new and not ignored: {}{more}",
                paths.len(),
                if paths.len() == 1 { "file" } else { "files" },
                first.join(", ")
            ));
        }
        let unheld = self.unheld_commits(c)?;
        if unheld > 0 {
            changes.push(format!(
                "{unheld} {} no remote-tracking branch holds",
                if unheld == 1 { "commit" } else { "commits" }
            ));
        }
        Ok(if changes.is_empty() {
            Found::Clean
        } else {
            Found::Changes(changes)
        })
    }

    /// How many commits made in `c`, by its own `HEAD`'s reflog, no
    /// remote-tracking branch holds. Commits fetched into it are not counted,
    /// so one a squash merge and a pruned branch left behind does not keep a
    /// checkout. With no reflog to read, every commit `HEAD` reaches that no
    /// remote-tracking branch holds counts.
    fn unheld_commits(&mut self, c: &Checkout) -> Result<usize, String> {
        let path = c.path.as_str();
        let made: Vec<String> = match self.git(
            c,
            &["git", "-C", path, "log", "-g", "--format=%H %gs", "HEAD"],
        ) {
            Ok(log) => log
                .lines()
                .filter_map(|l| l.split_once(' '))
                .filter(|(_, subject)| made_here(subject))
                .map(|(sha, _)| sha.to_string())
                .collect(),
            Err(_) => vec!["HEAD".to_string()],
        };
        if made.is_empty() {
            return Ok(0);
        }
        let mut argv = vec!["git", "-C", path, "rev-list", "--count"];
        argv.extend(made.iter().map(String::as_str));
        argv.extend(["--not", "--remotes"]);
        let count = self.git(c, &argv)?;
        count
            .trim()
            .parse()
            .map_err(|e| format!("git rev-list gave {count:?}: {e}"))
    }
}

/// Whether a reflog entry's subject records a commit made where the reflog is,
/// as opposed to a move to a commit that came from elsewhere.
fn made_here(subject: &str) -> bool {
    [
        "commit",
        "merge ",
        "cherry-pick",
        "revert",
        "rebase (pick)",
        "rebase (reword)",
        "rebase (squash)",
        "rebase (fixup)",
        "rebase (edit)",
        "am:",
    ]
    .iter()
    .any(|p| subject.starts_with(p))
}

/// The machine to run `c`'s commands on: none for this one.
fn machine(c: &Checkout) -> Option<&str> {
    (c.machine.0 != LOCAL).then_some(c.machine.0.as_str())
}

fn added(c: &Checkout) -> Event {
    Event::CheckoutAdded {
        checkout: Checkout {
            kept: None,
            ..c.clone()
        },
    }
}

fn kept(c: &Checkout, changes: Vec<String>) -> Event {
    Event::CheckoutKept {
        name: c.name.clone(),
        changes,
    }
}

impl Part for Checkouts {
    fn id(&self) -> &'static str {
        "checkout"
    }

    fn carry_out(&mut self, effect: &Effect, _world: &World) -> Outcome {
        let done = match effect {
            Effect::AddCheckout { checkout, then } => self.add(checkout, then),
            Effect::RemoveCheckout { checkout } => {
                self.remove(checkout).map(|events| Outcome::Done {
                    events,
                    effects: Vec::new(),
                })
            }
            _ => return Outcome::NotMine,
        };
        done.unwrap_or_else(Outcome::Failed)
    }
}

/// A checkout part that touches no disk: it reports each checkout made and
/// removed as asked, for tests of the daemon with every family's fake.
#[derive(Clone, Copy, Debug, Default)]
pub struct FakeCheckouts;

impl Part for FakeCheckouts {
    fn id(&self) -> &'static str {
        "checkout"
    }

    fn carry_out(&mut self, effect: &Effect, _world: &World) -> Outcome {
        match effect {
            Effect::AddCheckout { checkout, then } => Outcome::Done {
                events: vec![added(checkout)],
                effects: vec![(**then).clone()],
            },
            Effect::RemoveCheckout { checkout } => Outcome::Done {
                events: vec![Event::CheckoutRemoved {
                    name: checkout.name.clone(),
                }],
                effects: Vec::new(),
            },
            _ => Outcome::NotMine,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use temet_work_core::ids::{AgentId, MachineId, ProjectId};

    /// A shell that answers every command with success and records it.
    #[derive(Clone, Default)]
    struct Recording(Arc<Mutex<Vec<Vec<String>>>>);

    impl Shell for Recording {
        fn run(&mut self, machine: Option<&str>, argv: &[String]) -> Result<Ran, String> {
            self.0
                .lock()
                .map_err(|e| e.to_string())?
                .push(command_line(machine, argv));
            // Nothing is at the checkout's path yet, and the remote names main.
            let ok = argv.first().is_none_or(|a| a != "test");
            Ok(Ran {
                ok,
                out: "ref: refs/heads/main\tHEAD\n".into(),
                err: String::new(),
            })
        }
    }

    fn on(machine: &str) -> Checkout {
        Checkout {
            name: "watch-1".into(),
            agent: AgentId::new("reviewer-1"),
            project: ProjectId::new("watch"),
            machine: MachineId::new(machine),
            path: "/Users/vimes/temet-work-checkouts/l1/watch-1".into(),
            from: "/Users/vimes/watch".into(),
            branch: Some("watch-12-night shift's".into()),
            kept: None,
        }
    }

    #[test]
    fn a_checkout_on_another_machine_runs_its_git_through_ssh() {
        let shell = Recording::default();
        let mut part = Checkouts::new(Box::new(shell.clone()));
        let then = Effect::Investigate {
            group: temet_work_core::ids::GroupId::new("watch"),
        };
        let effect = Effect::AddCheckout {
            checkout: on("mac"),
            then: Box::new(then.clone()),
        };
        let Outcome::Done { events, effects } = part.carry_out(&effect, &World::default()) else {
            panic!("not done");
        };
        assert_eq!(events, vec![added(&on("mac"))]);
        assert_eq!(effects, vec![then]);
        let ran = shell.0.lock().expect("ran").clone();
        assert_eq!(
            ran[0],
            [
                "ssh",
                "mac",
                "git -C /Users/vimes/watch fetch --quiet origin \
                 '+refs/heads/watch-12-night shift'\\''s:refs/remotes/origin/watch-12-night shift'\\''s'"
            ]
        );
        assert!(
            ran.iter().all(|c| c[0] == "ssh" && c[1] == "mac"),
            "{ran:?}"
        );
        assert!(
            ran.last().is_some_and(|c| c[2].starts_with(
                "git -C /Users/vimes/watch worktree add --quiet --detach \
                 /Users/vimes/temet-work-checkouts/l1/watch-1"
            )),
            "{ran:?}"
        );
        // One on this machine runs git itself.
        let shell = Recording::default();
        let mut part = Checkouts::new(Box::new(shell.clone()));
        let effect = Effect::AddCheckout {
            checkout: on(LOCAL),
            then: Box::new(Effect::Investigate {
                group: temet_work_core::ids::GroupId::new("watch"),
            }),
        };
        part.carry_out(&effect, &World::default());
        let ran = shell.0.lock().expect("ran").clone();
        assert_eq!(ran[0][..3], ["git", "-C", "/Users/vimes/watch"]);
    }

    #[test]
    fn only_reflog_entries_that_make_a_commit_count_as_made_here() {
        for subject in [
            "commit: mend the clock",
            "commit (amend): mend the clock",
            "merge watch-12: Merge made by the 'ort' strategy.",
            "cherry-pick: mend the clock",
            "rebase (pick): mend the clock",
        ] {
            assert!(made_here(subject), "{subject}");
        }
        for subject in [
            "",
            "checkout: moving from main to 1234abcd",
            "reset: moving to origin/watch-12",
            "rebase (start): checkout origin/main",
            "pull: Fast-forward",
        ] {
            assert!(!made_here(subject), "{subject}");
        }
    }
}
