# 내장 Ollama LLM 구현 및 검증

작성일: 2026-09-11
브랜치: `HoilRyu/local-llm-ollama`
기준 커밋: `40d249df84dd8262d53599772943e8e00c6e958f`
상태: 구현 및 테스트 앱 설치 완료. 사용자 승인에 따라 통합 브랜치 병합 대상으로 정리.

## 사용자 흐름

AI 다듬기 설정에서 **내장 로컬 LLM** 또는 **외부 API 연결**을 선택한다.
내장 모드에서는 모델 선택 → 다운로드 → 이 모델 사용 순서로 설정한다.
목록을 탐색하거나 다운로드하는 것만으로 사용 모델을 변경하지 않는다.
초기 탐색 항목은 사용자가 사용 중인 12B이며, 실제 사용 모델은 명시적으로 선택해야 한다.

| 표시 이름 | 고정 모델 태그 | 전체 다운로드 크기(약) |
|---|---|---|
| Gemma 4 E2B | `gemma4:e2b` | 7.16 GB |
| Gemma 4 E4B | `gemma4:e4b` | 9.61 GB |
| Gemma 4 12B | `gemma4:12b` | 7.56 GB |

크기는 카탈로그의 모든 blob 크기를 합산한다. 공유 blob이 이미 있으면 실제 추가 다운로드는 작아진다.
E2B/E4B의 유효 파라미터 수는 파일 크기나 필요한 RAM을 뜻하지 않는다.
세 모델의 정확한 manifest와 blob SHA-256 및 크기는
`src-tauri/src/extensions/local_llm/catalog.json`에 고정했다.
임의 모델명·URL·클라우드 모델을 입력하여 다운로드하는 경로는 제공하지 않는다.

## 구현

- Ollama **0.34.0** macOS 런타임을 앱 리소스 `local-llm`에 포함한다.
  공식 아카이브 SHA-256은 `dd12b00bcce2d6551178e67ada90d5af9f75bdb54a118b96655250fa3e8ef734`이다.
  엔진과 필요한 라이브러리·라이선스를 함께 배치하고 Mach-O 파일을 앱 서명 전에 서명한다.
- 현재 지원 대상은 macOS Apple Silicon이다. 엔진이 없는 개발 실행에서는 다운로드·실행 기능이 비활성화된다.
- 가중치는 앱에 포함하지 않는다. 앱 데이터의 `local-llm/models`에 저장하므로 앱 업데이트와 분리된다.
  기존 `~/.ollama/models`, 외부 Ollama 서버, 시스템 환경 변수는 변경하지 않는다.
- 가변 태그를 `/api/pull`에 전달하는 대신 고정 digest의 registry blob을 직접 다운로드한다.
  취소 시 `.part`를 보존하며 재시도 시 Range 이어받기를 한다. 전체 크기와 SHA-256 검증 후 파일을 공개하고 manifest를 마지막에 저장한다.
  모델 선택/첫 추론 시 무결성을 확인하며 같은 파일의 크기·수정 시각이 유지되는 동안 결과를 캐시한다.
- 다운로드·선택·삭제·추론을 직렬화한다. 사용 중 변경 요청은 busy 오류로 거절한다.
- 별도 loopback 포트에서 앱 소유 엔진을 실행한다. 클라우드 기능을 끄고 모델 경로·컨텍스트 8192·동시 요청 1·상주 모델 1개를 지정한다.
  시작 시 고정 버전 응답과 자식 프로세스 생존을 확인한다. 새로 서명한 실행 파일의 첫 기동 지연을 고려해 최대 60초 기다린다. 포트 충돌로 자식이 종료되면 오류로 처리한다.
  로컬 HTTP 클라이언트는 프록시와 리다이렉트를 사용하지 않는다. loopback 자체는 다른 로컬 프로세스에 대한 인증 경계는 아니다.
- 기존 Ollama 다듬기 프로토콜과 프롬프트·스트리밍을 재사용한다. 호출 시 주소와 모델을 앱 소유 값으로 강제하고 API 키를 비운다.
  내장 실패 시 외부 API로 자동 전송하지 않는다.
- `llm_external_provider`에 직전 외부 제공자를 보존한다. 외부 주소·모델 필드는 유지하고 저장된 키도 기존 제공자에 연결한다.
- C supervisor가 엔진 프로세스 그룹을 소유한다. 정상 종료·부모 강제 종료·요청 취소 시 엔진 정리를 수행한다.
  성공한 요청은 런타임을 재사용하고 120초 유휴 후 정리한다(10초 간격 확인). 수동 메모리 해제 버튼도 제공한다.
- 화면에서 진행률·취소·오류·설치 상태·사용 모델·연결 검사·삭제·메모리 해제를 제공한다.

## 검증 결과

- 병합 전 전체 Rust 회귀 검사: 665개 통과, 10개 명시적 제외.
- 병합 전 전체 화면 검사: 58개 파일, 501개 통과. 내장/외부 전환 시 연결 설정 보존 검사 포함.
- TypeScript/Vite 빌드 및 ESLint 통과. Tauri 앱 번들 생성과 서명 검증(`codesign --verify --deep --strict`) 통과.
- 서명된 앱 리소스 경로에서 12B 실제 추론·한국어 다듬기·메모리 해제 검사 통과. 첫 기동 지연으로 기존 시작 제한을 넘는 사례를 확인해 대기 시간을 60초로 보완했다.
- supervisor 테스트에서 정상 종료와 부모 SIGKILL 양쪽 모두 엔진과 하위 프로세스 종료 확인.
- 작은 실제 registry config blob의 다운로드·이어받기·SHA-256 검사 통과.
- 사용자 기존 12B 파일을 APFS 복제로 분리된 테스트 폴더에 준비하고, 새 내장 런타임으로 실제 추론 확인.
  첫 측정의 콜드 요청은 약 27.7초, 재사용 요청은 약 0.6초였다. 장비·캐시 상태에 따라 달라진다.
- 기존 다듬기 제공자 경로로 한국어 입력을 처리하여 “오후 3시”, “2층”, “취소하지 말고” 의미 보존 확인.
  테스트의 외부 주소를 도달 불가능한 주소로 지정하여 내장 주소 사용도 확인했다.

실제 테스트 자산은 `~/.local/share/h-opentypeless/local-llm-test`에 있으며 기존 앱 설정과 분리되어 있다.
빌드 폴더는 `~/.local/share/h-opentypeless/local-llm-build-source`를 사용한다.
검증용 앱은 빌드 폴더의 `src-tauri/target/debug/bundle/macos/H-OpenTypeless.app`이다.
기존 설치 앱의 STT 리소스와 credential helper를 재사용하고 새 LLM 런타임을 포함한 debug 산출물이다.
정식 빌드 스크립트에는 STT 준비 다음에 LLM 준비를 수행하도록 연결했다.
실제 추론 검사는 환경 변수가 필요한 ignored Rust 테스트로 남겼다.

```sh
H_LLM_TEST_ROOT=/path/to/isolated-model-root \
H_LLM_TEST_ENGINES=/path/to/H-OpenTypeless.app/Contents/Resources/local-llm \
cargo test --manifest-path src-tauri/Cargo.toml real_bundled_model_test --lib -- --ignored --nocapture
python3 scripts/tests/test_llm_supervisor.py
```

첫 테스트는 카탈로그와 일치하는 12B 모델이 준비된 별도 경로에서만 실행한다.
사용 모델을 12B로 지정하고 추론 후 메모리를 해제하므로 실제 앱 데이터 경로를 전달하지 않는다.

## 남은 실사용 검증

E2B/E4B 전체 가중치 다운로드와 실제 추론은 아직 실행하지 않았다.
12B의 전체 네트워크 다운로드도 수행하지 않았으며, 다운로드 프로토콜은 작은 실제 blob으로 검증했다.
STT와 LLM을 동시에 사용하는 장시간 녹음, 메모리·GPU 사용량, 다양한 한국어 문장 품질,
디스크 부족 및 다운로드 중 앱 강제 종료에 대한 실사용 검증은 별도로 필요하다.
따라서 기본 모델이나 최소 RAM을 성능 측정 없이 권장값으로 확정하지 않는다.
Ollama 번들에는 여러 백엔드가 포함되므로 실제 선택된 GPU 백엔드의 성능은 별도 측정 대상이다.

## 근거

- https://github.com/ollama/ollama/releases/tag/v0.34.0
- https://github.com/ollama/ollama/blob/v0.34.0/LICENSE
- https://docs.ollama.com/faq
- https://docs.ollama.com/api/openai-compatibility
- https://docs.ollama.com/macos
- https://ollama.com/library/gemma4:e2b
- https://ollama.com/library/gemma4:e4b
- https://ollama.com/library/gemma4:12b

## 설치 후 확인한 성능 차이

사용자가 내장 모드에서 지연 증가를 보고했다. 당시 기존 외부 Ollama와 내장 Ollama는
모두 동일 digest의 Gemma 4 12B GGUF Q4_K_M을 llama.cpp/Metal GPU 경로로 실행했다.
STT는 MLX worker를 사용했다. LLM을 MLX에서 다른 백엔드로 전환한 경우는 아니었다.
다만 외부 엔진과 내장 엔진의 버전·Flash Attention·KV 캐시·배치 크기·유휴 정책은 다르다.
동일 모델이 두 엔진에 각각 약 8 GB씩 상주하고 시스템 스왑도 사용 중임을 관찰했다.
이것만으로 지연 원인을 확정할 수는 없으며, 중복 상주를 제거하고 동일 옵션으로 비교하는
성능 검증은 후속 과제다. 이번 통합은 기존 환경과의 성능 동등성을 보장하지 않는다.
