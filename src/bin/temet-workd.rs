//! `temet-workd`, the daemon: it reads the configuration, rebuilds its state
//! from the event log, and then every two seconds asks each part what it has
//! seen and carries out what the loop asks for.
//!
//! ```text
//! temet-workd [--config FILE] [--log FILE] [--once]
//! ```
//!
//! `--once` runs one round and prints the state as text. The configuration
//! defaults to `$XDG_CONFIG_HOME/temet-work/config.toml` and the log to
//! `$XDG_STATE_HOME/temet-work/events.jsonl`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use temet_work::config::Config;
use temet_work_core::Host;

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
    let mut host = Host::new(world, temet_work::parts(), journal);
    for event in config.events(host.world()) {
        host.handle(now_ms(), event)?;
    }
    let mut shown: BTreeMap<String, String> = BTreeMap::new();
    loop {
        host.tick(now_ms())?;
        if once {
            print!(
                "{}",
                temet_work_core::text::text(host.world(), host.problems())
            );
            return Ok(());
        }
        if host.problems() != &shown {
            for (from, what) in host.problems() {
                if shown.get(from) != Some(what) {
                    eprintln!("problem: {from}: {what}");
                }
            }
            shown = host.problems().clone();
        }
        std::thread::sleep(EVERY);
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
