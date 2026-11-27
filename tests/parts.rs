//! The boundary rules, checked:
//!
//! - no part depends on another part, and the core depends on none;
//! - an adapter depends on its family crate only;
//! - no part names another, and the core names no tool or service;
//! - no UI library is named outside a front end;
//! - the root package registers each part behind its own feature;
//! - the desktop app, `app/`, is a workspace of its own and depends on the core
//!   and the style part only.
//!
//! A part is a workspace crate `temet-work-<id>` other than the core and the
//! front ends. An adapter is `temet-work-<family>-<x>`: it belongs to its family
//! part and may depend on it, and on nothing else in the workspace.

use std::path::{Path, PathBuf};
use std::process::Command;

const APP: &str = env!("CARGO_PKG_NAME");

/// Front ends draw the view data and may name UI libraries. None is a member of
/// this workspace: the desktop app is `app/`, a workspace of its own, checked by
/// `the_app_depends_on_core_and_style_only`.
const FRONT_ENDS: [&str; 0] = [];

/// Tools and services only parts know. The core names none of them, in code,
/// comments or tests.
const DOMAIN_WORDS: [&str; 10] = [
    "github", "gitlab", "jira", "linear", "herdr", "tmux", "claude", "codex", "mike", "slack",
];

/// UI libraries: only a front end may name one.
const UI_WORDS: [&str; 7] = [
    "gpui",
    "ratatui",
    "crossterm",
    "egui",
    "iced",
    "winit",
    "slint",
];

/// Part ids that are everyday words elsewhere: only their crate name counts.
const WORDS: [&str; 6] = ["store", "style", "tracker", "runner", "runtime", "deploy"];

struct Krate {
    name: String,
    dir: PathBuf,
    deps: Vec<String>,
}

fn core() -> String {
    format!("{APP}-core")
}

fn workspace() -> Vec<Krate> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout).expect("metadata is JSON");
    meta["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .map(|p| Krate {
            name: p["name"].as_str().unwrap_or_default().to_string(),
            dir: Path::new(p["manifest_path"].as_str().unwrap_or_default())
                .parent()
                .expect("a manifest is in a folder")
                .to_path_buf(),
            deps: p["dependencies"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|d| d["name"].as_str().map(String::from))
                .collect(),
        })
        .collect()
}

fn is_part(name: &str) -> bool {
    name.starts_with(&format!("{APP}-")) && name != core() && !FRONT_ENDS.contains(&name)
}

fn is_adapter_of(adapter: &str, part: &str) -> bool {
    adapter.starts_with(&format!("{part}-"))
}

fn id_of(part: &str) -> &str {
    &part[APP.len() + 1..]
}

/// Every `.rs` file under `dir/src`, with its text.
fn sources(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let mut todo = vec![dir.join("src")];
    while let Some(d) = todo.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let path = e.path();
            if path.is_dir() {
                todo.push(path);
            } else if path.extension().is_some_and(|x| x == "rs") {
                out.push((
                    path.clone(),
                    std::fs::read_to_string(&path).unwrap_or_default(),
                ));
            }
        }
    }
    out
}

/// How a part is named in code: its crate (`app-x`, `app_x`) and its id as a string.
fn names_of(part: &str) -> Vec<String> {
    let id = id_of(part);
    let mut names = vec![part.to_string(), part.replace('-', "_")];
    if !WORDS.contains(&id) {
        names.push(format!("\"{id}\""));
    }
    names
}

/// Whether `line` has `name` whole: `app-git` is not named by `app-gitlab`.
fn names_whole(line: &str, name: &str) -> bool {
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    line.match_indices(name)
        .any(|(at, _)| name.ends_with('"') || !line[at + name.len()..].starts_with(word))
}

/// Whether `text` has `word` with no letter, digit or `_` either side.
fn has_word(text: &str, word: &str) -> bool {
    let inner = |c: char| c.is_ascii_alphanumeric() || c == '_';
    text.match_indices(word)
        .any(|(at, _)| !text[..at].ends_with(inner) && !text[at + word.len()..].starts_with(inner))
}

/// Lines in `k`'s sources that contain any of `words`, ignoring case: anywhere
/// in the line, or as a whole word when `whole` is set.
fn lines_naming(k: &Krate, words: &[&str], whole: bool) -> Vec<String> {
    let mut found = Vec::new();
    for (path, text) in sources(&k.dir) {
        for (n, line) in text.lines().enumerate() {
            let lower = line.to_lowercase();
            for w in words {
                if if whole {
                    has_word(&lower, w)
                } else {
                    lower.contains(w)
                } {
                    found.push(format!(
                        "{}:{}: {w}: {}",
                        path.display(),
                        n + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
    found
}

#[test]
fn no_part_depends_on_another_and_core_depends_on_none() {
    let ws = workspace();
    for k in ws.iter().filter(|k| is_part(&k.name) || k.name == core()) {
        for d in &k.deps {
            assert!(
                !(is_part(d) && *d != k.name && !is_adapter_of(&k.name, d)),
                "{} depends on the part {d}: parts meet only through {}",
                k.name,
                core()
            );
        }
    }
    let c = ws
        .iter()
        .find(|k| k.name == core())
        .expect("the core crate");
    let inside: Vec<&String> = c.deps.iter().filter(|d| d.starts_with(APP)).collect();
    assert!(inside.is_empty(), "{} depends on {inside:?}", core());
}

#[test]
fn an_adapter_depends_on_its_family_only() {
    let ws = workspace();
    let parts: Vec<&str> = ws
        .iter()
        .map(|k| k.name.as_str())
        .filter(|n| is_part(n))
        .collect();
    for k in &ws {
        let Some(family) = parts
            .iter()
            .find(|p| **p != k.name && is_adapter_of(&k.name, p))
        else {
            continue;
        };
        let inside: Vec<&String> = k
            .deps
            .iter()
            .filter(|d| d.starts_with(APP) && d.as_str() != *family)
            .collect();
        assert!(
            inside.is_empty(),
            "{} is an adapter of {family} and may depend on it alone, not on {inside:?}",
            k.name
        );
    }
}

#[test]
fn no_part_names_another_and_core_names_none() {
    let ws = workspace();
    let parts: Vec<String> = ws
        .iter()
        .map(|k| k.name.clone())
        .filter(|n| is_part(n))
        .collect();
    let mut found = Vec::new();
    for k in ws.iter().filter(|k| is_part(&k.name) || k.name == core()) {
        let others = parts
            .iter()
            .filter(|p| **p != k.name && !is_adapter_of(&k.name, p));
        for other in others {
            for (path, text) in sources(&k.dir) {
                for name in names_of(other) {
                    for (n, line) in text.lines().enumerate() {
                        if names_whole(line, &name) {
                            found.push(format!(
                                "{}:{}: names {other} ({name})",
                                path.display(),
                                n + 1
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "parts must not name each other:\n{}",
        found.join("\n")
    );
}

#[test]
fn core_names_no_tool_or_service() {
    let ws = workspace();
    let core = ws
        .iter()
        .find(|k| k.name == core())
        .expect("the core crate");
    let found = lines_naming(core, &DOMAIN_WORDS, false);
    assert!(
        found.is_empty(),
        "{} names a tool or service:\n{}",
        core.name,
        found.join("\n")
    );
}

#[test]
fn no_ui_library_outside_a_front_end() {
    let ws = workspace();
    let mut found = Vec::new();
    for k in ws.iter().filter(|k| is_part(&k.name) || k.name == core()) {
        found.extend(lines_naming(k, &UI_WORDS, true));
        found.extend(
            k.deps
                .iter()
                .filter(|d| UI_WORDS.iter().any(|w| d.contains(w)))
                .map(|d| format!("{} depends on {d}", k.name)),
        );
    }
    assert!(
        found.is_empty(),
        "only a front end may use a UI library:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_root_registers_each_part_behind_its_feature() {
    let ws = workspace();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("Cargo.toml");
    let lib = std::fs::read_to_string(root.join("src/lib.rs")).expect("src/lib.rs");
    for part in ws.iter().map(|k| k.name.as_str()).filter(|n| is_part(n)) {
        let id = id_of(part);
        let owner = ws
            .iter()
            .map(|k| k.name.as_str())
            .find(|p| is_part(p) && *p != part && is_adapter_of(part, p));
        let want = match owner {
            Some(o) => format!("{id} = [\"{}\", \"dep:{part}\"]", id_of(o)),
            None => format!("{id} = [\"dep:{part}\"]"),
        };
        assert!(
            manifest.contains(&want),
            "Cargo.toml: the feature `{want}` is missing"
        );
        assert!(
            lib.contains(&format!("#[cfg(feature = \"{id}\")]")),
            "src/lib.rs: {part} is not registered behind `#[cfg(feature = \"{id}\")]`"
        );
    }
}

#[test]
fn the_app_depends_on_core_and_style_only() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(root.join("app/Cargo.toml")).expect("app/Cargo.toml");
    let manifest: toml::Table = text.parse().expect("app/Cargo.toml is TOML");
    assert!(
        manifest.contains_key("workspace"),
        "app/Cargo.toml needs its own [workspace] table, so the main workspace does not build its UI library"
    );
    let members: Vec<String> = workspace().into_iter().map(|k| k.name).collect();
    let allowed = [core(), format!("{APP}-style")];
    let mut found = Vec::new();
    for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(deps) = manifest.get(table).and_then(toml::Value::as_table) else {
            continue;
        };
        for (name, spec) in deps {
            let package = spec
                .get("package")
                .and_then(toml::Value::as_str)
                .unwrap_or(name);
            let path = spec.get("path").and_then(toml::Value::as_str);
            let ours = members.iter().any(|m| m == package)
                || package.starts_with(&format!("{APP}-"))
                || path.is_some();
            if ours && !allowed.iter().any(|a| a == package) {
                found.push(format!("[{table}] {name}"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "the app may depend on {} only:\n{}",
        allowed.join(" and "),
        found.join("\n")
    );
}
