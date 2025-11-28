# Multi-stage build for Discord bot with web server and WASM UI

# Stage 1: Build everything using cargo xtask
FROM rust:latest AS builder
WORKDIR /app
COPY . .

# Install system dependencies including ca-certificates for HTTPS
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Install wasm-pack for WASM builds
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack

# Build WASM and bot using xtask (this creates dist/ with cache-busted files)
RUN cargo xtask bot

# Stage 2: Runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy the compiled bot binary
COPY --from=builder /app/target/release/bot /app/bot

# Copy the built dist directory (contains WASM, JS, HTML with cache busting)
COPY --from=builder /app/dist /app/dist

EXPOSE 3000
ENV RUST_LOG=info
# Optional: Set WEB_URL for bot replies (defaults to http://localhost:3000)
# ENV WEB_URL=https://your-domain.com
CMD ["./bot"]
