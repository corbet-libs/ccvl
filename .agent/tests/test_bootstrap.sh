#!/usr/bin/env bash
set -euo pipefail

# Keep the simulated probe matrix independent from the caller's environment.
export CCVL_BOOTSTRAP_FORCE_LOCAL=0
export CCVL_BOOTSTRAP_TESTING=1
export CCVL_BOOTSTRAP_TEST_FINGERPRINT=test-fingerprint

repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-bootstrap-test.XXXXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT

create_fake() {
  local directory="$1"
  local name="$2"
  local output="${3:-}"
  mkdir -p "$directory"
  printf '#!/bin/sh\nprintf "%%s\\n" %q\n' "$output" > "$directory/$name"
  chmod 0755 "$directory/$name"
}

create_stable_rustup() {
  local directory="$1"
  mkdir -p "$directory"
  printf '%s\n' \
    '#!/bin/sh' \
    'case "$*" in' \
    '  "run stable rustc --version") printf "%s\n" "rustc 1.94.0 (test)" ;;' \
    '  "run stable cargo --version") printf "%s\n" "cargo 1.94.0 (test)" ;;' \
    '  *) exit 1 ;;' \
    'esac' > "$directory/rustup"
  chmod 0755 "$directory/rustup"
}

mark_binary_ready() {
  local cache_root="$1"
  create_fake "$cache_root/bin" ccvl 'test-fingerprint'
  printf 'test-fingerprint\n' > "$cache_root/install.sha256"
}

complete_bin="$scratch/complete-bin"
empty_bin="$scratch/empty-bin"
partial_bin="$scratch/partial-bin"
mkdir -p "$complete_bin" "$empty_bin" "$partial_bin"

for command_name in curl sha256sum cc; do
  create_fake "$complete_bin" "$command_name"
  create_fake "$partial_bin" "$command_name"
done
create_fake "$complete_bin" rustc 'rustc 1.94.0 (test)'
create_fake "$complete_bin" cargo 'cargo 1.94.0 (test)'

complete_cache="$scratch/complete-cache"
mark_binary_ready "$complete_cache"
complete_output="$(
  CCVL_BOOTSTRAP_PROBE_PATH="$complete_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$complete_cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
  CCVL_BOOTSTRAP_TEST_MANAGER=apt \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$complete_output" == *'Rust toolchain: system 1.94.0'* ]]
[[ "$complete_output" == *'ccvl binary: ready'* ]]
[[ "$complete_output" == *'missing bootstrap commands: none'* ]]
[[ "$complete_output" == *'No ccvl build changes required.'* ]]

# A newer standalone stable compiler is reused; an old/prerelease compiler is not.
for rust_version in 1.97.0 1.100.0 1.93.9 1.94.0-nightly invalid; do
  create_fake "$complete_bin" rustc "rustc $rust_version (test)"
  version_output="$(
    CCVL_BOOTSTRAP_PROBE_PATH="$complete_bin" \
    CCVL_BOOTSTRAP_CACHE_ROOT="$scratch/version-$rust_version" \
    CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
    CCVL_BOOTSTRAP_TEST_MANAGER=apt \
      bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
  )"
  case "$rust_version" in
    1.97.0 | 1.100.0) [[ "$version_output" == *"Rust toolchain: system $rust_version"* ]] ;;
    *) [[ "$version_output" == *'Rust toolchain: install stable '* ]] ;;
  esac
done

empty_cache="$scratch/empty-cache"
empty_output="$(
  CCVL_BOOTSTRAP_PROBE_PATH="$empty_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$empty_cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
  CCVL_BOOTSTRAP_TEST_MANAGER=apt \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$empty_output" == *'Rust toolchain: install stable with pinned rustup-init 1.29.1'* ]]
[[ "$empty_output" == *'ccvl binary: install'* ]]
[[ "$empty_output" == *'missing bootstrap commands: checksum compiler downloader'* ]]
[[ "$empty_output" == *'host packages: coreutils build-essential curl'* ]]
[[ "$empty_output" == *'No changes made.'* ]]
[[ ! -e "$empty_cache" ]]

ready_without_toolchain_cache="$scratch/ready-without-toolchain-cache"
mark_binary_ready "$ready_without_toolchain_cache"
ready_without_toolchain_output="$(
  CCVL_BOOTSTRAP_PROBE_PATH="$empty_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$ready_without_toolchain_cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
  CCVL_BOOTSTRAP_TEST_MANAGER=apt \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$ready_without_toolchain_output" == *'ccvl binary: ready'* ]]
[[ "$ready_without_toolchain_output" == *'missing bootstrap commands: none'* ]]
[[ "$ready_without_toolchain_output" == *'host packages: none'* ]]

partial_cache="$scratch/partial-cache"
partial_output="$(
  CCVL_BOOTSTRAP_PROBE_PATH="$partial_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$partial_cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-aarch64 \
  CCVL_BOOTSTRAP_TEST_MANAGER=apt \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$partial_output" == *'platform: Linux-aarch64'* ]]
[[ "$partial_output" == *'Rust toolchain: install stable with pinned rustup-init 1.29.1'* ]]
[[ "$partial_output" == *'ccvl binary: install'* ]]
[[ "$partial_output" == *'missing bootstrap commands: none'* ]]

managed_cache="$scratch/managed-cache"
create_stable_rustup "$managed_cache/cargo/bin"
mark_binary_ready "$managed_cache"
managed_output="$(
  CCVL_BOOTSTRAP_FORCE_LOCAL=1 \
  CCVL_BOOTSTRAP_PROBE_PATH="$partial_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$managed_cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$managed_output" == *'Rust toolchain: managed 1.94.0'* ]]
[[ "$managed_output" == *'ccvl binary: ready'* ]]

mv "$managed_cache/bin/ccvl" "$scratch/ccvl-away"
managed_partial_output="$(
  CCVL_BOOTSTRAP_FORCE_LOCAL=1 \
  CCVL_BOOTSTRAP_PROBE_PATH="$partial_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$managed_cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$managed_partial_output" == *'Rust toolchain: managed 1.94.0'* ]]
[[ "$managed_partial_output" == *'ccvl binary: install'* ]]

system_rustup_bin="$scratch/system-rustup-bin"
for command_name in curl sha256sum cc; do
  create_fake "$system_rustup_bin" "$command_name"
done
create_stable_rustup "$system_rustup_bin"
system_rustup_cache="$scratch/system-rustup-cache"
mark_binary_ready "$system_rustup_cache"
system_rustup_output="$(
  CCVL_BOOTSTRAP_FORCE_LOCAL=0 \
  CCVL_BOOTSTRAP_PROBE_PATH="$system_rustup_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$system_rustup_cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$system_rustup_output" == *'Rust toolchain: system 1.94.0'* ]]
[[ "$system_rustup_output" == *'ccvl binary: ready'* ]]

# A versioned provisioned default works even without a stable alias.
cat > "$system_rustup_bin/rustup" <<'EOF'
#!/bin/sh
case "$*" in
  'toolchain list') printf '%s\n' '1.97.0-test-host (default)' ;;
  'run 1.97.0-test-host rustc --version') printf '%s\n' 'rustc 1.97.0 (test)' ;;
  'run 1.97.0-test-host cargo --version') printf '%s\n' 'cargo 1.97.0 (test)' ;;
  *) exit 1 ;;
esac
EOF
versioned_rustup_output="$(
  CCVL_BOOTSTRAP_PROBE_PATH="$system_rustup_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$scratch/versioned-system-cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$versioned_rustup_output" == *'Rust toolchain: system 1.97.0'* ]]

mac_empty_bin="$scratch/mac-empty-bin"
mkdir -p "$mac_empty_bin"
create_fake "$mac_empty_bin" shasum
mac_empty_output="$(
  CCVL_BOOTSTRAP_FORCE_LOCAL=1 \
  CCVL_BOOTSTRAP_PROBE_PATH="$mac_empty_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$scratch/mac-empty-cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Darwin-aarch64 \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$mac_empty_output" == *'Rust toolchain: install stable with Homebrew rustup'* ]]
[[ "$mac_empty_output" == *'missing bootstrap commands: homebrew rustup'* ]]
[[ "$mac_empty_output" == *'host packages: Homebrew rustup'* ]]
[[ "$mac_empty_output" == *'Homebrew install action:'* ]]
[[ "$mac_empty_output" == *'Homebrew package action: brew install rustup'* ]]

mac_partial_bin="$scratch/mac-partial-bin"
mac_prefix="$scratch/homebrew/opt/rustup"
mkdir -p "$mac_partial_bin"
create_fake "$mac_partial_bin" shasum
create_fake "$mac_partial_bin" brew "$mac_prefix"
mac_partial_output="$(
  CCVL_BOOTSTRAP_FORCE_LOCAL=1 \
  CCVL_BOOTSTRAP_PROBE_PATH="$mac_partial_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$scratch/mac-partial-cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Darwin-x86_64 \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$mac_partial_output" == *'missing bootstrap commands: rustup'* ]]
[[ "$mac_partial_output" == *'host packages: rustup'* ]]
[[ "$mac_partial_output" != *'Homebrew install action:'* ]]

create_stable_rustup "$mac_prefix/bin"
mac_complete_cache="$scratch/mac-complete-cache"
mark_binary_ready "$mac_complete_cache"
mac_complete_output="$(
  CCVL_BOOTSTRAP_FORCE_LOCAL=1 \
  CCVL_BOOTSTRAP_PROBE_PATH="$mac_partial_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$mac_complete_cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Darwin-x86_64 \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source
)"
[[ "$mac_complete_output" == *'Rust toolchain: managed 1.94.0'* ]]
[[ "$mac_complete_output" == *'ccvl binary: ready'* ]]
[[ "$mac_complete_output" == *'host packages: none'* ]]

if CCVL_BOOTSTRAP_TEST_PLATFORM=Plan9-x86_64 \
  bash "$repo_root/.agent/scripts/bootstrap.sh" plan --from-source >/dev/null 2>&1; then
  printf 'Unsupported platforms must fail.\n' >&2
  exit 1
fi

if CCVL_BOOTSTRAP_PROBE_PATH="$empty_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$scratch/managerless-cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
  bash "$repo_root/.agent/scripts/bootstrap.sh" install --from-source >/dev/null 2>"$scratch/no-manager-error"; then
  printf 'Installation without a package manager must fail.\n' >&2
  exit 1
fi
grep -Fxq \
  'No supported package manager found for missing bootstrap commands: checksum compiler downloader' \
  "$scratch/no-manager-error"

fetch_bin="$scratch/fetch-bin"
fetch_fixtures="$scratch/fetch-fixtures"
fetch_cache="$scratch/fetch-cache"
mkdir -p "$fetch_bin" "$fetch_fixtures"
create_fake "$fetch_bin" cc
if command -v sha256sum >/dev/null 2>&1; then
  ln -s "$(command -v sha256sum)" "$fetch_bin/sha256sum"
else
  ln -s "$(command -v shasum)" "$fetch_bin/shasum"
fi
printf '#!/bin/sh\nprintf "test-fingerprint\\n"\n' > "$fetch_fixtures/ccvl-linux-x86_64"
chmod 0755 "$fetch_fixtures/ccvl-linux-x86_64"
if command -v sha256sum >/dev/null 2>&1; then
  fixture_hash="$(sha256sum "$fetch_fixtures/ccvl-linux-x86_64" | awk '{ print $1 }')"
else
  fixture_hash="$(shasum -a 256 "$fetch_fixtures/ccvl-linux-x86_64" | awk '{ print $1 }')"
fi
printf '%s  %s\n' "$fixture_hash" "ccvl-linux-x86_64" > "$fetch_fixtures/ccvl-linux-x86_64.sha256"
cat > "$fetch_bin/curl" <<EOF
#!/bin/sh
out=""
url=""
prev=""
for arg in "\$@"; do
  if [ "\$prev" = "--output" ]; then out="\$arg"; fi
  case "\$arg" in https://*) url="\$arg" ;; esac
  prev="\$arg"
done
cp "$fetch_fixtures/\${url##*/}" "\$out"
EOF
chmod 0755 "$fetch_bin/curl"
fetch_plan="$(
  CCVL_BOOTSTRAP_PROBE_PATH="$fetch_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$fetch_cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
  CCVL_BOOTSTRAP_TEST_MANAGER=apt \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan
)"
[[ "$fetch_plan" == *'ccvl binary: install'* ]]
[[ "$fetch_plan" == *'prebuilt binary: ccvl-linux-x86_64 (matching runtime required)'* ]]
CCVL_BOOTSTRAP_PROBE_PATH="$fetch_bin" \
CCVL_BOOTSTRAP_CACHE_ROOT="$fetch_cache" \
CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
CCVL_BOOTSTRAP_TEST_MANAGER=apt \
  bash "$repo_root/.agent/scripts/bootstrap.sh" install >/dev/null
[[ -x "$fetch_cache/bin/ccvl" ]]

fetch_ready="$(
  CCVL_BOOTSTRAP_PROBE_PATH="$fetch_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$fetch_cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
  CCVL_BOOTSTRAP_TEST_MANAGER=apt \
    bash "$repo_root/.agent/scripts/bootstrap.sh" plan
)"
[[ "$fetch_ready" == *'ccvl binary: ready'* ]]

# A checksum-valid binary from another runtime must not be installed, even
# when an old cache stamp claims it belongs to this source tree.
export CCVL_BOOTSTRAP_TEST_FINGERPRINT=changed-runtime
if CCVL_BOOTSTRAP_PROBE_PATH="$fetch_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$scratch/stale-cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
    bash "$repo_root/.agent/scripts/bootstrap.sh" install >"$scratch/stale-log" 2>&1; then
  printf 'Stale prebuilt runtime must fail.\n' >&2
  exit 1
fi
[[ ! -e "$scratch/stale-cache/bin/ccvl" ]]
grep -Fq 'Downloaded binary does not match' "$scratch/stale-log"
export CCVL_BOOTSTRAP_TEST_FINGERPRINT=test-fingerprint
printf 'incorrect-checksum  ccvl-linux-x86_64\n' > "$fetch_fixtures/ccvl-linux-x86_64.sha256"
if CCVL_BOOTSTRAP_PROBE_PATH="$fetch_bin" \
  CCVL_BOOTSTRAP_CACHE_ROOT="$scratch/corrupt-cache" \
  CCVL_BOOTSTRAP_TEST_PLATFORM=Linux-x86_64 \
    bash "$repo_root/.agent/scripts/bootstrap.sh" install >"$scratch/corrupt-log" 2>&1; then
  printf 'Corrupt prebuilt runtime must fail.\n' >&2
  exit 1
fi
[[ ! -e "$scratch/corrupt-cache/bin/ccvl" ]]
grep -Fq 'Checksum mismatch' "$scratch/corrupt-log"

printf 'POSIX bootstrap handles Linux and macOS empty, partial, complete, unsupported, and manager-less states.\n'
