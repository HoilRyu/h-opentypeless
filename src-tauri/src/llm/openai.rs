use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;

use crate::error::AppError;

use super::{
    prompt, protocol, ChunkCallback, LlmConfig, LlmProvider, PolishRequest, PolishResponse,
};

pub struct OpenAiProvider {
    client: Client,
    local_runtime: bool,
}

// Local structured dictation edits supplied content; reduce sampling variation.
// Scenes, translations, selected-text operations and remote providers keep their settings.
fn request_temperature(config: &LlmConfig, req: &PolishRequest) -> f64 {
    if config.provider.trim().eq_ignore_ascii_case("ollama")
        && req.polish_style == "structured"
        && req.voice_intent.kind == crate::voice_intent::VoiceIntentKind::DictateInsert
        && !req.translate_enabled
        && req
            .selected_text
            .as_ref()
            .is_none_or(|s| s.trim().is_empty())
        && req.mapped_scene_prompt.trim().is_empty()
        && req.active_scene_prompt.trim().is_empty()
    {
        0.0
    } else {
        config.temperature
    }
}

impl Default for OpenAiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAiProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            local_runtime: false,
        }
    }

    pub fn with_client(client: Client) -> Self {
        Self {
            client,
            local_runtime: false,
        }
    }
    pub fn with_local_client(client: Client) -> Self {
        Self {
            client,
            local_runtime: true,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn polish(
        &self,
        config: &LlmConfig,
        req: &PolishRequest,
        on_chunk: Option<&ChunkCallback>,
    ) -> Result<PolishResponse, AppError> {
        let has_selected_text = req
            .selected_text
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty());

        let system_prompt = prompt::build_context_system_prompt(prompt::ContextPromptOptions {
            context: &req.context,
            dictionary: &req.dictionary,
            correction_rules: &req.correction_rules,
            polish_style: &req.polish_style,
            personal_style_prompt: "",
            mapped_scene_prompt: &req.mapped_scene_prompt,
            active_scene_prompt: &req.active_scene_prompt,
            polish_custom_prompt: &req.polish_custom_prompt,
            translate_enabled: req.translate_enabled,
            target_lang: &req.target_lang,
            has_selected_text,
            voice_intent: Some(&req.voice_intent),
        });

        let system_prompt = if super::dictation_guard::enabled(req) && !has_selected_text {
            super::h_polish::with_relevant_examples(system_prompt, &req.polish_style, &req.raw_text)
        } else {
            system_prompt
        };
        let mut messages = vec![serde_json::json!({ "role": "system", "content": system_prompt })];
        if has_selected_text {
            messages.push(serde_json::json!({
                "role": "user",
                "content": format!("<selected_text>\n{}\n</selected_text>", req.selected_text.as_ref().unwrap())
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt::transcription_message(&req.raw_text, req.voice_intent.kind == crate::voice_intent::VoiceIntentKind::DictateInsert)
        }));

        let request_timeout = if self.local_runtime {
            std::time::Duration::from_secs(120)
        } else {
            protocol::request_timeout(&config.provider, &config.base_url, &config.model)
        };
        let api_kind = protocol::detect_api_kind(&config.provider, &config.base_url);
        let endpoint = protocol::chat_endpoint(&config.provider, &config.base_url)
            .map_err(AppError::Config)?;
        let mut body = protocol::build_chat_body(
            &config.provider,
            &config.base_url,
            &config.model,
            messages,
            config.max_tokens,
            request_temperature(config, req),
            on_chunk.is_some(),
        );

        // GLM-4.7/4.5/5 default to thinking mode, but without explicitly enabling it
        // the API may return content in reasoning_content only, leaving content empty.
        // Explicitly enable thinking so both fields are properly populated.
        // Thinking mode also requires temperature >= 0.6 (recommended 1.0).
        if config.model.starts_with("glm-") {
            if let Some(obj) = body.as_object_mut() {
                obj.insert(
                    "thinking".to_string(),
                    serde_json::json!({"type": "enabled"}),
                );
                obj.insert("temperature".to_string(), serde_json::json!(1.0));
                obj.insert("top_p".to_string(), serde_json::json!(0.95));
            }
        }

        // Retry the initial connection (not once streaming starts)
        #[allow(unused_assignments)]
        let mut response = None;
        let mut last_error: Option<AppError> = None;
        let mut attempt = 0u32;

        loop {
            let request = self
                .client
                .post(&endpoint)
                .header("Content-Type", "application/json");
            match protocol::apply_auth_headers(
                request,
                &config.provider,
                &config.base_url,
                &config.api_key,
            )
            .json(&body)
            .timeout(request_timeout)
            .send()
            .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        response = Some(resp);
                        break;
                    } else if status.as_u16() >= 500 && attempt < 2 {
                        let body_text = crate::response_limits::text(resp).await?;
                        tracing::warn!(
                            "LLM server error {} (attempt {}/3), retrying",
                            status,
                            attempt + 1
                        );
                        last_error = Some(AppError::Api {
                            status: status.as_u16(),
                            body: body_text,
                        });
                        attempt += 1;
                        tokio::time::sleep(std::time::Duration::from_millis(
                            1000 * 2u64.pow(attempt - 1),
                        ))
                        .await;
                        continue;
                    } else {
                        let status = resp.status();
                        let text = crate::response_limits::text(resp).await?;
                        // Truncate at a valid UTF-8 char boundary to avoid panic on multi-byte chars
                        let truncate_at = text
                            .char_indices()
                            .take_while(|&(i, _)| i < 200)
                            .last()
                            .map(|(i, c)| i + c.len_utf8())
                            .unwrap_or(text.len());
                        let sanitized = &text[..truncate_at];
                        return Err(AppError::Api {
                            status: status.as_u16(),
                            body: sanitized.to_string(),
                        });
                    }
                }
                Err(e) if e.is_timeout() && attempt < 2 => {
                    tracing::warn!(
                        "LLM connection timeout (attempt {}/3), retrying",
                        attempt + 1
                    );
                    last_error = Some(e.into());
                    attempt += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(
                        1000 * 2u64.pow(attempt - 1),
                    ))
                    .await;
                    continue;
                }
                Err(e) if e.is_connect() && attempt < 2 => {
                    tracing::warn!(
                        "LLM connection failed (attempt {}/3), retrying",
                        attempt + 1
                    );
                    last_error = Some(e.into());
                    attempt += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(
                        1000 * 2u64.pow(attempt - 1),
                    ))
                    .await;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }

        let response = response.ok_or_else(|| last_error.unwrap())?;

        let guard_dictation = super::dictation_guard::enabled(req);
        if let Some(callback) = on_chunk {
            // Streaming mode
            let mut full_text = String::new();
            let mut stream = response.bytes_stream();

            let mut lines = crate::response_limits::Lines::default();
            let mut stream_done = false;
            while !stream_done {
                let Some(chunk) = stream.next().await else {
                    break;
                };
                let chunk = chunk?;

                // Process SSE lines
                for line in lines.push(&chunk)? {
                    let line = line.trim();

                    if let Some(data) = line.strip_prefix("data:").map(str::trim_start) {
                        if data == "[DONE]" {
                            stream_done = true;
                            break;
                        }
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                            let event = protocol::parse_stream_event(api_kind, &v);
                            if let Some(error) = event.error {
                                return Err(AppError::Config(error));
                            }
                            if let Some(content) = event.text {
                                if !content.is_empty() {
                                    full_text.push_str(&content);
                                    if !guard_dictation {
                                        callback(&content);
                                    }
                                }
                            }

                            stream_done = event.done;
                            if stream_done {
                                break;
                            }
                        }
                    }
                }
            }

            // Reasoning is never a final transcript, even when content is absent.
            if full_text.trim().is_empty() {
                return Err(AppError::Output("LLM returned no final text".into()));
            }

            if guard_dictation {
                super::dictation_guard::validate(&req.raw_text, &full_text)?;
                callback(&full_text);
            }
            Ok(PolishResponse {
                polished_text: full_text,
            })
        } else {
            // Non-streaming mode
            let v: serde_json::Value = crate::response_limits::json(response).await?;
            if let Some(error) = protocol::completion_error(api_kind, &v) {
                return Err(AppError::Output(error));
            }
            let text = protocol::final_response_text(api_kind, &v);

            if text.is_empty() {
                tracing::warn!("LLM non-streaming returned no final content");
            }

            if guard_dictation {
                super::dictation_guard::validate(&req.raw_text, &text)?;
            }
            Ok(PolishResponse {
                polished_text: text,
            })
        }
    }

    fn name(&self) -> &str {
        "OpenAI"
    }
}

#[cfg(test)]
mod dictation_tests {
    use super::*;
    use crate::{
        app_detector::types::ContextProfile,
        voice_intent::{VoiceIntent, VoiceIntentKind, VoiceOutputPlacement},
    };
    use std::sync::{Arc, Mutex};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    fn request() -> PolishRequest {
        PolishRequest {
            raw_text: "어 왜 안 되는 거야 알려줘".into(),
            context: ContextProfile::general_native().summary(),
            dictionary: vec![],
            correction_rules: vec![],
            polish_style: "clean".into(),
            mapped_scene_prompt: String::new(),
            active_scene_prompt: String::new(),
            polish_custom_prompt: String::new(),
            translate_enabled: false,
            target_lang: String::new(),
            selected_text: None,
            operation_id: None,
            voice_intent: VoiceIntent::from_parts(
                VoiceIntentKind::DictateInsert,
                VoiceOutputPlacement::InsertAtCursor,
                1.0,
                None,
                None,
                None,
                None,
            )
            .unwrap(),
        }
    }

    #[test]
    fn local_structured_sampling_does_not_change_other_operations() {
        let mut config = LlmConfig {
            provider: "ollama".into(),
            ..Default::default()
        };
        let mut req = request();
        assert_eq!(request_temperature(&config, &req), 0.3);
        req.polish_style = "structured".into();
        assert_eq!(request_temperature(&config, &req), 0.0);
        for field in ["mapped", "manual", "selected", "translation", "ask"] {
            let mut other = req.clone();
            match field {
                "mapped" => other.mapped_scene_prompt = "Story".into(),
                "manual" => other.active_scene_prompt = "Story".into(),
                "selected" => other.selected_text = Some("Existing text".into()),
                "translation" => other.translate_enabled = true,
                _ => other.voice_intent.kind = VoiceIntentKind::OpenQuestion,
            }
            assert_eq!(request_temperature(&config, &other), 0.3, "{field}");
        }
        config.provider = "openai".into();
        assert_eq!(request_temperature(&config, &req), 0.3);
    }

    #[tokio::test]
    async fn dictation_stream_is_validated_before_any_callback() {
        for (field, content, accepted, finish) in [
            (
                "content",
                "왜 안 되는 거야? 알려줘".to_string(),
                true,
                "stop",
            ),
            ("content", "왜 안 되는".to_string(), false, "length"),
            (
                "content",
                "새로운 해결 방법을 설명하겠습니다. ".repeat(30),
                false,
                "stop",
            ),
            (
                "reasoning_content",
                "질문에 답해야겠습니다".to_string(),
                false,
                "stop",
            ),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let event = serde_json::json!({"choices":[{"delta":{field:content}}]});
            let end = serde_json::json!({"choices":[{"delta":{},"finish_reason":finish}]});
            let body = format!("data: {event}\n\ndata: {end}\n\ndata: [DONE]\n\n");
            let server = tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut received = Vec::new();
                loop {
                    let mut buffer = [0; 4096];
                    let n = socket.read(&mut buffer).await.unwrap();
                    assert!(n > 0);
                    received.extend_from_slice(&buffer[..n]);
                    if let Some(end) = received.windows(4).position(|v| v == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&received[..end]);
                        let length: usize = headers
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|v| v.trim().parse().unwrap())
                            })
                            .unwrap();
                        if received.len() >= end + 4 + length {
                            break;
                        }
                    }
                }
                assert!(String::from_utf8_lossy(&received).contains("FINAL_DICTATION_CONTRACT"));
                let start = received.windows(4).position(|v| v == b"\r\n\r\n").unwrap() + 4;
                let request_body: serde_json::Value =
                    serde_json::from_slice(&received[start..]).unwrap();
                assert_eq!(request_body["temperature"], 0.0);
                assert_eq!(request_body["reasoning_effort"], "none");
                assert!(request_body["messages"][0]["content"]
                    .as_str()
                    .unwrap()
                    .contains("즐겨찾기도 할 수 있으면 좋겠어"));
                let header = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                socket.write_all(header.as_bytes()).await.unwrap();
                socket.write_all(body.as_bytes()).await.unwrap();
            });
            let chunks = Arc::new(Mutex::new(Vec::new()));
            let captured = chunks.clone();
            let callback: ChunkCallback =
                Box::new(move |s| captured.lock().unwrap().push(s.to_string()));
            let config = LlmConfig {
                provider: "ollama".into(),
                model: "gemma4:12b".into(),
                base_url: format!("http://{address}/v1"),
                ..Default::default()
            };
            let mut req = request();
            req.polish_style = "structured".into();
            req.raw_text = "검색도 되면 좋겠어".into();
            let result = OpenAiProvider::new()
                .polish(&config, &req, Some(&callback))
                .await;
            server.await.unwrap();
            assert_eq!(result.is_ok(), accepted);
            if accepted {
                assert_eq!(*chunks.lock().unwrap(), vec![content]);
            } else {
                assert!(chunks.lock().unwrap().is_empty());
            }
        }
    }
}
