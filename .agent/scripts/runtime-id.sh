#!/usr/bin/env bash
# Shared with bootstrap and launchers; keep the identity algorithm aligned with runtime_source.rs.
# shellcheck disable=SC2154
hash_stream() {
  if probe sha256sum >/dev/null; then
    sha256sum | awk '{ print $1 }'
  elif probe shasum >/dev/null; then
    shasum -a 256 | awk '{ print $1 }'
  else
    return 1
  fi
}

hash_file() {
  local path="$1"
  if probe sha256sum >/dev/null; then
    sha256sum "$path" | awk '{ print $1 }'
  elif probe shasum >/dev/null; then
    shasum -a 256 "$path" | awk '{ print $1 }'
  else
    return 1
  fi
}

source_fingerprint() {
  if [[ "${CCVL_BOOTSTRAP_TESTING:-0}" == 1 ]]; then
    printf '%s\n' "${CCVL_BOOTSTRAP_TEST_FINGERPRINT:-test-fingerprint}"
    return 0
  fi
  {
    for relative in Cargo.toml Cargo.lock rust-toolchain.toml .agent/build.rs; do
      printf '%s %s\n' "$relative" "$(hash_file "$repo_root/$relative")"
    done
    find "$repo_root/.agent/src" -type f -name '*.rs' -print | LC_ALL=C sort | while IFS= read -r path; do
      relative="${path#"$repo_root/"}"
      printf '%s %s\n' "$relative" "$(hash_file "$path")"
    done
  } | hash_stream
}
