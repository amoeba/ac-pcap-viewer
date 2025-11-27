# Multi-stage build for Discord bot with web server and WASM UI

# Stage 1: Build the WASM UI
FROM rust:latest as wasm-builder
WORKDIR /app
COPY . .
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack
RUN wasm-pack build crates/web --target web --release

# Stage 2: Build the bot (includes web server)
FROM rust:latest as bot-builder
WORKDIR /app
COPY . .
RUN cargo build --release -p bot

# Stage 3: Runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy the compiled bot binary
COPY --from=bot-builder /app/target/release/bot /app/bot

# Copy the built WASM assets
COPY --from=wasm-builder /app/crates/web/pkg /app/dist

EXPOSE 3000
ENV RUST_LOG=info
CMD ["./bot"]
