// ─────────────────────────────────────────────────────────────────────────────
// 模块声明
// ─────────────────────────────────────────────────────────────────────────────
pub mod character;
pub mod chat;
pub mod config;
pub mod elevation;
pub mod extensions;
pub mod finderst;
pub mod git;
pub mod node;
pub mod sillytavern;
pub mod types;
pub mod utils;
pub mod worldinfo;

// ─────────────────────────────────────────────────────────────────────────────
// 顶层 use
// ─────────────────────────────────────────────────────────────────────────────
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use crate::config::{apply_saved_window_position, setup_window_position_tracking};
use crate::types::{InstallState, ProcessState};
use crate::utils::{ensure_standard_layout, init_logger};

/// 在 setup 中提前构建、通过 manage 传递给 run 回调，解决生命周期问题
struct OwnedArcs {
    cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    git_child_pid: Arc<Mutex<Option<u32>>>,
}

fn resolve_app_working_dir() -> PathBuf {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("获取可执行文件路径失败: {e}，回退到当前工作目录");
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
    };

    #[cfg(target_os = "macos")]
    {
        let exe_str = exe_path.to_string_lossy();
        if exe_str.contains(".app/Contents/MacOS/") {
            if let Some(bundle_dir) = exe_path
                .parent() // -> MacOS/
                .and_then(|p| p.parent()) // -> Contents/
                .and_then(|p| p.parent()) // -> AppName.app/
                .and_then(|p| p.parent())
            {
                return bundle_dir.to_path_buf();
            }
        }
    }

    let exe_dir = exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let looks_like_target_build = {
        let components: Vec<String> = exe_dir
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
            .collect();
        components.windows(2).any(|pair| {
            pair[0] == "target" && (pair[1] == "debug" || pair[1] == "release")
        })
    };

    if cfg!(debug_assertions) || looks_like_target_build {
        let mut cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if cwd.ends_with("src-tauri") {
            cwd.pop();
        }
        cwd
    } else {
        exe_dir
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 应用入口
// ─────────────────────────────────────────────────────────────────────────────
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let git_child_pid_arc = Arc::new(Mutex::new(None::<u32>));
            let cancel_flag_arc = Arc::new(std::sync::atomic::AtomicBool::new(false));

            app.manage(ProcessState {
                kill_tx: Arc::new(Mutex::new(None)),
                child_pid: Arc::new(Mutex::new(None)),
            });
            app.manage(InstallState {
                cancel_flag: cancel_flag_arc.clone(),
                git_child_pid: git_child_pid_arc.clone(),
            });

            // 把 Arc 存到 app state 中供 run 回调捕获（用 OwnedArcs 包装）
            app.manage(OwnedArcs {
                cancel_flag: cancel_flag_arc,
                git_child_pid: git_child_pid_arc,
            });

            let path = resolve_app_working_dir();
            if let Err(e) = std::env::set_current_dir(&path) {
                eprintln!("设置工作目录失败: {e}");
            }


            if let Err(e) = ensure_standard_layout(&path) {
                #[cfg(target_os = "windows")]
                #[cfg(target_os = "windows")]
                {
                    if e.kind() == std::io::ErrorKind::PermissionDenied && !elevation::is_elevated()
                    {
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

            // 监听主窗口关闭事件：主窗口关闭时强制关掉 sillytavern-desktop 子窗口
            if let Some(main_win) = app.get_webview_window("main") {
                let handle2 = app.handle().clone();
                main_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::Destroyed = event {
                        if let Some(desktop_win) = handle2.get_webview_window("sillytavern-desktop")
                        {
                            tracing::info!("主窗口已销毁，强制关闭 sillytavern-desktop 子窗口");
                            let _ = desktop_win.close();
                        }
                    }
                });
            }

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
            config::get_system_cpu_cores,
            config::test_network_proxy,
            config::test_github_connection,
            config::test_github_multi,
            config::test_download_speed,
            config::get_system_proxy_info,
            // Node.js / npm
            node::check_nodejs,
            node::check_nodejs_both,
            node::check_npm,
            node::install_nodejs,
            // Git
            git::check_git,
            git::check_git_both,
            git::install_git,
            git::cancel_git_node_install,
            // SillyTavern 版本管理
            sillytavern::fetch_sillytavern_releases,
            sillytavern::get_installed_sillytavern_versions,
            sillytavern::get_installed_versions_info,
            sillytavern::switch_sillytavern_version,
            sillytavern::link_existing_sillytavern,
            sillytavern::install_sillytavern_version,
            sillytavern::install_sillytavern_dependencies,
            sillytavern::check_local_tavern_dependencies,
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
            // 全局配置操作（新版本）
            sillytavern::get_sillytavern_global_config_options,
            sillytavern::update_sillytavern_global_config_options,
            sillytavern::open_sillytavern_global_config_file,
            // 配置迁移
            sillytavern::list_config_migration_sources,
            sillytavern::migrate_tavern_config,
            // 资源迁移
            sillytavern::list_resource_migration_sources,
            sillytavern::scan_migration_conflicts,
            sillytavern::execute_resource_migration,
            // SillyTavern 进程
            sillytavern::start_sillytavern,
            sillytavern::stop_sillytavern,
            sillytavern::check_sillytavern_status,
            sillytavern::open_tavern_desktop_window,
            sillytavern::get_local_ip_addresses,
            sillytavern::get_public_ip_addresses,
            sillytavern::check_network_availability,
            sillytavern::repair_missing_deps,
            // 扩展管理
            extensions::get_extensions,
            extensions::toggle_extension_enable,
            extensions::delete_extension,
            extensions::toggle_extension_auto_update,
            extensions::open_extension_folder,
            extensions::install_extension_git,
            extensions::repair_extension_git,
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
            // 对话历史
            chat::list_chats,
            chat::read_chat,
            chat::delete_chats,
            // 提权支持
            elevation::is_elevated,
            elevation::elevate_process,
            // 本地酒馆扫描
            finderst::scan_local_sillytavern,
            finderst::cancel_scan_local_sillytavern,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { .. } => {
                let app = app_handle.clone();
                let _ = std::thread::spawn(move || {
                    // 关闭桌面程序模式子窗口（如果存在）
                    if let Some(desktop_win) = app.get_webview_window("sillytavern-desktop") {
                        let _ = desktop_win.close();
                    }

                    // 在进入 async 之前，从 state 里取出 Arc（同步代码，无生命周期问题）
                    let git_child_pid_arc: Arc<Mutex<Option<u32>>> = {
                        let owned = app.state::<OwnedArcs>();
                        owned
                            .cancel_flag
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                        // 将 Arc 内容通过 unsafe transmute_copy 延长生命周期 —— 实际上
                        // 我们只需要让编译器知道这个 Arc 可以独立存活，而 Arc 本身是
                        // 引用计数安全的。用更安全的方式：将内部指针重建为独立 Arc。
                        // 实际上最简单的做法是直接用 Arc<AtomicU32> 重建，但因为类型固定，
                        // 我们直接用 unsafe 重新 clone 一个独立 Arc。
                        // —— 改用更安全方式：把 raw pointer 重新包装
                        let raw = Arc::as_ptr(&owned.git_child_pid);
                        unsafe {
                            Arc::increment_strong_count(raw);
                            Arc::from_raw(raw)
                        }
                    };

                    if let Ok(rt) = tokio::runtime::Runtime::new() {
                        rt.block_on(async move {
                            // 停止本地酒馆扫描
                            let _ = finderst::cancel_scan_local_sillytavern().await;

                            // 获取 ProcessState 并停止酒馆（同时还原 git config）
                            let state = app.state::<ProcessState>();
                            let _ = crate::sillytavern::stop_sillytavern(app.clone(), state).await;

                            // kill 正在运行的 git 子进程（如 git clone / git fetch）
                            if let Some(pid) = git_child_pid_arc.lock().await.take() {
                                tracing::info!("程序退出：正在终止 git 子进程 PID={}", pid);
                                #[cfg(target_os = "windows")]
                                {
                                    let _ = std::process::Command::new("taskkill")
                                        .args(["/F", "/PID", &pid.to_string(), "/T"])
                                        .stdout(std::process::Stdio::null())
                                        .stderr(std::process::Stdio::null())
                                        .status();
                                }
                                #[cfg(not(target_os = "windows"))]
                                {
                                    let _ = std::process::Command::new("kill")
                                        .args(["-9", &pid.to_string()])
                                        .stdout(std::process::Stdio::null())
                                        .stderr(std::process::Stdio::null())
                                        .status();
                                }
                            }
                        });
                    }
                })
                .join();
            }
            _ => {}
        });
}
