//! The core of temet-work: the objects the system works with, the work loop as a
//! pure state machine, and the contract every part meets.
//!
//! The core does no I/O, names no tool or service, and holds no UI type. The
//! tests in `tests/parts.rs` check all three. It changes only for something every
//! part gains.

pub mod brief;
pub mod command;
pub mod effect;
pub mod event;
pub mod host;
pub mod ids;
pub mod investigation;
pub mod launch;
pub mod model;
pub mod placement;
pub mod sender;
pub mod settings;
pub mod text;
pub mod world;

pub use command::{Command, Request};
pub use effect::Effect;
pub use event::{Event, Verdict};
pub use host::{Host, Journal, Outcome, Polled};
pub use sender::{Digest, Peer, Sender};
pub use world::World;

/// An issue's own text, as a part last read it from where the issue comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssueText {
    /// Where the text came from and when it was read, in words that follow
    /// "Text of".
    pub from: String,
    /// The text; empty when the issue has none there.
    pub text: String,
}

/// A part of temet-work: a family of adapters (trackers, runners, agent
/// runtimes) or a service such as storage or the style guide. The root package
/// registers each compiled-in part once, behind its own feature.
pub trait Part: Send {
    /// A short, stable id. It is also the part's feature name.
    fn id(&self) -> &'static str;

    /// What the part has seen since it was last asked, as events: issues synced
    /// from a tracker, an agent that went idle. `now_ms` is the daemon's clock,
    /// so a part polls on its own schedule without reading one.
    fn poll(&mut self, world: &World, now_ms: u64) -> Polled {
        let _ = (world, now_ms);
        Polled::default()
    }

    /// What this part offers for a setting that names one of its adapters, so a
    /// value naming one the build lacks is refused.
    fn choices(&self) -> Vec<settings::Choice> {
        Vec::new()
    }

    /// Carries out `effect` if it is this part's to do.
    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let _ = (effect, world);
        Outcome::NotMine
    }

    /// What `agent`'s terminal shows now, if this part runs it.
    fn screen(&mut self, agent: &ids::AgentId) -> Option<Result<String, String>> {
        let _ = agent;
        None
    }

    /// The text of `issue` as this part last read it, or why it has not read it,
    /// if this part reads issue text. It asks nothing of any service.
    fn issue_text(&self, world: &World, issue: &ids::IssueId) -> Option<Result<IssueText, String>> {
        let _ = (world, issue);
        None
    }
}
