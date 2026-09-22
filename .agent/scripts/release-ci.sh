#!/usr/bin/env bash
# Release stages on existing workers; no installation, cross-build, or scheduler.
set -euo pipefail
repo_root="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"
action="${1:?Usage: release-ci.sh rust|lint|native|linux-deep|archive|check|publish}"
if [[ -n ${CI_REPO:-} ]]; then
  [[ "$CI_REPO" == corbet-libs/ccvl && ${CI_COMMIT_BRANCH:-} == main ]] || {
    echo 'Crow release stages require public ccvl main.' >&2; exit 2;
  }
  [[ -n ${CARGO_TARGET_DIR:-} && ${CCID_TARGET_LOCK_HELD:-} == "$CARGO_TARGET_DIR" ]] || {
    echo 'Crow release stages require the verified ccid target lock.' >&2; exit 2;
  }
fi
if [[ "$action" != publish && -n ${GH_TOKEN:-}${GITHUB_TOKEN:-} ]]; then
  echo 'Publication credentials are allowed only in the publish stage.' >&2; exit 2
fi
if [[ -z ${CCVL_RELEASE_DIR:-} ]]; then
  : "${CARGO_TARGET_DIR:?Set CCVL_RELEASE_DIR or the locked Cargo target}"
  : "${CI_COMMIT_SHA:?Missing source commit}"
  run="${CI_PIPELINE_NUMBER:-${CI_BUILD_NUMBER:-}}"
  : "${run:?Missing Crow run number}"
  CCVL_RELEASE_DIR="$CARGO_TARGET_DIR/ccvl-release/$CI_COMMIT_SHA/$run"
fi
mkdir -p "$CCVL_RELEASE_DIR"
CCVL_RELEASE_DIR="$(cd "$CCVL_RELEASE_DIR" && pwd)"
export CCVL_RELEASE_DIR
printf 'Release evidence directory: %s\n' "$CCVL_RELEASE_DIR"
case "$action" in
  rust|lint)
    bash .agent/scripts/ci-check.sh "$action"
    python3 .agent/scripts/release-evidence.py gate "$action" --output "$CCVL_RELEASE_DIR"
    ;;
  native)
    bash .agent/scripts/native-release.sh "${CCVL_RELEASE_PLATFORM:?Select an actual native platform}" "$CCVL_RELEASE_DIR"
    ;;
  linux-deep)
    [[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'Linux x86_64 verification worker required.' >&2; exit 2; }
    python3 .agent/scripts/release-evidence.py source >/dev/null
    (cd "$CCVL_RELEASE_DIR" && sha256sum --check --strict ccvl-linux-x86_64.sha256)
    mkdir -p .agent/cache/ccvl/bin
    cp "$CCVL_RELEASE_DIR/ccvl-linux-x86_64" .agent/cache/ccvl/bin/ccvl
    chmod 0755 .agent/cache/ccvl/bin/ccvl
    bash .agent/scripts/check-linux-deep.sh
    python3 .agent/scripts/release-evidence.py gate linux-deep --output "$CCVL_RELEASE_DIR"
    ;;
  archive)
    # This stage must run inside an already provided minimal executor. Do not
    # hide tools with PATH: absence is an executor property, not a mock test.
    bash .agent/scripts/check-release-archive.sh "$CCVL_RELEASE_DIR"
    ;;
  check)
    python3 .agent/scripts/release-evidence.py check --output "$CCVL_RELEASE_DIR"
    ;;
  publish)
    bash .agent/scripts/publish-release.sh "$CCVL_RELEASE_DIR"
    ;;
  *) echo "Unknown release stage: $action" >&2; exit 2 ;;
esac
