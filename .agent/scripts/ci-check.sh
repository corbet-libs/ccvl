#!/usr/bin/env bash
# Provider-independent checks using an already provisioned build worker.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
toolchain="${RUST_TOOLCHAIN:-1.94.0}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-2}"
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}"
export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-0}"

select_toolchain() {
  if command -v rustup >/dev/null; then
    # rustup run refuses a missing toolchain instead of installing one.
    rustup run "$toolchain" rustc --version
    cargo_command=(cargo "+$toolchain")
  else
    # Also support workers whose exact compiler is provisioned without rustup.
    installed=$(rustc --version)
    printf '%s\n' "$installed"
    [[ $(awk '{print $2}' <<<"$installed") == "$toolchain" ]] || {
      printf 'Required Rust %s is not installed; refusing an implicit toolchain change.\n' "$toolchain" >&2
      exit 2
    }
    cargo_command=(cargo)
  fi
}

if (($# == 0)); then
  set -- rust
fi
for check in "$@"; do
  case "$check" in
    rust)
      select_toolchain
      "${cargo_command[@]}" fmt --all -- --check
      "${cargo_command[@]}" test --locked --all-features
      "${cargo_command[@]}" clippy --locked --all-targets --all-features -- -D warnings
      ;;
    lint)
      actionlint -shellcheck shellcheck
      shellcheck .agent/scripts/*.sh .agent/tests/*.sh ccvl
      reuse lint
      ;;
    documents)
      [[ $(uname -s) == Linux ]] || { echo 'documents requires Linux' >&2; exit 2; }
      select_toolchain
      for command in file qpdf pdfinfo pdftotext pdffonts pdfdetach pdfimages pdftoppm jq; do
        command -v "$command" >/dev/null || { echo "Missing existing tool: $command" >&2; exit 2; }
      done
      "${cargo_command[@]}" build --locked --release
      binary="${CARGO_TARGET_DIR:-target}/release/ccvl"
      mkdir -p .agent/cache/ccvl/bin
      cp "$binary" .agent/cache/ccvl/bin/ccvl
      bash .agent/scripts/check-linux-deep.sh
      ;;
    *) echo "Unknown check: $check (rust lint documents)" >&2; exit 2 ;;
  esac
done
