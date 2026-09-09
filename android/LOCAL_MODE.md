# H Android 로컬 모드 (0.2.0)

## 프로젝트와 앱

이 기능은 **h-opentypeless/android** 안에 독립 구현되어 있다. 패키지는 `dev.hoilryu.hopentypeless.mobile`, 앱 표시는 **H-OpenTypeless 음성 키보드**, 버전은 **0.2.0 / versionCode 2**이다. 중단된 모바일 프로젝트의 소스나 서버를 런타임/빌드에서 참조하지 않는다. JDK·SDK 설치 위치는 환경 변수로 지정하는 공용 개발 도구다.

## 사용

1. H Android 설정 → **원격 / 로컬 설정과 모델 관리**를 연다.
2. 원격 또는 로컬을 직접 선택한다. 키보드 상단에서도 전환할 수 있고 마지막 선택을 저장한다. 녹음·처리 중에는 전환을 막는다.
3. 로컬 STT는 **Android 기본 온디바이스**, **Whisper base**, **Whisper small** 중 선택한다.
4. 로컬 교정은 **끄기**, **Gemma 4 E2B Mobile GPU**, **Gemma 4 E4B Mobile GPU** 중 선택한다. STT끼리 비교할 때는 교정을 끄고 먼저 원문을 비교하는 편이 좋다.
5. 필요한 모델을 다운로드하거나 지정 배포 파일을 가져온다. 실행 확인을 누르면 실제 로딩/짧은 추론을 검사한다.
6. 마이크 권한을 허용하고 **H-OpenTypeless 음성 키보드**에서 실제 발화·직접 삽입을 확인한다.

기본 선택은 기존 **H 원격**이며 로컬 교정은 **꺼짐**이다. 원격 실패 시 자동 전환하지 않는다. 로컬 모드는 서버 주소나 연결 확인 없이 동작한다. 모델 준비가 끝난 뒤 오프라인으로 사용할 수 있으며, 모델 다운로드와 기본 STT 언어 모델 준비는 별도의 온라인 준비 단계다.

## H와의 통합

- **원격:** 기존 `Api`와 연결 확인을 유지한다. `X-H-OpenTypeless-Client: android-v1`, `audio/wav`, H `/api/health` 프로토콜 검사, H 데스크톱 STT/교정 설정을 그대로 사용한다.
- **Whisper:** H의 `PcmRecorder`가 생성한 16kHz 모노 PCM16 WAV를 사용한다. `PcmAudio`는 H의 44바이트 WAV 헤더·길이를 검증한 뒤 PCM을 꺼낸다. AAC/MediaCodec/ffmpeg 변환은 없다.
- **녹음 안정화:** 기존 `PcmRecorder`, `RecordingLease`, `Wav`의 진행 중 변경을 보존한다. 기본 STT도 같은 `RecordingLease`를 획득하므로 이전 H 녹음이 아직 정리 중이면 시작을 거부한다. 시스템 인식 서비스의 내부 마이크 정리는 Android API의 cancel/destroy 계약에 의존한다.
- **기본 STT:** Android 12+ `createOnDeviceSpeechRecognizer`만 사용한다. 온라인 인식기로 대체하지 않는다. 한국어 지원/설치 상태를 확인하고 미지원·미설치 결과는 선택 상태에 반영한다. 시스템의 발화 종료 판단에 따라 일찍 종료될 수 있다.
- **로컬 교정:** LiteRT-LM 0.17.0 GPU, Gemma 4 E2B/E4B Mobile 배포 파일, 비추론 모드, 문맥 4096·출력 1536토큰 상한. 로컬 지침은 휴대폰에 저장하며 H 데스크톱의 사전/씬/번역/교정 설정을 자동 동기화하지 않는다. 긴 입력은 문맥 상한에 걸릴 수 있다.
- **결과:** 원문/교정문 확인 후 사용자가 삽입한다. 줄바꿈·제어문자 제거, 비밀번호 차단, 입력칸/앱 전환 및 키보드 숨김 시 취소·결과 폐기 정책을 유지한다.

추론은 앱의 비공개 `:inference` 프로세스에서 순차 실행한다. Whisper 원문을 먼저 메인 프로세스에 전달해 이후 Gemma 오류나 프로세스 종료에도 원문을 보존한다. 취소·180초 제한 시 추론 전용 프로세스를 종료한다. 작업이 끝나면 프로세스도 종료하므로 모델 로딩 시간이 매 작업에 포함된다. H 데스크톱 프로세스는 조작하지 않는다.

## 모델 배포와 기기 상태

가중치는 APK에 넣지 않는다. `app/src/main/assets/models.json`에 커밋 고정 URL, 크기, SHA-256을 기록하고 HTTPS로 다운로드한다. 완료한 파일을 검증한 뒤 이름 변경으로 설치한다. 손상된 가져오기가 기존 정상 파일을 덮어쓰지 않는다. 앱 내부 `files/models`에 저장하므로 업데이트 시 유지되고 앱 데이터 삭제/제거 시 삭제된다.

| 모델 | 다운로드 크기 | 총 RAM 기준 | 시작 전 여유 메모리 기준 |
|---|---:|---:|---:|
| Whisper base | 147,951,465 bytes | 4GB | 0.6GiB |
| Whisper small | 487,601,967 bytes | 6GB | 1.0GiB |
| Gemma E2B Mobile GPU | 2,008,432,640 bytes | 6GB | 1.8GiB |
| Gemma E4B Mobile GPU | 2,969,059,328 bytes | 8GB | 2.8GiB |

총 RAM 판정은 시스템 예약분을 고려해 10% 여유를 둔다. 이 수치는 첫 배포의 보수적인 정책이며 성능 보장이 아니다. 별도 Whisper/Gemma는 Android 12+ ARM64 대상이다. APK에는 ARM32 설치용 라이브러리도 넣어 기존 원격 사용이 가능하게 하되, ARM32에서 Whisper/Gemma 선택은 비활성화한다. ARM32 실기기는 미검증이다. 기본 STT에는 LLM용 RAM 하한을 적용하지 않는다.

미지원 OS/ABI/RAM, 모델 미설치, 최근 실행 실패를 구분한다. 여유 메모리 부족과 심각한 발열은 일시 제한한다. 키보드 재표시/설정 다시 열기로 재평가하고, 실패 모델은 **실행 확인 / 재검사**로 다시 확인할 수 있다. 성능이 느리다는 이유만으로 영구 차단하지 않는다.

다운로드 취소·이어받기·파일 가져오기·삭제를 지원한다. 화면을 떠나면 다운로드를 중단하고 `.part`를 보존한다. 백그라운드 다운로드 서비스는 없다. 파일 가져오기는 지정 배포 파일과 정확한 크기·SHA-256이 일치해야 한다. 임의 GGUF/다른 변형 파일은 지원하지 않는다.

Whisper 실행 확인은 1초 무음으로 로딩과 실행만 검사한다. 정확도 검사가 아니다. 처리 기록에는 엔진, 인식/교정 시간, 추론 프로세스 샘플링 최대 PSS만 저장하고 음성·원문·교정문은 기록하지 않는다. PSS는 GPU 메모리 전체를 나타내지 않는다. 기본 STT 시간은 발화 종료 후 결과 대기 시간이며 Whisper는 WAV 읽기·모델 로딩·인식 시간으로 서로 의미가 다르다.

## 빌드와 검사

JDK 17/21, SDK 36, NDK **28.2.13676358**, CMake **3.22.1**이 필요하다. whisper.cpp **v1.8.3**은 고정 URL/SHA-256으로 받아 H의 외부 빌드 디렉터리에 보관한다. debug APK에서도 네이티브 추론은 Release 최적화를 사용한다.

```sh
JAVA_HOME=<JDK 경로> ANDROID_HOME=<SDK 경로> android/build.sh assembleDebug assembleDebugAndroidTest testDebugUnitTest lintDebug
```

APK: `~/.local/share/h-opentypeless/android-build/app/outputs/apk/debug/app-debug.apk`

실기기 검사는 별도 AndroidTest APK의 `dev.hoilryu.hopentypeless.mobile.SmokeInstrumentation`으로 수행한다. 마이크를 열지 않고 기능 지원 조회, H WAV 합성 음성 인식, 교정, 무결성, 취소, H 녹음 lease와의 충돌 차단, View 렌더링을 검사한다. 테스트 입력은 앱 내부 캐시로 준비하며 디버그 테스트 APK는 제품 APK에 포함되지 않는다.

빌드와 lint 작업은 완료되었으나 AGP 8.13.2의 lint Kotlin 메타데이터 리더가 LiteRT-LM의 Kotlin 2.4 의존성에 대해 호환 버전 진단을 출력한다. 앱 소스는 Java이고 Java 컴파일/JVM 테스트/실기기 추론은 별도로 확인한다. 이 진단을 숨기려고 라이브러리 버전을 강제로 낮추지 않았다.

소스 기준: [Android SpeechRecognizer](https://developer.android.com/reference/android/speech/SpeechRecognizer), [whisper.cpp v1.8.3](https://github.com/ggml-org/whisper.cpp/tree/v1.8.3), [LiteRT-LM 0.17.0](https://github.com/google-ai-edge/LiteRT-LM/tree/v0.17.0), [Gemma 모바일 모델](https://ai.google.dev/gemma/docs/core).


## H 앱 실기기 검증 — 2026-09-09

기기: S26 Ultra SM-S948N, Android API 36, RAM 12GB 모델. 아래 결과는 모두 **H 패키지**에서 새로 검사한 결과다.

- H APK 0.2.0 / versionCode 2 업데이트 설치 성공. 기존 H 설정과 마이크 권한 유지.
- Android JVM 테스트 **22개**, failures/errors 0. H WAV/HTTP 계약, 녹음 lease, PCM 검증, 모델 기기 조건, 이어받기 범위 포함.
- 기본 온디바이스 STT 서비스 사용 가능, 설치된 언어 `[ko-KR]` 확인.
- 기본 STT가 기존 H 녹음 lease를 우회하지 못하고 다른 녹음의 lease를 잘못 해제하지 않는 실기기 검사 통과. 마이크를 열지 않은 충돌 검사다.
- H 앱 다운로드 코드로 Whisper base를 받아 SHA-256 검증 및 H 앱 내부 저장 성공.
- Gemma E2B는 H 호스트 캐시에서 SHA-256 검증 후 H 앱 내부로 전송하고 기기에서 다시 SHA-256 일치 확인.
- **Whisper base:** 약 8.36초 한국어 합성 음성을 H 표준 WAV로 입력해 약 **2.55초**에 처리. 샘플링 최대 PSS 약 **251MiB**. `원격`을 `원경`으로 인식하는 오류는 남아 있다.
- **Gemma E2B:** 짧은 한국어 교정 약 **7.43초**(모델 로딩 포함), 샘플링 최대 PSS 약 **1,715MiB**. `어 그럼 내일 오전 열 시에 회의를 시작하면 될 것 같아요.` → `그럼 내일 오전 열 시에 회의를 시작하면 될 것 같아요.`
- 잘못된 크기/해시의 가져오기 거부와 기존 정상 파일 보존, 로컬 교정 실패 시 원문 보존, 추론 취소 후 H 전용 프로세스 종료: 통과.
- H 로컬 설정 Activity 실행 및 설정·키보드 View 렌더링 검사: 통과. 잠금 화면과 무관하게 앱 자신의 View를 그리는 검사이며, 다른 앱 입력칸의 실제 터치/배치 검증을 대체하지 않는다.
- 작업 시작 시점과 비교하여 기존 `PcmRecorder`, `RecordingLease`, `Wav`, `Api`의 내용이 동일함을 SHA-256으로 확인. H 데스크톱 변경 파일도 이번 작업에서 수정하지 않았다.

로그와 렌더링 결과: `~/.local/share/h-opentypeless/verification/local-android/`.

남은 실사용 검증: 실제 마이크 발화, 비행기 모드, 2분/연속 사용 및 발열, Whisper small/Gemma E4B, ARM32 실제 기기, 다른 앱 입력칸 위에서의 최종 동작. 속도 수치는 합성 예문 한 건으로 얻은 관측값이며 정확도나 모든 기기의 성능 보장이 아니다.

### 받아쓰기 충실도 보강 (2026-09-09)

로컬 문체 설정 뒤에 고정 교정 규칙을 적용하고 발화를 교정 대상 경계 안에 전달한다. 원문의 질문·요청을 실행하지 않는 예시를 추가했다. 빈 결과·추론 태그·과도한 내용 확장은 기존 오류 처리로 원문을 유지한다. 이 검사는 완전한 의미 판별기가 아니다. S26 Ultra E2B에서 기존 답변 생성 사례의 수정 전후와 추가 5건을 확인했다. 상세 기준과 결과: [DICTATION_FIDELITY.md](../docs/fork/DICTATION_FIDELITY.md).

### 기본 STT 수동 종료 (2026-09-09)

기본 온디바이스 STT는 발화 구간마다 나온 최종 결과를 누적하고, 종료 버튼을 누르기 전까지 새 인식 구간을 시작한다. 침묵에 따른 NO_MATCH/SPEECH_TIMEOUT도 다시 듣는다. 이 경로의 119초 타이머는 제거했다. 이전 구간의 늦은 콜백은 토큰으로 무시하고, 의도적으로 같은 말을 반복한 경우는 보존한다. 종료 버튼 이후 최종 결과가 5초 내에 오지 않으면 마지막 부분 인식과 이미 누적된 원문을 사용한다. 취소·입력창 이탈 후에는 다시 시작하지 않는다. 권한/서비스 오류가 발생하면 자동 교정하지 않고 원문을 남긴다.

Android 인식기는 내부적으로 구간을 종료하므로 다시 듣는 사이에 짧은 공백이 생길 수 있다. 결과/오류 콜백을 받은 뒤 재시작한다([Android API 규약](https://developer.android.com/reference/android/speech/SpeechRecognizer)). 이 변경은 기본 STT 경로에 적용된다. Whisper·원격의 WAV 녹음 경로에는 기존 119초 제한이 남아 있다.

검사: 연속 구간·침묵 100회, 구간 사이 종료, 종료 후 늦은 결과, 부분 결과 보존, 취소·오류, 반복 발화 보존을 가짜 인식기 드라이버로 검증한다. 실제 목소리의 긴 침묵과 서비스 재시작 사이 유실 여부는 별도 실사용 확인이 필요하다.
