#!/usr/bin/env bash
# Exercise the shipped launcher with a matching and then a modified runtime.
set -euo pipefail
repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
binary="$1"
probe() { command -v "$1" 2>/dev/null; }
# shellcheck source=.agent/scripts/runtime-id.sh
source "$repo_root/.agent/scripts/runtime-id.sh"
expected="$(source_fingerprint)"
[[ "$("$binary" runtime-id)" == "$expected" ]]
scratch="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-runtime-test.XXXXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT
git -C "$repo_root" archive HEAD | tar -xf - -C "$scratch"
mkdir -p "$scratch/.agent/cache/ccvl/bin"
cp "$binary" "$scratch/.agent/cache/ccvl/bin/ccvl"
chmod 0755 "$scratch/.agent/cache/ccvl/bin/ccvl"
bash "$scratch/ccvl" doctor >/dev/null
printf '\n// Runtime edited after installation.\n' >> "$scratch/.agent/src/main.rs"
if bash "$scratch/ccvl" doctor >"$scratch/.agent/cache/stale.log" 2>&1; then
  printf 'Launcher accepted a stale runtime.\n' >&2
  exit 1
fi
grep -Fq 'runtime is stale' "$scratch/.agent/cache/stale.log"
if "$binary" --root "$scratch" doctor >"$scratch/.agent/cache/direct.log" 2>&1; then
  printf 'Direct executable accepted a stale runtime.\n' >&2
  exit 1
fi
grep -Fq 'does not match the workspace runtime' "$scratch/.agent/cache/direct.log"
printf 'Matching binaries run; source changes are rejected by both launcher and executable.\n'
