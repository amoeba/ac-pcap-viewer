use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use duct::cmd;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "xtask", about = "Build tasks for ac-pcap-parser")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the web UI (WASM)
    Web {
        /// Start a local web server after building
        #[arg(long)]
        serve: bool,
        /// Port for web server (default: 8080)
        #[arg(long, default_value = "8080")]
        port: u16,
        /// Use release-wasm profile for smaller builds
        #[arg(long)]
        small: bool,
    },
    /// Build the desktop application
    Desktop {
        /// Build in release mode
        #[arg(long)]
        release: bool,
        /// Run the application after building
        #[arg(long)]
        run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Web { serve, port, small } => build_web(serve, port, small),
        Commands::Desktop { release, run } => build_desktop(release, run),
    }
}

/// Find the project root directory
fn project_root() -> Result<PathBuf> {
    // Use cargo locate-project which returns the path to Cargo.toml
    let output = cmd!(
        "cargo",
        "locate-project",
        "--workspace",
        "--message-format",
        "plain"
    )
    .read()
    .context("Failed to locate project root with cargo locate-project")?;

    let cargo_toml_path = PathBuf::from(output.trim());
    let root = cargo_toml_path
        .parent()
        .context("Failed to get parent directory of Cargo.toml")?
        .to_path_buf();

    Ok(root)
}

/// Run a command with proper error handling and output
fn run_command(program: &str, args: &[&str]) -> Result<()> {
    println!("Running: {} {}", program, args.join(" "));

    let root = project_root()?;
    let command = cmd(program, args)
        .dir(&root)
        .stdin_null()
        .stdout_to_stderr()
        .run()
        .with_context(|| format!("Failed to run command: {} {}", program, args.join(" ")))?;

    if !command.status.success() {
        bail!(
            "Command failed with exit code: {}",
            command.status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

fn build_web(serve: bool, port: u16, small: bool) -> Result<()> {
    let root = project_root()?;
    let pkg_dir = root.join("crates/web/pkg");

    // Clean and recreate pkg directory (removes stale files from previous builds)
    if pkg_dir.exists() {
        std::fs::remove_dir_all(&pkg_dir).context("Failed to clean pkg directory")?;
    }
    std::fs::create_dir_all(&pkg_dir).context("Failed to create pkg directory")?;

    // Build WASM
    println!("Building WASM...");
    let mut args = vec!["build", "-p", "web", "--target", "wasm32-unknown-unknown"];
    let profile_dir = if small {
        args.push("--profile=release-wasm");
        "release-wasm"
    } else {
        args.push("--release");
        "release"
    };

    run_command("cargo", &args)?;

    // Run wasm-bindgen
    println!("Generating JS bindings...");
    let wasm_file = root.join(format!(
        "target/wasm32-unknown-unknown/{profile_dir}/web.wasm"
    ));

    let pkg_dir_str = pkg_dir.to_string_lossy().to_string();
    let wasm_file_str = wasm_file.to_string_lossy().to_string();

    run_command(
        "wasm-bindgen",
        &[
            "--target",
            "web",
            "--out-dir",
            &pkg_dir_str,
            "--no-typescript",
            &wasm_file_str,
        ],
    )
    .context("Failed to run wasm-bindgen. Is it installed? Run: cargo install wasm-bindgen-cli")?;

    // Apply cache busting to generated files
    println!("Applying cache busting...");
    let hash = apply_cache_busting(&pkg_dir)?;
    println!("  Content hash: {hash}");

    // Copy index.html with cache-busted references
    println!("Copying assets...");
    let web_root = root.join("crates/web");
    let index_src = web_root.join("index.html");
    let index_content = std::fs::read_to_string(&index_src).context("Failed to read index.html")?;
    let index_content = index_content.replace("./web.js", &format!("./web.{hash}.js"));
    let index_dst = pkg_dir.join("index.html");
    std::fs::write(&index_dst, index_content).context("Failed to write index.html")?;

    // Copy example PCAP
    let pcap_src = root.join("pkt_2025-11-18_1763490291_log.pcap");
    let pcap_dst = pkg_dir.join("example.pcap");
    if pcap_src.exists() {
        std::fs::copy(&pcap_src, &pcap_dst).context("Failed to copy example.pcap")?;
    }

    println!("\nBuild complete! Files in crates/web/pkg/");

    // List files
    for entry in std::fs::read_dir(&pkg_dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let size = metadata.len();
        let size_str = if size > 1024 * 1024 {
            format!("{:.1}M", size as f64 / 1024.0 / 1024.0)
        } else if size > 1024 {
            format!("{:.1}K", size as f64 / 1024.0)
        } else {
            format!("{size}B")
        };
        println!("  {:>8}  {}", size_str, entry.file_name().to_string_lossy());
    }

    if serve {
        println!("\nStarting web server on http://localhost:{port}");
        println!("Press Ctrl+C to stop");

        run_command("python3", &["-m", "http.server", &port.to_string()])
            .context("Failed to start web server. Is python3 installed?")?;
    } else {
        println!("\nTo test locally:");
        println!("  cargo xtask web --serve");
    }

    Ok(())
}

fn build_desktop(release: bool, run: bool) -> Result<()> {
    let root = project_root()?;

    println!("Building desktop application...");

    let mut args = vec!["build", "-p", "app", "--bin", "ac-pcap-viewer"];
    if release {
        args.push("--release");
    }

    run_command("cargo", &args)?;

    let profile = if release { "release" } else { "debug" };
    let binary_path = root.join(format!("target/{profile}/ac-pcap-viewer"));

    println!("\nBuild complete!");
    println!("  Binary: {}", binary_path.display());

    if run {
        println!("\nRunning application...");
        let status = Command::new(&binary_path)
            .current_dir(&root)
            .status()
            .context("Failed to run application")?;

        if !status.success() {
            bail!("Application exited with error");
        }
    } else {
        println!("\nTo run:");
        println!("  cargo xtask desktop --run");
        println!("  # or directly:");
        println!("  {}", binary_path.display());
    }

    Ok(())
}

/// Apply cache busting by renaming files with content hash
/// Returns the hash used for cache busting
fn apply_cache_busting(pkg_dir: &Path) -> Result<String> {
    // Read the wasm file and compute its hash
    let wasm_path = pkg_dir.join("web_bg.wasm");
    let wasm_content = std::fs::read(&wasm_path).context("Failed to read web_bg.wasm")?;

    let mut hasher = Sha256::new();
    hasher.update(&wasm_content);
    let hash_bytes = hasher.finalize();
    let hash = hex::encode(&hash_bytes[..8]); // First 8 bytes = 16 hex chars

    // Read the JS file and update the wasm reference
    let js_path = pkg_dir.join("web.js");
    let js_content = std::fs::read_to_string(&js_path).context("Failed to read web.js")?;
    let js_content = js_content.replace("web_bg.wasm", &format!("web_bg.{hash}.wasm"));

    // Write the new JS file with hash in name
    let new_js_path = pkg_dir.join(format!("web.{hash}.js"));
    std::fs::write(&new_js_path, js_content).context("Failed to write hashed web.js")?;

    // Rename the wasm file with hash
    let new_wasm_path = pkg_dir.join(format!("web_bg.{hash}.wasm"));
    std::fs::rename(&wasm_path, &new_wasm_path).context("Failed to rename wasm file")?;

    // Remove old JS file (keep pkg clean)
    std::fs::remove_file(&js_path).context("Failed to remove old web.js")?;

    Ok(hash)
}
