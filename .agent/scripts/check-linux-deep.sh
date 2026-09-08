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
"$binary" check
bash .agent/tests/test_bootstrap.sh
for shell_script in .agent/scripts/*.sh .agent/tests/*.sh ccvl; do
  bash -n "$shell_script"
done
"$binary" fmt --check

for filename in Archivo-Bold.ttf Archivo-Italic.ttf Archivo-Medium.ttf Archivo-Regular.ttf; do
  path="$repo_root/.agent/typst/fonts/$filename"
  if ! file --brief -- "$path" | grep -Eq 'TrueType|OpenType'; then
    printf 'Bundled font is missing, unresolved, or invalid: %s\n' "$path" >&2
    exit 1
  fi
done

profile_value() {
  local key="$1"
  local value
  value="$(sed -n \
    "s/^[[:space:]]*${key}[[:space:]]*=[[:space:]]*\"\([^\"]*\)\"[[:space:]]*$/\1/p" \
    cvl/profile.toml)"
  [[ -n "$value" ]] || {
    printf 'Could not read %s from cvl/profile.toml\n' "$key" >&2
    return 1
  }
  printf '%s\n' "$value"
}

public_name="$(profile_value name)"
public_email="$(profile_value email)"
public_phone="$(profile_value phone_label)"

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
    '^JavaScript:[[:space:]]+no$' \
    '^Page size:.*\(A4\)$'; do
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
  if ((text_size < 100)); then
    printf '%s has no usable text layer\n' "$pdf" >&2
    return 1
  fi
  for literal in "$public_name" "$public_email" "$public_phone"; do
    if ! grep -Fq "$literal" <<<"$extracted_text"; then
      printf '%s is missing machine-readable contact text: %s\n' "$pdf" "$literal" >&2
      return 1
    fi
  done

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
    NF && ($1 !~ /^[A-Z]+[+]Archivo-/ || $5 != "yes" || $6 != "yes" || $7 != "yes") { exit 1 }
  '; then
    printf '%s contains a fallback, unembedded, unsubstituted, or unmapped font\n' "$pdf" >&2
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

render_suite() {
  local destination="$1"
  local record _document substyle language region locale pages

  for record in cvl/cv/*/*/*/content.toml; do
    IFS=/ read -r _ _document substyle language region _ <<<"$record"
    locale="$language-$region"
    for pages in 2 3 4; do
      "$binary" build-cv "$locale" "$pages" --substyle "$substyle" \
        --application "$record" --profile cvl/profile.toml \
        --output "$destination/cv-$substyle-$locale-$pages.pdf" >/dev/null
    done
  done
  for record in cvl/cl/*/*/*/content.toml; do
    IFS=/ read -r _ _document substyle language region _ <<<"$record"
    locale="$language-$region"
    "$binary" build-cl "$locale" --substyle "$substyle" \
      --application "$record" --profile cvl/profile.toml \
      --output "$destination/cl-$substyle-$locale.pdf" >/dev/null
  done
}

first_build="$validation_dir/first"
second_build="$validation_dir/second"
mkdir -p -- "$first_build" "$second_build"
render_suite "$first_build"
render_suite "$second_build"

for record in cvl/cv/*/*/*/content.toml; do
  IFS=/ read -r _ _document substyle language region _ <<<"$record"
  locale="$language-$region"
  for pages in 2 3 4; do
    filename="cv-$substyle-$locale-$pages.pdf"
    pdf="$first_build/$filename"
    tracked="${record%/content.toml}/pdf/cv-$pages.pdf"
    check_pdf "$pdf" "$pages"
    check_pdf "$tracked" "$pages"
    cmp --silent "$pdf" "$second_build/$filename" || {
      printf 'CV build is not byte-reproducible: %s\n' "$filename" >&2
      exit 1
    }
    same_document "$pdf" "$tracked" || {
      printf 'Tracked CV output is stale: %s\n' "$tracked" >&2
      exit 1
    }
    pdftoppm -f 1 -l 2 -png -r 72 "$pdf" \
      "$validation_dir/pages-$substyle-$locale-$pages" >/dev/null 2>&1
  done
  for page in 1 2; do
    for pages in 3 4; do
      cmp --silent \
        "$validation_dir/pages-$substyle-$locale-2-$page.png" \
        "$validation_dir/pages-$substyle-$locale-$pages-$page.png" || {
        printf 'Shared CV page changed across presets: %s %s page %s (2 vs %s)\n' \
          "$substyle" "$locale" "$page" "$pages" >&2
        exit 1
      }
    done
  done
done

for record in cvl/cl/*/*/*/content.toml; do
  IFS=/ read -r _ _document substyle language region _ <<<"$record"
  locale="$language-$region"
  filename="cl-$substyle-$locale.pdf"
  pdf="$first_build/$filename"
  tracked="${record%/content.toml}/pdf/cl.pdf"
  check_pdf "$pdf" 1 yes
  check_pdf "$tracked" 1 yes
  cmp --silent "$pdf" "$second_build/$filename" || {
    printf 'Cover-letter build is not byte-reproducible: %s\n' "$filename" >&2
    exit 1
  }
  same_document "$pdf" "$tracked" || {
    printf 'Tracked cover-letter output is stale: %s\n' "$tracked" >&2
    exit 1
  }
done

printf 'Rust, data, font, PDF, reproducibility, CV, and cover-letter checks passed.\n'
