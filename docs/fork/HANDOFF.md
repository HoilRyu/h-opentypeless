# H-OpenTypeless 인계

## 2026-09-10 로그인 항목 앱 번들 등록

macOS 13+ 설치 앱은 SMAppService.mainAppService를 사용한다. 이전 LaunchAgent 활성 등록을 새 등록 성공 후 제거하고, 시작 시 마이그레이션한다. 설치본 적용 및 시스템 설정의 H-OpenTypeless 응용 프로그램 표시, 앱 켜기/끄기 단일 저장 검증 완료. 최종 자동 실행 켜짐. 이전 파일·서비스가 제거됐고, 최종 확인에서 과거 백그라운드 표시도 사라짐. [LOGIN_ITEM_BUNDLE_FIX.md](LOGIN_ITEM_BUNDLE_FIX.md) 참고. 이 워크트리에서 커밋하며, 병합 대상은 부모 브랜치 `feat/h-foundation-direct-providers`다.

## 2026-09-10 캡슐 상태 전환 수정 · Whisper MLX 적용

이 워크트리(`HoilRyu/fix-capsule-recording-preview`)에서 Whisper MLX 구현 및 설치 완료.
아래 과거 기록의 “Whisper MLX 미구현” 상태를 대체한다. 캡슐 종료 시 원문 잔류/겹침 수정,
기존 GGML 가중치의 오프라인 MLX 로딩, CPU 선택 보존, 기존 Qwen 회귀 확인.
Turbo 미리보기 연속 5발화 및 cold start 실시간 간격 3발화/최종 전사 통과.
미리보기 중단 정책은 변경하지 않았고 실제 사용자 마이크 환경은 후속 확인 대상.
상세 구현·측정·검증·백업은 [LIVE_STT_PREVIEW.md](LIVE_STT_PREVIEW.md) 참고.
설치본은 이 워크트리 버전이다. 변경을 커밋하며, 병합·푸시는 아직 수행하지 않았다.

## 2026-09-10 동일 프롬프트 모델 비교

Qwen3.5:9b 설치 모델로 동일 12전사문×3회=36회 검사, 직전 Gemma4 current 36개와 비교. 반말→존댓말 Gemma0/33 Qwen5/33; 선택지 보존3/3 대0/3. Qwen 교체 권하지 않음. [전체 결과](LLM_MODEL_POLISH_COMPARISON.md), tools/diagnostics/model-polish-review. 모델명/digest/양자화/프롬프트 해시 포함. 앱/프롬프트 변경 없음. 종료 후 Qwen 해제/Gemma4 재준비, 설정 gemma4:12b/structured 유지 확인. 실제 Gemma 말투 문제 해결을 의미하지 않음.

## 2026-09-10 프롬프트 길이·말투 비교

현재안 3405자/짧은안 502자, 12문장×각3회=72회 Gemma4:12b 로컬 비교 완료. 반말→존댓말 현재0/33·짧은5/33; 선택지 보존 현재3/3·짧은1/3. 짧은안은 명사형/지시문체 변형도 있어 적용하지 않음. [전체 결과](PROMPT_LENGTH_COMPARISON.md), 자료 tools/diagnostics/prompt-length-review. 이번 현재0/33이 실제 사용에서 확인한 말투 전환을 부정하지 않으며 길이만의 인과실험도 아님. 앱/모델/설정 변경 없음.

## 2026-09-10 Ask 제거·재매핑 Fn 오작동 수정

Ask 설정/시험 버튼과 기존 온보딩 안내 제거. H product_scope에서 로드/저장 시 Ask 키 비우기 및 실행 진입점 차단(원본 내부 코드는 보존). 실제 이벤트에서 BetterTouchTool이 방향키 뒤 합성 Fn down/up(source PID 66467)을 보내는 원인 확인. native Fn 처리에서 source PID > 0을 무시하며 사용자 BTT 설정은 변경하지 않았다. 자세한 내용: [ASK_REMOVAL_AND_FN_FIX.md](ASK_REMOVAL_AND_FN_FIX.md).

Rust 642 통과/7 ignored, 최종 전체 UI 492 통과, TypeScript/Vite·ESLint 통과. 자체 서명 앱 설치/재실행, 실행 바이너리와 helper 보존 확인. 백업 app-backups/before-ask-fn-fix-20260910-175854.app. 설치 후 Control+I/J/K/L 및 실제 Fn 검사 요청에 사용자가 정상 작동을 확인했다. 커밋/푸시는 하지 않았다.

## 2026-09-10 구조화 3차 통합안 적용 완료

사용자 승인 후 h_polish.rs의 STRUCTURED와 구조화 예시만 변경. 공통/다른 스타일 유지. Rust prompt 검사 35개 통과, 실제 빌더 export가 검토 candidate-v3.txt와 바깥 공백 외 일치 확인. TypeScript/Vite 및 자체 서명 빌드 완료, /Applications/H-OpenTypeless.app 교체/재실행. credential helper 바이트 보존. 백업 app-backups/before-structured-v3-20260910-174721.app. 설정 structured/polish on/Whisper Large-v3 Turbo 유지. 설치 후 실제 마이크 발화는 사용자 확인 필요. 커밋·푸시 미진행.

## 2026-09-10 구조화 3차 통합안 — 검토 준비

기존 공통/문맥/개인 지침 및 예시 유지, 구조화 본문 교체와 예시 1개 추가한 후보를 10문장+취약 3문장 각 2회로 16회 검사. 결과 보고 포함 4항목 및 최소 포함 4선택지 모두 해당 3회에서 유지. 짧은 복수 요청의 평문 출력/말투 강도 변화는 남음. [검토안](STRUCTURED_POLISH_V3_REVIEW.md), 재현 자료 `tools/diagnostics/polish-review/{structured-v3,candidate-v3}.txt`, `run-v3.py`, `results-v3*.json`. 사용자 검토 전이며 앱 소스/설정/설치본은 변경하지 않았다.

## 2026-09-10 구조화 품질 비교 — 적용 전 검토

개발 요청 재구성 10문장 × 기존/1차/2차 프롬프트, Gemma4:12b 로컬 호출 30회 완료. 2차안은 결과 보고와 단계 목록화가 개선됐으나 두 후보 모두 선택지 최소를 누락하여 앱 적용 보류. 실제 음성 데이터셋이나 다른 LLM 비교는 아님. 자료: `tools/diagnostics/polish-review`, [전체 검토](STRUCTURED_POLISH_REVIEW.md). 앱/설정/모델은 변경하지 않았다. Whisper MLX는 여전히 미구현(Qwen만 MLX).

## 2026-09-10 캡슐 배치·내장 모델 선택 간소화

- 녹음 행을 듣는 중 → 확장 음량 게이지 → (번역 언어) → 시간 → 스타일 → 닫기로 배치. 일반 녹음의 게이지는 남은 가로 공간을 사용하는 21개 막대이며, 다른 캡슐은 기존 7개를 유지한다.
- 내장 모델은 선택 상자와 선택한 모델 카드 하나로 표시한다. 살펴보기와 실제 사용/다운로드를 분리하고, 다른 항목을 보는 동안에도 다운로드 진행 보기로 돌아갈 수 있다.
- Whisper Large-v3 Turbo를 선택 다운로드 카탈로그에 추가. 원본 README의 Groq API 추천 모델과 같은 계열이며 기존 H 목록에서 빠져 있었다. 고정 revision/LFS SHA-256/크기와 HEAD 200 응답을 확인했다. 엔진 소스도 Turbo를 지원한다. 가중치는 자동 다운로드하지 않았고 실제 Turbo 전사는 미검증이다. 현재 Whisper는 CPU 전용이므로 Qwen MLX보다 빠르거나 정확하다고 보장하지 않는다.
- UI 494, TypeScript/ESLint/Vite 통과; Rust 카탈로그 고정 파일 검사 통과. 설치본에서 모델 선택 상자와 Turbo 정보/다운로드 버튼 표시 확인. 캡슐의 새 배치는 코드/UI 회귀 검증을 했으나 이번 OS 자동 캡처는 확보하지 못했다. Orca AX 조회는 권한 상태 granted인데도 permission_denied를 반환하여 설정 화면 확인은 기존 CUA 도구로 수행했다.
- /Applications/H-OpenTypeless.app 설치, 자체 서명·credential helper 보존. 이전 앱 백업은 ~/.local/share/h-opentypeless/model-selector-backup-20260910-170811. Qwen 1.7B 활성 선택과 기존 모델/설정 유지. 커밋·푸시는 아직 하지 않았다.

## 2026-09-10 다듬기 프롬프트·캡슐 스타일 선택

사용자 승인한 네 스타일과 번역 지침을 H 전용 모듈로 적용했다. 녹음 중 캡슐에서 현재 녹음의 스타일을 선택할 수 있다. 기본 스타일 구조화/짧은 개인 지침/콤보 박스 사전을 현재 설치본에 반영했다. UI 492, Rust 640 통과, 실제 메뉴 선택·녹음 지속·Esc 취소 확인. 긴 한국어에서 첫 글머리표가 누락되는 Gemma 출력 한계는 남는다. 상세 내용과 백업/검증은 [POLISH_STYLE_IMPLEMENTATION.md](POLISH_STYLE_IMPLEMENTATION.md). 아직 커밋·푸시하지 않았다.

## 2026-09-10 튜토리얼 디자인 간소화 (0.1.45-beta.2)

- 사용자 피드백에 따라 Superwhisper/Raycast/Linear 공식 시작 안내를 조사했다. [TUTORIAL_REDESIGN.md](TUTORIAL_REDESIGN.md)에 출처·설계 판단·검증 범위를 기록.
- 필수 흐름을 사용 방법 → 준비 → 연습으로 줄이고 마지막 완료 화면을 둠. 준비는 STT/AI/단축키·권한 세 줄이며 변경할 항목만 펼친다. 단색 버튼, 입력창 예시, 실제 단축키, 작은 진행 점으로 구성.
- 기존 polish/controls 진행 위치는 준비로 변환. 기존 외부 API 설정 다시 보기의 재시험 강제를 없앴으며 모델/프롬프트/키를 초기화하지 않음. 연습 결과는 최종 문장이 먼저, 원문/부가 설명은 펼쳐 확인.
- `/Applications/H-OpenTypeless.app`에 최종 자체 서명본 설치, 빌드 실행 파일 일치와 기존 키체인 helper 유지 확인. 최종 버튼 강조까지 실제 화면으로 확인하고 첫 화면을 열어 두었다.
- UI 487개, TypeScript/Vite 및 ESLint 통과. 실제 설치본 다크 테마 900×700에서 기본 화면과 설정 열기/닫기, 녹음 연습/완료 화면 확인. 네이티브 녹음 코드는 변경하지 않아 Rust 전체 검사는 반복하지 않았다. 시작 가이드 코드는 `d4bf270`으로 커밋했고 원격 푸시는 아직 수행하지 않았다.

## 2026-09-10 시작 가이드 초기 구현 (0.1.45-beta.1, 이후 beta.2로 간소화)

- H 전용 6단계 튜토리얼: 소개, STT 모델/API 준비, 선택적 AI, 권한·실제 단축키, 최대 15초 네이티브 녹음 연습, 일상 사용. 한국어/영어와 테마 대응 도식. 기존 Onboarding 소스는 보존.
- 최초 실행/단계 재개, 홈 및 설정→정보에서 다시 보기, 기존 설정 보존. 설정/완료 플래그 저장 실패를 표시하고 진행을 막는다. 연습은 외부 앱 입력·클립보드·기록 저장 없이 원문/교정문을 보여 준다.
- UI 484, Rust 644 통과(5 ignored), TypeScript/Vite·ESLint·Clippy 통과. 설치본 0.1.45-beta.1 자체 서명 검증, 기존 키체인 helper 해시 유지. 이전 앱은 `app-backups/before-tutorial-20260910-090921.app`에 보존.
- 설치본 실제 UI에서 초기 6단계 이동, Ollama 연결 시험, 완료 저장/홈 복귀/재진입을 확인했다. 이 화면은 beta.2의 간소화된 흐름으로 대체됐다. 실제 녹음 연습의 사용자 확인과 Windows/Linux 실기기 검증은 별도다. 구현과 범위는 [TUTORIAL.md](TUTORIAL.md)를 참고한다.

## 2026-09-10 내장 MLX 구현·설치 완료 (0.1.44-beta.1)

- 호환 Mac의 Qwen 자동 선택을 MLX GPU로 연결했다. 전용 Python/MLX를 앱에 포함하고 stdin/stdout으로 제어한다. 별도 STT 서버나 사용자 Python 설치를 요구하지 않는다. 기존 Qwen 1.7B 가중치/선택, LLM/Ollama, 키체인 도우미를 유지했다.
- 녹음 중 준비, 단일 Session 재사용, 약 2분 유휴/메모리 압박 해제, 취소·부모 사망·앱 종료 회수. 파일 검증 캐시와 Mac arm64 SHA 가속을 추가했다. 자동/MLX/CPU 및 현재 엔진/메모리 해제를 설정에 표시한다.
- `/Applications/H-OpenTypeless.app`에 최종 서명된 버전 설치. 실제 UI에서 자동 → MLX, Qwen 1.7B 유지와 메모리 해제 확인. 기존 helper SHA-256 `03ddf7a45a52a90ee92a25e9c576a41175e1e44523b12b5dd73bebecb4a9ef99` 유지.
- 설치본 모바일 `/api/dictate → MLX → Ollama`: 합성 음성 첫 요청 4.805초, 후속 0.907/0.917초, warning 없음. 최초 파일 검증/엔진 준비와 LLM을 포함하며 일반 성능 보장은 아니다.
- UI 472, Rust 643 통과(5 ignored 중 MLX 실제 제공자 별도 실행 통과), Clippy -D warnings, 릴리즈 테스트 10 통과. Qwen 0.6B/1.7B 실제 반복 전사·120초 입력·부모 사망/유휴 회수도 검증했다. 자세한 범위/한계는 [MLX_STT_VERIFICATION.md](MLX_STT_VERIFICATION.md).
- 검증용 DMG: `~/.local/share/h-opentypeless/mlx-build/H-OpenTypeless_0.1.44-beta.1_mlx-validation.dmg`. GitHub 공개 릴리즈는 아니다. MLX 통합은 `9561332`로 커밋했고 이전 앱은 `app-backups/before-mlx-20260910-014350.app`에 보존했다.
- 이후 [첫 실행 튜토리얼](ONBOARDING_PLAN.md)을 구현하고 간소화했다. Windows/Linux 실기기는 예정대로 추후 검증한다. 실패 음성을 별도 창에 보관해 즉시 재전사하는 UI는 이번 변경에 포함하지 않았다.

## 2026-09-10 내장 MLX 계획

[MLX_STT_PLAN.md](MLX_STT_PLAN.md)에 환경 감지, macOS arm64 전용 런타임 번들, 기존 Qwen 가중치 재사용, 단일 워커/유휴 해제, 성능·안정성 검증 순서를 작성했다. 호환 Mac은 자동 모드에서 MLX GPU를 선택하며 실패를 숨겨 CPU로 전환하지 않는다. 현재는 계획만 작성했고 앱·모델·실행 설정은 변경하지 않았다. 다음 작업은 실제 CPU 기준 계측과 MLX 번들 시제품 검증이다. 튜토리얼은 이 작업 이후 진행한다.

## 2026-09-10 튜토리얼 준비와 지연 점검

내장 STT 기능 커밋: `8b46610` (푸시 미실시). 사용자는 내장 Qwen 1.7B 정상 작동을 확인했으나 체감 지연을 보고했다. 현재 CPU 엔진·매회 전체 파일 검증·모델 로드가 기존 MLX GPU 서버와 다르다. 약 3초 합성 음성의 독립 측정에서 SHA-256 검증 2.14초, 엔진 로드/전사 1.91초였다(앱 전체 지연/LLM 시간 아님).
[ONBOARDING_PLAN.md](ONBOARDING_PLAN.md)에 튜토리얼 흐름과 선행 성능 측정/개선 순서를 기록했다. 아직 튜토리얼이나 성능 동작 변경은 구현하지 않았다.

## 2026-09-10 외부 Qwen STT 서버 제거

사용자 요청으로 `local.opentypeless.qwen3-asr` launchd 작업을 bootout하고 외부 서버 Python 환경·모델·키·전용 로그·plist를 제거했다. 8765 포트 종료 확인. `~/.local/share/opentypeless-local`에는 Ollama plist와 Ollama 관련 로그만 유지했다. Ollama 11434 정상 응답 확인.
내장 Qwen 1.7B 다운로드는 app data의 별도 디렉터리에 보존했다. 외부 서버 제거 후 사용자가 정상 작동을 확인했다. 아래의 외부 서버 유지/복원 기록은 제거 전 이력이다.

## 2026-09-10 내장 STT 추가

- `extensions/local_stt` + `LocalSttSetting.tsx`: Whisper Tiny/Base/Small, Qwen3-ASR 0.6B/1.7B 선택 다운로드·이어받기·검증·삭제·선택. 엔진은 앱 리소스에 포함하고 모델은 app data에 보존한다.
- 네이티브 전사 후 프로세스 종료, 취소/timeout 종료, 120초 PCM 제한, 4 스레드, 단일 작업 lease. STT에 키체인/API 키를 사용하지 않는다. 기존 LLM/API/Ollama 기능은 유지한다.
- macOS 앱 설치 및 실제 `mobile /api/dictate → builtin Qwen 0.6B → Ollama` 합성 한국어 전사/다듬기 성공, warning 없음. 이후 STT 제공자는 기존 custom-whisper로 복원했다. 내장 선택 모델은 qwen-0.6b로 준비했다.
- Base와 Qwen 0.6B는 검증한 파일을 `~/Library/Application Support/dev.hoilryu.hopentypeless/local-stt`에 준비했다. 기존 MLX 1.7B 원본 모델/서버/주소는 유지했다.
- frontend 471, Rust 639 통과(명시적 다운로드/실제 모델 테스트 등 4 ignored); 실제 모델 다운로드와 한국어 전사(Base/Qwen 0.6B/1.7B) 별도 통과. Clippy -D warnings 통과. 릴리즈 스크립트 테스트 10 통과.
- 자세한 사용법/구조/플랫폼 제한: [LOCAL_STT.md](LOCAL_STT.md). Windows Qwen은 비활성화. Windows/Linux 실기기 검증은 아직 미실시.
- 내장 STT 기능은 8b46610으로 커밋했다. 기존 f59e7f1 릴리즈 후보 DMG에는 이번 STT 기능이 포함되지 않는다. 릴리즈할 때 새 커밋 기준으로 패키지를 다시 생성해야 한다.
- **사용자 요청: STT 작업이 끝나면 첫 실행 튜토리얼을 다음 작업으로 안내할 것. 튜토리얼은 아직 구현하지 않았음.**

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

## USB 오디오 패닉 조사 및 안정화
사용자 Mac 재부팅 조사에서 usbaudiod/AppleUSBXHCI kernel data abort 확인. H의 음량 조절 비동기 완료 누락, 실제 capture.stop 완료 전 음량 복원, 오류 시 반복 조회를 개선했다. src/audio/lifecycle.rs 공유 gate와 fault latch, capture.stop/Drop 종료확인(최대3초), macOS HAL 알림+readback 완료확인(최대1초), 불확실한 변경 복구기록 보존, Off 즉시 end 및 Off시 시작복구 생략, 조회 크기 검증, metadata-only256KiB 로그2개를 구현. 상세 USB_AUDIO_PANIC_REVIEW.md.
전체 Rust590pass/1ignored, 최종 audio45pass/1ignored, clippy --lib --tests -Dwarnings 통과. 설정UI3개 및 lint 통과. 실제 USB 스트레스 테스트는 재부팅 위험으로 수행하지 않음. 시스템 패닉 원인이 확정되거나 제거됐다고 주장하지 말 것. 정지 대기는 기존 동기 stop 호출자를 보존하여 최대3초 블로킹이며 timeout 후 장치 명령을 차단한다. OS/native API 자체가 멈춘 작업을 강제 중단하지는 않는다.

안정화 Mac 빌드/기존 자체서명/strict 검증 성공(audio-fix-build.log), /Applications 교체·실행 완료. 접근성 배너 없음, mobile health200, h-audio-events.log의 audio_service_started 생성 확인. 사용자 기존 음량 설정5%는 유지. 실제 USB 녹음 재검증 미실시. 이번 수정은 미커밋 상태.

## 마이크 실패 회귀 수정 및 추가 자원 제한
첫 안정화 버전의 USB 음량 readback 허용오차0.002가 하드웨어 단계 반올림을 실패로 판단해 마이크까지 latch했다. EDIFIER 실측 요청0.119097216 → 실제0.125434026, HAL알림2개 확인. macOS write_observed가 실제확인값을 엔진에 반환하고 복구기록에 저장하도록 수정. 같은 값이면 쓰기생략; 실제쓰기는 HAL알림을 기다림. 음량 확인 실패는 음량쓰기만 중지하고 마이크 전역fault와 분리; native call gate timeout/캡처종료 미확인은 계속 전역보호. 상세 USB_AUDIO_PANIC_REVIEW.md.
진행 중 추가 변경: response_limits.rs8MiB HTTP/SSE총량·1MiB줄 제한/UTF8분할보존, Whisper PCM해제, 음량명령queue64, 로그회전실패시쓰기중지, Android RecordingLease/worker소유stop-release. Android18개테스트/lint/build 통과(stability-android.log); APK아직추가설치안함. Mac수정은 미커밋, 현재 빌드/실행검증 진행 중.

마이크 수정 최종검증: Rust595pass/1ignored, Mac native opt-in5%/35%/mute 복원1pass, clippy -Dwarnings 및 설정UI3개 통과. 자체서명 빌드·/Applications 교체·실행 완료(stability-final-*.log), 접근성배너없음. 기존 잘못된 복구기록은 현재 수동음량과 달라 h-audio-recovery.pre-quantization-fix.json으로 보존이동했고 장치음량은 변경하지 않음. 재부팅 후 Qwen/Ollama미실행 발견하여 기존 외부 plist bootstrap으로 현재로그인세션에서 기동. STT/health200 및 Ollama/api/tags200 확인. 로그인자동시작설정은 새로추가안함. Android192.168.0.117:42931재연결 후 안정화APK install-r 성공. CUA 합성단축키는 실제녹음미시작이므로 사용자실제발화확인 요청; native스피커검사를 마이크테스트로 확대해 보고하지 말 것. 현재 미커밋.

## 실제 사용 검증 2026-09-09 오전
REAL_USE_AUDIO_VALIDATION.md에 현재 결과 기록. 8시간이상 동일PID실행/새패닉없음/기존4회+검사중추가2회 capture정상. 추가2회STT교정결과생성, UI입력성공사용자확인대기. RSS3분37샘플115~119MiB. leaks14704→14736bytes(NSArray32B1개 증가)라 무누수주장금지; 할당스택없어원인미확정. USB5%3회짧게+20초1회+Esc취소 요청해둠. Off/내장장치비교 아직안함. 사용자답변전에설정/장치바꾸지말것.

다음단계실행: 현재사용자비율2%확인, CUA로음량Off전환. 설치앱Ask버튼/캡슐취소버튼으로 실제USB마이크3회시작·취소(54.834/19.161/6.082초), 음량쓰기0/fault0/종료확인정상. 일반받아쓰기나물리Esc검증은아님. leaks첫Ask후16336→3회후16800bytes; 녹음외AX/UI활동분리필요. 시스템사운드내장선택자동화가적용되지않아사용자에게내장마이크/스피커직접선택요청. read-only확인현재Razer입력/EDIFIER출력유지. 현재modeOff; 검사완료후reduce2%및원래USB장치복원필요. 상세REAL_USE_AUDIO_VALIDATION.md.

## 누수원인추적 및 중복캡처 2026-09-09
LEAK_CAUSE_TRACE.md 참고. CPAL0.15.3 name() CFString미해제 실제재현:독립CLI마이크미개방0/100/1000조회→0/6400/64000B누수,할당스택확인. Mac진단device.name호출제거판설치완료. UI-only대조는사용자녹음겹침으로무효. 이름제거판실기기Ask검증중동시capture_start2개후stop발견; 글로벌CaptureLease를native종료까지보유하여2번째시작거부하는수정과테스트추가후빌드중. 사용자에게녹음멈춤요청답변대기. 현재mute/2%사용자설정유지. 아직미커밋.

누수추적최종판(이름조회제거+CaptureLease+Ask오류안내) Rust597pass/1ignored,clippy/빌드/자체서명검증완료. 녹음종료상태확인후 /Applications최종교체·실행완료,권한배너없음. 수정후중복시나리오실기기검사는아직안함; lease1000회재사용및native종료전독점테스트통과. 사용자음소거mute/2%유지,추가음성입력중지요청은설치완료로해제가능. 원인추적자료LEAK_CAUSE_TRACE.md.

## 남은AX후보분리완료
독립AppKit앱MallocStackLogging 실험으로NSArray/AXObserverCookie 동일패턴재현, Apple비공개접근성observer제거함수할당스택확인. 연속100→200뷰교체(UI중간조회없음)에서추가증가0B. 녹음누수판정과분리; OS누수자체패치아님. AX_OBSERVER_INVESTIGATION.md및tools/diagnostics/에증거/재현절차. 이단계에서H실행파일/설정변경없음.


2026-09-09 코드 검토 후 맥 안정화: credentials 공통 blocking worker 1개/20초 제한, 받아쓰기 준비 취소, Ask 시작 세대 검사, useTauriEvents 전체 상태 구독 제거, CaptureFault로 큐 포화·마이크 오류 시 전송 중단 및 불완전 결과 취소. Ask done 신호 보존. Rust613pass/2ignored, UI458pass, lint/clippy/build 통과. 물리 마이크 start/read/stop100회 별도 통과(2회 실행), 측정 실행 RSS10회25680KiB→100회25760KiB. 전체 앱 장시간/GPU 누수 검사로 확대 해석하지 말 것. 기존 자체서명 debug 빌드 설치 및 재실행 PID69804, 홈/접근성 경고 없음, mobile /api/health200. 백업 ~/.local/share/h-opentypeless/improvement-backup-89x5mrdu/H-OpenTypeless.app. Android 소스 추가 변경/배포 없음, VAD 보류 유지. 상세 CODE_REVIEW_2026-09-09.md 후속 개선 절.


2026-09-09 후속: 사용자가 Mac 간헐 음량 복원 문제를 우선 요청하여 Android를 잠시 중단. CoreAudio 알림+이전값 성공오인 방지, 음량 복원4회 재시도, 캡처 종료/복원을 선택텍스트 조회보다 먼저 수행. Rust616pass/2ignored, 실제음량reduce5%/35%/mute복원1pass,clippy/빌드/자체서명설치완료. AUDIO_DUCKING.md 기록, 백업volume-restore-backup-fpkk9a3b.
그 뒤 Android0.3.6(versionCode9): PCM종료별도worker/UI취소nonblocking, 임시파일/세대보호, 실패종류별모델차단(일시OOM/timeout재시도가능,Linkage호환성재검사), Whisper JNI RAII/null/할당/예외검사, STT/LLM로딩·추론시간분리. API가드/새Back콜백 lint보완. 단위40pass, lint/build, S26JNI200회/지연종료취소/fallback/UI회귀/모델취소프로세스종료/WhisperBase·GemmaE2B 실행 통과. 설치APK0.3.6 확인. 상세 android/README.md, ~/.local/share/h-opentypeless/android-stability*.log. 모델항상종료 정책유지. VAD보류유지, 무음Whisper출력은품질검증아님.


## Mac mute 완화 추가 — 2026-09-09
HAL mute=0/volume≈32%인데 YouTube와 테스트음이 모두 무음인 사용자 사례가 있었고, 추가 녹음 시작/종료 후 사용자가 복구를 확인했다. Mac 새 음소거 세션은 하드웨어 mute 대신 volume=0을 사용하도록 변경·설치했다. 27개 관련 테스트 통과, 장치 조절 테스트는 생략. 실제 재발 해결은 미검증. 기존 mute 복구와 수동 조절 보존 유지. 자세한 증거/백업/검증 한계는 AUDIO_DUCKING.md 마지막 절 참조.

## 입력 대상 오판정과 결과 창 — 2026-09-09
최근 TargetChanged 복사 fallback 조사에서 녹음 시작의 오래된 컨텍스트 캐시와 출력 전 AppleScript 조회 실패 경로 확인. macOS 활성 앱 NSWorkspace 직접 조회로 시작/최종/스트리밍 guard를 변경하고 metadata-only 제한 로그 추가. 캡슐 근처 작업 영역 내 배치 및 캡슐 스타일 결과 창 적용. detector29/배치2/AskPanel20 통과, 자체서명 빌드·설치·실행 완료. 실제 사용자 입력과 화면 확인 대기. 정확한 과거 사건 원인은 구버전 PID 로그 부재로 미확정. 상세 RESULT_WINDOW_FOCUS.md. 키체인 helper 승인/업데이트 검증은 KEYCHAIN_UPDATE_PROMPTS.md 참조.

복사 창 닫기 두 번 클릭 후속: ask WebView의 기본 acceptFirstMouse=false와 비활성 표시 조합 확인. tauri.conf.json 및 fallback builder에서 true 적용. 빌드·서명 strict 검증 후 /Applications 교체 완료(/tmp/h-first-click-build.log). 첫 클릭 실사용 확인은 아직 대기. 메인 창 동반 표시 방지 방식과 helper 유지.

복사 창 클릭 후 H 메인 창 포커스 후속: acceptFirstMouse는 앱 활성화를 막지 못해 일반 NSWindow 클릭→H 활성화→hide 후 메인 창 key 전환 발생. macOS 전용 pinned tauri-nspanel 추가, result_window 모듈에서 재사용 NSPanel/NonactivatingPanel/key·main 불가로 전환하고 메인 스레드에서 표시. 다른 OS 일반 창 유지. 최종 빌드·자체서명 strict·설치·실행 완료(/tmp/h-nonactivating-panel-final-build.log). 사용자에게 메인 창을 열어둔 채 외부 앱에서 복사 창 닫기 실사용 검증 요청; 답변 대기. 전체화면·다중 모니터 실기기 회귀는 미검증. plugin 업데이트 시 네이티브 class 전환 호환성 점검 필요. 상세 RESULT_WINDOW_FOCUS.md.

사용자 후속 확인: 비활성 패널 설치 후 닫을 때 메인 창에 포커스가 남지 않는 것 같다고 응답. 해당 실사용 사례 해결 확인, 전체화면/다중 모니터 검증으로 확대하지 않음.

캡슐 위치 사용자 정의 정정: 사용자가 원하는 것은 글자 삽입점이 아니라 입력창 영역. macOS AXFocusedUIElement 및 최대4단계 부모에서 텍스트 컨트롤 역할을 확인하고 AXPosition/AXSize를 읽도록 교체. 입력창 아래 중앙/아래 공간 부족시 위/작업영역 경계 보정. 조회불가시 마우스를 사용하지 않고 화면하단. near_caret 저장키는 기존설정 호환을 위해 유지하고 UI 이름은 입력창 근처로 변경. production geometry 함수를 추출하여 rustc --test로 3개 통과(아래중앙·음수화면/위로배치·조회불가하단). 전체 macOS 빌드·자체서명 strict·설치 완료(/tmp/h-input-area-build.log). 앱별 실제 입력창 AX지원은 사용자 확인 대기.

## 반복 마이크 실패 안정화
사용자 입력창 배치 확인 후 반복 마이크 실패 발생. PID62791 sample에서 capture/CoreAudio 활성, 로그는 시작 후 stop요청없이 음량복원/시도반복. 종료후PID64966으로재시작,사용자정상입력+4초/8.9초STT/마이크해제확인. 최초오류stderr부재로확정원인은미상. 코드상 이전stop이잠금해제후늦게Idle설정하는경쟁후보수정: watch세대취소로start/stopfuture폐기, stop은최종cleanup까지lock유지(취소시즉시해제), 성공한start만abort초기화, Idle의자기마이크잔여handle정리, streamingworker소유자drop시abort, metadata상태로그보강. pipeline38/전체Rust628pass2ignored. 빌드진행중. 상세 RECORDING_SESSION_STABILITY.md. 재시작복구를근본해결로보고하지말것. 새버전실사용재검증필요.

안정화 최종 빌드(/tmp/h-session-stability-build.log)·서명strict·/Applications교체·실행 완료. 현재 stdout/stderr /tmp/h-session-stability-runtime.log, 기존제한로그 h-audio-events.log도사용. 사용자에게 일반2회와 처리중취소→즉시재녹음 확인요청, 아직답변대기. 장시간실사용 안정화완료주장금지. 미커밋상태.

## 복사 창 사용자 지정 단축키
hotkeys.copyResult(기본null)와 CopyResult 역할 추가. 설정→일반→단축키에 등록/수정/제거 UI(최대1개), 기존 충돌검사/전역등록/백업저장 사용. 표시중 ask 창에만 이벤트 전달하고 frontend가 표시/현재dictate_insert결과를 재확인해 native clipboard write. 포커스변경/자동닫기없음. Rust629pass2ignored, 관련UI119pass, 타입검사/빌드/서명strict/설치/실행 및 실제설정항목표시확인완료. 기본키는 미지정유지; 사용자 지정조합의 물리키 E2E는아직안함. 전체UI검사는 초기9실패 중 새설정행개수기대값1개수정후관련검사통과; 나머지8개는 이전클라우드제거/모바일버튼문구를기대하는기존테스트로 이번기능과무관하며아직미정리. 상세COPY_RESULT_SHORTCUT.md. 현재앱은 CUA로재실행하여stdout지정없음, h-audio-events.log 제한로그는계속기록.


## 커밋 정리 완료 — 2026-09-09

b371c84 이후 누적 변경을 기능별 7개 커밋과 디자인·문서 커밋으로 정리했다. 상세 내역은 COMMIT_PLAN.md 상단을 따른다. 사용자가 복사 단축키의 실제 동작을 확인했다. 정리 과정에서 오래된 UI 테스트 기대값을 현재 동작에 맞게 수정했고 전체 467개 통과 및 프런트엔드 빌드 통과를 확인했다. Rust는 직전 629개 통과/2개 ignored 기록을 유지한다. 이번 작업에서는 앱 재설치·물리 장치 검증·푸시를 수행하지 않았다.


## 첫 릴리즈 준비 — 2026-09-09

데스크톱 0.1.43-beta.1 / Android 0.3.7(versionCode 10) 준비. 원본 release/SignPath/staple/drafter 워크플로를 원본 저장소에서만 실행하도록 제한하고 H 수동 검증/Android artifact 워크플로를 추가했다. Mac은 기존 자체 서명/helper를 유지하는 release ZIP 스크립트, Android는 저장소 밖 영구 배포 키와 정식 서명 APK 스크립트를 추가했다. 원격 공개는 별도 Draft 전용 스크립트로 분리했고 태그/SHA/소스 청결/체크섬/실기기 검증 기록을 검사한다. 빌드 중 소스 변경도 거부한다.

UI 467개 통과, 배포 manifest 검사 9개 통과. Android release 빌드와 단위 40개 통과, lint 0 errors/15 warnings, 배포 서명 확인. Kotlin 2.4 메타데이터에 대한 기존 lint 엔진의 호환 진단 메시지가 있으나 lint 작업/빌드는 성공했다. 키가 없을 때 release 작업이 실패하는 것도 검사했다. 개발 APK와 release APK의 인증서가 달라 최초 전환에는 재설치가 필요하다. 실제 기기의 기존 앱은 삭제/교체하지 않았다.

Mac 최종 빌드/패키지 검증 결과는 별도 releases 폴더의 PREPARATION_VALIDATION.md에 기록한다. npm audit는 high/critical 0개, 개발 도구 관련 moderate 3개다. GitHub secrets 등록·태그 생성·릴리즈 공개는 아직 수행하지 않았다. 서명 키의 별도 위치 백업과 신규 설치/업데이트 실사용 검증은 공개 전 필요하다. 실행 방법: RELEASING.md, 공개 설명 초안: RELEASE_NOTES.md.


## DMG 설치 방식 — 2026-09-09

사용자 요청으로 macOS 공개 산출물을 ZIP에서 DMG로 전환했다. scripts/h-package-dmg.sh는 기존 자체 서명 앱을 변경하지 않고 dmgbuild 1.6.7의 고정 Finder 배치로 포장한다. 앱 → Applications 바로가기 드래그 방식이며 기본 배경 화살표를 사용한다. 도구는 저장소 밖 가상 환경에 설치한다. release/초안 등록/형식 검증/배포 안내도 DMG로 맞췄다. 미리보기 DMG의 hdiutil verify 통과. 최종 산출물의 마운트·서명·Finder 확인은 해당 releases 폴더의 검증 기록에 남긴다. 실제 /Applications 설치본은 변경하지 않는다.

## 내장 STT 발화 미리보기 — 2026-09-10

`LIVE_STT_PREVIEW.md` 참고. Earshot VAD와 bounded 구간 전사 작업자, 녹음 캡슐 두 줄 미리보기,
내장 STT 설정 토글 추가. UI 원문 미리보기와 최종 전사를 분리하여 구간 경계 누락을 방지한다.
최종 입력은 기존 전체 PCM 전사→LLM 경로를 사용한다. Ask/모바일은 미리보기를 요청하지 않는다.
Qwen 1.7B MLX와 Whisper Base CPU의 실제 합성음성 3구간 미리보기/최종 전사 통과.
플랫폼 공통 구현이나 Windows/Linux 실기기·CPU Qwen·사용자 실제 마이크 검증은 아직 미실시.

자체서명 debug 빌드·서명 strict 확인 후 /Applications 교체 및 홈 실행 확인 완료.
실행 파일 SHA256 빌드와 일치. 백업 utterance-preview-backup-68ey_j1o.
사용자에게 녹음 중 한 문장 뒤 1초 멈춰 캡슐 미리보기 확인 요청 상태.


## Esc 취소 — 2026-09-10

ESCAPE_CANCEL.md 참고. 활성 음성 작업 동안만 Esc 전역 등록, 대기 상태 해제.
일반 pipeline.abort 재사용, Ask STT/LLM watch 취소와 Cancelled 결과 표시 억제 추가.
플러그인 콜백의 mutex 재진입을 피하도록 비동기 전달하며 등록 세대를 검사한다.
Rust656/UI491 통과, lint/clippy/자체서명 설치 완료. 실제 설치 앱에 Fn/Esc CGEvent를 보내
마이크 종료·Idle·음량 복원 및 대기 시 Esc 미등록 확인. 백업 escape-cancel-backup-ki5e4bee.
기존 STT 미리보기 변경과 함께 미커밋 상태. Windows/Linux 실기기 검증은 아직 안 함.
