pub mod catalog;
pub mod commands;
pub mod db;
pub mod download;
pub mod eie;
pub mod knowledge;
pub mod paths;
pub mod settings;
pub mod setup;

use commands::*;
use eie::EngineRuntime;
use std::sync::Mutex;

pub struct RuntimeState {
    pub engine: Mutex<EngineRuntime>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            engine: Mutex::new(EngineRuntime::default()),
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(RuntimeState::default())
        .invoke_handler(tauri::generate_handler![
            setup_check_prereqs,
            setup_build_eie,
            engine_start,
            engine_stop,
            engine_status,
            models_catalog,
            models_download,
            models_import_local,
            models_set_default,
            models_load,
            models_unload,
            chat_send,
            settings_get,
            settings_update,
            knowledge_stacks_list,
            knowledge_stack_create,
            knowledge_stack_update,
            knowledge_stack_delete,
            knowledge_sources_list,
            knowledge_sources_add_files,
            knowledge_sources_add_folder,
            knowledge_source_remove,
            knowledge_stack_reindex,
            knowledge_search
        ])
        .run(tauri::generate_context!())
        .expect("error while running Helios Chat");
}
