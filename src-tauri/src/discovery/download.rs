use super::hf::build_hf_download_url;
use super::types::{DownloadStatus, ModelDownload};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::eie::types::{EieError, EieResult};

pub fn resolve_download_destination(model_dir: &Path, filename: &str) -> EieResult<PathBuf> {
    if !filename.to_lowercase().ends_with(".gguf") {
        return Err(EieError::new(
            "invalid_download_filename",
            "Only GGUF files can be downloaded.",
        ));
    }

    let path = Path::new(filename);
    if path.is_absolute()
        || filename.contains("..")
        || filename.contains('\\')
        || filename.contains('/')
        || filename.contains(':')
    {
        return Err(EieError::new(
            "invalid_download_filename",
            "Download filename must be a plain GGUF filename.",
        ));
    }

    Ok(model_dir.join(filename))
}

pub fn temp_download_path(destination: &Path) -> PathBuf {
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("model.gguf");
    destination.with_file_name(format!("{file_name}.helios-download"))
}

#[derive(Clone, Default)]
pub struct ModelDownloadManager {
    inner: Arc<ModelDownloadInner>,
}

#[derive(Default)]
struct ModelDownloadInner {
    next_id: AtomicU64,
    jobs: Mutex<HashMap<u64, ModelDownload>>,
    cancelled: Mutex<HashSet<u64>>,
}

impl ModelDownloadManager {
    pub fn start_download(
        &self,
        app: AppHandle,
        model_dir: PathBuf,
        repo_id: String,
        filename: String,
    ) -> EieResult<ModelDownload> {
        if !model_dir.is_dir() {
            return Err(EieError::new(
                "missing_model_directory",
                format!("Model directory was not found at {}.", model_dir.display()),
            ));
        }

        let destination = resolve_download_destination(&model_dir, &filename)?;
        if destination.exists() {
            return Err(EieError::new(
                "model_file_exists",
                format!("Model file already exists at {}.", destination.display()),
            ));
        }

        let id = self.inner.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let job = ModelDownload {
            id,
            repo_id: repo_id.clone(),
            filename: filename.clone(),
            destination: destination.display().to_string(),
            received_bytes: 0,
            total_bytes: None,
            status: DownloadStatus::Running,
            error: None,
        };
        self.replace_job(job.clone());
        emit_download(&app, "model-download://progress", &job);

        let manager = self.clone();
        thread::spawn(move || {
            manager.run_download(app, id, repo_id, filename, destination);
        });

        Ok(job)
    }

    pub fn cancel(&self, app: Option<AppHandle>, job_id: u64) -> EieResult<ModelDownload> {
        self.inner
            .cancelled
            .lock()
            .expect("download cancellation set poisoned")
            .insert(job_id);

        let Some(job) = self.update_job(job_id, |job| {
            job.status = DownloadStatus::Cancelled;
            job.error = None;
        }) else {
            return Err(EieError::new(
                "download_not_found",
                format!("Download job {job_id} was not found."),
            ));
        };

        if let Some(app) = app {
            emit_download(&app, "model-download://progress", &job);
        }

        Ok(job)
    }

    pub fn list(&self) -> Vec<ModelDownload> {
        let mut jobs: Vec<_> = self
            .inner
            .jobs
            .lock()
            .expect("download jobs poisoned")
            .values()
            .cloned()
            .collect();
        jobs.sort_by(|left, right| right.id.cmp(&left.id));
        jobs
    }

    fn run_download(
        &self,
        app: AppHandle,
        job_id: u64,
        repo_id: String,
        filename: String,
        destination: PathBuf,
    ) {
        let result = self.download_to_destination(&app, job_id, &repo_id, &filename, &destination);

        match result {
            Ok(()) => {
                if let Some(job) = self.update_job(job_id, |job| {
                    job.status = DownloadStatus::Completed;
                    job.error = None;
                }) {
                    emit_download(&app, "model-download://completed", &job);
                    emit_download(&app, "model-download://progress", &job);
                }
            }
            Err(error) if error.code == "download_cancelled" => {
                let _ = fs::remove_file(temp_download_path(&destination));
                if let Some(job) = self.update_job(job_id, |job| {
                    job.status = DownloadStatus::Cancelled;
                    job.error = None;
                }) {
                    emit_download(&app, "model-download://progress", &job);
                }
            }
            Err(error) => {
                let _ = fs::remove_file(temp_download_path(&destination));
                if let Some(job) = self.update_job(job_id, |job| {
                    job.status = DownloadStatus::Failed;
                    job.error = Some(error.message.clone());
                }) {
                    emit_download(&app, "model-download://failed", &job);
                    emit_download(&app, "model-download://progress", &job);
                }
            }
        }
    }

    fn download_to_destination(
        &self,
        app: &AppHandle,
        job_id: u64,
        repo_id: &str,
        filename: &str,
        destination: &Path,
    ) -> EieResult<()> {
        let temp_path = temp_download_path(destination);
        let url = build_hf_download_url(repo_id, filename);
        let client = reqwest::blocking::Client::builder()
            .user_agent("Helios-Chat/0.1")
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|error| EieError::new("download_client_failed", error.to_string()))?;

        let mut response = client
            .get(url)
            .send()
            .map_err(|error| EieError::new("download_request_failed", error.to_string()))?;

        if !response.status().is_success() {
            return Err(EieError::new(
                "download_request_failed",
                format!("Hugging Face returned HTTP {}.", response.status()),
            ));
        }

        let total_bytes = response.content_length();
        let mut file = File::create(&temp_path).map_err(|error| {
            EieError::new(
                "download_write_failed",
                format!("Could not create {}: {error}", temp_path.display()),
            )
        })?;

        let mut received_bytes = 0;
        let mut buffer = [0_u8; 64 * 1024];

        loop {
            if self.is_cancelled(job_id) {
                return Err(EieError::new("download_cancelled", "Download was cancelled."));
            }

            let count = response
                .read(&mut buffer)
                .map_err(|error| EieError::new("download_read_failed", error.to_string()))?;
            if count == 0 {
                break;
            }

            file.write_all(&buffer[..count])
                .map_err(|error| EieError::new("download_write_failed", error.to_string()))?;
            received_bytes += count as u64;

            if let Some(job) = self.update_job(job_id, |job| {
                job.received_bytes = received_bytes;
                job.total_bytes = total_bytes;
                job.status = DownloadStatus::Running;
            }) {
                emit_download(app, "model-download://progress", &job);
            }
        }

        file.flush()
            .map_err(|error| EieError::new("download_write_failed", error.to_string()))?;
        drop(file);
        fs::rename(&temp_path, destination)
            .map_err(|error| EieError::new("download_finalize_failed", error.to_string()))
    }

    fn is_cancelled(&self, job_id: u64) -> bool {
        self.inner
            .cancelled
            .lock()
            .expect("download cancellation set poisoned")
            .contains(&job_id)
    }

    fn replace_job(&self, job: ModelDownload) {
        self.inner
            .jobs
            .lock()
            .expect("download jobs poisoned")
            .insert(job.id, job);
    }

    fn update_job(
        &self,
        job_id: u64,
        update: impl FnOnce(&mut ModelDownload),
    ) -> Option<ModelDownload> {
        let mut jobs = self.inner.jobs.lock().expect("download jobs poisoned");
        let job = jobs.get_mut(&job_id)?;
        update(job);
        Some(job.clone())
    }
}

fn emit_download(app: &AppHandle, event: &str, job: &ModelDownload) {
    let _ = app.emit(event, job);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_accepts_plain_gguf_filename() {
        let destination =
            resolve_download_destination(Path::new(r"C:\Models"), "model-q4.gguf").unwrap();

        assert_eq!(destination, PathBuf::from(r"C:\Models\model-q4.gguf"));
    }

    #[test]
    fn destination_rejects_traversal() {
        let error =
            resolve_download_destination(Path::new(r"C:\Models"), r"..\model.gguf").unwrap_err();

        assert_eq!(error.code, "invalid_download_filename");
    }

    #[test]
    fn destination_rejects_non_gguf() {
        let error = resolve_download_destination(Path::new(r"C:\Models"), "README.md").unwrap_err();

        assert_eq!(error.code, "invalid_download_filename");
    }

    #[test]
    fn temp_destination_stays_in_model_directory() {
        let destination =
            resolve_download_destination(Path::new(r"C:\Models"), "model-q4.gguf").unwrap();
        let temp = temp_download_path(&destination);

        assert_eq!(
            temp,
            PathBuf::from(r"C:\Models\model-q4.gguf.helios-download")
        );
    }
}
