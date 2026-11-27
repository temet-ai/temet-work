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

//! Claude Code asks whether to trust a folder the first time it starts there, and
//! has no flag or setting that skips the question, so [`Claude::trust`] answers it
//! where Claude Code keeps the answer: `hasTrustDialogAccepted` for the folder in
//! its user file, `.claude.json` in the folder `CLAUDE_CONFIG_DIR` names, else in
//! the home folder. Listing a folder in the configuration is the person's choice
//! to give agents that folder.

use std::path::Path;

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

    fn trust(&self, folder: &Path, home: &Path) -> Result<(), String> {
        trust_folder(folder, &home.join(".claude.json"))
    }
}

/// Marks `folder` trusted in Claude Code's user file `file`. Only that folder's
/// entry is touched, a folder already trusted leaves the file as it is, and the
/// file is replaced whole (a temporary file, then a rename). A file that is not
/// a JSON object is an error and is not overwritten.
fn trust_folder(folder: &Path, file: &Path) -> Result<(), String> {
    use serde_json::{Map, Value};
    let key = std::fs::canonicalize(folder)
        .unwrap_or_else(|_| folder.to_path_buf())
        .to_string_lossy()
        .into_owned();
    let mut root = match std::fs::read_to_string(file) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(o)) => o,
            _ => return Err(format!("{} is not a JSON object", file.display())),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Map::new(),
        Err(e) => return Err(format!("cannot read {}: {e}", file.display())),
    };
    let projects = root
        .entry("projects")
        .or_insert_with(|| Value::Object(Map::new()));
    let Some(projects) = projects.as_object_mut() else {
        return Err(format!(
            "\"projects\" in {} is not an object",
            file.display()
        ));
    };
    let entry = projects
        .entry(key)
        .or_insert_with(|| Value::Object(Map::new()));
    let Some(entry) = entry.as_object_mut() else {
        return Err(format!(
            "the folder's entry in {} is not an object",
            file.display()
        ));
    };
    if entry.get("hasTrustDialogAccepted") == Some(&Value::Bool(true)) {
        return Ok(());
    }
    entry.insert("hasTrustDialogAccepted".into(), Value::Bool(true));
    let text = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    let temp = file.with_extension(format!("json.tmp{}", std::process::id()));
    let write = std::fs::write(&temp, text).and_then(|()| {
        if let Ok(meta) = std::fs::metadata(file) {
            std::fs::set_permissions(&temp, meta.permissions())?;
        }
        std::fs::rename(&temp, file)
    });
    write.map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        format!("cannot write {}: {e}", file.display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_the_runtime_contract() {
        assert_eq!(temet_work_runtime::contract::check(&Claude), Ok(()));
    }

    fn temp_home(name: &str) -> std::path::PathBuf {
        // Under the build folder, which belongs to the person running the tests,
        // not under a temp folder every user can write to.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/tw-test")
            .join(format!("claude-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("test dir");
        dir
    }

    fn read(home: &Path) -> serde_json::Value {
        let text = std::fs::read_to_string(home.join(".claude.json")).expect("file");
        serde_json::from_str(&text).expect("json")
    }

    #[test]
    fn a_missing_file_is_created_with_only_that_folder_trusted() {
        let home = temp_home("new");
        Claude.trust(Path::new("/work/a"), &home).expect("trust");
        assert_eq!(
            read(&home),
            serde_json::json!({"projects": {"/work/a": {"hasTrustDialogAccepted": true}}})
        );
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn an_existing_file_keeps_its_other_projects_and_keys() {
        let home = temp_home("keep");
        std::fs::write(
            home.join(".claude.json"),
            r#"{"theme":"dark","projects":{"/other":{"hasTrustDialogAccepted":false,"allowedTools":["x"]},"/work/a":{"history":[1]}}}"#,
        )
        .expect("write");
        Claude.trust(Path::new("/work/a"), &home).expect("trust");
        assert_eq!(
            read(&home),
            serde_json::json!({"theme":"dark","projects":{
                "/other":{"hasTrustDialogAccepted":false,"allowedTools":["x"]},
                "/work/a":{"history":[1],"hasTrustDialogAccepted":true}}})
        );
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn a_second_call_changes_nothing() {
        let home = temp_home("twice");
        Claude.trust(Path::new("/work/a"), &home).expect("trust");
        let file = home.join(".claude.json");
        let before = std::fs::read(&file).expect("read");
        let mtime = std::fs::metadata(&file)
            .and_then(|m| m.modified())
            .expect("mtime");
        Claude
            .trust(Path::new("/work/a"), &home)
            .expect("trust again");
        assert_eq!(std::fs::read(&file).expect("read"), before);
        assert_eq!(
            std::fs::metadata(&file)
                .and_then(|m| m.modified())
                .expect("mtime"),
            mtime,
            "a trusted folder is not written again"
        );
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn a_malformed_file_is_an_error_and_is_not_overwritten() {
        let home = temp_home("bad");
        std::fs::write(home.join(".claude.json"), "{ not json").expect("write");
        let why = Claude
            .trust(Path::new("/work/a"), &home)
            .expect_err("error");
        assert!(why.contains(".claude.json"), "{why}");
        assert_eq!(
            std::fs::read_to_string(home.join(".claude.json")).expect("read"),
            "{ not json"
        );
        let _ = std::fs::remove_dir_all(&home);
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
