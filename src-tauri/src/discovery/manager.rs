use crate::discovery::llmfit::{
    build_llmfit_models_url, llmfit_health_url, llmfit_system_url, parse_fit_models,
};
use crate::discovery::types::{FitModel, FitModelQuery, LlmfitRuntimeState, LlmfitStatus};
use crate::eie::config::validate_settings;
use crate::eie::types::{EieError, EieResult, EieSettings};
use serde_json::Value;
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct LlmfitManager {
    inner: Mutex<LlmfitManagerInner>,
}

struct LlmfitManagerInner {
    status: LlmfitStatus,
    child: Option<Child>,
}

impl Default for LlmfitManager {
    fn default() -> Self {
        Self {
            inner: Mutex::new(LlmfitManagerInner {
                status: status_for(8787, LlmfitRuntimeState::Stopped, None, None),
                child: None,
            }),
        }
    }
}

impl LlmfitManager {
    pub fn status(&self) -> LlmfitStatus {
        self.inner
            .lock()
            .expect("llmfit manager poisoned")
            .status
            .clone()
    }

    pub fn start(&self, settings: &EieSettings, app: Option<AppHandle>) -> EieResult<LlmfitStatus> {
        validate_llmfit_start_settings(settings)?;

        {
            let mut inner = self.inner.lock().expect("llmfit manager poisoned");
            if !matches!(
                inner.status.state,
                LlmfitRuntimeState::Stopped | LlmfitRuntimeState::Failed
            ) {
                return Err(EieError::new(
                    "llmfit_already_running",
                    "llmfit is already starting or running.",
                ));
            }

            inner.status = status_for(
                settings.llmfit_port,
                LlmfitRuntimeState::Starting,
                None,
                None,
            );
        }
        self.emit_status(app.as_ref());

        let binary_path = settings
            .llmfit_binary_path
            .as_ref()
            .expect("validated llmfit binary path");
        let port = settings.llmfit_port.to_string();
        let mut child = match Command::new(binary_path)
            .args(["serve", "--host", "127.0.0.1", "--port", &port])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                self.set_failed(
                    settings.llmfit_port,
                    format!("Could not start llmfit: {error}"),
                    app.as_ref(),
                );
                return Err(EieError::new(
                    "llmfit_spawn_failed",
                    format!("Could not start llmfit: {error}"),
                ));
            }
        };

        if let Some(stdout) = child.stdout.take() {
            capture_output("stdout", stdout);
        }

        if let Some(stderr) = child.stderr.take() {
            capture_output("stderr", stderr);
        }

        let pid = child.id();
        {
            let mut inner = self.inner.lock().expect("llmfit manager poisoned");
            inner.child = Some(child);
            inner.status.pid = Some(pid);
        }

        let state = if wait_for_health(settings.llmfit_port, 20, Duration::from_millis(250)) {
            LlmfitRuntimeState::Ready
        } else {
            LlmfitRuntimeState::Unhealthy
        };
        Ok(self.set_state(settings.llmfit_port, state, app.as_ref()))
    }

    pub fn stop(&self, app: Option<AppHandle>) -> EieResult<LlmfitStatus> {
        let (child, port) = {
            let mut inner = self.inner.lock().expect("llmfit manager poisoned");
            let port = status_port(&inner.status);
            (inner.child.take(), port)
        };

        if let Some(mut child) = child {
            let _ = child.kill();
            let _ = child.wait();
        }

        Ok(self.set_state(port, LlmfitRuntimeState::Stopped, app.as_ref()))
    }

    pub fn restart(
        &self,
        settings: &EieSettings,
        app: Option<AppHandle>,
    ) -> EieResult<LlmfitStatus> {
        self.stop(app.clone())?;
        self.start(settings, app)
    }

    pub fn system(&self, settings: &EieSettings) -> EieResult<Value> {
        request_json(&llmfit_system_url(settings.llmfit_port))
    }

    pub fn list_fit_models(
        &self,
        settings: &EieSettings,
        query: FitModelQuery,
    ) -> EieResult<Vec<FitModel>> {
        let value = request_json(&build_llmfit_models_url(settings.llmfit_port, &query))?;
        parse_fit_models(value)
    }

    fn set_failed(&self, port: u16, message: String, app: Option<&AppHandle>) -> LlmfitStatus {
        {
            let mut inner = self.inner.lock().expect("llmfit manager poisoned");
            inner.status = status_for(port, LlmfitRuntimeState::Failed, None, Some(message));
        }
        self.emit_status(app)
    }

    fn set_state(
        &self,
        port: u16,
        state: LlmfitRuntimeState,
        app: Option<&AppHandle>,
    ) -> LlmfitStatus {
        {
            let mut inner = self.inner.lock().expect("llmfit manager poisoned");
            let pid = inner.status.pid;
            inner.status = status_for(port, state, pid, None);
        }
        self.emit_status(app)
    }

    fn emit_status(&self, app: Option<&AppHandle>) -> LlmfitStatus {
        let status = self.status();
        if let Some(app) = app {
            let _ = app.emit("llmfit://status-changed", &status);
        }
        status
    }
}

fn validate_llmfit_start_settings(settings: &EieSettings) -> EieResult<()> {
    validate_settings(settings)?;

    let binary_path = settings.llmfit_binary_path.as_ref().ok_or_else(|| {
        EieError::new(
            "missing_llmfit_binary_path",
            "Choose a Windows llmfit .exe before starting discovery.",
        )
    })?;

    if !binary_path.is_file() {
        return Err(EieError::new(
            "missing_llmfit_binary",
            format!("llmfit binary was not found at {}.", binary_path.display()),
        ));
    }

    Ok(())
}

fn wait_for_health(port: u16, attempts: u32, delay: Duration) -> bool {
    for _ in 0..attempts {
        if request_json(&llmfit_health_url(port)).is_ok() {
            return true;
        }
        thread::sleep(delay);
    }

    false
}

fn request_json(url: &str) -> EieResult<Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|error| EieError::new("llmfit_client_failed", error.to_string()))?;

    let response = client
        .get(url)
        .send()
        .map_err(|error| EieError::new("llmfit_request_failed", error.to_string()))?;

    if !response.status().is_success() {
        return Err(EieError::new(
            "llmfit_request_failed",
            format!("llmfit returned HTTP {}.", response.status()),
        ));
    }

    response
        .json::<Value>()
        .map_err(|error| EieError::new("llmfit_response_failed", error.to_string()))
}

fn capture_output<R>(stream: &'static str, reader: R)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(Result::ok) {
            log::info!("llmfit {stream}: {line}");
        }
    });
}

fn status_for(
    port: u16,
    state: LlmfitRuntimeState,
    pid: Option<u32>,
    last_error: Option<String>,
) -> LlmfitStatus {
    LlmfitStatus {
        state,
        pid,
        base_url: format!("http://127.0.0.1:{port}"),
        last_error,
    }
}

fn status_port(status: &LlmfitStatus) -> u16 {
    status
        .base_url
        .rsplit(':')
        .next()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(8787)
}
