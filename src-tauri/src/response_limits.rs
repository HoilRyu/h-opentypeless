//! Bound model responses and decode SSE only after complete UTF-8 lines arrive.
use crate::error::AppError;
use futures_util::StreamExt;
pub const MAX_RESPONSE: usize = 8 * 1024 * 1024;
const MAX_LINE: usize = 1024 * 1024;
fn oversized() -> AppError {
    AppError::Config("Model response exceeded the supported size".into())
}
pub async fn text(response: reqwest::Response) -> Result<String, AppError> {
    if response
        .content_length()
        .is_some_and(|n| n > MAX_RESPONSE as u64)
    {
        return Err(oversized());
    }
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if chunk.len() > MAX_RESPONSE - bytes.len() {
            return Err(oversized());
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| AppError::Config("Model response is not UTF-8".into()))
}
pub async fn json(response: reqwest::Response) -> Result<serde_json::Value, AppError> {
    serde_json::from_str(&text(response).await?).map_err(|e| AppError::Config(e.to_string()))
}
#[derive(Default)]
pub struct Lines {
    pending: Vec<u8>,
    total: usize,
}
impl Lines {
    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<String>, AppError> {
        if chunk.len() > MAX_RESPONSE - self.total {
            return Err(oversized());
        }
        self.total += chunk.len();
        let mut lines = Vec::new();
        for part in chunk.split_inclusive(|b| *b == b'\n') {
            if part.len() > MAX_LINE - self.pending.len() {
                return Err(oversized());
            }
            self.pending.extend_from_slice(part);
            if part.last() == Some(&b'\n') {
                let line = String::from_utf8(std::mem::take(&mut self.pending))
                    .map_err(|_| AppError::Config("Invalid UTF-8 event".into()))?;
                lines.push(line);
            }
        }
        Ok(lines)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn korean_survives_every_byte_boundary() {
        let input = "data: {\"text\":\"한글과 🙂\"}\r\n\n";
        for split in 0..=input.len() {
            let mut lines = Lines::default();
            let mut actual = lines.push(&input.as_bytes()[..split]).unwrap();
            actual.extend(lines.push(&input.as_bytes()[split..]).unwrap());
            assert_eq!(actual.concat(), input);
        }
    }
    #[test]
    fn newline_free_and_total_responses_are_bounded() {
        let mut lines = Lines::default();
        assert!(lines.push(&vec![b'x'; MAX_LINE + 1]).is_err());
        let mut lines = Lines::default();
        let chunk = vec![b'\n'; 1024];
        for _ in 0..MAX_RESPONSE / 1024 {
            lines.push(&chunk).unwrap();
        }
        assert!(lines.push(b"\n").is_err());
    }
}
