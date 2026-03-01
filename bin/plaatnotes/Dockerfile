# Build container
FROM rust:1.93.0-slim-bookworm AS builder

ARG TARGETARCH

WORKDIR /src

RUN apt-get update && \
    apt-get install -y curl musl-tools && \
    curl -fsSL https://deb.nodesource.com/setup_24.x -o nodesource_setup.sh && \
    bash nodesource_setup.sh && \
    apt-get install -y nodejs && \
    if [ "$TARGETARCH" = "arm64" ]; then \
        rustup target add aarch64-unknown-linux-musl; \
    else \
        rustup target add x86_64-unknown-linux-musl; \
    fi

COPY . .

RUN if [ "$TARGETARCH" = "arm64" ]; then \
        cargo build --release --target aarch64-unknown-linux-musl --bin plaatnotes && \
        cp target/aarch64-unknown-linux-musl/release/plaatnotes /plaatnotes; \
    else \
        cargo build --release --target x86_64-unknown-linux-musl --bin plaatnotes && \
        cp target/x86_64-unknown-linux-musl/release/plaatnotes /plaatnotes; \
    fi

# Run container
FROM scratch

COPY --from=builder /plaatnotes /app/plaatnotes

WORKDIR /app

ENTRYPOINT ["/app/plaatnotes"]
