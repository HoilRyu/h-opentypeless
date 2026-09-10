//! Bounded, advisory utterance previews. Final dictation always uses the original PCM.
use super::*;
use std::ops::Range;
use tokio::sync::mpsc;

const FRAME: usize = 512; // 16 ms, mono PCM16 at 16 kHz
const PRE: usize = 19 * FRAME;
const MIN_VOICE: usize = 6;
const END_SILENCE: usize = 44;
const MAX_SEGMENT: usize = 750 * FRAME;

#[derive(Default)]
pub struct Segmenter {
    vad: earshot::Detector,
    cursor: usize,
    start: Option<usize>,
    voice: usize,
    quiet: usize,
    overlap: bool,
}
impl Segmenter {
    pub fn next(&mut self, pcm: &[u8]) -> Option<(Range<usize>, bool)> {
        while self.cursor + FRAME <= pcm.len() {
            let mut samples = [0i16; 256];
            for (sample, bytes) in samples
                .iter_mut()
                .zip(pcm[self.cursor..self.cursor + FRAME].chunks_exact(2))
            {
                *sample = i16::from_le_bytes([bytes[0], bytes[1]]);
            }
            let speech = self.vad.predict_i16(&samples) >= 0.5;
            self.cursor += FRAME;
            if let Some(segment) = self.advance(speech) {
                return Some(segment);
            }
        }
        None
    }
    fn advance(&mut self, speech: bool) -> Option<(Range<usize>, bool)> {
        if speech {
            self.start
                .get_or_insert(self.cursor.saturating_sub(FRAME + PRE));
            self.voice += 1;
            self.quiet = 0;
        } else {
            self.quiet += 1;
        }
        let start = self.start?;
        let forced = self.cursor - start >= MAX_SEGMENT;
        if self.quiet < END_SILENCE && !forced {
            return None;
        }
        let keep = self.voice >= MIN_VOICE;
        let overlap = self.overlap;
        self.start = None;
        self.voice = 0;
        self.quiet = 0;
        self.overlap = false;
        if forced && speech {
            self.start = Some(self.cursor.saturating_sub(PRE));
            self.overlap = true;
        }
        keep.then_some((start..self.cursor, overlap))
    }
}

pub struct Preview {
    pub segments: Segmenter,
    tx: Option<mpsc::Sender<(Vec<u8>, bool)>>,
    stop: watch::Sender<bool>,
    result: watch::Receiver<String>,
    task: tokio::task::JoinHandle<()>,
}
impl Preview {
    pub fn start(
        service: Arc<Service>,
        model: Model,
        language: Option<String>,
        lease: Arc<tokio::sync::OwnedMutexGuard<()>>,
    ) -> Self {
        let (tx, mut rx) = mpsc::channel::<(Vec<u8>, bool)>(2);
        let (result_tx, result) = watch::channel(String::new());
        let (stop, mut stopping) = watch::channel(false);
        let task = tokio::spawn(async move {
            let _lease = lease;
            let mut shutdown = service.shutdown.subscribe();
            let mut text = String::new();
            loop {
                let job = tokio::select! {
                    biased;
                    _ = stopping.wait_for(|v| *v) => break,
                    _ = shutdown.wait_for(|v| *v) => break,
                    job = rx.recv() => job,
                };
                let Some((pcm, overlap)) = job else {
                    break;
                };
                let started = std::time::Instant::now();
                let result = tokio::select! {
                    _ = shutdown.wait_for(|v| *v) => break,
                    result = tokio::time::timeout(Duration::from_secs(15), super::provider::transcribe(&service, &model, &pcm, language.as_deref())) => result,
                };
                match result {
                    Ok(Ok(part)) => {
                        append(&mut text, &part, overlap);
                        if text.len() > 65536 {
                            break;
                        }
                        result_tx.send_replace(text.clone());
                        tracing::info!(
                            "local_stt preview_ms={} audio_ms={}",
                            started.elapsed().as_millis(),
                            pcm.len() / 32
                        );
                        // Slow hardware falls back to final-only before a backlog can grow.
                        if started.elapsed().as_millis() > (pcm.len() / 32).max(2000) as u128 {
                            break;
                        }
                    }
                    _ => {
                        tracing::warn!("local_stt preview unavailable; retaining full recording for final transcription");
                        break;
                    }
                }
            }
        });
        Self {
            segments: Segmenter::default(),
            tx: Some(tx),
            stop,
            result,
            task,
        }
    }
    pub fn feed(&mut self, pcm: &[u8]) {
        while self.tx.is_some() {
            let Some((range, overlap)) = self.segments.next(pcm) else {
                break;
            };
            if self
                .tx
                .as_ref()
                .unwrap()
                .try_send((pcm[range].to_vec(), overlap))
                .is_err()
            {
                self.tx.take();
                self.stop.send_replace(true);
                break;
            }
        }
    }
    pub async fn recv(&mut self) -> String {
        if self.result.changed().await.is_err() {
            return std::future::pending().await;
        }
        self.result.borrow_and_update().clone()
    }
    pub async fn finish(&mut self) {
        self.tx.take();
        self.stop.send_replace(true);
        // Do not start final inference until the in-flight preview has relinquished the engine.
        if tokio::time::timeout(Duration::from_secs(16), &mut self.task)
            .await
            .is_err()
        {
            self.task.abort();
            let _ = (&mut self.task).await;
        }
    }
}
impl Drop for Preview {
    fn drop(&mut self) {
        self.task.abort();
    }
}
fn append(text: &mut String, part: &str, overlap: bool) {
    let part = part.trim();
    if part.is_empty() {
        return;
    }
    let mut skip = 0;
    if overlap {
        for (end, _) in part.char_indices().skip(2) {
            if text.ends_with(&part[..end]) {
                skip = end;
            }
        }
        if part.chars().count() >= 2 && text.ends_with(part) {
            skip = part.len();
        }
    }
    if skip == part.len() {
        return;
    }
    if !text.is_empty() && skip == 0 {
        text.push(' ');
    }
    text.push_str(&part[skip..]);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    async fn fixture(script: &str) -> (tempfile::TempDir, Arc<Service>, Model) {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let service = Service::new(dir.path().into(), dir.path().into()).unwrap();
        let model = Model {
            id: "fixture".into(),
            name: "fixture".into(),
            engine: "whisper".into(),
            recommended_ram_gb: 1,
            license: "MIT".into(),
            files: vec![ModelFile {
                name: "weights".into(),
                url: String::new(),
                size: 1,
                sha256: format!("{:x}", Sha256::digest(b"x")),
            }],
        };
        fs::create_dir(dir.path().join("fixture")).await.unwrap();
        fs::write(dir.path().join("fixture/weights"), b"x")
            .await
            .unwrap();
        fs::write(service.binary(&model), script).await.unwrap();
        std::fs::set_permissions(
            service.binary(&model),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        (dir, service, model)
    }
    #[tokio::test]
    #[cfg(unix)]
    async fn preview_accumulates_without_emitting_final_or_losing_cancelled_receives() {
        let (_dir, service, model) =
            fixture("#!/bin/sh\ncat >/dev/null\nprintf '설정 버튼'\n").await;
        let lease = Arc::new(service.gate.clone().lock_owned().await);
        let mut preview = Preview::start(service.clone(), model, None, lease);
        assert!(
            tokio::time::timeout(Duration::from_millis(5), preview.recv())
                .await
                .is_err()
        );
        for expected in ["설정 버튼", "설정 버튼 설정 버튼"] {
            preview
                .tx
                .as_ref()
                .unwrap()
                .send((vec![0; 32000], false))
                .await
                .unwrap();
            assert_eq!(
                tokio::time::timeout(Duration::from_secs(5), preview.recv())
                    .await
                    .unwrap(),
                expected
            );
        }
        preview.finish().await;
        assert!(service.gate.try_lock().is_ok());
    }
    #[tokio::test]
    #[cfg(unix)]
    async fn broken_preview_preserves_final_engine_lease_and_does_not_spin() {
        let (_dir, service, model) = fixture("#!/bin/sh\nexit 1\n").await;
        let lease = Arc::new(service.gate.clone().lock_owned().await);
        let mut preview = Preview::start(service.clone(), model, None, lease.clone());
        preview
            .tx
            .as_ref()
            .unwrap()
            .send((vec![0; 32000], false))
            .await
            .unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(100), preview.recv())
                .await
                .is_err()
        );
        preview.finish().await;
        assert!(service.gate.try_lock().is_err());
        drop(lease);
        assert!(service.gate.try_lock().is_ok());
    }
    #[tokio::test]
    #[cfg(unix)]
    async fn dropping_preview_kills_inflight_process_and_releases_lease() {
        let (dir, service, model) =
            fixture("#!/bin/sh\necho $$ > \"$(dirname \"$0\")/pid\"\nexec sleep 60\n").await;
        let lease = Arc::new(service.gate.clone().lock_owned().await);
        let preview = Preview::start(service.clone(), model, None, lease);
        preview
            .tx
            .as_ref()
            .unwrap()
            .send((vec![0; 32000], false))
            .await
            .unwrap();
        let pidfile = dir.path().join("pid");
        for _ in 0..500 {
            if pidfile.exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let pid = fs::read_to_string(pidfile).await.unwrap();
        drop(preview);
        let _lease = tokio::time::timeout(Duration::from_secs(2), service.gate.lock())
            .await
            .unwrap();
        for _ in 0..100 {
            if !std::process::Command::new("kill")
                .args(["-0", pid.trim()])
                .stderr(std::process::Stdio::null())
                .status()
                .unwrap()
                .success()
            {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("preview process survived cancellation");
    }
    #[tokio::test]
    #[cfg(unix)]
    async fn full_queue_disables_preview_without_blocking_audio() {
        let (_dir, service, model) = fixture("#!/bin/sh\ncat >/dev/null\nprintf 'preview'\n").await;
        let lease = Arc::new(service.gate.clone().lock_owned().await);
        let mut preview = Preview::start(service, model, None, lease);
        for _ in 0..2 {
            preview
                .tx
                .as_ref()
                .unwrap()
                .try_send((vec![0; 512], false))
                .unwrap();
        }
        preview.segments.cursor = MAX_SEGMENT - FRAME;
        preview.segments.start = Some(0);
        preview.segments.voice = MIN_VOICE;
        let pcm = vec![0; MAX_SEGMENT];
        preview.feed(&pcm);
        assert!(preview.tx.is_none());
        assert!(*preview.stop.borrow());
        preview.finish().await;
    }
    #[test]
    fn silence_and_short_noise_do_not_create_segments() {
        let mut s = Segmenter::default();
        assert!(s.next(&vec![0; 32000]).is_none());
        for i in 0..60 {
            s.cursor += FRAME;
            assert!(s.advance(i < 3).is_none());
        }
    }
    #[test]
    fn silence_closes_utterance_with_preroll() {
        let mut s = Segmenter::default();
        for _ in 0..20 {
            s.cursor += FRAME;
            s.advance(false);
        }
        for _ in 0..10 {
            s.cursor += FRAME;
            assert!(s.advance(true).is_none());
        }
        for _ in 0..43 {
            s.cursor += FRAME;
            assert!(s.advance(false).is_none());
        }
        s.cursor += FRAME;
        let (range, overlap) = s.advance(false).unwrap();
        assert_eq!(range, FRAME..74 * FRAME);
        assert!(!overlap);
    }
    #[test]
    fn long_speech_is_bounded_and_overlap_is_explicit() {
        let mut s = Segmenter::default();
        let mut results = Vec::new();
        for _ in 0..1600 {
            s.cursor += FRAME;
            if let Some(r) = s.advance(true) {
                results.push(r);
            }
        }
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0.len(), MAX_SEGMENT);
        assert!(!results[0].1);
        assert!(results[1].1);
    }
    #[test]
    fn only_forced_boundaries_remove_overlap() {
        let mut t = "취소 버튼".to_string();
        append(&mut t, "버튼을 추가해 주세요", true);
        assert_eq!(t, "취소 버튼을 추가해 주세요");
        append(&mut t, "추가해 주세요", false);
        assert!(t.ends_with("주세요 추가해 주세요"));
    }
    #[test]
    fn arbitrary_byte_chunks_and_silence_are_bounded() {
        let mut s = Segmenter::default();
        let mut pcm = Vec::new();
        for _ in 0..1000 {
            pcm.extend_from_slice(&[0; 73]);
            assert!(s.next(&pcm).is_none());
        }
        assert!(pcm.len() - s.cursor < FRAME);
    }
}
