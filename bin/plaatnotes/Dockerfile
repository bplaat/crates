# Build container
FROM rust:1.98-slim-bookworm AS builder

ARG TARGETARCH

WORKDIR /src

# hadolint ignore=DL3008
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl make perl && \
    curl -fsSL https://deb.nodesource.com/setup_26.x -o nodesource_setup.sh && \
    bash nodesource_setup.sh && \
    apt-get install -y --no-install-recommends nodejs && \
    rm -rf /var/lib/apt/lists/*

COPY rust-toolchain.toml .

RUN rustup toolchain install nightly --profile minimal --component rust-src

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    case "$TARGETARCH" in \
        arm64) rust_target=aarch64-unknown-linux-blibc; crt=crt1-aarch64.s; gcc_flags= ;; \
        amd64) rust_target=x86_64-unknown-linux-blibc; crt=crt1-x86_64.s; gcc_flags=-m64 ;; \
        *) echo "unsupported architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac && \
    mkdir /runtime && \
    gcc ${gcc_flags} -c "lib/blibc/${crt}" -o /runtime/crt1.o && \
    cp lib/blibc/linker /runtime/blibc-linker && \
    chmod +x /runtime/blibc-linker && \
    CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_PANIC=unwind cargo +nightly build \
        --manifest-path lib/blibc/Cargo.toml \
        --release \
        --target-dir /blibc-target \
        --target "lib/blibc/targets/${rust_target}.json" \
        -Z json-target-spec \
        -Z build-std=core,compiler_builtins && \
    bash lib/blibc/archive "/blibc-target/${rust_target}/release" /runtime/blibc.a && \
    cargo +nightly build \
        --release \
        --locked \
        --target "lib/blibc/targets/${rust_target}.json" \
        -Z json-target-spec \
        -Z build-std=std,panic_abort \
        --config 'build.rustflags=["-C", "panic=abort"]' \
        --bin plaatnotes && \
    cp "target/${rust_target}/release/plaatnotes" /plaatnotes

RUN ! readelf -l /plaatnotes | grep -q INTERP && \
    ! readelf -d /plaatnotes | grep -q NEEDED && \
    test -z "$(nm -u /plaatnotes)"

FROM builder AS tests

ARG TARGETARCH

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    case "$TARGETARCH" in \
        arm64) rust_target=aarch64-unknown-linux-blibc ;; \
        amd64) rust_target=x86_64-unknown-linux-blibc ;; \
        *) echo "unsupported architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac && \
    rm -rf "target/${rust_target}/debug" && \
    cargo +nightly test \
        --locked \
        --target "lib/blibc/targets/${rust_target}.json" \
        -Z json-target-spec \
        -Z build-std=std,panic_unwind \
        --features libsqlite3-sys/bundled \
        --lib \
        --tests \
        --package base64 \
        --package blibc \
        --package bsql \
        --package bsql_derive \
        --package chrono \
        --package copy_dir \
        --package digest \
        --package directories \
        --package from_derive \
        --package getrandom@0.4.3+patch \
        --package hmac \
        --package image \
        --package js \
        --package libsqlite3-sys \
        --package local-ip-address \
        --package maxminddb \
        --package mime \
        --package mime_guess \
        --package native-tls \
        --package openapi-generator \
        --package pbkdf2 \
        --package plaatnotes \
        --package plist \
        --package regex \
        --package reversi-engine \
        --package rust-embed \
        --package rust-embed-impl \
        --package serde_yaml \
        --package sha1 \
        --package sha2 \
        --package simple-useragent \
        --package simple_logger \
        --package small-http \
        --package small-router \
        --package small-websocket \
        --package terminal_size \
        --package threadpool \
        --package trybuild \
        --package url \
        --package uuid \
        --package validate \
        --package validate_derive \
        --package zip \
        -- \
        --test-threads=1

# Run container
FROM scratch

LABEL org.opencontainers.image.source="https://github.com/bplaat/crates"
LABEL org.opencontainers.image.description="A simple note-taking app"
LABEL org.opencontainers.image.licenses=MIT

COPY --from=builder /plaatnotes /app/plaatnotes

WORKDIR /app

EXPOSE 8080

VOLUME ["/data"]

ENV DATA_PATH=/data

ENTRYPOINT ["/app/plaatnotes"]
