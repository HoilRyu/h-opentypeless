use super::{processing::ResultText, wav};
use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::{watch, Semaphore};
#[async_trait::async_trait]
pub trait Processor: Send + Sync {
    async fn process(&self, pcm: &[u8]) -> Result<ResultText, String>;
}
pub struct Desktop(pub tauri::AppHandle);
#[async_trait::async_trait]
impl Processor for Desktop {
    async fn process(&self, pcm: &[u8]) -> Result<ResultText, String> {
        super::processing::process(&self.0, pcm).await
    }
}
#[derive(Clone)]
pub struct ApiState {
    pub processor: Arc<dyn Processor>,
    pub busy: Arc<Semaphore>,
    pub shutdown: watch::Receiver<bool>,
}
fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({"detail":message}))).into_response()
}
pub fn router(state: ApiState) -> Router {
    Router::new().route("/api/health",get(||async {Json(serde_json::json!({"status":"ok","service":"h-opentypeless","protocol":1,"audio":"wav-pcm16-16000-mono"}))}))
    .route("/api/dictate",post(dictate)).layer(DefaultBodyLimit::max(wav::MAX_BODY)).with_state(state)
}
async fn dictate(
    State(mut state): State<ApiState>,
    headers: HeaderMap,
    multipart: Result<Multipart, axum::extract::multipart::MultipartRejection>,
) -> Response {
    // Not authentication: prevents ordinary web pages submitting recordings without a CORS preflight.
    if headers.contains_key("origin")
        || headers
            .get("x-h-opentypeless-client")
            .and_then(|v| v.to_str().ok())
            != Some("android-v1")
    {
        return error(
            StatusCode::FORBIDDEN,
            "H-OpenTypeless Android 앱에서 연결해 주세요.",
        );
    }
    let _permit = match state.busy.clone().try_acquire_owned() {
        Ok(p) => p,
        Err(_) => {
            return error(
                StatusCode::TOO_MANY_REQUESTS,
                "다른 음성을 처리 중입니다. 잠시 후 다시 시도하세요.",
            )
        }
    };
    if *state.shutdown.borrow() {
        return error(StatusCode::SERVICE_UNAVAILABLE, "모바일 연결이 꺼졌습니다.");
    }
    let work = async {
        let mut multipart = match multipart {
            Ok(m) => m,
            Err(_) => {
                return error(
                    StatusCode::BAD_REQUEST,
                    "음성 파일 전송 형식이 올바르지 않습니다.",
                )
            }
        };
        let upload = tokio::time::timeout(Duration::from_secs(30), async {
            let mut audio = None;
            while let Some(field) = multipart.next_field().await.map_err(|_| ())? {
                if field.name() != Some("file") || audio.is_some() {
                    return Err(());
                }
                let bytes = field.bytes().await.map_err(|_| ())?;
                audio = Some(bytes);
            }
            audio.ok_or(())
        })
        .await;
        let audio = match upload {
            Ok(Ok(b)) => b,
            _ => {
                return error(
                    StatusCode::BAD_REQUEST,
                    "파일이 없거나 너무 큽니다. 전송은 30초 안에 완료되어야 합니다.",
                )
            }
        };
        let pcm = match wav::pcm(&audio) {
            Ok(p) => p,
            Err(e) => return error(StatusCode::UNSUPPORTED_MEDIA_TYPE, &e),
        };
        match tokio::time::timeout(Duration::from_secs(300), state.processor.process(pcm)).await {
            Ok(Ok(result)) => Json(result).into_response(),
            Ok(Err(message)) => error(StatusCode::BAD_GATEWAY, &message),
            Err(_) => error(
                StatusCode::GATEWAY_TIMEOUT,
                "음성 처리 시간이 초과되었습니다. 모델 엔진을 확인하세요.",
            ),
        }
    };
    tokio::select! { response=work=>response,_=state.shutdown.changed()=>error(StatusCode::SERVICE_UNAVAILABLE,"모바일 연결이 꺼져 요청을 취소했습니다.") }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Echo;
    #[async_trait::async_trait]
    impl Processor for Echo {
        async fn process(&self, pcm: &[u8]) -> Result<ResultText, String> {
            assert_eq!(pcm.len(), 4);
            Ok(ResultText {
                raw_text: "원문".into(),
                polished_text: "교정문".into(),
                warning: None,
            })
        }
    }
    fn wave() -> Vec<u8> {
        let mut b = Vec::from(&b"RIFF"[..]);
        b.extend(40u32.to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend([1, 0, 1, 0]);
        b.extend(16000u32.to_le_bytes());
        b.extend(32000u32.to_le_bytes());
        b.extend([2, 0, 16, 0]);
        b.extend(b"data");
        b.extend(4u32.to_le_bytes());
        b.extend([1, 0, 2, 0]);
        b
    }
    #[tokio::test]
    async fn native_upload_contract_limits_and_disable() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (stop, shutdown) = watch::channel(false);
        let busy = Arc::new(Semaphore::new(1));
        let app = router(ApiState {
            processor: Arc::new(Echo),
            busy: busy.clone(),
            shutdown,
        });
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let client = reqwest::Client::new();
        let health: serde_json::Value = client
            .get(format!("{url}/api/health"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(health["service"], "h-opentypeless");
        let request = |audio: Vec<u8>| {
            client
                .post(format!("{url}/api/dictate"))
                .header("x-h-opentypeless-client", "android-v1")
                .multipart(reqwest::multipart::Form::new().part(
                    "file",
                    reqwest::multipart::Part::bytes(audio).file_name("recording.wav"),
                ))
        };
        let ok = request(wave()).send().await.unwrap();
        assert_eq!(ok.status(), 200);
        let result: serde_json::Value = ok.json().await.unwrap();
        assert_eq!(result["polished_text"], "교정문");
        assert!(result["warning"].is_null());
        assert_eq!(request(vec![0; 44]).send().await.unwrap().status(), 415);
        assert_eq!(
            request(wave())
                .header("Origin", "https://example.com")
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        assert_eq!(
            client
                .post(format!("{url}/api/dictate"))
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        let permit = busy.acquire().await.unwrap();
        assert_eq!(request(wave()).send().await.unwrap().status(), 429);
        drop(permit);
        assert_eq!(
            request(vec![0; wav::MAX_BODY + 1])
                .send()
                .await
                .unwrap()
                .status(),
            400
        );
        stop.send(true).unwrap();
        assert_eq!(request(wave()).send().await.unwrap().status(), 503);
        task.abort();
    }
    struct Waiting;
    #[async_trait::async_trait]
    impl Processor for Waiting {
        async fn process(&self, _: &[u8]) -> Result<ResultText, String> {
            std::future::pending().await
        }
    }
    #[tokio::test]
    async fn disabling_cancels_processing_and_releases_busy_permit() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/api/dictate", listener.local_addr().unwrap());
        let (stop, shutdown) = watch::channel(false);
        let busy = Arc::new(Semaphore::new(1));
        let gate = busy.clone();
        let task = tokio::spawn(async move {
            axum::serve(
                listener,
                router(ApiState {
                    processor: Arc::new(Waiting),
                    busy: gate,
                    shutdown,
                }),
            )
            .await
            .unwrap()
        });
        let request = tokio::spawn(async move {
            reqwest::Client::new()
                .post(url)
                .header("x-h-opentypeless-client", "android-v1")
                .multipart(reqwest::multipart::Form::new().part(
                    "file",
                    reqwest::multipart::Part::bytes(wave()).file_name("a.wav"),
                ))
                .send()
                .await
                .unwrap()
        });
        tokio::time::timeout(Duration::from_secs(3), async {
            while busy.available_permits() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        stop.send(true).unwrap();
        assert_eq!(request.await.unwrap().status(), 503);
        assert_eq!(busy.available_permits(), 1);
        task.abort();
    }
}
