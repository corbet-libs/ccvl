#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Julian Y. Richard Corbet
# SPDX-License-Identifier: FSL-1.1-ALv2
# Isolated command fixtures: no network, builds, credentials or actual pushes.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture="$(mktemp -d)"
trap 'rm -rf -- "$fixture"' EXIT
mkdir -p "$fixture/bin" "$fixture/source/.agent/scripts"
cp "$repo_root/.agent/scripts/downstream-sync.sh" "$repo_root/.agent/scripts/rust-toolchain.sh" \
  "$fixture/source/.agent/scripts/"
cp "$repo_root/Cargo.toml" "$fixture/source/Cargo.toml"
export TEST_SOURCE_COMMIT=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
export TEST_MERGED_COMMIT=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
export TEST_TRACE="$fixture/trace"
export SOURCE_ARCHIVE="$fixture/source.tar" UPSTREAM_SOURCE_BUNDLE="$fixture/source.bundle"
printf 'source fixture\n' > "$SOURCE_ARCHIVE"
printf 'bundle fixture\n' > "$UPSTREAM_SOURCE_BUNDLE"
SOURCE_SHA256="$(sha256sum "$SOURCE_ARCHIVE" | awk '{print $1}')"
UPSTREAM_SOURCE_SHA256="$(sha256sum "$UPSTREAM_SOURCE_BUNDLE" | awk '{print $1}')"
export SOURCE_SHA256 UPSTREAM_SOURCE_SHA256
export CARGO_HOME="$fixture/cargo" CI_CACHE_ROOT="$fixture/cargo"
export CI_JOBS=1 CI_MIN_AVAILABLE_MB=1 CI_MEMORY_PER_JOB_MB=1
export DOWNSTREAM_PUSH_KEY=fixture-only DOWNSTREAM_SSH_URL=fixture-only
export PATH="$fixture/bin:$PATH"
unset RUST_TOOLCHAIN RUSTUP_TOOLCHAIN

cat > "$fixture/bin/git" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == -C ]]; then shift 2; fi
case "$1" in
  get-tar-commit-id) cat >/dev/null; printf '%s\n' "$TEST_SOURCE_COMMIT" ;;
  bundle)
    if [[ "$2" == list-heads ]]; then printf '%s refs/heads/source\n' "$TEST_SOURCE_COMMIT"; fi ;;
  init|remote|fetch|update-ref|config) ;;
  clone) mkdir -p "${@: -1}" ;;
  merge-base) exit 1 ;;
  merge) printf 'merge\n' >> "$TEST_TRACE" ;;
  rev-parse)
    if [[ "$2" == FETCH_HEAD ]]; then printf '%s\n' "$TEST_SOURCE_COMMIT"
    else printf '%s\n' "$TEST_MERGED_COMMIT"; fi ;;
  ls-remote)
    if [[ "$3" == upstream ]]; then
      printf 'upstream-freshness\n' >> "$TEST_TRACE"
      case "$TEST_MAIN_STATE" in
        unchanged) printf '%s\trefs/heads/main\n' "$TEST_SOURCE_COMMIT" ;;
        advanced) printf '%s\trefs/heads/main\n' "$TEST_MERGED_COMMIT" ;;
        unavailable) exit 128 ;;
      esac
    else printf '%s\trefs/heads/main\n' "$TEST_MERGED_COMMIT"; fi ;;
  push) printf 'push\n' >> "$TEST_TRACE" ;;
  *) printf 'Unexpected fixture Git command: %s\n' "$1" >&2; exit 99 ;;
esac
SH
cat > "$fixture/bin/rustup" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == run ]] || exit 99
shift 2
case "$1" in
  rustc) echo 'rustc 1.97.1 (fixture)' ;;
  cargo)
    if [[ "$2" == --version ]]; then echo 'cargo 1.97.1 (fixture)'; exit 0; fi
    [[ "$2" == build && "$3" == --locked ]] || exit 99
    mkdir -p "$CARGO_TARGET_DIR/debug"
    cat > "$CARGO_TARGET_DIR/debug/ccvl" <<'GATE'
#!/usr/bin/env bash
set -euo pipefail
printf 'gate:%s\n' "$1" >> "$TEST_TRACE"
if [[ "$1" == check && "$TEST_DOCUMENT_GATE" == failure ]]; then exit 1; fi
GATE
    chmod +x "$CARGO_TARGET_DIR/debug/ccvl"
    ;;
  *) exit 99 ;;
esac
SH
# These fixtures exercise publication ordering without depending on host memory.
cat > "$fixture/bin/python3" <<'SH'
#!/usr/bin/env bash
cat >/dev/null
printf '2\n'
SH
chmod +x "$fixture/bin/git" "$fixture/bin/rustup" "$fixture/bin/python3"

run_case() {
  export TEST_MAIN_STATE="$1" TEST_DOCUMENT_GATE="$2"
  : > "$TEST_TRACE"
  if bash "$fixture/source/.agent/scripts/downstream-sync.sh" "$TEST_SOURCE_COMMIT" > "$fixture/output" 2>&1; then
    [[ "$3" == success ]] || { cat "$fixture/output"; echo 'Expected sync failure.' >&2; exit 1; }
  else
    [[ "$3" == failure ]] || { cat "$fixture/output"; echo 'Unexpected sync failure.' >&2; exit 1; }
  fi
  if [[ "$3" == failure ]] && grep -qx push "$TEST_TRACE"; then
    echo 'Failed verification attempted a private push.' >&2
    exit 1
  fi
}
run_case unchanged success success
expected=$'merge\ngate:downstream-check\ngate:check\nupstream-freshness\npush'
[[ "$(cat "$TEST_TRACE")" == "$expected" ]] || { cat "$TEST_TRACE"; exit 1; }
run_case advanced success failure
run_case unavailable success failure
run_case unchanged failure failure
printf 'Downstream freshness and failed-gate publication guards passed.\n'
