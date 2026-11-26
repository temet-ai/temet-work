//! `temet-work`, the command line: sends one command to the daemon and prints
//! its reply. `temet-work help` lists the commands.

use std::process::ExitCode;

use temet_work::cli;
use temet_work::socket::{self, Reply};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("help" | "-h" | "--help") => {
            println!("{}", cli::USAGE);
            return ExitCode::SUCCESS;
        }
        Some("mcp") => return mcp(&args[1..]),
        Some("parts") => {
            let ids: Vec<&str> = temet_work::parts().iter().map(|p| p.id()).collect();
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
    let by = std::env::var(temet_work::cli::AGENT_VAR).ok();
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
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    match socket::send(&socket::path(), &request) {
        Ok(Reply::Done { text }) => {
            println!("{}", text.trim_end());
            ExitCode::SUCCESS
        }
        Ok(Reply::Refused { why }) => {
            eprintln!("refused: {why}");
            ExitCode::from(1)
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
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
    let path = socket::path();
    let mut send = |r: &temet_work_core::Request| socket::send(&path, r);
    let mut out = std::io::stdout().lock();
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if let Some(answer) = temet_work::mcp::handle(&msg, level, by.as_deref(), &mut send)
            && writeln!(out, "{answer}")
                .and_then(|()| out.flush())
                .is_err()
        {
            break;
        }
    }
    ExitCode::SUCCESS
}
