use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Serialize)]
struct DownloadProgress {
    status: String,
    progress: f64, // 0.0 to 1.0
    log: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GithubProxyConfig {
    enable: bool,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AppConfig {
    lang: String,
    theme: String,
    github_proxy: GithubProxyConfig,
}

impl Default for GithubProxyConfig {
    fn default() -> Self {
        Self {
            enable: false,
            url: "https://ghproxy.com/".to_string(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            lang: "zh-CN".to_string(),
            theme: "dark".to_string(),
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

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn get_config_path(app: &AppHandle) -> PathBuf {
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

 #[tauri::command]
 async fn get_app_config(app: AppHandle) -> Result<AppConfig, String> {
     let config_path = get_config_path(&app);
     
     // Log the path being used for debugging
     println!("Loading config from: {:?}", config_path);
     
     if !config_path.exists() {
         println!("Config file not found, using default.");
         return Ok(AppConfig::default());
     }

     let content = fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;
     let config: AppConfig = serde_json::from_str(&content).unwrap_or_default();
     Ok(config)
 }

 #[tauri::command]
 async fn save_app_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
     let config_path = get_config_path(&app);
     println!("Saving config to: {:?}", config_path);
     
     // Ensure data directory exists
     if let Some(parent) = config_path.parent() {
         fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
     }
     
     let content = serde_json::to_string_pretty(&config).map_err(|e| format!("Failed to serialize config: {}", e))?;
     fs::write(&config_path, content).map_err(|e| format!("Failed to write config file: {}", e))?;
     Ok(())
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
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            check_sillytavern_empty,
            download_sillytavern,
            get_app_config,
            save_app_config,
            fetch_github_proxies
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
