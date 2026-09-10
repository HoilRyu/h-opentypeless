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
