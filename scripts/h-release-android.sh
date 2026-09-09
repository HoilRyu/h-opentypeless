#!/usr/bin/env bash
# Release signing is required by Gradle. No install or upload is performed.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
source_record="$(mktemp -t h-release-source.XXXXXX)"
trap 'rm -f "$source_record"' EXIT
python3 "$root/scripts/h-release-manifest.py" snapshot "$source_record"
: "${JAVA_HOME:?Set JAVA_HOME to JDK 17 or 21}"
: "${ANDROID_HOME:?Set ANDROID_HOME to the Android SDK}"
cd "$root/android"
./gradlew --project-cache-dir "$HOME/.local/share/h-opentypeless/android-tools/project-cache" --no-daemon --max-workers=2 -PhReleaseAbis=arm64-v8a testDebugUnitTest lintRelease assembleRelease
apk="$HOME/.local/share/h-opentypeless/android-build/app/outputs/apk/release/app-release.apk"
apksigner="$ANDROID_HOME/build-tools/36.0.0/apksigner"
"$apksigner" verify --verbose --print-certs "$apk"
version="$(sed -n "s/.*versionName '\([^']*\)'.*/\1/p" app/build.gradle)"
desktop_version="$(node -p "require('$root/package.json').version")"
out="${H_RELEASE_DIR:-$HOME/.local/share/h-opentypeless/releases/$desktop_version}"
mkdir -p "$out"
dest="$out/H-OpenTypeless_${version}_android-arm64.apk"
[[ ! -e "$dest" ]] || { echo "Package already exists: $dest" >&2; exit 1; }
cp "$apk" "$dest"
python3 "$root/scripts/h-release-manifest.py" record "$dest" --platform android-arm64 --version "$version" --source "$source_record"
printf 'Release package: %s\n' "$dest"
