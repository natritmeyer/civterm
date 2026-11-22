![Splash screen](images/civterm_screen.png)

# civterm

[![Made with Zed](https://img.shields.io/badge/Made%20with-Zed-084CCF?logo=zed)](https://zed.dev)
[![Made with OpenCode](https://img.shields.io/badge/Made%20with-OpenCode-8A2BE2?logo=zed)](https://opencode.ai)
[![Licence](https://img.shields.io/github/license/natritmeyer/civterm)](https://github.com/natritmeyer/civterm/blob/main/LICENSE)
[![Language](https://img.shields.io/github/languages/top/natritmeyer/civterm)](https://github.com/natritmeyer/civterm)
[![Repo size](https://img.shields.io/github/repo-size/natritmeyer/civterm)](https://github.com/natritmeyer/civterm)
[![Last commit](https://img.shields.io/github/last-commit/natritmeyer/civterm)](https://github.com/natritmeyer/civterm/commits/main)
[![Build status](https://img.shields.io/github/actions/workflow/status/natritmeyer/civterm/ci.yml?label=build)](https://github.com/natritmeyer/civterm/actions)
[![Tests](https://img.shields.io/badge/tests-870%20passing-brightgreen)](https://github.com/natritmeyer/civterm/actions)

civterm is a terminal Civilization-style game written in Rust. Lead a
civilization from a single settler to a sprawling empire: found and grow
cities, research advances, build units and improvements, explore the
world, and deal with rivals by diplomacy or war. The human player is
always civilization 0 (English by default).

The game works best with **Ghostty** as your terminal.

## Installation

### macOS

```sh
brew install natritmeyer/civterm/civterm
```

### Linux

Download the `.deb`/`.rpm` for your architecture from the
[releases page](https://github.com/natritmeyer/civterm/releases), then:

```sh
# Debian/Ubuntu (.deb)
sudo apt-get install ./civterm_*_amd64.deb

# Fedora/RHEL (.rpm)
sudo dnf install ./civterm-*.x86_64.rpm
```

Pre-built static (musl) binaries are also provided for minimal/Alpine
systems; download those from the
[releases page](https://github.com/natritmeyer/civterm/releases) too.

### Windows

Download a pre-built binary from the
[releases page](https://github.com/natritmeyer/civterm/releases).

### From source

```sh
git clone https://github.com/natritmeyer/civterm.git
cd civterm
cargo build --release
./target/release/civterm
```
