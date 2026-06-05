use std::path::{Path, PathBuf};

use crate::eie::types::{EieError, EieResult};

pub fn resolve_download_destination(model_dir: &Path, filename: &str) -> EieResult<PathBuf> {
    if !filename.to_lowercase().ends_with(".gguf") {
        return Err(EieError::new(
            "invalid_download_filename",
            "Only GGUF files can be downloaded.",
        ));
    }

    let path = Path::new(filename);
    if path.is_absolute()
        || filename.contains("..")
        || filename.contains('\\')
        || filename.contains('/')
        || filename.contains(':')
    {
        return Err(EieError::new(
            "invalid_download_filename",
            "Download filename must be a plain GGUF filename.",
        ));
    }

    Ok(model_dir.join(filename))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_accepts_plain_gguf_filename() {
        let destination =
            resolve_download_destination(Path::new(r"C:\Models"), "model-q4.gguf").unwrap();

        assert_eq!(destination, PathBuf::from(r"C:\Models\model-q4.gguf"));
    }

    #[test]
    fn destination_rejects_traversal() {
        let error =
            resolve_download_destination(Path::new(r"C:\Models"), r"..\model.gguf").unwrap_err();

        assert_eq!(error.code, "invalid_download_filename");
    }

    #[test]
    fn destination_rejects_non_gguf() {
        let error = resolve_download_destination(Path::new(r"C:\Models"), "README.md").unwrap_err();

        assert_eq!(error.code, "invalid_download_filename");
    }
}
