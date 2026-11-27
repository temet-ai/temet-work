//! GitHub issues and milestones, through the `gh` command, so it uses the
//! sign-in `gh` already has. Tests replay `gh`'s recorded output through
//! [`Transport`]; the live test reads a public repository and never writes.
//!
//! While `TEMET_WORK_SANDBOX` is set to anything but empty, as it is in every
//! test run, the real transport runs nothing.

use serde_json::Value;
use temet_work_tracker::{
    TrackedIssue, TrackedMilestone, Tracker, TrackerError, depends_on, is_date,
};

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

/// The most open issues read from one repository.
const LIMIT: usize = 1000;
const LIMIT_ARG: &str = "1000";

/// The host GitHub is on when `GH_HOST` names none.
const GITHUB: &str = "github.com";

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
    /// A GitHub tracker that runs `gh` through `transport`, on github.com.
    pub fn with(transport: Box<dyn Transport>) -> Self {
        Self {
            transport,
            host: GITHUB.into(),
        }
    }

    /// The same tracker on `host`, such as a GitHub Enterprise server: a name,
    /// or an address whose scheme is dropped. An empty one leaves it as it was.
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
                // GitHub gives a time (`2026-12-12T08:00:00Z`); the date is its day.
                due: v
                    .get("due")
                    .and_then(Value::as_str)
                    .and_then(|d| d.get(..10))
                    .filter(|d| is_date(d))
                    .map(String::from),
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
            LIMIT_ARG,
            "--json",
            "number,title,body,milestone,state",
        ])?;
        let issues = parse_issues(&out)?;
        // A full page may be cut short; a cut-short list would read as issues closed.
        if issues.len() >= LIMIT {
            return Err(TrackerError::Other(format!(
                "{project} has {LIMIT} or more open issues; temet-work reads at most {LIMIT}"
            )));
        }
        Ok(issues)
    }

    fn milestones(&mut self, project: &str) -> Result<Vec<TrackedMilestone>, TrackerError> {
        let path = format!("repos/{project}/milestones?state=open");
        let out = self.run(&[
            "api",
            &path,
            "--paginate",
            "--jq",
            ".[] | {title: .title, open: .open_issues, due: .due_on}",
        ])?;
        parse_milestones(&out)
    }

    fn address(&self, project: &str) -> Option<String> {
        Some(format!("https://{}/{project}/issues", self.host))
    }

    fn closed(&mut self, project: &str, key: &str) -> Result<bool, TrackerError> {
        let out = self.run(&["issue", "view", key, "-R", project, "--json", "state"])?;
        let v: Value = serde_json::from_str(&out)
            .map_err(|e| TrackerError::Other(format!("gh issue view: {e}")))?;
        match v.get("state").and_then(Value::as_str) {
            Some("CLOSED") => Ok(true),
            Some(_) => Ok(false),
            None => Err(TrackerError::Other("gh issue view gave no state".into())),
        }
    }

    fn comment(&mut self, project: &str, key: &str, text: &str) -> Result<(), TrackerError> {
        self.run(&["issue", "comment", key, "-R", project, "--body", text])
            .map(|_| ())
    }

    fn close(&mut self, project: &str, key: &str) -> Result<(), TrackerError> {
        match self.run(&["issue", "close", key, "-R", project]) {
            // A pull request that said "Closes #N" closed it on merge.
            Err(TrackerError::Other(e)) if e.to_lowercase().contains("already closed") => Ok(()),
            other => other.map(|_| ()),
        }
    }

    fn create(
        &mut self,
        project: &str,
        title: &str,
        body: &str,
        milestone: Option<&str>,
    ) -> Result<TrackedIssue, TrackerError> {
        let mut args = vec![
            "issue", "create", "-R", project, "--title", title, "--body", body,
        ];
        if let Some(m) = milestone {
            args.extend(["--milestone", m]);
        }
        let out = self.run(&args)?;
        // gh prints the new issue's address, which ends in its number.
        let number = out
            .lines()
            .rev()
            .find_map(|l| l.trim().rsplit('/').next()?.parse::<u64>().ok())
            .ok_or_else(|| {
                TrackerError::Other(format!("gh issue create named no issue: {}", out.trim()))
            })?;
        Ok(TrackedIssue {
            key: number.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            milestone: milestone.map(String::from),
            depends_on: depends_on(body),
            order: number,
            closed: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Replays `gh` output recorded from temet-ai/radar on 26 November 2026;
    /// its milestones with their due dates on 27 November 2026.
    struct Recorded;

    impl Transport for Recorded {
        fn gh(&mut self, args: &[&str]) -> Result<String, String> {
            match args.first().copied() {
                Some("issue")
                    if args.get(1) == Some(&"view") && args.contains(&"temet-ai/radar") =>
                {
                    // Recorded: every listed issue is open; anything else is unknown.
                    let listed = ["11", "12", "19", "43", "44"];
                    if listed.contains(&args[2]) {
                        Ok(r#"{"state":"OPEN"}"#.into())
                    } else {
                        Err("GraphQL: Could not resolve to an issue or pull request".into())
                    }
                }
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
    fn a_milestones_due_date_is_the_day_of_github_due_on_and_none_without_one() {
        let due = |jsonl: &str| -> Vec<(String, Option<String>)> {
            parse_milestones(jsonl)
                .expect("milestones")
                .into_iter()
                .map(|m| (m.title, m.due))
                .collect()
        };
        assert_eq!(
            due(include_str!("../tests/fixtures/radar-milestones.jsonl")),
            [("Slack sign-in rework".to_string(), None)]
        );
        assert_eq!(
            due(include_str!("../tests/fixtures/milestones-due.jsonl")),
            [
                ("The Clacks".to_string(), Some("2026-12-12".to_string())),
                ("Snuff".to_string(), None)
            ]
        );
    }

    #[test]
    fn the_issues_address_is_on_github_or_the_host_gh_host_names() {
        let gh = GitHub::with(Box::new(Recorded));
        assert_eq!(
            gh.address("ankh-morpork/watch").as_deref(),
            Some("https://github.com/ankh-morpork/watch/issues")
        );
        for host in ["git.example.com", "https://git.example.com/"] {
            let gh = GitHub::with(Box::new(Recorded)).host(host);
            assert_eq!(
                gh.address("ankh-morpork/watch").as_deref(),
                Some("https://git.example.com/ankh-morpork/watch/issues")
            );
        }
        let gh = GitHub::with(Box::new(Recorded)).host("");
        assert_eq!(
            gh.address("ankh-morpork/watch").as_deref(),
            Some("https://github.com/ankh-morpork/watch/issues")
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

    /// The recorded output, with an issue filed through it listed afterwards as
    /// GitHub lists it.
    #[derive(Clone, Default)]
    struct Filing {
        created: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
    }

    impl Transport for Filing {
        fn gh(&mut self, args: &[&str]) -> Result<String, String> {
            let mut created = self.created.lock().map_err(|e| e.to_string())?;
            if args.starts_with(&["issue", "create"]) {
                created.push(args.iter().map(ToString::to_string).collect());
                return Ok("https://github.com/temet-ai/radar/issues/45\n".into());
            }
            let out = Recorded.gh(args)?;
            let Some(new) = created.last().filter(|_| args.get(1) == Some(&"list")) else {
                return Ok(out);
            };
            let arg = |flag: &str| {
                new.iter()
                    .position(|a| a == flag)
                    .and_then(|i| new.get(i + 1))
                    .cloned()
            };
            let mut list: Value = serde_json::from_str(&out).map_err(|e| e.to_string())?;
            if let Some(items) = list.as_array_mut() {
                items.push(serde_json::json!({
                    "number": 45,
                    "title": arg("--title"),
                    "body": arg("--body"),
                    "milestone": arg("--milestone").map(|m| serde_json::json!({"title": m})),
                    "state": "OPEN",
                }));
            }
            Ok(list.to_string())
        }
    }

    struct MayFile;

    impl temet_work_tracker::contract::Harness for MayFile {
        fn project(&self) -> String {
            "temet-ai/radar".into()
        }
        fn writable_issue(&self) -> Option<String> {
            None
        }
        fn may_create(&self) -> bool {
            true
        }
    }

    #[test]
    fn creates_an_issue_in_its_milestone_on_recorded_output() {
        let filing = Filing::default();
        let mut gh = GitHub::with(Box::new(filing.clone()));
        assert_eq!(
            temet_work_tracker::contract::check(&mut gh, &MayFile),
            Ok(())
        );
        let created = gh
            .create(
                "temet-ai/radar",
                "Mend the clock",
                "Depends on #44",
                Some("Slack sign-in rework"),
            )
            .expect("created");
        assert_eq!(
            (
                created.key.as_str(),
                created.order,
                created.depends_on.as_slice()
            ),
            ("45", 45, ["44".to_string()].as_slice())
        );
        let sent = filing.created.lock().expect("lock");
        assert_eq!(
            sent.last().map(Vec::as_slice),
            Some(
                [
                    "issue",
                    "create",
                    "-R",
                    "temet-ai/radar",
                    "--title",
                    "Mend the clock",
                    "--body",
                    "Depends on #44",
                    "--milestone",
                    "Slack sign-in rework"
                ]
                .map(String::from)
                .as_slice()
            )
        );
        let mut none = GitHub::with(Box::new(Filing::default()));
        none.create("temet-ai/radar", "Mend the clock", "", None)
            .expect("created with no milestone");
    }

    #[test]
    fn runs_nothing_in_the_sandbox() {
        if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
            assert!(Cli.gh(&["--version"]).is_err());
        }
    }
}
