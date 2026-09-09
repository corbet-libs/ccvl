#!/usr/bin/env bash
# Select existing Rust tools without installing or updating an owned worker.

ccvl_rust_version_supported() {
  local output="$1" minimum="$2" actual_major actual_minor actual_patch
  local minimum_major minimum_minor minimum_patch
  [[ "$output" =~ ^rustc\ ([0-9]+)\.([0-9]+)\.([0-9]+)($|\ ) ]] || return 1
  actual_major="${BASH_REMATCH[1]}"
  actual_minor="${BASH_REMATCH[2]}"
  actual_patch="${BASH_REMATCH[3]}"
  IFS=. read -r minimum_major minimum_minor minimum_patch <<<"$minimum"
  minimum_patch="${minimum_patch:-0}"
  ((actual_major > minimum_major ||
    (actual_major == minimum_major && actual_minor > minimum_minor) ||
    (actual_major == minimum_major && actual_minor == minimum_minor && actual_patch >= minimum_patch)))
}

ccvl_rust_minimum() {
  sed -n 's/^[[:space:]]*rust-version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$1/Cargo.toml"
}

ccvl_existing_rustup_toolchain() {
  local rustup="$1" minimum="$2" requested="${3:-}" candidate output installed
  local candidates=()
  if [[ -n "$requested" ]]; then
    candidates=("$requested")
  else
    candidates=(stable)
    # List installed tools instead of resolving an override that may be absent.
    installed="$(cd "${TMPDIR:-/tmp}" && "$rustup" toolchain list 2>/dev/null)" || installed=
    while IFS= read -r candidate; do
      if [[ "$candidate" == *"("*default*")" ]]; then candidates+=("${candidate%% *}"); fi
    done <<<"$installed"
  fi
  for candidate in "${candidates[@]}"; do
    # Unlike rustup proxies, `run` fails if the toolchain is not installed.
    output="$("$rustup" run "$candidate" rustc --version 2>/dev/null)" || continue
    ccvl_rust_version_supported "$output" "$minimum" || continue
    "$rustup" run "$candidate" cargo --version >/dev/null 2>&1 || continue
    printf '%s\n' "$candidate"
    return 0
  done
  return 1
}

ccvl_select_rust_toolchain() {
  local repo_root minimum rustup selected output
  repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
  minimum="$(ccvl_rust_minimum "$repo_root")"
  [[ -n "$minimum" ]] || { echo 'Cargo.toml does not declare rust-version.' >&2; return 2; }
  if rustup="$(command -v rustup)"; then
    selected="$(ccvl_existing_rustup_toolchain "$rustup" "$minimum" "${RUST_TOOLCHAIN:-${RUSTUP_TOOLCHAIN:-}}")" || {
      printf 'No existing suitable Rust toolchain%s (minimum %s); no installation attempted.\n' \
        "${RUST_TOOLCHAIN:+: $RUST_TOOLCHAIN}" "$minimum" >&2
      return 2
    }
    CCVL_CARGO_COMMAND=("$rustup" run "$selected" cargo)
    output="$("$rustup" run "$selected" rustc --version)"
  else
    output="$(rustc --version)" || return 2
    ccvl_rust_version_supported "$output" "$minimum" || {
      printf 'Existing compiler %s does not meet Rust %s.\n' "$output" "$minimum" >&2
      return 2
    }
    if [[ -n "${RUST_TOOLCHAIN:-}" && "$RUST_TOOLCHAIN" != stable ]]; then
      local actual="${output#rustc }"
      [[ "$RUST_TOOLCHAIN" == "${actual%% *}" ]] || {
        printf 'Explicit Rust selector %s needs rustup or that installed compiler.\n' "$RUST_TOOLCHAIN" >&2
        return 2
      }
    fi
    CCVL_CARGO_COMMAND=(cargo)
  fi
  printf 'Rust compiler: %s (minimum %s)\n' "$output" "$minimum"
  "${CCVL_CARGO_COMMAND[@]}" --version
}
