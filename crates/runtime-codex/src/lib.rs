//! Codex sessions: `codex --approve-for-me [--model M] TASK` starts an
//! interactive session with the task as its first prompt.
//!
//! Agents run unattended, as Claude's do under auto mode. `--approve-for-me`
//! keeps Codex's workspace-write sandbox and routes approval requests through its
//! automatic review, so an agent never stops at a prompt nobody answers and is
//! still not unsandboxed (`--dangerously-bypass-approvals-and-sandbox` would be).
//! The brief's `permission` setting is Claude's permission mode and is not used here.
//!
//! It is not the default runtime: a setting names it, such as `runtime = "codex"`
//! for reviewers. Like the other runtimes it passes on the daemon's proxy
//! variables and code-host sign-in (`temet_work_runtime::PASSED_ON`). Codex's own
//! sign-in stays in `~/.codex` and is not copied.

use temet_work_runtime::{Brief, Program, Runtime};

/// The Codex adapter.
#[derive(Default)]
pub struct Codex;

impl Codex {
    fn argv(brief: &Brief) -> Vec<String> {
        let mut argv = vec!["codex".to_string(), "--approve-for-me".to_string()];
        if let Some(model) = &brief.model {
            argv.push("--model".into());
            argv.push(model.clone());
        }
        argv.push(brief.task.clone());
        argv
    }
}

impl Runtime for Codex {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn program(&self, brief: &Brief) -> Program {
        Program {
            argv: Self::argv(brief),
            env: temet_work_runtime::passed_on_env(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temet_work_runtime::{IssueId, Role, passed_on_from};

    fn brief() -> Brief {
        Brief {
            role: Role::Author,
            issue: IssueId::new("a", "1"),
            task: "do it".into(),
            model: None,
            permission: "auto".into(),
            folder: None,
        }
    }

    #[test]
    fn passes_the_runtime_contract() {
        assert_eq!(temet_work_runtime::contract::check(&Codex), Ok(()));
    }

    #[test]
    fn argv_without_and_with_a_model() {
        let mut brief = brief();
        assert_eq!(
            Codex.program(&brief).argv,
            ["codex", "--approve-for-me", "do it"]
        );
        brief.model = Some("gpt-x".into());
        assert_eq!(
            Codex.program(&brief).argv,
            ["codex", "--approve-for-me", "--model", "gpt-x", "do it"]
        );
    }

    #[test]
    fn env_passes_the_shared_variables_on_and_leaves_unset_ones_out() {
        let set = passed_on_from(|k| (k == "GH_TOKEN").then(|| "t".to_string()));
        assert_eq!(set, [("GH_TOKEN".to_string(), "t".to_string())]);
        assert!(passed_on_from(|_| None).is_empty());
    }
}
