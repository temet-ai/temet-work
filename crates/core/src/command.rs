//! What a person or an agent asks the daemon to do, by name. The app, the
//! command line and the MCP server all send these, and agents run the same
//! commands a person does. The loop checks each one against the issue's stage
//! before it becomes an event, so a command cannot move an issue past a step it
//! is not at.

use serde::{Deserialize, Serialize};

use crate::event::{Event, Verdict};
use crate::ids::{AgentId, DecisionId, IssueId, ProjectId, RuleId};
use crate::model::{Decision, DecisionKind, Plan, Role, Rule, RuleState, Stage};
use crate::settings::{self, Agents, Scope, Target};
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
    /// A person's overturn of a decision an agent made.
    Overturn {
        /// The decision, as `status` lists it.
        decision: DecisionId,
        /// The option now preferred, counted from 1 as `status` lists them.
        option: usize,
        /// Why; required.
        reason: String,
    },
    /// A person's answer on a rule proposed from an overturn.
    Proposal {
        /// The rule, as `status` lists it.
        rule: RuleId,
        /// True to keep it, false to drop it.
        keep: bool,
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
            | Self::Settings { .. }
            | Self::Overturn { .. }
            | Self::Proposal { .. } => None,
            Self::Issue { issue } => Some(issue),
            Self::Plan { issue, .. }
            | Self::Ready { issue, .. }
            | Self::Review { issue, .. }
            | Self::Retry { issue }
            | Self::Rule { issue, .. } => Some(issue),
        }
    }
}

/// A command and who sent it. Its `Debug` form hides the secret.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    /// The agent that sent it; `None` for a person.
    pub by: Option<AgentId>,
    /// The secret the sending agent was started with
    /// ([`SECRET_VAR`](crate::launch::SECRET_VAR)), when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// Set by `temet-work relay`: the request came from another machine.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub relayed: bool,
    /// The command.
    pub command: Command,
}

impl Request {
    /// A request from `by`, or from a person, carrying no secret.
    pub fn new(by: Option<AgentId>, command: Command) -> Self {
        Self {
            by,
            secret: None,
            relayed: false,
            command,
        }
    }
}

impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("by", &self.by)
            .field("secret", &self.secret.as_ref().map(|_| "<hidden>"))
            .field("relayed", &self.relayed)
            .field("command", &self.command)
            .finish()
    }
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
                    role: settings::recorded_role(None, key, &who),
                    key: key.clone(),
                    value: value.clone(),
                    by: who.clone(),
                });
            }
        }
        // An overturn proposes a rule, once for the decision.
        // A person's change of a model also drops an agent's value for the
        // author under the model's other name. It comes after the change, which
        // is what the command is answered with.
        let other = match (&event, by) {
            (
                Event::SettingSet {
                    scope, role, key, ..
                }
                | Event::SettingUnset {
                    scope, role, key, ..
                },
                None,
            ) => self
                .agents_other_model(scope, *role, key)
                .map(|key| Event::SettingUnset {
                    scope: scope.clone(),
                    role: Some(Role::Author),
                    key,
                    by: PERSON.to_string(),
                }),
            _ => None,
        };
        let rule = match &event {
            Event::DecisionOverturned {
                decision,
                to,
                reason,
                by,
            } => self.propose(*decision, *to, reason, by),
            _ => None,
        };
        out.push(event);
        out.extend(other);
        out.extend(rule.map(|rule| Event::RuleProposed { rule }));
        Ok(out)
    }

    /// The rule an overturn of `decision` to option index `to` proposes, unless the
    /// decision already has one.
    fn propose(&self, decision: DecisionId, to: usize, reason: &str, by: &str) -> Option<Rule> {
        let d = self.decisions.get(&decision)?;
        if self.rules.values().any(|r| r.decision == decision) {
            return None;
        }
        let preferred = d.options.get(to)?;
        let id = self.rules.keys().next_back().map_or(1, |r| r.0 + 1);
        Some(Rule {
            id: RuleId(id),
            text: format!("{} Prefer: {preferred}. Why: {}", d.question, reason.trim()),
            decision,
            issue: d.issue.clone(),
            decided_by: d.by.clone(),
            overturned_by: by.to_string(),
            state: RuleState::Proposed,
        })
    }

    /// Why `by` may not change `key` at `scope` for `role` (none: the command
    /// names no role). A person may change any setting anywhere. An agent may
    /// change what the setting's [`settings::Spec`] leaves to agents, for the
    /// roles it leaves to them, and only as the refiner of the issue at `scope`,
    /// in its plan or by command, while that issue is refining.
    fn may_set(
        &self,
        by: Option<&AgentId>,
        scope: &Scope,
        role: Option<Role>,
        key: &str,
    ) -> Result<(), String> {
        let Some(by) = by else {
            return Ok(());
        };
        let Some(agent) = self.agents.get(by) else {
            return Err(format!("{by} is not an agent this daemon started"));
        };
        let spec = settings::spec(key)?;
        let agents = spec.agents;
        if let Agents::MayNot(why) = agents {
            return Err(format!("only a person sets {key}, because {why}"));
        }
        match scope {
            Scope::Task(issue)
                if agent.role == Role::Refiner && agent.issue.as_ref() == Some(issue) =>
            {
                match self.issues.get(issue).map(|i| i.stage) {
                    Some(Stage::Refining) => match role {
                        Some(other) if agents == Agents::ForAuthor && other != Role::Author => {
                            Err(format!(
                                "only a person sets {key} for the {}, because the {}s of \
                                 reviewers, arbiters, refiners and investigators are a person's \
                                 to set; a refiner's {key} is for the author",
                                other.name(),
                                spec.key
                            ))
                        }
                        _ => Ok(()),
                    },
                    Some(now) => Err(format!(
                        "{} may change settings of {issue} only while it is refining; \
                         it is {} now",
                        agent.id,
                        stage(now)
                    )),
                    None => Err(format!("there is no issue {issue}")),
                }
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
                let task = Scope::Task(issue.clone());
                for (key, value) in &plan.settings {
                    self.may_set(by, &task, None, key)
                        .and_then(|()| settings::check_value(key, value))
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
                self.may_set(by, &scope, *role, key)?;
                let value = settings::parse_value(key, value)?;
                self.check_choice(key, &value)?;
                let by = by.map_or_else(|| PERSON.to_string(), |a| a.0.clone());
                Event::SettingSet {
                    scope,
                    role: settings::recorded_role(*role, key, &by),
                    key: key.clone(),
                    value,
                    by,
                }
            }
            Command::Unset { target, role, key } => {
                let scope = self.scope_of(target)?;
                self.may_set(by, &scope, *role, key)?;
                settings::spec(key)?;
                let who = by.map_or_else(|| PERSON.to_string(), |a| a.0.clone());
                let at = settings::recorded_role(*role, key, &who);
                // A person's unset for every role also takes an agent's value for
                // the author away, so it has something to unset when only that is.
                let there = self
                    .settings
                    .contains_key(&(scope.clone(), at, key.clone()))
                    || (by.is_none()
                        && (at.is_none() && self.agents_author_value(&scope, key)
                            || self.agents_other_model(&scope, at, key).is_some()));
                if !there {
                    // A value left from a build that had the runtime can be unset.
                    self.check_model_runtime(key)?;
                    let for_role = at.map_or_else(String::new, |r| format!(" for {}", r.name()));
                    return Err(format!(
                        "{key} is not set at {target}{for_role}; nothing to unset"
                    ));
                }
                Event::SettingUnset {
                    scope,
                    role: at,
                    key: key.clone(),
                    by: who,
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
            Command::Overturn {
                decision,
                option,
                reason,
            } => {
                if let Some(by) = by {
                    return Err(format!(
                        "overturn is a person's: agents are the ones overturned, and {by} is an agent"
                    ));
                }
                let d = self
                    .decisions
                    .get(decision)
                    .ok_or_else(|| format!("there is no decision {}", decision.0))?;
                if *option == 0 || *option > d.options.len() {
                    return Err(format!(
                        "decision {} has options 1 to {}, not {option}",
                        decision.0,
                        d.options.len()
                    ));
                }
                let to = option - 1;
                if to == d.chosen {
                    return Err(format!(
                        "option {option} ({}) is the one already chosen for decision {}",
                        d.options[to], decision.0
                    ));
                }
                if d.overturned_to == Some(to) {
                    return Err(format!(
                        "decision {} is already overturned to option {option}",
                        decision.0
                    ));
                }
                if reason.trim().is_empty() {
                    return Err("an overturn needs a reason".into());
                }
                Event::DecisionOverturned {
                    decision: *decision,
                    to,
                    reason: reason.trim().to_string(),
                    by: PERSON.to_string(),
                }
            }
            Command::Proposal { rule, keep } => {
                if let Some(by) = by {
                    return Err(format!(
                        "proposal is a person's: {by} is an agent; kept rules reach its brief only through a person"
                    ));
                }
                let r = self
                    .rules
                    .get(rule)
                    .ok_or_else(|| format!("there is no rule {}", rule.0))?;
                if r.state != RuleState::Proposed {
                    return Err(format!(
                        "rule {} is {}, not proposed; only a proposed rule can be kept or dropped",
                        rule.0,
                        r.state.name()
                    ));
                }
                Event::RuleAnswered {
                    rule: *rule,
                    keep: *keep,
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
                        overturn_reason: String::new(),
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
            secret_digest: None,
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
    use crate::settings::Value;

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
                secret_digest: None,
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

    fn set(target: &str, role: Option<Role>, key: &str, value: &str) -> Command {
        Command::Set {
            target: Target::parse(target),
            role,
            key: key.into(),
            value: value.into(),
        }
    }

    fn unset(role: Option<Role>) -> Command {
        Command::Unset {
            target: Target::Issue(issue()),
            role,
            key: "model".into(),
        }
    }

    /// Runs `command` as `by` and applies what it becomes.
    fn run(w: &mut World, by: Option<&str>, command: &Command) -> Vec<Event> {
        let by = by.map(AgentId::new);
        let events = w.events(by.as_ref(), command).expect("accepted");
        apply_all(w, &events);
        events
    }

    /// The value `role` reads for ankh#1 and where it comes from.
    fn read(w: &World, role: Role, key: &str) -> Option<(String, String)> {
        w.setting(&issue(), Some(role), key)
            .map(|r| (r.value.to_string(), r.source()))
    }

    fn is(value: &str, source: &str) -> Option<(String, String)> {
        Some((value.to_string(), source.to_string()))
    }

    #[test]
    fn a_plan_with_settings_records_them_as_the_refiners_task_level_events_first() {
        let mut w = refining();
        let command = plan(&[
            ("model", Value::Text("opus".into())),
            ("starts", Value::Number(5)),
        ]);
        let events = run(&mut w, Some("refiner-1"), &command);
        assert!(matches!(events.last(), Some(Event::PlanWritten { .. })));
        assert_eq!(events.len(), 3, "two settings, then the plan");
        let recorded: Vec<(&str, Option<Role>)> = events[..2]
            .iter()
            .map(|e| match e {
                Event::SettingSet {
                    scope: Scope::Task(i),
                    role,
                    key,
                    by,
                    ..
                } if i == &issue() && by == "refiner-1" => (key.as_str(), *role),
                other => panic!("not the refiner's task-level setting: {other:?}"),
            })
            .collect();
        assert_eq!(recorded, [("model", Some(Role::Author)), ("starts", None)]);
        assert_eq!(
            read(&w, Role::Author, "model"),
            is("opus", "task, set by refiner-1")
        );
        assert_eq!(w.setting_number(&issue(), None, "starts"), 5);
    }

    #[test]
    fn a_refiners_model_is_for_the_author_only() {
        let mut w = refining();
        for role in [Role::Reviewer, Role::Arbiter] {
            run(&mut w, None, &set("ankh", Some(role), "model", "opus"));
        }
        let command = plan(&[("model", Value::Text("sonnet".into()))]);
        run(&mut w, Some("refiner-1"), &command);
        for (role, model) in [
            (Role::Author, "sonnet"),
            (Role::Reviewer, "opus"),
            (Role::Arbiter, "opus"),
        ] {
            let launch = w
                .launch(&GroupId::new("watch"), role, &issue(), None)
                .expect("known");
            assert_eq!(launch.brief.model.as_deref(), Some(model), "{role:?}");
        }
    }

    #[test]
    fn a_persons_model_overturns_the_refiners_under_either_of_its_names() {
        let names = ["model", "model.alpha"];
        for (theirs, mine) in [(names[0], names[1]), (names[1], names[0])] {
            for role in [None, Some(Role::Author)] {
                let mut w = refining();
                run(&mut w, None, &set("global", None, "runtime", "alpha"));
                run(
                    &mut w,
                    Some("refiner-1"),
                    &set("ankh#1", None, theirs, "opus"),
                );
                let events = run(&mut w, None, &set("ankh#1", role, mine, "haiku"));
                assert!(
                    matches!(&events[0], Event::SettingSet { key, .. } if key == mine),
                    "the command is answered with its own change: {events:?}"
                );
                let model = read(&w, Role::Author, "model").map(|(value, _)| value);
                assert_eq!(model.as_deref(), Some("haiku"), "{theirs} {mine} {role:?}");
            }
            // An unset by the other name takes the refiner's value away too.
            let mut w = refining();
            run(&mut w, None, &set("global", None, "runtime", "alpha"));
            run(
                &mut w,
                Some("refiner-1"),
                &set("ankh#1", None, theirs, "opus"),
            );
            let unset = Command::Unset {
                target: Target::Issue(issue()),
                role: None,
                key: mine.into(),
            };
            run(&mut w, None, &unset);
            assert_eq!(read(&w, Role::Author, "model"), None, "{theirs} {mine}");
        }
        // Another runtime's model is not the same model, so it takes nothing away.
        let mut w = refining();
        run(&mut w, None, &set("global", None, "runtime", "alpha"));
        run(
            &mut w,
            Some("refiner-1"),
            &set("ankh#1", None, "model", "opus"),
        );
        let events = run(&mut w, None, &set("ankh#1", None, "model.beta", "gpt-x"));
        assert_eq!(events.len(), 1, "{events:?}");
        let model = read(&w, Role::Author, "model").map(|(value, _)| value);
        assert_eq!(model.as_deref(), Some("opus"));
    }

    #[test]
    fn a_refiners_set_of_model_is_for_the_author_and_refused_for_other_roles() {
        let refiner = AgentId::new("refiner-1");
        for role in [None, Some(Role::Author)] {
            let mut w = refining();
            let command = set("ankh#1", role, "model", "x");
            let events = run(&mut w, Some("refiner-1"), &command);
            assert_eq!(
                events,
                [Event::SettingSet {
                    scope: Scope::Task(issue()),
                    role: Some(Role::Author),
                    key: "model".into(),
                    value: Value::Text("x".into()),
                    by: "refiner-1".into(),
                }]
            );
        }
        for role in [
            Role::Reviewer,
            Role::Arbiter,
            Role::Refiner,
            Role::Investigator,
        ] {
            let w = refining();
            for command in [set("ankh#1", Some(role), "model", "x"), unset(Some(role))] {
                let why = w.events(Some(&refiner), &command).expect_err("refused");
                assert!(
                    why.starts_with(&format!(
                        "only a person sets model for the {}, because ",
                        role.name()
                    )) && why.contains("are a person's to set"),
                    "{why}"
                );
            }
        }
        // Its unset, with no role, removes the author's value.
        let mut w = refining();
        run(
            &mut w,
            Some("refiner-1"),
            &set("ankh#1", None, "model", "x"),
        );
        let events = run(&mut w, Some("refiner-1"), &unset(None));
        assert_eq!(
            events,
            [Event::SettingUnset {
                scope: Scope::Task(issue()),
                role: Some(Role::Author),
                key: "model".into(),
                by: "refiner-1".into(),
            }]
        );
        assert_eq!(read(&w, Role::Author, "model"), None);
    }

    #[test]
    fn a_machine_in_a_plan_is_the_refiners_task_level_setting() {
        let mut w = refining();
        let refiner = AgentId::new("refiner-1");
        let command = plan(&[("machine", Value::Text("mac".into()))]);
        let events = w.events(Some(&refiner), &command).expect("accepted");
        assert_eq!(
            events[0],
            Event::SettingSet {
                scope: Scope::Task(issue()),
                role: None,
                key: "machine".into(),
                value: Value::Text("mac".into()),
                by: "refiner-1".into(),
            }
        );
        apply_all(&mut w, &events);
        assert_eq!(
            w.machine_for(&issue(), Role::Author),
            Some(MachineId::new("mac"))
        );
        // No folder is on it: accepted all the same, and the issue waits.
        assert_eq!(w.issues[&issue()].stage, Stage::Ready);
        assert_eq!(w.machine_waits().len(), 1);
        let empty = plan(&[("machine", Value::Text(String::new()))]);
        assert!(
            refining()
                .check(Some(&refiner), &empty)
                .is_err_and(|e| e.contains("machine"))
        );
    }

    #[test]
    fn a_person_can_overturn_what_a_refiner_set() {
        let mut w = refining();
        let command = plan(&[("model", Value::Text("opus".into()))]);
        run(&mut w, Some("refiner-1"), &command);
        run(&mut w, None, &set("ankh#1", None, "model", "haiku"));
        for role in [Role::Author, Role::Reviewer, Role::Arbiter] {
            assert_eq!(
                read(&w, role, "model"),
                is("haiku", "task, set by person"),
                "{role:?}"
            );
        }
    }

    #[test]
    fn a_persons_own_value_for_the_author_is_kept() {
        let mut w = refining();
        run(
            &mut w,
            None,
            &set("ankh#1", Some(Role::Author), "model", "opus"),
        );
        run(&mut w, None, &set("ankh#1", None, "model", "haiku"));
        assert_eq!(
            read(&w, Role::Author, "model"),
            is("opus", "task, set by person")
        );
        run(&mut w, None, &unset(None));
        assert_eq!(
            read(&w, Role::Author, "model"),
            is("opus", "task, set by person")
        );
        assert!(
            w.events(None, &unset(None))
                .is_err_and(|e| e.contains("nothing to unset"))
        );
    }

    #[test]
    fn a_persons_unset_for_every_role_removes_a_refiners_value_for_the_author() {
        let mut w = refining();
        run(&mut w, None, &set("ankh", None, "model", "opus"));
        let command = plan(&[("model", Value::Text("haiku".into()))]);
        run(&mut w, Some("refiner-1"), &command);
        run(&mut w, None, &unset(None));
        assert_eq!(
            read(&w, Role::Author, "model"),
            is("opus", "project, set by person")
        );
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

    /// A refiner's plan, set and unset of `key`, for every role, are each refused
    /// naming it, and leave nothing behind; a person's set is accepted.
    fn only_a_person_may_set(key: &str, typed: &str, value: Value) {
        let mut w = refining();
        let refiner = AgentId::new("refiner-1");
        let set = |role| Command::Set {
            target: Target::Issue(issue()),
            role,
            key: key.into(),
            value: typed.into(),
        };
        let unset = |role| Command::Unset {
            target: Target::Issue(issue()),
            role,
            key: key.into(),
        };
        let refused = |w: &World, command: &Command| {
            let why = w.events(Some(&refiner), command).expect_err("refused");
            assert!(
                why.contains(&format!("only a person sets {key}, because ")),
                "{why}"
            );
        };
        refused(&w, &plan(&[(key, value.clone())]));
        let roles = [
            None,
            Some(Role::Refiner),
            Some(Role::Author),
            Some(Role::Reviewer),
            Some(Role::Arbiter),
            Some(Role::Investigator),
        ];
        for role in roles {
            refused(&w, &set(role));
            refused(&w, &unset(role));
            // A person's set is accepted, and the value is not an agent's to remove.
            let events = w.events(None, &set(role)).expect("a person may set it");
            assert_eq!(
                events,
                vec![Event::SettingSet {
                    scope: Scope::Task(issue()),
                    role,
                    key: key.into(),
                    value: value.clone(),
                    by: PERSON.into(),
                }]
            );
            apply_all(&mut w, &events);
            refused(&w, &unset(role));
            assert!(
                w.events(None, &unset(role)).is_ok(),
                "a person may unset it"
            );
        }
        assert_eq!(w.issues[&issue()].stage, Stage::Refining, "no plan is in");
        assert_eq!(w.issues[&issue()].plan, None);
    }

    #[test]
    fn a_refiners_brief_says_its_model_is_the_authors() {
        let w = refining();
        let launch = w
            .launch(&GroupId::new("watch"), Role::Refiner, &issue(), None)
            .expect("known");
        let task = launch.brief.task;
        assert!(
            task.contains(
                "A model you set is the author's only: the models of reviewers, arbiters and \
                 refiners are a person's to set."
            ),
            "{task}"
        );
        assert!(
            task.contains(
                "Only a person sets runtime, runner, permission, review_rounds, ci, deploy, \
                 verify, run_in:"
            ),
            "{task}"
        );
    }

    #[test]
    fn an_agent_may_not_set_deploy() {
        only_a_person_may_set("deploy", "make deploy", Value::Text("make deploy".into()));
    }

    #[test]
    fn an_agent_may_not_set_verify() {
        only_a_person_may_set("verify", "make smoke", Value::Text("make smoke".into()));
    }

    #[test]
    fn an_agent_may_not_set_run_in() {
        only_a_person_may_set("run_in", "/srv/ankh", Value::Text("/srv/ankh".into()));
    }

    #[test]
    fn an_agent_may_not_set_ci() {
        only_a_person_may_set("ci", "false", Value::Flag(false));
    }

    #[test]
    fn an_agent_may_not_set_permission() {
        let bypass = Value::Text("bypassPermissions".into());
        only_a_person_may_set("permission", "bypassPermissions", bypass);
        // A person's value reaches the next agent's brief.
        let mut w = refining();
        let set = Command::Set {
            target: Target::Issue(issue()),
            role: None,
            key: "permission".into(),
            value: "plan".into(),
        };
        let events = w.events(None, &set).expect("a person may");
        apply_all(&mut w, &events);
        let events = w
            .events(Some(&AgentId::new("refiner-1")), &plan(&[]))
            .expect("accepted");
        apply_all(&mut w, &events);
        let launch = w
            .launch(&GroupId::new("watch"), Role::Author, &issue(), None)
            .expect("known");
        assert_eq!(launch.brief.permission, "plan");
    }

    #[test]
    fn an_agent_may_not_set_runtime() {
        // Any adapter id: this world's build reports none to check it against.
        let other = Value::Text("clacks".into());
        only_a_person_may_set("runtime", "clacks", other);
        let why = refining()
            .events(
                Some(&AgentId::new("refiner-1")),
                &plan(&[("runtime", Value::Text("clacks".into()))]),
            )
            .expect_err("refused");
        assert!(
            why.contains(
                "a runtime may not follow permission, so the runtime decides what an agent \
                 may do without asking"
            ),
            "{why}"
        );
    }

    #[test]
    fn an_agent_may_not_set_runner() {
        only_a_person_may_set("runner", "clacks", Value::Text("clacks".into()));
    }

    #[test]
    fn an_agent_may_not_set_review_rounds() {
        only_a_person_may_set("review_rounds", "1", Value::Number(1));
    }

    #[test]
    fn a_refiners_plan_may_still_set_merge_and_model() {
        let mut w = refining();
        let command = plan(&[
            ("merge", Value::Text("rebase".into())),
            ("model", Value::Text("opus".into())),
        ]);
        let events = w
            .events(Some(&AgentId::new("refiner-1")), &command)
            .expect("accepted");
        apply_all(&mut w, &events);
        for (key, value) in [("merge", "rebase"), ("model", "opus")] {
            let r = w.setting(&issue(), Some(Role::Author), key).expect("set");
            assert_eq!(r.value, Value::Text(value.into()));
            assert_eq!(r.source(), "task, set by refiner-1");
        }
    }

    #[test]
    fn a_refiners_set_after_its_issue_left_refining_is_refused() {
        let mut w = refining();
        let refiner = AgentId::new("refiner-1");
        let set = Command::Set {
            target: Target::Issue(issue()),
            role: None,
            key: "model".into(),
            value: "opus".into(),
        };
        let unset = Command::Unset {
            target: Target::Issue(issue()),
            role: None,
            key: "model".into(),
        };
        let events = w.events(Some(&refiner), &set).expect("while refining");
        apply_all(&mut w, &events);
        assert!(w.events(Some(&refiner), &unset).is_ok(), "while refining");
        let events = w.events(Some(&refiner), &plan(&[])).expect("accepted");
        apply_all(&mut w, &events);
        for command in [&set, &unset] {
            assert_eq!(
                w.events(Some(&refiner), command),
                Err(
                    "refiner-1 may change settings of ankh#1 only while it is refining; \
                     it is ready now"
                        .into()
                )
            );
        }
        assert!(w.events(None, &unset).is_ok(), "a person still may");
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

    /// A world with decision 1: refiner-1 chose option 2 of 3 for ankh#1.
    fn decided() -> World {
        let mut w = refining();
        w.apply(&Event::DecisionMade {
            decision: Decision {
                id: DecisionId(1),
                issue: issue(),
                by: AgentId::new("refiner-1"),
                kind: DecisionKind::Choice,
                question: "Which connection?".into(),
                options: vec!["second".into(), "legacy".into(), "none".into()],
                chosen: 1,
                reason: "easiest to undo".into(),
                overturned_to: None,
                overturn_reason: String::new(),
            },
        });
        w
    }

    fn overturn(decision: u64, option: usize, reason: &str) -> Command {
        Command::Overturn {
            decision: DecisionId(decision),
            option,
            reason: reason.into(),
        }
    }

    #[test]
    fn overturn_is_accepted_from_a_person_and_records_who_and_why() {
        let w = decided();
        let events = w
            .events(None, &overturn(1, 1, "legacy breaks channels"))
            .expect("a person may");
        assert!(matches!(
            &events[0],
            Event::DecisionOverturned { decision, to: 0, reason, by }
                if *decision == DecisionId(1) && reason == "legacy breaks channels" && by == PERSON
        ));
        assert_eq!(events.len(), 2, "and a proposed rule");
    }

    #[test]
    fn overturn_is_refused_with_the_reason_for_each_bad_request() {
        let w = decided();
        let why = |c: Command, by: Option<&AgentId>| w.events(by, &c).expect_err("refused");
        assert!(why(overturn(9, 1, "x"), None).contains("no decision 9"));
        assert!(why(overturn(1, 0, "x"), None).contains("options 1 to 3"));
        assert!(why(overturn(1, 4, "x"), None).contains("options 1 to 3"));
        assert!(why(overturn(1, 2, "x"), None).contains("already chosen"));
        assert!(why(overturn(1, 1, "  "), None).contains("needs a reason"));
        let agent = AgentId::new("author-1");
        assert!(why(overturn(1, 1, "x"), Some(&agent)).contains("a person's"));
        let mut w = decided();
        let events = w.events(None, &overturn(1, 1, "x")).expect("first");
        for e in &events {
            w.apply(e);
        }
        let again = w.events(None, &overturn(1, 1, "x")).expect_err("twice");
        assert!(again.contains("already overturned"), "{again}");
    }

    #[test]
    fn rule_proposed_carries_text_origin_and_state_and_is_made_once_per_decision() {
        let mut w = decided();
        for e in w
            .events(None, &overturn(1, 1, "legacy breaks channels"))
            .expect("ok")
        {
            w.apply(&e);
        }
        let rule = w.rules[&RuleId(1)].clone();
        assert_eq!(
            rule.text,
            "Which connection? Prefer: second. Why: legacy breaks channels"
        );
        assert_eq!(rule.state, RuleState::Proposed);
        assert_eq!(
            (
                rule.decision,
                &rule.issue,
                rule.decided_by.0.as_str(),
                rule.overturned_by.as_str()
            ),
            (DecisionId(1), &issue(), "refiner-1", "person")
        );
        // Overturned again, to another option: no second rule.
        let events = w.events(None, &overturn(1, 3, "or none")).expect("ok");
        assert_eq!(events.len(), 1);
        for e in &events {
            w.apply(e);
        }
        assert_eq!(w.rules.len(), 1);
        // And a replayed RuleProposed for the same decision is ignored.
        let dup = Event::RuleProposed { rule };
        w.apply(&dup);
        assert_eq!(w.rules.len(), 1);
    }

    #[test]
    fn rule_proposed_shows_in_the_rules_section_of_status() {
        let mut w = decided();
        for e in w
            .events(None, &overturn(1, 1, "legacy breaks channels"))
            .expect("ok")
        {
            w.apply(&e);
        }
        let text = crate::text::text(&w, &Default::default());
        let rules = text.split("Rules\n").nth(1).expect("a Rules section");
        assert!(
            rules.contains("1 proposed: Which connection? Prefer: second."),
            "{rules}"
        );
        assert!(
            rules.contains("from decision 1 on ankh#1, decided by refiner-1, overturned by person")
        );
        assert!(text.contains("1=second, 2=legacy, 3=none"));
        assert!(text.contains("overturned to option 1 (second): legacy breaks channels"));
    }

    fn proposal(rule: u64, keep: bool) -> Command {
        Command::Proposal {
            rule: RuleId(rule),
            keep,
        }
    }

    fn with_rule() -> World {
        let mut w = decided();
        for e in w
            .events(None, &overturn(1, 1, "legacy breaks channels"))
            .expect("ok")
        {
            w.apply(&e);
        }
        w
    }

    #[test]
    fn proposal_keep_and_drop_move_the_rule_and_refuse_the_rest_with_why() {
        let mut w = with_rule();
        let agent = AgentId::new("author-1");
        assert!(
            w.events(Some(&agent), &proposal(1, true))
                .expect_err("agent")
                .contains("a person's")
        );
        assert!(
            w.events(None, &proposal(7, true))
                .expect_err("unknown")
                .contains("no rule 7")
        );
        let events = w.events(None, &proposal(1, true)).expect("keep");
        assert_eq!(
            events,
            vec![Event::RuleAnswered {
                rule: RuleId(1),
                keep: true
            }]
        );
        w.apply(&events[0]);
        assert_eq!(w.rules[&RuleId(1)].state, RuleState::Kept);
        let again = w
            .events(None, &proposal(1, false))
            .expect_err("not proposed");
        assert!(again.contains("kept, not proposed"), "{again}");
        let mut w = with_rule();
        w.apply(&Event::RuleAnswered {
            rule: RuleId(1),
            keep: false,
        });
        assert_eq!(w.rules[&RuleId(1)].state, RuleState::Dropped);
        let text = crate::text::text(&w, &Default::default());
        assert!(
            !text.contains("Prefer: second"),
            "a dropped rule is not in status"
        );
    }

    #[test]
    fn brief_ends_with_the_kept_rules_for_every_role_and_omits_proposed_and_dropped() {
        let roles = [
            Role::Refiner,
            Role::Author,
            Role::Reviewer,
            Role::Arbiter,
            Role::Investigator,
        ];
        let task = |w: &World, role| {
            w.launch(&GroupId::new("watch"), role, &issue(), None)
                .expect("launch")
                .brief
                .task
        };
        let mut w = with_rule();
        for role in roles {
            assert!(
                !task(&w, role).contains("Rules from overturned"),
                "proposed: {role:?}"
            );
        }
        w.apply(&Event::RuleAnswered {
            rule: RuleId(1),
            keep: true,
        });
        for role in roles {
            let t = task(&w, role);
            assert!(
                t.ends_with("Rules from overturned decisions:\n- Which connection? Prefer: second. Why: legacy breaks channels\n"),
                "{role:?}: {t}"
            );
        }
        let mut w = with_rule();
        w.apply(&Event::RuleAnswered {
            rule: RuleId(1),
            keep: false,
        });
        for role in roles {
            assert!(
                !task(&w, role).contains("Prefer: second"),
                "dropped: {role:?}"
            );
        }
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
            secret_digest: None,
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
