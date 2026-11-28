//! The checkout part against a real repository: a bare one under
//! `CARGO_TARGET_TMPDIR` stands for the code host, and a clone of it for the
//! work folder a checkout is added from.

use std::path::{Path, PathBuf};
use std::process::Command;

use temet_work_checkout::{Checkouts, Real};
use temet_work_core::ids::{AgentId, GroupId, MachineId, ProjectId};
use temet_work_core::model::Checkout;
use temet_work_core::world::LOCAL;
use temet_work_core::{Effect, Event, Outcome, Part, World};

/// What every git the tests run is given: no fsync, which costs half a second a
/// command on a busy disk and keeps nothing a test needs.
fn quick() -> Vec<(String, String)> {
    [
        ("GIT_CONFIG_COUNT", "1"),
        ("GIT_CONFIG_KEY_0", "core.fsync"),
        ("GIT_CONFIG_VALUE_0", "none"),
    ]
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .into()
}

/// The checkout part, running git as the tests do.
fn part() -> Checkouts {
    Checkouts::new(Box::new(Real::with_env(quick())))
}

/// Runs git in `dir` with a fixed identity and no signing, so a person's own git
/// configuration does not change what the test sees.
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
        "git {args:?} in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A code host and a work folder cloned from it, under a fresh folder named
/// `name`. The default branch `main` ignores `target/`, and a pull request's
/// branch `watch-12-night-shift` is one commit ahead of it, pushed from another
/// clone, so the work folder has not fetched it.
struct Repo {
    root: PathBuf,
    bare: PathBuf,
    work: PathBuf,
}

impl Repo {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checkout-{name}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("root");
        let bare = root.join("watch.git");
        git(&root, &["init", "--quiet", "--bare", "watch.git"]);
        git(&root, &["clone", "--quiet", "watch.git", "author"]);
        let author = root.join("author");
        std::fs::write(author.join(".gitignore"), "target/\n").expect("ignore");
        std::fs::write(author.join("clock.rs"), "fn strike() {}\n").expect("file");
        git(&author, &["add", "."]);
        git(&author, &["commit", "--quiet", "-m", "The clock"]);
        git(&author, &["push", "--quiet", "origin", "HEAD:main"]);
        git(&root, &["clone", "--quiet", "watch.git", "work"]);
        git(
            &author,
            &["switch", "--quiet", "-c", "watch-12-night-shift"],
        );
        std::fs::write(author.join("rota.rs"), "fn night() {}\n").expect("file");
        git(&author, &["add", "."]);
        git(&author, &["commit", "--quiet", "-m", "A night shift rota"]);
        git(
            &author,
            &["push", "--quiet", "origin", "watch-12-night-shift"],
        );
        Self {
            work: root.join("work"),
            root,
            bare,
        }
    }

    /// The checkout `name` for reviewer-1, beside the work folder, at `branch`.
    fn checkout(&self, name: &str, branch: Option<&str>) -> Checkout {
        Checkout {
            name: name.into(),
            agent: AgentId::new("reviewer-1"),
            project: ProjectId::new("watch"),
            machine: MachineId::new(LOCAL),
            path: self
                .root
                .join("temet-work-checkouts/l1")
                .join(name)
                .display()
                .to_string(),
            from: self.work.display().to_string(),
            branch: branch.map(String::from),
            kept: None,
        }
    }

    /// The commit the code host's `branch` is at.
    fn head_of(&self, branch: &str) -> String {
        git(&self.bare, &["rev-parse", &format!("refs/heads/{branch}")])
    }

    /// The worktrees git lists for the work folder's repository.
    fn worktrees(&self) -> String {
        git(&self.work, &["worktree", "list", "--porcelain"])
    }
}

fn then() -> Effect {
    Effect::Investigate {
        group: GroupId::new("watch"),
    }
}

fn add(part: &mut Checkouts, c: &Checkout) -> Outcome {
    part.carry_out(
        &Effect::AddCheckout {
            checkout: c.clone(),
            then: Box::new(then()),
        },
        &World::default(),
    )
}

fn remove(part: &mut Checkouts, c: &Checkout) -> Outcome {
    part.carry_out(
        &Effect::RemoveCheckout {
            checkout: c.clone(),
        },
        &World::default(),
    )
}

fn added(c: &Checkout) -> Outcome {
    Outcome::Done {
        events: vec![Event::CheckoutAdded {
            checkout: c.clone(),
        }],
        effects: vec![then()],
    }
}

fn removed(c: &Checkout) -> Outcome {
    Outcome::Done {
        events: vec![Event::CheckoutRemoved {
            name: c.name.clone(),
        }],
        effects: vec![],
    }
}

#[test]
fn adds_a_detached_worktree_at_a_branchs_fetched_head_and_one_at_the_default_branchs() {
    let repo = Repo::new("add");
    let status_before = git(&repo.work, &["status", "--porcelain"]);
    let mut part = part();
    let review = repo.checkout("watch-1", Some("watch-12-night-shift"));
    assert_eq!(add(&mut part, &review), added(&review));
    let at = Path::new(&review.path);
    assert_eq!(
        git(at, &["rev-parse", "HEAD"]),
        repo.head_of("watch-12-night-shift")
    );
    assert!(at.join("rota.rs").exists());
    // Detached, so the branch stays free for a work folder to check out.
    assert_eq!(git(at, &["rev-parse", "--abbrev-ref", "HEAD"]), "HEAD");
    let refine = repo.checkout("watch-2", None);
    assert_eq!(add(&mut part, &refine), added(&refine));
    assert_eq!(
        git(Path::new(&refine.path), &["rev-parse", "HEAD"]),
        repo.head_of("main")
    );
    // The work folder's own files are as they were.
    assert_eq!(git(&repo.work, &["status", "--porcelain"]), status_before);
    assert_eq!(
        git(&repo.work, &["rev-parse", "--abbrev-ref", "HEAD"]),
        "main"
    );
    let listed = repo.worktrees();
    assert!(listed.contains(&review.path), "{listed}");
    assert!(listed.contains(&refine.path), "{listed}");
}

#[test]
fn removes_a_clean_worktree_with_a_build_and_a_real_run_under_target() {
    let repo = Repo::new("remove");
    let mut part = part();
    let c = repo.checkout("watch-1", Some("watch-12-night-shift"));
    assert_eq!(add(&mut part, &c), added(&c));
    let target = Path::new(&c.path).join("target");
    std::fs::create_dir_all(target.join("debug")).expect("build");
    std::fs::write(target.join("debug/temet-workd"), "a build").expect("build");
    std::fs::create_dir_all(target.join("real-run")).expect("run");
    std::fs::write(target.join("real-run/events.jsonl"), "{}\n").expect("run");
    assert_eq!(remove(&mut part, &c), removed(&c));
    assert!(!Path::new(&c.path).exists());
    assert!(!repo.worktrees().contains(&c.path), "{}", repo.worktrees());
    // The empty folders it was in go with it.
    assert!(!repo.root.join("temet-work-checkouts").exists());
    // One already gone is reported removed.
    assert_eq!(remove(&mut part, &c), removed(&c));
}

/// Adds a checkout, makes `change` in it, and asks for its removal: it is kept
/// with what `found` says, and nothing in it is deleted.
fn kept_after(name: &str, change: fn(&Path), found: &str) {
    let repo = Repo::new(name);
    let mut part = part();
    let c = repo.checkout("watch-1", Some("watch-12-night-shift"));
    assert_eq!(add(&mut part, &c), added(&c));
    let at = PathBuf::from(&c.path);
    change(&at);
    let before = git(&at, &["status", "--porcelain"]);
    let head = git(&at, &["rev-parse", "HEAD"]);
    assert_eq!(
        remove(&mut part, &c),
        Outcome::Done {
            events: vec![Event::CheckoutKept {
                name: c.name.clone(),
                changes: vec![found.to_string()],
            }],
            effects: vec![],
        }
    );
    assert!(at.exists());
    assert_eq!(git(&at, &["status", "--porcelain"]), before);
    assert_eq!(git(&at, &["rev-parse", "HEAD"]), head);
    assert!(repo.worktrees().contains(&c.path));
}

#[test]
fn keeps_a_worktree_with_an_uncommitted_change() {
    kept_after(
        "keep-edited",
        |at| std::fs::write(at.join("clock.rs"), "fn strike() { thirteen() }\n").expect("edit"),
        "1 file changed, or new and not ignored: clock.rs",
    );
}

#[test]
fn keeps_a_worktree_with_a_file_git_does_not_ignore() {
    kept_after(
        "keep-untracked",
        |at| std::fs::write(at.join("notes.txt"), "a reviewer's notes\n").expect("new"),
        "1 file changed, or new and not ignored: notes.txt",
    );
}

#[test]
fn keeps_a_worktree_with_a_commit_no_remote_tracking_branch_holds() {
    kept_after(
        "keep-committed",
        |at| {
            std::fs::write(at.join("rota.rs"), "fn night() { watch() }\n").expect("edit");
            git(at, &["commit", "--quiet", "-am", "A reviewer's own commit"]);
        },
        "1 commit no remote-tracking branch holds",
    );
}

#[test]
fn a_reviewer_who_switched_to_a_new_head_that_was_then_pruned_is_still_removed() {
    let repo = Repo::new("pruned");
    let mut part = part();
    let c = repo.checkout("watch-1", Some("watch-12-night-shift"));
    assert_eq!(add(&mut part, &c), added(&c));
    let at = PathBuf::from(&c.path);
    // The branch is deleted after a squash merge, and a fetch prunes it: the
    // checkout's head is held by no remote-tracking branch, but nothing in it
    // was made there.
    git(&repo.bare, &["branch", "-D", "watch-12-night-shift"]);
    git(&at, &["fetch", "--quiet", "--prune", "origin"]);
    assert_eq!(remove(&mut part, &c), removed(&c));
    assert!(!at.exists());
}
