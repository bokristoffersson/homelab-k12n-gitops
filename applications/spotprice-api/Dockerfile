FROM rust:slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies (openssl for reqwest/a2 TLS + JWT signing)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first to cache the dependency build layer
COPY Cargo.toml Cargo.lock ./

# Build dependencies with a dummy source file
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
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
    libssl3 \
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
