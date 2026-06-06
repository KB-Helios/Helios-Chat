use helios_chat_lib::catalog::{catalog_by_id, load_builtin_catalog, recommended_model};

#[test]
fn builtin_catalog_has_balanced_recommendation() {
    let catalog = load_builtin_catalog().expect("catalog should parse");
    let recommended = recommended_model(&catalog).expect("recommended model");

    assert_eq!(recommended.id, "qwen3-4b-q4-k-m");
    assert_eq!(recommended.hf_repo, "ggml-org/Qwen3-4B-GGUF");
}

#[test]
fn catalog_can_be_indexed_by_id() {
    let catalog = load_builtin_catalog().expect("catalog should parse");
    let by_id = catalog_by_id(&catalog);

    assert!(by_id.contains_key("qwen3-4b-q4-k-m"));
}
