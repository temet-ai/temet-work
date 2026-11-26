//! The command line's arguments, read into a request for the daemon.

use std::path::Path;

use temet_work_core::ids::{AgentId, IssueId};
use temet_work_core::model::Plan;
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
temet-work mcp [--allow read|act]           the same views and commands as MCP tools, for agents;
                                            read (the default) offers status, issue and screen
temet-work parts                            the parts in this build

ISSUE is project#key, such as radar#44. Agents' commands are recorded as theirs
through TEMET_WORK_AGENT, which the daemon sets when it starts them.";

/// A plan from its YAML text.
pub fn plan(yaml: &str) -> Result<Plan, String> {
    serde_saphyr::from_str(yaml).map_err(|e| format!("the plan is not readable: {e}"))
}

/// Reads the arguments after the program name into a request. `by` is the
/// agent sending it; `read` reads a file.
pub fn request(
    args: &[String],
    by: Option<String>,
    read: impl Fn(&Path) -> Result<String, String>,
) -> Result<Request, String> {
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
        "rule" => Command::Rule {
            issue: issue()?,
            approve: match arg(2, "approve or fix")? {
                "approve" => true,
                "fix" => false,
                other => return Err(format!("rule takes approve or fix, not {other}")),
            },
            reason: args.get(3..).unwrap_or_default().join(" "),
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
