#!/usr/bin/env bash
# Execute the real downloaded bundle in an already provided toolchain-free Linux executor.
set -euo pipefail
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'Archive verification requires Linux x86_64.' >&2; exit 2; }
for tool in cargo rustc git; do
  if command -v "$tool" >/dev/null; then
    echo 'The archive gate requires an executor without Git or Rust; masking PATH is not permitted.' >&2
    exit 2
  fi
done
dist="$(cd "${1:?Missing native package directory}" && pwd)"
receipt="$dist/ccvl-linux-x86_64.receipt.json"
(cd "$dist" && sha256sum --check --strict ccvl-linux-x86_64.tar.gz.sha256)
# Receipt fields are generated as one JSON string per line. Accept only the
# fixed public identity and hexadecimal values; this minimal executor needs no JSON tool.
field() { sed -n 's/^  "'"$1"'": "\([^"]*\)"[,]*$/\1/p' "$receipt"; }
sha="$(field source_commit)"
[[ "$sha" =~ ^[0-9a-f]{40}$ && "$sha" == "${CI_COMMIT_SHA:-${GITHUB_SHA:-}}" ]] || exit 2
[[ $(field repository) == corbet-libs/ccvl && $(field platform) == linux-x86_64 && $(field rust_host) == x86_64-unknown-linux-gnu ]] || exit 2
source_hash="$(field source_sha256)"
archive_hash="$(field source_archive_sha256)"
if [[ -n ${SOURCE_SHA256:-} && "$archive_hash" != "$SOURCE_SHA256" ]]; then
  echo 'Archive gate source archive differs from its native package.' >&2; exit 2
fi
lock_hash="$(field cargo_lock_sha256)"
runtime_id="$(field runtime_id)"
for value in "$source_hash" "$archive_hash" "$lock_hash" "$runtime_id"; do
  [[ "$value" =~ ^[0-9a-f]{64}$ ]] || exit 2
done
scratch="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-archive.XXXXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT
tar -xzf "$dist/ccvl-linux-x86_64.tar.gz" -C "$scratch"
cd "$scratch"
[[ ! -e .git ]]
[[ $(sha256sum Cargo.lock | cut -d ' ' -f 1) == "$lock_hash" ]]
[[ $(.agent/cache/ccvl/bin/ccvl runtime-id) == "$runtime_id" ]]
bash ./ccvl setup
provider=gha
[[ -z ${CI_REPO:-} ]] || provider=crow
run="${CI_PIPELINE_NUMBER:-${CI_BUILD_NUMBER:-${GITHUB_RUN_ID:-}}}"
[[ "$run" =~ ^[0-9]+$ ]] || { echo 'Missing CI run identity.' >&2; exit 2; }
native_hash="$(sha256sum "$receipt" | cut -d ' ' -f 1)"
cat > "$scratch/gate-archive.json" <<EOF
{
  "schema": 1,
  "repository": "corbet-libs/ccvl",
  "source_commit": "$sha",
  "source_sha256": "$source_hash",
  "source_archive_sha256": "$archive_hash",
  "cargo_lock_sha256": "$lock_hash",
  "runtime_id": "$runtime_id",
  "kind": "gate",
  "gate": "archive",
  "provider": "$provider",
  "run": "$run",
  "native_receipt_sha256": "$native_hash"
}
EOF
if [[ -e "$dist/gate-archive.json" ]]; then
  cmp "$scratch/gate-archive.json" "$dist/gate-archive.json"
else
  cp "$scratch/gate-archive.json" "$dist/gate-archive.json"
fi
