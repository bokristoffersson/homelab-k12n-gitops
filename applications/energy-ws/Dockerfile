# Build stage
FROM rust:1.83 AS builder

# Install build dependencies for rdkafka
RUN apt-get update && \
    apt-get install -y cmake g++ libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to build dependencies
# For lib+bin crates, need to declare all modules that main.rs imports
RUN mkdir src && \
    echo "pub mod auth {}" > src/lib.rs && \
    echo "pub mod config {}" >> src/lib.rs && \
    echo "pub mod error {}" >> src/lib.rs && \
    echo "pub mod kafka {}" >> src/lib.rs && \
    echo "pub mod ws {}" >> src/lib.rs && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code and config
COPY src ./src
COPY config ./config

# Build the actual application
# Touch src/main.rs to force rebuild (dependencies are already cached)
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install CA certificates and other runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/energy-ws /usr/local/bin/energy-ws

# Copy config example (can be overridden by ConfigMap)
COPY --from=builder /app/config/config.example.yaml /app/config/config.yaml

# Create non-root user
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 8080

# Set environment
ENV RUST_LOG=energy_ws=info,tower_http=info
ENV CONFIG_PATH=/app/config/config.yaml

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD [ "sh", "-c", "wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1" ]

# Run the binary
CMD ["energy-ws"]
