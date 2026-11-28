//! What `temet-work change`, `show`, `search` and `ci` print. The first three
//! read the family's shared copy of a repository ([`crate::copy`]) and run away
//! from the daemon's loop; `ci` asks the adapter.

use std::sync::Arc;

use temet_work_core::ids::ProjectId;

use crate::copy::{Copies, Copy, Point};
use crate::{CheckRun, RunState};

/// The most lines a search prints; a line after them counts the rest.
pub const MOST_LINES: usize = 1000;

/// A repository a read looks in.
#[derive(Clone, Debug)]
pub struct Repo {
    /// The project.
    pub project: ProjectId,
    /// The code host's name for the repository.
    pub name: String,
    /// The code host's id, which the copy is kept under.
    pub host: String,
    /// Where git clones it from.
    pub url: String,
}

impl Repo {
    /// The project and the repository, as a refusal names them.
    fn what(&self) -> String {
        format!("{} ({})", self.project, self.name)
    }

    fn with<T>(
        &self,
        copies: &Copies,
        read: impl FnOnce(&Copy<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        copies.with(&self.host, &self.name, &self.url, read)
    }
}

/// One open pull request a change is read from.
#[derive(Clone, Debug)]
pub struct Pull {
    /// Its repository.
    pub repo: Repo,
    /// Its number.
    pub number: String,
    /// The ref of its head.
    pub head: String,
}

/// The first 12 characters of a commit id.
fn short(id: &str) -> &str {
    id.get(..12).unwrap_or(id)
}

/// Text from git's output, whatever its bytes.
fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

/// Each pull request whole, as the diff from where its branch left the default
/// branch to its head, or, `by_commit`, each commit in order with its id,
/// message and diff. Each starts with a line naming its project, repository and
/// number, and the two commits.
pub fn change(copies: Arc<Copies>, pulls: Vec<Pull>, by_commit: bool) -> Result<String, String> {
    let mut out = String::new();
    for (i, pull) in pulls.iter().enumerate() {
        let what = pull.repo.what();
        let part = pull.repo.with(&copies, |copy| {
            let head = copy.resolve(
                &Point::Pull {
                    number: pull.number.clone(),
                    head: pull.head.clone(),
                },
                &what,
            )?;
            let main = copy.resolve(&Point::Default, &what)?;
            let base = text(copy.read(&["merge-base", &main, &head]).map_err(|e| {
                format!(
                    "pull request {} of {what} shares no history with its default branch: {e}",
                    pull.number
                )
            })?)
            .trim()
            .to_string();
            let range = format!("{base}..{head}");
            let commits = text(copy.read(&["rev-list", "--count", &range])?)
                .trim()
                .parse::<u64>()
                .unwrap_or(0);
            let diff = if by_commit {
                copy.read(&[
                    "log",
                    "--reverse",
                    "--no-color",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--format=commit %H%n%n%B",
                    "-p",
                    &range,
                ])?
            } else {
                copy.read(&[
                    "diff",
                    "--no-color",
                    "--no-ext-diff",
                    "--no-textconv",
                    &base,
                    &head,
                ])?
            };
            let diff = text(diff);
            let commits = match commits {
                1 => "1 commit".to_string(),
                n => format!("{n} commits"),
            };
            let shown = if by_commit {
                "each commit"
            } else {
                "the whole change"
            };
            let mut part = format!(
                "{what}: pull request {}, {commits} from {} on the default branch to {}; \
                 {shown}:\n",
                pull.number,
                short(&base),
                short(&head),
            );
            if diff.trim().is_empty() {
                part.push_str("(no change)\n");
            } else {
                part.push_str(&diff);
                if !diff.ends_with('\n') {
                    part.push('\n');
                }
            }
            Ok(part)
        })?;
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&part);
    }
    Ok(out)
}

/// The path as a tree path: no slash at either end, and `.` for the top.
fn tidy(path: &str) -> String {
    let path = path.trim().trim_matches('/');
    if path == "." {
        String::new()
    } else {
        path.to_string()
    }
}

/// The file at `path` at `point`, or, for a directory, its entries, one per
/// line with a slash after each directory. `at` names the point in words.
pub fn show(
    copies: Arc<Copies>,
    repo: Repo,
    path: String,
    point: Point,
    at: String,
) -> Result<String, String> {
    let what = repo.what();
    let path = tidy(&path);
    repo.with(&copies, |copy| {
        let commit = copy.resolve(&point, &what)?;
        let object = format!("{commit}:{path}");
        let named = if path.is_empty() {
            "its top folder".to_string()
        } else {
            path.clone()
        };
        let kind = copy
            .read(&["cat-file", "-t", &object])
            .map(text)
            .map_err(|_| format!("{what} has no {named} at {at}"))?;
        match kind.trim() {
            "blob" => {
                let bytes = copy.read(&["cat-file", "blob", &object])?;
                if bytes.iter().take(8000).any(|b| *b == 0) {
                    return Ok(format!(
                        "{named} is a binary file of {} bytes at {at}\n",
                        bytes.len()
                    ));
                }
                Ok(text(bytes))
            }
            "tree" => {
                let listed = text(copy.read(&["ls-tree", "-z", &object])?);
                let mut lines = String::new();
                for entry in listed.split('\0').filter(|e| !e.is_empty()) {
                    let Some((meta, name)) = entry.split_once('\t') else {
                        continue;
                    };
                    match meta.split(' ').nth(1) {
                        Some("tree") => lines.push_str(&format!("{name}/\n")),
                        Some("commit") => lines.push_str(&format!("{name} (a submodule)\n")),
                        _ => lines.push_str(&format!("{name}\n")),
                    }
                }
                if lines.is_empty() {
                    lines = format!("{named} is empty at {at}\n");
                }
                Ok(lines)
            }
            "commit" => Ok(format!("{named} is a submodule at {at}\n")),
            other => Err(format!(
                "{named} at {at} is a git {other}, not a file or a folder"
            )),
        }
    })
}

/// Each line of the repository's files at `point` that matches `pattern`, an
/// extended regular expression, as `path:line:text`; at most [`MOST_LINES`],
/// then a line counting the rest. Says so when nothing matches.
pub fn search(
    copies: Arc<Copies>,
    repo: Repo,
    pattern: String,
    point: Point,
    at: String,
) -> Result<String, String> {
    let what = repo.what();
    if pattern.is_empty() {
        return Err("search needs a pattern".into());
    }
    repo.with(&copies, |copy| {
        let commit = copy.resolve(&point, &what)?;
        let found = copy.read_with_code(&[
            "grep",
            "-n",
            "-I",
            "--no-color",
            "-E",
            "-e",
            &pattern,
            &commit,
            "--",
        ])?;
        let out = match found.code {
            Some(0) => text(found.out),
            Some(1) => return Ok(format!("no line matches {pattern} in {what} at {at}\n")),
            _ => return Err(format!("search for {pattern} in {what}: {}", found.err)),
        };
        // git names each line's commit before its path.
        let prefix = format!("{commit}:");
        let lines: Vec<&str> = out
            .lines()
            .map(|l| l.strip_prefix(&prefix).unwrap_or(l))
            .collect();
        let mut text: String = lines
            .iter()
            .take(MOST_LINES)
            .map(|l| format!("{l}\n"))
            .collect();
        if lines.len() > MOST_LINES {
            text.push_str(&format!(
                "and {} more matching lines not printed; give a narrower pattern\n",
                lines.len() - MOST_LINES
            ));
        }
        Ok(text)
    })
}

/// One pull request's checks as `temet-work ci` prints them: a line for the
/// pull request, a line for each check with its state, then the output of each
/// failing check's failed steps whole, or, for one with no log, its summary and
/// web address.
pub fn ci(what: &str, number: &str, merged: bool, runs: &[CheckRun]) -> String {
    let merged = if merged { " (merged)" } else { "" };
    let mut out = format!("{what}: pull request {number}{merged}\n");
    if runs.is_empty() {
        out.push_str("  no checks reported\n");
        return out;
    }
    for run in runs {
        out.push_str(&format!("  {}: {}\n", run.name, run.state.word()));
    }
    for run in runs.iter().filter(|r| r.state == RunState::Failed) {
        match &run.log {
            Some(log) => {
                out.push_str(&format!("\nThe output of {}'s failed steps:\n", run.name));
                out.push_str(log);
                if !log.ends_with('\n') {
                    out.push('\n');
                }
            }
            None => {
                let summary = if run.summary.trim().is_empty() {
                    "it gives no summary".to_string()
                } else {
                    run.summary.trim().to_string()
                };
                let address = run
                    .address
                    .as_ref()
                    .map_or_else(|| "and no web address".to_string(), |a| a.clone());
                out.push_str(&format!(
                    "\n{} keeps no log the code host gives: {summary}\n{address}\n",
                    run.name
                ));
            }
        }
    }
    out
}
