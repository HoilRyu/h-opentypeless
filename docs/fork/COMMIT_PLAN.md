# 커밋 정리 — 내장 MLX와 시작 가이드 (2026-09-10)

기준 커밋은 내장 STT `8b46610`이다. 내장 MLX는 `9561332`, 시작 가이드는 `d4bf270`으로 커밋했다. 현재 버전은 `0.1.45-beta.2`이며 이 문서와 사용자 안내를 마지막 문서 커밋으로 묶는다. 원격 푸시는 아직 수행하지 않았다.

## 1. 내장 MLX 기능과 배포 통합

제안 제목: `feat: run built-in Qwen STT with managed MLX on Apple Silicon`

환경 감지와 자동/MLX/CPU 선택, 녹음 중 준비와 모델 재사용, 취소/종료/유휴 회수, 파일 검증 캐시와 ARM SHA 가속을 포함한다. 전용 Python/MLX 번들·고정 의존성·서명·tokenizer 파일, 설정 UI와 테스트, 버전 변경, 실제 검증 문서도 함께 기록한다. 실행 코드가 번들 리소스와 빌드 설정에 의존하므로 이들을 독립 커밋으로 나누지 않는다.

포함 파일:

- `docs/fork/LOCAL_STT.md`
- `docs/fork/LOCAL_STT_NOTICES.md`
- `docs/fork/MLX_STT_PLAN.md`
- `docs/fork/MLX_STT_VERIFICATION.md`
- `native/mlx-stt/tokenizers.json`
- `native/mlx-stt/tokenizers/qwen-0.6b.json`
- `native/mlx-stt/tokenizers/qwen-1.7b.json`
- `native/mlx-stt/worker.py`
- `package-lock.json`
- `package.json`
- `scripts/h-prepare-local-stt.py`
- `scripts/h-prepare-mlx.py`
- `scripts/h-sign-macos.sh`
- `scripts/h-sign-mlx.py`
- `scripts/mlx/requirements.in`
- `scripts/mlx/requirements.lock`
- `src-tauri/Cargo.lock`
- `src-tauri/Cargo.toml`
- `src-tauri/src/extensions/local_stt/mlx.rs`
- `src-tauri/src/extensions/local_stt/mod.rs`
- `src-tauri/src/extensions/local_stt/provider.rs`
- `src-tauri/src/extensions/local_stt/verification.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/tauri.conf.json`
- `src/components/Settings/LocalSttSetting.tsx`
- `src/components/Settings/__tests__/LocalSttSetting.test.tsx`
- `src/i18n/locales/de.json`
- `src/i18n/locales/en.json`
- `src/i18n/locales/es.json`
- `src/i18n/locales/fr.json`
- `src/i18n/locales/it.json`
- `src/i18n/locales/ja.json`
- `src/i18n/locales/ko.json`
- `src/i18n/locales/pt.json`
- `src/i18n/locales/ru.json`
- `src/i18n/locales/zh.json`
- `tools/diagnostics/check_mlx_parent_exit.py`
- `tools/diagnostics/check_mlx_stt.py`

## 2. 인계와 튜토리얼 후속 계획

제안 제목: `docs: record MLX handoff and prepare first-run onboarding`

- `docs/fork/HANDOFF.md`: 실제 설치·검증 결과와 남은 작업.
- `docs/fork/ONBOARDING_PLAN.md`: 첫 실행 튜토리얼 계획. 선행 MLX 작업과 현재 상태를 구분한다.
- `docs/fork/COMMIT_PLAN.md`: 이 준비 목록과 기존 커밋 이력.

## 3. 사용자 README 간소화 (후속 요청)

제안 제목: `docs: simplify H setup and voice typing guide`

- `README.md`: H 기준의 설치 → STT → 선택적 AI 다듬기 → 단축키/입력 안내. Android 연결과 상세 문서 링크.
- `README_ko.md`: 중복된 원본 안내 대신 메인 한국어 사용 안내로 연결.
- `android/README.md`: 현재 모바일 연결 메뉴 위치 반영.

튜토리얼 구현은 포함하지 않는다. README 변경은 문서 링크와 표기만 검사하며 앱 재빌드/재설치를 하지 않는다.

## 4. H 첫 실행 튜토리얼 (후속 구현)

제안 제목: `feat: add a guided H setup and native voice practice`

- `src/components/HTutorial/`: 한국어/영어 3단계 안내 + 완료 화면, 설정 재사용, 진행 재개, 실제 녹음 미리보기, 오류/취소 회귀 검사.
- `src/App.tsx`, `src/lib/router.ts`, HomePage, AboutPane: H 전용 최초 진입과 다시 보기. 원본 Onboarding은 보존.
- `src-tauri/src/extensions/tutorial.rs`, extensions/mod.rs, lib.rs, mobile/mod.rs: main 창 전용 녹음 명령, 세션 취소/시간·메모리 제한, 기존 제공자 경로 재사용. 모바일 리스너를 요구하지 않는다.
- 앱 버전 `0.1.45-beta.2`, README 가이드 진입 안내, TUTORIAL/TUTORIAL_REDESIGN/HANDOFF/ONBOARDING 문서.
- 이전 MLX 변경과 공유하는 `lib.rs`는 기능별 hunk로 나눠 커밋했다. Cargo/package 버전은 MLX 커밋에서 `0.1.45-beta.2`로 올렸다.

## 검사와 제외 대상

- 직전 구현 검증: UI 472개, Rust 643개 통과. Rust 기본 실행의 ignored 5개 중 실제 MLX 제공자는 별도 실행해 통과했다. Clippy와 릴리즈 스크립트 테스트 10개 통과.
- Qwen 0.6B/1.7B 반복 전사, 120초 입력, 취소·부모 사망·유휴 회수, 설치본 모바일 API→MLX→Ollama와 서명된 DMG 검사 완료. 실제 마이크/Android 실기기 및 Windows/Linux 실기기를 이번에 다시 검증했다는 뜻은 아니다.
- 이번 준비에서는 변경 목록, 파일 크기, 명백한 개인키/토큰 패턴과 `git diff --check`를 확인했다. 기능 코드를 변경하지 않아 전체 테스트는 반복 실행하지 않았다. 패턴 검사는 모든 비밀 정보의 부재를 보증하는 검사는 아니다.
- 모델 가중치, Python 설치본, DMG/앱, 개인 인증서·개인키, 사용자 설정, 실행 로그와 벤치마크 결과 JSON은 포함하지 않는다. 저장소의 tokenizer JSON과 의존성 잠금 파일은 재현 가능한 빌드에 필요한 고정 공개 자료라 포함한다.
- 2026-09-10 원격 갱신 기준 브랜치는 원격보다 앞서 있으며, 이번에 만든 `9561332`와 `d4bf270`을 포함한 로컬 커밋은 문서 커밋 후 함께 푸시해야 한다.

아래는 이전 작업의 기록이며 현재 커밋 대상 목록은 위 네 묶음이다.

---

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
