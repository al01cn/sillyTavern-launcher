use std::fs;
use std::path::PathBuf;

use sys_locale::get_locale;
use tauri::{AppHandle, Manager, PhysicalPosition, Position, WindowEvent};

use crate::types::{AppConfig, Lang};
use crate::utils::get_config_path;

// ─────────────────────────────────────────────
// Lang 实现
// ─────────────────────────────────────────────

impl Lang {
    pub fn from_str(s: &str) -> Self {
        match s {
            "en-US" | "en" => Lang::EnUs,
            "zh-CN" | "zh" => Lang::ZhCn,
            "auto" => {
                let locale = get_locale().unwrap_or_else(|| "zh-CN".to_string());
                if locale.to_lowercase().starts_with("en") {
                    Lang::EnUs
                } else {
                    Lang::ZhCn
                }
            }
            _ => Lang::ZhCn,
        }
    }
}

pub fn get_current_lang(app: &AppHandle) -> Lang {
    let config = read_app_config_from_disk(app);
    Lang::from_str(&config.lang)
}

// ─────────────────────────────────────────────
// 配置读写
// ─────────────────────────────────────────────

pub fn read_app_config_from_disk(app: &AppHandle) -> AppConfig {
    let config_path = get_config_path(app);
    if !config_path.exists() {
        return AppConfig::default();
    }
    let content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(_) => return AppConfig::default(),
    };
    serde_json::from_str::<AppConfig>(&content).unwrap_or_default()
}

pub fn write_app_config_to_disk(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path(app);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&config_path, content).map_err(|e| format!("Failed to write config file: {}", e))?;
    Ok(())
}

// ─────────────────────────────────────────────
// 窗口位置
// ─────────────────────────────────────────────

pub fn apply_saved_window_position(app: &AppHandle) {
    let config = read_app_config_from_disk(app);
    if !config.remember_window_position {
        return;
    }
    let Some(position) = config.window_position else {
        return;
    };
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let _ = window.set_position(Position::Physical(PhysicalPosition::new(
        position.x, position.y,
    )));
}

pub fn setup_window_position_tracking(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let app_handle = app.clone();
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { .. } = event {
            let mut config = read_app_config_from_disk(&app_handle);
            if !config.remember_window_position {
                return;
            }
            if let Ok(position) = window_clone.outer_position() {
                config.window_position = Some(crate::types::WindowPosition {
                    x: position.x,
                    y: position.y,
                });
                let _ = write_app_config_to_disk(&app_handle, &config);
            }
        }
    });
}

// ─────────────────────────────────────────────
// Tauri commands
// ─────────────────────────────────────────────

#[tauri::command]
pub async fn get_app_config(app: AppHandle) -> Result<AppConfig, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || Ok(read_app_config_from_disk(&app_clone)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn save_app_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || write_app_config_to_disk(&app_clone, &config))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub fn open_directory(
    app: AppHandle,
    dir_type: String,
    custom_path: Option<String>,
) -> Result<(), String> {
    tracing::info!(
        "打开目录，类型: {}, 自定义路径: {:?}",
        dir_type,
        custom_path
    );
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();

    let target_dir = match dir_type.as_str() {
        "root" => data_dir,
        "logs" => data_dir.join("logs"),
        "tavern" => data_dir.join("sillytavern"),
        "data" => data_dir.join("st_data"),
        "node" => {
            if let Some(path) = custom_path {
                let path_buf = PathBuf::from(path);
                if path_buf.is_file() {
                    path_buf
                        .parent()
                        .unwrap_or(&data_dir.join("node"))
                        .to_path_buf()
                } else {
                    path_buf
                }
            } else {
                data_dir.join("node")
            }
        }
        _ => return Err(format!("Unknown directory type: {}", dir_type)),
    };

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(target_dir);
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        cmd.stdin(std::process::Stdio::null());
        cmd.spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(target_dir)
            .stdin(std::process::Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(target_dir)
            .stdin(std::process::Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn fetch_github_proxies() -> Result<Vec<crate::types::ProxyItem>, String> {
    let client = reqwest::Client::builder()
        .user_agent("sillyTavern-launcher")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response: crate::types::ProxyResponse = client
        .get("https://api.akams.cn/github")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if response.code == 200 {
        let mut proxies = response.data;

        // 添加 ghfast.top 到列表并测试延迟
        let ghfast_url = "https://ghfast.top/";
        if !proxies.iter().any(|p| p.url == ghfast_url) {
            let test_url = format!("{}https://github.com", ghfast_url);
            let start = std::time::Instant::now();
            let latency = match client.head(&test_url).send().await {
                Ok(_) => start.elapsed().as_millis() as u32,
                Err(_) => 9999,
            };

            proxies.insert(
                0,
                crate::types::ProxyItem {
                    url: ghfast_url.to_string(),
                    server: "ghfast.top".to_string(),
                    ip: "".to_string(),
                    location: "Default".to_string(),
                    latency,
                    speed: 0.0,
                    tag: "推荐".to_string(),
                },
            );
        }

        Ok(proxies)
    } else {
        Err(format!("API Error: {}", response.msg))
    }
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
