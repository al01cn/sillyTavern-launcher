use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position, WindowEvent};
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};

struct ProcessState {
    kill_tx: Arc<Mutex<Option<tokio::sync::mpsc::Sender<()>>>>,
}

struct InstallState {
    cancel_flag: Arc<std::sync::atomic::AtomicBool>,
}

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
struct SillyTavernConfig {
    version: String,
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
            url: "https://gh.llkk.cc".to_string(),
        }
    }
}

impl Default for SillyTavernConfig {
    fn default() -> Self {
        Self {
            version: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
struct AppConfig {
    lang: String,
    theme: String,
    remember_window_position: bool,
    window_position: Option<WindowPosition>,
    github_proxy: GithubProxyConfig,
    sillytavern: SillyTavernConfig,
    npm_registry: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            lang: "zh-CN".to_string(),
            theme: "dark".to_string(),
            remember_window_position: false,
            window_position: None,
            github_proxy: GithubProxyConfig::default(),
            sillytavern: SillyTavernConfig::default(),
            npm_registry: "https://registry.npmjs.org/".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NodeInfo {
    version: Option<String>,
    path: Option<String>,
    source: String, // "system", "local", or "none"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NpmInfo {
    version: Option<String>,
    path: Option<String>,
    source: String, // "system", "local", or "none"
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
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Release {
    tag_name: String,
    name: String,
    body: String,
    created_at: String,
    published_at: String,
    zipball_url: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernConfigPayload {
    port: i64,
    listen: bool,
    listen_address: TavernDualStackAddress,
    protocol: TavernDualStackProtocol,
    basic_auth_mode: bool,
    enable_user_accounts: bool,
    enable_discreet_login: bool,
    per_user_basic_auth: bool,
    basic_auth_user: TavernBasicAuthUser,
    whitelist_mode: bool,
    whitelist: Vec<String>,
    cors: TavernCorsConfig,
    request_proxy: TavernRequestProxyConfig,
    backups: TavernBackupsConfig,
    thumbnails: TavernThumbnailsConfig,
    browser_launch_enabled: bool,
    browser_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernDualStackAddress {
    ipv4: String,
    ipv6: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernDualStackProtocol {
    ipv4: bool,
    ipv6: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernCorsConfig {
    enabled: bool,
    origin: Vec<String>,
    methods: Vec<String>,
    allowed_headers: Vec<String>,
    exposed_headers: Vec<String>,
    credentials: bool,
    max_age: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernRequestProxyConfig {
    enabled: bool,
    url: String,
    bypass: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernBasicAuthUser {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernBackupsConfig {
    common: TavernBackupsCommonConfig,
    chat: TavernBackupsChatConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernBackupsCommonConfig {
    number_of_backups: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernBackupsChatConfig {
    enabled: bool,
    check_integrity: bool,
    max_total_backups: i64,
    throttle_interval: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernThumbnailsConfig {
    enabled: bool,
    format: String,
    quality: i64,
    dimensions: TavernThumbnailsDimensionsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TavernThumbnailsDimensionsConfig {
    bg: Vec<i64>,
    avatar: Vec<i64>,
    persona: Vec<i64>,
}

#[tauri::command]
async fn fetch_sillytavern_releases() -> Result<Vec<Release>, String> {
    let client = reqwest::Client::builder()
        .user_agent("sillyTavern-launcher")
        .build()
        .map_err(|e| e.to_string())?;

    let url = "https://api.github.com/repos/SillyTavern/SillyTavern/releases";
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("GitHub API Error: {}", response.status()));
    }

    let releases: Vec<Release> = response.json().await.map_err(|e| e.to_string())?;
    Ok(releases)
}

#[tauri::command]
fn get_installed_sillytavern_versions(app: AppHandle) -> Result<Vec<String>, String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let sillytavern_dir = data_dir.join("sillytavern");

    if !sillytavern_dir.exists() {
        return Ok(vec![]);
    }

    let mut versions = Vec::new();
    if let Ok(entries) = fs::read_dir(sillytavern_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if !name.starts_with(".") {
                             versions.push(name);
                        }
                    }
                }
            }
        }
    }
    Ok(versions)
}

fn get_npm_install_command(data_dir: &Path, registry: &str) -> Option<(PathBuf, Vec<String>)> {
    let node_dir = data_dir.join("node");
    
    // Check local node first
    let local_node_path = if cfg!(target_os = "windows") {
        node_dir.join("node.exe")
    } else {
        node_dir.join("bin/node")
    };

    if local_node_path.exists() {
        // 1. Try npm-cli.js with local node (Preferred as it uses the bundled node)
        // Windows layout: node_modules/npm/bin/npm-cli.js
        // Linux layout: lib/node_modules/npm/bin/npm-cli.js
        let npm_cli_paths = vec![
            node_dir.join("node_modules/npm/bin/npm-cli.js"),
            node_dir.join("lib/node_modules/npm/bin/npm-cli.js"),
        ];

        for cli in npm_cli_paths {
            if cli.exists() {
                 return Some((
                    local_node_path.clone(), 
                    vec![
                        cli.to_string_lossy().to_string(), 
                        "install".to_string(), 
                        "--production".to_string(), 
                        "--no-audit".to_string(),
                        "--no-fund".to_string(),
                        format!("--registry={}", registry)
                    ]
                ));
            }
        }

        // 2. Try npm.cmd / npm binary in local node dir
        let npm_exec = if cfg!(target_os = "windows") {
            node_dir.join("npm.cmd")
        } else {
            node_dir.join("bin/npm")
        };
        
        if npm_exec.exists() {
            return Some((
                npm_exec, 
                vec![
                    "install".to_string(), 
                    "--production".to_string(), 
                    "--no-audit".to_string(),
                    "--no-fund".to_string(),
                    format!("--registry={}", registry)
                ]
            ));
        }
    }

    // 3. Fallback to system npm
    let system_npm = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };
    // Check if system npm is available
    if std::process::Command::new(system_npm).arg("-v").output().is_ok() {
        return Some((
            PathBuf::from(system_npm), 
            vec![
                "install".to_string(), 
                "--production".to_string(), 
                "--no-audit".to_string(),
                "--no-fund".to_string(),
                format!("--registry={}", registry)
            ]
        ));
    }

    None
}

#[tauri::command]
async fn switch_sillytavern_version(app: AppHandle, version: String) -> Result<(), String> {
    let mut config = read_app_config_from_disk(&app);
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let version_dir = data_dir.join("sillytavern").join(&version);
    
    if !version_dir.exists() {
        return Err(format!("Version {} not found", version));
    }

    config.sillytavern.version = version;
    write_app_config_to_disk(&app, &config)
}

#[tauri::command]
fn cancel_install(state: tauri::State<'_, InstallState>) {
    state.cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[tauri::command]
async fn install_sillytavern_version(app: AppHandle, state: tauri::State<'_, InstallState>, version: String, url: String) -> Result<(), String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let sillytavern_dir = data_dir.join("sillytavern").join(&version);
    
    if sillytavern_dir.exists() {
         return Ok(()); // Already installed
    }
    
    fs::create_dir_all(&sillytavern_dir).map_err(|e| e.to_string())?;

    state.cancel_flag.store(false, std::sync::atomic::Ordering::Relaxed);

    let emit_progress = |status: &str, progress: f64, log: &str| {
        let _ = app.emit("install-progress", DownloadProgress {
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
        .user_agent("sillyTavern-launcher")
        .build()
        .map_err(|e| e.to_string())?;
        
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let total_size = response.content_length().unwrap_or(0);
    
    let mut file = fs::File::create(&temp_zip_path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        // Only check cancel flag periodically or do it fast
        if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = fs::remove_file(&temp_zip_path);
            let _ = fs::remove_dir_all(&sillytavern_dir);
            emit_progress("error", 0.0, "下载已取消");
            return Err("下载已取消".to_string());
        }

        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        
        let progress = if total_size > 0 {
            (downloaded as f64) / (total_size as f64)
        } else {
            0.0
        };
        
        let mb_downloaded = downloaded as f64 / 1_048_576.0;
        
        emit_progress(
            "downloading",
            progress,
            &format!("已下载: {:.2} MB", mb_downloaded),
        );
    }
    
    emit_progress("extracting", 0.0, "下载完成，准备解压...");
    
    // Extract zip
    let file = fs::File::open(&temp_zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let total_files = archive.len();
    
    for i in 0..total_files {
        if i % 10 == 0 && state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = fs::remove_file(&temp_zip_path);
            let _ = fs::remove_dir_all(&sillytavern_dir);
            emit_progress("error", 0.0, "解压已取消");
            return Err("解压已取消".to_string());
        }

        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        
        // Strip the first component
        let mut components = outpath.components();
        components.next(); 
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
    
    // Install dependencies
    emit_progress("installing", 0.0, "正在安装依赖 (npm install)... 这可能需要几分钟");
    
    if let Err(e) = run_npm_install(&app, &sillytavern_dir).await {
         emit_progress("error", 0.0, &format!("安装依赖失败: {}", e));
    } else {
         emit_progress("done", 1.0, "安装完成！");
    }
    
    Ok(())
}

async fn run_npm_install(app: &AppHandle, target_dir: &Path) -> Result<(), String> {
    let data_dir = get_config_path(app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let config = read_app_config_from_disk(app);
    let registry = config.npm_registry;
    
    let npm_cmd = get_npm_install_command(&data_dir, &registry);
    
    let emit_progress = |status: &str, progress: f64, log: &str| {
        let _ = app.emit("install-progress", DownloadProgress {
            status: status.to_string(),
            progress,
            log: log.to_string(),
        });
    };

    if let Some((cmd, args)) = npm_cmd {
        use std::io::{BufRead, BufReader};
        use std::process::Stdio;

        // Log the command we are running
        emit_progress("installing", 0.1, &format!("执行命令: {:?} {:?}", cmd, args));

        let mut child = std::process::Command::new(&cmd)
            .args(&args)
            .current_dir(target_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start npm: {}", e))?;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    emit_progress("installing", 0.5, &line);
                }
            }
        }
        
        // Wait for completion
        let status = child.wait().map_err(|e| e.to_string())?;
        if !status.success() {
             return Err("npm install failed".to_string());
        }
    } else {
        return Err("未找到 npm，跳过依赖安装。请确保已安装 Node.js 或在设置中配置了正确的环境。".to_string());
    }
    
    Ok(())
}

#[tauri::command]
async fn install_sillytavern_dependencies(app: AppHandle, version: String) -> Result<(), String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let sillytavern_dir = data_dir.join("sillytavern").join(&version);
    
    if !sillytavern_dir.exists() {
         return Err(format!("Version {} not found", version));
    }
    
    // We reuse run_npm_install logic
    run_npm_install(&app, &sillytavern_dir).await?;
    
    // Emit done event
    let _ = app.emit("install-progress", DownloadProgress {
        status: "done".to_string(),
        progress: 1.0,
        log: "依赖安装完成！".to_string(),
    });
    
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct InstalledVersionInfo {
    version: String,
    has_node_modules: bool,
}

#[tauri::command]
fn get_installed_versions_info(app: AppHandle) -> Result<Vec<InstalledVersionInfo>, String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let sillytavern_dir = data_dir.join("sillytavern");

    if !sillytavern_dir.exists() {
        return Ok(vec![]);
    }

    let mut versions = Vec::new();
    if let Ok(entries) = fs::read_dir(sillytavern_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if !name.starts_with(".") {
                             let node_modules_path = entry.path().join("node_modules");
                             let has_node_modules = if node_modules_path.exists() {
                                 // Check if it's not empty
                                 if let Ok(nm_entries) = fs::read_dir(node_modules_path) {
                                     nm_entries.count() > 0
                                 } else {
                                     false
                                 }
                             } else {
                                 false
                             };
                             
                             versions.push(InstalledVersionInfo {
                                 version: name,
                                 has_node_modules,
                             });
                        }
                    }
                }
            }
        }
    }
    Ok(versions)
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
        .user_agent("sillyTavern-launcher")
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
async fn check_npm(app: AppHandle) -> Result<NpmInfo, String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let node_dir = data_dir.join("node");
    
    // Check local npm
    let local_node_path = if cfg!(target_os = "windows") {
        node_dir.join("node.exe")
    } else {
        node_dir.join("bin/node")
    };

    if local_node_path.exists() {
        // Try to find npm.cmd next to node.exe (Windows) or npm in bin (Unix)
        let npm_cmd = if cfg!(target_os = "windows") {
            node_dir.join("npm.cmd")
        } else {
            node_dir.join("bin/npm")
        };
        
        if npm_cmd.exists() {
             if let Ok(output) = std::process::Command::new(&npm_cmd).arg("-v").output() {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return Ok(NpmInfo {
                        version: Some(version),
                        path: Some(npm_cmd.to_string_lossy().to_string()),
                        source: "local".to_string(),
                    });
                }
            }
        }
        
        // Try finding npm-cli.js and run with node
        let npm_cli = if cfg!(target_os = "windows") {
             node_dir.join("node_modules").join("npm").join("bin").join("npm-cli.js")
        } else {
             node_dir.join("lib/node_modules/npm/bin/npm-cli.js") // Standard linux layout
        };
        
        // Also check flat structure if not found
        let npm_cli_flat = node_dir.join("node_modules/npm/bin/npm-cli.js");

        let target_cli = if npm_cli.exists() {
            Some(npm_cli)
        } else if npm_cli_flat.exists() {
            Some(npm_cli_flat)
        } else {
            None
        };

        if let Some(cli) = target_cli {
             if let Ok(output) = std::process::Command::new(&local_node_path)
                .arg(&cli)
                .arg("-v")
                .output() {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return Ok(NpmInfo {
                        version: Some(version),
                        path: Some(cli.to_string_lossy().to_string()),
                        source: "local".to_string(),
                    });
                }
            }
        }
    }

    // Check system npm
    let cmd = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };
    
    if let Ok(output) = std::process::Command::new(cmd).arg("-v").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            
             // Try to find path
            let path_cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
            let mut npm_path = "system".to_string();
            
            if let Ok(path_output) = std::process::Command::new(path_cmd).arg("npm").output() {
                if path_output.status.success() {
                    let path_str = String::from_utf8_lossy(&path_output.stdout);
                    if let Some(first_line) = path_str.lines().next() {
                        let trimmed = first_line.trim();
                        if !trimmed.is_empty() {
                            npm_path = trimmed.to_string();
                        }
                    }
                }
            }
            
            return Ok(NpmInfo {
                version: Some(version),
                path: Some(npm_path),
                source: "system".to_string(),
            });
        }
    }

    Ok(NpmInfo {
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
        .user_agent("sillyTavern-launcher")
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
        emit_progress("downloading", progress, &format!("已下载: {:.2} MB", mb_downloaded));
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
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let sillytavern_dir = data_dir.join("sillytavern");
    
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
        "data" => data_dir.join("st_data"),
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
    let config = read_app_config_from_disk(&app);
    let current_version = config.sillytavern.version;

    if current_version.is_empty() {
        return Err("未设置".to_string());
    }

    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let version_dir = data_dir.join("sillytavern").join(&current_version);
    let package_json_path = version_dir.join("package.json");
    
    if !version_dir.exists() {
        return Err("未安装".to_string());
    }
    
    if package_json_path.exists() {
        if let Ok(content) = fs::read_to_string(&package_json_path) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(version) = parsed.get("version").and_then(|v| v.as_str()) {
                    return Ok(version.to_string());
                }
            }
        }
    }
    
    // Fallback to configured version if package.json read fails or version not found
    Ok(current_version)
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
    let st_data_dir = data_dir.join("st_data");
    let config_path = data_dir.join("config.json");

    fs::create_dir_all(&sillytavern_dir)?;
    fs::create_dir_all(&logs_dir)?;
    fs::create_dir_all(&st_data_dir)?;
    
    let default_config = AppConfig::default();
    let default_config_str = serde_json::to_string_pretty(&default_config).unwrap();
    
    ensure_file_with_default(
        &config_path,
        &default_config_str,
    )?;

    Ok(())
}

fn get_sillytavern_config_file_path(app: &AppHandle, version: &str) -> Result<PathBuf, String> {
    if version.trim().is_empty() {
        return Err("版本号不能为空".to_string());
    }
    if version.contains("..") || version.contains('/') || version.contains('\\') {
        return Err("版本号不合法".to_string());
    }
    let data_dir = get_config_path(app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let config_path = data_dir.join("sillytavern").join(version).join("config.yaml");
    if !config_path.exists() {
        return Err(format!("配置文件不存在: {:?}", config_path));
    }
    Ok(config_path)
}

fn parse_tavern_config_payload(yaml_str: &str) -> Result<TavernConfigPayload, String> {
    let root: serde_yaml::Value =
        serde_yaml::from_str(yaml_str).map_err(|e| format!("解析配置失败: {}", e))?;
    let mapping = root
        .as_mapping()
        .ok_or("配置文件格式无效，根节点必须是对象".to_string())?;

    let get_bool = |key: &str, default: bool| -> bool {
        mapping
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(default)
    };

    let get_i64 = |key: &str, default: i64| -> i64 {
        mapping
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(serde_yaml::Value::as_i64)
            .unwrap_or(default)
    };

    let parse_string_sequence = |value: Option<&serde_yaml::Value>, default: Vec<String>| -> Vec<String> {
        value
            .and_then(serde_yaml::Value::as_sequence)
            .map(|seq| {
                seq.iter()
                    .filter_map(serde_yaml::Value::as_str)
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or(default)
    };

    let parse_i64_sequence = |value: Option<&serde_yaml::Value>, default: Vec<i64>| -> Vec<i64> {
        value
            .and_then(serde_yaml::Value::as_sequence)
            .map(|seq| {
                seq.iter()
                    .filter_map(serde_yaml::Value::as_i64)
                    .collect::<Vec<_>>()
            })
            .filter(|seq| !seq.is_empty())
            .unwrap_or(default)
    };

    let listen_address = mapping
        .get(serde_yaml::Value::String("listenAddress".to_string()))
        .and_then(serde_yaml::Value::as_mapping)
        .map(|item| TavernDualStackAddress {
            ipv4: item
                .get(serde_yaml::Value::String("ipv4".to_string()))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("0.0.0.0")
                .to_string(),
            ipv6: item
                .get(serde_yaml::Value::String("ipv6".to_string()))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("[::]")
                .to_string(),
        })
        .unwrap_or(TavernDualStackAddress {
            ipv4: "0.0.0.0".to_string(),
            ipv6: "[::]".to_string(),
        });

    let protocol = mapping
        .get(serde_yaml::Value::String("protocol".to_string()))
        .and_then(serde_yaml::Value::as_mapping)
        .map(|item| TavernDualStackProtocol {
            ipv4: item
                .get(serde_yaml::Value::String("ipv4".to_string()))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true),
            ipv6: item
                .get(serde_yaml::Value::String("ipv6".to_string()))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
        })
        .unwrap_or(TavernDualStackProtocol {
            ipv4: true,
            ipv6: false,
        });

    let whitelist = mapping
        .get(serde_yaml::Value::String("whitelist".to_string()))
        .and_then(serde_yaml::Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter_map(serde_yaml::Value::as_str)
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["::1".to_string(), "127.0.0.1".to_string()]);

    let basic_auth_user = mapping
        .get(serde_yaml::Value::String("basicAuthUser".to_string()))
        .and_then(serde_yaml::Value::as_mapping)
        .map(|item| TavernBasicAuthUser {
            username: item
                .get(serde_yaml::Value::String("username".to_string()))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("user")
                .to_string(),
            password: item
                .get(serde_yaml::Value::String("password".to_string()))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("password")
                .to_string(),
        })
        .unwrap_or(TavernBasicAuthUser {
            username: "user".to_string(),
            password: "password".to_string(),
        });
    let basic_auth_mode = get_bool("basicAuthMode", false);
    let enable_user_accounts = get_bool("enableUserAccounts", false);
    let enable_discreet_login = get_bool("enableDiscreetLogin", false);
    let per_user_basic_auth = get_bool("perUserBasicAuth", false);

    let cors = mapping
        .get(serde_yaml::Value::String("cors".to_string()))
        .and_then(serde_yaml::Value::as_mapping)
        .map(|item| TavernCorsConfig {
            enabled: item
                .get(serde_yaml::Value::String("enabled".to_string()))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true),
            origin: parse_string_sequence(
                item.get(serde_yaml::Value::String("origin".to_string())),
                vec!["null".to_string()],
            ),
            methods: parse_string_sequence(
                item.get(serde_yaml::Value::String("methods".to_string())),
                vec!["OPTIONS".to_string()],
            ),
            allowed_headers: parse_string_sequence(
                item.get(serde_yaml::Value::String("allowedHeaders".to_string())),
                vec![],
            ),
            exposed_headers: parse_string_sequence(
                item.get(serde_yaml::Value::String("exposedHeaders".to_string())),
                vec![],
            ),
            credentials: item
                .get(serde_yaml::Value::String("credentials".to_string()))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
            max_age: item
                .get(serde_yaml::Value::String("maxAge".to_string()))
                .and_then(serde_yaml::Value::as_i64),
        })
        .unwrap_or(TavernCorsConfig {
            enabled: true,
            origin: vec!["null".to_string()],
            methods: vec!["OPTIONS".to_string()],
            allowed_headers: vec![],
            exposed_headers: vec![],
            credentials: false,
            max_age: None,
        });

    let request_proxy = mapping
        .get(serde_yaml::Value::String("requestProxy".to_string()))
        .and_then(serde_yaml::Value::as_mapping)
        .map(|item| TavernRequestProxyConfig {
            enabled: item
                .get(serde_yaml::Value::String("enabled".to_string()))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
            url: item
                .get(serde_yaml::Value::String("url".to_string()))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("")
                .to_string(),
            bypass: item
                .get(serde_yaml::Value::String("bypass".to_string()))
                .and_then(serde_yaml::Value::as_sequence)
                .map(|seq| {
                    seq.iter()
                        .filter_map(serde_yaml::Value::as_str)
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        })
        .unwrap_or(TavernRequestProxyConfig {
            enabled: false,
            url: "".to_string(),
            bypass: vec![],
        });

    let backups = mapping
        .get(serde_yaml::Value::String("backups".to_string()))
        .and_then(serde_yaml::Value::as_mapping)
        .map(|item| {
            let common = item
                .get(serde_yaml::Value::String("common".to_string()))
                .and_then(serde_yaml::Value::as_mapping);
            let chat = item
                .get(serde_yaml::Value::String("chat".to_string()))
                .and_then(serde_yaml::Value::as_mapping);

            TavernBackupsConfig {
                common: TavernBackupsCommonConfig {
                    number_of_backups: common
                        .and_then(|x| {
                            x.get(serde_yaml::Value::String("numberOfBackups".to_string()))
                                .and_then(serde_yaml::Value::as_i64)
                        })
                        .unwrap_or(50),
                },
                chat: TavernBackupsChatConfig {
                    enabled: chat
                        .and_then(|x| {
                            x.get(serde_yaml::Value::String("enabled".to_string()))
                                .and_then(serde_yaml::Value::as_bool)
                        })
                        .unwrap_or(true),
                    check_integrity: chat
                        .and_then(|x| {
                            x.get(serde_yaml::Value::String("checkIntegrity".to_string()))
                                .and_then(serde_yaml::Value::as_bool)
                        })
                        .unwrap_or(true),
                    max_total_backups: chat
                        .and_then(|x| {
                            x.get(serde_yaml::Value::String("maxTotalBackups".to_string()))
                                .and_then(serde_yaml::Value::as_i64)
                        })
                        .unwrap_or(-1),
                    throttle_interval: chat
                        .and_then(|x| {
                            x.get(serde_yaml::Value::String("throttleInterval".to_string()))
                                .and_then(serde_yaml::Value::as_i64)
                        })
                        .unwrap_or(10000),
                },
            }
        })
        .unwrap_or(TavernBackupsConfig {
            common: TavernBackupsCommonConfig {
                number_of_backups: 50,
            },
            chat: TavernBackupsChatConfig {
                enabled: true,
                check_integrity: true,
                max_total_backups: -1,
                throttle_interval: 10000,
            },
        });

    let thumbnails = mapping
        .get(serde_yaml::Value::String("thumbnails".to_string()))
        .and_then(serde_yaml::Value::as_mapping)
        .map(|item| {
            let dimensions = item
                .get(serde_yaml::Value::String("dimensions".to_string()))
                .and_then(serde_yaml::Value::as_mapping);

            TavernThumbnailsConfig {
                enabled: item
                    .get(serde_yaml::Value::String("enabled".to_string()))
                    .and_then(serde_yaml::Value::as_bool)
                    .unwrap_or(true),
                format: item
                    .get(serde_yaml::Value::String("format".to_string()))
                    .and_then(serde_yaml::Value::as_str)
                    .unwrap_or("jpg")
                    .to_string(),
                quality: item
                    .get(serde_yaml::Value::String("quality".to_string()))
                    .and_then(serde_yaml::Value::as_i64)
                    .unwrap_or(95),
                dimensions: TavernThumbnailsDimensionsConfig {
                    bg: parse_i64_sequence(
                        dimensions.and_then(|x| x.get(serde_yaml::Value::String("bg".to_string()))),
                        vec![160, 90],
                    ),
                    avatar: parse_i64_sequence(
                        dimensions.and_then(|x| {
                            x.get(serde_yaml::Value::String("avatar".to_string()))
                        }),
                        vec![96, 144],
                    ),
                    persona: parse_i64_sequence(
                        dimensions.and_then(|x| {
                            x.get(serde_yaml::Value::String("persona".to_string()))
                        }),
                        vec![96, 144],
                    ),
                },
            }
        })
        .unwrap_or(TavernThumbnailsConfig {
            enabled: true,
            format: "jpg".to_string(),
            quality: 95,
            dimensions: TavernThumbnailsDimensionsConfig {
                bg: vec![160, 90],
                avatar: vec![96, 144],
                persona: vec![96, 144],
            },
        });

    let (browser_launch_enabled, browser_type) = mapping
        .get(serde_yaml::Value::String("browserLaunch".to_string()))
        .and_then(serde_yaml::Value::as_mapping)
        .map(|browser_launch| {
            let enabled = browser_launch
                .get(serde_yaml::Value::String("enabled".to_string()))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true);
            let browser = browser_launch
                .get(serde_yaml::Value::String("browser".to_string()))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("default")
                .to_string();
            (enabled, browser)
        })
        .unwrap_or((true, "default".to_string()));

    Ok(TavernConfigPayload {
        port: get_i64("port", 8000),
        listen: get_bool("listen", false),
        listen_address,
        protocol,
        basic_auth_mode,
        enable_user_accounts,
        enable_discreet_login,
        per_user_basic_auth,
        basic_auth_user,
        whitelist_mode: get_bool("whitelistMode", true),
        whitelist,
        cors,
        request_proxy,
        backups,
        thumbnails,
        browser_launch_enabled,
        browser_type,
    })
}

fn upsert_yaml_value(
    mapping: &mut serde_yaml::Mapping,
    key: &str,
    value: serde_yaml::Value,
) {
    mapping.insert(serde_yaml::Value::String(key.to_string()), value);
}

fn get_or_init_child_mapping<'a>(
    mapping: &'a mut serde_yaml::Mapping,
    key: &str,
) -> Result<&'a mut serde_yaml::Mapping, String> {
    let child_key = serde_yaml::Value::String(key.to_string());
    if !mapping.contains_key(&child_key) {
        mapping.insert(
            child_key.clone(),
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
    }
    mapping
        .get_mut(&child_key)
        .and_then(serde_yaml::Value::as_mapping_mut)
        .ok_or(format!("{} 配置格式无效", key))
}

#[tauri::command]
async fn delete_sillytavern_version(app: AppHandle, version: String) -> Result<(), String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let version_dir = data_dir.join("sillytavern").join(&version);

    if !version_dir.exists() {
        return Err(format!("版本 {} 不存在", version));
    }

    // Double check we are not deleting the whole sillytavern dir or something wrong
    if version.trim().is_empty() || version.contains("..") || version.contains("/") || version.contains("\\") {
         return Err("无效的版本号".to_string());
    }

    let app_clone = app.clone();
    let version_dir_clone = version_dir.clone();
    let version_clone = version.clone();

    let result = tokio::task::spawn_blocking(move || {
        let _ = app_clone.emit("install-progress", DownloadProgress {
            status: "deleting".to_string(),
            progress: 0.1,
            log: format!("开始删除版本 {}...", version_clone),
        });
        
        std::thread::sleep(std::time::Duration::from_millis(100));

        let _ = app_clone.emit("install-progress", DownloadProgress {
            status: "deleting".to_string(),
            progress: 0.3,
            log: format!("正在快速清理版本 {} 的全部文件...", version_clone),
        });

        // Collect some top-level file/dir names to simulate progress
        let mut sample_paths = Vec::new();
        if let Ok(entries) = fs::read_dir(&version_dir_clone) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    sample_paths.push(name);
                }
            }
        }

        let total_samples = sample_paths.len();
        // Emit fake deletion logs for files to give visual feedback
        for (i, name) in sample_paths.iter().enumerate() {
            std::thread::sleep(std::time::Duration::from_millis(15));
            let _ = app_clone.emit("install-progress", DownloadProgress {
                status: "deleting".to_string(),
                progress: 0.3 + (0.5 * (i as f64 / total_samples as f64)),
                log: format!("已删除：{}/{}", version_clone, name),
            });
        }

        fn fast_remove_dir_all(dir: &Path) -> io::Result<()> {
            if dir.is_dir() {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_dir() {
                                let _ = fast_remove_dir_all(&path);
                            } else {
                                if let Err(_e) = fs::remove_file(&path) {
                                    #[cfg(target_os = "windows")]
                                    {
                                        if let Ok(mut perms) = fs::metadata(&path).map(|m| m.permissions()) {
                                            if perms.readonly() {
                                                perms.set_readonly(false);
                                                let _ = fs::set_permissions(&path, perms);
                                                let _ = fs::remove_file(&path);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                let _ = fs::remove_dir(dir);
            }
            Ok(())
        }
        
        let _ = fast_remove_dir_all(&version_dir_clone);
        
        // Finally, ensure the directory is completely removed using standard library
        // to handle any edge cases
        if version_dir_clone.exists() {
            if let Err(e) = fs::remove_dir_all(&version_dir_clone) {
                return Err(e);
            }
        }
        
        let _ = app_clone.emit("install-progress", DownloadProgress {
            status: "deleting".to_string(),
            progress: 1.0,
            log: format!("版本 {} 的文件已全部删除完成", version_clone),
        });
        
        Ok(())
    }).await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(format!("删除失败: {}", e)),
        Err(e) => Err(format!("任务执行失败: {}", e)),
    }
}

#[tauri::command]
fn read_sillytavern_config(app: AppHandle, version: String) -> Result<String, String> {
    let config_path = get_sillytavern_config_file_path(&app, &version)?;
    fs::read_to_string(&config_path).map_err(|e| format!("读取失败: {}", e))
}

#[tauri::command]
fn write_sillytavern_config(app: AppHandle, version: String, content: String) -> Result<(), String> {
    let config_path = get_sillytavern_config_file_path(&app, &version)?;
    fs::write(&config_path, content).map_err(|e| format!("写入失败: {}", e))
}

#[tauri::command]
fn get_sillytavern_config_path(app: AppHandle, version: String) -> Result<String, String> {
    let config_path = get_sillytavern_config_file_path(&app, &version)?;
    Ok(config_path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_sillytavern_config_options(
    app: AppHandle,
    version: String,
) -> Result<TavernConfigPayload, String> {
    let config_path = get_sillytavern_config_file_path(&app, &version)?;
    let content = fs::read_to_string(&config_path).map_err(|e| format!("读取失败: {}", e))?;
    parse_tavern_config_payload(&content)
}

#[tauri::command]
fn update_sillytavern_config_options(
    app: AppHandle,
    version: String,
    config: TavernConfigPayload,
) -> Result<TavernConfigPayload, String> {
    let config_path = get_sillytavern_config_file_path(&app, &version)?;
    let content = fs::read_to_string(&config_path).map_err(|e| format!("读取失败: {}", e))?;
    let mut root: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| format!("解析配置失败: {}", e))?;
    let mapping = root
        .as_mapping_mut()
        .ok_or("配置文件格式无效，根节点必须是对象".to_string())?;

    upsert_yaml_value(
        mapping,
        "port",
        serde_yaml::Value::Number(serde_yaml::Number::from(config.port)),
    );
    upsert_yaml_value(mapping, "listen", serde_yaml::Value::Bool(config.listen));
    let listen_address = get_or_init_child_mapping(mapping, "listenAddress")?;
    upsert_yaml_value(
        listen_address,
        "ipv4",
        serde_yaml::Value::String(config.listen_address.ipv4.clone()),
    );
    upsert_yaml_value(
        listen_address,
        "ipv6",
        serde_yaml::Value::String(config.listen_address.ipv6.clone()),
    );

    let protocol = get_or_init_child_mapping(mapping, "protocol")?;
    upsert_yaml_value(protocol, "ipv4", serde_yaml::Value::Bool(config.protocol.ipv4));
    upsert_yaml_value(protocol, "ipv6", serde_yaml::Value::Bool(config.protocol.ipv6));
    upsert_yaml_value(
        mapping,
        "basicAuthMode",
        serde_yaml::Value::Bool(config.basic_auth_mode),
    );
    upsert_yaml_value(
        mapping,
        "enableUserAccounts",
        serde_yaml::Value::Bool(config.enable_user_accounts),
    );
    upsert_yaml_value(
        mapping,
        "enableDiscreetLogin",
        serde_yaml::Value::Bool(config.enable_discreet_login),
    );
    upsert_yaml_value(
        mapping,
        "perUserBasicAuth",
        serde_yaml::Value::Bool(config.per_user_basic_auth),
    );

    let basic_auth_user = get_or_init_child_mapping(mapping, "basicAuthUser")?;
    upsert_yaml_value(
        basic_auth_user,
        "username",
        serde_yaml::Value::String(config.basic_auth_user.username.clone()),
    );
    upsert_yaml_value(
        basic_auth_user,
        "password",
        serde_yaml::Value::String(config.basic_auth_user.password.clone()),
    );

    upsert_yaml_value(
        mapping,
        "whitelistMode",
        serde_yaml::Value::Bool(config.whitelist_mode),
    );
    upsert_yaml_value(
        mapping,
        "whitelist",
        serde_yaml::Value::Sequence(
            config
                .whitelist
                .iter()
                .map(|item| serde_yaml::Value::String(item.clone()))
                .collect(),
        ),
    );

    let cors = get_or_init_child_mapping(mapping, "cors")?;
    upsert_yaml_value(cors, "enabled", serde_yaml::Value::Bool(config.cors.enabled));
    upsert_yaml_value(
        cors,
        "origin",
        serde_yaml::Value::Sequence(
            config
                .cors
                .origin
                .iter()
                .map(|item| serde_yaml::Value::String(item.clone()))
                .collect(),
        ),
    );
    upsert_yaml_value(
        cors,
        "methods",
        serde_yaml::Value::Sequence(
            config
                .cors
                .methods
                .iter()
                .map(|item| serde_yaml::Value::String(item.clone()))
                .collect(),
        ),
    );
    upsert_yaml_value(
        cors,
        "allowedHeaders",
        serde_yaml::Value::Sequence(
            config
                .cors
                .allowed_headers
                .iter()
                .map(|item| serde_yaml::Value::String(item.clone()))
                .collect(),
        ),
    );
    upsert_yaml_value(
        cors,
        "exposedHeaders",
        serde_yaml::Value::Sequence(
            config
                .cors
                .exposed_headers
                .iter()
                .map(|item| serde_yaml::Value::String(item.clone()))
                .collect(),
        ),
    );
    upsert_yaml_value(
        cors,
        "credentials",
        serde_yaml::Value::Bool(config.cors.credentials),
    );
    upsert_yaml_value(
        cors,
        "maxAge",
        match config.cors.max_age {
            Some(value) => serde_yaml::Value::Number(serde_yaml::Number::from(value)),
            None => serde_yaml::Value::Null,
        },
    );

    let request_proxy = get_or_init_child_mapping(mapping, "requestProxy")?;
    upsert_yaml_value(
        request_proxy,
        "enabled",
        serde_yaml::Value::Bool(config.request_proxy.enabled),
    );
    upsert_yaml_value(
        request_proxy,
        "url",
        serde_yaml::Value::String(config.request_proxy.url.clone()),
    );
    upsert_yaml_value(
        request_proxy,
        "bypass",
        serde_yaml::Value::Sequence(
            config
                .request_proxy
                .bypass
                .iter()
                .map(|item| serde_yaml::Value::String(item.clone()))
                .collect(),
        ),
    );

    let backups = get_or_init_child_mapping(mapping, "backups")?;
    let backups_common = get_or_init_child_mapping(backups, "common")?;
    upsert_yaml_value(
        backups_common,
        "numberOfBackups",
        serde_yaml::Value::Number(serde_yaml::Number::from(config.backups.common.number_of_backups)),
    );
    let backups_chat = get_or_init_child_mapping(backups, "chat")?;
    upsert_yaml_value(
        backups_chat,
        "enabled",
        serde_yaml::Value::Bool(config.backups.chat.enabled),
    );
    upsert_yaml_value(
        backups_chat,
        "checkIntegrity",
        serde_yaml::Value::Bool(config.backups.chat.check_integrity),
    );
    upsert_yaml_value(
        backups_chat,
        "maxTotalBackups",
        serde_yaml::Value::Number(serde_yaml::Number::from(config.backups.chat.max_total_backups)),
    );
    upsert_yaml_value(
        backups_chat,
        "throttleInterval",
        serde_yaml::Value::Number(serde_yaml::Number::from(config.backups.chat.throttle_interval)),
    );

    let thumbnails = get_or_init_child_mapping(mapping, "thumbnails")?;
    upsert_yaml_value(
        thumbnails,
        "enabled",
        serde_yaml::Value::Bool(config.thumbnails.enabled),
    );
    upsert_yaml_value(
        thumbnails,
        "format",
        serde_yaml::Value::String(config.thumbnails.format.clone()),
    );
    upsert_yaml_value(
        thumbnails,
        "quality",
        serde_yaml::Value::Number(serde_yaml::Number::from(config.thumbnails.quality)),
    );
    let dimensions = get_or_init_child_mapping(thumbnails, "dimensions")?;
    upsert_yaml_value(
        dimensions,
        "bg",
        serde_yaml::Value::Sequence(
            config
                .thumbnails
                .dimensions
                .bg
                .iter()
                .map(|value| serde_yaml::Value::Number(serde_yaml::Number::from(*value)))
                .collect(),
        ),
    );
    upsert_yaml_value(
        dimensions,
        "avatar",
        serde_yaml::Value::Sequence(
            config
                .thumbnails
                .dimensions
                .avatar
                .iter()
                .map(|value| serde_yaml::Value::Number(serde_yaml::Number::from(*value)))
                .collect(),
        ),
    );
    upsert_yaml_value(
        dimensions,
        "persona",
        serde_yaml::Value::Sequence(
            config
                .thumbnails
                .dimensions
                .persona
                .iter()
                .map(|value| serde_yaml::Value::Number(serde_yaml::Number::from(*value)))
                .collect(),
        ),
    );

    let browser_launch_key = serde_yaml::Value::String("browserLaunch".to_string());
    if !mapping.contains_key(&browser_launch_key) {
        upsert_yaml_value(
            mapping,
            "browserLaunch",
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
    }
    let browser_launch = mapping
        .get_mut(&browser_launch_key)
        .and_then(serde_yaml::Value::as_mapping_mut)
        .ok_or("browserLaunch 配置格式无效".to_string())?;
    upsert_yaml_value(
        browser_launch,
        "enabled",
        serde_yaml::Value::Bool(config.browser_launch_enabled),
    );
    upsert_yaml_value(
        browser_launch,
        "browser",
        serde_yaml::Value::String(config.browser_type.clone()),
    );

    let new_content = serde_yaml::to_string(&root).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, new_content).map_err(|e| format!("写入失败: {}", e))?;
    Ok(config)
}

#[tauri::command]
async fn start_sillytavern(app: AppHandle, state: tauri::State<'_, ProcessState>) -> Result<(), String> {
    let mut kill_tx_guard = state.kill_tx.lock().await;
    if kill_tx_guard.is_some() {
        return Err("进程已经在运行中了".to_string());
    }

    let config = read_app_config_from_disk(&app);
    let version = config.sillytavern.version;
    if version.is_empty() {
        return Err("未选择酒馆版本，请先在版本页面选择或安装".to_string());
    }

    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let sillytavern_dir = data_dir.join("sillytavern").join(&version);
    
    // 全局数据目录
    let st_data_dir = data_dir.join("st_data");
    if !st_data_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&st_data_dir) {
            return Err(format!("无法创建全局数据目录: {}", e));
        }
    }

    if !sillytavern_dir.exists() {
        return Err(format!("版本 {} 的目录不存在，请检查是否已正确安装", version));
    }

    let node_dir = data_dir.join("node");
    let mut node_path = if cfg!(target_os = "windows") {
        node_dir.join("node.exe")
    } else {
        node_dir.join("bin/node")
    };

    if !node_path.exists() {
        node_path = PathBuf::from("node"); // Fallback to system node
    }

    let server_js = sillytavern_dir.join("server.js");

    if !server_js.exists() {
        return Err("找不到 server.js，酒馆文件可能损坏".to_string());
    }

    let mut std_cmd = std::process::Command::new(&node_path);
    std_cmd.arg(&server_js);
    
    // 使用 --dataRoot 参数来指定全局数据目录
    let st_data_dir_str = st_data_dir.to_string_lossy().to_string();
    std_cmd.arg("--dataRoot");
    std_cmd.arg(&st_data_dir_str);
    
    std_cmd.current_dir(&sillytavern_dir);
    // 同时设置环境变量，确保兼容性
    std_cmd.env("SILLYTAVERN_DATA_DIR", &st_data_dir_str);
    std_cmd.env("SillyTavern_DATA_DIR", &st_data_dir_str);
    std_cmd.stdout(std::process::Stdio::piped());
    std_cmd.stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        std_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let mut cmd = tokio::process::Command::from(std_cmd);
    let mut child = cmd.spawn().map_err(|e| format!("启动进程失败: {}", e))?;

    let stdout = child.stdout.take().ok_or("无法获取标准输出")?;
    let stderr = child.stderr.take().ok_or("无法获取标准错误")?;

    let app_clone1 = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_clone1.emit("process-log", format!("INFO: {}", line));
        }
    });

    let app_clone2 = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_clone2.emit("process-log", format!("ERROR: {}", line));
        }
    });

    let (kill_tx, mut kill_rx) = tokio::sync::mpsc::channel::<()>(1);
    *kill_tx_guard = Some(kill_tx);

    let app_clone3 = app.clone();
    let kill_tx_arc = state.inner().kill_tx.clone();
    
    tokio::spawn(async move {
        tokio::select! {
            _ = child.wait() => {
                let _ = app_clone3.emit("process-log", "INFO: 进程已退出".to_string());
            }
            _ = kill_rx.recv() => {
                let _ = child.kill().await;
                let _ = app_clone3.emit("process-log", "INFO: 进程已被终止".to_string());
            }
        }
        *kill_tx_arc.lock().await = None;
        let _ = app_clone3.emit("process-exit", ());
    });

    Ok(())
}

#[tauri::command]
async fn stop_sillytavern(state: tauri::State<'_, ProcessState>) -> Result<(), String> {
    let mut kill_tx_guard = state.kill_tx.lock().await;
    if let Some(kill_tx) = kill_tx_guard.take() {
        let _ = kill_tx.send(()).await;
    }
    Ok(())
}

#[tauri::command]
async fn check_sillytavern_status(state: tauri::State<'_, ProcessState>) -> Result<bool, String> {
    let kill_tx_guard = state.kill_tx.lock().await;
    Ok(kill_tx_guard.is_some())
}

#[tauri::command]
fn open_sillytavern_config_file(app: AppHandle, version: String) -> Result<(), String> {
    let config_path = get_sillytavern_config_file_path(&app, &version)?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(config_path)
            .spawn()
            .map_err(|e| format!("打开失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(config_path)
            .spawn()
            .map_err(|e| format!("打开失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(config_path)
            .spawn()
            .map_err(|e| format!("打开失败: {}", e))?;
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct ExtensionManifest {
    #[serde(rename = "display_name", default)]
    display_name: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(rename = "homePage", default)]
    home_page: Option<String>,
    #[serde(default)]
    auto_update: Option<bool>,
    #[serde(default)]
    minimum_client_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExtensionInfo {
    id: String,
    manifest: ExtensionManifest,
    dir_path: String,
    enabled: bool,
    is_official: bool,
    scope: String, // "global" or "user"
}

#[tauri::command]
fn get_extensions(app: tauri::AppHandle, version: String) -> Result<Vec<ExtensionInfo>, String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&std::path::PathBuf::from(".")).to_path_buf();
    let mut extensions = Vec::new();
    
    // Helper function to scan a directory for extensions
    let scan_dir = |dir_path: &PathBuf, is_official: bool, scope: &str, exts: &mut Vec<ExtensionInfo>| {
        if !dir_path.exists() {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        // Skip the third-party folder itself when scanning official extensions
                        if is_official && entry.file_name() == "third-party" {
                            continue;
                        }
                        
                        let mut manifest_path = entry.path().join("manifest.json");
                        let mut enabled = true;
                        
                        if !manifest_path.exists() {
                            manifest_path = entry.path().join("manifest.json.disable");
                            enabled = false;
                        }
                        
                        if manifest_path.exists() {
                            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                                if let Ok(manifest) = serde_json::from_str::<ExtensionManifest>(&content) {
                                    exts.push(ExtensionInfo {
                                        id: entry.file_name().to_string_lossy().to_string(),
                                        manifest,
                                        dir_path: entry.path().to_string_lossy().to_string(),
                                        enabled,
                                        is_official,
                                        scope: scope.to_string(),
                                    });
                                } else {
                                    let value: Result<serde_json::Value, _> = serde_json::from_str(&content);
                                    if let Ok(val) = value {
                                        let mut m = ExtensionManifest::default();
                                        if let Some(obj) = val.as_object() {
                                            m.display_name = obj.get("display_name").and_then(|v| v.as_str()).map(|s| s.to_string());
                                            m.author = obj.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());
                                            m.version = obj.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());
                                            m.home_page = obj.get("homePage").and_then(|v| v.as_str()).map(|s| s.to_string());
                                            m.auto_update = obj.get("auto_update").and_then(|v| v.as_bool());
                                            m.minimum_client_version = obj.get("minimum_client_version").and_then(|v| v.as_str()).map(|s| s.to_string());
                                        }
                                        exts.push(ExtensionInfo {
                                            id: entry.file_name().to_string_lossy().to_string(),
                                            manifest: m,
                                            dir_path: entry.path().to_string_lossy().to_string(),
                                            enabled,
                                            is_official,
                                            scope: scope.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    // 1. User Extensions (Current User)
    let user_extensions_dir = data_dir.join("st_data").join("default-user").join("extensions");
    scan_dir(&user_extensions_dir, false, "user", &mut extensions);
    
    // If a version is provided, scan global extensions for that version
    if !version.is_empty() {
        // 2. Global Official Extensions
        let global_official_dir = data_dir.join("sillytavern").join(&version).join("public").join("scripts").join("extensions");
        scan_dir(&global_official_dir, true, "global", &mut extensions);
        
        // 3. Global Third-Party Extensions
        let global_third_party_dir = global_official_dir.join("third-party");
        scan_dir(&global_third_party_dir, false, "global", &mut extensions);
    }
    
    Ok(extensions)
}

#[tauri::command]
fn toggle_extension_enable(_app: tauri::AppHandle, _id: String, enable: bool, dir_path: String) -> Result<(), String> {
    let extension_dir = PathBuf::from(&dir_path);
    
    if !extension_dir.exists() {
        return Err("扩展目录不存在".to_string());
    }
    
    let manifest_path = extension_dir.join("manifest.json");
    let disabled_manifest_path = extension_dir.join("manifest.json.disable");
    
    if enable {
        if disabled_manifest_path.exists() {
            std::fs::rename(&disabled_manifest_path, &manifest_path).map_err(|e| e.to_string())?;
        } else if !manifest_path.exists() {
            return Err("未找到清单文件".to_string());
        }
    } else {
        if manifest_path.exists() {
            std::fs::rename(&manifest_path, &disabled_manifest_path).map_err(|e| e.to_string())?;
        } else if !disabled_manifest_path.exists() {
            return Err("未找到清单文件".to_string());
        }
    }
    
    Ok(())
}

#[tauri::command]
fn delete_extension(_app: tauri::AppHandle, _id: String, dir_path: String) -> Result<(), String> {
    let extension_dir = PathBuf::from(&dir_path);
    
    if !extension_dir.exists() {
        return Err("扩展目录不存在".to_string());
    }
    
    std::fs::remove_dir_all(&extension_dir).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn toggle_extension_auto_update(_app: tauri::AppHandle, _id: String, auto_update: bool, dir_path: String) -> Result<(), String> {
    let extension_dir = PathBuf::from(&dir_path);
    
    let mut manifest_path = extension_dir.join("manifest.json");
    if !manifest_path.exists() {
        manifest_path = extension_dir.join("manifest.json.disable");
    }
    
    if !manifest_path.exists() {
        return Err("扩展清单不存在".to_string());
    }
    
    let content = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let mut val: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    if let Some(obj) = val.as_object_mut() {
        obj.insert("auto_update".to_string(), serde_json::Value::Bool(auto_update));
    }
    
    let new_content = serde_json::to_string_pretty(&val).map_err(|e| e.to_string())?;
    std::fs::write(manifest_path, new_content).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn open_extension_folder(app: tauri::AppHandle, scope: String, version: String) -> Result<(), String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&std::path::PathBuf::from(".")).to_path_buf();
    
    let extensions_dir = if scope == "global" {
        if version.is_empty() {
            return Err("未指定酒馆版本，无法打开全局扩展目录".to_string());
        }
        data_dir.join("sillytavern").join(&version).join("public").join("scripts").join("extensions")
    } else {
        data_dir.join("st_data").join("default-user").join("extensions")
    };
    
    if !extensions_dir.exists() {
        std::fs::create_dir_all(&extensions_dir).map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&extensions_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&extensions_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&extensions_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn open_specific_extension_folder(_app: tauri::AppHandle, dir_path: String) -> Result<(), String> {
    let extension_dir = PathBuf::from(&dir_path);
    
    if !extension_dir.exists() {
        return Err("扩展目录不存在".to_string());
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&extension_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&extension_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&extension_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            ensure_standard_layout(&path)?;
            let app_handle = app.handle().clone();
            apply_saved_window_position(&app_handle);
            setup_window_position_tracking(&app_handle);
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            check_sillytavern_empty,
            fetch_sillytavern_releases,
            get_installed_sillytavern_versions,
            switch_sillytavern_version,
            install_sillytavern_version,
            cancel_install,
            get_app_config,
            save_app_config,
            fetch_github_proxies,
            check_nodejs,
            check_npm,
             install_nodejs,
             delete_sillytavern_version,
             install_sillytavern_dependencies,
            get_installed_versions_info,
            open_directory,
            get_app_version,
            get_tavern_version,
            read_sillytavern_config,
            write_sillytavern_config,
            get_sillytavern_config_path,
            get_sillytavern_config_options,
            update_sillytavern_config_options,
            open_sillytavern_config_file,
            start_sillytavern,
            stop_sillytavern,
            check_sillytavern_status,
            get_extensions,
            toggle_extension_enable,
            delete_extension,
            toggle_extension_auto_update,
            open_extension_folder,
            open_specific_extension_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
