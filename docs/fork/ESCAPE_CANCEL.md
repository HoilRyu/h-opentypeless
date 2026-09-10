# Esc로 음성 작업 취소

음성 입력 준비·녹음·전사·AI 다듬기·질문 처리 중 Esc를 누르면 작업을 취소한다.
캡슐은 대기로 돌아가고 녹음이 중단되며 조절한 음량을 복원한다.
취소한 전사/LLM 결과는 입력하거나 결과 창으로 표시하지 않는다.
이미 입력이 완료된 텍스트를 되돌리는 기능은 아니다.

구현은 `extensions/escape_cancel.rs`에 격리했다.
공통 voice_feedback 상태 전환에서 Esc를 등록/해제하므로 Windows/Linux에도 같은 등록 경로가 적용된다.
대기/출력 완료 상태에서는 Esc를 등록하지 않아 다른 앱이 원래대로 사용한다.
기존 단축키 재등록 후에도 활성 세션의 Esc 등록을 복구한다. 단독 Esc는 취소용 예약 키다.

단축키 플러그인이 레지스트리 mutex를 잡고 콜백을 호출하므로 취소는 비동기 런타임으로 전달한다.
콜백 안에서 Esc 등록을 해제하면 교착될 수 있어 해당 경로를 분리했다.
등록 세대를 검사하여 대기열에 남은 이전 세션의 Esc가 새 세션을 취소하지 않도록 한다.

일반 받아쓰기는 기존 pipeline.abort()의 세대 취소·마이크 해제·STT/LLM future 폐기를 재사용한다.
Ask에는 세션별 watch 취소 신호를 추가했다. 녹음/STT 작업과 답변 처리 future를 함께 중단하고,
Cancelled 결과는 팝업과 내장 AskPanel에서 표시하지 않는다. 처리 정리가 끝날 때까지 busy를 유지한다.

검증 기록은 아래에 추가한다. Windows/Linux 실기기 검증은 후속 단계다.

## 2026-09-10 검증

- Rust 전체 656 pass / 6 ignored, UI 전체 491 pass, lint/clippy와 앱 빌드 통과.
- 자체서명 strict 검사 후 /Applications에 설치·실행. 기존 앱 백업은
  `~/.local/share/h-opentypeless/escape-cancel-backup-ki5e4bee/H-OpenTypeless.app`.
- 설치 앱에 macOS CGEvent로 현재 Fn 단축키 및 Esc 전달: 실제 마이크 녹음 시작→Esc 취소,
  capture_stop_confirmed·Idle·volume_restore_completed 확인.
- 대기 상태에서 두 번째 Esc를 보냈을 때 취소 콜백이 다시 호출되지 않음 확인.
- 전사/LLM 취소의 세대 보호 및 Ask 취소 신호/결과 억제는 자동 테스트로 검사.
- 키 이벤트는 자동 주입한 것이므로 사용자가 직접 누른 키·Windows/Linux 검증과 구분한다.

- 추가 설치 앱 검사: Fn으로 정상 녹음 종료→capture_stop_confirmed 후 Esc→취소·Idle,
  outputting 없음 확인. Transcribing은 기존 pipeline의 직접 상태 변경 경로라
  h-audio-events.log에 해당 상태 이름을 기록하지 않는다. 최초 검사 스크립트의 그 문자열
  기대값은 실패했으며, 실제 정상 종료/취소 이벤트 순서와 소스 상태 전환으로 검증했다.
