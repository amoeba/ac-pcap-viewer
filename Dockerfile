# Stage 1: Build WASM
FROM rust:1.83-slim-bookworm AS builder

# Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install wasm target and wasm-bindgen
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build WASM (release profile with LTO for smaller output)
RUN cargo build -p web --release --target wasm32-unknown-unknown

# Generate JS bindings
RUN wasm-bindgen \
    --target web \
    --out-dir /app/pkg \
    --no-typescript \
    /app/target/wasm32-unknown-unknown/release/web.wasm

# Copy static assets
RUN cp crates/web/index.html /app/pkg/

# Stage 2: Serve with nginx
FROM nginx:alpine

# Copy built files
COPY --from=builder /app/pkg /usr/share/nginx/html

# Configure nginx for SPA and proper MIME types
RUN cat > /etc/nginx/conf.d/default.conf << 'EOF'
server {
    listen 80;
    listen 5000;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    # WASM MIME type
    types {
        application/wasm wasm;
    }

    # Gzip compression
    gzip on;
    gzip_types text/plain text/css application/javascript application/wasm;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # Cache static assets
    location ~* \.(js|wasm)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
EOF

EXPOSE 80 5000
