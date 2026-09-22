#!/usr/bin/env bash
set -euo pipefail

repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/ccvl-skill-eval-test.XXXXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT
bash_command="$(command -v bash)"
mkdir -p "$scratch/repo/.agent/scripts" "$scratch/repo/.agent/tests" "$scratch/bin" "$scratch/target/debug"
cp "$repo_root/.agent/scripts/skill-eval-ci.sh" "$repo_root/.agent/scripts/rust-toolchain.sh" "$scratch/repo/.agent/scripts/"
cp "$repo_root/Cargo.toml" "$scratch/repo/Cargo.toml"
printf 'fixture lock\n' > "$scratch/repo/Cargo.lock"
printf '{"cases":[]}\n' > "$scratch/repo/.agent/tests/skill-cases.json"
for utility in dirname sed cp mkdir sha256sum cat mktemp timeout; do
  ln -s "$(command -v "$utility")" "$scratch/bin/$utility"
done
cat > "$scratch/bin/rustc" <<'EOF'
#!/bin/sh
printf 'rustc 1.97.0 (fixture)\n'
EOF
cat > "$scratch/bin/cargo" <<'EOF'
#!/bin/sh
test -z "${GROQ_API_KEY:-}" || exit 90
printf '%s\n' "$*" >> "$FAKE_CARGO_LOG"
case "$*" in
  '--version') printf 'cargo 1.97.0 (fixture)\n' ;;
  'build --locked --bin ccvl') cp "$FAKE_BINARY" "$CARGO_TARGET_DIR/debug/ccvl" ;;
  *) exit 91 ;;
esac
EOF
cat > "$scratch/fake-ccvl" <<'EOF'
#!/bin/sh
if test "$1" = runtime-id; then printf 'fixture-runtime\n'; exit 0; fi
test "$1" = skill-eval || exit 92
printf '%s\n' "$*" > "$FAKE_ARGUMENTS"
while test "$#" -gt 0; do
  case "$1" in
    --output) shift; output="$1" ;;
    --summary) shift; summary="$1" ;;
  esac
  shift
done
status="${FAKE_EVAL_STATUS:-0}"
if test -z "${GROQ_API_KEY:-}"; then status=2; fi
if test "${FAKE_SKIP_REPORT:-0}" = 1; then exit 0; fi
printf '{"fixture_exit":%s}\n' "$status" > "$output"
printf 'fixture summary\n' > "$summary"
exit "$status"
EOF
chmod +x "$scratch/bin/rustc" "$scratch/bin/cargo" "$scratch/fake-ccvl"
export FAKE_BINARY="$scratch/fake-ccvl" FAKE_CARGO_LOG="$scratch/cargo.log" FAKE_ARGUMENTS="$scratch/arguments"
export CARGO_TARGET_DIR="$scratch/target" CCID_TARGET_LOCK_HELD="$scratch/target"
export CI_REPO=corbet-libs/ccvl CI_COMMIT_BRANCH=main CI_REPO_DEFAULT_BRANCH=main
export CI_COMMIT_SHA=0123456789012345678901234567890123456789
export SOURCE_SHA256=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
unset RUST_TOOLCHAIN RUSTUP_TOOLCHAIN GROQ_API_KEY

run() {
  PATH="$scratch/bin" "$bash_command" "$scratch/repo/.agent/scripts/skill-eval-ci.sh" "$1" > "$scratch/output" 2>&1
}
expect_status() {
  local expected="$1" action="$2" actual=0
  run "$action" || actual=$?
  if [[ $actual != "$expected" ]]; then
    cat "$scratch/output" >&2
    printf 'Expected status %s, got %s for %s\n' "$expected" "$actual" "$action" >&2
    exit 1
  fi
}

# Neither private source nor an unlocked target can build or evaluate.
CI_REPO=corbet-labs/applications expect_status 2 build
SOURCE_SHA256=missing expect_status 2 build
CCID_TARGET_LOCK_HELD=wrong expect_status 2 build
GROQ_API_KEY=fixture-secret expect_status 2 build
[[ ! -f "$FAKE_CARGO_LOG" ]]
expect_status 2 evaluate
expect_status 0 build
grep -Fxq 'build --locked --bin ccvl' "$FAKE_CARGO_LOG"
cp "$FAKE_CARGO_LOG" "$scratch/cargo-before-evaluation.log"

# Missing credentials and provider failures remain failures with retained reports.
expect_status 2 evaluate
grep -Fq 'ccvl_groq_api_key is required' "$scratch/output"
grep -Fq '"fixture_exit":2' "$scratch/output"
export GROQ_API_KEY=fixture-secret
FAKE_EVAL_STATUS=75 expect_status 75 evaluate
grep -Fq '"fixture_exit":75' "$scratch/output"
FAKE_EVAL_STATUS=1 expect_status 1 evaluate
FAKE_SKIP_REPORT=1 expect_status 2 evaluate
grep -Fq 'Successful evaluation requires a nonempty report.json and summary.md.' "$scratch/output"
expect_status 0 evaluate
grep -Fq -- '--cases .agent/tests/skill-cases.json --skills-root .agent/skills --model openai/gpt-oss-20b' "$FAKE_ARGUMENTS"
grep -Fq "source_commit=$CI_COMMIT_SHA" "$scratch/output"
if grep -Fq "$GROQ_API_KEY" "$scratch/output"; then echo 'Credential appeared in output.' >&2; exit 1; fi
cmp "$FAKE_CARGO_LOG" "$scratch/cargo-before-evaluation.log"

# A different build between the two Crow steps fails closed before provider work.
printf '\n# replaced binary\n' >> "$CARGO_TARGET_DIR/debug/ccvl"
expect_status 1 evaluate
printf 'Skill evaluation keeps credentials out of builds, enforces source/target identity, and preserves failed reports.\n'
