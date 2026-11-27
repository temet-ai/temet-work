//! `temet-work`, the command line: sends one command to the daemon and prints
//! its reply. `temet-work help` lists the commands.

use std::process::ExitCode;

use temet_work::cli;
use temet_work::socket::{self, Reply};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The commands answered here take no `--json`; `cli::request` refuses it for
    // the rest.
    if let Some("help" | "-h" | "--help" | "mcp" | "parts" | "--version") =
        args.first().map(String::as_str)
        && args.iter().any(|a| a == cli::JSON)
    {
        eprintln!("{}", cli::json_refused());
        return ExitCode::from(2);
    }
    match args.first().map(String::as_str) {
        None | Some("help" | "-h" | "--help") => {
            println!("{}", cli::USAGE);
            return ExitCode::SUCCESS;
        }
        Some("mcp") => return mcp(&args[1..]),
        Some("relay") => return relay(&args[1..]),
        Some("trust-folder") => return trust_folder(&args[1..]),
        Some("--version") => {
            println!(
                "temet-work {} ({})",
                env!("CARGO_PKG_VERSION"),
                temet_work::BUILD
            );
            return ExitCode::SUCCESS;
        }
        Some("parts") => {
            let parts = temet_work::parts();
            let ids: Vec<String> = parts
                .iter()
                .map(|p| {
                    // A family lists its adapters: `runtime (claude, codex)`.
                    let adapters: Vec<String> = p
                        .choices()
                        .into_iter()
                        .find(|c| c.key == p.id())
                        .map(|c| c.allowed)
                        .unwrap_or_default();
                    if adapters.is_empty() {
                        p.id().to_string()
                    } else {
                        format!("{} ({})", p.id(), adapters.join(", "))
                    }
                })
                .collect();
            let ids = if ids.is_empty() {
                "none".to_string()
            } else {
                ids.join(", ")
            };
            println!("temet-work {} (parts: {ids})", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        _ => {}
    }
    let by = std::env::var(cli::AGENT_VAR).ok();
    // `-` is standard input, so an agent can send a plan without writing a file.
    let read = |p: &std::path::Path| {
        if p == std::path::Path::new("-") {
            let mut text = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut text)
                .map_err(|e| format!("standard input: {e}"))?;
            return Ok(text);
        }
        std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))
    };
    let request = match cli::request(&args, by, read) {
        Ok(r) => temet_work_core::Request {
            secret: secret(),
            ..r
        },
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let transport = match socket::from_env() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    match transport.send(&request) {
        Ok(Reply::Done { text }) => {
            println!("{}", text.trim_end());
            ExitCode::SUCCESS
        }
        Ok(Reply::Refused { why }) => {
            eprintln!("refused: {why}");
            ExitCode::from(1)
        }
        Ok(Reply::View { view }) => match serde_json::to_string_pretty(&view) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("the view cannot be written as JSON: {e}");
                ExitCode::from(2)
            }
        },
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

/// The secret this process was started with, when the daemon started it as an agent.
fn secret() -> Option<String> {
    std::env::var(cli::SECRET_VAR)
        .ok()
        .filter(|s| !s.is_empty())
}

/// `temet-work relay` (hidden): the far end of an SSH transport. Copies one
/// request line from standard input to this machine's daemon and the reply line
/// back to standard output, when the caller names the running daemon's build.
fn relay(args: &[String]) -> ExitCode {
    let (mut path, mut caller) = (None, socket::Caller::default());
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        match (flag.as_str(), args.next()) {
            ("--socket", Some(value)) => path = Some(std::path::PathBuf::from(value)),
            ("--build", Some(value)) => caller.build = Some(value),
            ("--host", Some(value)) => caller.host = Some(value),
            _ => {
                eprintln!("temet-work relay [--build BUILD] [--host HOST] [--socket PATH]");
                return ExitCode::from(2);
            }
        }
    }
    let path = path.unwrap_or_else(socket::path);
    let (input, out) = (std::io::stdin().lock(), std::io::stdout().lock());
    match socket::relay(&path, caller, input, out) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            let no_daemon = e.starts_with("no temet-work daemon answers");
            ExitCode::from(if no_daemon {
                socket::RELAY_NO_DAEMON as u8
            } else {
                1
            })
        }
    }
}

/// `temet-work trust-folder RUNTIME PATH` (hidden): marks PATH trusted for the
/// runtime in its user files on this machine, as the daemon does before an agent
/// starts in a folder on its own machine. The daemon runs it on the machine a
/// folder on another machine is on. Exits 0 when the folder is trusted or
/// already was, else 1 with the reason on one line of standard error.
fn trust_folder(args: &[String]) -> ExitCode {
    let [runtime, path] = args else {
        eprintln!("temet-work trust-folder RUNTIME PATH");
        return ExitCode::from(2);
    };
    match temet_work::trust_folder(runtime, std::path::Path::new(path)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}

/// Serves MCP on standard input and output until the client closes it.
fn mcp(args: &[String]) -> ExitCode {
    use std::io::{BufRead, Write};
    let level = match args {
        [] => temet_work::mcp::Level::Read,
        [flag, level] if flag == "--allow" && level == "read" => temet_work::mcp::Level::Read,
        [flag, level] if flag == "--allow" && level == "act" => temet_work::mcp::Level::Act,
        _ => {
            eprintln!("temet-work mcp [--allow read|act]");
            return ExitCode::from(2);
        }
    };
    let by = std::env::var(cli::AGENT_VAR).ok();
    let secret = secret();
    let transport = match socket::from_env() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let mut send = |r: &temet_work_core::Request| transport.send(r);
    let mut out = std::io::stdout().lock();
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if let Some(answer) =
            temet_work::mcp::handle(&msg, level, by.as_deref(), secret.as_deref(), &mut send)
            && writeln!(out, "{answer}")
                .and_then(|()| out.flush())
                .is_err()
        {
            break;
        }
    }
    ExitCode::SUCCESS
}
