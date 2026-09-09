#!/usr/bin/env bash
# Output path only on stdout. Cache the signed helper unchanged across app builds.
# Helper source changes intentionally produce a new binary and may require one new approval.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
identity="${H_SIGNING_IDENTITY:-$(cat "$HOME/.local/share/h-opentypeless/signing/identity.sha1")}"
[[ "$identity" =~ ^[0-9a-fA-F]{40}$ ]] || exit 1
source_hash="$(shasum -a 256 "$root/native/credential-helper/main.m" | cut -d' ' -f1)"
cache="$HOME/.local/share/h-opentypeless/credential-helper/$identity/$source_hash/$(uname -m)"
helper="$cache/h-credential-helper"
mkdir -p "$cache"
if [[ ! -f "$helper" ]]; then
  clang -O2 -Wno-deprecated-declarations -framework Foundation -framework Security \
    "$root/native/credential-helper/main.m" -o "$helper.tmp" >&2
  codesign --force --options runtime --timestamp=none --sign "$identity" \
    --identifier dev.hoilryu.hopentypeless.credentials "$helper.tmp" >&2
  mv "$helper.tmp" "$helper"
fi
codesign --verify --strict -R "=identifier \"dev.hoilryu.hopentypeless.credentials\" and certificate leaf = H\"$identity\"" "$helper" >&2
printf '%s\n' "$helper"
