# Build container
FROM rust:1.90.0-slim-bookworm AS builder

WORKDIR /src

COPY . .

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
