#!/usr/bin/env bash
# Pre-release hook for cargo-release.
# Called with env vars: CRATE_NAME, NEW_VERSION, DRY_RUN
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
CHANGELOG="${REPO_ROOT}/crates/${CRATE_NAME}/CHANGELOG.md"

echo "Generating changelog for ${CRATE_NAME} v${NEW_VERSION}..."

CLIFF_ARGS=(
    --config "${REPO_ROOT}/cliff.toml"
    --include-path "crates/${CRATE_NAME}/**"
    --tag-pattern "${CRATE_NAME}-v[0-9].*"
    --tag "${CRATE_NAME}-v${NEW_VERSION}"
    --unreleased
)

if [[ -f "${CHANGELOG}" ]]; then
    CLIFF_ARGS+=(--prepend "${CHANGELOG}")
else
    CLIFF_ARGS+=(--output "${CHANGELOG}")
fi

git cliff "${CLIFF_ARGS[@]}"
git add "${CHANGELOG}"
echo "Changelog written to ${CHANGELOG}"
