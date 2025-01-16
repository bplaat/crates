PROFILE ?= default

.PHONY: all
all: check

# Clean garbage
.PHONY: clean
clean:
	cargo clean

.PHONY: check
check:
# 	Format
	./meta/check_copyright.sh
	cargo +nightly fmt -- --check
# 	Lint
	cargo clippy --locked --all-targets --all-features -- -D warnings
	cargo deny check --hide-inclusion-graph
# 	Test
	cargo test --doc --all-features --locked
	cargo nextest run --all-features --locked --config-file nextest.toml --profile $(PROFILE)

# Get test coverage
.PHONY: coverage
coverage:
	cargo llvm-cov nextest --all-features --locked --config-file nextest.toml --profile $(PROFILE)

# Build release binaries
TARGETS = x86_64-unknown-linux-musl aarch64-unknown-linux-musl \
	x86_64-apple-darwin aarch64-apple-darwin \
	x86_64-pc-windows-gnu
.PHONY: release
release:
	$(foreach target,$(TARGETS),cargo zigbuild --release --target $(target);)
	rm .intentionally-empty-file.o
