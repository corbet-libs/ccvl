#!/usr/bin/env bash
# Package the exact native executable after native-release.sh's mandatory checks.
set -euo pipefail
repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"
exec python3 .agent/scripts/release-evidence.py native \
  --platform "${1:?Missing native platform}" --binary "${2:?Missing tested binary}" \
  --output "${3:?Missing package output directory}" \
  --host "${CCVL_RELEASE_RUST_HOST:?Run native-release.sh first}" \
  --rustc "${CCVL_RELEASE_RUSTC:?Missing compiler identity}" \
  --cargo "${CCVL_RELEASE_CARGO:?Missing Cargo identity}"
