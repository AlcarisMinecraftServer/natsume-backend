FROM rust:1.86.0-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config libssl-dev clang \
    && rm -rf /var/lib/apt/lists/*
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY . .
RUN cargo build --release --package api --locked

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/api /usr/local/bin/api
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/config ./config

ENV TZ=Asia/Tokyo
EXPOSE 9000
CMD ["api"]
