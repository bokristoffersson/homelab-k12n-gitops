FROM rust:slim-bookworm AS builder

WORKDIR /usr/src/app

# Pure-Rust TLS (rustls/ring) throughout — no system OpenSSL needed.

# Copy manifests first to cache the dependency build layer
COPY Cargo.toml Cargo.lock ./

# Build dependencies with dummy source files. Create both lib.rs and main.rs
# so the cache layer works for both lib+bin and bin-only crates.
RUN mkdir src && \
    echo "pub fn lib_dummy() {}" > src/lib.rs && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the real source and build the application (dependencies already cached)
COPY src ./src
COPY config ./config
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/spotprice-api /app/spotprice-api
COPY --from=builder /usr/src/app/config /app/config

ENV APP_CONFIG=/app/config/config.yaml
ENV RUST_LOG=info
# Scheduling uses local time; align the container clock with the Nord Pool market.
ENV TZ=Europe/Stockholm

EXPOSE 8080

CMD ["/app/spotprice-api"]
