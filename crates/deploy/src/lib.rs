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
//! A deploy may restart the daemon that is running it (temet-work deploying
//! itself). Then the command builds and installs first, records the issue and
//! the build it installed as a line `ISSUE BUILD` in the file named by
//! `TEMET_WORK_HANDOFF` (next to the daemon's log), and ends by starting the
//! restart detached, so the thread running it is never what gets killed. The
//! old daemon does not record `Deployed` for such a deploy; the new daemon does,
//! when it starts with the build the file names. Until then the deploy counts as
//! pending and is not run again.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use temet_work_core::ids::IssueId;
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

/// The environment variable that names the handoff file for a deploy command.
pub const HANDOFF_VAR: &str = "TEMET_WORK_HANDOFF";

/// What the handoff file says: one line per deploy, `ISSUE BUILD`.
fn read_handoff(path: &Path) -> Vec<(IssueId, String)> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let (issue, build) = line.trim().split_once(char::is_whitespace)?;
            Some((IssueId::parse(issue)?, build.trim().to_string()))
        })
        .collect()
}

/// Takes `issue`'s line out of the handoff file, and the file itself when it
/// was the last one.
fn clear_handoff(path: &Path, issue: &IssueId) {
    let rest: Vec<String> = read_handoff(path)
        .into_iter()
        .filter(|(i, _)| i != issue)
        .map(|(i, b)| format!("{i} {b}\n"))
        .collect();
    let _ = if rest.is_empty() {
        std::fs::remove_file(path)
    } else {
        std::fs::write(path, rest.concat())
    };
}

/// A command running for an issue, and where its result arrives.
struct Job {
    issue: IssueId,
    step: Step,
    done: Receiver<Result<(), String>>,
}

/// The deploy part.
pub struct Deploys {
    shell: Arc<Shell>,
    running: Vec<Job>,
    failed: BTreeMap<IssueId, Failure>,
    /// The handoff file and this daemon's own build, when deploys may restart it.
    handoff: Option<(PathBuf, String)>,
    /// Deploys that finished and handed the restart off, and when.
    pending: BTreeMap<IssueId, Instant>,
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
        read_handoff(file)
            .into_iter()
            .find_map(|(i, b)| (&i == issue).then_some(b))
    }

    /// How many commands are running.
    pub fn running(&self) -> usize {
        self.running.len()
    }
}

impl Deploys {
    /// What to do about a deploy asked for again when it handed its restart
    /// off: `None` when it did not, so it runs as usual.
    fn handed_off(&mut self, issue: &IssueId) -> Option<Outcome> {
        let own = self.handoff.as_ref()?.1.clone();
        if let Some(at) = self.pending.get(issue) {
            // This daemon ran the deploy and is waiting to be restarted.
            if at.elapsed() < RETRY_AFTER {
                return Some(Outcome::done());
            }
            self.pending.remove(issue);
            if let Some((file, _)) = &self.handoff {
                clear_handoff(file, issue);
            }
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
        if build != own {
            return Some(Outcome::Failed(format!(
                "a deploy installed build {build} and handed the restart off, but this daemon is \
                 build {own}; the restart did not happen, or it ran an older build \
                 (delete the line for {issue} in {} to deploy again)",
                self.handoff
                    .as_ref()
                    .map_or_else(String::new, |h| h.0.display().to_string())
            )));
        }
        // The restart brought the new build: that is the deploy done.
        if let Some((file, _)) = &self.handoff {
            clear_handoff(file, issue);
        }
        self.failed.remove(issue);
        Some(Outcome::Done {
            events: vec![Event::Deployed {
                issue: issue.clone(),
            }],
            effects: Vec::new(),
        })
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
            let Job { issue, step, .. } = job;
            match (step, result) {
                (Step::Deploy, Ok(())) => {
                    self.failed.remove(&issue);
                    match self.handed_to(&issue) {
                        // The restart is handed off: this daemon may be gone
                        // before it records anything, so the next one does.
                        Some(build) if Some(&build) != self.handoff.as_ref().map(|h| &h.1) => {
                            self.pending.insert(issue, Instant::now());
                        }
                        Some(_) | None => {
                            if let Some((file, _)) = &self.handoff {
                                clear_handoff(file, &issue);
                            }
                            events.push(Event::Deployed { issue });
                        }
                    }
                }
                (Step::Deploy, Err(why)) => {
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
        // Forget a failure or a pending deploy once its issue has moved on.
        let deploying = |issue: &IssueId| {
            world
                .issues
                .get(issue)
                .is_some_and(|i| i.stage == temet_work_core::model::Stage::Deploying)
        };
        self.failed.retain(|issue, _| deploying(issue));
        self.pending.retain(|issue, _| deploying(issue));
        Polled {
            events,
            problem: None,
        }
    }

    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let (issue, step) = match effect {
            Effect::Deploy { issue } => (issue, Step::Deploy),
            Effect::Verify { issue } => (issue, Step::Verify),
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
                Step::Deploy => Event::Deployed {
                    issue: issue.clone(),
                },
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
        let cwd = world
            .setting_text(issue, None, "run_in")
            .map(PathBuf::from)
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

    /// Polls until the running commands finish: usually within a millisecond,
    /// at most two seconds when the machine is busy.
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
            std::thread::sleep(std::time::Duration::from_millis(1));
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

    #[test]
    fn a_handoff_to_another_build_is_a_problem_not_a_deploy() {
        let dir = scratch("other");
        let handoff = dir.join("deploy-handoff");
        std::fs::write(&handoff, "ankh#1 new-build\n").expect("handoff");
        let w = world(Some("make deploy"), None);
        let mut part = Deploys::with(shell()).handoff(handoff.clone(), "old-build");
        assert!(matches!(
            part.carry_out(&Effect::Deploy { issue: issue() }, &w),
            Outcome::Failed(why) if why.contains("new-build") && why.contains("old-build")
        ));
        assert!(handoff.exists());
        let _ = std::fs::remove_dir_all(&dir);
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
