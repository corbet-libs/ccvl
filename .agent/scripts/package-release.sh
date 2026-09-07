#!/usr/bin/env bash
# Package the already-tested executable together with its exact workspace.
set -euo pipefail
repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"
name="$1"
exe="$2"
output="$3"
mkdir -p "$output"
output="$(cd "$output" && pwd)"
package="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-package.XXXXXXXX")"
trap 'rm -rf -- "$package"' EXIT
git archive HEAD | tar -xf - -C "$package"
mkdir -p "$package/.agent/cache/ccvl/bin"
cp "target/release/$exe" "$package/.agent/cache/ccvl/bin/$exe"
cp "target/release/$exe" "$output/$name"
chmod 0755 "$output/$name" "$package/.agent/cache/ccvl/bin/$exe"
"$output/$name" runtime-id > "$output/$name.runtime-id"
tar -czf "$output/${name%.exe}.tar.gz" -C "$package" .
(
  cd "$output"
  sha256sum "$name" > "$name.sha256"
  sha256sum "${name%.exe}.tar.gz" > "${name%.exe}.tar.gz.sha256"
)
