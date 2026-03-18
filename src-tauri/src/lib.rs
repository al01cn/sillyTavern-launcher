use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position, WindowEvent};

#[derive(Clone, Serialize)]
struct DownloadProgress {
    status: String,
    progress: f64, // 0.0 to 1.0
    log: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
struct WindowPosition {
    x: i32,
    y: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
struct GithubProxyConfig {
    enable: bool,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
struct AppConfig {
    lang: String,
    theme: String,
    remember_window_position: bool,
    window_position: Option<WindowPosition>,
    github_proxy: GithubProxyConfig,
}

impl Default for WindowPosition {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Default for GithubProxyConfig {
    fn default() -> Self {
        Self {
            enable: false,
            url: "https://gitproxy.click/".to_string(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            lang: "zh-CN".to_string(),
            theme: "dark".to_string(),
            remember_window_position: false,
            window_position: None,
            github_proxy: GithubProxyConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProxyItem {
    url: String,
    server: String,
    ip: String,
    location: String,
    latency: u32,
    speed: f64,
    tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProxyResponse {
    code: u32,
    msg: String,
    data: Vec<ProxyItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NodeInfo {
    version: Option<String>,
    path: Option<String>,
    source: String, // "system", "local", or "none"
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn get_config_path(_app: &AppHandle) -> PathBuf {
    // Determine the base path based on environment
    // In development (tauri dev), we want to use the project root 'data' folder
    // In production, we might want to use the executable's directory or app_data_dir
    
    // For now, let's try to locate the 'data' folder relative to the current working directory
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    // If we are in src-tauri (common during dev), move up one level
    if path.ends_with("src-tauri") {
        path.pop();
    }
    
    path.join("data/config.json")
}

fn read_app_config_from_disk(app: &AppHandle) -> AppConfig {
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

fn write_app_config_to_disk(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path(app);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&config_path, content).map_err(|e| format!("Failed to write config file: {}", e))?;
    Ok(())
}

fn apply_saved_window_position(app: &AppHandle) {
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
    let _ = window.set_position(Position::Physical(PhysicalPosition::new(position.x, position.y)));
}

fn setup_window_position_tracking(app: &AppHandle) {
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
                config.window_position = Some(WindowPosition {
                    x: position.x,
                    y: position.y,
                });
                let _ = write_app_config_to_disk(&app_handle, &config);
            }
        }
    });
}

 #[tauri::command]
 async fn get_app_config(app: AppHandle) -> Result<AppConfig, String> {
     println!("Loading config from: {:?}", get_config_path(&app));
     Ok(read_app_config_from_disk(&app))
 }

 #[tauri::command]
 async fn save_app_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
     println!("Saving config to: {:?}", get_config_path(&app));
     write_app_config_to_disk(&app, &config)
 }

#[tauri::command]
async fn fetch_github_proxies() -> Result<Vec<ProxyItem>, String> {
    let client = reqwest::Client::builder()
        .user_agent("tavern-assistant")
        .build()
        .map_err(|e| e.to_string())?;
        
    let response: ProxyResponse = client.get("https://api.akams.cn/github")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
        
    if response.code == 200 {
        Ok(response.data)
    } else {
        Err(format!("API Error: {}", response.msg))
    }
}

#[tauri::command]
async fn check_nodejs(app: AppHandle) -> Result<NodeInfo, String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let node_dir = data_dir.join("node");
    
    // Check local node
    let local_node_path = if cfg!(target_os = "windows") {
        node_dir.join("node.exe")
    } else {
        node_dir.join("bin/node")
    };

    if local_node_path.exists() {
        // Use full path for command on Windows to ensure we pick the right one? 
        // Actually Command::new works with paths.
        if let Ok(output) = std::process::Command::new(&local_node_path).arg("-v").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(NodeInfo {
                    version: Some(version),
                    path: Some(local_node_path.to_string_lossy().to_string()),
                    source: "local".to_string(),
                });
            }
        }
    }

    // Check system node
    let cmd = if cfg!(target_os = "windows") { "node" } else { "node" };
    
    if let Ok(output) = std::process::Command::new(cmd).arg("-v").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            
            // Try to find the actual path of the system node
            let path_cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
            let mut node_path = "system".to_string();
            
            if let Ok(path_output) = std::process::Command::new(path_cmd).arg("node").output() {
                if path_output.status.success() {
                    let path_str = String::from_utf8_lossy(&path_output.stdout);
                    // 'where' might return multiple lines, take the first one
                    if let Some(first_line) = path_str.lines().next() {
                        let trimmed = first_line.trim();
                        if !trimmed.is_empty() {
                            node_path = trimmed.to_string();
                        }
                    }
                }
            }

            return Ok(NodeInfo {
                version: Some(version),
                path: Some(node_path), 
                source: "system".to_string(),
            });
        }
    }

    Ok(NodeInfo {
        version: None,
        path: None,
        source: "none".to_string(),
    })
}

#[tauri::command]
async fn install_nodejs(app: AppHandle) -> Result<(), String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let node_os = match os {
        "windows" => "win",
        "linux" => "linux",
        "macos" => "darwin",
        _ => return Err(format!("Unsupported OS: {}", os)),
    };

    let node_arch = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => return Err(format!("Unsupported Arch: {}", arch)),
    };

    let filename = format!("node-v18.20.4-{}-{}.zip", node_os, node_arch);
    let url = format!("https://npmmirror.com/mirrors/node/v18.20.4/{}", filename);

    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let node_dir = data_dir.join("node");
    
    let emit_progress = |status: &str, progress: f64, log: &str| {
         let _ = app.emit("download-progress", DownloadProgress {
            status: status.to_string(),
            progress,
            log: log.to_string(),
        });
    };

    emit_progress("downloading", 0.0, &format!("开始下载 Node.js: {}", filename));

    let temp_dir = std::env::temp_dir();
    let temp_zip_path = temp_dir.join(&filename);
    
    let client = reqwest::Client::builder()
        .user_agent("tavern-assistant")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let total_size = response.content_length().unwrap_or(0);

    let mut file = fs::File::create(&temp_zip_path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        
        let progress = if total_size > 0 {
            (downloaded as f64) / (total_size as f64)
        } else {
            0.0
        };
        
        // Only emit every 1% or so to avoid spamming? Or just let it be.
        // Let's limit it slightly implicitly by the chunk size.
        let mb_downloaded = downloaded as f64 / 1_048_576.0;
        let mb_total = total_size as f64 / 1_048_576.0;
        emit_progress("downloading", progress, &format!("下载中: {:.2} MB / {:.2} MB", mb_downloaded, mb_total));
    }

    emit_progress("extracting", 0.0, "下载完成，正在解压...");

    if node_dir.exists() {
        fs::remove_dir_all(&node_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&node_dir).map_err(|e| e.to_string())?;

    let file = fs::File::open(&temp_zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let total_files = archive.len();

    for i in 0..total_files {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let mut components = outpath.components();
        components.next(); // Skip root folder
        let stripped_path: PathBuf = components.collect();

        if stripped_path.as_os_str().is_empty() {
             continue;
        }

        let target_path = node_dir.join(&stripped_path);

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;
        } else {
             if let Some(p) = target_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = fs::File::create(&target_path).map_err(|e| e.to_string())?;
            io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
        
        if i % 50 == 0 || i == total_files - 1 {
             let progress = (i as f64) / (total_files as f64);
             emit_progress("extracting", progress, &format!("解压中: {}/{} 文件...", i + 1, total_files));
        }
    }

    let _ = fs::remove_file(temp_zip_path);
    emit_progress("done", 1.0, "Node.js 安装完成");
    
    Ok(())
}

#[tauri::command]
async fn check_sillytavern_empty(app: AppHandle) -> Result<bool, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let sillytavern_dir = app_data_dir.join("data").join("sillytavern");
    
    if !sillytavern_dir.exists() {
        return Ok(true);
    }
    
    let entries = match fs::read_dir(&sillytavern_dir) {
        Ok(e) => e,
        Err(_) => return Ok(true), // If we can't read it, assume it's empty/invalid
    };
    
    let mut has_valid_files = false;
    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            if file_name_str != ".gitkeep" && file_name_str != ".DS_Store" {
                has_valid_files = true;
                break;
            }
        }
    }
    
    Ok(!has_valid_files)
}

#[tauri::command]
fn open_directory(app: AppHandle, dir_type: String, custom_path: Option<String>) -> Result<(), String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    
    let target_dir = match dir_type.as_str() {
        "root" => data_dir,
        "logs" => data_dir.join("logs"),
        "tavern" => data_dir.join("sillytavern"),
        "node" => {
            if let Some(path) = custom_path {
                let path_buf = PathBuf::from(path);
                if path_buf.is_file() {
                    path_buf.parent().unwrap_or(&data_dir.join("node")).to_path_buf()
                } else {
                    path_buf
                }
            } else {
                data_dir.join("node")
            }
        },
        _ => return Err(format!("Unknown directory type: {}", dir_type)),
    };
    
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(target_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(target_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(target_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
fn get_tavern_version(app: AppHandle) -> Result<String, String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let package_json_path = data_dir.join("sillytavern").join("package.json");
    
    if !package_json_path.exists() {
        return Err("未安装".to_string());
    }
    
    let content = fs::read_to_string(&package_json_path).map_err(|e| e.to_string())?;
    let parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    if let Some(version) = parsed.get("version").and_then(|v| v.as_str()) {
        Ok(version.to_string())
    } else {
        Err("未知".to_string())
    }
}

#[tauri::command]
async fn download_sillytavern(app: AppHandle, version: String, url: String) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let sillytavern_dir = app_data_dir.join("data").join("sillytavern").join(&version);
    
    if !sillytavern_dir.exists() {
        fs::create_dir_all(&sillytavern_dir).map_err(|e| e.to_string())?;
    }

    let emit_progress = |status: &str, progress: f64, log: &str| {
        let _ = app.emit("download-progress", DownloadProgress {
            status: status.to_string(),
            progress,
            log: log.to_string(),
        });
    };

    emit_progress("downloading", 0.0, &format!("准备下载版本 {}...", version));
    
    // Download zip to temp dir
    let temp_dir = std::env::temp_dir();
    let temp_zip_path = temp_dir.join(format!("sillytavern_{}.zip", version));
    
    let client = reqwest::Client::builder()
        .user_agent("tavern-assistant")
        .build()
        .map_err(|e| e.to_string())?;
        
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let total_size = response.content_length().unwrap_or(0);
    
    let mut file = fs::File::create(&temp_zip_path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        
        let progress = if total_size > 0 {
            (downloaded as f64) / (total_size as f64)
        } else {
            0.0
        };
        
        let mb_downloaded = downloaded as f64 / 1_048_576.0;
        let mb_total = total_size as f64 / 1_048_576.0;
        
        emit_progress(
            "downloading",
            progress,
            &format!("下载中: {:.2} MB / {:.2} MB", mb_downloaded, mb_total),
        );
    }
    
    emit_progress("extracting", 0.0, "下载完成，准备解压...");
    
    // Extract zip
    let file = fs::File::open(&temp_zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let total_files = archive.len();
    
    for i in 0..total_files {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        
        // Strip the first component (the root folder of the github zip)
        let mut components = outpath.components();
        components.next(); // Skip the first directory
        let stripped_path: PathBuf = components.collect();
        
        if stripped_path.as_os_str().is_empty() {
            continue;
        }
        
        let target_path = sillytavern_dir.join(&stripped_path);
        
        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = target_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = fs::File::create(&target_path).map_err(|e| e.to_string())?;
            io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }

        if i % 50 == 0 || i == total_files - 1 {
            let progress = (i as f64) / (total_files as f64);
            emit_progress(
                "extracting",
                progress,
                &format!("解压中: {}/{} 文件...", i + 1, total_files),
            );
        }
    }
    
    // Clean up temp file
    let _ = fs::remove_file(temp_zip_path);
    
    emit_progress("done", 1.0, "安装完成！");
    
    Ok(())
}

fn ensure_file_with_default(path: &Path, content: &str) -> io::Result<()> {
    if !path.exists() {
        fs::write(path, content)?;
    }
    Ok(())
}

fn ensure_standard_layout(base_dir: &Path) -> io::Result<()> {
    let data_dir = base_dir.join("data");
    let sillytavern_dir = data_dir.join("sillytavern");
    let logs_dir = data_dir.join("logs");
    let config_path = data_dir.join("config.json");

    fs::create_dir_all(&sillytavern_dir)?;
    fs::create_dir_all(&logs_dir)?;
    
    let default_config = AppConfig::default();
    let default_config_str = serde_json::to_string_pretty(&default_config).unwrap();
    
    ensure_file_with_default(
        &config_path,
        &default_config_str,
    )?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|err| io::Error::other(err.to_string()))?;
            ensure_standard_layout(&app_data_dir)?;
            let app_handle = app.handle().clone();
            apply_saved_window_position(&app_handle);
            setup_window_position_tracking(&app_handle);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            check_sillytavern_empty,
            download_sillytavern,
            get_app_config,
            save_app_config,
            fetch_github_proxies,
            check_nodejs,
            install_nodejs,
            open_directory,
            get_app_version,
            get_tavern_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
