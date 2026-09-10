# 음성 단계별 평가

20개 한국어 발화(개발 15, 보류 평가 5)로 음성 인식과 문장 가공을 구분한다.
숫자·고유명사·부정·정정·순서·역할·불확실성·한영 혼용을 포함한다.

## 범위와 판정

`h-voice-eval.py`는 명시적으로 지정한 WAV만 읽는다. 마이크를 녹음하거나 다른 앱에 입력하지 않는다.
설치된 Whisper MLX worker와 로컬 Ollama를 사용한다. 원본 WAV 해시, 모델 가중치 해시,
worker/평가 도구/프롬프트 해시, 단계별 텍스트와 시간을 저장한다.
프롬프트는 Rust `h_export_polish_fixtures`로 내보낸 고정 평가 문맥이다.
실제 사용자의 사전·장면·개인 설정 전체를 재현한 것이 아니다. temperature=0, reasoning_effort=none을 명시한다.

- CER은 공백·문장부호를 제외한 문자 오류율이다. 숫자 표기 차이도 오류로 셀 수 있다.
- required/forbidden/줄바꿈 검사는 검토 대상을 찾는 보조 지표다. 의미 보존 합격 판정이 아니다.
- `--reference-polish`는 정답 전사도 같은 프롬프트에 넣는다. STT 원문에 이미 있는 오류인지,
  정확한 원문을 넣어도 LLM이 바꾸는지 함께 비교한다.
- 모든 결과는 `semantic_verdict=unreviewed`, `insertion_verdict=not_tested`로 시작한다.
- `--inserted`는 실제 대상 앱에서 확인한 텍스트를 case ID별 JSON으로 제공하는 선택 기능이다.
  제공하지 않은 사례를 통과로 처리하지 않는다. 정확히 같은 실행 결과인지도 사람이 확인해야 한다.
- 합성 음성 결과는 마이크, VAD, 녹음 끊김, 실제 억양, 채팅창 포커스/붙여넣기 검증을 대체하지 않는다.

## 실행

워크트리 루트에서 시작한다. 모든 출력 디렉터리는 새 경로여야 한다.

```sh
python3 scripts/h-voice-eval.py prepare-synthetic --split all --audio-dir /tmp/h-voice-audio
H_POLISH_FIXTURES=/tmp/h-voice-prompts.json cargo test --manifest-path src-tauri/Cargo.toml --lib h_export_polish_fixtures -- --ignored
python3 scripts/h-voice-eval.py run --split development \
  --audio-dir /tmp/h-voice-audio \
  --runtime '/Applications/H-OpenTypeless.app/Contents/Resources/local-stt/mlx' \
  --stt-model "$HOME/Library/Application Support/dev.hoilryu.hopentypeless/local-stt/large-v3-turbo" \
  --prompts /tmp/h-voice-prompts.json --reference-polish --output /tmp/h-voice-results
```

`prepare-synthetic`는 macOS Yuna 음성을 파일로 생성하며 스피커로 재생하지 않는다.
`run`은 모델 다운로드를 하지 않는다. 실행 전 해당 STT 모델과 Ollama 모델이 설치되어 있어야 한다.
최종 평가용 holdout은 조정이 끝난 뒤 별도 실행한다. 이미 결과를 보고 수정했다면 새로운 보류 사례를 추가한다.

## 실제 음성으로 이어서 확인할 항목

1. cases.json 발화를 사용자가 평소 속도로 말해 `<id>.wav`로 준비한다(16 kHz, mono, PCM16).
2. provenance.json에 `kind: human_recorded`, `samples: {case_id: WAV SHA256}`를 기록한다.
3. 앱에서도 같은 발화를 녹음하여 최종 입력을 편집기와 평소 채팅창에서 확인한다.
4. 각 사례의 조건·숫자·이름·부정·정정·말투를 대조하고, 의미 변형 여부와 구조 적절성을 직접 판정한다.
5. 잘못된 입력창, 줄바꿈 손실, 한글 조합, 중복 붙여넣기, 장치 전환을 별도로 기록한다.

평가 음성과 결과는 명시적으로 제공한 자료만 사용한다. 개인 녹음은 저장소에 커밋하지 않는다.
