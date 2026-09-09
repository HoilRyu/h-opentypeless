# USB audio panic review — 2026-09-08

Reviewed commit: b371c84. Read-only runtime inspection; no audio stress/reproduction, app replacement, or setting changes performed. This report identifies code risks, not a proven kernel-panic fix.

## Evidence

- Panic artifact: /Library/Logs/DiagnosticReports/panic-full-2026-09-08-231446.0002.panic. Report date 23:14:46 +0900, kernel Calendar epoch corresponds to 23:13:45 +0900. Do not treat report creation time as the exact fault time.
- Kernel data abort, FAR 0x10. Panicked task usbaudiod PID8245; backtrace includes AppleUSBXHCI and IOUSBHostFamily. No proof of which application initiated the fatal request.
- Preserved USB logs at23:13:03.316 show output volume control; at23:13:03.368–.373 Razer Seiren V2 X input/output starts. The roughly42-second gap to kernel Calendar prevents claiming these were immediately adjacent to the fault.
- Compressor and swap marked OK. No evidence here that LLM memory exhaustion caused the panic.
- After reboot, USB logs also contain ASMedia isochronous mismatched-event-TRB messages associated with EDIFIER M60. These are post-reboot observations, not proof of the pre-reboot culprit.
- Current devices include USB Razer input, EDIFIER output, CalDigit audio. Current device defaults do not prove defaults at fault time. Saved attenuation configuration: reduce/5%.

## Findings

### 1. Volume change completion is not synchronized with capture startup (high priority)

pipeline.rs:1322–1329 awaits Service.begin then starts AudioCaptureHandle. macos.rs write only waits for AudioObjectSetPropertyData to return. The installed Apple SDK AudioHardware.h (AudioObjectSetPropertyData discussion) explicitly says property values must not be considered changed until HAL listener notification, because many changes are asynchronous. There is no property listener/acknowledgement in this backend. Thus the worker acknowledgement is only API-call completion, not device-state completion. This can overlap volume controls with USB audio startup. It also permits quick stop/restore to observe old values, clear the recovery journal, and leave a delayed attenuation unaccounted for. Actual occurrence is unproven.

### 2. Capture stop does not await stream destruction before volume restoration (high priority)

AudioCaptureHandle.stop (audio/capture.rs:147) drops the stop sender and immediately marks Idle. The detached capture thread destroys its CPAL stream later (capture.rs:296–299). pipeline.rs:1843–1857 and commands/ask.rs:1388–1389 queue volume restoration immediately after stop. Stream teardown and output control can overlap; rapid subsequent recording also has no old-stream-completion barrier. capture.rs is unchanged from upstream68cf6f11, but H added volume restoration next to that asynchronous stop, so the interaction is our integration responsibility.

### 3. Device-change recovery errors keep polling the failed path (medium priority)

Service recv_timeout250ms calls engine.tick irrespective of last error. If default output changes while pending restoration points to a disconnected device, tick repeatedly calls restore and fails before updating seen. No backoff/circuit breaker. This is repeated property lookup/read in that case, not proof of repeated writes. Other error cases may repeat restoration attempts. Successful stable-device operation does NOT set volume four times per second: seen suppresses repeated application.

### 4. Off is only a next-recording setting (medium priority)

set_audio_ducking saves config and updates Status; it does not send an engine stop command. During a recording, switching Off leaves the active engine mode/polling in place until End. Startup recovery also runs regardless of saved Off. Therefore an Off indicator is not a general guarantee of no device operations. The next-session behavior may have been intentional, but it is inadequate as an immediate troubleshooting stop control.

### 5. Insufficient lifecycle diagnostics (medium priority)

lib.rs:817 initializes stdout tracing without an app-owned rolling file. No session-correlated timestamps for volume request/ack, capture stop requested/completed, device UID transitions. Current evidence cannot identify the exact H operation in the failing usbaudiod thread. Add metadata-only bounded logs; do not log audio/transcripts/secrets.

## Other checks

- The AudioObjectIsPropertySettable Boolean* binding uses u8, consistent with the SDK ABI. UID CFRelease is required by AudioHardwareBase.h and is not an identified over-release.
- resolve allocates size/4 u32 entries but passes the original byte size and ignores the actual returned byte count. Validate alignment/returned length and retry legitimate device-list size changes. A malformed non-multiple-of4 size would overstate writable capacity; no evidence the normal Apple driver returned such a size here. Do not call this a demonstrated corruption or panic cause.
- Android mobile processing reuses model providers and does not open the Mac microphone or invoke attenuation; no direct USB manipulation in that path.

## Proposed order

1. Immediate mitigation: disable attenuation and use built-in audio for ordinary work; avoid deliberately stress-testing the suspect USB route.
2. Add awaitable capture teardown and a serialized capture/attenuation lifecycle with bounded property-change completion handling, including cancellation and errors. No fixed sleep presented as a correctness fix.
3. Make Off terminate the active attenuation session; define controlled recovery and failure backoff. Preserve recovery evidence on unresolved asynchronous changes.
4. Harden Core Audio data-size handling and add metadata-only persistent diagnostics.
5. Use fake delayed backends to test late property completion, delayed stream teardown, rapid cancel/restart, device removal, Off and errors. Run real USB checks only after safeguards; no claim of a confirmed kernel fix without evidence.

Conclusion: H has concrete lifecycle weaknesses that could expose a USB/driver problem. Kernel fault is established; H causation, exact hardware culprit, and a complete fix are not yet established. Do not dismiss this as unrelated to H, or assert an application bug alone proves the kernel fault's root cause.

## Implemented safeguards

- Added shared audio lifecycle transition gate. Capture construction/play and stream destruction cannot overlap H macOS volume writes. stop/Drop waits up to3seconds for backend teardown acknowledgement; missing completion latches a process-wide fault preventing further capture starts/macOS property access. Native calls themselves cannot be forcibly interrupted; blocked worker teardown remains serialized and does not start parallel recovery writes.
- macOS writes register HAL property listeners before the request and read the actual hardware level after notification within1second. Already-matching levels skip the write because no-op changes need not notify. USB scalar quantization is allowed; confirmed actual levels are journaled. Callback uses static state, avoiding dangling context pointers. Volume confirmation errors suspend further volume writes, retaining recovery metadata, without latching the microphone fault. A genuinely blocked transition or missing capture teardown still blocks new capture. No speculative rollback writes.
- Journals now distinguish unconfirmed application from observed application, preserve unresolved delayed changes, and store observed applied values after confirmation. Legacy records remain readable.
- Engine stops periodic work on error. Off queues immediate end of active attenuation; saved Off performs no startup recovery or begin recovery. A recovery failure retains its record rather than blindly restoring.
- Native fixed-size reads and device-list byte lengths are validated. Bounded metadata-only h-audio-events.log plus one rotated file record requests/completions/faults (no speech, prompts or credentials).
- Added six regression tests: delayed teardown acknowledgement; completion/readback wait; completion timeout/error; device-transition failure stops polling; uncertain delayed write retains journal; Off does not recover hardware. Existing recovery tests also pass.
- Full Rust suite590passed/1hardwaretestignored; subsequent focused audio suite and clippy recorded in ~/.local/share/h-opentypeless/audio-fix-*.log. No physical USB stress test performed. This is mitigation of established code risks, not proof the underlying Apple USB driver panic is eliminated.

## Microphone regression found and corrected (2026-09-08)

The first safeguard build incorrectly required the USB readback to match the requested scalar within0.002. At23:38:44.999 H requested a volume change; usbaudiod logged scalar0.023632813 at23:38:45.071; H latched the global fault at23:38:46.004, before any capture_start_requested. This explains the user's microphone error without a permission reset.

With H stopped, one bounded diagnostic on the current EDIFIER output requested0.119097216 from0.340277761. Readback was0.125434026 and two notifications arrived. The previous volume was restored and verified. This reproduces hardware quantization exceeding our tolerance; it is not evidence of a microphone failure or a new panic.

The corrected backend returns the notified actual level to the engine, which journals that value for conditional restoration. Unit tests cover quantization, notification requirements, and preserving manual adjustments. The native opt-in test covers5%/35%/mute and restoration on this output. Native API calls that never return still cannot be forcibly interrupted.

## Additional bounded-resource changes

- Custom/OpenAI-compatible and cloud LLM HTTP responses and Whisper-compatible STT responses have an8MiB cap. SSE lines have a1MiB cap, retain partial UTF-8 bytes correctly, and stop on completion markers. Whisper releases the PCM allocation after encoding WAV.
- Audio attenuation command queue is capped at64; cancelled Begin requests are skipped. Metadata log rotation stops appending if rotation fails.
- Android AudioRecord stop/release is owned by the recording worker. A process-wide lease prevents another recorder if native cleanup is still blocked; repeated cleanup is idempotent. This prevents spawning overlapping recorders after a join timeout.
- These address concrete risks; no long-duration heap profiler run or claim that every leak/kernel panic is eliminated. Windows/Linux hardware validation remains deferred by user request.
