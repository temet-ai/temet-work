//! Where agent sessions run: labelled terminals that can be started, read, written to, waited on and followed.
//!
//! This crate holds the `Runner` contract and, later, a deterministic fake and the
//! contract tests every adapter runs. Adapter crates depend on this crate only.

use temet_work_core::Part;

/// One runner, added by an adapter crate.
pub trait Runner: Send {
    /// A short, stable id, the adapter's feature name after `runner-`.
    fn id(&self) -> &'static str;
}

/// The runner part: the runners compiled into this build, in the order they were added.
#[derive(Default)]
pub struct Runners {
    adapters: Vec<Box<dyn Runner>>,
}

impl Runners {
    /// Adds a runner. The root package calls this once per compiled-in adapter.
    pub fn add(&mut self, adapter: Box<dyn Runner>) {
        self.adapters.push(adapter);
    }

    /// The ids of the runners in this build.
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }
}

impl Part for Runners {
    fn id(&self) -> &'static str {
        "runner"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Named(&'static str);

    impl Runner for Named {
        fn id(&self) -> &'static str {
            self.0
        }
    }

    #[test]
    fn keeps_adapters_in_the_order_added() {
        let mut part = Runners::default();
        part.add(Box::new(Named("first")));
        part.add(Box::new(Named("second")));
        assert_eq!(part.ids(), ["first", "second"]);
        assert_eq!(part.id(), "runner");
    }
}
