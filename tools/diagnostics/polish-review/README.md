# 구조화 프롬프트 비교용 자료

2026-09-10. 앱 적용 전 검토 자료. 실행 중인 앱의 설정/프롬프트는 변경하지 않는다.

- `cases.json`: 대화에 등장한 개발 요청을 재구성한 10문장과 사람이 확인할 기준. 원본 음성 데이터셋이 아니며 STT 정확도 점수에 사용하지 않는다. `stt_error`는 관찰했던 전사 오류를 합성한 사례다.
- `baseline.txt`: 기존 Rust 프롬프트 빌더에서 내보낸 일반 문맥/구조화 fixture. 현재 소스의 STRUCTURED 본문으로 동기화. 콤보 박스 사전과 짧은 개발 개인 지침 포함. 실시간 데스크톱 요청 캡처는 아니다.
- `candidate.txt`: 중복을 줄인 독립적인 구조화 편집 프롬프트 제안. 원문/표현 보존보다 의미에 따른 재구성을 설명한다. 프로덕션 적용 시 기존 번역·선택 텍스트·사용자 장면 계약과 통합해야 한다. 전체 기존 프롬프트를 그대로 대체하는 배포 파일이 아니다.
- `run.py`: 로컬 Ollama의 gemma4:12b, temperature 0.3, reasoning_effort none에 각 문장을 두 방식으로 순차 요청한다. 외부 API/앱 기록/설정을 사용하지 않는다.
- `results.json`: 실제 출력과 요청 소요 시간. 시간은 엔진 준비 등 영향을 받으므로 속도 벤치마크가 아니다. 각 조합 1회로 품질 안정성을 보장하지 않는다.

실행: `python3 tools/diagnostics/polish-review/run.py` (기존 results.json을 덮어쓴다).
평가는 내용 누락/추가, 조건과 부정, 자연스러움, 목록 구분을 함께 본다. 글머리표 개수만으로 정답을 판정하지 않는다.

2차 후보: `candidate-v2.txt`는 선택지 개수/이름 및 독립 요청 누락 확인을 추가한다. 특정 인식 오류 치환표를 추가하지 않는다. `python3 tools/diagnostics/polish-review/run.py --v2`로 `results-v2.json`에 별도 기록한다.

3차 통합안: `structured-v3.txt`를 기존 `baseline.txt`의 구조화 본문에만 대입하고 복수 요청/완료 보고 예시 한 개를 추가한 것이 `candidate-v3.txt`다. 기존 공통/문맥/개인 지침 및 기존 예시를 유지한다. `run-v3.py`는 시작 시 프롬프트와 입력을 메모리에 고정하고 프롬프트 SHA-256을 결과에 기록한다. 기본 10문장: `python3 tools/diagnostics/polish-review/run-v3.py`. 취약 사례 반복: `python3 tools/diagnostics/polish-review/run-v3.py --ids multi choices deferred --repeat 2 --output results-v3-repeat.json`. 후보 파일만 변경하며 프로덕션에 적용하지 않는다.
