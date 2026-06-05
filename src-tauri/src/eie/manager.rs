use super::config::{default_settings, render_config, validate_settings};
use super::logs::EieLogBuffer;
use super::process::{spawn_eie, wait_for_health};
use super::types::{
    EieConfigPreview, EieError, EieLogLine, EieResult, EieRuntimeState, EieSettings, EieStatus,
};
use std::process::Child;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct EieManager {
    inner: Mutex<EieManagerInner>,
    logs: EieLogBuffer,
}

struct EieManagerInner {
    settings: EieSettings,
    status: EieStatus,
    child: Option<Child>,
}

impl EieManager {
    pub fn new(settings: EieSettings) -> Self {
        let status = status_for(&settings, EieRuntimeState::Stopped, None, None, None);

        Self {
            inner: Mutex::new(EieManagerInner {
                settings,
                status,
                child: None,
            }),
            logs: EieLogBuffer::default(),
        }
    }

    pub fn settings(&self) -> EieSettings {
        self.inner
            .lock()
            .expect("EIE manager poisoned")
            .settings
            .clone()
    }

    pub fn replace_settings(&self, settings: EieSettings) -> EieResult<EieSettings> {
        validate_settings(&settings)?;

        let mut inner = self.inner.lock().expect("EIE manager poisoned");
        inner.settings = settings.clone();
        inner.status.base_url = base_url(settings.port);
        Ok(settings)
    }

    pub fn status(&self) -> EieStatus {
        self.inner
            .lock()
            .expect("EIE manager poisoned")
            .status
            .clone()
    }

    pub fn logs(&self) -> Vec<EieLogLine> {
        self.logs.list()
    }

    pub fn clear_logs(&self) {
        self.logs.clear();
    }

    pub fn config_preview(&self, path: String) -> EieResult<EieConfigPreview> {
        let settings = self.settings();
        let yaml = render_config(&settings)?;
        Ok(EieConfigPreview { path, yaml })
    }

    pub fn start(&self, app: Option<AppHandle>) -> EieResult<EieStatus> {
        let settings = {
            let mut inner = self.inner.lock().expect("EIE manager poisoned");
            if !matches!(
                inner.status.state,
                EieRuntimeState::Stopped | EieRuntimeState::Failed
            ) {
                return Err(EieError::new(
                    "already_running",
                    "EIE is already starting or running.",
                ));
            }

            inner.status.state = EieRuntimeState::Starting;
            inner.status.last_error = None;
            inner.settings.clone()
        };

        let child = match spawn_eie(&settings, self.logs.clone(), app.clone()) {
            Ok(child) => child,
            Err(error) => {
                self.set_failed(error.message.clone(), app.as_ref());
                return Err(error);
            }
        };

        let pid = child.id();
        {
            let mut inner = self.inner.lock().expect("EIE manager poisoned");
            inner.child = Some(child);
            inner.status.pid = Some(pid);
        }

        let ready = wait_for_health(settings.port, 30, Duration::from_millis(500));
        let state = if ready {
            EieRuntimeState::Ready
        } else {
            EieRuntimeState::Unhealthy
        };

        let status = self.set_runtime_state_with_app(state, app.as_ref());
        Ok(status)
    }

    pub fn stop(&self, app: Option<AppHandle>) -> EieResult<EieStatus> {
        let child = {
            let mut inner = self.inner.lock().expect("EIE manager poisoned");
            inner.status.state = EieRuntimeState::Stopping;
            inner.child.take()
        };

        if let Some(mut child) = child {
            let _ = child.kill();
            let _ = child.wait();
        }

        Ok(self.set_runtime_state_with_app(EieRuntimeState::Stopped, app.as_ref()))
    }

    pub fn set_config_path(&self, config_path: Option<String>) {
        self.inner
            .lock()
            .expect("EIE manager poisoned")
            .status
            .config_path = config_path;
    }

    fn set_failed(&self, message: String, app: Option<&AppHandle>) -> EieStatus {
        {
            let mut inner = self.inner.lock().expect("EIE manager poisoned");
            inner.status.state = EieRuntimeState::Failed;
            inner.status.last_error = Some(message);
        }
        self.emit_status(app)
    }

    fn set_runtime_state_with_app(
        &self,
        state: EieRuntimeState,
        app: Option<&AppHandle>,
    ) -> EieStatus {
        self.set_runtime_state(state);
        self.emit_status(app)
    }

    fn set_runtime_state(&self, state: EieRuntimeState) {
        self.inner
            .lock()
            .expect("EIE manager poisoned")
            .status
            .state = state;
    }

    fn emit_status(&self, app: Option<&AppHandle>) -> EieStatus {
        let status = self.status();
        if let Some(app) = app {
            let _ = app.emit("eie://status-changed", &status);
        }
        status
    }
}

impl Default for EieManager {
    fn default() -> Self {
        Self::new(default_settings())
    }
}

fn status_for(
    settings: &EieSettings,
    state: EieRuntimeState,
    pid: Option<u32>,
    config_path: Option<String>,
    last_error: Option<String>,
) -> EieStatus {
    EieStatus {
        state,
        pid,
        base_url: base_url(settings.port),
        config_path,
        last_error,
    }
}

fn base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eie::config::default_settings;
    use crate::eie::types::EieRuntimeState;

    #[test]
    fn start_reports_missing_binary_when_settings_are_incomplete() {
        let manager = EieManager::new(default_settings());

        let error = manager.start(None).unwrap_err();
        let status = manager.status();

        assert_eq!(error.code, "missing_binary_path");
        assert_eq!(status.state, EieRuntimeState::Failed);
        assert_eq!(status.last_error.as_deref(), Some("Choose a Windows EIE .exe before starting the server."));
    }

    #[test]
    fn start_rejects_duplicate_running_processes() {
        let manager = EieManager::new(default_settings());
        manager.set_runtime_state(EieRuntimeState::Ready);

        let error = manager.start(None).unwrap_err();

        assert_eq!(error.code, "already_running");
    }
}
