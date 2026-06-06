use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub complete: bool,
}

pub fn verify_sha256(path: &Path, expected_hex: &str) -> anyhow::Result<bool> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let actual = format!("{:x}", hasher.finalize());
    Ok(actual.eq_ignore_ascii_case(expected_hex))
}

pub async fn download_with_resume(
    app: &AppHandle,
    model_id: &str,
    url: &str,
    target: &Path,
    expected_sha256: Option<&str>,
) -> anyhow::Result<()> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let partial = target.with_extension("part");
    let existing = partial.metadata().map(|m| m.len()).unwrap_or(0);
    let client = reqwest::Client::new();
    let mut request = client.get(url);
    if existing > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={}-", existing));
    }

    let response = request.send().await?.error_for_status()?;
    let total = response.content_length().map(|length| length + existing);
    let mut stream = response.bytes_stream();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&partial)?;
    let mut downloaded = existing;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        let _ = app.emit(
            "model-download:progress",
            DownloadProgress {
                model_id: model_id.to_string(),
                downloaded_bytes: downloaded,
                total_bytes: total,
                complete: false,
            },
        );
    }

    file.flush()?;
    std::fs::rename(&partial, target)?;

    if let Some(expected) = expected_sha256 {
        if !verify_sha256(target, expected)? {
            anyhow::bail!("downloaded file checksum did not match expected SHA-256");
        }
    }

    let _ = app.emit(
        "model-download:progress",
        DownloadProgress {
            model_id: model_id.to_string(),
            downloaded_bytes: downloaded,
            total_bytes: total,
            complete: true,
        },
    );
    Ok(())
}
