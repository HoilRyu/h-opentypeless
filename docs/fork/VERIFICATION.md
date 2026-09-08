# H-OpenTypeless 실행 검증

검증일: 2026-09-08
기준: v1.1.57 / 68cf6f11ad6632b86217a1729e33bb8d560610cf

## 환경과 범위

- 지원 대상: macOS, Windows, Linux. 이번 로컬 환경은 Apple Silicon Mac이다.
- H 저장소 소스를 동기화 외부 `~/.local/share/h-opentypeless/build-source`에 복사하여 빌드한다. 이 사본은 검증용이며 개발 원본은 H 저장소다.
- 기존 모바일 프로젝트 코드는 빌드에 사용하지 않는다.
- Rust 1.98.0, CMake 4.4.3을 Homebrew로 설치했다. Node 24.20.0, npm 11.19.0을 사용한다.
- 임시 빌드 사본에만 테스트 이름·bundle identifier·URL scheme·Keychain 서비스 이름을 분리하고 updater endpoints를 비웠다. H 저장소의 제품 코드는 아직 수정하지 않았다.
- 식별자: dev.hoilryu.hopentypeless.test / Keychain: H-OpenTypeless-Test.
- 이 사본의 임시 분리는 실행 검증을 위한 것이며 포크 제품의 정식 식별자 분리 구현이 아니다.

## 확인 결과

- npm ci: 성공. moderate 취약점 1개 보고; 자동 의존성 변경하지 않음.
- TypeScript 검사 및 프런트엔드 production build: 통과. 번들 크기·정적/동적 import 혼용 경고 있음.
- Vitest: 48개 파일, 449개 테스트 통과.
- ESLint, Prettier 검사: 통과.
- cargo fmt --check: 통과.
- Mac 네이티브 debug 앱 번들 빌드: 성공 (Rust 컴파일 약 3분).
- 실제 앱 실행: 초기 설정 화면 표시, 한국어 선택, 홈·설정 이동, 36초 이상 프로세스 유지 확인.
- Command+Q 정상 종료 및 재실행: 성공. 한국어와 초기 설정 완료 상태가 유지됨.
- 별도 test bundle identifier 경로에 설정·SQLite 파일 생성 확인.
- 음성인식·교정·실제 입력: 이번 테스트 앱에서는 제공자·권한을 설정하지 않았으므로 미검증.
- Rust 단위 테스트: 564개 통과, 실패 0. cargo clippy는 이번 실행 검증에서 수행하지 않음.
- 검증 후 테스트 앱은 종료했다. 테스트 번들과 별도 설정은 재검증을 위해 유지한다.
- Cargo 최초 의존성 다운로드에서 저속 타임아웃 발생. CARGO_HTTP_MULTIPLEXING=false 재시도로 다운로드 및 컴파일 진행.

## Windows·Linux 확인 범위

원본 .github/workflows/ci.yml은 Windows x64, macOS ARM64, Linux x64/ARM64에서 Rust fmt/clippy/test를 실행하도록 구성되어 있다. release.yml도 각 운영체제 산출물 구성을 갖고 있다. 이번에는 설정을 읽어 확인했으며 GitHub Actions 실행이나 해당 운영체제의 실제 실행은 수행하지 않았다.

배포 승인 조건은 각 운영체제 빌드와 앱 실행·트레이·단축키·마이크·텍스트 출력·모바일 연결이다. Linux는 X11/Wayland를 구분한다. 커서 복원은 운영체제별 지원 범위 검증이 추가로 필요하다.

## 독립 앱·직접 제공자 구현 후 확인

- 임시 Test 앱과 별개로 H 소스에 제품 식별자 분리를 반영하고 H-OpenTypeless.app을 빌드·실행했다.
- 프런트엔드449개, Rust565개 통과. ESLint/Prettier, cargo fmt, clippy -D warnings 통과.
- 실제 Qwen ASR와 Gemma4의 일반/스트리밍 직접 호출 통합 테스트1개 통과 (약11.5초).
- 합성 음성 테스트: “Please add a cancel button to the settings page.” 원문·교정문·스트리밍 결과 일치.
- 사용자가 H 접근성 권한을 허용하고 실제 마이크 녹음 및 입력 성공을 확인했다.
- 교정 끄기·취소·서버 장애의 실제 UI 조작 검증은 별도로 남아 있다. 정상 흐름 성공과 구분한다.
- Windows/Linux 실제 검증은 마지막 단계로 연기했다.
