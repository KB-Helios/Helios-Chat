use helios_chat_lib::db::{delete_preset, list_presets, migrate, save_preset, Preset};

fn make_preset(id: &str, name: &str, provider_id: &str, model: &str) -> Preset {
    Preset {
        id: id.to_string(),
        name: name.to_string(),
        provider_id: provider_id.to_string(),
        model: model.to_string(),
        system_prompt: "You are a helpful assistant.".to_string(),
        temperature: 0.7,
        top_p: 0.9,
        max_tokens: 1024,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

#[test]
fn save_preset_with_empty_id_generates_uuid() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let preset = make_preset("", "My preset", "eie-local", "qwen3-4b-q4-k-m");
    let saved = save_preset(&db_path, &preset).expect("save preset");

    assert!(!saved.id.is_empty(), "id should be generated when empty");
    assert_eq!(saved.name, "My preset");
    assert_eq!(saved.provider_id, "eie-local");
    assert_eq!(saved.model, "qwen3-4b-q4-k-m");
}

#[test]
fn save_preset_with_whitespace_id_generates_uuid() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let preset = make_preset("   ", "Whitespace id preset", "openai", "gpt-4.1");
    let saved = save_preset(&db_path, &preset).expect("save preset");

    // The whitespace id is trimmed to empty, triggering UUID generation
    assert!(!saved.id.trim().is_empty(), "id should be generated when whitespace only");
}

#[test]
fn save_preset_with_explicit_id_preserves_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let preset = make_preset("my-explicit-id", "Named preset", "anthropic", "claude-4-sonnet");
    let saved = save_preset(&db_path, &preset).expect("save preset");

    assert_eq!(saved.id, "my-explicit-id");
}

#[test]
fn list_presets_returns_presets_sorted_alphabetically_by_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    save_preset(&db_path, &make_preset("id-z", "Zephyr config", "eie-local", "qwen3-4b-q4-k-m"))
        .expect("save z");
    save_preset(&db_path, &make_preset("id-a", "Alpha config", "openai", "gpt-4.1"))
        .expect("save a");
    save_preset(&db_path, &make_preset("id-m", "Midpoint config", "anthropic", "claude-4-sonnet"))
        .expect("save m");

    let presets = list_presets(&db_path).expect("list presets");
    assert_eq!(presets.len(), 3);
    assert_eq!(presets[0].name, "Alpha config");
    assert_eq!(presets[1].name, "Midpoint config");
    assert_eq!(presets[2].name, "Zephyr config");
}

#[test]
fn save_preset_upserts_existing_preset() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    // Insert original
    let original = make_preset("upsert-id", "Original name", "eie-local", "qwen3-4b-q4-k-m");
    let saved = save_preset(&db_path, &original).expect("save original");
    let created_at = saved.created_at.clone();

    // Update via upsert
    let updated_preset = Preset {
        id: "upsert-id".to_string(),
        name: "Updated name".to_string(),
        provider_id: "openai".to_string(),
        model: "gpt-4.1".to_string(),
        system_prompt: "Updated prompt.".to_string(),
        temperature: 1.0,
        top_p: 0.95,
        max_tokens: 2048,
        created_at: created_at.clone(),
        updated_at: String::new(),
    };
    let updated = save_preset(&db_path, &updated_preset).expect("upsert preset");

    assert_eq!(updated.id, "upsert-id");
    assert_eq!(updated.name, "Updated name");
    assert_eq!(updated.provider_id, "openai");
    assert_eq!(updated.model, "gpt-4.1");
    assert_eq!(updated.temperature, 1.0);
    assert_eq!(updated.max_tokens, 2048);

    // Only one preset should exist
    let all = list_presets(&db_path).expect("list");
    assert_eq!(all.len(), 1);
}

#[test]
fn save_preset_preserves_created_at_on_update() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let original = make_preset("ts-id", "Timestamp test", "eie-local", "qwen3-4b-q4-k-m");
    let saved = save_preset(&db_path, &original).expect("save original");
    let original_created_at = saved.created_at.clone();

    // Upsert with explicit created_at matching original
    let update = Preset {
        id: "ts-id".to_string(),
        name: "Timestamp test".to_string(),
        provider_id: "eie-local".to_string(),
        model: "qwen3-4b-q4-k-m".to_string(),
        system_prompt: "New prompt".to_string(),
        temperature: 0.5,
        top_p: 0.8,
        max_tokens: 512,
        created_at: original_created_at.clone(),
        updated_at: String::new(),
    };
    let updated = save_preset(&db_path, &update).expect("upsert");

    // created_at should be preserved from original insert
    assert_eq!(updated.created_at, original_created_at);
}

#[test]
fn delete_preset_removes_it_from_list() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let preset = make_preset("del-id", "To be deleted", "eie-local", "qwen3-4b-q4-k-m");
    save_preset(&db_path, &preset).expect("save");

    delete_preset(&db_path, "del-id").expect("delete");

    let remaining = list_presets(&db_path).expect("list");
    assert!(remaining.is_empty(), "preset should be removed after delete");
}

#[test]
fn delete_nonexistent_preset_does_not_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    // Deleting an id that was never inserted should succeed without error
    delete_preset(&db_path, "does-not-exist").expect("delete nonexistent preset");
}

#[test]
fn list_presets_returns_empty_when_no_presets_exist() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let presets = list_presets(&db_path).expect("list");
    assert!(presets.is_empty());
}

#[test]
fn preset_fields_are_stored_and_retrieved_correctly() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("helios.sqlite3");
    migrate(&db_path).expect("migrate");

    let preset = Preset {
        id: "field-test-id".to_string(),
        name: "Field test".to_string(),
        provider_id: "google".to_string(),
        model: "gemini-2.5-pro".to_string(),
        system_prompt: "Be concise.".to_string(),
        temperature: 0.3,
        top_p: 0.85,
        max_tokens: 512,
        created_at: String::new(),
        updated_at: String::new(),
    };
    let saved = save_preset(&db_path, &preset).expect("save");

    assert_eq!(saved.name, "Field test");
    assert_eq!(saved.provider_id, "google");
    assert_eq!(saved.model, "gemini-2.5-pro");
    assert_eq!(saved.system_prompt, "Be concise.");
    assert!((saved.temperature - 0.3_f32).abs() < 1e-6);
    assert!((saved.top_p - 0.85_f32).abs() < 1e-6);
    assert_eq!(saved.max_tokens, 512);
    assert!(!saved.created_at.is_empty(), "created_at should be set");
    assert!(!saved.updated_at.is_empty(), "updated_at should be set");
}