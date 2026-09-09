# H-OpenTypeless 첫 GitHub 릴리즈 계획

기준: 2026-09-09, 99f04ba. 이 문서는 계획이며 태그 생성, 워크플로 변경, 릴리즈 공개는 수행하지 않았다.

2026-09-09 준비 구현: H 전용 워크플로/빌드/서명/초안 등록 스크립트와 버전 변경을 반영했다. 아래 현재 상태는 계획 작성 당시의 기준이며 실행 절차는 RELEASING.md, 최신 상태는 HANDOFF.md를 따른다.

## 배포 범위

첫 공개는 macOS Apple Silicon + Android arm64의 GitHub Pre-release를 권장한다. Windows/Linux의 개발 지원은 유지하지만 실제 검증 전 정식 지원 바이너리로 공개하지 않는다. Intel Mac도 helper와 앱의 아키텍처 및 실제 실행 검증 후 추가한다. 최초에는 수동 다운로드/교체 방식으로 배포하고 자동 업데이트는 후속 단계로 둔다.

## 확인한 현재 상태

- .github/workflows/release.yml은 v* 태그에 반응하고 tover0314-w/opentypeless, releaseCommitish: main, 원본 RELEASE_TOKEN 및 Apple 공증 비밀값을 요구한다. Windows SignPath와 macOS staple 워크플로도 원본 저장소를 참조한다.
- scripts/h-build-macos.sh는 --debug --bundles app을 사용한다. credential helper를 별도 준비하고 해시를 주입한 뒤 번들에 넣고 로컬 고정 인증서로 서명한다. 기존 tauri-action 경로만 실행해서는 이 작업을 대체할 수 없다.
- updater endpoints는 비어 있고 createUpdaterArtifacts는 false다. 공개키 값은 남아 있으므로 추후 H 전용 키로 교체해야 한다.
- 데스크톱 버전은 0.1.42, Android는 0.3.6/versionCode 9다. Android Gradle에는 배포용 signingConfig가 없다.
- 현재 소스는 feature 브랜치에 있다. main이 현재 기능을 포함한다고 가정하면 안 된다.

## 실행 순서

1. **H 배포 경로 분리**
   - 원본 배포 워크플로 자동 트리거를 정리하고 H 전용 워크플로를 추가한다. 원본 개발 코드 수정은 최소화한다.
   - 배포 대상은 HoilRyu/h-opentypeless로 고정하고 해당 저장소의 GITHUB_TOKEN/최소 권한을 사용한다.
   - 처음에는 수동 실행 → 빌드 산출물 검사 → Draft Release 순서로 한다. 입력받은 태그의 실제 커밋을 checkout하고 빌드 SHA를 기록한다.
   - 현재 브랜치를 검토 후 main에 반영하거나 검증된 커밋에서 릴리즈 브랜치를 생성한다. main을 자동으로 배포 원본으로 가정하지 않는다.
2. **버전 및 서명 확정**
   - 예시 첫 태그: v0.1.43-beta.1. 실제 번호는 기존 태그/릴리즈 중복 확인 후 확정한다. 데스크톱 package/lock/Cargo/Tauri 버전을 함께 동기화한다.
   - Android는 독립 버전을 유지하고 매 업데이트마다 versionCode를 증가시킨다. 릴리즈 노트에 데스크톱/모바일 호환 버전과 연결 API를 적는다.
   - Mac은 기존 자체 인증서 및 helper 재사용 원칙을 유지한다. 인증서/개인키와 helper 캐시의 안전한 백업·복원 절차를 정하고 Git에는 넣지 않는다.
   - Android는 장기 보관할 release keystore를 만들고 안전하게 백업한다. 현재 설치된 APK 서명과 비교하여 debug → release 전환 시 재설치 필요 여부와 설정/모델 보존 방법을 확인한다.
3. **배포용 빌드**
   - Mac release 프로파일 전용 스크립트를 추가한다. 기존 helper 준비/해시 주입/번들 포함/최종 서명 순서를 유지하고 .app을 ZIP 또는 DMG로 포장한다. 첫 회는 기존 인증서가 있는 로컬 Mac에서 빌드한다.
   - Android는 정식 서명 APK를 생성한다. 첫 지원 ABI는 실제 검증한 arm64 기준으로 표기하고 다른 ABI는 STT/LLM 라이브러리 호환성 확인 후 추가한다.
   - 모델 파일은 앱에 무조건 포함하지 않는다. 모델 다운로드 출처, 라이선스, 크기와 시스템 STT의 오프라인 지원 한계를 안내한다.
   - 모든 산출물에 버전/OS/CPU를 표시하고 SHA256SUMS, 라이선스 및 서드파티 고지를 준비한다.
4. **실제 배포 파일 검증**
   - UI/Rust/Android 검사와 release 빌드를 통과시킨다. 개발용 빌드의 테스트 기록을 release 설치 검증으로 간주하지 않는다.
   - GitHub에서 내려받은 Mac 패키지로 신규 설치·실행 허용·마이크/접근성·키체인 최초 허용을 확인한다. 기존 버전 교체 후 권한과 설정 유지도 별도로 확인한다.
   - 녹음/중지/취소 반복, 포커스 유지 입력과 이탈 시 복사 창, 복사 단축키, 음량 복원, 잠자기 복귀 및 장치 변경을 확인한다.
   - Android에서 APK 설치/업데이트, 원격 연결, 네트워크 단절, 로컬 STT/LLM, 취소/키보드 전환을 검사한다. 앱만 설치하면 모델 엔진까지 자동 제공되는 것으로 안내하지 않는다.
   - 재부팅의 단일 원인은 확정되지 않았으므로 관련 완전 해결을 릴리즈 노트에서 주장하지 않는다.
5. **시험판 공개**
   - Draft에 패키지, 체크섬, 설치/업데이트 방법, 필요한 모델 엔진, 알려진 제약, 문제 신고 양식을 올린다.
   - Mac 자체 서명은 Apple의 개발자 확인/공증을 대신하지 않음을 안내한다. 첫 실행 보안 안내는 Apple의 개별 앱 허용 절차를 따른다.
   - 최종 파일/태그/SHA 일치 확인 후 Pre-release로 공개한다. 수정 배포에는 새 버전/태그를 사용하고 공개된 바이너리를 조용히 덮어쓰지 않는다.
6. **후속 확대**
   - Windows: x64 설치 패키지, 마이크·단축키·입력·모바일 서버·업데이트 검증. 유료 인증서 없는 첫 시험판의 서명 상태를 명확히 표기한다.
   - Linux: 먼저 x64 AppImage 또는 deb 하나를 검증하고 X11/Wayland별 입력/단축키 제한을 명시한다. 이후 배포 형식과 CPU 범위를 넓힌다.
   - 자동 업데이트: H 전용 Tauri 서명 키/공개키/엔드포인트, 시험판 채널, 패키지 교체 후 Mac helper/권한 유지를 검증한 뒤 활성화한다. 이 키는 macOS 코드 서명 인증서 및 Android keystore와 별개다.

## 완료 기준

검증된 커밋과 태그가 일치하고, 다운로드한 release 패키지의 신규 설치 및 기존 설치본 교체를 통과하며, 서명 키 백업과 설치 안내가 준비되어야 첫 공개를 완료로 본다. Windows/Linux 실기기 검증은 별도 완료 조건으로 유지한다.

## 공식 참고

- Apple, 미확인 개발자 앱의 개별 실행 허용: https://support.apple.com/en-gb/102445
- Android, 앱 서명과 업데이트 키: https://developer.android.com/studio/publish/app-signing
- Tauri, updater 서명과 배포: https://tauri.app/plugin/updater/
