#!/usr/bin/env bash
# Publish complete, immutable runtime assets and a workspace bundle per source revision.
set -euo pipefail
repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"
dist="$1"
probe() { command -v "$1" 2>/dev/null; }
# shellcheck source=.agent/scripts/runtime-id.sh
source "$repo_root/.agent/scripts/runtime-id.sh"
runtime_id="$(source_fingerprint)"
sha="$(git rev-parse HEAD)"
[[ "$sha" == "$GITHUB_SHA" ]]
platforms=(linux-x86_64 linux-aarch64 macos-x86_64 macos-arm64 windows-x86_64 windows-arm64)
runtime_files=()
bundle_files=()
for platform in "${platforms[@]}"; do
  name="ccvl-$platform"
  if [[ "$platform" == windows-* ]]; then name="$name.exe"; fi
  [[ "$(cat "$dist/$name.runtime-id")" == "$runtime_id" ]]
  (cd "$dist" && sha256sum --check "$name.sha256" "ccvl-$platform.tar.gz.sha256")
  runtime_files+=("$dist/$name" "$dist/$name.sha256" "$dist/$name.runtime-id")
  bundle_files+=("$dist/ccvl-$platform.tar.gz" "$dist/ccvl-$platform.tar.gz.sha256")
done

is_current() {
  [[ "$(gh api "repos/$GITHUB_REPOSITORY/git/ref/heads/main" --jq '.object.sha')" == "$sha" ]]
}
if ! is_current; then
  printf 'Revision %s was superseded; the newer CI run owns publication.\n' "$sha"
  exit 0
fi

jq -n --arg source_commit "$sha" --arg runtime_id "$runtime_id" \
  --argjson platforms '["linux-x86_64","linux-aarch64","macos-x86_64","macos-arm64","windows-x86_64","windows-arm64"]' \
  '{source_commit: $source_commit, runtime_id: $runtime_id, platforms: $platforms}' > "$dist/manifest.json"

publish() {
  local tag="$1" title="$2" kind="$3"
  shift 3
  local release_state expected actual
  if release_state="$(gh release view "$tag" --json isDraft --jq '.isDraft' 2>/dev/null)"; then
    if [[ "$release_state" == false ]]; then
      # Published assets are immutable. A repeated run must find the complete set.
      expected="$(for path in "$@"; do basename "$path"; done | LC_ALL=C sort)"
      actual="$(gh release view "$tag" --json assets --jq '.assets[].name' | LC_ALL=C sort)"
      [[ "$actual" == "$expected" ]] || { printf 'Published release %s is incomplete.\n' "$tag" >&2; return 1; }
      return 0
    fi
  else
    gh release create "$tag" --draft --target "$sha" --title "$title" \
      --notes "Verified runtime $runtime_id from source $sha. All six platforms passed native checks. Download your platform workspace archive, extract it, and run the included ccvl launcher."
  fi
  gh release upload "$tag" --clobber "$@"
  expected="$(for path in "$@"; do basename "$path"; done | LC_ALL=C sort)"
  actual="$(gh release view "$tag" --json assets --jq '.assets[].name' | LC_ALL=C sort)"
  [[ "$actual" == "$expected" ]]
  if [[ "$kind" == runtime ]]; then
    gh release edit "$tag" --draft=false --prerelease --latest=false
  elif is_current; then
    gh release edit "$tag" --draft=false --prerelease=false --latest
  else
    printf 'Source advanced during upload; leaving %s as a draft.\n' "$tag"
  fi
}

publish "runtime-$runtime_id" "Compiled runtime ${runtime_id:0:12}" runtime \
  "${runtime_files[@]}" "$dist/manifest.json"
publish "build-$sha" "ccvl ${sha:0:12}" workspace \
  "${bundle_files[@]}" "$dist/manifest.json"
printf 'Published six native runtime assets and six matching workspace bundles for %s.\n' "$sha"
