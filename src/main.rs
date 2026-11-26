//! `temet-work`: prints the version and the parts compiled into this build.
//! Nothing runs yet; see docs/architecture.md for what comes next.

fn main() {
    let parts = temet_work::parts();
    let ids: Vec<&str> = parts.iter().map(|p| p.id()).collect();
    println!(
        "temet-work {} (parts: {})",
        env!("CARGO_PKG_VERSION"),
        if ids.is_empty() {
            "none".to_string()
        } else {
            ids.join(", ")
        }
    );
}
