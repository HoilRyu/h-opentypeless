# 한국어 다듬기 평가

실제 Rust 프롬프트 빌더의 출력과 로컬 Ollama를 사용한다. `korean.json`은 **원음 없는 텍스트** 16개다. 기존 실패의 재구성, 기존 회귀 예시, 새 합성 문장을 구분했다. 실제 사용자 발화가 확보되면 출처를 구분해 별도 추가한다. 이 결과를 STT 정확도나 Typeless 대비 성능으로 해석하지 않는다.

```sh
H_POLISH_FIXTURES=/tmp/h-polish-prompts.json cargo test --manifest-path src-tauri/Cargo.toml --lib h_export_polish_fixtures -- --ignored
python3 scripts/h-polish-eval.py --prompts /tmp/h-polish-prompts.json --candidate evaluation/polish/speech-act-candidate.txt --output /tmp/h-polish-evaluation
python3 -m unittest discover -s scripts/tests -p test_polish_eval.py
```

모델 기본값은 설치된 `gemma4:12b`다. `--model`, `--port`, `--repeats`를 지정할 수 있다. 127.0.0.1의 Ollama만 호출하며 API 키·설치 앱 설정·개인 기록을 읽지 않고 모델을 다운로드하지 않는다. 기존 출력 폴더를 덮어쓰지 않는다. 생성 실패도 결과 행에 기록하므로 `summary.json`의 flagged와 각 행 issues를 확인한다.

`manifest.json`에는 전체 조합 프롬프트·모델 digest·생성 설정·평가 문장과 hash를 남긴다. `results.jsonl`에는 각 요청 hash·출력·종료 이유·시간·자동 검사 결과를 기록한다. baseline/candidate 순서를 번갈아 실행하지만 정밀한 속도 벤치마크는 아니다. export fixture는 일반 문맥, 고정 사전·개인 지침을 사용하며 실행 중 앱 요청 캡처가 아니다. 현재 도구는 비번역 받아쓰기에 한정한다.

자동 required/forbidden 검사는 필수 부분 문자열 확인일 뿐, 의미 평가가 아니다. 특히 ‘최소 깔끔하게 구조화하고 전문적으로…’는 네 단어를 포함해도 메뉴 이름을 훼손한 실패다. 원문·조건·화행·말투·요청 누락은 별도 검토한다. `review`에 적힌 기준과 원문을 함께 읽고, 사람이 작성한 정답 문자열 한 개와의 완전 일치만으로 점수를 만들지 않는다.

2026-09-10 실행: [원본 결과](results/2026-09-10/results.jsonl), [설정](results/2026-09-10/manifest.json), [Codex 검토](results/2026-09-10/review.json).

- 16문장 × 두 조건 × 1회 = 32회. 유료 API/개인 음성 사용 없음.
- 자동 flagged: 기존 1/16, 후보 0/16.
- 실제 텍스트 검토에서는 **양쪽 모두 조건부 요청의 반말→존댓말 전환과 선택지 이름 훼손**이 남았다.
- 원음이 없어 반복 강조 보존 등은 판단이 제한된다. 검토자는 Codex이며 독립된 사람의 블라인드 평가가 아니다.
- 후보의 기본 적용은 보류했다. `speech-act-candidate.txt`는 실험용 추가 규칙이며 제품 프롬프트에 들어가지 않았다. 기존 예시가 포함된 소표본 1회 결과로 통계적인 개선을 주장하지 않는다.

다음 평가에서는 실제 실패 발화를 수집하고, 높임·화행·선택지 이름을 각각 구분한 미사용 문장과 반복 실행을 추가한다. 실험용 규칙을 제품에 적용하려면 실제 빌더에서 같은 조합이 생성되는지도 확인해야 한다.

## 긴 발화 구조화 평가

`structure.json`은 기존 16개와 새로 작성한 10개를 합친 텍스트 평가 자료다. 카드뉴스 사례는 [영상](https://www.youtube.com/watch?v=8yCyUD3Nsuk&t=580s)의 입력 구조를 참고해 다른 내용으로 작성했다. 영상의 실제 음성 인식 결과나 Typeless 출력 복제본이 아니다. 제품의 few-shot 예시는 다른 주제로 작성했다. 개발 중 이 평가를 보고 수정했으므로 최종 결과는 미사용 자료의 블라인드 평가가 아니다.

`--candidate-prompts`로 **변경된 Rust 빌더에서 내보낸 완전한 프롬프트**를 비교할 수 있다. 규칙을 뒤에 덧붙이는 `--candidate`와 동시에 사용할 수 없다. 단독 반복 검증에는 `--label revised --repeats 2`를 사용한다.

```sh
# 변경 전과 변경 후 각각 같은 exporter를 실행해 before.json / after.json을 준비한다.
python3 scripts/h-polish-eval.py --prompts before.json --candidate-prompts after.json --corpus evaluation/polish/structure.json --output /tmp/structure-comparison
python3 scripts/h-polish-eval.py --prompts after.json --label revised --corpus evaluation/polish/structure.json --repeats 2 --output /tmp/structure-repeat
```

`shape`는 최소 목록·구획 수와 평문 여부의 보조 검사다. 관계가 올바른지, 조건이 맞는 요청에 붙었는지, 선택지가 이름으로 보존됐는지는 자동 판정하지 않는다. 제목 감지는 휴리스틱이며 다른 유효한 형식을 놓칠 수 있다. 원문과 `review` 기준을 함께 읽어 검토해야 한다. 반복 출력이 같은 경우도 독립적인 성공 확률로 해석하지 않는다.

최종 구현의 Ollama 일반 구조화 받아쓰기를 재현할 때는 `--temperature 0`을 지정한다. 기본 `0.3`은 이전 비교를 재현하기 위해 남겼다. `structure-holdout.json`은 처음에는 별도 자료였지만 발견한 오류를 보완하는 데 사용했으므로 최종 시점에는 회귀 자료다. 결과와 한계는 [구현 기록](../../docs/fork/STRUCTURED_DICTATION_IMPLEMENTATION.md)과 [검토 기록](results/2026-09-10-structure/review.json)에 남긴다.
