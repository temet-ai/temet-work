//! The code-host contract against GitLab itself, reading a public merge request
//! through `glab`. It never writes. Runs only with the `live` feature and the
//! sandbox off, with `glab` signed in:
//!
//! ```sh
//! TEMET_WORK_SANDBOX= cargo nextest run -p temet-work-codehost-gitlab --features live --run-ignored only
//! ```
//!
//! To read another merge request, change `REPO` and `PULL` to a public one.

#![cfg(feature = "live")]

use temet_work_codehost::contract::{self, Harness};
use temet_work_codehost_gitlab::GitLab;

/// A public project and a merged merge request in it (its iid).
const REPO: &str = "gitlab-org/cli";
const PULL: &str = "4594";

struct ReadOnly;

impl Harness for ReadOnly {
    fn repo(&self) -> String {
        REPO.into()
    }

    fn pull(&self) -> String {
        PULL.into()
    }

    fn mergeable(&self) -> Option<String> {
        None
    }
}

#[test]
#[ignore = "reaches GitLab: run with --run-ignored only and TEMET_WORK_SANDBOX unset"]
fn passes_the_code_host_contract_against_gitlab() {
    let mut glab = GitLab::default();
    assert_eq!(contract::check(&mut glab, &ReadOnly), Ok(()));
}
