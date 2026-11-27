//! Gives the binaries a build id: the checkout's commit, with `-dirty` when it
//! has uncommitted changes, as `git describe --always --dirty` says it. A
//! deploy that restarts the daemon records the id of what it installed, and the
//! daemon that starts with that id knows the deploy is done.

fn main() {
    let id = std::process::Command::new("git")
        .args(["describe", "--always", "--dirty"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=TEMET_WORK_BUILD={id}");
    // Without a rerun-if line, cargo reruns this when any file in the package
    // changes; a commit that changes no file is caught by the head below.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=crates");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
}
