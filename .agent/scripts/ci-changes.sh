#!/usr/bin/env bash
# Map changed paths to the CI checks they require. A false skip is worse than a
# slow run: unknown paths, unusable ranges and other events select everything.
#
# Rust tests never read the showcase styles, shared families, profile or assets
# under cvl/: they use the synthetic workspace in .agent/tests/fixtures, and the
# real-data invariants they used to check (frozen contracts, leaf inventory,
# tracked outputs, measurement ownership, document isolation, portable copies,
# the station plan) are enforced by `ccvl check`, which the document job runs.
# So cvl/ style, family, profile and asset changes select only the document
# job. cvl/README.md and every path outside cvl/ that Rust tests still read
# (skills, scaffolds, schemas, fixtures, data-root READMEs, licenses bundled
# by style exports, CI scripts) select the Rust job.
#
#   ci-changes.sh classify        read changed paths, one per line, from stdin
#   ci-changes.sh range BASE HEAD classify `git diff BASE...HEAD` (merge base)
#   ci-changes.sh github          select from the GitHub event; write GITHUB_OUTPUT
#
# Output lines:
#   rust=true|false       the Rust job (format, tests, Clippy) must run
#   documents=true|false  the document job must run
#   styles=all|<doc>/<style> ...  styles whose documents the document job checks
set -euo pipefail

repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
rust=false
all_styles=false
styles=
reason=

select_everything() {
  rust=true
  all_styles=true
  reason="${reason:-$1}"
}

add_style() {
  styles+="$1"$'\n'
}

# A touched style that no longer exists (removed or renamed) changes the
# inventory, so every style is checked; the engine rejects unknown filters.
touch_style() {
  local document="$1" style="$2"
  if [[ -f "$repo_root/cvl/$document/$style/style.toml" ]]; then
    add_style "$document/$style"
  else
    select_everything "removed or renamed style cvl/$document/$style"
  fi
}

# Shared family files serve the styles that reference them (style.toml
# defaults, Typst imports) and the same-named styles. Unreferenced families
# select every style.
touch_family() {
  local family="$1" found=false file relative document style
  for document in cv cl; do
    if [[ -f "$repo_root/cvl/$document/$family/style.toml" ]]; then
      add_style "$document/$family"
      found=true
    fi
  done
  while IFS= read -r file; do
    [[ -n "$file" ]] || continue
    relative="${file#"$repo_root"/cvl/}"
    document="${relative%%/*}"
    relative="${relative#*/}"
    style="${relative%%/*}"
    if [[ "$relative" == */* && -f "$repo_root/cvl/$document/$style/style.toml" ]]; then
      add_style "$document/$style"
      found=true
    fi
  done < <(grep -rlIF -- "shared/$family/" "$repo_root/cvl/cv" "$repo_root/cvl/cl" 2>/dev/null || true)
  [[ "$found" == true ]] || select_everything "unreferenced shared family cvl/shared/$family"
}

# The interview station plan is rendered into, and validated by, every CV
# style whose contract declares the station layout protocol.
touch_station_plan() {
  local contract style
  for contract in "$repo_root"/cvl/cv/*/contract.toml; do
    [[ -f "$contract" ]] || continue
    if grep -q '^\[layout_contract' "$contract"; then
      style="${contract%/contract.toml}"
      add_style "cv/${style##*/}"
    fi
  done
}

classify_path() {
  local path="$1" rest
  case "$path" in
    # The selection itself: a change must prove the full workflow.
    .github/workflows/ci.yml | .agent/scripts/ci-changes.sh)
      select_everything "CI selection changed: $path" ;;
    # Engine, renderer and workspace manifest: every document can change.
    .agent/src/* | .agent/core/* | .agent/build.rs | .agent/typst/* | \
      Cargo.toml | Cargo.lock | rust-toolchain.toml | ccvl.json)
      select_everything "engine input changed: $path" ;;
    # Showcase styles and shared families: their documents only.
    cvl/cv/*/* | cvl/cl/*/*)
      rest="${path#cvl/*/}"
      touch_style "${path:4:2}" "${rest%%/*}" ;;
    cvl/shared/*/*)
      rest="${path#cvl/shared/}"
      touch_family "${rest%%/*}" ;;
    # Read by Rust tests (existence), never rendered.
    cvl/README.md) rust=true ;;
    # The profile and assets reach every document, but no Rust test.
    cvl/profile.toml | cvl/assets/*) all_styles=true ;;
    # Any other cvl input could be a new data root: everything.
    cvl/*) select_everything "unclassified presentation input changed: $path" ;;
    # Read by Rust tests (README) and rendered into station-protocol CVs.
    interview/stations.toml)
      rust=true
      touch_station_plan ;;
    # Real-workspace inputs that Rust tests read, plus the Rust job's own
    # scripts and release-gate inputs.
    .agent/skills/* | .agent/scaffolds/* | .agent/schemas/* | \
      .agent/tests/fixtures/* | .agent/tests/skill-cases.json | \
      .agent/docs/editorial.md | interview/* | opportunities/* | \
      REUSE.toml | LICENSES/* | .crow/downstream-sync.yaml | \
      .agent/release-platforms.txt | .agent/scripts/ci-check.sh | \
      .agent/scripts/release-ci.sh | .agent/scripts/rust-toolchain.sh | \
      .agent/scripts/release-evidence.py | .agent/scripts/downstream-sync.sh)
      rust=true ;;
    # Prose that no Rust test or document reads. Links are checked by ccvl check.
    .agent/docs/* | .agent/AGENT.md | AGENTS.md | README.md | LICENSE.md | \
      todo.md | .github/*.md) ;;
    # Workflows, scripts and tests covered by the always-running lint job or
    # used only by explicit release, bootstrap or Crow routes.
    .github/workflows/binaries.yml | .github/workflows/downstream-sync.yml | \
      .github/workflows/skill-eval.yml | .agent/tests/test_* | \
      .agent/tests/*.ps1 | .agent/tests/review-evaluation/* | \
      .agent/scripts/bootstrap.sh | .agent/scripts/bootstrap.ps1 | \
      .agent/scripts/existing-tool-path.sh | .agent/scripts/runtime-id.sh | \
      .agent/scripts/runtime-id.ps1 | .agent/scripts/render-previews.sh | \
      .agent/scripts/package-release.sh | .agent/scripts/native-release.sh | \
      .agent/scripts/publish-release.sh | .agent/scripts/check-linux-deep.sh | \
      .agent/scripts/check-release-archive.sh | .agent/scripts/skill-eval-ci.sh | \
      .agent/scripts/downstream-check.py | .agent/scripts/candidate-date.py | \
      .agent/scripts/build-cluster-comparison.sh | .agent/scripts/tool-versions.sh | \
      .agent/scripts/tool-assets.csv | .agent/scripts/web-session-setup.sh | .ci/* | .crow/* | justfile | ccvl | \
      ccvl.cmd | ccvl.ps1) ;;
    *) select_everything "unclassified path: $path" ;;
  esac
}

classify() {
  local path
  while IFS= read -r path || [[ -n "$path" ]]; do
    [[ -n "$path" ]] || continue
    classify_path "$path"
  done
}

report() {
  local selected documents=false
  if [[ "$all_styles" == true ]]; then
    selected=all
  else
    selected="$(printf '%s' "$styles" | LC_ALL=C sort -u | tr '\n' ' ')"
    selected="${selected% }"
  fi
  [[ -z "$selected" ]] || documents=true
  printf 'rust=%s\ndocuments=%s\nstyles=%s\n' "$rust" "$documents" "$selected"
}

usable_commit() {
  [[ "$1" =~ ^[0-9a-f]{40}$ && ! "$1" =~ ^0+$ ]] &&
    git -C "$repo_root" cat-file -e "$1^{commit}" 2>/dev/null
}

# Resolve a local revision name; GitHub events supply exact commit IDs.
resolve_commit() {
  git -C "$repo_root" rev-parse --verify --quiet "$1^{commit}" || true
}

# Classify a revision range, or select everything when it cannot be resolved.
classify_range() {
  local base="$1" head="$2" separator="$3" paths
  if usable_commit "$base" && usable_commit "$head" &&
    paths="$(git -C "$repo_root" diff --no-renames --name-only "$base$separator$head" --)"; then
    classify <<<"$paths"
    reason="${reason:-$(printf '%s' "$paths" | grep -c . || true) changed paths in $base$separator$head}"
  else
    select_everything "unusable range ${base:-<none>}$separator${head:-<none>}"
  fi
}

github() {
  local event="${GITHUB_EVENT_NAME:-}" output result
  case "$event" in
    pull_request) classify_range "${PR_BASE_SHA:-}" "${PR_HEAD_SHA:-}" '...' ;;
    push) classify_range "${PUSH_BEFORE_SHA:-}" "${PUSH_AFTER_SHA:-}" '..' ;;
    *) select_everything "${event:-unknown} event checks everything" ;;
  esac
  result="$(report)"
  printf '%s\nreason=%s\n' "$result" "$reason"
  output="${GITHUB_OUTPUT:?GITHUB_OUTPUT is required in github mode}"
  printf '%s\n' "$result" >> "$output"
  if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
    printf '## Selected checks\n\n%s\n\n%s\n%s\n%s\n' "$reason" '```text' "$result" '```' >> "$GITHUB_STEP_SUMMARY"
  fi
}

case "${1:-}" in
  classify) classify; report ;;
  range)
    (($# == 3)) || { echo 'Usage: ci-changes.sh range BASE HEAD' >&2; exit 2; }
    classify_range "$(resolve_commit "$2")" "$(resolve_commit "$3")" '...'
    report ;;
  github) github ;;
  *) echo 'Usage: ci-changes.sh classify|range BASE HEAD|github' >&2; exit 2 ;;
esac
