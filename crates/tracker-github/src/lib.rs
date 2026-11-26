//! GitHub issues and milestones, through the `gh` command, so it uses the
//! sign-in `gh` already has. Tests replay `gh`'s recorded output through
//! [`Transport`]; the live test reads a public repository and never writes.
//!
//! While `TEMET_WORK_SANDBOX` is set to anything but empty, as it is in every
//! test run, the real transport runs nothing.

use serde_json::Value;
use temet_work_tracker::{TrackedIssue, TrackedMilestone, Tracker, TrackerError, depends_on};

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
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
}

/// The GitHub adapter.
pub struct GitHub {
    transport: Box<dyn Transport>,
}

impl Default for GitHub {
    fn default() -> Self {
        Self::with(Box::new(Cli))
    }
}

impl GitHub {
    /// A GitHub tracker that runs `gh` through `transport`.
    pub fn with(transport: Box<dyn Transport>) -> Self {
        Self { transport }
    }

    fn run(&mut self, args: &[&str]) -> Result<String, TrackerError> {
        self.transport.gh(args).map_err(|e| {
            let lower = e.to_lowercase();
            if lower.contains("could not resolve") || lower.contains("not found") {
                TrackerError::NotFound(e)
            } else if lower.contains("auth")
                || lower.contains("permission")
                || lower.contains("403")
            {
                TrackerError::Denied(e)
            } else {
                TrackerError::Other(e)
            }
        })
    }
}

fn parse_issues(json: &str) -> Result<Vec<TrackedIssue>, TrackerError> {
    let v: Value = serde_json::from_str(json)
        .map_err(|e| TrackerError::Other(format!("gh issue list: {e}")))?;
    let list = v
        .as_array()
        .ok_or_else(|| TrackerError::Other("gh issue list: not a list".into()))?;
    Ok(list
        .iter()
        .filter_map(|i| {
            let number = i.get("number")?.as_u64()?;
            let body = i
                .get("body")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            Some(TrackedIssue {
                key: number.to_string(),
                title: i
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                depends_on: depends_on(&body),
                body,
                milestone: i
                    .pointer("/milestone/title")
                    .and_then(Value::as_str)
                    .map(String::from),
                order: number,
                closed: i.get("state").and_then(Value::as_str) == Some("CLOSED"),
            })
        })
        .collect())
}

fn parse_milestones(jsonl: &str) -> Result<Vec<TrackedMilestone>, TrackerError> {
    jsonl
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let v: Value = serde_json::from_str(l)
                .map_err(|e| TrackerError::Other(format!("milestones: {e}")))?;
            Ok(TrackedMilestone {
                title: v
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                open: v.get("open").and_then(Value::as_u64).unwrap_or(0),
            })
        })
        .collect()
}

impl Tracker for GitHub {
    fn id(&self) -> &'static str {
        "github"
    }

    fn issues(&mut self, project: &str) -> Result<Vec<TrackedIssue>, TrackerError> {
        let out = self.run(&[
            "issue",
            "list",
            "-R",
            project,
            "--state",
            "open",
            "--limit",
            "1000",
            "--json",
            "number,title,body,milestone,state",
        ])?;
        parse_issues(&out)
    }

    fn milestones(&mut self, project: &str) -> Result<Vec<TrackedMilestone>, TrackerError> {
        let path = format!("repos/{project}/milestones?state=open");
        let out = self.run(&[
            "api",
            &path,
            "--paginate",
            "--jq",
            ".[] | {title: .title, open: .open_issues}",
        ])?;
        parse_milestones(&out)
    }

    fn comment(&mut self, project: &str, key: &str, text: &str) -> Result<(), TrackerError> {
        self.run(&["issue", "comment", key, "-R", project, "--body", text])
            .map(|_| ())
    }

    fn close(&mut self, project: &str, key: &str) -> Result<(), TrackerError> {
        self.run(&["issue", "close", key, "-R", project])
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Replays `gh` output recorded from temet-ai/radar on 26 November 2026.
    struct Recorded;

    impl Transport for Recorded {
        fn gh(&mut self, args: &[&str]) -> Result<String, String> {
            match args.first().copied() {
                Some("issue") if args.contains(&"temet-ai/radar") => {
                    Ok(include_str!("../tests/fixtures/radar-issues.json").into())
                }
                Some("api") if args.iter().any(|a| a.starts_with("repos/temet-ai/radar/")) => {
                    Ok(include_str!("../tests/fixtures/radar-milestones.jsonl").into())
                }
                _ => Err("GraphQL: Could not resolve to a Repository".into()),
            }
        }
    }

    struct ReadOnly;

    impl temet_work_tracker::contract::Harness for ReadOnly {
        fn project(&self) -> String {
            "temet-ai/radar".into()
        }
        fn writable_issue(&self) -> Option<String> {
            None
        }
    }

    #[test]
    fn passes_the_tracker_contract_on_recorded_output() {
        let mut gh = GitHub::with(Box::new(Recorded));
        assert_eq!(
            temet_work_tracker::contract::check(&mut gh, &ReadOnly),
            Ok(())
        );
    }

    #[test]
    fn reads_number_title_milestone_and_order() {
        let mut gh = GitHub::with(Box::new(Recorded));
        let issues = gh.issues("temet-ai/radar").expect("recorded issues");
        let first = issues.iter().find(|i| i.key == "44").expect("issue 44");
        assert_eq!(first.milestone.as_deref(), Some("Slack sign-in rework"));
        assert_eq!(first.order, 44);
        assert!(issues.iter().any(|i| i.milestone.is_none()));
    }

    #[test]
    fn runs_nothing_in_the_sandbox() {
        if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
            assert!(Cli.gh(&["--version"]).is_err());
        }
    }
}
