//! Runs the UI review check over the front-end crate, `app/src`, and the layer map,
//! `docs/ui.md`. Until `app/` exists there are no files to review; the unit tests in
//! `src/review.rs` show each rule catching what it names.

use std::path::{Path, PathBuf};

use temet_work_style::review::{parse_map, review, unknown_parts};
use temet_work_style::tokens::PARTS;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.rs` file under `dir`, as its path below `dir` with `/` between folders, and its text.
fn files(dir: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut todo = vec![dir.to_path_buf()];
    while let Some(d) = todo.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let path = e.path();
            if path.is_dir() {
                todo.push(path);
            } else if path.extension().is_some_and(|x| x == "rs") {
                let rel = path.strip_prefix(dir).expect("under dir");
                let name: Vec<String> = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                let text = std::fs::read_to_string(&path).expect("a source file reads");
                out.push((name.join("/"), text));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn the_app_keeps_the_layer_map() {
    let map = std::fs::read_to_string(root().join("docs/ui.md")).expect("docs/ui.md reads");
    let parsed = parse_map(&map);
    assert!(
        parsed.components.len() >= 30 && parsed.composites.len() >= 10 && parsed.pages.len() >= 2,
        "docs/ui.md's Components, Composites and Pages tables did not parse: {} / {} / {} rows",
        parsed.components.len(),
        parsed.composites.len(),
        parsed.pages.len()
    );
    let unknown = unknown_parts(&parsed, &PARTS);
    assert!(
        unknown.is_empty(),
        "docs/ui.md names parts the design system does not have:\n{}",
        unknown.join("\n")
    );
    let findings = review(&files(&root().join("app/src")), &map);
    let lines: Vec<String> = findings.iter().map(ToString::to_string).collect();
    assert!(
        lines.is_empty(),
        "the app breaks docs/ui.md:\n{}",
        lines.join("\n")
    );
}

/// The spike styles by hand, as code written before the map did. Read as plumbing, its
/// `main.rs` shows the rules catching real UI code, not only the unit tests' fixtures.
#[test]
fn the_check_catches_the_spikes_hand_styling() {
    let map = std::fs::read_to_string(root().join("docs/ui.md")).expect("docs/ui.md reads");
    let spike = files(&root().join("spikes/gpui-shell/src"));
    let findings: Vec<String> = review(&spike, &map)
        .iter()
        .map(ToString::to_string)
        .collect();
    for want in [
        "a text size from the library's scale: `text_xs(`",
        "draws its own `div()`",
    ] {
        assert!(
            findings.iter().any(|f| f.contains(want)),
            "no finding with {want:?} in:\n{}",
            findings.join("\n")
        );
    }
}
