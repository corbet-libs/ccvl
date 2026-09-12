#!/usr/bin/env bash
set -euo pipefail

repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-ci-toolchain.XXXXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT
bash_command="$(command -v bash)"
mkdir -p "$scratch/bin"
for utility in dirname sed; do
  ln -s "$(command -v "$utility")" "$scratch/bin/$utility"
done
cat > "$scratch/bin/rustc" <<'EOF'
#!/bin/sh
printf 'rustc %s (fixture)\n' "$FAKE_RUST_VERSION"
EOF
cat > "$scratch/bin/cargo" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$FAKE_CARGO_LOG"
EOF
chmod +x "$scratch/bin/rustc" "$scratch/bin/cargo"
export FAKE_CARGO_LOG="$scratch/cargo.log"
export FAKE_RUST_VERSION=1.97.0
unset RUST_TOOLCHAIN RUSTUP_TOOLCHAIN

run_check() {
  : > "$FAKE_CARGO_LOG"
  PATH="$scratch/bin" "$bash_command" "$repo_root/.agent/scripts/ci-check.sh" rust > "$scratch/output" 2>&1
}

run_check
grep -Fq 'rustc 1.97.0' "$scratch/output"
grep -Fxq 'test --locked --workspace --all-features' "$FAKE_CARGO_LOG"
grep -Fxq 'clippy --locked --workspace --all-targets --all-features -- -D warnings' "$FAKE_CARGO_LOG"
export FAKE_RUST_VERSION=1.93.9
if run_check; then echo 'CI accepted a compiler below the minimum.' >&2; exit 1; fi
[[ ! -s "$FAKE_CARGO_LOG" ]]

cat > "$scratch/bin/rustup" <<'EOF'
#!/bin/sh
case "$*" in
  'toolchain list') printf '%s\n' '1.97.0-test-host (default)' ;;
  'run stable rustc --version')
    test "${FAKE_STABLE_AVAILABLE:-0}" = 1 || exit 1
    printf '%s\n' 'rustc 1.98.0 (fixture)' ;;
  'run 1.97.0-test-host rustc --version') printf '%s\n' 'rustc 1.97.0 (fixture)' ;;
  'run stable cargo '* | 'run 1.97.0-test-host cargo '*)
    printf '%s\n' "$*" >> "$FAKE_CARGO_LOG" ;;
  *) printf 'Unexpected rustup operation: %s\n' "$*" >&2; exit 1 ;;
esac
EOF
chmod +x "$scratch/bin/rustup"
run_check
grep -Fq 'rustc 1.97.0' "$scratch/output"
grep -Fxq 'run 1.97.0-test-host cargo test --locked --workspace --all-features' "$FAKE_CARGO_LOG"
export FAKE_STABLE_AVAILABLE=1
run_check
grep -Fq 'rustc 1.98.0' "$scratch/output"
grep -Fxq 'run stable cargo test --locked --workspace --all-features' "$FAKE_CARGO_LOG"

export RUST_TOOLCHAIN=missing-explicit-toolchain
if run_check; then echo 'CI ignored an unavailable explicit toolchain.' >&2; exit 1; fi
[[ ! -s "$FAKE_CARGO_LOG" ]]
printf 'CI reuses suitable existing Rust, preserves explicit selection, and never installs a toolchain.\n'
