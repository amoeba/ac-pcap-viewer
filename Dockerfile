# Simple static file server for pre-built WASM
FROM nginx:alpine

# Copy pre-built static files
COPY pkg/ /usr/share/nginx/html/

# Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
