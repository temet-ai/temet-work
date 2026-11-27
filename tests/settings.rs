//! What the code and `docs/settings.md` say about each setting agrees with its
//! `Spec`:
//!
//! - a reader passes a role only for a setting read per role, and no role only
//!   for one that is the same for every role;
//! - the table of settings says which take a role.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use temet_work_core::settings::{spec, specs};

/// The methods of the world that read a setting; the key is their last argument.
/// The role is the one before it, for those that take one.
const READERS: [(&str, bool); 7] = [
    ("setting", true),
    ("setting_text", true),
    ("setting_number", true),
    ("setting_flag", true),
    ("resolve", true),
    ("source_number", false),
    ("repo_flag", false),
];

/// Where the readers are written, so they pass a key on.
const RESOLVER: &str = "crates/core/src/settings.rs";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file of the root package's and each crate's `src`, tests left out,
/// by its path in the repository.
fn sources() -> Vec<(String, String)> {
    let mut todo: Vec<PathBuf> = std::fs::read_dir(root().join("crates"))
        .expect("crates")
        .flatten()
        .map(|e| e.path().join("src"))
        .collect();
    todo.push(root().join("src"));
    let mut out = Vec::new();
    while let Some(dir) = todo.pop() {
        for e in std::fs::read_dir(&dir).expect("a source folder").flatten() {
            let path = e.path();
            if path.is_dir() {
                todo.push(path);
            } else if path.extension().is_some_and(|x| x == "rs")
                && !path.to_string_lossy().ends_with("_tests.rs")
            {
                let name = path.strip_prefix(root()).expect("inside the repository");
                let text = std::fs::read_to_string(&path).expect("readable");
                out.push((name.display().to_string(), without_tests(&text)));
            }
        }
    }
    out.sort();
    out
}

/// `text` with its comment lines and `#[cfg(test)]` modules left blank, so line
/// numbers stay. rustfmt closes a module with a brace at its own indentation.
fn without_tests(text: &str) -> String {
    let mut out = Vec::new();
    // The line that closes the test module being left out.
    let mut closes: Option<String> = None;
    let mut test_only = false;
    for line in text.lines() {
        let code = line.trim_start();
        let comment = code.starts_with("//");
        if let Some(end) = &closes {
            if line == end {
                closes = None;
            }
            out.push("");
            continue;
        }
        if test_only && !comment && !code.starts_with("#[") {
            test_only = false;
            if code.starts_with("mod ") && code.ends_with('{') {
                closes = Some(format!("{}}}", &line[..line.len() - code.len()]));
                out.push("");
                continue;
            }
        }
        test_only |= code == "#[cfg(test)]";
        out.push(if comment { "" } else { line });
    }
    out.join("\n")
}

/// The arguments of a call, from the text after its opening bracket.
fn arguments(after: &str) -> Vec<String> {
    let mut args = vec![String::new()];
    let (mut depth, mut quoted, mut escaped) = (0, false, false);
    for c in after.chars() {
        if quoted {
            quoted = escaped || c != '"';
            escaped = !escaped && c == '\\';
        } else {
            match c {
                '"' => quoted = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' if depth == 0 => break,
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    args.push(String::new());
                    continue;
                }
                _ => {}
            }
        }
        args.last_mut().expect("one at least").push(c);
    }
    args.iter()
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .collect()
}

#[test]
fn readers_match_per_role() {
    let mut wrong = Vec::new();
    let mut read = BTreeSet::new();
    for (file, text) in sources() {
        for (name, takes_role) in READERS {
            // The resolver asks for the global runtime itself, for every role.
            if file == RESOLVER && name == "resolve" {
                continue;
            }
            let call = format!("{name}(");
            for (at, _) in text.match_indices(&call) {
                if !(text[..at].ends_with('.') || text[..at].ends_with("World::")) {
                    continue;
                }
                let args = arguments(&text[at + call.len()..]);
                let line = text[..at].matches('\n').count() + 1;
                let here = format!("{file}:{line}: {name}");
                let key = args.last().map_or("", String::as_str);
                let Some(key) = key.strip_prefix('"').and_then(|k| k.strip_suffix('"')) else {
                    if file != RESOLVER && name != "resolve" {
                        wrong.push(format!("{here} reads {key}; write the key as a literal"));
                    }
                    continue;
                };
                let Ok(spec) = spec(key) else {
                    wrong.push(format!("{here} reads {key}, which is not a setting"));
                    continue;
                };
                read.insert(spec.key);
                let role = if takes_role {
                    args.iter().rev().nth(1).map_or("", String::as_str)
                } else {
                    "None"
                };
                let fits = if spec.per_role {
                    role.starts_with("Some(")
                } else {
                    role == "None"
                };
                if !fits {
                    wrong.push(format!(
                        "{here} reads {key} with role {role}, but its Spec has per_role: {}",
                        spec.per_role
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    // A setting the scan saw no reader of is one it checked nothing about.
    let unread: Vec<&str> = specs()
        .map(|s| s.key)
        .filter(|k| !read.contains(k))
        .collect();
    assert!(unread.is_empty(), "no reader found for {unread:?}");
}

#[test]
fn docs_settings_table_matches_specs() {
    let doc = include_str!("../docs/settings.md");
    let (_, section) = doc
        .split_once("\n## The settings\n")
        .expect("the section is there");
    let mut rows = section
        .lines()
        .skip_while(|l| !l.starts_with('|'))
        .take_while(|l| l.starts_with('|'))
        .map(|l| {
            l.trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>()
        });
    let head = rows.next().expect("a table");
    let column = head
        .iter()
        .position(|c| *c == "Per role")
        .expect("a Per role column");
    let mut listed = Vec::new();
    for row in rows.skip(1) {
        // The keys are what is between backticks in the first cell.
        for key in row[0].split('`').skip(1).step_by(2) {
            let spec = spec(key).expect("the table names settings only");
            let want = if spec.per_role { "yes" } else { "no" };
            assert_eq!(row[column], want, "{key}");
            listed.push(spec.key);
        }
    }
    listed.sort_unstable();
    let mut keys: Vec<&str> = specs().map(|s| s.key).collect();
    keys.sort_unstable();
    assert_eq!(listed, keys, "each setting is in the table once");
}
