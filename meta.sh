#!/bin/bash

PROFILE=${PROFILE:-default}
set -e

function clean() {
    cargo clean
    find . -name "*.db*" -delete
    find . -type d -name "target" -exec rm -rf {} +
    find . -type d -name "node_modules" -exec rm -rf {} +
    find . -type d -name "dist" -exec rm -rf {} +
}

function check_copyright() {
    exit=0
    for file in $(find bin examples lib -name "*.rs"); do
        if ! grep -E -q "Copyright \(c\) 20[0-9]{2}(-20[0-9]{2})? Bastiaan van der Plaat" "$file"; then
            echo "Bad copyright header in: $file"
            exit=1
        fi
    done
    if [ "$exit" -ne 0 ]; then
        exit 1
    fi
}

function check() {
    # Format
    check_copyright
    cargo +nightly fmt -- --check
    # Lint
    cargo clippy --locked --all-targets --all-features -- -D warnings
    cargo deny check --hide-inclusion-graph
    # Test
    cargo test --doc --all-features --locked
    cargo nextest run --all-features --locked --config-file nextest.toml --profile "$PROFILE"
}

function coverage() {
    cargo llvm-cov nextest --all-features --locked --config-file nextest.toml --profile "$PROFILE"
}

function release() {
    targets=("x86_64-unknown-linux-musl" "aarch64-unknown-linux-musl" "x86_64-apple-darwin" "aarch64-apple-darwin" "x86_64-pc-windows-gnu")
    for target in "${targets[@]}"; do
        cargo zigbuild --release --target "$target"
    done
    rm .intentionally-empty-file.o
}

case "${1:-check}" in
    clean)
        clean
        ;;
    check)
        check
        ;;
    coverage)
        coverage
        ;;
    release)
        release
        ;;
    *)
        echo "Usage: $0 {clean|check|coverage|release}"
        exit 1
        ;;
esac
