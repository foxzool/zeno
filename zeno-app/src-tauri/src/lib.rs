pub mod commands;
pub mod models;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_version,
            commands::get_app_info,
            commands::read_file_content,
            commands::write_file_content,
            commands::list_notes,
            commands::get_file_tree,
            commands::create_note,
            commands::create_folder,
            commands::delete_note,
            commands::delete_folder,
            commands::rename_folder,
            commands::show_in_folder,
            commands::parse_markdown,
            commands::get_config,
            commands::save_config,
            commands::select_workspace_directory,
            commands::create_workspace_directory,
            commands::validate_workspace_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
