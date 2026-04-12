# Build container
FROM rust:1.94.1-slim-bookworm AS builder

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
        cargo build --release --target aarch64-unknown-linux-musl --bin plaatdeploy && \
        cp target/aarch64-unknown-linux-musl/release/plaatdeploy /plaatdeploy; \
    else \
        cargo build --release --target x86_64-unknown-linux-musl --bin plaatdeploy && \
        cp target/x86_64-unknown-linux-musl/release/plaatdeploy /plaatdeploy; \
    fi

# Run container
FROM debian:bookworm-slim

LABEL org.opencontainers.image.source="https://github.com/bplaat/crates"
LABEL org.opencontainers.image.description="A minimal self-hosted deployment service"
LABEL org.opencontainers.image.licenses=MIT

RUN apt-get update && \
    apt-get install -y git ca-certificates curl && \
    install -m 0755 -d /etc/apt/keyrings && \
    curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc && \
    chmod a+r /etc/apt/keyrings/docker.asc && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian $(. /etc/os-release && echo "$VERSION_CODENAME") stable" \
        > /etc/apt/sources.list.d/docker.list && \
    apt-get update && \
    apt-get install -y docker-ce-cli && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /plaatdeploy /app/plaatdeploy

WORKDIR /app

VOLUME ["/data"]

EXPOSE 8080

ENV DATABASE_PATH=/data/database.db
ENV DEPLOY_PATH=/data/projects

ENTRYPOINT ["/app/plaatdeploy"]
