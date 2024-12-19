.PHONY: all
all: ci

# Clean garbage
.PHONY: clean
clean:
	cargo clean

.PHONY: ci
ci:
# 	Format
	./meta/check_copyright.sh
	cargo +nightly fmt --all -- --check
# 	Lint
	cargo clippy --locked --all --all-targets -- -D warnings
	CARGO_TARGET_DIR=target/udeps cargo +nightly udeps --locked --all-targets
	cargo deny check --hide-inclusion-graph
# 	Test
	cargo nextest run

# Get test coverage
.PHONY: coverage
coverage:
	cargo llvm-cov nextest

# Build linux static release binaries
TARGETS = x86_64-unknown-linux-musl aarch64-unknown-linux-musl
.PHONY: release
release:
	$(foreach target,$(TARGETS),cargo zigbuild --release --target $(target);)
