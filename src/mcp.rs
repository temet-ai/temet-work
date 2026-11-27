//! `temet-work mcp`: the daemon's commands as MCP tools over standard input and
//! output, so an agent uses temet-work the way a person does: it reads the same
//! views and runs the same commands, through the same socket as the command line.
//!
//! The server has two levels. At `read`, the default, it offers the views:
//! `status`, `issue`, `screen` and `settings`. At `act` it also offers the commands
//! that move work and change settings: `plan`, `ready`, `review`, `retry`, `rule`, `overturn`, `proposal`, `set` and
//! `unset`. A tool above the server's level is
//! not listed, and calling it is refused.

use serde_json::{Value, json};
use temet_work_core::ids::{AgentId, DecisionId, IssueId, ProjectId, RuleId};
use temet_work_core::model::{Check, Plan, Role};
use temet_work_core::settings::{Target, Value as Setting};
use temet_work_core::{Command, Request, Verdict};

use crate::socket::Reply;

/// What the server lets its client do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// Read the views.
    Read,
    /// Read the views and run the commands that move work.
    Act,
}

/// The protocol version answered when the client names none.
pub const PROTOCOL: &str = "2025-06-18";

struct Tool {
    name: &'static str,
    description: &'static str,
    acts: bool,
    schema: Value,
}

fn issue_field() -> Value {
    json!({"type": "string", "description": "The issue as project#key, such as radar#44."})
}

fn target_field() -> Value {
    json!({"type": "string", "description": "global, an area or project id, a milestone as project:Milestone name, or an issue as project#key."})
}

fn role_field() -> Value {
    json!({"type": "string", "enum": ["refiner", "author", "reviewer", "arbiter", "investigator"], "description": "The role the setting is for; every role when left out."})
}

fn role(args: &Value) -> Result<Option<Role>, String> {
    args.get("role")
        .and_then(Value::as_str)
        .map(Role::parse)
        .transpose()
}

fn target(args: &Value) -> Result<Target, String> {
    text(args, "target").map(|t| Target::parse(&t))
}

/// A setting's value from JSON, for a plan's `settings`.
fn setting(value: &Value) -> Result<Setting, String> {
    match value {
        Value::Bool(b) => Ok(Setting::Flag(*b)),
        Value::Number(n) => n
            .as_u64()
            .map(Setting::Number)
            .ok_or_else(|| format!("{n} is not a whole number")),
        Value::String(t) => Ok(Setting::Text(t.clone())),
        other => Err(format!("a setting takes a single value, not {other}")),
    }
}

fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "status",
            description: "The whole state: projects, groups, issues and their stages, agents and what each is doing, decisions, and what is wrong now.",
            acts: false,
            schema: json!({"type": "object", "properties": {}}),
        },
        Tool {
            name: "issue",
            description: "One issue in full: its stage, plan, pull request, what it is to fix, its dependencies and its agents.",
            acts: false,
            schema: json!({"type": "object", "properties": {"issue": issue_field()}, "required": ["issue"]}),
        },
        Tool {
            name: "screen",
            description: "What an agent's terminal shows now.",
            acts: false,
            schema: json!({"type": "object", "properties": {"agent": {"type": "string", "description": "The agent's id, such as author-3."}}, "required": ["agent"]}),
        },
        Tool {
            name: "settings",
            description: "Every setting in effect at a target, each with its value and the level that set it. The target is global, an area or project id, a milestone as project:Milestone name, or an issue.",
            acts: false,
            schema: json!({"type": "object", "properties": {"target": target_field(), "role": role_field()}, "required": ["target"]}),
        },
        Tool {
            name: "plan",
            description: "A refiner's plan for an issue it is refining: one entry per check, each an outcome a person can observe and what shows it.",
            acts: true,
            schema: json!({"type": "object", "properties": {
                "issue": issue_field(),
                "checks": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {
                    "done_when": {"type": "string"}, "proven_by": {"type": "string"}}, "required": ["done_when", "proven_by"]}},
                "repos": {"type": "array", "items": {"type": "string"}, "description": "For an issue of an area: the repositories the change needs, in the order they merge."},
                "settings": {"type": "object", "description": "Task-level settings for this issue, such as {\"model\": \"opus\"} for a hard one. Recorded as set by you; a person can overturn them."}
            }, "required": ["issue", "checks"]}),
        },
        Tool {
            name: "ready",
            description: "The author's pull requests are open and every check of the plan holds. Their numbers are needed the first time: pull for an issue that changes one repository, pulls (repository to number) for each repository of one that changes several.",
            acts: true,
            schema: json!({"type": "object", "properties": {"issue": issue_field(), "pull": {"type": "string"}, "pulls": {"type": "object", "additionalProperties": {"type": "string"}}}, "required": ["issue"]}),
        },
        Tool {
            name: "review",
            description: "A reviewer's verdict on the round under review: approve, or block with each thing to fix.",
            acts: true,
            schema: json!({"type": "object", "properties": {
                "issue": issue_field(),
                "verdict": {"type": "string", "enum": ["approve", "block"]},
                "points": {"type": "array", "items": {"type": "string"}}
            }, "required": ["issue", "verdict"]}),
        },
        Tool {
            name: "retry",
            description: "Runs the deploy or verify of an issue that is deploying or verifying again at once, clearing its failures. A person's command.",
            acts: true,
            schema: json!({"type": "object", "properties": {"issue": issue_field()}, "required": ["issue"]}),
        },
        Tool {
            name: "rule",
            description: "The arbiter's ruling once review rounds ran out: approve to merge, or not, with the reason.",
            acts: true,
            schema: json!({"type": "object", "properties": {
                "issue": issue_field(), "approve": {"type": "boolean"}, "reason": {"type": "string"}
            }, "required": ["issue", "approve", "reason"]}),
        },
        Tool {
            name: "overturn",
            description: "A person's: choose another option of a decision an agent made. The option is its number as status lists it, counted from 1; the reason is required. It sends an approved issue back to its author before it merges, and proposes a rule.",
            acts: true,
            schema: json!({"type": "object", "properties": {
                "decision": {"type": "integer", "description": "The decision's number, as status lists it."},
                "option": {"type": "integer", "description": "The option now preferred, counted from 1."},
                "reason": {"type": "string"}
            }, "required": ["decision", "option", "reason"]}),
        },
        Tool {
            name: "proposal",
            description: "A person's: keep or drop a rule proposed from an overturned decision. A kept rule ends every brief written afterwards.",
            acts: true,
            schema: json!({"type": "object", "properties": {
                "rule": {"type": "integer", "description": "The rule's number, as status lists it under Rules."},
                "action": {"type": "string", "enum": ["keep", "drop"]}
            }, "required": ["rule", "action"]}),
        },
        Tool {
            name: "set",
            description: "Gives a setting a value at one level, for one role or for every role. A refiner may set task-level values of the issue it is refining; a wider level is a person's.",
            acts: true,
            schema: json!({"type": "object", "properties": {
                "target": target_field(), "role": role_field(),
                "key": {"type": "string", "description": "The setting, such as model or review_rounds."},
                "value": {"type": "string", "description": "The value, as typed on the command line."}
            }, "required": ["target", "key", "value"]}),
        },
        Tool {
            name: "unset",
            description: "Removes a setting's value at one level, so the next level answers.",
            acts: true,
            schema: json!({"type": "object", "properties": {
                "target": target_field(), "role": role_field(), "key": {"type": "string"}
            }, "required": ["target", "key"]}),
        },
    ]
}

fn text(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| format!("{key} is missing"))
}

fn number(args: &Value, key: &str) -> Result<u64, String> {
    args.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{key} is missing or not a whole number"))
}

fn issue(args: &Value) -> Result<IssueId, String> {
    let s = text(args, "issue")?;
    IssueId::parse(&s).ok_or_else(|| format!("{s} is not an issue; write project#key"))
}

/// The command a tool call asks for.
fn command(name: &str, args: &Value) -> Result<Command, String> {
    Ok(match name {
        "status" => Command::Status,
        "issue" => Command::Issue {
            issue: issue(args)?,
        },
        "screen" => Command::Screen {
            agent: AgentId::new(text(args, "agent")?),
        },
        "plan" => Command::Plan {
            issue: issue(args)?,
            plan: Plan {
                repos: args
                    .get("repos")
                    .and_then(Value::as_array)
                    .map(|r| {
                        r.iter()
                            .filter_map(Value::as_str)
                            .map(temet_work_core::ids::ProjectId::new)
                            .collect()
                    })
                    .unwrap_or_default(),
                settings: args
                    .get("settings")
                    .and_then(Value::as_object)
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| Ok((k.clone(), setting(v)?)))
                            .collect::<Result<_, String>>()
                    })
                    .transpose()?
                    .unwrap_or_default(),
                checks: args
                    .get("checks")
                    .and_then(Value::as_array)
                    .ok_or("checks is missing")?
                    .iter()
                    .map(|c| {
                        Ok(Check {
                            done_when: text(c, "done_when")?,
                            proven_by: text(c, "proven_by")?,
                        })
                    })
                    .collect::<Result<_, String>>()?,
            },
        },
        "ready" => Command::Ready {
            issue: issue(args)?,
            pull: args.get("pull").and_then(Value::as_str).map(String::from),
            pulls: args
                .get("pulls")
                .and_then(Value::as_object)
                .map(|m| {
                    m.iter()
                        .map(|(repo, n)| {
                            n.as_str()
                                .map(String::from)
                                .or_else(|| n.as_u64().map(|n| n.to_string()))
                                .map(|n| (ProjectId::new(repo.as_str()), n))
                                .ok_or_else(|| format!("pulls.{repo} is not a pull request number"))
                        })
                        .collect::<Result<_, String>>()
                })
                .transpose()?
                .unwrap_or_default(),
        },
        "review" => Command::Review {
            issue: issue(args)?,
            verdict: match text(args, "verdict")?.as_str() {
                "approve" => Verdict::Approve,
                "block" => Verdict::Block {
                    points: args
                        .get("points")
                        .and_then(Value::as_array)
                        .map(|p| {
                            p.iter()
                                .filter_map(Value::as_str)
                                .map(String::from)
                                .collect()
                        })
                        .unwrap_or_default(),
                },
                other => return Err(format!("verdict is approve or block, not {other}")),
            },
        },
        "retry" => Command::Retry {
            issue: issue(args)?,
        },
        "rule" => Command::Rule {
            issue: issue(args)?,
            approve: args
                .get("approve")
                .and_then(Value::as_bool)
                .ok_or("approve is missing")?,
            reason: text(args, "reason")?,
        },
        "overturn" => Command::Overturn {
            decision: DecisionId(number(args, "decision")?),
            option: usize::try_from(number(args, "option")?)
                .map_err(|_| "option is too large".to_string())?,
            reason: text(args, "reason")?,
        },
        "proposal" => Command::Proposal {
            rule: RuleId(number(args, "rule")?),
            keep: match text(args, "action")?.as_str() {
                "keep" => true,
                "drop" => false,
                other => return Err(format!("action is keep or drop, not {other}")),
            },
        },
        "settings" => Command::Settings {
            target: target(args)?,
            role: role(args)?,
        },
        "set" => Command::Set {
            target: target(args)?,
            role: role(args)?,
            key: text(args, "key")?,
            value: text(args, "value")?,
        },
        "unset" => Command::Unset {
            target: target(args)?,
            role: role(args)?,
            key: text(args, "key")?,
        },
        other => return Err(format!("no tool {other}")),
    })
}

fn result(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn tool_text(text: String, is_error: bool) -> Value {
    json!({"content": [{"type": "text", "text": text}], "isError": is_error})
}

/// Answers one message. `by` is the agent the server runs for; `send` passes a
/// request to the daemon. A notification gets no answer.
pub fn handle(
    msg: &Value,
    level: Level,
    by: Option<&str>,
    send: &mut dyn FnMut(&Request) -> Result<Reply, String>,
) -> Option<Value> {
    let id = msg.get("id")?;
    let method = msg
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    Some(match method {
        "initialize" => {
            let version = params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or(PROTOCOL);
            result(
                id,
                json!({
                    "protocolVersion": version,
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "temet-work", "version": env!("CARGO_PKG_VERSION")},
                    "instructions": "temet-work runs issues through refine, work, review, merge, deploy and verify. Read the state with status and issue, look into an agent's terminal with screen, and report your step done with the tool your brief names."
                }),
            )
        }
        "ping" => result(id, json!({})),
        "tools/list" => {
            let listed: Vec<Value> = tools()
                .into_iter()
                .filter(|t| !t.acts || level == Level::Act)
                .map(|t| json!({"name": t.name, "description": t.description, "inputSchema": t.schema}))
                .collect();
            result(id, json!({"tools": listed}))
        }
        "tools/call" => {
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let acts = tools().iter().any(|t| t.name == name && t.acts);
            if acts && level != Level::Act {
                return Some(result(
                    id,
                    tool_text(
                        format!("{name} changes work; this server is read only"),
                        true,
                    ),
                ));
            }
            let reply = command(name, &args).and_then(|command| {
                send(&Request {
                    by: by.filter(|b| !b.is_empty()).map(AgentId::new),
                    command,
                })
            });
            result(
                id,
                match reply {
                    Ok(Reply::Done { text }) => tool_text(text, false),
                    Ok(Reply::Refused { why }) => tool_text(format!("refused: {why}"), true),
                    Err(e) => tool_text(e, true),
                },
            )
        }
        other => {
            json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": format!("no method {other}")}})
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(level: Level, name: &str, args: Value, sent: &mut Vec<Request>) -> Value {
        let msg = json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": name, "arguments": args}});
        let mut send = |r: &Request| {
            sent.push(r.clone());
            Ok(Reply::Done { text: "ok".into() })
        };
        handle(&msg, level, Some("reviewer-1"), &mut send).expect("an answer")
    }

    fn names(level: Level) -> Vec<String> {
        let msg = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"});
        let mut send = |_: &Request| Err("not called".to_string());
        let answer = handle(&msg, level, None, &mut send).expect("an answer");
        answer["result"]["tools"]
            .as_array()
            .expect("tools")
            .iter()
            .map(|t| t["name"].as_str().unwrap_or_default().to_string())
            .collect()
    }

    #[test]
    fn the_read_level_lists_only_the_views() {
        assert_eq!(
            names(Level::Read),
            ["status", "issue", "screen", "settings"]
        );
        assert_eq!(names(Level::Act).len(), 13);
    }

    #[test]
    fn a_command_above_the_level_is_refused_and_not_sent() {
        let mut sent = Vec::new();
        let answer = call(Level::Read, "ready", json!({"issue": "ankh#1"}), &mut sent);
        assert_eq!(answer["result"]["isError"], true);
        assert!(sent.is_empty());
    }

    #[test]
    fn a_review_becomes_the_same_request_the_command_line_sends() {
        let mut sent = Vec::new();
        let answer = call(
            Level::Act,
            "review",
            json!({"issue": "ankh#1", "verdict": "block", "points": ["no test for a stopped clock"]}),
            &mut sent,
        );
        assert_eq!(answer["result"]["isError"], false);
        assert_eq!(
            sent,
            [Request {
                by: Some(AgentId::new("reviewer-1")),
                command: Command::Review {
                    issue: IssueId::new("ankh", "1"),
                    verdict: Verdict::Block {
                        points: vec!["no test for a stopped clock".into()]
                    }
                }
            }]
        );
    }

    #[test]
    fn ready_takes_the_pull_request_of_each_repository() {
        let mut sent = Vec::new();
        call(
            Level::Act,
            "ready",
            json!({"issue": "guild#GLD-1", "pulls": {"ui": "12", "api": 34}}),
            &mut sent,
        );
        assert_eq!(
            sent.first().map(|r| &r.command),
            Some(&Command::Ready {
                issue: IssueId::new("guild", "GLD-1"),
                pull: None,
                pulls: [
                    (ProjectId::new("ui"), "12".to_string()),
                    (ProjectId::new("api"), "34".to_string())
                ]
                .into(),
            })
        );
    }

    #[test]
    fn overturn_and_proposal_become_the_same_requests_the_command_line_sends() {
        let mut sent = Vec::new();
        call(
            Level::Act,
            "overturn",
            json!({"decision": 7, "option": 1, "reason": "too slow"}),
            &mut sent,
        );
        call(
            Level::Act,
            "proposal",
            json!({"rule": 2, "action": "drop"}),
            &mut sent,
        );
        let commands: Vec<_> = sent.iter().map(|r| r.command.clone()).collect();
        assert_eq!(
            commands,
            [
                Command::Overturn {
                    decision: DecisionId(7),
                    option: 1,
                    reason: "too slow".into()
                },
                Command::Proposal {
                    rule: RuleId(2),
                    keep: false
                }
            ]
        );
        assert!(names(Level::Act).contains(&"overturn".to_string()));
        assert!(!names(Level::Read).contains(&"proposal".to_string()));
    }

    #[test]
    fn a_plan_is_structured_not_yaml() {
        let mut sent = Vec::new();
        call(
            Level::Act,
            "plan",
            json!({"issue": "ankh#1", "checks": [{"done_when": "the clock strikes", "proven_by": "listening"}]}),
            &mut sent,
        );
        assert!(matches!(&sent[0].command, Command::Plan { plan, .. } if plan.checks.len() == 1));
    }

    #[test]
    fn set_and_unset_are_offered_only_at_the_act_level() {
        let read = names(Level::Read);
        assert!(read.contains(&"settings".to_string()));
        assert!(!read.contains(&"set".to_string()) && !read.contains(&"unset".to_string()));
        let act = names(Level::Act);
        assert!(act.contains(&"set".to_string()) && act.contains(&"unset".to_string()));
        let mut sent = Vec::new();
        for name in ["set", "unset"] {
            let args = json!({"target": "radar#44", "key": "model", "value": "opus"});
            let answer = call(Level::Read, name, args, &mut sent);
            assert_eq!(answer["result"]["isError"], true, "{name}");
        }
        assert!(sent.is_empty(), "nothing above the level was sent");
    }

    #[test]
    fn settings_set_and_unset_become_the_commands_the_command_line_sends() {
        let mut sent = Vec::new();
        call(
            Level::Read,
            "settings",
            json!({"target": "radar#44", "role": "author"}),
            &mut sent,
        );
        call(
            Level::Act,
            "set",
            json!({"target": "radar:Slack sign-in rework", "role": "reviewer", "key": "runtime", "value": "codex"}),
            &mut sent,
        );
        call(
            Level::Act,
            "unset",
            json!({"target": "global", "key": "model"}),
            &mut sent,
        );
        let by = Some(AgentId::new("reviewer-1"));
        assert_eq!(
            sent,
            [
                Request {
                    by: by.clone(),
                    command: Command::Settings {
                        target: Target::Issue(IssueId::new("radar", "44")),
                        role: Some(Role::Author)
                    }
                },
                Request {
                    by: by.clone(),
                    command: Command::Set {
                        target: Target::Milestone {
                            of: "radar".into(),
                            name: "Slack sign-in rework".into()
                        },
                        role: Some(Role::Reviewer),
                        key: "runtime".into(),
                        value: "codex".into()
                    }
                },
                Request {
                    by,
                    command: Command::Unset {
                        target: Target::Global,
                        role: None,
                        key: "model".into()
                    }
                },
            ]
        );
    }

    #[test]
    fn a_plan_may_carry_task_level_settings() {
        let mut sent = Vec::new();
        call(
            Level::Act,
            "plan",
            json!({"issue": "ankh#1", "checks": [{"done_when": "a", "proven_by": "b"}],
                   "settings": {"model": "opus", "review_rounds": 5, "ci": false}}),
            &mut sent,
        );
        let Command::Plan { plan, .. } = &sent[0].command else {
            panic!("not a plan");
        };
        assert_eq!(plan.settings["model"], Setting::Text("opus".into()));
        assert_eq!(plan.settings["review_rounds"], Setting::Number(5));
        assert_eq!(plan.settings["ci"], Setting::Flag(false));
    }

    #[test]
    fn initialize_answers_and_a_notification_gets_nothing() {
        let mut send = |_: &Request| Err("not called".to_string());
        let init = json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {"protocolVersion": "2025-03-26"}});
        let answer = handle(&init, Level::Read, None, &mut send).expect("an answer");
        assert_eq!(answer["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(answer["result"]["serverInfo"]["name"], "temet-work");
        let note = json!({"jsonrpc": "2.0", "method": "notifications/initialized"});
        assert!(handle(&note, Level::Read, None, &mut send).is_none());
    }
}
