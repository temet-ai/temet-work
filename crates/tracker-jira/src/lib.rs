//! Jira Cloud issues and fix versions, through the REST API v3, called with
//! `curl`. The site and the sign-in come from the environment, never from the
//! configuration: `JIRA_URL` (`https://your-site.atlassian.net`), `JIRA_USER`
//! (the account's email) and `JIRA_TOKEN` (an API token). Tests replay recorded
//! JSON through [`Transport`]; the live test reads one project and never writes.
//!
//! While `TEMET_WORK_SANDBOX` is set to anything but empty, as it is in every
//! test run, the real transport runs nothing.

use serde_json::{Value, json};
use temet_work_tracker::{TrackedIssue, TrackedMilestone, Tracker, TrackerError, depends_on};

/// One call to the Jira REST API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// `GET` or `POST`.
    pub method: &'static str,
    /// The path and query after the site, such as `/rest/api/3/issue/KEY-1`.
    pub path: String,
    /// The JSON body, if any.
    pub body: Option<String>,
}

/// What Jira answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    /// The HTTP status.
    pub status: u16,
    /// The body.
    pub body: String,
}

/// Sends a request to Jira.
pub trait Transport: Send {
    /// Sends `request`. An error is a call that never got an answer.
    fn send(&mut self, request: &Request) -> Result<Reply, TrackerError>;
}

type Env = Box<dyn Fn(&str) -> Option<String> + Send>;

/// The real thing: `curl`, signed in from the environment.
pub struct Curl {
    env: Env,
}

impl Default for Curl {
    fn default() -> Self {
        Self::with_env(Box::new(|name| std::env::var(name).ok()))
    }
}

/// Quotes a value for a curl config file.
fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

impl Curl {
    /// A transport that reads its settings through `env`.
    pub fn with_env(env: Env) -> Self {
        Self { env }
    }

    fn var(&self, name: &str) -> Result<String, TrackerError> {
        self.env(name)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| TrackerError::Denied(format!("{name} is not set")))
    }

    fn env(&self, name: &str) -> Option<String> {
        (self.env)(name)
    }
}

impl Transport for Curl {
    fn send(&mut self, request: &Request) -> Result<Reply, TrackerError> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        if self
            .env("TEMET_WORK_SANDBOX")
            .is_some_and(|v| !v.is_empty())
        {
            return Err(TrackerError::Other(
                "TEMET_WORK_SANDBOX is set: curl is not run".into(),
            ));
        }
        let site = self.var("JIRA_URL")?;
        let user = self.var("JIRA_USER")?;
        let token = self.var("JIRA_TOKEN")?;
        let site = site.trim_end_matches('/');
        if !site.starts_with("https://") {
            return Err(TrackerError::Denied(
                "JIRA_URL must start with https://, so the token is never sent in the clear".into(),
            ));
        }
        // The sign-in goes to curl on its standard input, not in its arguments,
        // where any user on the machine could read it.
        let mut config = format!(
            "url = {}\nuser = {}\nrequest = {}\nheader = \"Accept: application/json\"\n",
            quote(&format!("{site}{}", request.path)),
            quote(&format!("{user}:{token}")),
            quote(request.method),
        );
        if let Some(body) = &request.body {
            config.push_str("header = \"Content-Type: application/json\"\n");
            config.push_str(&format!("data = {}\n", quote(body)));
        }
        let mut child = Command::new("curl")
            .args(["--silent", "--show-error", "--max-time", "30"])
            .args(["--write-out", "\n%{http_code}", "--config", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TrackerError::Other(format!("cannot run curl: {e}")))?;
        child
            .stdin
            .take()
            .ok_or_else(|| TrackerError::Other("curl has no input".into()))?
            .write_all(config.as_bytes())
            .map_err(|e| TrackerError::Other(format!("cannot run curl: {e}")))?;
        let out = child
            .wait_with_output()
            .map_err(|e| TrackerError::Other(format!("cannot run curl: {e}")))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(TrackerError::Other(format!("curl: {err}")));
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let (body, status) = text
            .rsplit_once('\n')
            .ok_or_else(|| TrackerError::Other("curl gave no status".into()))?;
        let status = status
            .trim()
            .parse()
            .map_err(|_| TrackerError::Other("curl gave no status".into()))?;
        Ok(Reply {
            status,
            body: body.to_string(),
        })
    }
}

/// The most open issues read from one project.
const LIMIT: usize = 1000;
/// Issues asked for in one page.
const PAGE: usize = 100;

const FIELDS: [&str; 5] = [
    "summary",
    "description",
    "fixVersions",
    "issuelinks",
    "status",
];

/// The Jira adapter.
pub struct Jira {
    transport: Box<dyn Transport>,
}

impl Default for Jira {
    fn default() -> Self {
        Self::with(Box::new(Curl::default()))
    }
}

/// A project key is letters, digits and underscores; anything else is not one,
/// and never reaches a query.
fn valid_project(project: &str) -> Result<(), TrackerError> {
    if !project.is_empty()
        && project
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        Ok(())
    } else {
        Err(TrackerError::NotFound(format!("project {project}")))
    }
}

/// An issue key is a project key, a hyphen and a number.
fn valid_key(key: &str) -> Result<(), TrackerError> {
    match key.rsplit_once('-') {
        Some((p, n))
            if valid_project(p).is_ok()
                && !n.is_empty()
                && n.chars().all(|c| c.is_ascii_digit()) =>
        {
            Ok(())
        }
        _ => Err(TrackerError::NotFound(format!("issue {key}"))),
    }
}

/// What Jira said went wrong, in a few words.
fn problem(reply: &Reply) -> String {
    let v: Value = serde_json::from_str(&reply.body).unwrap_or(Value::Null);
    let first = v
        .get("errorMessages")
        .and_then(Value::as_array)
        .and_then(|m| m.first())
        .and_then(Value::as_str)
        .map(String::from)
        .or_else(|| {
            v.get("errors")
                .and_then(Value::as_object)
                .and_then(|m| m.values().next())
                .and_then(Value::as_str)
                .map(String::from)
        });
    match first {
        Some(m) => format!("Jira answered {}: {m}", reply.status),
        None => format!("Jira answered {}", reply.status),
    }
}

impl Jira {
    /// A Jira tracker that calls the REST API through `transport`.
    pub fn with(transport: Box<dyn Transport>) -> Self {
        Self { transport }
    }

    /// Sends a request and returns the JSON Jira answered with (`null` for no body).
    fn call(
        &mut self,
        method: &'static str,
        path: String,
        body: Option<Value>,
    ) -> Result<Value, TrackerError> {
        let reply = self.transport.send(&Request {
            method,
            path,
            body: body.map(|b| b.to_string()),
        })?;
        match reply.status {
            200..=299 => {
                if reply.body.trim().is_empty() {
                    Ok(Value::Null)
                } else {
                    serde_json::from_str(&reply.body)
                        .map_err(|e| TrackerError::Other(format!("Jira gave bad JSON: {e}")))
                }
            }
            401 | 403 => Err(TrackerError::Denied(problem(&reply))),
            404 => Err(TrackerError::NotFound(problem(&reply))),
            _ => Err(TrackerError::Other(problem(&reply))),
        }
    }

    /// The raw open issues of a project, every page, failing at [`LIMIT`].
    fn open_issues(&mut self, project: &str) -> Result<Vec<Value>, TrackerError> {
        valid_project(project)?;
        let mut all: Vec<Value> = Vec::new();
        let mut token: Option<String> = None;
        loop {
            let mut body = json!({
                "jql": format!("project = {project} AND statusCategory != Done ORDER BY key ASC"),
                "fields": FIELDS,
                "maxResults": PAGE,
            });
            if let Some(t) = &token {
                body["nextPageToken"] = json!(t);
            }
            let page = self.call("POST", "/rest/api/3/search/jql".into(), Some(body))?;
            let issues = page
                .get("issues")
                .and_then(Value::as_array)
                .ok_or_else(|| TrackerError::Other("Jira search: no issues list".into()))?;
            all.extend(issues.iter().cloned());
            // A full list may be cut short; a cut-short list would read as issues closed.
            if all.len() >= LIMIT {
                return Err(TrackerError::Other(format!(
                    "{project} has {LIMIT} or more open issues; temet-work reads at most {LIMIT}"
                )));
            }
            token = page
                .get("nextPageToken")
                .and_then(Value::as_str)
                .map(String::from);
            if page.get("isLast").and_then(Value::as_bool) == Some(true)
                || token.is_none()
                || issues.is_empty()
            {
                return Ok(all);
            }
        }
    }

    fn status_category(&mut self, key: &str) -> Result<String, TrackerError> {
        valid_key(key)?;
        let v = self.call(
            "GET",
            format!("/rest/api/3/issue/{key}?fields=status"),
            None,
        )?;
        v.pointer("/fields/status/statusCategory/key")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| TrackerError::Other(format!("Jira gave {key} no status")))
    }
}

/// Plain text from Atlassian document format: blocks on their own lines, list
/// items after a dash, and nothing else of the formatting.
pub fn flatten(doc: &Value) -> String {
    fn end_line(out: &mut String) {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
    }
    fn walk(node: &Value, out: &mut String) {
        let kind = node.get("type").and_then(Value::as_str).unwrap_or_default();
        match kind {
            "text" => out.push_str(node.get("text").and_then(Value::as_str).unwrap_or_default()),
            "hardBreak" => out.push('\n'),
            "mention" | "emoji" => out.push_str(
                node.pointer("/attrs/text")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            ),
            "inlineCard" => out.push_str(
                node.pointer("/attrs/url")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            ),
            _ => {
                if !out.ends_with("- ") {
                    end_line(out);
                }
                if kind == "listItem" {
                    out.push_str("- ");
                }
                for child in node
                    .get("content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    walk(child, out);
                }
                end_line(out);
            }
        }
    }
    match doc {
        Value::String(s) => s.clone(),
        Value::Object(_) => {
            let mut out = String::new();
            walk(doc, &mut out);
            out.trim().to_string()
        }
        _ => String::new(),
    }
}

/// Atlassian document format for a plain comment: a paragraph per line.
fn document(text: &str) -> Value {
    let content: Vec<Value> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| json!({"type": "paragraph", "content": [{"type": "text", "text": l}]}))
        .collect();
    json!({"type": "doc", "version": 1, "content": content})
}

fn first_version(issue: &Value) -> Option<String> {
    issue
        .pointer("/fields/fixVersions/0/name")
        .and_then(Value::as_str)
        .map(String::from)
}

fn parse_issue(issue: &Value) -> Option<TrackedIssue> {
    let key = issue.get("key")?.as_str()?.to_string();
    let order = key.rsplit('-').next()?.parse().ok()?;
    let fields = issue.get("fields")?;
    let body = fields.get("description").map(flatten).unwrap_or_default();
    let mut depends = depends_on(&body);
    for link in fields
        .get("issuelinks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let blocked_by = link
            .pointer("/type/inward")
            .and_then(Value::as_str)
            .is_some_and(|s| s.eq_ignore_ascii_case("is blocked by"));
        if let Some(other) = link.pointer("/inwardIssue/key").and_then(Value::as_str)
            && blocked_by
            && !depends.iter().any(|d| d == other)
        {
            depends.push(other.to_string());
        }
    }
    Some(TrackedIssue {
        title: fields
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        depends_on: depends,
        body,
        milestone: first_version(issue),
        order,
        closed: false,
        key,
    })
}

impl Tracker for Jira {
    fn id(&self) -> &'static str {
        "jira"
    }

    fn issues(&mut self, project: &str) -> Result<Vec<TrackedIssue>, TrackerError> {
        Ok(self
            .open_issues(project)?
            .iter()
            .filter_map(parse_issue)
            .collect())
    }

    fn milestones(&mut self, project: &str) -> Result<Vec<TrackedMilestone>, TrackerError> {
        valid_project(project)?;
        let versions = self.call(
            "GET",
            format!("/rest/api/3/project/{project}/versions"),
            None,
        )?;
        let versions = versions
            .as_array()
            .ok_or_else(|| TrackerError::Other("Jira versions: not a list".into()))?;
        let issues = self.open_issues(project)?;
        Ok(versions
            .iter()
            .filter(|v| {
                v.get("released").and_then(Value::as_bool) != Some(true)
                    && v.get("archived").and_then(Value::as_bool) != Some(true)
            })
            .filter_map(|v| v.get("name").and_then(Value::as_str))
            .map(|name| TrackedMilestone {
                title: name.to_string(),
                open: issues
                    .iter()
                    .filter(|i| first_version(i).as_deref() == Some(name))
                    .count() as u64,
            })
            .collect())
    }

    fn closed(&mut self, project: &str, key: &str) -> Result<bool, TrackerError> {
        let _ = project;
        Ok(self.status_category(key)? == "done")
    }

    fn comment(&mut self, project: &str, key: &str, text: &str) -> Result<(), TrackerError> {
        let _ = project;
        valid_key(key)?;
        self.call(
            "POST",
            format!("/rest/api/3/issue/{key}/comment"),
            Some(json!({"body": document(text)})),
        )
        .map(|_| ())
    }

    fn close(&mut self, project: &str, key: &str) -> Result<(), TrackerError> {
        let _ = project;
        if self.status_category(key)? == "done" {
            return Ok(());
        }
        let path = format!("/rest/api/3/issue/{key}/transitions");
        let listed = self.call("GET", path.clone(), None)?;
        let transitions = listed
            .get("transitions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let done = transitions
            .iter()
            .find(|t| t.pointer("/to/statusCategory/key").and_then(Value::as_str) == Some("done"));
        let Some(id) = done.and_then(|t| t.get("id")).and_then(Value::as_str) else {
            let names: Vec<&str> = transitions
                .iter()
                .filter_map(|t| t.get("name").and_then(Value::as_str))
                .collect();
            return Err(TrackerError::Other(format!(
                "{key} has no transition to a Done status; its transitions are: {}",
                if names.is_empty() {
                    "none".to_string()
                } else {
                    names.join(", ")
                }
            )));
        };
        self.call("POST", path, Some(json!({"transition": {"id": id}})))
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    const SEARCH: &str = include_str!("../tests/fixtures/demo-search.json");
    const VERSIONS: &str = include_str!("../tests/fixtures/demo-versions.json");
    const TRANSITIONS: &str = include_str!("../tests/fixtures/demo-transitions.json");

    fn reply(status: u16, body: &str) -> Result<Reply, TrackerError> {
        Ok(Reply {
            status,
            body: body.into(),
        })
    }

    /// Replays Jira output recorded from a made-up project, DEMO, and notes each request.
    #[derive(Clone, Default)]
    struct Recorded {
        sent: Arc<Mutex<Vec<Request>>>,
        /// The status category an issue is in; `done` for DEMO-99.
        no_done_transition: bool,
    }

    impl Transport for Recorded {
        fn send(&mut self, r: &Request) -> Result<Reply, TrackerError> {
            self.sent.lock().expect("lock").push(r.clone());
            let path = r.path.as_str();
            match (r.method, path) {
                ("POST", "/rest/api/3/search/jql") => reply(200, SEARCH),
                ("GET", "/rest/api/3/project/DEMO/versions") => reply(200, VERSIONS),
                ("GET", p) if p.starts_with("/rest/api/3/issue/DEMO-99?") => reply(
                    200,
                    r#"{"fields":{"status":{"statusCategory":{"key":"done"}}}}"#,
                ),
                ("GET", p)
                    if p.starts_with("/rest/api/3/issue/DEMO-") && p.contains("/transitions") =>
                {
                    if self.no_done_transition {
                        reply(
                            200,
                            r#"{"transitions":[{"id":"11","name":"Start","to":{"statusCategory":{"key":"indeterminate"}}}]}"#,
                        )
                    } else {
                        reply(200, TRANSITIONS)
                    }
                }
                ("GET", p)
                    if ["41", "12", "20"]
                        .iter()
                        .any(|n| p.starts_with(&format!("/rest/api/3/issue/DEMO-{n}?"))) =>
                {
                    reply(
                        200,
                        r#"{"fields":{"status":{"statusCategory":{"key":"new"}}}}"#,
                    )
                }
                ("GET", _) => reply(404, r#"{"errorMessages":["Issue does not exist"]}"#),
                ("POST", _) => reply(201, "{}"),
                _ => reply(500, ""),
            }
        }
    }

    struct ReadOnly;

    impl temet_work_tracker::contract::Harness for ReadOnly {
        fn project(&self) -> String {
            "DEMO".into()
        }
        fn writable_issue(&self) -> Option<String> {
            None
        }
    }

    fn jira() -> (Jira, Arc<Mutex<Vec<Request>>>) {
        let t = Recorded::default();
        let sent = t.sent.clone();
        (Jira::with(Box::new(t)), sent)
    }

    #[test]
    fn passes_the_tracker_contract_on_recorded_output() {
        let (mut jira, _) = jira();
        assert_eq!(
            temet_work_tracker::contract::check(&mut jira, &ReadOnly),
            Ok(())
        );
    }

    #[test]
    fn reads_keys_titles_bodies_milestones_order_and_dependencies() {
        let (mut jira, sent) = jira();
        let issues = jira.issues("DEMO").expect("recorded issues");
        assert_eq!(issues.len(), 3);
        let a = &issues[0];
        assert_eq!(a.key, "DEMO-41");
        assert_eq!(a.title, "Rework the sign-in page");
        assert_eq!(a.order, 41);
        assert_eq!(a.milestone.as_deref(), Some("Sign-in rework"));
        assert_eq!(
            a.body,
            "Replace the form with a single button.\nDepends on DEMO-7, DEMO-9\n- keep the old link\nfor a week"
        );
        assert_eq!(a.depends_on, ["DEMO-7", "DEMO-9"]);
        let b = &issues[1];
        // "is blocked by" counts; "blocks" and "relates to" do not.
        assert_eq!(b.depends_on, ["DEMO-41"]);
        assert_eq!((b.body.as_str(), b.milestone.as_deref()), ("", None));
        assert_eq!(issues[2].depends_on, ["DEMO-12"]);
        assert_eq!(issues[2].milestone.as_deref(), Some("Sign-in rework"));
        let sent = sent.lock().expect("lock");
        let body: Value =
            serde_json::from_str(sent[0].body.as_deref().expect("body")).expect("json");
        assert_eq!(
            body["jql"],
            "project = DEMO AND statusCategory != Done ORDER BY key ASC"
        );
    }

    #[test]
    fn pages_past_a_hundred_and_stops_at_a_thousand() {
        struct Pages(usize);
        impl Transport for Pages {
            fn send(&mut self, r: &Request) -> Result<Reply, TrackerError> {
                let body: Value =
                    serde_json::from_str(r.body.as_deref().unwrap_or("{}")).expect("json");
                let page = body
                    .get("nextPageToken")
                    .and_then(Value::as_str)
                    .map_or(0, |t| t.parse::<usize>().expect("token"));
                let issues: Vec<Value> = (0..PAGE)
                    .map(|i| json!({"key": format!("BIG-{}", page * PAGE + i + 1), "fields": {"summary": "x"}}))
                    .collect();
                let last = page + 1 >= self.0;
                reply(200, &json!({"issues": issues, "nextPageToken": (page + 1).to_string(), "isLast": last}).to_string())
            }
        }
        let mut jira = Jira::with(Box::new(Pages(3)));
        assert_eq!(jira.issues("BIG").expect("three pages").len(), 300);
        let mut jira = Jira::with(Box::new(Pages(20)));
        let err = jira.issues("BIG").expect_err("too many");
        assert!(err.to_string().contains("1000"), "{err}");
    }

    #[test]
    fn flattens_atlassian_document_format() {
        let doc = json!({"type":"doc","content":[
            {"type":"heading","content":[{"type":"text","text":"Title"}]},
            {"type":"paragraph","content":[{"type":"text","text":"a"},{"type":"hardBreak"},{"type":"text","text":"b"}]},
            {"type":"orderedList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"one"}]}]}]}
        ]});
        assert_eq!(flatten(&doc), "Title\na\nb\n- one");
        assert_eq!(flatten(&json!("plain")), "plain");
        assert_eq!(flatten(&Value::Null), "");
    }

    #[test]
    fn milestones_are_unreleased_versions_with_their_open_counts() {
        let (mut jira, _) = jira();
        let m = jira.milestones("DEMO").expect("recorded versions");
        let m: Vec<(&str, u64)> = m.iter().map(|m| (m.title.as_str(), m.open)).collect();
        assert_eq!(m, [("Sign-in rework", 2), ("Later", 0)]);
    }

    #[test]
    fn closed_follows_the_status_category_and_an_unknown_key_is_not_found() {
        let (mut jira, _) = jira();
        assert_eq!(jira.closed("DEMO", "DEMO-41"), Ok(false));
        assert_eq!(jira.closed("DEMO", "DEMO-99"), Ok(true));
        assert!(matches!(
            jira.closed("DEMO", "DEMO-500"),
            Err(TrackerError::NotFound(_))
        ));
        assert!(matches!(
            jira.closed("DEMO", "not a key"),
            Err(TrackerError::NotFound(_))
        ));
    }

    #[test]
    fn a_comment_is_posted_in_document_format() {
        let (mut jira, sent) = jira();
        jira.comment("DEMO", "DEMO-41", "merged\n\nin #5")
            .expect("posted");
        let sent = sent.lock().expect("lock");
        let r = sent.last().expect("a request");
        assert_eq!(
            (r.method, r.path.as_str()),
            ("POST", "/rest/api/3/issue/DEMO-41/comment")
        );
        let body: Value = serde_json::from_str(r.body.as_deref().expect("body")).expect("json");
        assert_eq!(body["body"]["type"], "doc");
        assert_eq!(body["body"]["content"].as_array().map(Vec::len), Some(2));
        assert_eq!(body["body"]["content"][1]["content"][0]["text"], "in #5");
    }

    #[test]
    fn close_takes_the_transition_that_leads_to_done() {
        let (mut jira, sent) = jira();
        jira.close("DEMO", "DEMO-41").expect("closed");
        let sent = sent.lock().expect("lock");
        let r = sent.last().expect("a request");
        assert_eq!(
            (r.method, r.path.as_str()),
            ("POST", "/rest/api/3/issue/DEMO-41/transitions")
        );
        assert_eq!(r.body.as_deref(), Some(r#"{"transition":{"id":"31"}}"#));
    }

    #[test]
    fn closing_a_done_issue_sends_nothing_more() {
        let (mut jira, sent) = jira();
        jira.close("DEMO", "DEMO-99").expect("already done");
        assert!(sent.lock().expect("lock").iter().all(|r| r.method == "GET"));
    }

    #[test]
    fn an_issue_with_no_done_transition_names_the_ones_it_has() {
        let mut jira = Jira::with(Box::new(Recorded {
            no_done_transition: true,
            ..Recorded::default()
        }));
        match jira.close("DEMO", "DEMO-41") {
            Err(TrackerError::Other(e)) => assert!(e.contains("Start"), "{e}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn statuses_map_to_errors() {
        struct Fixed(u16);
        impl Transport for Fixed {
            fn send(&mut self, _: &Request) -> Result<Reply, TrackerError> {
                reply(self.0, r#"{"errorMessages":["no"]}"#)
            }
        }
        let call = |status| Jira::with(Box::new(Fixed(status))).closed("DEMO", "DEMO-1");
        assert!(matches!(call(401), Err(TrackerError::Denied(_))));
        assert!(matches!(call(403), Err(TrackerError::Denied(_))));
        assert!(matches!(call(404), Err(TrackerError::NotFound(_))));
        assert!(matches!(call(500), Err(TrackerError::Other(_))));
    }

    fn env(pairs: &[(&str, &str)]) -> Curl {
        let map: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Curl::with_env(Box::new(move |name| {
            map.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())
        }))
    }

    fn get() -> Request {
        Request {
            method: "GET",
            path: "/rest/api/3/myself".into(),
            body: None,
        }
    }

    #[test]
    fn runs_nothing_in_the_sandbox() {
        let mut curl = env(&[
            ("TEMET_WORK_SANDBOX", "1"),
            ("JIRA_URL", "https://x.invalid"),
            ("JIRA_USER", "a@example.com"),
            ("JIRA_TOKEN", "t"),
        ]);
        match curl.send(&get()) {
            Err(TrackerError::Other(e)) => assert!(e.contains("TEMET_WORK_SANDBOX"), "{e}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_missing_variable_is_denied_and_named() {
        for missing in ["JIRA_URL", "JIRA_USER", "JIRA_TOKEN"] {
            let all = [
                ("JIRA_URL", "https://x.invalid"),
                ("JIRA_USER", "a@example.com"),
                ("JIRA_TOKEN", "t"),
            ];
            let present: Vec<_> = all.iter().copied().filter(|(k, _)| *k != missing).collect();
            match env(&present).send(&get()) {
                Err(TrackerError::Denied(e)) => assert!(e.contains(missing), "{e}"),
                other => panic!("{missing}: {other:?}"),
            }
        }
    }

    #[test]
    fn a_site_without_https_is_denied() {
        let mut curl = env(&[
            ("JIRA_URL", "http://x.invalid"),
            ("JIRA_USER", "a@example.com"),
            ("JIRA_TOKEN", "t"),
        ]);
        assert!(matches!(curl.send(&get()), Err(TrackerError::Denied(_))));
    }

    #[test]
    fn quotes_for_a_curl_config() {
        assert_eq!(quote("a\"b\\c\n"), r#""a\"b\\c\n""#);
    }
}
