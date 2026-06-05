use super::types::{HfGgufFile, HfModelInfo};
use crate::eie::types::{EieError, EieResult};

pub fn gguf_files_from_model_info(model: HfModelInfo) -> Vec<HfGgufFile> {
    model
        .siblings
        .into_iter()
        .filter(|sibling| sibling.rfilename.to_lowercase().ends_with(".gguf"))
        .map(|sibling| HfGgufFile {
            repo_id: model.id.clone(),
            download_url: build_hf_download_url(&model.id, &sibling.rfilename),
            filename: sibling.rfilename,
            size_bytes: sibling.size,
        })
        .collect()
}

pub fn build_hf_download_url(repo_id: &str, filename: &str) -> String {
    format!("https://huggingface.co/{repo_id}/resolve/main/{filename}")
}

pub fn fetch_hf_gguf_files(repo_id: &str) -> EieResult<Vec<HfGgufFile>> {
    if repo_id.trim().is_empty()
        || repo_id.contains("://")
        || repo_id.contains('\\')
        || repo_id.contains("..")
    {
        return Err(EieError::new(
            "invalid_hf_repo",
            "Hugging Face repo id is not valid.",
        ));
    }

    let url = format!("https://huggingface.co/api/models/{repo_id}");
    let client = reqwest::blocking::Client::builder()
        .user_agent("Helios-Chat/0.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| EieError::new("hf_client_failed", error.to_string()))?;

    let response = client
        .get(url)
        .send()
        .map_err(|error| EieError::new("hf_metadata_failed", error.to_string()))?;

    if !response.status().is_success() {
        return Err(EieError::new(
            "hf_metadata_failed",
            format!("Hugging Face returned HTTP {}.", response.status()),
        ));
    }

    let model = response
        .json::<HfModelInfo>()
        .map_err(|error| EieError::new("hf_metadata_parse_failed", error.to_string()))?;

    Ok(gguf_files_from_model_info(model))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_keeps_only_gguf_siblings() {
        let json = r#"{
          "id": "org/model",
          "siblings": [
            { "rfilename": "model-q4.gguf", "size": 42 },
            { "rfilename": "README.md" },
            { "rfilename": "subdir/model-q5.GGUF", "size": 99 }
          ]
        }"#;
        let model: HfModelInfo = serde_json::from_str(json).unwrap();

        let files = gguf_files_from_model_info(model);

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].filename, "model-q4.gguf");
        assert_eq!(files[1].filename, "subdir/model-q5.GGUF");
    }

    #[test]
    fn download_url_uses_hugging_face_resolve_main() {
        let url = build_hf_download_url("Qwen/Qwen2.5-7B-GGUF", "model-q4.gguf");

        assert_eq!(
            url,
            "https://huggingface.co/Qwen/Qwen2.5-7B-GGUF/resolve/main/model-q4.gguf"
        );
    }
}
