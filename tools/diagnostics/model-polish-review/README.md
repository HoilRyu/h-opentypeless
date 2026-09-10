# 동일 프롬프트의 Gemma / Qwen 비교

2026-09-10. 설치된 Gemma4:12b와 Qwen3.5:9b만 사용한다. 모델을 새로 다운로드하거나 앱 설정을 바꾸지 않는다.

- 기준: 직전 `prompt-length-review/results.json`의 current 프롬프트 Gemma 결과 36개. 프롬프트 SHA-256 일치 확인 후 복사한다. 같은 시각 교차 실행한 새 Gemma 시험은 아니므로 시간/시스템 부하를 통제한 속도 벤치마크가 아니다.
- 비교: 동일 12문장을 Qwen에 각 3회(36회). system/user 메시지, temperature 0.3, reasoning_effort none, max_tokens 4096, 비스트리밍 API 동일. Qwen 출력에 reasoning 필드가 나오면 비교를 중단한다.
- 모델명뿐 아니라 Ollama 태그/digest/양자화 메타데이터를 기록한다. 모델 계열 전체의 성능을 대표하지 않는다. 파라미터 규모도 다르다.
- 순차 사용: Gemma를 메모리에서 해제하고 Qwen 시험 후 Qwen 해제/Gemma 재준비. 사용자의 STT/LLM 선택과 프롬프트를 바꾸지 않는다.
- 실제 마이크/STT 경로는 비교하지 않는다. 재구성 전사문/일반 문맥 fixture이며, 합성 STT 오류 1사례 포함.

평가는 앞선 비교 기준 그대로: 높임 수준, 내용·조건·선택지 보존, 구조화, 질문에 답하지 않는 역할 유지. 명사형 문체 변경과 존댓말 전환은 구분한다. 각 조건 3회이므로 일반 정확도나 통계적 우열을 주장하지 않는다. 실제 사용에서 이미 관찰한 Gemma의 말투 전환을 이번 표본의 통과가 부정하지 않는다.

실행: `python3 tools/diagnostics/model-polish-review/run.py`. 결과 파일을 덮어쓴다.
