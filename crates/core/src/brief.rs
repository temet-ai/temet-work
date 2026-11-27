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

/// Where an agent runs.
#[derive(Debug, Default)]
pub struct Workplace {
    /// The folder it runs in.
    pub folder: Option<PathBuf>,
    /// Whether another issue's author holds that folder, so the agent must not
    /// change it.
    pub shared: bool,
    /// An author's work folders, one per repository the plan names.
    pub held: Vec<Held>,
}

/// One of an issue's pull requests: the project, the code host's name for its
/// repository, the number there, and whether it has merged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pulled {
    /// The project.
    pub project: ProjectId,
    /// The code host's name for the repository.
    pub repo: String,
    /// The pull request's number.
    pub number: String,
    /// Whether it has merged.
    pub merged: bool,
}

/// The pull requests of an issue that changes several repositories, one per line.
fn pull_list(pulls: &[Pulled]) -> String {
    pulls
        .iter()
        .map(|p| {
            let merged = if p.merged { " (merged already)" } else { "" };
            format!(
                "- {} ({}): pull request {}{merged}\n",
                p.project, p.repo, p.number
            )
        })
        .collect()
}

/// What a refiner's plain `model` becomes (`docs/settings.md`, "A model belongs
/// to a runtime").
const PLAIN_MODEL: &str = "A plain model is recorded as the global runtime's, as \
model.<that runtime>, when you write it, and is refused when the author runs another runtime; \
model.<runtime> names the runtime a model is for.";

/// What a change is held against: by a reviewer in its brief and again on each
/// later round, and by an arbiter. Each adds what it does with a point that is
/// not met.
fn hold_to_the_issue(issue: &IssueId) -> String {
    format!(
        "Read the issue's own text: run temet-work issue {issue}, which prints the issue's text \
         from its tracker. Check each of its points and its \"Done is\" line against the change, not \
         only the plan's checks.\n\
         For each of the plan's checks, confirm that the test or evidence named under \"shown by\" \
         shows the whole of what the check states, including any failure or persistent case it \
         names.\n\
         A real run the issue asks for must have happened, with its evidence in the pull request. \
         A plan that leaves it out does not excuse it."
    )
}

/// What a reviewer does with a point that is not met, or that the plan or a
/// decision departs from.
const BLOCK_WHAT_IS_NOT_MET: &str = "\
Block on any point that is not met, and name that point in the block. A test that shows only part \
of its check is a block. A plan check or a recorded decision that departs from a point the issue \
states, or that leaves the issue's failure scenario happening, is a block, even when the plan \
sanctions it. A decision the change writes down, such as under \"Decided, open to change\", is a \
recorded decision.";

/// What an arbiter does with a point that is not met, or that the plan or a
/// decision departs from.
const FIX_WHAT_IS_NOT_MET: &str = "\
Where a point is not met, rule fix and name that point in the ruling. A test that shows only part \
of its check is a fix. An author's answer that the plan does not list the point is no reason to \
approve. A plan check or a recorded decision that departs from a point the issue states, or that \
leaves the issue's failure scenario happening, is a fix, even when the plan sanctions it. A \
decision the change writes down, such as under \"Decided, open to change\", is a recorded \
decision.";

/// Writes the brief for an agent of `role` on `issue`, run in `place`.
/// `rules` are the kept rules from overturned decisions; each brief ends with them.
/// `source` is the tracker's name for where the issue comes from, and `repos`
/// the repositories of its area when it comes from an area's tracker. `pulls`
/// are the issue's pull requests, in the order they merge.
pub fn write(
    role: Role,
    issue: &Issue,
    source: Option<&str>,
    repos: &[ProjectId],
    place: Workplace,
    pulls: &[Pulled],
    rules: &[String],
) -> Brief {
    let Workplace {
        folder,
        shared,
        held,
    } = place;
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
        let reply = if crate::World::answered_round(issue).is_some() {
            format!(
                "They came from a review or a ruling: when you run temet-work ready, reply to each, \
                 in their order, with what you did about it or why you did not, as in \
                 temet-work ready {} --reply \"<reply to the first>\" [--reply \"<reply to the next>\" ...]\n",
                issue.id
            )
        } else {
            String::new()
        };
        format!("Earlier work on it came back with these to fix first:\n{list}{reply}")
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
    let pulls = if pulls.len() > 1 {
        format!(
            "It changes several repositories; review every one of its pull requests:\n{}",
            pull_list(pulls)
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
             Each point the issue states needs a check that meets it as written. \
             Use a chosen default only where the issue leaves the choice open (it says \"or\", \"pick\" or \
             \"record which\"): pick one of the options it names, the one you would defend, and say why \
             in done_when. \
             When a point cannot be done as written, say why in that point's check and fix the issue's \
             failure scenario some other way: the plan may not leave it in place.\n\
             For an issue that is hard, add a settings: map beside checks: with task-level values, such as model: opus; \
             they are recorded as yours and a person can overturn them. \
             A model you set is the author's only: the models of reviewers, arbiters and refiners are \
             a person's to set. \
             {PLAIN_MODEL} \
             Only a person sets {persons}: a plan that names one is refused.",
            title = issue.title,
            persons = only_a_persons().join(", "),
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
             {hold}\n\
             {BLOCK_WHAT_IS_NOT_MET}\n\
             Then run: temet-work review {id} approve\n\
             or: temet-work review {id} block \"<what to fix>\" [\"<what else>\" ...]",
            title = issue.title,
            hold = hold_to_the_issue(id),
        ),
        Role::Arbiter => format!(
            "Review rounds on {id} ({title}) ran out. It is {source}.\n\
             {pulls}The plan's checks:\n{checks}\
             Read the pull request, the review points and the author's answers. Decide whether it merges.\n\
             {hold}\n\
             {FIX_WHAT_IS_NOT_MET}\n\
             Then run: temet-work rule {id} approve \"<why>\"\n\
             or: temet-work rule {id} fix \"<what must change, and why>\"",
            title = issue.title,
            hold = hold_to_the_issue(id),
        ),
        Role::Investigator => format!(
            "Answer questions about {id}: {title}. It is {source}. Change no file.",
            title = issue.title,
        ),
    };
    Brief {
        role,
        issue: Some(id.clone()),
        task: with_rules(task, rules),
        model: None,
        permission: builtin_permission(),
        folder,
    }
}

/// The permission an agent starts with when no setting names one.
fn builtin_permission() -> String {
    match crate::settings::builtin("permission") {
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

/// A task ended by the kept rules, when there are any.
fn with_rules(task: String, rules: &[String]) -> String {
    if rules.is_empty() {
        task
    } else {
        let list: String = rules.iter().map(|r| format!("- {r}\n")).collect();
        format!("{task}\n\nRules from overturned decisions:\n{list}")
    }
}

/// One place an investigator files issues in: a project, or an area whose tracker
/// holds its repositories' issues.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Covered {
    /// The project or area, as `temet-work file` takes it.
    pub source: ProjectId,
    /// The tracker's name for it.
    pub tracker: String,
    /// Whether it is an area.
    pub area: bool,
    /// The repositories whose issues it holds, each with a checkout's path when
    /// one is known.
    pub repos: Vec<(ProjectId, Option<String>)>,
}

/// What an investigator is told about its group.
#[derive(Debug)]
pub struct Investigating<'a> {
    /// The group.
    pub group: &'a crate::model::Group,
    /// Where it files issues.
    pub covered: Vec<Covered>,
    /// The issues there that a person closed, in the tracker's order.
    pub closed: Vec<&'a Issue>,
    /// Each repository's `investigate` setting, under the repository's name.
    pub notes: Vec<(ProjectId, String)>,
    /// The milestone its issues get when it names none.
    pub milestone: Option<crate::ids::MilestoneId>,
    /// When the group's last investigation started, in words; none for its first.
    pub since: Option<String>,
}

/// Writes the brief for a group's investigator, run in `place`. It names no tool
/// or service: the investigator files issues through `temet-work file`.
pub fn investigate(what: &Investigating, place: Workplace, rules: &[String]) -> Brief {
    let g = what.group;
    let folder_note = if place.shared {
        "Another issue's author is changing files in this folder: do not switch its branch or change \
         anything here.\n"
    } else {
        ""
    };
    let covered: String = what
        .covered
        .iter()
        .map(|c| {
            let repos: Vec<String> = c
                .repos
                .iter()
                .map(|(r, path)| match path {
                    Some(p) => format!("{r} (a checkout is at {p})"),
                    None => r.to_string(),
                })
                .collect();
            if c.area {
                format!(
                    "- {}: {} in its tracker, for the area's repositories {}\n",
                    c.source,
                    c.tracker,
                    repos.join(", ")
                )
            } else {
                let at = c
                    .repos
                    .first()
                    .and_then(|(_, p)| p.as_deref())
                    .map_or_else(String::new, |p| format!("; a checkout is at {p}"));
                format!("- {}: {} in its tracker{at}\n", c.source, c.tracker)
            }
        })
        .collect();
    let milestone = match (&what.milestone, g.milestones.len()) {
        (Some(m), 1) => {
            format!("Each issue you file goes in milestone {m}, the one the group takes.\n")
        }
        (Some(m), _) => {
            let all: Vec<String> = g.milestones.iter().map(ToString::to_string).collect();
            format!(
                "Each issue you file goes in milestone {m}, the last the group takes; to file one in \
                 another the group takes, add --milestone M. The group takes, first to last: {}.\n",
                all.join(", ")
            )
        }
        (None, _) => "The group takes issues of every milestone; file them with none.\n".into(),
    };
    let since = what.since.as_ref().map_or_else(
        || {
            format!(
                "in the last {} days",
                crate::investigation::FIRST_SINCE_DAYS
            )
        },
        |s| format!("since {s}, when the group's last investigation started"),
    );
    let notes: String = what
        .notes
        .iter()
        .map(|(p, text)| format!("For {p}:\n{}\n", text.trim_end()))
        .collect();
    let notes = if notes.is_empty() {
        String::new()
    } else {
        format!("What the projects add:\n{notes}")
    };
    // With none a person closed, the brief says nothing of them.
    let (read_open, never, closed) = if what.closed.is_empty() {
        (
            "The open issues of each project, so you know what is covered already.",
            "Never file an issue an open one already covers.",
            String::new(),
        )
    } else {
        let list: Vec<String> = what
            .closed
            .iter()
            .map(|i| format!("{} {}", i.id, i.title))
            .collect();
        (
            "The open issues of each project, and the ones a person closed, listed below, so you \
             know what is covered already and what was turned down.",
            "Never file an issue that an open one, or one a person closed, already covers.",
            format!(
                "A person closed these issues. Never file one of them again, because closing an \
                 issue is how a person says no, and a person who wants it built reopens it: {}\n",
                list.join("; ")
            ),
        )
    };
    let task = format!(
        "Investigate for group {name} ({id}): its work queue is short. Find the work that should \
         come next and file an issue for each item.\n\
         {folder_note}It takes issues from:\n{covered}{milestone}\
         Read, in this order:\n\
         1. Each project's docs, for work they plan that is not built yet.\n\
         2. {read_open}\n\
         3. The pull requests merged {since}, for defects you can show in that work.\n\
         Then file one issue for each planned item no open issue covers, and one for each defect \
         you can show in merged work:\n\
         temet-work file PROJECT \"<title>\" - <<'BODY'\n\
         <what to change, and what done means>\n\
         BODY\n\
         PROJECT is one of the ids above. The reply names the new issue and the Depends on line \
         an issue that waits for it carries: file the issue another waits for first, and end the \
         waiting one's body with that line.\n\
         {never} Change no file.\n\
         {closed}{notes}\
         When you have filed every issue you found, run: temet-work investigated {id}",
        name = g.name,
        id = g.id,
    );
    Brief {
        role: Role::Investigator,
        issue: None,
        task: with_rules(task, rules),
        model: None,
        permission: builtin_permission(),
        folder: place.folder,
    }
}

/// What an investigator hears when it has been idle a while.
pub fn investigation_nudge(group: &crate::ids::GroupId) -> String {
    format!(
        "The investigation for {group} is still running. If you have filed every issue you found, \
         run: temet-work investigated {group}. If a choice stops you, take the default you would \
         defend, say so, and finish."
    )
}

/// The settings no agent may set.
fn only_a_persons() -> Vec<&'static str> {
    crate::settings::specs()
        .filter(|s| matches!(s.agents, crate::settings::Agents::MayNot(_)))
        .map(|s| s.key)
        .collect()
}

/// What an author is told when the review or the checks found things to fix.
pub fn fixes(issue: &IssueId, points: &[String]) -> String {
    let list: String = points.iter().map(|p| format!("- {p}\n")).collect();
    format!(
        "Fix these, from the review or the checks:\n{list}Then push and run: temet-work ready {issue}\n\
         When the points came from a review or a ruling, reply to each, in their order, with what you \
         did about it or why you did not: temet-work ready {issue} --reply \"<reply to the first>\" \
         [--reply \"<reply to the next>\" ...]"
    )
}

/// What a running reviewer is told when the author is ready again.
pub fn next_round(issue: &IssueId, round: u8) -> String {
    format!(
        "The author pushed fixes for {issue}; this is review round {round}. Review the whole change again, \
         not only the new commits. The issue's text may have changed since the last round, so read it again.\n\
         {hold}\n\
         {BLOCK_WHAT_IS_NOT_MET}\n\
         Then run: temet-work review {issue} approve\n\
         or: temet-work review {issue} block \"<what is still wrong>\" [\"<what else>\" ...]",
        hold = hold_to_the_issue(issue),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use crate::ids::{AgentId, FolderId, GroupId, MachineId, MilestoneId, ProjectId};
    use crate::model::{Check, Plan};
    use crate::world::World;

    #[test]
    fn the_refiners_brief_names_the_settings_only_a_person_sets() {
        let id = IssueId::new("ankh", "1");
        let mut world = World::default();
        world.apply(&Event::IssueSynced {
            issue: id.clone(),
            title: "Mend the clock".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        let brief = write(
            Role::Refiner,
            &world.issues[&id],
            None,
            &[],
            Workplace::default(),
            &[],
            &[],
        );
        let (_, named) = brief
            .task
            .split_once("Only a person sets ")
            .expect("the sentence");
        let (named, _) = named.split_once(':').expect("its end");
        let named: Vec<&str> = named.split(", ").collect();
        for key in ["runtime", "runner", "review_rounds"] {
            assert!(named.contains(&key), "{key} is not in {named:?}");
        }
        assert!(
            brief.task.contains(
                "A plain model is recorded as the global runtime's, as model.<that runtime>, \
                 when you write it, and is refused when the author runs another runtime; \
                 model.<runtime> names the runtime a model is for."
            ),
            "{}",
            brief.task
        );
    }

    #[test]
    fn a_reviewer_holds_the_change_against_the_issue_not_only_the_plan() {
        let id = IssueId::new("watch", "7");
        let mut world = World::default();
        // The issue's own text stays in ankh-morpork/watch. Its Done line there is
        // "Done is a real run in which the bell rings at midnight".
        world.apply(&Event::IssueSynced {
            issue: id.clone(),
            title: "Ring the bell at midnight".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        let mut issue = world.issues[&id].clone();
        // Neither check names a run.
        issue.plan = Some(Plan {
            checks: vec![
                Check {
                    done_when: "the bell rings at midnight".into(),
                    proven_by: "the midnight test".into(),
                },
                Check {
                    done_when: "a bell that jams is rung again and then reported".into(),
                    proven_by: "the jammed bell test".into(),
                },
            ],
            ..Plan::default()
        });
        let brief = write(
            Role::Reviewer,
            &issue,
            Some("ankh-morpork/watch"),
            &[],
            Workplace::default(),
            &[],
            &[],
        )
        .task;
        assert!(
            brief.contains("It is issue 7 of ankh-morpork/watch in its tracker."),
            "{brief}"
        );
        assert!(
            brief.contains("- a bell that jams is rung again and then reported (shown by: "),
            "{brief}"
        );
        let again = next_round(&id, 2);
        for line in [
            "run temet-work issue watch#7, which prints the issue's text from its tracker.",
            "Check each of its points and its \"Done is\" line against the change, not only the \
             plan's checks.",
            "Block on any point that is not met, and name that point in the block.",
            "confirm that the test or evidence named under \"shown by\" shows the whole of what \
             the check states, including any failure or persistent case it names.",
            "A test that shows only part of its check is a block.",
            "A real run the issue asks for must have happened, with its evidence in the pull \
             request. A plan that leaves it out does not excuse it.",
        ] {
            assert!(brief.contains(line), "missing {line:?} in:\n{brief}");
        }
        let shared = hold_to_the_issue(&id);
        assert!(brief.contains(&shared), "{brief}");
        assert!(again.contains(&shared), "{again}");
        assert!(
            again.contains("run temet-work issue watch#7, which prints"),
            "{again}"
        );
        assert!(again.contains("review round 2"), "{again}");
        assert!(
            again.contains("may have changed since the last round, so read it again."),
            "{again}"
        );
        assert!(
            again.ends_with(
                "Then run: temet-work review watch#7 approve\n\
                 or: temet-work review watch#7 block \"<what is still wrong>\" [\"<what else>\" ...]"
            ),
            "{again}"
        );
    }

    #[test]
    fn an_arbiter_holds_the_change_against_the_issue_not_only_the_plan() {
        let id = IssueId::new("watch", "12");
        let mut world = World::default();
        // The issue's own text stays in ankh-morpork/watch. Its Done line there is
        // "Done is a real run in which the gate is shut at dusk".
        world.apply(&Event::IssueSynced {
            issue: id.clone(),
            title: "Shut the gate at dusk".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        let mut issue = world.issues[&id].clone();
        // Neither check names a run.
        issue.plan = Some(Plan {
            checks: vec![
                Check {
                    done_when: "the gate shuts at dusk".into(),
                    proven_by: "the dusk test".into(),
                },
                Check {
                    done_when: "a gate that sticks is tried again and then reported".into(),
                    proven_by: "the stuck gate test".into(),
                },
            ],
            ..Plan::default()
        });
        let task = |role| {
            write(
                role,
                &issue,
                Some("ankh-morpork/watch"),
                &[],
                Workplace::default(),
                &[],
                &[],
            )
            .task
        };
        let brief = task(Role::Arbiter);
        for line in [
            "It is issue 12 of ankh-morpork/watch in its tracker.",
            "The plan's checks:\n\
             - the gate shuts at dusk (shown by: the dusk test)\n\
             - a gate that sticks is tried again and then reported (shown by: the stuck gate \
             test)\n",
            "run temet-work issue watch#12, which prints the issue's text from its tracker.",
            "Check each of its points and its \"Done is\" line against the change, not only the \
             plan's checks.",
            "confirm that the test or evidence named under \"shown by\" shows the whole of what \
             the check states, including any failure or persistent case it names.",
            "A real run the issue asks for must have happened, with its evidence in the pull \
             request. A plan that leaves it out does not excuse it.",
            "Where a point is not met, rule fix and name that point in the ruling.",
            "A test that shows only part of its check is a fix.",
            "An author's answer that the plan does not list the point is no reason to approve.",
            "A plan check or a recorded decision that departs from a point the issue states, or \
             that leaves the issue's failure scenario happening, is a fix, even when the plan \
             sanctions it.",
            "A decision the change writes down, such as under \"Decided, open to change\", is a \
             recorded decision.",
        ] {
            assert!(brief.contains(line), "missing {line:?} in:\n{brief}");
        }
        assert!(
            !brief.contains("block"),
            "an arbiter cannot block:\n{brief}"
        );
        let again = next_round(&id, 2);
        assert!(
            again.contains("run temet-work issue watch#12, which prints"),
            "{again}"
        );
        let shared = hold_to_the_issue(&id);
        for text in [&task(Role::Reviewer), &again, &brief] {
            assert!(text.contains(&shared), "{text}");
        }
        assert!(
            brief.ends_with(
                "Then run: temet-work rule watch#12 approve \"<why>\"\n\
                 or: temet-work rule watch#12 fix \"<what must change, and why>\""
            ),
            "{brief}"
        );
    }

    /// A synced issue of ankh-morpork/watch. Its text stays there, so its title
    /// carries the point a plan could depart from.
    fn lamp_issue() -> Issue {
        let id = IssueId::new("watch", "31");
        let mut world = World::default();
        world.apply(&Event::IssueSynced {
            issue: id.clone(),
            title: "A lamp with no oil: refuse it, or list it under Problems".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        world.issues[&id].clone()
    }

    fn lamp_task(role: Role, issue: &Issue) -> String {
        write(
            role,
            issue,
            Some("ankh-morpork/watch"),
            &[],
            Workplace::default(),
            &[],
            &[],
        )
        .task
    }

    #[test]
    fn an_author_fixing_a_review_round_is_told_to_reply_to_each_point() {
        let reply = "--reply \"<reply to the first>\" [--reply \"<reply to the next>\" ...]";
        let told = fixes(&IssueId::new("watch", "31"), &["light it at dusk".into()]);
        assert!(told.contains(reply), "{told}");
        // A new author on an issue sent back by a review hears it in its brief.
        let mut issue = lamp_issue();
        issue.stage = crate::model::Stage::Fixing { round: 1 };
        issue.fix = vec!["light it at dusk".into()];
        let brief = lamp_task(Role::Author, &issue);
        assert!(brief.contains(reply), "{brief}");
        // A fix for a failed check answers no review round.
        issue.fix_keeps_round = true;
        let brief = lamp_task(Role::Author, &issue);
        assert!(!brief.contains("--reply"), "{brief}");
    }

    #[test]
    fn a_refiner_meets_each_point_and_chooses_only_where_the_issue_leaves_it_open() {
        let brief = lamp_task(Role::Refiner, &lamp_issue());
        for line in [
            "A lamp with no oil: refuse it, or list it under Problems. It is issue 31 of \
             ankh-morpork/watch in its tracker.",
            "Each point the issue states needs a check that meets it as written.",
            "Use a chosen default only where the issue leaves the choice open (it says \"or\", \
             \"pick\" or \"record which\"): pick one of the options it names, the one you would \
             defend, and say why in done_when.",
            "When a point cannot be done as written, say why in that point's check and fix the \
             issue's failure scenario some other way: the plan may not leave it in place.",
        ] {
            assert!(brief.contains(line), "missing {line:?} in:\n{brief}");
        }
        assert!(
            !brief.contains(
                "Where the issue leaves a choice open, pick the default you would defend"
            ),
            "{brief}"
        );
    }

    #[test]
    fn a_departure_from_the_issue_blocks_the_review_even_when_the_plan_sanctions_it() {
        let mut issue = lamp_issue();
        issue.plan = Some(Plan {
            checks: vec![Check {
                done_when: "Chosen rule: lamps on the Watch House's own oil are neither refused \
                            nor listed under Problems"
                    .into(),
                proven_by: "the Watch House lamp test".into(),
            }],
            ..Plan::default()
        });
        let brief = lamp_task(Role::Reviewer, &issue);
        assert!(
            brief.contains("refuse it, or list it under Problems. It is issue 31 of"),
            "{brief}"
        );
        assert!(
            brief.contains(
                "The plan's checks:\n\
                 - Chosen rule: lamps on the Watch House's own oil are neither refused nor listed \
                 under Problems (shown by: the Watch House lamp test)\n"
            ),
            "{brief}"
        );
        let again = next_round(&issue.id, 2);
        for line in [
            "A plan check or a recorded decision that departs from a point the issue states, or \
             that leaves the issue's failure scenario happening, is a block, even when the plan \
             sanctions it.",
            "A decision the change writes down, such as under \"Decided, open to change\", is a \
             recorded decision.",
        ] {
            assert!(brief.contains(line), "missing {line:?} in:\n{brief}");
            assert!(again.contains(line), "missing {line:?} in:\n{again}");
        }
    }

    /// A world where the group watch, over ankh and the area guild (whose tracker
    /// holds its repositories' issues), investigates; `since_ms` is when its last
    /// investigation started, if it had one.
    fn investigating(since_ms: Option<u64>) -> World {
        let mut events = vec![
            Event::ProjectAdded {
                project: ProjectId::new("ankh"),
                tracker: "fake".into(),
                name: "city/ankh".into(),
                code_host: None,
                repo: None,
            },
            Event::ProjectAdded {
                project: ProjectId::new("ui"),
                tracker: String::new(),
                name: String::new(),
                code_host: Some("fake".into()),
                repo: Some("guild/ui".into()),
            },
            Event::AreaDefined {
                area: ProjectId::new("guild"),
                name: "Thieves' Guild".into(),
                projects: vec![ProjectId::new("ui")],
                tracker: "fake".into(),
                tracker_name: "GLD".into(),
            },
            Event::FolderAdded {
                folder: FolderId::new("f1"),
                project: ProjectId::new("ankh"),
                machine: MachineId::new("morpork"),
                path: "/srv/ankh/1".into(),
            },
            Event::GroupCreated {
                group: GroupId::new("watch"),
                name: "Night watch".into(),
                projects: vec![ProjectId::new("ankh")],
                areas: vec![ProjectId::new("guild")],
                milestones: vec![MilestoneId::new("First"), MilestoneId::new("Second")],
                refining_at_once: 1,
                model: None,
                queue_low: None,
                investigate_every: None,
            },
        ];
        let start = |at_ms| Event::InvestigationStarted {
            group: GroupId::new("watch"),
            agent: AgentId::new("investigator-1"),
            at_ms,
        };
        if let Some(at) = since_ms {
            events.push(start(at));
            events.push(Event::InvestigationEnded {
                group: GroupId::new("watch"),
                at_ms: at + 1,
                filed: 0,
            });
        }
        events.push(start(1_795_784_700_000 + 7_200_000));
        World::replay(&events).0
    }

    fn task(w: &World) -> String {
        w.launch_investigator(&GroupId::new("watch"))
            .expect("launch")
            .brief
            .task
    }

    #[test]
    fn the_investigator_brief_names_projects_milestone_and_since() {
        let t = task(&investigating(Some(1_795_784_700_000)));
        for line in [
            "Investigate for group Night watch (watch): its work queue is short.",
            "It takes issues from:\n\
             - ankh: city/ankh in its tracker; a checkout is at /srv/ankh/1\n\
             - guild: GLD in its tracker, for the area's repositories ui\n",
            "Each issue you file goes in milestone Second, the last the group takes; to file one \
             in another the group takes, add --milestone M. The group takes, first to last: First, \
             Second.",
            "1. Each project's docs, for work they plan that is not built yet.",
            "2. The open issues of each project, so you know what is covered already.",
            "3. The pull requests merged since 2026-11-27 13:05 UTC, when the group's last \
             investigation started, for defects you can show in that work.",
            "Then file one issue for each planned item no open issue covers, and one for each \
             defect you can show in merged work:\ntemet-work file PROJECT \"<title>\" - <<'BODY'\n",
            "the Depends on line an issue that waits for it carries",
            "Never file an issue an open one already covers. Change no file.",
        ] {
            assert!(t.contains(line), "missing {line:?} in:\n{t}");
        }
        assert!(
            t.ends_with(
                "When you have filed every issue you found, run: temet-work investigated watch"
            ),
            "{t}"
        );
    }

    #[test]
    fn a_first_investigation_reads_the_last_seven_days() {
        let t = task(&investigating(None));
        assert!(
            t.contains("3. The pull requests merged in the last 7 days, for defects"),
            "{t}"
        );
        assert!(!t.contains("since"), "{t}");
    }
}
