.PHONY: all
all: ci

# Clean garbage
.PHONY: clean
clean:
	cargo clean

.PHONY: ci
ci:
	./meta/check_copyright.sh
# 	Lint
	cargo +nightly fmt --all -- --check
	cargo clippy --locked --all --all-targets -- -D warnings
# 	Build
	cargo build --locked --release
	CARGO_TARGET_DIR=target/udeps cargo +nightly udeps --locked --all-targets
	cargo deny check --hide-inclusion-graph
# 	Test
	cargo test --locked --all --all-targets

# Build linux static release binaries
TARGETS = x86_64-unknown-linux-musl aarch64-unknown-linux-musl
.PHONY: release
release:
	$(foreach target,$(TARGETS),cargo build --release --target $(target);)
