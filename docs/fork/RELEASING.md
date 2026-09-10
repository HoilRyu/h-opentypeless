# Apple Silicon 릴리즈 실행 가이드

소스 브랜치: `feat/h-foundation-direct-providers`. 대상: macOS 14 이상 arm64.
버전: 0.1.45-beta.3. 산출물: 서명된 DMG 한 개와 출처 JSON, SHA256SUMS.

## 로컬 검증 및 빌드

```sh
python3 scripts/h-release-check.py
python3 -m unittest discover -s scripts/tests -p 'test_release*.py'
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --lib
./scripts/h-release-macos.sh
```

Node 24, Rust, Xcode Command Line Tools, CMake가 필요하다. Mac 패키지는 기존 로컬 서명 인증서와 credential helper로 만든다. 서명 키를 새로 생성하거나 교체하지 않는다. 키 백업은 별도 안전한 위치에 보관하며 배포 파일에 넣지 않는다.

빌드는 저장소 밖 `~/.local/share/h-opentypeless/build-source`에서 release 프로파일로 수행한다. worktrees/.worktrees, Git, node_modules, target, dist는 소스 복사에서 제외한다. 기본 Cargo 동시 작업은 2개다.

내장 STT 엔진과 고정 버전 Python/MLX 환경을 번들에 포함한다. 모델 가중치·사용자 설정·자격 증명은 포함하지 않는다. 엔진과 실행 환경을 먼저 서명하고 앱을 최종 서명한다.

기본 출력 폴더: `~/.local/share/h-opentypeless/releases/0.1.45-beta.3/`.
기존 패키지는 덮어쓰지 않는다. 반복 후보는 `H_RELEASE_DIR`로 별도 폴더를 지정한다.
소스를 커밋한 뒤 빌드하며 빌드 도중 소스가 바뀌면 패키지 기록을 거부한다.

```sh
python3 scripts/h-release-manifest.py verify "$HOME/.local/share/h-opentypeless/releases/0.1.45-beta.3" --commit "$(git rev-parse HEAD)" --version 0.1.45-beta.3
```

## 배포 파일 검증

패키지 폴더 `VALIDATION.md`에 기기·OS·소스 SHA와 다음 항목의 실제 결과를 기록한다.

- DMG 무결성/마운트, 앱 서명, arm64 실행 파일, 최소 OS, 버전, 모델 미포함.
- 신규 설치 및 마이크·접근성·키체인 최초 허용.
- 기존 앱 업데이트 후 설정·기록·모델·권한 유지.
- 실제 음성 입력·중지·취소, Fn/Stream Deck, 포커스 유지와 이탈 시 복사.
- 구조화 역할·숫자 정정, 기록 재다듬기·비교·복사·실패 처리.
- 음량 복원, Bluetooth 재연결, 잠자기 복귀.

자동 테스트는 실제 마이크·권한·가청 결과 검사를 대신하지 않는다. 미완료 항목은 그대로 표시한다.

## 초안 및 공개

검증된 깨끗한 커밋에 버전과 같은 태그를 생성하여 `HoilRyu/h-opentypeless`로 푸시한 후:

```sh
./scripts/h-draft-release.sh v0.1.45-beta.3 "$HOME/.local/share/h-opentypeless/releases/0.1.45-beta.3"
```

스크립트는 Mac DMG 한 개의 소스 SHA·버전·체크섬을 확인한 후 Draft Pre-release만 만든다.
VALIDATION.md의 존재만 검사하므로 실제 통과 여부는 초안 준비 전에 검토해야 한다.
로컬 스크립트만으로 준비할 수 있으며 GitHub Actions 수동 실행은 필수 단계가 아니다.
Android 빌드 도구는 보존하지만 이번 초안에는 APK를 요구하거나 업로드하지 않는다.
다운로드 검증과 최종 릴리즈 노트를 확인한 뒤 공개한다. 공개된 태그나 파일은 덮어쓰지 않는다.
