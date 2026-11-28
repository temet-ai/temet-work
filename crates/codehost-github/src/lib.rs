//! GitHub pull requests and their CI, through the `gh` command, so it uses the
//! sign-in `gh` already has. Tests replay recorded `gh` output through
//! [`Transport`]; the live test reads a public repository and never writes.
//!
//! While `TEMET_WORK_SANDBOX` is set to anything but empty, as it is in every
//! test run, the real transport runs nothing.

use serde_json::Value;
use temet_work_codehost::{CheckRun, Checks, CodeHost, CodeHostError, Pull, PullState, RunState};

/// Runs `gh` with arguments and returns what it printed.
pub trait Transport: Send {
    /// Runs `gh args…`; the error is what `gh` printed to stderr.
    fn gh(&mut self, args: &[&str]) -> Result<String, String>;
}

/// The real `gh`.
pub struct Cli;

impl Transport for Cli {
    fn gh(&mut self, args: &[&str]) -> Result<String, String> {
        if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
            return Err("TEMET_WORK_SANDBOX is set: gh is not run".into());
        }
        let out = std::process::Command::new("gh")
            .args(args)
            .output()
            .map_err(|e| format!("cannot run gh: {e}"))?;
        // `gh pr checks` exits 8 while some check is failing or pending, yet
        // prints every check on stdout; that output is the answer, not an error.
        let checks_pending_or_failed = out.status.code() == Some(8)
            && args.starts_with(&["pr", "checks"])
            && !out.stdout.is_empty();
        if out.status.success() || checks_pending_or_failed {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
}

/// The GitHub adapter.
pub struct GitHub {
    transport: Box<dyn Transport>,
    /// The host `gh` reaches, which the web addresses name.
    host: String,
}

impl Default for GitHub {
    /// Runs the real `gh`, on the host `GH_HOST` names when the daemon has it,
    /// as `gh` does.
    fn default() -> Self {
        Self::with(Box::new(Cli)).host(&std::env::var("GH_HOST").unwrap_or_default())
    }
}

impl GitHub {
    /// A GitHub code host that runs `gh` through `transport`, on github.com.
    pub fn with(transport: Box<dyn Transport>) -> Self {
        Self {
            transport,
            host: "github.com".into(),
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
        self.transport.gh(args).map_err(|e| {
            let lower = e.to_lowercase();
            if lower.contains("could not resolve") || lower.contains("not found") {
                CodeHostError::NotFound(e)
            } else if lower.contains("auth")
                || lower.contains("permission")
                || lower.contains("403")
            {
                CodeHostError::Denied(e)
            } else {
                CodeHostError::Other(e)
            }
        })
    }
}

/// The job number in the address of a GitHub Actions job,
/// `https://HOST/OWNER/REPO/actions/runs/RUN/job/JOB`; none for any other.
fn actions_job(link: &str) -> Option<&str> {
    let (_, tail) = link.split_once("/actions/runs/")?;
    let (run, job) = tail.split_once("/job/")?;
    let job = job.split(['/', '?', '#']).next()?;
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    (digits(run) && digits(job)).then_some(job)
}

/// What a pull request's `statusCheckRollup` says. A failure is reported as soon
/// as one check fails, so the author can start on it while others still run.
pub fn checks_of(rollup: &[Value]) -> Checks {
    if rollup.is_empty() {
        return Checks::None;
    }
    let text = |c: &Value, k: &str| c.get(k).and_then(Value::as_str).map(str::to_string);
    let mut failing = Vec::new();
    let mut pending = false;
    for c in rollup {
        let name = text(c, "name")
            .or_else(|| text(c, "context"))
            .unwrap_or_else(|| "a check without a name".into());
        if text(c, "__typename").as_deref() == Some("StatusContext") {
            match text(c, "state").as_deref() {
                Some("SUCCESS") => {}
                Some("PENDING" | "EXPECTED") => pending = true,
                _ => failing.push(name),
            }
        } else if text(c, "status").as_deref() != Some("COMPLETED") {
            pending = true;
        } else if !matches!(
            text(c, "conclusion").as_deref(),
            Some("SUCCESS" | "NEUTRAL" | "SKIPPED")
        ) {
            failing.push(name);
        }
    }
    if !failing.is_empty() {
        Checks::Failed { failing }
    } else if pending {
        Checks::Pending
    } else {
        Checks::Passed
    }
}

fn parse_pull(json: &str) -> Result<Pull, CodeHostError> {
    let v: Value =
        serde_json::from_str(json).map_err(|e| CodeHostError::Other(format!("gh pr view: {e}")))?;
    let number = v
        .get("number")
        .and_then(Value::as_u64)
        .ok_or_else(|| CodeHostError::Other("gh pr view gave no number".into()))?;
    let state = match v.get("state").and_then(Value::as_str) {
        Some("MERGED") => PullState::Merged,
        Some("CLOSED") => PullState::Closed,
        _ => PullState::Open,
    };
    let rollup = v
        .get("statusCheckRollup")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    Ok(Pull {
        number: number.to_string(),
        state,
        checks: checks_of(rollup),
        conflicting: v.get("mergeable").and_then(Value::as_str) == Some("CONFLICTING"),
        draft: v.get("isDraft").and_then(Value::as_bool) == Some(true),
        branch: v
            .get("headRefName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    })
}

/// What the workflow runs of one commit on the default branch say, read from
/// `gh run list --json name,status,conclusion`. Their words are lower case; the
/// rollup's are upper case.
fn main_checks(json: &str) -> Result<Checks, CodeHostError> {
    let runs: Vec<Value> = serde_json::from_str(json)
        .map_err(|e| CodeHostError::Other(format!("gh run list: {e}")))?;
    let upper = |r: &Value, k: &str| {
        Value::from(
            r.get(k)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_uppercase(),
        )
    };
    let rollup: Vec<Value> = runs
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.get("name").cloned().unwrap_or(Value::Null),
                "status": upper(r, "status"),
                "conclusion": upper(r, "conclusion"),
            })
        })
        .collect();
    Ok(checks_of(&rollup))
}

impl CodeHost for GitHub {
    fn id(&self) -> &'static str {
        "github"
    }

    fn pull(&mut self, repo: &str, number: &str) -> Result<Pull, CodeHostError> {
        let out = self.run(&[
            "pr",
            "view",
            number,
            "-R",
            repo,
            "--json",
            "number,state,statusCheckRollup,mergeable,isDraft,headRefName",
        ])?;
        parse_pull(&out)
    }

    fn merge(&mut self, repo: &str, number: &str, how: &str) -> Result<(), CodeHostError> {
        let flag = match how {
            "squash" => "--squash",
            "merge" => "--merge",
            "rebase" => "--rebase",
            other => {
                return Err(CodeHostError::Other(format!(
                    "no way to merge called {other}; squash, merge or rebase"
                )));
            }
        };
        self.run(&["pr", "merge", number, "-R", repo, flag])
            .map(|_| ())
    }

    fn address(&self, repo: &str) -> Option<String> {
        Some(format!("https://{}/{repo}", self.host))
    }

    fn clone_address(&self, repo: &str) -> Option<String> {
        Some(format!("https://{}/{repo}.git", self.host))
    }

    fn pull_ref(&self, number: &str) -> String {
        format!("refs/pull/{number}/head")
    }

    fn ci(&mut self, repo: &str, number: &str) -> Result<Vec<CheckRun>, CodeHostError> {
        let out = match self.run(&[
            "pr",
            "checks",
            number,
            "-R",
            repo,
            "--json",
            "name,state,bucket,link,description,workflow",
        ]) {
            Ok(out) => out,
            // A pull request that has no checks is not a missing one.
            Err(CodeHostError::Other(e)) if e.to_lowercase().contains("no checks reported") => {
                return Ok(Vec::new());
            }
            Err(e) => return Err(e),
        };
        let checks: Vec<Value> = serde_json::from_str(&out)
            .map_err(|e| CodeHostError::Other(format!("gh pr checks: {e}")))?;
        let mut runs = Vec::new();
        for c in &checks {
            let text = |k: &str| c.get(k).and_then(Value::as_str).filter(|t| !t.is_empty());
            let state = match text("bucket") {
                Some("pass") => RunState::Passed,
                Some("fail") => RunState::Failed,
                Some("skipping") => RunState::Skipped,
                Some("cancel") => RunState::Cancelled,
                _ => RunState::Running,
            };
            let link = text("link");
            // Only a failing check that ran as an Actions job has a log to read.
            // One an outside app reports has none, and says what it can instead.
            // A log that cannot be read leaves the check without it, not the read
            // of every check failed.
            let log = match (state, link.and_then(actions_job)) {
                (RunState::Failed, Some(job)) => self
                    .run(&["run", "view", "--job", job, "-R", repo, "--log-failed"])
                    .ok(),
                _ => None,
            };
            runs.push(CheckRun {
                name: text("name").unwrap_or("a check without a name").to_string(),
                state,
                log,
                summary: text("description").unwrap_or_default().to_string(),
                address: link.map(str::to_string),
            });
        }
        Ok(runs)
    }

    fn main_pipeline(&mut self, repo: &str, number: &str) -> Result<Checks, CodeHostError> {
        let out = self.run(&["pr", "view", number, "-R", repo, "--json", "mergeCommit"])?;
        let v: Value = serde_json::from_str(&out)
            .map_err(|e| CodeHostError::Other(format!("gh pr view: {e}")))?;
        let sha = v
            .pointer("/mergeCommit/oid")
            .and_then(Value::as_str)
            .ok_or_else(|| CodeHostError::Other(format!("pull request {number} has not merged")))?;
        let out = self.run(&[
            "run",
            "list",
            "-R",
            repo,
            "--commit",
            sha,
            "--json",
            "name,status,conclusion",
        ])?;
        main_checks(&out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Replays `gh` output recorded from temet-ai/radar on 26 November 2026.
    struct Recorded;

    impl Transport for Recorded {
        fn gh(&mut self, args: &[&str]) -> Result<String, String> {
            if args.starts_with(&[
                "pr",
                "view",
                "46",
                "-R",
                "temet-ai/radar",
                "--json",
                "mergeCommit",
            ]) {
                return Ok(include_str!("../tests/fixtures/radar-pr-46-merge.json").into());
            }
            if args.starts_with(&["pr", "view", "46", "-R", "temet-ai/radar"]) {
                return Ok(include_str!("../tests/fixtures/radar-pr-46.json").into());
            }
            if args.starts_with(&[
                "run",
                "list",
                "-R",
                "temet-ai/radar",
                "--commit",
                "eb7f7a2ed029b4426d58361d4b10e4d3fff86aca",
            ]) {
                return Ok(include_str!("../tests/fixtures/radar-main-runs.json").into());
            }
            if args.starts_with(&["pr", "checks", "46", "-R", "temet-ai/radar"]) {
                return Ok(include_str!("../tests/fixtures/radar-pr-46-checks.json").into());
            }
            Err("GraphQL: Could not resolve to a PullRequest".into())
        }
    }

    /// Replays `gh` output recorded from temet-ai/ask-kit, pull request 20, on
    /// 28 November 2026: two failing jobs, one whose log is recorded and one
    /// whose log read fails.
    struct AskKit;

    impl Transport for AskKit {
        fn gh(&mut self, args: &[&str]) -> Result<String, String> {
            if args.starts_with(&["pr", "checks", "20", "-R", "temet-ai/ask-kit"]) {
                return Ok(include_str!("../tests/fixtures/ask-kit-pr-20-checks.json").into());
            }
            if args
                == [
                    "run",
                    "view",
                    "--job",
                    "137710259021",
                    "-R",
                    "temet-ai/ask-kit",
                    "--log-failed",
                ]
            {
                return Ok(include_str!(
                    "../tests/fixtures/ask-kit-job-137710259021-log-failed.txt"
                )
                .into());
            }
            Err("failed to get run log: log not found".into())
        }
    }

    struct ReadOnly;

    impl temet_work_codehost::contract::Harness for ReadOnly {
        fn repo(&self) -> String {
            "temet-ai/radar".into()
        }
        fn pull(&self) -> String {
            "46".into()
        }
        fn mergeable(&self) -> Option<String> {
            None
        }
    }

    #[test]
    fn the_code_is_on_github_or_the_host_gh_host_names() {
        let at = |gh: GitHub| gh.address("ankh-morpork/watch");
        assert_eq!(
            at(GitHub::with(Box::new(Recorded))).as_deref(),
            Some("https://github.com/ankh-morpork/watch")
        );
        for host in ["git.example.com", "https://git.example.com/"] {
            assert_eq!(
                at(GitHub::with(Box::new(Recorded)).host(host)).as_deref(),
                Some("https://git.example.com/ankh-morpork/watch")
            );
        }
        assert_eq!(
            at(GitHub::with(Box::new(Recorded)).host("")).as_deref(),
            Some("https://github.com/ankh-morpork/watch")
        );
    }

    #[test]
    fn only_conflicting_counts_as_a_conflict_and_unknown_does_not() {
        for (mergeable, want) in [
            ("CONFLICTING", true),
            ("MERGEABLE", false),
            ("UNKNOWN", false),
        ] {
            let json = json!({"number": 1, "state": "OPEN", "mergeable": mergeable}).to_string();
            assert_eq!(
                parse_pull(&json).expect("parses").conflicting,
                want,
                "{mergeable}"
            );
        }
    }

    #[test]
    fn reads_whether_a_pull_request_is_a_draft() {
        for (draft, want) in [(Some(true), true), (Some(false), false), (None, false)] {
            let mut json = json!({"number": 1, "state": "OPEN"});
            if let Some(draft) = draft {
                json["isDraft"] = draft.into();
            }
            let pull = parse_pull(&json.to_string()).expect("parses");
            assert_eq!(pull.draft, want, "{draft:?}");
        }
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        // The Spy prints nothing, so only what it was asked matters.
        let _ = GitHub::with(Box::new(Spy(seen.clone()))).pull("temet-ai/radar", "46");
        let ran = seen.lock().expect("lock").clone();
        let fields = ran[0]
            .iter()
            .skip_while(|a| *a != "--json")
            .nth(1)
            .expect("a --json field list");
        assert!(fields.split(',').any(|f| f == "isDraft"), "{fields}");
    }

    #[test]
    fn reads_a_pull_requests_branch() {
        let pull = GitHub::with(Box::new(Recorded))
            .pull("temet-ai/radar", "46")
            .expect("recorded");
        assert_eq!(pull.branch, "docs/installation-and-guide-audit");
        let json = json!({"number": 1, "state": "OPEN", "headRefName": "watch-12-night-shift"});
        assert_eq!(
            parse_pull(&json.to_string()).expect("parses").branch,
            "watch-12-night-shift"
        );
        let none = json!({"number": 1, "state": "OPEN"}).to_string();
        assert_eq!(parse_pull(&none).expect("parses").branch, "");
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let _ = GitHub::with(Box::new(Spy(seen.clone()))).pull("temet-ai/radar", "46");
        let ran = seen.lock().expect("lock").clone();
        let fields = ran[0]
            .iter()
            .skip_while(|a| *a != "--json")
            .nth(1)
            .expect("a --json field list");
        assert!(fields.split(',').any(|f| f == "headRefName"), "{fields}");
    }

    #[test]
    fn passes_the_code_host_contract_on_recorded_output() {
        let mut gh = GitHub::with(Box::new(Recorded));
        assert_eq!(
            temet_work_codehost::contract::check(&mut gh, &ReadOnly),
            Ok(())
        );
    }

    #[test]
    fn reads_a_merged_pull_request_whose_checks_passed() {
        let pull = GitHub::with(Box::new(Recorded))
            .pull("temet-ai/radar", "46")
            .expect("recorded");
        assert_eq!(pull.state, PullState::Merged);
        assert_eq!(pull.checks, Checks::Passed);
    }

    #[test]
    fn reads_the_main_pipeline_of_the_commit_a_pull_request_merged_as() {
        let mut gh = GitHub::with(Box::new(Recorded));
        assert_eq!(gh.main_pipeline("temet-ai/radar", "46"), Ok(Checks::Passed));
        assert!(gh.main_pipeline("temet-ai/radar", "999999999").is_err());
    }

    #[test]
    fn a_running_or_failed_main_run_is_pending_or_failed() {
        let runs = |status: &str, conclusion: &str| {
            format!(r#"[{{"name":"CI","status":"{status}","conclusion":"{conclusion}"}}]"#)
        };
        assert_eq!(main_checks(&runs("in_progress", "")), Ok(Checks::Pending));
        assert_eq!(
            main_checks(&runs("completed", "failure")),
            Ok(Checks::Failed {
                failing: vec!["CI".into()]
            })
        );
        assert_eq!(main_checks("[]"), Ok(Checks::None));
    }

    #[test]
    fn reads_ci_states_from_check_runs_and_status_contexts() {
        let run = |name: &str, status: &str, conclusion: &str| json!({"__typename": "CheckRun", "name": name, "status": status, "conclusion": conclusion});
        let ctx = |name: &str, state: &str| json!({"__typename": "StatusContext", "context": name, "state": state});
        assert_eq!(checks_of(&[]), Checks::None);
        assert_eq!(
            checks_of(&[
                run("rust", "COMPLETED", "SUCCESS"),
                run("docs", "COMPLETED", "SKIPPED")
            ]),
            Checks::Passed
        );
        assert_eq!(
            checks_of(&[run("rust", "IN_PROGRESS", ""), ctx("deploy", "SUCCESS")]),
            Checks::Pending
        );
        assert_eq!(
            checks_of(&[
                run("rust", "IN_PROGRESS", ""),
                run("docs", "COMPLETED", "FAILURE"),
                ctx("deploy", "ERROR")
            ]),
            Checks::Failed {
                failing: vec!["docs".into(), "deploy".into()]
            }
        );
    }

    /// Remembers the arguments it was run with.
    struct Spy(std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>);

    impl Transport for Spy {
        fn gh(&mut self, args: &[&str]) -> Result<String, String> {
            self.0
                .lock()
                .expect("lock")
                .push(args.iter().map(ToString::to_string).collect());
            Ok(String::new())
        }
    }

    #[test]
    fn the_merge_setting_picks_the_flag_gh_gets() {
        for (how, flag) in [
            ("squash", "--squash"),
            ("merge", "--merge"),
            ("rebase", "--rebase"),
        ] {
            let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let mut gh = GitHub::with(Box::new(Spy(seen.clone())));
            gh.merge("temet-ai/radar", "46", how).expect("merges");
            let ran = seen.lock().expect("lock").clone();
            assert_eq!(
                ran,
                vec![vec!["pr", "merge", "46", "-R", "temet-ai/radar", flag]]
            );
        }
        let mut gh = GitHub::with(Box::new(Recorded));
        assert!(
            gh.merge("temet-ai/radar", "46", "fast-forward")
                .is_err_and(|e| e.to_string().contains("squash, merge or rebase"))
        );
    }

    fn ask_kit_runs() -> Vec<CheckRun> {
        GitHub::with(Box::new(AskKit))
            .ci("temet-ai/ask-kit", "20")
            .expect("recorded")
    }

    #[test]
    fn the_recorded_checks_read_as_their_states() {
        let runs = ask_kit_runs();
        assert_eq!(runs.len(), 8);
        let failed: Vec<&str> = runs
            .iter()
            .filter(|r| r.state == RunState::Failed)
            .map(|r| r.name.as_str())
            .collect();
        assert_eq!(
            failed,
            ["test (macos-latest, 22.x)", "test (macos-latest, 24.x)"]
        );
        assert_eq!(
            runs.iter().filter(|r| r.state == RunState::Passed).count(),
            6
        );
        assert_eq!(
            runs[4].address.as_deref(),
            Some("https://github.com/temet-ai/ask-kit/actions/runs/45179774991/job/137710259021")
        );
        let radar = GitHub::with(Box::new(Recorded))
            .ci("temet-ai/radar", "46")
            .expect("recorded");
        assert_eq!(radar.len(), 5);
        assert!(
            radar
                .iter()
                .all(|r| r.state == RunState::Passed && r.log.is_none())
        );
    }

    #[test]
    fn a_failing_job_carries_its_whole_recorded_log_and_a_passing_one_none() {
        let runs = ask_kit_runs();
        let log = include_str!("../tests/fixtures/ask-kit-job-137710259021-log-failed.txt");
        assert_eq!(runs[4].log.as_deref(), Some(log));
        assert!(log.contains("AssertionError"));
        assert_eq!(runs.iter().filter(|r| r.log.is_some()).count(), 1);
    }

    #[test]
    fn a_log_that_cannot_be_read_keeps_the_check_with_its_address() {
        let runs = ask_kit_runs();
        assert_eq!(runs[5].state, RunState::Failed);
        assert_eq!(runs[5].log, None);
        assert!(
            runs[5]
                .address
                .as_deref()
                .is_some_and(|a| a.ends_with("/job/137710258908"))
        );
    }

    /// Answers every call with `reply` and remembers what it was asked.
    struct Fixed(
        Result<String, String>,
        std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
    );

    impl Transport for Fixed {
        fn gh(&mut self, args: &[&str]) -> Result<String, String> {
            self.1
                .lock()
                .expect("lock")
                .push(args.iter().map(ToString::to_string).collect());
            self.0.clone()
        }
    }

    #[test]
    fn a_failing_status_an_outside_app_reports_has_no_log_but_its_summary_and_address() {
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let json = r#"[{"bucket":"fail","state":"FAILURE","name":"deploy/preview","workflow":"","description":"Preview build broke","link":"https://ci.example.com/builds/12"}]"#;
        let runs = GitHub::with(Box::new(Fixed(Ok(json.into()), seen.clone())))
            .ci("ankh-morpork/watch", "12")
            .expect("reads");
        assert_eq!(
            runs,
            vec![CheckRun {
                name: "deploy/preview".into(),
                state: RunState::Failed,
                log: None,
                summary: "Preview build broke".into(),
                address: Some("https://ci.example.com/builds/12".into()),
            }]
        );
        // It asked for the checks only, not for a log.
        assert_eq!(seen.lock().expect("lock").len(), 1);
    }

    #[test]
    fn every_bucket_has_a_state() {
        let one = |bucket: &str| {
            let json =
                format!(r#"[{{"bucket":"{bucket}","name":"x","link":"","description":""}}]"#);
            let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            GitHub::with(Box::new(Fixed(Ok(json), seen)))
                .ci("a/b", "1")
                .expect("reads")[0]
                .state
        };
        assert_eq!(one("pass"), RunState::Passed);
        assert_eq!(one("fail"), RunState::Failed);
        assert_eq!(one("pending"), RunState::Running);
        assert_eq!(one("skipping"), RunState::Skipped);
        assert_eq!(one("cancel"), RunState::Cancelled);
        assert_eq!(one("something new"), RunState::Running);
    }

    #[test]
    fn no_checks_is_an_empty_list_and_a_missing_pull_request_an_error() {
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let none = Fixed(
            Err("no checks reported on the 'watch-12' branch".into()),
            seen,
        );
        assert_eq!(GitHub::with(Box::new(none)).ci("a/b", "1"), Ok(Vec::new()));
        assert!(
            GitHub::with(Box::new(Recorded))
                .ci("temet-ai/radar", "999999999")
                .is_err()
        );
    }

    #[test]
    fn only_an_actions_job_address_names_a_job() {
        let job = "https://github.com/temet-ai/radar/actions/runs/39757830147/job/119754241492";
        assert_eq!(actions_job(job), Some("119754241492"));
        assert_eq!(actions_job(&format!("{job}?pr=46")), Some("119754241492"));
        for other in [
            "https://ci.example.com/builds/12",
            "https://github.com/temet-ai/radar/actions/runs/39757830147",
            "",
        ] {
            assert_eq!(actions_job(other), None, "{other}");
        }
    }

    #[test]
    fn runs_nothing_in_the_sandbox() {
        if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
            assert!(Cli.gh(&["--version"]).is_err());
        }
    }
}
