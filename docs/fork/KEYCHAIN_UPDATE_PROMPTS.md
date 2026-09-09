# Repeated keychain approval after updates

2026-09-09: read-only native Security.framework audit with interaction disabled. No password/private-key bytes requested; no ACL modified.

## Findings

- Signing private key `H-OpenTypeless Local Code Signing`: trusted application `/usr/bin/codesign`; partitions `apple-tool:` and `apple:`. This is not evidence of a missing Always Allow setting for codesign.
- One generic-password item under service `H-OpenTypeless`: account `stt.custom-whisper.api_key`.
- Its application ACL contains repeated H app paths, including installed and old build-output paths.
- Its partition ACL contains **28 cdhash entries**, with no stable team partition.
- Latest partition `cdhash:a5ebdd12a2e4529344cd7a376749de0d610aeffe` matches the currently installed app's CDHash.
- The installed app has `TeamIdentifier=not set`, while its designated requirement still pins the same bundle ID and local signing certificate.

This directly supports the user's report: repeated approval is being recorded against successive executable hashes. A fixed signing certificate/designated requirement preserved Accessibility identity but did not provide stable keychain partition authorization here. Do not tell the user to select Always Allow again as a complete fix.

## Resolution constraints

- Preserve the stored key. Do not silently remove it or assume a local endpoint never uses authentication.
- Do not disable keychain protection or permit arbitrary applications.
- An explicit no-auth provider mode could avoid accessing a key when the configured endpoint does not require one; this would not solve updates for authenticated providers.
- A stable credential helper is a possible free architectural solution: its executable/hash must remain unchanged across ordinary app updates, and it must verify callers against the app's pinned signing identity. This requires implementation and A/B update testing; it is not implemented or validated yet. Updating the helper itself may require approval.
- Apple-issued signing identity is another deployment path, outside the user's current free-signing requirement.

No functional fix or access-control change was applied during this diagnosis.

## Implemented and verified

2026-09-09: implemented bundled macOS helper (`native/credential-helper`). Existing credentials are accessed through the same cached signed helper; Windows/Linux retain native keyring. User approved its first access. No ACL was programmatically broadened and no real secret was read into diagnostic output.

Installed app A CDHash: `8fd584f42556b6362f54c247f106fc069cc5c0ab`. App B: `612cc1f64586fb47c89481613de4b973c077077d`. Helper SHA-256 in both: `03ddf7a45a52a90ee92a25e9c576a41175e1e44523b12b5dd73bebecb4a9ef99`.

App B read the existing STT key from its settings pane immediately after replacement, without another approval prompt. Before and after B, the item retained 29 partition entries; the last was the helper CDHash `7d291af3795b18500d547c488a4a01fba0f92b03`. The main app's new CDHash was not added. This demonstrates the intended update behavior on this Mac, not a guarantee for helper replacement or another machine.

Validation: two distinct signed diagnostic callers read the dummy item and cleaned it up; unsigned shell caller rejected. Rust credential tests: 17 passed. Window reopen tests: 2 passed. AskPanel and locale tests: 21 passed. User confirmed the copy fallback appears without the main window; follow-up layout correction addresses the truncated title.
