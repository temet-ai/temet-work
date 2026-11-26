//! What a person or an agent asks the daemon to do, by name. The app, the
//! command line and the MCP server all send these, and agents run the same
//! commands a person does. The loop checks each one against the issue's stage
//! before it becomes an event, so a command cannot move an issue past a step it
//! is not at.

use serde::{Deserialize, Serialize};

use crate::event::{Event, Verdict};
use crate::ids::{AgentId, DecisionId, IssueId};
use crate::model::{Decision, DecisionKind, Plan, Stage};
use crate::text::stage;
use crate::world::World;

/// One command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    /// The state as text.
    Status,
    /// One issue in full, as text.
    Issue {
        /// The issue.
        issue: IssueId,
    },
    /// What an agent's terminal shows now.
    Screen {
        /// The agent.
        agent: AgentId,
    },
    /// A refiner's plan for an issue it is refining.
    Plan {
        /// The issue.
        issue: IssueId,
        /// The plan.
        plan: Plan,
    },
    /// The author's pull request is open and the plan's checks hold.
    Ready {
        /// The issue.
        issue: IssueId,
        /// The pull request's number; needed the first time.
        #[serde(default)]
        pull: Option<String>,
    },
    /// A reviewer's verdict on the round under review.
    Review {
        /// The issue.
        issue: IssueId,
        /// The verdict.
        verdict: Verdict,
    },
    /// The arbiter's ruling once review rounds ran out.
    Rule {
        /// The issue.
        issue: IssueId,
        /// Whether it merges as it is.
        approve: bool,
        /// Why, or what must change.
        reason: String,
    },
}

impl Command {
    /// The issue the command is about; `None` for `Status`.
    pub fn issue(&self) -> Option<&IssueId> {
        match self {
            Self::Status | Self::Screen { .. } => None,
            Self::Issue { issue } => Some(issue),
            Self::Plan { issue, .. }
            | Self::Ready { issue, .. }
            | Self::Review { issue, .. }
            | Self::Rule { issue, .. } => Some(issue),
        }
    }
}

/// A command and who sent it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    /// The agent that sent it; `None` for a person.
    pub by: Option<AgentId>,
    /// The command.
    pub command: Command,
}

/// Who a decision is recorded as made by when a person makes it.
pub const PERSON: &str = "person";

impl World {
    /// The event a command becomes, or why the loop refuses it. `Status`
    /// becomes no event.
    pub fn check(&self, by: Option<&AgentId>, command: &Command) -> Result<Option<Event>, String> {
        let at = |issue: &IssueId| {
            self.issues
                .get(issue)
                .map(|i| i.stage)
                .ok_or_else(|| format!("there is no issue {issue}"))
        };
        let refuse = |issue: &IssueId, now: Stage, wanted: &str| {
            Err(format!("{issue} is {}, not {wanted}", stage(now)))
        };
        let event = match command {
            Command::Status | Command::Screen { .. } => return Ok(None),
            Command::Issue { issue } => {
                at(issue)?;
                return Ok(None);
            }
            Command::Plan { issue, plan } => {
                let now = at(issue)?;
                if now != Stage::Refining {
                    return refuse(issue, now, "refining");
                }
                if plan.checks.is_empty() {
                    return Err("a plan needs at least one check".into());
                }
                if let Some(c) = plan
                    .checks
                    .iter()
                    .find(|c| c.done_when.trim().is_empty() || c.proven_by.trim().is_empty())
                {
                    return Err(format!(
                        "every check needs done_when and proven_by; one has done_when {:?} and proven_by {:?}",
                        c.done_when, c.proven_by
                    ));
                }
                Event::PlanWritten {
                    issue: issue.clone(),
                    plan: plan.clone(),
                }
            }
            Command::Ready { issue, pull } => match at(issue)? {
                Stage::Working | Stage::Fixing { .. } => {
                    let known = self.issues.get(issue).and_then(|i| i.pull.as_ref());
                    if pull.is_none() && known.is_none() {
                        return Err(format!(
                            "say which pull request: temet-work ready {issue} <number>"
                        ));
                    }
                    Event::PullRequestReady {
                        issue: issue.clone(),
                        pull: pull.clone(),
                    }
                }
                now => return refuse(issue, now, "being worked on"),
            },
            Command::Review { issue, verdict } => {
                let Stage::InReview { round } = at(issue)? else {
                    return refuse(issue, at(issue)?, "in review");
                };
                // A reviewer that has not heard of this round started its review on
                // the one before: its verdict is about code that has since changed.
                let heard = self.issues.get(issue).is_some_and(|i| i.told);
                if by.is_some() && !heard {
                    return Err(format!(
                        "review round {round} of {issue} started after your review began; \
                         review the whole change again once its message reaches you"
                    ));
                }
                if matches!(verdict, Verdict::Block { points } if points.iter().all(|p| p.trim().is_empty()))
                {
                    return Err("a block needs at least one point to fix".into());
                }
                Event::ReviewFinished {
                    issue: issue.clone(),
                    round,
                    verdict: verdict.clone(),
                }
            }
            Command::Rule {
                issue,
                approve,
                reason,
            } => {
                let Stage::Arbitration { round } = at(issue)? else {
                    return refuse(issue, at(issue)?, "with the arbiter");
                };
                if reason.trim().is_empty() {
                    return Err("a ruling needs a reason".into());
                }
                let id = self.decisions.keys().next_back().map_or(1, |d| d.0 + 1);
                Event::DecisionMade {
                    decision: Decision {
                        id: DecisionId(id),
                        issue: issue.clone(),
                        by: by.cloned().unwrap_or_else(|| AgentId::new(PERSON)),
                        kind: DecisionKind::Ruling { approve: *approve },
                        question: format!("Does {issue} merge after {round} review rounds?"),
                        options: vec!["merge".into(), "fix first".into()],
                        chosen: usize::from(!approve),
                        reason: reason.clone(),
                        overturned_to: None,
                    },
                }
            }
        };
        Ok(Some(event))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{FolderId, GroupId, MachineId, ProjectId};
    use crate::model::Check;

    fn issue() -> IssueId {
        IssueId::new("ankh", "1")
    }

    fn plan() -> Plan {
        Plan {
            repos: Vec::new(),
            checks: vec![Check {
                done_when: "the clock strikes thirteen".into(),
                proven_by: "listening at midnight".into(),
            }],
        }
    }

    /// A world where ankh#1 is being refined.
    fn refining() -> World {
        let (w, _) = World::replay(&[
            Event::FolderAdded {
                folder: FolderId::new("f1"),
                project: ProjectId::new("ankh"),
                machine: MachineId::new("morpork"),
                path: "/srv/ankh/f1".into(),
            },
            Event::GroupCreated {
                group: GroupId::new("watch"),
                name: "Night watch".into(),
                projects: vec![ProjectId::new("ankh")],
                areas: Vec::new(),
                milestones: vec![],
                refining_at_once: 1,
                model: None,
            },
            Event::IssueSynced {
                issue: issue(),
                title: "Mend the clock".into(),
                milestone: None,
                depends_on: vec![],
                order: 1,
                closed: false,
            },
        ]);
        w
    }

    fn run(w: &mut World, command: Command) -> Result<(), String> {
        let event = w.check(None, &command)?.expect("an event");
        w.apply(&event);
        Ok(())
    }

    #[test]
    fn commands_take_an_issue_from_refining_to_merging() {
        let mut w = refining();
        run(
            &mut w,
            Command::Plan {
                issue: issue(),
                plan: plan(),
            },
        )
        .expect("plan");
        assert_eq!(
            w.issues[&issue()].stage,
            Stage::Working,
            "the folder was free"
        );
        run(
            &mut w,
            Command::Ready {
                issue: issue(),
                pull: Some("57".into()),
            },
        )
        .expect("ready");
        let approve = Command::Review {
            issue: issue(),
            verdict: Verdict::Approve,
        };
        run(&mut w, approve).expect("review");
        w.apply(&Event::ChecksFinished {
            issue: issue(),
            passed: true,
            failing: vec![],
        });
        assert_eq!(w.issues[&issue()].stage, Stage::Merging);
    }

    #[test]
    fn a_command_for_another_step_is_refused_with_the_stage() {
        let w = refining();
        assert_eq!(
            w.check(
                None,
                &Command::Ready {
                    issue: issue(),
                    pull: Some("57".into())
                }
            ),
            Err("ankh#1 is refining, not being worked on".into())
        );
        assert!(
            w.check(
                None,
                &Command::Ready {
                    issue: IssueId::new("ankh", "9"),
                    pull: None
                }
            )
            .is_err_and(|e| e.contains("no issue ankh#9"))
        );
        let empty = Command::Plan {
            issue: issue(),
            plan: Plan::default(),
        };
        assert!(w.check(None, &empty).is_err());
    }

    #[test]
    fn ready_needs_the_pull_request_the_first_time_only() {
        let mut w = refining();
        run(
            &mut w,
            Command::Plan {
                issue: issue(),
                plan: plan(),
            },
        )
        .expect("plan");
        let bare = Command::Ready {
            issue: issue(),
            pull: None,
        };
        assert!(
            w.check(None, &bare)
                .is_err_and(|e| e.contains("temet-work ready ankh#1 <number>"))
        );
        run(
            &mut w,
            Command::Ready {
                issue: issue(),
                pull: Some("57".into()),
            },
        )
        .expect("ready");
        assert_eq!(w.issues[&issue()].pull.as_deref(), Some("57"));
        let block = Command::Review {
            issue: issue(),
            verdict: Verdict::Block {
                points: vec!["no test for a stopped clock".into()],
            },
        };
        run(&mut w, block).expect("block");
        run(&mut w, bare).expect("the pull request is on record");
    }

    #[test]
    fn a_reviewer_that_has_not_heard_the_new_round_is_refused() {
        let mut w = refining();
        run(
            &mut w,
            Command::Plan {
                issue: issue(),
                plan: plan(),
            },
        )
        .expect("plan");
        run(
            &mut w,
            Command::Ready {
                issue: issue(),
                pull: Some("57".into()),
            },
        )
        .expect("ready");
        // Round 1 is up and no reviewer has heard of it yet.
        let approve = Command::Review {
            issue: issue(),
            verdict: Verdict::Approve,
        };
        let reviewer = AgentId::new("reviewer-1");
        assert!(
            w.check(Some(&reviewer), &approve)
                .is_err_and(|e| e.contains("round 1 of ankh#1 started after your review began"))
        );
        assert!(w.check(None, &approve).is_ok(), "a person may still review");
        w.apply(&Event::Told { issue: issue() });
        assert!(w.check(Some(&reviewer), &approve).is_ok());
    }

    #[test]
    fn an_arbiter_ruling_is_a_recorded_decision() {
        let mut w = refining();
        run(
            &mut w,
            Command::Plan {
                issue: issue(),
                plan: plan(),
            },
        )
        .expect("plan");
        for _ in 0..crate::world::REVIEW_ROUNDS {
            run(
                &mut w,
                Command::Ready {
                    issue: issue(),
                    pull: Some("57".into()),
                },
            )
            .expect("ready");
            let block = Command::Review {
                issue: issue(),
                verdict: Verdict::Block {
                    points: vec!["no test for a stopped clock".into()],
                },
            };
            run(&mut w, block).expect("block");
        }
        let rule = Command::Rule {
            issue: issue(),
            approve: false,
            reason: "a stopped clock is the bug this issue is about".into(),
        };
        let by = AgentId::new("arbiter-1");
        let event = w.check(Some(&by), &rule).expect("check").expect("event");
        w.apply(&event);
        let d = &w.decisions[&DecisionId(1)];
        assert_eq!((d.by.0.as_str(), d.chosen), ("arbiter-1", 1));
        assert!(matches!(w.issues[&issue()].stage, Stage::Fixing { .. }));
    }
}
