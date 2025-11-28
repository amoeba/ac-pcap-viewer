# PCAP Parser web UI - static file server (no Discord bot)

# Base stage with cargo-chef and sccache for optimal caching
FROM rust:1.91 AS base

# Install sccache (compiler cache) and cargo-chef (dependency cache)
# cargo-chef: Analyzes dependencies and caches them separately
# sccache: Caches compiler output for faster incremental builds
RUN cargo install sccache --version ^0.7 && \
    cargo install cargo-chef --version ^0.1 && \
    cargo install wasm-pack

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

# Install Rust WASM target
RUN rustup target add wasm32-unknown-unknown

# Copy full source code
COPY . .

# Build WASM UI using xtask
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    cargo xtask pcap

# Stage 2: Runtime image - Nginx for static file serving
FROM nginx:alpine

# Copy the built dist directory (contains WASM, JS, HTML with cache busting)
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80

# Nginx runs on port 80 by default
