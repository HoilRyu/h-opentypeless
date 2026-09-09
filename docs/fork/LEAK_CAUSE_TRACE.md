# 메모리 원인 추적 — 2026-09-09

## 확인한 누수: CPAL 0.15.3의 macOS Device::name

현재 의존성 소스의 src/host/coreaudio/macos/mod.rs:165 부근 name()은 AudioObjectGetPropertyData로 소유권 있는 CFString을 받은 뒤 CFRelease하지 않는다. 앱 audio/capture.rs의 진단 로그에서 매 녹음 시작마다 호출했다.

마이크를 열지 않는 별도 Rust 프로세스에서 같은 CPAL0.15.3과 현재 기본 입력 장치를 사용했다. MallocStackLogging=1, leaks --noContent --fullStacks --atExit로 측정:

| 장치 이름 조회 횟수 | 검출 누수 | 바이트 |
|---|---:|---:|
| 0 | 0 | 0 |
| 100 | 100 | 6,400 |
| 1,000 | 1,000 | 64,000 |

할당스택은 probe main → cpal DeviceTrait::name → macos Device::name → CoreAudio → CFString 생성으로 이어진다. 따라서 이 조회의 선형 누수는 재현됐다. 전체 앱에서 관찰한 CFString/NSArray 누수 전부의 원인이라거나 kernel panic의 원인이라고 주장하지 않는다.

수정: Mac에서는 불필요한 장치이름 진단 호출을 없애고 ‘Using default input device’를 기록한다. 다른 OS는 기존 로그를 유지. 라이브러리 교체/벤더링/전역해제 실험 없음. 이름 외 샘플레이트·채널 진단 유지.

## UI 대조의 한계

홈/설정 왕복3회와 전후 memgraph 비교를 시도했지만 구간 중 실제 녹음이 추가됐다(오디오 로그 바이트 비교 불일치). 따라서 UI-only 대조로 판정하지 않는다. NSArray/AXObserverCookie 후보는 남아 있으며, 접근성 자동화/시스템 프레임워크/앱 소유권을 할당스택 없이 확정하지 않는다.

## 별도 발견: 중복 마이크 스트림

이름 호출 제거 설치본에서 Ask 검사와 실제 사용이 겹친 구간에 capture_start_confirmed가1788912276009와1788912279539에 연속 발생하고 첫stop은1788912298493이었다. 실제 공유캡처 경로에서 동시스트림이 허용됐다는 증거다. 기록만으로 두 스트림의 호출자를 개별식별할 수는 없지만 Ask와일반받아쓰기는별도상태를가지며공통capture함수에단일소유보호가없었다.

추가수정: 프로세스공통 CaptureLease를 start에서 획득하고 native스레드가실제stream파괴후해제한뒤종료ack전송. 두번째시작은마이크를열기전에거부. native정리가멈추면lease유지. 단순일시적인장치gate와달리녹음전체수명을보호한다. Ask에는다른녹음종료안내. 소유권반복1000회/스레드종료전보호회귀테스트추가.

## 자료

외부진단폴더 ~/.local/share/h-opentypeless/verification/leak-trace/ 에 name-probe 소스/Cargo.lock, name-0.txt/name-100.txt/name-1000.txt(할당스택), baseline/ui-before.memgraph, ui-diff.txt, 빌드·테스트로그보관. 음성/키/프롬프트를보고서에복사하지않음.

이름호출제거판은기존자체서명으로설치완료. 단일캡처보호추가판은빌드검증중. 사용자에게녹음종료및잠시사용중지요청한상태이므로동시입력을강제로중단하지말고교체시점조율할것. 현재사용자음량설정mute/2%; 이전reduce2%나Off로임의복원하지말것.

## 최종 적용

이름조회제거+CaptureLease+Ask중복녹음안내 포함 전체Rust597pass/1ignored, clippy --lib --tests -Dwarnings 통과. 자체서명최종빌드완료, 녹음종료로그확인후 /Applications 교체·재실행완료. 서명검증성공/접근성재등록배너없음. 실제두경로중복시나리오의수정후실기기재현검사는미실시; 1000회lease재사용및native스레드종료전독점유지단위테스트로보호검증. 사용자실사용과진단녹음을더겹치게하지않기위해추가자동녹음은시작하지않음. 현재mute/2%유지. UI/AX누수후보와패닉원인완전해결은미확정.

## NSArray 후속분리
오디오/H코드없는 AppKit 앱에서동일NSArray/AXObserverCookie 재현. 할당스택_NSAccessibilityRemoveAllObserversAndSendDestroyedNotification → NSArray복사 확인. 연속대조첫100회+448B, UI재조회없는다음100회+0B. AX_OBSERVER_INVESTIGATION.md참고. OS자체패치나H의모든누수제거로과장하지말것. 재현소스 tools/diagnostics/macos-ax-probe.swift.
