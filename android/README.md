# H-OpenTypeless Android 키보드

H 데스크톱 앱의 모바일 연결에 사용하는 독립 Android IME다. 기존 mobile-opentypeless 저장소나 Python 서버를 런타임/빌드에서 참조하지 않는다.

1. H 앱에서 STT/AI 제공자를 설정하고 설정 → 일반 → Android 연결을 켠다.
2. 이 앱에 H가 표시한 내부 주소를 저장하고 연결 확인을 누른다.
3. 마이크 권한을 허용하고 시스템에서 H-OpenTypeless 음성 키보드를 활성화/선택한다.
4. 시험 입력칸에서 녹음 후 결과를 확인하고 삽입한다.

연결키는 없다. 내부 Wi-Fi/WireGuard를 사용한다. 전체 API/지원 제공자는 `../docs/fork/MOBILE_API.md`를 참고한다.

## 빌드

JDK17 또는21, Android SDK Platform36/Build Tools35.0.0이 필요하다. `JAVA_HOME`과 `ANDROID_HOME`을 설치 위치로 지정한 후 실행한다.

```sh
./build.sh assembleDebug testDebugUnitTest lintDebug
```

Gradle wrapper8.13의 체크섬을 고정했다. Gradle 캐시는 ~/.local/share/h-opentypeless/android-tools, 빌드 결과는 ~/.local/share/h-opentypeless/android-build에 저장한다. APK는 app/outputs/apk/debug/app-debug.apk이다. debug 서명 APK는 개인 검증용이며 정식 Android 배포 서명은 별도 작업이다. Mac 자체 서명 인증서를 Android 서명에 사용하지 않는다.

이 Mac에서 기존에 설치된 JDK/SDK 도구를 환경 변수로 가리킬 수 있다. 해당 도구 설치 위치는 소스 코드 의존성이 아니며 다른 컴퓨터에서는 그 컴퓨터의 표준 JDK/SDK를 지정한다.
