//! Where issues, milestones and pull requests come from, and where status, comments and merges go back to.
//!
//! This crate holds the `Tracker` contract and, later, a deterministic fake and the
//! contract tests every adapter runs. Adapter crates depend on this crate only.

use temet_work_core::Part;

/// One tracker, added by an adapter crate.
pub trait Tracker: Send {
    /// A short, stable id, the adapter's feature name after `tracker-`.
    fn id(&self) -> &'static str;
}

/// The tracker part: the trackers compiled into this build, in the order they were added.
#[derive(Default)]
pub struct Trackers {
    adapters: Vec<Box<dyn Tracker>>,
}

impl Trackers {
    /// Adds a tracker. The root package calls this once per compiled-in adapter.
    pub fn add(&mut self, adapter: Box<dyn Tracker>) {
        self.adapters.push(adapter);
    }

    /// The ids of the trackers in this build.
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }
}

impl Part for Trackers {
    fn id(&self) -> &'static str {
        "tracker"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Named(&'static str);

    impl Tracker for Named {
        fn id(&self) -> &'static str {
            self.0
        }
    }

    #[test]
    fn keeps_adapters_in_the_order_added() {
        let mut part = Trackers::default();
        part.add(Box::new(Named("first")));
        part.add(Box::new(Named("second")));
        assert_eq!(part.ids(), ["first", "second"]);
        assert_eq!(part.id(), "tracker");
    }
}
