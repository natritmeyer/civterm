.PHONY: setup build clean run dev build-release dist deb rpm formula update-tap release-publish

# Single source of truth for the version is Cargo.toml. Override on the
# command line (make VERSION=1.2.3) when needed.
VERSION ?= $(shell sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' Cargo.toml | head -n 1)
TARGET ?= $(shell rustc -vV | sed -n 's/host: //p')
DIST_DIR := dist

setup:
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-commit

clean:
	cargo clean
	find . -name .DS_Store -delete
	rm -rf $(DIST_DIR)

build:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo build
	cargo test

run:
	cargo run

dev:
	cargo build

build-release:
	cargo build --release --target $(TARGET)

dist: build-release

release-publish:
	cargo release $(VERSION) --no-publish
	mkdir -p $(DIST_DIR)
	tar -czf $(DIST_DIR)/civterm-$(TARGET).tar.gz -C target/$(TARGET)/release civterm

deb:
	@command -v cargo-deb >/dev/null 2>&1 || { echo "cargo-deb not installed: cargo install cargo-deb"; exit 1; }
	cargo deb --target $(TARGET)

rpm:
	@command -v cargo-generate-rpm >/dev/null 2>&1 || { echo "cargo-generate-rpm not installed: cargo install cargo-generate-rpm"; exit 1; }
	cargo generate-rpm --target $(TARGET)

formula:
	@./scripts/update_formula.sh v$(VERSION)

update-tap: formula
	@if [ -n "$(TAP_DIR)" ]; then \
		mkdir -p "$(TAP_DIR)/Formula"; \
		cp Formula/civterm.rb "$(TAP_DIR)/Formula/civterm.rb"; \
		echo "Copied Formula/civterm.rb -> $(TAP_DIR)/Formula/civterm.rb"; \
	else \
		echo "Formula updated in place. Set TAP_DIR=/path/to/homebrew-civterm to copy into a tap checkout."; \
	fi