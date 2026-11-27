//! What a person or an agent asks the daemon to do, by name. The app, the
//! command line and the MCP server all send these, and agents run the same
//! commands a person does. The loop checks each one against the issue's stage
//! before it becomes an event, so a command cannot move an issue past a step it
//! is not at.

use serde::{Deserialize, Serialize};

use crate::event::{Event, Verdict};
use crate::ids::{AgentId, DecisionId, IssueId, ProjectId};
use crate::model::{Decision, DecisionKind, Plan, Role, Stage};
use crate::settings::{self, Scope, Target};
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
        /// A bare pull request number, for an issue that changes one repository.
        #[serde(default)]
        pull: Option<String>,
        /// The pull request numbers by repository; needed the first time, and
        /// for an issue that changes several repositories.
        #[serde(default)]
        pulls: std::collections::BTreeMap<crate::ids::ProjectId, String>,
    },
    /// A reviewer's verdict on the round under review.
    Review {
        /// The issue.
        issue: IssueId,
        /// The verdict.
        verdict: Verdict,
    },
    /// Gives a setting a value at one level, for a role or for every role.
    Set {
        /// Where: `global`, an area, a project, a milestone or an issue.
        target: Target,
        /// The role it is for; every role when none.
        #[serde(default)]
        role: Option<Role>,
        /// The setting.
        key: String,
        /// The value, as typed.
        value: String,
    },
    /// Removes a setting's value at one level, so the next level answers.
    Unset {
        /// Where.
        target: Target,
        /// The role it was for; every role when none.
        #[serde(default)]
        role: Option<Role>,
        /// The setting.
        key: String,
    },
    /// Every setting in effect at a target, and the level that set each.
    Settings {
        /// Where.
        target: Target,
        /// The role to read them for.
        #[serde(default)]
        role: Option<Role>,
    },
    /// Runs a deploy or a verify again at once, clearing its failures.
    Retry {
        /// The issue.
        issue: IssueId,
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
            Self::Status
            | Self::Screen { .. }
            | Self::Set { .. }
            | Self::Unset { .. }
            | Self::Settings { .. } => None,
            Self::Issue { issue } => Some(issue),
            Self::Plan { issue, .. }
            | Self::Ready { issue, .. }
            | Self::Review { issue, .. }
            | Self::Retry { issue }
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
    /// The events a command becomes, in the order they are applied: a plan's
    /// settings come before the plan, so they hold when its author starts.
    pub fn events(&self, by: Option<&AgentId>, command: &Command) -> Result<Vec<Event>, String> {
        let Some(event) = self.check(by, command)? else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        if let Event::PlanWritten { issue, plan } = &event {
            let who = by.map_or_else(|| PERSON.to_string(), |a| a.0.clone());
            for (key, value) in &plan.settings {
                out.push(Event::SettingSet {
                    scope: Scope::Task(issue.clone()),
                    role: None,
                    key: key.clone(),
                    value: value.clone(),
                    by: who.clone(),
                });
            }
        }
        out.push(event);
        Ok(out)
    }

    /// Why `by` may not change a setting at `scope`: an agent may only give the
    /// issue it is refining task-level settings in its plan or by command.
    fn may_set(&self, by: Option<&AgentId>, scope: &Scope) -> Result<(), String> {
        let Some(by) = by else {
            return Ok(());
        };
        let Some(agent) = self.agents.get(by) else {
            return Err(format!("{by} is not an agent this daemon started"));
        };
        match scope {
            Scope::Task(issue)
                if agent.role == Role::Refiner && agent.issue.as_ref() == Some(issue) =>
            {
                Ok(())
            }
            Scope::Task(_) => Err(format!(
                "{} may change settings of the issue it is refining only",
                agent.id
            )),
            _ => Err(format!(
                "{} may change task-level settings only; a wider level is a person's to change",
                agent.id
            )),
        }
    }

    /// Why `by` may not run a step command of `role` on `issue`. An agent must
    /// have that role and work on that issue; a person may only rule.
    fn may_step(
        &self,
        by: Option<&AgentId>,
        step: &str,
        role: Role,
        issue: &IssueId,
    ) -> Result<(), String> {
        let owner = role.name();
        let Some(by) = by else {
            return if role == Role::Arbiter {
                Ok(())
            } else {
                Err(format!(
                    "{step} for {issue} is the {owner}'s: a person may not run it; \
                     a person may rule, or set and unset settings"
                ))
            };
        };
        let Some(agent) = self.agents.get(by) else {
            return Err(format!(
                "{step} for {issue} is the {owner}'s: {by} is not an agent this daemon started"
            ));
        };
        if agent.role == role && agent.issue.as_ref() == Some(issue) {
            return Ok(());
        }
        let working = self
            .agents
            .values()
            .find(|a| a.role == role && a.issue.as_ref() == Some(issue))
            .map_or_else(
                || format!("no {owner} is working on it"),
                |a| format!("{} is working on it", a.id),
            );
        Err(format!(
            "{step} for {issue} is the {owner}'s: {working}; you are {}",
            agent.id
        ))
    }

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
                self.may_step(by, "plan", Role::Refiner, issue)?;
                let now = at(issue)?;
                if now != Stage::Refining {
                    return refuse(issue, now, "refining");
                }
                if plan.checks.is_empty() {
                    return Err("a plan needs at least one check".into());
                }
                for (key, value) in &plan.settings {
                    settings::check_value(key, value)
                        .and_then(|()| self.check_choice(key, value))
                        .map_err(|e| format!("the plan's settings: {e}"))?;
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
            Command::Ready { issue, pull, pulls } => {
                self.may_step(by, "ready", Role::Author, issue)?;
                match at(issue)? {
                    Stage::Working | Stage::Fixing { .. } => {
                        let (repos, known) = self.issues.get(issue).map_or_else(
                            || (Vec::new(), Vec::new()),
                            |i| (self.repos_of(i), i.pulls.keys().cloned().collect()),
                        );
                        let list: Vec<String> = repos.iter().map(ToString::to_string).collect();
                        let mut named = pulls.clone();
                        if let Some(n) = pull {
                            let [only] = repos.as_slice() else {
                                return Err(format!(
                                    "{issue} changes several repositories ({}); say which: \
                                     repo=number for each, as in temet-work ready {issue} {}",
                                    list.join(", "),
                                    list.iter()
                                        .map(|r| format!("{r}=<number>"))
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                ));
                            };
                            named.insert(only.clone(), n.clone());
                        }
                        if let Some(r) = named.keys().find(|r| !repos.contains(r)) {
                            return Err(format!(
                                "{r} is not a repository {issue} changes; they are {}",
                                list.join(", ")
                            ));
                        }
                        let lacking: Vec<&ProjectId> = repos
                            .iter()
                            .filter(|r| !named.contains_key(*r) && !known.contains(*r))
                            .collect();
                        if !lacking.is_empty() {
                            return Err(if repos.len() > 1 {
                                format!(
                                    "say which pull request in each repository: temet-work ready {issue} {}",
                                    lacking
                                        .iter()
                                        .map(|r| format!("{r}=<number>"))
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                )
                            } else {
                                format!("say which pull request: temet-work ready {issue} <number>")
                            });
                        }
                        Event::PullRequestReady {
                            issue: issue.clone(),
                            pull: None,
                            pulls: named,
                        }
                    }
                    now => return refuse(issue, now, "being worked on"),
                }
            }
            Command::Review { issue, verdict } => {
                self.may_step(by, "review", Role::Reviewer, issue)?;
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
            Command::Settings { target, .. } => {
                self.scope_of(target)?;
                return Ok(None);
            }
            Command::Set {
                target,
                role,
                key,
                value,
            } => {
                let scope = self.scope_of(target)?;
                self.may_set(by, &scope)?;
                let value = settings::parse_value(key, value)?;
                self.check_choice(key, &value)?;
                Event::SettingSet {
                    scope,
                    role: *role,
                    key: key.clone(),
                    value,
                    by: by.map_or_else(|| PERSON.to_string(), |a| a.0.clone()),
                }
            }
            Command::Unset { target, role, key } => {
                let scope = self.scope_of(target)?;
                self.may_set(by, &scope)?;
                settings::spec(key)?;
                if !self
                    .settings
                    .contains_key(&(scope.clone(), *role, key.clone()))
                {
                    let for_role = role.map_or_else(String::new, |r| format!(" for {}", r.name()));
                    return Err(format!(
                        "{key} is not set at {target}{for_role}; nothing to unset"
                    ));
                }
                Event::SettingUnset {
                    scope,
                    role: *role,
                    key: key.clone(),
                    by: by.map_or_else(|| PERSON.to_string(), |a| a.0.clone()),
                }
            }
            Command::Retry { issue } => {
                if let Some(by) = by {
                    return Err(format!("retry for {issue} is a person's: {by} is an agent"));
                }
                match at(issue)? {
                    Stage::Deploying | Stage::Verifying => Event::Retried {
                        issue: issue.clone(),
                    },
                    now => return refuse(issue, now, "deploying or verifying"),
                }
            }
            Command::Rule {
                issue,
                approve,
                reason,
            } => {
                self.may_step(by, "rule", Role::Arbiter, issue)?;
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
            settings: Default::default(),
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
            agent(Role::Refiner, "ankh", "1"),
            agent(Role::Author, "ankh", "1"),
            agent(Role::Reviewer, "ankh", "1"),
            agent(Role::Arbiter, "ankh", "1"),
            agent(Role::Author, "ankh", "2"),
        ]);
        w
    }

    fn agent(role: Role, project: &str, key: &str) -> Event {
        let id = if key == "1" {
            format!("{}-1", role.name())
        } else {
            format!("{}-{key}", role.name())
        };
        Event::AgentStarted {
            agent: AgentId::new(&id),
            group: GroupId::new("watch"),
            role,
            issue: Some(IssueId::new(project, key)),
            machine: MachineId::new("morpork"),
            folder: None,
        }
    }

    /// The agent whose step the command is, on ankh#1.
    fn owner(command: &Command) -> Option<AgentId> {
        let role = match command {
            Command::Plan { .. } => Role::Refiner,
            Command::Ready { .. } => Role::Author,
            Command::Review { .. } => Role::Reviewer,
            Command::Rule { .. } => Role::Arbiter,
            _ => return None,
        };
        Some(AgentId::new(format!("{}-1", role.name())))
    }

    fn run(w: &mut World, command: Command) -> Result<(), String> {
        if matches!(command, Command::Review { .. }) {
            // The reviewer has heard of the round.
            w.apply(&Event::Told { issue: issue() });
        }
        let event = w
            .check(owner(&command).as_ref(), &command)?
            .expect("an event");
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
                pulls: Default::default(),
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
        let author = AgentId::new("author-1");
        assert_eq!(
            w.check(
                Some(&author),
                &Command::Ready {
                    issue: issue(),
                    pull: Some("57".into()),
                    pulls: Default::default()
                }
            ),
            Err("ankh#1 is refining, not being worked on".into())
        );
        assert!(
            w.check(
                Some(&author),
                &Command::Ready {
                    issue: IssueId::new("ankh", "9"),
                    pull: None,
                    pulls: Default::default(),
                }
            )
            .is_err()
        );
        let empty = Command::Plan {
            issue: issue(),
            plan: Plan::default(),
        };
        let refiner = AgentId::new("refiner-1");
        assert!(w.check(Some(&refiner), &empty).is_err());
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
            pulls: Default::default(),
        };
        assert!(
            w.check(owner(&bare).as_ref(), &bare)
                .is_err_and(|e| e.contains("temet-work ready ankh#1 <number>"))
        );
        run(
            &mut w,
            Command::Ready {
                issue: issue(),
                pull: Some("57".into()),
                pulls: Default::default(),
            },
        )
        .expect("ready");
        assert_eq!(
            w.issues[&issue()]
                .pulls
                .get(&ProjectId::new("ankh"))
                .map(String::as_str),
            Some("57")
        );
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
                pulls: Default::default(),
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
        assert!(
            w.check(None, &approve)
                .is_err_and(|e| e.contains("the reviewer's")),
            "a person may not review"
        );
        w.apply(&Event::Told { issue: issue() });
        assert!(w.check(Some(&reviewer), &approve).is_ok());
    }

    #[test]
    fn a_step_command_is_refused_unless_it_is_the_senders_step_on_that_issue() {
        let mut w = refining();
        run(
            &mut w,
            Command::Plan {
                issue: issue(),
                plan: plan(),
            },
        )
        .expect("plan");
        let commands = [
            (
                "plan",
                Role::Refiner,
                Command::Plan {
                    issue: issue(),
                    plan: plan(),
                },
            ),
            (
                "ready",
                Role::Author,
                Command::Ready {
                    issue: issue(),
                    pull: Some("57".into()),
                    pulls: Default::default(),
                },
            ),
            (
                "review",
                Role::Reviewer,
                Command::Review {
                    issue: issue(),
                    verdict: Verdict::Approve,
                },
            ),
            (
                "rule",
                Role::Arbiter,
                Command::Rule {
                    issue: issue(),
                    approve: true,
                    reason: "fine".into(),
                },
            ),
        ];
        for (step, role, command) in &commands {
            let owner = format!("{}-1", role.name());
            let other = if *role == Role::Author {
                "reviewer-1"
            } else {
                "author-1"
            };
            // Another role: refused, naming the step, the owner and the sender.
            let why = w
                .check(Some(&AgentId::new(other)), command)
                .expect_err("wrong role");
            assert!(
                why.contains(&format!("{step} for ankh#1 is the {}'s", role.name())),
                "{why}"
            );
            assert!(why.contains(&format!("{owner} is working on it")), "{why}");
            assert!(why.contains(&format!("you are {other}")), "{why}");
            // The right role on another issue.
            if *role == Role::Author {
                let why = w
                    .check(Some(&AgentId::new("author-2")), command)
                    .expect_err("other issue");
                assert!(why.contains("author-1 is working on it"), "{why}");
                assert!(why.contains("you are author-2"), "{why}");
            }
            // An id the daemon did not start.
            let why = w
                .check(Some(&AgentId::new("ghost-9")), command)
                .expect_err("unknown");
            assert!(why.contains("not an agent this daemon started"), "{why}");
        }
        assert_eq!(
            w.check(
                Some(&AgentId::new("reviewer-1")),
                &Command::Ready {
                    issue: issue(),
                    pull: Some("57".into()),
                    pulls: Default::default()
                }
            ),
            Err(
                "ready for ankh#1 is the author's: author-1 is working on it; you are reviewer-1"
                    .into()
            ),
            "the agent is checked before the stage"
        );
        for (_, role, command) in &commands {
            let by = AgentId::new(format!("{}-1", role.name()));
            let verdict = w.check(Some(&by), command);
            assert!(
                verdict.as_ref().is_ok() || verdict.as_ref().is_err_and(|e| !e.contains("is the")),
                "the owner passes the agent check: {verdict:?}"
            );
        }
    }

    #[test]
    fn a_person_may_rule_but_not_plan_ready_or_review() {
        let mut w = refining();
        for command in [
            Command::Plan {
                issue: issue(),
                plan: plan(),
            },
            Command::Ready {
                issue: issue(),
                pull: Some("57".into()),
                pulls: Default::default(),
            },
            Command::Review {
                issue: issue(),
                verdict: Verdict::Approve,
            },
        ] {
            let why = w.check(None, &command).expect_err("a person is refused");
            assert!(why.contains("'s: a person may not run it"), "{why}");
        }
        run(
            &mut w,
            Command::Plan {
                issue: issue(),
                plan: plan(),
            },
        )
        .expect("plan");
        for _ in 0..3 {
            run(
                &mut w,
                Command::Ready {
                    issue: issue(),
                    pull: Some("57".into()),
                    pulls: Default::default(),
                },
            )
            .expect("ready");
            let block = Command::Review {
                issue: issue(),
                verdict: Verdict::Block {
                    points: vec!["x".into()],
                },
            };
            run(&mut w, block).expect("block");
        }
        let rule = Command::Rule {
            issue: issue(),
            approve: true,
            reason: "good enough".into(),
        };
        let event = w
            .check(None, &rule)
            .expect("a person may rule")
            .expect("event");
        w.apply(&event);
        assert_eq!(w.decisions[&DecisionId(1)].by.0, PERSON);
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
        for _ in 0..3 {
            run(
                &mut w,
                Command::Ready {
                    issue: issue(),
                    pull: Some("57".into()),
                    pulls: Default::default(),
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

#[cfg(test)]
mod plan_settings {
    use super::*;
    use crate::ids::{GroupId, MachineId, ProjectId};
    use crate::model::Check;
    use crate::settings::{Level, Value};

    fn issue() -> IssueId {
        IssueId::new("ankh", "1")
    }

    /// ankh#1 is being refined by refiner-1.
    fn refining() -> World {
        let (w, _) = World::replay(&[
            Event::ProjectAdded {
                project: ProjectId::new("ankh"),
                tracker: "t".into(),
                name: "city/ankh".into(),
                code_host: None,
                repo: None,
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
            Event::AgentStarted {
                agent: AgentId::new("refiner-1"),
                group: GroupId::new("watch"),
                role: Role::Refiner,
                issue: Some(issue()),
                machine: MachineId::new("local"),
                folder: None,
            },
        ]);
        w
    }

    fn plan(settings: &[(&str, Value)]) -> Command {
        Command::Plan {
            issue: issue(),
            plan: Plan {
                repos: Vec::new(),
                settings: settings
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect(),
                checks: vec![Check {
                    done_when: "the clock strikes thirteen".into(),
                    proven_by: "listening at midnight".into(),
                }],
            },
        }
    }

    fn apply_all(w: &mut World, events: &[Event]) {
        for e in events {
            w.apply(e);
        }
    }

    #[test]
    fn a_plan_with_settings_records_them_as_the_refiners_task_level_events_first() {
        let mut w = refining();
        let refiner = AgentId::new("refiner-1");
        let command = plan(&[
            ("model", Value::Text("opus".into())),
            ("review_rounds", Value::Number(5)),
        ]);
        let events = w.events(Some(&refiner), &command).expect("accepted");
        assert!(matches!(events.last(), Some(Event::PlanWritten { .. })));
        assert_eq!(events.len(), 3, "two settings, then the plan");
        assert!(events[..2].iter().all(|e| matches!(
            e,
            Event::SettingSet { scope: Scope::Task(i), by, .. } if i == &issue() && by == "refiner-1"
        )));
        apply_all(&mut w, &events);
        let model = w
            .setting(&issue(), Some(Role::Author), "model")
            .expect("set");
        assert_eq!(model.source(), "task, set by refiner-1");
        assert_eq!(w.setting_number(&issue(), None, "review_rounds"), 5);
    }

    #[test]
    fn a_person_can_overturn_what_a_refiner_set() {
        let mut w = refining();
        let refiner = AgentId::new("refiner-1");
        let events = w
            .events(
                Some(&refiner),
                &plan(&[("model", Value::Text("opus".into()))]),
            )
            .expect("accepted");
        apply_all(&mut w, &events);
        let set = Command::Set {
            target: Target::Issue(issue()),
            role: None,
            key: "model".into(),
            value: "haiku".into(),
        };
        let events = w.events(None, &set).expect("a person may");
        apply_all(&mut w, &events);
        let r = w.setting(&issue(), None, "model").expect("set");
        assert_eq!(
            (r.value, r.level),
            (Value::Text("haiku".into()), Level::Task)
        );
        assert_eq!(r.by.as_deref(), Some(PERSON));
    }

    #[test]
    fn the_same_key_at_project_level_is_refused_for_a_refiner() {
        let w = refining();
        let refiner = AgentId::new("refiner-1");
        let set = |target: Target| Command::Set {
            target,
            role: None,
            key: "model".into(),
            value: "opus".into(),
        };
        let why = w
            .check(Some(&refiner), &set(Target::Id("ankh".into())))
            .expect_err("refused");
        assert!(why.contains("task-level settings only"), "{why}");
        assert!(w.check(Some(&refiner), &set(Target::Global)).is_err());
        assert!(
            w.check(Some(&refiner), &set(Target::Issue(issue())))
                .is_ok(),
            "its own issue is its to set"
        );
        assert!(
            w.check(
                Some(&refiner),
                &set(Target::Issue(IssueId::new("ankh", "2")))
            )
            .is_err()
        );
    }

    #[test]
    fn a_plan_with_an_unknown_key_or_a_value_of_the_wrong_kind_is_refused() {
        let w = refining();
        let refiner = AgentId::new("refiner-1");
        let bad = plan(&[("colour", Value::Text("red".into()))]);
        assert!(
            w.check(Some(&refiner), &bad)
                .is_err_and(|e| e.contains("colour"))
        );
        let bad = plan(&[("review_rounds", Value::Text("many".into()))]);
        assert!(
            w.check(Some(&refiner), &bad)
                .is_err_and(|e| e.contains("review_rounds"))
        );
    }

    #[test]
    fn an_unset_of_what_is_not_set_is_refused() {
        let w = refining();
        let unset = Command::Unset {
            target: Target::Issue(issue()),
            role: None,
            key: "model".into(),
        };
        assert!(
            w.check(None, &unset)
                .is_err_and(|e| e.contains("nothing to unset"))
        );
    }

    #[test]
    fn retry_is_accepted_when_deploying_or_verifying_and_refused_in_every_other_stage() {
        let mut w = refining();
        let retry = Command::Retry { issue: issue() };
        for (s, ok) in [
            (Stage::New, false),
            (Stage::Refining, false),
            (Stage::Ready, false),
            (Stage::Working, false),
            (Stage::InReview { round: 1 }, false),
            (Stage::Fixing { round: 1 }, false),
            (Stage::Arbitration { round: 3 }, false),
            (Stage::Merging, false),
            (Stage::Deploying, true),
            (Stage::Verifying, true),
            (Stage::Done, false),
        ] {
            if let Some(i) = w.issues.get_mut(&issue()) {
                i.stage = s;
            }
            let got = w.events(None, &retry);
            if ok {
                assert_eq!(got, Ok(vec![Event::Retried { issue: issue() }]), "{s:?}");
            } else {
                let why = got.expect_err("refused");
                assert!(why.contains(&stage(s)), "{why} names {s:?}");
                assert!(why.contains("deploying or verifying"), "{why}");
            }
        }
        let unknown = Command::Retry {
            issue: IssueId::new("ankh", "99"),
        };
        assert!(
            w.events(None, &unknown)
                .is_err_and(|e| e.contains("no issue"))
        );
    }

    #[test]
    fn an_agent_may_not_retry() {
        let mut w = refining();
        if let Some(i) = w.issues.get_mut(&issue()) {
            i.stage = Stage::Deploying;
        }
        let retry = Command::Retry { issue: issue() };
        assert!(
            w.events(Some(&AgentId::new("author-1")), &retry)
                .is_err_and(|e| e.contains("a person's"))
        );
    }
}

#[cfg(test)]
mod ready_in_several_repositories {
    use super::*;
    use crate::ids::{FolderId, GroupId, MachineId};

    fn pid(s: &str) -> ProjectId {
        ProjectId::new(s)
    }

    fn gld() -> IssueId {
        IssueId::new("guild", "GLD-1")
    }

    /// GLD-1 of an area, planned for the repositories `repos`, with the author at work.
    fn working(repos: &[&str]) -> World {
        let mut events = vec![Event::AreaDefined {
            area: pid("guild"),
            name: "Guild".into(),
            projects: vec![pid("ui"), pid("api")],
            tracker: "fake".into(),
            tracker_name: "GLD".into(),
        }];
        for p in ["ui", "api"] {
            events.push(Event::ProjectAdded {
                project: pid(p),
                tracker: String::new(),
                name: format!("group/{p}"),
                code_host: Some("fake".into()),
                repo: None,
            });
            events.push(Event::FolderAdded {
                folder: FolderId::new(format!("{p}-1")),
                project: pid(p),
                machine: MachineId::new("morpork"),
                path: format!("/srv/{p}"),
            });
        }
        events.push(Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Watch".into(),
            projects: vec![],
            areas: vec![pid("guild")],
            milestones: vec![],
            refining_at_once: 1,
            model: None,
        });
        events.push(Event::IssueSynced {
            issue: gld(),
            title: "t".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        events.push(Event::PlanWritten {
            issue: gld(),
            plan: Plan {
                repos: repos.iter().map(|r| pid(r)).collect(),
                ..Plan::default()
            },
        });
        events.push(Event::AgentStarted {
            agent: AgentId::new("author-1"),
            group: GroupId::new("watch"),
            role: Role::Author,
            issue: Some(gld()),
            machine: MachineId::new("morpork"),
            folder: None,
        });
        World::replay(&events).0
    }

    fn ready(pull: Option<&str>, pulls: &[(&str, &str)]) -> Command {
        Command::Ready {
            issue: gld(),
            pull: pull.map(String::from),
            pulls: pulls.iter().map(|(r, n)| (pid(r), n.to_string())).collect(),
        }
    }

    fn check(w: &World, c: &Command) -> Result<Event, String> {
        w.check(Some(&AgentId::new("author-1")), c)?
            .ok_or_else(|| "no event".to_string())
    }

    #[test]
    fn names_every_repositorys_pull_request() {
        let w = working(&["ui", "api"]);
        let event = check(&w, &ready(None, &[("ui", "12"), ("api", "34")])).expect("ready");
        assert_eq!(
            event,
            Event::PullRequestReady {
                issue: gld(),
                pull: None,
                pulls: [
                    (pid("ui"), "12".to_string()),
                    (pid("api"), "34".to_string())
                ]
                .into(),
            }
        );
    }

    #[test]
    fn a_bare_number_works_for_one_repository() {
        let w = working(&["ui"]);
        let event = check(&w, &ready(Some("12"), &[])).expect("ready");
        assert!(matches!(
            event,
            Event::PullRequestReady { pulls, .. } if pulls.get(&pid("ui")).map(String::as_str) == Some("12")
        ));
    }

    #[test]
    fn a_bare_number_is_refused_when_the_plan_names_several() {
        let w = working(&["ui", "api"]);
        let e = check(&w, &ready(Some("12"), &[])).expect_err("refused");
        assert!(e.contains("say which: repo=number"), "{e}");
    }

    #[test]
    fn a_repository_left_out_the_first_time_or_not_in_the_plan_is_refused() {
        let w = working(&["ui", "api"]);
        let e = check(&w, &ready(None, &[("ui", "12")])).expect_err("api left out");
        assert!(
            e.contains("api=<number>") && !e.contains("ui=<number>"),
            "{e}"
        );
        let e = check(&w, &ready(None, &[("ui", "12"), ("db", "3")])).expect_err("db");
        assert!(e.contains("db is not a repository"), "{e}");
    }

    #[test]
    fn pull_requests_on_record_need_not_be_named_again() {
        let mut w = working(&["ui", "api"]);
        w.apply(&Event::PullRequestReady {
            issue: gld(),
            pull: None,
            pulls: [
                (pid("ui"), "12".to_string()),
                (pid("api"), "34".to_string()),
            ]
            .into(),
        });
        w.apply(&Event::ReviewFinished {
            issue: gld(),
            round: 1,
            verdict: Verdict::Block {
                points: vec!["x".into()],
            },
        });
        assert!(check(&w, &ready(None, &[])).is_ok());
    }
}
