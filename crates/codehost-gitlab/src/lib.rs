//! GitLab merge requests and their pipelines, through the `glab` command, so it
//! uses the sign-in `glab` already has. Tests replay recorded `glab` output
//! through [`Transport`]; the live test reads a public project and never writes.
//!
//! While `TEMET_WORK_SANDBOX` is set to anything but empty, as it is in every
//! test run, the real transport runs nothing.

use serde_json::Value;
use temet_work_codehost::{Checks, CodeHost, CodeHostError, Pull, PullState};

/// Runs `glab` with arguments and returns what it printed.
pub trait Transport: Send {
    /// Runs `glab args…`; the error is what `glab` printed to stderr.
    fn glab(&mut self, args: &[&str]) -> Result<String, String>;
}

/// The real `glab`.
pub struct Cli;

impl Transport for Cli {
    fn glab(&mut self, args: &[&str]) -> Result<String, String> {
        if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
            return Err("TEMET_WORK_SANDBOX is set: glab is not run".into());
        }
        let out = std::process::Command::new("glab")
            .args(args)
            .output()
            .map_err(|e| format!("cannot run glab: {e}"))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            let err = String::from_utf8_lossy(&out.stderr);
            let err = if err.trim().is_empty() {
                String::from_utf8_lossy(&out.stdout)
            } else {
                err
            };
            Err(err.trim().to_string())
        }
    }
}

/// The GitLab adapter.
pub struct GitLab {
    transport: Box<dyn Transport>,
}

impl Default for GitLab {
    fn default() -> Self {
        Self::with(Box::new(Cli))
    }
}

/// A repository's full path as one API path segment: `group/sub/project`
/// becomes `group%2Fsub%2Fproject`.
fn encode(repo: &str) -> String {
    let mut out = String::new();
    for b in repo.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

impl GitLab {
    /// A GitLab code host that runs `glab` through `transport`.
    pub fn with(transport: Box<dyn Transport>) -> Self {
        Self { transport }
    }

    fn run(&mut self, args: &[&str]) -> Result<String, CodeHostError> {
        self.transport.glab(args).map_err(|e| {
            let lower = e.to_lowercase();
            // A status stands alone: a number inside a URL is not one.
            let status = |code: &str| {
                lower
                    .split_whitespace()
                    .any(|w| w.trim_matches(|c: char| !c.is_ascii_alphanumeric()) == code)
            };
            if status("404") || lower.contains("not found") {
                CodeHostError::NotFound(e)
            } else if status("401")
                || status("403")
                || lower.contains("permission")
                || lower.contains("unauthorized")
            {
                CodeHostError::Denied(e)
            } else {
                CodeHostError::Other(e)
            }
        })
    }
}

/// What a pipeline's status and jobs say. A failure is reported as soon as one
/// job that may not fail has failed, so the author can start on it while others
/// still run. `name` is what a pipeline that failed with no failed job is called.
pub fn checks_of(status: &str, jobs: &[Value], name: &str) -> Checks {
    let failing: Vec<String> = jobs
        .iter()
        .filter(|j| j.get("status").and_then(Value::as_str) == Some("failed"))
        .filter(|j| j.get("allow_failure").and_then(Value::as_bool) != Some(true))
        .map(|j| {
            j.get("name")
                .and_then(Value::as_str)
                .unwrap_or("a job without a name")
                .to_string()
        })
        .collect();
    if !failing.is_empty() {
        return Checks::Failed { failing };
    }
    match status {
        "success" | "manual" | "skipped" => Checks::Passed,
        "failed" | "canceled" => Checks::Failed {
            failing: vec![name.to_string()],
        },
        // running, pending, created, waiting_for_resource, preparing, scheduled
        _ => Checks::Pending,
    }
}

fn parse_error(what: &str, e: impl std::fmt::Display) -> CodeHostError {
    CodeHostError::Other(format!("{what}: {e}"))
}

impl CodeHost for GitLab {
    fn id(&self) -> &'static str {
        "gitlab"
    }

    fn pull(&mut self, repo: &str, number: &str) -> Result<Pull, CodeHostError> {
        let project = encode(repo);
        let out = self.run(&[
            "api",
            &format!("projects/{project}/merge_requests/{number}"),
        ])?;
        let v: Value = serde_json::from_str(&out).map_err(|e| parse_error("glab api", e))?;
        let iid = v
            .get("iid")
            .and_then(Value::as_u64)
            .ok_or_else(|| CodeHostError::Other("glab api gave no iid".into()))?;
        let state = match v.get("state").and_then(Value::as_str) {
            Some("merged") => PullState::Merged,
            Some("closed" | "locked") => PullState::Closed,
            _ => PullState::Open,
        };
        let pipeline = v
            .get("head_pipeline")
            .filter(|p| p.is_object())
            .or_else(|| v.get("pipeline").filter(|p| p.is_object()));
        let checks = match pipeline {
            None => Checks::None,
            Some(p) => {
                let status = p.get("status").and_then(Value::as_str).unwrap_or("pending");
                let id = p.get("id").and_then(Value::as_u64);
                let jobs = match (id, status) {
                    // Every job passed or was skipped: nothing to name.
                    (Some(_), "success" | "manual" | "skipped") | (None, _) => Vec::new(),
                    (Some(id), _) => {
                        // A merge request from a fork runs its pipeline in the fork.
                        let runs_in = match p.get("project_id").and_then(Value::as_u64) {
                            Some(other)
                                if v.get("source_project_id") != v.get("target_project_id")
                                    && v.get("source_project_id").and_then(Value::as_u64)
                                        == Some(other) =>
                            {
                                other.to_string()
                            }
                            _ => project.clone(),
                        };
                        let out = self.run(&[
                            "api",
                            &format!(
                                "projects/{runs_in}/pipelines/{id}/jobs?scope[]=failed&per_page=100"
                            ),
                        ])?;
                        serde_json::from_str::<Vec<Value>>(&out)
                            .map_err(|e| parse_error("glab api jobs", e))?
                    }
                };
                let name = match (p.get("name").and_then(Value::as_str), id) {
                    (Some(name), _) if !name.is_empty() => name.to_string(),
                    (_, Some(id)) => format!("pipeline {id}"),
                    _ => "the pipeline".into(),
                };
                checks_of(status, &jobs, &name)
            }
        };
        Ok(Pull {
            number: iid.to_string(),
            state,
            checks,
            conflicting: v.get("has_conflicts").and_then(Value::as_bool) == Some(true),
        })
    }

    fn merge(&mut self, repo: &str, number: &str, how: &str) -> Result<(), CodeHostError> {
        let flag = match how {
            "squash" => Some("--squash"),
            "merge" => None,
            "rebase" => Some("--rebase"),
            other => {
                return Err(CodeHostError::Other(format!(
                    "no way to merge called {other}; squash, merge or rebase"
                )));
            }
        };
        // Without `--auto-merge=false`, `glab` leaves a request whose pipeline
        // still runs to be merged later and reports success.
        let mut args = vec![
            "mr",
            "merge",
            number,
            "-R",
            repo,
            "--yes",
            "--auto-merge=false",
        ];
        args.extend(flag);
        self.run(&args).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    const REPO: &str = "example-group/example-project";

    /// Replays `glab` output made up in the shape `glab api` prints.
    struct Recorded;

    impl Transport for Recorded {
        fn glab(&mut self, args: &[&str]) -> Result<String, String> {
            let base = "projects/example-group%2Fexample-project";
            if args == ["api", &format!("{base}/merge_requests/7")] {
                return Ok(include_str!("../tests/fixtures/mr-7-merged.json").into());
            }
            if args == ["api", &format!("{base}/merge_requests/8")] {
                return Ok(include_str!("../tests/fixtures/mr-8-failed.json").into());
            }
            if args
                == [
                    "api",
                    &format!("{base}/pipelines/9002/jobs?scope[]=failed&per_page=100"),
                ]
            {
                return Ok(include_str!("../tests/fixtures/pipeline-9002-jobs.json").into());
            }
            Err("glab: 404 Not Found".into())
        }
    }

    struct ReadOnly;

    impl temet_work_codehost::contract::Harness for ReadOnly {
        fn repo(&self) -> String {
            REPO.into()
        }
        fn pull(&self) -> String {
            "7".into()
        }
        fn mergeable(&self) -> Option<String> {
            None
        }
    }

    type Seen = Arc<Mutex<Vec<Vec<String>>>>;

    /// Remembers the arguments it was run with and answers with `reply`.
    struct Spy(Seen, Result<String, String>);

    impl Transport for Spy {
        fn glab(&mut self, args: &[&str]) -> Result<String, String> {
            self.0
                .lock()
                .expect("lock")
                .push(args.iter().map(ToString::to_string).collect());
            self.1.clone()
        }
    }

    fn job(name: &str, status: &str, allow: bool) -> Value {
        json!({"name": name, "status": status, "allow_failure": allow})
    }

    #[test]
    fn passes_the_code_host_contract_on_recorded_output() {
        let mut host = GitLab::with(Box::new(Recorded));
        assert_eq!(
            temet_work_codehost::contract::check(&mut host, &ReadOnly),
            Ok(())
        );
    }

    #[test]
    fn reads_a_merged_request_whose_pipeline_passed() {
        let pull = GitLab::with(Box::new(Recorded))
            .pull(REPO, "7")
            .expect("recorded");
        assert_eq!(pull.number, "7", "the iid, not the global id");
        assert_eq!(pull.state, PullState::Merged);
        assert_eq!(pull.checks, Checks::Passed);
        assert!(!pull.conflicting);
    }

    #[test]
    fn reads_an_open_request_with_a_failed_job() {
        let pull = GitLab::with(Box::new(Recorded))
            .pull(REPO, "8")
            .expect("recorded");
        assert_eq!(pull.state, PullState::Open);
        assert_eq!(
            pull.checks,
            Checks::Failed {
                failing: vec!["unit-tests".into()]
            },
            "the job that may fail is not counted"
        );
    }

    #[test]
    fn asks_for_the_request_with_the_path_encoded() {
        let seen = Seen::default();
        let mut host = GitLab::with(Box::new(Spy(seen.clone(), Err("boom".into()))));
        let _ = host.pull("group/sub/project", "12");
        assert_eq!(
            *seen.lock().expect("lock"),
            vec![vec![
                "api",
                "projects/group%2Fsub%2Fproject/merge_requests/12"
            ]]
        );
    }

    #[test]
    fn reads_the_request_state_and_conflicts() {
        for (state, want) in [
            ("opened", PullState::Open),
            ("merged", PullState::Merged),
            ("closed", PullState::Closed),
            ("locked", PullState::Closed),
        ] {
            let body = json!({"iid": 3, "id": 999, "state": state, "has_conflicts": true});
            let mut host = GitLab::with(Box::new(Spy(Seen::default(), Ok(body.to_string()))));
            let pull = host.pull(REPO, "3").expect("parses");
            assert_eq!(pull.state, want, "{state}");
            assert_eq!(pull.number, "3");
            assert!(pull.conflicting);
            assert_eq!(pull.checks, Checks::None, "no pipeline");
        }
    }

    #[test]
    fn maps_each_pipeline_status() {
        for (status, want) in [
            ("running", Checks::Pending),
            ("pending", Checks::Pending),
            ("created", Checks::Pending),
            ("waiting_for_resource", Checks::Pending),
            ("waiting", Checks::Pending),
            ("preparing", Checks::Pending),
            ("scheduled", Checks::Pending),
            ("success", Checks::Passed),
            ("manual", Checks::Passed),
            ("skipped", Checks::Passed),
            (
                "failed",
                Checks::Failed {
                    failing: vec!["pipeline 5".into()],
                },
            ),
            (
                "canceled",
                Checks::Failed {
                    failing: vec!["pipeline 5".into()],
                },
            ),
        ] {
            assert_eq!(checks_of(status, &[], "pipeline 5"), want, "{status}");
        }
    }

    #[test]
    fn a_failed_job_is_reported_while_others_still_run() {
        let jobs = [
            job("build", "success", false),
            job("lint", "failed", true),
            job("unit-tests", "failed", false),
            job("deploy", "running", false),
        ];
        assert_eq!(
            checks_of("running", &jobs, "pipeline 5"),
            Checks::Failed {
                failing: vec!["unit-tests".into()]
            }
        );
        assert_eq!(
            checks_of("running", &jobs[..2], "pipeline 5"),
            Checks::Pending,
            "a job allowed to fail does not fail the pipeline"
        );
    }

    /// Answers a request from a fork with a failed pipeline, then its jobs.
    struct Fork(Seen);

    impl Transport for Fork {
        fn glab(&mut self, args: &[&str]) -> Result<String, String> {
            self.0
                .lock()
                .expect("lock")
                .push(args.iter().map(ToString::to_string).collect());
            Ok(if args[1].contains("/jobs") {
                "[]".into()
            } else {
                json!({"iid": 4, "state": "opened", "source_project_id": 77, "target_project_id": 5501,
                    "head_pipeline": {"id": 9, "project_id": 77, "status": "failed", "name": "nightly"}})
                .to_string()
            })
        }
    }

    #[test]
    fn a_fork_pipeline_is_read_in_the_fork_and_named_when_no_job_is() {
        let seen = Seen::default();
        let pull = GitLab::with(Box::new(Fork(seen.clone())))
            .pull(REPO, "4")
            .expect("parses");
        assert_eq!(
            pull.checks,
            Checks::Failed {
                failing: vec!["nightly".into()]
            }
        );
        assert_eq!(
            seen.lock().expect("lock")[1][1],
            "projects/77/pipelines/9/jobs?scope[]=failed&per_page=100"
        );
    }

    #[test]
    fn the_merge_setting_picks_the_arguments_glab_gets() {
        for (how, extra) in [
            ("squash", Some("--squash")),
            ("merge", None),
            ("rebase", Some("--rebase")),
        ] {
            let seen = Seen::default();
            let mut host = GitLab::with(Box::new(Spy(seen.clone(), Ok(String::new()))));
            host.merge(REPO, "7", how).expect("merges");
            let mut want = vec![
                "mr",
                "merge",
                "7",
                "-R",
                REPO,
                "--yes",
                "--auto-merge=false",
            ];
            want.extend(extra);
            assert_eq!(*seen.lock().expect("lock"), vec![want], "{how}");
        }
        let mut host = GitLab::with(Box::new(Recorded));
        assert!(
            host.merge(REPO, "7", "fast-forward")
                .is_err_and(|e| e.to_string().contains("squash, merge or rebase"))
        );
    }

    #[test]
    fn glab_errors_map_to_the_contract_errors() {
        let read = |msg: &str| {
            GitLab::with(Box::new(Spy(Seen::default(), Err(msg.into()))))
                .pull(REPO, "1")
                .expect_err("fails")
        };
        assert!(matches!(
            read("GET https://gitlab.example.com/api/v4/x: 404 Not Found"),
            CodeHostError::NotFound(_)
        ));
        for msg in ["merge request not found", "glab: 404 Nothing (HTTP 404)"] {
            assert!(matches!(read(msg), CodeHostError::NotFound(_)), "{msg}");
        }
        let url = "Get https://gitlab.example.com/api/v4/merge_requests/403: timed out";
        assert!(matches!(read(url), CodeHostError::Other(_)));
        for msg in [
            "401 Unauthorized",
            "403 Forbidden",
            "you do not have permission",
        ] {
            assert!(matches!(read(msg), CodeHostError::Denied(_)), "{msg}");
        }
        assert_eq!(
            read("connection reset"),
            CodeHostError::Other("connection reset".into())
        );
    }

    #[test]
    fn runs_nothing_in_the_sandbox() {
        if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
            assert!(Cli.glab(&["--version"]).is_err());
        }
    }
}
