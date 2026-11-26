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
    let read = |p: &std::path::Path| {
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
