# Build container
FROM rust:1.93.0-slim-bookworm AS builder

WORKDIR /src

RUN apt-get update && \
    apt-get install -y curl && \
    curl -fsSL https://deb.nodesource.com/setup_24.x -o nodesource_setup.sh && \
    bash nodesource_setup.sh && \
    apt-get install -y nodejs

COPY . .

RUN cargo build --release --bin plaatnotes

# Run container
FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /src/target/release/plaatnotes ./plaatnotes

ENTRYPOINT ["./plaatnotes"]
