# macOS credential helper

H bundles this single-purpose executable under `Contents/Helpers/h-credential-helper`.
It is not a daemon and has no socket, launch agent, Dock icon, or extra installation step.
Windows and Linux continue to use their existing native keyring implementations.

## Identity and transport

- `h-prepare-credential-helper.sh` caches the signed binary outside the repository by signing certificate, source hash, and architecture. Ordinary app changes reuse those exact bytes; source changes or cache recreation may require another approval. Preserve the cached artifact when producing subsequent releases.
- The helper runs with hardened runtime, without permissive entitlements. Before reading input, it validates its live parent process with Security.framework against the H bundle identifier and its own signing certificate fingerprint. It rejects an unsigned shell caller.
- The app pins the bundled helper's SHA-256 at compile time via `H_CREDENTIAL_HELPER_SHA256`; mismatched/missing helper fails closed. Local macOS builds must use `scripts/h-build-macos.sh`. Ad-hoc/unsigned callers cannot use the helper.
- One bounded JSON request and response use anonymous stdin/stdout pipes. Secrets are not passed through command arguments, environment variables, files, or diagnostic logs. Each process exits after the operation; the app terminates/reaps a helper after 60 seconds.
- Only the `H-OpenTypeless` service and stt/llm/session API-key account names are supported. Existing payloads are preserved. There is no automatic key deletion or ACL broadening.
- The first access to a pre-existing item needs the user's approval for the helper. Subsequent app updates should retain the helper's CDHash. Updating the helper itself can require a new approval.

## Verification

`tools/diagnostics/credential-helper-probe.m` is a test caller. Compile two copies with `PROBE_BUILD=1` and `PROBE_BUILD=2`, sign both with the H identifier and certificate, and pass the same helper path as their sole argument. A creates/reads a dummy `llm.h-helper-probe.api_key`; B reads/deletes it. Only success/failure is printed. The probe must never be repurposed to print real credential values.

2026-09-09: both distinct signed callers read the dummy value successfully; a shell invocation of the helper was rejected. First installed app access was approved by the user. The existing STT item's ACL then gained the helper CDHash `7d291af3795b18500d547c488a4a01fba0f92b03`, and the app read the stored key successfully. Main-app A→B update verification is recorded in `docs/fork/KEYCHAIN_UPDATE_PROMPTS.md`.

This protects the application's keychain access boundary; it is not a defense against a compromised signing private key, OS, or user account. No claim is made that an independently rebuilt helper on another machine will have the same hash.
