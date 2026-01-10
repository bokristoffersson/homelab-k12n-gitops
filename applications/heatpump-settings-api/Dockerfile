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

# Install CA certificates for HTTPS connections
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary and config from builder
COPY --from=builder /app/target/release/heatpump-settings-api /app/heatpump-settings-api
COPY --from=builder /app/config /app/config

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

CMD ["/app/heatpump-settings-api"]
