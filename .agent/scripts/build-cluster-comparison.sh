#!/usr/bin/env bash
# Assemble D+ with the chronological opening retained as comparison page 5.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
for tool in qpdf pdftoppm; do
  command -v "$tool" >/dev/null || { echo "$tool is required for the comparison gallery." >&2; exit 2; }
done
locales=("$@")
if ((${#locales[@]} == 0)); then locales=(de-ch en-ch); fi
for locale in "${locales[@]}"; do
  case "$locale" in de-ch|en-ch) ;; *) echo "Unsupported locale: $locale" >&2; exit 2 ;; esac
done
for locale in "${locales[@]}"; do
  language="${locale%-ch}"
  leaf="cvl/cv/cluster/d-plus/$language/ch"
  opening="$leaf/pdf/cv-1.pdf"
  reference="cvl/cv/harvard/d-plus/$language/ch/pdf/cv-4.pdf"
  comparison="$leaf/pdf/comparison-5.pdf"
  bash ./ccvl build-cv "$locale" 1 --style cluster --substyle d-plus
  bash ./ccvl build-cv "$locale" 4 --style harvard --substyle d-plus
  [[ $(qpdf --show-npages "$opening") == 1 ]]
  [[ $(qpdf --show-npages "$reference") == 4 ]]
  qpdf --deterministic-id --empty --pages "$opening" 1 "$reference" 2-4 "$reference" 1 -- "$comparison"
  [[ $(qpdf --show-npages "$comparison") == 5 ]]
  qpdf --check "$comparison"
  mkdir -p "$leaf/preview"
  pdftoppm -scale-to 1600 -png "$comparison" "$leaf/preview/comparison-5"
  printf 'D+ comparison: %s\n' "$comparison"
done
