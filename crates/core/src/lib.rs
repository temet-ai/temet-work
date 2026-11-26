//! The core of temet-work: the objects the system works with, the work loop as a
//! pure state machine, and the contract every part meets.
//!
//! The core does no I/O, names no tool or service, and holds no UI type. The
//! tests in `tests/parts.rs` check all three. It changes only for something every
//! part gains.

pub mod effect;
pub mod event;
pub mod ids;
pub mod model;
pub mod world;

pub use effect::Effect;
pub use event::{Event, Verdict};
pub use world::World;

/// A part of temet-work: a family of adapters (trackers, runners, agent
/// runtimes) or a service such as storage or the style guide. The root package
/// registers each compiled-in part once, behind its own feature.
pub trait Part: Send {
    /// A short, stable id. It is also the part's feature name.
    fn id(&self) -> &'static str;
}
