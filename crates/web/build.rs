use std::process::Command;

fn main() {
    // Try to get GIT_SHA from environment (set by Docker/CI)
    // Fall back to running git command for local builds
    let git_sha = std::env::var("GIT_SHA").unwrap_or_else(|_| {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "dev".to_string())
    });

    println!("cargo:rustc-env=GIT_SHA={}", git_sha);
    println!("cargo:rerun-if-env-changed=GIT_SHA");
}
