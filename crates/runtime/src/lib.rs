//! The agent programs that do the work: starting a session with a brief and a model, and telling working from idle.
//!
//! This crate holds the `Runtime` contract and, later, a deterministic fake and the
//! contract tests every adapter runs. Adapter crates depend on this crate only.

use temet_work_core::Part;

/// One runtime, added by an adapter crate.
pub trait Runtime: Send {
    /// A short, stable id, the adapter's feature name after `runtime-`.
    fn id(&self) -> &'static str;
}

/// The runtime part: the runtimes compiled into this build, in the order they were added.
#[derive(Default)]
pub struct Runtimes {
    adapters: Vec<Box<dyn Runtime>>,
}

impl Runtimes {
    /// Adds a runtime. The root package calls this once per compiled-in adapter.
    pub fn add(&mut self, adapter: Box<dyn Runtime>) {
        self.adapters.push(adapter);
    }

    /// The ids of the runtimes in this build.
    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }
}

impl Part for Runtimes {
    fn id(&self) -> &'static str {
        "runtime"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Named(&'static str);

    impl Runtime for Named {
        fn id(&self) -> &'static str {
            self.0
        }
    }

    #[test]
    fn keeps_adapters_in_the_order_added() {
        let mut part = Runtimes::default();
        part.add(Box::new(Named("first")));
        part.add(Box::new(Named("second")));
        assert_eq!(part.ids(), ["first", "second"]);
        assert_eq!(part.id(), "runtime");
    }
}
