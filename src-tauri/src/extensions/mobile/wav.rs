//! The mobile wire format is bounded PCM, so no external decoder is needed.
pub const MAX_PCM: usize = 16000 * 2 * 120;
pub const MAX_BODY: usize = MAX_PCM + 8192;
pub fn pcm(bytes: &[u8]) -> Result<&[u8], String> {
    let invalid = || "16kHz 모노 16비트 PCM WAV 파일이 필요합니다.".to_string();
    if bytes.len() < 44 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(invalid());
    }
    let u32_at = |n| u32::from_le_bytes(bytes[n..n + 4].try_into().unwrap()) as usize;
    if u32_at(4).checked_add(8) != Some(bytes.len()) {
        return Err(invalid());
    }
    let mut offset = 12;
    let mut format = false;
    let mut data = None;
    while offset + 8 <= bytes.len() {
        let size = u32_at(offset + 4);
        let start = offset + 8;
        let end = start
            .checked_add(size)
            .filter(|n| *n <= bytes.len())
            .ok_or_else(invalid)?;
        match &bytes[offset..offset + 4] {
            b"fmt " => {
                if format || size < 16 {
                    return Err(invalid());
                }
                let f = &bytes[start..end];
                if f[..2] != [1, 0]
                    || f[2..4] != [1, 0]
                    || f[4..8] != 16000u32.to_le_bytes()
                    || f[8..12] != 32000u32.to_le_bytes()
                    || f[12..14] != [2, 0]
                    || f[14..16] != [16, 0]
                {
                    return Err(invalid());
                }
                format = true;
            }
            b"data" => {
                if data.is_some() || size == 0 || size > MAX_PCM || size % 2 != 0 {
                    return Err(invalid());
                }
                data = Some(&bytes[start..end]);
            }
            _ => {}
        }
        offset = end.checked_add(size % 2).ok_or_else(invalid)?;
    }
    if !format || offset != bytes.len() {
        return Err(invalid());
    }
    data.ok_or_else(invalid)
}
#[cfg(test)]
mod tests {
    use super::*;
    pub fn wave() -> Vec<u8> {
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
    #[test]
    fn valid_audio_and_malformed_layouts() {
        let b = wave();
        assert_eq!(pcm(&b).unwrap(), [1, 0, 2, 0]);
        for index in [0, 4, 8, 20, 22, 24, 28, 32, 34, 40] {
            let mut broken = b.clone();
            broken[index] ^= 128;
            assert!(pcm(&broken).is_err(), "index {index}");
        }
        assert!(pcm(&b[..43]).is_err());
    }
}
