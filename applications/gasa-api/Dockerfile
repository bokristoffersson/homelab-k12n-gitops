# Build stage
FROM rust:1.96 AS builder

WORKDIR /app

# Cache dependencies by building with dummy source files first
# Both lib.rs and main.rs dummies per CLAUDE.md (handles lib+bin and bin-only crates)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "pub fn lib_dummy() {}" > src/lib.rs && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
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
COPY --from=builder /app/target/release/gasa-api /app/gasa-api
COPY --from=builder /app/config /app/config

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

CMD ["/app/gasa-api"]
