# Build container
FROM rust:1.93.0-slim-bookworm AS builder

WORKDIR /src

COPY --exclude=target --exclude=dist --exclude=node_modules . .

RUN cargo build --release --bin webhook-puller

# Run container
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y git curl && \
    rm -rf /var/lib/apt/lists/* && \
    git config --global safe.directory '*'

COPY --from=builder /src/target/release/webhook-puller ./webhook-puller

ENTRYPOINT ["./webhook-puller"]
