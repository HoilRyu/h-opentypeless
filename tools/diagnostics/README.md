# macOS 접근성 누수 대조군

이 진단 앱은 AppKit 창과 가짜 텍스트/버튼만 사용한다. H, CPAL, 마이크, STT/LLM, 네트워크, 사용자 설정을 사용하지 않는다. 프로덕션 앱에 포함하지 않는다.

`macos-ax-probe.swift`를 `swiftc -g -framework AppKit`으로 별도 `.app/Contents/MacOS/h-ax-probe`에 컴파일한다. Info.plist에는 CFBundleExecutable=h-ax-probe, 고유 CFBundleIdentifier, CFBundlePackageType=APPL, HProbeDirectory=진단 출력 디렉터리, LSEnvironment={MallocStackLogging:1,MallocStackLoggingNoCompact:1}을 지정한다. 만든 번들은 로컬 ad-hoc 서명하고 실행한다. 시스템 전역 환경변수나 H 설정을 변경하지 않는다.

1. UI를 읽은 초기 상태에서 `leaks --noContent --fullStacks PID`를 저장한다.
2. Replace view 버튼을 3회 누르고 매번 AX 트리를 읽는 대조군을 측정한다.
3. 새 프로세스에서 Run two batches를 한 번 누른다. 직후 한 번 상태를 읽는 것 외에는 두 구간 사이에 UI 자동화/스크린샷/AX 조회를 하지 않는다.
4. phase.txt=100이 되면 같은 leaks 명령으로 측정한다. 15초 휴식 뒤 다음100회가 진행되고 phase.txt=200에서 다시 측정한다. phase 파일은 앱이 직접 기록하므로 UI 조회가 필요 없다.
5. 녹음/실제 앱 활동과 섞인 비교는 무효 처리한다. 서로 다른 프로세스의 총량을 단순 차감해 녹음당 누수라고 보고하지 않는다. 시스템 NSXPC 일회성 후보와 지속 증가를 분리한다.
6. 완료 후 창을 닫아 진단 프로세스를 종료한다. MallocStackLogging은 진단 앱에만 적용되며 프로덕션 환경에 유지하지 않는다.

확인된 결과/한계: ../../docs/fork/AX_OBSERVER_INVESTIGATION.md.
