use crate::app_config;
use crate::discovery::download::ModelDownloadManager;
use crate::discovery::hf::fetch_hf_gguf_files;
use crate::discovery::manager::LlmfitManager;
use crate::discovery::types::{FitModel, FitModelQuery, HfGgufFile, LlmfitStatus, ModelDownload};
use crate::eie::types::EieError;
use serde_json::Value;
use std::path::PathBuf;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn validate_llmfit_binary(path: String) -> Result<bool, String> {
    let path = PathBuf::from(path);
    let is_exe = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"));

    if path.is_file() && is_exe {
        Ok(true)
    } else {
        Err(command_error(EieError::new(
            "invalid_llmfit_binary",
            "llmfit binary must be an existing Windows .exe file.",
        )))
    }
}

#[tauri::command]
pub fn get_llmfit_status(manager: State<'_, LlmfitManager>) -> Result<LlmfitStatus, String> {
    Ok(manager.status())
}

#[tauri::command]
pub fn start_llmfit(
    app: AppHandle,
    manager: State<'_, LlmfitManager>,
) -> Result<LlmfitStatus, String> {
    let settings = app_config::load_settings(&app).map_err(command_error)?;
    manager.start(&settings, Some(app)).map_err(command_error)
}

#[tauri::command]
pub fn stop_llmfit(
    app: AppHandle,
    manager: State<'_, LlmfitManager>,
) -> Result<LlmfitStatus, String> {
    manager.stop(Some(app)).map_err(command_error)
}

#[tauri::command]
pub fn restart_llmfit(
    app: AppHandle,
    manager: State<'_, LlmfitManager>,
) -> Result<LlmfitStatus, String> {
    let settings = app_config::load_settings(&app).map_err(command_error)?;
    manager.restart(&settings, Some(app)).map_err(command_error)
}

#[tauri::command]
pub fn get_llmfit_system(
    app: AppHandle,
    manager: State<'_, LlmfitManager>,
) -> Result<Value, String> {
    let settings = app_config::load_settings(&app).map_err(command_error)?;
    manager.system(&settings).map_err(command_error)
}

#[tauri::command]
pub fn list_fit_models(
    app: AppHandle,
    manager: State<'_, LlmfitManager>,
    query: FitModelQuery,
) -> Result<Vec<FitModel>, String> {
    let settings = app_config::load_settings(&app).map_err(command_error)?;
    manager
        .list_fit_models(&settings, query)
        .map_err(command_error)
}

#[tauri::command]
pub fn get_hf_gguf_files(repo_id: String) -> Result<Vec<HfGgufFile>, String> {
    fetch_hf_gguf_files(&repo_id).map_err(command_error)
}

#[tauri::command]
pub fn download_hf_gguf(
    app: AppHandle,
    manager: State<'_, ModelDownloadManager>,
    repo_id: String,
    filename: String,
) -> Result<ModelDownload, String> {
    let settings = app_config::load_settings(&app).map_err(command_error)?;
    let model_directory = settings.model_directory.ok_or_else(|| {
        command_error(EieError::new(
            "missing_model_directory",
            "Choose a model directory before downloading GGUF files.",
        ))
    })?;
    manager
        .start_download(app, model_directory, repo_id, filename)
        .map_err(command_error)
}

#[tauri::command]
pub fn cancel_model_download(
    app: AppHandle,
    manager: State<'_, ModelDownloadManager>,
    job_id: u64,
) -> Result<ModelDownload, String> {
    manager.cancel(Some(app), job_id).map_err(command_error)
}

#[tauri::command]
pub fn get_model_downloads(
    manager: State<'_, ModelDownloadManager>,
) -> Result<Vec<ModelDownload>, String> {
    Ok(manager.list())
}

fn command_error(error: EieError) -> String {
    serde_json::to_string(&error).unwrap_or_else(|_| error.message)
}
