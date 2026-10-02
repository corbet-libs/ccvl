#!/usr/bin/env bash
# Behaviour of the CI change selector on a fixture workspace and on Git ranges.
set -euo pipefail

repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-ci-changes.XXXXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT
fixture="$scratch/repo"
selector="$fixture/.agent/scripts/ci-changes.sh"

write() {
  mkdir -p "$(dirname -- "$fixture/$1")"
  printf '%s\n' "$2" > "$fixture/$1"
}

mkdir -p "$fixture/.agent/scripts"
cp "$repo_root/.agent/scripts/ci-changes.sh" "$selector"
write cvl/cv/harvard/style.toml 'defaults = "../../shared/harvard/defaults.toml"'
write cvl/cv/harvard/standard/en/ch/content.toml '[cv]'
write cvl/cv/harvard/contract.toml $'presets = [2, 3, 4]\n\n[layout_contract.page_1.entries]\nminimum = 6'
write cvl/cv/modern/contract.toml 'content_fields = []'
write cvl/cl/harvard/style.toml 'defaults = "../../shared/harvard/defaults.toml"'
write cvl/cv/cluster/style.toml 'defaults = "../../shared/cluster/defaults.toml"'
write cvl/cv/cluster/layout.typ '#import "/cvl/shared/harvard/style.typ": document-style'
write cvl/cv/modern/style.toml 'defaults = "../../shared/modern/defaults.toml"'
write cvl/cv/slot-4/style.toml 'empty = true'
write cvl/shared/harvard/defaults.toml '[page]'
write cvl/shared/modern/defaults.toml '[page]'
write cvl/shared/orphan/defaults.toml '[page]'
write README.md '# Fixture'

fail() {
  printf 'ci-changes: %s\n' "$*" >&2
  exit 1
}

# expect RUST DOCUMENTS STYLES PATH... : classify PATHs from stdin.
expect() {
  local rust="$1" documents="$2" styles="$3" actual expected
  shift 3
  actual="$(printf '%s\n' "$@" | bash "$selector" classify)"
  expected="$(printf 'rust=%s\ndocuments=%s\nstyles=%s' "$rust" "$documents" "$styles")"
  [[ "$actual" == "$expected" ]] || fail "unexpected selection for $*: $actual"
}

# Style-only changes check only the touched style, without the Rust job:
# Rust tests use synthetic fixtures and check enforces the real-data rules.
expect false true cv/modern cvl/cv/modern/standard/en/ch/layout.toml
expect false true cv/modern cvl/cv/modern/style.toml cvl/cv/modern/timeline/de/ch/content.toml
expect false true 'cl/harvard cv/modern' cvl/cv/modern/README.md cvl/cl/harvard/frame/substyle.toml
expect false true cv/harvard cvl/cv/harvard/standard/en/ch/content.toml cvl/cv/harvard/contract.toml
# A lone empty slot style is a valid selection that renders no documents.
expect false true cv/slot-4 cvl/cv/slot-4/style.toml
# Shared families serve their same-named and referencing styles.
expect false true 'cl/harvard cv/cluster cv/harvard' cvl/shared/harvard/style.typ
expect false true cv/modern cvl/shared/modern/defaults.toml
# An unreferenced family or a removed style changes the inventory: everything.
expect true true all cvl/shared/orphan/defaults.toml
expect true true all cvl/cv/removed/standard/en/ch/content.toml
# The profile and assets reach every document but no Rust test.
expect false true all cvl/profile.toml
expect false true all cvl/assets/signature.png
# Other cvl inputs could be new data roots; the README is read by Rust tests.
expect true true all cvl/cv/README.md
expect true false '' cvl/README.md
# Cloud session setup is shellchecked by the lint job only.
expect false false '' .agent/scripts/web-session-setup.sh
# The station plan feeds the CVs declaring the station layout protocol.
expect true true cv/harvard interview/stations.toml
expect true true 'cl/harvard cv/harvard' interview/stations.toml cvl/cl/harvard/left-rule/en/ch/strings.toml
mv "$fixture/cvl/cv/harvard/contract.toml" "$scratch/contract.toml"
expect true false '' interview/stations.toml
mv "$scratch/contract.toml" "$fixture/cvl/cv/harvard/contract.toml"
# Engine and manifest changes check every document; so do selection changes.
for path in .agent/src/check.rs .agent/core/src/lib.rs .agent/build.rs \
  .agent/typst/document.typ Cargo.toml Cargo.lock rust-toolchain.toml ccvl.json \
  .github/workflows/ci.yml .agent/scripts/ci-changes.sh; do
  expect true true all "$path"
done
# Real-workspace inputs that Rust tests read, and the Rust job's scripts.
for path in .agent/skills/ccvl-cv/SKILL.md .agent/scaffolds/opportunity/application.toml \
  .agent/schemas/review-result.schema.json .agent/tests/fixtures/application-dates.json \
  .agent/tests/skill-cases.json .agent/docs/editorial.md interview/README.md \
  opportunities/README.md REUSE.toml LICENSES/CC-BY-4.0.txt .crow/downstream-sync.yaml \
  .agent/release-platforms.txt .agent/scripts/ci-check.sh .agent/scripts/release-ci.sh \
  .agent/scripts/rust-toolchain.sh .agent/scripts/release-evidence.py \
  .agent/scripts/downstream-sync.sh; do
  expect true false '' "$path"
done
# Prose, lint-covered scripts and explicit release routes run lint only.
for path in .agent/docs/ci.md .agent/AGENT.md AGENTS.md README.md LICENSE.md todo.md \
  .github/SECURITY.md .github/workflows/binaries.yml .github/workflows/skill-eval.yml \
  .agent/tests/test_ci_changes.sh .agent/tests/test_runtime.ps1 \
  .agent/tests/review-evaluation/cases.json .agent/scripts/bootstrap.sh \
  .agent/scripts/runtime-id.sh .agent/scripts/native-release.sh .ci/ccid.toml \
  .crow/ccid.yaml justfile ccvl ccvl.cmd ccvl.ps1; do
  expect false false '' "$path"
done
expect false false ''
expect false false '' .agent/docs/ci.md README.md
# Anything unclassified fails safe.
expect true true all new-root/file.txt
expect true true all .gitignore
expect true true all cvl/cv/modern/style.toml .agent/docs/ci.md new-root/file.txt

# Git ranges and GitHub events.
git_fixture() { git -C "$fixture" -c user.name=ccvl -c user.email=ccvl@example.invalid -c commit.gpgsign=false "$@"; }
git_fixture init -q -b main
git_fixture add -A
git_fixture commit -q -m base
base="$(git_fixture rev-parse HEAD)"
git_fixture checkout -q -b feature
write cvl/cv/modern/standard/en/ch/layout.toml '[page]'
write .agent/docs/guide.md '# Guide'
git_fixture add -A
git_fixture commit -q -m feature
feature="$(git_fixture rev-parse HEAD)"
git_fixture checkout -q main
write .agent/src/lib.rs '// moved on main'
git_fixture add -A
git_fixture commit -q -m main
main="$(git_fixture rev-parse HEAD)"
git_fixture checkout -q "$feature"

range="$(bash "$selector" range "$main" "$feature")"
[[ "$range" == $'rust=false\ndocuments=true\nstyles=cv/modern' ]] ||
  fail "a pull-request range must use the merge base: $range"
range="$(bash "$selector" range main HEAD)"
[[ "$range" == $'rust=false\ndocuments=true\nstyles=cv/modern' ]] ||
  fail "local revision names must resolve: $range"
range="$(bash "$selector" range missing-revision HEAD)"
[[ "$range" == $'rust=true\ndocuments=true\nstyles=all' ]] ||
  fail "an unknown revision must select everything: $range"

github() {
  : > "$scratch/output"
  env -i PATH="$PATH" HOME="$scratch" GITHUB_OUTPUT="$scratch/output" "$@" \
    bash "$selector" github > "$scratch/log"
  sed -n 's/^reason=//p' "$scratch/log" > "$scratch/reason"
}
expect_output() {
  [[ "$(cat "$scratch/output")" == "$1" ]] || fail "unexpected GitHub output: $(cat "$scratch/output")"
}

github GITHUB_EVENT_NAME=pull_request PR_BASE_SHA="$main" PR_HEAD_SHA="$feature"
expect_output $'rust=false\ndocuments=true\nstyles=cv/modern'
grep -Fq '2 changed paths' "$scratch/reason" || fail "missing range reason: $(cat "$scratch/reason")"
github GITHUB_EVENT_NAME=push PUSH_BEFORE_SHA="$base" PUSH_AFTER_SHA="$feature"
expect_output $'rust=false\ndocuments=true\nstyles=cv/modern'
github GITHUB_EVENT_NAME=push PUSH_BEFORE_SHA="$main" PUSH_AFTER_SHA="$main"
expect_output $'rust=false\ndocuments=false\nstyles='
# Unusable ranges and every other event select everything.
for before in 0000000000000000000000000000000000000000 '' "$(printf 'f%.0s' {1..40})" HEAD; do
  github GITHUB_EVENT_NAME=push PUSH_BEFORE_SHA="$before" PUSH_AFTER_SHA="$feature"
  expect_output $'rust=true\ndocuments=true\nstyles=all'
  grep -Fq 'unusable range' "$scratch/reason" || fail "missing fallback reason for before=$before"
done
github GITHUB_EVENT_NAME=pull_request PR_BASE_SHA= PR_HEAD_SHA="$feature"
expect_output $'rust=true\ndocuments=true\nstyles=all'
for event in workflow_dispatch schedule merge_group ''; do
  github GITHUB_EVENT_NAME="$event"
  expect_output $'rust=true\ndocuments=true\nstyles=all'
done
github GITHUB_EVENT_NAME=pull_request PR_BASE_SHA="$main" PR_HEAD_SHA="$feature" \
  GITHUB_STEP_SUMMARY="$scratch/summary"
grep -Fq 'styles=cv/modern' "$scratch/summary" || fail 'missing step summary'
if env -i PATH="$PATH" GITHUB_EVENT_NAME=workflow_dispatch bash "$selector" github >/dev/null 2>&1; then
  fail 'github mode must require GITHUB_OUTPUT'
fi
if bash "$selector" unknown >/dev/null 2>&1; then fail 'unknown mode accepted'; fi

# Every tracked path of the real workspace classifies; all of them together
# select every check.
all="$(git -C "$repo_root" ls-files | bash "$repo_root/.agent/scripts/ci-changes.sh" classify)"
[[ "$all" == $'rust=true\ndocuments=true\nstyles=all' ]] || fail "full workspace selection: $all"
echo 'CI change selection tests passed.'
