use std::fs;
use std::path::PathBuf;

use sys_locale::get_locale;
use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Position, WebviewWindow, WindowEvent,
};

use crate::types::{AppConfig, GithubProxyConfig, Lang};
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
                    new_ver_obj
                        .insert("path".to_string(), serde_json::Value::String(String::new()));

                    *version_val = serde_json::Value::Object(new_ver_obj);
                } else if let Some(ver_obj) = version_val.as_object_mut() {
                    // Cleanup: If sillytavern.version.path matches the default online path, set it to ""
                    if let (Some(path_val), Some(v_val)) =
                        (ver_obj.get("path"), ver_obj.get("version"))
                    {
                        if let (Some(path_str), Some(ver_str)) = (path_val.as_str(), v_val.as_str())
                        {
                            if !path_str.is_empty() {
                                let data_dir =
                                    config_path.parent().unwrap_or(std::path::Path::new("."));
                                let default_path = data_dir.join("sillytavern").join(ver_str);

                                let is_match = if let (Ok(p1), Ok(p2)) = (
                                    std::fs::canonicalize(path_str),
                                    std::fs::canonicalize(&default_path),
                                ) {
                                    p1 == p2
                                } else {
                                    let c1: Vec<_> = std::path::Path::new(path_str)
                                        .components()
                                        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
                                        .collect();
                                    let c2: Vec<_> = default_path
                                        .components()
                                        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
                                        .collect();
                                    c1 == c2
                                };

                                if is_match {
                                    ver_obj.insert(
                                        "path".to_string(),
                                        serde_json::Value::String(String::new()),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        serde_json::from_value::<AppConfig>(json_val).unwrap_or_default()
    };

    // 兜底：github_proxy.url 若被清空或不是合法 http/https URL，自动恢复默认值
    {
        let url = config.github_proxy.url.trim().to_string();
        let is_valid = !url.is_empty()
            && (url.starts_with("http://") || url.starts_with("https://"))
            && url.len() > 10
            && url.contains('.');
        if !is_valid {
            config.github_proxy.url = GithubProxyConfig::default().url;
        }
    }

    if !config.region_auto_configured {
        let locale = sys_locale::get_locale()
            .unwrap_or_else(|| "".to_string())
            .to_lowercase();
        if locale.starts_with("zh")
            || locale.ends_with("cn")
            || locale.ends_with("hk")
            || locale.ends_with("mo")
            || locale.ends_with("tw")
        {
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

fn clamp_window_position(
    window: &WebviewWindow,
    target: PhysicalPosition<i32>,
    saved_size: Option<PhysicalSize<u32>>,
) -> PhysicalPosition<i32> {
    let Ok(monitors) = window.available_monitors() else {
        return target;
    };

    let mut clamped = target;
    let size =
        saved_size.unwrap_or_else(|| window.outer_size().unwrap_or(PhysicalSize::new(1200, 800)));
    let width = size.width as i32;
    let height = size.height as i32;

    for monitor in monitors {
        let area = monitor.position();
        let monitor_size = monitor.size();
        let mx = area.x;
        let my = area.y;
        let mw = monitor_size.width as i32;
        let mh = monitor_size.height as i32;

        let max_x = mx + mw - width.max(0);
        let max_y = my + mh - height.max(0);

        if target.x >= mx && target.x <= max_x && target.y >= my && target.y <= max_y {
            return target;
        }

        clamped.x = clamped.x.clamp(mx, max_x);
        clamped.y = clamped.y.clamp(my, max_y);
    }

    clamped
}

pub fn apply_saved_window_position(app: &AppHandle) {
    let config = read_app_config_from_disk(app);
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    if config.remember_window_position {
        if let Some(position) = config.window_position {
            let saved_size = match (position.width, position.height) {
                (Some(w), Some(h)) if w > 0 && h > 0 => Some(PhysicalSize::new(w as u32, h as u32)),
                _ => None,
            };
            let clamped = clamp_window_position(
                &window,
                PhysicalPosition::new(position.x, position.y),
                saved_size,
            );
            let _ = window.set_position(Position::Physical(clamped));
            if let Some(size) = saved_size {
                let _ = window.set_size(size);
            }
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
                let size = window_clone
                    .outer_size()
                    .unwrap_or(PhysicalSize::new(1200, 800));
                let clamped = clamp_window_position(&window_clone, position, Some(size));
                config.window_position = Some(crate::types::WindowPosition {
                    x: clamped.x,
                    y: clamped.y,
                    width: Some(size.width as i32),
                    height: Some(size.height as i32),
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
                    data_dir
                        .join("sillytavern")
                        .join(&cfg.sillytavern.version.version)
                }
            } else {
                data_dir.join("sillytavern")
            }
        }
        "data" => {
            let cfg = read_app_config_from_disk(&app);
            if cfg.data_mode == "local" {
                if !cfg.sillytavern.version.version.is_empty() {
                    let st_dir = if !cfg.sillytavern.version.path.is_empty() {
                        PathBuf::from(&cfg.sillytavern.version.path)
                    } else {
                        data_dir
                            .join("sillytavern")
                            .join(&cfg.sillytavern.version.version)
                    };
                    st_dir.join("data")
                } else {
                    data_dir.join("st_data")
                }
            } else {
                data_dir.join("st_data")
            }
        }
        "node" => {
            if let Some(path) = custom_path {
                let path_buf = PathBuf::from(path);
                if path_buf.is_file() {
                    let mut p = path_buf
                        .parent()
                        .unwrap_or(std::path::Path::new("."))
                        .to_path_buf();
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
                    let mut p = path_buf
                        .parent()
                        .unwrap_or(std::path::Path::new("."))
                        .to_path_buf();
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
    let response = client.get("https://api.akams.cn/github").send().await;

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

// ─── Windows 系统代理读取（三级回退） ────────────────────────────────────────

/// 从各渠道收集到的代理原始值，统一解析
#[cfg(target_os = "windows")]
fn parse_proxy_values(server_raw: &str, enable_raw: &str) -> Option<(String, bool)> {
    let server = server_raw.trim().to_string();
    if server.is_empty() {
        return None;
    }
    // enable_raw 可能是 "1", "0x1", "0x0" 等
    let enabled = matches!(
        enable_raw.trim().to_lowercase().as_str(),
        "1" | "0x1" | "0x00000001"
    );
    Some((server, enabled))
}

/// 方式1: winreg 直接读注册表
#[cfg(target_os = "windows")]
fn try_read_proxy_via_winreg() -> Option<(String, bool)> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        .ok()?;

    let proxy_server: String = settings.get_value("ProxyServer").ok()?;
    let proxy_enable: u32 = settings.get_value("ProxyEnable").unwrap_or(0);

    if proxy_server.is_empty() {
        return None;
    }
    Some((proxy_server, proxy_enable != 0))
}

/// 方式2: PowerShell 查询（无需管理员）
#[cfg(target_os = "windows")]
fn try_read_proxy_via_powershell() -> Option<(String, bool)> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    // 检测 powershell.exe 是否可用（隐藏窗口）
    let ps_exe = if Command::new("powershell.exe")
        .arg("-?")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .is_ok()
    {
        "powershell.exe"
    } else if Command::new("pwsh.exe")
        .arg("-?")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .is_ok()
    {
        "pwsh.exe"
    } else {
        return None;
    };

    let script = r#"
$p = Get-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings' -ErrorAction SilentlyContinue;
if ($p) { Write-Output "ProxyServer=$($p.ProxyServer)"; Write-Output "ProxyEnable=$($p.ProxyEnable)" }
:"#;

    let output = Command::new(ps_exe)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .stdin(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut server = "";
    let mut enable = "";
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("ProxyServer=") {
            server = v;
        }
        if let Some(v) = line.strip_prefix("ProxyEnable=") {
            enable = v;
        }
    }
    parse_proxy_values(server, enable)
}

/// 方式3: reg query (CMD 兼容)
#[cfg(target_os = "windows")]
fn try_read_proxy_via_reg_query() -> Option<(String, bool)> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            "ProxyServer",
        ])
        .stdin(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let server_text = String::from_utf8_lossy(&output.stdout);
    // 格式: "    ProxyServer    REG_SZ    127.0.0.1:7890"
    let server = server_text
        .lines()
        .find(|l| l.trim_start().starts_with("ProxyServer"))
        .and_then(|l| l.split("REG_SZ").nth(1))
        .map(|s| s.trim())
        .unwrap_or("")
        .to_string();

    let output2 = Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            "/v",
            "ProxyEnable",
        ])
        .stdin(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let enable_text = String::from_utf8_lossy(&output2.stdout);
    // 格式: "    ProxyEnable    REG_DWORD    0x1"
    let enable = enable_text
        .lines()
        .find(|l| l.trim_start().starts_with("ProxyEnable"))
        .and_then(|l| l.split("REG_DWORD").nth(1))
        .map(|s| s.trim())
        .unwrap_or("0")
        .to_string();

    parse_proxy_values(&server, &enable)
}

/// 三级回退：winreg → PowerShell → reg query
/// 返回 (proxy_server, proxy_enable)，例如 ("127.0.0.1:7890", true)
#[cfg(target_os = "windows")]
pub(crate) fn read_windows_system_proxy() -> Option<(String, bool)> {
    try_read_proxy_via_winreg()
        .or_else(|| try_read_proxy_via_powershell())
        .or_else(|| try_read_proxy_via_reg_query())
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn read_windows_system_proxy() -> Option<(String, bool)> {
    None
}

/// 获取系统代理信息（供前端展示用）
/// 返回 { server: "host:port", enabled: bool }；系统无代理设置返回 null
#[tauri::command]
pub fn get_system_proxy_info() -> Option<serde_json::Value> {
    read_windows_system_proxy()
        .map(|(server, enabled)| serde_json::json!({ "server": server, "enabled": enabled }))
}

/// 测试代理连通性，返回延迟 ms；失败返回错误字符串
/// mode: "none" | "system" | "custom"
#[tauri::command]
pub async fn test_network_proxy(mode: String, host: String, port: u16) -> Result<u64, String> {
    let mut builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(8));

    match mode.as_str() {
        "custom" => {
            let proxy_url = format!("http://{}:{}", host, port);
            let proxy =
                reqwest::Proxy::all(&proxy_url).map_err(|e| format!("Invalid proxy: {}", e))?;
            builder = builder.proxy(proxy);
        }
        "system" => {
            // 读取 Windows 注册表中的系统代理
            match read_windows_system_proxy() {
                Some((server, true)) => {
                    // ProxyServer 格式可能是 "host:port" 或 "http=host:port;https=host:port;..."
                    // 提取第一个可用地址
                    let proxy_addr = if server.contains('=') {
                        // 有协议前缀，尝试提取 https 或 http 的值
                        server
                            .split(';')
                            .find_map(|part| {
                                let kv: Vec<&str> = part.splitn(2, '=').collect();
                                if kv.len() == 2 && (kv[0] == "https" || kv[0] == "http") {
                                    Some(kv[1].to_string())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| {
                                // fallback: 取第一段 '=' 后的值
                                server
                                    .split(';')
                                    .next()
                                    .and_then(|p| p.splitn(2, '=').nth(1))
                                    .unwrap_or(&server)
                                    .to_string()
                            })
                    } else {
                        server.clone()
                    };
                    let proxy_url = format!("http://{}", proxy_addr);
                    let proxy = reqwest::Proxy::all(&proxy_url)
                        .map_err(|e| format!("Invalid system proxy: {}", e))?;
                    builder = builder.proxy(proxy);
                }
                Some((_, false)) => {
                    // 系统代理已配置但未启用，走直连
                    builder = builder.no_proxy();
                }
                None => {
                    // 未检测到系统代理设置，走直连
                    builder = builder.no_proxy();
                }
            }
        }
        _ => {
            // none：禁用所有代理
            builder = builder.no_proxy();
        }
    }

    let client = builder
        .build()
        .map_err(|e| format!("Build client failed: {}", e))?;

    let start = std::time::Instant::now();
    client
        .get("https://www.google.com")
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    Ok(start.elapsed().as_millis() as u64)
}

/// 测试 GitHub 连接是否可达（考虑代理设置）
/// mode: "none" | "system" | "custom" | "proxy"
/// 当 mode 为 "proxy" 时，host 参数应传入 GitHub 加速地址
#[tauri::command]
pub async fn test_github_connection(
    app: AppHandle,
    mode: String,
    host: String,
    port: u16,
) -> Result<u64, String> {
    // Git 环境前置检查
    let git_exe = crate::git::get_git_exe(&app);
    let git_exists = if git_exe.to_string_lossy() == "git" {
        crate::git::has_system_git()
    } else {
        git_exe.exists()
    };
    if !git_exists {
        return Err("Git not found".to_string());
    }

    let mut builder = reqwest::Client::builder()
        .user_agent("SillyTavern-launcher")
        .gzip(true)
        .timeout(std::time::Duration::from_secs(10));

    match mode.as_str() {
        "proxy" => {
            // 使用 GitHub 加速地址作为代理
            let proxy_url = if host.starts_with("http://") || host.starts_with("https://") {
                host.clone()
            } else {
                format!("http://{}", host)
            };
            let proxy =
                reqwest::Proxy::all(&proxy_url).map_err(|e| format!("Invalid proxy: {}", e))?;
            builder = builder.proxy(proxy);
        }
        "custom" => {
            let proxy_url = format!("http://{}:{}", host, port);
            let proxy =
                reqwest::Proxy::all(&proxy_url).map_err(|e| format!("Invalid proxy: {}", e))?;
            builder = builder.proxy(proxy);
        }
        "system" => match read_windows_system_proxy() {
            Some((server, true)) => {
                let proxy_addr = if server.contains('=') {
                    server
                        .split(';')
                        .find_map(|part| {
                            let kv: Vec<&str> = part.splitn(2, '=').collect();
                            if kv.len() == 2 && (kv[0] == "https" || kv[0] == "http") {
                                Some(kv[1].to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| {
                            server
                                .split(';')
                                .next()
                                .and_then(|p| p.splitn(2, '=').nth(1))
                                .unwrap_or(&server)
                                .to_string()
                        })
                } else {
                    server.clone()
                };
                let proxy_url = format!("http://{}", proxy_addr);
                let proxy = reqwest::Proxy::all(&proxy_url)
                    .map_err(|e| format!("Invalid system proxy: {}", e))?;
                builder = builder.proxy(proxy);
            }
            Some((_, false)) => {
                // 系统代理已配置但未启用，走直连
                builder = builder.no_proxy();
            }
            None => {
                // 未检测到系统代理设置，走直连
                builder = builder.no_proxy();
            }
        },
        _ => {
            builder = builder.no_proxy();
        }
    }

    let client = builder
        .build()
        .map_err(|e| format!("Build client failed: {}", e))?;

    // 测试 GitHub API 连接
    let start = std::time::Instant::now();
    let response = client
        .get("https://www.github.com")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    Ok(start.elapsed().as_millis() as u64)
}

/// GitHub 多链接测试结果项
#[derive(serde::Serialize)]
pub struct GithubTestResultItem {
    pub key: String,
    pub name: String,
    pub url: String,
    pub success: bool,
    pub latency: Option<u64>,
    pub error: Option<String>,
    /// 警告消息，如果有则表示加速地址可用但无法加速特定资源
    pub warning: Option<String>,
}

/// 测试多个 GitHub 相关链接
/// mode: "none" | "system" | "custom" | "proxy" | "accelerate"
/// - "accelerate": 加速模式，URL 前面拼接加速地址，仓库用 git ls-remote 测试
/// - "proxy": 代理模式，使用 GitHub 加速地址作为代理
/// - "custom" / "system" / "none": 直连或自定义代理模式
/// include_api: 是否包含 api.github.com 测试（仅非加速模式生效）
#[tauri::command]
pub async fn test_github_multi(
    app: AppHandle,
    mode: String,
    host: String,
    port: u16,
    include_api: bool,
) -> Result<Vec<GithubTestResultItem>, String> {
    // 加速模式：URL 拼接 + git ls-remote
    if mode == "accelerate" {
        return test_github_accelerate(&app, &host).await;
    }

    let mut builder = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .gzip(true)
        .timeout(std::time::Duration::from_secs(10));

    match mode.as_str() {
        "proxy" => {
            let proxy_url = if host.starts_with("http://") || host.starts_with("https://") {
                host.clone()
            } else {
                format!("http://{}", host)
            };
            let proxy =
                reqwest::Proxy::all(&proxy_url).map_err(|e| format!("Invalid proxy: {}", e))?;
            builder = builder.proxy(proxy);
        }
        "custom" => {
            let proxy_url = format!("http://{}:{}", host, port);
            let proxy =
                reqwest::Proxy::all(&proxy_url).map_err(|e| format!("Invalid proxy: {}", e))?;
            builder = builder.proxy(proxy);
        }
        "system" => match read_windows_system_proxy() {
            Some((server, true)) => {
                let proxy_addr = if server.contains('=') {
                    server
                        .split(';')
                        .find_map(|part| {
                            let kv: Vec<&str> = part.splitn(2, '=').collect();
                            if kv.len() == 2 && (kv[0] == "https" || kv[0] == "http") {
                                Some(kv[1].to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| {
                            server
                                .split(';')
                                .next()
                                .and_then(|p| p.splitn(2, '=').nth(1))
                                .unwrap_or(&server)
                                .to_string()
                        })
                } else {
                    server.clone()
                };
                let proxy_url = format!("http://{}", proxy_addr);
                let proxy = reqwest::Proxy::all(&proxy_url)
                    .map_err(|e| format!("Invalid system proxy: {}", e))?;
                builder = builder.proxy(proxy);
            }
            Some((_, false)) => {
                // 系统代理已配置但未启用，走直连
                builder = builder.no_proxy();
            }
            None => {
                // 未检测到系统代理设置，走直连
                builder = builder.no_proxy();
            }
        },
        _ => {
            builder = builder.no_proxy();
        }
    }

    let client = builder
        .build()
        .map_err(|e| format!("Build client failed: {}", e))?;

    // 定义测试链接列表
    let mut test_urls = vec![
        (
            "raw",
            "文件访问",
            "https://raw.githubusercontent.com/SillyTavern/SillyTavern/release/start.sh",
        ),
        (
            "repo",
            "仓库访问",
            "https://github.com/SillyTavern/SillyTavern",
        ),
        ("homepage", "首页访问", "https://www.github.com"),
    ];

    if include_api {
        test_urls.push((
            "api",
            "API 访问",
            "https://api.github.com/repos/SillyTavern/SillyTavern/releases",
        ));
    }

    let mut results = Vec::new();

    // 系统代理模式下首次请求可能因代理未就绪而失败，添加重试
    let is_system_proxy = mode == "system";
    let max_retries = if is_system_proxy { 2 } else { 1 };

    for (key, name, url) in test_urls {
        let start = std::time::Instant::now();

        // 主页测试不需要 Accept header，否则会返回 406
        // API 测试使用 application/json
        let request = match key {
            "homepage" => client.get(url),
            "api" => client.get(url).header("Accept", "application/json"),
            _ => client
                .get(url)
                .header("Accept", "application/vnd.github.v3+json"),
        };

        // 带重试的请求
        let mut result = request.send().await;
        let mut retry_count = 0;

        // 如果是系统代理模式，遇到网络错误或 503 时重试
        while retry_count < max_retries {
            match &result {
                Ok(response) => {
                    if response.status().as_u16() == 503 {
                        // 503 错误，可能是代理未就绪，等待后重试
                        retry_count += 1;
                        if retry_count < max_retries {
                            tracing::warn!(
                                "直连测试 [{}] 收到 503，{}ms 后重试...",
                                name,
                                retry_count * 500
                            );
                            tokio::time::sleep(std::time::Duration::from_millis(500 * retry_count))
                                .await;
                            result = client.get(url).send().await;
                            continue;
                        }
                    }
                    break; // 其他状态码或成功则退出重试
                }
                Err(_) => {
                    // 网络错误，重试
                    retry_count += 1;
                    if retry_count < max_retries {
                        tracing::warn!(
                            "直连测试 [{}] 请求失败，{}ms 后重试...",
                            name,
                            retry_count * 500
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(500 * retry_count))
                            .await;
                        result = client.get(url).send().await;
                        continue;
                    }
                    break;
                }
            }
        }

        let latency = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) => {
                if response.status().is_success()
                    || response.status().as_u16() == 301
                    || response.status().as_u16() == 302
                {
                    results.push(GithubTestResultItem {
                        key: key.to_string(),
                        name: name.to_string(),
                        url: url.to_string(),
                        success: true,
                        latency: Some(latency),
                        error: None,
                        warning: None,
                    });
                } else {
                    let status = response.status();
                    tracing::error!("直连测试 [{}] HTTP 错误: {}", name, status);
                    results.push(GithubTestResultItem {
                        key: key.to_string(),
                        name: name.to_string(),
                        url: url.to_string(),
                        success: false,
                        latency: None,
                        error: None,
                        warning: None,
                    });
                }
            }
            Err(e) => {
                let err_msg = e.to_string();
                tracing::error!("直连测试 [{}] 失败: {}", name, err_msg);
                results.push(GithubTestResultItem {
                    key: key.to_string(),
                    name: name.to_string(),
                    url: url.to_string(),
                    success: false,
                    latency: None,
                    error: None,
                    warning: None,
                });
            }
        }
    }

    // 6. 直连下载速度测试：复用 MiniGit 流式下载逻辑
    let speed_test_url = "https://github.com/al01cn/sillyTavern-launcher/releases/download/v0.1.5/SillyTavern.Launcher.GUI_x64.app.tar.gz";

    let speed_result = run_download_speed_test(&client, speed_test_url).await;
    results.push(GithubTestResultItem {
        key: "speed".to_string(),
        name: "下载速度".to_string(),
        url: speed_test_url.to_string(),
        success: speed_result
            .as_ref()
            .map(|r| r.speed_mbps > 0.0)
            .unwrap_or(false),
        latency: speed_result.as_ref().ok().map(|r| r.download_time_ms),
        error: None,
        warning: speed_result
            .ok()
            .map(|r| format!("{:.2} MB/s", r.speed_mbps)),
    });

    Ok(results)
}

/// 判断加速测试的 HTTP 响应是否成功
/// 返回 (is_success, warning_message)
/// - 200: 完全成功，无警告
/// - 403/404/无效输入: 加速地址可用但无法加速特定资源，返回警告
/// - 其他: 完全失败
fn is_accelerate_success(status: reqwest::StatusCode, body: &str) -> (bool, Option<String>) {
    if status.is_success() {
        return (true, None);
    }
    let lower = body.to_lowercase();
    // 403、404 或包含特定错误信息的，说明加速地址能连通但无法加速目标资源
    if status.as_u16() == 403 {
        return (
            true,
            Some("加速地址可用，但该资源无法加速 (403)".to_string()),
        );
    }
    if status.as_u16() == 404 {
        return (
            true,
            Some("加速地址可用，但该资源无法加速 (404)".to_string()),
        );
    }
    if lower.contains("invalid input") || lower.contains("无效输入") {
        return (true, Some("加速地址可用，但该资源无法加速".to_string()));
    }
    (false, None)
}

/// GitHub 加速模式测试：URL 拼接 + git ls-remote
async fn test_github_accelerate(
    app: &AppHandle,
    accel_url: &str,
) -> Result<Vec<GithubTestResultItem>, String> {
    let mut results = Vec::new();

    // 检查 Git 环境：如果没有 Git，返回特殊错误提示用户安装
    let git_exe = crate::git::get_git_exe(app);
    let git_exists = if git_exe.to_string_lossy() == "git" {
        // 系统 Git：直接尝试运行 git --version 检测
        crate::git::has_system_git()
    } else {
        // 内置 Git：检查文件是否存在
        git_exe.exists()
    };
    if !git_exists {
        return Err("Git not found".to_string());
    }

    // 标准化加速地址：确保以 / 结尾
    let accel_base = if accel_url.ends_with('/') {
        accel_url.trim_end_matches('/')
    } else {
        accel_url
    };

    // 1. 文件访问测试：加速地址 + 完整URL
    let raw_url = "https://raw.githubusercontent.com/SillyTavern/SillyTavern/release/start.sh";
    let accel_raw_url = format!("{}/{}", accel_base, raw_url);

    let start = std::time::Instant::now();
    let client = reqwest::Client::builder()
        .user_agent("SillyTavern-launcher")
        .gzip(true) // 启用 gzip 解压，避免 "error decoding response body"
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Build client failed: {}", e))?;

    let result = client.get(&accel_raw_url).send().await;
    let latency = start.elapsed().as_millis() as u64;

    match result {
        Ok(response) => {
            // 先获取 status，再读取 body（text 会夺走所有权）
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let (success, warning) = is_accelerate_success(status, &body);
            if !success {
                tracing::error!("Github 加速测试 [文件访问] HTTP 错误: {}", status);
            }
            results.push(GithubTestResultItem {
                key: "raw".to_string(),
                name: "文件访问".to_string(),
                url: accel_raw_url,
                success: success || warning.is_some(), // 有警告也算成功
                latency: if success { Some(latency) } else { None },
                error: None,
                warning,
            });
        }
        Err(e) => {
            let err_msg = e.to_string();
            tracing::error!("Github 加速测试 [文件访问] 失败: {}", err_msg);
            results.push(GithubTestResultItem {
                key: "raw".to_string(),
                name: "文件访问".to_string(),
                url: accel_raw_url,
                success: false,
                latency: None,
                error: None,
                warning: None,
            });
        }
    }

    // 2. 主页访问测试
    let homepage_url = "https://www.github.com";
    let accel_homepage_url = format!("{}/{}", accel_base, homepage_url);

    let start = std::time::Instant::now();
    let result = client.get(&accel_homepage_url).send().await;
    let latency = start.elapsed().as_millis() as u64;

    match result {
        Ok(response) => {
            // 先获取 status，再读取 body（text 会夺走所有权）
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let (success, warning) = is_accelerate_success(status, &body);
            if !success {
                tracing::error!("Github 加速测试 [首页访问] HTTP 错误: {}", status);
            }
            results.push(GithubTestResultItem {
                key: "homepage".to_string(),
                name: "首页访问".to_string(),
                url: accel_homepage_url,
                success: success || warning.is_some(),
                latency: if success { Some(latency) } else { None },
                error: None,
                warning,
            });
        }
        Err(e) => {
            let err_msg = e.to_string();
            tracing::error!("Github 加速测试 [首页访问] 失败: {}", err_msg);
            results.push(GithubTestResultItem {
                key: "homepage".to_string(),
                name: "首页访问".to_string(),
                url: accel_homepage_url,
                success: false,
                latency: None,
                error: None,
                warning: None,
            });
        }
    }

    // 3. 仓库访问测试：使用 git ls-remote
    let repo_url = "https://github.com/SillyTavern/SillyTavern";
    let accel_repo_url = format!("{}/{}", accel_base, repo_url);

    let start = std::time::Instant::now();

    // 获取可用的 git 可执行文件路径
    let git_exe = crate::git::get_git_exe(app);

    // 使用 git ls-remote 测试，设置临时环境变量禁用 terminal prompt
    let mut cmd = std::process::Command::new(&git_exe);
    cmd.args(["-c", "credential.helper=", "ls-remote", &accel_repo_url])
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output = cmd.output();

    let latency = start.elapsed().as_millis() as u64;

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // 成功时应该返回类似 "hash1\tHEAD\nhash2\tref1\n" 的内容
            if stdout.contains('\t') {
                results.push(GithubTestResultItem {
                    key: "repo".to_string(),
                    name: "仓库访问".to_string(),
                    url: accel_repo_url,
                    success: true,
                    latency: Some(latency),
                    error: None,
                    warning: None,
                });
            } else {
                tracing::error!("Github 加速测试 [仓库访问] 返回格式异常");
                results.push(GithubTestResultItem {
                    key: "repo".to_string(),
                    name: "仓库访问".to_string(),
                    url: accel_repo_url,
                    success: false,
                    latency: None,
                    error: None,
                    warning: None,
                });
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::error!("Github 加速测试 [仓库访问] 失败: {}", stderr);
            results.push(GithubTestResultItem {
                key: "repo".to_string(),
                name: "仓库访问".to_string(),
                url: accel_repo_url,
                success: false,
                latency: None,
                error: Some(format!("git exited with error: {}", stderr)),
                warning: None,
            });
        }
        Err(e) => {
            let err_msg = e.to_string();
            tracing::error!("Github 加速测试 [仓库访问] 异常: {}", err_msg);
            results.push(GithubTestResultItem {
                key: "repo".to_string(),
                name: "仓库访问".to_string(),
                url: accel_repo_url,
                success: false,
                latency: None,
                error: Some(err_msg),
                warning: None,
            });
        }
    }

    // 4. API 访问测试
    let api_url = "https://api.github.com/repos/SillyTavern/SillyTavern/releases";
    let accel_api_url = format!("{}/{}", accel_base, api_url);

    let start = std::time::Instant::now();
    let result = client
        .get(&accel_api_url)
        .header("Accept", "application/json")
        .send()
        .await;
    let latency = start.elapsed().as_millis() as u64;

    match result {
        Ok(response) => {
            // 先获取 status，再读取 body（text 会夺走所有权）
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let (success, warning) = is_accelerate_success(status, &body);
            if !success {
                tracing::error!("Github 加速测试 [API 访问] HTTP 错误: {}", status);
            }
            results.push(GithubTestResultItem {
                key: "api".to_string(),
                name: "API 访问".to_string(),
                url: accel_api_url,
                success: success || warning.is_some(),
                latency: if success { Some(latency) } else { None },
                error: None,
                warning,
            });
        }
        Err(e) => {
            let err_msg = e.to_string();
            tracing::error!("Github 加速测试 [API 访问] 失败: {}", err_msg);
            results.push(GithubTestResultItem {
                key: "api".to_string(),
                name: "API 访问".to_string(),
                url: accel_api_url,
                success: false,
                latency: None,
                error: None,
                warning: None,
            });
        }
    }

    // 5. 下载速度测试：复用 MiniGit 流式下载逻辑
    let speed_test_url = "https://github.com/al01cn/sillyTavern-launcher/releases/download/v0.1.5/SillyTavern.Launcher.GUI_x64.app.tar.gz";
    let accel_speed_url = format!("{}/{}", accel_base, speed_test_url);

    let speed_client = match reqwest::Client::builder()
        .user_agent("sillyTavern-launcher")
        .redirect(reqwest::redirect::Policy::limited(15))
        .timeout(std::time::Duration::from_secs(60))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Github 加速测试 [下载速度] 创建客户端失败: {}", e);
            results.push(GithubTestResultItem {
                key: "speed".to_string(),
                name: "下载速度".to_string(),
                url: accel_speed_url,
                success: false,
                latency: None,
                error: None,
                warning: None,
            });
            return Ok(results);
        }
    };

    let speed_result = run_download_speed_test(&speed_client, &accel_speed_url).await;
    results.push(GithubTestResultItem {
        key: "speed".to_string(),
        name: "下载速度".to_string(),
        url: accel_speed_url,
        success: speed_result
            .as_ref()
            .map(|r| r.speed_mbps >= 1.0)
            .unwrap_or(false),
        latency: speed_result.as_ref().ok().map(|r| r.download_time_ms),
        error: None,
        warning: speed_result
            .ok()
            .map(|r| format_speed_message(r.speed_mbps)),
    });

    Ok(results)
}

/// 格式化速度提示信息
fn format_speed_message(speed_mbps: f64) -> String {
    if speed_mbps < 1.0 {
        format!("速度太慢 ({:.2} MB/s)", speed_mbps)
    } else if speed_mbps < 4.0 {
        format!("速度正常 ({:.2} MB/s)", speed_mbps)
    } else if speed_mbps < 10.0 {
        format!("速度很快 ({:.2} MB/s)", speed_mbps)
    } else {
        format!("速度极快 ({:.2} MB/s)", speed_mbps)
    }
}

/// 通用下载测速辅助函数
///
/// 复用 MiniGit 流式下载逻辑：
/// - 流式下载到 AppData 缓存目录下的随机临时文件
/// - 最多下载 4MB，统计平均速度
/// - 下载完成后异步延迟 2 秒删除临时文件
async fn run_download_speed_test(
    client: &reqwest::Client,
    url: &str,
) -> Result<DownloadSpeedResult, String> {
    use futures_util::StreamExt;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};
    use tokio::io::AsyncWriteExt;

    // 生成随机文件名（纳秒时间戳 + 进程 ID）
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let tmp_name = format!("spd_{:x}_{}.tmp", ts, pid);

    // 存放到系统临时目录 ($env:TEMP / %TEMP%)
    let cache_dir = std::env::temp_dir();
    let _ = tokio::fs::create_dir_all(&cache_dir).await;
    let tmp_path = cache_dir.join(tmp_name);

    // 发送请求
    let start_request = Instant::now();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let request_latency = start_request.elapsed().as_millis() as u64;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    // 流式下载，写入临时文件，限制 4 MB
    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(|e| format!("创建临时文件失败: {}", e))?;
    let mut downloaded: u64 = 0;
    let download_start = Instant::now();
    let mut stream = response.bytes_stream();
    const MAX_TEST_BYTES: u64 = 4 * 1024 * 1024; // 最多下载 4MB

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("读取数据失败: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("写入临时文件失败: {}", e))?;
        downloaded += chunk.len() as u64;
        if downloaded >= MAX_TEST_BYTES {
            break;
        }
    }

    let _ = file.flush().await;
    drop(file);

    if downloaded == 0 {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err("下载失败：未接收到任何数据".to_string());
    }

    let download_time = download_start.elapsed();
    let download_time_ms = download_time.as_millis() as u64;
    let speed_mbps = (downloaded as f64 / 1_048_576.0) / download_time.as_secs_f64().max(0.001);

    tracing::info!(
        "下载速度测试完成：{:.2} MB/s，下载 {} bytes，耗时 {} ms",
        speed_mbps,
        downloaded,
        download_time_ms
    );

    // 异步延迟 2 秒后删除临时文件
    let tmp_path_clone = tmp_path.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let _ = tokio::fs::remove_file(&tmp_path_clone).await;
        tracing::debug!("已删除下载测速临时文件: {:?}", tmp_path_clone);
    });

    Ok(DownloadSpeedResult {
        request_latency,
        speed_mbps,
        download_time_ms,
        downloaded_bytes: downloaded,
        error: None,
    })
}

/// 下载速度测试结果
#[derive(serde::Serialize)]
pub struct DownloadSpeedResult {
    /// 请求延迟 (ms)
    pub request_latency: u64,
    /// 平均下载速度 (MB/s)
    pub speed_mbps: f64,
    /// 下载用时 (ms)
    pub download_time_ms: u64,
    /// 下载数据量 (bytes)
    pub downloaded_bytes: u64,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 测试下载速度
/// mode: "direct" | "accelerate"
/// - "direct": 直连测试
/// - "accelerate": 加速地址 + 目标 URL
#[tauri::command]
pub async fn test_download_speed(
    mode: String,
    host: String,
) -> Result<DownloadSpeedResult, String> {
    // 目标文件地址
    let target_url = "https://github.com/al01cn/sillyTavern-launcher/releases/download/v0.1.5/SillyTavern.Launcher.GUI_x64.app.tar.gz";

    // 根据模式拼接 URL
    let test_url = if mode == "accelerate" {
        let accel_base = host.trim_end_matches('/');
        format!("{}/{}", accel_base, target_url)
    } else {
        target_url.to_string()
    };

    let client = reqwest::Client::builder()
        .user_agent("sillyTavern-launcher")
        .redirect(reqwest::redirect::Policy::limited(15))
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Build client failed: {}", e))?;

    run_download_speed_test(&client, &test_url).await
}
