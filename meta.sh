#!/bin/bash
set -eo pipefail

detect_os() {
    case "$OSTYPE" in
        linux-gnu*|linux-musl*)
            echo "linux"
            ;;
        darwin*)
            echo "macos"
            ;;
        msys*|cygwin*|win32*|mingw*)
            echo "windows"
            ;;
        *)
            case "$(uname -s)" in
                Linux*)               echo "linux" ;;
                Darwin*)              echo "macos" ;;
                CYGWIN*|MINGW*|MSYS*) echo "windows" ;;
                *)
                    echo "Unsupported OS: $OSTYPE / $(uname -s)" >&2
                    exit 1
                    ;;
            esac
            ;;
    esac
}

OS=$(detect_os)

# Packages with features that completely swap the backend or linkage.
# ASAN tests run without these features; global tests add an extra pass without them.
BACKEND_SWAP_PAIRS=(
    "native-tls:vendored"
    "bsqlite:bundled"
)

backend_swap_feature() {
    local package=$1
    for pair in "${BACKEND_SWAP_PAIRS[@]}"; do
        if [ "${pair%%:*}" = "$package" ]; then
            echo "${pair##*:}"
            return
        fi
    done
}

# Returns "--features a,b,c" with all features except the backend-swap one,
# or "--all-features" if the package has no backend-swap feature,
# or "" if no features remain after excluding the swap feature.
features_without_swap() {
    local package=$1
    local swap_feat
    swap_feat=$(backend_swap_feature "$package")
    if [ -z "$swap_feat" ]; then
        echo "--all-features"
        return
    fi
    local remaining
    remaining=$(cargo metadata --no-deps --format-version 1 2>/dev/null \
        | jq -r --arg pkg "$package" --arg swap "$swap_feat" \
            '.packages[] | select(.name == $pkg) | .features | keys[] | select(. != "default" and . != $swap)' \
        | sort | tr '\n' ',' | sed 's/,$//')
    [ -n "$remaining" ] && echo "--features $remaining" || echo ""
}

is_platform_excluded() {
    local pkg=$1
    local i
    for ((i = 0; i < ${#excludes[@]} - 1; i++)); do
        if [ "${excludes[$i]}" = "--exclude" ] && [ "${excludes[$((i + 1))]}" = "$pkg" ]; then
            return 0
        fi
    done
    return 1
}

clean() {
    cargo clean
    rm -rf projects
    find . \( -name "*.db*" -o -type d -name "target" -o -type d -name "node_modules" -o -type d -name "playwright" -o -type d -name "playwright-report" -o -type d -name "test-results" \) -exec rm -rf {} +
}

check_copyright() {
    echo "Checking copyright headers..."
    exit=0
    # shellcheck disable=SC2044
    for file in $(find . -type f \( -name "*.rs" -o -name "*.html" -o -name "*.css" -o -name "*.js" -o -name "*.jsx" -o -name "*.ts" -o -name "*.tsx" -o -name "*.cc" -o -name "*.hh" \) ! -path "*/node_modules/*" ! -path "*/dist/*" ! -path "./projects/*" ! -path "*/src-gen/*" ! -path "*/target/*" ! -path "*.min.js" ! -path "*bob/examples/*" ! -path "*ccontinue/tests/*.cc" ! -path "*playwright-report/*"); do
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
    find . -type f \( -name "*.md" -o -name "*.json" -o -name "*.yml" -o -name "*.yaml" -o -name "*.html" -o -name "*.css" -o -name "*.js" -o -name "*.jsx" -o -name "*.ts" -o -name "*.tsx" \) ! -path "*/node_modules/*" ! -path "*/dist/*" ! -path "./projects/*" ! -path "*/src-gen/*" ! -path "*/target/*" ! -path "*/.vscode/*" ! -path "*.min.js" ! -path "*playwright/*" ! -path "*playwright-report/*" ! -path "*test-results/*" -print0 \
        | xargs -0 npx --prefer-offline --yes prettier@3.8.4 --check

    echo "Checking clang-format formatting..."
    find bin/bob/examples -type f \( -name "*.c" -o -name "*.h" -o -name "*.cpp" -o -name "*.hpp" -o -name "*.m" -o -name "*.mm" -o -name "*.java" \) ! -path "*/target/*" -print0 \
        | xargs -0 clang-format --dry-run --Werror
}

platform_excludes() {
    cargo metadata --no-deps --format-version 1 2>/dev/null \
        | jq -r --arg os "$OS" '
            .packages[] |
            select(
                .metadata.platforms != null and
                (.metadata.platforms | index($os) | not)
            ) |
            "--exclude", .name
        ' | tr -d '\r'
}

check_rust() {
    excludes=()
    while IFS= read -r arg; do
        excludes+=("$arg")
    done < <(platform_excludes)

    # Format
    echo "Checking Rust formatting..."
    cargo +nightly fmt -- --check

    # Lint
    echo "Linting Rust code..."
    cargo clippy --workspace "${excludes[@]}" --locked --all-targets --all-features -- -D warnings -W clippy::uninlined_format_args

    # Test
    echo "Running Rust tests..."
    cargo test --doc --all-features --locked --workspace "${excludes[@]}"
    cargo nextest run --all-features --locked --config-file nextest.toml ${CI:+--profile ci} --workspace "${excludes[@]}"

    # Also test backend-swapping packages without their swap feature to exercise the native backends
    for pair in "${BACKEND_SWAP_PAIRS[@]}"; do
        pkg="${pair%%:*}"
        feat="${pair##*:}"
        is_platform_excluded "$pkg" && continue
        echo "Running Rust tests for $pkg without $feat feature..."
        IFS=' ' read -ra feat_args <<< "$(features_without_swap "$pkg")"
        cargo test --doc --locked -p "$pkg" "${feat_args[@]}"
        cargo nextest run --locked --config-file nextest.toml ${CI:+--profile ci} -p "$pkg" "${feat_args[@]}"
    done

    if [ "$OS" != "windows" ]; then
        echo "Running Rust tests with address sanitizer on unsafe libs..."
        target=$(rustc +nightly -vV | sed -n 's/^host: //p')
        # FIXME: Enable also for bwebview in future
        # shellcheck disable=SC2044
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
            IFS=' ' read -ra feat_args <<< "$(features_without_swap "$package")"
            RUSTFLAGS="-Zsanitizer=address" cargo +nightly test -p "$package" --lib --tests --locked "${feat_args[@]}" --target "$target" -Zbuild-std
        done
    fi
}

check_rust_deps() {
    echo "Checking Rust dependencies..."
    cargo deny check --hide-inclusion-graph
}

check_shell() {
    echo "Checking shell scripts..."
    shellcheck meta.sh
}

check_docker() {
    echo "Checking Dockerfiles..."
    find . -type f \( -name "Dockerfile" -o -name "*.Dockerfile" \) ! -path "*/node_modules/*" ! -path "*/dist/*" ! -path "./projects/*" ! -path "*/src-gen/*" ! -path "*/target/*" -print0 \
        | while IFS= read -r -d '' file; do
            hadolint "$file"
        done
}

check_e2e() {
    echo "Running end-to-end tests..."
    cargo build -p plaatnotes --locked
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
    excludes=()
    while IFS= read -r arg; do
        excludes+=("$arg")
    done < <(platform_excludes)
    cargo llvm-cov nextest --all-features --locked --no-fail-fast --retries 2 --workspace "${excludes[@]}"
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

installable_apps() {
    cargo metadata --no-deps --format-version 1 2>/dev/null \
        | jq -r --arg os "$OS" '
            .packages[] |
            select(
                .metadata.bundle.name != null and
                .metadata.bundle.identifier != null and
                (
                    .metadata.platforms == null or
                    (.metadata.platforms | index($os) != null)
                )
            ) |
            .name + ":" + (.metadata.bundle.identifier | split(".") | last)
        ' | tr -d '\r'
}

build_bundle() {
    cargo install --path bin/cargo-bundle
    # shellcheck disable=SC2044
    for app in $(installable_apps); do
        pkg=${app%:*}
        cargo bundle --path "bin/$pkg"
    done
}

build_install_windows() {
    package=$1
    name=$2
    cargo build --release --bin "$package"
    cp "target/release/${package}.exe" "$USERPROFILE/Desktop/$name.exe"
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

    if [ "$OS" = "macos" ]; then
        build_bundle
        # shellcheck disable=SC2044
        for app in $(installable_apps); do
            pkg=${app%:*}
            cp -r "target/bundle/$pkg/"*.app /Applications
        done
    elif [ "$OS" = "windows" ]; then
        # shellcheck disable=SC2044
        for app in $(installable_apps); do
            pkg=${app%:*}
            name=${app#*:}
            build_install_windows "$pkg" "$name"
        done
    else
        mkdir -p ~/.local/bin ~/.local/share/applications ~/.local/share/icons
        # shellcheck disable=SC2044
        for app in $(installable_apps); do
            pkg=${app%:*}
            name=${app#*:}
            build_install_freedesktop "$pkg" "$name"
        done
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
        check_shell
        check_docker
        check_rust
        check_rust_deps
        check_e2e
        ;;
    check-shared)
        check_copyright
        check_formatting
        check_shell
        check_docker
        check_rust_deps
        ;;
    check-rust)
        check_rust
        ;;
    check-e2e)
        check_e2e
        ;;
    check-docker)
        check_docker
        ;;
    coverage)
        coverage
        ;;
    install)
        install
        ;;
    *)
        echo "Usage: $0 {build-pages|build-bundle|clean|check|check-shared|check-rust|check-e2e|check-docker|coverage|install}"
        exit 1
        ;;
esac
