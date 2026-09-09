//! H-owned native STT: opt-in pinned models and short-lived engine processes.
mod provider;
pub use provider::Provider;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{watch, Mutex as AsyncMutex},
};
pub const ID: &str = "builtin-stt";
pub const FINALIZE_SECONDS: u64 = 90;
pub const MAX_AUDIO: usize = 120 * 16000 * 2;
static SERVICE: OnceLock<Arc<Service>> = OnceLock::new();
pub fn service() -> Result<Arc<Service>, String> {
    SERVICE
        .get()
        .cloned()
        .ok_or("Built-in STT is not initialized".into())
}
#[derive(Clone, Deserialize, Serialize)]
pub struct ModelFile {
    pub name: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub engine: String,
    pub recommended_ram_gb: u32,
    pub license: String,
    pub files: Vec<ModelFile>,
}
pub fn catalog() -> Vec<Model> {
    serde_json::from_str(include_str!("catalog.json")).expect("compiled model catalog")
}
fn model(id: &str) -> Result<Model, String> {
    catalog()
        .into_iter()
        .find(|m| m.id == id)
        .ok_or("Unknown model".into())
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
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
    #[serde(flatten)]
    model: Model,
    installed: bool,
    available: bool,
    partial_bytes: u64,
}
#[derive(Serialize)]
pub struct Status {
    memory_gb: Option<u64>,
    models: Vec<ModelStatus>,
    selected: Option<String>,
    progress: Progress,
    busy: bool,
}
pub struct Service {
    qwen_platform: bool,
    memory_gb: Option<u64>,
    root: PathBuf,
    engines: PathBuf,
    gate: Arc<AsyncMutex<()>>,
    progress: Mutex<Progress>,
    cancel: Mutex<Option<watch::Sender<bool>>>,
    shutdown: watch::Sender<bool>,
    client: reqwest::Client,
}
impl Service {
    pub fn new(root: PathBuf, engines: PathBuf) -> Result<Arc<Self>, String> {
        std::fs::create_dir_all(&root).map_err(err)?;
        Ok(Arc::new(Self {
            qwen_platform: qwen_supported_platform(),
            memory_gb: physical_memory_gb(),
            root,
            engines,
            gate: Arc::new(AsyncMutex::new(())),
            progress: Mutex::new(Progress::default()),
            cancel: Mutex::new(None),
            shutdown: watch::channel(false).0,
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(20))
                .read_timeout(Duration::from_secs(60))
                .build()
                .map_err(err)?,
        }))
    }
    pub fn install(root: PathBuf, engines: PathBuf) -> Result<(), String> {
        SERVICE
            .set(Self::new(root, engines)?)
            .map_err(|_| "STT already initialized".into())
    }
    pub fn shutdown(&self) {
        self.shutdown.send_replace(true);
        self.cancel();
    }
    pub async fn wait_idle(&self) {
        // Let cancellation drop/kill the worker before the desktop runtime exits.
        let _ = tokio::time::timeout(Duration::from_secs(2), self.gate.lock()).await;
    }
    fn binary(&self, m: &Model) -> PathBuf {
        self.engines.join(format!(
            "{}{}",
            if m.engine == "whisper" {
                "whisper-cli"
            } else {
                "qwen_asr"
            },
            std::env::consts::EXE_SUFFIX
        ))
    }
    fn available(&self, m: &Model) -> bool {
        self.binary(m).is_file() && (m.engine == "whisper" || self.qwen_platform)
    }
    fn fingerprint(m: &Model) -> String {
        format!("{:x}", Sha256::digest(serde_json::to_vec(m).unwrap()))
    }
    async fn installed(&self, m: &Model) -> bool {
        let dir = self.root.join(&m.id);
        if fs::read_to_string(dir.join("verified"))
            .await
            .ok()
            .as_deref()
            != Some(Self::fingerprint(m).as_str())
        {
            return false;
        }
        for f in &m.files {
            if fs::metadata(dir.join(&f.name)).await.map(|x| x.len()).ok() != Some(f.size) {
                return false;
            }
        }
        true
    }
    fn selected(&self) -> Option<String> {
        std::fs::read_to_string(self.root.join("selected"))
            .ok()
            .filter(|id| model(id).is_ok())
    }
    async fn status(&self) -> Status {
        let mut models = Vec::new();
        for m in catalog() {
            let mut partial_bytes = 0;
            for f in &m.files {
                partial_bytes +=
                    fs::metadata(self.root.join(&m.id).join(format!("{}.part", f.name)))
                        .await
                        .map(|m| m.len())
                        .unwrap_or(0);
            }
            models.push(ModelStatus {
                installed: self.installed(&m).await,
                available: self.available(&m),
                partial_bytes,
                model: m,
            });
        }
        Status {
            memory_gb: self.memory_gb,
            models,
            selected: self.selected(),
            progress: self.progress.lock().unwrap().clone(),
            busy: self.gate.try_lock().is_err(),
        }
    }
    pub fn cancel(&self) {
        if let Some(tx) = self.cancel.lock().unwrap().as_ref() {
            tx.send_replace(true);
        }
    }
    async fn start_download(self: Arc<Self>, id: String) -> Result<(), String> {
        let m = model(&id)?;
        if !self.available(&m) {
            return Err("This build does not include this platform's engine".into());
        }
        let lease = self
            .gate
            .clone()
            .try_lock_owned()
            .map_err(|_| "STT is busy")?;
        let (tx, mut rx) = watch::channel(false);
        *self.cancel.lock().unwrap() = Some(tx);
        *self.progress.lock().unwrap() = Progress {
            model: id,
            total: m.files.iter().map(|f| f.size).sum(),
            phase: "downloading".into(),
            ..Default::default()
        };
        tauri::async_runtime::spawn(async move {
            let _lease = lease;
            let result = tokio::select! { biased; _ = rx.wait_for(|v| *v) => Err("Download paused".into()), r = self.download(&m) => r };
            let mut p = self.progress.lock().unwrap();
            p.phase = if result.is_ok() { "complete" } else { "paused" }.into();
            p.error = result.err();
            *self.cancel.lock().unwrap() = None;
        });
        Ok(())
    }
    async fn download(&self, m: &Model) -> Result<(), String> {
        let dir = self.root.join(&m.id);
        fs::create_dir_all(&dir).await.map_err(err)?;
        let mut completed = 0;
        for file in &m.files {
            let dest = dir.join(&file.name);
            if verify(&dest, file).await.is_ok() {
                completed += file.size;
                continue;
            }
            let part = dir.join(format!("{}.part", file.name));
            let mut offset = fs::metadata(&part).await.map(|m| m.len()).unwrap_or(0);
            if offset > file.size {
                fs::remove_file(&part).await.map_err(err)?;
                offset = 0;
            }
            if offset < file.size {
                let mut request = self.client.get(&file.url);
                if offset > 0 {
                    request = request.header(reqwest::header::RANGE, format!("bytes={offset}-"));
                }
                let mut response = request
                    .send()
                    .await
                    .map_err(err)?
                    .error_for_status()
                    .map_err(err)?;
                let append = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
                if append {
                    let range = response
                        .headers()
                        .get(reqwest::header::CONTENT_RANGE)
                        .and_then(|h| h.to_str().ok())
                        .unwrap_or("");
                    if range != format!("bytes {}-{}/{}", offset, file.size - 1, file.size) {
                        return Err("Invalid download range".into());
                    }
                } else if response.status() == reqwest::StatusCode::OK {
                    offset = 0;
                } else {
                    return Err("Unexpected download response".into());
                }
                let mut out = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(!append)
                    .append(append)
                    .open(&part)
                    .await
                    .map_err(err)?;
                while let Some(chunk) = response.chunk().await.map_err(err)? {
                    if offset + chunk.len() as u64 > file.size {
                        return Err("Model exceeds expected size".into());
                    }
                    out.write_all(&chunk).await.map_err(err)?;
                    offset += chunk.len() as u64;
                    self.progress.lock().unwrap().downloaded = completed + offset;
                }
                out.flush().await.map_err(err)?;
                out.sync_all().await.map_err(err)?;
            }
            self.progress.lock().unwrap().phase = "verifying".into();
            if let Err(error) = verify(&part, file).await {
                if fs::metadata(&part).await.map(|m| m.len()).ok() == Some(file.size) {
                    let _ = fs::remove_file(&part).await;
                }
                return Err(error);
            }
            if dest.exists() {
                fs::remove_file(&dest).await.map_err(err)?;
            }
            fs::rename(&part, &dest).await.map_err(err)?;
            completed += file.size;
            let mut p = self.progress.lock().unwrap();
            p.downloaded = completed;
            p.phase = "downloading".into();
        }
        fs::write(dir.join("verified"), Self::fingerprint(m))
            .await
            .map_err(err)
    }
}
async fn verify(path: &Path, f: &ModelFile) -> Result<(), String> {
    let mut input = fs::File::open(path).await.map_err(err)?;
    if input.metadata().await.map_err(err)?.len() != f.size {
        return Err("Download incomplete; resume to continue".into());
    }
    let mut hash = Sha256::new();
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let n = input.read(&mut buf).await.map_err(err)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    if format!("{:x}", hash.finalize()) != f.sha256 {
        return Err("Model checksum mismatch; download again".into());
    }
    Ok(())
}
#[tauri::command]
pub async fn get_local_stt_status() -> Result<Status, String> {
    Ok(service()?.status().await)
}
#[tauri::command]
pub async fn download_local_stt_model(id: String) -> Result<(), String> {
    service()?.start_download(id).await
}
#[tauri::command]
pub fn cancel_local_stt_download() -> Result<(), String> {
    service()?.cancel();
    Ok(())
}
#[tauri::command]
pub async fn select_local_stt_model(id: String) -> Result<(), String> {
    let s = service()?;
    let _lease = s.gate.try_lock().map_err(|_| "STT is busy")?;
    let m = model(&id)?;
    if !s.available(&m) || !s.installed(&m).await {
        return Err("Download a supported model first".into());
    }
    fs::write(s.root.join("selected.tmp"), &id)
        .await
        .map_err(err)?;
    #[cfg(target_os = "windows")]
    if s.root.join("selected").exists() {
        fs::remove_file(s.root.join("selected"))
            .await
            .map_err(err)?;
    }
    fs::rename(s.root.join("selected.tmp"), s.root.join("selected"))
        .await
        .map_err(err)
}
#[tauri::command]
pub async fn delete_local_stt_model(id: String) -> Result<(), String> {
    let s = service()?;
    let _lease = s.gate.try_lock().map_err(|_| "STT is busy")?;
    let m = model(&id)?;
    let dir = s.root.join(&m.id);
    if dir.exists() {
        fs::remove_dir_all(dir).await.map_err(err)?;
    }
    if s.selected().as_deref() == Some(&id) {
        fs::remove_file(s.root.join("selected"))
            .await
            .map_err(err)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(url: String, body: &[u8]) -> Model {
        Model {
            id: "fixture".into(),
            name: "Fixture".into(),
            engine: "whisper".into(),
            recommended_ram_gb: 4,
            license: "MIT".into(),
            files: vec![ModelFile {
                name: "model.bin".into(),
                url,
                size: body.len() as u64,
                sha256: format!("{:x}", Sha256::digest(body)),
            }],
        }
    }
    async fn server(response: String, expected: &'static str) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/model", listener.local_addr().unwrap());
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0; 4096];
            let n = stream.read(&mut buf).await.unwrap();
            assert!(String::from_utf8_lossy(&buf[..n])
                .to_lowercase()
                .contains(expected));
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        url
    }
    #[test]
    fn catalog_has_pinned_safe_files() {
        assert!(model("../../selected").is_err());
        let models = catalog();
        assert_eq!(models.len(), 5);
        for m in models {
            assert!(m.files.iter().all(|f| f.sha256.len() == 64
                && f.sha256.bytes().all(|b| b.is_ascii_hexdigit())
                && f.size > 0
                && !f.name.contains('/')
                && !f.name.contains('\\')
                && f.url.starts_with("https://huggingface.co/")
                && !f.url.contains("/main/")));
        }
    }
    #[tokio::test]
    async fn verifies_content_not_only_size() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file");
        let m = fixture("".into(), b"hello");
        fs::write(&path, b"hello").await.unwrap();
        assert!(verify(&path, &m.files[0]).await.is_ok());
        fs::write(&path, b"jello").await.unwrap();
        assert!(verify(&path, &m.files[0]).await.is_err());
    }
    #[tokio::test]
    async fn download_resumes_verified_range() {
        let url = server("HTTP/1.1 206 Partial Content\r\nContent-Length: 3\r\nContent-Range: bytes 2-4/5\r\nConnection: close\r\n\r\nllo".into(), "range: bytes=2-").await;
        let dir = tempfile::tempdir().unwrap();
        let s = Service::new(dir.path().into(), dir.path().into()).unwrap();
        let m = fixture(url, b"hello");
        fs::create_dir(dir.path().join("fixture")).await.unwrap();
        fs::write(dir.path().join("fixture/model.bin.part"), b"he")
            .await
            .unwrap();
        s.download(&m).await.unwrap();
        assert!(s.installed(&m).await);
        assert!(!dir.path().join("fixture/model.bin.part").exists());
    }
    #[tokio::test]
    async fn range_ignored_restarts_without_duplicate_prefix() {
        let url = server(
            "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello".into(),
            "range: bytes=2-",
        )
        .await;
        let dir = tempfile::tempdir().unwrap();
        let s = Service::new(dir.path().into(), dir.path().into()).unwrap();
        let m = fixture(url, b"hello");
        fs::create_dir(dir.path().join("fixture")).await.unwrap();
        fs::write(dir.path().join("fixture/model.bin.part"), b"he")
            .await
            .unwrap();
        s.download(&m).await.unwrap();
        assert!(s.installed(&m).await);
    }
    #[tokio::test]
    async fn corrupt_download_never_installed_and_can_retry() {
        let url = server(
            "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\njello".into(),
            "get /model",
        )
        .await;
        let dir = tempfile::tempdir().unwrap();
        let s = Service::new(dir.path().into(), dir.path().into()).unwrap();
        let m = fixture(url, b"hello");
        assert!(s.download(&m).await.is_err());
        assert!(!s.installed(&m).await);
        assert!(!dir.path().join("fixture/model.bin.part").exists());
    }
    #[tokio::test]
    async fn rejects_wrong_range_without_destroying_partial() {
        let url = server("HTTP/1.1 206 Partial Content\r\nContent-Length: 3\r\nContent-Range: bytes 1-3/5\r\nConnection: close\r\n\r\nllo".into(), "range: bytes=2-").await;
        let dir = tempfile::tempdir().unwrap();
        let s = Service::new(dir.path().into(), dir.path().into()).unwrap();
        let m = fixture(url, b"hello");
        fs::create_dir(dir.path().join("fixture")).await.unwrap();
        fs::write(dir.path().join("fixture/model.bin.part"), b"he")
            .await
            .unwrap();
        assert!(s.download(&m).await.is_err());
        assert_eq!(
            fs::read(dir.path().join("fixture/model.bin.part"))
                .await
                .unwrap(),
            b"he"
        );
    }
    #[tokio::test]
    #[ignore = "explicit multi-GB model download; set H_LOCAL_STT_TEST_ROOT and H_LOCAL_STT_ENGINE_DIR"]
    async fn download_real_models() {
        let root = PathBuf::from(std::env::var("H_LOCAL_STT_TEST_ROOT").unwrap());
        let s = Service::new(
            root,
            PathBuf::from(std::env::var("H_LOCAL_STT_ENGINE_DIR").unwrap()),
        )
        .unwrap();
        for id in ["base", "qwen-0.6b"] {
            s.download(&model(id).unwrap()).await.unwrap();
        }
    }
}

fn physical_memory_gb() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("/usr/sbin/sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()?;
        String::from_utf8(out.stdout)
            .ok()?
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| v / (1024 * 1024 * 1024))
    }
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/meminfo").ok()?;
        text.lines()
            .find_map(|l| l.strip_prefix("MemTotal:"))?
            .split_whitespace()
            .next()?
            .parse::<u64>()
            .ok()
            .map(|v| v.div_ceil(1024 * 1024))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    None
}

fn qwen_supported_platform() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Qwen's BLAS backend uses the Accelerate 13.3 ABI.
        let version = std::process::Command::new("/usr/bin/sw_vers")
            .arg("-productVersion")
            .output()
            .ok();
        let version = version
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        let mut parts = version
            .trim()
            .split('.')
            .filter_map(|p| p.parse::<u32>().ok());
        (parts.next().unwrap_or(0), parts.next().unwrap_or(0)) >= (13, 3)
    }
    #[cfg(not(target_os = "macos"))]
    cfg!(target_os = "linux")
}
