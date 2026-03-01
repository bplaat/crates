# Build container
FROM rust:1.93.0-slim-bookworm AS builder

WORKDIR /src

RUN apt-get update && \
    apt-get install -y curl musl-tools && \
    curl -fsSL https://deb.nodesource.com/setup_24.x -o nodesource_setup.sh && \
    bash nodesource_setup.sh && \
    apt-get install -y nodejs && \
    rustup target add x86_64-unknown-linux-musl

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl --bin plaatnotes

# Run container
FROM scratch

COPY --from=builder /src/target/x86_64-unknown-linux-musl/release/plaatnotes /app/plaatnotes

WORKDIR /app

ENTRYPOINT ["/app/plaatnotes"]
