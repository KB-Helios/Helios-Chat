use super::config::{validate_settings, validate_start_settings};
use super::manager::EieManager;
use super::models::{discover_gguf_models as scan_gguf_models, DiscoveredModel};
use super::types::{EieConfigPreview, EieError, EieLogLine, EieSettings, EieStatus};
use crate::app_config;
use std::path::PathBuf;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub fn get_eie_settings(
    app: AppHandle,
    manager: State<'_, EieManager>,
) -> Result<EieSettings, String> {
    let settings = app_config::load_settings(&app).map_err(command_error)?;
    manager
        .replace_settings(settings.clone())
        .map_err(command_error)?;
    Ok(settings)
}

#[tauri::command]
pub fn save_eie_settings(
    app: AppHandle,
    manager: State<'_, EieManager>,
    settings: EieSettings,
) -> Result<EieSettings, String> {
    validate_settings(&settings).map_err(command_error)?;
    app_config::save_settings(&app, &settings).map_err(command_error)?;
    manager
        .replace_settings(settings.clone())
        .map_err(command_error)?;
    Ok(settings)
}

#[tauri::command]
pub fn validate_eie_binary(path: String) -> Result<bool, String> {
    let path = PathBuf::from(path);
    let is_exe = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"));

    if path.is_file() && is_exe {
        Ok(true)
    } else {
        Err(command_error(EieError::new(
            "invalid_binary",
            "EIE binary must be an existing Windows .exe file.",
        )))
    }
}

#[tauri::command]
pub fn discover_gguf_models(
    manager: State<'_, EieManager>,
    model_directory: Option<String>,
) -> Result<Vec<DiscoveredModel>, String> {
    let directory = model_directory
        .map(PathBuf::from)
        .or_else(|| manager.settings().model_directory)
        .ok_or_else(|| {
            command_error(EieError::new(
                "missing_model_directory",
                "Choose a model directory before discovery.",
            ))
        })?;

    scan_gguf_models(&directory).map_err(command_error)
}

#[tauri::command]
pub fn generate_eie_config(
    app: AppHandle,
    manager: State<'_, EieManager>,
) -> Result<EieConfigPreview, String> {
    let path = app_config::generated_config_path(&app).map_err(command_error)?;
    manager
        .config_preview(path.display().to_string())
        .map_err(command_error)
}

#[tauri::command]
pub fn start_eie(app: AppHandle, manager: State<'_, EieManager>) -> Result<EieStatus, String> {
    let settings = app_config::load_settings(&app).map_err(command_error)?;
    validate_start_settings(&settings).map_err(command_error)?;
    manager
        .replace_settings(settings.clone())
        .map_err(command_error)?;
    let config_path = app_config::write_generated_config(&app, &settings).map_err(command_error)?;
    manager.set_config_path(Some(config_path.display().to_string()));
    manager.start(Some(app)).map_err(command_error)
}

#[tauri::command]
pub fn stop_eie(app: AppHandle, manager: State<'_, EieManager>) -> Result<EieStatus, String> {
    manager.stop(Some(app)).map_err(command_error)
}

#[tauri::command]
pub fn restart_eie(app: AppHandle, manager: State<'_, EieManager>) -> Result<EieStatus, String> {
    manager.stop(Some(app.clone())).map_err(command_error)?;
    start_eie(app, manager)
}

#[tauri::command]
pub fn get_eie_status(manager: State<'_, EieManager>) -> Result<EieStatus, String> {
    Ok(manager.status())
}

#[tauri::command]
pub fn get_eie_logs(manager: State<'_, EieManager>) -> Result<Vec<EieLogLine>, String> {
    Ok(manager.logs())
}

#[tauri::command]
pub fn clear_eie_logs(manager: State<'_, EieManager>) -> Result<(), String> {
    manager.clear_logs();
    Ok(())
}

#[tauri::command]
pub fn open_log_dir(app: AppHandle) -> Result<String, String> {
    let path = app_config::log_dir(&app).map_err(command_error)?;
    let path_string = path.display().to_string();
    app.opener()
        .open_path(&path_string, None::<&str>)
        .map_err(|error| command_error(EieError::new("open_log_dir_failed", error.to_string())))?;
    Ok(path_string)
}

fn command_error(error: EieError) -> String {
    serde_json::to_string(&error).unwrap_or_else(|_| error.message)
}
