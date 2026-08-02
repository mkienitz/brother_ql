#!/usr/bin/env bash
set -euo pipefail

readonly REMOTE=origin
readonly BRANCH=main
readonly CLI_MANIFEST=crates/brother-label/Cargo.toml

die() {
    echo "release: $*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Usage:
  scripts/release.sh prepare <brother_ql|brother-label> <major|minor|patch>
  scripts/release.sh finish  <brother_ql|brother-label>
EOF
}

select_crate() {
    CRATE=$1
    case "$CRATE" in
        brother_ql) CRATE_DIR=crates/brother_ql ;;
        brother-label) CRATE_DIR=crates/brother-label ;;
        *) die "unknown crate: ${CRATE}" ;;
    esac
    MANIFEST=${CRATE_DIR}/Cargo.toml
    CHANGELOG=${CRATE_DIR}/CHANGELOG.md
}

manifest_version() {
    sed -n 's/^version = "\([0-9][0-9.]*\)"$/\1/p' "$1" | head -n 1
}

head_manifest_version() {
    git show "HEAD:$1" | sed -n 's/^version = "\([0-9][0-9.]*\)"$/\1/p' | head -n 1
}

require_synced_main() {
    local branch upstream
    git fetch --prune "$REMOTE" "$BRANCH" --tags
    branch=$(git symbolic-ref --quiet --short HEAD) || die "detached HEAD is not releasable"
    [[ $branch == "$BRANCH" ]] || die "releases must run on ${BRANCH}, not ${branch}"
    upstream=$(git rev-parse --abbrev-ref '@{upstream}' 2>/dev/null) || die "${BRANCH} has no upstream"
    [[ $upstream == "${REMOTE}/${BRANCH}" ]] || die "expected upstream ${REMOTE}/${BRANCH}"
    [[ $(git rev-parse HEAD) == $(git rev-parse "${REMOTE}/${BRANCH}") ]] ||
        die "HEAD must exactly match ${REMOTE}/${BRANCH}"
}

require_current_tag() {
    local version=$1 latest
    latest=$(git tag --list "${CRATE}-v[0-9]*" --sort=-version:refname | head -n 1)
    [[ $latest == "${CRATE}-v${version}" ]] ||
        die "manifest version ${version} does not match latest tag (${latest:-none})"
}

require_tag_absent() {
    ! git show-ref --verify --quiet "refs/tags/$1" || die "tag already exists: $1"
}

generate_changelog() {
    git cliff \
        --config cliff.toml \
        --include-path "${CRATE_DIR}/**" \
        --tag-pattern "${CRATE}-v[0-9].*" \
        --tag "${CRATE}-v$1" \
        --unreleased \
        --prepend "$CHANGELOG"
}

require_expected_changes() {
    local allowed path expected
    allowed=$'Cargo.lock\n'"$MANIFEST"$'\n'"$CHANGELOG"
    [[ -n ${1:-} ]] && allowed+=$'\n'"$1"

    while IFS= read -r path; do
        [[ -n $path ]] || continue
        grep -Fxq "$path" <<<"$allowed" || die "unexpected release change: ${path}"
    done < <(git diff --name-only HEAD)
    [[ -z $(git ls-files --others --exclude-standard) ]] || die "untracked files are not allowed"

    for expected in Cargo.lock "$MANIFEST" "$CHANGELOG"; do
        ! git diff --quiet HEAD -- "$expected" || die "expected release change is missing: ${expected}"
    done
}

prepare() {
    local bump=$1 old_version new_version tag cli_version
    [[ $bump == major || $bump == minor || $bump == patch ]] || die "invalid bump: ${bump}"

    require_synced_main
    [[ -z $(git status --porcelain) ]] || die "the entire repository must be clean"
    old_version=$(manifest_version "$MANIFEST")
    [[ -n $old_version ]] || die "could not read ${MANIFEST} version"
    require_current_tag "$old_version"

    if [[ $CRATE == brother_ql ]]; then
        cli_version=$(manifest_version "$CLI_MANIFEST")
    fi

    cargo set-version -p "$CRATE" --bump "$bump"
    new_version=$(manifest_version "$MANIFEST")
    tag="${CRATE}-v${new_version}"
    require_tag_absent "$tag"

    if [[ $CRATE == brother_ql ]]; then
        [[ $(manifest_version "$CLI_MANIFEST") == "$cli_version" ]] ||
            die "cargo-edit unexpectedly changed brother-label's package version"
    fi

    generate_changelog "$new_version"
    local paths=(Cargo.lock "$MANIFEST" "$CHANGELOG")
    [[ $CRATE == brother_ql ]] && paths+=("$CLI_MANIFEST")
    git add -- "${paths[@]}"

    echo
    echo "Prepared ${CRATE} ${old_version} -> ${new_version}."
    echo "Review with: git diff --cached"
    echo "Edit ${CHANGELOG}; review those edits with: git diff"
    echo "Finish with: scripts/release.sh finish ${CRATE}"
}

finish() {
    local old_version new_version tag extra_manifest=path cli_version
    require_synced_main

    old_version=$(head_manifest_version "$MANIFEST")
    new_version=$(manifest_version "$MANIFEST")
    [[ -n $old_version && $new_version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] ||
        die "could not determine the prepared stable version"
    [[ $new_version != "$old_version" ]] || die "${CRATE} version was not changed"
    require_current_tag "$old_version"
    tag="${CRATE}-v${new_version}"
    require_tag_absent "$tag"

    if [[ $CRATE == brother_ql ]]; then
        extra_manifest=$CLI_MANIFEST
        ! git diff --quiet HEAD -- "$CLI_MANIFEST" ||
            die "a brother_ql release must update brother-label's dependency requirement"
        cli_version=$(head_manifest_version "$CLI_MANIFEST")
        [[ $(manifest_version "$CLI_MANIFEST") == "$cli_version" ]] ||
            die "brother-label's package version must remain unchanged"
    fi

    require_expected_changes "$extra_manifest"
    while IFS= read -r path; do
        [[ -z $path || $path == "$CHANGELOG" ]] || die "only the changelog may be edited after prepare"
    done < <(git diff --name-only)
    grep -Eq "^## \[${new_version//./\\.}\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$" "$CHANGELOG" ||
        die "${CHANGELOG} has no ${new_version} release heading"
    git diff --check HEAD
    cargo metadata --locked --format-version 1 >/dev/null
    cargo owner --list "$CRATE" >/dev/null || die "run 'cargo login' and retry"
    cargo publish --dry-run --locked --allow-dirty -p "$CRATE"

    git add -- "$CHANGELOG"
    [[ -z $(git diff --name-only) ]] || die "unstaged release changes remain"
    git commit -m "chore: release ${CRATE} v${new_version}"
    git tag "$tag"

    git push "$REMOTE" "HEAD:${BRANCH}" ||
        die "branch push failed; the release commit and ${tag} remain local"
    if ! cargo publish --locked -p "$CRATE"; then
        cat >&2 <<EOF
release: publish failed or timed out; ${tag} remains local
release: check crates.io, then retry publication and push the tag:
  cargo publish --locked -p ${CRATE}
  git push ${REMOTE} refs/tags/${tag}
EOF
        exit 1
    fi
    git push "$REMOTE" "refs/tags/${tag}" ||
        die "crate is published; retry with: git push ${REMOTE} refs/tags/${tag}"
    echo "Published ${CRATE} ${new_version} and pushed ${tag}."
}

main() {
    [[ $# -ge 1 ]] || { usage >&2; return 2; }
    local command=$1
    shift
    case "$command" in
        prepare)
            [[ $# -eq 2 ]] || { usage >&2; return 2; }
            select_crate "$1"
            prepare "$2"
            ;;
        finish)
            [[ $# -eq 1 ]] || { usage >&2; return 2; }
            select_crate "$1"
            finish
            ;;
        -h | --help | help) usage ;;
        *) usage >&2; return 2 ;;
    esac
}

cd "$(git rev-parse --show-toplevel 2>/dev/null)" || die "not inside a Git repository"
main "$@"
