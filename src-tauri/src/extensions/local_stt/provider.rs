use super::*;
use crate::{
    error::AppError,
    stt::{SttConfig, SttProvider, TranscriptEvent},
};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::{process::Command, sync::OwnedMutexGuard};

pub struct Provider {
    service: Arc<Service>,
    model: Option<Model>,
    lease: Option<Arc<OwnedMutexGuard<()>>>,
    warmup: Option<tokio::task::JoinHandle<Result<(), String>>>,
    pcm: Vec<u8>,
    language: Option<String>,
    preview_requested: bool,
    preview: Option<super::preview::Preview>,
}
impl Provider {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self::with_service(service().map_err(AppError::Config)?))
    }
    fn with_service(service: Arc<Service>) -> Self {
        Self {
            service,
            model: None,
            lease: None,
            warmup: None,
            pcm: Vec::new(),
            language: None,
            preview_requested: false,
            preview: None,
        }
    }
}
impl Drop for Provider {
    fn drop(&mut self) {
        self.preview.take();
        if let Some(task) = self.warmup.take() {
            task.abort();
        }
        if self.model.is_some() {
            // Cancellation before disconnect also releases a completed warmup.
            // An in-flight warmup owns its worker and is killed by task abort.
            if let Ok(mut worker) = self.service.mlx_worker.try_lock() {
                worker.take();
            }
        }
    }
}
#[async_trait]
impl SttProvider for Provider {
    fn enable_preview(&mut self) {
        self.preview_requested = true;
    }
    async fn connect(&mut self, config: &SttConfig) -> Result<(), AppError> {
        if config.sample_rate != 16000 {
            return Err(AppError::Config(
                "Built-in STT requires 16 kHz mono PCM16".into(),
            ));
        }
        let lease = self
            .service
            .gate
            .clone()
            .try_lock_owned()
            .map_err(|_| AppError::Config("Built-in STT is busy".into()))?;
        let m =
            model(&self.service.selected().ok_or_else(|| {
                AppError::Config("Select a downloaded model in STT settings".into())
            })?)
            .map_err(AppError::Config)?;
        if !self.service.available(&m) || !self.service.installed(&m).await {
            return Err(AppError::Config(
                "Download a supported STT model first".into(),
            ));
        }
        self.preview_requested &= self.service.preview_enabled();
        let lease = Arc::new(lease);
        if m.engine == "qwen"
            && self.service.engine_preference() != "cpu"
            && mlx::platform_reason(&self.service).is_none()
        {
            let service = self.service.clone();
            let warm_model = m.clone();
            let warm_lease = lease.clone();
            self.warmup = Some(tokio::spawn(async move {
                let _lease = warm_lease;
                let mut shutdown = service.shutdown.subscribe();
                tokio::select! {
                    _ = shutdown.wait_for(|v| *v) => Err("STT stopped".into()),
                    result = tokio::time::timeout(Duration::from_secs(FINALIZE_SECONDS), async {
                        service.verify_model(&warm_model).await?;
                        mlx::probe(&service).await?;
                        mlx::transcribe(&service, &warm_model, &[], None).await?;
                        Ok(())
                    }) => result.unwrap_or_else(|_| Err("MLX model preparation timed out".into())),
                }
            }));
        }
        self.model = Some(m);
        self.lease = Some(lease);
        self.pcm.clear();
        self.language = config.language.clone();
        Ok(())
    }
    async fn send_audio(&mut self, chunk: &[u8]) -> Result<(), AppError> {
        if self.lease.is_none() || self.pcm.len().saturating_add(chunk.len()) > MAX_AUDIO {
            return Err(AppError::Config(
                "Built-in STT recording is limited to 120 seconds".into(),
            ));
        }
        self.pcm.extend_from_slice(chunk);
        if self.preview_requested
            && self.preview.is_none()
            && self.warmup.as_ref().is_none_or(|task| task.is_finished())
        {
            // Join/propagate warmup failures at finalization. It no longer owns an active worker.
            self.preview = Some(super::preview::Preview::start(
                self.service.clone(),
                self.model.as_ref().unwrap().clone(),
                self.language.clone(),
                self.lease.as_ref().unwrap().clone(),
            ));
        }
        if let Some(preview) = self.preview.as_mut() {
            preview.feed(&self.pcm);
        }
        Ok(())
    }
    async fn recv_transcript(&mut self) -> Result<Option<TranscriptEvent>, AppError> {
        if let Some(preview) = self.preview.as_mut() {
            return Ok(Some(TranscriptEvent::Partial {
                text: preview.recv().await,
            }));
        }
        std::future::pending().await
    }
    async fn disconnect(&mut self) -> Result<Option<String>, AppError> {
        let _lease = self.lease.take();
        if let Some(preview) = self.preview.as_mut() {
            preview.finish().await;
        }
        self.preview.take();
        // Keep the JoinHandle owned by Provider while awaiting: cancellation of
        // disconnect must still abort preparation in Drop.
        if let Some(warmup) = self.warmup.as_mut() {
            warmup
                .await
                .map_err(|e| AppError::Config(e.to_string()))?
                .map_err(AppError::Config)?;
        }
        self.warmup.take();
        let m = self
            .model
            .take()
            .ok_or_else(|| AppError::Config("STT is not connected".into()))?;
        let pcm = std::mem::take(&mut self.pcm);
        if pcm.is_empty() {
            return Ok(None);
        }
        if !pcm.len().is_multiple_of(2) {
            return Err(AppError::Config("Invalid PCM16 input".into()));
        }
        if pcm.iter().all(|v| *v == 0) {
            return Ok(None);
        }
        let mut shutdown = self.service.shutdown.subscribe();
        let result = tokio::select! {
            biased;
            _ = shutdown.wait_for(|v| *v) => Err("STT stopped".into()),
            r = tokio::time::timeout(Duration::from_secs(FINALIZE_SECONDS), transcribe(&self.service, &m, &pcm, self.language.as_deref())) => r.unwrap_or_else(|_| Err("Built-in STT timed out after 90 seconds".into())),
        }.map_err(AppError::Config)?;
        Ok((!result.is_empty()).then_some(result))
    }
    fn recording_limit_override_seconds(&self) -> Option<u32> {
        Some(120)
    }
    fn name(&self) -> &str {
        ID
    }
}
async fn read_bounded(input: impl tokio::io::AsyncRead + Unpin) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    input
        .take(65537)
        .read_to_end(&mut bytes)
        .await
        .map_err(err)?;
    if bytes.len() > 65536 {
        return Err("STT engine output exceeded limit".into());
    }
    Ok(bytes)
}
pub(super) async fn transcribe(
    s: &Service,
    m: &Model,
    pcm: &[u8],
    language: Option<&str>,
) -> Result<String, String> {
    let started = std::time::Instant::now();
    s.verify_model(m).await?;
    tracing::info!(
        "local_stt model={} verify_ms={}",
        m.id,
        started.elapsed().as_millis()
    );
    if mlx::use_mlx(s, m).await? {
        return mlx::transcribe(
            s,
            m,
            pcm,
            language
                .filter(|l| *l != "auto" && *l != "multi")
                .map(qwen_language)
                .transpose()?,
        )
        .await;
    }
    let inference = std::time::Instant::now();
    let mut cmd = Command::new(s.binary(m));
    cmd.kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.env("VECLIB_MAXIMUM_THREADS", "4")
        .env("OPENBLAS_NUM_THREADS", "4")
        .env("OMP_NUM_THREADS", "4")
        .env("QWEN_BF16_CACHE_MB", "256");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000);
    let language = language.filter(|l| *l != "multi" && *l != "auto");
    if m.engine == "whisper" {
        // A non-stdin output basename keeps segment callbacks on stdout. No
        // output-format flag is supplied, so whisper-cli creates no text file.
        cmd.arg("-m")
            .arg(s.root.join(&m.id).join(&m.files[0].name))
            .args([
                "-f",
                "-",
                "-of",
                "transcript",
                "-nt",
                "-np",
                "-t",
                "4",
                "-l",
                language.unwrap_or("auto"),
            ]);
    } else {
        cmd.arg("-d")
            .arg(s.root.join(&m.id))
            .args(["--stdin", "--silent", "-t", "4", "-S", "20"]);
        if let Some(lang) = language {
            cmd.args(["--language", qwen_language(lang)?]);
        }
    }
    let mut child = cmd.spawn().map_err(err)?;
    let mut stdin = child.stdin.take().ok_or("Missing engine stdin")?;
    let stdout = child.stdout.take().ok_or("Missing engine stdout")?;
    let stderr = child.stderr.take().ok_or("Missing engine stderr")?;
    let wav = crate::stt::whisper_compat::WhisperCompatProvider::build_wav(pcm, 16000);
    let writer = async {
        stdin.write_all(&wav).await.map_err(err)?;
        stdin.shutdown().await.map_err(err)?;
        drop(stdin);
        Ok::<_, String>(())
    };
    let ((), output, _diagnostic, status) =
        tokio::try_join!(writer, read_bounded(stdout), read_bounded(stderr), async {
            child.wait().await.map_err(err)
        })?;
    if !status.success() {
        return Err(format!(
            "STT engine exited unsuccessfully ({status}). Try a smaller model."
        ));
    }
    tracing::info!(
        "local_stt backend=cpu model={} process_ms={}",
        m.id,
        inference.elapsed().as_millis()
    );
    String::from_utf8(output)
        .map(|s| s.trim().to_owned())
        .map_err(|_| "Invalid STT output".into())
}
fn qwen_language(lang: &str) -> Result<&str, String> {
    Ok(match lang { "ko" => "Korean", "en" => "English", "zh" | "zh-CN" | "zh-TW" => "Chinese", "ja" => "Japanese", "de" => "German", "fr" => "French", "es" => "Spanish", "it" => "Italian", "pt" => "Portuguese", "ru" => "Russian", "ar" => "Arabic", "hi" => "Hindi", "id" => "Indonesian", "tr" => "Turkish", "vi" => "Vietnamese", "th" => "Thai", "nl" => "Dutch", "pl" => "Polish", "sv" => "Swedish", "da" => "Danish", "fi" => "Finnish", "el" => "Greek", "hu" => "Hungarian", "cs" => "Czech", "ro" => "Romanian", "fa" => "Persian", "ms" => "Malay", "fil" => "Filipino", "mk" => "Macedonian", _ => return Err("This language is not supported by the bundled Qwen engine. Choose automatic detection or Whisper.".into()) })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore = "real local models; H_LOCAL_STT_TEST_ROOT, H_LOCAL_STT_ENGINE_DIR, H_LOCAL_STT_TEST_PCM"]
    async fn real_utterance_preview_and_final() {
        let s = Service::new(
            PathBuf::from(std::env::var("H_LOCAL_STT_TEST_ROOT").unwrap()),
            PathBuf::from(std::env::var("H_LOCAL_STT_ENGINE_DIR").unwrap()),
        )
        .unwrap();
        let pcm = fs::read(std::env::var("H_LOCAL_STT_TEST_PCM").unwrap())
            .await
            .unwrap();
        let mut p = Provider::with_service(s.clone());
        p.enable_preview();
        p.connect(&SttConfig {
            language: Some("ko".into()),
            ..Default::default()
        })
        .await
        .unwrap();
        if let Some(warmup) = p.warmup.as_mut() {
            warmup.await.unwrap().unwrap();
        }
        p.warmup.take();
        for round in 0..std::env::var("H_PREVIEW_TEST_ROUNDS")
            .ok()
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(3)
        {
            p.send_audio(&pcm).await.unwrap();
            let started = std::time::Instant::now();
            p.send_audio(&vec![0; 32000]).await.unwrap();
            let event = tokio::time::timeout(Duration::from_secs(20), p.recv_transcript())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            let TranscriptEvent::Partial { text } = event else {
                panic!("expected preview");
            };
            eprintln!(
                "preview round={round} after_silence_ms={} synthetic_text={text}",
                started.elapsed().as_millis()
            );
            assert!(text.contains("설정"));
        }
        let started = std::time::Instant::now();
        let final_text = p.disconnect().await.unwrap().unwrap();
        eprintln!(
            "final_ms={} synthetic_text={final_text}",
            started.elapsed().as_millis()
        );
        assert!(final_text.contains("설정") && final_text.contains("버튼"));
        mlx::stop(&s).await;
        assert!(s.gate.try_lock().is_ok());
    }
    #[tokio::test]
    async fn recording_does_not_busy_spin() {
        let dir = tempfile::tempdir().unwrap();
        let mut p =
            Provider::with_service(Service::new(dir.path().into(), dir.path().into()).unwrap());
        assert!(
            tokio::time::timeout(Duration::from_millis(30), p.recv_transcript())
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn output_is_bounded() {
        assert!(read_bounded(&vec![b'x'; 70000][..]).await.is_err());
    }
    #[tokio::test]
    async fn pcm_requires_connection_and_is_bounded() {
        let dir = tempfile::tempdir().unwrap();
        let mut p =
            Provider::with_service(Service::new(dir.path().into(), dir.path().into()).unwrap());
        assert!(p.send_audio(&[0, 0]).await.is_err());
        p.lease = Some(Arc::new(p.service.gate.clone().lock_owned().await));
        p.pcm.resize(MAX_AUDIO, 0);
        assert!(p.send_audio(&[0, 0]).await.is_err());
        assert!(p.service.gate.try_lock().is_err());
        drop(p.lease.take());
        assert!(p.service.gate.try_lock().is_ok());
    }
    #[tokio::test]
    #[cfg(unix)]
    async fn cancelled_process_is_killed_and_reaped() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let s = Service::new(dir.path().into(), dir.path().into()).unwrap();
        let m = Model {
            id: "fake".into(),
            name: "fake".into(),
            engine: "whisper".into(),
            recommended_ram_gb: 1,
            license: "MIT".into(),
            files: vec![ModelFile {
                name: "model".into(),
                url: "".into(),
                size: 1,
                sha256: format!("{:x}", Sha256::digest(b"x")),
            }],
        };
        fs::create_dir(dir.path().join("fake")).await.unwrap();
        fs::write(dir.path().join("fake/model"), b"x")
            .await
            .unwrap();
        let pidfile = dir.path().join("pid");
        fs::write(
            s.binary(&m),
            format!(
                "#!/bin/sh\necho $$ > '{}'\nexec sleep 60\n",
                pidfile.display()
            ),
        )
        .await
        .unwrap();
        std::fs::set_permissions(s.binary(&m), std::fs::Permissions::from_mode(0o755)).unwrap();
        let worker = tokio::spawn(async move { transcribe(&s, &m, &[1, 0], None).await });
        for _ in 0..100 {
            if pidfile.exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        worker.abort();
        let _ = worker.await;
        let pid = fs::read_to_string(pidfile).await.unwrap();
        for _ in 0..40 {
            let result = std::process::Command::new("kill")
                .args(["-0", pid.trim()])
                .stderr(Stdio::null())
                .status()
                .unwrap();
            if !result.success() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("native engine survived cancellation");
    }
    #[tokio::test]
    #[ignore = "real MLX inference; H_LOCAL_STT_TEST_ROOT, H_LOCAL_STT_ENGINE_DIR, H_LOCAL_STT_TEST_PCM"]
    async fn real_mlx_provider_reuses_and_releases_model() {
        let s = Service::new(
            PathBuf::from(std::env::var("H_LOCAL_STT_TEST_ROOT").unwrap()),
            PathBuf::from(std::env::var("H_LOCAL_STT_ENGINE_DIR").unwrap()),
        )
        .unwrap();
        let pcm = fs::read(std::env::var("H_LOCAL_STT_TEST_PCM").unwrap())
            .await
            .unwrap();
        for _ in 0..3 {
            let mut p = Provider::with_service(s.clone());
            let start = std::time::Instant::now();
            p.connect(&SttConfig {
                language: Some("ko".into()),
                ..Default::default()
            })
            .await
            .unwrap();
            p.send_audio(&pcm).await.unwrap();
            let text = p.disconnect().await.unwrap().unwrap();
            eprintln!(
                "provider wall_ms={} synthetic_result={text}",
                start.elapsed().as_millis()
            );
            assert!(text.contains("설정") && text.contains("버튼"));
            assert!(s.mlx_worker.lock().await.is_some());
        }
        mlx::stop(&s).await;
        assert!(s.mlx_worker.lock().await.is_none());
        let mut p = Provider::with_service(s.clone());
        p.connect(&SttConfig::default()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        drop(p);
        let _lease = tokio::time::timeout(Duration::from_secs(3), s.gate.lock())
            .await
            .unwrap();
        mlx::stop(&s).await;
    }
    #[tokio::test]
    #[ignore = "real native inference; set H_LOCAL_STT_TEST_ROOT, H_LOCAL_STT_ENGINE_DIR, H_LOCAL_STT_TEST_PCM"]
    async fn transcribe_real_models() {
        let root = PathBuf::from(std::env::var("H_LOCAL_STT_TEST_ROOT").unwrap());
        let s = Service::new(
            root,
            PathBuf::from(std::env::var("H_LOCAL_STT_ENGINE_DIR").unwrap()),
        )
        .unwrap();
        let pcm = fs::read(std::env::var("H_LOCAL_STT_TEST_PCM").unwrap())
            .await
            .unwrap();
        for id in ["base", "qwen-0.6b", "qwen-1.7b"] {
            let m = model(id).unwrap();
            let result = tokio::time::timeout(
                Duration::from_secs(180),
                transcribe(&s, &m, &pcm, Some("ko")),
            )
            .await
            .unwrap()
            .unwrap();
            eprintln!("{id}: {result}");
            assert!(result.contains("설정"));
            assert!(result.contains("버튼"));
        }
    }
}
