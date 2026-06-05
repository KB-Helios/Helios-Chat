use crate::eie::config::{default_settings, render_config};
use crate::eie::types::{EieError, EieResult, EieSettings};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn load_settings(app: &AppHandle) -> EieResult<EieSettings> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(default_settings());
    }

    let json = fs::read_to_string(&path).map_err(|error| {
        EieError::new(
            "read_settings_failed",
            format!("Could not read settings: {error}"),
        )
    })?;

    serde_json::from_str(&json).map_err(|error| {
        EieError::new(
            "parse_settings_failed",
            format!("Could not parse settings: {error}"),
        )
    })
}

pub fn save_settings(app: &AppHandle, settings: &EieSettings) -> EieResult<()> {
    let path = settings_path(app)?;
    ensure_parent(&path)?;

    let json = serde_json::to_string_pretty(settings).map_err(|error| {
        EieError::new(
            "serialize_settings_failed",
            format!("Could not serialize settings: {error}"),
        )
    })?;

    fs::write(&path, json).map_err(|error| {
        EieError::new(
            "write_settings_failed",
            format!("Could not write settings: {error}"),
        )
    })
}

pub fn write_generated_config(app: &AppHandle, settings: &EieSettings) -> EieResult<PathBuf> {
    let path = generated_config_path(app)?;
    ensure_parent(&path)?;
    fs::write(&path, render_config(settings)?).map_err(|error| {
        EieError::new(
            "write_config_failed",
            format!("Could not write generated EIE config: {error}"),
        )
    })?;
    Ok(path)
}

pub fn generated_config_path(app: &AppHandle) -> EieResult<PathBuf> {
    Ok(eie_config_dir(app)?.join("engine.generated.yaml"))
}

pub fn log_dir(app: &AppHandle) -> EieResult<PathBuf> {
    let dir = app
        .path()
        .app_log_dir()
        .map_err(|error| EieError::new("resolve_log_dir_failed", error.to_string()))?;
    fs::create_dir_all(&dir)
        .map_err(|error| EieError::new("create_log_dir_failed", error.to_string()))?;
    Ok(dir)
}

fn settings_path(app: &AppHandle) -> EieResult<PathBuf> {
    Ok(eie_config_dir(app)?.join("settings.json"))
}

fn eie_config_dir(app: &AppHandle) -> EieResult<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| EieError::new("resolve_config_dir_failed", error.to_string()))?
        .join("eie");
    fs::create_dir_all(&dir)
        .map_err(|error| EieError::new("create_config_dir_failed", error.to_string()))?;
    Ok(dir)
}

fn ensure_parent(path: &PathBuf) -> EieResult<()> {
    let Some(parent) = path.parent() else {
        return Err(EieError::new(
            "invalid_path",
            format!("Path has no parent: {}", path.display()),
        ));
    };
    fs::create_dir_all(parent).map_err(|error| {
        EieError::new(
            "create_parent_dir_failed",
            format!("Could not create {}: {error}", parent.display()),
        )
    })
}
