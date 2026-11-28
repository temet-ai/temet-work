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
mod fit;
pub mod host;
pub mod ids;
pub mod investigation;
pub mod launch;
pub mod model;
pub mod placement;
pub mod reply;
pub mod sender;
pub mod settings;
pub mod socket;
pub mod text;
pub mod view;
pub mod world;

pub use command::{At, Command, Request};
pub use effect::Effect;
pub use event::{Event, Verdict};
pub use host::{Host, Journal, Outcome, Polled};
pub use reply::Reply;
pub use sender::{Digest, Peer, Sender};
pub use view::View;
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

/// How a part answers a command that reads code or CI ([`Part::read`]).
pub enum Answer {
    /// The text, made now.
    Now(String),
    /// Work that makes the text, or the reason it could not, run away from the
    /// daemon's loop: it may fetch from a code host, and the loop answers other
    /// requests and polls its parts meanwhile. It holds no part, only what the
    /// read needs.
    Later(Box<dyn FnOnce() -> Result<String, String> + Send>),
}

impl Answer {
    /// The text, making it now for [`Answer::Later`].
    pub fn wait(self) -> Result<String, String> {
        match self {
            Self::Now(text) => Ok(text),
            Self::Later(work) => work(),
        }
    }
}

impl std::fmt::Debug for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Now(text) => f.debug_tuple("Now").field(text).finish(),
            Self::Later(_) => f.write_str("Later(..)"),
        }
    }
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

    /// The answer to a command that reads code or CI ([`Command::reads_code`]):
    /// a pull request's change, a file, a search or CI's results, if this part
    /// reads them. It changes nothing.
    fn read(&mut self, world: &World, command: &Command) -> Option<Result<Answer, String>> {
        let _ = (world, command);
        None
    }

    /// The web addresses this part gives for the projects and areas it holds,
    /// made from their names in the configuration: a tracker family gives where
    /// their issues are, a code-host family where their code is. It asks nothing
    /// of any service. The host puts them in the world each round.
    fn addresses(
        &self,
        world: &World,
    ) -> std::collections::BTreeMap<ids::ProjectId, model::Addresses> {
        let _ = world;
        std::collections::BTreeMap::new()
    }
}
