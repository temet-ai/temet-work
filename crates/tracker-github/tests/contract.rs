//! The tracker contract against GitHub itself, reading the public repository
//! temet-ai/radar through `gh`. It never writes. Runs only with the `live`
//! feature and the sandbox off:
//!
//! ```sh
//! TEMET_WORK_SANDBOX= cargo nextest run -p temet-work-tracker-github --features live --run-ignored only
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
#[ignore = "reaches GitHub: run with --run-ignored only and TEMET_WORK_SANDBOX unset"]
fn passes_the_tracker_contract_against_github() {
    let mut gh = GitHub::default();
    assert_eq!(contract::check(&mut gh, &ReadOnly), Ok(()));
}
