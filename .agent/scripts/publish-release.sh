#!/usr/bin/env bash
# Publish complete, source-bound native results; retries never replace asset bytes.
set +x
set -euo pipefail
repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"
dist="$(cd "${1:?Missing verified release directory}" && pwd)"
identity="$(python3 .agent/scripts/release-evidence.py check --output "$dist")"
read -r sha runtime_id <<<"$identity"
repository="${CI_REPO:-${GITHUB_REPOSITORY:-}}"
[[ "$repository" == corbet-labs/ccvl && "$sha" =~ ^[0-9a-f]{40}$ && "$runtime_id" =~ ^[0-9a-f]{64}$ ]] || exit 2
[[ -n ${GH_TOKEN:-${GITHUB_TOKEN:-}} ]] || { echo 'A publication credential is required.' >&2; exit 2; }
platform_policy="$(python3 .agent/scripts/release-evidence.py platforms)"
mapfile -t platforms <<<"$platform_policy"
runtime_files=()
bundle_files=()
evidence_files=("$dist/manifest.json" "$dist"/gate-*.json)
for platform in "${platforms[@]}"; do
  name="ccvl-$platform"
  if [[ "$platform" == windows-* ]]; then name="$name.exe"; fi
  runtime_files+=("$dist/$name" "$dist/$name.sha256" "$dist/$name.runtime-id")
  bundle_files+=("$dist/ccvl-$platform.tar.gz" "$dist/ccvl-$platform.tar.gz.sha256")
  evidence_files+=("$dist/ccvl-$platform.receipt.json")
done
is_current() {
  [[ $(gh api "repos/$repository/git/ref/heads/main" --jq '.object.sha') == "$sha" ]]
}
is_current || { printf 'Source %s is no longer main; publication refused.\n' "$sha" >&2; exit 2; }
scratch="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-publication.XXXXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT

api_object() {
  local endpoint="$1" destination="$2" status=0
  gh api --include "$endpoint" > "$scratch/response" 2> "$scratch/error" || status=$?
  # Only a definite 404 permits creation. Network/auth/rate-limit failures stop.
  if ((status != 0)); then
    if head -n 1 "$scratch/response" | grep -Eq '^HTTP/[^ ]+ 404 '; then return 4; fi
    cat "$scratch/error" >&2
    return 2
  fi
  sed '1,/^\r\{0,1\}$/d' "$scratch/response" > "$destination"
  jq -e 'type == "object"' "$destination" >/dev/null
}

release_state() {
  local tag="$1" status=0 release_id
  api_object "repos/$repository/releases/tags/$tag" "$scratch/release.json" || status=$?
  if [[ $status == 4 ]]; then
    # GitHub's tag endpoint can omit drafts. Resolve their database ID through
    # GraphQL before deciding that a release does not exist.
    # GraphQL variables must reach the API literally.
    # shellcheck disable=SC2016
    gh api graphql -f query='query($owner:String!,$name:String!,$tag:String!){repository(owner:$owner,name:$name){release(tagName:$tag){databaseId tagName}}}' \
      -f owner="${repository%/*}" -f name="${repository#*/}" -f tag="$tag" > "$scratch/lookup.json" || return 2
    jq -e '((.errors // []) | length) == 0 and (.data.repository | type == "object" and has("release"))' "$scratch/lookup.json" >/dev/null || return 2
    if jq -e '.data.repository.release == null' "$scratch/lookup.json" >/dev/null; then return 4; fi
    release_id="$(jq -er --arg tag "$tag" '.data.repository.release | select(.tagName == $tag) | .databaseId | select(type == "number" and . > 0 and . == floor)' "$scratch/lookup.json")" || return 2
    api_object "repos/$repository/releases/$release_id" "$scratch/release.json" || return 2
    jq -e --arg tag "$tag" --argjson id "$release_id" '.id == $id and .tag_name == $tag' "$scratch/release.json" >/dev/null || return 2
  elif [[ $status != 0 ]]; then
    return "$status"
  fi
  jq -er '.id and (.draft | type == "boolean")' "$scratch/release.json" >/dev/null
}

validate_tag() {
  local tag="$1" expected="$2" draft="$3" status=0 object_type object_sha depth
  [[ "$expected" =~ ^[0-9a-f]{40}$ ]] || { echo "Release $tag has no exact target commit." >&2; return 2; }
  api_object "repos/$repository/git/ref/tags/$tag" "$scratch/tag.json" || status=$?
  if [[ $status == 4 && "$draft" == true ]]; then return 0; fi
  [[ $status == 0 ]] || { echo "Release $tag has no verifiable immutable tag." >&2; return 2; }
  for ((depth = 0; depth < 8; depth++)); do
    object_type="$(jq -r '.object.type' "$scratch/tag.json")"
    object_sha="$(jq -r '.object.sha' "$scratch/tag.json")"
    [[ "$object_sha" =~ ^[0-9a-f]{40}$ ]] || return 2
    if [[ "$object_type" == commit ]]; then
      [[ "$object_sha" == "$expected" ]] || { echo "Existing tag $tag targets a different source commit." >&2; return 2; }
      return 0
    fi
    [[ "$object_type" == tag ]] || return 2
    api_object "repos/$repository/git/tags/$object_sha" "$scratch/tag.json" || return "$?"
  done
  echo "Cannot resolve the immutable tag $tag." >&2
  return 2
}

verify_existing() {
  local tag="$1"; shift
  local path name expected_hash downloaded actual expected
  expected="$(for path in "$@"; do basename "$path"; done | LC_ALL=C sort)"
  actual="$(jq -r '.assets[].name' "$scratch/release.json" | LC_ALL=C sort)"
  while IFS= read -r name; do
    [[ -z "$name" ]] && continue
    grep -Fxq "$name" <<<"$expected" || { echo "Unexpected immutable asset: $name" >&2; return 2; }
    path="$dist/$name"
    expected_hash="$(sha256sum "$path" | cut -d ' ' -f 1)"
    downloaded="$scratch/download-$name"
    # Download to a fresh path. Published/draft assets must match exact bytes.
    if [[ ! -e "$downloaded" ]]; then
      gh release download "$tag" --repo "$repository" --pattern "$name" --output "$downloaded"
    fi
    [[ $(sha256sum "$downloaded" | cut -d ' ' -f 1) == "$expected_hash" ]] || {
      echo "Existing release asset differs: $name. No overwrite attempted." >&2; return 2;
    }
  done <<<"$actual"
}

publish() {
  local tag="$1" title="$2" kind="$3" status=0 path name
  shift 3
  release_state "$tag" || status=$?
  if [[ $status == 4 ]]; then
    validate_tag "$tag" "$sha" true
    gh release create "$tag" --repo "$repository" --draft --target "$sha" --title "$title" \
      --notes "Verified runtime $runtime_id from source $sha. Released platforms: ${platforms[*]}. Native checks and all required release gates passed for this scope. Receipts and source/dependency identities are included."
    release_state "$tag"
  elif [[ $status != 0 ]]; then
    return "$status"
  fi
  if [[ "$kind" == workspace || $(jq -r '.draft' "$scratch/release.json") == true ]]; then
    [[ $(jq -r '.target_commitish' "$scratch/release.json") == "$sha" ]] || {
      echo "Release $tag targets a different source commit." >&2; return 2;
    }
  fi
  validate_tag "$tag" "$(jq -r '.target_commitish' "$scratch/release.json")" "$(jq -r '.draft' "$scratch/release.json")"
  verify_existing "$tag" "$@"
  for path in "$@"; do
    name="$(basename "$path")"
    if ! jq -e --arg name "$name" '.assets | any(.name == $name)' "$scratch/release.json" >/dev/null; then
      [[ $(jq -r '.draft' "$scratch/release.json") == true ]] || { echo "Published release $tag is incomplete." >&2; return 2; }
      gh release upload "$tag" --repo "$repository" "$path"
    fi
  done
  release_state "$tag"
  verify_existing "$tag" "$@"
  [[ $(jq -r '.assets | length' "$scratch/release.json") == "$#" ]] || return 2
  if [[ $(jq -r '.draft' "$scratch/release.json") == false ]]; then return 0; fi
  if [[ "$kind" == runtime ]]; then
    gh release edit "$tag" --repo "$repository" --draft=false --prerelease --latest=false
  elif is_current; then
    gh release edit "$tag" --repo "$repository" --draft=false --prerelease=false --latest
  else
    echo "Source advanced; leaving $tag as a draft." >&2
    return 2
  fi
  release_state "$tag"
  [[ $(jq -r '.draft' "$scratch/release.json") == false ]]
  validate_tag "$tag" "$(jq -r '.target_commitish' "$scratch/release.json")" false
}
publish "runtime-$runtime_id" "Compiled runtime ${runtime_id:0:12}" runtime "${runtime_files[@]}" "$dist/runtime-manifest.json"
publish "build-$sha" "ccvl ${sha:0:12}" workspace "${bundle_files[@]}" "${evidence_files[@]}"
printf 'Published %s native runtime(s) and matching workspace bundle(s) for %s: %s.\n' "${#platforms[@]}" "$sha" "${platforms[*]}"
