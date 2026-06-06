use helios_chat_lib::eie::{render_eie_config, EieConfigInput};
use std::path::PathBuf;

#[test]
fn renders_eie_config_for_default_model_and_smart_warm_loading() {
    let yaml = render_eie_config(&EieConfigInput {
        host: "127.0.0.1".to_string(),
        port: 8090,
        model_dir: PathBuf::from("C:/Helios/models"),
        models: vec![
            (
                "qwen3-4b-q4-k-m".to_string(),
                PathBuf::from("C:/Helios/models/Qwen3-4B-Q4_K_M.gguf"),
            ),
            (
                "qwen3-8b-q4-k-m".to_string(),
                PathBuf::from("C:/Helios/models/Qwen3_8B.Q4_K_M.gguf"),
            ),
        ],
        default_model_alias: Some("qwen3-4b-q4-k-m".to_string()),
        n_ctx: 4096,
        type_k: "turbo3".to_string(),
        type_v: "turbo3".to_string(),
        n_gpu_layers: 99,
    });

    assert!(yaml.contains("strategy: generic"));
    assert!(yaml.contains("model_dir: \"C:/Helios/models\""));
    assert!(yaml.contains("\"qwen3-4b-q4-k-m\": \"C:/Helios/models/Qwen3-4B-Q4_K_M.gguf\""));
    assert!(yaml.contains("\"qwen3-8b-q4-k-m\": \"C:/Helios/models/Qwen3_8B.Q4_K_M.gguf\""));
    assert!(yaml.contains("type_k: turbo3"));
}
