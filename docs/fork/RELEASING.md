# H 릴리즈 실행 가이드

첫 범위: macOS arm64 자체 서명 ZIP + Android arm64 release APK. 공개는 별도의 단계이며 기본 스크립트는 설치된 앱을 교체하지 않는다.

## 서명과 빌드 환경

Mac: Node 24, Rust stable, Xcode Command Line Tools, CMake. 기존 `~/.local/share/h-opentypeless/signing/identity.sha1`의 인증서와 개인키를 사용한다. 키체인 helper의 서명된 캐시를 같은 인증서/소스 버전에서 재사용한다. 인증서/P12와 개인키는 안전한 별도 저장소에 백업해야 한다.

Android: JDK 17 또는 21, Android SDK 36 및 Build Tools 36.0.0, NDK 28.2.13676358, CMake 3.22.1. JDK 25는 현재 Gradle 8.13과 호환되지 않는다. 로컬 JDK 21은 `~/.local/share/h-opentypeless/android-tools/jdk21/Contents/Home`에 준비했다.

`JAVA_HOME=... python3 scripts/h-prepare-android-signing.py`는 최초 1회만 키를 생성한다. `~/.local/share/h-opentypeless/signing/android/`에 keystore와 비밀번호 설정을 0600 권한으로 보관하며 기존 키는 교체하지 않는다. 이 폴더를 암호화된 별도 저장소에 백업한다. 저장소나 릴리즈 파일로 업로드하지 않는다.

## 로컬 준비

```sh
python3 scripts/h-release-check.py
python3 -m unittest discover -s scripts/tests -p 'test_release*.py'
npm test
./scripts/h-release-macos.sh
```

Android는 환경에 맞게 JAVA_HOME/ANDROID_HOME을 설정한 후 실행한다.

```sh
export JAVA_HOME="$HOME/.local/share/h-opentypeless/android-tools/jdk21/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export GRADLE_USER_HOME="$HOME/.local/share/h-opentypeless/android-tools/gradle-home"
./scripts/h-release-android.sh
```

기본 산출물 폴더는 `~/.local/share/h-opentypeless/releases/<desktop-version>/`이다. 반복 검증 시 `H_RELEASE_DIR`에 별도 폴더를 지정한다. 같은 이름의 패키지를 덮어쓰지 않는다. Mac release 빌드는 기본 2개 Cargo 작업, Android는 2개 Gradle worker로 실행한다.

빌드 전후 소스 상태가 달라지면 패키징을 중단한다. 각 패키지 옆 JSON에는 소스 커밋·미커밋 여부·SHA256을 기록한다. `SHA256SUMS`는 패키지의 체크섬이다. 미커밋 빌드는 준비/검사용이며 공개용으로 등록할 수 없다. 공개 전 소스를 커밋한 뒤 동일 커밋에서 두 패키지를 다시 빌드한다.

## GitHub Actions

`H Release Preparation`은 수동 실행만 지원한다. source_ref에 확정 커밋 SHA나 태그를 입력한다. 프런트엔드와 배포 설정 검사를 수행하며 선택 시 같은 확정 SHA에서 Android release APK를 생성한다. 바이너리는 Actions artifact로만 보관하고 공개하지 않는다.

Android CI 서명에는 기존 키의 `H_ANDROID_KEYSTORE_BASE64`, `H_ANDROID_KEYSTORE_PASSWORD`, `H_ANDROID_KEY_ALIAS` secrets가 필요하다. base64는 암호화가 아니다. 소스 코드/로그에 값을 넣지 않는다. 이 준비 작업에서는 GitHub secrets를 등록하지 않았다. Mac은 기존 로컬 인증서/helper를 사용하여 빌드한다.

원본 release/SignPath/staple/drafter 작업에는 원본 저장소 전용 조건을 추가했다. 원본 기능 코드를 삭제하지 않고 H에서 실행을 막는다. H 워크플로를 수동 실행 메뉴에 노출하려면 기본 브랜치에도 반영해야 한다.

## 공개 전 검증 기록

다음 항목을 실제 배포 패키지로 검사하고 결과를 패키지 폴더의 `VALIDATION.md`에 작성한다. 자동 검사로 대체하지 않는다.

- Mac: ZIP 재다운로드/압축 해제/서명 검증, 신규 설치와 첫 권한 설정, 기존 버전 교체 후 설정·권한·키체인 유지
- 녹음 시작/중지/취소, 포커스 유지 입력 및 이탈 복사, 복사 단축키, 음량 복원
- Android: 정식 서명 설치/업데이트, 키보드 입력, 원격 통신, 네트워크 단절/로컬 추론/취소
- 기록: OS/기기/버전/소스 SHA, 실제 통과한 항목과 아직 미검증인 항목
- 배포 키의 오프라인/별도 위치 백업, 패키지 라이선스 고지 확인

기존 Android 개발 APK와 release APK는 서명이 다르면 업데이트할 수 없다. 사용자 기기에서 앱 삭제를 자동 실행하지 않는다. 전환 전에 설정·모델 삭제 영향을 안내하고 설치 방법을 정한다.

## 초안 등록과 공개

검증된 커밋에서 `v<desktop-version>` 태그를 생성해 **HoilRyu/h-opentypeless**에 푸시한 뒤 다음을 실행한다. 스크립트는 태그를 만들거나 푸시하지 않으며, 원격 태그/SHA·깨끗한 소스·체크섬을 확인하고 Draft Pre-release만 만든다.

```sh
./scripts/h-draft-release.sh v0.1.43-beta.1 "$HOME/.local/share/h-opentypeless/releases/0.1.43-beta.1"
```

`RELEASE_NOTES.md`의 버전/지원 범위/제약을 최종 확인한 후 GitHub에서 공개한다. 게시된 태그와 패키지는 교체하지 않고 수정 사항은 새 버전으로 낸다. Windows/Linux 및 자동 업데이트는 RELEASE_PLAN.md의 후속 단계다.
