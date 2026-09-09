# Built-in speech recognition notices

No model weights are redistributed inside H-OpenTypeless. Downloads are optional and fetched directly from pinned Hugging Face revisions with SHA-256 verification. Model authors retain their rights.

- whisper.cpp v1.8.3: https://github.com/ggml-org/whisper.cpp — MIT. Its license is bundled as LICENSE-whisper.txt.
- Whisper models: https://github.com/openai/whisper — MIT. Converted multilingual weights: https://huggingface.co/ggerganov/whisper.cpp.
- qwen-asr native C engine, commit b00b789b17051aea61e9717458171100662318a4: https://github.com/antirez/qwen-asr — MIT. Its license is bundled as LICENSE-qwen-asr.txt.
- Qwen3-ASR-0.6B and Qwen3-ASR-1.7B: https://huggingface.co/Qwen/Qwen3-ASR-0.6B and https://huggingface.co/Qwen/Qwen3-ASR-1.7B — Apache License 2.0: https://www.apache.org/licenses/LICENSE-2.0.

H's native engine integration is independent of OpenAI and Alibaba/Qwen. Memory recommendations in the app are H's conservative estimates, not upstream hardware guarantees.
