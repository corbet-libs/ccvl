#!/usr/bin/env bash
# Native release validation shared by GHA and provisioned Crow workers.
set -euo pipefail
repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"
platform="${1:?Usage: native-release.sh PLATFORM OUTPUT}"
output="${2:?Missing package output directory}"
[[ -z ${GH_TOKEN:-}${GITHUB_TOKEN:-} ]] || { echo 'Publication credentials must not reach the native build.' >&2; exit 2; }
if [[ -n ${CI_REPO:-} ]]; then
  [[ -n ${CARGO_TARGET_DIR:-} && ${CCID_TARGET_LOCK_HELD:-} == "$CARGO_TARGET_DIR" ]] || {
    echo 'Crow native builds require the verified ccid target lock.' >&2; exit 2;
  }
fi
python3 .agent/scripts/release-evidence.py source >/dev/null
# shellcheck source=.agent/scripts/rust-toolchain.sh
source .agent/scripts/rust-toolchain.sh
ccvl_select_rust_toolchain
host="$("${CCVL_RUSTC_COMMAND[@]}" -vV | sed -n 's/^host: //p')"
case "$platform" in
  linux-x86_64) expected_host=x86_64-unknown-linux-gnu; native_os=Linux; native_arch=x86_64 ;;
  linux-aarch64) expected_host=aarch64-unknown-linux-gnu; native_os=Linux; native_arch=aarch64 ;;
  macos-x86_64) expected_host=x86_64-apple-darwin; native_os=Darwin; native_arch=x86_64 ;;
  macos-arm64) expected_host=aarch64-apple-darwin; native_os=Darwin; native_arch=arm64 ;;
  windows-x86_64) expected_host=x86_64-pc-windows-msvc; native_os=Windows; native_arch=AMD64 ;;
  windows-arm64) expected_host=aarch64-pc-windows-msvc; native_os=Windows; native_arch=ARM64 ;;
  *) echo "Unknown native platform: $platform" >&2; exit 2 ;;
esac
actual_os="$(uname -s)"
actual_arch="$(uname -m)"
if [[ "$actual_os" == Darwin && $(sysctl -in sysctl.proc_translated 2>/dev/null || true) == 1 ]]; then
  echo 'A translated macOS process cannot provide native release evidence.' >&2; exit 2
fi
if [[ "$actual_os" == MINGW* || "$actual_os" == MSYS* ]]; then
  actual_os=Windows
  actual_arch="${PROCESSOR_ARCHITEW6432:-${PROCESSOR_ARCHITECTURE:-}}"
fi
[[ "$host" == "$expected_host" && "$actual_os" == "$native_os" && "$actual_arch" == "$native_arch" ]] || {
  printf 'Native %s executor required; got %s/%s, Rust host %s. No cross-build substitution.\n' \
    "$platform" "$actual_os" "$actual_arch" "$host" >&2
  exit 2
}
exe=ccvl
[[ "$native_os" != Windows ]] || exe=ccvl.exe
# Explicitly build for the verified compiler's native host, ignoring configured cross targets.
"${CCVL_CARGO_COMMAND[@]}" build --locked --release --target "$host"
binary="${CARGO_TARGET_DIR:-$repo_root/target}/$host/release/$exe"
"$binary" public-check
if [[ "$native_os" == Windows ]]; then
  powershell.exe -NoProfile -ExecutionPolicy Bypass -File .agent/tests/test_bootstrap.ps1
  powershell.exe -NoProfile -ExecutionPolicy Bypass -File .agent/tests/test_runtime.ps1 -Binary "$(cygpath -w "$binary")"
else
  bash .agent/tests/test_runtime.sh "$binary"
fi
export CCVL_RELEASE_RUST_HOST="$host"
export CCVL_RELEASE_RUSTC
CCVL_RELEASE_RUSTC="$("${CCVL_RUSTC_COMMAND[@]}" --version)"
export CCVL_RELEASE_CARGO
CCVL_RELEASE_CARGO="$("${CCVL_CARGO_COMMAND[@]}" --version)"
bash .agent/scripts/package-release.sh "$platform" "$binary" "$output"
