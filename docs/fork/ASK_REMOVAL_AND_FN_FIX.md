# Ask 제거와 방향키 재매핑의 Fn 오작동

후속 수정: [Stream Deck Fn 입력 복구](STREAM_DECK_FN_FIX.md)에서 Stream Deck 발신 Fn을 허용한다. 아래는 최초 필터 도입 당시 기록이다.

2026-09-10.

## 원인 확인

사용자 Control+I/J/K/L 입력을 로컬 CGEvent listen-only tap으로 관찰했다. BetterTouchTool 프로세스가 방향키 down/up(keycode 123~126, flags 0x20a00000)을 보낸 직후 Fn(keycode 63) flagsChanged down(0x800000)/up(0)도 생성했다. 이때 source PID는 BetterTouchTool이고, 일반 키보드 Control/Esc는 PID 0이었다. 실제 Fn 누름 여부를 CGEventSource.keyState만으로 판단하면 합성 Fn에도 true가 반환되어 이 문제를 막지 못한다.

기존 native_hotkey의 Fn 처리부는 keycode/flags만 보고 이 합성 신호를 녹음 단축키로 전달했다. 원인은 Control+문자 단축키가 앱에 등록됐기 때문이 아니다.

## 변경

- H 전용 fn_event 판정으로 프로그램이 생성한 Fn base 전환(source PID > 0)을 무시한다. 실제 하드웨어 Fn 이벤트와 다른 키의 단축키 처리는 유지한다. BetterTouchTool 설정을 수정하지 않는다. 프로그램으로 Fn을 의도적으로 합성하여 녹음을 호출하던 방식도 이 필터 대상이다.
- 설정의 Ask 단축키/시험 버튼과 기존 시작 안내의 Ask 안내를 제거했다.
- H product_scope에서 설정 읽기/저장 시 legacy Ask 키와 Ask binding 목록을 비운다. 기존 설정 파일에 남아도 재시작 후 등록되지 않으며 다음 저장 시 정리된다.
- Ask 녹음/답변 진입점도 차단한다. 원본 Ask 내부 구현은 upstream 병합을 위해 보존한다.

## 검증

설정 UI는 제거된 Ask 단축키가 받아쓰기와 충돌하지 않는 동작까지 검사한다. Fn 필터는 관찰한 합성 이벤트와 source PID 0의 실제 키 구분을 검사한다. 설치 및 최종 검사 결과는 HANDOFF의 최신 항목에 기록한다.

실제 사용자 재매핑 입력과 하드웨어 Fn 동작은 설치 후 확인해야 한다. 개발 도구가 합성한 Fn은 이제 의도적으로 무시되므로 기존 CGEvent 기반 녹음 자동화가 실제 Fn 검증을 대체하지 않는다.

설치 후 사용자가 Control+I/J/K/L 및 실제 Fn 확인 요청에 “이제 정상인것같아”라고 정상 동작을 확인했다. Rust 642/설정 UI 65/린트/빌드 통과.
