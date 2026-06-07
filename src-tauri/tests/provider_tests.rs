use helios_chat_lib::providers::{
    delete_provider_key, list_providers, openai_compatible_chat_url, provider_http_error_message,
    provider_key_exists, set_provider_key,
};

#[test]
fn eie_provider_is_enabled_without_a_key() {
    let dir = tempfile::tempdir().expect("tempdir");
    let key_path = dir.path().join("provider-keys.json");

    let providers = list_providers(&key_path).expect("providers");
    let eie = providers
        .iter()
        .find(|provider| provider.id == "eie-local")
        .expect("eie provider");

    assert!(eie.enabled);
    assert!(!eie.requires_key);
    assert!(!eie.has_key);
    assert_eq!(eie.kind, "eie-local");
}

#[test]
fn cloud_provider_status_is_key_masked() {
    let dir = tempfile::tempdir().expect("tempdir");
    let key_path = dir.path().join("provider-keys.json");

    set_provider_key(&key_path, "openai", "sk-test-secret").expect("set key");
    let providers = list_providers(&key_path).expect("providers");
    let openai = providers
        .iter()
        .find(|provider| provider.id == "openai")
        .expect("openai provider");

    assert!(openai.enabled);
    assert!(openai.requires_key);
    assert!(openai.has_key);
    assert!(!format!("{:?}", openai).contains("sk-test-secret"));
    assert!(provider_key_exists(&key_path, "openai").expect("key exists"));
}

#[test]
fn deleting_a_cloud_key_disables_the_provider() {
    let dir = tempfile::tempdir().expect("tempdir");
    let key_path = dir.path().join("provider-keys.json");

    set_provider_key(&key_path, "anthropic", "sk-ant-test").expect("set key");
    delete_provider_key(&key_path, "anthropic").expect("delete key");

    let providers = list_providers(&key_path).expect("providers");
    let anthropic = providers
        .iter()
        .find(|provider| provider.id == "anthropic")
        .expect("anthropic provider");

    assert!(!anthropic.enabled);
    assert!(!anthropic.has_key);
}

#[test]
fn openai_compatible_chat_url_normalizes_v1_suffix() {
    assert_eq!(
        openai_compatible_chat_url("http://127.0.0.1:1234/v1"),
        "http://127.0.0.1:1234/v1/chat/completions"
    );
    assert_eq!(
        openai_compatible_chat_url("http://127.0.0.1:1234/v1/"),
        "http://127.0.0.1:1234/v1/chat/completions"
    );
}

#[test]
fn provider_http_error_message_keeps_response_body() {
    let message = provider_http_error_message("OpenAI", "401 Unauthorized", "{\"error\":\"invalid key\"}");

    assert!(message.contains("OpenAI API error (401 Unauthorized)"));
    assert!(message.contains("invalid key"));
}
