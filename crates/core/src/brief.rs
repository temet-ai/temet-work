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
    /// Whether another issue's author holds that folder. An investigator is told
    /// so; a refiner, reviewer or arbiter is told to leave its folder alone
    /// either way.
    pub shared: bool,
    /// An author's work folders, one per repository the plan names.
    pub held: Vec<Held>,
}

/// One of an issue's pull requests: the project, the code host's name for its
/// repository, the number there, its branch, and whether it has merged or a
/// person closed it.
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
    /// Its branch, once the code host has reported it.
    pub branch: Option<String>,
    /// Whether a person closed it without merging, so it takes no more pushes:
    /// the author reopens it or opens another in its repository.
    pub closed: bool,
}

/// An issue's pull requests, one per line, each with its branch when one is on
/// record.
fn pull_list(pulls: &[Pulled]) -> String {
    pulls
        .iter()
        .map(|p| {
            let branch = p
                .branch
                .as_ref()
                .map_or_else(String::new, |b| format!(", branch {b}"));
            let state = if p.merged {
                " (merged already)"
            } else if p.closed {
                " (closed without merging)"
            } else {
                ""
            };
            format!(
                "- {} ({}): pull request {}{branch}{state}\n",
                p.project, p.repo, p.number
            )
        })
        .collect()
}

/// How to check out `branch`: plain git, the same on every code host. A work
/// folder is a person's own checkout, so no remote is named.
fn switch_to(branch: &str) -> String {
    format!("git fetch, then git switch {branch}")
}

/// What a refiner, reviewer and arbiter are told about the folder they run in.
/// They hold none, and an author may be given it while they work, so each hears
/// this whether or not the folder was held at its start.
const LEAVE_THE_FOLDER: &str = "\
This folder is not yours: an author may be changing files in it or be given it while you work, and \
it may be on another issue's branch. Do not switch its branch or change any file in it. Read a \
pull request's changes from its code host. To run checks, make a separate checkout of the pull \
request (with none, of the commit you need) outside this folder, and remove it when you are done.";

/// What an author is told about making its change, and the end of its brief: the
/// `ready` command for `issue`. An author on an issue with pull requests is given
/// each one with its branch and how to check it out, and opens one only in a
/// repository that has none, or where a person closed the one on record, which
/// it may reopen instead.
fn authors_work(issue: &IssueId, held: &[Held], pulls: &[Pulled]) -> (String, String) {
    let several = held.len() > 1;
    let folders = if several {
        let list: String = held
            .iter()
            .map(|h| format!("- {} ({}): {}\n", h.project, h.repo, h.path))
            .collect();
        format!(
            "This change spans several repositories. You hold a folder in each; you start in the first:\n{list}"
        )
    } else {
        String::new()
    };
    // The repositories with no pull request at all.
    let lacking: Vec<&Held> = held
        .iter()
        .filter(|h| !pulls.iter().any(|p| p.project == h.project))
        .collect();
    let closed: Vec<&Pulled> = pulls.iter().filter(|p| p.closed).collect();
    // The ones still taking pushes.
    let open: Vec<&Pulled> = pulls.iter().filter(|p| !p.merged && !p.closed).collect();
    // `ready` has to name a number in each repository with none, and in each
    // where a person closed the one on record.
    let mut needs: Vec<&ProjectId> = held
        .iter()
        .map(|h| &h.project)
        .filter(|h| !pulls.iter().any(|p| &p.project == *h && !p.closed))
        .collect();
    for p in &closed {
        if !needs.contains(&&p.project) {
            needs.push(&p.project);
        }
    }
    let several_repos = several || pulls.len() > 1;
    let numbers = if several_repos {
        needs
            .iter()
            .map(|p| format!(" {p}=<pull request number>"))
            .collect()
    } else if pulls.is_empty() || !needs.is_empty() {
        " <pull request number>".to_string()
    } else {
        String::new()
    };
    if pulls.is_empty() {
        let open = if several {
            "Make the branch and open the pull requests yourself: use the same branch name in every repository \
             and open one pull request in each.\n"
        } else {
            ""
        };
        return (
            format!(" Make a branch, change the code and open a pull request.\n{folders}{open}"),
            format!(
                "When the pull request is open and every check holds, run: temet-work ready {issue}{numbers}"
            ),
        );
    }
    let has = if pulls.len() > 1 {
        "pull requests"
    } else {
        "a pull request"
    };
    // Where to check out each branch the fixes go to, before pushing to it.
    let switches: Vec<String> = open
        .iter()
        .filter_map(|p| {
            let branch = p.branch.as_ref()?;
            Some(if several_repos {
                format!("in the folder of {}, {}", p.project, switch_to(branch))
            } else {
                switch_to(branch)
            })
        })
        .collect();
    let checkout = match switches.as_slice() {
        [] => String::new(),
        [one] if !several_repos => format!("Check out its branch first: {one}."),
        _ => format!("Check out each branch first: {}.", switches.join("; ")),
    };
    let marked: Vec<&str> = [
        (pulls.iter().any(|p| p.merged), "(merged already)"),
        (!closed.is_empty(), "(closed without merging)"),
    ]
    .into_iter()
    .filter_map(|(any, mark)| any.then_some(mark))
    .collect();
    // When a closed one leaves none open, there is no branch to push to.
    let push = match (open.len(), marked.is_empty(), pulls.len()) {
        (0, ..) if !closed.is_empty() => String::new(),
        (_, true, 1) => "Push your fixes to that pull request's branch.".to_string(),
        (_, true, _) => "Push your fixes to each pull request's branch.".to_string(),
        (_, false, _) => format!(
            "Push your fixes to the branch of each one not marked {}.",
            marked.join(" or ")
        ),
    };
    // Merged or not: `ready` with no number for a repository keeps the one on
    // record, so the loop would not follow a second one. One a person closed is
    // no longer on record: its repository is named below.
    let kept: Vec<String> = pulls
        .iter()
        .filter(|p| !p.closed)
        .map(|p| p.project.to_string())
        .collect();
    let no_other = if !closed.is_empty() {
        if kept.is_empty() {
            String::new()
        } else {
            format!("Do not open another pull request in {}.", kept.join(", "))
        }
    } else if lacking.is_empty() {
        "Do not open another pull request.".to_string()
    } else {
        "Do not open another pull request in a repository that has one, merged or not.".to_string()
    };
    let said: Vec<&str> = [checkout.as_str(), push.as_str(), no_other.as_str()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    let said = if said.is_empty() {
        String::new()
    } else {
        format!("{}\n", said.join(" "))
    };
    // Pushing to a closed pull request does not reopen it.
    let reopen: String = closed
        .iter()
        .map(|p| {
            let there = if several_repos {
                format!(" in {}", p.project)
            } else {
                String::new()
            };
            let start = p.branch.as_ref().map_or_else(String::new, |b| {
                let folder = if several_repos {
                    format!("in the folder of {}, ", p.project)
                } else {
                    String::new()
                };
                format!(
                    " A new one can start from its branch, {b}: {folder}{}.",
                    switch_to(b)
                )
            });
            format!(
                "Pull request {}{there} was closed without merging: reopen it, or open a new pull \
                 request{there}.{start}\n",
                p.number
            )
        })
        .collect();
    let open_new = if lacking.is_empty() {
        String::new()
    } else {
        let list: Vec<String> = lacking
            .iter()
            .map(|h| format!("{} ({})", h.project, h.repo))
            .collect();
        format!(
            "Open a pull request only where the issue has none yet: {}. Use the same branch name there \
             as in the other repositories.\n",
            list.join(", ")
        )
    };
    let when = if open.is_empty() && !closed.is_empty() {
        "When the pull request is open and every check holds"
    } else {
        "When your fixes are pushed and every check holds"
    };
    (
        format!(
            "\n{folders}The issue already has {has}:\n{list}{said}{reopen}{open_new}",
            list = pull_list(pulls)
        ),
        format!("{when}, run: temet-work ready {issue}{numbers}"),
    )
}

/// Said beside every fetch a brief asks for: an agent told to change no file
/// would otherwise skip it and read a stale default branch.
const FETCHING_IS_ALLOWED: &str =
    "Fetching is allowed: it moves no branch and changes no file an author works on.";

/// Sends a reader to a default branch's head, since the files in a folder may be
/// on another issue's branch or an older one. `read` says what to read there,
/// `stand` whose files not to read it from, and `outside` where a separate
/// checkout goes.
fn at_the_default_branch(read: &str, stand: &str, outside: &str) -> String {
    format!(
        "{read}, not as the files in {stand} stand: fetch, then read with git show from the \
         remote-tracking branch the fetch updated (a local branch may be behind), or in a separate \
         checkout of that commit {outside}, which you remove when you are done. \
         {FETCHING_IS_ALLOWED}"
    )
}

/// What a refiner's plain `model` becomes (`docs/settings.md`, "A model belongs
/// to a runtime").
const PLAIN_MODEL: &str = "A plain model is recorded as the global runtime's, as \
model.<that runtime>, when you write it, and is refused when the author runs another runtime; \
model.<runtime> names the runtime a model is for.";

/// Sends an agent to the issue's own text, which its brief does not carry.
fn read_the_issue(issue: &IssueId) -> String {
    format!(
        "Read the issue's own text: run temet-work issue {issue}, which prints the issue's text \
         from its tracker."
    )
}

/// What a change is held against: by a reviewer in its brief and again on each
/// later round, and by an arbiter. Each adds what it does with a point that is
/// not met.
fn hold_to_the_issue(issue: &IssueId) -> String {
    let read = read_the_issue(issue);
    format!(
        "{read} Check each of its points and its \"Done is\" line against the change, not \
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
states, or that leaves any failure the issue describes happening, is a block, even when the \
plan sanctions it. A decision the change writes down, such as under \"Decided, open to \
change\", is a recorded decision.";

/// What an arbiter does with a point that is not met, or that the plan or a
/// decision departs from.
const FIX_WHAT_IS_NOT_MET: &str = "\
Where a point is not met, rule fix and name that point in the ruling. A test that shows only part \
of its check is a fix. An author's answer that the plan does not list the point is no reason to \
approve. A plan check or a recorded decision that departs from a point the issue states, or that \
leaves any failure the issue describes happening, is a fix, even when the plan sanctions it. A \
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
    let Workplace { folder, held, .. } = place;
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
    let area_read = if repos.is_empty() {
        ""
    } else {
        " Do this for each of the area's repositories the change may touch, at the head of its \
         own default branch; clone one you have no checkout of, outside this folder."
    };
    let checks = issue.plan.as_ref().map_or_else(String::new, |plan| {
        plan.checks
            .iter()
            .map(|c| format!("- {} (shown by: {})\n", c.done_when, c.proven_by))
            .collect()
    });
    // An author started on an issue that came back hears what to fix first, and
    // the points its ready replies to.
    let fixes = if issue.fix.is_empty() {
        String::new()
    } else {
        let list: String = issue.fix.iter().map(|p| format!("- {p}\n")).collect();
        let reply = issue.block_to_answer().map_or_else(String::new, |block| {
            format!(
                "Your ready answers review round {}, so it takes one reply for each of that \
                 round's points, in this order, {SAYING}:\n{}",
                block.round,
                numbered(&block.points)
            )
        });
        format!("Earlier work on it came back with these to fix first:\n{list}{reply}")
    };
    let replies = issue.block_to_answer().map_or_else(String::new, |block| {
        format!(" {}", reply_flags(block.points.len()))
    });
    let (work, ready) = authors_work(id, &held, pulls);
    // A reader is not sent to one a person closed.
    let reviewed: Vec<Pulled> = pulls.iter().filter(|p| !p.closed).cloned().collect();
    let to_review = if reviewed.len() > 1 {
        format!(
            "It changes several repositories; review every one of its pull requests:\n{}",
            pull_list(&reviewed)
        )
    } else {
        String::new()
    };
    let task = match role {
        Role::Refiner => format!(
            "Refine {id}: {title}. It is {source}.\n\
             {LEAVE_THE_FOLDER}\n{area_note}Read the issue. {code}{area_read} Change no file here.\n\
             Then send the plan, one entry per check, as YAML on standard input:\n\
             temet-work plan {id} - <<'PLAN'\n\
             checks:\n  - done_when: one outcome a person can observe\n    proven_by: the command, test or page that shows it\n\
             PLAN\n\
             Each point the issue states needs a check that meets it as written. \
             Use a chosen default only where the issue leaves the choice open (it says \"or\", \"pick\" or \
             \"record which\"): pick one of the options it names, the one you would defend, and say why \
             in done_when. \
             Where the issue says nothing about a choice, take the default you would defend and say \
             why in done_when; a default never replaces a point the issue states. \
             When a point cannot be done as written, say why in that point's check and fix any \
             failure the issue describes some other way: the plan may not leave one in place.\n\
             For an issue that is hard, add a settings: map beside checks: with task-level values, such as model: opus; \
             they are recorded as yours and a person can overturn them. \
             A model you set is the author's only: the models of reviewers, arbiters and refiners are \
             a person's to set. \
             {PLAIN_MODEL} \
             Only a person sets {persons}: a plan that names one is refused.",
            title = issue.title,
            code = at_the_default_branch(
                "Read the code it touches at the current head of the repository's default branch",
                "this folder",
                "outside this folder",
            ),
            persons = only_a_persons().join(", "),
        ),
        Role::Author => format!(
            "Work on {id}: {title}. It is {source}.\n\
             {read} A decision may not depart from a point the issue states or leave any failure it \
             describes in place. Where a point cannot be met, say so in the pull request.\n\
             This folder is yours until the change merges.{work}\
             {fixes}The plan's checks, each to hold before you finish:\n{checks}\
             {ready}{replies}",
            title = issue.title,
            read = read_the_issue(id),
        ),
        Role::Reviewer => format!(
            "Review the pull request for {id}: {title}. It is {source}.\n\
             {to_review}{judged}\
             {LEAVE_THE_FOLDER}\n\
             Read the whole change, not only the last commit. Run the checks yourself where you can, \
             in that separate checkout.\n\
             {hold}\n\
             {BLOCK_WHAT_IS_NOT_MET}\n\
             {verdict}",
            title = issue.title,
            judged = to_judge(issue, under_review(issue)),
            hold = hold_to_the_issue(id),
            verdict = how_to_judge(issue, under_review(issue), "<what to fix>"),
        ),
        Role::Arbiter => format!(
            "Review rounds on {id} ({title}) ran out. It is {source}.\n\
             {to_review}The plan's checks:\n{checks}\
             {rounds}\
             {LEAVE_THE_FOLDER}\n\
             Read the pull request and the review rounds listed above, with the author's replies; \
             temet-work issue {id} lists the rounds again. Decide whether it merges.\n\
             {hold}\n\
             {FIX_WHAT_IS_NOT_MET}\n\
             Then run: temet-work rule {id} approve \"<why>\"\n\
             or: temet-work rule {id} fix \"<what must change, and why>\"",
            title = issue.title,
            rounds = crate::text::review_rounds(issue),
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
                    Some(p) => format!("{r} (a checkout to fetch in is at {p})"),
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
                    .map_or_else(String::new, |p| {
                        format!("; a checkout to fetch in is at {p}")
                    });
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
         {docs_and_code}\n\
         Read, in this order:\n\
         1. Each project's docs at that head, for work they plan that is not built yet.\n\
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
        docs_and_code = at_the_default_branch(
            "Read each repository's docs and code at the current head of that repository's \
             default branch",
            "a checkout",
            "outside the checkouts named above and outside the folder you run in",
        ),
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

/// How an author is to reply to each point of a block.
const SAYING: &str = "saying what you did about it or why you did not";

/// `items` as a list numbered from 1, one to a line.
pub(crate) fn numbered(items: &[String]) -> String {
    (1..)
        .zip(items)
        .map(|(n, p)| format!("{n}. {p}\n"))
        .collect()
}

/// One `--reply` for each of `count` points, as `temet-work ready` takes them.
pub(crate) fn reply_flags(count: usize) -> String {
    (1..=count)
        .map(|n| format!("--reply \"<reply to point {n}>\""))
        .collect::<Vec<_>>()
        .join(" ")
}

/// What an author is told when the review or the checks found things to fix
/// (`points`) on `issue`, which changes `repos`. When its next ready answers a
/// block, it is told that block's points, numbered, and to reply to each. A
/// repository with no pull request on record, as after a person closed it,
/// needs a number in the ready.
pub fn fixes(issue: &Issue, repos: &[ProjectId], points: &[String]) -> String {
    let id = &issue.id;
    let lacking = repos.iter().filter(|r| !issue.pulls.contains_key(*r));
    let numbers: String = if repos.len() > 1 {
        lacking
            .map(|r| format!(" {r}=<pull request number>"))
            .collect()
    } else if lacking.count() > 0 {
        " <pull request number>".into()
    } else {
        String::new()
    };
    let Some(block) = issue.block_to_answer() else {
        let list: String = points.iter().map(|p| format!("- {p}\n")).collect();
        return format!(
            "Fix these, from the review or the checks:\n{list}Then push and run: temet-work ready \
             {id}{numbers}"
        );
    };
    let run = format!(
        "temet-work ready {id}{numbers} {}\nA ready without one reply for each point is refused.",
        reply_flags(block.points.len())
    );
    if points == block.points.as_slice() {
        format!(
            "Review round {round} blocked {id}. Fix each of its points:\n{list}\
             Then push and run this, with one reply for each point, in this order, {SAYING}:\n{run}",
            round = block.round,
            list = numbered(points),
        )
    } else {
        let list: String = points.iter().map(|p| format!("- {p}\n")).collect();
        format!(
            "Fix these, from the review or the checks:\n{list}\
             Then push and run this, with one reply for each point of review round {round}, in this \
             order, {SAYING}:\n{points}{run}",
            round = block.round,
            points = numbered(&block.points),
        )
    }
}

/// The review round `issue` is in, or its latest.
fn under_review(issue: &Issue) -> u8 {
    match issue.stage {
        crate::model::Stage::InReview { round } => round,
        _ => issue.round,
    }
}

/// What a reviewer of round `round` of `issue` judges besides the change: the
/// plan's checks, numbered as `--failed` takes them, and the points of the round
/// before with the author's reply to each, numbered as `--open` takes them.
fn to_judge(issue: &Issue, round: u8) -> String {
    let checks: String = issue
        .plan
        .iter()
        .flat_map(|p| &p.checks)
        .zip(1..)
        .map(|(c, n)| format!("{n}. {} (shown by: {})\n", c.done_when, c.proven_by))
        .collect();
    let checks = format!("The plan's checks, numbered as --failed takes them:\n{checks}");
    let Some(before) = issue.round_before(round).filter(|r| r.blocks()) else {
        return checks;
    };
    let points: String = (1..)
        .zip(&before.points)
        .map(|(n, p)| {
            let reply = before
                .replies
                .get(n - 1)
                .map_or("none on record", String::as_str);
            format!("{n}. {p}\n   The author's reply: {reply}\n")
        })
        .collect();
    format!(
        "{checks}Review round {} blocked on these points, numbered as --open takes them:\n{points}",
        before.round
    )
}

/// How a reviewer of round `round` of `issue` gives its verdict, with
/// `what` as the placeholder for a block's first point.
fn how_to_judge(issue: &Issue, round: u8, what: &str) -> String {
    let id = &issue.id;
    let before = issue
        .round_before(round)
        .filter(|r| r.blocks())
        .map(|r| r.round);
    let (open_flag, open, neither) = match before {
        Some(before) => (
            " [--open N ...]",
            format!(
                " Give --open N for each point of review round {before} that is still not dealt \
                 with: it stays open and joins your block's points, so the author fixes it again \
                 and the next round judges it again. Every point you leave unnamed counts as \
                 resolved, and a block may consist of kept points alone."
            ),
            " and resolves every point, and takes neither --failed nor --open",
        ),
        None => ("", String::new(), " and takes no --failed"),
    };
    format!(
        "Then run: temet-work review {id} approve\n\
         or: temet-work review {id} block \"{what}\" [\"<what else>\" ...] [--failed N ...]{open_flag}\n\
         On a block, give --failed N for each check that does not hold; a check you leave unnamed \
         counts as passed.{open} An approve passes every check{neither}."
    )
}

/// What a running reviewer is told when the author is ready again for round
/// `round` of `issue`.
pub fn next_round(issue: &Issue, round: u8) -> String {
    let id = &issue.id;
    format!(
        "The author pushed fixes for {id}; this is review round {round}. Review the whole change again, \
         not only the new commits. The issue's text may have changed since the last round, so read it again.\n\
         {judged}\
         {LEAVE_THE_FOLDER}\n\
         {hold}\n\
         {BLOCK_WHAT_IS_NOT_MET}\n\
         {verdict}",
        judged = to_judge(issue, round),
        hold = hold_to_the_issue(id),
        verdict = how_to_judge(issue, round, "<what is still wrong>"),
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
         If a choice stops you, take the default you would defend (where the issue leaves it open, \
         one of the options it names; never in place of a point the issue states), say so, and \
         finish."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use crate::ids::{AgentId, FolderId, GroupId, MachineId, MilestoneId, ProjectId};
    use crate::model::{Check, Plan, Stage};
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
    fn a_reader_is_told_to_leave_its_folder_alone_held_or_not() {
        let issue = lamp_issue();
        let told = [
            "an author may be changing files in it or be given it while you work",
            "Do not switch its branch or change any file in it.",
            "Read a pull request's changes from its code host.",
            "To run checks, make a separate checkout of the pull request (with none, of the commit \
             you need) outside this folder, and remove it when you are done.",
        ];
        for role in [Role::Refiner, Role::Reviewer, Role::Arbiter] {
            for shared in [false, true] {
                let place = Workplace {
                    shared,
                    ..Workplace::default()
                };
                let task = write(role, &issue, None, &[], place, &[], &[]).task;
                for line in told {
                    assert!(
                        task.contains(line),
                        "{role:?}, shared {shared}: missing {line:?} in:\n{task}"
                    );
                }
            }
        }
        let reviewer = lamp_task(Role::Reviewer, &issue);
        assert!(
            reviewer.contains("Run the checks yourself where you can, in that separate checkout."),
            "{reviewer}"
        );
        // A running reviewer hears it again on each later round.
        let again = next_round(&issue, 2);
        for line in told {
            assert!(again.contains(line), "missing {line:?} in:\n{again}");
        }
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
                    state: None,
                },
                Check {
                    done_when: "a bell that jams is rung again and then reported".into(),
                    proven_by: "the jammed bell test".into(),
                    state: None,
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
            brief.contains("2. a bell that jams is rung again and then reported (shown by: "),
            "{brief}"
        );
        let again = next_round(&issue, 2);
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
                 or: temet-work review watch#7 block \"<what is still wrong>\" [\"<what else>\" ...] \
                 [--failed N ...]\n\
                 On a block, give --failed N for each check that does not hold; a check you leave \
                 unnamed counts as passed. An approve passes every check and takes no --failed."
            ),
            "{again}"
        );
    }

    #[test]
    fn a_reviewer_is_told_the_checks_and_the_points_of_the_round_before_by_number() {
        let mut issue = lamp_issue();
        issue.plan = Some(Plan {
            checks: vec![
                Check::new("a lamp with no oil is refused", "the dry lamp test"),
                Check::new(
                    "a refused lamp is listed under Problems",
                    "the problems test",
                ),
            ],
            ..Plan::default()
        });
        issue.stage = crate::model::Stage::InReview { round: 2 };
        issue.round = 2;
        issue.reviews = vec![crate::model::Review {
            round: 1,
            reviewer: Some(AgentId::new("reviewer-3")),
            approve: false,
            points: vec!["the wick is untrimmed".into(), "the glass is sooty".into()],
            replies: vec!["trimmed it".into()],
            states: Vec::new(),
        }];
        let numbered = "The plan's checks, numbered as --failed takes them:\n\
                        1. a lamp with no oil is refused (shown by: the dry lamp test)\n\
                        2. a refused lamp is listed under Problems (shown by: the problems test)\n\
                        Review round 1 blocked on these points, numbered as --open takes them:\n\
                        1. the wick is untrimmed\n   The author's reply: trimmed it\n\
                        2. the glass is sooty\n   The author's reply: none on record\n";
        let how = "or: temet-work review watch#31 block \"<what is still wrong>\" [\"<what else>\" ...] \
                   [--failed N ...] [--open N ...]\n\
                   On a block, give --failed N for each check that does not hold; a check you leave \
                   unnamed counts as passed. Give --open N for each point of review round 1 that is \
                   still not dealt with: it stays open and joins your block's points, so the author \
                   fixes it again and the next round judges it again. Every point you leave unnamed \
                   counts as resolved, and a block may consist of kept points alone. An approve passes \
                   every check and resolves every point, and takes neither --failed nor --open.";
        let again = next_round(&issue, 2);
        assert!(again.contains(numbered), "{again}");
        assert!(again.ends_with(how), "{again}");
        // A reviewer started on round 2 hears the same in its brief.
        let brief = lamp_task(Role::Reviewer, &issue);
        assert!(brief.contains(numbered), "{brief}");
        assert!(
            brief.ends_with(&how.replace("<what is still wrong>", "<what to fix>")),
            "{brief}"
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
                    state: None,
                },
                Check {
                    done_when: "a gate that sticks is tried again and then reported".into(),
                    proven_by: "the stuck gate test".into(),
                    state: None,
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
             that leaves any failure the issue describes happening, is a fix, even when the plan \
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
        assert!(!brief.contains("failure scenario"), "{brief}");
        let again = next_round(&issue, 2);
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
        let reply = "--reply \"<reply to point 1>\" --reply \"<reply to point 2>\"";
        let points = vec!["light it at dusk".to_string(), "trim the wick".into()];
        let mut issue = lamp_issue();
        issue.stage = crate::model::Stage::Fixing { round: 1 };
        issue.fix = points.clone();
        let watch = [ProjectId::new("watch")];
        issue.pulls.insert(watch[0].clone(), "57".into());
        issue.reviews = vec![crate::model::Review {
            round: 1,
            reviewer: Some(AgentId::new("reviewer-3")),
            approve: false,
            points: points.clone(),
            replies: Vec::new(),
            states: Vec::new(),
        }];
        // The running author hears the points numbered, once, and how to reply.
        let told = fixes(&issue, &watch, &points);
        assert!(
            told.starts_with(
                "Review round 1 blocked watch#31. Fix each of its points:\n\
                 1. light it at dusk\n\
                 2. trim the wick\n\
                 Then push and run this, with one reply for each point, in this order, saying \
                 what you did about it or why you did not:\n"
            ),
            "{told}"
        );
        assert!(
            told.ends_with(&format!(
                "temet-work ready watch#31 {reply}\nA ready without one reply for each point is refused."
            )),
            "{told}"
        );
        // A new author hears them in its brief.
        let brief = lamp_task(Role::Author, &issue);
        for line in [
            "Your ready answers review round 1, so it takes one reply for each of that round's \
             points, in this order, saying what you did about it or why you did not:\n\
             1. light it at dusk\n2. trim the wick\n",
            &format!("run: temet-work ready watch#31 <pull request number> {reply}"),
        ] {
            assert!(brief.contains(line), "missing {line:?} in:\n{brief}");
        }
        // After an arbiter's fix ruling, the author fixes what the ruling says and
        // replies to the reviewer's last block.
        let ruling = ["the wick must be trimmed by hand".to_string()];
        let told = fixes(&issue, &watch, &ruling);
        assert!(
            told.contains(
                "Fix these, from the review or the checks:\n\
                 - the wick must be trimmed by hand\n\
                 Then push and run this, with one reply for each point of review round 1, in this \
                 order, saying what you did about it or why you did not:\n\
                 1. light it at dusk\n2. trim the wick\n"
            ),
            "{told}"
        );
        // Once the author has replied, a later fix answers no block.
        issue.reviews[0].replies = vec!["lit".into(), "trimmed".into()];
        issue.fix_keeps_round = true;
        let brief = lamp_task(Role::Author, &issue);
        assert!(!brief.contains("--reply"), "{brief}");
        let told = fixes(&issue, &watch, &["CI check lint failed".into()]);
        assert_eq!(
            told,
            "Fix these, from the review or the checks:\n- CI check lint failed\n\
             Then push and run: temet-work ready watch#31"
        );
    }

    #[test]
    fn an_author_reads_the_issue_and_may_not_depart_from_it() {
        let brief = lamp_task(Role::Author, &lamp_issue());
        for line in [
            "Read the issue's own text: run temet-work issue watch#31, which prints the issue's \
             text from its tracker.",
            "A decision may not depart from a point the issue states or leave any failure it \
             describes in place.",
            "Where a point cannot be met, say so in the pull request.",
        ] {
            assert!(brief.contains(line), "missing {line:?} in:\n{brief}");
        }
    }

    /// The author's task on an issue of `held` folders and `pulls` pull requests,
    /// with one check in its plan: an issue being fixed when it has pull requests,
    /// else one being worked on.
    fn author_task(held: &[(&str, &str)], pulls: &[(&str, &str, &str, bool)]) -> String {
        let stage = if pulls.is_empty() {
            Stage::Working
        } else {
            Stage::Fixing { round: 1 }
        };
        author_task_in(stage, held, pulls)
    }

    /// [`author_task`] on an issue in `stage`, with a point to fix when that is
    /// `Fixing`.
    fn author_task_in(
        stage: Stage,
        held: &[(&str, &str)],
        pulls: &[(&str, &str, &str, bool)],
    ) -> String {
        let pulls: Vec<Pulled> = pulls
            .iter()
            .map(|(project, repo, number, merged)| Pulled {
                project: ProjectId::new(*project),
                repo: (*repo).into(),
                number: (*number).into(),
                merged: *merged,
                branch: None,
                closed: false,
            })
            .collect();
        author_task_of(stage, held, &pulls)
    }

    /// [`author_task_in`] with the pull requests given whole, branch and all.
    fn author_task_of(stage: Stage, held: &[(&str, &str)], pulls: &[Pulled]) -> String {
        let mut issue = lamp_issue();
        issue.plan = Some(Plan {
            checks: vec![Check::new("a dry lamp is refused", "the dry lamp test")],
            ..Plan::default()
        });
        issue.stage = stage;
        if matches!(issue.stage, Stage::Fixing { .. }) {
            issue.fix = vec!["CI check tests failed".into()];
            issue.fix_keeps_round = true;
        }
        let held = held
            .iter()
            .map(|(project, repo)| Held {
                project: ProjectId::new(*project),
                repo: (*repo).into(),
                path: format!("/srv/{project}"),
            })
            .collect();
        write(
            Role::Author,
            &issue,
            Some("ankh-morpork/watch"),
            &[],
            Workplace {
                held,
                ..Workplace::default()
            },
            pulls,
            &[],
        )
        .task
    }

    const TWO_HELD: [(&str, &str); 2] = [
        ("watch", "ankh-morpork/watch"),
        ("clacks", "ankh-morpork/clacks"),
    ];

    #[test]
    fn the_first_authors_brief_asks_for_a_new_pull_request() {
        assert_eq!(
            author_task(&TWO_HELD[..1], &[]),
            "Work on watch#31: A lamp with no oil: refuse it, or list it under Problems. It is issue \
             31 of ankh-morpork/watch in its tracker.\n\
             Read the issue's own text: run temet-work issue watch#31, which prints the issue's text \
             from its tracker. A decision may not depart from a point the issue states or leave any \
             failure it describes in place. Where a point cannot be met, say so in the pull request.\n\
             This folder is yours until the change merges. Make a branch, change the code and open a \
             pull request.\n\
             The plan's checks, each to hold before you finish:\n\
             - a dry lamp is refused (shown by: the dry lamp test)\n\
             When the pull request is open and every check holds, run: temet-work ready watch#31 \
             <pull request number>"
        );
        assert_eq!(
            author_task(&TWO_HELD, &[]),
            "Work on watch#31: A lamp with no oil: refuse it, or list it under Problems. It is issue \
             31 of ankh-morpork/watch in its tracker.\n\
             Read the issue's own text: run temet-work issue watch#31, which prints the issue's text \
             from its tracker. A decision may not depart from a point the issue states or leave any \
             failure it describes in place. Where a point cannot be met, say so in the pull request.\n\
             This folder is yours until the change merges. Make a branch, change the code and open a \
             pull request.\n\
             This change spans several repositories. You hold a folder in each; you start in the \
             first:\n\
             - watch (ankh-morpork/watch): /srv/watch\n\
             - clacks (ankh-morpork/clacks): /srv/clacks\n\
             Make the branch and open the pull requests yourself: use the same branch name in every \
             repository and open one pull request in each.\n\
             The plan's checks, each to hold before you finish:\n\
             - a dry lamp is refused (shown by: the dry lamp test)\n\
             When the pull request is open and every check holds, run: temet-work ready watch#31 \
             watch=<pull request number> clacks=<pull request number>"
        );
    }

    #[test]
    fn a_new_author_is_given_the_issues_pull_request_and_told_not_to_open_another() {
        for held in [&TWO_HELD[..0], &TWO_HELD[..1]] {
            let task = author_task(held, &[("watch", "ankh-morpork/watch", "57", false)]);
            assert!(
                task.contains(
                    "The issue already has a pull request:\n\
                     - watch (ankh-morpork/watch): pull request 57\n\
                     Push your fixes to that pull request's branch. Do not open another pull request.\n\
                     Earlier work on it came back with these to fix first:\n\
                     - CI check tests failed\n"
                ),
                "{task}"
            );
            assert!(!task.contains("open a pull request"), "{task}");
            assert!(!task.contains("<pull request number>"), "{task}");
            assert!(
                task.ends_with(
                    "When your fixes are pushed and every check holds, run: temet-work ready watch#31"
                ),
                "{task}"
            );
        }
    }

    #[test]
    fn a_new_author_is_given_every_pull_request_and_pushes_to_those_not_merged() {
        let task = author_task(
            &TWO_HELD,
            &[
                ("watch", "ankh-morpork/watch", "57", true),
                ("clacks", "ankh-morpork/clacks", "12", false),
            ],
        );
        assert!(
            task.contains(
                "The issue already has pull requests:\n\
                 - watch (ankh-morpork/watch): pull request 57 (merged already)\n\
                 - clacks (ankh-morpork/clacks): pull request 12\n\
                 Push your fixes to the branch of each one not marked (merged already). Do not open \
                 another pull request.\n"
            ),
            "{task}"
        );
        assert!(!task.contains("open one pull request in each"), "{task}");
        assert!(!task.contains("open a pull request"), "{task}");
        assert!(!task.contains("<pull request number>"), "{task}");
        assert!(task.ends_with("run: temet-work ready watch#31"), "{task}");

        // Neither merged: it pushes to both and opens no other.
        let task = author_task(
            &TWO_HELD,
            &[
                ("watch", "ankh-morpork/watch", "57", false),
                ("clacks", "ankh-morpork/clacks", "12", false),
            ],
        );
        assert!(
            task.contains(
                "- watch (ankh-morpork/watch): pull request 57\n\
                 - clacks (ankh-morpork/clacks): pull request 12\n\
                 Push your fixes to each pull request's branch. Do not open another pull request.\n"
            ),
            "{task}"
        );
    }

    #[test]
    fn a_new_author_opens_a_pull_request_only_where_the_issue_has_none() {
        let task = author_task(&TWO_HELD, &[("watch", "ankh-morpork/watch", "57", false)]);
        assert!(
            task.contains(
                "- watch (ankh-morpork/watch): pull request 57\n\
                 Push your fixes to that pull request's branch. Do not open another pull request in a \
                 repository that has one, merged or not.\n\
                 Open a pull request only where the issue has none yet: clacks \
                 (ankh-morpork/clacks). Use the same branch name there as in the other \
                 repositories.\n"
            ),
            "{task}"
        );
        assert!(!task.contains("open one pull request in each"), "{task}");
        assert!(
            task.ends_with("run: temet-work ready watch#31 clacks=<pull request number>"),
            "{task}"
        );

        // The one on record merged: no other there either.
        let task = author_task(&TWO_HELD, &[("watch", "ankh-morpork/watch", "57", true)]);
        assert!(
            task.contains(
                "- watch (ankh-morpork/watch): pull request 57 (merged already)\n\
                 Push your fixes to the branch of each one not marked (merged already). Do not open \
                 another pull request in a repository that has one, merged or not.\n"
            ),
            "{task}"
        );
    }

    #[test]
    fn the_pull_requests_not_the_stage_decide_what_an_author_is_told_to_open() {
        let pulls = [("watch", "ankh-morpork/watch", "57", false)];
        let task = author_task_in(Stage::Working, &TWO_HELD[..1], &pulls);
        assert!(
            task.contains(
                "The issue already has a pull request:\n\
                 - watch (ankh-morpork/watch): pull request 57\n\
                 Push your fixes to that pull request's branch. Do not open another pull request.\n"
            ),
            "{task}"
        );
        assert!(!task.contains("open a pull request"), "{task}");
        assert!(task.ends_with("run: temet-work ready watch#31"), "{task}");

        // Being fixed with none: the first author's text, with the point to fix.
        for held in [&TWO_HELD[..1], &TWO_HELD[..]] {
            let first = author_task_in(Stage::Working, held, &[]);
            assert_eq!(
                author_task_in(Stage::Fixing { round: 1 }, held, &[]),
                first.replace(
                    "The plan's checks,",
                    "Earlier work on it came back with these to fix first:\n\
                     - CI check tests failed\n\
                     The plan's checks,"
                )
            );
        }
    }

    /// Pull request `number` in `project` of the `TWO_HELD` repositories, open,
    /// on `branch` when one is given.
    fn pulled(project: &str, number: &str, branch: Option<&str>) -> Pulled {
        Pulled {
            project: ProjectId::new(project),
            repo: format!("ankh-morpork/{project}"),
            number: number.into(),
            merged: false,
            branch: branch.map(Into::into),
            closed: false,
        }
    }

    #[test]
    fn an_author_is_told_its_pull_requests_branch_and_how_to_check_it_out() {
        let night = Some("watch-12-night-shift");
        let fixing = Stage::Fixing { round: 1 };
        for held in [&TWO_HELD[..0], &TWO_HELD[..1]] {
            let task = author_task_of(fixing, held, &[pulled("watch", "57", night)]);
            assert!(
                task.contains(
                    "The issue already has a pull request:\n\
                     - watch (ankh-morpork/watch): pull request 57, branch watch-12-night-shift\n\
                     Check out its branch first: git fetch, then git switch watch-12-night-shift. \
                     Push your fixes to that pull request's branch. Do not open another pull \
                     request.\n"
                ),
                "{task}"
            );
            let at = |s: &str| task.find(s).unwrap_or_else(|| panic!("no {s:?} in {task}"));
            assert!(at("git fetch") < at("git switch watch-12-night-shift"));
            assert!(at("git switch watch-12-night-shift") < at("Push your fixes"));
            assert!(task.ends_with("run: temet-work ready watch#31"), "{task}");
        }

        // Several repositories: each branch is checked out in its own folder.
        let task = author_task_of(
            fixing,
            &TWO_HELD,
            &[
                pulled("watch", "57", night),
                pulled("clacks", "12", Some("watch-12-semaphore")),
            ],
        );
        assert!(
            task.contains(
                "- watch (ankh-morpork/watch): pull request 57, branch watch-12-night-shift\n\
                 - clacks (ankh-morpork/clacks): pull request 12, branch watch-12-semaphore\n\
                 Check out each branch first: in the folder of watch, git fetch, then git switch \
                 watch-12-night-shift; in the folder of clacks, git fetch, then git switch \
                 watch-12-semaphore. Push your fixes to each pull request's branch."
            ),
            "{task}"
        );

        // Closed: its branch is where a new pull request can start.
        let closed = Pulled {
            closed: true,
            ..pulled("watch", "57", night)
        };
        let task = author_task_of(fixing, &TWO_HELD[..1], std::slice::from_ref(&closed));
        assert!(
            task.contains(
                "- watch (ankh-morpork/watch): pull request 57, branch watch-12-night-shift (closed \
                 without merging)\n\
                 Pull request 57 was closed without merging: reopen it, or open a new pull request. \
                 A new one can start from its branch, watch-12-night-shift: git fetch, then git \
                 switch watch-12-night-shift.\n"
            ),
            "{task}"
        );
        for not in [
            "Push your fixes",
            "Do not open another pull request",
            "Check out its",
        ] {
            assert!(!task.contains(not), "{not:?} in {task}");
        }
        assert!(
            task.ends_with(
                "When the pull request is open and every check holds, run: temet-work ready \
                 watch#31 <pull request number>"
            ),
            "{task}"
        );

        // No branch on record: none is named, and no empty one is printed.
        for pulls in [
            vec![pulled("watch", "57", None)],
            vec![Pulled {
                branch: None,
                ..closed
            }],
        ] {
            let task = author_task_of(fixing, &TWO_HELD[..1], &pulls);
            for not in ["branch ", "git switch", "branch,", "start from"] {
                assert!(!task.contains(not), "{not:?} in {task}");
            }
            assert!(
                task.contains("- watch (ankh-morpork/watch): pull request 57"),
                "{task}"
            );
        }
    }

    #[test]
    fn a_reminder_limits_the_default_a_stuck_agent_may_take() {
        let id = lamp_issue().id;
        for role in [Role::Refiner, Role::Author, Role::Reviewer, Role::Arbiter] {
            let text = nudge(role, &id);
            assert!(
                text.starts_with("watch#31 is still waiting on you"),
                "{text}"
            );
            assert!(
                text.ends_with(
                    "If a choice stops you, take the default you would defend (where the issue \
                     leaves it open, one of the options it names; never in place of a point the \
                     issue states), say so, and finish."
                ),
                "{text}"
            );
            assert!(
                !text.contains("take the default you would defend, say so"),
                "{text}"
            );
        }
    }

    #[test]
    fn a_refiner_meets_each_point_and_chooses_only_where_the_issue_leaves_it_open() {
        let brief = lamp_task(Role::Refiner, &lamp_issue());
        let open = "Use a chosen default only where the issue leaves the choice open (it says \
                    \"or\", \"pick\" or \"record which\"): pick one of the options it names, the one \
                    you would defend, and say why in done_when.";
        let silent = "Where the issue says nothing about a choice, take the default you would \
                      defend and say why in done_when; a default never replaces a point the issue \
                      states.";
        assert!(brief.contains(&format!("{open} {silent}")), "{brief}");
        for line in [
            "A lamp with no oil: refuse it, or list it under Problems. It is issue 31 of \
             ankh-morpork/watch in its tracker.",
            "Each point the issue states needs a check that meets it as written.",
            "When a point cannot be done as written, say why in that point's check and fix any \
             failure the issue describes some other way: the plan may not leave one in place.",
        ] {
            assert!(brief.contains(line), "missing {line:?} in:\n{brief}");
        }
        assert!(
            !brief.contains(
                "Where the issue leaves a choice open, pick the default you would defend"
            ),
            "{brief}"
        );
        assert!(!brief.contains("failure scenario"), "{brief}");
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
                state: None,
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
                "The plan's checks, numbered as --failed takes them:\n\
                 1. Chosen rule: lamps on the Watch House's own oil are neither refused nor listed \
                 under Problems (shown by: the Watch House lamp test)\n"
            ),
            "{brief}"
        );
        let again = next_round(&issue, 2);
        for line in [
            "A plan check or a recorded decision that departs from a point the issue states, or \
             that leaves any failure the issue describes happening, is a block, even when the plan \
             sanctions it.",
            "A decision the change writes down, such as under \"Decided, open to change\", is a \
             recorded decision.",
        ] {
            assert!(brief.contains(line), "missing {line:?} in:\n{brief}");
            assert!(again.contains(line), "missing {line:?} in:\n{again}");
        }
        for text in [&brief, &again] {
            assert!(!text.contains("failure scenario"), "{text}");
        }
    }

    /// The sentences of `text` that tell its reader to read something.
    fn reading(text: &str) -> Vec<&str> {
        text.split('\n')
            .flat_map(|line| line.split_inclusive(". "))
            .map(str::trim)
            .filter(|s| {
                s.split(|c: char| !c.is_alphabetic())
                    .any(|word| word.eq_ignore_ascii_case("read"))
            })
            .collect()
    }

    #[test]
    fn a_refiner_reads_the_default_branch_not_the_files_in_its_folder() {
        let code = "Read the code it touches at the current head of the repository's default \
                    branch, not as the files in this folder stand: fetch, then read with git \
                    show from the remote-tracking branch the fetch updated (a local branch may be \
                    behind), or in a separate checkout of that commit outside this folder, which \
                    you remove when you are done. Fetching is allowed: it moves no branch and \
                    changes no file an author works on.";
        let each = " Do this for each of the area's repositories the change may touch, at the head \
                    of its own default branch; clone one you have no checkout of, outside this \
                    folder.";
        let area = [ProjectId::new("ui"), ProjectId::new("docs")];
        for repos in [&area[..0], &area[..]] {
            for shared in [false, true] {
                let place = Workplace {
                    shared,
                    ..Workplace::default()
                };
                let task = write(Role::Refiner, &lamp_issue(), None, repos, place, &[], &[]).task;
                let each = if repos.is_empty() { "" } else { each };
                let told = format!("Read the issue. {code}{each} Change no file here.\n");
                assert!(task.contains(&told), "shared {shared}:\n{task}");
                // Nothing else in the brief sends it to read code.
                let mut rest = reading(&task);
                rest.retain(|s| !told.contains(s));
                assert_eq!(
                    rest,
                    ["Read a pull request's changes from its code host."],
                    "{task}"
                );
                assert!(!task.contains("touches in this folder"), "{task}");
            }
        }
    }

    #[test]
    fn an_investigator_reads_each_checkout_at_its_default_branch() {
        let docs_and_code = "Read each repository's docs and code at the current head of that \
                             repository's default branch, not as the files in a checkout stand: \
                             fetch, then read with git show from the remote-tracking branch the \
                             fetch updated (a local branch may be behind), or in a separate \
                             checkout of that commit outside the checkouts named above and outside \
                             the folder you run in, which you remove when you are done. Fetching is \
                             allowed: it moves no branch and changes no file an author works on.";
        for held in [false, true] {
            let mut world = investigating(None);
            world.apply(&Event::FolderAdded {
                folder: FolderId::new("f2"),
                project: ProjectId::new("ui"),
                machine: MachineId::new("morpork"),
                path: "/srv/ui/1".into(),
            });
            if held {
                let issue = IssueId::new("ankh", "1");
                world.apply(&Event::IssueSynced {
                    issue: issue.clone(),
                    title: "Mend the clock".into(),
                    milestone: Some(MilestoneId::new("First")),
                    depends_on: vec![],
                    order: 1,
                    closed: false,
                });
                world.apply(&Event::PlanWritten {
                    issue,
                    plan: Plan::default(),
                });
            }
            let folder = &world.folders[&FolderId::new("f1")];
            assert_eq!(folder.holder.is_some(), held);
            let launch = world
                .launch_investigator(&GroupId::new("watch"))
                .expect("launch");
            assert_eq!(launch.brief.folder, Some(PathBuf::from("/srv/ankh/1")));
            let task = launch.brief.task;
            // The lines that name a path give it as a place to fetch in.
            let named: Vec<&str> = task.lines().filter(|l| l.contains("/srv/")).collect();
            assert_eq!(
                named,
                [
                    "- ankh: city/ankh in its tracker; a checkout to fetch in is at /srv/ankh/1",
                    "- guild: GLD in its tracker, for the area's repositories ui (a checkout to \
                     fetch in is at /srv/ui/1)",
                ],
                "held {held}:\n{task}"
            );
            assert!(
                task.contains(&format!(
                    "{docs_and_code}\nRead, in this order:\n1. Each project's docs at that head, "
                )),
                "held {held}:\n{task}"
            );
            // Nothing else in the brief sends it to read docs or code.
            let mut rest = reading(&task);
            rest.retain(|s| !docs_and_code.contains(s));
            assert_eq!(rest, ["Read, in this order:"], "held {held}:\n{task}");
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
             - ankh: city/ankh in its tracker; a checkout to fetch in is at /srv/ankh/1\n\
             - guild: GLD in its tracker, for the area's repositories ui\n",
            "Each issue you file goes in milestone Second, the last the group takes; to file one \
             in another the group takes, add --milestone M. The group takes, first to last: First, \
             Second.",
            "1. Each project's docs at that head, for work they plan that is not built yet.",
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
