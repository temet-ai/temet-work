//! `temet-work mcp`: the daemon's commands as MCP tools over standard input and
//! output, so an agent uses temet-work the way a person does: it reads the same
//! views and runs the same commands, through the same socket as the command line.
//!
//! The server has two levels. At `read`, the default, it offers the views:
//! `status`, `issue` and `screen`. At `act` it also offers the commands that move
//! work: `plan`, `ready`, `review` and `rule`. A tool above the server's level is
//! not listed, and calling it is refused.

use serde_json::{Value, json};
use temet_work_core::ids::{AgentId, IssueId};
use temet_work_core::model::{Check, Plan};
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
            name: "plan",
            description: "A refiner's plan for an issue it is refining: one entry per check, each an outcome a person can observe and what shows it.",
            acts: true,
            schema: json!({"type": "object", "properties": {
                "issue": issue_field(),
                "checks": {"type": "array", "minItems": 1, "items": {"type": "object", "properties": {
                    "done_when": {"type": "string"}, "proven_by": {"type": "string"}}, "required": ["done_when", "proven_by"]}}
            }, "required": ["issue", "checks"]}),
        },
        Tool {
            name: "ready",
            description: "The author's pull request is open and every check of the plan holds. The pull request's number is needed the first time.",
            acts: true,
            schema: json!({"type": "object", "properties": {"issue": issue_field(), "pull": {"type": "string"}}, "required": ["issue"]}),
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
            name: "rule",
            description: "The arbiter's ruling once review rounds ran out: approve to merge, or not, with the reason.",
            acts: true,
            schema: json!({"type": "object", "properties": {
                "issue": issue_field(), "approve": {"type": "boolean"}, "reason": {"type": "string"}
            }, "required": ["issue", "approve", "reason"]}),
        },
    ]
}

fn text(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| format!("{key} is missing"))
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
        "rule" => Command::Rule {
            issue: issue(args)?,
            approve: args
                .get("approve")
                .and_then(Value::as_bool)
                .ok_or("approve is missing")?,
            reason: text(args, "reason")?,
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
        assert_eq!(names(Level::Read), ["status", "issue", "screen"]);
        assert_eq!(names(Level::Act).len(), 7);
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
