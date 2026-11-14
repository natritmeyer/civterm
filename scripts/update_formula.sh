#!/usr/bin/env bash
# Update the sha256 values in Formula/civterm.rb from a published release.
# Usage: scripts/update_formula.sh [v0.9.0]  (defaults to Cargo.toml's version)
set -euo pipefail

# Default the tag to the version declared in Cargo.toml when not given.
if [ $# -ge 1 ]; then
  TAG="$1"
else
  VER="$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' Cargo.toml | head -n 1)"
  TAG="v${VER}"
fi
VERSION="${TAG#v}"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

sha_mac_arm=""
sha_mac_intel=""
sha_lin_arm=""
sha_lin_intel=""

for spec in "civterm-macos-aarch64:mac_arm" "civterm-macos-x86_64:mac_intel" "civterm-linux-aarch64:lin_arm" "civterm-linux-x86_64:lin_intel"; do
  name="${spec%%:*}"
  var="${spec##*:}"
  url="https://github.com/natritmeyer/civterm/releases/download/${TAG}/${name}.tar.gz"
  echo "Fetching ${url}"
  curl -fsSL "$url" -o "${tmp}/${name}.tar.gz"
  h="$(shasum -a 256 "${tmp}/${name}.tar.gz" | awk '{print $1}')"
  case "$var" in
    mac_arm) sha_mac_arm="$h" ;;
    mac_intel) sha_mac_intel="$h" ;;
    lin_arm) sha_lin_arm="$h" ;;
    lin_intel) sha_lin_intel="$h" ;;
  esac
done

python3 - "$VERSION" "$sha_mac_arm" "$sha_mac_intel" "$sha_lin_arm" "$sha_lin_intel" <<'PY'
import sys
version, mac_arm, mac_intel, lin_arm, lin_intel = sys.argv[1:]
template = open("Formula/civterm.rb.tmpl").read()
out = (template
       .replace("{{VERSION}}", version)
       .replace("{{SHA_MAC_ARM}}", mac_arm)
       .replace("{{SHA_MAC_INTEL}}", mac_intel)
       .replace("{{SHA_LIN_ARM}}", lin_arm)
       .replace("{{SHA_LIN_INTEL}}", lin_intel))
open("Formula/civterm.rb", "w").write(out)
print("generated Formula/civterm.rb")
PY
