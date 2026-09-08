#!/usr/bin/env bash
# Exercise cache invalidation and failed-render recovery without a PDF engine.
set -euo pipefail
repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-preview-test.XXXXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT
fixture="$scratch/workspace"
export PREVIEW_REAL_SHA256SUM
PREVIEW_REAL_SHA256SUM="$(command -v sha256sum)"
mkdir -p "$fixture/.agent/scripts" "$fixture/.agent/cache/ccvl/bin" \
  "$fixture/cvl/demo/pdf" "$scratch/bin"
cp "$repo_root/.agent/scripts/render-previews.sh" "$fixture/.agent/scripts/"
cp "$repo_root/.agent/scripts/runtime-id.sh" "$fixture/.agent/scripts/"
cat > "$fixture/.agent/cache/ccvl/bin/ccvl" <<'BINARY'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == list-documents ]]
cat "$PREVIEW_FIXTURE/documents.json"
BINARY
cat > "$scratch/bin/pdftoppm" <<'RENDERER'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == -v ]]; then
  [[ ! -f "$PREVIEW_FIXTURE/fail-version" ]] || exit 1
  cat "$PREVIEW_FIXTURE/version"
  exit
fi
page="$2"
pdf="${@: -2:1}"
out="${@: -1}"
if [[ -f "$PREVIEW_FIXTURE/fail-page" ]] && [[ "$(cat "$PREVIEW_FIXTURE/fail-page")" == "$page" ]]; then
  exit 1
fi
printf '%s %s\n' "$pdf" "$page" >> "$PREVIEW_FIXTURE/renders"
{ cat "$pdf"; printf '\n%s\n' "$*"; } > "$out.png"
RENDERER
cat > "$scratch/bin/sha256sum" <<'CHECKSUM'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == */pdf/first.pdf && -f "$PREVIEW_FIXTURE/fail-hash" ]]; then
  exit 1
fi
exec "$PREVIEW_REAL_SHA256SUM" "$@"
CHECKSUM
chmod +x "$fixture/.agent/cache/ccvl/bin/ccvl" "$scratch/bin/pdftoppm" "$scratch/bin/sha256sum"
export PREVIEW_FIXTURE="$scratch"
export PATH="$scratch/bin:$PATH"
printf 'Poppler fixture 1\n' > "$scratch/version"
printf 'First PDF\n' > "$fixture/cvl/demo/pdf/first.pdf"
printf 'Second PDF\n' > "$fixture/cvl/demo/pdf/second.pdf"
set_pages() {
  printf '[{"output":"cvl/demo/pdf/first.pdf","pages":%s},{"output":"cvl/demo/pdf/second.pdf","pages":1}]\n' "$1" > "$scratch/documents.json"
}
expect_counts() {
  local expected="$1" output
  output="$(bash "$fixture/.agent/scripts/render-previews.sh")"
  [[ "$output" == "$expected" ]] || {
    printf 'Unexpected preview result: %s\n' "$output" >&2
    exit 1
  }
}
set_pages 2
expect_counts 'Preview PDFs: 2 rendered, 0 reused.'
[[ "$(wc -l < "$scratch/renders")" == 3 ]]
expect_counts 'Preview PDFs: 0 rendered, 2 reused.'
[[ "$(wc -l < "$scratch/renders")" == 3 ]]

# Missing fingerprint inputs must abort even when existing previews would
# otherwise be reusable. Neither cache records nor output files may change.
cp -R "$fixture/cvl/demo/preview" "$scratch/probe-preview"
cp -R "$fixture/.agent/cache/previews" "$scratch/probe-cache"
for failure in version hash; do
  touch "$scratch/fail-$failure"
  if bash "$fixture/.agent/scripts/render-previews.sh" > "$scratch/probe.log" 2>&1; then
    printf 'Preview cache accepted a failed %s fingerprint input.\n' "$failure" >&2
    exit 1
  fi
  [[ "$(wc -l < "$scratch/renders")" == 3 ]]
  diff -r "$scratch/probe-preview" "$fixture/cvl/demo/preview"
  diff -r "$scratch/probe-cache" "$fixture/.agent/cache/previews"
  mv "$scratch/fail-$failure" "$scratch/failed-$failure"
done
expect_counts 'Preview PDFs: 0 rendered, 2 reused.'

printf 'Changed PDF\n' >> "$fixture/cvl/demo/pdf/first.pdf"
expect_counts 'Preview PDFs: 1 rendered, 1 reused.'
mv "$fixture/cvl/demo/preview/first-1.png" "$scratch/missing.png"
expect_counts 'Preview PDFs: 1 rendered, 1 reused.'
printf 'Damaged image\n' > "$fixture/cvl/demo/preview/first-1.png"
expect_counts 'Preview PDFs: 1 rendered, 1 reused.'
set_pages 1
printf 'Unrelated preview\n' > "$fixture/cvl/demo/preview/first-notes.png"
expect_counts 'Preview PDFs: 1 rendered, 1 reused.'
[[ ! -e "$fixture/cvl/demo/preview/first-2.png" ]]
[[ -f "$fixture/cvl/demo/preview/first-notes.png" ]]
[[ -f "$fixture/cvl/demo/preview/second-1.png" ]]

printf 'Poppler fixture 2\n' > "$scratch/version"
expect_counts 'Preview PDFs: 2 rendered, 0 reused.'
printf '\n# Changed renderer build.\n' >> "$scratch/bin/pdftoppm"
expect_counts 'Preview PDFs: 2 rendered, 0 reused.'
sed 's/-r 96/-r 97/' "$fixture/.agent/scripts/render-previews.sh" > "$scratch/updated.sh"
mv "$scratch/updated.sh" "$fixture/.agent/scripts/render-previews.sh"
expect_counts 'Preview PDFs: 2 rendered, 0 reused.'
expect_counts 'Preview PDFs: 0 rendered, 2 reused.'

# A failure on a later page must preserve all previously published previews
# and the cache entry, then retry that document on the next invocation.
set_pages 2
cp "$fixture/cvl/demo/preview/first-1.png" "$scratch/before.png"
cp -R "$fixture/.agent/cache/previews" "$scratch/before-cache"
printf 'Previous third page\n' > "$fixture/cvl/demo/preview/first-3.png"
cp "$fixture/cvl/demo/preview/first-3.png" "$scratch/before-surplus.png"
printf '2\n' > "$scratch/fail-page"
if bash "$fixture/.agent/scripts/render-previews.sh" > "$scratch/failure.log" 2>&1; then
  printf 'Preview rendering unexpectedly accepted a failed page.\n' >&2
  exit 1
fi
cmp -s "$scratch/before.png" "$fixture/cvl/demo/preview/first-1.png"
cmp -s "$scratch/before-surplus.png" "$fixture/cvl/demo/preview/first-3.png"
diff -r "$scratch/before-cache" "$fixture/.agent/cache/previews"
mv "$scratch/fail-page" "$scratch/failed-page"
expect_counts 'Preview PDFs: 1 rendered, 1 reused.'
[[ ! -e "$fixture/cvl/demo/preview/first-3.png" ]]
[[ -f "$fixture/cvl/demo/preview/first-notes.png" ]]
expect_counts 'Preview PDFs: 0 rendered, 2 reused.'
printf 'Preview cache reuse, input/output invalidation and failed-render recovery passed.\n'
