#!/bin/sh
set -e

clean() {
    cargo clean
    find . -name "*.db*" -delete
    find . -type d -name "target" -exec rm -rf {} +
    find . -type d -name "node_modules" -exec rm -rf {} +
    find . -type d -name "dist" -exec rm -rf {} +
}

check_copyright() {
    echo "Checking copyright headers..."
    exit=0
    for file in $(find bin examples lib -name "*.rs" -o -name "*.html" -o -name "*.css" -o -name "*.js" -o -name "*.jsx" -o -name "*.ts" -o -name "*.tsx" | grep -v "node_modules" | grep -v "dist" | grep -v "api.ts"); do
        if ! grep -E -q "Copyright \(c\) 20[0-9]{2}(-20[0-9]{2})? \w+" "$file"; then
            echo "Bad copyright header in: $file"
            exit=1
        fi
    done
    if [ "$exit" -ne 0 ]; then
        exit 1
    fi
}

check_web() {
    # Format
    echo "Checking web formatting..."
    # This is the default Prettier version, in the VSCode extension :|
    npx --prefer-offline --yes prettier@2.8.8 --check --write $(find bin examples lib -name "*.html" -o -name "*.css" -o -name "*.js" -o -name "*.jsx" | grep -v "node_modules" | grep -v "dist")
}

check_bob_examples() {
    # Format
    echo "Checking Bob examples formatting..."
    clang-format --dry-run --Werror $(find bin/bob/examples -name "*.c" -o -name "*.h" -o -name "*.cpp" -o -name "*.hpp" -o -name "*.m" -o -name "*.mm" -o -name "*.java" | grep -v "src-gen")
}

check_rust() {
    # Format
    echo "Checking Rust formatting..."
    cargo +nightly fmt -- --check
    # Lint
    echo "Linting Rust code..."
    cargo clippy --locked --all-targets --all-features -- -D warnings -W clippy::uninlined_format_args
    # Dependencies
    echo "Checking Rust dependencies..."
    cargo deny check --hide-inclusion-graph
    # Test
    echo "Running Rust tests..."
    cargo test --doc --all-features --locked
    cargo nextest run --all-features --locked --no-fail-fast --retries 2
}

check() {
    check_copyright
    check_web
    check_bob_examples
    check_rust
}

coverage() {
    cargo llvm-cov nextest --all-features --locked --no-fail-fast --retries 2
}

build_pages() {
    mkdir -p target/pages
    cp index.html target/pages/
    build_pages_baksteen
}

build_pages_baksteen() {
    mkdir -p target/pages/baksteen
    cp -r bin/baksteen/public/* target/pages/baksteen
    cargo build --release -p baksteen --target wasm32-unknown-unknown
    wasm-bindgen --target web --no-typescript --out-dir target/pages/baksteen --out-name baksteen target/wasm32-unknown-unknown/release/baksteen.wasm
}

build_bundle() {
    cargo install --path bin/cargo-bundle
    cargo bundle --path bin/bassielight
    cargo bundle --path bin/manexplorer
    cargo bundle --path bin/navidrome
}

install() {
    cargo install --force --path bin/bob
    cargo install --force --path bin/music-dl

    if [ "$(uname)" = "Darwin" ]; then
        build_bundle
        cp -r target/bundle/bassielight/BassieLight.app /Applications
        cp -r target/bundle/manexplorer/ManExplorer.app /Applications
        cp -r target/bundle/navidrome/Navidrome.app /Applications
    fi
}

case "${1:-check}" in
    build-pages)
        build_pages
        ;;
    build-bundle)
        build_bundle
        ;;
    clean)
        clean
        ;;
    check)
        check
        ;;
    coverage)
        coverage
        ;;
    install)
        install
        ;;
    *)
        echo "Usage: $0 {build-pages|build-bundle|clean|check|coverage|install}"
        exit 1
        ;;
esac
