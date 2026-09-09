# 반복 마이크 실패 조사 — 2026-09-09

## 확인된 현상

입력창 기준 캡슐 배치 버전에서 사용자가 반복적인 ‘마이크 실패’를 보고했다. 당시 PID 62791의 sample은 CoreAudio 콜백과 run_capture가 살아 있음을 보여 주었다. h-audio-events.log에는 네 번째 녹음의 capture_start_confirmed 이후 capture_stop_requested/confirmed 없이 volume_restore_completed와 새 시도들이 반복됐다. 실패 당시 stderr는 보존되지 않아 정확한 최초 상태 전환/오류 메시지는 확인하지 못했다. 단순 마이크 권한 거부로 단정하지 않는다.

앱 종료 후 PID 64966으로 재실행해 점유를 해제했다. 사용자가 정상 입력을 확인했고, 추가 4.0초/8.9초 녹음은 시작·종료·음량 복원·STT 성공이 로그에 남았다. 재시작 복구를 근본 원인 해결로 간주하지 않는다.

## 코드에서 확인한 재발 가능 경로와 수정

1. stop()은 STT/LLM/히스토리 완료 전에 pipeline_lock을 해제했다. abort 후 새 start가 공용 abort_flag를 false로 되돌리면 이전 stop의 늦은 완료가 새 세션의 state를 Idle로 덮어쓸 수 있었다. 이는 마이크가 살아 있는데 상태가 Idle인 관찰과 맞는 후보이며, 과거 사건의 확정 원인은 아니다.
2. start/stop에 watch 세대 기반 취소를 적용했다. abort는 세대를 증가시키며 새 start가 과거 작업의 취소를 지울 수 없다. 취소 시 이전 future를 drop하여 후속 상태 변경을 중단한다.
3. stop의 잠금을 출력/히스토리 완료까지 유지한다. 취소 시 future가 drop되며 잠금이 해제되므로 새 녹음이 이전 LLM 제한시간 전체를 기다릴 필요가 없다.
4. start가 Idle→Preparing에 성공한 경우에만 abort_flag를 초기화한다. 거절된 중복 시작으로 취소를 되돌리지 않는다.
5. Idle에서 시작할 때 자신의 audio_handle이 남아 있으면 먼저 stop한다. 다른 Ask 세션의 마이크 점유나 확인되지 않은 네이티브 종료 보호를 무조건 해제하지 않는다.
6. 스트리밍 출력 worker를 소유자가 drop되면 abort하도록 했다. 취소된 stop의 백그라운드 출력이 남지 않게 한다.
7. 기존 제한 크기 h-audio-events.log에 dictation 상태/abort/마이크 점유 거절/남은 핸들 정리 이벤트를 추가했다. 음성 내용/키를 기록하지 않는다.

## 검증 및 범위

pipeline 테스트 38개 통과. 취소된 세대의 재개 방지, 대기 중 finalize 취소 후 잠금 해제, 출력 worker 취소 검사 포함. 전체 Rust 테스트/빌드/설치 결과는 HANDOFF.md에 기록한다. 네이티브 API 자체가 멈춘 경우 강제 중단을 보장하지 않는다. 실제 빠른 취소→재녹음과 장시간 사용 검증을 별도로 이어가야 한다. 최근 UI 배치 변경을 최초 원인으로 확정할 증거는 없다.
