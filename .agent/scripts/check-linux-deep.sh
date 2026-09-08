#!/usr/bin/env bash
# Linux-only secondary checks using file, Poppler, and QPDF.
set -euo pipefail
export LC_ALL=C

repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
binary="$repo_root/.agent/cache/ccvl/bin/ccvl"
validation_dir="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-check.XXXXXXXX")"
trap 'rm -rf -- "$validation_dir"' EXIT

cd "$repo_root"
[[ -x "$binary" ]] || {
  printf 'The repository-local ccvl binary is missing. Run bash ./ccvl setup first.\n' >&2
  exit 2
}

"$binary" doctor >/dev/null
# Reuse only PDFs produced and fully verified by this invocation. The Rust
# check renders report/enforce pairs; a fresh process below supplies the third
# render, preserving cross-process reproducibility without another full suite.
first_build="$validation_dir/first"
"$binary" public-check --artifacts "$first_build"
bash .agent/tests/test_bootstrap.sh
bash .agent/tests/test_previews.sh
for shell_script in .agent/scripts/*.sh .agent/tests/*.sh ccvl; do
  bash -n "$shell_script"
done

for filename in Archivo-Bold.ttf Archivo-Italic.ttf Archivo-Medium.ttf Archivo-Regular.ttf; do
  path="$repo_root/.agent/typst/fonts/$filename"
  if ! file --brief -- "$path" | grep -Eq 'TrueType|OpenType'; then
    printf 'Bundled font is missing, unresolved, or invalid: %s\n' "$path" >&2
    exit 1
  fi
done

check_pdf() {
  local pdf="$1"
  local expected_pages="$2"
  local require_image="${3:-no}"
  local actual_pages
  local extracted_text
  local pdf_info
  local text_size

  if ! qpdf --check "$pdf" >/dev/null; then
    printf '%s failed independent PDF structure validation\n' "$pdf" >&2
    return 1
  fi

  pdf_info="$(pdfinfo "$pdf" 2>"$validation_dir/pdfinfo-errors.txt")"
  if [[ -s "$validation_dir/pdfinfo-errors.txt" ]]; then
    while IFS= read -r diagnostic; do
      if [[ "$diagnostic" != 'Syntax Error: Suspects object is wrong type (boolean)' ]]; then
        printf '%s emitted an unexpected pdfinfo diagnostic: %s\n' "$pdf" "$diagnostic" >&2
        return 1
      fi
    done < "$validation_dir/pdfinfo-errors.txt"
  fi

  actual_pages="$(awk '/^Pages:/ { print $2 }' <<<"$pdf_info")"
  if [[ "$actual_pages" != "$expected_pages" ]]; then
    printf '%s rendered %s pages; expected %s\n' "$pdf" "$actual_pages" "$expected_pages" >&2
    return 1
  fi

  for metadata_rule in \
    '^Encrypted:[[:space:]]+no$' \
    '^Form:[[:space:]]+none$' \
    '^JavaScript:[[:space:]]+no$'; do
    if ! grep -Eq "$metadata_rule" <<<"$pdf_info"; then
      printf '%s failed PDF metadata rule: %s\n' "$pdf" "$metadata_rule" >&2
      return 1
    fi
  done

  if ! pdfdetach -list "$pdf" | grep -Fxq '0 embedded files'; then
    printf '%s contains an unexpected embedded file\n' "$pdf" >&2
    return 1
  fi

  extracted_text="$(pdftotext "$pdf" -)"
  text_size="$(tr -d '[:space:]' <<<"$extracted_text" | wc -c)"
  if ((text_size < 1)); then
    printf '%s has no usable text layer\n' "$pdf" >&2
    return 1
  fi
  if ! pdffonts "$pdf" | tail -n +3 | awk 'NF && $5 != "yes" { exit 1 }'; then
    printf '%s contains a font that is not embedded\n' "$pdf" >&2
    return 1
  fi

  if [[ "$require_image" == yes ]] \
    && ! pdfimages -list "$pdf" | awk 'NR > 2 && $3 == "image" { found = 1 } END { exit !found }'; then
    printf '%s is missing its rendered signature image\n' "$pdf" >&2
    return 1
  fi

  if ! pdffonts "$pdf" | tail -n +3 | awk '
    NF && ($5 != "yes" || $7 != "yes") { exit 1 }
  '; then
    printf '%s contains a unembedded or unmapped font\n' "$pdf" >&2
    return 1
  fi
}

comparison_count=0
same_document() {
  local left="$1"
  local right="$2"
  local left_dir
  local right_dir
  local left_count
  local right_count
  local left_page
  local page_name

  ((comparison_count += 1))
  left_dir="$validation_dir/comparison-$comparison_count-left"
  right_dir="$validation_dir/comparison-$comparison_count-right"
  mkdir -p "$left_dir" "$right_dir"
  pdftoppm -png -r 144 "$left" "$left_dir/page" >/dev/null 2>&1
  pdftoppm -png -r 144 "$right" "$right_dir/page" >/dev/null 2>&1
  left_count="$(find "$left_dir" -type f -name 'page-*.png' | wc -l)"
  right_count="$(find "$right_dir" -type f -name 'page-*.png' | wc -l)"
  [[ "$left_count" == "$right_count" && "$left_count" -gt 0 ]] || return 1
  for left_page in "$left_dir"/page-*.png; do
    page_name="${left_page##*/}"
    cmp --silent "$left_page" "$right_dir/$page_name" || return 1
  done
  pdftotext "$left" "$left_dir/text.txt"
  pdftotext "$right" "$right_dir/text.txt"
  cmp --silent "$left_dir/text.txt" "$right_dir/text.txt"
}

# Enumerate actual style/substyle/locale/page variants from the registry.
"$binary" list-documents > "$validation_dir/documents.json"
jq -er 'length > 0' "$validation_dir/documents.json" >/dev/null
jq -r '.[] | [.document,.style,.substyle,.locale,.pages,.content,.output,
  (if .require_image then "yes" else "no" end)] | @tsv' \
  "$validation_dir/documents.json" > "$validation_dir/documents.tsv"

render_suite() {
  local destination="$1"
  local document style substyle locale pages record tracked require_image filename
  while IFS=$'\t' read -r document style substyle locale pages record tracked require_image; do
    filename="$document-$style-$substyle-$locale-$pages.pdf"
    if [[ "$document" == cv ]]; then
      "$binary" build-cv "$locale" "$pages" --style "$style" --substyle "$substyle" \
        --application "$record" --profile cvl/profile.toml \
        --output "$destination/$filename" >/dev/null
    else
      "$binary" build-cl "$locale" --pages "$pages" --style "$style" --substyle "$substyle" \
        --application "$record" --profile cvl/profile.toml \
        --output "$destination/$filename" >/dev/null
    fi
  done < "$validation_dir/documents.tsv"
}

second_build="$validation_dir/second"
mkdir -p -- "$second_build"
render_suite "$second_build"

while IFS=$'\t' read -r document style substyle locale pages record tracked require_image; do
  filename="$document-$style-$substyle-$locale-$pages.pdf"
  pdf="$first_build/$filename"
  check_pdf "$pdf" "$pages" "$require_image"
  check_pdf "$tracked" "$pages" "$require_image"
  cmp --silent "$pdf" "$second_build/$filename" || {
    printf 'Build is not byte-reproducible: %s\n' "$filename" >&2
    exit 1
  }
  same_document "$pdf" "$tracked" || {
    printf 'Tracked output is stale: %s\n' "$tracked" >&2
    exit 1
  }
done < "$validation_dir/documents.tsv"

# Only styles that declare shared pages require identical pages across presets.
jq -r 'group_by([.document,.style,.substyle,.locale])[] |
  . as $variants | .[0].shared_pages[] as $page |
  $variants[] | select(.pages >= $page) |
  [.document,.style,.substyle,.locale,.pages,$page] | @tsv' \
  "$validation_dir/documents.json" > "$validation_dir/shared.tsv"
while IFS=$'\t' read -r document style substyle locale pages page; do
  filename="$document-$style-$substyle-$locale-$pages.pdf"
  key="$document-$style-$substyle-$locale-$page"
  pdftoppm -f "$page" -l "$page" -singlefile -png -r 72 \
    "$first_build/$filename" "$validation_dir/current" >/dev/null 2>&1
  if [[ -f "$validation_dir/$key.png" ]]; then
    cmp --silent "$validation_dir/current.png" "$validation_dir/$key.png" || {
      printf 'Shared page changed across presets: %s (%s pages)\n' "$key" "$pages" >&2
      exit 1
    }
  else
    cp "$validation_dir/current.png" "$validation_dir/$key.png"
  fi
done < "$validation_dir/shared.tsv"

printf 'Rust, data, font, PDF, reproducibility, CV, and cover-letter checks passed.\n'
