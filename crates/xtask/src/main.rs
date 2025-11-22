use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Web { serve, port, small } => build_web(serve, port, small),
    }
}

fn project_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    PathBuf::from(manifest_dir)
        .ancestors()
        .nth(2)
        .expect("Failed to find project root")
        .to_path_buf()
}

fn build_web(serve: bool, port: u16, small: bool) -> Result<()> {
    let root = project_root();
    let web_dir = root.join("crates/web");
    let pkg_dir = web_dir.join("pkg");

    // Ensure pkg directory exists
    std::fs::create_dir_all(&pkg_dir).context("Failed to create pkg directory")?;

    // Build WASM
    println!("Building WASM...");
    let mut args = vec![
        "build",
        "-p",
        "web",
        "--target",
        "wasm32-unknown-unknown",
    ];
    if small {
        args.push("--profile=release-wasm");
    } else {
        args.push("--release");
    }
    let status = Command::new("cargo")
        .args(&args)
        .current_dir(&root)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        bail!("cargo build failed");
    }

    // Run wasm-bindgen
    println!("Generating JS bindings...");
    let wasm_file = root
        .join("target/wasm32-unknown-unknown/release/web.wasm");

    let status = Command::new("wasm-bindgen")
        .args([
            "--target",
            "web",
            "--out-dir",
            pkg_dir.to_str().unwrap(),
            "--no-typescript",
            wasm_file.to_str().unwrap(),
        ])
        .status()
        .context("Failed to run wasm-bindgen. Is it installed? Run: cargo install wasm-bindgen-cli")?;

    if !status.success() {
        bail!("wasm-bindgen failed");
    }

    // Copy index.html
    println!("Copying assets...");
    let index_src = web_dir.join("index.html");
    let index_dst = pkg_dir.join("index.html");
    std::fs::copy(&index_src, &index_dst).context("Failed to copy index.html")?;

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
            format!("{}B", size)
        };
        println!("  {:>8}  {}", size_str, entry.file_name().to_string_lossy());
    }

    if serve {
        println!("\nStarting web server on http://localhost:{}", port);
        println!("Press Ctrl+C to stop");

        let status = Command::new("python3")
            .args(["-m", "http.server", &port.to_string()])
            .current_dir(&pkg_dir)
            .status()
            .context("Failed to start web server. Is python3 installed?")?;

        if !status.success() {
            bail!("Web server failed");
        }
    } else {
        println!("\nTo test locally:");
        println!("  cargo xtask web --serve");
    }

    Ok(())
}
