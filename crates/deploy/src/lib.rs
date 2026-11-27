//! Deploy and verify: after a merge, the project's deploy command; after a
//! deploy, its verify command. Each runs on its own thread, so a long deploy
//! does not hold up the daemon, and the result is reported when the part is
//! next polled. A project with no deploy command counts as deployed when it
//! merges, and one with no verify command as verified when it deploys.
//!
//! Commands run through `sh -c` in the project's `run_in` folder, with
//! `TEMET_WORK_ISSUE`, `TEMET_WORK_PROJECT` and `TEMET_WORK_REPO` set. While
//! `TEMET_WORK_SANDBOX` is set to anything but empty, as it is in every test
//! run, the real shell runs nothing.
//!
//! One deploy or verify runs at a time per project and per `run_in` folder: two
//! deploys would both pull and build in one checkout, and a verify compares the
//! installed build with that checkout's commit, which a deploy moves. A command
//! asked for while another of its project or folder runs, or while a deploy
//! there waits for its restart, is reported done without starting, and runs
//! when the loop asks again in a later round. Projects with different folders
//! run side by side.
//!
//! A deploy may restart the daemon that is running it (temet-work deploying
//! itself). Then the command builds and installs first, records the issue and
//! the build it installed as a line `ISSUE BUILD` in the file named by
//! `TEMET_WORK_HANDOFF` (next to the daemon's log), and ends by starting the
//! restart detached, so the thread running it is never what gets killed. The
//! old daemon does not record `Deployed` for such a deploy; the new daemon does,
//! when it starts with the build the file names. Until then the deploy counts as
//! pending and is not run again.
//!
//! A daemon that finds a line naming another build, with no deploy of its own
//! waiting for that issue, runs the deploy again. It first moves the line to the
//! stale file beside the handoff file, as `ISSUE BUILD RUNNING`, so the reruns
//! are counted across the restarts they cause: after three for one issue, the
//! deploy waits for `temet-work retry ISSUE`. A retry, a recorded `Deployed`, or
//! the issue leaving the deploying stage clears its stale lines.

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use temet_work_core::ids::{IssueId, ProjectId};
use temet_work_core::model::Stage;
use temet_work_core::{Effect, Event, Outcome, Part, Polled, World};

/// Runs one command in a folder with extra environment, and says whether it
/// succeeded; the error holds the end of what it printed.
pub type Shell = dyn Fn(&str, &PathBuf, &[(String, String)]) -> Result<(), String> + Send + Sync;

/// The real shell: `sh -c COMMAND`.
pub fn sh(command: &str, cwd: &PathBuf, env: &[(String, String)]) -> Result<(), String> {
    if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
        return Err("TEMET_WORK_SANDBOX is set: commands are not run".into());
    }
    run(command, cwd, env)
}

/// Runs `sh -c COMMAND` whatever the sandbox says; `sh` guards it.
fn run(command: &str, cwd: &PathBuf, env: &[(String, String)]) -> Result<(), String> {
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .envs(env.iter().map(|(k, v)| (k, v)))
        .output()
        .map_err(|e| format!("cannot run sh: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let tail: Vec<&str> = text.lines().rev().take(5).collect();
    let tail: Vec<&str> = tail.into_iter().rev().collect();
    Err(format!("{} ({})", tail.join(" / "), out.status))
}

/// Which command a job runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Deploy,
    Verify,
}

impl Step {
    fn name(self) -> &'static str {
        match self {
            Self::Deploy => "deploy",
            Self::Verify => "verify",
        }
    }
}

/// Where a command works: the project whose build it installs, and the folder
/// it runs in when `run_in` is set. Either can be set per milestone or task.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Place {
    project: ProjectId,
    run_in: Option<PathBuf>,
}

impl Place {
    /// Whether two commands would collide: one project, or one `run_in` folder.
    fn meets(&self, other: &Self) -> bool {
        self.project == other.project || (self.run_in.is_some() && self.run_in == other.run_in)
    }
}

/// How long after a failed deploy it is run again.
pub const RETRY_AFTER: Duration = Duration::from_secs(600);

/// How many times a deploy is run before it is left for a person.
pub const TRIES: u32 = 3;

/// A deploy that failed: why, how many times, and when last.
struct Failure {
    why: String,
    tries: u32,
    at: Instant,
}

/// A deploy that finished and handed the restart off: when, and where it ran.
struct Handed {
    at: Instant,
    place: Place,
}

/// The environment variable that names the handoff file for a deploy command.
pub const HANDOFF_VAR: &str = "TEMET_WORK_HANDOFF";

/// The file beside the handoff file that keeps the lines of deploys run again
/// because another build came up: `ISSUE BUILD RUNNING`, one per rerun.
fn stale_file(handoff: &Path) -> PathBuf {
    let mut name = handoff.as_os_str().to_owned();
    name.push(".stale");
    PathBuf::from(name)
}

/// What a handoff or stale file says: one line per deploy, `ISSUE REST`, where
/// the rest is the build (or for the stale file, the build installed and the
/// build running).
fn read_lines(path: &Path) -> Vec<(IssueId, String)> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let (issue, rest) = line.trim().split_once(char::is_whitespace)?;
            Some((IssueId::parse(issue)?, rest.trim().to_string()))
        })
        .collect()
}

/// Keeps the lines whose issue `keep` accepts, and removes the file when none
/// is left. A file with nothing to take out is not touched.
fn keep_lines(path: &Path, keep: impl Fn(&IssueId) -> bool) {
    let lines = read_lines(path);
    let rest: Vec<String> = lines
        .iter()
        .filter(|(i, _)| keep(i))
        .map(|(i, r)| format!("{i} {r}\n"))
        .collect();
    if rest.len() == lines.len() {
        return;
    }
    let _ = if rest.is_empty() {
        std::fs::remove_file(path)
    } else {
        std::fs::write(path, rest.concat())
    };
}

/// Takes `issue`'s lines out of a handoff or stale file.
fn clear_lines(path: &Path, issue: &IssueId) {
    keep_lines(path, |i| i != issue);
}

/// A command running for an issue, where it runs, and where its result arrives.
struct Job {
    issue: IssueId,
    step: Step,
    place: Place,
    done: Receiver<Result<(), String>>,
}

/// The deploy part.
pub struct Deploys {
    shell: Arc<Shell>,
    running: Vec<Job>,
    failed: BTreeMap<IssueId, Failure>,
    /// The handoff file and this daemon's own build, when deploys may restart it.
    handoff: Option<(PathBuf, String)>,
    /// Deploys that finished and handed the restart off.
    pending: BTreeMap<IssueId, Handed>,
}

impl Default for Deploys {
    fn default() -> Self {
        Self::with(Arc::new(sh))
    }
}

impl Deploys {
    /// A deploy part that runs commands through `shell`.
    pub fn with(shell: Arc<Shell>) -> Self {
        Self {
            shell,
            running: Vec::new(),
            failed: BTreeMap::new(),
            handoff: None,
            pending: BTreeMap::new(),
        }
    }

    /// Lets a deploy restart the daemon: it records itself in `file` (given to
    /// the command as `TEMET_WORK_HANDOFF`), and counts as done once a daemon
    /// whose build is `build` starts and finds the line.
    pub fn handoff(mut self, file: PathBuf, build: impl Into<String>) -> Self {
        self.handoff = Some((file, build.into()));
        self
    }

    /// The build a deploy of `issue` handed its restart to, if it did.
    fn handed_to(&self, issue: &IssueId) -> Option<String> {
        let (file, _) = self.handoff.as_ref()?;
        read_lines(file)
            .into_iter()
            .find_map(|(i, b)| (&i == issue).then_some(b))
    }

    /// How many commands are running.
    pub fn running(&self) -> usize {
        self.running.len()
    }

    /// Whether a command that meets `place` runs, or a deploy there waits for
    /// its restart (for at most the ten minutes a handoff waits).
    fn busy(&self, place: &Place) -> bool {
        self.running.iter().any(|j| j.place.meets(place))
            || self
                .pending
                .values()
                .any(|h| h.at.elapsed() < RETRY_AFTER && h.place.meets(place))
    }

    /// Records `issue` as deployed: its handoff line, its stale lines and its
    /// failures are spent.
    fn deployed(&mut self, issue: IssueId) -> Event {
        if let Some((file, _)) = &self.handoff {
            clear_lines(file, &issue);
            clear_lines(&stale_file(file), &issue);
        }
        self.failed.remove(&issue);
        Event::Deployed { issue }
    }

    /// Before a deploy of `issue` runs again: moves a handoff line naming
    /// another build to the stale file, where it counts as a rerun. When the
    /// count cannot be kept the deploy does not run, so a restart that keeps
    /// coming up as another build cannot rebuild for ever.
    fn set_aside(&self, issue: &IssueId) -> Result<(), String> {
        let Some((file, own)) = &self.handoff else {
            return Ok(());
        };
        let Some(build) = self.handed_to(issue).filter(|b| b != own) else {
            return Ok(());
        };
        let stale = stale_file(file);
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&stale)
            .and_then(|mut f| f.write_all(format!("{issue} {build} {own}\n").as_bytes()))
            .map_err(|e| format!("cannot count the rerun in {}: {e}", stale.display()))?;
        clear_lines(file, issue);
        Ok(())
    }
}

impl Deploys {
    /// What to do about a deploy asked for again when it handed its restart
    /// off: `None` when it did not, or when it is to run again.
    fn handed_off(&mut self, issue: &IssueId) -> Option<Outcome> {
        let (file, own) = self.handoff.clone()?;
        if let Some(handed) = self.pending.get(issue) {
            // This daemon ran the deploy and is waiting to be restarted.
            if handed.at.elapsed() < RETRY_AFTER {
                return Some(Outcome::done());
            }
            self.pending.remove(issue);
            clear_lines(&file, issue);
            let tries = self.failed.get(issue).map_or(0, |f| f.tries) + 1;
            self.failed.insert(
                issue.clone(),
                Failure {
                    why: "the restart was handed off but this daemon was not restarted".into(),
                    tries,
                    at: Instant::now(),
                },
            );
            return None;
        }
        let build = self.handed_to(issue)?;
        if build == own {
            // The restart brought the new build: that is the deploy done.
            return Some(Outcome::Done {
                events: vec![self.deployed(issue.clone())],
                effects: Vec::new(),
            });
        }
        // Another build came up: the restart started something other than what
        // was installed, or a later deploy installed over this one. The deploy
        // runs again, and the line moves to the stale file as it starts.
        let stale = stale_file(&file);
        let reruns = read_lines(&stale)
            .iter()
            .filter(|(i, _)| i == issue)
            .count();
        if reruns < TRIES as usize {
            return None;
        }
        Some(Outcome::Failed(format!(
            "ran {reruns} times, and each time the daemon that came up was not the build the \
             deploy installed (now {build} installed, {own} running; every try is in {}); a \
             person has to look at it, then `temet-work retry {issue}` deploys it again",
            stale.display()
        )))
    }
}

impl Part for Deploys {
    fn id(&self) -> &'static str {
        "deploy"
    }

    fn poll(&mut self, world: &World, _now_ms: u64) -> Polled {
        let mut events = Vec::new();
        let mut still = Vec::new();
        for job in std::mem::take(&mut self.running) {
            let result = match job.done.try_recv() {
                Ok(result) => result,
                Err(TryRecvError::Empty) => {
                    still.push(job);
                    continue;
                }
                Err(TryRecvError::Disconnected) => Err("the command's thread stopped".into()),
            };
            let Job {
                issue, step, place, ..
            } = job;
            match (step, result) {
                (Step::Deploy, Ok(())) => match self.handed_to(&issue) {
                    // The restart is handed off: this daemon may be gone before
                    // it records anything, so the next one does. Earlier tries
                    // still count, in case the restart never comes.
                    Some(build) if Some(&build) != self.handoff.as_ref().map(|h| &h.1) => {
                        self.pending.insert(
                            issue,
                            Handed {
                                at: Instant::now(),
                                place,
                            },
                        );
                    }
                    Some(_) | None => events.push(self.deployed(issue)),
                },
                (Step::Deploy, Err(why)) => {
                    // A failed deploy hands nothing off, whatever it wrote.
                    if let Some((file, _)) = &self.handoff {
                        clear_lines(file, &issue);
                    }
                    let tries = self.failed.get(&issue).map_or(0, |f| f.tries) + 1;
                    self.failed.insert(
                        issue,
                        Failure {
                            why,
                            tries,
                            at: Instant::now(),
                        },
                    );
                }
                (Step::Verify, result) => events.push(Event::Verified {
                    issue,
                    passed: result.is_ok(),
                    output: result.err().unwrap_or_default(),
                }),
            }
        }
        self.running = still;
        // Forget a failure, a pending deploy or a count of reruns once its issue
        // has moved on.
        let deploying = |issue: &IssueId| {
            world
                .issues
                .get(issue)
                .is_some_and(|i| i.stage == Stage::Deploying)
        };
        self.failed.retain(|issue, _| deploying(issue));
        self.pending.retain(|issue, _| deploying(issue));
        if let Some((file, _)) = &self.handoff {
            keep_lines(&stale_file(file), deploying);
        }
        Polled {
            events,
            problem: None,
        }
    }

    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let (issue, step) = match effect {
            Effect::Deploy { issue } => (issue, Step::Deploy),
            Effect::Verify { issue } => (issue, Step::Verify),
            Effect::Retry { issue } => {
                // A person asked: forget every failure, any handed-off restart
                // and the count of reruns, and run whichever step the issue is at.
                let step = match world.issues.get(issue).map(|i| i.stage) {
                    Some(Stage::Deploying) => Step::Deploy,
                    Some(Stage::Verifying) => Step::Verify,
                    _ => return Outcome::Failed(format!("{issue} is not deploying or verifying")),
                };
                self.failed.remove(issue);
                self.pending.remove(issue);
                if let Some((file, _)) = &self.handoff {
                    clear_lines(file, issue);
                    clear_lines(&stale_file(file), issue);
                }
                (issue, step)
            }
            _ => return Outcome::NotMine,
        };
        // Asked again while it runs: it is already being done.
        if self
            .running
            .iter()
            .any(|j| &j.issue == issue && j.step == step)
        {
            return Outcome::done();
        }
        if step == Step::Deploy
            && let Some(done) = self.handed_off(issue)
        {
            return done;
        }
        if step == Step::Deploy
            && let Some(f) = self.failed.get(issue)
        {
            if f.tries >= TRIES {
                return Outcome::Failed(format!(
                    "failed {} times, last: {}; a person has to look at it",
                    f.tries, f.why
                ));
            }
            if f.at.elapsed() < RETRY_AFTER {
                return Outcome::Failed(format!(
                    "failed: {}; runs again {} minutes after it failed",
                    f.why,
                    RETRY_AFTER.as_secs() / 60
                ));
            }
        }
        let Some(project) = world.issues.get(issue).and_then(|i| world.repo_of(i)) else {
            return Outcome::Failed(format!("{issue} has no repository configured"));
        };
        let command = match step {
            Step::Deploy => world.setting_text(issue, None, "deploy"),
            Step::Verify => world.setting_text(issue, None, "verify"),
        };
        let Some(command) = command else {
            let event = match step {
                Step::Deploy => self.deployed(issue.clone()),
                Step::Verify => Event::Verified {
                    issue: issue.clone(),
                    passed: true,
                    output: String::new(),
                },
            };
            return Outcome::Done {
                events: vec![event],
                effects: Vec::new(),
            };
        };
        let run_in = world.setting_text(issue, None, "run_in").map(PathBuf::from);
        let place = Place {
            project: project.id.clone(),
            run_in: run_in.clone(),
        };
        // Another command of this project or folder runs, or a deploy there
        // waits for its restart: this one starts when the loop asks again.
        if self.busy(&place) {
            return Outcome::done();
        }
        if step == Step::Deploy
            && let Err(why) = self.set_aside(issue)
        {
            return Outcome::Failed(why);
        }
        let cwd = run_in
            .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("/"));
        let mut env = vec![
            ("TEMET_WORK_ISSUE".to_string(), issue.to_string()),
            ("TEMET_WORK_PROJECT".to_string(), project.id.to_string()),
            ("TEMET_WORK_REPO".to_string(), project.repo.clone()),
        ];
        if let Some((file, _)) = &self.handoff {
            env.push((HANDOFF_VAR.to_string(), file.display().to_string()));
        }
        let (tx, rx) = mpsc::channel();
        let shell = Arc::clone(&self.shell);
        let spawned = std::thread::Builder::new()
            .name(format!("{} {issue}", step.name()))
            .spawn(move || {
                let _ = tx.send(shell(&command, &cwd, &env));
            });
        match spawned {
            Ok(_) => {
                self.running.push(Job {
                    issue: issue.clone(),
                    step,
                    place,
                    done: rx,
                });
                Outcome::done()
            }
            Err(e) => Outcome::Failed(format!("cannot start the {} command: {e}", step.name())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temet_work_core::ids::ProjectId;

    fn world(deploy: Option<&str>, verify: Option<&str>) -> World {
        let mut w = World::default();
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new("ankh"),
            tracker: "fake".into(),
            name: "city/ankh".into(),
            code_host: None,
            repo: None,
        });
        w.apply(&Event::ProjectCommandsSet {
            project: ProjectId::new("ankh"),
            deploy: deploy.map(String::from),
            verify: verify.map(String::from),
            run_in: Some("/srv/ankh".into()),
            ci: None,
        });
        w.apply(&Event::IssueSynced {
            issue: issue(),
            title: "Mend the clock".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        w
    }

    /// A shell that fails any command containing "fail" and records the rest.
    fn shell() -> Arc<Shell> {
        Arc::new(|command: &str, cwd: &PathBuf, env: &[(String, String)]| {
            assert_eq!(cwd, &PathBuf::from("/srv/ankh"));
            assert!(env.contains(&("TEMET_WORK_ISSUE".into(), "ankh#1".into())));
            if command.contains("fail") {
                Err("exit status: 1".into())
            } else {
                Ok(())
            }
        })
    }

    fn issue() -> IssueId {
        IssueId::new("ankh", "1")
    }

    /// Polls until the running commands finish, giving way to their threads
    /// between polls: usually within a millisecond, at most two seconds when
    /// the machine is busy.
    fn settle(part: &mut Deploys, w: &World) -> Polled {
        let mut out = Polled::default();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            let p = part.poll(w, 0);
            out.events.extend(p.events);
            out.problem = p.problem;
            if part.running() == 0 || std::time::Instant::now() > deadline {
                break;
            }
            std::thread::yield_now();
        }
        out
    }

    #[test]
    fn with_no_commands_merged_is_deployed_and_deployed_is_verified() {
        let w = world(None, None);
        let mut part = Deploys::with(shell());
        let deploy = Effect::Deploy { issue: issue() };
        assert!(matches!(
            part.carry_out(&deploy, &w),
            Outcome::Done { events, .. } if events == [Event::Deployed { issue: issue() }]
        ));
        let verify = Effect::Verify { issue: issue() };
        assert!(matches!(
            part.carry_out(&verify, &w),
            Outcome::Done { events, .. } if events == [Event::Verified { issue: issue(), passed: true ,    output: String::new(),}]
        ));
    }

    #[test]
    fn runs_the_commands_and_reports_when_they_finish() {
        let w = world(Some("make deploy"), Some("make fail-check"));
        let mut part = Deploys::with(shell());
        assert_eq!(
            part.carry_out(&Effect::Deploy { issue: issue() }, &w),
            Outcome::done()
        );
        assert_eq!(
            settle(&mut part, &w).events,
            [Event::Deployed { issue: issue() }]
        );
        assert_eq!(
            part.carry_out(&Effect::Verify { issue: issue() }, &w),
            Outcome::done()
        );
        assert_eq!(
            settle(&mut part, &w).events,
            [Event::Verified {
                issue: issue(),
                passed: false,
                output: "exit status: 1".into(),
            }]
        );
    }

    #[test]
    fn a_failed_deploy_is_not_run_again_at_once_and_a_running_one_not_twice() {
        let mut w = world(Some("make fail"), None);
        w.apply(&Event::IssueSynced {
            issue: issue(),
            title: "Mend the clock".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = temet_work_core::model::Stage::Deploying;
        }
        let deploy = Effect::Deploy { issue: issue() };
        let mut part = Deploys::with(shell());
        part.carry_out(&deploy, &w);
        assert_eq!(
            part.carry_out(&deploy, &w),
            Outcome::done(),
            "already running"
        );
        assert_eq!(part.running(), 1);
        assert_eq!(settle(&mut part, &w).events, []);
        assert!(
            matches!(part.carry_out(&deploy, &w), Outcome::Failed(why) if why.contains("runs again 10 minutes")),
            "asked again at once, it says why and waits"
        );
        assert_eq!(part.running(), 0);
    }

    #[test]
    fn a_retry_clears_three_failures_and_runs_the_deploy_again() {
        let runs = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counted = Arc::clone(&runs);
        let shell: Arc<Shell> = Arc::new(move |_: &str, _: &PathBuf, _: &[(String, String)]| {
            let n = counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            if n <= 3 {
                Err(format!("run {n} failed"))
            } else {
                Ok(())
            }
        });
        let mut w = world(Some("make deploy"), None);
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = Stage::Deploying;
        }
        let deploy = Effect::Deploy { issue: issue() };
        let mut part = Deploys::with(shell);
        for _ in 0..3 {
            assert_eq!(part.carry_out(&deploy, &w), Outcome::done());
            settle(&mut part, &w);
            // Pretend the ten minutes have passed.
            if let Some(f) = part.failed.get_mut(&issue()) {
                f.at = Instant::now() - RETRY_AFTER;
            }
        }
        assert!(
            matches!(part.carry_out(&deploy, &w), Outcome::Failed(why) if why.contains("a person has to look at it"))
        );
        assert_eq!(runs.load(std::sync::atomic::Ordering::SeqCst), 3);
        let retry = Effect::Retry { issue: issue() };
        assert_eq!(part.carry_out(&retry, &w), Outcome::done());
        assert_eq!(
            settle(&mut part, &w).events,
            [Event::Deployed { issue: issue() }]
        );
        assert_eq!(runs.load(std::sync::atomic::Ordering::SeqCst), 4);
        assert!(part.failed.is_empty());
    }

    #[test]
    fn a_retry_in_the_verifying_stage_runs_the_verify_command_again() {
        let mut w = world(Some("make deploy"), Some("make check"));
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = Stage::Verifying;
        }
        let mut part = Deploys::with(shell());
        assert_eq!(
            part.carry_out(&Effect::Retry { issue: issue() }, &w),
            Outcome::done()
        );
        assert_eq!(
            settle(&mut part, &w).events,
            [Event::Verified {
                issue: issue(),
                passed: true,
                output: String::new(),
            }]
        );
    }

    /// A scratch folder for one test.
    fn scratch(name: &str) -> PathBuf {
        // Under the workspace's target folder, not the shared temp folder.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/test-scratch")
            .join(format!("deploy-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch folder");
        dir
    }

    /// Runs commands for real, whatever the sandbox says, in the scratch folder.
    fn real(dir: &Path) -> Arc<Shell> {
        let dir = dir.to_path_buf();
        Arc::new(
            move |command: &str, _: &PathBuf, env: &[(String, String)]| run(command, &dir, env),
        )
    }

    #[test]
    fn self_deploy() {
        let dir = scratch("self");
        let handoff = dir.join("deploy-handoff");
        let mut w = world(
            Some(
                "touch installed && echo \"$TEMET_WORK_ISSUE new-build\" >> \"$TEMET_WORK_HANDOFF\" \
                 && touch restart-started",
            ),
            Some("true"),
        );
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = temet_work_core::model::Stage::Deploying;
        }
        let mut old = Deploys::with(real(&dir)).handoff(handoff.clone(), "old-build");
        let deploy = Effect::Deploy { issue: issue() };
        assert_eq!(old.carry_out(&deploy, &w), Outcome::done());
        // The deploy finished, handed the restart off, and is not reported done.
        assert_eq!(settle(&mut old, &w).events, []);
        assert!(dir.join("installed").exists() && dir.join("restart-started").exists());
        assert_eq!(
            std::fs::read_to_string(&handoff).expect("handoff file"),
            "ankh#1 new-build\n",
            "the pending deploy is recorded"
        );
        // Asked again before the restart: not run again, not reported.
        std::fs::remove_file(dir.join("installed")).expect("rm");
        assert_eq!(old.carry_out(&deploy, &w), Outcome::done());
        assert_eq!(settle(&mut old, &w).events, []);
        assert!(
            !dir.join("installed").exists(),
            "the deploy did not run again"
        );
        // The daemon is killed mid-job: a new one with the new build starts.
        drop(old);
        let mut new = Deploys::with(real(&dir)).handoff(handoff.clone(), "new-build");
        assert_eq!(
            new.carry_out(&deploy, &w),
            Outcome::Done {
                events: vec![Event::Deployed { issue: issue() }],
                effects: vec![]
            }
        );
        assert!(
            !dir.join("installed").exists(),
            "and it did not deploy again"
        );
        assert!(!handoff.exists(), "the handoff is spent");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn handed_off_is_deployed_then_verified_once() {
        let dir = scratch("handed");
        let handoff = dir.join("deploy-handoff");
        std::fs::write(&handoff, "ankh#1 new-build\n").expect("handoff");
        let mut w = world(Some("make deploy"), Some("make smoke"));
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = temet_work_core::model::Stage::Deploying;
        }
        let mut part = Deploys::with(shell()).handoff(handoff.clone(), "new-build");
        let mut events = Vec::new();
        for _ in 0..2 {
            // The loop asks again every round while the issue is deploying.
            if w.issues[&issue()].stage != temet_work_core::model::Stage::Deploying {
                continue;
            }
            if let Outcome::Done { events: e, .. } =
                part.carry_out(&Effect::Deploy { issue: issue() }, &w)
            {
                for event in e {
                    w.apply(&event);
                    events.push(event);
                }
            }
        }
        assert_eq!(events, [Event::Deployed { issue: issue() }]);
        let verify = Effect::Verify { issue: issue() };
        assert_eq!(part.carry_out(&verify, &w), Outcome::done());
        assert_eq!(part.carry_out(&verify, &w), Outcome::done());
        events.extend(settle(&mut part, &w).events);
        events.extend(settle(&mut part, &w).events);
        assert_eq!(
            events,
            [
                Event::Deployed { issue: issue() },
                Event::Verified {
                    issue: issue(),
                    passed: true,
                    output: String::new()
                }
            ],
            "once each, and a later poll adds none"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// How many lines the deploy command has appended to `runs`.
    fn runs(dir: &Path) -> usize {
        std::fs::read_to_string(dir.join("runs")).map_or(0, |t| t.lines().count())
    }

    #[test]
    fn a_handoff_that_never_restarts_waits_for_a_person() {
        let dir = scratch("never");
        let handoff = dir.join("deploy-handoff");
        let mut w = world(
            Some(
                "echo run >> runs && echo \"$TEMET_WORK_ISSUE new-build\" >> \"$TEMET_WORK_HANDOFF\"",
            ),
            None,
        );
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = Stage::Deploying;
        }
        let mut part = Deploys::with(real(&dir)).handoff(handoff, "old-build");
        let deploy = Effect::Deploy { issue: issue() };
        for _ in 0..TRIES {
            assert_eq!(part.carry_out(&deploy, &w), Outcome::done());
            assert_eq!(settle(&mut part, &w).events, []);
            // Ten minutes pass with no restart, and the try is counted.
            part.pending.get_mut(&issue()).expect("pending").at -= RETRY_AFTER;
            part.carry_out(&deploy, &w);
            // Then the wait before the next run passes.
            part.failed.get_mut(&issue()).expect("failure").at -= RETRY_AFTER;
        }
        assert_eq!(
            part.carry_out(&deploy, &w),
            Outcome::Failed(
                "failed 3 times, last: the restart was handed off but this daemon was not \
                 restarted; a person has to look at it"
                    .into()
            )
        );
        assert_eq!(part.running(), 0);
        assert_eq!(runs(&dir), TRIES as usize);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_deploy_that_fails_after_its_handoff_line_runs_again() {
        let dir = scratch("after");
        let handoff = dir.join("deploy-handoff");
        let mut w = world(
            Some(
                "echo run >> runs && echo \"$TEMET_WORK_ISSUE new-build\" >> \"$TEMET_WORK_HANDOFF\" \
                 && false",
            ),
            None,
        );
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = Stage::Deploying;
        }
        let mut part = Deploys::with(real(&dir)).handoff(handoff.clone(), "old-build");
        let deploy = Effect::Deploy { issue: issue() };
        assert_eq!(part.carry_out(&deploy, &w), Outcome::done());
        assert_eq!(settle(&mut part, &w).events, []);
        assert!(!handoff.exists(), "a failed deploy hands nothing off");
        assert_eq!(part.failed.get(&issue()).map(|f| f.tries), Some(1));
        assert!(matches!(
            part.carry_out(&deploy, &w),
            Outcome::Failed(why) if why.contains("runs again 10 minutes after it failed")
        ));
        part.failed.get_mut(&issue()).expect("failure").at -= RETRY_AFTER;
        assert_eq!(part.carry_out(&deploy, &w), Outcome::done());
        settle(&mut part, &w);
        assert_eq!(runs(&dir), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_handoff_to_another_build_runs_the_deploy_again() {
        let dir = scratch("other");
        let handoff = dir.join("deploy-handoff");
        std::fs::write(&handoff, "ankh#1 X\n").expect("handoff");
        let mut w = world(
            Some("echo run >> runs && echo \"$TEMET_WORK_ISSUE Y\" >> \"$TEMET_WORK_HANDOFF\""),
            None,
        );
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = Stage::Deploying;
        }
        let mut part = Deploys::with(real(&dir)).handoff(handoff.clone(), "Y");
        assert_eq!(
            part.carry_out(&Effect::Deploy { issue: issue() }, &w),
            Outcome::done(),
            "not a problem: the deploy runs again"
        );
        assert_eq!(
            std::fs::read_to_string(stale_file(&handoff)).expect("stale file"),
            "ankh#1 X Y\n",
            "the line naming X was moved aside before the rerun"
        );
        let events = settle(&mut part, &w).events;
        assert_eq!(events, [Event::Deployed { issue: issue() }]);
        assert_eq!(runs(&dir), 1);
        for event in &events {
            w.apply(event);
        }
        assert_eq!(w.issues[&issue()].stage, Stage::Verifying);
        assert!(!handoff.exists(), "the rerun's own line is spent");
        assert!(
            !stale_file(&handoff).exists(),
            "Deployed clears the count of reruns"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stale_handoff_reruns_stop_after_three_restarts() {
        let dir = scratch("stale");
        let handoff = dir.join("deploy-handoff");
        std::fs::write(&handoff, "ankh#1 X\n").expect("handoff");
        // The deploy installs X, but the service keeps starting Y.
        let mut w = world(
            Some("echo run >> runs && echo \"$TEMET_WORK_ISSUE X\" >> \"$TEMET_WORK_HANDOFF\""),
            None,
        );
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = Stage::Deploying;
        }
        let deploy = Effect::Deploy { issue: issue() };
        for restart in 1..=TRIES as usize {
            // A new daemon at build Y each time, with nothing in memory.
            let mut part = Deploys::with(real(&dir)).handoff(handoff.clone(), "Y");
            assert_eq!(part.carry_out(&deploy, &w), Outcome::done());
            assert_eq!(settle(&mut part, &w).events, [], "handed off to X again");
            assert_eq!(runs(&dir), restart);
            let stale = std::fs::read_to_string(stale_file(&handoff)).expect("stale file");
            assert_eq!(stale, "ankh#1 X Y\n".repeat(restart));
        }
        let mut part = Deploys::with(real(&dir)).handoff(handoff.clone(), "Y");
        let Outcome::Failed(why) = part.carry_out(&deploy, &w) else {
            panic!("a fourth rerun is left for a person");
        };
        assert!(why.contains("a person has to look at it"), "{why}");
        assert!(why.contains("temet-work retry ankh#1"), "{why}");
        assert!(why.contains("X installed, Y running"), "{why}");
        assert_eq!(part.running(), 0);
        assert_eq!(
            runs(&dir),
            TRIES as usize,
            "the command ran exactly three times"
        );
        // The retry the message names clears the count and deploys again.
        assert_eq!(
            part.carry_out(&Effect::Retry { issue: issue() }, &w),
            Outcome::done()
        );
        assert!(!stale_file(&handoff).exists(), "the count is cleared");
        settle(&mut part, &w);
        assert_eq!(runs(&dir), TRIES as usize + 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_stale_count_goes_when_its_issue_leaves_deploying() {
        let dir = scratch("leaves");
        let handoff = dir.join("deploy-handoff");
        std::fs::write(stale_file(&handoff), "ankh#1 X Y\nankh#2 X Y\n").expect("stale");
        let mut w = world(None, None);
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = Stage::Deploying;
        }
        let other = add_issue(&mut w, "ankh", "2", Stage::Verifying);
        let mut part = Deploys::with(shell()).handoff(handoff.clone(), "Y");
        part.poll(&w, 0);
        assert_eq!(
            std::fs::read_to_string(stale_file(&handoff)).expect("stale"),
            "ankh#1 X Y\n",
            "{other} is verifying: its count is gone"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Adds issue `key` of `project` to the world at `stage`.
    fn add_issue(w: &mut World, project: &str, key: &str, stage: Stage) -> IssueId {
        let id = IssueId::new(project, key);
        w.apply(&Event::IssueSynced {
            issue: id.clone(),
            title: "Wind the clock".into(),
            milestone: None,
            depends_on: vec![],
            order: 2,
            closed: false,
        });
        w.issues.get_mut(&id).expect("synced").stage = stage;
        id
    }

    /// Adds project `id`, deploying with `make deploy` in `run_in`.
    fn add_project(w: &mut World, id: &str, run_in: &str) {
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new(id),
            tracker: "fake".into(),
            name: format!("city/{id}"),
            code_host: None,
            repo: None,
        });
        w.apply(&Event::ProjectCommandsSet {
            project: ProjectId::new(id),
            deploy: Some("make deploy".into()),
            verify: None,
            run_in: Some(run_in.into()),
            ci: None,
        });
    }

    /// A shell that holds every command until the test opens it, and counts how
    /// many are inside it at once and which issue each came in for.
    #[derive(Default)]
    struct Gate {
        inside: std::sync::Mutex<Inside>,
        changed: std::sync::Condvar,
    }

    #[derive(Default)]
    struct Inside {
        now: usize,
        most: usize,
        came: Vec<String>,
        opened: usize,
    }

    impl Gate {
        fn shell(self: &Arc<Self>) -> Arc<Shell> {
            let gate = Arc::clone(self);
            Arc::new(move |_: &str, _: &PathBuf, env: &[(String, String)]| {
                let issue = env
                    .iter()
                    .find_map(|(k, v)| (k == "TEMET_WORK_ISSUE").then(|| v.clone()))
                    .unwrap_or_default();
                let mut inside = gate.inside.lock().map_err(|e| e.to_string())?;
                inside.now += 1;
                inside.most = inside.most.max(inside.now);
                inside.came.push(issue);
                let ticket = inside.came.len();
                gate.changed.notify_all();
                let mut inside = gate
                    .changed
                    .wait_while(inside, |i| i.opened < ticket)
                    .map_err(|e| e.to_string())?;
                inside.now -= 1;
                Ok(())
            })
        }

        /// Waits until `n` commands have come in, woken by each arrival: at
        /// once when they have, at most two seconds when the machine is busy.
        fn came(&self, n: usize) -> Vec<String> {
            let inside = self.inside.lock().expect("gate");
            let (inside, _) = self
                .changed
                .wait_timeout_while(inside, Duration::from_secs(2), |i| i.came.len() < n)
                .expect("gate");
            inside.came.clone()
        }

        /// The most commands that were inside at once.
        fn most(&self) -> usize {
            self.inside.lock().expect("gate").most
        }

        /// Lets every command that has come in finish.
        fn open(&self) {
            let mut inside = self.inside.lock().expect("gate");
            inside.opened = inside.came.len();
            self.changed.notify_all();
        }
    }

    /// Settles the running commands and applies what they reported, as the
    /// loop does.
    fn settle_into(part: &mut Deploys, w: &mut World) -> Vec<Event> {
        let events = settle(part, w).events;
        for event in &events {
            w.apply(event);
        }
        events
    }

    #[test]
    fn two_deploys_of_one_project_run_one_after_the_other() {
        let mut w = world(Some("make deploy"), None);
        w.issues.get_mut(&issue()).expect("issue").stage = Stage::Deploying;
        let second = add_issue(&mut w, "ankh", "2", Stage::Deploying);
        let gate = Arc::new(Gate::default());
        let mut part = Deploys::with(gate.shell());
        let deploy = |issue: &IssueId| Effect::Deploy {
            issue: issue.clone(),
        };
        // One round asks for both.
        assert_eq!(part.carry_out(&deploy(&issue()), &w), Outcome::done());
        assert_eq!(part.carry_out(&deploy(&second), &w), Outcome::done());
        // Every command that started has come in, and that is the first alone.
        assert_eq!(gate.came(part.running()), ["ankh#1"]);
        // A later round, while the first still runs: the second waits again.
        assert_eq!(part.poll(&w, 1).events, []);
        assert_eq!(part.carry_out(&deploy(&second), &w), Outcome::done());
        assert_eq!(part.running(), 1);
        gate.open();
        assert_eq!(
            settle_into(&mut part, &mut w),
            [Event::Deployed { issue: issue() }]
        );
        // The round after: the second runs, alone.
        assert_eq!(part.carry_out(&deploy(&second), &w), Outcome::done());
        assert_eq!(gate.came(2), ["ankh#1", "ankh#2"]);
        gate.open();
        assert_eq!(
            settle_into(&mut part, &mut w),
            [Event::Deployed {
                issue: second.clone()
            }]
        );
        assert_eq!(gate.most(), 1, "never two at once");
        assert_eq!(w.issues[&issue()].stage, Stage::Verifying);
        assert_eq!(w.issues[&second].stage, Stage::Verifying);
    }

    #[test]
    fn a_deploy_waiting_for_its_restart_holds_back_its_project() {
        let dir = scratch("waiting");
        let handoff = dir.join("deploy-handoff");
        let mut w = world(
            Some(
                "echo \"$TEMET_WORK_ISSUE\" >> runs && \
                 echo \"$TEMET_WORK_ISSUE new-build\" >> \"$TEMET_WORK_HANDOFF\"",
            ),
            None,
        );
        w.issues.get_mut(&issue()).expect("issue").stage = Stage::Deploying;
        let second = add_issue(&mut w, "ankh", "2", Stage::Deploying);
        let mut part = Deploys::with(real(&dir)).handoff(handoff, "old-build");
        let deploy = |issue: &IssueId| Effect::Deploy {
            issue: issue.clone(),
        };
        assert_eq!(part.carry_out(&deploy(&issue()), &w), Outcome::done());
        assert_eq!(settle(&mut part, &w).events, [], "handed off");
        assert_eq!(part.carry_out(&deploy(&second), &w), Outcome::done());
        assert_eq!(part.running(), 0, "the second waits for the restart");
        // Ten minutes pass with no restart: the second runs.
        part.pending.get_mut(&issue()).expect("pending").at -= RETRY_AFTER;
        assert_eq!(part.carry_out(&deploy(&second), &w), Outcome::done());
        settle(&mut part, &w);
        assert_eq!(
            std::fs::read_to_string(dir.join("runs")).expect("runs"),
            "ankh#1\nankh#2\n"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn deploys_of_two_projects_run_side_by_side() {
        let mut w = world(Some("make deploy"), None);
        w.issues.get_mut(&issue()).expect("issue").stage = Stage::Deploying;
        add_project(&mut w, "klatch", "/srv/klatch");
        let klatch = add_issue(&mut w, "klatch", "1", Stage::Deploying);
        let gate = Arc::new(Gate::default());
        let mut part = Deploys::with(gate.shell());
        for issue in [issue(), klatch.clone()] {
            assert_eq!(
                part.carry_out(&Effect::Deploy { issue }, &w),
                Outcome::done()
            );
        }
        assert_eq!(part.running(), 2);
        assert_eq!(gate.came(2).len(), 2);
        assert_eq!(gate.most(), 2, "both at once");
        gate.open();
        let mut events = settle(&mut part, &w).events;
        events.sort_by_key(|e| format!("{e:?}"));
        assert_eq!(
            events,
            [
                Event::Deployed { issue: issue() },
                Event::Deployed { issue: klatch }
            ]
        );
    }

    #[test]
    fn two_projects_in_one_run_in_folder_take_turns() {
        let mut w = world(Some("make deploy"), None);
        w.issues.get_mut(&issue()).expect("issue").stage = Stage::Deploying;
        add_project(&mut w, "morpork", "/srv/ankh/");
        let morpork = add_issue(&mut w, "morpork", "1", Stage::Deploying);
        let gate = Arc::new(Gate::default());
        let mut part = Deploys::with(gate.shell());
        for issue in [issue(), morpork.clone()] {
            assert_eq!(
                part.carry_out(&Effect::Deploy { issue }, &w),
                Outcome::done()
            );
        }
        assert_eq!(gate.came(part.running()), ["ankh#1"]);
        gate.open();
        settle(&mut part, &w);
        assert_eq!(
            part.carry_out(&Effect::Deploy { issue: morpork }, &w),
            Outcome::done()
        );
        assert_eq!(gate.came(2), ["ankh#1", "morpork#1"]);
        gate.open();
        settle(&mut part, &w);
        assert_eq!(gate.most(), 1);
    }

    #[test]
    fn a_verify_and_a_deploy_of_one_project_do_not_overlap() {
        let mut w = world(Some("make deploy"), Some("make check"));
        w.issues.get_mut(&issue()).expect("issue").stage = Stage::Verifying;
        let deploying = add_issue(&mut w, "ankh", "2", Stage::Deploying);
        let gate = Arc::new(Gate::default());
        let mut part = Deploys::with(gate.shell());
        // A verify runs: a deploy of its project asked after it waits.
        let verify = Effect::Verify { issue: issue() };
        let deploy = Effect::Deploy {
            issue: deploying.clone(),
        };
        assert_eq!(part.carry_out(&verify, &w), Outcome::done());
        assert_eq!(part.carry_out(&deploy, &w), Outcome::done());
        assert_eq!(gate.came(part.running()), ["ankh#1"]);
        gate.open();
        assert!(matches!(
            settle_into(&mut part, &mut w).as_slice(),
            [Event::Verified { passed: true, .. }]
        ));
        // A deploy runs: a verify of its project asked after it waits.
        let third = add_issue(&mut w, "ankh", "3", Stage::Verifying);
        let verify = Effect::Verify {
            issue: third.clone(),
        };
        assert_eq!(part.carry_out(&deploy, &w), Outcome::done());
        assert_eq!(part.carry_out(&verify, &w), Outcome::done());
        assert_eq!(gate.came(1 + part.running()), ["ankh#1", "ankh#2"]);
        gate.open();
        assert_eq!(
            settle_into(&mut part, &mut w),
            [Event::Deployed { issue: deploying }]
        );
        // The round after, the verify runs.
        assert_eq!(part.carry_out(&verify, &w), Outcome::done());
        assert_eq!(gate.came(3), ["ankh#1", "ankh#2", "ankh#3"]);
        gate.open();
        settle(&mut part, &w);
        assert_eq!(gate.most(), 1, "never two at once");
    }

    #[test]
    fn build_fails() {
        let dir = scratch("build");
        let handoff = dir.join("deploy-handoff");
        let mut w = world(
            Some(
                "echo compiling; echo 'error[E0425]: cannot find value x' >&2; false \
                 && echo \"$TEMET_WORK_ISSUE new-build\" >> \"$TEMET_WORK_HANDOFF\" \
                 && touch restart-started",
            ),
            None,
        );
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = temet_work_core::model::Stage::Deploying;
        }
        let mut part = Deploys::with(real(&dir)).handoff(handoff.clone(), "old-build");
        let deploy = Effect::Deploy { issue: issue() };
        part.carry_out(&deploy, &w);
        assert_eq!(settle(&mut part, &w).events, [], "no Deployed");
        assert!(!dir.join("restart-started").exists(), "no restart");
        assert!(!handoff.exists(), "nothing handed off");
        let Outcome::Failed(why) = part.carry_out(&deploy, &w) else {
            panic!("a failed build is listed as a problem");
        };
        assert!(why.contains("cannot find value x"), "{why}");
        // The three-tries rule still holds.
        for _ in 1..TRIES {
            part.failed.get_mut(&issue()).expect("failure").at -= RETRY_AFTER;
            part.carry_out(&deploy, &w);
            settle(&mut part, &w);
        }
        assert!(matches!(
            part.carry_out(&deploy, &w),
            Outcome::Failed(why) if why.contains("failed 3 times") && why.contains("a person")
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn runs_nothing_in_the_sandbox() {
        if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
            assert!(sh("true", &PathBuf::from("/"), &[]).is_err());
        }
    }
}
