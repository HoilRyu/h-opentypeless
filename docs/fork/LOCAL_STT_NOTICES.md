# Built-in speech recognition notices

No model weights are redistributed inside H-OpenTypeless. Downloads are optional and fetched directly from pinned Hugging Face revisions with SHA-256 verification. Model authors retain their rights.

- whisper.cpp v1.8.3: https://github.com/ggml-org/whisper.cpp — MIT. Its license is bundled as LICENSE-whisper.txt.
- Whisper models: https://github.com/openai/whisper — MIT. Converted multilingual weights: https://huggingface.co/ggerganov/whisper.cpp.
- qwen-asr native C engine, commit b00b789b17051aea61e9717458171100662318a4: https://github.com/antirez/qwen-asr — MIT. Its license is bundled as LICENSE-qwen-asr.txt.
- Qwen3-ASR-0.6B and Qwen3-ASR-1.7B: https://huggingface.co/Qwen/Qwen3-ASR-0.6B and https://huggingface.co/Qwen/Qwen3-ASR-1.7B — Apache License 2.0: https://www.apache.org/licenses/LICENSE-2.0.

H's native engine integration is independent of OpenAI and Alibaba/Qwen. Memory recommendations in the app are H's conservative estimates, not upstream hardware guarantees.

## macOS arm64 MLX runtime

The app includes a private Python runtime, not an externally hosted STT service.
- CPython 3.12.14 (Python Software Foundation License); distribution from [python-build-standalone](https://github.com/astral-sh/python-build-standalone).
- [MLX](https://github.com/ml-explore/mlx) 0.31.1, MIT.
- [mlx-qwen3-asr](https://github.com/moona3k/mlx-qwen3-asr) 0.4.0, Apache-2.0.
- Pinned dependencies and artifact hashes: `scripts/mlx/requirements.lock`. License texts/metadata remain inside the runtime's Python distribution and package dist-info directories.
- Qwen tokenizer configuration files use the same pinned model revisions and Apache-2.0 license as the weights. Their provenance and SHA-256 values are in `native/mlx-stt/tokenizers.json`.

MLX GPU support here applies to compatible Apple Silicon Macs; it does not advertise Qwen GPU support on Windows or Linux.


## Earshot 1.2.2 — voice activity detection

https://github.com/pykeio/earshot — distributed under the MIT option.

MIT License

Copyright (c) 2025-2026 pyke.io

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## 2026-09-10 Whisper Large-v3 Turbo 카탈로그 추가

기존 Whisper와 동일한 OpenAI Whisper MIT 모델 계열이며, whisper.cpp 배포의 고정된 GGML 파일을 선택 다운로드한다. 배포 URL·크기·SHA-256은 catalog.json에 포함한다. 엔진/가중치 라이선스 처리는 기존 Whisper와 동일하다.

- mlx-whisper 0.4.3: https://github.com/ml-explore/mlx-examples/tree/main/whisper — MIT, Apple Inc. Package license and dependency licenses are retained in the bundled Python distribution. The local GGML adapter follows whisper.cpp's tensor format and MLX Whisper's tensor mapping without downloading additional weights.
