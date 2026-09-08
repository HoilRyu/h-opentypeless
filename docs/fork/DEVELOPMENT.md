# H-OpenTypeless 개발

## 현재 단계

macOS·Windows·Linux용 포크다. 현재는 Mac의 독립 앱·직접 STT/LLM·입력 검증을 진행하며 Windows/Linux 실제 검증은 마지막 단계로 미룬다.

- 기반: 공식 v1.1.57 (68cf6f11ad6632b86217a1729e33bb8d560610cf).
- 소스: 이 H 저장소. 기존 모바일 프로젝트는 실행·빌드 의존성이 아니다.
- 앱 식별자: dev.hoilryu.hopentypeless. Keychain 서비스: H-OpenTypeless.
- 앱 링크: h-opentypeless. 공식 cloud OAuth 서비스가 이 포크 scheme으로 반환하는지는 검증하지 않았다. 이번 범위는 로컬/BYOK 연결이다.
- 공식 업데이트 확인 UI를 앱에서 분리하고 updater endpoints를 비웠다. 포크 전용 서명·자동 업데이트는 배포 단계에서 구성한다.

## Mac 로컬 빌드

Node 24, Rust stable, CMake 및 Xcode Command Line Tools가 필요하다. 저장소 루트에서 `./scripts/h-build-macos.sh`를 실행한다. 기본 빌드 사본은 `~/.local/share/h-opentypeless/build-source`이며 소스 스냅샷·node_modules·Rust target·dist를 함께 둔다. 개발 수정은 항상 H 저장소에서 한다. 이 스크립트는 선택한 외부 빌드 사본의 소스 파일을 동기화하므로 해당 위치에 별도 작업을 저장하지 않는다.

산출물: `~/.local/share/h-opentypeless/build-source/src-tauri/target/debug/bundle/macos/H-OpenTypeless.app`.
디버그·배포 서명 없는 로컬 검증용 앱이며, 배포용 서명/설치본은 아직 아니다.

Windows/Linux는 공식 Tauri/npm/Cargo 구조를 유지한다. 이 Mac 보조 스크립트는 그 플랫폼용 빌드 스크립트를 대신하지 않는다. 해당 OS의 빌드·실행 검증은 후속 단계다.

## 로컬 연결

앱 설정에서 다음 값을 지정한다. 주소와 모델은 이 Mac의 엔진 구성에 따른 예시이며 제품 기본값에 개인 주소·인증키를 넣지 않는다.

- STT: Local / Custom Whisper, `http://127.0.0.1:8765/v1`, `Qwen/Qwen3-ASR-1.7B`.
- LLM: Ollama, `http://127.0.0.1:11434/v1`, `gemma4:12b`.
- Custom Polish Instructions는 앱 설정으로 관리한다. 서버 코드에 프롬프트를 복제하지 않는다.
- H의 Ollama 제공자는 reasoning_effort=none을 보낸다. 다른 제공자는 이 필드를 보내지 않는다. 기존 모델·메시지·생성 옵션은 유지한다.
- Qwen ASR·Ollama는 별도 실행 엔진이다. H는 기존 Python 게이트웨이8787·추론 프록시11435를 호출하지 않는다.
- 이번 Mac 설정은 순정 앱의 STT 주소·모델·사용자 지침만 선별 복사했고 ASR 키는 로컬 런타임에서 읽었다. 공식 앱 설정은 수정하지 않았다.
- 순정 앱과 충돌을 피하려고 H의 이번 테스트 단축키는 Ctrl+Shift+Space, 토글 모드로 지정했다. 앱 설정에서 변경할 수 있다.

## 검사

외부 빌드 사본에서 `npm test`, `npm run lint`, `npm run format:check`, `cargo test --locked --manifest-path src-tauri/Cargo.toml`, `cargo clippy --locked --manifest-path src-tauri/Cargo.toml -- -D warnings`를 사용한다.

직접 엔진 통합 테스트는 일반 CI에서 ignored다. 16kHz mono signed 16-bit PCM 파일 경로를 H_PCM, H 앱 settings.json 경로를 H_SETTINGS로 지정하고 아래를 실행한다.

```sh
cargo test --locked --manifest-path src-tauri/Cargo.toml --test h_local_providers -- --ignored --nocapture
```

이 검사는 실제 Rust STT·LLM 제공자와 공식 프롬프트 생성 코드를 호출한다. 실제 마이크·전역 단축키·현재 입력칸 삽입 검증을 대체하지 않는다. 테스트용 발화만 사용한다.
