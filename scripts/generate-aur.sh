#!/usr/bin/env bash
# Generate AUR (Arch User Repository) package files for a given tag.
# Usage: scripts/generate-aur.sh v0.9.0 [output-dir]
set -euo pipefail

TAG="${1:?usage: $0 <tag> [output-dir]}"
OUTDIR="${2:-aur}"
VERSION="${TAG#v}"

mkdir -p "$OUTDIR"

URL="https://github.com/natritmeyer/civterm/archive/refs/tags/${TAG}.tar.gz"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
echo "Fetching $URL"
curl -fsSL "$URL" -o "$tmp"
SHA="$(shasum -a 256 "$tmp" | awk '{print $1}')"

cat > "$OUTDIR/PKGBUILD" <<EOF
# Maintainer: Nat Ritmeyer <nat@natritmeyer.com>
pkgname=civterm
pkgver=${VERSION}
pkgrel=1
pkgdesc="A civilization game in your terminal"
arch=('x86_64' 'aarch64')
url="https://github.com/natritmeyer/civterm"
license=('MIT')
depends=('gcc-libs' 'glibc')
makedepends=('cargo' 'rust')
source=("\${pkgname}-\${pkgver}.tar.gz::https://github.com/natritmeyer/civterm/archive/refs/tags/v\${pkgver}.tar.gz")
sha256sums=('${SHA}')

build() {
  cd "\${pkgname}-\${pkgver}"
  cargo build --release --locked
}

package() {
  cd "\${pkgname}-\${pkgver}"
  install -Dm755 "target/release/civterm" "\${pkgdir}/usr/bin/civterm"
}
EOF

cat > "$OUTDIR/.SRCINFO" <<EOF
pkgbase = civterm
	pkgdesc = A civilization game in your terminal
	pkgver = ${VERSION}
	pkgrel = 1
	url = https://github.com/natritmeyer/civterm
	arch = x86_64
	arch = aarch64
	license = MIT
	makedepends = cargo
	makedepends = rust
	depends = gcc-libs
	depends = glibc
	source = civterm-${VERSION}.tar.gz::https://github.com/natritmeyer/civterm/archive/refs/tags/v${VERSION}.tar.gz
	sha256sums = ${SHA}

pkgname = civterm
EOF

echo "Generated AUR files in $OUTDIR"