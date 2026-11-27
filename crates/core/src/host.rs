//! The daemon's loop, without its I/O: it logs each event, applies it, and
//! offers each effect to the parts until one carries it out. Parts do the I/O;
//! the journal keeps the log.
//!
//! Work runs depth first: what an effect leads to is handled before the next
//! effect, so an agent started by one effect is known before the next one is
//! numbered.

use std::collections::BTreeMap;

use crate::Part;
use crate::effect::Effect;
use crate::event::Event;
use crate::model::AgentState;
use crate::world::World;

/// Where the host writes every event before it applies it.
pub trait Journal: Send {
    /// Appends one event, stamped `at_ms`. An error stops the host: state that
    /// is not logged would be lost on restart.
    fn append(&mut self, at_ms: u64, event: &Event) -> Result<(), String>;
}

/// A journal in memory, for tests and builds without a store.
#[derive(Default)]
pub struct Memory {
    /// The events, with their times, in order.
    pub events: Vec<(u64, Event)>,
}

impl Journal for Memory {
    fn append(&mut self, at_ms: u64, event: &Event) -> Result<(), String> {
        self.events.push((at_ms, event.clone()));
        Ok(())
    }
}

/// What a part did with an effect it was offered.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum Outcome {
    /// Not this part's to do.
    #[default]
    NotMine,
    /// Done, with what it led to.
    Done {
        /// Events to log and apply.
        events: Vec<Event>,
        /// Effects to offer next.
        effects: Vec<Effect>,
    },
    /// This part's to do, and it failed.
    Failed(String),
}

impl Outcome {
    /// Done, leading to nothing more.
    pub fn done() -> Self {
        Self::Done {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }
}

/// What a part reports when it is polled.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Polled {
    /// What it saw, as events.
    pub events: Vec<Event>,
    /// What is wrong for it now, if anything. It stays shown until a poll
    /// reports none; a part reports it on every poll while it lasts.
    pub problem: Option<String>,
}

/// How problems from giving up on an issue's agents are keyed.
const GAVE_UP: &str = "gave up: ";

/// How problems from an agent stopped at a prompt are keyed.
const BLOCKED: &str = "blocked: ";

/// How problems from an agent whose turn ended in an error are keyed.
const ERRORED: &str = "errored: ";

/// The most events and effects one call handles before it stops, so a loop
/// between parts cannot hang the daemon.
pub const MAX_STEPS: usize = 10_000;

enum Work {
    Event(Event),
    Effect(Effect),
}

/// The loop with its parts and journal.
pub struct Host {
    world: World,
    parts: Vec<Box<dyn Part>>,
    journal: Box<dyn Journal>,
    problems: BTreeMap<String, String>,
}

/// What the parts' families offer for the settings that name an adapter, by
/// setting. The daemon puts it in the world before it replays the log, since an
/// older log's plain model is bound to the build's default runtime.
pub fn catalog(parts: &[Box<dyn Part>]) -> BTreeMap<String, crate::settings::Choice> {
    parts
        .iter()
        .flat_map(|p| p.choices())
        .map(|c| (c.key.clone(), c))
        .collect()
}

impl Host {
    /// A host over `world`, already built from the journal's events.
    pub fn new(mut world: World, parts: Vec<Box<dyn Part>>, journal: Box<dyn Journal>) -> Self {
        world.catalog = catalog(&parts);
        Self {
            world,
            parts,
            journal,
            problems: BTreeMap::new(),
        }
    }

    /// The loop's state.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// What is wrong now, by where it came from: a part whose last poll failed,
    /// or an effect that no part carried out.
    pub fn problems(&self) -> &BTreeMap<String, String> {
        &self.problems
    }

    /// What `agent`'s terminal shows now, from the part that runs it.
    pub fn screen(&mut self, agent: &crate::ids::AgentId) -> Result<String, String> {
        self.parts
            .iter_mut()
            .find_map(|p| p.screen(agent))
            .unwrap_or_else(|| Err(format!("no part in this build shows {agent}'s terminal")))
    }

    /// Logs and applies `event`, then carries out everything it leads to, and
    /// ends the investigations that it ended (`World::investigation_events`).
    pub fn handle(&mut self, now_ms: u64, event: Event) -> Result<(), String> {
        self.run(now_ms, vec![Work::Event(event)])?;
        self.investigations(now_ms, false)
    }

    /// Carries out `effect` now, offered to the parts in order, and returns the
    /// events it led to, or why it failed. A command that waits for a part's
    /// answer, such as filing an issue, goes through here. The outer error stops
    /// the daemon: an event could not be logged.
    pub fn perform(
        &mut self,
        now_ms: u64,
        effect: &Effect,
    ) -> Result<Result<Vec<Event>, String>, String> {
        match self.offer(effect) {
            Outcome::Done { events, effects } => {
                self.problems.remove(&effect.describe());
                let mut stack: Vec<Work> = effects.into_iter().rev().map(Work::Effect).collect();
                stack.extend(events.iter().cloned().rev().map(Work::Event));
                self.run(now_ms, stack)?;
                self.investigations(now_ms, false)?;
                Ok(Ok(events))
            }
            Outcome::Failed(why) => Ok(Err(why)),
            Outcome::NotMine => Ok(Err(format!(
                "no part in this build can {}",
                effect.describe()
            ))),
        }
    }

    /// Logs and applies what the clock makes due at `now_ms`: the end of each
    /// investigation whose investigator is done, and, when `start`, the start of
    /// one for each group short of work.
    fn investigations(&mut self, now_ms: u64, start: bool) -> Result<(), String> {
        let due = self.world.investigation_events(now_ms, start);
        if due.is_empty() {
            return Ok(());
        }
        self.run(now_ms, due.into_iter().rev().map(Work::Event).collect())
    }

    /// Starts an investigation for each group short of work, then the agents
    /// issues and investigations need and do not have (`World::missing`).
    pub fn recover(&mut self, now_ms: u64) -> Result<(), String> {
        self.investigations(now_ms, true)?;
        let missing = self.world.missing();
        self.problems.retain(|k, _| !k.starts_with(GAVE_UP));
        for line in missing.given_up {
            self.problems.insert(
                format!("{GAVE_UP}{line}"),
                "a person has to look at it".into(),
            );
        }
        self.run(
            now_ms,
            missing.start.into_iter().rev().map(Work::Effect).collect(),
        )
    }

    fn run(&mut self, now_ms: u64, mut stack: Vec<Work>) -> Result<(), String> {
        let mut steps = 0;
        while let Some(work) = stack.pop() {
            steps += 1;
            if steps > MAX_STEPS {
                return Err(format!(
                    "stopped after {MAX_STEPS} steps: the parts keep asking for more"
                ));
            }
            match work {
                Work::Event(e) => {
                    self.journal.append(now_ms, &e)?;
                    let effects = self.world.apply(&e);
                    stack.extend(effects.into_iter().rev().map(Work::Effect));
                }
                Work::Effect(effect) => match self.offer(&effect) {
                    Outcome::Done { events, effects } => {
                        self.problems.remove(&effect.describe());
                        stack.extend(effects.into_iter().rev().map(Work::Effect));
                        stack.extend(events.into_iter().rev().map(Work::Event));
                    }
                    Outcome::Failed(why) => {
                        self.problems.insert(effect.describe(), why);
                    }
                    Outcome::NotMine if effect.needs_a_part() => {
                        self.problems
                            .insert(effect.describe(), "no part in this build does this".into());
                    }
                    Outcome::NotMine => {}
                },
            }
        }
        Ok(())
    }

    /// Asks each part what it has seen and handles the events it reports, then
    /// starts the agents issues are missing. Returns how many events there were.
    pub fn tick(&mut self, now_ms: u64) -> Result<usize, String> {
        let mut seen = Vec::new();
        for part in &mut self.parts {
            let polled = part.poll(&self.world, now_ms);
            match polled.problem {
                Some(why) => self.problems.insert(part.id().to_string(), why),
                None => self.problems.remove(part.id()),
            };
            seen.extend(polled.events);
        }
        let n = seen.len();
        for event in seen {
            self.handle(now_ms, event)?;
        }
        self.recover(now_ms)?;
        self.report_blocked();
        Ok(n)
    }

    /// Lists every agent stopped at a prompt as a problem: the daemon never
    /// answers a prompt, so a person has to. Lists every agent whose turn ended in
    /// an error too, with what the daemon will do about it. Rebuilt on each tick,
    /// so each goes when the agent leaves the state or ends.
    fn report_blocked(&mut self) {
        self.problems
            .retain(|k, _| !k.starts_with(BLOCKED) && !k.starts_with(ERRORED));
        for agent in self.world.agents.values() {
            let place = || {
                agent
                    .issue
                    .as_ref()
                    .and_then(|i| self.world.issues.get(i))
                    .and_then(|i| self.world.folder_for(i, agent.role, agent.folder.as_ref()))
                    // A reader's folder is not recorded: name none on another machine.
                    .filter(|f| f.machine == agent.machine)
                    .map_or_else(String::new, |f| format!(" in {}", f.path))
            };
            match &agent.state {
                AgentState::Blocked { reason } => {
                    self.problems.insert(
                        format!("{BLOCKED}{}", agent.id),
                        format!(
                            "{}{} is blocked: {reason}; the daemon cannot answer it",
                            agent.id,
                            place()
                        ),
                    );
                }
                AgentState::Errored { line, .. } => {
                    let line = if line.is_empty() {
                        String::new()
                    } else {
                        format!(": {line}")
                    };
                    let next = if agent.limit_reset().is_some() {
                        "the daemon sends continue after it resets"
                    } else {
                        "the daemon reminds it and stops it as it does an idle agent"
                    };
                    self.problems.insert(
                        format!("{ERRORED}{}", agent.id),
                        format!("{}{} stopped on an error{line}; {next}", agent.id, place()),
                    );
                }
                _ => {}
            }
        }
    }

    fn offer(&mut self, effect: &Effect) -> Outcome {
        for part in &mut self.parts {
            match part.carry_out(effect, &self.world) {
                Outcome::NotMine => {}
                outcome => return outcome,
            }
        }
        Outcome::NotMine
    }
}

#[cfg(test)]
#[path = "host_tests.rs"]
mod tests;
