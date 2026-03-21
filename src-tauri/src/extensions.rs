use std::path::PathBuf;

use crate::types::{ExtensionInfo, ExtensionManifest};
use crate::utils::get_config_path;

// ─────────────────────────────────────────────
// Tauri commands
// ─────────────────────────────────────────────

#[tauri::command]
pub fn verify_extension_zip(zip_path: String) -> Result<ExtensionManifest, String> {
    let file = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();

        if name == "manifest.json" || name.ends_with("/manifest.json") {
            let mut contents = String::new();
            std::io::Read::read_to_string(&mut file, &mut contents).map_err(|e| e.to_string())?;

            let manifest: ExtensionManifest = serde_json::from_str(&contents)
                .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;
            return Ok(manifest);
        }
    }

    Err("未在压缩包中找到 manifest.json 文件，这不是一个有效的扩展".to_string())
}

#[tauri::command]
pub fn install_extension_zip(
    app: tauri::AppHandle,
    zip_path: String,
    scope: String,
    version: String,
) -> Result<(), String> {
    tracing::info!(
        "开始安装扩展, zip_path: {}, scope: {}, version: {}",
        zip_path,
        scope,
        version
    );
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&std::path::PathBuf::from("."))
        .to_path_buf();

    let target_dir = if scope == "user" {
        data_dir
            .join("st_data")
            .join("default-user")
            .join("extensions")
    } else {
        if version.is_empty() {
            return Err("未指定酒馆版本，无法安装全局扩展".to_string());
        }
        data_dir
            .join("sillytavern")
            .join(&version)
            .join("public")
            .join("scripts")
            .join("extensions")
            .join("third-party")
    };

    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    }

    let file = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    // 检测是否有公共根目录
    let mut root_dir: Option<String> = None;
    let mut single_root = true;

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        let first_component = name.split('/').next().unwrap_or("").to_string();

        if root_dir.is_none() {
            root_dir = Some(first_component.clone());
        } else if root_dir.as_ref().unwrap() != &first_component {
            single_root = false;
            break;
        }
    }

    let file_stem = std::path::Path::new(&zip_path)
        .file_stem()
        .unwrap_or(std::ffi::OsStr::new("extension"))
        .to_string_lossy()
        .to_string();

    let extract_target = if single_root {
        target_dir.clone()
    } else {
        target_dir.join(&file_stem)
    };

    if !extract_target.exists() {
        std::fs::create_dir_all(&extract_target).map_err(|e| e.to_string())?;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let target_path = extract_target.join(&outpath);

        if (*file.name()).ends_with('/') {
            std::fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = target_path.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = std::fs::File::create(&target_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_extensions(
    app: tauri::AppHandle,
    version: String,
) -> Result<Vec<ExtensionInfo>, String> {
    tracing::info!("获取扩展列表，当前酒馆版本: {}", version);
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let start_time = std::time::Instant::now();
        let data_dir = get_config_path(&app_clone)
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .to_path_buf();
        let mut extensions = Vec::new();

        let scan_dir = |dir_path: &PathBuf,
                        is_system: bool,
                        scope: &str,
                        exts: &mut Vec<ExtensionInfo>| {
            if !dir_path.exists() {
                return;
            }
            if let Ok(entries) = std::fs::read_dir(dir_path) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            if is_system && entry.file_name() == "third-party" {
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
                                    if let Ok(manifest) =
                                        serde_json::from_str::<ExtensionManifest>(&content)
                                    {
                                        exts.push(ExtensionInfo {
                                            id: entry.file_name().to_string_lossy().to_string(),
                                            manifest,
                                            dir_path: entry.path().to_string_lossy().to_string(),
                                            enabled,
                                            is_system,
                                            scope: scope.to_string(),
                                        });
                                    } else {
                                        let value: Result<serde_json::Value, _> =
                                            serde_json::from_str(&content);
                                        if let Ok(val) = value {
                                            let mut m = ExtensionManifest::default();
                                            if let Some(obj) = val.as_object() {
                                                m.display_name = obj
                                                    .get("display_name")
                                                    .and_then(|v| v.as_str())
                                                    .map(|s| s.to_string());
                                                m.author = obj
                                                    .get("author")
                                                    .and_then(|v| v.as_str())
                                                    .map(|s| s.to_string());
                                                m.version = obj
                                                    .get("version")
                                                    .and_then(|v| v.as_str())
                                                    .map(|s| s.to_string());
                                                m.home_page = obj
                                                    .get("homePage")
                                                    .and_then(|v| v.as_str())
                                                    .map(|s| s.to_string());
                                                m.auto_update = obj
                                                    .get("auto_update")
                                                    .and_then(|v| v.as_bool());
                                                m.minimum_client_version = obj
                                                    .get("minimum_client_version")
                                                    .and_then(|v| v.as_str())
                                                    .map(|s| s.to_string());
                                            }
                                            exts.push(ExtensionInfo {
                                                id: entry
                                                    .file_name()
                                                    .to_string_lossy()
                                                    .to_string(),
                                                manifest: m,
                                                dir_path: entry
                                                    .path()
                                                    .to_string_lossy()
                                                    .to_string(),
                                                enabled,
                                                is_system,
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

        // 1. 用户扩展
        let user_extensions_dir = data_dir
            .join("st_data")
            .join("default-user")
            .join("extensions");
        scan_dir(&user_extensions_dir, false, "user", &mut extensions);

        // 2. 全局官方 + 第三方扩展
        if !version.is_empty() {
            let global_official_dir = data_dir
                .join("sillytavern")
                .join(&version)
                .join("public")
                .join("scripts")
                .join("extensions");
            scan_dir(&global_official_dir, true, "global", &mut extensions);

            let global_third_party_dir = global_official_dir.join("third-party");
            scan_dir(&global_third_party_dir, false, "global", &mut extensions);
        }

        let ext_names: Vec<String> = extensions
            .iter()
            .map(|ext| {
                ext.manifest
                    .display_name
                    .clone()
                    .unwrap_or_else(|| ext.id.clone())
            })
            .collect();
        tracing::info!(
            "共获取到 {} 个扩展: {:?}, 耗时: {:?}",
            extensions.len(),
            ext_names,
            start_time.elapsed()
        );

        Ok(extensions)
    })
    .await
    .map_err(|e| {
        tracing::error!("获取扩展列表失败: {}", e);
        e.to_string()
    })?
}

#[tauri::command]
pub fn toggle_extension_enable(
    _app: tauri::AppHandle,
    _id: String,
    enable: bool,
    dir_path: String,
) -> Result<(), String> {
    tracing::info!(
        "切换扩展启用状态: id={}, enable={}, dir={}",
        _id,
        enable,
        dir_path
    );
    let extension_dir = PathBuf::from(&dir_path);

    if !extension_dir.exists() {
        tracing::warn!("扩展目录不存在: {:?}", extension_dir);
        return Err("扩展目录不存在".to_string());
    }

    let manifest_path = extension_dir.join("manifest.json");
    let disabled_manifest_path = extension_dir.join("manifest.json.disable");

    if enable {
        if disabled_manifest_path.exists() {
            std::fs::rename(&disabled_manifest_path, &manifest_path).map_err(|e| {
                tracing::error!("重命名 manifest 失败: {}", e);
                e.to_string()
            })?;
        } else if !manifest_path.exists() {
            tracing::warn!("未找到清单文件 (启用操作)");
            return Err("未找到清单文件".to_string());
        }
    } else {
        if manifest_path.exists() {
            std::fs::rename(&manifest_path, &disabled_manifest_path).map_err(|e| {
                tracing::error!("重命名 manifest 失败: {}", e);
                e.to_string()
            })?;
        } else if !disabled_manifest_path.exists() {
            tracing::warn!("未找到清单文件 (禁用操作)");
            return Err("未找到清单文件".to_string());
        }
    }

    Ok(())
}

#[tauri::command]
pub fn delete_extension(
    _app: tauri::AppHandle,
    _id: String,
    dir_path: String,
) -> Result<(), String> {
    tracing::info!("删除扩展: id={}, dir={}", _id, dir_path);
    let extension_dir = PathBuf::from(&dir_path);

    if !extension_dir.exists() {
        tracing::warn!("要删除的扩展目录不存在: {:?}", extension_dir);
        return Err("扩展目录不存在".to_string());
    }

    std::fs::remove_dir_all(&extension_dir).map_err(|e| {
        tracing::error!("删除扩展目录失败: {}", e);
        e.to_string()
    })?;

    Ok(())
}

#[tauri::command]
pub fn toggle_extension_auto_update(
    _app: tauri::AppHandle,
    _id: String,
    auto_update: bool,
    dir_path: String,
) -> Result<(), String> {
    tracing::info!(
        "切换扩展自动更新状态: id={}, auto_update={}, dir={}",
        _id,
        auto_update,
        dir_path
    );
    let extension_dir = PathBuf::from(&dir_path);

    let mut manifest_path = extension_dir.join("manifest.json");
    if !manifest_path.exists() {
        manifest_path = extension_dir.join("manifest.json.disable");
    }

    if !manifest_path.exists() {
        tracing::warn!("扩展清单不存在: {:?}", manifest_path);
        return Err("扩展清单不存在".to_string());
    }

    let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
        tracing::error!("读取 manifest 失败: {}", e);
        e.to_string()
    })?;
    let mut val: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        tracing::error!("解析 manifest JSON 失败: {}", e);
        e.to_string()
    })?;

    if let Some(obj) = val.as_object_mut() {
        obj.insert(
            "auto_update".to_string(),
            serde_json::Value::Bool(auto_update),
        );
    }

    let new_content = serde_json::to_string_pretty(&val).map_err(|e| {
        tracing::error!("序列化 manifest JSON 失败: {}", e);
        e.to_string()
    })?;
    std::fs::write(manifest_path, new_content).map_err(|e| {
        tracing::error!("写入 manifest 失败: {}", e);
        e.to_string()
    })?;

    Ok(())
}

#[tauri::command]
pub fn open_extension_folder(
    app: tauri::AppHandle,
    scope: String,
    version: String,
) -> Result<(), String> {
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&std::path::PathBuf::from("."))
        .to_path_buf();

    let extensions_dir = if scope == "global" {
        if version.is_empty() {
            return Err("未指定酒馆版本，无法打开全局扩展目录".to_string());
        }
        data_dir
            .join("sillytavern")
            .join(&version)
            .join("public")
            .join("scripts")
            .join("extensions")
    } else {
        data_dir
            .join("st_data")
            .join("default-user")
            .join("extensions")
    };

    if !extensions_dir.exists() {
        std::fs::create_dir_all(&extensions_dir).map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(&extensions_dir);
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
        cmd.spawn().map_err(|e| e.to_string())?;
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
pub fn open_specific_extension_folder(
    _app: tauri::AppHandle,
    dir_path: String,
) -> Result<(), String> {
    let extension_dir = PathBuf::from(&dir_path);

    if !extension_dir.exists() {
        return Err("扩展目录不存在".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(&extension_dir);
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
        cmd.spawn().map_err(|e| e.to_string())?;
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
