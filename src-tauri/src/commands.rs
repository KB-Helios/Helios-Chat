use crate::catalog::{catalog_by_id, load_builtin_catalog, CatalogModel};
use crate::db;
use crate::download;
use crate::eie::{self, BuildResult, ChatRequest, ChatResponse, EngineStatus};
use crate::paths::AppPaths;
use crate::settings::{load_settings, save_settings, HeliosSettings};
use crate::setup::{self, BuildBackend, ToolStatus};
use crate::RuntimeState;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn setup_check_prereqs() -> Vec<ToolStatus> {
    setup::check_prereqs()
}

#[tauri::command]
pub async fn setup_build_eie(app: AppHandle) -> Result<BuildResult, String> {
    let tools = setup::check_prereqs();
    let backend = setup::choose_build_backend(&tools);
    if backend == BuildBackend::Blocked {
        return Err("Missing Git, CMake, or MSVC C++ build tools.".to_string());
    }

    let paths = AppPaths::resolve(&app).map_err(to_string)?;
    paths.ensure().map_err(to_string)?;
    let log_path = paths.logs.join("eie-build.log");
    let backend_name = match backend {
        BuildBackend::Cuda => "cuda",
        BuildBackend::Cpu => "cpu",
        BuildBackend::Blocked => "blocked",
    };
    let binary_path = paths.engine.join(format!("eie-server{}", std::env::consts::EXE_SUFFIX));
    let source_dir = vendor_eie_dir();
    if !source_dir.join("CMakeLists.txt").exists() {
        return Err(format!("Vendored EIE source not found at {}", source_dir.display()));
    }
    let build_dir = paths.engine.join(format!("build-{}", backend_name));

    let _ = app.emit("setup:progress", format!("Configuring EIE {}", backend_name));
    let mut configure_args = vec![
        "-S".to_string(),
        source_dir.display().to_string(),
        "-B".to_string(),
        build_dir.display().to_string(),
        "-DCMAKE_BUILD_TYPE=Release".to_string(),
    ];
    if backend == BuildBackend::Cuda {
        configure_args.push("-DGGML_CUDA=ON".to_string());
    }
    run_logged("cmake", &configure_args, &log_path).map_err(to_string)?;

    let _ = app.emit("setup:progress", "Building EIE");
    run_logged(
        "cmake",
        &[
            "--build".to_string(),
            build_dir.display().to_string(),
            "--config".to_string(),
            "Release".to_string(),
        ],
        &log_path,
    )
    .map_err(to_string)?;

    let built = find_eie_binary(&build_dir)
        .ok_or_else(|| format!("EIE build completed but no eie-server binary was found in {}", build_dir.display()))?;
    std::fs::copy(&built, &binary_path).map_err(to_string)?;
    let _ = app.emit("setup:progress", format!("EIE ready at {}", binary_path.display()));

    Ok(BuildResult {
        backend: backend_name.to_string(),
        binary_path,
        log_path,
    })
}

#[tauri::command]
pub async fn engine_start(app: AppHandle, state: State<'_, RuntimeState>) -> Result<EngineStatus, String> {
    let paths = AppPaths::resolve(&app).map_err(to_string)?;
    paths.ensure().map_err(to_string)?;
    db::migrate(&paths.database).map_err(to_string)?;
    let settings = load_settings(&paths.settings).map_err(to_string)?;
    let default_model_path = default_model_path(&settings, &paths);
    let input = eie::config_input_from_settings(&settings, paths.models.clone(), default_model_path);
    eie::write_eie_config(&paths.eie_config, &input).map_err(to_string)?;

    let binary_path = paths.engine.join("eie-server.exe");
    if !binary_path.exists() {
        return Err(format!(
            "EIE binary not found at {}. Run first-run setup before starting the engine.",
            binary_path.display()
        ));
    }

    let status = {
        let mut runtime = state.engine.lock().map_err(|_| "engine lock poisoned".to_string())?;
        runtime
            .start(&binary_path, &paths.eie_config, settings.engine_port)
            .map_err(to_string)?
    };
    let _ = app.emit("engine:status", &status);
    Ok(status)
}

#[tauri::command]
pub fn engine_stop(app: AppHandle, state: State<'_, RuntimeState>) -> Result<EngineStatus, String> {
    let settings = settings_for_app(&app)?;
    let status = {
        let mut runtime = state.engine.lock().map_err(|_| "engine lock poisoned".to_string())?;
        runtime.stop().map_err(to_string)?;
        runtime.status(settings.engine_port)
    };
    let _ = app.emit("engine:status", &status);
    Ok(status)
}

#[tauri::command]
pub fn engine_status(app: AppHandle, state: State<'_, RuntimeState>) -> Result<EngineStatus, String> {
    let settings = settings_for_app(&app)?;
    let runtime = state.engine.lock().map_err(|_| "engine lock poisoned".to_string())?;
    Ok(runtime.status(settings.engine_port))
}

#[tauri::command]
pub fn models_catalog() -> Result<Vec<CatalogModel>, String> {
    load_builtin_catalog().map_err(to_string)
}

#[tauri::command]
pub async fn models_download(app: AppHandle, model_id: String) -> Result<PathBuf, String> {
    let catalog = load_builtin_catalog().map_err(to_string)?;
    let by_id = catalog_by_id(&catalog);
    let model = by_id
        .get(&model_id)
        .ok_or_else(|| format!("Unknown catalog model: {}", model_id))?;
    let paths = AppPaths::resolve(&app).map_err(to_string)?;
    paths.ensure().map_err(to_string)?;
    let target = paths.models.join(&model.hf_file);
    download::download_with_resume(
        &app,
        &model.id,
        &model.download_url,
        &target,
        model.sha256.as_deref(),
    )
    .await
    .map_err(to_string)?;
    Ok(target)
}

#[tauri::command]
pub fn models_import_local(app: AppHandle, source_path: PathBuf) -> Result<PathBuf, String> {
    let paths = AppPaths::resolve(&app).map_err(to_string)?;
    paths.ensure().map_err(to_string)?;
    let file_name = source_path
        .file_name()
        .ok_or_else(|| "Selected model path has no filename".to_string())?;
    let target = paths.models.join(file_name);
    std::fs::copy(&source_path, &target).map_err(to_string)?;
    Ok(target)
}

#[tauri::command]
pub fn models_set_default(app: AppHandle, model_id: String) -> Result<HeliosSettings, String> {
    let paths = AppPaths::resolve(&app).map_err(to_string)?;
    let mut settings = load_settings(&paths.settings).map_err(to_string)?;
    settings.default_model_id = Some(model_id);
    save_settings(&paths.settings, &settings).map_err(to_string)?;
    Ok(settings)
}

#[tauri::command]
pub async fn models_load(app: AppHandle, state: State<'_, RuntimeState>, model_id: String) -> Result<(), String> {
    let status = engine_status(app.clone(), state)?;
    if !status.running {
        return Err("Start EIE before loading a model.".to_string());
    }
    let client = reqwest::Client::new();
    client
        .post(format!("{}/v1/admin/models/load", status.endpoint))
        .json(&serde_json::json!({ "model": model_id }))
        .send()
        .await
        .map_err(to_string)?
        .error_for_status()
        .map_err(to_string)?;
    let _ = app.emit("model-load:progress", "loaded");
    Ok(())
}

#[tauri::command]
pub async fn models_unload(app: AppHandle, state: State<'_, RuntimeState>, model_id: String) -> Result<(), String> {
    let status = engine_status(app, state)?;
    if !status.running {
        return Err("EIE is not running.".to_string());
    }
    reqwest::Client::new()
        .post(format!("{}/v1/admin/models/unload", status.endpoint))
        .json(&serde_json::json!({ "model": model_id }))
        .send()
        .await
        .map_err(to_string)?
        .error_for_status()
        .map_err(to_string)?;
    Ok(())
}

#[tauri::command]
pub async fn chat_send(app: AppHandle, state: State<'_, RuntimeState>, request: ChatRequest) -> Result<ChatResponse, String> {
    let settings = settings_for_app(&app)?;
    let status = {
        let runtime = state.engine.lock().map_err(|_| "engine lock poisoned".to_string())?;
        runtime.status(settings.engine_port)
    };
    if !status.running {
        let _ = app.emit("chat:error", "EIE is not running");
        return Err("EIE is not running. Complete setup and start the engine first.".to_string());
    }

    eie::send_chat_request(&app, &status.endpoint, &request)
        .await
        .map_err(|error| {
            let message = error.to_string();
            let _ = app.emit("chat:error", &message);
            message
        })
}

#[tauri::command]
pub fn settings_get(app: AppHandle) -> Result<HeliosSettings, String> {
    settings_for_app(&app)
}

#[tauri::command]
pub fn settings_update(app: AppHandle, settings: HeliosSettings) -> Result<HeliosSettings, String> {
    let paths = AppPaths::resolve(&app).map_err(to_string)?;
    paths.ensure().map_err(to_string)?;
    save_settings(&paths.settings, &settings).map_err(to_string)?;
    Ok(settings)
}

fn settings_for_app(app: &AppHandle) -> Result<HeliosSettings, String> {
    let paths = AppPaths::resolve(app).map_err(to_string)?;
    paths.ensure().map_err(to_string)?;
    db::migrate(&paths.database).map_err(to_string)?;
    load_settings(&paths.settings).map_err(to_string)
}

fn default_model_path(settings: &HeliosSettings, paths: &AppPaths) -> Option<PathBuf> {
    let catalog = load_builtin_catalog().ok()?;
    let by_id = catalog_by_id(&catalog);
    let model_id = settings.default_model_id.as_ref()?;
    by_id.get(model_id).map(|model| paths.models.join(&model.hf_file))
}

fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn vendor_eie_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("vendor").join("eie")
}

fn run_logged(program: &str, args: &[String], log_path: &std::path::Path) -> anyhow::Result<()> {
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let output = Command::new(program).args(args).output()?;
    let mut log = String::new();
    log.push_str(&format!("$ {} {}\n", program, args.join(" ")));
    log.push_str(&String::from_utf8_lossy(&output.stdout));
    log.push_str(&String::from_utf8_lossy(&output.stderr));
    log.push('\n');
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?
        .write_all(log.as_bytes())?;

    if !output.status.success() {
        anyhow::bail!("{} failed; see {}", program, log_path.display());
    }
    Ok(())
}

fn find_eie_binary(build_dir: &std::path::Path) -> Option<PathBuf> {
    let candidates = [
        build_dir.join("Release").join("eie-server.exe"),
        build_dir.join("Debug").join("eie-server.exe"),
        build_dir.join("eie-server.exe"),
        build_dir.join("eie-server"),
        build_dir.join("bin").join("Release").join("eie-server.exe"),
        build_dir.join("bin").join("eie-server.exe"),
    ];
    candidates.into_iter().find(|path| path.exists())
}
