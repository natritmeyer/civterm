.PHONY: build clean run

clean:
	cargo clean

build:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo build
	cargo test

run:
	cargo run
