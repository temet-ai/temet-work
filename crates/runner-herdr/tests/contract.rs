//! The runner contract against the herdr-core on this machine. It starts real
//! terminals (labelled `owner=temet-work`, removed again), so it runs only with
//! the `live` feature and with the sandbox off:
//!
//! ```sh
//! TEMET_WORK_SANDBOX= cargo nextest run -p temet-work-runner-herdr --features live --run-ignored only
//! ```

#![cfg(feature = "live")]

use std::path::PathBuf;
use std::time::Duration;

use temet_work_runner::Runner;
use temet_work_runner::contract::{self, Harness};
use temet_work_runner_herdr::Herdr;

struct Live;

impl Harness for Live {
    fn folder(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn echo_program(&self) -> Vec<String> {
        vec!["cat".into()]
    }

    fn prompt_program(&self) -> Option<Vec<String>> {
        // A prompt needs an agent that reports `blocked`; a plain program reports no
        // status. The fake covers this step.
        None
    }

    fn settle(&mut self, _runner: &mut dyn Runner) {
        std::thread::sleep(Duration::from_millis(1500));
    }
}

#[test]
#[ignore = "reaches herdr-core: run with --run-ignored only and TEMET_WORK_SANDBOX unset"]
fn passes_the_runner_contract_against_the_local_core() {
    let mut runner = Herdr::default();
    assert_eq!(contract::check(&mut runner, &mut Live), Ok(()));
}
