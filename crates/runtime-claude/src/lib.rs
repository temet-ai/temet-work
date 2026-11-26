//! Claude Code sessions: `claude [--model M] TASK` starts an interactive session
//! with the task as its first prompt.
//!
//! A terminal gets its terminal service's environment, not the daemon's, so the
//! proxy variables a capturing proxy needs (`HTTPS_PROXY`, `NO_PROXY`,
//! `NODE_EXTRA_CA_CERTS`) are passed on from the daemon when it has them.

use temet_work_runtime::{Brief, Program, Runtime};

/// Variables passed on to every session from the daemon's environment.
pub const PASSED_ON: [&str; 3] = ["HTTPS_PROXY", "NO_PROXY", "NODE_EXTRA_CA_CERTS"];

/// The Claude Code adapter.
#[derive(Default)]
pub struct Claude;

impl Runtime for Claude {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn program(&self, brief: &Brief) -> Program {
        let mut argv = vec!["claude".to_string()];
        if let Some(model) = &brief.model {
            argv.push("--model".into());
            argv.push(model.clone());
        }
        argv.push(brief.task.clone());
        let env = PASSED_ON
            .iter()
            .filter_map(|k| {
                std::env::var(k)
                    .ok()
                    .filter(|v| !v.is_empty())
                    .map(|v| (k.to_string(), v))
            })
            .collect();
        Program { argv, env }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_the_runtime_contract() {
        assert_eq!(temet_work_runtime::contract::check(&Claude), Ok(()));
    }
}
