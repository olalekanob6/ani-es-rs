#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_version="$(cargo metadata --no-deps --format-version 1 \
    | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')"
pkgbuild_version="$(awk -F= '/^pkgver=/ { print $2; exit }' "$repo_root/PKGBUILD")"

if [[ "$pkgbuild_version" != "$cargo_version" ]]; then
    printf 'FAIL: Cargo.toml is %s but PKGBUILD is %s\n' "$cargo_version" "$pkgbuild_version" >&2
    exit 1
fi
if grep -Eq "^sha256sums=\('SKIP'\)$" "$repo_root/PKGBUILD"; then
    echo 'FAIL: PKGBUILD must pin the source archive SHA-256' >&2
    exit 1
fi

printf 'PASS: PKGBUILD matches Cargo.toml version %s and pins SHA-256\n' "$cargo_version"
