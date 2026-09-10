//! H-owned private MLX worker. Taking a worker out of the cache makes cancellation
//! drop/kill it; only a successful request can put it back for reuse.
use super::*;
use std::process::Stdio;
use tokio::process::{Child, Command};

pub struct Worker {
    child: Child,
    model: String,
    used: std::time::Instant,
}
impl Worker {
    pub async fn stop(mut self) {
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
    }
}

#[derive(Clone, Serialize)]
pub struct EngineStatus {
    pub preference: String,
    pub active: String,
    pub reason: Option<String>,
    pub resident: bool,
}

fn command(s: &Service) -> Command {
    let runtime = s.engines.join("mlx");
    let mut cmd = Command::new(runtime.join("bin/python3"));
    cmd.args(["-I", "-B"])
        .arg(runtime.join("worker.py"))
        .env("HF_HUB_OFFLINE", "1")
        .env("HF_HUB_DISABLE_TELEMETRY", "1")
        .env("TOKENIZERS_PARALLELISM", "false")
        .env("OMP_NUM_THREADS", "4")
        .kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    cmd
}

pub fn platform_reason(s: &Service) -> Option<String> {
    s.mlx_platform.get_or_init(|| detect_platform(s)).clone()
}
fn detect_platform(s: &Service) -> Option<String> {
    if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        return Some("MLX Qwen requires a native Apple Silicon Mac build".into());
    }
    if !s.engines.join("mlx/bin/python3").is_file() {
        return Some("This build does not include the MLX runtime".into());
    }
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("/usr/bin/sw_vers")
            .arg("-productVersion")
            .output()
            .ok();
        let major = out
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|v| v.split('.').next()?.parse::<u32>().ok())
            .unwrap_or(0);
        if major < 14 {
            return Some("MLX requires macOS 14 or later".into());
        }
    }
    None
}

async fn read_response(
    reader: &mut (impl tokio::io::AsyncRead + Unpin),
) -> Result<serde_json::Value, String> {
    let size = reader.read_u32().await.map_err(err)? as usize;
    if size > 65536 {
        return Err("MLX output exceeded limit".into());
    }
    let mut data = vec![0; size];
    reader.read_exact(&mut data).await.map_err(err)?;
    serde_json::from_slice(&data).map_err(err)
}

pub async fn probe(s: &Service) -> Result<(), String> {
    if let Some(reason) = platform_reason(s) {
        return Err(reason);
    }
    let mut guard = s.mlx_probe.lock().await;
    if let Some(result) = guard.as_ref() {
        return result.clone();
    }
    let result = async {
        let mut child = command(s).arg("--probe").spawn().map_err(err)?;
        let result = tokio::time::timeout(Duration::from_secs(15), async {
            let response =
                read_response(child.stdout.as_mut().ok_or("MLX stdout missing")?).await?;
            let status = child.wait().await.map_err(err)?;
            if status.success() && response["ok"] == true {
                Ok(())
            } else {
                Err("MLX GPU probe failed".into())
            }
        })
        .await
        .unwrap_or_else(|_| Err("MLX GPU probe timed out".into()));
        let _ = child.kill().await;
        let _ = child.wait().await;
        result
    }
    .await;
    *guard = Some(result.clone());
    result
}

pub async fn use_mlx(s: &Service, m: &Model) -> Result<bool, String> {
    if m.engine != "qwen" || s.engine_preference() == "cpu" {
        return Ok(false);
    }
    if let Some(reason) = platform_reason(s) {
        return if s.engine_preference() == "mlx" {
            Err(reason)
        } else {
            Ok(false)
        };
    }
    // A failed real GPU probe is surfaced, never silently treated as CPU success.
    probe(s)
        .await
        .map_err(|e| format!("{e}. Retry or select CPU in built-in STT settings."))?;
    Ok(true)
}

pub async fn status(s: &Service) -> EngineStatus {
    let preference = s.engine_preference();
    let unsupported = platform_reason(s);
    let reason = unsupported.clone().or_else(|| {
        s.mlx_probe
            .try_lock()
            .ok()
            .and_then(|p| p.as_ref().and_then(|r| r.as_ref().err().cloned()))
    });
    let qwen = s
        .selected()
        .and_then(|id| model(&id).ok())
        .is_some_and(|m| m.engine == "qwen");
    let active = if !qwen || preference == "cpu" {
        "cpu"
    } else if reason.is_some() {
        if preference == "mlx" || unsupported.is_none() {
            "unavailable"
        } else {
            "cpu"
        }
    } else if s
        .mlx_probe
        .try_lock()
        .ok()
        .is_some_and(|p| matches!(p.as_ref(), Some(Ok(()))))
    {
        "mlx"
    } else {
        "checking"
    };
    let mut worker = s.mlx_worker.lock().await;
    if let Some(w) = worker.as_mut() {
        if w.child.try_wait().ok().flatten().is_some() {
            worker.take();
        }
    }
    EngineStatus {
        preference,
        active: active.into(),
        reason,
        resident: worker.is_some(),
    }
}

pub async fn stop(s: &Service) {
    if let Some(w) = s.mlx_worker.lock().await.take() {
        w.stop().await;
    }
}

pub async fn reap_idle(s: &Service) {
    let mut cache = s.mlx_worker.lock().await;
    if let Some(w) = cache.as_mut() {
        if w.used.elapsed() >= Duration::from_secs(115)
            || w.child.try_wait().ok().flatten().is_some()
        {
            if let Some(w) = cache.take() {
                w.stop().await;
            }
        }
    }
}

async fn tokenizer(s: &Service, m: &Model) -> Result<(), String> {
    let manifest: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../../native/mlx-stt/tokenizers.json"))
            .map_err(err)?;
    let entry = manifest
        .iter()
        .find(|v| v["id"] == m.id)
        .ok_or("Missing tokenizer manifest")?;
    let bytes = fs::read(
        s.engines
            .join("mlx/tokenizers")
            .join(format!("{}.json", m.id)),
    )
    .await
    .map_err(err)?;
    if format!("{:x}", Sha256::digest(&bytes)) != entry["sha256"].as_str().unwrap_or("") {
        return Err("Tokenizer checksum mismatch".into());
    }
    let dest = s.root.join(&m.id).join("tokenizer_config.json");
    if fs::read(&dest).await.ok().as_deref() != Some(&bytes) {
        fs::write(&dest, bytes).await.map_err(err)?;
    }
    Ok(())
}

pub async fn transcribe(
    s: &Service,
    m: &Model,
    pcm: &[u8],
    language: Option<&str>,
) -> Result<String, String> {
    tokenizer(s, m).await?;
    let mut worker = s.mlx_worker.lock().await.take();
    if let Some(w) = worker.as_mut() {
        if w.model != m.id
            || w.used.elapsed() > Duration::from_secs(115)
            || w.child.try_wait().map_err(err)?.is_some()
        {
            worker.take().unwrap().stop().await;
        }
    }
    let mut worker = match worker {
        Some(w) => w,
        None => Worker {
            child: command(s).spawn().map_err(err)?,
            model: m.id.clone(),
            used: std::time::Instant::now(),
        },
    };
    let id = s
        .next_mlx_request
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let request = serde_json::to_vec(&serde_json::json!({"id": id, "model": s.root.join(&m.id), "pcm_bytes": pcm.len(), "language": language, "weights": m.files.iter().filter(|f| f.name.ends_with(".safetensors")).map(|f| &f.name).collect::<Vec<_>>()})).map_err(err)?;
    let result = async {
        let stdin = worker.child.stdin.as_mut().ok_or("MLX stdin missing")?;
        stdin.write_u32(request.len() as u32).await.map_err(err)?;
        stdin.write_all(&request).await.map_err(err)?;
        stdin.write_all(pcm).await.map_err(err)?;
        stdin.flush().await.map_err(err)?;
        let response =
            read_response(worker.child.stdout.as_mut().ok_or("MLX stdout missing")?).await?;
        if response["id"] != id || response["ok"] != true {
            return Err(
                "MLX transcription failed. Retry or select CPU in built-in STT settings.".into(),
            );
        }
        tracing::info!(
            "local_stt backend=mlx model={} load_ms={} inference_ms={}",
            m.id,
            response["load_ms"],
            response["inference_ms"]
        );
        response["text"]
            .as_str()
            .map(|s| s.trim().to_owned())
            .ok_or("Invalid MLX text response".into())
    }
    .await;
    match result {
        Ok(text) => {
            worker.used = std::time::Instant::now();
            *s.mlx_worker.lock().await = Some(worker);
            Ok(text)
        }
        Err(e) => {
            worker.stop().await;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn rejects_unbounded_and_truncated_frames() {
        assert!(read_response(&mut &65537u32.to_be_bytes()[..])
            .await
            .is_err());
        assert!(read_response(&mut &[0, 0, 0, 10, b'{'][..]).await.is_err());
    }
    #[tokio::test]
    #[cfg(unix)]
    async fn cancellation_kills_worker() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let s = Service::new(dir.path().into(), dir.path().into()).unwrap();
        fs::create_dir_all(dir.path().join("mlx/bin"))
            .await
            .unwrap();
        fs::create_dir_all(dir.path().join("mlx/tokenizers"))
            .await
            .unwrap();
        fs::create_dir_all(dir.path().join("qwen-1.7b"))
            .await
            .unwrap();
        fs::write(
            dir.path().join("mlx/tokenizers/qwen-1.7b.json"),
            include_bytes!("../../../../native/mlx-stt/tokenizers/qwen-1.7b.json"),
        )
        .await
        .unwrap();
        let pidfile = dir.path().join("pid");
        let binary = dir.path().join("mlx/bin/python3");
        fs::write(
            &binary,
            format!(
                "#!/bin/sh\necho $$ > '{}'\nexec sleep 60\n",
                pidfile.display()
            ),
        )
        .await
        .unwrap();
        std::fs::set_permissions(binary, std::fs::Permissions::from_mode(0o755)).unwrap();
        let task = tokio::spawn(async move {
            transcribe(&s, &model("qwen-1.7b").unwrap(), &[1, 0], None).await
        });
        for _ in 0..500 {
            if pidfile.exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        task.abort();
        let _ = task.await;
        let pid = fs::read_to_string(pidfile).await.unwrap();
        for _ in 0..100 {
            if !std::process::Command::new("kill")
                .args(["-0", pid.trim()])
                .stderr(Stdio::null())
                .status()
                .unwrap()
                .success()
            {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("MLX worker survived cancellation");
    }
}
