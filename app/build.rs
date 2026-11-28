//! Gives the app a build id, as the root package's build.rs gives the daemon and the CLI one:
//! the checkout's commit, with `-dirty` when it has uncommitted changes, as
//! `git describe --always --dirty` says it.

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
    println!("cargo:rustc-env=TEMET_WORK_APP_BUILD={id}");
    for path in ["src", "assets", "Cargo.toml", "Cargo.lock", "../crates"] {
        println!("cargo:rerun-if-changed={path}");
    }
    // A commit that changes no file still moves the checkout's head.
    if let Ok(out) = std::process::Command::new("git")
        .args(["rev-parse", "--git-path", "HEAD"])
        .output()
    {
        let head = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !head.is_empty() {
            println!("cargo:rerun-if-changed={head}");
        }
    }
}
