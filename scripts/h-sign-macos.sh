#!/usr/bin/env bash
# Sign the final bundle with a persistent local identity, never an ad-hoc fallback.
set -euo pipefail
[[ "$(uname -s)" == Darwin ]] || { echo 'macOS is required.' >&2; exit 1; }
source_root="$(cd "$(dirname "$0")/.." && pwd)"
identity_file="$HOME/.local/share/h-opentypeless/signing/identity.sha1"
identity="${H_SIGNING_IDENTITY:-}"
if [[ -z "$identity" && -f "$identity_file" ]]; then
  identity="$(cat "$identity_file")"
fi
[[ "$identity" =~ ^[0-9A-Fa-f]{40}$ ]] || {
  echo 'Set H_SIGNING_IDENTITY to your persistent certificate SHA-1 fingerprint.' >&2; exit 1;
}
# Self-signed identities may be marked untrusted; no system trust override is needed.
identities="$(security find-identity -p codesigning)"
if ! echo "$identities" | grep -qi "$identity"; then
  echo 'The configured signing identity is absent from the keychain.' >&2; exit 1
fi
[[ "${1:-}" != --check ]] || exit 0
bundle="${1:?Usage: h-sign-macos.sh /path/to/H-OpenTypeless.app | --check}"
identifier="$(/usr/libexec/PlistBuddy -c 'Print CFBundleIdentifier' "$bundle/Contents/Info.plist")"
[[ "$identifier" == dev.hoilryu.hopentypeless ]] || {
  echo 'Refusing to sign a bundle with an unexpected identifier.' >&2; exit 1;
}
requirement="designated => identifier \"$identifier\" and certificate leaf = H\"$identity\""
echo 'Signing with the existing local certificate. macOS may ask codesign to access its private key.'
echo 'This is a build-time signing prompt, separate from application API-key or Accessibility access.'
codesign --force --sign "$identity" --timestamp=none \
  --identifier "$identifier" --requirements "=$requirement" \
  --entitlements "$source_root/src-tauri/Entitlements.plist" "$bundle"
codesign --verify --deep --strict --verbose=2 "$bundle"
codesign --verify --strict -R "=identifier \"$identifier\" and certificate leaf = H\"$identity\"" "$bundle"
codesign -d -r- "$bundle"
