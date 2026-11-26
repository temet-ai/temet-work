//! The code-host contract against GitHub itself, reading the public repository
//! temet-ai/radar through `gh`. It never writes. Runs only with the `live`
//! feature and the sandbox off:
//!
//! ```sh
//! TEMET_WORK_SANDBOX= cargo nextest run -p temet-work-codehost-github --features live
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
fn passes_the_code_host_contract_against_github() {
    if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
        eprintln!(
            "skipped: TEMET_WORK_SANDBOX is set; run with TEMET_WORK_SANDBOX= to reach GitHub"
        );
        return;
    }
    let mut gh = GitHub::default();
    assert_eq!(contract::check(&mut gh, &ReadOnly), Ok(()));
}
