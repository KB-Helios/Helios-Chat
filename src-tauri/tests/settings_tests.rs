use helios_chat_lib::settings::{load_settings, save_settings, HeliosSettings};

#[test]
fn settings_round_trip_to_json_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("helios.settings.json");
    let settings = HeliosSettings {
        default_model_id: Some("qwen3-4b-q4-k-m".to_string()),
        ..HeliosSettings::default()
    };

    save_settings(&path, &settings).expect("save settings");
    let loaded = load_settings(&path).expect("load settings");

    assert_eq!(loaded.default_model_id.as_deref(), Some("qwen3-4b-q4-k-m"));
}
