# NSArray / AXObserverCookie 후보 분리 결과

2026-09-09, macOS26.6.2(25G83), /usr/bin/leaks. 기존 사용자 앱에 적용한 CPAL 문자열 누수/단일CaptureLease 수정은 유지했다. 이번 실험은 별도 프로세스이며 실제 H 앱/장치/음량을 변경하지 않았다.

## 재현 및 할당 위치

마이크와 H 코드가 없는 AppKit 진단 앱에서 동일 NSArray/AXObserverCookie 패턴을 재현했다. LSEnvironment로 이 진단 프로세스에만 MallocStackLogging을 켜고 --noContent --fullStacks로 수집했다.

스택:

```
NSWindow setContentView / NSView removeFromSuperview
_NSAccessibilityUnregisterUniqueIdForUIElementAndSendDestroyedNotification
_NSAccessibilityRemoveAllObserversAndSendDestroyedNotification
NSArray initWithArray:range:copyItems:
__NSArrayI_new
```

별도 스택은 macOS의 NSLocalWindowSharingWindowController 닫기에서 같은 함수로 들어간다. 배열 하위에 AXObserverCookie가 남는다. 이것은 AppKit 접근성/창 공유 내부 경로에서 동일 패턴이 발생한다는 직접 증거다. H 메모리그래프는 할당스택 없이 수집했으므로 과거 H 후보 각각의 호출자를 일대일 확정한 것으로 확대하지 않는다.

## 대조 결과

| 실행 | 초기 총 검출 | 뒤 총 검출 | 증가 |
|---|---:|---:|---:|
| UI/AX 조회하며3회뷰교체 |18,672B|19,232B|560B|
| 별도실행, 시작관찰후자동100회 |18,720B|19,168B|448B|
| 연속실행 첫100회 |18,816B|19,264B|448B|
| 같은실행, 관찰없이 다음100회 |19,264B|19,264B|0B|

총량에는 기존 NSXPC 등 초기화 후보도 포함된다. 연속실행에서는 AXObserverCookie 참조도 첫100회와다음100회에동일했다. 따라서 이 실험에서 지속적인 뷰교체당 누수는 입증되지 않았고, 접근성/창공유 관찰이 기존 수백바이트 후보 측정에 영향을 주는 것으로 좁혀졌다. VoiceOver 등 다른 관찰자를 모두 시험한 결과는 아니다.

## 조치와 한계

- 남은 NSArray 후보를 근거 없이 H의 녹음 메모리 누수로 집계하던 방식을 중단한다. 스택과 독립 대조군 없이 원인을 단정하지 않는다.
- 재현 소스를 tools/diagnostics/macos-ax-probe.swift에 보관하여 같은 비교를 재실행할 수 있게 했다. 실제 음성 입력과 UI검사가 섞인 이전 비교는 원인 판정에서 제외한다.
- H의 접근성을 끄거나, macOS 비공개 객체를 강제해제/swizzle하는 수정은 하지 않는다. OS가 소유한 배열의 수명을 앱에서 직접 관리할 수 없다. 이 실험은 Apple 내부 누수 자체를 패치했다는 뜻이 아니다.
- CPAL name() 누수와 중복 마이크는 이미 앱에서 우회/보호했다. 별도 OS 접근성 후보와 커널패닉 직접원인은 분리한다. 이번 결과로 커널패닉 해결을 확정하지 않는다.
- H/기존 오디오 설정/실행 파일 변경 없음. 진단 앱은 종료해 추가 관찰/계측을 남기지 않는다.

원본 증거: ~/.local/share/h-opentypeless/verification/ax-probe/{baseline,after-3,auto-baseline,auto-after-100,batch-0,batch-100,batch-200}.txt. 내용 덤프 없이 할당스택만 수집했다.
