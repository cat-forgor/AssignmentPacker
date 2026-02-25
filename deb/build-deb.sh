#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?usage: build-deb.sh <version> <binary-path>}"
BIN="${2:?usage: build-deb.sh <version> <binary-path>}"

PKG="ap_${VERSION}_amd64"
mkdir -p "${PKG}/DEBIAN" "${PKG}/usr/bin"

cp "$BIN" "${PKG}/usr/bin/ap"
chmod 755 "${PKG}/usr/bin/ap"

cat > "${PKG}/DEBIAN/control" <<EOF
Package: ap
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: cat-forgor <catforgor@users.noreply.github.com>
Homepage: https://github.com/cat-forgor/AssignmentPacker
Description: CLI tool that packs C assignment submissions for Canvas upload
 Builds the exact folder and zip structure Canvas wants for C assignment
 submissions. Can auto-generate a .doc with source code and a terminal
 screenshot of the program running.
EOF

dpkg-deb --build "$PKG"
echo "Built ${PKG}.deb"
