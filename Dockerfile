
FROM rust:1.91 AS base

RUN cargo install sccache --version ^0.7 && \
    cargo install cargo-chef --version ^0.1 && \
    cargo install wasm-pack

ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache
ENV CARGO_HOME=/usr/local/cargo

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

FROM base AS chef

WORKDIR /app
COPY . .
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef prepare --recipe-path recipe.json

FROM base AS cacher

WORKDIR /app
COPY --from=chef /app/recipe.json recipe.json

RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

FROM base AS builder
WORKDIR /app

COPY --from=cacher /app/target target
COPY --from=cacher /app/Cargo.lock Cargo.lock
COPY . .

RUN rustup target add wasm32-unknown-unknown
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    cargo xtask web

FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY ./conf/nginx.conf /etc/nginx/nginx.conf

EXPOSE 80
