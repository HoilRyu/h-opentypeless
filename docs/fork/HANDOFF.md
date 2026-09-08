# H-OpenTypeless 인계

2026-09-08. 작업 소스: /Users/ryuhoil/syncthing/workspace/h-opentypeless.
기능 브랜치: feat/h-foundation-direct-providers. origin HoilRyu/h-opentypeless, upstream tover0314-w/opentypeless. 기반 v1.1.57(68cf6f11). 아직 커밋·푸시하지 않음.

## 현재 기능

- H 이름·bundle identifier(dev.hoilryu.hopentypeless)·Keychain·deep-link 분리. 공식 자동 업데이트 연결 제거.
- Qwen ASR8765와 Ollama11434 직접 연결. Ollama에만 reasoning_effort=none 추가. 공식 프롬프트 생성 경로 유지.
- 단축키 Ctrl+Shift+Space, 토글 모드. 실제 마이크 입력 사용자 확인.
- macOS 창 닫기 시 Dock에서 숨기고 메뉴 막대 상주. 다시 열면 Foreground 복귀 확인.
- 사용자 결정: 자동 포커스/커서 복원 폐기. 다른 앱으로 이동한 상태에서 결과가 나오면 자동 입력 대신 복사 창 표시.
- extensions/input_target 전체 삭제. pipeline/app_detector의 캡처 참조·AX 복원·Electron 접근성 활성화·재시도 모두 제거. app_detector 두 파일은 원본으로 복원.
- 일반 교정 결과, 교정 없는 직접 출력, 스트리밍 실패 복구에 복사 창 연결. 종료 시 다른 앱의 선택 텍스트를 읽지 않도록 가드 추가. Ask의 원본 동작은 유지.

## 실행/빌드

실행 앱: /Applications/H-OpenTypeless.app. 빌드 후 이 사본을 교체해야 한다.
빌드: scripts/h-build-macos.sh. 외부 mirror ~/.local/share/h-opentypeless/build-source 사용. 소스는 H 저장소만 수정.
설정: ~/Library/Application Support/dev.hoilryu.hopentypeless/settings.json. 개인 키/지침은 저장소에 저장하지 않음. hotkeys.dictationBindings는 camelCase.
서명 없는 개발 빌드이므로 교체 후 접근성 배너가 나타나면 사용자 허용 필요.

## 검증 및 남은 일

- 앞 단계 프런트엔드449개, Rust565개 및 실제 STT/LLM 통합 테스트 통과.
- 정리 빌드 성공, Rust566개 전체 테스트 및 git diff --check 통과. 자동 복원을 호출하지 않고 복사+팝업만 실행하는 회귀 테스트 포함. 로그 cleanup-build.log, cleanup-tests.log.
- 자동 복원 실험은 메모장에서는 성공했지만 대화 앱에서 AX 입력칸을 제공받지 못해 실패했고 사용자 요청으로 폐기했다. 재도입하지 말 것.
- 포커스 판단은 원본 앱/프로세스 가드를 사용한다. 동일 앱 내 다른 입력칸이나 떠났다가 돌아온 이력을 추적하지 않는다.
- Windows/Linux 지원 구조 유지, 실제 OS 검증은 사용자 요청으로 마지막 단계. Android 통합은 아직 미구현.

## 외부 구성

순정 OpenTypeless 앱·데이터 및 옛 Python 게이트웨이/추론 프록시는 제거/종료했다. 모델 엔진 Qwen ASR와 Ollama는 유지한다. ~/.local/share/opentypeless-local에 남은 것은 엔진 런타임이다. 구 mobile-opentypeless 저장소는 참고용이며 H의 런타임 의존성이 아니다.

정리 버전을 /Applications에 설치·실행했다. 이번 CUA 실행에서는 접근성 필요 배너가 다시 나타나 실제 키보드 입력은 사용자 재허용 대기. 소스에서 input_target 참조와 AX 복원 코드가 제거됐으며 app_detector 두 파일은 git diff 없음으로 확인.

정리 후 대화 앱의 미이동 입력 실패 확인: history48~50 OutputFailed. H UI는 접근성 권한 없음/클립보드 복사를 명시. 시스템 설정의 H 토글은 on인데 실행 앱은 미허용으로 판단하여 빌드 교체 후 등록 불일치가 의심됨. 설정 화면을 열었으며 사용자에게 H 항목 제거 후 /Applications 사본 재등록 안내. 이 실패 때문에 포커스 검사 코드를 다시 수정하지 않음.

## 녹음 중 음량 조절 추가
사용자 계획에 따라 extensions/audio_ducking으로 별도 구현. 설정 → 일반 → 녹음 중 다른 소리, 기본Off/20%감소/음소거. 일반 받아쓰기·번역·Ask 마이크 생명주기에 연결. 출력 음량만 제어하며 원본 프롬프트/모델 경로는 보존. 수동 음량 변경을 속성별로 존중, 장치 교체/취소/정상종료 복원, 비정상 종료 후 조건부 복구 기록. AUDIO_DUCKING.md에 구현/플랫폼 한계 정리. Windows/Linux 실제 검증은 마지막으로 연기한 상태 유지.
프런트엔드451개 및 lint/prettier/build 통과. 공통 엔진11개와 Mac 실제 output reduce/mute/restore1개 총12개 통과. Mac 실장치 확인은 명시적 opt-in 테스트로 수행했으며 일반 자동테스트에서는 ignored이다. 음악 재생 중 실제 발화와 물리 장치 교체를 검증한 것으로 보고하지 않는다.

음량 기능 최종 검증: Rust577개 통과 + 실제 장치 검사1개는 기본실행 ignored(별도 실행 성공). frontend451개, clippy -D warnings, lint/prettier, git diff --check, 최종 Mac build 통과. /Applications/H-OpenTypeless.app 설치/실행 및 설정 UI 확인. 소리 줄이기 선택→별도 설정 JSON mode=reduce 저장 확인→끄기로 되돌려 최종off 확인. 이번 빌드에서 접근성 권한 배너가 나타나 실제 발화/입력 검사 전 사용자 권한 재등록 필요. 설정 UI와 native 음량 테스트는 이 권한과 별개로 확인 완료. 외부 로그 duck-final-tests.log / duck-native-tests.log / duck-ui-tests.log / duck-clippy.log / duck-final-build.log.

## 사용자 지정 음량 비율
사용자 요청으로 고정20%를 volume_percent(0~100, 기본20) 설정으로 변경. 일반 설정에서 소리 줄이기 선택 시 슬라이더·숫자 입력 제공. 기존 비율 없는 설정은20% 유지. 다음 녹음 시작 시 비율을 고정하여 진행 중 녹음/장치 전환에 동일 비율 사용. 백엔드101이상 거부, UI범위 검증. frontend452개/lint 통과. Rust 및 native35% 검증 결과는 duck-percent-tests.log 확인.

비율 변경 검증: frontend452개/lint 통과, audio_ducking15개(실제 Mac35% 및 음소거 복원 포함) 통과. 최초 native 검사에서 즉시 읽은 값 불일치가 한 차례 있었고 진단값을 추가한 후 재실행 및 추가5회는 모두 통과. 최초 실패 원인은 확정하지 않았으며 실제 장치 상태/비동기 반영 가능성은 남아 있다. 로그 duck-percent-tests.log, duck-percent-native-repeat.log.

비율 설정 Mac 빌드 성공 및 /Applications 설치 완료(duck-percent-build.log). CUA 일반 설정에서45 입력/Tab 후 h-audio-ducking.json mode=reduce,volume_percent=45 저장 확인. 기존20으로 되돌리려던 중 사용자 앱 변경으로 CUA가 중단했으며 최신 창은 녹음 Capsule이므로 추가 조작 중단. 최종45는 UI검사용 값임을 사용자에게 알림. 실행 시 접근성 권한 배너가 다시 표시됨.

## 처리 중 창 이동 후 복사창 미표시 조사
최근 history69에 TargetChanged / clipboard_fallback 기록 확인(음성 내용 읽지 않음). AskPanel은 native focus-loss 때 결과를 지우고 hide하므로 결과 창 표시와 늦은 포커스 이벤트가 겹치면 사라질 수 있다. 자동 focus-loss dismiss 제거, Esc/명시적 닫기는 유지. Ask 결과창을 모든 workspace에 표시하도록 설정(캡슐과 동일), show/unminimize 실패를 전달하고 voice popup 오류를 로그에 남기도록 보완. 프런트 회귀 테스트는 결과 이벤트 전후 focus-loss에도 결과 유지 및 Esc로 닫기 검증19개 통과. 실제 사용자 발화 시나리오 재현/해결 확정은 아직 아님. MACOS_SIGNING_PLAN.md에 권한 유지 서명 계획 작성. 현재 인증서0개, 서명은 변경하지 않음.
복사창 보완: AskPanel19개 테스트/lint/Mac 빌드 성공(copy-popup-*.log). /Applications 설치·실행 완료. 설치 직후 접근성 필요 배너 표시, 실제 마이크→생각 중→창 전환 재검증은 사용자 확인 필요. 서명 인증서는 계획 단계이므로 이번 설치도 임시 서명이다. 사용자 지정 음량5%는 변경하지 않음.

## 무료 Mac 자체 서명 적용
H-OpenTypeless Local Code Signing 인증서와 개인키를 로그인 키체인에 등록. 지문83267F96432D2D0CED9C132F66750B97118C3968, 공개 인증서/identity.sha1은 ~/.local/share/h-opentypeless/signing에만 저장. 개인키 임시 파일 삭제, 시스템 trust 변경 없음. h-sign-macos.sh 추가, h-build-macos.sh 기본 self-signed/명시적 adhoc 모드 적용. 앱 번들 서명/strict 검증/DR 검사 통과 및 /Applications 설치·실행. 사용자에게 접근성 재등록 요청한 상태. 다음 코드 변경 빌드 간 권한 유지 검증, 실제 다운로드 설치 검증, 개인키 암호화 백업은 미완료. 인증서와 키를 재생성하지 말 것. Git SSH origin/upstream은 이전 턴에 확인 완료. 이번 소스 변경은 아직 미커밋.

자체 서명 업데이트 검증 완료(이 Mac): 사용자가 버전 A 접근성 권한 재등록 후 배너가 사라짐을 확인. extensions/mod.rs에 설정 파일 로드 실패 진단 로그를 추가한 버전 B를 h-build-macos.sh 전체 경로로 빌드/서명. 실행 파일 SHA256이 A와 다르고 designated requirement가 동일함을 확인. /Applications에 B를 교체·실행한 뒤 권한 재등록 없이 접근성 배너 없음. 로그 self-signed-build-b.log. 실제 발화/텍스트 입력의 사용자 재확인과 다른 Mac 다운로드 설치는 별도이며, 이번 결과로 모든 OS/장치의 권한 유지를 보장하지 않는다.

사용자 최종 확인: 자체 서명 업데이트 후 음성 인식이 정상 작동한다고 보고했다. 해당 확인을 기록하고 자체 서명 구현/문서 커밋·SSH 푸시를 요청받았다. 다음 작업 순서는 NEXT_STEPS.md에 정리했다. 처리 중 창 이동 복사창 재검증과 다른 Mac 다운로드 설치는 별도 미완료다.

## 한국어 교정 지침 완화
사용자가 어색한 문장도 자연스럽게 고쳐 달라고 요청하여 Custom polish instructions를 새로 작성하고 앱 UI에서 저장했다. docs/fork/prompts/KOREAN_POLISH.txt가 복사 가능한 지침 사본이며 런타임 원본은 앱 설정이다. 동사/어미/어순 무조건 보존 제약을 제거하고 명확한 인식 오류와 문장 구조 교정을 허용했다. 의미·말투·숫자·조건·부정 표현 보존, 질문 답변/명령 실행 금지는 유지. 프롬프트만 바꿨으며 앱 재빌드/서명/권한 변경 없음.
기존 Rust build_context_system_prompt(professional, general context, 선택/번역/씬 없음)와 기존/신규 지침을 조합하여 실제 Gemma4:12b reasoning_effort=none, temperature0.3로 예문5개씩 비교했다. 새 지침은 커미터 푸시→커밋과 푸시/알려줄→알려줘, 키보드 가림 문장 정리에 성공. 숫자·부정·작업 순서 예문3개 보존. 첫 예문은 지침 내 예시이므로 독립 일반화 검증으로 간주하지 않는다. 현재 앱별 ChatGPT 문맥과 실제 녹음 전체 경로를 그대로 재현한 테스트는 아니며 사용 중 추가 확인 필요. 테스트 자료는 ~/.local/share/h-opentypeless/prompt-eval에만 보관. 앱 저장값과 문서 지침 일치 확인.

## H Android 통합 구현 진행
이전 프롬프트 문서는2982ab1로 커밋·SSH푸시 후 작업 시작. H 안에 extensions/mobile(Axum) API 추가, 일반 설정 Android 연결 카드, h-mobile.json 별도설정/명시적내부IP/기본off/앱재시작재연결. 기존 STT Whisper-compatible 및 LLM 직접provider/공식 prompt builder 재사용. 상세 지원범위는 MOBILE_API.md. Android는 dev.hoilryu.hopentypeless.mobile로 독립 이관, PCM WAV 녹음으로 변경하여 Python/ffmpeg 불필요. 기존 편집키·반복·입력칸변경취소 유지.
프런트454개, Rust584개(+native audio1ignored), Android14개, Android lint/build 및 Rust clippy 통과. 모바일 API4개에는 WAV 형식/업로드/브라우저요청거부/중복429/연결끄기취소 검증 포함. 최초 Mac 통합 빌드를 자체서명/설치, 접근성배너없음. UI에서192.168.0.123:8787 켬, H 프로세스가 정확한IP로 리슨하고 health 응답 확인.
실제 합성음성 업로드는 키체인 암호 조회에서 대기. process sample로 keyring→SecKeychainFindGenericPassword 대기 확인, 사용자에게 H 인증정보 접근 승인 창 확인 요청. 기존 앱의 동기 keychain읽기를 모바일에 그대로 사용하면 timeout취소를 막는 문제 발견: 모바일에서는 spawn_blocking+20초timeout+별도세마포어(승인대기작업1개)로 보완 중. 키나 보안설정을 우회해 읽지 않았으며 STT/LLM 실제 통합 성공으로 아직 보고하지 않음.
ADB devices/mdns 비어있고 과거192.168.0.117:46235는 connection refused. 사용자에게 현재 무선디버깅IP/포트 요청했으나 아직 답변 없음. APK는 ~/.local/share/h-opentypeless/android-build/app/outputs/apk/debug/app-debug.apk. JDK/SDK는 기존에 설치된 ~/.local/share/mobile-opentypeless/android-tools의 도구를 환경변수로 사용했으며 소스 의존성이 아님. H 소스에는 SDK/캐시/키/기존설정 이관없음.

모바일 최신 수정: keychain spawn_blocking/20초 제한/동시 승인대기1개와 STT100초 제한 적용 완료. 모바일 Rust4개 재통과 후 자체 서명 빌드·/Applications 교체 완료. 재시작 후 저장된192.168.0.123:8787에서 health200 확인. 합성 WAV 실제 업로드는 HTTP502와 키체인 허용 안내를 반환하여 무한대기 보완 확인. STT/LLM 성공과 Android 설치는 사용자 승인/현재 ADB 주소 대기이며 미검증. 이번 모바일 변경은 아직 미커밋.

## Android 설치 및 복구 검증 완료 (2026-09-08)
사용자가 새 H Android 앱의 정상 음성 입력을 확인했다. 이번 검증에서 Mac H 앱 종료 시 API 연결 거부, 재실행 후 설정 유지/health200 및 합성 WAV→Qwen STT→Gemma4 교정 응답(warning=null)을 확인했다. 이전 키체인 대기는 현재 재현되지 않았다.
실제 연결된 Android에서 연결 확인 성공 → Mac 앱 종료 → 연결 실패 안내 → Mac 재실행 → Android 재시작 없이 연결 확인 성공을 확인했다. 녹음 시작→취소 안내/준비 상태 복귀, 다시 녹음 시작→시험 입력칸에서 서버 주소 입력칸으로 직접 이동→준비 상태 복귀 및 녹음 임시 WAV 삭제 확인. 입력값 변경이나 자동 삽입 없음. 키보드 숨김 후 다시 열기도 확인했다.
ApiTest에 진행 중 HTTP 요청 취소 후 새 요청 성공, 서버 연결 거부 후 동일 주소 재연결 성공 검사를 추가했다. Android 총16개 테스트와 lint 통과. 제품 코드는 이전 설치본과 같으며 이번 검증 추가분은 테스트/문서뿐이다.
물리 Wi-Fi 끄기/켜기와 WireGuard 전환, 처리 중 입력칸 이동의 늦은 응답 실기기 주입은 별도 미검증이다. 서버 단절/복구 실기기 확인과 클라이언트 요청 취소 자동 테스트를 이 시나리오 전체의 실기기 검증으로 확대하지 않는다. Windows/Linux 실제 검증은 마지막 단계 유지.
