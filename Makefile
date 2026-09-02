.PHONY: build

build:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo build
	cargo test
