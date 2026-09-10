#!/usr/bin/env bash
# Keep dependencies and build output outside the Syncthing source checkout.
set -euo pipefail
if [[ "$(uname -s)" != Darwin ]]; then
  echo 'This helper is for local macOS builds; see docs/fork/DEVELOPMENT.md.' >&2
  exit 1
fi
source_root="$(cd "$(dirname "$0")/.." && pwd)"
sign_mode="${H_SIGN_MODE:-self-signed}"
profile="${H_BUILD_PROFILE:-debug}"
case "$profile" in
  debug) profile_args=(--debug --bundles app --no-sign) ;;
  release) profile_args=(--bundles app --no-sign) ;;
  *) echo 'H_BUILD_PROFILE must be debug or release.' >&2; exit 1 ;;
esac
case "$sign_mode" in
  self-signed) "$source_root/scripts/h-sign-macos.sh" --check ;;
  adhoc) ;;
  *) echo 'H_SIGN_MODE must be self-signed or adhoc.' >&2; exit 1 ;;
esac
build_root="${H_BUILD_ROOT:-$HOME/.local/share/h-opentypeless/build-source}"
mkdir -p "$build_root"
build_root="$(cd "$build_root" && pwd)"
case "$build_root/" in
  "$source_root/"*) echo 'Build directory must be outside the source checkout.' >&2; exit 1 ;;
esac
rsync -a --delete --exclude=.git --exclude=node_modules --exclude=target --exclude=dist --exclude=worktrees --exclude=.worktrees \
  "$source_root/" "$build_root/"
cd "$build_root"
npm ci
export VITE_APP_VERSION="v$(node -p "require('./package.json').version")"
helper="$("$source_root/scripts/h-prepare-credential-helper.sh")"
export H_CREDENTIAL_HELPER_SHA256="$(shasum -a 256 "$helper" | cut -d' ' -f1)"
export CARGO_HTTP_MULTIPLEXING=false
python3 "$source_root/scripts/h-prepare-local-stt.py" --config "$build_root/local-stt-bundle.json"
npm run tauri build -- "${profile_args[@]}" --config "$build_root/local-stt-bundle.json"

bundle="$build_root/src-tauri/target/$profile/bundle/macos/H-OpenTypeless.app"
mkdir -p "$bundle/Contents/Helpers"
cp "$helper" "$bundle/Contents/Helpers/h-credential-helper"
if [[ "$sign_mode" == self-signed ]]; then
  "$source_root/scripts/h-sign-macos.sh" "$bundle"
fi
