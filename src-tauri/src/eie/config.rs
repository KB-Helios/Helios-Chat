pub use super::types::{ConfigPreset, EieBinarySource, EieError, EieResult, EieSettings};

pub fn default_settings() -> EieSettings {
    EieSettings {
        binary_source: EieBinarySource::UserPath,
        binary_path: None,
        model_directory: None,
        host: "127.0.0.1".to_string(),
        port: 8090,
        context_length: 8192,
        gpu_layers: 99,
        config_preset: ConfigPreset::Generic,
        auto_start: false,
    }
}

pub fn validate_settings(settings: &EieSettings) -> EieResult<()> {
    if settings.host != "127.0.0.1" {
        return Err(EieError::new(
            "invalid_host",
            "EIE must bind to 127.0.0.1 in the Windows MVP.",
        ));
    }

    if !(1024..=65535).contains(&settings.port) {
        return Err(EieError::new(
            "invalid_port",
            "Port must be between 1024 and 65535.",
        ));
    }

    if !(512..=262_144).contains(&settings.context_length) {
        return Err(EieError::new(
            "invalid_context_length",
            "Context length must be between 512 and 262144.",
        ));
    }

    if settings.gpu_layers > 999 {
        return Err(EieError::new(
            "invalid_gpu_layers",
            "GPU layers must be between 0 and 999.",
        ));
    }

    if let Some(binary_path) = &settings.binary_path {
        let is_exe = binary_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"));

        if !is_exe {
            return Err(EieError::new(
                "invalid_binary_extension",
                "EIE binary must be a Windows .exe file.",
            ));
        }
    }

    Ok(())
}

pub fn validate_start_settings(settings: &EieSettings) -> EieResult<()> {
    validate_settings(settings)?;

    let binary_path = settings.binary_path.as_ref().ok_or_else(|| {
        EieError::new(
            "missing_binary_path",
            "Choose a Windows EIE .exe before starting the server.",
        )
    })?;

    if !binary_path.is_file() {
        return Err(EieError::new(
            "missing_binary",
            format!("EIE binary was not found at {}.", binary_path.display()),
        ));
    }

    let model_directory = settings.model_directory.as_ref().ok_or_else(|| {
        EieError::new(
            "missing_model_directory",
            "Choose a model directory before starting the server.",
        )
    })?;

    if !model_directory.is_dir() {
        return Err(EieError::new(
            "missing_model_directory",
            format!(
                "Model directory was not found at {}.",
                model_directory.display()
            ),
        ));
    }

    Ok(())
}

pub fn render_config(settings: &EieSettings) -> EieResult<String> {
    validate_settings(settings)?;

    let model_directory = settings.model_directory.as_ref().ok_or_else(|| {
        EieError::new(
            "missing_model_directory",
            "Model directory is required to generate EIE config.",
        )
    })?;

    let strategy = match settings.config_preset {
        ConfigPreset::Generic => "generic",
        ConfigPreset::Development => "generic",
        ConfigPreset::Custom => "generic",
    };

    Ok(format!(
        "host: 127.0.0.1\nport: {}\nstrategy: {}\nmodel_dir: {}\nauto_discover: true\ntype_k: turbo3\ntype_v: turbo3\nflash_attn: true\nn_ctx: {}\nreserve_mb: 512\nlog_level: info\n",
        settings.port,
        strategy,
        model_directory.display(),
        settings.context_length
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn default_settings_are_windows_local_and_generic() {
        let settings = default_settings();

        assert_eq!(settings.binary_source, EieBinarySource::UserPath);
        assert_eq!(settings.host, "127.0.0.1");
        assert_eq!(settings.port, 8090);
        assert_eq!(settings.context_length, 8192);
        assert_eq!(settings.gpu_layers, 99);
        assert_eq!(settings.config_preset, ConfigPreset::Generic);
        assert!(!settings.auto_start);
    }

    #[test]
    fn validation_rejects_non_exe_binary_paths() {
        let mut settings = default_settings();
        settings.binary_path = Some(PathBuf::from(r"C:\Tools\eie-server.txt"));
        settings.model_directory = Some(PathBuf::from(r"C:\Models"));

        let error = validate_settings(&settings).unwrap_err();

        assert_eq!(error.code, "invalid_binary_extension");
    }

    #[test]
    fn validation_rejects_non_local_hosts() {
        let mut settings = default_settings();
        settings.host = "0.0.0.0".to_string();

        let error = validate_settings(&settings).unwrap_err();

        assert_eq!(error.code, "invalid_host");
    }

    #[test]
    fn render_config_pins_local_host_and_numeric_settings() {
        let mut settings = default_settings();
        settings.model_directory = Some(PathBuf::from(r"C:\Users\kevin\models"));
        settings.port = 9001;
        settings.context_length = 4096;
        settings.gpu_layers = 42;

        let yaml = render_config(&settings).unwrap();

        assert!(yaml.contains("host: 127.0.0.1"));
        assert!(yaml.contains("port: 9001"));
        assert!(yaml.contains("strategy: generic"));
        assert!(yaml.contains(r"model_dir: C:\Users\kevin\models"));
        assert!(yaml.contains("n_ctx: 4096"));
        assert!(!yaml.contains("0.0.0.0"));
    }
}
