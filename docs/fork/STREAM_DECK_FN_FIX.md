# Stream Deck Fn 입력 복구 (2026-09-10)

## 확인한 원인

사용자가 Stream Deck 버튼을 누를 때 Fn flagsChanged down/up이 source PID 1012로 도착했다.
이 PID는 `/Applications/Elgato Stream Deck.app/Contents/MacOS/Stream Deck`이었다.
실제 키보드 Fn은 동작했으나 c311398의 source PID > 0 필터가 Stream Deck 입력까지 차단했다.

## 수정

Fn 이벤트마다 macOS proc_pidpath로 발신 실행 파일을 확인한다. Elgato Stream Deck 앱의
주 실행 파일에서 발생한 Fn은 누름과 해제 모두 허용한다. PID를 고정하거나 캐시하지 않아
프로그램 재시작과 PID 재사용에 영향을 받지 않는다. /Applications 이외 설치 위치도 허용한다.
BetterTouchTool 및 식별되지 않는 다른 프로그램의 합성 Fn 차단은 유지한다.
프로세스 경로 조회 실패 시 기존 차단 정책을 유지한다. 보안 인증 기능이 아닌 입력 호환성 규칙이다.
Stream Deck에서 BetterTouchTool을 거쳐 Fn을 생성하는 별도 구성은 이번 예외에 포함되지 않는다.

## 검증

- 관찰된 Stream Deck 발신 경로, 변경된 PID, 사용자 Applications 경로의 허용 검사.
- BetterTouchTool 합성 Fn 차단, 실제 Fn 및 일반 키 유지, 조회 실패와 유사 경로 차단 검사.
- macOS에서 실제 프로세스 경로 조회 검사.
- 설치 후 실제 버튼으로 녹음이 시작되는지 사용자 확인 필요.
