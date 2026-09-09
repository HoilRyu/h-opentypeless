use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

#[derive(Default)]
pub(crate) struct CaptureFault {
    code: std::sync::atomic::AtomicU8,
    dropped_bytes: std::sync::atomic::AtomicU64,
    changed: tokio::sync::Notify,
}
impl CaptureFault {
    fn report(&self, code: u8, dropped_bytes: usize) {
        use std::sync::atomic::Ordering;
        self.dropped_bytes
            .fetch_add(dropped_bytes as u64, Ordering::Relaxed);
        if self
            .code
            .compare_exchange(0, code, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.changed.notify_one();
        }
    }
    fn failed(&self) -> bool {
        self.code.load(std::sync::atomic::Ordering::SeqCst) != 0
    }
    fn message(&self) -> String {
        let reason = match self.code.load(std::sync::atomic::Ordering::SeqCst) {
            1 => "Audio processing could not keep up with the microphone",
            2 => "The audio receiver disconnected",
            _ => "The microphone stopped working",
        };
        format!("{reason}. Recording was cancelled to avoid inserting incomplete text. Please try again.")
    }
    /// Interrupt even a provider that is stuck inside send_audio/disconnect.
    pub(crate) async fn guard<F: std::future::Future<Output = ()>>(
        &self,
        work: F,
    ) -> Option<String> {
        tokio::select! {
            biased;
            _ = async {
                while !self.failed() { self.changed.notified().await; }
            } => {},
            _ = work => {},
        }
        if self.failed() {
            tracing::error!(
                dropped_bytes = self
                    .dropped_bytes
                    .load(std::sync::atomic::Ordering::Relaxed),
                "Capture failed; discarding recording"
            );
            Some(self.message())
        } else {
            None
        }
    }
}

fn send_audio_chunk(sender: &mpsc::Sender<Vec<u8>>, fault: &CaptureFault, bytes: Vec<u8>) {
    match sender.try_send(bytes) {
        Ok(()) => {}
        Err(mpsc::error::TrySendError::Full(bytes)) => fault.report(1, bytes.len()),
        Err(mpsc::error::TrySendError::Closed(bytes)) => fault.report(2, bytes.len()),
    }
}

struct CaptureStartupNotifier {
    sender: Option<
        oneshot::Sender<std::result::Result<crate::recording_deadline::CaptureReadyAt, String>>,
    >,
}

struct CaptureStartupWaiter {
    receiver:
        oneshot::Receiver<std::result::Result<crate::recording_deadline::CaptureReadyAt, String>>,
}

fn capture_startup_channel() -> (CaptureStartupNotifier, CaptureStartupWaiter) {
    let (sender, receiver) = oneshot::channel();
    (
        CaptureStartupNotifier {
            sender: Some(sender),
        },
        CaptureStartupWaiter { receiver },
    )
}

impl CaptureStartupNotifier {
    fn ready(&mut self, ready_at: crate::recording_deadline::CaptureReadyAt) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(Ok(ready_at));
        }
    }

    fn failed(&mut self, message: String) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(Err(message));
        }
    }
}

impl CaptureStartupWaiter {
    async fn wait(self) -> std::result::Result<crate::recording_deadline::CaptureReadyAt, String> {
        self.receiver.await.unwrap_or_else(|_| {
            Err("Audio capture thread ended before reporting readiness".to_string())
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureState {
    Idle,
    Starting,
    Recording,
}

fn initial_capture_state() -> CaptureState {
    CaptureState::Starting
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub chunk_duration_ms: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            chunk_duration_ms: 20,
        }
    }
}

/// Maximum audio buffer size in samples before we stop accumulating.
/// ~24 MB of i16 samples ≈ 12.5 min at 16kHz mono, matching the STT provider limits.
const MAX_BUFFER_SAMPLES: usize = 12 * 1024 * 1024;
const AUDIO_CHANNEL_BUFFER_DURATION_MS: u32 = 60_000;

fn audio_channel_capacity(config: &AudioConfig) -> usize {
    let chunk_duration_ms = config.chunk_duration_ms.max(1);
    AUDIO_CHANNEL_BUFFER_DURATION_MS.div_ceil(chunk_duration_ms) as usize
}

// Dictation and Ask own separate sessions but share one physical microphone.
// Keep the lease on the native thread until stream destruction has completed.
struct CaptureLease(Arc<std::sync::atomic::AtomicBool>);
impl CaptureLease {
    fn acquire(state: Arc<std::sync::atomic::AtomicBool>) -> Result<Self> {
        use std::sync::atomic::Ordering;
        state
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| {
                super::lifecycle::event("capture_rejected_existing_owner");
                anyhow::anyhow!("Microphone is already in use by another H recording")
            })?;
        Ok(Self(state))
    }
}
impl Drop for CaptureLease {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
static CAPTURE_LEASE: std::sync::OnceLock<Arc<std::sync::atomic::AtomicBool>> =
    std::sync::OnceLock::new();

/// Handle to control audio capture running on a dedicated thread.
/// This is Send + Sync safe because it only holds channels and atomic state.
pub struct AudioCaptureHandle {
    fault: Arc<CaptureFault>,
    stop_tx: Option<std::sync::mpsc::Sender<()>>,
    startup_waiter: Option<CaptureStartupWaiter>,
    stopped: Option<std::sync::mpsc::Receiver<()>>,
    volume: Arc<Mutex<f32>>,
    state: Arc<Mutex<CaptureState>>,
}

impl AudioCaptureHandle {
    /// Start audio capture on a dedicated thread. Returns a handle and a receiver for audio chunks.
    pub fn start(config: AudioConfig) -> Result<(Self, mpsc::Receiver<Vec<u8>>)> {
        super::lifecycle::check().map_err(anyhow::Error::msg)?;
        let lease = CaptureLease::acquire(
            CAPTURE_LEASE
                .get_or_init(|| Arc::new(std::sync::atomic::AtomicBool::new(false)))
                .clone(),
        )?;
        let (stopped_tx, stopped_rx) = std::sync::mpsc::channel();
        let (audio_tx, audio_rx) = mpsc::channel::<Vec<u8>>(audio_channel_capacity(&config));
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        let volume = Arc::new(Mutex::new(0.0f32));
        let state = Arc::new(Mutex::new(initial_capture_state()));
        let (mut startup_notifier, startup_waiter) = capture_startup_channel();

        let fault = Arc::new(CaptureFault::default());
        let thread_fault = fault.clone();
        let vol_clone = volume.clone();
        let state_clone = state.clone();
        let failed_state = state.clone();

        // Audio capture must run on a dedicated OS thread because cpal::Stream is !Send
        std::thread::spawn(move || {
            if let Err(e) = run_capture(
                config,
                audio_tx,
                stop_rx,
                vol_clone,
                state_clone,
                &mut startup_notifier,
                thread_fault.clone(),
            ) {
                *failed_state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()) = CaptureState::Idle;
                thread_fault.report(3, 0);
                startup_notifier.failed(e.to_string());
                tracing::error!("Audio capture thread error: {}", e);
            }
            drop(lease);
            let _ = stopped_tx.send(());
        });

        Ok((
            Self {
                fault,
                stop_tx: Some(stop_tx),
                startup_waiter: Some(startup_waiter),
                stopped: Some(stopped_rx),
                volume,
                state,
            },
            audio_rx,
        ))
    }

    /// Wait until the platform backend has opened the input stream and
    /// `play()` has succeeded. The CPAL boundary is shared by CoreAudio,
    /// WASAPI, ALSA and PipeWire, so callers do not need platform delays.
    pub async fn wait_until_ready(&mut self) -> Result<crate::recording_deadline::CaptureReadyAt> {
        let waiter = self
            .startup_waiter
            .take()
            .ok_or_else(|| anyhow::anyhow!("Audio capture readiness was already consumed"))?;
        waiter.wait().await.map_err(anyhow::Error::msg)
    }

    pub fn stop(&mut self) {
        // Signal the capture thread to stop
        if self.stop_tx.is_some() {
            super::lifecycle::event("capture_stop_requested");
        }
        self.stop_tx = None;
        if let Some(stopped) = self.stopped.take() {
            if stopped
                .recv_timeout(std::time::Duration::from_secs(3))
                .is_err()
            {
                super::lifecycle::fail();
                tracing::error!("Capture teardown unconfirmed; further audio transitions disabled");
            }
        }
        *self.volume.lock().unwrap_or_else(|e| e.into_inner()) = 0.0;
        *self.state.lock().unwrap_or_else(|e| e.into_inner()) = CaptureState::Idle;
    }

    pub(crate) fn fault(&self) -> Arc<CaptureFault> {
        self.fault.clone()
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn state(&self) -> CaptureState {
        *self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl Drop for AudioCaptureHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Downsample audio from `from_rate` to `to_rate` (simple linear interpolation, mono).
fn downsample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;
        let s = if idx + 1 < samples.len() {
            samples[idx] as f64 * (1.0 - frac) + samples[idx + 1] as f64 * frac
        } else {
            samples[idx.min(samples.len() - 1)] as f64
        };
        out.push(s as f32);
    }
    out
}

/// Mix multi-channel audio down to mono by averaging channels.
fn to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return samples.to_vec();
    }
    let ch = channels as usize;
    samples
        .chunks(ch)
        .map(|frame| frame.iter().sum::<f32>() / ch as f32)
        .collect()
}

fn run_capture(
    config: AudioConfig,
    sender: mpsc::Sender<Vec<u8>>,
    stop_rx: std::sync::mpsc::Receiver<()>,
    volume: Arc<Mutex<f32>>,
    state: Arc<Mutex<CaptureState>>,
    startup_notifier: &mut CaptureStartupNotifier,
    fault: Arc<CaptureFault>,
) -> Result<()> {
    let transition = super::lifecycle::enter().map_err(anyhow::Error::msg)?;
    super::lifecycle::event("capture_start_requested");
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

    // CPAL 0.15.3 on macOS does not release the CFString returned by name().
    // Device identity is diagnostic only; do not allocate one leaked string per recording.
    #[cfg(not(target_os = "macos"))]
    tracing::info!("Using input device: {:?}", device.name());
    #[cfg(target_os = "macos")]
    tracing::info!("Using default input device");

    // Use the device's default config instead of forcing 16kHz mono
    let default_config = device.default_input_config()?;
    let device_sample_rate = default_config.sample_rate().0;
    let device_channels = default_config.channels();

    tracing::info!(
        "Device default config: {}Hz, {} channels, format: {:?}",
        device_sample_rate,
        device_channels,
        default_config.sample_format()
    );

    let stream_config = cpal::StreamConfig {
        channels: device_channels,
        sample_rate: cpal::SampleRate(device_sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let target_rate = config.sample_rate;
    let target_channels = config.channels;
    let samples_per_chunk = (target_rate * config.chunk_duration_ms / 1000) as usize;
    let buffer: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::with_capacity(samples_per_chunk)));

    let callback_fault = fault.clone();
    let device_fault = fault.clone();
    let stream = device.build_input_stream(
        &stream_config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if callback_fault.failed() || data.is_empty() {
                return;
            }
            // Calculate RMS volume from raw data
            let rms = (data.iter().map(|s| s * s).sum::<f32>() / data.len() as f32).sqrt();
            if let Ok(mut v) = volume.lock() {
                *v = rms.min(1.0);
            }

            // Convert to mono if needed
            let mono = if device_channels > target_channels {
                to_mono(data, device_channels)
            } else {
                data.to_vec()
            };

            // Downsample to target rate if needed
            let resampled = if device_sample_rate != target_rate {
                downsample(&mono, device_sample_rate, target_rate)
            } else {
                mono
            };

            // Convert f32 to i16 PCM and buffer
            let mut buf = buffer.lock().unwrap_or_else(|e| e.into_inner());
            for &sample in &resampled {
                if buf.len() >= MAX_BUFFER_SAMPLES {
                    callback_fault.report(1, std::mem::size_of_val(&resampled));
                    return;
                }
                let s = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                buf.push(s);
            }

            // Send complete chunks
            while buf.len() >= samples_per_chunk {
                let chunk: Vec<i16> = buf.drain(..samples_per_chunk).collect();
                let bytes: Vec<u8> = chunk.iter().flat_map(|s| s.to_le_bytes()).collect();
                send_audio_chunk(&sender, &callback_fault, bytes);
                if callback_fault.failed() {
                    return;
                }
            }
        },
        move |_err| {
            // Keep the realtime callback allocation-free on the error path.
            device_fault.report(3, 0);
        },
        None,
    )?;

    stream.play()?;
    super::lifecycle::event("capture_start_confirmed");
    let capture_ready_at = crate::recording_deadline::CaptureReadyAt::now();
    *state.lock().unwrap_or_else(|e| e.into_inner()) = CaptureState::Recording;
    startup_notifier.ready(capture_ready_at);
    tracing::info!(
        "Audio capture started (device: {}Hz {}ch -> target: {}Hz {}ch)",
        device_sample_rate,
        device_channels,
        target_rate,
        target_channels
    );

    drop(transition);
    // Block until stop signal (sender dropped)
    while !fault.failed() {
        match stop_rx.recv_timeout(std::time::Duration::from_millis(25)) {
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            _ => break,
        }
    }

    // Stream is dropped here, stopping capture
    // If the gate faulted, still release the stream; new device writes are disabled.
    let _transition = super::lifecycle::teardown();
    drop(stream);
    super::lifecycle::event("capture_stop_confirmed");
    *state.lock().unwrap_or_else(|e| e.into_inner()) = CaptureState::Idle;
    tracing::info!("Audio capture stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn overflow_interrupts_stalled_provider_and_does_not_leak_sessions() {
        for _ in 0..100 {
            let fault = Arc::new(CaptureFault::default());
            let weak = Arc::downgrade(&fault);
            let (tx, mut rx) = mpsc::channel(1);
            send_audio_chunk(&tx, &fault, vec![1; 640]);
            assert!(!fault.failed());
            send_audio_chunk(&tx, &fault, vec![2; 640]);
            let message = tokio::time::timeout(
                Duration::from_millis(100),
                fault.guard(std::future::pending()),
            )
            .await
            .unwrap()
            .unwrap();
            assert!(message.contains("could not keep up"));
            assert_eq!(
                fault
                    .dropped_bytes
                    .load(std::sync::atomic::Ordering::Relaxed),
                640
            );
            assert_eq!(rx.recv().await.unwrap(), vec![1; 640]);
            drop(tx);
            assert!(rx.recv().await.is_none());
            drop(fault);
            assert!(weak.upgrade().is_none());
        }
    }

    #[tokio::test]
    async fn device_failure_wakes_a_running_provider_and_preserves_first_error() {
        let fault = Arc::new(CaptureFault::default());
        let task_fault = fault.clone();
        let task = tokio::spawn(async move { task_fault.guard(std::future::pending()).await });
        tokio::task::yield_now().await;
        fault.report(3, 0);
        fault.report(2, 640);
        let message = tokio::time::timeout(Duration::from_secs(1), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(message.contains("microphone stopped"));
    }

    #[tokio::test]
    async fn normal_completion_and_closed_receiver_are_distinct() {
        let fault = CaptureFault::default();
        assert!(fault.guard(async {}).await.is_none());
        let (tx, rx) = mpsc::channel(1);
        drop(rx);
        send_audio_chunk(&tx, &fault, vec![0; 640]);
        assert!(fault
            .guard(async {})
            .await
            .unwrap()
            .contains("receiver disconnected"));
    }

    #[tokio::test]
    #[ignore = "opens the physical microphone 100 times; run explicitly with no other recording active"]
    async fn physical_microphone_repeated_start_stop() {
        for iteration in 0..100 {
            let (mut handle, mut audio) =
                AudioCaptureHandle::start(AudioConfig::default()).unwrap();
            tokio::time::timeout(Duration::from_secs(5), handle.wait_until_ready())
                .await
                .unwrap()
                .unwrap();
            let chunk = tokio::time::timeout(Duration::from_secs(2), audio.recv())
                .await
                .unwrap()
                .unwrap();
            assert!(!chunk.is_empty());
            handle.stop();
            assert!(
                !handle.fault.failed(),
                "capture failed on iteration {iteration}"
            );
            assert_eq!(handle.state(), CaptureState::Idle);
            super::super::lifecycle::check().unwrap();
            drop(handle);
            if iteration == 9 || iteration == 99 {
                let memory = std::process::Command::new("ps")
                    .args(["-o", "rss=", "-p", &std::process::id().to_string()])
                    .output()
                    .unwrap();
                println!(
                    "capture_probe completed={} rss_kib={}",
                    iteration + 1,
                    String::from_utf8_lossy(&memory.stdout).trim()
                );
            }
        }
    }

    #[test]
    fn overlapping_capture_is_rejected_until_owner_releases() {
        let state = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let owner = CaptureLease::acquire(state.clone()).unwrap();
        assert!(CaptureLease::acquire(state.clone()).is_err());
        drop(owner);
        for _ in 0..1000 {
            let owner = CaptureLease::acquire(state.clone()).unwrap();
            assert!(CaptureLease::acquire(state.clone()).is_err());
            drop(owner);
        }
        assert!(CaptureLease::acquire(state).is_ok());
    }
    #[test]
    fn native_thread_keeps_lease_until_teardown_finishes() {
        let state = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let lease = CaptureLease::acquire(state.clone()).unwrap();
        let (finish, wait) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            wait.recv().unwrap();
            drop(lease);
        });
        assert!(CaptureLease::acquire(state.clone()).is_err());
        finish.send(()).unwrap();
        thread.join().unwrap();
        assert!(CaptureLease::acquire(state).is_ok());
    }
    #[test]
    fn stop_waits_for_backend_teardown_acknowledgement() {
        let (stop_tx, stop_rx) = std::sync::mpsc::channel();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let completed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = completed.clone();
        let backend = std::thread::spawn(move || {
            let _ = stop_rx.recv();
            std::thread::sleep(Duration::from_millis(30));
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
            done_tx.send(()).unwrap();
        });
        let mut handle = AudioCaptureHandle {
            fault: Arc::new(CaptureFault::default()),
            stop_tx: Some(stop_tx),
            startup_waiter: None,
            stopped: Some(done_rx),
            volume: Arc::new(Mutex::new(0.5)),
            state: Arc::new(Mutex::new(CaptureState::Recording)),
        };
        handle.stop();
        assert!(completed.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(handle.state(), CaptureState::Idle);
        handle.stop(); // duplicate stop is harmless
        backend.join().unwrap();
    }
    #[test]
    fn capture_does_not_report_recording_before_the_backend_is_ready() {
        assert_eq!(initial_capture_state(), CaptureState::Starting);
    }

    #[test]
    fn audio_queue_preserves_a_minute_while_the_provider_connects() {
        assert_eq!(audio_channel_capacity(&AudioConfig::default()), 3_000);
    }

    #[tokio::test]
    async fn capture_startup_waits_for_the_backend_ready_signal() {
        let (_notifier, waiter) = capture_startup_channel();

        assert!(
            tokio::time::timeout(Duration::from_millis(20), waiter.wait())
                .await
                .is_err(),
            "capture startup completed before the backend reported readiness"
        );
    }

    #[tokio::test]
    async fn capture_startup_completes_after_the_backend_is_ready() {
        let (mut notifier, waiter) = capture_startup_channel();
        let ready_at = crate::recording_deadline::CaptureReadyAt::now();
        notifier.ready(ready_at);

        let observed = waiter.wait().await.unwrap();
        assert_eq!(observed.unix_millis, ready_at.unix_millis);
        assert_eq!(observed.monotonic, ready_at.monotonic);
    }

    #[tokio::test]
    async fn capture_startup_propagates_backend_failure() {
        let (mut notifier, waiter) = capture_startup_channel();
        notifier.failed("input device unavailable".to_string());

        assert_eq!(
            waiter.wait().await,
            Err("input device unavailable".to_string())
        );
    }
}
