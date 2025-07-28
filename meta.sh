#!/bin/bash
set -e

function clean() {
    cargo clean
    rm -rf dist
}

function check_copyright() {
    echo "Checking copyright headers..."
    exit=0
    for file in $(find . -name "*.rs" -o -name "*.html" | grep -v "dist"); do
        if ! grep -E -q "Copyright \(c\) 20[0-9]{2}(-20[0-9]{2})? \w+" "$file"; then
            echo "Bad copyright header in: $file"
            exit=1
        fi
    done
    if [ "$exit" -ne 0 ]; then
        exit 1
    fi
}

function check_web() {
    # Format
    echo "Checking web formatting..."
    npx --prefer-offline --yes prettier@2.8.8 --check --write $(find . -name "*.html" | grep -v "dist")
}

function check_rust() {
    # Format
    echo "Checking Rust formatting..."
    cargo +nightly fmt -- --check
    # Lint
    echo "Linting Rust code..."
    cargo clippy --locked --all-targets --all-features -- -D warnings
}

function check() {
    check_copyright
    check_web
    check_rust
}

function build() {
    mkdir -p dist
    cp index.html dist/
    cargo build --target wasm32-unknown-unknown --release
    wasm-bindgen --target web --no-typescript --out-dir dist/ --out-name baksteen target/wasm32-unknown-unknown/release/baksteen.wasm
}

case "${1:-check}" in
    clean)
        clean
        ;;
    check)
        check
        ;;
    build)
        build
        ;;
    *)
        echo "Usage: $0 {clean|check|build}"
        exit 1
        ;;
esac
