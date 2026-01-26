# Stage 1: Build the Rust WASM application
FROM rust:1.83 AS builder

# Install wasm target and trunk
RUN rustup target add wasm32-unknown-unknown && \
    cargo install trunk --locked

WORKDIR /app

# Copy manifests first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir src && \
    echo "pub fn lib_dummy() {}" > src/lib.rs && \
    cargo build --release --target wasm32-unknown-unknown && \
    rm -rf src

# Copy actual source code and assets
COPY src ./src
COPY index.html ./
COPY style ./style
COPY public ./public
COPY Trunk.toml ./

# Build the WASM application with Trunk
# Touch lib.rs to force rebuild of actual code (not cached dependencies)
RUN touch src/lib.rs && \
    trunk build --release

# Stage 2: Serve with nginx
FROM nginx:alpine

# Install envsubst for runtime configuration
RUN apk add --no-cache gettext

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Copy built assets from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy env-config template for runtime configuration
COPY env-config.js.template /usr/share/nginx/html/env-config.js.template

# Create startup script to inject environment variables
RUN echo '#!/bin/sh' > /docker-entrypoint.sh && \
    echo 'envsubst < /usr/share/nginx/html/env-config.js.template > /usr/share/nginx/html/env-config.js' >> /docker-entrypoint.sh && \
    echo 'exec nginx -g "daemon off;"' >> /docker-entrypoint.sh && \
    chmod +x /docker-entrypoint.sh

EXPOSE 80

CMD ["/docker-entrypoint.sh"]
