# Android 0.3.0 디자인 적용

2026-09-09 · H-OpenTypeless Android · S26 Ultra SM-S948N / Android API 36

데스크톱의 차콜 배경, 민트 강조색, 얇은 테두리와 둥근 카드 스타일을 Android 네이티브 화면에 적용했다. 이미지 시안의 구성을 실제 기능과 터치 영역에 맞춰 조정했다. 기능 아이콘은 해상도에 관계없이 선명한 벡터로 구현했다. 글꼴은 Android 사용자 설정을 따른다.

- 홈: 처리 방식을 직접 선택한다. 로컬에서는 기기·STT·교정 모델만, 원격에서는 컴퓨터 주소·연결 확인만 표시한다.
- 모델 관리: 음성 인식·문장 교정·교정 지침 탭과 모델 상세 화면으로 분리했다. 기존 다운로드·이어받기·가져오기·무결성 확인·실행 확인·삭제 및 기기 지원 판정을 연결했다. 진행 중에는 표시와 취소 버튼을 제공한다.
- 설정·도움말: 마이크·키보드 준비와 사용 방법을 안내한다. 선택한 처리 방식에 해당하는 도움말을 표시한다.
- 키보드: 원문·교정문을 함께 표시하고 사용할 문장을 선택해 삽입한다. 긴 결과 영역은 스크롤하며 녹음·삽입·취소 버튼은 고정한다. 기본 STT의 수동 종료와 기존 원격/Whisper 119초 제한을 유지한다.

## 확인

- assembleDebug / assembleDebugAndroidTest / testDebugUnitTest 성공, 단위 테스트 31개 통과.
- S26 Ultra에 0.3.0 업데이트 설치. 테스트는 기존 처리 방식 선택을 복원하며 모델 파일을 변경하지 않는다.
- 기기 instrumentation: 로컬/원격 정보 분리, 화면 렌더링, 원문/교정문 삽입 값, 녹음 종료·처리 취소 버튼 검사 통과.
- 아래 이미지는 S26 Ultra에서 앱 View를 직접 렌더링한 결과다. 키보드 예문은 UI 검사 입력이며 실제 음성 추론 결과가 아니다. 이번 검사는 디자인과 UI 동작에 집중했으며 모델 정확도와 실제 원격 통신을 다시 측정하지 않았다.

## 실제 화면

| 화면 | 이미지 |
| --- | --- |
| 로컬 홈 | [보기](implemented/local.png) |
| 원격 홈 | [보기](implemented/remote.png) |
| 모델 목록 | [보기](implemented/models.png) |
| 모델 상세 | [보기](implemented/detail.png) |
| 교정 지침 | [보기](implemented/preferences.png) |
| 사용 설정 | [보기](implemented/settings.png) |
| 듣는 중 | [보기](implemented/listening.png) |
| 결과 선택·삽입 | [보기](implemented/result.png) |
