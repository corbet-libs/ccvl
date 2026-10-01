#!/usr/bin/env bash
# Provider-independent checks using an already provisioned build worker.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
# shellcheck source=.agent/scripts/rust-toolchain.sh
source .agent/scripts/rust-toolchain.sh
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-2}"
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}"
export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-0}"

if (($# == 0)); then
  set -- rust
fi
for check in "$@"; do
  case "$check" in
    rust)
      ccvl_select_rust_toolchain
      if [[ -n "${CI_COMMIT_SHA:-}" ]]; then
        export CCVL_STYLE_EVIDENCE="${CARGO_TARGET_DIR:-target}/style-parity/$CI_COMMIT_SHA"
      fi
      "${CCVL_CARGO_COMMAND[@]}" fmt --all -- --check
      # Clippy's metadata-only check is the quicker gate on a cold cache, so
      # lint findings surface before the full test build and run.
      "${CCVL_CARGO_COMMAND[@]}" clippy --locked --workspace --all-targets --all-features -- -D warnings
      "${CCVL_CARGO_COMMAND[@]}" test --locked --workspace --all-features
      ;;
    core-package)
      ccvl_select_rust_toolchain
      "${CCVL_CARGO_COMMAND[@]}" package --locked -p ccvl-core
      ;;
    lint)
      # shellcheck source=.agent/scripts/existing-tool-path.sh
      source .agent/scripts/existing-tool-path.sh
      ccvl_use_existing_lint_tools
      # The two legacy private workflows deliberately keep their jobs disabled.
      actionlint -ignore '^constant expression "false" in condition\. remove the if: section$' \
        -shellcheck shellcheck .github/workflows/*.yml
      shellcheck .agent/scripts/*.sh .agent/tests/*.sh ccvl
      reuse lint
      bash .agent/tests/test_bootstrap.sh
      bash .agent/tests/test_ci_changes.sh
      bash .agent/tests/test_ci_toolchain.sh
      bash .agent/tests/test_downstream_sync.sh
      bash .agent/tests/test_skill_eval_ci.sh
      python3 -B .agent/tests/test_release_evidence.py
      python3 -B .agent/tests/test_downstream_check.py
      ;;
    documents)
      [[ $(uname -s) == Linux ]] || { echo 'documents requires Linux' >&2; exit 2; }
      ccvl_select_rust_toolchain
      for command in file qpdf pdfinfo pdftotext pdffonts pdfdetach pdfimages pdftoppm jq; do
        command -v "$command" >/dev/null || { echo "Missing existing tool: $command" >&2; exit 2; }
      done
      "${CCVL_CARGO_COMMAND[@]}" build --locked --release
      binary="${CARGO_TARGET_DIR:-target}/release/ccvl"
      bash .agent/scripts/check-linux-deep.sh "$binary"
      ;;
    styles)
      # Scoped document checks: every workspace-wide check, plus rendering and
      # PDF verification for the space-separated <doc>/<style> selections in
      # CCVL_CHECK_STYLES (every style when unset). Dependencies keep the
      # release optimization level; thin LTO and one codegen unit are dropped
      # because they cost minutes of linking and gain seconds of checking.
      ccvl_select_rust_toolchain
      "${CCVL_CARGO_COMMAND[@]}" build --locked --bin ccvl \
        --config 'profile.ci-documents.inherits="release"' \
        --config 'profile.ci-documents.lto=false' \
        --config 'profile.ci-documents.codegen-units=16' \
        --profile ci-documents
      read -r -a selected <<<"${CCVL_CHECK_STYLES:-}"
      filters=()
      for style in ${selected[@]+"${selected[@]}"}; do filters+=(--style "$style"); done
      "${CARGO_TARGET_DIR:-target}/ci-documents/ccvl" public-check ${filters[@]+"${filters[@]}"}
      ;;
    skill-eval-build)
      bash .agent/scripts/skill-eval-ci.sh build
      ;;
    skill-eval)
      bash .agent/scripts/skill-eval-ci.sh evaluate
      ;;
    release-*)
      bash .agent/scripts/release-ci.sh "${check#release-}"
      ;;
    *) echo "Unknown check: $check (rust lint documents styles skill-eval-build skill-eval)" >&2; exit 2 ;;
  esac
done
