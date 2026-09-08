#!/usr/bin/env bash
# Render changed registered PDFs for visual review and the gallery.
set -euo pipefail
export LC_ALL=C
repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"
binary="$repo_root/.agent/cache/ccvl/bin/ccvl"
cache="$repo_root/.agent/cache/previews"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-previews.XXXXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT
probe() { command -v "$1" 2>/dev/null; }
# shellcheck source=.agent/scripts/runtime-id.sh
source "$repo_root/.agent/scripts/runtime-id.sh"

# Hash the rasterizer executable and reported version as well as this script:
# changing the rendering options or installed Poppler invalidates every entry.
renderer="$(command -v pdftoppm)"
# Keep required inputs in separate assignments: an earlier failure inside a
# grouped command substitution can otherwise be hidden by its final command.
renderer_digest="$(hash_file "$renderer")"
renderer_version="$("$renderer" -v 2>&1)"
script_digest="$(hash_file "$repo_root/.agent/scripts/render-previews.sh")"
renderer_key="$(printf '%s\n' "$renderer_digest" "$renderer_version" "$script_digest" | hash_stream)"
"$binary" list-documents > "$scratch/documents.json"
jq -r '.[] | [.output,.pages] | @tsv' "$scratch/documents.json" > "$scratch/documents.tsv"
mkdir -p -- "$cache"
rendered=0
reused=0

preview_state() {
  local page
  printf '%s\n' "$input_key"
  for ((page = 1; page <= pages; page++)); do
    if [[ -f "$leaf/preview/$stem-$page.png" ]]; then
      hash_file "$leaf/preview/$stem-$page.png"
    else
      printf 'missing\n'
    fi
  done
}

while IFS=$'\t' read -r pdf pages; do
  leaf="${pdf%/pdf/*}"
  stem="${pdf##*/}"
  stem="${stem%.pdf}"
  entry="$cache/$(printf '%s\n' "$pdf" | hash_stream)"
  pdf_digest="$(hash_file "$pdf")"
  input_key="$(printf '%s\n' "$pdf_digest" "$pages" "$renderer_key" | hash_stream)"
  preview_state > "$scratch/state"
  if [[ -f "$entry" ]] && cmp -s "$scratch/state" "$entry"; then
    ((reused += 1))
    continue
  fi

  # Complete all pages before replacing outputs or recording a cache hit.
  for ((page = 1; page <= pages; page++)); do
    "$renderer" -f "$page" -l "$page" -singlefile -png -r 96 \
      "$pdf" "$scratch/$stem-$page" >/dev/null 2>&1
  done
  mkdir -p -- "$leaf/preview"
  for ((page = 1; page <= pages; page++)); do
    mv -- "$scratch/$stem-$page.png" "$leaf/preview/$stem-$page.png"
  done
  # A shorter PDF must not leave obsolete pages in its preview set. Remove
  # only this PDF's positive, numbered page files after the full render succeeds.
  for preview in "$leaf/preview/$stem-"*.png; do
    [[ -f "$preview" ]] || continue
    number="${preview#"$leaf/preview/$stem-"}"
    number="${number%.png}"
    [[ "$number" =~ ^[1-9][0-9]*$ ]] || continue
    if ((${#number} > ${#pages})) || {
      ((${#number} == ${#pages})) && [[ "$number" > "$pages" ]]
    }; then
      rm -- "$preview"
    fi
  done
  preview_state > "$scratch/state"
  mv -- "$scratch/state" "$entry"
  ((rendered += 1))
done < "$scratch/documents.tsv"
printf 'Preview PDFs: %s rendered, %s reused.\n' "$rendered" "$reused"
