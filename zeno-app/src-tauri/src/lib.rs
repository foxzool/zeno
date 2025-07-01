pub mod commands;
pub mod models;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_version,
            commands::get_app_info,
            commands::read_file_content,
            commands::write_file_content,
            commands::list_notes,
            commands::parse_markdown,
            commands::get_config,
            commands::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
