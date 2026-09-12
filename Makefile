.PHONY: setup build clean run

setup:
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-commit

clean:
	cargo clean

build:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo build
	cargo test

run:
	cargo run
