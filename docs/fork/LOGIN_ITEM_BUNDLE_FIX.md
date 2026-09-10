# 로그인 항목을 H-OpenTypeless 앱 번들로 등록

기존 `tauri-plugin-autostart`의 LaunchAgent는 앱 내부 `opentypeless` 실행 파일을 직접 등록했다.
앱 번들의 CFBundleName/CFBundleDisplayName은 H-OpenTypeless였지만, 시스템 설정의
백그라운드 활동에는 실행 파일 이름과 확인되지 않은 개발자로 표시됐다.
AssociatedBundleIdentifiers를 추가해도 이 환경의 표시 이름은 바뀌지 않았다.

macOS 13 이상에서 설치된 앱은 ServiceManagement의 SMAppService.mainAppService로 등록한다.
이에 따라 시스템 설정의 ‘로그인 시 열기’에서 실제 앱 번들을 관리한다.
macOS 구버전과 개발 실행 파일, Windows/Linux는 기존 플러그인을 유지한다.

- 시작 시 기존 설정과 등록 상태를 확인하고, 활성화된 이전 LaunchAgent만 이전한다.
- 새 등록이 성공하기 전에는 이전 plist를 제거하지 않는다.
- 끄기는 새 등록과 이전 등록을 모두 해제한다.
- 승인 대기 상태를 활성화 성공으로 처리하거나 자동으로 우회하지 않는다.
- 다른 앱의 로그인 항목 및 시스템 BTM 데이터베이스는 초기화하지 않는다.

구현: `src-tauri/src/extensions/auto_start.rs`, 설정 명령과 시작 시 동기화에서 공통 호출.
참고: https://developer.apple.com/documentation/servicemanagement/smappservice

## 검증 및 적용

- 자동 실행 관련 Rust 테스트 4개 통과, TypeScript/Vite 빌드 통과.
- 실제 앱에서 끄기 → 저장 → 켜기 → 저장을 검증하고 최종 켜짐으로 복원.
- macOS API 성공 직후 이전 status가 잠시 반환되는 상황을 확인해, 이 전파 지연을
  저장 실패로 처리하지 않도록 수정했다. 최초 수정본에서 저장을 두 번 해야 하던 문제도 해소.
- 시스템 설정 ‘로그인 시 열기’에서 `H-OpenTypeless / 응용 프로그램` 표시 확인.
- sfltool에서 `2.dev.hoilryu.hopentypeless`, URL `/Applications/H-OpenTypeless.app/`,
  disposition enabled/allowed 확인.
- 기존 `~/Library/LaunchAgents/H-OpenTypeless.plist` 제거 확인. launchctl에도 해당 서비스 없음.
  최종 시스템 설정 확인에서 과거 `opentypeless` 백그라운드 항목도 사라졌다.
  다른 앱에 영향을 주는 BTM 전체 초기화는 수행하지 않았다.
- 설치본의 기존 서명·인증 helper·Whisper MLX 리소스를 보존하고 서명 검증했다.
- 원본 앱 백업: `~/.local/share/h-opentypeless/app-backups/before-login-item-fix-20260910-202218.app`.
- 최초 LaunchAgent 백업: `~/.local/share/h-opentypeless/login-item-fix/before-20260910-201424.plist`.
- 실제 로그아웃/재로그인은 진행하지 않았으며 OS 등록 상태와 앱의 켜기/끄기 동작을 검증했다.
