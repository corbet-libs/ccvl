#!/usr/bin/env bash
# Render the registered, already built PDFs for visual review and the gallery.
set -euo pipefail
export LC_ALL=C
repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"
binary="$repo_root/.agent/cache/ccvl/bin/ccvl"
manifest="$(mktemp "${TMPDIR:-/tmp}/ccvl-previews.XXXXXXXX")"
trap 'rm -f -- "$manifest"' EXIT
"$binary" list-documents > "$manifest"
while IFS=$'\t' read -r pdf pages; do
  leaf="${pdf%/pdf/*}"
  stem="${pdf##*/}"
  stem="${stem%.pdf}"
  mkdir -p -- "$leaf/preview"
  for ((page = 1; page <= pages; page++)); do
    pdftoppm -f "$page" -l "$page" -singlefile -png -r 96 \
      "$pdf" "$leaf/preview/$stem-$page" >/dev/null 2>&1
  done
done < <(jq -r '.[] | [.output,.pages] | @tsv' "$manifest")
printf 'Rendered all registered PDF pages into adjacent preview/ folders.\n'
