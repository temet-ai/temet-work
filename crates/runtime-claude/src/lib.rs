//! Claude Code sessions: `claude --permission-mode auto [--model M] TASK` starts
//! an interactive session with the task as its first prompt.
//!
//! The permission mode is the brief's `permission` setting, `auto` unless one is
//! set. Agents run unattended, so they run in auto mode: Claude Code's own check
//! approves what is safe and refuses what is not, where the default mode would stop
//! at a prompt nobody answers. A live run on the sandbox stopped at the first
//! shell loop and the first MCP tool without it.
//!
//! A terminal gets its terminal service's environment, not the daemon's, so what a
//! session started from the person's own shell would have is passed on from the
//! daemon when it has it: the proxy variables a capturing proxy needs
//! (`HTTPS_PROXY`, `NO_PROXY`, `NODE_EXTRA_CA_CERTS`) and the code host's sign-in
//! (`GITHUB_TOKEN`, `GH_TOKEN`), without which `gh` cannot read an issue or open a
//! pull request.

use temet_work_runtime::{Brief, Program, Runtime};

/// The Claude Code adapter.
#[derive(Default)]
pub struct Claude;

impl Runtime for Claude {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn is_default(&self) -> bool {
        true
    }

    fn program(&self, brief: &Brief) -> Program {
        let mut argv = vec![
            "claude".to_string(),
            "--permission-mode".to_string(),
            brief.permission.clone(),
        ];
        if let Some(model) = &brief.model {
            argv.push("--model".into());
            argv.push(model.clone());
        }
        argv.push(brief.task.clone());
        Program {
            argv,
            env: temet_work_runtime::passed_on_env(),
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

    #[test]
    fn the_brief_permission_is_the_permission_mode() {
        use temet_work_runtime::{IssueId, Role};
        let brief = Brief {
            role: Role::Author,
            issue: IssueId::new("a", "1"),
            task: "do it".into(),
            model: None,
            permission: "plan".into(),
            folder: None,
        };
        assert_eq!(
            Claude.program(&brief).argv,
            ["claude", "--permission-mode", "plan", "do it"]
        );
    }
}
