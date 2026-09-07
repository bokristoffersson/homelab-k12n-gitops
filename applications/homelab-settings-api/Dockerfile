# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Cache dependencies by building with dummy main.rs first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build actual application
COPY src ./src
COPY config ./config

# Touch main.rs to force rebuild and build release binary
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage - use newer Debian with GLIBC 2.38+
FROM debian:trixie-slim

# CA certificates for HTTPS; tzdata so the TZ env var resolves (the plug
# scheduler runs on local time — without zoneinfo chrono falls back to UTC)
RUN apt-get update && \
    apt-get install -y ca-certificates tzdata && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary and config from builder
COPY --from=builder /app/target/release/homelab-settings-api /app/homelab-settings-api
COPY --from=builder /app/config /app/config

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

CMD ["/app/homelab-settings-api"]
