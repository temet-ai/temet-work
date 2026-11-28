//! A person's refine (`temet-work refine`): an issue's title, the checks the
//! person holds it to, and a send to a refiner. `docs/commands.md` says what a
//! send does at each stage; [`World::apply`] carries it out.

use crate::command::Command;
use crate::effect::Effect;
use crate::event::Event;
use crate::ids::{AgentId, GroupId, IssueId};
use crate::model::{Check, CheckState, Issue, Role, Stage};
use crate::placement::Unlisted;
use crate::text::{clean, stage};
use crate::world::World;

/// A title as it is stored: control characters dropped, and each run of spaces
/// and line breaks one space, with none at either end.
pub fn title(text: &str) -> String {
    clean(text).split_whitespace().collect::<Vec<_>>().join(" ")
}

impl World {
    /// The event a `refine` from `by` becomes, or why it is refused; none when it
    /// changes nothing and sends nothing. Only a person refines. A title equal to
    /// the issue's is not written again, and an empty one is refused. Each check
    /// is stored with its controls dropped, as a plan's are, and one with an
    /// empty done_when is refused; checks equal to the issue's change nothing. A
    /// send is refused for the reasons [`Self::may_send`] gives.
    pub(crate) fn refined(
        &self,
        by: Option<&AgentId>,
        command: &Command,
    ) -> Result<Option<Event>, String> {
        let Command::Refine {
            issue,
            title: new_title,
            checks,
            send,
        } = command
        else {
            return Err(format!("{} is not refine", command.name()));
        };
        if let Some(by) = by {
            return Err(format!(
                "refine is a person's: {by} is an agent; a refiner sends its plan with \
                 temet-work plan, and a person refines from a shell outside any agent's terminal"
            ));
        }
        let i = self
            .issues
            .get(issue)
            .ok_or_else(|| format!("there is no issue {issue}"))?;
        let new_title = match new_title {
            None => None,
            Some(t) => {
                let t = title(t);
                if t.is_empty() {
                    return Err(format!(
                        "a title cannot be empty; {issue} keeps \"{}\"",
                        i.title
                    ));
                }
                (t != i.title).then_some(t)
            }
        };
        let checks = match checks {
            None => None,
            Some(list) => {
                let mut kept = Vec::with_capacity(list.len());
                for (n, c) in (1..).zip(list) {
                    let done_when = clean(&c.done_when);
                    let proven_by = clean(&c.proven_by);
                    if done_when.trim().is_empty() {
                        return Err(format!(
                            "every check needs a done_when; check {n} has none{}",
                            if proven_by.trim().is_empty() {
                                String::new()
                            } else {
                                format!(", only proven_by {proven_by:?}")
                            }
                        ));
                    }
                    kept.push(Check::new(done_when, proven_by));
                }
                (kept != i.person_checks).then_some(kept)
            }
        };
        if *send {
            self.may_send(i)?;
        }
        if new_title.is_none() && checks.is_none() && !send {
            return Ok(None);
        }
        Ok(Some(Event::Refined {
            issue: issue.clone(),
            title: new_title,
            checks,
            send: *send,
        }))
    }

    /// The group a person's send hands `issue` to: the group that took it, or for
    /// a new issue the first group whose scope covers it. None when no group may
    /// refine it now.
    pub(crate) fn sending_group(&self, issue: &Issue) -> Option<GroupId> {
        match &issue.group {
            Some(g) if issue.stage != Stage::New => Some(g.clone()),
            _ => self
                .groups
                .values()
                .find(|g| self.in_scope(g, issue))
                .map(|g| g.id.clone()),
        }
    }

    /// What a person's send (`temet-work refine --send`) does to `issue`: it goes
    /// to a refiner at once, by the group that took it or, for a new issue, the
    /// first group whose scope covers it, past that group's `refining_at_once`.
    /// It is taken back as a person's move is ([`World::take_back`]): its agents
    /// are stopped and from before the send, its folders released, and its pull
    /// requests (left open on the code host), review rounds, fixes, approvals
    /// and a person's waiver dropped. Unlike a move to new, it keeps its group
    /// and its plan, with each check waiting, until the new plan replaces it. A
    /// merge is not undone: an issue that is finishing or done is not sent, as
    /// the command already refused.
    pub(crate) fn send_to_refiner(&mut self, issue: &IssueId, out: &mut Vec<Effect>) {
        let Some(i) = self.issues.get(issue) else {
            return;
        };
        if i.finishing() || i.stage == Stage::Done {
            return;
        }
        let Some(group) = self.sending_group(i) else {
            return;
        };
        self.take_back(issue, Stage::Refining, out);
        let Some(i) = self.issues.get_mut(issue) else {
            return;
        };
        i.set_checks(CheckState::Waiting);
        i.stage = Stage::Refining;
        i.group = Some(group.clone());
        out.push(Effect::StartAgent {
            group,
            role: Role::Refiner,
            issue: issue.clone(),
            folder: None,
        });
    }

    /// Why `issue` cannot go to a refiner now. A send is refused while the
    /// daemon is held, since it stops the agents at work that a hold lets finish
    /// their turn; on an issue that is merging, deploying, being verified or
    /// done, or that has a pull request merged, since a merge is not undone; on
    /// a new issue no group's scope covers, and on one that has left its group's
    /// scope; and when its refiner has no folder on the machine its `machine`
    /// setting names.
    pub(crate) fn may_send(&self, i: &Issue) -> Result<(), String> {
        let id = &i.id;
        if self.held {
            return Err(format!(
                "{id} is not sent to a refiner while the daemon is held: a send stops the agents \
                 at work, which the hold lets finish their turn. Run temet-work resume first, or \
                 save without send"
            ));
        }
        match i.stage {
            Stage::Done => {
                return Err(format!(
                    "{id} is done, so it is not sent to a refiner: a done issue is worked again \
                     by reopening it in its tracker"
                ));
            }
            Stage::Merging | Stage::Deploying | Stage::Verifying => {
                return Err(format!(
                    "{id} is {}, so it is not sent to a refiner: a merge is not undone",
                    stage(i.stage)
                ));
            }
            _ => {}
        }
        if !i.merged.is_empty() {
            let merged: Vec<String> = i.merged.iter().map(ToString::to_string).collect();
            return Err(format!(
                "{id} has its pull request in {} merged, so it is not sent to a refiner: a merge \
                 is not undone",
                merged.join(" and ")
            ));
        }
        if i.stage == Stage::New {
            if self.sending_group(i).is_none() {
                return Err(format!(
                    "no group's scope covers {id}, so no group takes it for refining: a group \
                     takes issues of its projects and areas, in its milestones when it names any"
                ));
            }
        } else if let Some(group) = i.group.as_ref().and_then(|g| self.groups.get(g))
            && let Some(unlisted) = self.unlisted(group, i)
        {
            let why = match unlisted {
                Unlisted::Project(p) => format!("{p} is not a project of {}", group.id),
                Unlisted::Area(a) => format!("{a} is not an area of {}", group.id),
                Unlisted::Milestone(m) => format!("{m} is not a milestone of {}", group.id),
                Unlisted::NoMilestone => {
                    format!("it has no milestone, and {} names some", group.id)
                }
            };
            return Err(format!(
                "{id} has left the scope of {}, so no refiner is started on it: {why}",
                group.id
            ));
        }
        if !self.has_place(i, Role::Refiner) {
            let machine = self
                .machine_for(id, Role::Refiner)
                .map_or_else(String::new, |m| m.to_string());
            return Err(format!(
                "{id}'s refiner has no folder to run in: its machine setting is {machine}, and \
                 {id}'s repository has no work folder there"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::title;

    #[test]
    fn a_title_keeps_one_space_for_each_run_of_spaces_and_line_breaks() {
        assert_eq!(
            title("  The gate\r\n shuts \u{1b}[31mat\t\tdusk  "),
            "The gate shuts [31mat dusk"
        );
        assert_eq!(title(" \n\t "), "");
    }
}
