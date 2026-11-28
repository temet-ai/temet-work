//! A person's move of an issue to another stage (`temet-work move`): which moves
//! the loop takes, what each does to the issue's agents, folders, pull requests
//! and review rounds, and what the reply says.
//!
//! Two moves are taken, both backwards: to new, so the issue is refined again
//! from nothing, and to ready, so a new author does it again from its plan.
//! Every other move skips a step the loop or an agent owns, or would undo a
//! merge, and is refused with its reason. [`World::may_move`] is the one rule:
//! the command is checked by it and the view lists the moves it takes, so the
//! window offers only moves the daemon takes.

use crate::effect::Effect;
use crate::ids::{IssueId, ProjectId};
use crate::model::{Agent, AgentState, CheckState, Issue, Moved, Stage};
use crate::text;
use crate::view::StageWord;
use crate::world::World;

impl World {
    /// Why a person may not move `issue` to `to`; `Ok` when the move is taken.
    /// Taken: to new from refining, ready, working, in review, fixing or with
    /// the arbiter, and to ready from working, in review, fixing or with the
    /// arbiter, while none of its pull requests has merged. Refused: a word this
    /// build does not know; any move out of done (reopen it in its tracker), of
    /// an issue merging, deploying or verifying, or of one with a pull request
    /// merged (the loop cannot undo a merge); to the stage it is at; to done
    /// (close it in its tracker); to ready before it has a plan; and to every
    /// other stage, which the loop or an agent moves it to.
    pub fn may_move(&self, issue: &Issue, to: StageWord) -> Result<(), String> {
        let id = &issue.id;
        let at = text::stage(issue.stage);
        if to == StageWord::Unknown {
            return Err(format!(
                "move takes a stage this build knows: {}",
                StageWord::listed()
            ));
        }
        if issue.stage == Stage::Done {
            return Err(format!(
                "{id} is done, and the loop moves a done issue nowhere: reopen it in its \
                 tracker to have it worked again"
            ));
        }
        if issue.finishing() {
            let what = if issue.merged.is_empty()
                || matches!(
                    issue.stage,
                    Stage::Merging | Stage::Deploying | Stage::Verifying
                ) {
                format!("{id} is {at}")
            } else {
                let repos: Vec<String> = self
                    .repos_of(issue)
                    .iter()
                    .filter(|r| issue.merged.contains(*r))
                    .map(ToString::to_string)
                    .collect();
                format!("{id} has its pull request in {} merged", repos.join(", "))
            };
            return Err(format!(
                "{what}: the loop cannot undo a merge, so it moves no issue once its merge has \
                 begun; it finishes, and once it is done a person reopens it in its tracker to \
                 have it worked again"
            ));
        }
        if StageWord::from(issue.stage) == to {
            return Err(format!("{id} is already {at}"));
        }
        let owner = match to {
            StageWord::New => return Ok(()),
            StageWord::Ready => {
                return match issue.stage {
                    Stage::New | Stage::Refining => Err(format!(
                        "{id} is {at} and has no plan yet: its refiner's plan moves it to ready"
                    )),
                    _ => Ok(()),
                };
            }
            StageWord::Done => {
                return Err(format!(
                    "{id} is not moved to done: close it in its tracker, and the loop takes it \
                     as done at the next sync"
                ));
            }
            StageWord::Refining => {
                "the loop takes an issue at new for refining when its group has a refining slot \
                 free: move it to new to have it refined again"
            }
            StageWord::Working => {
                "the loop starts an author on a ready issue once a work folder is free: move it \
                 to ready for a new author"
            }
            StageWord::InReview => "its author's ready puts it in review",
            StageWord::Fixing => {
                "a reviewer's block, an arbiter's ruling, a failed check or a conflict sends it \
                 back to its author"
            }
            StageWord::Arbitration => {
                "the loop hands it to an arbiter when the review rounds run out with the \
                 reviewer still blocking"
            }
            StageWord::Merging => "the loop merges it once its review and its checks have passed",
            StageWord::Deploying | StageWord::Verifying => {
                "the loop deploys and verifies it after its merge"
            }
            StageWord::Unknown => unreachable!("refused above"),
        };
        Err(format!(
            "{id} is not moved to {}, a step the loop or an agent owns: {owner}",
            to.name()
        ))
    }

    /// The stages a person may move `issue` to, in the loop's order: each word
    /// [`World::may_move`] takes.
    pub fn moves_of(&self, issue: &Issue) -> Vec<StageWord> {
        StageWord::ALL
            .into_iter()
            .filter(|to| self.may_move(issue, *to).is_ok())
            .collect()
    }

    /// The stage a move to `to` puts an issue at: new or ready, the two moves
    /// [`World::may_move`] can take.
    pub(crate) fn move_target(to: StageWord) -> Option<Stage> {
        match to {
            StageWord::New => Some(Stage::New),
            StageWord::Ready => Some(Stage::Ready),
            _ => None,
        }
    }

    /// Whether `agent` started before a person last moved its issue. Its step is
    /// over for good ([`World::over_for_good`]), whatever stage the issue has
    /// reached since, and it is not the issue's agent of its role
    /// ([`World::running`]), so a fresh one starts in its place.
    pub fn before_move(&self, agent: &Agent) -> bool {
        agent
            .issue
            .as_ref()
            .and_then(|id| self.issues.get(id))
            .and_then(|i| i.moved)
            .is_some_and(|m| agent.started < m.at)
    }

    /// Why `agent` may not run `step` on its issue because a person moved the
    /// issue after it started; `None` when it started after the last move, or
    /// the issue was never moved.
    pub(crate) fn moved_past(&self, agent: &Agent, step: &str) -> Option<String> {
        if !self.before_move(agent) {
            return None;
        }
        let issue = agent.issue.as_ref()?;
        let to = self.issues.get(issue)?.moved?.to;
        let now = self.running(issue, agent.role).map_or_else(
            || format!("no {} is working on it now", agent.role.name()),
            |a| format!("{} is working on it now", a.id),
        );
        let what = if to == Stage::Refining {
            format!("sent {issue} to a refiner")
        } else {
            format!("moved {issue} to {}", text::stage(to))
        };
        Some(format!(
            "{step} for {issue} is refused: a person {what} after {} started, so its step is \
             over and its session is being stopped; {now}",
            agent.id
        ))
    }

    /// Each pull request of `issue` on record, by repository in the order they
    /// merge: what a move of it drops and leaves open on the code host.
    pub fn open_pulls(&self, issue: &IssueId) -> Vec<(ProjectId, String)> {
        self.issues.get(issue).map_or_else(Vec::new, |i| {
            self.pulls_of(i)
                .into_iter()
                .filter(|(repo, _)| !i.merged.contains(*repo))
                .map(|(repo, number)| (repo.clone(), number.clone()))
                .collect()
        })
    }

    /// Applies a person's move of `issue` to `stage`, when [`World::may_move`]
    /// takes it, through [`World::take_back`]. Back to new it has no group and
    /// no plan, and is taken for refining in its turn; back to ready it keeps
    /// both, with each check waiting, and a new author starts once a folder is
    /// free. Its decisions, its issue-level settings and what a failed verify
    /// left stay. While the daemon is held the stops and releases are made at
    /// once, and nothing starts on it until a person resumes it.
    pub(crate) fn apply_move(&mut self, issue: &IssueId, stage: Stage, out: &mut Vec<Effect>) {
        let Some(i) = self.issues.get(issue) else {
            return;
        };
        if self.may_move(i, stage.into()).is_err() {
            return;
        }
        self.take_back(issue, stage, out);
        let Some(i) = self.issues.get_mut(issue) else {
            return;
        };
        i.stage = stage;
        if stage == Stage::New {
            i.group = None;
            i.plan = None;
        } else {
            i.set_checks(CheckState::Waiting);
        }
    }

    /// What a person's move and a person's send to a refiner
    /// (`temet-work refine --send`) do alike to `issue`, which goes to `to`.
    /// Every agent on the issue that has not ended is stopped, and its step is
    /// over for good, also once a fresh agent works on the issue
    /// ([`World::before_move`]). Every work folder it held is released, though
    /// no author is placed in one while an agent that held it runs
    /// ([`World::in_use`]). Its pull requests, branches and closed-pull notes,
    /// fix points, approval, check results, review waiver and review rounds go,
    /// so its next change starts at round 1. The caller sets the stage.
    pub(crate) fn take_back(&mut self, issue: &IssueId, to: Stage, out: &mut Vec<Effect>) {
        out.extend(
            self.agents
                .values()
                .filter(|a| a.issue.as_ref() == Some(issue) && a.state != AgentState::Ended)
                .map(|a| Effect::StopAgent {
                    agent: a.id.clone(),
                }),
        );
        let at = self.applied;
        let Some(i) = self.issues.get_mut(issue) else {
            return;
        };
        let folders = i.take_folders();
        i.review_ok = false;
        i.checks_ok = None;
        i.pulls.clear();
        i.merged.clear();
        i.branches.clear();
        i.closed.clear();
        i.fix.clear();
        i.told = false;
        i.fix_keeps_round = false;
        i.round = 0;
        i.review_waived = false;
        i.reviews.clear();
        i.moved = Some(Moved { at, to });
        self.release(folders, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_new_and_ready_are_targets() {
        let targets: Vec<StageWord> = StageWord::ALL
            .into_iter()
            .filter(|w| World::move_target(*w).is_some())
            .collect();
        assert_eq!(targets, [StageWord::New, StageWord::Ready]);
    }
}
