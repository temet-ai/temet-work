//! The tracker contract against GitHub itself, reading the public repository
//! temet-ai/radar through `gh`. It never writes. Runs only with the `live`
//! feature and the sandbox off:
//!
//! ```sh
//! TEMET_WORK_SANDBOX= cargo nextest run -p temet-work-tracker-github --features live
//! ```

#![cfg(feature = "live")]

use temet_work_tracker::contract::{self, Harness};
use temet_work_tracker_github::GitHub;

struct ReadOnly;

impl Harness for ReadOnly {
    fn project(&self) -> String {
        "temet-ai/radar".into()
    }

    fn writable_issue(&self) -> Option<String> {
        None
    }
}

#[test]
fn passes_the_tracker_contract_against_github() {
    if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
        eprintln!(
            "skipped: TEMET_WORK_SANDBOX is set; run with TEMET_WORK_SANDBOX= to reach GitHub"
        );
        return;
    }
    let mut gh = GitHub::default();
    assert_eq!(contract::check(&mut gh, &ReadOnly), Ok(()));
}
