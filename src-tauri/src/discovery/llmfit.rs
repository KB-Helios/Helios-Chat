use serde_json::Value;

use crate::eie::types::{EieError, EieResult};

pub fn build_llmfit_models_url(port: u16, query: &super::types::FitModelQuery) -> String {
    let mut url = format!(
        "http://127.0.0.1:{}/api/v1/models?runtime=llamacpp&include_too_tight={}&limit={}&sort={}",
        port, query.include_too_tight, query.limit, query.sort
    );

    if let Some(search) = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|search| !search.is_empty())
    {
        url.push_str("&search=");
        url.push_str(&percent_encode_query(search));
    }

    if query.fit != "runnable" && query.fit != "all" {
        url.push_str("&min_fit=");
        url.push_str(&percent_encode_query(&query.fit));
    }

    url
}

pub fn parse_fit_models(value: Value) -> EieResult<Vec<super::types::FitModel>> {
    if let Some(items) = value.as_array() {
        return Ok(items
            .iter()
            .filter_map(super::types::FitModel::from_value)
            .collect());
    }

    if let Some(items) = value.get("models").and_then(|models| models.as_array()) {
        return Ok(items
            .iter()
            .filter_map(super::types::FitModel::from_value)
            .collect());
    }

    Err(EieError::new(
        "invalid_llmfit_response",
        "llmfit returned an unknown model list shape.",
    ))
}

pub fn llmfit_health_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}/health")
}

pub fn llmfit_system_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}/api/v1/system")
}

fn percent_encode_query(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            b' ' => vec!['%', '2', '0'],
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::types::FitModelQuery;

    #[test]
    fn model_query_targets_local_llamacpp_runtime() {
        let query = FitModelQuery {
            search: Some("qwen".to_string()),
            fit: "good".to_string(),
            include_too_tight: true,
            limit: 25,
            sort: "score".to_string(),
        };

        let url = build_llmfit_models_url(8787, &query);

        assert!(url.starts_with("http://127.0.0.1:8787/api/v1/models?"));
        assert!(url.contains("runtime=llamacpp"));
        assert!(url.contains("include_too_tight=true"));
        assert!(url.contains("limit=25"));
        assert!(url.contains("sort=score"));
    }

    #[test]
    fn parser_keeps_fit_fields_from_array_response() {
        let value = serde_json::json!([
            {
                "name": "Qwen 2.5 7B",
                "provider": "Qwen",
                "fit_level": "good",
                "fit_label": "Good",
                "score": 0.82,
                "estimated_tps": 44.5,
                "best_quant": "Q4_K_M",
                "memory_required_gb": 5.2,
                "gguf_sources": ["Qwen/Qwen2.5-7B-GGUF"]
            }
        ]);

        let models = parse_fit_models(value).unwrap();

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "Qwen 2.5 7B");
        assert_eq!(models[0].fit_level.as_deref(), Some("good"));
        assert_eq!(models[0].gguf_sources, vec!["Qwen/Qwen2.5-7B-GGUF"]);
    }
}
