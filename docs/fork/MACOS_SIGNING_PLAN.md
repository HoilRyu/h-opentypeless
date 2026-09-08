# macOS 업데이트 후 권한 유지 계획

## 현재 상태 (2026-09-08)

빌드 보조 스크립트는 --no-sign을 사용한다. 설치 앱 codesign 검사 결과 Signature=adhoc, TeamIdentifier=not set, Info.plist=not bound이며 사용 가능한 코드 서명 identity는 0개다. 임시 서명은 빌드별 코드에 묶여 있으므로 접근성 권한의 앱 식별이 업데이트 후 유지되지 않을 수 있다. 단순히 앱 이름이나 설치 경로만 고정해서 해결할 수 없다.

## 권장 순서

1. 서명 identity를 결정한다. 외부 배포까지 할 앱이면 Apple Developer Program의 Developer ID Application을 권장한다. 개발 전용 Apple Development 서명에서 배포 서명으로 전환하면 다시 권한 요청이 생길 수 있다. 로컬 자체 서명 인증서는 개발용 후보지만 대상 macOS에서 TCC 유지 여부를 두 빌드로 먼저 검증해야 하며 배포용 대안으로 보장하지 않는다.
2. 앱 identifier dev.hoilryu.hopentypeless와 서명 identity/호환 designated requirement를 유지한다. 인증서와 개인키는 키체인 또는 배포 비밀 저장소에 보관하고 Git/Syncthing 소스에 넣지 않는다.
3. h-build-macos.sh에 명시적 서명 모드를 추가한다. 서명 모드에서는 --no-sign을 제거하고 APPLE_SIGNING_IDENTITY를 Tauri에 전달한다. 인증서가 없을 때 임시 서명으로 조용히 대체하지 않고 실패시킨다. 단순 UI 개발용 unsigned 빌드와 설치용 signed 빌드를 구분한다.
4. 설치 전에 codesign --verify --deep --strict와 codesign -d -r-로 번들 서명/식별 조건을 확인한다. 기존 설치본과 업데이트본의 식별 조건이 호환되는지 검사한다. 실행 중 앱 종료 후 /Applications 사본을 교체한다.
5. 새 서명 앱에서 접근성 권한을 최초 한 번 등록하고 버전 A→코드가 다른 버전 B로 교체한다. 재등록 없이 단축키·실제 텍스트 입력·마이크가 동작하는지 확인한다. 동일 산출물 재설치만으로 검증을 대신하지 않는다.
6. 외부 배포 때는 Developer ID 서명에 공증/notarization과 staple을 추가한다. 공증은 배포 신뢰용이며 TCC 권한 보존 자체와는 별도다. Tauri 업데이트 패키지 서명 키도 macOS 앱 서명 인증서와 별도로 관리한다.

이번 요청에서는 계획만 작성했다. 인증서 생성·키체인 신뢰 변경·개발자 프로그램 가입은 수행하지 않았다. macOS 서명 변경은 Windows/Linux 구현과 분리한다.

## 근거

- Apple TN3127: https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements
- Tauri macOS signing: https://tauri.app/distribute/sign/macos/

## 무료 자체 서명 실행 계획 (사용자 선택)

유료 가입은 진행하지 않는다. 아래 작업은 아직 미실행이며, 먼저 현재 변경의 커밋·푸시를 마친 뒤 시작한다.

1. 키체인에서 H-OpenTypeless 전용 코드 서명 인증서와 개인키를 한 번 생성한다. 매 빌드마다 생성하지 않는다. 인증서 용도는 Code Signing으로 제한하고, 개인키는 소스/Syncthing/GitHub에 저장하지 않는다. 암호화한 백업은 별도 보관한다. 시스템 전체 인증서 신뢰나 Gatekeeper/SIP 설정을 완화하지 않는다.
2. scripts/h-build-macos.sh에 self-signed 모드를 추가한다. 지정한 identity가 없으면 빌드를 실패시킨다. 앱 identifier를 유지하고 완성된 번들을 서명한다. 개발용 임시 서명 빌드와 설치용 자체 서명 빌드를 명확히 구분한다.
3. 서명 유효성과 designated requirement를 검사한 뒤 버전 A를 설치한다. 사용자가 접근성·마이크 권한을 등록한다. 이어 실제 코드가 달라진 버전 B를 동일 identity로 서명하여 교체하고 권한 유지/단축키/실제 입력을 검증한다. 자체 서명으로 TCC가 유지된다고 미리 보장하지 않는다. 실패하면 서명 요구조건과 TCC 동작을 조사하고 한계를 기록한다.
4. GitHub 다운로드 환경을 재현해 ZIP/DMG 설치와 macOS의 '확인 없이 열기' 절차를 검증한다. 기존 개발 Mac에서 실행된다는 것만으로 다른 Mac 설치 검증을 대체하지 않는다. Apple Developer ID/공증이 없다는 점을 릴리스에 명시한다.
5. 소스, 설치 파일, SHA256 체크섬, 설치/권한 안내를 Releases에 제공한다. 공증되지 않은 앱의 최초 실행 경고는 남을 수 있다. 자동 업데이트를 나중에 연결한다면 별도의 Tauri 업데이트 서명 키를 사용한다.

완료 기준: 서로 다른 두 빌드 간 업데이트 검증 기록, 비밀키가 제외된 산출물, 실제 다운로드 설치 검증, 반복 권한 요청 여부와 한계가 명시된 문서. 현재는 계획만 작성했고 인증서 생성·서명 방식 변경은 수행하지 않았다.
