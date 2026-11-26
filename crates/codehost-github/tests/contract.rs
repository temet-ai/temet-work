//! The code-host contract against GitHub itself, reading the public repository
//! temet-ai/radar through `gh`. It never writes. Runs only with the `live`
//! feature and the sandbox off:
//!
//! ```sh
//! TEMET_WORK_SANDBOX= cargo nextest run -p temet-work-codehost-github --features live --run-ignored only
//! ```

#![cfg(feature = "live")]

use temet_work_codehost::contract::{self, Harness};
use temet_work_codehost_github::GitHub;

struct ReadOnly;

impl Harness for ReadOnly {
    fn repo(&self) -> String {
        "temet-ai/radar".into()
    }

    fn pull(&self) -> String {
        "46".into()
    }

    fn mergeable(&self) -> Option<String> {
        None
    }
}

#[test]
#[ignore = "reaches GitHub: run with --run-ignored only and TEMET_WORK_SANDBOX unset"]
fn passes_the_code_host_contract_against_github() {
    let mut gh = GitHub::default();
    assert_eq!(contract::check(&mut gh, &ReadOnly), Ok(()));
}
