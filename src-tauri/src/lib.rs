// ─────────────────────────────────────────────────────────────────────────────
// 模块声明
// ─────────────────────────────────────────────────────────────────────────────
pub mod character;
pub mod config;
pub mod extensions;
pub mod node;
pub mod sillytavern;
pub mod types;
pub mod utils;
pub mod worldinfo;
pub mod elevation;

// ─────────────────────────────────────────────────────────────────────────────
// 顶层 use
// ─────────────────────────────────────────────────────────────────────────────
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use crate::types::{InstallState, ProcessState};
use crate::config::{apply_saved_window_position, setup_window_position_tracking};
use crate::utils::{ensure_standard_layout, init_logger};

// ─────────────────────────────────────────────────────────────────────────────
// 应用入口
// ─────────────────────────────────────────────────────────────────────────────
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            app.manage(ProcessState {
                kill_tx: Arc::new(Mutex::new(None)),
            });
            app.manage(InstallState {
                cancel_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            });

            let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if path.ends_with("src-tauri") {
                path.pop();
            }

            if let Err(e) = ensure_standard_layout(&path) {
                #[cfg(target_os = "windows")]
                {
                    if e.kind() == std::io::ErrorKind::PermissionDenied && !elevation::is_elevated() {
                        tracing::warn!("检测到无法写入应用目录且未提权，尝试自动请求管理员权限...");
                        let _ = elevation::elevate_process(app.handle().clone());
                    }
                }
                return Err(Box::new(e));
            }

            // 初始化日志
            init_logger(&path.join("data"));
            tracing::info!("应用启动");

            let handle = app.handle().clone();
            apply_saved_window_position(&handle);
            setup_window_position_tracking(&handle);
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // 通用
            config::greet,
            config::get_app_config,
            config::save_app_config,
            config::get_app_version,
            config::open_directory,
            config::fetch_github_proxies,
            // Node.js / npm
            node::check_nodejs,
            node::check_npm,
            node::install_nodejs,
            // SillyTavern 版本管理
            sillytavern::fetch_sillytavern_releases,
            sillytavern::get_installed_sillytavern_versions,
            sillytavern::get_installed_versions_info,
            sillytavern::switch_sillytavern_version,
            sillytavern::install_sillytavern_version,
            sillytavern::install_sillytavern_dependencies,
            sillytavern::cancel_install,
            sillytavern::delete_sillytavern_version,
            sillytavern::check_sillytavern_empty,
            sillytavern::get_tavern_version,
            // SillyTavern 配置
            sillytavern::read_sillytavern_config,
            sillytavern::write_sillytavern_config,
            sillytavern::get_sillytavern_config_path,
            sillytavern::get_sillytavern_config_options,
            sillytavern::update_sillytavern_config_options,
            sillytavern::open_sillytavern_config_file,
            // SillyTavern 进程
            sillytavern::start_sillytavern,
            sillytavern::stop_sillytavern,
            sillytavern::check_sillytavern_status,
            // 扩展管理
            extensions::get_extensions,
            extensions::toggle_extension_enable,
            extensions::delete_extension,
            extensions::toggle_extension_auto_update,
            extensions::open_extension_folder,
            extensions::open_specific_extension_folder,
            extensions::verify_extension_zip,
            extensions::install_extension_zip,
            extensions::verify_extension_zip_from_bytes,
            extensions::install_extension_zip_from_bytes,
            // 角色卡
            character::list_character_card_pngs,
            character::read_character_card_png,
            character::delete_character_cards,
            character::import_character_card,
            character::read_local_file,
            character::import_character_card_from_bytes,
            // 世界书
            worldinfo::list_world_infos,
            worldinfo::read_world_info,
            worldinfo::delete_world_infos,
            worldinfo::import_world_info,
            worldinfo::import_world_info_from_bytes,
            // 提权支持
            elevation::is_elevated,
            elevation::elevate_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
