use super::types::{EieError, EieResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredModel {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
}

pub fn discover_gguf_models(model_directory: &Path) -> EieResult<Vec<DiscoveredModel>> {
    if !model_directory.is_dir() {
        return Err(EieError::new(
            "missing_model_directory",
            format!(
                "Model directory was not found at {}.",
                model_directory.display()
            ),
        ));
    }

    let mut models = Vec::new();

    for entry in fs::read_dir(model_directory).map_err(|error| {
        EieError::new(
            "read_model_directory_failed",
            format!("Could not read model directory: {error}"),
        )
    })? {
        let entry = entry.map_err(|error| {
            EieError::new(
                "read_model_entry_failed",
                format!("Could not read model entry: {error}"),
            )
        })?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let is_gguf = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("gguf"));

        if !is_gguf {
            continue;
        }

        let metadata = entry.metadata().map_err(|error| {
            EieError::new(
                "read_model_metadata_failed",
                format!("Could not read model metadata: {error}"),
            )
        })?;
        let name = path
            .file_stem()
            .and_then(|file_stem| file_stem.to_str())
            .unwrap_or("model")
            .to_string();

        models.push(DiscoveredModel {
            name,
            path: path.display().to_string(),
            size_bytes: metadata.len(),
        });
    }

    models.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn discover_gguf_models_returns_only_gguf_files() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("helios-gguf-test-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("model-a.gguf"), b"a").unwrap();
        fs::write(dir.join("model-b.GGUF"), b"b").unwrap();
        fs::write(dir.join("notes.txt"), b"nope").unwrap();

        let models = discover_gguf_models(&dir).unwrap();
        let names: Vec<_> = models.iter().map(|model| model.name.as_str()).collect();

        assert_eq!(models.len(), 2);
        assert!(names.contains(&"model-a"));
        assert!(names.contains(&"model-b"));

        fs::remove_dir_all(&dir).unwrap();
    }
}
