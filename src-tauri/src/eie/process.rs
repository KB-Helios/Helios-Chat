use super::config::validate_start_settings;
use super::logs::EieLogBuffer;
use super::types::{EieError, EieResult, EieSettings};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub fn build_launch_args(settings: &EieSettings) -> EieResult<Vec<String>> {
    let model_directory = settings.model_directory.as_ref().ok_or_else(|| {
        EieError::new(
            "missing_model_directory",
            "Choose a model directory before starting the server.",
        )
    })?;

    Ok(vec![
        "--models-dir".to_string(),
        model_directory.display().to_string(),
        "-c".to_string(),
        settings.context_length.to_string(),
        "--port".to_string(),
        settings.port.to_string(),
        "-ngl".to_string(),
        settings.gpu_layers.to_string(),
    ])
}

pub fn spawn_eie(
    settings: &EieSettings,
    logs: EieLogBuffer,
    app: Option<AppHandle>,
) -> EieResult<Child> {
    validate_start_settings(settings)?;

    let binary_path = settings.binary_path.as_ref().expect("validated binary path");
    let args = build_launch_args(settings)?;
    let mut child = Command::new(binary_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .map_err(|error| {
            EieError::new(
                "spawn_failed",
                format!("Could not start EIE process: {error}"),
            )
        })?;

    if let Some(stdout) = child.stdout.take() {
        capture_output("stdout", stdout, logs.clone(), app.clone());
    }

    if let Some(stderr) = child.stderr.take() {
        capture_output("stderr", stderr, logs, app);
    }

    Ok(child)
}

pub fn wait_for_health(port: u16, attempts: u32, delay: Duration) -> bool {
    for _ in 0..attempts {
        if health_request(port) {
            return true;
        }

        thread::sleep(delay);
    }

    false
}

fn capture_output<R>(stream: &'static str, reader: R, logs: EieLogBuffer, app: Option<AppHandle>)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);

        for line in reader.lines().map_while(Result::ok) {
            let entry = logs.push(stream, line);
            if let Some(app) = &app {
                let _ = app.emit("eie://log-line", &entry);
            }
        }
    });
}

fn health_request(port: u16) -> bool {
    let address = ("127.0.0.1", port)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addresses| addresses.next());
    let Some(address) = address else {
        return false;
    };

    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(350)) else {
        return false;
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(350)));
    let request = format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");

    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }

    let mut response = String::new();
    stream.read_to_string(&mut response).is_ok() && response.starts_with("HTTP/1.1 200")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eie::config::default_settings;
    use std::path::PathBuf;

    #[test]
    fn build_launch_args_uses_eie_windows_mvp_flags() {
        let mut settings = default_settings();
        settings.model_directory = Some(PathBuf::from(r"C:\Models"));
        settings.port = 9001;
        settings.context_length = 4096;
        settings.gpu_layers = 42;

        let args = build_launch_args(&settings).unwrap();

        assert_eq!(
            args,
            vec![
                "--models-dir",
                r"C:\Models",
                "-c",
                "4096",
                "--port",
                "9001",
                "-ngl",
                "42"
            ]
        );
    }
}
