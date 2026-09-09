#!/usr/bin/env bash
# Build and package the local self-signed release; never installs or uploads it.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
source_record="$(mktemp -t h-release-source.XXXXXX)"
trap 'rm -f "$source_record"' EXIT
python3 "$root/scripts/h-release-manifest.py" snapshot "$source_record"
[[ "$(uname -s)" == Darwin && "$(uname -m)" == arm64 ]] || { echo 'First release supports Apple Silicon only.' >&2; exit 1; }
export H_BUILD_PROFILE=release H_SIGN_MODE=self-signed
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export H_BUILD_ROOT="${H_BUILD_ROOT:-$HOME/.local/share/h-opentypeless/build-source}"
"$root/scripts/h-build-macos.sh"
bundle="$H_BUILD_ROOT/src-tauri/target/release/bundle/macos/H-OpenTypeless.app"
version="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$bundle/Contents/Info.plist")"
out="${H_RELEASE_DIR:-$HOME/.local/share/h-opentypeless/releases/$version}"
mkdir -p "$out"
archive="$out/H-OpenTypeless_${version}_macos-arm64.dmg"
[[ ! -e "$archive" ]] || { echo "Package already exists: $archive" >&2; exit 1; }
codesign --verify --deep --strict "$bundle"
"$root/scripts/h-package-dmg.sh" "$bundle" "$archive"
python3 "$root/scripts/h-release-manifest.py" record "$archive" --platform macos-arm64 --version "$version" --source "$source_record"
printf 'Release package: %s\n' "$archive"
