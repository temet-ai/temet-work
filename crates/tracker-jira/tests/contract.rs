//! The tracker contract against a real Jira project, read only. The project key
//! comes from `JIRA_CONTRACT_PROJECT`, and `JIRA_URL`, `JIRA_USER` and `JIRA_TOKEN`
//! sign in. It never writes. Runs only with the `live` feature and the sandbox off:
//!
//! ```sh
//! TEMET_WORK_SANDBOX= JIRA_CONTRACT_PROJECT=KEY cargo nextest run -p temet-work-tracker-jira --features live --run-ignored only
//! ```

#![cfg(feature = "live")]

use temet_work_tracker::contract::{self, Harness};
use temet_work_tracker_jira::Jira;

struct ReadOnly;

impl Harness for ReadOnly {
    fn project(&self) -> String {
        std::env::var("JIRA_CONTRACT_PROJECT").expect("JIRA_CONTRACT_PROJECT names a project key")
    }

    fn writable_issue(&self) -> Option<String> {
        None
    }
}

#[test]
#[ignore = "reaches Jira: run with --run-ignored only and TEMET_WORK_SANDBOX unset"]
fn passes_the_tracker_contract_against_jira() {
    let mut jira = Jira::default();
    assert_eq!(contract::check(&mut jira, &ReadOnly), Ok(()));
}
