//! Optional mobile endpoint owned by the desktop process. No Python runtime.
pub(crate) mod processing;
mod server;
mod wav;
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tauri::Manager;
use tokio::sync::{watch, Mutex, Semaphore};
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub enabled: bool,
    pub address: String,
    pub port: u16,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            address: "127.0.0.1".into(),
            port: 8787,
        }
    }
}
#[derive(Serialize)]
pub struct Status {
    pub config: Config,
    pub running: bool,
    pub warning: Option<String>,
    pub addresses: Vec<String>,
}
struct Running {
    stop: watch::Sender<bool>,
    task: tokio::task::JoinHandle<()>,
    alive: Arc<AtomicBool>,
}
struct Inner {
    config: Config,
    running: Option<Running>,
    warning: Option<String>,
}
pub struct Service {
    inner: Mutex<Inner>,
    path: PathBuf,
    busy: Arc<Semaphore>,
}
fn private(ip: Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback()
}
fn socket(config: &Config) -> Result<SocketAddrV4, String> {
    let ip = config
        .address
        .parse::<Ipv4Addr>()
        .map_err(|_| "내부 IPv4 주소를 입력하세요.")?;
    if !private(ip) || config.port == 0 {
        return Err("특정 내부 IPv4 주소와 1~65535 포트가 필요합니다. 전체 인터페이스 주소는 사용할 수 없습니다.".into());
    }
    Ok(SocketAddrV4::new(ip, config.port))
}
fn addresses() -> Vec<String> {
    let mut values = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|i| match i.ip() {
            std::net::IpAddr::V4(ip) if private(ip) => Some(ip.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}
impl Service {
    pub fn new(dir: PathBuf) -> Self {
        let path = dir.join("h-mobile.json");
        let config = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<Config>(&b).ok())
            .unwrap_or_default();
        Self {
            inner: Mutex::new(Inner {
                config,
                running: None,
                warning: None,
            }),
            path,
            busy: Arc::new(Semaphore::new(1)),
        }
    }
    async fn status(&self) -> Status {
        let inner = self.inner.lock().await;
        Status {
            config: inner.config.clone(),
            running: inner
                .running
                .as_ref()
                .is_some_and(|r| r.alive.load(Ordering::SeqCst)),
            warning: inner.warning.clone(),
            addresses: addresses(),
        }
    }
    pub async fn resume(&self, app: tauri::AppHandle) {
        let config = self.inner.lock().await.config.clone();
        if let Err(error) = self.apply(app, config).await {
            self.inner.lock().await.warning = Some(error);
        }
    }
    async fn apply(&self, app: tauri::AppHandle, config: Config) -> Result<Status, String> {
        let addr = if config.enabled {
            Some(socket(&config)?)
        } else {
            None
        };
        let mut inner = self.inner.lock().await;
        if config.enabled
            && inner.config.address == config.address
            && inner.config.port == config.port
            && inner
                .running
                .as_ref()
                .is_some_and(|r| r.alive.load(Ordering::SeqCst))
        {
            drop(inner);
            return Ok(self.status().await);
        }
        // Bind first so an invalid replacement does not tear down a working listener.
        let listener = if let Some(addr) = addr {
            Some(tokio::net::TcpListener::bind(addr).await.map_err(|e| {
                format!("이 주소에서 연결을 열지 못했습니다. IP와 포트 사용 여부를 확인하세요: {e}")
            })?)
        } else {
            None
        };
        super::audio_ducking::save_config_json(&self.path, &config)?;
        if let Some(running) = inner.running.take() {
            let _ = running.stop.send(true);
            let mut task = running.task;
            if tokio::time::timeout(std::time::Duration::from_secs(3), &mut task)
                .await
                .is_err()
            {
                task.abort();
            }
        }
        inner.config = config;
        inner.warning = None;
        if let Some(listener) = listener {
            let (stop, shutdown) = watch::channel(false);
            let mut server_stop = shutdown.clone();
            let state = server::ApiState {
                processor: Arc::new(server::Desktop(app)),
                busy: self.busy.clone(),
                shutdown,
            };
            let alive = Arc::new(AtomicBool::new(true));
            let running = alive.clone();
            let task = tokio::spawn(async move {
                let result = axum::serve(listener, server::router(state))
                    .with_graceful_shutdown(async move {
                        let _ = server_stop.changed().await;
                    })
                    .await;
                if let Err(error) = result {
                    tracing::warn!("Mobile listener stopped: {error}");
                }
                running.store(false, Ordering::SeqCst);
            });
            inner.running = Some(Running { stop, task, alive });
        }
        drop(inner);
        Ok(self.status().await)
    }
}
#[tauri::command]
pub async fn get_mobile_status(app: tauri::AppHandle) -> Result<Status, String> {
    Ok(app.state::<Service>().status().await)
}
#[tauri::command]
pub async fn set_mobile_config(app: tauri::AppHandle, config: Config) -> Result<Status, String> {
    app.state::<Service>().apply(app.clone(), config).await
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_explicit_private_bindings() {
        for address in ["127.0.0.1", "192.168.0.123", "10.1.2.3", "172.16.0.1"] {
            assert!(socket(&Config {
                address: address.into(),
                ..Default::default()
            })
            .is_ok());
        }
        for address in [
            "0.0.0.0",
            "8.8.8.8",
            "::",
            "localhost",
            "169.254.1.1",
            "172.32.0.1",
        ] {
            assert!(socket(&Config {
                address: address.into(),
                ..Default::default()
            })
            .is_err());
        }
    }
}
