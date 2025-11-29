# Bot-only Docker image (no WASM/web UI)

# Base stage with cargo-chef and sccache for optimal caching
FROM rust:1.91 AS base

# Install sccache (compiler cache) and cargo-chef (dependency cache)
RUN cargo install sccache --version ^0.7 && \
    cargo install cargo-chef --version ^0.1

# Set up sccache
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache
ENV CARGO_HOME=/usr/local/cargo

# Install system dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Stage 1a: Planner - analyzes dependencies and creates recipe
FROM base AS planner
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef prepare --recipe-path recipe.json

# Stage 1b: Cacher - builds dependencies only (cached layer)
FROM base AS cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies with both cargo-chef and sccache
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# Stage 1c: Builder - builds the actual application with caching
FROM base AS builder
WORKDIR /app

# Copy compiled dependencies from cacher
COPY --from=cacher /app/target target
COPY --from=cacher /app/Cargo.lock Cargo.lock

# Copy full source code
COPY . .

# Build bot binary only
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    cargo build --release -p bot

# Stage 2: Runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy the compiled bot binary
COPY --from=builder /app/target/release/bot /app/bot

EXPOSE 3000
ENV RUST_LOG=info

# Environment variables for bot configuration
# WEB_URL: Public URL where the web UI is hosted (for bot responses)
# DATABASE_URL: SQLite database path (defaults to ./data/bot.db)
# DISCORD_OAUTH_TOKEN: Discord bot token (required)
# PORT: Port for web server (defaults to 3000)

CMD ["./bot"]
