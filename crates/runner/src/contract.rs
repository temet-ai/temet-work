//! The contract every runner adapter runs: the same steps against the fake in
//! this crate, against each adapter in this repository, and against a runner
//! outside it through its bridge.

use std::collections::BTreeMap;

use temet_work_core::launch::Program;

use crate::{Runner, RunnerError, Start, Status};

/// What a runner's tests supply so the contract can drive it.
pub trait Harness {
    /// The folder the contract's terminals start in. The harness owns it; the
    /// contract writes nothing there.
    fn folder(&self) -> std::path::PathBuf;

    /// A program that prints each line it reads and runs until stopped.
    fn echo_program(&self) -> Vec<String>;

    /// A program that stops at a prompt as soon as it starts, when this runner
    /// can run one. `None` skips the prompt step.
    fn prompt_program(&self) -> Option<Vec<String>>;

    /// Returns once the updates for what was just done can be read. A runner
    /// that reports asynchronously waits here; the fake returns at once.
    fn settle(&mut self, runner: &mut dyn Runner);
}

fn start(argv: Vec<String>, item: &str, cwd: std::path::PathBuf) -> Start {
    Start {
        program: Program {
            argv,
            env: Vec::new(),
        },
        cwd,
        labels: BTreeMap::from([
            ("item".to_string(), item.to_string()),
            ("role".to_string(), "contract".to_string()),
        ]),
        machine: None,
    }
}

fn ensure(ok: bool, what: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(what.to_string()) }
}

/// Runs the contract. Returns the first step that failed, in words.
pub fn check(runner: &mut dyn Runner, harness: &mut dyn Harness) -> Result<(), String> {
    let t = runner
        .start(&start(
            harness.echo_program(),
            "contract-1",
            harness.folder(),
        ))
        .map_err(|e| format!("start: {e}"))?;
    harness.settle(runner);
    let updates = runner.updates();
    ensure(
        updates.iter().any(|u| {
            u.terminal == t && u.labels.get("item").map(String::as_str) == Some("contract-1")
        }),
        "the first update for a new terminal carries the labels it was started with",
    )?;

    runner
        .send(&t, "contract-line-1", false)
        .map_err(|e| format!("send to a running terminal: {e}"))?;
    harness.settle(runner);
    let screen = runner.screen(&t).map_err(|e| format!("screen: {e}"))?;
    ensure(
        screen.contains("contract-line-1"),
        "text sent to a terminal shows on its screen",
    )?;

    runner.stop(&t).map_err(|e| format!("stop: {e}"))?;
    harness.settle(runner);
    ensure(
        runner
            .updates()
            .iter()
            .any(|u| u.terminal == t && u.status == Status::Gone),
        "stopping a terminal reports it gone",
    )?;
    ensure(
        matches!(
            runner.send(&t, "after the end", false),
            Err(RunnerError::Gone | RunnerError::Unknown(_))
        ),
        "a send to a stopped terminal is refused as gone",
    )?;

    if let Some(argv) = harness.prompt_program() {
        let p = runner
            .start(&start(argv, "contract-2", harness.folder()))
            .map_err(|e| format!("start the prompt program: {e}"))?;
        harness.settle(runner);
        ensure(
            runner.send(&p, "should not be typed", false) == Err(RunnerError::Blocked),
            "a send to a terminal at a prompt is refused, and nothing is typed",
        )?;
        runner
            .stop(&p)
            .map_err(|e| format!("stop the prompt program: {e}"))?;
    }
    Ok(())
}
