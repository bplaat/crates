#!/bin/sh
set -e

clean() {
    cargo clean
    find . \( -name "*.db*" -o -type d -name "target" -o -type d -name "node_modules" -o -type d -name "playwright" -o -type d -name "playwright-report" -o -type d -name "test-results" \) -exec rm -rf {} +
}

check_copyright() {
    echo "Checking copyright headers..."
    exit=0
    for file in $(find . -type f \( -name "*.rs" -o -name "*.html" -o -name "*.css" -o -name "*.js" -o -name "*.jsx" -o -name "*.ts" -o -name "*.tsx" -o -name "*.cc" -o -name "*.hh" \) ! -path "*/node_modules/*" ! -path "*/dist/*" ! -path "*/src-gen/*" ! -path "*/target/*" ! -path "*.min.js" ! -path "*bob/examples/*" ! -path "*ccontinue/tests/*.cc" ! -path "*playwright-report/*"); do
        if ! grep -E -q "Copyright \(c\) 20[0-9]{2}(-20[0-9]{2})? \w+" "$file"; then
            echo "Bad copyright header in: $file"
            exit=1
        fi
    done
    if [ "$exit" -ne 0 ]; then
        exit 1
    fi
}

check_formatting() {
    echo "Checking prettier formatting..."
    npx --prefer-offline --yes prettier --check $(find . -type f \( -name "*.md" -o -name "*.json" -o -name "*.yml" -o -name "*.yaml" -o -name "*.html" -o -name "*.css" -o -name "*.js" -o -name "*.jsx" -o -name "*.ts" -o -name "*.tsx" \) ! -path "*/node_modules/*" ! -path "*/dist/*" ! -path "*/src-gen/*" ! -path "*/target/*" ! -path "*/.vscode/*" ! -path "*.min.js" ! -path "*playwright/*" ! -path "*playwright-report/*" ! -path "*test-results/*")

    echo "Checking clang-format formatting..."
    clang-format --dry-run --Werror $(find bin/bob/examples -type f \( -name "*.c" -o -name "*.h" -o -name "*.cpp" -o -name "*.hpp" -o -name "*.m" -o -name "*.mm" -o -name "*.java" \) ! -path "*/target/*")
}

check_rust() {
    # Format
    echo "Checking Rust formatting..."
    cargo +nightly fmt -- --check

    # Lint
    echo "Linting Rust code..."
    cargo clippy --locked --all-targets --all-features -- -D warnings -W clippy::uninlined_format_args

    # Test
    echo "Running Rust tests..."
    cargo test --doc --all-features --locked
    cargo nextest run --all-features --locked --config-file nextest.toml ${CI:+--profile ci}

    if [ "$(uname)" != "MINGW64_NT" ] && [ -z "$USERPROFILE" ]; then
        echo "Running Rust tests with address sanitizer on unsafe libs..."
        target=$(rustc +nightly -vV | sed -n 's/^host: //p')
        # FIXME: Enable also for bwebview in future
        for crate_dir in $(find lib -mindepth 1 -maxdepth 1 -type d | grep -v 'lib/bwebview' | sort); do
            if ! find "$crate_dir" -type f -name "*.rs" -exec grep -Eq 'unsafe[[:space:]]*\{' {} +; then
                continue
            fi
            package=$(sed -n '/^\[package\]/,/^\[/{s/^name = "\(.*\)"/\1/p;}' "$crate_dir/Cargo.toml" | head -n 1)
            if [ -z "$package" ]; then
                echo "Failed to determine package name for $crate_dir"
                exit 1
            fi
            echo "Testing $package with address sanitizer..."
            RUSTFLAGS="-Zsanitizer=address" cargo +nightly test -p "$package" --lib --tests --locked --all-features --target "$target" -Zbuild-std
        done
    fi
}

check_rust_deps() {
    echo "Checking Rust dependencies..."
    cargo deny check --hide-inclusion-graph
}

check_e2e() {
    echo "Running end-to-end tests..."
    (cd bin/plaatnotes/web && check_e2e_plaatnotes)
}

check_e2e_plaatnotes() {
    if [ ! -d node_modules ]; then
        npm ci --prefer-offline
    fi
    npx playwright install --with-deps
    npm test
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
    cargo bundle --path bin/sequelexplorer
    cargo bundle --path bin/navidrome
    cargo bundle --path bin/game2048
}

build_install_windows() {
    package=$1
    name=$2
    cargo build --release --bin "$package"
    cp target/release/$package.exe "$USERPROFILE/Desktop/$name.exe"
}

build_install_freedesktop() {
    package=$1
    name=$2
    cargo build --release --bin "$package"
    cp "target/release/$package" "$HOME/.local/bin/$name"
    cp "bin/$package/meta/freedesktop/.desktop" "$HOME/.local/share/applications/$name.desktop"
    cp "bin/$package/docs/images/icon.svg" "$HOME/.local/share/icons/$name.svg"
}

install() {
    cargo install --force --path bin/bob
    cargo install --force --path bin/ccontinue
    cargo install --force --path bin/music-dl

    if [ "$(uname)" = "Darwin" ]; then
        build_bundle
        cp -r target/bundle/bassielight/BassieLight.app /Applications
        cp -r target/bundle/manexplorer/ManExplorer.app /Applications
        cp -r "target/bundle/sequelexplorer/Sequel Explorer.app" /Applications
        cp -r target/bundle/navidrome/Navidrome.app /Applications
        cp -r target/bundle/game2048/2048.app /Applications
    elif [ "$(uname)" = "MINGW64_NT" ] || [ -n "$USERPROFILE" ]; then
        build_install_windows bassielight BassieLight
        build_install_windows sequelexplorer SequelExplorer
        build_install_windows navidrome Navidrome
        build_install_windows game2048 2048
    else
        mkdir -p ~/.local/bin ~/.local/share/applications ~/.local/share/icons
        build_install_freedesktop bassielight BassieLight
        build_install_freedesktop sequelexplorer SequelExplorer
        build_install_freedesktop manexplorer ManExplorer
        build_install_freedesktop navidrome Navidrome
        build_install_freedesktop game2048 2048
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
        check_copyright
        check_formatting
        check_rust
        check_rust_deps
        check_e2e
        ;;
    check-shared)
        check_copyright
        check_formatting
        check_rust_deps
        ;;
    check-rust)
        check_rust
        ;;
    check-e2e)
        check_e2e
        ;;
    coverage)
        coverage
        ;;
    install)
        install
        ;;
    *)
        echo "Usage: $0 {build-pages|build-bundle|clean|check|check-shared|check-rust|check-e2e|coverage|install}"
        exit 1
        ;;
esac
