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

#[test]
fn set_empty_key_returns_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let key_path = dir.path().join("provider-keys.json");

    let result = set_provider_key(&key_path, "openai", "");
    assert!(result.is_err(), "empty key should return an error");
    let message = result.unwrap_err().to_string();
    assert!(message.contains("empty"), "error should mention empty key");
}

#[test]
fn set_whitespace_only_key_returns_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let key_path = dir.path().join("provider-keys.json");

    let result = set_provider_key(&key_path, "anthropic", "   \t  ");
    assert!(result.is_err(), "whitespace-only key should return an error");
}

#[test]
fn openai_compatible_chat_url_without_v1_suffix() {
    // A plain base URL without any /v1 path should get /v1/chat/completions appended
    assert_eq!(
        openai_compatible_chat_url("http://localhost:8080"),
        "http://localhost:8080/v1/chat/completions"
    );
    assert_eq!(
        openai_compatible_chat_url("http://localhost:8080/"),
        "http://localhost:8080/v1/chat/completions"
    );
}

#[test]
fn provider_http_error_message_with_empty_body() {
    let message = provider_http_error_message("Anthropic", 500, "");
    assert_eq!(message, "Anthropic API error (500)");
    assert!(!message.contains(':'), "empty body should produce no colon suffix");
}

#[test]
fn provider_http_error_message_with_whitespace_body() {
    let message = provider_http_error_message("Google", "503 Service Unavailable", "  \n  ");
    // Whitespace-only body should be treated as empty
    assert_eq!(message, "Google API error (503 Service Unavailable)");
}

#[test]
fn list_providers_returns_all_five_providers() {
    let dir = tempfile::tempdir().expect("tempdir");
    let key_path = dir.path().join("provider-keys.json");

    let providers = list_providers(&key_path).expect("providers");
    assert_eq!(providers.len(), 5, "expected exactly 5 providers");

    let ids: Vec<&str> = providers.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&"eie-local"), "missing eie-local");
    assert!(ids.contains(&"openai"), "missing openai");
    assert!(ids.contains(&"anthropic"), "missing anthropic");
    assert!(ids.contains(&"google"), "missing google");
    assert!(ids.contains(&"openai-compatible"), "missing openai-compatible");
}

#[test]
fn google_and_openai_compatible_providers_are_disabled_without_key() {
    let dir = tempfile::tempdir().expect("tempdir");
    let key_path = dir.path().join("provider-keys.json");

    let providers = list_providers(&key_path).expect("providers");

    let google = providers.iter().find(|p| p.id == "google").expect("google");
    assert!(!google.enabled);
    assert!(google.requires_key);
    assert!(!google.has_key);

    let compat = providers
        .iter()
        .find(|p| p.id == "openai-compatible")
        .expect("openai-compatible");
    assert!(!compat.enabled);
    assert!(compat.requires_key);
    assert!(!compat.has_key);
}

#[test]
fn set_and_delete_key_on_nonexistent_file_is_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let key_path = dir.path().join("provider-keys.json");

    // Deleting a key when the file doesn't exist should succeed silently
    delete_provider_key(&key_path, "openai").expect("delete on nonexistent file");

    // key_exists on nonexistent file should return false
    assert!(!provider_key_exists(&key_path, "openai").expect("key exists check"));
}

#[test]
fn key_is_trimmed_before_saving() {
    let dir = tempfile::tempdir().expect("tempdir");
    let key_path = dir.path().join("provider-keys.json");

    // Key with surrounding whitespace should be saved without the whitespace
    set_provider_key(&key_path, "openai", "  sk-trimmed-key  ").expect("set trimmed key");

    // Key must exist (not whitespace-only after trimming)
    assert!(provider_key_exists(&key_path, "openai").expect("key exists"));
}
