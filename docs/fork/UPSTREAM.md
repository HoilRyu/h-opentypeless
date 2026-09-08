# Upstream 변경 경계

기반: 공식 v1.1.57 / 68cf6f11ad6632b86217a1729e33bb8d560610cf.
origin: HoilRyu/h-opentypeless. upstream: tover0314-w/opentypeless.
현재 기능 브랜치: feat/h-foundation-direct-providers.

| 공식 파일 | 변경 이유 |
|---|---|
| src-tauri/tauri.conf.json | 제품 이름, 식별자, URL scheme 분리; 공식 업데이트 endpoint 제거 및 updater 산출물 생성 중지 |
| src-tauri/src/credentials.rs | Keychain 서비스 분리 |
| src-tauri/src/lib.rs | 트레이 기본 표시 이름과 CLI deep-link 테스트 scheme |
| src/lib/deep-link.ts 및 대응 테스트 | H 전용 URL scheme만 처리 |
| src/App.tsx | 공식 자동 업데이트 안내 UI 연결 제거 |
| src/components/MainLayout/index.tsx | H 앱 이름 표시 |
| src-tauri/src/llm/protocol.rs | Ollama에만 reasoning_effort=none 추가 및 제공자 격리 회귀 검사 |

프롬프트 생성·오디오·모델 제공자 공통 경로는 보존한다. 빌드 보조 스크립트, 선택 실행 로컬 통합 테스트, 포크 문서는 추가 파일이다.

## 포커스 변경·메뉴 막대 변경 경계

- extensions/mac_window: macOS Dock 정책 전환. lib.rs에서 main 표시/숨김에 연결.
- pipeline.rs: 다른 앱의 선택 텍스트 읽기 차단, 직접 출력/스트리밍 복구에 복사 창 연결.
- voice_intent/executor.rs: 대상 앱 변경 시 복사와 팝업 실행, 복원/입력을 하지 않는 회귀 테스트.
- 자동 커서 복원 실험은 사용자 요청으로 삭제. app_detector의 파일은 원본으로 복원.

공식 업데이트는 별도 브랜치에서 릴리스 단위로 병합하고 위 경계와 제공자 회귀 검사를 확인한다.

## 출력 음량 조절
별도 extensions/audio_ducking과 ForkSettings/AudioDuckingSetting에 구현. 원본 연결은 pipeline/commands/ask의 마이크 생명주기, lib 서비스 초기화/종료·명령등록, GeneralPane 컴포넌트 한 줄, Windows API 의존성에 한정. 상세는 AUDIO_DUCKING.md.

## Android 연결
별도 extensions/mobile, ForkSettings/MobileConnectionSetting, android/에 구현. 원본 연결은 lib.rs의 서비스 초기화·명령 등록과 GeneralPane 설정 카드에 한정하며 Cargo에 Axum/if-addrs 및 Tokio 네트워크 기능을 추가했다. 기존 STT/LLM 제공자와 프롬프트 빌더를 호출하고 원본 프롬프트 로직은 수정하지 않는다. 별도 Python 서버는 사용하지 않는다. 지원 제공자와 API 제약은 MOBILE_API.md 참조.
