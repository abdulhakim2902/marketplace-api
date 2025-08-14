FROM rust:1.87-slim-bookworm AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y \
    git \
    pkg-config \
    libssl-dev \
    cmake \
    build-essential \
    g++ \
    libdw-dev \
    --no-install-recommends \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

COPY /Cargo.toml /Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release --locked

RUN rm -rf src

COPY src /build/src

COPY .sqlx /build/.sqlx

COPY .git .git

RUN cargo build --release --locked

FROM debian:bookworm-slim AS final

RUN apt-get update && apt-get install -y \
    git \
    libssl3 \
    ca-certificates \
    --no-install-recommends \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY migrations /app/migrations
COPY --from=builder /build/target/release /app/bin

ENV RUST_LOG=info

CMD ["./bin/nft-aggregator-api"]