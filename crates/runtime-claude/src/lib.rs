//! Claude Code sessions: `claude [--model M] TASK` starts an interactive session
//! with the task as its first prompt.

use temet_work_runtime::{Brief, Program, Runtime};

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
        Program {
            argv,
            env: Vec::new(),
        }
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
