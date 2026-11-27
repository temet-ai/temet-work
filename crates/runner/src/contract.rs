//! The contract every runner adapter runs: the same steps against the fake in
//! this crate, against each adapter in this repository, and against a runner
//! outside it through its bridge.

use std::collections::BTreeMap;

use temet_work_core::launch::Program;

use crate::{Runner, RunnerError, Start, Status, TerminalId};

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

    /// A machine other than this one that the runner accepts, when it reaches
    /// more than one. `None` skips the run on another machine.
    fn machine(&self) -> Option<String>;

    /// Whether `terminal`, started on `machine`, is on that machine as the runner
    /// shows it (herdr-core addresses it `machine/t7`).
    fn reached(&self, machine: &str, terminal: &TerminalId) -> bool;

    /// Returns once the updates for what was just done can be read. A runner
    /// that reports asynchronously waits here; the fake returns at once.
    fn settle(&mut self, runner: &mut dyn Runner);
}

fn start(argv: Vec<String>, item: &str, cwd: std::path::PathBuf, machine: Option<String>) -> Start {
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
        machine,
    }
}

fn ensure(ok: bool, what: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(what.to_string()) }
}

/// Runs the contract on this machine, and again on the harness's other machine
/// when it has one. Returns the first step that failed, in words.
pub fn check(runner: &mut dyn Runner, harness: &mut dyn Harness) -> Result<(), String> {
    check_on(runner, harness, None)?;
    if let Some(machine) = harness.machine() {
        check_on(runner, harness, Some(machine))?;
    }
    Ok(())
}

/// Runs the contract with every start on `machine`.
pub fn check_on(
    runner: &mut dyn Runner,
    harness: &mut dyn Harness,
    machine: Option<String>,
) -> Result<(), String> {
    let t = runner
        .start(&start(
            harness.echo_program(),
            "contract-1",
            harness.folder(),
            machine.clone(),
        ))
        .map_err(|e| format!("start: {e}"))?;
    harness.settle(runner);
    if let Some(m) = &machine {
        ensure(
            harness.reached(m, &t),
            "the machine of a start reaches the runner: a terminal started on another machine is on that machine",
        )?;
    }
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

    let program = |name: &str| Program {
        argv: vec![name.to_string()],
        env: Vec::new(),
    };
    runner
        .exec(machine.as_deref(), &program("true"))
        .map_err(|e| format!("exec a program that exits 0: {e}"))?;
    ensure(
        runner.exec(machine.as_deref(), &program("false")).is_err(),
        "a program run to its end that exits nonzero is an error",
    )?;

    if let Some(argv) = harness.prompt_program() {
        let p = runner
            .start(&start(argv, "contract-2", harness.folder(), machine))
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
