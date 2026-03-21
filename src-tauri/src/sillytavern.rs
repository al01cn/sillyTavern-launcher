use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

use crate::config::{get_current_lang, read_app_config_from_disk, write_app_config_to_disk};
use crate::node::run_npm_install;
use crate::types::{
    DownloadProgress, InstalledVersionInfo, Lang, ProcessState, Release,
    TavernBackupsChatConfig, TavernBackupsCommonConfig, TavernBackupsConfig,
    TavernCorsConfig, TavernConfigPayload, TavernDualStackAddress, TavernDualStackProtocol,
    TavernRequestProxyConfig, TavernBasicAuthUser, TavernThumbnailsConfig,
    TavernThumbnailsDimensionsConfig,
};
use crate::types::InstallState;
use crate::utils::get_config_path;

// ─── GitHub Releases ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn fetch_sillytavern_releases() -> Result<Vec<Release>, String> {
    let client = reqwest::Client::builder()
        .user_agent("sillyTavern-launcher")
        .build()
        .map_err(|e| e.to_string())?;
    let url = "https://api.github.com/repos/SillyTavern/SillyTavern/releases";
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("GitHub API Error: {}", response.status()));
    }
    let releases: Vec<Release> = response.json().await.map_err(|e| e.to_string())?;
    Ok(releases)
}

// ─── 版本列表 ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_installed_sillytavern_versions(app: AppHandle) -> Result<Vec<String>, String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!("获取已安装的酒馆版本列表"),
        Lang::EnUs => tracing::info!("Getting installed SillyTavern version list"),
    }
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let start = std::time::Instant::now();
        let data_dir = get_config_path(&app_clone).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
        let st_dir = data_dir.join("sillytavern");
        if !st_dir.exists() {
            match lang {
                Lang::ZhCn => tracing::info!("酒馆目录不存在，返回空列表, 耗时: {:?}", start.elapsed()),
                Lang::EnUs => tracing::info!("SillyTavern directory not found, elapsed: {:?}", start.elapsed()),
            }
            return Ok(vec![]);
        }
        let mut versions = Vec::new();
        if let Ok(entries) = fs::read_dir(&st_dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        if let Ok(name) = entry.file_name().into_string() {
                            if !name.starts_with('.') { versions.push(name); }
                        }
                    }
                }
            }
        }
        match lang {
            Lang::ZhCn => tracing::info!("找到已安装的版本: {:?}, 耗时: {:?}", versions, start.elapsed()),
            Lang::EnUs => tracing::info!("Found versions: {:?}, elapsed: {:?}", versions, start.elapsed()),
        }
        Ok(versions)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_installed_versions_info(app: AppHandle) -> Result<Vec<InstalledVersionInfo>, String> {
    let lang = get_current_lang(&app);
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let start = std::time::Instant::now();
        let data_dir = get_config_path(&app_clone).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
        let st_dir = data_dir.join("sillytavern");
        if !st_dir.exists() {
            return Ok(vec![]);
        }
        let mut versions = Vec::new();
        if let Ok(entries) = fs::read_dir(&st_dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        if let Ok(name) = entry.file_name().into_string() {
                            if !name.starts_with('.') {
                                let nm = entry.path().join("node_modules");
                                let has_node_modules = nm.exists() && fs::read_dir(&nm).map(|mut d| d.next().is_some()).unwrap_or(false);
                                versions.push(InstalledVersionInfo { version: name, has_node_modules });
                            }
                        }
                    }
                }
            }
        }
        match lang {
            Lang::ZhCn => tracing::info!("获取到版本详细信息, 耗时: {:?}", start.elapsed()),
            Lang::EnUs => tracing::info!("Got version info, elapsed: {:?}", start.elapsed()),
        }
        Ok(versions)
    }).await.map_err(|e| e.to_string())?
}

// ─── 版本切换 ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn switch_sillytavern_version(app: AppHandle, version: String) -> Result<(), String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!("切换酒馆版本到: {}", version),
        Lang::EnUs => tracing::info!("Switching version to: {}", version),
    }
    let mut config = read_app_config_from_disk(&app);
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let version_dir = data_dir.join("sillytavern").join(&version);
    if !version_dir.exists() {
        match lang {
            Lang::ZhCn => { tracing::error!("版本 {} 不存在", version); return Err(format!("版本 {} 不存在", version)); }
            Lang::EnUs => { tracing::error!("Version {} not found", version); return Err(format!("Version {} not found", version)); }
        }
    }
    config.sillytavern.version = version;
    write_app_config_to_disk(&app, &config)
}

// ─── 取消安装 ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn cancel_install(state: tauri::State<'_, InstallState>) {
    state.cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
}

// ─── 安装版本 ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn install_sillytavern_version(
    app: AppHandle,
    state: tauri::State<'_, InstallState>,
    version: String,
    url: String,
) -> Result<(), String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!("开始安装酒馆版本: {}，URL: {}", version, url),
        Lang::EnUs => tracing::info!("Installing version: {}, URL: {}", version, url),
    }
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let st_dir = data_dir.join("sillytavern").join(&version);

    if st_dir.exists() {
        match lang {
            Lang::ZhCn => tracing::info!("版本 {} 已存在，跳过安装", version),
            Lang::EnUs => tracing::info!("Version {} already exists, skipping", version),
        }
        return Ok(());
    }

    fs::create_dir_all(&st_dir).map_err(|e| { match lang { Lang::ZhCn => tracing::error!("创建目录失败: {}", e), Lang::EnUs => tracing::error!("Failed to create dir: {}", e) } e.to_string() })?;
    state.cancel_flag.store(false, std::sync::atomic::Ordering::Relaxed);

    let emit = |status: &str, progress: f64, log: &str| {
        let _ = app.emit("install-progress", DownloadProgress { status: status.to_string(), progress, log: log.to_string() });
    };

    emit("downloading", 0.0, &match lang { Lang::ZhCn => format!("准备下载版本 {}...", version), Lang::EnUs => format!("Preparing to download version {}...", version) });

    let temp_zip = std::env::temp_dir().join(format!("sillytavern_{}.zip", version));
    let client = reqwest::Client::builder().user_agent("sillyTavern-launcher").build().map_err(|e| e.to_string())?;
    let response = client.get(&url).send().await.map_err(|e| { match lang { Lang::ZhCn => tracing::error!("请求下载失败: {}", e), Lang::EnUs => tracing::error!("Download failed: {}", e) } e.to_string() })?;
    let total_size = response.content_length().unwrap_or(0);

    let mut file = tokio::fs::File::create(&temp_zip).await.map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut last_emit = std::time::Instant::now();

    while let Some(item) = stream.next().await {
        if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = tokio::fs::remove_file(&temp_zip).await;
            let _ = tokio::fs::remove_dir_all(&st_dir).await;
            emit("error", 0.0, match lang { Lang::ZhCn => "下载已取消", Lang::EnUs => "Download cancelled" });
            return Err(match lang { Lang::ZhCn => "下载已取消".to_string(), Lang::EnUs => "Download cancelled".to_string() });
        }
        let chunk = item.map_err(|e| e.to_string())?;
        use tokio::io::AsyncWriteExt;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        if last_emit.elapsed() > std::time::Duration::from_millis(200) || downloaded == total_size {
            let progress = if total_size > 0 { downloaded as f64 / total_size as f64 } else { 0.0 };
            emit("downloading", progress, &match lang { Lang::ZhCn => format!("已下载: {:.2} MB", downloaded as f64 / 1_048_576.0), Lang::EnUs => format!("Downloaded: {:.2} MB", downloaded as f64 / 1_048_576.0) });
            last_emit = std::time::Instant::now();
        }
    }

    emit("extracting", 0.0, match lang { Lang::ZhCn => "下载完成，准备解压...", Lang::EnUs => "Download complete, extracting..." });

    let cancel_flag = state.cancel_flag.clone();
    let app_clone = app.clone();
    let temp_zip_clone = temp_zip.clone();
    let st_dir_clone = st_dir.clone();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let emit2 = |status: &str, progress: f64, log: &str| {
            let _ = app_clone.emit("install-progress", DownloadProgress { status: status.to_string(), progress, log: log.to_string() });
        };
        let file = fs::File::open(&temp_zip_clone).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let total = archive.len();
        for i in 0..total {
            if i % 10 == 0 && cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = fs::remove_file(&temp_zip_clone);
                let _ = fs::remove_dir_all(&st_dir_clone);
                emit2("error", 0.0, match lang { Lang::ZhCn => "解压已取消", Lang::EnUs => "Extraction cancelled" });
                return Err(match lang { Lang::ZhCn => "解压已取消".to_string(), Lang::EnUs => "Extraction cancelled".to_string() });
            }
            let mut f = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match f.enclosed_name() { Some(p) => p.to_owned(), None => continue };
            let mut comps = outpath.components(); comps.next();
            let stripped: PathBuf = comps.collect();
            if stripped.as_os_str().is_empty() { continue; }
            let target = st_dir_clone.join(&stripped);
            if (*f.name()).ends_with('/') {
                fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = target.parent() { if !p.exists() { fs::create_dir_all(p).map_err(|e| e.to_string())?; } }
                let mut out = fs::File::create(&target).map_err(|e| e.to_string())?;
                io::copy(&mut f, &mut out).map_err(|e| e.to_string())?;
            }
            if i % 500 == 0 || i == total - 1 {
                emit2("extracting", i as f64 / total as f64, &match lang { Lang::ZhCn => format!("解压中: {}/{} 文件...", i + 1, total), Lang::EnUs => format!("Extracting: {}/{} files...", i + 1, total) });
            }
        }
        Ok(())
    }).await.map_err(|e| e.to_string())??;

    let _ = fs::remove_file(&temp_zip);

    emit("installing", 0.0, match lang { Lang::ZhCn => "正在安装依赖 (npm install)... 这可能需要几分钟", Lang::EnUs => "Installing dependencies (npm install)... this may take a few minutes" });

    let app2 = app.clone();
    let st_dir2 = st_dir.clone();
    tokio::spawn(async move {
        if let Err(e) = run_npm_install(&app2, &st_dir2).await {
            let _ = app2.emit("install-progress", DownloadProgress { status: "error".to_string(), progress: 0.0, log: match lang { Lang::ZhCn => format!("安装依赖失败: {}", e), Lang::EnUs => format!("Failed to install dependencies: {}", e) } });
        } else {
            let _ = app2.emit("install-progress", DownloadProgress { status: "done".to_string(), progress: 1.0, log: match lang { Lang::ZhCn => "安装完成！".to_string(), Lang::EnUs => "Installation complete!".to_string() } });
        }
    });

    Ok(())
}

// ─── 单独安装依赖 ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn install_sillytavern_dependencies(app: AppHandle, version: String) -> Result<(), String> {
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let st_dir = data_dir.join("sillytavern").join(&version);
    if !st_dir.exists() { return Err(format!("Version {} not found", version)); }
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::spawn(async move {
        if let Err(e) = run_npm_install(&app2, &st_dir).await {
            let _ = app2.emit("install-progress", DownloadProgress { status: "error".to_string(), progress: 0.0, log: match lang { Lang::ZhCn => format!("安装依赖失败: {}", e), Lang::EnUs => format!("Failed to install dependencies: {}", e) } });
        } else {
            let _ = app2.emit("install-progress", DownloadProgress { status: "done".to_string(), progress: 1.0, log: match lang { Lang::ZhCn => "依赖安装完成！".to_string(), Lang::EnUs => "Dependency installation complete!".to_string() } });
        }
    });
    Ok(())
}

// ─── 删除版本 ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn delete_sillytavern_version(app: AppHandle, version: String) -> Result<(), String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!("准备删除酒馆版本: {}", version),
        Lang::EnUs => tracing::info!("Deleting version: {}", version),
    }
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let version_dir = data_dir.join("sillytavern").join(&version);

    if !version_dir.exists() {
        return match lang {
            Lang::ZhCn => Err(format!("版本 {} 不存在", version)),
            Lang::EnUs => Err(format!("Version {} not found", version)),
        };
    }
    if version.trim().is_empty() || version.contains("..") || version.contains('/') || version.contains('\\') {
        return match lang {
            Lang::ZhCn => Err("无效的版本号".to_string()),
            Lang::EnUs => Err("Invalid version number".to_string()),
        };
    }

    let app2 = app.clone();
    let vdir = version_dir.clone();
    let vc = version.clone();

    let result = tokio::task::spawn_blocking(move || {
        let _ = app2.emit("install-progress", DownloadProgress { status: "deleting".to_string(), progress: 0.1, log: match lang { Lang::ZhCn => format!("开始删除版本 {}...", vc), Lang::EnUs => format!("Deleting version {}...", vc) } });
        std::thread::sleep(std::time::Duration::from_millis(100));

        let mut samples = Vec::new();
        if let Ok(entries) = fs::read_dir(&vdir) {
            for e in entries.flatten() { if let Ok(n) = e.file_name().into_string() { samples.push(n); } }
        }
        let total = samples.len();
        for (i, name) in samples.iter().enumerate() {
            std::thread::sleep(std::time::Duration::from_millis(15));
            let _ = app2.emit("install-progress", DownloadProgress { status: "deleting".to_string(), progress: 0.3 + 0.5 * (i as f64 / total as f64), log: match lang { Lang::ZhCn => format!("已删除：{}/{}", vc, name), Lang::EnUs => format!("Deleted: {}/{}", vc, name) } });
        }

        fn fast_remove(dir: &Path) -> io::Result<()> {
            if dir.is_dir() {
                if let Ok(entries) = fs::read_dir(dir) {
                    for e in entries.flatten() {
                        let p = e.path();
                        if let Ok(ft) = e.file_type() {
                            if ft.is_dir() { let _ = fast_remove(&p); }
                            else if fs::remove_file(&p).is_err() {
                                #[cfg(target_os = "windows")] {
                                    if let Ok(mut perms) = fs::metadata(&p).map(|m| m.permissions()) {
                                        if perms.readonly() { perms.set_readonly(false); let _ = fs::set_permissions(&p, perms); let _ = fs::remove_file(&p); }
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
        let _ = fast_remove(&vdir);
        if vdir.exists() { fs::remove_dir_all(&vdir)?; }
        let _ = app2.emit("install-progress", DownloadProgress { status: "deleting".to_string(), progress: 1.0, log: match lang { Lang::ZhCn => format!("版本 {} 已全部删除", vc), Lang::EnUs => format!("Version {} deleted", vc) } });
        Ok::<(), io::Error>(())
    }).await;

    match result {
        Ok(Ok(_)) => { match lang { Lang::ZhCn => tracing::info!("版本 {} 删除成功", version), Lang::EnUs => tracing::info!("Version {} deleted", version) } Ok(()) }
        Ok(Err(e)) => { match lang { Lang::ZhCn => Err(format!("删除失败: {}", e)), Lang::EnUs => Err(format!("Deletion failed: {}", e)) } }
        Err(e) => { match lang { Lang::ZhCn => Err(format!("任务执行失败: {}", e)), Lang::EnUs => Err(format!("Task failed: {}", e)) } }
    }
}

// ─── 检查 ST 是否为空 ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn check_sillytavern_empty(app: AppHandle) -> Result<bool, String> {
    let lang = get_current_lang(&app);
    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let st_dir = data_dir.join("sillytavern");
    if !st_dir.exists() { return Ok(true); }
    let entries = match fs::read_dir(&st_dir) { Ok(e) => e, Err(_) => return Ok(true) };
    let mut has_valid = false;
    for entry in entries {
        if let Ok(entry) = entry {
            let n = entry.file_name();
            let s = n.to_string_lossy();
            if s != ".gitkeep" && s != ".DS_Store" { has_valid = true; break; }
        }
    }
    match lang {
        Lang::ZhCn => tracing::info!("酒馆目录检查结果: isEmpty={}", !has_valid),
        Lang::EnUs => tracing::info!("SillyTavern directory isEmpty={}", !has_valid),
    }
    Ok(!has_valid)
}

// ─── ST 当前版本 ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_tavern_version(app: AppHandle) -> Result<String, String> {
    let _lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let config = read_app_config_from_disk(&app2);
        let ver = config.sillytavern.version;
        if ver.is_empty() { return Err("未设置".to_string()); }
        let data_dir = get_config_path(&app2).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
        let ver_dir = data_dir.join("sillytavern").join(&ver);
        if !ver_dir.exists() { return Err("未安装".to_string()); }
        let pkg = ver_dir.join("package.json");
        if pkg.exists() {
            if let Ok(content) = fs::read_to_string(&pkg) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(v) = parsed.get("version").and_then(|v| v.as_str()) {
                        return Ok(v.to_string());
                    }
                }
            }
        }
        Ok(ver)
    }).await.map_err(|e| e.to_string())?
}

// ─── ST 配置文件路径 ────────────────────────────────────────────────────────────

fn get_st_config_path(app: &AppHandle, version: &str, use_global: bool) -> Result<PathBuf, String> {
    if version.trim().is_empty() { return Err("版本号不能为空".to_string()); }
    let data_dir = get_config_path(app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let config_path = if use_global {
        let st_data = data_dir.join("st_data");
        if !st_data.exists() { std::fs::create_dir_all(&st_data).map_err(|e| format!("无法创建全局数据目录: {}", e))?; }
        let global = st_data.join("config.yaml");
        if !global.exists() && !version.contains("..") && !version.contains('/') {
            let ver_cfg = data_dir.join("sillytavern").join(version).join("config.yaml");
            if ver_cfg.exists() { let _ = std::fs::copy(&ver_cfg, &global); }
        }
        global
    } else {
        if version.contains("..") || version.contains('/') || version.contains('\\') { return Err("版本号不合法".to_string()); }
        data_dir.join("sillytavern").join(version).join("config.yaml")
    };
    if !config_path.exists() { return Err(format!("配置文件不存在: {:?}", config_path)); }
    Ok(config_path)
}

// ─── ST Config YAML 读写 ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn read_sillytavern_config(app: AppHandle, version: String) -> Result<String, String> {
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = read_app_config_from_disk(&app2);
        let path = get_st_config_path(&app2, &version, cfg.sillytavern.use_global_config)?;
        fs::read_to_string(&path).map_err(|e| match lang { Lang::ZhCn => format!("读取失败: {}", e), Lang::EnUs => format!("Read failed: {}", e) })
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn write_sillytavern_config(app: AppHandle, version: String, content: String) -> Result<(), String> {
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = read_app_config_from_disk(&app2);
        let path = get_st_config_path(&app2, &version, cfg.sillytavern.use_global_config)?;
        fs::write(&path, content).map_err(|e| match lang { Lang::ZhCn => format!("写入失败: {}", e), Lang::EnUs => format!("Write failed: {}", e) })
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn get_sillytavern_config_path(app: AppHandle, version: String) -> Result<String, String> {
    let cfg = read_app_config_from_disk(&app);
    let path = get_st_config_path(&app, &version, cfg.sillytavern.use_global_config)?;
    Ok(path.to_string_lossy().to_string())
}

// ─── ST 高级配置解析 ────────────────────────────────────────────────────────────

fn parse_tavern_config_payload(yaml_str: &str) -> Result<TavernConfigPayload, String> {
    let root: serde_yaml::Value = serde_yaml::from_str(yaml_str).map_err(|e| format!("解析配置失败: {}", e))?;
    let mapping = root.as_mapping().ok_or("配置文件格式无效，根节点必须是对象".to_string())?;

    let get_bool = |key: &str, default: bool| mapping.get(serde_yaml::Value::String(key.to_string())).and_then(serde_yaml::Value::as_bool).unwrap_or(default);
    let get_i64  = |key: &str, default: i64 | mapping.get(serde_yaml::Value::String(key.to_string())).and_then(serde_yaml::Value::as_i64).unwrap_or(default);
    let parse_str_seq = |value: Option<&serde_yaml::Value>, default: Vec<String>| -> Vec<String> {
        value.and_then(serde_yaml::Value::as_sequence).map(|seq| seq.iter().filter_map(serde_yaml::Value::as_str).map(|s| s.to_string()).collect()).unwrap_or(default)
    };
    let parse_i64_seq = |value: Option<&serde_yaml::Value>, default: Vec<i64>| -> Vec<i64> {
        value.and_then(serde_yaml::Value::as_sequence).map(|seq| seq.iter().filter_map(serde_yaml::Value::as_i64).collect::<Vec<_>>()).filter(|s| !s.is_empty()).unwrap_or(default)
    };
    let key = |s: &str| serde_yaml::Value::String(s.to_string());
    let sub = |k: &str| mapping.get(serde_yaml::Value::String(k.to_string())).and_then(serde_yaml::Value::as_mapping);

    let listen_address = sub("listenAddress").map(|m| TavernDualStackAddress {
        ipv4: m.get(key("ipv4")).and_then(serde_yaml::Value::as_str).unwrap_or("0.0.0.0").to_string(),
        ipv6: m.get(key("ipv6")).and_then(serde_yaml::Value::as_str).unwrap_or("[::]").to_string(),
    }).unwrap_or(TavernDualStackAddress { ipv4: "0.0.0.0".to_string(), ipv6: "[::]".to_string() });

    let protocol = sub("protocol").map(|m| TavernDualStackProtocol {
        ipv4: m.get(key("ipv4")).and_then(serde_yaml::Value::as_bool).unwrap_or(true),
        ipv6: m.get(key("ipv6")).and_then(serde_yaml::Value::as_bool).unwrap_or(false),
    }).unwrap_or(TavernDualStackProtocol { ipv4: true, ipv6: false });

    let whitelist = parse_str_seq(mapping.get(key("whitelist")), vec!["::1".to_string(), "127.0.0.1".to_string()]);

    let basic_auth_user = sub("basicAuthUser").map(|m| TavernBasicAuthUser {
        username: m.get(key("username")).and_then(serde_yaml::Value::as_str).unwrap_or("user").to_string(),
        password: m.get(key("password")).and_then(serde_yaml::Value::as_str).unwrap_or("password").to_string(),
    }).unwrap_or(TavernBasicAuthUser { username: "user".to_string(), password: "password".to_string() });

    let cors = sub("cors").map(|m| TavernCorsConfig {
        enabled: m.get(key("enabled")).and_then(serde_yaml::Value::as_bool).unwrap_or(true),
        origin: parse_str_seq(m.get(key("origin")), vec!["null".to_string()]),
        methods: parse_str_seq(m.get(key("methods")), vec!["OPTIONS".to_string()]),
        allowed_headers: parse_str_seq(m.get(key("allowedHeaders")), vec![]),
        exposed_headers: parse_str_seq(m.get(key("exposedHeaders")), vec![]),
        credentials: m.get(key("credentials")).and_then(serde_yaml::Value::as_bool).unwrap_or(false),
        max_age: m.get(key("maxAge")).and_then(serde_yaml::Value::as_i64),
    }).unwrap_or(TavernCorsConfig { enabled: true, origin: vec!["null".to_string()], methods: vec!["OPTIONS".to_string()], allowed_headers: vec![], exposed_headers: vec![], credentials: false, max_age: None });

    let request_proxy = sub("requestProxy").map(|m| TavernRequestProxyConfig {
        enabled: m.get(key("enabled")).and_then(serde_yaml::Value::as_bool).unwrap_or(false),
        url: m.get(key("url")).and_then(serde_yaml::Value::as_str).unwrap_or("").to_string(),
        bypass: m.get(key("bypass")).and_then(serde_yaml::Value::as_sequence).map(|s| s.iter().filter_map(serde_yaml::Value::as_str).map(|s| s.to_string()).collect()).unwrap_or_default(),
    }).unwrap_or(TavernRequestProxyConfig { enabled: false, url: "".to_string(), bypass: vec![] });

    let backups = sub("backups").map(|item| {
        let common = item.get(key("common")).and_then(serde_yaml::Value::as_mapping);
        let chat = item.get(key("chat")).and_then(serde_yaml::Value::as_mapping);
        TavernBackupsConfig {
            common: TavernBackupsCommonConfig { number_of_backups: common.and_then(|x| x.get(key("numberOfBackups")).and_then(serde_yaml::Value::as_i64)).unwrap_or(50) },
            chat: TavernBackupsChatConfig {
                enabled: chat.and_then(|x| x.get(key("enabled")).and_then(serde_yaml::Value::as_bool)).unwrap_or(true),
                check_integrity: chat.and_then(|x| x.get(key("checkIntegrity")).and_then(serde_yaml::Value::as_bool)).unwrap_or(true),
                max_total_backups: chat.and_then(|x| x.get(key("maxTotalBackups")).and_then(serde_yaml::Value::as_i64)).unwrap_or(-1),
                throttle_interval: chat.and_then(|x| x.get(key("throttleInterval")).and_then(serde_yaml::Value::as_i64)).unwrap_or(10000),
            },
        }
    }).unwrap_or(TavernBackupsConfig { common: TavernBackupsCommonConfig { number_of_backups: 50 }, chat: TavernBackupsChatConfig { enabled: true, check_integrity: true, max_total_backups: -1, throttle_interval: 10000 } });

    let thumbnails = sub("thumbnails").map(|item| {
        let dims = item.get(key("dimensions")).and_then(serde_yaml::Value::as_mapping);
        TavernThumbnailsConfig {
            enabled: item.get(key("enabled")).and_then(serde_yaml::Value::as_bool).unwrap_or(true),
            format: item.get(key("format")).and_then(serde_yaml::Value::as_str).unwrap_or("jpg").to_string(),
            quality: item.get(key("quality")).and_then(serde_yaml::Value::as_i64).unwrap_or(95),
            dimensions: TavernThumbnailsDimensionsConfig {
                bg: parse_i64_seq(dims.and_then(|x| x.get(key("bg"))), vec![160, 90]),
                avatar: parse_i64_seq(dims.and_then(|x| x.get(key("avatar"))), vec![96, 144]),
                persona: parse_i64_seq(dims.and_then(|x| x.get(key("persona"))), vec![96, 144]),
            },
        }
    }).unwrap_or(TavernThumbnailsConfig { enabled: true, format: "jpg".to_string(), quality: 95, dimensions: TavernThumbnailsDimensionsConfig { bg: vec![160, 90], avatar: vec![96, 144], persona: vec![96, 144] } });

    let (browser_launch_enabled, browser_type) = sub("browserLaunch").map(|m| {
        let e = m.get(key("enabled")).and_then(serde_yaml::Value::as_bool).unwrap_or(true);
        let b = m.get(key("browser")).and_then(serde_yaml::Value::as_str).unwrap_or("default").to_string();
        (e, b)
    }).unwrap_or((true, "default".to_string()));

    Ok(TavernConfigPayload { port: get_i64("port", 8000), listen: get_bool("listen", false), listen_address, protocol, basic_auth_mode: get_bool("basicAuthMode", false), enable_user_accounts: get_bool("enableUserAccounts", false), enable_discreet_login: get_bool("enableDiscreetLogin", false), per_user_basic_auth: get_bool("perUserBasicAuth", false), basic_auth_user, whitelist_mode: get_bool("whitelistMode", true), whitelist, cors, request_proxy, backups, thumbnails, browser_launch_enabled, browser_type })
}

fn upsert(m: &mut serde_yaml::Mapping, k: &str, v: serde_yaml::Value) {
    m.insert(serde_yaml::Value::String(k.to_string()), v);
}

fn child_map<'a>(m: &'a mut serde_yaml::Mapping, k: &str) -> Result<&'a mut serde_yaml::Mapping, String> {
    let ck = serde_yaml::Value::String(k.to_string());
    if !m.contains_key(&ck) { m.insert(ck.clone(), serde_yaml::Value::Mapping(serde_yaml::Mapping::new())); }
    m.get_mut(&ck).and_then(serde_yaml::Value::as_mapping_mut).ok_or(format!("{} 配置格式无效", k))
}

#[tauri::command]
pub async fn get_sillytavern_config_options(app: AppHandle, version: String) -> Result<TavernConfigPayload, String> {
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = read_app_config_from_disk(&app2);
        let path = get_st_config_path(&app2, &version, cfg.sillytavern.use_global_config)?;
        let content = fs::read_to_string(&path).map_err(|e| match lang { Lang::ZhCn => format!("读取失败: {}", e), Lang::EnUs => format!("Read failed: {}", e) })?;
        parse_tavern_config_payload(&content)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_sillytavern_config_options(app: AppHandle, version: String, config: TavernConfigPayload) -> Result<TavernConfigPayload, String> {
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = read_app_config_from_disk(&app2);
        let path = get_st_config_path(&app2, &version, cfg.sillytavern.use_global_config)?;
        let content = fs::read_to_string(&path).map_err(|e| match lang { Lang::ZhCn => format!("读取失败: {}", e), Lang::EnUs => format!("Read failed: {}", e) })?;
        let mut root: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| match lang { Lang::ZhCn => format!("解析配置失败: {}", e), Lang::EnUs => format!("Parse failed: {}", e) })?;
        let m = root.as_mapping_mut().ok_or("配置文件格式无效，根节点必须是对象".to_string())?;

        upsert(m, "port", serde_yaml::Value::Number(serde_yaml::Number::from(config.port)));
        upsert(m, "listen", serde_yaml::Value::Bool(config.listen));
        { let la = child_map(m, "listenAddress")?; upsert(la, "ipv4", serde_yaml::Value::String(config.listen_address.ipv4.clone())); upsert(la, "ipv6", serde_yaml::Value::String(config.listen_address.ipv6.clone())); }
        { let p = child_map(m, "protocol")?; upsert(p, "ipv4", serde_yaml::Value::Bool(config.protocol.ipv4)); upsert(p, "ipv6", serde_yaml::Value::Bool(config.protocol.ipv6)); }
        upsert(m, "basicAuthMode", serde_yaml::Value::Bool(config.basic_auth_mode));
        upsert(m, "enableUserAccounts", serde_yaml::Value::Bool(config.enable_user_accounts));
        upsert(m, "enableDiscreetLogin", serde_yaml::Value::Bool(config.enable_discreet_login));
        upsert(m, "perUserBasicAuth", serde_yaml::Value::Bool(config.per_user_basic_auth));
        { let bau = child_map(m, "basicAuthUser")?; upsert(bau, "username", serde_yaml::Value::String(config.basic_auth_user.username.clone())); upsert(bau, "password", serde_yaml::Value::String(config.basic_auth_user.password.clone())); }
        upsert(m, "whitelistMode", serde_yaml::Value::Bool(config.whitelist_mode));
        upsert(m, "whitelist", serde_yaml::Value::Sequence(config.whitelist.iter().map(|s| serde_yaml::Value::String(s.clone())).collect()));
        { let c = child_map(m, "cors")?; upsert(c, "enabled", serde_yaml::Value::Bool(config.cors.enabled)); upsert(c, "origin", serde_yaml::Value::Sequence(config.cors.origin.iter().map(|s| serde_yaml::Value::String(s.clone())).collect())); upsert(c, "methods", serde_yaml::Value::Sequence(config.cors.methods.iter().map(|s| serde_yaml::Value::String(s.clone())).collect())); upsert(c, "allowedHeaders", serde_yaml::Value::Sequence(config.cors.allowed_headers.iter().map(|s| serde_yaml::Value::String(s.clone())).collect())); upsert(c, "exposedHeaders", serde_yaml::Value::Sequence(config.cors.exposed_headers.iter().map(|s| serde_yaml::Value::String(s.clone())).collect())); upsert(c, "credentials", serde_yaml::Value::Bool(config.cors.credentials)); upsert(c, "maxAge", config.cors.max_age.map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(v))).unwrap_or(serde_yaml::Value::Null)); }
        { let rp = child_map(m, "requestProxy")?; upsert(rp, "enabled", serde_yaml::Value::Bool(config.request_proxy.enabled)); upsert(rp, "url", serde_yaml::Value::String(config.request_proxy.url.clone())); upsert(rp, "bypass", serde_yaml::Value::Sequence(config.request_proxy.bypass.iter().map(|s| serde_yaml::Value::String(s.clone())).collect())); }
        { let bk = child_map(m, "backups")?; { let bc = child_map(bk, "common")?; upsert(bc, "numberOfBackups", serde_yaml::Value::Number(serde_yaml::Number::from(config.backups.common.number_of_backups))); } { let bch = child_map(bk, "chat")?; upsert(bch, "enabled", serde_yaml::Value::Bool(config.backups.chat.enabled)); upsert(bch, "checkIntegrity", serde_yaml::Value::Bool(config.backups.chat.check_integrity)); upsert(bch, "maxTotalBackups", serde_yaml::Value::Number(serde_yaml::Number::from(config.backups.chat.max_total_backups))); upsert(bch, "throttleInterval", serde_yaml::Value::Number(serde_yaml::Number::from(config.backups.chat.throttle_interval))); } }
        { let th = child_map(m, "thumbnails")?; upsert(th, "enabled", serde_yaml::Value::Bool(config.thumbnails.enabled)); upsert(th, "format", serde_yaml::Value::String(config.thumbnails.format.clone())); upsert(th, "quality", serde_yaml::Value::Number(serde_yaml::Number::from(config.thumbnails.quality))); { let d = child_map(th, "dimensions")?; upsert(d, "bg", serde_yaml::Value::Sequence(config.thumbnails.dimensions.bg.iter().map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(*v))).collect())); upsert(d, "avatar", serde_yaml::Value::Sequence(config.thumbnails.dimensions.avatar.iter().map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(*v))).collect())); upsert(d, "persona", serde_yaml::Value::Sequence(config.thumbnails.dimensions.persona.iter().map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(*v))).collect())); } }
        { let bl = child_map(m, "browserLaunch")?; upsert(bl, "enabled", serde_yaml::Value::Bool(config.browser_launch_enabled)); upsert(bl, "browser", serde_yaml::Value::String(config.browser_type.clone())); }

        let new_content = serde_yaml::to_string(&root).map_err(|e| match lang { Lang::ZhCn => format!("序列化配置失败: {}", e), Lang::EnUs => format!("Serialize failed: {}", e) })?;
        fs::write(&path, new_content).map_err(|e| match lang { Lang::ZhCn => format!("写入失败: {}", e), Lang::EnUs => format!("Write failed: {}", e) })?;
        Ok(config)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn open_sillytavern_config_file(app: AppHandle, version: String) -> Result<(), String> {
    let cfg = read_app_config_from_disk(&app);
    let path = get_st_config_path(&app, &version, cfg.sillytavern.use_global_config)?;
    #[cfg(target_os = "windows")] { let mut cmd = std::process::Command::new("explorer"); cmd.arg(path); use std::os::windows::process::CommandExt; cmd.creation_flags(0x08000000); cmd.spawn().map_err(|e| format!("打开失败: {}", e))?; }
    #[cfg(target_os = "macos")] { std::process::Command::new("open").arg(path).spawn().map_err(|e| format!("打开失败: {}", e))?; }
    #[cfg(target_os = "linux")] { std::process::Command::new("xdg-open").arg(path).spawn().map_err(|e| format!("打开失败: {}", e))?; }
    Ok(())
}

// ─── 启动 / 停止 / 状态 ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_sillytavern(app: AppHandle, state: tauri::State<'_, ProcessState>) -> Result<(), String> {
    let lang = get_current_lang(&app);
    let mut kill_tx_guard = state.kill_tx.lock().await;
    if kill_tx_guard.is_some() {
        return match lang { Lang::ZhCn => Err("进程已经在运行中了".to_string()), Lang::EnUs => Err("Process is already running".to_string()) };
    }

    let config = read_app_config_from_disk(&app);
    let version = config.sillytavern.version;
    if version.is_empty() {
        return match lang { Lang::ZhCn => Err("未选择酒馆版本，请先在版本页面选择或安装".to_string()), Lang::EnUs => Err("No version selected".to_string()) };
    }

    let data_dir = get_config_path(&app).parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    let st_dir = data_dir.join("sillytavern").join(&version);
    let st_data = data_dir.join("st_data");

    if !st_data.exists() { std::fs::create_dir_all(&st_data).map_err(|e| format!("无法创建全局数据目录: {}", e))?; }
    if !st_dir.exists() { return match lang { Lang::ZhCn => Err(format!("版本 {} 的目录不存在", version)), Lang::EnUs => Err(format!("Directory for version {} not found", version)) }; }

    let mut node_path = if cfg!(target_os = "windows") { data_dir.join("node").join("node.exe") } else { data_dir.join("node").join("bin/node") };
    if !node_path.exists() { node_path = PathBuf::from("node"); }

    let server_js = st_dir.join("server.js");
    if !server_js.exists() { return match lang { Lang::ZhCn => Err("找不到 server.js，酒馆文件可能损坏".to_string()), Lang::EnUs => Err("server.js not found".to_string()) }; }

    let mut std_cmd = std::process::Command::new(&node_path);
    std_cmd.arg(&server_js);
    let st_data_str = st_data.to_string_lossy().to_string();
    std_cmd.arg("--dataRoot").arg(&st_data_str);

    if config.sillytavern.use_global_config {
        let global_cfg = st_data.join("config.yaml");
        if global_cfg.exists() { std_cmd.arg("--config").arg(global_cfg.to_string_lossy().as_ref()); }
    }

    std_cmd.current_dir(&st_dir)
        .env("SILLYTAVERN_DATA_DIR", &st_data_str)
        .env("SillyTavern_DATA_DIR", &st_data_str)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")] { use std::os::windows::process::CommandExt; std_cmd.creation_flags(0x08000000); }

    let mut cmd = tokio::process::Command::from(std_cmd);
    let mut child = cmd.spawn().map_err(|e| match lang { Lang::ZhCn => format!("启动进程失败: {}", e), Lang::EnUs => format!("Failed to start: {}", e) })?;

    let stdout = child.stdout.take().ok_or("无法获取标准输出")?;
    let stderr = child.stderr.take().ok_or("无法获取标准错误")?;

    let app1 = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            tracing::info!("ST_STDOUT: {}", line);
            let _ = app1.emit("process-log", format!("INFO: {}", line));
        }
    });

    let app2 = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            tracing::error!("ST_STDERR: {}", line);
            let _ = app2.emit("process-log", format!("ERROR: {}", line));
        }
    });

    let (kill_tx, mut kill_rx) = tokio::sync::mpsc::channel::<()>(1);
    *kill_tx_guard = Some(kill_tx);

    let app3 = app.clone();
    let kill_tx_arc = state.inner().kill_tx.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = child.wait() => { let _ = app3.emit("process-log", "INFO: 进程已退出".to_string()); }
            _ = kill_rx.recv() => { let _ = child.kill().await; let _ = app3.emit("process-log", "INFO: 进程已被终止".to_string()); }
        }
        *kill_tx_arc.lock().await = None;
        let _ = app3.emit("process-exit", ());
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_sillytavern(state: tauri::State<'_, ProcessState>) -> Result<(), String> {
    tracing::info!("尝试停止酒馆...");
    let mut guard = state.kill_tx.lock().await;
    if let Some(tx) = guard.take() { let _ = tx.send(()).await; }
    Ok(())
}

#[tauri::command]
pub async fn check_sillytavern_status(state: tauri::State<'_, ProcessState>) -> Result<bool, String> {
    Ok(state.kill_tx.lock().await.is_some())
}
