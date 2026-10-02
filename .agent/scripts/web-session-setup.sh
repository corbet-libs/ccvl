#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Julian Y. Richard Corbet
# SPDX-License-Identifier: FSL-1.1-ALv2

# Cloud session setup (e.g. the Claude Code on the web environment setup
# script): install the matching ccvl runtime and the tools the checks and CI
# use. Idempotent; a ready container finishes in about a second.
set -euo pipefail

repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"
# Keep these in step with .github/workflows/ci.yml.
actionlint_version=1.7.12
actionlint_sha256=8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8
# A system bin directory when writable, so every later shell finds actionlint.
if [[ -w /usr/local/bin ]]; then tool_bin=/usr/local/bin; else tool_bin="$repo_root/.agent/cache/tools/bin"; fi
summary=()

probe() { command -v "$1" 2>/dev/null; }

# The same Ubuntu packages CI installs for document and lint checks.
missing=()
for package_command in file:file poppler-utils:pdftoppm qpdf:qpdf jq:jq shellcheck:shellcheck; do
  probe "${package_command#*:}" >/dev/null || missing+=("${package_command%%:*}")
done
if ((${#missing[@]})); then
  sudo_cmd=()
  ((EUID == 0)) || sudo_cmd=(sudo)
  "${sudo_cmd[@]}" apt-get update -qq
  DEBIAN_FRONTEND=noninteractive "${sudo_cmd[@]}" apt-get install -y -qq --no-install-recommends "${missing[@]}" >/dev/null
  summary+=("installed ${missing[*]}")
else
  summary+=("system tools present")
fi

# actionlint at CI's pinned version and checksum.
if [[ "$("$tool_bin/actionlint" -version 2>/dev/null | head -n 1)" != "$actionlint_version" ]]; then
  mkdir -p "$tool_bin"
  archive="$(mktemp)"
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
    "https://github.com/rhysd/actionlint/releases/download/v${actionlint_version}/actionlint_${actionlint_version}_linux_amd64.tar.gz" \
    --output "$archive"
  echo "$actionlint_sha256  $archive" | sha256sum --check --quiet
  tar -xzf "$archive" -C "$tool_bin" actionlint
  rm -f "$archive"
  summary+=("installed actionlint $actionlint_version")
fi
case ":$PATH:" in
  *":$tool_bin:"*) ;;
  *) summary+=("add $tool_bin to PATH for actionlint") ;;
esac

# The runtime whose embedded identity matches this checkout. A missing release
# is reported, never compiled here: that is an explicit developer action.
current_runtime() { bash ./ccvl runtime-id >/dev/null 2>&1; }
if current_runtime; then
  summary+=("ccvl runtime current")
else
  # setup ends with a full check; installation is judged by the runtime itself.
  setup_status=0
  setup_log="$(bash ./ccvl setup 2>&1)" || setup_status=$?
  if ! current_runtime; then
    printf '%s\n' "$setup_log" | tail -n 3 >&2
    summary+=("ccvl runtime NOT installed (no matching release); build it with: bash ./ccvl setup --from-source")
  elif ((setup_status)); then
    summary+=("installed ccvl runtime; its check reported problems: run bash ./ccvl check")
  else
    summary+=("installed ccvl runtime")
  fi
fi

joined="$(printf '%s; ' "${summary[@]}")"
printf 'ccvl session setup: %s\n' "${joined%; }"
