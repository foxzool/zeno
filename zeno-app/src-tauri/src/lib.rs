pub mod commands;
pub mod models;
pub mod services;

use std::sync::Mutex;
use services::LinkIndex;
use models::tag::TagHierarchy;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(LinkIndex::new()))
        .manage(Mutex::new(TagHierarchy::new()))
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
            // 新增的链接相关命令
            commands::parse_links,
            commands::extract_link_context,
            commands::replace_link,
            commands::replace_multiple_links,
            commands::register_note_in_index,
            commands::update_note_links,
            commands::get_backlinks,
            commands::get_outgoing_links,
            commands::find_similar_notes,
            commands::get_broken_links,
            commands::get_orphaned_notes,
            commands::get_link_statistics,
            commands::rebuild_link_index,
            commands::preview_link_parsing,
            commands::validate_link_targets,
            // 新增的标签相关命令
            commands::parse_and_add_tags,
            commands::get_all_tags,
            commands::get_root_tags,
            commands::get_tag_children,
            commands::get_tag_ancestors,
            commands::get_tag_descendants,
            commands::search_tags,
            commands::get_popular_tags,
            commands::suggest_related_tags,
            commands::update_tag_usage,
            commands::get_tag_info,
            commands::get_tag_statistics,
            commands::cleanup_unused_tags,
            commands::rebuild_tag_hierarchy,
            commands::suggest_tags_for_content,
            commands::extract_tags_from_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
