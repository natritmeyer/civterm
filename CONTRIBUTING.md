# Contributing to civterm

Thanks for taking a look. This is a terminal Civilization clone written in Rust.
Everything needed to build and test it is described below.

## Prerequisites

- **Rust** — install with [rustup](https://rustup.rs). The toolchain is pinned in
  `rust-toolchain.toml` to `stable` with `clippy`, `rustfmt` and
  `llvm-tools-preview`; rustup installs those automatically on first build.
- **make** — the repo gate and setup are Makefile targets. On macOS this comes
  with the Xcode command-line tools; on Debian/Ubuntu install `build-essential`.
- **git**

No other system libraries are needed; the UI is pure terminal (crossterm +
ratatui).

## Setup

Clone the repo and run the one-time setup target. It points git at the committed
hooks and makes the pre-commit hook executable:

```sh
git clone https://github.com/natritmeyer/civterm.git
cd civterm
make setup
```

The pre-commit hook runs `make build` before every commit. You can skip it for a
single commit with `git commit --no-verify`.

## Build and test

The single gate for the whole repo is:

```sh
make build
```

which runs, in order:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build
cargo test
```

Run it before opening a pull request — CI runs exactly the same command. To fix
formatting failures automatically, run `cargo fmt --all` first.

Useful targets and commands:

```sh
make run          # build and launch the game
cargo test        # tests only
make clean        # cargo clean + remove .DS_Store files
```

Tests are colocated with the code (`#[cfg(test)] mod tests;` beside each module).
New behaviour should come with tests, and the baseline count is kept in step in
`AGENTS.md` and the README badge.

## Running

```sh
make run
```

or `cargo run`. The game needs an interactive terminal. `civterm.civ` in the repo
root is a sample saved game you can load from the menu (`L`).

## Before you start

The project has a few strong conventions, documented in:

- **`AGENTS.md`** — module layout, layering rules (notably: `model` must never
  reference `game_engine`), movement/reveal invariants, save/load policy, and the
  behaviour each subsystem is expected to keep.
- **`ARCHITECTURE.md`** — the broader design: module boundaries, command flow,
  turn loop and rendering passes.

Skim both before a substantial change. `scratch/` holds working reference notes
and is not part of the build.

## Pull requests

- Keep changes focused and run `make build` before submitting.
- Match the existing code style; `cargo fmt` and clippy enforce most of it.
- Do not commit secrets or generated artifacts (`target/` is ignored).
