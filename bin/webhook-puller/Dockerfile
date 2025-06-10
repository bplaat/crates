# Build container
FROM rust:1.87-slim AS builder

WORKDIR /src

COPY . .

RUN cargo build --release --bin webhook-puller

# Run container
FROM alpine

WORKDIR /app

COPY --from=builder /src/target/release/webhook-puller ./webhook-puller

ENTRYPOINT ["./webhook-puller"]
