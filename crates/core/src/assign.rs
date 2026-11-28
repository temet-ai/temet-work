//! A person's `assign` and `unassign` (`docs/commands.md`, "Assigning an agent"):
//! which agent a person may put on which issue, and what an accepted one does to
//! the agents of both issues.
//!
//! An assignment moves a running refiner, reviewer or arbiter of the issue's
//! group, now on another issue of the same project, onto an issue whose stage
//! needs that role now. An author is never moved (decision 56): it holds its
//! issue's work folders and may be mid-edit in them, so moving one would leave
//! two authors in one folder; unassigning an issue's author starts a fresh one in
//! its folders instead. Unassigning takes a running agent off its issue and stops
//! its session at once, as a person's stop does (decision 57); the issue's stage
//! gets what it needs once that session has ended.

use crate::brief;
use crate::effect::Effect;
use crate::ids::{AgentId, IssueId};
use crate::model::{AgentState, Issue, Role, Stage};
use crate::text::stage;
use crate::world::World;

impl World {
    /// The role `issue`'s stage needs an agent of now: a refiner while it is
    /// refining, an author while it is worked on or fixed, a reviewer while it
    /// is in review with no approval in, an arbiter while it is with the arbiter
    /// with no ruling in. None for any other stage.
    pub fn stage_needs(issue: &Issue) -> Option<Role> {
        match issue.stage {
            Stage::Refining => Some(Role::Refiner),
            Stage::Working | Stage::Fixing { .. } => Some(Role::Author),
            Stage::InReview { .. } | Stage::Arbitration { .. } if issue.review_ok => None,
            Stage::InReview { .. } => Some(Role::Reviewer),
            Stage::Arbitration { .. } => Some(Role::Arbiter),
            _ => None,
        }
    }

    /// The issue `agent` works on now, when a person may assign it to `issue`;
    /// why not otherwise. The agent must be a running refiner, reviewer or
    /// arbiter this daemon started, of `issue`'s group, on another issue of the
    /// same project and not unassigned from it; `issue` must not be paused
    /// ([`World::paused`]: decision 60, a pause means no work starts on it), and
    /// its stage must need the agent's role now ([`World::stage_needs`]). The
    /// view's `assignable` list is worked out by this same rule (decision 59).
    pub fn assignment(&self, issue: &IssueId, agent: &AgentId) -> Result<IssueId, String> {
        let Some(a) = self.agents.get(agent) else {
            return Err(format!("{agent} is not an agent this daemon started"));
        };
        if a.state == AgentState::Ended {
            return Err(format!("{agent} has ended; only a running agent is assigned"));
        }
        match a.role {
            Role::Author => {
                return Err(format!(
                    "{agent} is an author, and an author is not assigned: it holds its issue's \
                     work folders and may be mid-edit in them, so moving it would leave two \
                     authors in one folder. Unassigning an issue's author starts a fresh one in \
                     its folders: temet-work unassign {issue} AUTHOR"
                ));
            }
            Role::Investigator => {
                return Err(format!(
                    "{agent} is an investigator, which works for its group on no issue, so it is \
                     not assigned to one"
                ));
            }
            Role::Refiner | Role::Reviewer | Role::Arbiter => {}
        }
        let Some(i) = self.issues.get(issue) else {
            return Err(format!("there is no issue {issue}"));
        };
        let Some(from) = a.issue.clone() else {
            return Err(format!("{agent} works on no issue"));
        };
        if a.unassigned {
            return Err(format!(
                "{agent} was unassigned from {from} by a person and is being stopped; assign a \
                 running agent that works on an issue"
            ));
        }
        if from == *issue {
            return Err(format!("{agent} is on {issue} already"));
        }
        if i.group.as_ref() != Some(&a.group) {
            let theirs = i.group.as_ref().map_or_else(
                || "is taken by no group".to_string(),
                |g| format!("is {g}'s"),
            );
            return Err(format!(
                "{agent} is of group {}, and {issue} {theirs}: an agent is assigned only within \
                 its own group",
                a.group
            ));
        }
        if from.project != issue.project {
            return Err(format!(
                "{agent} works on {from}, of project {}, and {issue} is of {}: an agent is \
                 assigned only within the project it works in",
                from.project, issue.project
            ));
        }
        if self.paused(i) {
            return Err(if self.held {
                format!(
                    "{issue} is paused while the daemon is held, and no work starts on it until \
                     a person resumes the daemon"
                )
            } else {
                format!(
                    "{issue} is paused: it left its group's scope, and no work starts on it until \
                     it is back"
                )
            });
        }
        match Self::stage_needs(i) {
            Some(role) if role == a.role => Ok(from),
            Some(role) => Err(format!(
                "{issue} is {}, so it needs {} {} now, not {} {}",
                stage(i.stage),
                article(role),
                role.name(),
                article(a.role),
                a.role.name()
            )),
            None => Err(format!(
                "{issue} is {} and needs no agent now, so no {} is assigned to it",
                stage(i.stage),
                a.role.name()
            )),
        }
    }

    /// The agents a person may assign to `issue` now, by [`World::assignment`],
    /// in agent order.
    pub fn assignable(&self, issue: &IssueId) -> Vec<AgentId> {
        self.agents
            .keys()
            .filter(|a| self.assignment(issue, a).is_ok())
            .cloned()
            .collect()
    }

    /// Why a person may not unassign `agent` from `issue`: it is not an agent
    /// this daemon started, has ended, is an investigator (which works on no
    /// issue), or is not on `issue`, or was unassigned from it already. An
    /// unassign works on a paused issue too, since a stop starts no work.
    pub fn unassignment(&self, issue: &IssueId, agent: &AgentId) -> Result<(), String> {
        let Some(a) = self.agents.get(agent) else {
            return Err(format!("{agent} is not an agent this daemon started"));
        };
        if a.state == AgentState::Ended {
            return Err(format!("{agent} has ended; there is nothing to unassign"));
        }
        if a.role == Role::Investigator {
            return Err(format!(
                "{agent} is an investigator, which works for its group on no issue, so it is not \
                 unassigned from one"
            ));
        }
        if !self.issues.contains_key(issue) {
            return Err(format!("there is no issue {issue}"));
        }
        if a.issue.as_ref() == Some(issue) && a.unassigned {
            return Err(format!(
                "{agent} was unassigned from {issue} already and is being stopped"
            ));
        }
        match a.issue.as_ref() {
            Some(on) if on == issue => Ok(()),
            Some(on) if a.unassigned => Err(format!(
                "{agent} is not on {issue}: it was unassigned from {on} and is being stopped"
            )),
            Some(on) => Err(format!("{agent} is not on {issue}; it works on {on}")),
            None => Err(format!("{agent} is not on {issue}; it works on no issue")),
        }
    }

    /// Whether an agent of `role` a person took off `issue` still runs. No fresh
    /// agent of that role starts on the issue until it has ended, as after a
    /// stop for sitting idle, so two sessions never do one step and an author's
    /// folder never has two writers.
    pub fn stopping(&self, issue: &IssueId, role: Role) -> bool {
        self.agents.values().any(|a| {
            a.issue.as_ref() == Some(issue)
                && a.role == role
                && a.unassigned
                && a.state != AgentState::Ended
        })
    }

    /// Whether `effect` would start a fresh agent on an issue while an agent of
    /// its role that a person took off the issue still runs ([`World::stopping`]).
    /// The loop drops it; [`World::missing`] asks again once that one has ended.
    /// A review round with a reviewer working on the issue tells that one and
    /// starts none, so it goes through.
    pub(crate) fn waits_for_unassigned(&self, effect: &Effect) -> bool {
        let (issue, role) = match effect {
            Effect::StartAgent { issue, role, .. } => (issue, *role),
            Effect::StartReview { issue, .. } if self.running(issue, Role::Reviewer).is_none() => {
                (issue, Role::Reviewer)
            }
            _ => return false,
        };
        self.stopping(issue, role)
    }

    /// What a person's accepted assign of `agent` from `from` to `issue` does.
    /// The agent of its role `issue` had is unassigned and stopped, as
    /// `unassign` does. The agent works on `issue` from now on, keeping its
    /// terminal and labels (decision 62), and is sent the brief for it
    /// ([`Effect::Brief`]); a moved reviewer's verdict waits for that brief.
    /// `from` gets a fresh agent of the role at once when its stage needs one
    /// now; none of `starts` is used, since the agent did not end there.
    pub(crate) fn assign(
        &mut self,
        issue: &IssueId,
        agent: &AgentId,
        from: &IssueId,
        out: &mut Vec<Effect>,
    ) {
        // Only a running agent that works on `from` moves, so a log replays to
        // the same world.
        let Some(role) = self
            .agents
            .get(agent)
            .filter(|a| a.works_on(from))
            .map(|a| a.role)
        else {
            return;
        };
        if !self.issues.contains_key(issue) || issue == from {
            return;
        }
        let had: Vec<AgentId> = self
            .agents
            .values()
            .filter(|a| a.id != *agent && a.role == role && a.works_on(issue))
            .map(|a| a.id.clone())
            .collect();
        for id in &had {
            self.unassign(id, out);
        }
        if let Some(a) = self.agents.get_mut(agent) {
            a.issue = Some(issue.clone());
            a.moved_from = Some(from.clone());
            a.briefed = false;
        }
        // A moved reviewer has not heard this round: its verdict is refused
        // until its brief has reached it.
        if role == Role::Reviewer
            && let Some(i) = self.issues.get_mut(issue)
        {
            i.told = false;
        }
        out.push(Effect::Brief {
            agent: agent.clone(),
            issue: issue.clone(),
        });
        let others = self
            .agents
            .values()
            .any(|a| a.role == role && a.works_on(from));
        if let Some(f) = self.issues.get(from)
            && !others
            && Self::stage_needs(f) == Some(role)
            && let Some(group) = f.group.clone()
        {
            out.push(match (role, f.stage) {
                (Role::Reviewer, Stage::InReview { round }) => Effect::StartReview {
                    issue: from.clone(),
                    round,
                },
                _ => Effect::StartAgent {
                    group,
                    role,
                    issue: from.clone(),
                    folder: None,
                },
            });
        }
    }

    /// Takes `agent` off its issue and asks for its stop, unless it has ended
    /// or was taken off already.
    pub(crate) fn unassign(&mut self, agent: &AgentId, out: &mut Vec<Effect>) {
        if let Some(a) = self.agents.get_mut(agent)
            && a.state != AgentState::Ended
            && !a.unassigned
        {
            a.unassigned = true;
            out.push(Effect::StopAgent {
                agent: agent.clone(),
            });
        }
    }

    /// The brief for its issue reached a moved agent: a moved reviewer has heard
    /// its round, so its verdict is taken from now on.
    pub(crate) fn briefed(&mut self, agent: &AgentId, issue: &IssueId) {
        let Some(a) = self
            .agents
            .get_mut(agent)
            .filter(|a| a.works_on(issue) && a.assigned())
        else {
            return;
        };
        a.briefed = true;
        if a.role == Role::Reviewer
            && let Some(i) = self.issues.get_mut(issue)
            && matches!(i.stage, Stage::InReview { .. })
        {
            i.told = true;
        }
    }

    /// A [`Effect::Brief`] for each running agent a person moved whose brief has
    /// not reached it, in agent order: the effect asked for at the move may have
    /// failed, or been lost to a restart, since a replay drops effects.
    pub(crate) fn briefs_again(&self) -> Vec<Effect> {
        self.agents
            .values()
            .filter(|a| a.awaits_brief())
            .filter_map(|a| {
                Some(Effect::Brief {
                    agent: a.id.clone(),
                    issue: a.issue.clone()?,
                })
            })
            .collect()
    }

    /// What a running agent a person moved is sent: a line saying who moved it
    /// from which issue to which, and that its commands for the old one are
    /// refused, then the brief a fresh agent of its role on its new issue would
    /// start with, held to the size every brief is. `None` for an agent no
    /// person moved, or whose issue is not known.
    pub fn moved_brief(&self, agent: &AgentId) -> Option<String> {
        let a = self.agents.get(agent)?;
        let from = a.moved_from.as_ref()?;
        let i = self.issues.get(a.issue.as_ref()?)?;
        let line = format!(
            "A person moved you from {from} to {issue}: you work on {issue} now, and your commands \
             for {from} are refused from here on. Leave {from} as it is. Your brief for {issue} \
             follows.\n\n",
            issue = i.id
        );
        Some(brief::moved(
            &line,
            a.role,
            i,
            self.source_name(&i.id),
            &self.area_repos(&i.id),
            &self.pulled(i),
            &self.kept_rules(),
        ))
    }
}

/// "a" or "an" before a role's name.
fn article(role: Role) -> &'static str {
    match role {
        Role::Author | Role::Arbiter | Role::Investigator => "an",
        Role::Refiner | Role::Reviewer => "a",
    }
}

#[cfg(test)]
#[path = "world_assign_tests.rs"]
mod tests;
