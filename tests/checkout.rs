//! The daemon's first round after a restart removes a checkout whose reader
//! ended without its removal being carried out: a host built from such a log,
//! with the real checkout part over a real repository, ticks once and the
//! checkout's folder is gone.

#![cfg(feature = "checkout")]

use std::path::{Path, PathBuf};
use std::process::Command;

use temet_work_checkout::{Checkouts, Real};
use temet_work_core::ids::{AgentId, FolderId, GroupId, IssueId, MachineId, ProjectId};
use temet_work_core::model::{AgentCheckout, AgentState, Checkout, Role};
use temet_work_core::world::LOCAL;
use temet_work_core::{Event, Host, World};

/// No fsync for the git the test runs: it costs a busy disk half a second a
/// command and keeps nothing the test needs.
fn quick() -> Vec<(String, String)> {
    [
        ("GIT_CONFIG_COUNT", "1"),
        ("GIT_CONFIG_KEY_0", "core.fsync"),
        ("GIT_CONFIG_VALUE_0", "none"),
    ]
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .into()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .envs(quick())
        .args([
            "-c",
            "user.name=Sam Vimes",
            "-c",
            "user.email=vimes@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn the_first_round_removes_an_ended_readers_checkout_left_on_record() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("checkout-swept-at-start");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("root");
    git(&root, &["init", "--quiet", "--bare", "watch.git"]);
    git(&root, &["clone", "--quiet", "watch.git", "w1"]);
    let work = root.join("w1");
    // Git ignores target/, as in the repository itself, so a build there does
    // not keep the checkout.
    std::fs::write(work.join(".gitignore"), "target/\n").expect("ignore");
    std::fs::write(work.join("clock.rs"), "fn strike() {}\n").expect("file");
    git(&work, &["add", "."]);
    git(&work, &["commit", "--quiet", "-m", "The clock"]);
    git(&work, &["push", "--quiet", "origin", "HEAD:main"]);
    // reviewer-1's checkout, made as the part makes one, with a build in it.
    let path = root.join("temet-work-checkouts/l1/watch-1");
    let path_text = path.display().to_string();
    git(
        &work,
        &[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            &path_text,
            "origin/main",
        ],
    );
    std::fs::create_dir_all(path.join("target/debug")).expect("build");
    std::fs::write(path.join("target/debug/watch"), "a build").expect("build");
    let checkout = Checkout {
        name: "watch-1".into(),
        agent: AgentId::new("reviewer-1"),
        project: ProjectId::new("watch"),
        machine: MachineId::new(LOCAL),
        path: path_text.clone(),
        from: work.display().to_string(),
        branch: None,
        kept: None,
    };
    // The log of a daemon that stopped after reviewer-1 ended and before its
    // checkout was removed.
    let log = [
        Event::LogStarted { id: "l1".into() },
        Event::ProjectAdded {
            project: ProjectId::new("watch"),
            tracker: String::new(),
            name: "ankh-morpork/watch".into(),
            code_host: None,
            repo: None,
        },
        Event::FolderAdded {
            folder: FolderId::new("w1"),
            project: ProjectId::new("watch"),
            machine: MachineId::new(LOCAL),
            path: work.display().to_string(),
        },
        Event::CheckoutAdded {
            checkout: checkout.clone(),
        },
        Event::AgentStarted {
            agent: AgentId::new("reviewer-1"),
            group: GroupId::new("watch"),
            role: Role::Reviewer,
            issue: Some(IssueId::new("watch", "12")),
            machine: MachineId::new(LOCAL),
            folder: None,
            secret_digest: None,
            runtime: None,
            model: None,
            checkouts: vec![AgentCheckout {
                project: ProjectId::new("watch"),
                path: path_text.clone(),
            }],
        },
        Event::AgentChanged {
            agent: AgentId::new("reviewer-1"),
            state: AgentState::Ended,
            lost: false,
        },
    ];
    let (world, _) = World::replay(&log);
    assert!(world.checkouts.contains_key("watch-1"));
    let part = Checkouts::new(Box::new(Real::with_env(quick())));
    let mut host = Host::new(
        world,
        vec![Box::new(part)],
        Box::new(temet_work_core::host::Memory::default()),
    );
    host.tick(1_000).expect("tick");
    assert!(!path.exists(), "{path_text} is still there");
    let listed = git(&work, &["worktree", "list", "--porcelain"]);
    assert!(!listed.contains(&path_text), "{listed}");
    assert!(host.world().checkouts.is_empty());
    assert!(host.problems().is_empty(), "{:?}", host.problems());
}
