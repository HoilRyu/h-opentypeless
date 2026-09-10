# 내장 STT

설정 → 음성 인식 → **내장 STT · Whisper / Qwen**에서 **STT 모델 선택 상자**로 모델을 고르고 다운로드한 뒤 **이 모델 사용**을 누른다. 기존 외부 STT 주소·모델·API 키와 LLM API/Ollama 설정은 유지된다. 내장 STT로 바꿔도 LLM 다듬기에 필요한 네트워크/모델 엔진은 별도다.

| 모델 | 다운로드 | 권장 시스템 메모리 |
|---|---:|---:|
| Whisper Tiny | 약 74 MiB | 4 GB |
| Whisper Base | 약 141 MiB | 4 GB |
| Whisper Small | 약 465 MiB | 8 GB |
| Whisper Large-v3 Turbo | 약 1.51 GiB | 16 GB |
| Qwen3-ASR 0.6B | 약 1.75 GiB | 8 GB |
| Qwen3-ASR 1.7B | 약 4.38 GiB | 16 GB |

권장 메모리는 H의 보수적인 선택 가이드이며 속도·정확도 보장이 아니다. Whisper는 모두 다국어 모델이다. 한국어 정확도는 모델 크기와 발음/환경에 따라 다르다. 저사양 PC는 Base부터 검토한다. Whisper와 Qwen은 실행 엔진을 자동/MLX GPU/CPU 중 선택할 수 있다. 호환되는 Apple Silicon Mac에서는 자동 모드가 내장 MLX GPU를 사용한다. 최초 모델 검증·준비는 반복 입력보다 오래 걸릴 수 있다.

## 구조와 제한

- H 소유 구현: `src-tauri/src/extensions/local_stt`; 기존 STT trait/factory에 하나의 제공자만 추가한다.
- 엔진은 앱의 `local-stt` 리소스에 포함한다. 사용자가 Python·추가 서버를 설치하거나 실행할 필요가 없다.
- 모델은 app data의 `local-stt/<model-id>`에 저장한다. 앱 업데이트 시 보존한다.
- 모델별 고정 revision, 크기, SHA-256은 `catalog.json`에 있다. 임의 URL/경로를 IPC로 받지 않는다.
- `.part` 이어받기, Range 응답 검증, 완료 체크섬 검증 후 이름 변경. 중단된 다운로드는 사용자가 재개한다. 자동 대용량 다운로드는 없다.
- 앱 세션의 최초 실행 전 전체 체크섬을 검사하고 이후에는 파일 식별자·크기·변경 시각으로 검증 결과를 재사용한다. 변경된 파일은 재검증하며 모델 메모리 해제로 캐시를 지울 수 있다. 모델 삭제/선택/다운로드와 녹음/전사는 단일 lease로 충돌을 막는다.
- 최대 녹음 120초, PCM16 mono 16kHz, 버퍼 3.84 MB, CPU 최대 4개 스레드, Qwen 세그먼트 20초. 전사 timeout 90초.
- CPU 엔진에는 stdin으로 WAV를 전달한다. MLX는 전용 Python 워커에 길이가 제한된 헤더와 PCM을 전달하며 HTTP 포트를 열지 않는다. 음성 임시 파일을 쓰지 않고 출력은 64 KiB로 제한한다.
- MLX는 녹음 중 준비하고 모델 하나를 재사용한다. 약 2분 유휴 또는 메모리 압박 시 해제하며, 취소/timeout/future drop 시 프로세스를 종료한다. 앱 종료 및 부모 프로세스 사망도 감시한다. CPU 엔진은 요청마다 종료한다.
- 일반 입력·Ask·모바일 서버가 같은 제공자를 쓴다. Ask의 내장 모델 최종 전사 대기만 95초로 조정한다. 외부 제공자의 대기 설정은 유지한다.
- Windows에서는 Qwen을 비활성화한다. POSIX 기반 Qwen 엔진의 Windows 포팅/검증 전까지 지원한다고 표시하지 않는다.
- MLX 번들은 네이티브 Apple Silicon과 macOS 14 이상을 대상으로 하며 실제 GPU 연산 검사를 통과해야 사용한다. 실패를 숨겨 CPU로 전환하지 않으며 사용자가 설정에서 CPU를 선택할 수 있다.
- macOS CPU Qwen은 Accelerate ABI 때문에 13.3 이상, 번들 Whisper는 11 이상을 기준으로 빌드한다. Windows/Linux 실기기 검증은 추후 수행한다.

## 모델 선택과 비교

선택 상자는 모델의 정보와 다운로드/사용 버튼을 보여 준다. 둘러보기만으로 다운로드하거나 활성 모델을 바꾸지 않는다. 다운로드 중 다른 항목을 확인해도 ‘진행 보기’로 돌아갈 수 있다.

원본 OpenTypeless README의 추천 예는 [Groq의 whisper-large-v3-turbo](https://github.com/tover0314-w/opentypeless)다. H의 내장 목록에는 기존 Tiny/Base/Small에 이어 동일 모델 계열의 **Whisper Large-v3 Turbo**를 추가했다. [whisper.cpp 모델 배포](https://huggingface.co/ggerganov/whisper.cpp/tree/5359861c739e955e79d9a303bcbc70fb988958b1)의 고정 revision, 파일 크기(1,624,555,275 bytes), SHA-256을 사용한다. 자동 다운로드하지 않는다.

호환 Apple Silicon Mac에서는 Whisper와 Qwen 모두 MLX GPU를 사용한다. Whisper는 기존에 검증한 GGML F16/F32 가중치를 메모리에서 읽으며 별도 모델 다운로드나 변환 파일을 만들지 않는다. CPU 선택 및 MLX 미지원 환경에서는 기존 CPU 엔진을 사용한다. 따라서 Turbo가 현재 Qwen보다 빠르거나 한국어 인식이 더 좋다고 단정할 수 없다. 표의 권장 RAM은 앱에서 정한 보수적인 안내이며, 개발사 최소 요구 사양이 아니다. 같은 녹음을 AI 다듬기 전 전사문 기준으로 비교해야 STT 자체의 오류를 판단할 수 있다. 긴 음성은 기존 90초 전사 제한에 걸릴 수 있다.

개인 사전은 최종 LLM 문맥 교정을 돕는 보조 수단이다. 이번 변경은 STT를 재학습하거나 모든 오인식을 자동 교정하는 기능은 아니다.

## 빌드

macOS는 기존 `scripts/h-build-macos.sh`를 사용한다. 이 스크립트가 `h-prepare-local-stt.py`를 호출해 고정된 엔진 소스를 외부 캐시에 빌드하고 Tauri resource mapping을 생성한다. 모델 가중치는 패키지에 넣지 않는다. macOS arm64에서는 `h-prepare-mlx.py`가 해시로 고정된 재배치 가능 Python 3.12.14, MLX 0.31.1, mlx-qwen3-asr 0.4.0을 포함한다. credential helper는 기존 서명을 보존하고 MLX 라이브러리/실행 파일을 포함한 STT 코드를 별도로 서명한 뒤 앱을 서명한다.

Windows/Linux 개발 빌드 준비:

```sh
python3 scripts/h-prepare-local-stt.py --config /absolute/external/build/local-stt-bundle.json
npm run tauri build -- --config /absolute/external/build/local-stt-bundle.json
```

개발 의존성: Git, CMake, C/C++ compiler, Python(빌드 도구; macOS arm64 배포본은 별도의 전용 Python 런타임도 포함). Linux Qwen은 OpenBLAS 개발 패키지가 필요하다. Linux 배포 전 AppImage 내부에서 OpenBLAS 포함 여부/동적 라이브러리 경로를 확인해야 한다. Windows는 MSVC/Windows SDK로 Whisper를 빌드한다. 검증되지 않은 OS 패키지를 배포하지 않는다.

## 검증과 다음 작업

네트워크 다운로드와 실제 모델 전사 테스트는 평소 테스트에서 제외한다. 명시적으로 외부 테스트 디렉터리와 모델/음성 경로를 지정할 때만 실행한다. 사용자 모델 가중치와 설정을 보존한다. MLX용 고정 tokenizer 설정만 기존 모델 폴더에 추가한다.

- 모델 크기/체크섬, Range 이어받기와 무시된 Range, 손상 파일 거부
- 출력/PCM 제한, 녹음 루프 대기, 취소 후 프로세스 종료
- 다운로드 전 사용자 동작 필요, 다운로드 중 일시 정지, 엔진 미지원 표시
- 한국어 합성 음성으로 Whisper Base, Qwen 0.6B/1.7B 실제 엔진 검증

**이 작업 완료 후 사용자에게 튜토리얼/첫 실행 안내를 다음 작업으로 제안한다. 이번 작업에서는 튜토리얼을 구현하지 않는다.**
