# Build container
FROM rust:1.87-slim-bookworm AS builder

WORKDIR /src

COPY . .

RUN cargo build --release --bin webhook-puller

# Run container
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y git && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/webhook-puller ./webhook-puller

ENTRYPOINT ["./webhook-puller"]
