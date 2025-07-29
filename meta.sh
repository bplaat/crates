#!/bin/bash
set -e

function clean() {
    cargo clean
    find . -name "*.db*" -delete
    find . -type d -name "target" -exec rm -rf {} +
    find . -type d -name "node_modules" -exec rm -rf {} +
    find . -type d -name "dist" -exec rm -rf {} +
}

function check_copyright() {
    echo "Checking copyright headers..."
    exit=0
    for file in $(find bin examples lib -name "*.rs" -o -name "*.html" -o -name "*.css" -o -name "*.js" -o -name "*.jsx" | grep -v "node_modules" | grep -v "dist"); do
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
    # This is the default Prettier version, in the VSCode extension :|
    npx --prefer-offline --yes prettier@2.8.8 --check --write $(find bin examples lib -name "*.html" -o -name "*.css" -o -name "*.js" -o -name "*.jsx" | grep -v "node_modules" | grep -v "dist")
}

function check_rust() {
    # Format
    echo "Checking Rust formatting..."
    cargo +nightly fmt -- --check
    # Lint
    echo "Linting Rust code..."
    cargo clippy --locked --all-targets --all-features -- -D warnings
    # Dependencies
    echo "Checking Rust dependencies..."
    cargo deny check --hide-inclusion-graph
    # Test
    echo "Running Rust tests..."
    cargo test --doc --all-features --locked
    cargo nextest run --all-features --locked --no-fail-fast --retries 2
}

function check() {
    check_copyright
    check_web
    check_rust
}

function coverage() {
    cargo llvm-cov nextest --all-features --locked --no-fail-fast --retries 2
}

function release() {
    # FIXME: Find way to do this better
    release_bassielight_macos
    release_navidrome_macos
    open target
}

function release_bassielight_macos() {
    bundle_dir="target/BassieLight.app/Contents"
    mkdir -p $bundle_dir/MacOS $bundle_dir/Resources
    for target in x86_64-apple-darwin aarch64-apple-darwin; do
        cargo build --release --bin "bassielight" --target $target
    done
    lipo -create target/x86_64-apple-darwin/release/bassielight target/aarch64-apple-darwin/release/bassielight \
        -output $bundle_dir/MacOS/BassieLight
    cp target/BassieLight/icon.icns $bundle_dir/Resources
    cp target/BassieLight/Info.plist $bundle_dir
    cd target && rm -f BassieLight.zip && zip -r BassieLight.zip BassieLight.app && cd ..
}

function release_navidrome_macos() {
    bundle_dir="target/Navidrome.app/Contents"
    mkdir -p $bundle_dir/MacOS $bundle_dir/Resources
    for target in x86_64-apple-darwin aarch64-apple-darwin; do
        cargo build --release --bin "navidrome" --target $target
    done
    lipo -create target/x86_64-apple-darwin/release/navidrome target/aarch64-apple-darwin/release/navidrome \
        -output $bundle_dir/MacOS/Navidrome
    cp target/Navidrome/icon.icns $bundle_dir/Resources
    cp target/Navidrome/Info.plist $bundle_dir
    cd target && rm -f Navidrome.zip && zip -r Navidrome.zip Navidrome.app && cd ..
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
