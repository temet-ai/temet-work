//! `temet-workd`, the daemon: it reads the configuration, rebuilds its state
//! from the event log, answers commands on its socket as they come, and every
//! two seconds asks each part what it has seen and carries out what the loop
//! asks for.
//!
//! ```text
//! temet-workd [--config FILE] [--log FILE] [--once]
//! temet-workd --version | --build
//! ```
//!
//! `--version` prints the version and the build id, `--build` the id alone: the
//! commit it was built from, which a deploy that restarts the daemon compares.
//!
//! `--once` runs one round and prints the state as text. The configuration
//! defaults to `$XDG_CONFIG_HOME/temet-work/config.toml` and the log to
//! `$XDG_STATE_HOME/temet-work/events.jsonl`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use temet_work::config::Config;
use temet_work::socket;
use temet_work_core::{Event, Host};

const EVERY: Duration = Duration::from_secs(2);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

/// `$var/rest`, or `$HOME/fallback/rest` when the variable is unset.
fn xdg(var: &str, fallback: &str, rest: &str) -> PathBuf {
    std::env::var_os(var)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(fallback)))
        .unwrap_or_default()
        .join("temet-work")
        .join(rest)
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut config_path = xdg("XDG_CONFIG_HOME", ".config", "config.toml");
    let mut log_path = xdg("XDG_STATE_HOME", ".local/state", "events.jsonl");
    let mut once = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                config_path = args
                    .next()
                    .map(PathBuf::from)
                    .ok_or("--config needs a file")?
            }
            "--log" => log_path = args.next().map(PathBuf::from).ok_or("--log needs a file")?,
            "--once" => once = true,
            "--version" => {
                println!(
                    "temet-workd {} ({})",
                    env!("CARGO_PKG_VERSION"),
                    temet_work::BUILD
                );
                return Ok(());
            }
            "--build" => {
                println!("{}", temet_work::BUILD);
                return Ok(());
            }
            other => {
                return Err(format!(
                    "unknown argument {other}; see temet-workd --help in docs/configuration.md"
                ));
            }
        }
    }
    let text = std::fs::read_to_string(&config_path).map_err(|e| {
        format!(
            "cannot read the configuration at {}: {e}. docs/configuration.md has an example.",
            config_path.display()
        )
    })?;
    let config = Config::parse(&text).map_err(|e| format!("{}: {e}", config_path.display()))?;
    if let Some(dir) = log_path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let (journal, world) = temet_work::journal(&log_path)?;
    let mut host = Host::new(
        world,
        // A deploy that restarts this daemon leaves its note next to the log.
        temet_work::parts_with(
            config.daemon_host.clone(),
            Some(log_path.with_file_name("deploy-handoff")),
        ),
        journal,
    );
    if host.world().log.is_none() {
        // A new log: its id labels the terminals its agents run in.
        let id = format!("{:x}{:x}", now_ms(), std::process::id());
        host.handle(now_ms(), Event::LogStarted { id })?;
    }
    // A setting that names a runtime or runner this build lacks stops the start,
    // before anything is logged.
    config
        .check_build(host.world())
        .map_err(|e| format!("{}: {e}", config_path.display()))?;
    for event in config.events(host.world()) {
        host.handle(now_ms(), event)?;
    }
    let unready = host.world().unready();
    if !unready.is_empty() {
        return Err(unready.join("; "));
    }
    if once {
        host.tick(now_ms())?;
        print!(
            "{}",
            temet_work_core::text::text(host.world(), host.problems())
        );
        return Ok(());
    }
    let socket_path = socket::path();
    let requests = socket::listen(&socket_path)?;
    eprintln!("temet-workd: listening at {}", socket_path.display());
    let mut shown: BTreeMap<String, String> = BTreeMap::new();
    let mut next_round = Instant::now();
    loop {
        if Instant::now() >= next_round {
            host.tick(now_ms())?;
            next_round = Instant::now() + EVERY;
            if host.problems() != &shown {
                for (from, what) in host.problems() {
                    if shown.get(from) != Some(what) {
                        eprintln!("problem: {from}: {what}");
                    }
                }
                shown = host.problems().clone();
            }
        }
        match requests.recv_timeout(next_round.saturating_duration_since(Instant::now())) {
            Ok(pending) => {
                let reply = socket::answer(&mut host, now_ms(), &pending.request)?;
                // Who sent what, and what came of it: a request log on stderr.
                let debug = format!("{:?}", pending.request.command);
                eprintln!(
                    "request: by={} {} -> {}",
                    pending.request.by.as_ref().map_or("none", |a| a.0.as_str()),
                    debug.split([' ', '{', '(']).next().unwrap_or_default(),
                    match &reply {
                        socket::Reply::Done { .. } => "done".to_string(),
                        socket::Reply::Refused { why } => format!("refused: {why}"),
                    }
                );
                pending.answer(reply);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                return Err("the socket stopped listening".into());
            }
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("temet-workd: {e}");
            ExitCode::FAILURE
        }
    }
}
