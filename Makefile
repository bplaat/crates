.PHONY: all
all: ci

.PHONY: clean
clean:
	cargo clean

.PHONY: ci
ci:
	./meta/check_copyright.sh
	cargo +nightly fmt --all -- --check
	cargo clippy --locked --all --all-targets -- -D warnings
	cargo build --locked --release
	CARGO_TARGET_DIR=target/udeps cargo +nightly udeps --locked --all-targets
	cargo deny check --hide-inclusion-graph

TARGETS = x86_64-unknown-linux-musl aarch64-unknown-linux-musl
.PHONY: release
release:
	$(foreach target,$(TARGETS),cargo build --release --target $(target);)
