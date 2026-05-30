# syntax=docker/dockerfile:1

# --- Build stage: compile release binary ---
FROM rust:1-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependency layer (faster rebuilds when only src changes)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
RUN cargo build --release

# --- Runtime stage: small image with binary only ---
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/cricscorebackend /app/cricscorebackend
RUN chmod +x /app/cricscorebackend

ENV HOST=0.0.0.0
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

EXPOSE 8080

USER nobody:nogroup

CMD ["/app/cricscorebackend"]
