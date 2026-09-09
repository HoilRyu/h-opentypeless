# 커밋 정리 결과 — 2026-09-09

기준 b371c84 이후 변경을 아래와 같이 기록했다. 원래 10개 제안에서 컴파일 의존성과 공유 파일을 고려해 Android 기능/UI, 데스크톱 녹음/표시/결과 창을 각각 묶었다. 푸시는 수행하지 않았다.

| 커밋 | 내용 |
|---|---|
| f6c86c3 | macOS 키체인 helper와 업데이트 시 접근 안정화 |
| 85f7f2a | Android·데스크톱·메뉴 막대 아이콘 |
| ee18698 | 받아쓰기 의도 보존과 AI 응답 크기 제한 |
| ff3177f | Android 온디바이스 처리와 키보드·설정 UI |
| 8b0bb8d | 모바일 연결 메뉴 분리와 계정·업그레이드 노출 제거 |
| 684e189 | 녹음 수명 주기 안정화, 입력창 기준 캡슐, 포커스를 빼앗지 않는 복사 창 |
| 73c9430 | 설정 가능한 복사 결과 단축키 |
| 이 문서의 커밋 | 디자인 시안 보존, 조사 기록과 인수인계 문서 |

커밋 정리 중 이전 UI 기대값을 현재 계정 메뉴 제거 및 모바일 저장/오류 표시에 맞게 수정했다. 전체 UI 467개(53개 파일) 통과, `npm run build` 통과. 번들 크기와 혼합 동적 import 경고는 남아 있다. Rust는 직전 실행 기록 629개 통과·장치 테스트 2개 ignored를 확인했으며 이번 정리에서 재실행하지 않았다. Android 역시 기존 실기기 검증 기록을 유지한다. Windows/Linux 실제 검증과 장시간 안정성 검증은 완료로 취급하지 않는다.

아래는 실행 전 제안의 기록이다. 당시 미커밋 상태와 테스트 실패 수치는 위 결과로 대체된다.

# 미커밋 변경 정리 — 2026-09-09

기준 HEAD: b371c84 `feat: integrate Android keyboard with built-in mobile server`.
검토 시 추적 파일 변경 87개, 새 파일 133개이며 staging된 변경은 없다. 아래는 제안 순서이며 아직 커밋/푸시는 수행하지 않았다.

| 순서 | 제안 커밋 | 포함할 내용 |
|---|---|---|
| 1 | fix: stabilize audio capture and recording cancellation | 마이크 독점/종료 확인, CPAL 이름 조회 누수 회피, 상태 세대 취소, 스트리밍 worker 정리, 응답 크기 제한, USB 음량 변경·복원과 관련 회귀 검사/조사 문서 |
| 2 | fix: preserve dictation intent during AI polishing | 교정 프롬프트, dictation_guard, 추론만 있는 응답 제외, 비정상 확장 시 원문 복귀, 데스크톱 STT/LLM 통합 검사 |
| 3 | feat: add Android on-device speech and text processing | Whisper JNI, Gemma, 모델 관리/다운로드, 로컬 추론 서비스, 폴백과 취소·메모리 관리, 라이선스 고지 및 Android 검사 |
| 4 | feat: refine Android keyboard and settings | Android 키보드/설정 화면, 파형/편집 UI, 연결 안내, 앱 버전 및 관련 검사. 3번과 공유 코드가 많으면 기능별 hunk로 나누되 각 커밋이 빌드되어야 함 |
| 5 | design: apply H icons across supported platforms | 승인된 원본 아이콘, Android adaptive/themed 리소스, 데스크톱 PNG/ICNS/ICO, macOS 메뉴 막대 템플릿, 내보내기 스크립트 |
| 6 | feat: simplify H navigation and separate mobile connection | 순정 클라우드 계정·업그레이드 노출 제거, H 기능 분기, 모바일 연결 메뉴 분리와 UI 검사 |
| 7 | fix: keep macOS credential access stable across updates | credential helper 소스, 호출자 검증/타임아웃, 빌드·서명 helper 재사용, credentials 연동과 검사/문서. 개인 인증서·개인키는 제외 |
| 8 | feat: anchor recording feedback to the input field | 화면 테두리·캡슐 표시 설정, 파형, 입력창 AX 영역 기반 위치, 화면 경계 보정, 조회 불가 시 하단 배치 |
| 9 | fix: keep copy results visible without taking app focus | 실제 활성 앱 판정, 복사 창 제목/디자인/캡슐 근처 배치, NSPanel 비활성 표시, 첫 클릭 처리, Tauri capability 및 검사 |
| 10 | feat: add a configurable copy-result shortcut | hotkeys.copyResult, 등록/변경/제거/충돌/백업, 표시된 복사 결과만 네이티브 클립보드로 복사, 복사됨/실패 표시 및 검사 |

## 커밋 전 정리

- pipeline.rs, lib.rs, Cargo 파일, GeneralPane, Android 설정/IME 등은 여러 묶음이 섞여 있어 파일 단위 `git add`만으로 나누면 안 된다. 기능별 diff를 stage하고 각 단계의 컴파일 의존성을 확인한다.
- 기능별 문서와 테스트는 해당 커밋에 포함한다. HANDOFF.md는 최종 상태를 반영한다.
- docs/design/mobile-redesign-v1/v2의 PNG 시안은 런타임 자산과 구분한다. 필요한 최종 설계만 남길지 결정하고, 보존한다면 별도 docs 커밋으로 묶는다. 여기서는 삭제하지 않았다.
- 다운로드 모델, APK/앱 빌드, 개인 인증서/키, 설정 DB, 진단 실행 로그는 커밋 대상에서 제외한다. silence.wav 등 의도적으로 포함한 테스트/앱 자산은 용도 확인 후 포함한다.
- 전체 UI 검사에서 남은 8개 실패는 이전 클라우드 메뉴 제거와 모바일 연결 버튼 변경을 아직 반영하지 않은 기대값이다. 실제 H 동작에 맞게 테스트를 정리하고 전체 검사 재통과 후 커밋 묶음을 확정하는 것이 좋다.

## 검증 상태

- 최신 Rust: 629 통과, 물리 장치 테스트 2개 ignored.
- 최신 복사 단축키/설정/백업/스토어 관련 UI: 119 통과.
- macOS 자체 서명 빌드/설치 및 설정 화면 확인 완료. 사용자가 복사 단축키 정상 동작을 확인했다.
- Android 검증은 android/README.md 및 HANDOFF.md의 이전 실기기/테스트 기록을 따른다. 이번 정리에서 재실행하지 않았다.
- Windows/Linux 실제 검증과 장시간 녹음 안정성 검증은 아직 완료로 취급하지 않는다.
