//! What an agent is told when it starts. Each role gets its task in full: what to
//! do, what done means, and the one command that reports it done. Agents use the
//! same `temet-work` commands a person does.

use std::path::PathBuf;

use crate::ids::IssueId;
use crate::launch::Brief;
use crate::model::{Issue, Project, Role};

/// Writes the brief for an agent of `role` on `issue`, run in `folder`.
pub fn write(
    role: Role,
    issue: &Issue,
    project: Option<&Project>,
    folder: Option<PathBuf>,
) -> Brief {
    let id = &issue.id;
    let source = project.map_or_else(
        || format!("issue {} of project {}", id.key, id.project),
        |p| format!("issue {} of {} in its tracker", id.key, p.name),
    );
    let checks = issue.plan.as_ref().map_or_else(String::new, |plan| {
        plan.checks
            .iter()
            .map(|c| format!("- {} (shown by: {})\n", c.done_when, c.proven_by))
            .collect()
    });
    // An author started on an issue sent back to it hears what to fix first.
    let fixes = if matches!(issue.stage, crate::model::Stage::Fixing { .. })
        && !issue.fix.is_empty()
    {
        let list: String = issue.fix.iter().map(|p| format!("- {p}\n")).collect();
        format!(
            "The pull request is open; the review or the checks found these to fix first:\n{list}"
        )
    } else {
        String::new()
    };
    let task = match role {
        Role::Refiner => format!(
            "Refine {id}: {title}. It is {source}.\n\
             Read the issue and the code it touches in this folder. Change no file here.\n\
             Write a plan as YAML to a file outside this folder, one entry per check:\n\
             checks:\n  - done_when: one outcome a person can observe\n    proven_by: the command, test or page that shows it\n\
             Then run: temet-work plan {id} <file>\n\
             Where the issue leaves a choice open, pick the default you would defend and say why in the plan.",
            title = issue.title,
        ),
        Role::Author => format!(
            "Work on {id}: {title}. It is {source}.\n\
             This folder is yours until the change merges. Make a branch, change the code and open a pull request.\n\
             {fixes}The plan's checks, each to hold before you finish:\n{checks}\
             When the pull request is open and every check holds, run: temet-work ready {id} <pull request number>",
            title = issue.title,
        ),
        Role::Reviewer => format!(
            "Review the pull request for {id}: {title}. It is {source}.\n\
             The plan's checks:\n{checks}\
             Read the whole change, not only the last commit. Run the checks yourself.\n\
             Then run: temet-work review {id} approve\n\
             or: temet-work review {id} block \"<what to fix>\" [\"<what else>\" ...]",
            title = issue.title,
        ),
        Role::Arbiter => format!(
            "Review rounds on {id} ({title}) ran out. It is {source}.\n\
             Read the pull request, the review points and the author's answers. Decide whether it merges.\n\
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
