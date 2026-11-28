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
    /// The host `glab` reaches, which the web addresses name.
    host: String,
}

impl Default for GitLab {
    /// Runs the real `glab`, on the host `GITLAB_HOST` names when the daemon has
    /// it, as `glab` does.
    fn default() -> Self {
        Self::with(Box::new(Cli)).host(&std::env::var("GITLAB_HOST").unwrap_or_default())
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
    /// A GitLab code host that runs `glab` through `transport`, on gitlab.com.
    pub fn with(transport: Box<dyn Transport>) -> Self {
        Self {
            transport,
            host: "gitlab.com".into(),
        }
    }

    /// The same code host on `host`: a name, or an address whose scheme is
    /// dropped. An empty one leaves it as it was.
    pub fn host(mut self, host: &str) -> Self {
        let host = host
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');
        if !host.is_empty() {
            self.host = host.to_string();
        }
        self
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

impl GitLab {
    /// The merge request `number` of the project with API path segment `project`.
    fn request(&mut self, project: &str, number: &str) -> Result<Value, CodeHostError> {
        let out = self.run(&[
            "api",
            &format!("projects/{project}/merge_requests/{number}"),
        ])?;
        serde_json::from_str(&out).map_err(|e| parse_error("glab api", e))
    }

    /// The jobs of pipeline `id` that failed, run in the project `runs_in`.
    fn failed_jobs(&mut self, runs_in: &str, id: u64) -> Result<Vec<Value>, CodeHostError> {
        let out = self.run(&[
            "api",
            &format!("projects/{runs_in}/pipelines/{id}/jobs?scope[]=failed&per_page=100"),
        ])?;
        serde_json::from_str(&out).map_err(|e| parse_error("glab api jobs", e))
    }
}

/// What a pipeline is called when it failed with no failed job to name.
fn pipeline_name(pipeline: &Value, id: Option<u64>) -> String {
    match (pipeline.get("name").and_then(Value::as_str), id) {
        (Some(name), _) if !name.is_empty() => name.to_string(),
        (_, Some(id)) => format!("pipeline {id}"),
        _ => "the pipeline".into(),
    }
}

impl CodeHost for GitLab {
    fn id(&self) -> &'static str {
        "gitlab"
    }

    fn pull(&mut self, repo: &str, number: &str) -> Result<Pull, CodeHostError> {
        let project = encode(repo);
        let v = self.request(&project, number)?;
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
                        self.failed_jobs(&runs_in, id)?
                    }
                };
                checks_of(status, &jobs, &pipeline_name(p, id))
            }
        };
        Ok(Pull {
            number: iid.to_string(),
            state,
            checks,
            conflicting: v.get("has_conflicts").and_then(Value::as_bool) == Some(true),
            // Draft merge requests are not read yet.
            draft: false,
            branch: v
                .get("source_branch")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
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

    fn address(&self, repo: &str) -> Option<String> {
        Some(format!("https://{}/{repo}", self.host))
    }

    fn main_pipeline(&mut self, repo: &str, number: &str) -> Result<Checks, CodeHostError> {
        let project = encode(repo);
        let v = self.request(&project, number)?;
        if v.get("state").and_then(Value::as_str) != Some("merged") {
            return Err(CodeHostError::Other(format!(
                "merge request {number} has not merged"
            )));
        }
        // The commit the target branch got: a merge commit, else the squashed
        // one, else (a fast-forward) the request's own last commit.
        let text = |k: &str| v.get(k).and_then(Value::as_str);
        let sha = text("merge_commit_sha")
            .or_else(|| text("squash_commit_sha"))
            .or_else(|| text("sha"))
            .ok_or_else(|| CodeHostError::Other("glab api gave no merged commit".into()))?;
        let branch = text("target_branch")
            .ok_or_else(|| CodeHostError::Other("glab api gave no target branch".into()))?;
        let out = self.run(&[
            "api",
            &format!(
                "projects/{project}/pipelines?sha={sha}&ref={}",
                encode(branch)
            ),
        ])?;
        let pipelines: Vec<Value> =
            serde_json::from_str(&out).map_err(|e| parse_error("glab api pipelines", e))?;
        // Newest first: a pipeline run again decides.
        let Some(p) = pipelines.first() else {
            return Ok(Checks::None);
        };
        let status = p.get("status").and_then(Value::as_str).unwrap_or("pending");
        let id = p.get("id").and_then(Value::as_u64);
        let jobs = match (id, status) {
            (Some(_), "success" | "skipped") | (None, _) => Vec::new(),
            (Some(id), _) => self.failed_jobs(&project, id)?,
        };
        let name = pipeline_name(p, id);
        match checks_of(status, &jobs, &name) {
            // On the main branch a pipeline held at a manual job has not run what
            // comes after it, such as tests of a deploy, and only a person moves it.
            Checks::Passed if status == "manual" => {
                let at = p
                    .get("web_url")
                    .and_then(Value::as_str)
                    .map_or_else(String::new, |url| format!(" ({url})"));
                Err(CodeHostError::Other(format!(
                    "{name} on {branch}{at} is held at a manual job; it passes once someone runs it"
                )))
            }
            checks => Ok(checks),
        }
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
            if args == ["api", &format!("{base}/pipelines?sha=4b1d0c7e&ref=main")] {
                return Ok(include_str!("../tests/fixtures/main-pipelines.json").into());
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
    fn the_code_is_on_gitlab_or_the_host_gitlab_host_names() {
        let at = |host: GitLab| host.address("ankh-morpork/watch/clacks");
        assert_eq!(
            at(GitLab::with(Box::new(Recorded))).as_deref(),
            Some("https://gitlab.com/ankh-morpork/watch/clacks")
        );
        for host in ["git.example.com", "https://git.example.com/"] {
            assert_eq!(
                at(GitLab::with(Box::new(Recorded)).host(host)).as_deref(),
                Some("https://git.example.com/ankh-morpork/watch/clacks")
            );
        }
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
    fn reads_a_pull_requests_branch() {
        let mut host = GitLab::with(Box::new(Recorded));
        assert_eq!(
            host.pull(REPO, "7").expect("recorded").branch,
            "night-watch"
        );
        let body = json!({"iid": 3, "state": "opened", "source_branch": "watch-12-night-shift"});
        let mut host = GitLab::with(Box::new(Spy(Seen::default(), Ok(body.to_string()))));
        assert_eq!(
            host.pull(REPO, "3").expect("parses").branch,
            "watch-12-night-shift"
        );
        let none = json!({"iid": 3, "state": "opened"});
        let mut host = GitLab::with(Box::new(Spy(Seen::default(), Ok(none.to_string()))));
        assert_eq!(host.pull(REPO, "3").expect("parses").branch, "");
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
    fn reads_the_main_pipeline_of_the_commit_a_request_merged_as() {
        let mut host = GitLab::with(Box::new(Recorded));
        assert_eq!(host.main_pipeline(REPO, "7"), Ok(Checks::Passed));
        assert!(host.main_pipeline(REPO, "8").is_err(), "8 has not merged");
    }

    /// A merged request, then the pipelines of its commit.
    struct Main(&'static str);

    impl Transport for Main {
        fn glab(&mut self, args: &[&str]) -> Result<String, String> {
            Ok(if args[1].contains("/jobs") {
                "[]".into()
            } else if args[1].contains("/pipelines?") {
                assert!(args[1].ends_with("pipelines?sha=abc&ref=release%2F2"));
                self.0.into()
            } else {
                json!({"iid": 3, "state": "merged", "merge_commit_sha": null,
                    "squash_commit_sha": "abc", "sha": "head", "target_branch": "release/2"})
                .to_string()
            })
        }
    }

    #[test]
    fn a_main_pipeline_that_is_missing_held_or_failed_does_not_pass() {
        let read = |pipelines| GitLab::with(Box::new(Main(pipelines))).main_pipeline(REPO, "3");
        assert_eq!(read("[]"), Ok(Checks::None));
        assert_eq!(
            read(r#"[{"id":1,"status":"manual","web_url":"https://gitlab.example.com/p/1"}]"#),
            Err(CodeHostError::Other(
                "pipeline 1 on release/2 (https://gitlab.example.com/p/1) is held at a manual \
                 job; it passes once someone runs it"
                    .into()
            ))
        );
        assert_eq!(
            read(r#"[{"id":1,"status":"running"}]"#),
            Ok(Checks::Pending)
        );
        assert_eq!(
            read(r#"[{"id":1,"status":"failed"}]"#),
            Ok(Checks::Failed {
                failing: vec!["pipeline 1".into()]
            })
        );
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
