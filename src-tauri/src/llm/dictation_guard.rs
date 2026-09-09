//! Conservative output bounds, not a semantic answer classifier.
use super::PolishRequest;
use crate::{error::AppError, voice_intent::VoiceIntentKind};

pub fn enabled(req: &PolishRequest) -> bool {
    req.voice_intent.kind == VoiceIntentKind::DictateInsert && !req.translate_enabled
}

pub fn validate(raw: &str, output: &str) -> Result<(), AppError> {
    let count = |text: &str| text.chars().filter(|c| c.is_alphanumeric()).count();
    if output.trim().is_empty()
        || output.contains("<think>")
        || output.contains("<|channel>")
        || count(output) > (count(raw) * 2).max(count(raw) + 120)
    {
        return Err(AppError::Output(
            "Dictation output failed fidelity bounds; keep the original transcript".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_questions_requests_and_formatting() {
        for (raw, output) in [
            ("어 왜 안 되는 거야 알려줘", "왜 안 되는 거야? 알려줘"),
            ("하나 테스트 둘 배포 금지", "1. 테스트\n2. 배포 금지"),
            ("정답은 사십이야", "정답은 42야"),
        ] {
            assert!(validate(raw, output).is_ok());
        }
    }
    #[test]
    fn rejects_empty_reasoning_and_large_expansion() {
        assert!(validate("질문", " ").is_err());
        assert!(validate("질문", "<think>설명을 생각해 보자").is_err());
        assert!(validate(
            "오프라인 STT는 어떻게 구현해",
            &"먼저 모델을 다운로드하고 실행합니다. ".repeat(20)
        )
        .is_err());
    }
}
