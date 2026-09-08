# H Android 연결

## 실행 구성

H 데스크톱 앱의 Rust 모듈 `src-tauri/src/extensions/mobile`이 HTTP 연결을 직접 받는다. 별도 Python 게이트웨이, ffmpeg, 프롬프트 프록시는 필요 없다. Qwen ASR·Ollama 등 사용자가 선택한 모델 엔진은 별도로 실행한다.

설정 → 일반 → Android 연결에서 컴퓨터의 특정 내부 IPv4와 포트(기본8787)를 선택하고 연결을 켠다. 초기값은 꺼짐이다. 켠 상태로 앱을 종료하면 다음 앱 실행 때 같은 주소로 재시도한다. IP가 바뀌었거나 포트가 사용 중이면 설정에 오류를 표시한다. `0.0.0.0`과 공인 IP는 바인딩하지 않는다. 연결키 없는 내부 Wi-Fi/WireGuard 정책을 유지한다. 같은 주소에 접근 가능한 기기가 요청할 수 있다.

## API v1

- `GET /api/health`: `{status:"ok",service:"h-opentypeless",protocol:1,audio:"wav-pcm16-16000-mono"}`. 서버 생존 확인이며 모델 엔진 상태 확인은 아니다.
- `POST /api/dictate`: multipart의 `file` 필드 하나. `X-H-OpenTypeless-Client: android-v1` 헤더 필수. WAV PCM signed16 little-endian, mono,16000Hz,최대120초. 이름/경로는 서버 저장에 사용하지 않는다.
- 성공: `{raw_text,polished_text,warning}`. 교정을 끄면 두 텍스트가 같다. 교정 실패 시 원문과 경고를 반환한다. STT 실패는 HTTP502와 `{detail}`.
- 잘못된 전송400, 형식415, 동시요청429, 연결비활성화503, 처리시간초과504. 본문 최대3,848,192바이트(PCM3,840,000+8192), 업로드30초, 처리300초 제한. 앱 전체에서 모바일 요청1개만 처리한다.
- 연결 끄기/주소변경 시 이전 리스너의 진행 중 처리를 취소한다. 클라이언트의 연결 종료만으로 이미 모델에 전달된 계산이 반드시 중단되는 것은 아니다. Android는 취소 이후 응답을 무시한다.
- Origin이 있는 요청을 거부하고 CORS 허용을 제공하지 않는다. 고정 클라이언트 헤더는 브라우저의 일반 폼 제출을 막기 위한 프로토콜 표식이며 비밀키/인증은 아니다.

## 음성·교정 재사용 범위

현재 모바일 STT는 Local / Custom Whisper(Qwen의 Whisper 호환 API 포함), OpenAI Whisper, Groq Whisper, SiliconFlow, GLM-ASR이다. 기존 `stt::create_provider`와 파일 전송 처리를 사용한다. 실시간 WebSocket STT, Apple Speech, 관리형 Cloud STT는 이번 모바일 입력에 아직 지원하지 않는다. 지원하지 않는 설정은 명시적 오류로 반환한다.

LLM은 기존 `llm::create_provider` 및 `PolishRequest`를 사용한다. Ollama와 직접 API 제공자의 앱 설정, Custom Polish Instructions, 교정 스타일, 사전, 교정 규칙, 활성 씬, 번역 설정을 재사용한다. Cloud LLM은 이번 모바일 경로에서 미지원이며 원문과 안내를 반환한다. 별도의 시스템 프롬프트를 새로 만들지 않는다.

모바일 문맥은 일반 텍스트 문맥이다. Mac의 현재 포커스, 선택 텍스트, 앱별 매핑을 가져오거나 키보드/클립보드를 조작하지 않는다. 서버는 모바일 음성 파일과 결과를 디스크/데스크톱 기록에 저장하지 않는다.

## Android

`android/`는 과거 mobile-opentypeless 구현에서 필요한 소스만 이관했다. 패키지 `dev.hoilryu.hopentypeless.mobile`로 기존 앱과 독립 설치된다. H 앱에 표시된 주소를 Android 설정에 저장한다. 연결 확인은 H 프로토콜 버전을 검사한다.

AudioRecord로16kHz PCM WAV를 최대119초 녹음한다. 원문/교정문 확인 후 사용자가 삽입한다. 편집키 좌우/Backspace 반복, 비밀번호 입력칸 제한, 입력칸 변경 시 녹음/결과 폐기, 늦은 응답 차단, 결과의 줄바꿈/제어문자 제거를 유지했다. Python/AAC 변환 경로는 사용하지 않는다.

## 검사와 남은 범위

API 계약/잘못된 WAV/중복 요청/연결끄기 취소에 Rust 테스트가 있다. Android WAV 헤더, HTTP 계약, 취소/리다이렉트, 편집키/반복 수명주기 테스트가 있다. UI는 IP/포트 저장·오류 표시를 검증한다.

실제 사용 중인 Mac/Qwen/Ollama 통합, Android 실기기 녹음/입력, Windows/Linux는 각각 별도로 결과를 기록한다. Windows/Linux 실제 검증은 사용자 결정대로 마지막 단계다.

키체인 인증 정보는 별도 blocking 작업으로 읽고20초 안에 승인되지 않으면 안내 오류를 반환한다. 네이티브 승인 창이 HTTP 요청보다 오래 남아도 추가 키체인 작업을 무한히 쌓지 않도록1개로 제한한다. STT 업로드 응답은100초, LLM은150초 제한이며 실패 시 위 정책을 따른다.
