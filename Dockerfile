# syntax=docker/dockerfile:1.4

# Stage 1: Build WASM
FROM rust:1.91-slim-bookworm AS builder

# Install dependencies (including git for build.rs)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install wasm target and wasm-bindgen
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli

WORKDIR /app

# Copy git info for build.rs to read SHA
COPY .git .git

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build WASM with cargo cache mounts for faster rebuilds
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build -p web --release --target wasm32-unknown-unknown && \
    cp /app/target/wasm32-unknown-unknown/release/web.wasm /app/web.wasm

# Generate JS bindings
RUN wasm-bindgen \
    --target web \
    --out-dir /app/pkg \
    --no-typescript \
    /app/web.wasm

# Copy static assets
RUN cp crates/web/index.html /app/pkg/

# Stage 2: Serve with nginx
FROM nginx:alpine

# Copy built files
COPY --from=builder /app/pkg /usr/share/nginx/html

# Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
