# 발화별 내장 STT 미리보기

설정 → 음성 인식 → 내장 STT에서 `녹음 중 인식 내용 표시`를 켜면,
말을 잠시 멈출 때마다 캡슐 아래 두 줄에 원문 미리보기가 표시된다. 기본값은 켜짐.
일반 데스크톱 받아쓰기에 적용하며 모바일 업로드·Ask는 기존 흐름을 사용한다.

## 동작

- Earshot 1.2.2 VAD: 16 kHz PCM16, 16ms 프레임, 점수 0.5 이상.
- 발화 앞 304ms 보존, 음성 최소 96ms, 침묵 704ms 후 구간 확정.
- 12초 연속 발화는 강제 분할하며 다음 구간에 304ms를 겹친다.
  겹친 구간에만 정확히 일치하는 접미/접두 문자열을 제거한다.
- 추론 작업자 하나, 대기 구간 최대 2개, 구간당 최대 15초.
  큐 포화·오류·음성 길이보다 느린 추론 시 해당 세션의 미리보기를 중단한다.
- 마이크 수신 경로에서는 VAD와 제한된 큐 삽입만 수행한다. 추론은 별도 task다.
- MLX 준비 완료 후 기존 모델 작업자를 재사용한다. CPU Qwen/Whisper도 같은 경로다.
- 종료 시 미처리 미리보기 구간을 버리고 실행 중인 구간만 제한 시간 내 마친다.
  이후 원본 전체 PCM을 전사하여 LLM에 한 번 전달한다. 미리보기를 최종 결과에 덧붙이지 않는다.
  초기 계획의 구간 결과 결합 대신 정확도를 우선한 결정이며 추가 연산 비용이 있다.
- 원본 PCM은 기존 120초 상한을 유지한다. 취소 시 추론 task/프로세스와 대기 큐를 해제한다.
- 모델·STT 설정 변경을 막는 기존 lease를 미리보기 작업도 보유하여 엔진 동시 사용을 방지한다.
- 캡슐은 포커스를 가져오지 않고, 텍스트가 나타난 뒤 고정 크기로 최근 두 줄을 표시한다.
- 설정은 모델 저장소의 `preview` 파일에 on/off로 보관한다. 세션 중 변경은 거부한다.

## 검증

2026-09-10, 이 Mac의 기존 합성 한국어 음성으로 실제 모델을 호출했다.

- Qwen 1.7B / MLX: 3개 발화 미리보기 349/308/302ms, 전체 최종 전사 768ms.
- Whisper Base / CPU: 3개 발화 미리보기 772/582/586ms, 전체 최종 전사 775ms.
- 위 시간은 모델 준비 후 구간이 도착한 뒤의 측정값이다. 실제 표시에는 침묵 감지 약 0.7초가 추가된다.
- 합성 음성의 ‘취소’를 ‘최소/쥐소’로 인식한 사례가 있어 정확도 보장을 뜻하지 않는다.
- VAD 무음/짧은 소리/긴 발화/임의 바이트 경계, 중복 경계 처리, 큐 포화,
  수신 future 취소, 작업자 실패, 추론 중 취소 및 프로세스 회수 테스트 포함.
- Windows/Linux 실기기와 실제 마이크를 통한 사용자 발화, 잡음 환경, 장시간 메모리는 후속 검증 대상이다.

실제 모델 회귀는 `real_utterance_preview_and_final` ignored 테스트다.
`H_LOCAL_STT_TEST_ROOT`, `H_LOCAL_STT_ENGINE_DIR`, `H_LOCAL_STT_TEST_PCM` 환경 변수를 사용한다.
`H_PREVIEW_TEST_ROUNDS`로 반복 횟수를 지정할 수 있다. 시험용 음성만 사용한다.

## 유지보수

주 구현은 `extensions/local_stt/preview.rs`와 `provider.rs`에 격리했다.
공통 trait에는 기본 no-op `enable_preview()`만 추가하고 일반 dictation pipeline에서 호출한다.
Earshot의 MSRV에 맞춰 Rust 최소 버전을 1.87로 올렸다. MIT 고지는 LOCAL_STT_NOTICES.md에 포함된다.

## 설치와 회귀 기록

- 2026-09-10 자체 서명 debug 앱을 `/Applications/H-OpenTypeless.app`에 설치·실행했다.
- 이전 앱: `~/.local/share/h-opentypeless/utterance-preview-backup-68ey_j1o/H-OpenTypeless.app`.
- 설치본과 빌드 산출물 실행 파일 SHA-256 일치, 기존 인증서/credential helper 유지, 앱 홈 실행 확인.
- 전체 Rust 652 pass/6 ignored, UI 489 pass. 이후 큐 포화·이벤트 초기화 테스트를 추가하고 관련 검사를 재실행했다.
- 실제 마이크에서 캡슐 표시 여부는 사용자 확인 요청 상태다. 커밋/푸시는 아직 하지 않았다.

## 2026-09-10 캡슐 전환 및 Whisper MLX 재진단

녹음 종료 시 미리보기를 비우고 이후 partial 이벤트를 무시한다. 변환 단계는 항상
상태 문구를 표시하며 이전 녹음 화면의 exit 애니메이션을 함께 렌더링하지 않는다.

Whisper large-v3-turbo CPU에서 약 3.97초 시험 음성 처리에 7.04초/14.90초가 걸렸다.
첫 결과를 전달한 직후 `max(음성 길이, 2초)` 초과 조건으로 미리보기 작업이 종료되고,
세션 내 재시작도 중단 안내도 없어 첫 결과만 남는 현상과 부합한다.

Whisper MLX 적용 후 같은 입력 10회: 첫 인식 2.41초, 이후 0.58~0.68초.
첫 인식 이후 MLX active bytes는 1,618,308,302로 일정했다. 이는 제한된 반복 시험이며
장시간 앱 전체 메모리 누수가 없음을 보장하는 결과는 아니다.
실제 Rust Provider/Preview/VAD 경로의 5개 발화 갱신과 전체 최종 전사도 통과했다.
미리보기 중단 정책은 변경하지 않았으므로 느린 환경·짧은 첫 발화·큐 포화에서는
여전히 종료 후 전사로 전환될 수 있다. 실제 사용자 마이크 환경은 별도 확인이 필요하다.

재현 도구: `tools/diagnostics/check_whisper_mlx.py` (명시한 시험용 PCM만 사용).
GGML 파서 회귀: `scripts/tests/test_whisper_ggml.py`.
MLX 공식 구현 참고: https://github.com/ml-explore/mlx-examples/tree/main/whisper
GGML 형식 참고: https://github.com/ggml-org/whisper.cpp/blob/master/models/convert-pt-to-ggml.py

추가 검증: cold start부터 16ms 간격으로 PCM을 공급한 3발화 미리보기/최종 전사 통과
(`real_streaming_preview_from_cold_start`, 약 13.4초). 준비 완료 후 시작하는 시험과 별도다.
UI 492, Rust 전체 642(이후 추가한 실제 cold start 테스트 별도 통과), GGML 파서 2개,
TypeScript/Vite 및 ESLint 통과. Whisper Base MLX 및 기존 Qwen MLX 반복 인식도 통과했다.

설치: `/Applications/H-OpenTypeless.app`, 기존 인증서 서명과 credential helper 보존,
빌드/설치 실행 파일 SHA-256 일치 확인. 설정 화면에서 `Whisper Large-v3 Turbo · 사용 중`,
`실행 엔진: MLX` 확인. 설치된 런타임으로 추가 3회 인식은 0.78/0.37/0.38초,
MLX active bytes는 모두 1,618,308,302였다. 환경 부하에 따른 변동이 있으므로 보장값은 아니다.
백업: `~/.local/share/h-opentypeless/app-backups/before-capsule-whisper-mlx-20260910-200516.app`.
진단 원본: `~/.local/share/h-opentypeless/whisper-mlx-build/{turbo-repeat,base-repeat,installed-turbo,install}.json`.
변경은 `HoilRyu/fix-capsule-recording-preview` 워크트리에서 커밋한다. 병합 대상은 부모 브랜치 `feat/h-foundation-direct-providers`이며, 병합·푸시는 아직 수행하지 않았다.
