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
    
    let mut config = if !config_path.exists() {
        AppConfig::default()
    } else {
        let content = match fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(_) => return AppConfig::default(),
        };
        
        let mut json_val: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
        
        // Migration: If sillytavern.version is a string, convert it to LocalTavernItem object
        if let Some(st) = json_val.get_mut("sillytavern") {
            if let Some(version_val) = st.get_mut("version") {
                if version_val.is_string() {
                    let ver_str = version_val.as_str().unwrap().to_string();
                    let mut new_ver_obj = serde_json::Map::new();
                    new_ver_obj.insert("version".to_string(), serde_json::Value::String(ver_str));
                    new_ver_obj.insert("path".to_string(), serde_json::Value::String(String::new()));
                    
                    *version_val = serde_json::Value::Object(new_ver_obj);
                } else if let Some(ver_obj) = version_val.as_object_mut() {
                    // Cleanup: If sillytavern.version.path matches the default online path, set it to ""
                    if let (Some(path_val), Some(v_val)) = (ver_obj.get("path"), ver_obj.get("version")) {
                        if let (Some(path_str), Some(ver_str)) = (path_val.as_str(), v_val.as_str()) {
                            if !path_str.is_empty() {
                                let data_dir = config_path.parent().unwrap_or(std::path::Path::new("."));
                                let default_path = data_dir.join("sillytavern").join(ver_str);
                                
                                let is_match = if let (Ok(p1), Ok(p2)) = (std::fs::canonicalize(path_str), std::fs::canonicalize(&default_path)) {
                                    p1 == p2
                                } else {
                                    let c1: Vec<_> = std::path::Path::new(path_str).components().map(|c| c.as_os_str().to_string_lossy().to_lowercase()).collect();
                                    let c2: Vec<_> = default_path.components().map(|c| c.as_os_str().to_string_lossy().to_lowercase()).collect();
                                    c1 == c2
                                };
                                
                                if is_match {
                                    ver_obj.insert("path".to_string(), serde_json::Value::String(String::new()));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        serde_json::from_value::<AppConfig>(json_val).unwrap_or_default()
    };
    
    if !config.region_auto_configured {
        let locale = sys_locale::get_locale().unwrap_or_else(|| "".to_string()).to_lowercase();
        if locale.starts_with("zh") || locale.ends_with("cn") || locale.ends_with("hk") || locale.ends_with("mo") || locale.ends_with("tw") {
            config.github_proxy.enable = true;
            config.npm_registry = "https://registry.npmmirror.com/".to_string();
        }
        config.region_auto_configured = true;
        let _ = write_app_config_to_disk(app, &config);
    }
    
    config
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
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    if config.remember_window_position {
        if let Some(position) = config.window_position {
            let _ = window.set_position(Position::Physical(PhysicalPosition::new(
                position.x, position.y,
            )));
            return;
        }
    }

    // 默认居中
    let _ = window.center();
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
        "tavern" => {
            let cfg = read_app_config_from_disk(&app);
            if !cfg.sillytavern.version.version.is_empty() {
                if !cfg.sillytavern.version.path.is_empty() {
                    PathBuf::from(&cfg.sillytavern.version.path)
                } else {
                    data_dir.join("sillytavern").join(&cfg.sillytavern.version.version)
                }
            } else {
                data_dir.join("sillytavern")
            }
        },
        "data" => data_dir.join("st_data"),
        "node" => {
            if let Some(path) = custom_path {
                let path_buf = PathBuf::from(path);
                if path_buf.is_file() {
                    let mut p = path_buf.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
                    // 如果是在 bin 目录下，再往上一层
                    if p.file_name().map_or(false, |n| n == "bin") {
                        if let Some(parent) = p.parent() {
                            p = parent.to_path_buf();
                        }
                    }
                    p
                } else {
                    path_buf
                }
            } else {
                data_dir.join("node")
            }
        }
        "git" => {
            if let Some(path) = custom_path {
                let path_buf = PathBuf::from(path);
                if path_buf.is_file() {
                    let mut p = path_buf.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
                    // 如果在 cmd 或 bin 目录下，通常是 Git 的安装子目录，往上一层
                    if p.file_name().map_or(false, |n| n == "cmd" || n == "bin") {
                        if let Some(parent) = p.parent() {
                            p = parent.to_path_buf();
                        }
                    }
                    p
                } else {
                    path_buf
                }
            } else {
                data_dir.join("git")
            }
        }
        _ => return Err(format!("Unknown directory type: {}", dir_type)),
    };

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    }

    tracing::info!("最终打开目录: {:?}", target_dir);

    #[cfg(target_os = "windows")]
    {
        let win_path = target_dir.to_string_lossy().replace('/', "\\");
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(&win_path);
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

// 静态回退列表
const FALLBACK_PROXIES: &[&str] = &[
    "https://ghfast.top/",
    "https://ghproxy.net/",
    "https://mirror.ghproxy.com/",
    "https://gh.api.99988866.xyz/",
    "https://gh.llkk.cc/",
];

#[tauri::command]
pub async fn fetch_github_proxies() -> Result<Vec<crate::types::ProxyItem>, String> {
    let client = reqwest::Client::builder()
        .user_agent("sillyTavern-launcher")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    // 尝试从 API 获取
    let response = client
        .get("https://api.akams.cn/github")
        .send()
        .await;

    let proxies = match response {
        Ok(resp) => {
            let json: Result<crate::types::ProxyResponse, _> = resp.json().await;
            match json {
                Ok(data) if data.code == 200 => {
                    let mut proxies = data.data;

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
                    proxies
                }
                _ => use_fallback_proxies(&client).await,
            }
        }
        Err(_) => use_fallback_proxies(&client).await,
    };

    Ok(proxies)
}

async fn use_fallback_proxies(client: &reqwest::Client) -> Vec<crate::types::ProxyItem> {
    let mut proxies = Vec::new();

    for url in FALLBACK_PROXIES {
        let test_url = format!("{}https://github.com", url);
        let start = std::time::Instant::now();
        let latency = match client.head(&test_url).send().await {
            Ok(_) => start.elapsed().as_millis() as u32,
            Err(_) => 9999,
        };

        // 从 URL 中提取服务器名称
        let server = url.trim_start_matches("https://").trim_end_matches("/");

        proxies.push(crate::types::ProxyItem {
            url: url.to_string(),
            server: server.to_string(),
            ip: "".to_string(),
            location: "回退".to_string(),
            latency,
            speed: 0.0,
            tag: "备用".to_string(),
        });
    }

    proxies.sort_by(|a, b| a.latency.cmp(&b.latency));
    proxies
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn get_system_cpu_cores() -> usize {
    num_cpus::get()
}
