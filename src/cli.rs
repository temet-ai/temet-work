//! The command line's arguments, read into a request for the daemon.

use std::path::Path;

use temet_work_core::ids::{AgentId, DecisionId, GroupId, IssueId, ProjectId, RuleId};
use temet_work_core::model::{Plan, Role};
use temet_work_core::settings::Target;
use temet_work_core::view::StageWord;
use temet_work_core::{Command, Request, Verdict};

pub use temet_work_core::launch::{AGENT_VAR, SECRET_VAR};

/// How to call `temet-work`.
pub const USAGE: &str = "\
temet-work status [--json]                  the state as text, or with --json as data
temet-work issue ISSUE                      one issue in full, with its text from the last sync
temet-work screen AGENT                     what an agent's terminal shows now
temet-work plan ISSUE FILE                  a refiner's plan, as YAML; FILE - reads it from standard input
temet-work ready ISSUE PULL|REPO=PULL...     the pull request is open and the checks hold;
                                            PULL is its number, needed the first time;
                                            the first ready after a block takes
                                            --reply TEXT once for each of its points
temet-work review ISSUE approve
temet-work review ISSUE block POINT...      what to fix, one argument each; --failed N
                                            for each check that does not hold, and
                                            --open N for each point of the round before
                                            still not dealt with, each counted from 1
temet-work rule ISSUE approve REASON
temet-work rule ISSUE fix REASON
temet-work choose ISSUE --question TEXT --option TEXT --option TEXT [--option TEXT...]
           --took TEXT --why TEXT --cost low|medium|high
                                            an agent's: record a choice on the issue it works on, as
                                            a decision a person can overturn; --took is one of the
                                            options as written, --cost what overturning it costs;
                                            work carries on at once
temet-work overturn DECISION OPTION REASON
                                            a person's: choose another option of a decision, as
                                            `status` numbers them; REASON is required
temet-work proposal RULE keep|drop          a person's: keep or drop a rule proposed from an overturn
temet-work hold                             a person's: hold the daemon, so no issue is taken, no
                                            investigation starts, and an issue with nothing merged
                                            gets no new agent, fix, review or merge; agents at
                                            work finish their turn
temet-work resume                           a person's: lift the hold, so work starts again
temet-work move ISSUE STAGE                 a person's: move an issue back to new, to be refined
                                            again, or to ready, for a new author; STAGE is one of
                                            new, refining, ready, working, in_review, fixing,
                                            arbitration, merging, deploying, verifying, done, and
                                            any other move is refused with its reason
temet-work file PROJECT TITLE BODY [--milestone M]
                                            an investigator's: file an issue in a project or area its
                                            group takes; BODY - reads it from standard input; the
                                            reply names the new issue
temet-work investigated GROUP               an investigator's: its group's investigation is done
temet-work set TARGET [--role ROLE] KEY VALUE
                                            give a setting a value at one level
temet-work unset TARGET [--role ROLE] KEY   remove it, so the next level answers
temet-work settings TARGET [--role ROLE]    every setting in effect there, and the level that set it
temet-work mcp [--allow read|act]           the same views and commands as MCP tools, for agents;
                                            read (the default) offers status, issue and screen
temet-work parts                            the parts in this build
temet-work --version                        the version and the build id

ISSUE is project#key, such as radar#44. TARGET is global, an area or project id, a
milestone as project:Milestone name, or an issue. ROLE is refiner, author, reviewer,
arbiter or investigator. Agents' commands are recorded as theirs
through TEMET_WORK_AGENT and carry the secret in TEMET_WORK_AGENT_TOKEN; the
daemon sets both when it starts an agent.";

/// The flag that asks `status` for data.
pub const JSON: &str = "--json";

/// The flag before each of the author's replies in `ready`.
const REPLY: &str = "--reply";

/// The flag before each check a reviewer's block finds does not hold.
const FAILED: &str = "--failed";

/// The flag before each point of the round before a reviewer's block keeps open.
const OPEN: &str = "--open";

/// What `--json` on a command other than `status` is answered with.
pub fn json_refused() -> String {
    format!("{JSON} goes with status only\n\n{USAGE}")
}

/// Whether `args` give [`JSON`] as a flag. In a command's free text (a value, a
/// title or body, a review point or a reason) it is a word like any other.
fn has_json(args: &[String]) -> bool {
    let word = |i: usize| args.get(i).map(String::as_str);
    let before_text = match word(0) {
        Some("file" | "choose") => 2,
        Some("set" | "rule" | "overturn") => 3,
        Some("review") if word(2) == Some("block") => 3,
        Some("ready") => args.iter().position(|a| a == REPLY).unwrap_or(args.len()),
        _ => args.len(),
    };
    args.iter().take(before_text).any(|a| a == JSON)
}

/// A plan from its YAML text.
pub fn plan(yaml: &str) -> Result<Plan, String> {
    serde_saphyr::from_str(yaml).map_err(|e| format!("the plan is not readable: {e}"))
}

/// Takes `--role ROLE` out of `args`, wherever it is.
fn take_role(args: &[String]) -> Result<(Vec<String>, Option<Role>), String> {
    let (rest, role) = take(args, "--role")?;
    Ok((rest, role.as_deref().map(Role::parse).transpose()?))
}

/// Takes `flag VALUE` out of `args`, wherever it is.
fn take(args: &[String], flag: &str) -> Result<(Vec<String>, Option<String>), String> {
    let mut rest = Vec::new();
    let mut value = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == flag {
            let v = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
            value = Some(v.clone());
        } else {
            rest.push(a.clone());
        }
    }
    Ok((rest, value))
}

/// Takes every `flag N` out of `args`, wherever it is, with each N read as a
/// number counted from 1; `what` says what N counts.
fn take_numbers(
    args: &[String],
    flag: &str,
    what: &str,
) -> Result<(Vec<String>, Vec<usize>), String> {
    let mut rest = Vec::new();
    let mut numbers = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == flag {
            let v = it
                .next()
                .ok_or_else(|| format!("{flag} needs the number of {what}"))?;
            numbers.push(v.parse().map_err(|_| {
                format!("{flag} takes the number of {what}, counted from 1, not {v}")
            })?);
        } else {
            rest.push(a.clone());
        }
    }
    Ok((rest, numbers))
}

/// Reads `choose ISSUE --question TEXT --option TEXT ... --took TEXT --why TEXT
/// --cost COST`, its flags in any order. Each flag's text is the argument after
/// it, whatever it says, so a text may be `--json` or `--role`. A flag left out
/// reads as empty and a cost left out as none, and the daemon refuses the choice
/// with the reason, as it does a choice from MCP or the socket.
fn choose(args: &[String]) -> Result<Command, String> {
    let issue = args
        .get(1)
        .ok_or_else(|| format!("missing ISSUE\n\n{USAGE}"))?;
    let issue = IssueId::parse(issue)
        .ok_or_else(|| format!("{issue} is not an issue; write project#key, such as radar#44"))?;
    let (mut question, mut took, mut why, mut cost) = (None, None, None, None);
    let mut options = Vec::new();
    let mut rest = args.get(2..).unwrap_or_default().iter();
    while let Some(flag) = rest.next() {
        let slot = match flag.as_str() {
            "--option" | "--question" | "--took" | "--why" | "--cost" => flag.as_str(),
            other => {
                return Err(format!(
                    "choose takes --question, --option, --took, --why and --cost, each followed \
                     by its text; {other} is none of them\n\n{USAGE}"
                ));
            }
        };
        let text = rest
            .next()
            .ok_or_else(|| format!("{slot} needs its text"))?
            .clone();
        let once = match slot {
            "--option" => {
                options.push(text);
                continue;
            }
            "--question" => &mut question,
            "--took" => &mut took,
            "--why" => &mut why,
            _ => &mut cost,
        };
        if once.replace(text).is_some() {
            return Err(format!("{slot} is given twice; give it once"));
        }
    }
    Ok(Command::Choose {
        issue,
        question: question.unwrap_or_default(),
        options,
        took: took.unwrap_or_default(),
        why: why.unwrap_or_default(),
        cost,
    })
}

/// Reads the arguments after the program name into a request. `by` is the
/// agent sending it; `read` reads a file. The request carries no secret: the
/// caller adds the one in [`SECRET_VAR`].
pub fn request(
    args: &[String],
    by: Option<String>,
    read: impl Fn(&Path) -> Result<String, String>,
) -> Result<Request, String> {
    let by = by.filter(|b| !b.is_empty()).map(AgentId::new);
    // Every argument of a choice after its issue is a flag or a flag's text, so
    // no flag is taken out of it first.
    if args.first().is_some_and(|a| a == "choose") {
        if has_json(args) {
            return Err(json_refused());
        }
        return Ok(Request::new(by, choose(args)?));
    }
    let (args, role) = take_role(args)?;
    let (args, milestone) = take(&args, "--milestone")?;
    let args = args.as_slice();
    let arg = |i: usize, what: &str| {
        args.get(i)
            .map(String::as_str)
            .ok_or_else(|| format!("missing {what}\n\n{USAGE}"))
    };
    let issue = || {
        let s = arg(1, "ISSUE")?;
        IssueId::parse(s)
            .ok_or_else(|| format!("{s} is not an issue; write project#key, such as radar#44"))
    };
    let name = arg(0, "a command")?;
    let json = has_json(args);
    if json && name != "status" {
        return Err(json_refused());
    }
    let command = match name {
        "status" if json => Command::View,
        "status" => Command::Status,
        "issue" => Command::Issue { issue: issue()? },
        "screen" => Command::Screen {
            agent: AgentId::new(arg(1, "AGENT")?),
        },
        "plan" => Command::Plan {
            issue: issue()?,
            // `-` is standard input, so an agent can send a plan without writing a file.
            plan: plan(&read(Path::new(arg(2, "FILE")?))?)?,
        },
        "ready" => {
            let (mut pull, mut pulls) = (None, std::collections::BTreeMap::new());
            let mut replies = Vec::new();
            let mut rest = args.get(2..).unwrap_or_default().iter();
            while let Some(a) = rest.next() {
                if a == REPLY {
                    let text = rest
                        .next()
                        .ok_or_else(|| format!("{REPLY} needs the reply's text"))?;
                    replies.push(text.clone());
                    continue;
                }
                match a.split_once('=') {
                    Some((repo, number)) if !repo.is_empty() && !number.is_empty() => {
                        pulls.insert(ProjectId::new(repo), number.to_string());
                    }
                    Some(_) => return Err(format!("{a} is not repo=number")),
                    None if pull.is_none() => pull = Some(a.clone()),
                    None => {
                        return Err(
                            "name one bare pull request number, or repo=number for each".into()
                        );
                    }
                }
            }
            Command::Ready {
                issue: issue()?,
                pull,
                pulls,
                replies,
            }
        }
        "review" => {
            let (rest, failed) = take_numbers(
                args,
                FAILED,
                "a check, in the plan's order, that does not hold",
            )?;
            let (rest, open) = take_numbers(
                &rest,
                OPEN,
                "a point of the round before, in its order, still not dealt with",
            )?;
            let verdict = rest
                .get(2)
                .map(String::as_str)
                .ok_or_else(|| format!("missing approve or block\n\n{USAGE}"))?;
            Command::Review {
                issue: issue()?,
                verdict: match verdict {
                    "approve" => Verdict::Approve,
                    "block" => Verdict::Block {
                        points: rest.get(3..).unwrap_or_default().to_vec(),
                    },
                    other => return Err(format!("review takes approve or block, not {other}")),
                },
                failed,
                open,
            }
        }
        "retry" => Command::Retry { issue: issue()? },
        "move" => Command::Move {
            issue: issue()?,
            stage: StageWord::parse(arg(2, "STAGE")?)?,
        },
        "hold" => Command::Hold,
        "resume" => Command::Resume,
        "rule" => Command::Rule {
            issue: issue()?,
            approve: match arg(2, "approve or fix")? {
                "approve" => true,
                "fix" => false,
                other => return Err(format!("rule takes approve or fix, not {other}")),
            },
            reason: args.get(3..).unwrap_or_default().join(" "),
        },
        "overturn" => {
            let number = |i: usize, what: &str| {
                let s = arg(i, what)?;
                s.parse::<usize>().map_err(|_| {
                    format!("{s} is not a number; {what} is the number `status` lists")
                })
            };
            Command::Overturn {
                decision: DecisionId(number(1, "DECISION")? as u64),
                option: number(2, "OPTION")?,
                reason: args.get(3..).unwrap_or_default().join(" "),
            }
        }
        "proposal" => Command::Proposal {
            rule: {
                let s = arg(1, "RULE")?;
                RuleId(s.parse().map_err(|_| {
                    format!("{s} is not a rule number; `status` lists them under Rules")
                })?)
            },
            keep: match arg(2, "keep or drop")? {
                "keep" => true,
                "drop" => false,
                other => return Err(format!("proposal takes keep or drop, not {other}")),
            },
        },
        "set" => Command::Set {
            target: Target::parse(arg(1, "TARGET")?),
            role,
            key: arg(2, "KEY")?.to_string(),
            // A command such as `make deploy` may arrive as separate words.
            value: {
                arg(3, "VALUE")?;
                args[3..].join(" ")
            },
        },
        "unset" => Command::Unset {
            target: Target::parse(arg(1, "TARGET")?),
            role,
            key: arg(2, "KEY")?.to_string(),
        },
        "settings" => Command::Settings {
            target: Target::parse(arg(1, "TARGET")?),
            role,
        },
        "file" => Command::File {
            project: ProjectId::new(arg(1, "PROJECT")?),
            title: arg(2, "TITLE")?.to_string(),
            // `-` is standard input, so a body can run over several lines.
            body: match arg(3, "BODY")? {
                "-" => read(Path::new("-"))?,
                _ => args[3..].join(" "),
            },
            milestone,
        },
        "investigated" => Command::Investigated {
            group: GroupId::new(arg(1, "GROUP")?),
        },
        other => return Err(format!("no command {other}\n\n{USAGE}")),
    };
    Ok(Request::new(by, command))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split(' ').map(String::from).collect()
    }

    fn no_files(_: &Path) -> Result<String, String> {
        Err("no files in this test".into())
    }

    #[test]
    fn move_reads_an_issue_and_one_of_the_views_stage_words() {
        let read = |line: &str| request(&args(line), None, no_files).map(|r| r.command);
        assert_eq!(
            read("move radar#1 ready"),
            Ok(Command::Move {
                issue: IssueId::new("radar", "1"),
                stage: StageWord::Ready,
            })
        );
        assert_eq!(
            read("move radar#1 in_review"),
            Ok(Command::Move {
                issue: IssueId::new("radar", "1"),
                stage: StageWord::InReview,
            })
        );
        assert_eq!(
            read("move radar#1 sideways"),
            Err(
                "no stage sideways; the stages are new, refining, ready, working, in_review, \
                 fixing, arbitration, merging, deploying, verifying, done"
                    .into()
            )
        );
        assert!(read("move radar#1").is_err_and(|e| e.starts_with("missing STAGE")));
        assert!(read("move radar ready").is_err_and(|e| e.contains("not an issue")));
        assert!(
            read("move radar#1 new --json").is_err(),
            "--json goes with status only"
        );
        assert!(USAGE.contains("temet-work move ISSUE STAGE"));
    }

    #[test]
    fn hold_and_resume_are_read_with_no_arguments() {
        let read = |line: &str| request(&args(line), None, no_files).map(|r| r.command);
        assert_eq!(read("hold"), Ok(Command::Hold));
        assert_eq!(read("resume"), Ok(Command::Resume));
        assert!(read("hold --json").is_err(), "--json goes with status only");
        assert!(USAGE.contains("temet-work hold ") && USAGE.contains("temet-work resume "));
    }

    #[test]
    fn ready_reads_a_bare_number_or_repo_number_pairs() {
        let ready = |line: &str| request(&args(line), None, no_files).map(|r| r.command);
        assert_eq!(
            ready("ready guild#GLD-1 ui=12 api=34"),
            Ok(Command::Ready {
                issue: IssueId::new("guild", "GLD-1"),
                pull: None,
                pulls: [
                    (ProjectId::new("ui"), "12".to_string()),
                    (ProjectId::new("api"), "34".to_string())
                ]
                .into(),
                replies: Vec::new(),
            })
        );
        assert_eq!(
            ready("ready ankh#1 57"),
            Ok(Command::Ready {
                issue: IssueId::new("ankh", "1"),
                pull: Some("57".into()),
                pulls: Default::default(),
                replies: Vec::new(),
            })
        );
        assert!(ready("ready ankh#1 57 58").is_err());
        assert!(ready("ready ankh#1 ui=").is_err());
    }

    #[test]
    fn ready_takes_a_reply_after_each_reply_flag_and_json_in_one_is_a_word() {
        let words = |words: &[&str]| words.iter().map(ToString::to_string).collect::<Vec<_>>();
        let ready = |line: &[&str]| request(&words(line), None, no_files).map(|r| r.command);
        assert_eq!(
            ready(&[
                "ready",
                "ankh#1",
                "--reply",
                "added a test for a stopped clock",
                "57",
                "--reply",
                "--json stays as it was",
            ]),
            Ok(Command::Ready {
                issue: IssueId::new("ankh", "1"),
                pull: Some("57".into()),
                pulls: Default::default(),
                replies: vec![
                    "added a test for a stopped clock".into(),
                    "--json stays as it was".into()
                ],
            })
        );
        assert!(
            ready(&["ready", "ankh#1", "--reply"])
                .is_err_and(|e| e.contains("needs the reply's text"))
        );
        assert!(ready(&["ready", "ankh#1", "--json", "--reply", "x"]).is_err());
    }

    /// The words of `author-1`'s choice of the pigeon, with `cost` after it when given.
    fn choose_words(cost: Option<&str>) -> Vec<String> {
        let mut words = vec![
            "choose",
            "ankh#1",
            "--question",
            "Which way does the clacks message go?",
            "--option",
            "clacks",
            "--option",
            "pigeon",
            "--option",
            "Post Office",
            "--took",
            "pigeon",
            "--why",
            "the towers are down",
        ];
        if let Some(cost) = cost {
            words.extend(["--cost", cost]);
        }
        words.into_iter().map(String::from).collect()
    }

    /// The choice [`choose_words`] reads into.
    pub(crate) fn pigeon(cost: Option<&str>) -> Command {
        Command::Choose {
            issue: IssueId::new("ankh", "1"),
            question: "Which way does the clacks message go?".into(),
            options: vec!["clacks".into(), "pigeon".into(), "Post Office".into()],
            took: "pigeon".into(),
            why: "the towers are down".into(),
            cost: cost.map(String::from),
        }
    }

    #[test]
    fn choose_reads_each_option_and_a_missing_cost_as_none() {
        let r = request(
            &choose_words(Some("medium")),
            Some("author-1".into()),
            no_files,
        )
        .expect("request");
        assert_eq!(r.by, Some(AgentId::new("author-1")));
        assert_eq!(r.command, pigeon(Some("medium")));
        // Without --cost the choice still reads, with no cost: the daemon refuses it.
        let r = request(&choose_words(None), Some("author-1".into()), no_files).expect("request");
        assert_eq!(r.command, pigeon(None));
    }

    #[test]
    fn a_chooses_texts_are_whatever_follows_each_flag() {
        let line = |words: &[&str]| words.iter().map(ToString::to_string).collect::<Vec<_>>();
        let choose = |words: &[&str]| request(&line(words), None, no_files).map(|r| r.command);
        assert_eq!(
            choose(&[
                "choose",
                "ankh#1",
                "--cost",
                "low",
                "--option",
                "--role",
                "--question",
                "--json",
                "--option",
                "--milestone",
                "--why",
                "--reply",
                "--took",
                "--role",
            ]),
            Ok(Command::Choose {
                issue: IssueId::new("ankh", "1"),
                question: "--json".into(),
                options: vec!["--role".into(), "--milestone".into()],
                took: "--role".into(),
                why: "--reply".into(),
                cost: Some("low".into()),
            })
        );
        for (words, refused) in [
            (
                &["choose", "--json", "ankh#1"][..],
                "--json goes with status only",
            ),
            (&["choose"][..], "missing ISSUE"),
            (&["choose", "ankh"][..], "ankh is not an issue"),
            (&["choose", "ankh#1", "--took"][..], "--took needs its text"),
            (
                &["choose", "ankh#1", "pigeon"][..],
                "pigeon is none of them",
            ),
            (
                &["choose", "ankh#1", "--cost", "low", "--cost", "high"][..],
                "--cost is given twice",
            ),
        ] {
            let why = choose(words).expect_err("refused");
            assert!(why.contains(refused), "{words:?}: {why}");
        }
    }

    #[test]
    fn file_reads_project_title_body_and_milestone_and_investigated_its_group() {
        let line = |words: &[&str]| words.iter().map(ToString::to_string).collect::<Vec<_>>();
        let r = request(
            &line(&[
                "file",
                "ankh",
                "Wind the clock",
                "-",
                "--milestone",
                "First",
            ]),
            Some("investigator-1".into()),
            |p| {
                assert_eq!(p, Path::new("-"));
                Ok("Wind it nightly.\nDepends on #12\n".into())
            },
        )
        .expect("request");
        assert_eq!(r.by, Some(AgentId::new("investigator-1")));
        assert_eq!(
            r.command,
            Command::File {
                project: ProjectId::new("ankh"),
                title: "Wind the clock".into(),
                body: "Wind it nightly.\nDepends on #12\n".into(),
                milestone: Some("First".into()),
            }
        );
        let inline = request(
            &line(&["file", "ankh", "Wind the clock", "Wind", "it"]),
            None,
            no_files,
        )
        .expect("request");
        assert!(matches!(
            inline.command,
            Command::File { body, milestone: None, .. } if body == "Wind it"
        ));
        assert!(
            request(&args("file ankh title"), None, no_files).is_err(),
            "no body"
        );
        assert_eq!(
            request(&args("investigated watch"), None, no_files).map(|r| r.command),
            Ok(Command::Investigated {
                group: GroupId::new("watch")
            })
        );
    }

    #[test]
    fn json_asks_status_for_the_view_and_is_refused_on_any_other_command() {
        let read = |line: &str| request(&args(line), None, no_files).map(|r| r.command);
        assert_eq!(read("status"), Ok(Command::Status));
        assert_eq!(read("status --json"), Ok(Command::View));
        for line in [
            "issue ankh#1 --json",
            "screen author-1 --json",
            "ready ankh#1 --json",
            "review ankh#1 approve --json",
            "retry ankh#1 --json",
            "settings radar --json",
            "unset radar model --json",
            "set radar --json model opus",
            "file --json ankh title body",
            "investigated watch --json",
            "--json status",
        ] {
            assert!(
                read(line).is_err_and(|e| e.contains("temet-work status [--json]")),
                "{line}"
            );
        }
        // In free text it is a word like any other.
        assert!(matches!(
            read("set radar verify gh run list --json status"),
            Ok(Command::Set { value, .. }) if value == "gh run list --json status"
        ));
        assert!(matches!(
            read("rule ankh#1 fix print --json too"),
            Ok(Command::Rule { reason, .. }) if reason == "print --json too"
        ));
        assert!(matches!(
            read("review ankh#1 block --json"),
            Ok(Command::Review { verdict: Verdict::Block { points }, .. }) if points == ["--json"]
        ));
    }

    #[test]
    fn reads_a_plan_from_yaml() {
        let yaml = "checks:\n  - done_when: the clock strikes thirteen\n    proven_by: listening at midnight\n";
        let r = request(
            &args("plan ankh#1 plan.yaml"),
            Some("refiner-1".into()),
            |_| Ok(yaml.to_string()),
        )
        .expect("request");
        assert_eq!(r.by, Some(AgentId::new("refiner-1")));
        let Command::Plan { issue, plan } = r.command else {
            panic!("not a plan");
        };
        assert_eq!(issue, IssueId::new("ankh", "1"));
        assert_eq!(plan.checks[0].proven_by, "listening at midnight");
    }

    #[test]
    fn reads_review_points_one_argument_each() {
        let mut a = args("review ankh#1 block");
        a.push("no test for a stopped clock".into());
        a.push("the bell is too loud".into());
        let r = request(&a, None, no_files).expect("request");
        assert_eq!(
            r.command,
            Command::Review {
                issue: IssueId::new("ankh", "1"),
                verdict: Verdict::Block {
                    points: vec![
                        "no test for a stopped clock".into(),
                        "the bell is too loud".into()
                    ]
                },
                failed: Vec::new(),
                open: Vec::new(),
            }
        );
    }

    #[test]
    fn review_reads_each_failed_check_and_open_point_wherever_it_is() {
        let words = |words: &[&str]| words.iter().map(ToString::to_string).collect::<Vec<_>>();
        let review = |line: &[&str]| request(&words(line), None, no_files).map(|r| r.command);
        assert_eq!(
            review(&[
                "review",
                "watch#4",
                "--open",
                "2",
                "block",
                "the clapper is loose",
                "--failed",
                "2",
                "--open",
                "1",
            ]),
            Ok(Command::Review {
                issue: IssueId::new("watch", "4"),
                verdict: Verdict::Block {
                    points: vec!["the clapper is loose".into()]
                },
                failed: vec![2],
                open: vec![2, 1],
            })
        );
        // On an approve too: the daemon says why it refuses them.
        assert!(matches!(
            review(&["review", "watch#4", "approve", "--failed", "1"]),
            Ok(Command::Review { verdict: Verdict::Approve, failed, .. }) if failed == [1]
        ));
        assert!(
            review(&["review", "watch#4", "block", "x", "--failed", "two"])
                .is_err_and(|e| e.contains("--failed takes the number of a check")),
        );
        assert!(
            review(&["review", "watch#4", "block", "--open"])
                .is_err_and(|e| e.contains("--open needs the number of a point")),
        );
    }

    #[test]
    fn reads_set_unset_and_settings_with_a_role_anywhere() {
        let set = request(
            &args("set radar --role reviewer runtime codex"),
            None,
            no_files,
        )
        .expect("request");
        assert_eq!(
            set.command,
            Command::Set {
                target: Target::Id("radar".into()),
                role: Some(Role::Reviewer),
                key: "runtime".into(),
                value: "codex".into()
            }
        );
        let unset = request(&args("unset radar#44 model"), None, no_files).expect("request");
        assert_eq!(
            unset.command,
            Command::Unset {
                target: Target::Issue(IssueId::new("radar", "44")),
                role: None,
                key: "model".into()
            }
        );
        let all = request(&args("settings global --role author"), None, no_files).expect("request");
        assert_eq!(
            all.command,
            Command::Settings {
                target: Target::Global,
                role: Some(Role::Author)
            }
        );
        let deploy =
            request(&args("set radar deploy make deploy"), None, no_files).expect("request");
        assert!(matches!(deploy.command, Command::Set { value, .. } if value == "make deploy"));
        assert!(request(&args("set radar model"), None, no_files).is_err());
        assert!(
            request(&args("set radar --role janitor model x"), None, no_files)
                .is_err_and(|e| e.contains("no role janitor"))
        );
    }

    #[test]
    fn reads_overturn_and_proposal() {
        let r =
            request(&args("overturn 7 1 it was easier to undo"), None, no_files).expect("request");
        assert_eq!(
            r.command,
            Command::Overturn {
                decision: DecisionId(7),
                option: 1,
                reason: "it was easier to undo".into()
            }
        );
        let keep = request(&args("proposal 2 keep"), None, no_files).expect("request");
        assert_eq!(
            keep.command,
            Command::Proposal {
                rule: RuleId(2),
                keep: true
            }
        );
        let drop = request(&args("proposal 2 drop"), None, no_files).expect("request");
        assert!(matches!(
            drop.command,
            Command::Proposal { keep: false, .. }
        ));
        assert!(request(&args("proposal 2 maybe"), None, no_files).is_err());
        assert!(request(&args("overturn x 1 why"), None, no_files).is_err());
        assert!(request(&args("overturn 7"), None, no_files).is_err());
    }

    #[test]
    fn refuses_a_bad_issue_or_command_with_how_to_call_it() {
        assert!(
            request(&args("ready radar"), None, no_files).is_err_and(|e| e.contains("project#key"))
        );
        assert!(
            request(&args("launch ankh#1"), None, no_files)
                .is_err_and(|e| e.contains("temet-work status"))
        );
        assert!(request(&args("plan ankh#1 x.yaml"), None, no_files).is_err());
    }
}
