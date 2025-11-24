# Multi-stage: build + slim runtime
FROM --platform=$BUILDPLATFORM rust:1.90 AS builder

ARG TARGETPLATFORM
WORKDIR /app

# Install cross-compilation tools
RUN apt-get update && apt-get install -y gcc-aarch64-linux-gnu && rm -rf /var/lib/apt/lists/*

# Add Rust targets
RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup target add x86_64-unknown-linux-gnu

COPY Cargo.toml Cargo.lock ./

# Cache deps
RUN mkdir -p src && echo "fn main(){}" > src/main.rs

# Determine target based on platform
RUN case "$TARGETPLATFORM" in \
    "linux/arm64") \
      export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc && \
      cargo build --release --target aarch64-unknown-linux-gnu || true ;; \
    "linux/amd64") \
      cargo build --release --target x86_64-unknown-linux-gnu || true ;; \
    *) cargo build --release || true ;; \
    esac

# Real sources
COPY . .

# Build for the correct target
RUN case "$TARGETPLATFORM" in \
    "linux/arm64") \
      export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc && \
      cargo build --release --target aarch64-unknown-linux-gnu ;; \
    "linux/amd64") \
      cargo build --release --target x86_64-unknown-linux-gnu ;; \
    *) cargo build --release ;; \
    esac

# Copy the correct binary based on platform
RUN case "$TARGETPLATFORM" in \
    "linux/arm64") \
      cp target/aarch64-unknown-linux-gnu/release/heatpump-api /app/heatpump-api ;; \
    "linux/amd64") \
      cp target/x86_64-unknown-linux-gnu/release/heatpump-api /app/heatpump-api ;; \
    *) \
      cp target/release/heatpump-api /app/heatpump-api ;; \
    esac

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/heatpump-api /usr/local/bin/heatpump-api

ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/heatpump-api"]
