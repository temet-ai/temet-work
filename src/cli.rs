//! The command line's arguments, read into a request for the daemon.

use std::path::Path;

use temet_work_core::ids::{AgentId, DecisionId, IssueId, RuleId};
use temet_work_core::model::{Plan, Role};
use temet_work_core::settings::Target;
use temet_work_core::{Command, Request, Verdict};

pub use temet_work_core::launch::AGENT_VAR;

/// How to call `temet-work`.
pub const USAGE: &str = "\
temet-work status
temet-work issue ISSUE                      one issue in full
temet-work screen AGENT                     what an agent's terminal shows now
temet-work plan ISSUE FILE                  a refiner's plan, as YAML; FILE - reads it from standard input
temet-work ready ISSUE PULL                 the pull request is open and the checks hold;
                                            PULL is its number, needed the first time
temet-work review ISSUE approve
temet-work review ISSUE block POINT...      what to fix, one argument each
temet-work rule ISSUE approve REASON
temet-work rule ISSUE fix REASON
temet-work overturn DECISION OPTION REASON
                                            a person's: choose another option of a decision, as
                                            `status` numbers them; REASON is required
temet-work proposal RULE keep|drop          a person's: keep or drop a rule proposed from an overturn
temet-work set TARGET [--role ROLE] KEY VALUE
                                            give a setting a value at one level
temet-work unset TARGET [--role ROLE] KEY   remove it, so the next level answers
temet-work settings TARGET [--role ROLE]    every setting in effect there, and the level that set it
temet-work mcp [--allow read|act]           the same views and commands as MCP tools, for agents;
                                            read (the default) offers status, issue and screen
temet-work parts                            the parts in this build

ISSUE is project#key, such as radar#44. TARGET is global, an area or project id, a
milestone as project:Milestone name, or an issue. ROLE is refiner, author, reviewer,
arbiter or investigator. Agents' commands are recorded as theirs
through TEMET_WORK_AGENT, which the daemon sets when it starts them.";

/// A plan from its YAML text.
pub fn plan(yaml: &str) -> Result<Plan, String> {
    serde_saphyr::from_str(yaml).map_err(|e| format!("the plan is not readable: {e}"))
}

/// Takes `--role ROLE` out of `args`, wherever it is.
fn take_role(args: &[String]) -> Result<(Vec<String>, Option<Role>), String> {
    let mut rest = Vec::new();
    let mut role = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--role" {
            let name = it.next().ok_or("--role needs a role")?;
            role = Some(Role::parse(name)?);
        } else {
            rest.push(a.clone());
        }
    }
    Ok((rest, role))
}

/// Reads the arguments after the program name into a request. `by` is the
/// agent sending it; `read` reads a file.
pub fn request(
    args: &[String],
    by: Option<String>,
    read: impl Fn(&Path) -> Result<String, String>,
) -> Result<Request, String> {
    let (args, role) = take_role(args)?;
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
    let command = match arg(0, "a command")? {
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
        "ready" => Command::Ready {
            issue: issue()?,
            pull: args.get(2).cloned(),
        },
        "review" => Command::Review {
            issue: issue()?,
            verdict: match arg(2, "approve or block")? {
                "approve" => Verdict::Approve,
                "block" => Verdict::Block {
                    points: args.get(3..).unwrap_or_default().to_vec(),
                },
                other => return Err(format!("review takes approve or block, not {other}")),
            },
        },
        "retry" => Command::Retry { issue: issue()? },
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
        other => return Err(format!("no command {other}\n\n{USAGE}")),
    };
    Ok(Request {
        by: by.filter(|b| !b.is_empty()).map(AgentId::new),
        command,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split(' ').map(String::from).collect()
    }

    fn no_files(_: &Path) -> Result<String, String> {
        Err("no files in this test".into())
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
                }
            }
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
