//! What a person or an agent asks the daemon to do, by name. The app, the
//! command line and the MCP server all send these, and agents run the same
//! commands a person does. The loop checks each one against the issue's stage
//! before it becomes an event, so a command cannot move an issue past a step it
//! is not at.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::brief::{MOST_POINTS, numbered, reply_flags};
use crate::effect::Effect;
use crate::event::{Event, Verdict};
use crate::ids::{AgentId, DecisionId, GroupId, IssueId, MilestoneId, ProjectId, RuleId};
use crate::model::{
    CheckState, Cost, Decision, DecisionKind, Plan, Point, Role, Rule, RuleState, Stage,
};
use crate::settings::{self, Agents, Scope, Target};
use crate::text::{clean, stage};
use crate::world::World;

/// One command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    /// The state as text.
    Status,
    /// The state as data ([`View`](crate::View)), for a front end.
    View,
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
        /// The author's replies to the points of the block this ready answers
        /// ([`crate::model::Issue::block_to_answer`]), one for each point, in
        /// their order; none for any other ready.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        replies: Vec<String>,
    },
    /// A reviewer's verdict on the round under review.
    Review {
        /// The issue.
        issue: IssueId,
        /// The verdict.
        verdict: Verdict,
        /// On a block, the plan's checks that do not hold, counted from 1 in the
        /// plan's order; every check left out counts as passed.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        failed: Vec<usize>,
        /// On a block after round 1, the points of the round before that are
        /// still not dealt with, counted from 1 in its order: each stays open
        /// and joins this block's points. Every point left out is resolved.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        open: Vec<usize>,
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
    /// An agent records a choice it made on the issue it works on, as a decision
    /// a person can overturn later. The agent carries on at once; the reply
    /// names the new decision. Every field may be missing here, so that the loop
    /// refuses an incomplete choice with its reason, the same from the command
    /// line, MCP and a raw socket request.
    Choose {
        /// The issue.
        issue: IssueId,
        /// What was decided, in a line.
        #[serde(default)]
        question: String,
        /// The options weighed: at least two, each once.
        #[serde(default)]
        options: Vec<String>,
        /// The option taken, as written among `options`.
        #[serde(default)]
        took: String,
        /// Why it was taken.
        #[serde(default)]
        why: String,
        /// What overturning it costs: `low`, `medium` or `high`; refused when
        /// none is given.
        #[serde(default)]
        cost: Option<String>,
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
    /// An investigator files an issue in the tracker of a project or area its
    /// group takes issues from. The reply names the new issue.
    File {
        /// The project or area.
        project: ProjectId,
        /// The issue's title.
        title: String,
        /// Its text.
        #[serde(default)]
        body: String,
        /// Its milestone; the last of the group's when none is named.
        #[serde(default)]
        milestone: Option<String>,
    },
    /// An investigator says its group's investigation is done.
    Investigated {
        /// The group.
        group: GroupId,
    },
    /// A person holds the daemon: nothing new starts, and agents at work finish
    /// their turn ([`World::held`]).
    Hold,
    /// A person lifts the hold, so work starts again.
    Resume,
    /// A person stops one agent: its terminal is stopped, and no agent starts
    /// on its issue until a person starts the issue again ([`Command::Start`]).
    Stop {
        /// The agent.
        agent: AgentId,
    },
    /// A person lets agents start again on an issue whose agent a person
    /// stopped. The stopped agent stays stopped.
    Start {
        /// The issue.
        issue: IssueId,
    },
}

/// How long a front end waits after one [`Command::View`] ends, answered or failed,
/// before it sends the next. Counting from the end of an ask, not at a fixed rate,
/// keeps asks from overlapping and a slow daemon from gathering a queue of them.
pub const VIEW_PERIOD: Duration = Duration::from_secs(1);

/// How long a front end waits for the daemon's answer to [`Command::View`] before
/// it counts the ask as failed. A local reply takes milliseconds and the daemon
/// answers requests one at a time, so no answer in this time means its loop is
/// busy or stuck.
pub const VIEW_DEADLINE: Duration = Duration::from_secs(5);

impl Command {
    /// The issue the command is about; `None` for `Status`.
    pub fn issue(&self) -> Option<&IssueId> {
        match self {
            Self::Status
            | Self::View
            | Self::Screen { .. }
            | Self::Set { .. }
            | Self::Unset { .. }
            | Self::Settings { .. }
            | Self::Overturn { .. }
            | Self::Proposal { .. }
            | Self::File { .. }
            | Self::Investigated { .. }
            | Self::Hold
            | Self::Resume
            | Self::Stop { .. } => None,
            Self::Issue { issue } | Self::Start { issue } => Some(issue),
            Self::Plan { issue, .. }
            | Self::Ready { issue, .. }
            | Self::Review { issue, .. }
            | Self::Retry { issue }
            | Self::Rule { issue, .. }
            | Self::Choose { issue, .. } => Some(issue),
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

/// What a reviewer's verdict names besides approve or block, as the daemon
/// takes it.
struct Judged {
    /// The plan's checks that do not hold, counted from 1.
    failed: Vec<usize>,
    /// The points of the round before that stay open, counted from 1.
    open: Vec<usize>,
    /// Those points' text, in that order, for the block to end with.
    kept: Vec<String>,
}

impl World {
    /// The events a command becomes, in the order they are applied: a plan's
    /// settings come before the plan, so they hold when its author starts.
    pub fn events(&self, by: Option<&AgentId>, command: &Command) -> Result<Vec<Event>, String> {
        let Some(event) = self.check(by, command)? else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        // The plan keeps what the refiner wrote; its settings are recorded under
        // the keys they are bound to.
        if let Event::PlanWritten { issue, plan } = &event {
            let who = by.map_or_else(|| PERSON.to_string(), |a| a.0.clone());
            let scope = Scope::Task(issue.clone());
            for (key, value) in &plan.settings {
                let role = settings::recorded_role(None, key, &who);
                out.push(Event::SettingSet {
                    scope: scope.clone(),
                    role,
                    key: self.model_key(&scope, role, key)?,
                    value: value.clone(),
                    by: who.clone(),
                });
            }
        }
        // An overturn proposes a rule, once for the decision.
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

    /// The author's replies `ready` sends for `issue`, trimmed, or why they are
    /// refused. The first ready after a block
    /// ([`crate::model::Issue::block_to_answer`]) carries exactly one reply for
    /// each of its points, none of them empty, so every point it answers has its
    /// reply on record; any other ready carries none.
    fn replies(&self, issue: &IssueId, replies: &[String]) -> Result<Vec<String>, String> {
        let replies: Vec<String> = replies
            .iter()
            .map(|r| clean(r).trim().to_string())
            .collect();
        let Some(block) = self.issues.get(issue).and_then(|i| i.block_to_answer()) else {
            if replies.is_empty() {
                return Ok(replies);
            }
            return Err(format!(
                "{issue} has no review point to reply to: replies answer the points of the block \
                 that sent the change back, once, with the first ready after it. Run temet-work \
                 ready {issue} without --reply"
            ));
        };
        if replies.len() == block.points.len() && !replies.iter().any(String::is_empty) {
            return Ok(replies);
        }
        let got = if replies.iter().any(String::is_empty) {
            "one of its replies is empty".to_string()
        } else {
            match replies.len() {
                0 => "it has none".to_string(),
                1 => "it has 1 reply".to_string(),
                n => format!("it has {n} replies"),
            }
        };
        Err(format!(
            "this ready answers review round {round} of {issue}, so it needs exactly one reply for \
             each of the round's points below, in this order, saying what you did about it or why \
             you did not; {got}:\n{points}Run: temet-work ready {issue} {flags}\n\
             In the MCP ready tool, give replies with one entry for each point, in this order.",
            round = block.round,
            points = numbered(&block.points),
            flags = reply_flags(block.points.len()),
        ))
    }

    /// The `review` command's `failed` and `open` for `issue` in review round
    /// `round`, sorted and each named once, with the points they keep open; or
    /// why they are refused: either on an approve, a check the plan does not
    /// have, a point in round 1, or one the round before does not have.
    fn judged(
        &self,
        issue: &IssueId,
        round: u8,
        verdict: &Verdict,
        failed: &[usize],
        open: &[usize],
    ) -> Result<Judged, String> {
        let tidy = |list: &[usize]| {
            let mut list = list.to_vec();
            list.sort_unstable();
            list.dedup();
            list
        };
        let (failed, open) = (tidy(failed), tidy(open));
        if matches!(verdict, Verdict::Approve) {
            if !failed.is_empty() {
                return Err(format!(
                    "an approve passes every check of {issue}; to name a check that does not hold \
                     with --failed, block"
                ));
            }
            if !open.is_empty() {
                return Err(
                    "an approve resolves every point of the round before; to keep a point of it \
                     open with --open, block"
                        .into(),
                );
            }
        }
        let i = self.issues.get(issue);
        let checks = i
            .and_then(|i| i.plan.as_ref())
            .map_or(0, |p| p.checks.len());
        if let Some(n) = failed.iter().find(|n| **n == 0 || **n > checks) {
            return Err(match checks {
                0 => format!("{issue}'s plan has no checks, so --failed {n} names none"),
                1 => format!("{issue}'s plan has 1 check, numbered 1, not {n}"),
                _ => format!("{issue}'s plan has checks 1 to {checks}, not {n}"),
            });
        }
        let before = i.and_then(|i| i.round_before(round));
        let kept = match (open.first(), before) {
            (None, _) => Vec::new(),
            (Some(_), _) if round <= 1 => {
                return Err(format!(
                    "review round 1 of {issue} has no round before it, so --open names no point; \
                     put what is wrong in the block's points"
                ));
            }
            (Some(n), None) => {
                return Err(format!(
                    "review round {round} of {issue} has no round before it on record, so --open \
                     {n} names no point"
                ));
            }
            (Some(_), Some(before)) => {
                let count = before.points.len();
                if let Some(n) = open.iter().find(|n| **n == 0 || **n > count) {
                    return Err(match count {
                        0 => format!(
                            "review round {} of {issue} raised no points, so --open {n} names none",
                            before.round
                        ),
                        1 => format!(
                            "review round {} of {issue} has 1 point, numbered 1, not {n}",
                            before.round
                        ),
                        _ => format!(
                            "review round {} of {issue} has points 1 to {count}, not {n}",
                            before.round
                        ),
                    });
                }
                open.iter().map(|n| before.points[n - 1].clone()).collect()
            }
        };
        Ok(Judged { failed, open, kept })
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

    /// The decision a `choose` from `by` records, or why it is refused. Only an
    /// agent this daemon started may record a choice, and only on the issue it
    /// works on, whatever its role. A person is refused: a person decides
    /// through settings and overturns, and a choice recorded as a person's would
    /// have that person overturn themselves. The question, the options and the
    /// option taken are each kept on one line, so that `status` lists the
    /// decision on one; the reason keeps its line feeds, as a ruling's does.
    /// The point is the issue's merge, the only point the loop has.
    fn choice(&self, by: Option<&AgentId>, command: &Command) -> Result<Decision, String> {
        let Command::Choose {
            issue,
            question,
            options,
            took,
            why,
            cost,
        } = command
        else {
            return Err(format!("{} is not choose", command.name()));
        };
        let Some(by) = by else {
            return Err(format!(
                "choose for {issue} is an agent's: a person decides through settings and \
                 overturns, and a choice recorded as yours would have you overturn yourself"
            ));
        };
        let Some(agent) = self.agents.get(by) else {
            return Err(format!(
                "choose for {issue} is an agent's that works on it: {by} is not an agent this \
                 daemon started"
            ));
        };
        if agent.issue.as_ref() != Some(issue) {
            let holds = agent.issue.as_ref().map_or_else(
                || "works on no issue".to_string(),
                |i| format!("works on {i}"),
            );
            return Err(format!(
                "choose for {issue} is for the agents that work on it: {by} {holds}; record a \
                 choice on the issue you work on"
            ));
        }
        if !self.issues.contains_key(issue) {
            return Err(format!("there is no issue {issue}"));
        }
        let one_line = |text: &str| clean(text).split_whitespace().collect::<Vec<_>>().join(" ");
        let question = one_line(question);
        if question.is_empty() {
            return Err("a choice needs its question: what you decided, in a line".into());
        }
        let options: Vec<String> = options.iter().map(|o| one_line(o)).collect();
        if options.len() < 2 {
            return Err(format!(
                "a choice needs at least two options, since overturning it takes another; this \
                 one has {}",
                options.len()
            ));
        }
        for (n, option) in options.iter().enumerate() {
            if option.is_empty() {
                return Err(format!(
                    "option {} is empty; give each option its text",
                    n + 1
                ));
            }
            if let Some(first) = options[..n].iter().position(|o| o == option) {
                return Err(format!(
                    "option {} repeats option {} ({option}); give each option once",
                    n + 1,
                    first + 1
                ));
            }
        }
        let took = one_line(took);
        let Some(chosen) = options.iter().position(|o| *o == took) else {
            let listed: Vec<String> = options
                .iter()
                .enumerate()
                .map(|(n, o)| format!("{}={o}", n + 1))
                .collect();
            return Err(format!(
                "the option taken, \"{took}\", is not among the options ({}); give the one you \
                 took as written there",
                listed.join(", ")
            ));
        };
        let reason = clean(why).trim().to_string();
        if reason.is_empty() {
            return Err("a choice needs its reason: why you took that option".into());
        }
        let given = cost.as_deref().map(str::trim);
        let cost = match given.map(str::to_ascii_lowercase).as_deref() {
            Some("low") => Cost::Low,
            Some("medium") => Cost::Medium,
            Some("high") => Cost::High,
            None | Some("") => {
                return Err(
                    "a choice needs its cost to undo: low, medium or high, so that every \
                     decision a person reads can be ordered by what overturning it costs"
                        .into(),
                );
            }
            Some(_) => {
                return Err(format!(
                    "{} is not a cost to undo; give low, medium or high",
                    given.unwrap_or_default()
                ));
            }
        };
        let id = self.decisions.keys().next_back().map_or(1, |d| d.0 + 1);
        Ok(Decision {
            id: DecisionId(id),
            issue: issue.clone(),
            by: by.clone(),
            kind: DecisionKind::Choice,
            question,
            options,
            chosen,
            reason,
            overturned_to: None,
            overturn_reason: String::new(),
            cost: Some(cost),
            point: Some(Point::Merge),
        })
    }

    /// The investigator `by` names, with its group, when it runs its group's
    /// investigation; why `step` is not `by`'s otherwise.
    fn investigator(
        &self,
        by: Option<&AgentId>,
        step: &str,
    ) -> Result<&crate::model::Group, String> {
        let Some(by) = by else {
            return Err(format!(
                "{step} is a group's investigator's: a person may not run it; a person files \
                 issues in the tracker itself"
            ));
        };
        let agent = self.agents.get(by).ok_or_else(|| {
            format!("{step} is an investigator's: {by} is not an agent this daemon started")
        })?;
        if agent.role != Role::Investigator {
            return Err(format!(
                "{step} is an investigator's: {by} is the {} of {}",
                agent.role.name(),
                agent
                    .issue
                    .as_ref()
                    .map_or_else(|| agent.group.to_string(), ToString::to_string)
            ));
        }
        match self.investigating(&agent.group) {
            Some(i) if i.agent == *by && !i.done => self
                .groups
                .get(&agent.group)
                .ok_or_else(|| format!("there is no group {}", agent.group)),
            _ => Err(format!(
                "{step} is the running investigator's: {by}'s investigation of {} is over",
                agent.group
            )),
        }
    }

    /// The effect a `file` command becomes: an issue to file in the tracker of
    /// `project`, for the investigator `by`. Refused for anyone but a group's
    /// running investigator, for a project or area its group does not take issues
    /// from, for a milestone the group does not take, and for a title that an
    /// issue of that project already has, open or closed by a person (the
    /// plainest duplicate; judging whether one covers it otherwise is the
    /// investigator's). The milestone is the last of the group's when none is
    /// named, and none for a group that names none.
    pub fn filing(&self, by: Option<&AgentId>, command: &Command) -> Result<Effect, String> {
        let Command::File {
            project,
            title,
            body,
            milestone,
        } = command
        else {
            return Err(format!("{} is not file", command.name()));
        };
        let group = self.investigator(by, "file")?;
        let title = title.trim();
        if title.is_empty() {
            return Err("an issue needs a title".into());
        }
        let sources = self.group_sources(group);
        if !sources.contains(project) {
            let list: Vec<String> = sources.iter().map(ToString::to_string).collect();
            return Err(format!(
                "{} takes no issues from {project}; it files in: {}",
                group.id,
                if list.is_empty() {
                    "none".into()
                } else {
                    list.join(", ")
                }
            ));
        }
        let milestone = match milestone
            .as_deref()
            .map(str::trim)
            .filter(|m| !m.is_empty())
        {
            Some(m) if group.milestones.is_empty() || group.milestones.iter().any(|g| g.0 == m) => {
                Some(MilestoneId::new(m))
            }
            Some(m) => {
                let list: Vec<String> = group.milestones.iter().map(ToString::to_string).collect();
                return Err(format!(
                    "{} does not take milestone {m}; it takes: {}",
                    group.id,
                    list.join(", ")
                ));
            }
            None => self.filing_milestone(group),
        };
        let same = |t: &str| t.trim().eq_ignore_ascii_case(title);
        let titled = || {
            self.issues
                .values()
                .filter(|i| &i.id.project == project && same(&i.title))
        };
        if let Some(open) = titled().find(|i| i.stage != Stage::Done) {
            return Err(format!(
                "{} is open with the title \"{}\" already; file an issue no open one covers",
                open.id, open.title
            ));
        }
        if let Some(closed) = titled().find(|i| i.closed_by_a_person()) {
            return Err(format!(
                "{} was closed by a person with the title \"{}\"; do not file it again: a person \
                 who wants it built reopens it",
                closed.id, closed.title
            ));
        }
        Ok(Effect::File {
            group: group.id.clone(),
            agent: by.cloned().unwrap_or_else(|| AgentId::new(PERSON)),
            source: project.clone(),
            title: title.to_string(),
            body: body.clone(),
            milestone,
        })
    }

    /// The event a command becomes, or why the loop refuses it. A command that
    /// only reads, such as `Status`, becomes no event.
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
        // A person stopped it: whatever it still sends, as while its stop keeps
        // failing, moves nothing. It may still read.
        if let Some(by) = by
            && command.changes()
            && self.person_stops.contains_key(by)
        {
            return Err(format!(
                "{by} was stopped by a person, so it takes no further step; its session is \
                 being stopped"
            ));
        }
        let event = match command {
            Command::Status | Command::View | Command::Screen { .. } => return Ok(None),
            // Filing waits for the tracker's answer: the daemon carries it out
            // through `World::filing`, not as an event of its own.
            Command::File { .. } => {
                self.filing(by, command)?;
                return Ok(None);
            }
            Command::Investigated { group } => {
                let g = self.investigator(by, "investigated")?;
                if &g.id != group {
                    return Err(format!(
                        "investigated {group}: you investigate for {}, not {group}",
                        g.id
                    ));
                }
                Event::Investigated {
                    group: group.clone(),
                    agent: by.cloned().unwrap_or_else(|| AgentId::new(PERSON)),
                }
            }
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
                let who = by.map_or_else(|| PERSON.to_string(), |a| a.0.clone());
                let mut bound: std::collections::BTreeMap<String, &str> = Default::default();
                for (key, value) in &plan.settings {
                    let role = settings::recorded_role(None, key, &who);
                    let at = self
                        .may_set(by, &task, None, key)
                        .and_then(|()| settings::check_level(key, task.level()))
                        .and_then(|()| settings::check_value(key, value))
                        .and_then(|()| self.check_choice(key, value))
                        .and_then(|()| self.model_key(&task, role, key))
                        .map_err(|e| format!("the plan's settings: {e}"))?;
                    if let Some(other) = bound.insert(at.clone(), key) {
                        return Err(format!(
                            "the plan's settings: {other} and {key} are both {at}; give one"
                        ));
                    }
                }
                // Stored with its controls dropped and its line breaks line feeds.
                let mut plan = plan.clone();
                for c in &mut plan.checks {
                    c.done_when = clean(&c.done_when);
                    c.proven_by = clean(&c.proven_by);
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
                // No change has been checked yet.
                for c in &mut plan.checks {
                    c.state = Some(CheckState::Waiting);
                }
                Event::PlanWritten {
                    issue: issue.clone(),
                    plan,
                }
            }
            Command::Ready {
                issue,
                pull,
                pulls,
                replies,
            } => {
                self.may_step(by, "ready", Role::Author, issue)?;
                match at(issue)? {
                    Stage::Working | Stage::Fixing { .. } => {
                        let replies = self.replies(issue, replies)?;
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
                            replies,
                        }
                    }
                    now => return refuse(issue, now, "being worked on"),
                }
            }
            Command::Review {
                issue,
                verdict,
                failed,
                open,
            } => {
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
                let Judged { failed, open, kept } =
                    self.judged(issue, round, verdict, failed, open)?;
                let verdict = match verdict {
                    Verdict::Approve => Verdict::Approve,
                    // A point kept open joins the block, so the author fixes it
                    // again and the next round judges it again.
                    Verdict::Block { points } => {
                        let mut points: Vec<String> = points
                            .iter()
                            .map(|p| clean(p))
                            .filter(|p| !p.trim().is_empty())
                            .collect();
                        let new = points.len();
                        points.extend(kept);
                        if points.is_empty() {
                            return Err(
                                "a block needs at least one point to fix, or a point of the \
                                 round before kept open with --open"
                                    .into(),
                            );
                        }
                        // The author replies to each point, and its brief spells
                        // out one --reply for each.
                        if points.len() > MOST_POINTS {
                            return Err(format!(
                                "a block takes at most {MOST_POINTS} points, new and kept open \
                                 together, so that the author's brief can list one reply for \
                                 each; this one has {} ({new} new, {} kept open): put related \
                                 points together in one",
                                points.len(),
                                points.len() - new
                            ));
                        }
                        Verdict::Block { points }
                    }
                };
                Event::ReviewFinished {
                    issue: issue.clone(),
                    round,
                    verdict,
                    failed: Some(failed),
                    open: Some(open),
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
                if role.is_some() {
                    settings::check_role(key, "--role")?;
                }
                self.may_set(by, &scope, *role, key)?;
                settings::check_level(key, scope.level())?;
                let value = settings::parse_value(key, value)?;
                self.check_choice(key, &value)?;
                let by = by.map_or_else(|| PERSON.to_string(), |a| a.0.clone());
                let role = settings::recorded_role(*role, key, &by);
                Event::SettingSet {
                    key: self.model_key(&scope, role, key)?,
                    scope,
                    role,
                    value,
                    by,
                }
            }
            Command::Unset { target, role, key } => {
                let scope = self.scope_of(target)?;
                if role.is_some() {
                    settings::check_role(key, "--role")?;
                }
                self.may_set(by, &scope, *role, key)?;
                settings::spec(key)?;
                let who = by.map_or_else(|| PERSON.to_string(), |a| a.0.clone());
                let at = settings::recorded_role(*role, key, &who);
                // A plain model is the global runtime's model now, as when it is set.
                let bound = self.bound_key(key);
                // A person's unset for every role also takes an agent's value for
                // the author away, so it has something to unset when only that is.
                let held = |key: &str| {
                    self.settings
                        .contains_key(&(scope.clone(), at, key.to_string()))
                        || (by.is_none() && at.is_none() && self.agents_author_value(&scope, key))
                };
                if !held(&bound) {
                    // A value left from a build that had the runtime can be unset.
                    self.check_model_runtime(key)?;
                    let for_role = at.map_or_else(String::new, |r| format!(" for {}", r.name()));
                    // A plain model set under another global runtime is under
                    // that runtime's key: say which, so it can be taken away.
                    let other = (bound != *key)
                        .then(|| {
                            self.settings
                                .keys()
                                .map(|(_, _, k)| k)
                                .filter(|k| k.starts_with("model.") && **k != bound)
                                .find(|k| held(k))
                        })
                        .flatten();
                    return Err(match other {
                        Some(other) => format!(
                            "a plain {key} is {bound} now, the global runtime's, and {bound} is \
                             not set at {target}{for_role}; {other} is: unset {other} to take it away"
                        ),
                        None if bound != *key => format!(
                            "a plain {key} is {bound} now, the global runtime's, and {bound} is \
                             not set at {target}{for_role}; nothing to unset"
                        ),
                        None => format!("{key} is not set at {target}{for_role}; nothing to unset"),
                    });
                }
                Event::SettingUnset {
                    scope,
                    role: at,
                    key: bound,
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
            Command::Hold => {
                if let Some(by) = by {
                    return Err(format!(
                        "hold is a person's: {by} is an agent; an agent finishes its step, and a \
                         person holds the daemon from a shell outside any agent's terminal"
                    ));
                }
                if self.held {
                    return Err(format!(
                        "the daemon is already held{}; temet-work resume lifts the hold",
                        self.held_since
                            .map(|at| format!(", since {}", crate::investigation::utc(at)))
                            .unwrap_or_default()
                    ));
                }
                Event::Held
            }
            Command::Resume => {
                if let Some(by) = by {
                    return Err(format!(
                        "resume is a person's: {by} is an agent, and only a person lifts a hold"
                    ));
                }
                if !self.held {
                    return Err("the daemon is not held; temet-work hold holds it".into());
                }
                Event::Resumed
            }
            Command::Stop { agent } => {
                if let Some(by) = by {
                    return Err(format!(
                        "stop is a person's: {by} is an agent; an agent finishes its step, and a \
                         person stops agents from a shell outside any agent's terminal"
                    ));
                }
                let Some(a) = self.agents.get(agent) else {
                    return Err(format!("{agent} is not an agent this daemon started"));
                };
                if let Some(at) = self.person_stops.get(agent) {
                    let since = at
                        .map(|at| format!(" at {}", crate::investigation::utc(at)))
                        .unwrap_or_default();
                    let then = match &a.issue {
                        Some(issue) if self.stopped_issues.contains_key(issue) => {
                            format!("; temet-work start {issue} lets agents start on {issue} again")
                        }
                        _ => String::new(),
                    };
                    return Err(format!(
                        "{agent} is already stopped by a person{since}{then}"
                    ));
                }
                if a.state == crate::model::AgentState::Ended {
                    return Err(format!(
                        "{agent} has ended: its session is over, so there is nothing to stop"
                    ));
                }
                Event::AgentStoppedByPerson {
                    agent: agent.clone(),
                }
            }
            Command::Start { issue } => {
                if let Some(by) = by {
                    return Err(format!(
                        "start is a person's: {by} is an agent, and only a person lets agents \
                         start again on an issue a person stopped"
                    ));
                }
                at(issue)?;
                if !self.stopped_issues.contains_key(issue) {
                    return Err(format!(
                        "{issue} is not stopped: no person has stopped an agent of it since it \
                         last started, so nothing holds its agents back; temet-work stop AGENT \
                         stops one"
                    ));
                }
                Event::IssueStartedByPerson {
                    issue: issue.clone(),
                }
            }
            // A choice moves no work: its agent carries on at once.
            Command::Choose { .. } => Event::DecisionMade {
                decision: self.choice(by, command)?,
            },
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
                let reason = clean(reason);
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
                let reason = clean(reason);
                if reason.trim().is_empty() {
                    return Err("a ruling needs a reason".into());
                }
                let id = self.decisions.keys().next_back().map_or(1, |d| d.0 + 1);
                let (cost, point) = Decision::ruling_cost(*approve);
                Event::DecisionMade {
                    decision: Decision {
                        id: DecisionId(id),
                        issue: issue.clone(),
                        by: by.cloned().unwrap_or_else(|| AgentId::new(PERSON)),
                        kind: DecisionKind::Ruling { approve: *approve },
                        question: format!("Does {issue} merge after {round} review rounds?"),
                        options: vec!["merge".into(), "fix first".into()],
                        chosen: usize::from(!approve),
                        reason,
                        overturned_to: None,
                        overturn_reason: String::new(),
                        cost: Some(cost),
                        point: Some(point),
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
                state: None,
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
                queue_low: None,
                investigate_every: None,
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
            runtime: None,
            model: None,
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

    /// A ready from ankh#1's author, with a reply to each point of the block it
    /// answers.
    fn ready_replying(w: &World) -> Command {
        let replies = w.issues[&issue()]
            .block_to_answer()
            .map_or_else(Vec::new, |b| {
                b.points.iter().map(|p| format!("fixed: {p}")).collect()
            });
        Command::Ready {
            issue: issue(),
            pull: Some("57".into()),
            pulls: Default::default(),
            replies,
        }
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

    fn stop(agent: &str) -> Command {
        Command::Stop {
            agent: AgentId::new(agent),
        }
    }

    #[test]
    fn stop_is_taken_from_a_person_as_an_event_of_its_own_and_refused_when_it_changes_nothing() {
        let mut w = refining();
        let why = |w: &World, c: &Command| w.events(None, c).expect_err("refused");
        let stopped = Event::AgentStoppedByPerson {
            agent: AgentId::new("author-1"),
        };
        assert_eq!(w.events(None, &stop("author-1")), Ok(vec![stopped.clone()]));
        assert_eq!(
            why(&w, &stop("author-9")),
            "author-9 is not an agent this daemon started"
        );
        w.apply(&Event::AgentChanged {
            agent: AgentId::new("reviewer-1"),
            state: crate::model::AgentState::Ended,
            lost: false,
        });
        assert_eq!(
            why(&w, &stop("reviewer-1")),
            "reviewer-1 has ended: its session is over, so there is nothing to stop"
        );
        let at = 1_795_000_000_000;
        w.apply_at(at, &stopped);
        assert_eq!(
            why(&w, &stop("author-1")),
            format!(
                "author-1 is already stopped by a person at {}; temet-work start ankh#1 lets \
                 agents start on ankh#1 again",
                crate::investigation::utc(at)
            )
        );
        // Still refused once its issue has started again.
        w.apply(&Event::IssueStartedByPerson { issue: issue() });
        assert_eq!(
            why(&w, &stop("author-1")),
            format!(
                "author-1 is already stopped by a person at {}",
                crate::investigation::utc(at)
            )
        );
    }

    #[test]
    fn start_is_taken_on_an_issue_a_person_stopped_and_refused_on_any_other() {
        let mut w = refining();
        let start = Command::Start { issue: issue() };
        assert_eq!(
            w.events(None, &start).expect_err("refused"),
            "ankh#1 is not stopped: no person has stopped an agent of it since it last started, \
             so nothing holds its agents back; temet-work stop AGENT stops one"
        );
        let unknown = Command::Start {
            issue: IssueId::new("ankh", "404"),
        };
        assert_eq!(
            w.events(None, &unknown).expect_err("refused"),
            "there is no issue ankh#404"
        );
        w.apply(&Event::AgentStoppedByPerson {
            agent: AgentId::new("refiner-1"),
        });
        assert_eq!(
            w.events(None, &start),
            Ok(vec![Event::IssueStartedByPerson { issue: issue() }])
        );
    }

    #[test]
    fn stop_and_start_are_a_persons_and_an_agent_is_refused_with_its_own_secret() {
        let mut w = refining();
        let author = AgentId::new("author-1");
        let why = w
            .events(Some(&author), &stop("reviewer-1"))
            .expect_err("refused");
        assert_eq!(
            why,
            "stop is a person's: author-1 is an agent; an agent finishes its step, and a person \
             stops agents from a shell outside any agent's terminal"
        );
        let why = w
            .events(Some(&author), &stop("author-1"))
            .expect_err("refused");
        assert!(
            why.starts_with("stop is a person's: author-1 is an agent"),
            "{why}"
        );
        w.apply(&Event::AgentStoppedByPerson {
            agent: AgentId::new("refiner-1"),
        });
        let why = w
            .events(Some(&author), &Command::Start { issue: issue() })
            .expect_err("refused");
        assert!(
            why.starts_with("start is a person's: author-1 is an agent"),
            "{why}"
        );
    }

    #[test]
    fn an_agent_a_person_stopped_takes_no_further_step_but_may_read() {
        let mut w = refining();
        let refiner = AgentId::new("refiner-1");
        w.apply(&Event::AgentStoppedByPerson {
            agent: refiner.clone(),
        });
        let plan = Command::Plan {
            issue: issue(),
            plan: plan(),
        };
        assert_eq!(
            w.events(Some(&refiner), &plan).expect_err("refused"),
            "refiner-1 was stopped by a person, so it takes no further step; its session is \
             being stopped"
        );
        assert_eq!(w.events(Some(&refiner), &Command::Status), Ok(vec![]));
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
                replies: Vec::new(),
            },
        )
        .expect("ready");
        let approve = Command::Review {
            issue: issue(),
            verdict: Verdict::Approve,
            failed: Vec::new(),
            open: Vec::new(),
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
                    pulls: Default::default(),
                    replies: Vec::new(),
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
                    replies: Vec::new(),
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
    fn view_reads_back_from_its_line_and_becomes_no_event() {
        let line = r#"{"command":"view"}"#;
        assert_eq!(serde_json::to_string(&Command::View).expect("json"), line);
        assert_eq!(
            serde_json::from_str::<Command>(line).expect("a command"),
            Command::View
        );
        let w = refining();
        for by in [None, Some(AgentId::new("author-1"))] {
            assert_eq!(w.events(by.as_ref(), &Command::View), Ok(Vec::new()));
        }
        assert!(!Command::View.changes());
        assert_eq!(Command::View.issue(), None);
        assert_eq!(Command::View.name(), "view");
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
            replies: Vec::new(),
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
                replies: Vec::new(),
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
            failed: Vec::new(),
            open: Vec::new(),
        };
        run(&mut w, block).expect("block");
        let Command::Ready {
            issue, pull, pulls, ..
        } = bare
        else {
            unreachable!("a ready")
        };
        let replied = Command::Ready {
            issue,
            pull,
            pulls,
            replies: vec!["added one that stops the clock".into()],
        };
        run(&mut w, replied).expect("the pull request is on record");
    }

    #[test]
    fn ready_puts_the_authors_replies_on_the_review_round_its_fix_answers() {
        let mut w = refining();
        run(
            &mut w,
            Command::Plan {
                issue: issue(),
                plan: plan(),
            },
        )
        .expect("plan");
        let ready = |replies: &[&str]| Command::Ready {
            issue: issue(),
            pull: Some("57".into()),
            pulls: Default::default(),
            replies: replies.iter().map(ToString::to_string).collect(),
        };
        let refused = |w: &World, replies: &[&str]| {
            let command = ready(replies);
            w.check(owner(&command).as_ref(), &command)
                .expect_err("refused")
        };
        // The first pull request answers no block.
        assert!(refused(&w, &["early"]).contains("no review point to reply to"));
        run(&mut w, ready(&[])).expect("ready");
        run(
            &mut w,
            Command::Review {
                issue: issue(),
                verdict: Verdict::Block {
                    points: vec!["no test for a stopped clock".into()],
                },
                failed: Vec::new(),
                open: Vec::new(),
            },
        )
        .expect("block");
        assert!(refused(&w, &[" "]).contains("one of its replies is empty"));
        run(&mut w, ready(&[" added one that stops the clock "])).expect("ready");
        assert_eq!(
            w.issues[&issue()].reviews[0].replies,
            ["added one that stops the clock"]
        );
        // A failed check sends it back for the same round: that fix answers no
        // review round.
        w.apply(&Event::ChecksFinished {
            issue: issue(),
            passed: false,
            failing: vec!["lint".into()],
        });
        assert!(refused(&w, &["fixed the lint"]).contains("no review point to reply to"));
        run(&mut w, ready(&[])).expect("ready without replies");
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
                replies: Vec::new(),
            },
        )
        .expect("ready");
        // Round 1 is up and no reviewer has heard of it yet.
        let approve = Command::Review {
            issue: issue(),
            verdict: Verdict::Approve,
            failed: Vec::new(),
            open: Vec::new(),
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
                    replies: Vec::new(),
                },
            ),
            (
                "review",
                Role::Reviewer,
                Command::Review {
                    issue: issue(),
                    verdict: Verdict::Approve,
                    failed: Vec::new(),
                    open: Vec::new(),
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
                    pulls: Default::default(),
                    replies: Vec::new(),
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
                replies: Vec::new(),
            },
            Command::Review {
                issue: issue(),
                verdict: Verdict::Approve,
                failed: Vec::new(),
                open: Vec::new(),
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
            let ready = ready_replying(&w);
            run(&mut w, ready).expect("ready");
            let block = Command::Review {
                issue: issue(),
                verdict: Verdict::Block {
                    points: vec!["x".into()],
                },
                failed: Vec::new(),
                open: Vec::new(),
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
            let ready = ready_replying(&w);
            run(&mut w, ready).expect("ready");
            let block = Command::Review {
                issue: issue(),
                verdict: Verdict::Block {
                    points: vec!["no test for a stopped clock".into()],
                },
                failed: Vec::new(),
                open: Vec::new(),
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

    /// Text holding the controls the intake drops and the line breaks it makes
    /// line feeds.
    pub(super) const NOISY: &str =
        "a\u{b}b\u{c}c\u{8}d\u{1b}[1Ee\0f\u{7f}g\u{85}h\u{2028}i\u{2029}j\rk\r\nl\tm";

    /// [`NOISY`] as it is stored: only line feeds and tabs are left.
    pub(super) const STORED: &str = "abcd[1Eefg\nh\ni\nj\nk\nl\tm";

    /// The event `command` from its owner becomes on `w`, applied to it.
    fn logged(w: &mut World, command: &Command) -> Event {
        if matches!(command, Command::Review { .. }) {
            w.apply(&Event::Told { issue: issue() });
        }
        let event = w
            .check(owner(command).as_ref(), command)
            .expect("accepted")
            .expect("an event");
        w.apply(&event);
        event
    }

    fn block(points: &[&str]) -> Command {
        Command::Review {
            issue: issue(),
            verdict: Verdict::Block {
                points: points.iter().map(ToString::to_string).collect(),
            },
            failed: Vec::new(),
            open: Vec::new(),
        }
    }

    /// ankh#1 with its plan written and its first block, `points`, in.
    fn blocked(points: &[&str]) -> World {
        let mut w = refining();
        logged(
            &mut w,
            &Command::Plan {
                issue: issue(),
                plan: plan(),
            },
        );
        let ready = ready_replying(&w);
        logged(&mut w, &ready);
        logged(&mut w, &block(points));
        w
    }

    /// What author-1's choice on ankh#1 asks.
    const CLACKS: &str = "Which way does the clacks message go?";

    /// Why author-1 takes the option it takes.
    const NOON: &str = "the towers are up, and it reaches Pseudopolis by noon";

    /// A choice on ankh#1 between `options`, taking `took`, at `cost`.
    fn choice(
        question: &str,
        options: &[&str],
        took: &str,
        why: &str,
        cost: Option<&str>,
    ) -> Command {
        Command::Choose {
            issue: issue(),
            question: question.into(),
            options: options.iter().map(ToString::to_string).collect(),
            took: took.into(),
            why: why.into(),
            cost: cost.map(String::from),
        }
    }

    /// The clacks, the pigeon or the Post Office.
    const WAYS: [&str; 3] = ["clacks", "pigeon", "Post Office"];

    /// author-1's choice of the pigeon, at `cost`.
    fn pigeon(cost: Option<&str>) -> Command {
        choice(CLACKS, &WAYS, "pigeon", NOON, cost)
    }

    /// ankh#1 being worked on by author-1, its plan written.
    fn working() -> World {
        let mut w = refining();
        logged(
            &mut w,
            &Command::Plan {
                issue: issue(),
                plan: plan(),
            },
        );
        assert_eq!(w.issues[&issue()].stage, Stage::Working);
        w
    }

    #[test]
    fn an_agents_choice_is_one_decision_numbered_after_the_others() {
        let mut w = working();
        // Decision 4 is on record already.
        let Event::DecisionMade { decision: before } = working()
            .events(Some(&AgentId::new("refiner-1")), &pigeon(Some("low")))
            .expect("a refiner may choose")
            .remove(0)
        else {
            panic!("not a decision");
        };
        w.apply(&Event::DecisionMade {
            decision: Decision {
                id: DecisionId(4),
                ..before
            },
        });
        let events = w
            .events(Some(&AgentId::new("author-1")), &pigeon(Some("medium")))
            .expect("accepted");
        assert_eq!(
            events,
            [Event::DecisionMade {
                decision: Decision {
                    id: DecisionId(5),
                    issue: issue(),
                    by: AgentId::new("author-1"),
                    kind: DecisionKind::Choice,
                    question: CLACKS.into(),
                    options: WAYS.iter().map(ToString::to_string).collect(),
                    chosen: 1,
                    reason: NOON.into(),
                    overturned_to: None,
                    overturn_reason: String::new(),
                    cost: Some(Cost::Medium),
                    point: Some(Point::Merge),
                },
            }]
        );
    }

    #[test]
    fn a_choice_moves_no_work_and_its_agent_carries_on() {
        let mut w = working();
        let author = AgentId::new("author-1");
        let (issue_before, agents_before) = (w.issues[&issue()].clone(), w.agents.clone());
        let mut effects = Vec::new();
        for e in w
            .events(Some(&author), &pigeon(Some("medium")))
            .expect("accepted")
        {
            effects.extend(w.apply(&e));
        }
        // No stop, no fix, no merge: the arbiter on ankh#1 is left running too.
        assert_eq!(effects, []);
        assert_eq!(w.decisions.len(), 1);
        assert_eq!(w.issues[&issue()], issue_before);
        assert_eq!(w.issues[&issue()].stage, Stage::Working);
        assert_eq!(w.agents, agents_before);
        let ready = ready_replying(&w);
        let events = w.events(Some(&author), &ready).expect("ready is accepted");
        assert!(
            matches!(events.as_slice(), [Event::PullRequestReady { .. }]),
            "{events:?}"
        );
    }

    #[test]
    fn a_choice_is_refused_with_the_reason_and_none_is_recorded() {
        let w = working();
        let author = AgentId::new("author-1");
        let mut refused = vec![
            (
                pigeon(None),
                "a choice needs its cost to undo: low, medium or high",
            ),
            (
                pigeon(Some(" ")),
                "a choice needs its cost to undo: low, medium or high",
            ),
            (
                pigeon(Some("huge")),
                "huge is not a cost to undo; give low, medium or high",
            ),
            (
                choice(CLACKS, &WAYS, "camel", NOON, Some("low")),
                "the option taken, \"camel\", is not among the options (1=clacks, 2=pigeon, 3=Post \
                 Office); give the one you took as written there",
            ),
            (
                choice(" \n", &WAYS, "pigeon", NOON, Some("low")),
                "a choice needs its question",
            ),
            (
                choice(CLACKS, &WAYS, "pigeon", "  ", Some("low")),
                "a choice needs its reason",
            ),
            (
                choice(CLACKS, &["clacks"], "clacks", NOON, Some("low")),
                "a choice needs at least two options, since overturning it takes another; this \
                 one has 1",
            ),
            (
                choice(
                    CLACKS,
                    &["clacks", " ", "pigeon"],
                    "clacks",
                    NOON,
                    Some("low"),
                ),
                "option 2 is empty",
            ),
            (
                choice(
                    CLACKS,
                    &["clacks", "pigeon", "clacks"],
                    "clacks",
                    NOON,
                    Some("low"),
                ),
                "option 3 repeats option 1 (clacks)",
            ),
        ];
        // A missing question, options, option taken or reason reads as empty.
        refused.push((
            serde_json::from_str(r#"{"command":"choose","issue":{"project":"ankh","key":"1"}}"#)
                .expect("a bare choice reads"),
            "a choice needs its question",
        ));
        for (command, want) in refused {
            let why = w.events(Some(&author), &command).expect_err("refused");
            assert!(why.contains(want), "{why}");
        }
        assert!(w.decisions.is_empty());
        // The words of the cost are read in any case, and every text without
        // its surrounding space.
        let events = w
            .events(
                Some(&author),
                &choice(
                    " Which way?\n",
                    &[" clacks", "pigeon "],
                    "pigeon",
                    " noon ",
                    Some("HIGH"),
                ),
            )
            .expect("accepted");
        assert!(matches!(
            events.as_slice(),
            [Event::DecisionMade { decision }]
                if decision.question == "Which way?"
                    && decision.options == ["clacks", "pigeon"]
                    && decision.chosen == 1
                    && decision.reason == "noon"
                    && decision.cost == Some(Cost::High)
        ));
    }

    #[test]
    fn only_an_agent_that_works_on_the_issue_records_a_choice_on_it() {
        let w = working();
        let command = pigeon(Some("medium"));
        let why = |by: Option<&str>| {
            w.events(by.map(AgentId::new).as_ref(), &command)
                .expect_err("refused")
        };
        assert_eq!(
            why(Some("author-2")),
            "choose for ankh#1 is for the agents that work on it: author-2 works on ankh#2; \
             record a choice on the issue you work on"
        );
        assert_eq!(
            why(Some("author-9")),
            "choose for ankh#1 is an agent's that works on it: author-9 is not an agent this \
             daemon started"
        );
        let person = why(None);
        assert!(
            person.starts_with(
                "choose for ankh#1 is an agent's: a person decides through settings \
                                and overturns"
            ),
            "{person}"
        );
        // Any role on the issue may.
        for agent in ["refiner-1", "author-1", "reviewer-1", "arbiter-1"] {
            assert!(
                w.events(Some(&AgentId::new(agent)), &command).is_ok(),
                "{agent}"
            );
        }
    }

    #[test]
    fn a_plans_checks_are_stored_with_their_controls_dropped() {
        let mut w = refining();
        let mut noisy = plan();
        noisy.checks[0] = Check::new(NOISY, NOISY);
        let Event::PlanWritten { plan, .. } = logged(
            &mut w,
            &Command::Plan {
                issue: issue(),
                plan: noisy,
            },
        ) else {
            panic!("not a plan");
        };
        assert_eq!(plan.checks[0].done_when, STORED);
        assert_eq!(plan.checks[0].proven_by, STORED);
    }

    #[test]
    fn a_blocks_points_are_stored_with_their_controls_dropped() {
        let w = blocked(&[NOISY]);
        assert_eq!(w.issues[&issue()].reviews[0].points, [STORED]);
    }

    #[test]
    fn a_block_takes_at_most_the_points_the_authors_brief_can_list_replies_for() {
        let many = |n: usize| -> Vec<String> { (1..=n).map(|i| format!("p{i}")).collect() };
        fn refs(points: &[String]) -> Vec<&str> {
            points.iter().map(String::as_str).collect()
        }
        // One over, new: refused, naming the most.
        let mut w = blocked(&["the clock stops"]);
        let ready = ready_replying(&w);
        logged(&mut w, &ready);
        w.apply(&Event::Told { issue: issue() });
        let over = block(&refs(&many(MOST_POINTS + 1)));
        let why = w
            .check(owner(&over).as_ref(), &over)
            .expect_err("over the most");
        assert!(
            why.contains("a block takes at most 300 points, new and kept open together"),
            "{why}"
        );
        assert!(
            why.contains("this one has 301 (301 new, 0 kept open)"),
            "{why}"
        );

        // The most is taken.
        let w = blocked(&refs(&many(MOST_POINTS)));
        assert_eq!(w.issues[&issue()].reviews[0].points.len(), MOST_POINTS);

        // Kept points count with the new ones.
        let mut w = w;
        let ready = ready_replying(&w);
        logged(&mut w, &ready);
        w.apply(&Event::Told { issue: issue() });
        let kept = Command::Review {
            issue: issue(),
            verdict: Verdict::Block {
                points: vec!["the clock is slow".into()],
            },
            failed: Vec::new(),
            open: (1..=MOST_POINTS).collect(),
        };
        let why = w
            .check(owner(&kept).as_ref(), &kept)
            .expect_err("over the most with kept points");
        assert!(
            why.contains("this one has 301 (1 new, 300 kept open)"),
            "{why}"
        );
    }

    #[test]
    fn a_ready_s_replies_are_stored_with_their_controls_dropped() {
        let mut w = blocked(&["the clock stops"]);
        let Event::PullRequestReady { replies, .. } = logged(
            &mut w,
            &Command::Ready {
                issue: issue(),
                pull: None,
                pulls: Default::default(),
                replies: vec![NOISY.into()],
            },
        ) else {
            panic!("not a ready");
        };
        assert_eq!(replies, [STORED]);
    }

    #[test]
    fn a_rulings_reason_is_stored_with_its_controls_dropped() {
        let mut w = blocked(&["the clock stops"]);
        for _ in 0..2 {
            let ready = ready_replying(&w);
            logged(&mut w, &ready);
            logged(&mut w, &block(&["the clock is slow"]));
        }
        assert_eq!(w.issues[&issue()].stage, Stage::Arbitration { round: 3 });
        let Event::DecisionMade { decision } = logged(
            &mut w,
            &Command::Rule {
                issue: issue(),
                approve: false,
                reason: NOISY.into(),
            },
        ) else {
            panic!("not a ruling");
        };
        assert_eq!(decision.reason, STORED);
        // It is the author's point to fix.
        assert_eq!(w.issues[&issue()].fix, [STORED]);
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
                queue_low: None,
                investigate_every: None,
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
                runtime: None,
                model: None,
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
                    state: None,
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

    /// ankh#1 being refined, in a build with the runtimes alpha (its default)
    /// and beta, and alpha set globally by a person.
    fn on_alpha() -> World {
        let mut w = refining();
        w.catalog.insert(
            "runtime".into(),
            crate::settings::Choice {
                key: "runtime".into(),
                allowed: vec!["alpha".into(), "beta".into()],
                default: Some("alpha".into()),
            },
        );
        run(&mut w, None, &set("global", None, "runtime", "alpha"));
        w
    }

    #[test]
    fn a_plain_model_is_logged_under_the_global_runtimes_key_when_it_is_written() {
        let mut w = on_alpha();
        let events = run(&mut w, None, &set("ankh#1", None, "model", "opus"));
        assert_eq!(
            events,
            [Event::SettingSet {
                scope: Scope::Task(issue()),
                role: None,
                key: "model.alpha".into(),
                value: Value::Text("opus".into()),
                by: PERSON.into(),
            }]
        );
        // A refiner's plan: its model is the author's model.alpha, and the plan
        // keeps what the refiner wrote.
        let mut w = on_alpha();
        let command = plan(&[("model", Value::Text("opus".into()))]);
        let events = run(&mut w, Some("refiner-1"), &command);
        assert_eq!(
            events[0],
            Event::SettingSet {
                scope: Scope::Task(issue()),
                role: Some(Role::Author),
                key: "model.alpha".into(),
                value: Value::Text("opus".into()),
                by: "refiner-1".into(),
            }
        );
        assert!(
            matches!(&events[1], Event::PlanWritten { plan, .. }
                if plan.settings.keys().eq(["model"])),
            "{events:?}"
        );
        // model and model.alpha are one key, so a plan gives one of them.
        let both = plan(&[
            ("model", Value::Text("opus".into())),
            ("model.alpha", Value::Text("haiku".into())),
        ]);
        let why = on_alpha()
            .check(Some(&AgentId::new("refiner-1")), &both)
            .expect_err("refused");
        assert!(why.contains("both model.alpha"), "{why}");
    }

    #[test]
    fn a_plain_model_is_refused_where_that_scope_and_role_run_another_runtime() {
        let mut w = on_alpha();
        run(
            &mut w,
            None,
            &set("global", Some(Role::Reviewer), "runtime", "beta"),
        );
        let why = w
            .events(None, &set("ankh", Some(Role::Reviewer), "model", "gpt-x"))
            .expect_err("refused");
        assert!(why.contains("model.beta"), "{why}");
        // For every role it is alpha's, which the reviewers do not read.
        let events = run(&mut w, None, &set("ankh", None, "model", "haiku"));
        assert!(
            matches!(&events[..], [Event::SettingSet { key, role: None, .. }] if key == "model.alpha"),
            "{events:?}"
        );
        // A plan's model is the author's: refused where the author runs beta.
        let mut w = on_alpha();
        run(
            &mut w,
            None,
            &set("ankh", Some(Role::Author), "runtime", "beta"),
        );
        let before = w.clone();
        let refiner = AgentId::new("refiner-1");
        let command = plan(&[("model", Value::Text("opus".into()))]);
        let why = w.events(Some(&refiner), &command).expect_err("refused");
        assert!(
            why.starts_with("the plan's settings: ") && why.contains("model.beta"),
            "{why}"
        );
        // Its own runtime's key is taken, and nothing else was recorded.
        assert_eq!(w, before);
        let command = plan(&[("model.beta", Value::Text("gpt-x".into()))]);
        assert!(w.events(Some(&refiner), &command).is_ok());
    }

    #[test]
    fn a_persons_model_or_model_of_the_global_runtime_overturns_the_refiners() {
        for mine in ["model", "model.alpha"] {
            for role in [None, Some(Role::Author)] {
                let mut w = on_alpha();
                run(
                    &mut w,
                    Some("refiner-1"),
                    &set("ankh#1", None, "model", "opus"),
                );
                let events = run(&mut w, None, &set("ankh#1", role, mine, "haiku"));
                assert!(
                    matches!(&events[..], [Event::SettingSet { key, .. }] if key == "model.alpha"),
                    "one change, nothing else: {events:?}"
                );
                assert_eq!(
                    read(&w, Role::Author, "model"),
                    is("haiku", "task, set by person"),
                    "{mine} {role:?}"
                );
            }
            // An unset takes the refiner's value away, in one change.
            let mut w = on_alpha();
            run(
                &mut w,
                Some("refiner-1"),
                &set("ankh#1", None, "model", "opus"),
            );
            let unset = Command::Unset {
                target: Target::Issue(issue()),
                role: None,
                key: mine.into(),
            };
            let events = run(&mut w, None, &unset);
            assert!(
                matches!(&events[..], [Event::SettingUnset { key, role: None, .. }] if key == "model.alpha"),
                "{events:?}"
            );
            assert_eq!(read(&w, Role::Author, "model"), None, "{mine}");
        }
        // Another runtime's model is not the same model, so it takes nothing away.
        let mut w = on_alpha();
        run(
            &mut w,
            Some("refiner-1"),
            &set("ankh#1", None, "model", "opus"),
        );
        let events = run(&mut w, None, &set("ankh#1", None, "model.beta", "gpt-x"));
        assert_eq!(events.len(), 1, "{events:?}");
        assert_eq!(
            read(&w, Role::Author, "model"),
            is("opus", "task, set by refiner-1")
        );
    }

    #[test]
    fn an_unset_of_a_plain_model_names_the_model_key_that_is_set() {
        let mut w = on_alpha();
        run(&mut w, None, &set("ankh#1", None, "model", "opus"));
        run(&mut w, None, &set("global", None, "runtime", "beta"));
        let why = w.events(None, &unset(None)).expect_err("refused");
        assert!(
            why.contains("model.beta") && why.contains("model.alpha is: unset model.alpha"),
            "{why}"
        );
        let unset_alpha = Command::Unset {
            target: Target::Issue(issue()),
            role: None,
            key: "model.alpha".into(),
        };
        run(&mut w, None, &unset_alpha);
        let why = w.events(None, &unset(None)).expect_err("refused");
        assert!(why.ends_with("nothing to unset"), "{why}");
        // A refiner's value for the author is named to a person too.
        let mut w = on_alpha();
        run(
            &mut w,
            Some("refiner-1"),
            &set("ankh#1", None, "model", "opus"),
        );
        run(&mut w, None, &set("global", None, "runtime", "beta"));
        let why = w.events(None, &unset(None)).expect_err("refused");
        assert!(why.contains("model.alpha is"), "{why}");
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

    #[test]
    fn a_role_on_a_setting_not_read_per_role_is_refused() {
        let mut w = refining();
        let reviewer = Some(Role::Reviewer);
        let refiner = AgentId::new("refiner-1");
        let set_for = |role| set("ankh", role, "review_rounds", "1");
        let unset_for_reviewer = Command::Unset {
            target: Target::parse("ankh"),
            role: reviewer,
            key: "review_rounds".into(),
        };
        // A person and an agent are told the same, and no event comes of it.
        for by in [None, Some(&refiner)] {
            for command in [set_for(reviewer), unset_for_reviewer.clone()] {
                let why = w.events(by, &command).expect_err("refused");
                assert_eq!(
                    why,
                    "review_rounds is the same for every role, so it takes no --role; the \
                     settings that take a role are: runtime, model, runner, machine, \
                     permission, remind_after, reminders, starts"
                );
            }
        }
        let events = run(&mut w, None, &set_for(None));
        assert!(
            matches!(events.as_slice(), [Event::SettingSet { role: None, .. }]),
            "{events:?}"
        );
        // With a value for every role there, the reason is still the role.
        assert!(
            w.events(None, &unset_for_reviewer)
                .is_err_and(|e| e.contains("every role") && !e.contains("nothing to unset"))
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
        let per_role = settings::spec(key).expect("a setting").per_role;
        for role in roles.into_iter().filter(|r| per_role || r.is_none()) {
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
                 verify, run_in, investigate:"
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
    fn hold_and_resume_are_a_persons_and_each_is_refused_when_it_would_change_nothing() {
        let mut w = refining();
        let agent = AgentId::new("author-1");
        let why =
            |w: &World, by: Option<&AgentId>, c: &Command| w.events(by, c).expect_err("refused");
        assert!(
            why(&w, Some(&agent), &Command::Hold)
                .starts_with("hold is a person's: author-1 is an agent"),
            "{}",
            why(&w, Some(&agent), &Command::Hold)
        );
        assert_eq!(
            why(&w, None, &Command::Resume),
            "the daemon is not held; temet-work hold holds it"
        );
        assert_eq!(w.events(None, &Command::Hold), Ok(vec![Event::Held]));
        let at = 1_795_000_000_000;
        w.apply_at(at, &Event::Held);
        assert_eq!(
            why(&w, None, &Command::Hold),
            format!(
                "the daemon is already held, since {}; temet-work resume lifts the hold",
                crate::investigation::utc(at)
            )
        );
        assert!(
            why(&w, Some(&agent), &Command::Resume)
                .starts_with("resume is a person's: author-1 is an agent")
        );
        assert_eq!(w.events(None, &Command::Resume), Ok(vec![Event::Resumed]));
        // Agents finishing their step are still heard while the daemon is held.
        let refiner = AgentId::new("refiner-1");
        assert!(w.events(Some(&refiner), &plan(&[])).is_ok());
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
                cost: None,
                point: None,
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
        // An investigator works for its group: it starts once its group investigates.
        let task = |w: &World, role| {
            let launch = if role == Role::Investigator {
                let mut w = w.clone();
                w.apply(&Event::InvestigationStarted {
                    group: GroupId::new("watch"),
                    agent: AgentId::new("investigator-1"),
                    at_ms: 0,
                });
                w.launch_investigator(&GroupId::new("watch"))
            } else {
                w.launch(&GroupId::new("watch"), role, &issue(), None)
            };
            launch.expect("launch").brief.task
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

    #[test]
    fn an_overturns_reason_is_stored_with_its_controls_dropped() {
        let events = decided()
            .events(None, &overturn(1, 1, super::tests::NOISY))
            .expect("a person may");
        let Some(Event::DecisionOverturned { reason, .. }) = events.first() else {
            panic!("not an overturn: {events:?}");
        };
        assert_eq!(reason, super::tests::STORED);
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
            queue_low: None,
            investigate_every: None,
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
            runtime: None,
            model: None,
        });
        World::replay(&events).0
    }

    fn ready(pull: Option<&str>, pulls: &[(&str, &str)]) -> Command {
        Command::Ready {
            issue: gld(),
            pull: pull.map(String::from),
            pulls: pulls.iter().map(|(r, n)| (pid(r), n.to_string())).collect(),
            replies: Vec::new(),
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
                replies: Vec::new(),
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
            replies: Vec::new(),
        });
        w.apply(&Event::ReviewFinished {
            issue: gld(),
            round: 1,
            verdict: Verdict::Block {
                points: vec!["x".into()],
            },
            failed: None,
            open: None,
        });
        let replied = Command::Ready {
            issue: gld(),
            pull: None,
            pulls: Default::default(),
            replies: vec!["done as asked".into()],
        };
        assert!(check(&w, &replied).is_ok());
    }
}
