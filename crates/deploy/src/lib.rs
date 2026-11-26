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

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};

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
    failed: BTreeMap<IssueId, String>,
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
        }
    }

    /// How many commands are running.
    pub fn running(&self) -> usize {
        self.running.len()
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
                (Step::Deploy, Ok(())) => events.push(Event::Deployed { issue }),
                // A failed deploy stays a problem for a person: nothing retries it.
                (Step::Deploy, Err(e)) => {
                    self.failed.insert(issue, e);
                }
                (Step::Verify, result) => {
                    if let Err(e) = &result {
                        self.failed.insert(issue.clone(), e.clone());
                    }
                    events.push(Event::Verified {
                        issue,
                        passed: result.is_ok(),
                    });
                }
            }
        }
        self.running = still;
        // Keep a failure listed while its issue is still where it failed.
        self.failed.retain(|issue, _| {
            world
                .issues
                .get(issue)
                .is_some_and(|i| i.stage == temet_work_core::model::Stage::Deploying)
        });
        let problem = (!self.failed.is_empty()).then(|| {
            self.failed
                .iter()
                .map(|(issue, e)| format!("deploy of {issue} failed: {e}"))
                .collect::<Vec<_>>()
                .join("; ")
        });
        Polled { events, problem }
    }

    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let (issue, step) = match effect {
            Effect::Deploy { issue } => (issue, Step::Deploy),
            Effect::Verify { issue } => (issue, Step::Verify),
            _ => return Outcome::NotMine,
        };
        let Some(project) = world.projects.get(&issue.project) else {
            return Outcome::Failed(format!("project {} is not configured", issue.project));
        };
        let command = match step {
            Step::Deploy => project.deploy.clone(),
            Step::Verify => project.verify.clone(),
        };
        let Some(command) = command else {
            let event = match step {
                Step::Deploy => Event::Deployed {
                    issue: issue.clone(),
                },
                Step::Verify => Event::Verified {
                    issue: issue.clone(),
                    passed: true,
                },
            };
            return Outcome::Done {
                events: vec![event],
                effects: Vec::new(),
            };
        };
        let cwd = project
            .run_in
            .clone()
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("/"));
        let env = vec![
            ("TEMET_WORK_ISSUE".to_string(), issue.to_string()),
            ("TEMET_WORK_PROJECT".to_string(), issue.project.to_string()),
            ("TEMET_WORK_REPO".to_string(), project.repo.clone()),
        ];
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

    /// Polls until the running commands finish; they finish in microseconds.
    fn settle(part: &mut Deploys, w: &World) -> Polled {
        let mut out = Polled::default();
        for _ in 0..1000 {
            let p = part.poll(w, 0);
            out.events.extend(p.events);
            out.problem = p.problem;
            if part.running() == 0 {
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
            Outcome::Done { events, .. } if events == [Event::Verified { issue: issue(), passed: true }]
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
                passed: false
            }]
        );
    }

    #[test]
    fn a_failed_deploy_is_a_problem_while_the_issue_waits_on_it() {
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
        let mut part = Deploys::with(shell());
        part.carry_out(&Effect::Deploy { issue: issue() }, &w);
        let polled = settle(&mut part, &w);
        assert_eq!(polled.events, []);
        assert!(
            polled
                .problem
                .is_some_and(|p| p.contains("deploy of ankh#1 failed"))
        );
    }

    #[test]
    fn runs_nothing_in_the_sandbox() {
        if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
            assert!(sh("true", &PathBuf::from("/"), &[]).is_err());
        }
    }
}
