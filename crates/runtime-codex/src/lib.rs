//! Codex sessions: `codex [--model M] (--full-auto | --ask-for-approval P) TASK`
//! starts an interactive session with the task as its first prompt.
//!
//! The brief's `permission` setting is Codex's approval policy. `auto`, the
//! built-in value, runs Codex in its low-friction automatic mode, which approves
//! what is safe inside its sandbox, so an unattended agent does not stop at a
//! prompt nobody answers; any other value is passed as the approval policy
//! (`untrusted`, `on-failure`, `on-request`, `never`).
//!
//! It is not the default runtime: a setting names it, such as `runtime = "codex"`
//! for reviewers. Like the other runtimes it passes on the daemon's proxy
//! variables and code-host sign-in, which a terminal service's environment lacks.

use temet_work_runtime::{Brief, Program, Runtime};

/// Variables passed on to every session from the daemon's environment.
pub const PASSED_ON: [&str; 5] = [
    "HTTPS_PROXY",
    "NO_PROXY",
    "NODE_EXTRA_CA_CERTS",
    "GITHUB_TOKEN",
    "GH_TOKEN",
];

/// The Codex adapter.
#[derive(Default)]
pub struct Codex;

impl Runtime for Codex {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn program(&self, brief: &Brief) -> Program {
        let mut argv = vec!["codex".to_string()];
        if let Some(model) = &brief.model {
            argv.push("--model".into());
            argv.push(model.clone());
        }
        if brief.permission == "auto" {
            argv.push("--full-auto".into());
        } else {
            argv.push("--ask-for-approval".into());
            argv.push(brief.permission.clone());
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
    use temet_work_runtime::{IssueId, Role};

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
    fn auto_is_full_auto_and_another_value_is_the_approval_policy() {
        let mut brief = brief();
        assert_eq!(
            Codex.program(&brief).argv,
            ["codex", "--full-auto", "do it"]
        );
        brief.permission = "on-request".into();
        brief.model = Some("gpt-x".into());
        assert_eq!(
            Codex.program(&brief).argv,
            [
                "codex",
                "--model",
                "gpt-x",
                "--ask-for-approval",
                "on-request",
                "do it"
            ]
        );
    }
}
