pub mod commands;
pub mod models;
pub mod services;

use std::sync::{Mutex, Arc};
use services::{LinkIndex, ImportExportManager, PluginManager, PluginAPIService, PluginRuntimeManager};
use models::tag::TagHierarchy;
use models::publisher::PublishConfig;
use models::wechat::WeChatConfig;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Mutex::new(LinkIndex::new()))
        .manage(Mutex::new(TagHierarchy::new()))
        .manage(Mutex::new(PublishConfig::default()))
        .manage(Mutex::new(WeChatConfig::default()))
        .manage(Arc::new(tokio::sync::Mutex::new(ImportExportManager::new())))
        .manage(Arc::new(Mutex::new(PluginManager::default())))
        .manage(Arc::new(Mutex::new(PluginAPIService::default())))
        .manage(Arc::new(Mutex::new(PluginRuntimeManager::default())))
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
            // 发布相关命令
            commands::initialize_zola_site,
            commands::publish_notes_to_site,
            commands::get_publish_config,
            commands::save_publish_config,
            commands::create_default_zola_config,
            commands::check_zola_installation,
            commands::get_site_stats,
            commands::preview_publish_content,
            // 微信公众号相关命令
            commands::test_wechat_config,
            commands::get_wechat_config,
            commands::save_wechat_config,
            commands::create_default_wechat_config,
            commands::publish_note_to_wechat,
            commands::publish_notes_to_wechat,
            commands::preview_wechat_content,
            commands::get_wechat_stats,
            commands::refresh_wechat_token,
            commands::upload_media_to_wechat,
            commands::create_default_wechat_settings,
            commands::validate_wechat_content,
            // 导入导出相关命令
            commands::get_available_importers,
            commands::get_available_exporters,
            commands::create_default_import_config,
            commands::create_default_export_config,
            commands::preview_import,
            commands::execute_import,
            commands::preview_export,
            commands::execute_export,
            commands::validate_import_source,
            commands::validate_export_target,
            commands::get_import_conflicts,
            commands::create_import_options,
            commands::create_export_options,
            commands::get_supported_import_formats,
            commands::get_supported_export_formats,
            commands::scan_import_directory,
            commands::estimate_export_size,
            // 插件相关命令
            commands::install_plugin,
            commands::uninstall_plugin,
            commands::enable_plugin,
            commands::disable_plugin,
            commands::get_all_plugins,
            commands::get_enabled_plugins,
            commands::check_plugin_updates,
            commands::update_plugin,
            commands::get_plugin_runtime_states,
            commands::restart_plugin_runtime,
            commands::call_plugin_api,
            commands::send_message_to_plugin,
            commands::broadcast_event_to_plugins,
            commands::get_plugin_api_stats,
            commands::register_plugin_event_handler,
            commands::save_plugin_configs,
            commands::cleanup_crashed_plugin_runtimes,
            commands::get_plugin_marketplace_info,
            commands::search_plugin_marketplace,
            commands::get_plugin_details,
            commands::validate_plugin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
