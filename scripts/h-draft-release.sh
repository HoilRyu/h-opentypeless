#!/usr/bin/env bash
# Explicitly run only after clean builds and physical validation. Always draft.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
tag="${1:?Usage: h-draft-release.sh TAG PACKAGE_DIRECTORY}"
packages="${2:?Specify the package directory}"
version="$(node -p "require('./package.json').version")"
[[ "$tag" == "v$version" ]] || { echo 'Tag must match the desktop version.' >&2; exit 1; }
[[ -z "$(git status --porcelain)" ]] || { echo 'Commit changes and rebuild before drafting a release.' >&2; exit 1; }
commit="$(git rev-parse HEAD)"
[[ "$(git rev-parse "$tag^{commit}")" == "$commit" ]] || { echo 'Local release tag must point at HEAD.' >&2; exit 1; }
# Check the H repository explicitly; never use upstream or a caller-selected remote.
remote_tags="$(git ls-remote git@github.com:HoilRyu/h-opentypeless.git "refs/tags/$tag" "refs/tags/$tag^{}")"
remote_commit="$(printf '%s\n' "$remote_tags" | awk '/\^\{\}$/ {print $1; found=1} END {if (!found) exit 1}')" || remote_commit="$(printf '%s\n' "$remote_tags" | awk 'NR == 1 {print $1}')"
[[ "$remote_commit" == "$commit" ]] || { echo 'Push the verified H release tag before drafting.' >&2; exit 1; }
python3 scripts/h-release-check.py
python3 scripts/h-release-manifest.py verify "$packages" --commit "$commit"
[[ -f "$packages/VALIDATION.md" ]] || { echo 'Add the completed installation/update validation record as VALIDATION.md.' >&2; exit 1; }
gh release create "$tag" --repo HoilRyu/h-opentypeless --verify-tag --draft --prerelease \
  --title "H-OpenTypeless $tag" --notes-file docs/fork/RELEASE_NOTES.md \
  "$packages"/*.dmg "$packages"/*.apk "$packages"/*.json "$packages/SHA256SUMS" "$packages/VALIDATION.md" \
  LICENSE android/app/src/main/assets/THIRD_PARTY_NOTICES.txt
