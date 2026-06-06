use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BuildBackend {
    Cuda,
    Cpu,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolStatus {
    pub name: String,
    pub present: bool,
    pub path: Option<String>,
    pub message: String,
    pub install_url: String,
}

impl ToolStatus {
    pub fn present(name: impl Into<String>, path: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name,
            present: true,
            path: Some(path.into()),
            message: "Ready".to_string(),
            install_url: String::new(),
        }
    }

    pub fn missing(name: impl Into<String>, message: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            install_url: install_url_for(&name).to_string(),
            name,
            present: false,
            path: None,
            message: message.into(),
        }
    }
}

pub fn detect_prereq_from_path(name: &str, explicit_path: Option<&Path>) -> ToolStatus {
    if let Some(path) = explicit_path {
        if path.exists() {
            return ToolStatus::present(name, path.display().to_string());
        }
    }

    let output = if cfg!(windows) {
        Command::new("where.exe").arg(name).output()
    } else {
        Command::new("which").arg(name).output()
    };

    match output {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or(name)
                .trim()
                .to_string();
            ToolStatus::present(name, path)
        }
        _ => ToolStatus::missing(name, format!("Install {} before building EIE.", name)),
    }
}

pub fn check_prereqs() -> Vec<ToolStatus> {
    vec![
        detect_prereq_from_path("git", None),
        detect_prereq_from_path("cmake", None),
        detect_prereq_from_path("cl", None),
        detect_prereq_from_path("nvcc", cuda_path_hint().as_deref()),
    ]
}

pub fn choose_build_backend(tools: &[ToolStatus]) -> BuildBackend {
    let has = |name: &str| tools.iter().any(|tool| tool.name == name && tool.present);
    if !(has("git") && has("cmake") && has("cl")) {
        return BuildBackend::Blocked;
    }
    if has("nvcc") {
        BuildBackend::Cuda
    } else {
        BuildBackend::Cpu
    }
}

pub fn install_url_for(name: &str) -> &'static str {
    match name {
        "git" => "https://git-scm.com/download/win",
        "cmake" => "https://cmake.org/download/",
        "cl" => "https://visualstudio.microsoft.com/downloads/",
        "nvcc" => "https://developer.nvidia.com/cuda-downloads",
        _ => "https://github.com/KB01111/EIE",
    }
}

fn cuda_path_hint() -> Option<PathBuf> {
    std::env::var_os("CUDA_PATH").map(|path| PathBuf::from(path).join("bin").join("nvcc.exe"))
}
