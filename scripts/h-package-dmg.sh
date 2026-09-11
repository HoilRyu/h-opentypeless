#!/usr/bin/env bash
# Package an already signed app. Never edits or installs the source app.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
[[ "$(uname -s)" == Darwin ]] || { echo 'DMG packaging requires macOS.' >&2; exit 1; }
bundle="${1:?Usage: h-package-dmg.sh /path/to/H-OpenTypeless.app /path/to/output.dmg}"
output="${2:?Specify a new DMG output path}"
[[ "$bundle" = /* && "$output" = /* && "$output" == *.dmg ]] || { echo 'Use absolute app and DMG paths.' >&2; exit 1; }
[[ ! -e "$output" ]] || { echo 'Refusing to overwrite an existing DMG.' >&2; exit 1; }
identifier="$(/usr/libexec/PlistBuddy -c 'Print CFBundleIdentifier' "$bundle/Contents/Info.plist")"
[[ "$identifier" == dev.hoilryu.hopentypeless ]] || { echo 'Unexpected application identifier.' >&2; exit 1; }
codesign --verify --deep --strict "$bundle"
venv="${H_DMG_TOOLS:-$HOME/.local/share/h-opentypeless/dmg-tools}"
if [[ ! -x "$venv/bin/python" ]]; then
  python3 -m venv "$venv"
fi
"$venv/bin/python" -m pip install --disable-pip-version-check -r "$root/scripts/dmg/requirements.txt"
mkdir -p "$(dirname "$output")"
"$venv/bin/dmgbuild" -s "$root/scripts/dmg/settings.py" -D "app=$bundle" H-OpenTypeless "$output"
hdiutil verify "$output"
# A valid disk-image checksum does not guarantee its copied app signature is valid.
mount="$(mktemp -d -t h-dmg-verify)"
cleanup() {
  hdiutil detach "$mount" >/dev/null 2>&1 || true
  rmdir "$mount" 2>/dev/null || true
}
trap cleanup EXIT
hdiutil attach -readonly -nobrowse -mountpoint "$mount" "$output" >/dev/null
codesign --verify --deep --strict "$mount/$(basename "$bundle")"
hdiutil detach "$mount" >/dev/null
rmdir "$mount"
trap - EXIT
printf 'DMG created: %s\n'  "$output"
