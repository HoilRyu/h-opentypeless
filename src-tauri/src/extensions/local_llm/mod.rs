//! H-owned Ollama runtime and an immutable, three-model download catalog.
use crate::{
    error::AppError,
    llm::{ChunkCallback, LlmConfig, LlmProvider, PolishRequest, PolishResponse},
};
use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant, SystemTime},
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{watch, Mutex as AsyncMutex},
};
pub const ID: &str = "builtin-llm";
static SERVICE: OnceLock<Arc<Service>> = OnceLock::new();
type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn service() -> Result<Arc<Service>> {
    SERVICE
        .get()
        .cloned()
        .ok_or("Built-in LLM is not initialized".into())
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Blob {
    digest: String,
    size: u64,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub tag: String,
    manifest: String,
    sha256: String,
    files: Vec<Blob>,
}
pub fn catalog() -> Vec<Model> {
    serde_json::from_str(include_str!("catalog.json")).expect("compiled LLM catalog")
}
fn model(id: &str) -> Result<Model> {
    catalog()
        .into_iter()
        .find(|m| m.id == id)
        .ok_or("Unknown built-in LLM model".into())
}
fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
#[derive(Clone, Default, Serialize)]
pub struct Progress {
    model: String,
    downloaded: u64,
    total: u64,
    phase: String,
    error: Option<String>,
}
#[derive(Serialize)]
pub struct ModelStatus {
    id: String,
    name: String,
    size: u64,
    installed: bool,
}
#[derive(Serialize)]
pub struct Status {
    models: Vec<ModelStatus>,
    selected: Option<String>,
    progress: Progress,
    busy: bool,
    available: bool,
    running: bool,
}
struct Runtime {
    child: tokio::process::Child,
    url: String,
    last_used: Instant,
}
impl Runtime {
    async fn stop(mut self) -> Result<()> {
        #[cfg(unix)]
        if let Some(pid) = self.child.id() {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
        tokio::time::timeout(Duration::from_secs(3), self.child.wait())
            .await
            .map_err(|_| "Ollama shutdown timed out".to_string())?
            .map_err(err)?;
        Ok(())
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        // The supervisor handles SIGTERM and kills/reaps Ollama's process group.
        #[cfg(unix)]
        if let Some(pid) = self.child.id() {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
    }
}
pub struct Service {
    root: PathBuf,
    engines: PathBuf,
    client: reqwest::Client,
    downloads: reqwest::Client,
    gate: AsyncMutex<()>,
    runtime: AsyncMutex<Option<Runtime>>,
    progress: Mutex<Progress>,
    cancel: Mutex<Option<watch::Sender<bool>>>,
    shutdown: watch::Sender<bool>,
    verified: Mutex<HashMap<PathBuf, (u64, SystemTime)>>,
}
impl Service {
    pub fn new(root: PathBuf, engines: PathBuf) -> Result<Arc<Self>> {
        std::fs::create_dir_all(&root).map_err(err)?;
        let (shutdown, _) = watch::channel(false);
        Ok(Arc::new(Self {
            root,
            engines,
            client: reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(120))
                .build()
                .map_err(err)?,
            downloads: reqwest::Client::builder()
                .https_only(true)
                .connect_timeout(Duration::from_secs(20))
                .timeout(Duration::from_secs(7200))
                .build()
                .map_err(err)?,
            gate: AsyncMutex::new(()),
            runtime: AsyncMutex::new(None),
            progress: Mutex::new(Progress::default()),
            cancel: Mutex::new(None),
            shutdown,
            verified: Mutex::new(HashMap::new()),
        }))
    }
    pub fn install(root: PathBuf, engines: PathBuf) -> Result<()> {
        let s = Self::new(root, engines)?;
        SERVICE
            .set(s.clone())
            .map_err(|_| "LLM already initialized".to_string())?;
        tauri::async_runtime::spawn(async move {
            let mut stop = s.shutdown.subscribe();
            loop {
                tokio::select! { _ = stop.changed() => break, _ = tokio::time::sleep(Duration::from_secs(10)) => {} }
                if let Ok(mut runtime) = s.runtime.try_lock() {
                    if runtime
                        .as_ref()
                        .is_some_and(|r| r.last_used.elapsed() > Duration::from_secs(120))
                    {
                        if let Some(r) = runtime.take() {
                            let _ = r.stop().await;
                        }
                    }
                }
            }
            if let Some(r) = s.runtime.lock().await.take() {
                let _ = r.stop().await;
            }
        });
        Ok(())
    }
    pub fn shutdown(&self) {
        self.shutdown.send_replace(true);
        self.cancel();
    }
    fn available(&self) -> bool {
        cfg!(all(target_os = "macos", target_arch = "aarch64"))
            && self.engines.join("ollama").is_file()
            && self.engines.join("h-llm-supervisor").is_file()
    }
    fn manifest(&self, m: &Model) -> PathBuf {
        self.root
            .join("models/manifests/registry.ollama.ai/library/gemma4")
            .join(&m.id)
    }
    fn blob(&self, b: &Blob) -> PathBuf {
        self.root
            .join("models/blobs")
            .join(b.digest.replace(':', "-"))
    }
    async fn installed(&self, m: &Model) -> bool {
        fs::read(self.manifest(m))
            .await
            .is_ok_and(|v| v == m.manifest.as_bytes())
            && futures_util::future::join_all(m.files.iter().map(|b| async move {
                fs::metadata(self.blob(b))
                    .await
                    .is_ok_and(|v| v.len() == b.size)
            }))
            .await
            .into_iter()
            .all(|v| v)
    }
    pub async fn status(&self) -> Status {
        let mut models = Vec::new();
        for m in catalog() {
            models.push(ModelStatus {
                installed: self.installed(&m).await,
                size: m.files.iter().map(|b| b.size).sum(),
                id: m.id,
                name: m.name,
            });
        }
        let selected = fs::read_to_string(self.root.join("selected"))
            .await
            .ok()
            .filter(|s| model(s).is_ok());
        Status {
            models,
            selected,
            progress: lock(&self.progress).clone(),
            busy: self.gate.try_lock().is_err(),
            available: self.available(),
            running: self.runtime.try_lock().map_or(true, |r| r.is_some()),
        }
    }
    async fn verify(&self, path: &Path, blob: &Blob) -> Result<()> {
        let meta = fs::metadata(path).await.map_err(err)?;
        let stamp = (meta.len(), meta.modified().map_err(err)?);
        if stamp.0 != blob.size {
            return Err("Model file size mismatch".into());
        }
        if lock(&self.verified).get(path) == Some(&stamp) {
            return Ok(());
        }
        let mut f = fs::File::open(path).await.map_err(err)?;
        let mut hash = Sha256::new();
        let mut buffer = vec![0u8; 1024 * 1024];
        loop {
            let n = f.read(&mut buffer).await.map_err(err)?;
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
        }
        if format!("sha256:{:x}", hash.finalize()) != blob.digest {
            return Err("Model checksum mismatch; delete and download again".into());
        }
        let after = fs::metadata(path).await.map_err(err)?;
        if (after.len(), after.modified().map_err(err)?) != stamp {
            return Err("Model changed during verification".into());
        }
        lock(&self.verified).insert(path.into(), stamp);
        Ok(())
    }
    pub fn cancel(&self) {
        if let Some(tx) = lock(&self.cancel).as_ref() {
            tx.send_replace(true);
        }
    }
    async fn download_blob(&self, b: &Blob) -> Result<()> {
        let path = self.blob(b);
        if path.exists() {
            self.verify(&path, b).await?;
            lock(&self.progress).downloaded += b.size;
            return Ok(());
        }
        fs::create_dir_all(path.parent().unwrap())
            .await
            .map_err(err)?;
        let part = path.with_extension("part");
        let offset = fs::metadata(&part).await.map(|m| m.len()).unwrap_or(0);
        if offset > b.size {
            fs::remove_file(&part).await.map_err(err)?;
            return Err("Invalid partial model; retry download".into());
        }
        if offset < b.size {
            let response = self
                .downloads
                .get(format!(
                    "https://registry.ollama.ai/v2/library/gemma4/blobs/{}",
                    b.digest
                ))
                .header("Range", format!("bytes={offset}-"))
                .send()
                .await
                .map_err(err)?
                .error_for_status()
                .map_err(err)?;
            let resume = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
            if resume {
                let expected = format!("bytes {offset}-{}/{}", b.size - 1, b.size);
                if response
                    .headers()
                    .get("content-range")
                    .and_then(|h| h.to_str().ok())
                    != Some(expected.as_str())
                {
                    return Err("Invalid download range".into());
                }
            }
            let mut written = if resume { offset } else { 0 };
            lock(&self.progress).downloaded += written;
            let mut file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(resume)
                .truncate(!resume)
                .open(&part)
                .await
                .map_err(err)?;
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(err)?;
                if written + chunk.len() as u64 > b.size {
                    return Err("Download exceeds catalog size".into());
                }
                file.write_all(&chunk).await.map_err(err)?;
                written += chunk.len() as u64;
                lock(&self.progress).downloaded += chunk.len() as u64;
            }
            file.sync_all().await.map_err(err)?;
        } else {
            lock(&self.progress).downloaded += offset;
        }
        lock(&self.progress).phase = "verifying".into();
        if let Err(e) = self.verify(&part, b).await {
            let _ = fs::remove_file(&part).await;
            return Err(e);
        }
        fs::rename(&part, &path).await.map_err(err)?;
        lock(&self.progress).phase = "downloading".into();
        Ok(())
    }
    pub async fn download(&self, id: &str) -> Result<()> {
        let m = model(id)?;
        let _gate = self.gate.try_lock().map_err(|_| "LLM is busy")?;
        if !self.available() {
            return Err("Built-in Ollama runtime is unavailable".into());
        }
        let (tx, mut cancel) = watch::channel(false);
        *lock(&self.cancel) = Some(tx);
        *lock(&self.progress) = Progress {
            model: m.id.clone(),
            total: m.files.iter().map(|b| b.size).sum(),
            phase: "downloading".into(),
            ..Default::default()
        };
        let mut shutdown = self.shutdown.subscribe();
        let result = tokio::select! {
            _ = cancel.changed() => Err("Download cancelled; partial files retained".into()),
            _ = shutdown.changed() => Err("App is shutting down".into()),
            result = async {
                for b in &m.files { self.download_blob(b).await?; }
                let path = self.manifest(&m); fs::create_dir_all(path.parent().unwrap()).await.map_err(err)?;
                let part = path.with_extension("tmp"); fs::write(&part, &m.manifest).await.map_err(err)?; fs::rename(part,path).await.map_err(err)?;
                Ok(())
            } => result,
        };
        *lock(&self.cancel) = None;
        let mut p = lock(&self.progress);
        p.phase = if result.is_ok() {
            "complete"
        } else {
            "stopped"
        }
        .into();
        p.error = result.as_ref().err().cloned();
        result
    }
    pub async fn select(&self, id: &str) -> Result<()> {
        let m = model(id)?;
        let _gate = self.gate.try_lock().map_err(|_| "LLM is busy")?;
        if !self.installed(&m).await {
            return Err("Download this model first".into());
        }
        for b in &m.files {
            self.verify(&self.blob(b), b).await?;
        }
        if let Some(r) = self.runtime.lock().await.take() {
            r.stop().await?;
        }
        let tmp = self.root.join("selected.tmp");
        fs::write(&tmp, id).await.map_err(err)?;
        fs::rename(tmp, self.root.join("selected"))
            .await
            .map_err(err)?;
        Ok(())
    }
    pub async fn delete(&self, id: &str) -> Result<()> {
        let m = model(id)?;
        let _gate = self.gate.try_lock().map_err(|_| "LLM is busy")?;
        if let Some(r) = self.runtime.lock().await.take() {
            r.stop().await?;
        }
        let manifest = self.manifest(&m);
        if manifest.exists() {
            fs::remove_file(manifest).await.map_err(err)?;
        }
        let other: Vec<_> = catalog().into_iter().filter(|v| v.id != id).collect();
        for b in &m.files {
            // Shared license/config blobs stay available for other catalog models.
            if !other
                .iter()
                .any(|v| v.files.iter().any(|o| o.digest == b.digest))
            {
                for path in [self.blob(b), self.blob(b).with_extension("part")] {
                    if path.exists() {
                        fs::remove_file(&path).await.map_err(err)?;
                    }
                    lock(&self.verified).remove(&path);
                }
            }
        }
        if fs::read_to_string(self.root.join("selected"))
            .await
            .ok()
            .as_deref()
            == Some(id)
        {
            fs::remove_file(self.root.join("selected"))
                .await
                .map_err(err)?;
        }
        Ok(())
    }
    pub async fn unload(&self) -> Result<()> {
        let _gate = self.gate.try_lock().map_err(|_| "LLM is busy")?;
        if let Some(r) = self.runtime.lock().await.take() {
            r.stop().await?;
        }
        lock(&self.verified).clear();
        Ok(())
    }
    async fn start(&self) -> Result<Runtime> {
        if !self.available() {
            return Err("Built-in LLM requires the bundled macOS Apple Silicon runtime".into());
        }
        let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(err)?;
        let address = listener.local_addr().map_err(err)?;
        drop(listener);
        let home = self.root.join("home");
        fs::create_dir_all(&home).await.map_err(err)?;
        let mut command = tokio::process::Command::new(self.engines.join("h-llm-supervisor"));
        command
            .arg(std::process::id().to_string())
            .arg(self.engines.join("ollama"))
            .env("HOME", home)
            .env("OLLAMA_MODELS", self.root.join("models"))
            .env("OLLAMA_HOST", address.to_string())
            .env("OLLAMA_NO_CLOUD", "1")
            .env("OLLAMA_ORIGINS", "http://127.0.0.1")
            .env("OLLAMA_CONTEXT_LENGTH", "8192")
            .env("OLLAMA_NUM_PARALLEL", "1")
            .env("OLLAMA_MAX_LOADED_MODELS", "1")
            .env("OLLAMA_KEEP_ALIVE", "120s")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let mut runtime = Runtime {
            child: command.spawn().map_err(err)?,
            url: format!("http://{address}"),
            last_used: Instant::now(),
        };
        // macOS can spend several seconds validating newly signed native
        // libraries before Ollama finishes GPU discovery on its first launch.
        let startup = Instant::now();
        while startup.elapsed() < Duration::from_secs(60) {
            if runtime.child.try_wait().map_err(err)?.is_some() {
                return Err("Ollama failed to start".into());
            }
            if let Ok(response) = self
                .client
                .get(format!("{}/api/version", runtime.url))
                .timeout(Duration::from_millis(200))
                .send()
                .await
            {
                if response.status().is_success() {
                    let version: serde_json::Value = response.json().await.map_err(err)?;
                    if version["version"].as_str() != Some("0.34.0") {
                        return Err("Unexpected Ollama runtime version".into());
                    }
                    // The supervisor spends up to one second cleaning up a failed
                    // engine. Do not accept an unrelated server that won the port
                    // race while our own engine was exiting after a bind failure.
                    tokio::time::sleep(Duration::from_millis(1200)).await;
                    if runtime.child.try_wait().map_err(err)?.is_some() {
                        return Err("Ollama failed to bind its private port".into());
                    }
                    return Ok(runtime);
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err("Ollama startup timed out".into())
    }
    async fn ready(&self) -> Result<(Model, Runtime)> {
        if *self.shutdown.borrow() {
            return Err("App is shutting down".into());
        }
        let id = fs::read_to_string(self.root.join("selected"))
            .await
            .map_err(|_| "Select a downloaded LLM model first")?;
        let m = model(&id)?;
        if !self.installed(&m).await {
            return Err("Selected LLM is missing or incomplete".into());
        }
        for b in &m.files {
            self.verify(&self.blob(b), b).await?;
        }
        let mut saved = self.runtime.lock().await.take();
        if let Some(r) = saved.as_mut() {
            if r.child.try_wait().map_err(err)?.is_some() {
                saved = None;
            }
        }
        if *self.shutdown.borrow() {
            return Err("App is shutting down".into());
        }
        let runtime = match saved {
            Some(r) => r,
            None => self.start().await?,
        };
        Ok((m, runtime))
    }
    pub async fn test(&self) -> Result<u32> {
        let _gate = self.gate.try_lock().map_err(|_| "LLM is busy")?;
        let start = Instant::now();
        let (m, mut runtime) = self.ready().await?;
        let response=self.client.post(format!("{}/api/chat",runtime.url)).json(&serde_json::json!({"model":m.tag,"messages":[{"role":"user","content":"안녕하세요"}],"think":false,"stream":false,"options":{"num_predict":16,"num_ctx":8192}})).send().await.map_err(err)?.error_for_status().map_err(err)?;
        let body: serde_json::Value = response.json().await.map_err(err)?;
        if body["message"]["content"]
            .as_str()
            .is_none_or(|s| s.is_empty())
        {
            return Err("Model returned no text".into());
        }
        runtime.last_used = Instant::now();
        *self.runtime.lock().await = Some(runtime);
        Ok(start.elapsed().as_millis().min(u32::MAX as u128) as u32)
    }
}
pub struct Provider;
#[async_trait]
impl LlmProvider for Provider {
    fn name(&self) -> &str {
        ID
    }
    async fn polish(
        &self,
        config: &LlmConfig,
        request: &PolishRequest,
        on_chunk: Option<&ChunkCallback>,
    ) -> std::result::Result<PolishResponse, AppError> {
        let s = service().map_err(AppError::Config)?;
        let _gate = s
            .gate
            .try_lock()
            .map_err(|_| AppError::Config("Built-in LLM is busy".into()))?;
        let (m, mut runtime) = s.ready().await.map_err(AppError::Config)?;
        let mut local = config.clone();
        local.provider = "ollama".into();
        local.api_key.clear();
        local.base_url = format!("{}/v1", runtime.url);
        local.model = m.tag;
        let provider = crate::llm::openai::OpenAiProvider::with_local_client(s.client.clone());
        let mut stop = s.shutdown.subscribe();
        let result = tokio::select! { _=stop.changed()=>Err(AppError::Config("App is shutting down".into())), result=provider.polish(&local,request,on_chunk)=>result };
        if result.is_ok() {
            runtime.last_used = Instant::now();
            *s.runtime.lock().await = Some(runtime);
        }
        // Cancellation drops the owned runtime and asks the supervisor to stop.
        result
    }
}
#[tauri::command]
pub async fn get_local_llm_status() -> Result<Status> {
    Ok(service()?.status().await)
}
#[tauri::command]
pub async fn download_local_llm_model(id: String) -> Result<()> {
    service()?.download(&id).await
}
#[tauri::command]
pub fn cancel_local_llm_download() -> Result<()> {
    service()?.cancel();
    Ok(())
}
#[tauri::command]
pub async fn select_local_llm_model(id: String) -> Result<()> {
    service()?.select(&id).await
}
#[tauri::command]
pub async fn delete_local_llm_model(id: String) -> Result<()> {
    service()?.delete(&id).await
}
#[tauri::command]
pub async fn unload_local_llm() -> Result<()> {
    service()?.unload().await
}
#[tauri::command]
pub async fn test_local_llm() -> Result<u32> {
    service()?.test().await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_service() -> Arc<Service> {
        Service::new(
            std::env::temp_dir().join(format!("h-llm-test-{}", uuid::Uuid::new_v4())),
            PathBuf::from("/missing-runtime"),
        )
        .unwrap()
    }
    #[test]
    fn catalog_contains_only_approved_immutable_models() {
        let models = catalog();
        assert_eq!(
            models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            vec!["e2b", "e4b", "12b"]
        );
        for m in models {
            assert_eq!(
                format!("{:x}", Sha256::digest(m.manifest.as_bytes())),
                m.sha256
            );
            let raw: serde_json::Value = serde_json::from_str(&m.manifest).unwrap();
            assert_eq!(m.files.len(), raw["layers"].as_array().unwrap().len() + 1);
            for blob in m.files {
                assert!(blob.digest.starts_with("sha256:"));
                assert_eq!(blob.digest.len(), 71);
                assert!(blob.digest[7..].bytes().all(|b| b.is_ascii_hexdigit()));
                assert!(blob.size > 0);
            }
        }
        for id in [
            "../12b",
            "gemma4:12b",
            "31b-cloud",
            "http://example.com",
            "qwen3",
        ] {
            assert!(model(id).is_err());
        }
    }
    #[tokio::test]
    async fn unknown_or_incomplete_models_cannot_be_selected_or_downloaded() {
        let s = test_service();
        assert!(s.select("12b").await.is_err());
        assert!(s.download("../12b").await.is_err());
        assert!(s.delete("../12b").await.is_err());
        assert!(!s.root.join("selected").exists());
        fs::remove_dir_all(&s.root).await.unwrap();
    }
    #[tokio::test]
    async fn checksum_verification_rejects_same_size_corruption() {
        let s = test_service();
        let p = s.root.join("test");
        let b = Blob {
            size: 5,
            digest: format!("sha256:{:x}", Sha256::digest(b"hello")),
        };
        fs::write(&p, b"wrong").await.unwrap();
        assert!(s.verify(&p, &b).await.is_err());
        fs::write(&p, b"hello").await.unwrap();
        s.verify(&p, &b).await.unwrap();
        lock(&s.verified).clear();
        fs::write(&p, b"wrong").await.unwrap();
        assert!(s.verify(&p, &b).await.is_err());
        fs::remove_dir_all(&s.root).await.unwrap();
    }
    #[tokio::test]
    async fn busy_operations_do_not_change_selection() {
        let s = test_service();
        let _guard = s.gate.lock().await;
        assert!(s.select("12b").await.is_err());
        assert!(s.delete("12b").await.is_err());
        assert!(s.unload().await.is_err());
        assert!(s.status().await.busy);
        fs::remove_dir_all(&s.root).await.unwrap();
    }
    #[tokio::test]
    #[ignore = "Downloads only a small pinned config blob into a temporary directory"]
    async fn real_config_download_resumes_and_verifies() {
        let s = test_service();
        let blob = model("12b").unwrap().files[0].clone();
        s.download_blob(&blob).await.unwrap();
        let path = s.blob(&blob);
        let data = fs::read(&path).await.unwrap();
        fs::remove_file(&path).await.unwrap();
        fs::write(path.with_extension("part"), &data[..10])
            .await
            .unwrap();
        s.download_blob(&blob).await.unwrap();
        assert_eq!(fs::read(&path).await.unwrap(), data);
        fs::remove_dir_all(&s.root).await.unwrap();
    }
    #[tokio::test]
    #[ignore = "Requires an isolated test model root and bundled runtime; loads a real LLM"]
    async fn real_bundled_model_test() {
        let root = PathBuf::from(std::env::var("H_LLM_TEST_ROOT").expect("isolated test root"));
        let engines = PathBuf::from(std::env::var("H_LLM_TEST_ENGINES").expect("bundled engines"));
        let s = Service::new(root, engines).unwrap();
        s.select("12b").await.unwrap();
        for _ in 0..2 {
            eprintln!("local LLM test latency={} ms", s.test().await.unwrap());
        }
        assert!(SERVICE.set(s.clone()).is_ok());
        use crate::voice_intent::{VoiceIntent, VoiceIntentKind, VoiceOutputPlacement};
        let request = PolishRequest {
            raw_text: "어 내일 오후 3시 회의는 취소하지 말고 장소만 2층으로 바꿔줘".into(),
            context: crate::app_detector::types::ContextProfile::general_native().summary(),
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
        };
        let config = LlmConfig {
            provider: ID.into(),
            api_key: "must-not-be-sent".into(),
            base_url: "https://must-not-be-contacted.invalid".into(),
            max_tokens: 512,
            ..Default::default()
        };
        let output = Provider.polish(&config, &request, None).await.unwrap();
        eprintln!("local polish: {}", output.polished_text);
        assert!(output.polished_text.contains("3시"));
        assert!(output.polished_text.contains("2층"));
        assert!(output.polished_text.contains("취소하지"));
        s.unload().await.unwrap();
        assert!(s.runtime.lock().await.is_none());
    }
}
