//! macOS-only transport. Secrets travel through anonymous pipes, never argv or logs.
use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn request(op: &str, account: &str, value: Option<&str>) -> Result<Option<String>> {
    let expected = option_env!("H_CREDENTIAL_HELPER_SHA256")
        .ok_or_else(|| anyhow!("Build H with h-build-macos.sh to include its credential helper"))?;
    let executable = std::env::current_exe()?;
    let path = executable
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow!("Cannot locate app bundle"))?
        .join("Helpers/h-credential-helper");
    let bytes = std::fs::read(&path).context("read bundled credential helper")?;
    if format!("{:x}", Sha256::digest(&bytes)) != expected {
        return Err(anyhow!("Credential helper integrity check failed"));
    }
    let input =
        serde_json::to_vec(&serde_json::json!({"op": op, "account": account, "value": value}))?;
    if input.len() > 65536 {
        return Err(anyhow!("Credential request too large"));
    }
    let mut child = Command::new(path)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("start credential helper")?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("helper stdout missing"))?;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.take(65537).read_to_end(&mut bytes).map(|_| bytes)
    });
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("helper stdin missing"))?;
    // Start the deadline before any potentially blocked pipe write.
    let writer = std::thread::spawn(move || stdin.write_all(&input));
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(25)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(anyhow!("Keychain helper timed out"));
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(anyhow!("Keychain helper wait failed"));
            }
        }
    };
    let written = writer
        .join()
        .map_err(|_| anyhow!("Keychain request worker stopped"))?;
    let output = reader
        .join()
        .map_err(|_| anyhow!("Keychain response worker stopped"))??;
    let status = status?;
    written.context("write credential request")?;
    if output.len() > 65536 {
        return Err(anyhow!("Keychain response too large"));
    }
    // Do not attach the raw response to parse errors; it may contain credentials.
    let response: serde_json::Value =
        serde_json::from_slice(&output).map_err(|_| anyhow!("Invalid keychain helper response"))?;
    if !status.success() || response.get("error").is_some() {
        return Err(anyhow!(
            "{}",
            response
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Keychain operation failed")
        ));
    }
    Ok(response
        .get("value")
        .and_then(|v| v.as_str())
        .map(str::to_owned))
}
