use super::types::{HfGgufFile, HfModelInfo};

pub fn gguf_files_from_model_info(model: HfModelInfo) -> Vec<HfGgufFile> {
    model
        .siblings
        .into_iter()
        .filter(|sibling| sibling.rfilename.to_lowercase().ends_with(".gguf"))
        .map(|sibling| HfGgufFile {
            repo_id: model.id.clone(),
            filename: sibling.rfilename,
            size_bytes: sibling.size,
            download_url: String::new(),
        })
        .collect()
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
}
