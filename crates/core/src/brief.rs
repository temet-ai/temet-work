//! What an agent is told when it starts. Each role gets its task in full: what to
//! do, what done means, and the one command that reports it done. Agents use the
//! same `temet-work` commands a person does.

use std::path::PathBuf;

use crate::ids::IssueId;
use crate::ids::ProjectId;
use crate::launch::Brief;
use crate::model::{Issue, Role};

/// A work folder an author holds: the project it is a checkout of, the code
/// host's name for that repository, and where it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    /// The project.
    pub project: ProjectId,
    /// The code host's name for the repository.
    pub repo: String,
    /// Where the folder is on its machine.
    pub path: String,
}

/// The pull requests of an issue that changes several repositories, one per line.
fn pull_list(issue: &Issue) -> String {
    issue
        .pulls
        .iter()
        .map(|(repo, number)| {
            let merged = if issue.merged.contains(repo) {
                " (merged already)"
            } else {
                ""
            };
            format!("- {repo}: pull request {number}{merged}\n")
        })
        .collect()
}

/// Writes the brief for an agent of `role` on `issue`, run in `folder`.
/// `shared` says the folder is held by another issue's author, so the agent must
/// not change it.
/// `source` is the tracker's name for where the issue comes from, and `repos`
/// the repositories of its area when it comes from an area's tracker. `held` is
/// the author's work folders, one per repository the plan names.
pub fn write(
    role: Role,
    issue: &Issue,
    source: Option<&str>,
    repos: &[ProjectId],
    folder: Option<PathBuf>,
    shared: bool,
    held: &[Held],
) -> Brief {
    let id = &issue.id;
    let source = source.map_or_else(
        || format!("issue {} of project {}", id.key, id.project),
        |name| format!("issue {} of {name} in its tracker", id.key),
    );
    // An area's issue may touch any of its repositories; the refiner says which.
    let area_note = if repos.is_empty() {
        String::new()
    } else {
        let list: Vec<String> = repos.iter().map(ToString::to_string).collect();
        format!(
            "It belongs to an area whose repositories are: {}. Under checks, add repos: the ones \
             the change needs, in the order they should merge.\n",
            list.join(", ")
        )
    };
    let checks = issue.plan.as_ref().map_or_else(String::new, |plan| {
        plan.checks
            .iter()
            .map(|c| format!("- {} (shown by: {})\n", c.done_when, c.proven_by))
            .collect()
    });
    // An author started on an issue that came back hears what to fix first.
    let fixes = if issue.fix.is_empty() {
        String::new()
    } else {
        let list: String = issue.fix.iter().map(|p| format!("- {p}\n")).collect();
        format!("Earlier work on it came back with these to fix first:\n{list}")
    };
    let folder_note = if shared {
        "Another issue's author is changing files in this folder: do not switch its branch or change \
         anything here; read the pull request's changes from its code host.\n"
    } else {
        ""
    };
    let several = held.len() > 1;
    let folders = if several {
        let list: String = held
            .iter()
            .map(|h| format!("- {} ({}): {}\n", h.project, h.repo, h.path))
            .collect();
        format!(
            "This change spans several repositories. You hold a folder in each; you start in the first:\n{list}\
             Make the branch and open the pull requests yourself: use the same branch name in every repository \
             and open one pull request in each.\n"
        )
    } else {
        String::new()
    };
    let ready = if several {
        let names: Vec<String> = held
            .iter()
            .map(|h| format!("{}=<pull request number>", h.project))
            .collect();
        names.join(" ")
    } else {
        "<pull request number>".to_string()
    };
    let pulls = if issue.pulls.len() > 1 {
        format!(
            "It changes several repositories; review every one of its pull requests:\n{}",
            pull_list(issue)
        )
    } else {
        String::new()
    };
    let task = match role {
        Role::Refiner => format!(
            "Refine {id}: {title}. It is {source}.\n\
             {folder_note}{area_note}Read the issue and the code it touches in this folder. Change no file here.\n\
             Then send the plan, one entry per check, as YAML on standard input:\n\
             temet-work plan {id} - <<'PLAN'\n\
             checks:\n  - done_when: one outcome a person can observe\n    proven_by: the command, test or page that shows it\n\
             PLAN\n\
             Where the issue leaves a choice open, pick the default you would defend and say why in done_when.\n\
             For an issue that is hard, add a settings: map beside checks: with task-level values, such as model: opus; \
             they are recorded as yours and a person can overturn them.",
            title = issue.title,
        ),
        Role::Author => format!(
            "Work on {id}: {title}. It is {source}.\n\
             This folder is yours until the change merges. Make a branch, change the code and open a pull request.\n\
             {folders}{fixes}The plan's checks, each to hold before you finish:\n{checks}\
             When the pull request is open and every check holds, run: temet-work ready {id} {ready}",
            title = issue.title,
        ),
        Role::Reviewer => format!(
            "Review the pull request for {id}: {title}. It is {source}.\n\
             {pulls}The plan's checks:\n{checks}\
             {folder_note}Read the whole change, not only the last commit. Run the checks yourself where you can.\n\
             Then run: temet-work review {id} approve\n\
             or: temet-work review {id} block \"<what to fix>\" [\"<what else>\" ...]",
            title = issue.title,
        ),
        Role::Arbiter => format!(
            "Review rounds on {id} ({title}) ran out. It is {source}.\n\
             {pulls}Read the pull request, the review points and the author's answers. Decide whether it merges.\n\
             Then run: temet-work rule {id} approve \"<why>\"\n\
             or: temet-work rule {id} fix \"<what must change, and why>\"",
            title = issue.title,
        ),
        Role::Investigator => format!(
            "Answer questions about {id}: {title}. It is {source}. Change no file.",
            title = issue.title,
        ),
    };
    Brief {
        role,
        issue: id.clone(),
        task,
        model: None,
        permission: match crate::settings::builtin("permission") {
            Some(v) => v.to_string(),
            None => String::new(),
        },
        folder,
    }
}

/// What an author is told when the review or the checks found things to fix.
pub fn fixes(issue: &IssueId, points: &[String]) -> String {
    let list: String = points.iter().map(|p| format!("- {p}\n")).collect();
    format!(
        "Fix these, from the review or the checks:\n{list}Then push and run: temet-work ready {issue}"
    )
}

/// What a running reviewer is told when the author is ready again.
pub fn next_round(issue: &IssueId, round: u8) -> String {
    format!(
        "The author pushed fixes for {issue}; this is review round {round}. Review the whole change again, \
         not only the new commits. Then run: temet-work review {issue} approve\n\
         or: temet-work review {issue} block \"<what is still wrong>\" [\"<what else>\" ...]"
    )
}

/// What an agent its issue is waiting on hears when it has been idle a while.
pub fn nudge(role: Role, issue: &IssueId) -> String {
    let command = match role {
        Role::Refiner => format!("temet-work plan {issue} - with the plan as YAML"),
        Role::Author => format!(
            "temet-work ready {issue} <pull request number>, or repo=number for each repository it changes"
        ),
        Role::Reviewer => format!("temet-work review {issue} approve, or block with what to fix"),
        Role::Arbiter => format!("temet-work rule {issue} approve or fix, with the reason"),
        Role::Investigator => return format!("{issue} is still waiting on your answer."),
    };
    format!(
        "{issue} is still waiting on you. If your step is done, run: {command}. \
         If a choice stops you, take the default you would defend, say so, and finish."
    )
}
