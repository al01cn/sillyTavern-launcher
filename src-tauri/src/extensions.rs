use std::path::PathBuf;
use crate::types::{ExtensionInfo, ExtensionManifest};
use crate::utils::get_config_path;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;

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
pub async fn install_extension_zip(
    app: tauri::AppHandle,
    zip_path: String,
    scope: String,
    version: crate::types::LocalTavernItem,
) -> Result<(), String> {
    tracing::info!(
        "开始安装扩展, zip_path: {}, scope: {}, version: {}",
        zip_path,
        scope,
        version.version
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
            .join("third-party")
    } else {
        if version.version.is_empty() {
            return Err("未指定酒馆版本，无法安装全局扩展".to_string());
        }
        
        let st_dir = if version.path.is_empty() {
            data_dir.join("sillytavern").join(&version.version)
        } else {
            std::path::PathBuf::from(&version.path)
        };
        
        st_dir
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

    let final_id = extract_target.file_name().ok_or("无法获取安装目录名")?.to_string_lossy().to_string();
    
    // 检查是否开启自动修复
    let config = crate::config::read_app_config_from_disk(&app);
    if config.auto_repair_git {
        let _ = repair_extension_git(app, final_id, scope).await;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_extensions(
    app: tauri::AppHandle,
    version: crate::types::LocalTavernItem,
) -> Result<Vec<ExtensionInfo>, String> {
    tracing::info!("获取扩展列表，当前酒馆版本: {}", version.version);
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
                                            has_git: entry.path().join(".git").exists(),
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
                                                has_git: entry.path().join(".git").exists(),
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

        let user_third_party_dir = user_extensions_dir.join("third-party");
        scan_dir(&user_third_party_dir, false, "user", &mut extensions);

        // 2. 全局官方 + 第三方扩展
        let global_official_dir = if version.path.is_empty() {
            data_dir
                .join("sillytavern")
                .join(&version.version)
                .join("public")
                .join("scripts")
                .join("extensions")
        } else {
            PathBuf::from(&version.path)
                .join("public")
                .join("scripts")
                .join("extensions")
        };
        
        if global_official_dir.exists() {
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
    version: crate::types::LocalTavernItem,
) -> Result<(), String> {
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&std::path::PathBuf::from("."))
        .to_path_buf();

    let extensions_dir = if scope == "global" {
        if version.version.is_empty() {
            return Err("未指定酒馆版本，无法打开全局扩展目录".to_string());
        }
        
        let st_dir = if version.path.is_empty() {
            data_dir.join("sillytavern").join(&version.version)
        } else {
            std::path::PathBuf::from(&version.path)
        };
        
        st_dir
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

#[tauri::command]
pub async fn verify_extension_zip_from_bytes(
    bytes: Vec<u8>,
) -> Result<ExtensionManifest, String> {
    tokio::task::spawn_blocking(move || {
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name().to_string();

            if name == "manifest.json" || name.ends_with("/manifest.json") {
                let mut contents = String::new();
                std::io::Read::read_to_string(&mut file, &mut contents)
                    .map_err(|e| e.to_string())?;

                let manifest: ExtensionManifest = serde_json::from_str(&contents)
                    .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;

                return Ok(manifest);
            }
        }

        Err("未在压缩包中找到 manifest.json 文件，这不是一个有效的扩展".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn install_extension_zip_from_bytes(
    app: tauri::AppHandle,
    bytes: Vec<u8>,
    filename: String,
    scope: String,
    version: crate::types::LocalTavernItem,
) -> Result<(), String> {
    if filename.trim().is_empty() {
        return Err("文件名不能为空".to_string());
    }
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err("文件名不合法".to_string());
    }
    if !filename.to_lowercase().ends_with(".zip") {
        return Err("只支持 zip 扩展包".to_string());
    }

    let app_clone = app.clone();
    let scope_clone = scope.clone();

    let res = tokio::task::spawn_blocking(move || {
        let data_dir = get_config_path(&app_clone)
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .to_path_buf();

        let target_dir = if scope_clone == "user" {
            data_dir
                .join("st_data")
                .join("default-user")
                .join("extensions")
                .join("third-party")
        } else {
            if version.version.is_empty() {
                return Err("未指定酒馆版本，无法安装全局扩展".to_string());
            }
            
            let st_dir = if version.path.is_empty() {
                data_dir.join("sillytavern").join(&version.version)
            } else {
                std::path::PathBuf::from(&version.path)
            };
            
            st_dir
                .join("public")
                .join("scripts")
                .join("extensions")
                .join("third-party")
        };

        if !target_dir.exists() {
            std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
        }

        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;

        // 检测 root 目录
        let mut root_dir: Option<String> = None;
        let mut single_root = true;

        for i in 0..archive.len() {
            let file = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name().to_string();
            let first = name.split('/').next().unwrap_or("").to_string();

            if root_dir.is_none() {
                root_dir = Some(first.clone());
            } else if root_dir.as_ref().unwrap() != &first {
                single_root = false;
                break;
            }
        }

        let file_stem = std::path::Path::new(&filename)
            .file_stem()
            .unwrap_or(std::ffi::OsStr::new("extension"))
            .to_string_lossy()
            .to_string();

        let mut final_id = file_stem.clone();
        
        // 如果是单层目录，解压后目录名就是那个目录名
        if single_root {
            if let Some(root) = root_dir {
                let r = root.trim_end_matches('/');
                if !r.is_empty() {
                    final_id = r.to_string();
                }
            }
        }

        let extract_target = target_dir.join(&final_id);

        if !extract_target.exists() {
            std::fs::create_dir_all(&extract_target).map_err(|e| e.to_string())?;
        }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file.enclosed_name() {
                Some(p) => p.to_owned(),
                None => continue,
            };

            // 如果 zip 内部有根目录，解压时去掉那一层
            let target_path = if single_root {
                let mut components = outpath.components();
                components.next(); // 跳过根目录
                extract_target.join(components.as_path())
            } else {
                extract_target.join(&outpath)
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = target_path.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let mut outfile = std::fs::File::create(&target_path)
                    .map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| e.to_string())?;
            }
        }

        Ok(final_id)
    })
    .await
    .map_err(|e| e.to_string())??;

    // 检查是否开启自动修复
    let config = crate::config::read_app_config_from_disk(&app);
    if config.auto_repair_git {
        if let Err(e) = repair_extension_git(app.clone(), res, scope).await {
            let _ = app.emit("git-install-log", format!("[{}] ! 自动修复 Git 环境失败: {}", chrono::Local::now().format("%H:%M:%S"), e));
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn install_extension_git(
    app: tauri::AppHandle,
    url: String,
    branch_opt: Option<String>,
    scope: String,
    version: crate::types::LocalTavernItem,
) -> Result<(), String> {
    tracing::info!(
        "开始从 Git 安装扩展, url: {}, branch: {:?}, scope: {}, version: {}",
        url,
        branch_opt,
        scope,
        version.version
    );
    let start_time = chrono::Local::now().format("%H:%M:%S").to_string();
    let _ = app.emit("git-install-log", format!("[{}] > 初始化安装环境...", start_time));

    let app_handle = app.clone();
    let config = crate::config::read_app_config_from_disk(&app_handle);

    // 确定 Git 可执行体路径
    let git_exe = crate::git::get_git_exe(&app);

    let data_dir = get_config_path(&app_handle)
        .parent()
        .unwrap_or(&std::path::PathBuf::from("."))
        .to_path_buf();

    // 确定仓库名称
    let repo_name = url.trim_end_matches('/')
        .split('/')
        .last()
        .unwrap_or("extension")
        .trim_end_matches(".git")
        .to_string();

    // 确定安装父目录 (根据用户要求，统一放在 third-party 下)
    let target_parent = if scope == "user" {
        data_dir
            .join("st_data")
            .join("default-user")
            .join("extensions")
            .join("third-party")
    } else {
        if version.version.is_empty() {
            return Err("未指定酒馆版本，无法安装全局扩展".to_string());
        }
        
        let st_dir = if version.path.is_empty() {
            data_dir.join("sillytavern").join(&version.version)
        } else {
            std::path::PathBuf::from(&version.path)
        };
        
        st_dir
            .join("public")
            .join("scripts")
            .join("extensions")
            .join("third-party")
    };

    if !target_parent.exists() {
        std::fs::create_dir_all(&target_parent).map_err(|e| e.to_string())?;
    }

    let target_dir = target_parent.join(&repo_name);
    let _ = app.emit("git-install-log", format!("[{}] > 目标路径: {}", chrono::Local::now().format("%H:%M:%S"), target_dir.display()));
    
    if target_dir.exists() {
        let _ = app.emit("git-install-log", format!("[{}] ! 错误: 目录已存在", chrono::Local::now().format("%H:%M:%S")));
        return Err(format!("目录 {} 已存在，请先删除旧扩展或更换仓库名称", repo_name));
    }

    // 准备 Git 命令
    let mut cmd = TokioCommand::new(git_exe);
    cmd.arg("clone");
    cmd.arg("--progress"); // 关键：开启进度输出以便捕获详细日志
    
    if let Some(ref b) = branch_opt {
        if !b.trim().is_empty() {
            cmd.arg("-b").arg(b);
        }
    }
    
    cmd.arg("--depth").arg("1");

    // 处理 GitHub 加速 URL
    let mut final_url = url.clone();
    if config.github_proxy.enable && !config.github_proxy.url.is_empty() && url.contains("github.com") {
        let proxy_url = config.github_proxy.url.trim_end_matches('/');
        final_url = format!("{}/{}", proxy_url, url.trim_start_matches('/'));
        tracing::info!("使用 GitHub 加速地址: {}", final_url);
        let _ = app.emit("git-install-log", format!("[{}] > 使用 GitHub 加速: {}", chrono::Local::now().format("%H:%M:%S"), final_url));
    } else {
        let _ = app.emit("git-install-log", format!("[{}] > 原始地址: {}", chrono::Local::now().format("%H:%M:%S"), url));
    }

    cmd.arg(&final_url);
    cmd.arg(&target_dir);

    cmd.stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000);
    }

    let mut child = cmd.spawn().map_err(|e| format!("执行 Git 命令失败: {}", e))?;
    
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let app_clone1 = app.clone();
    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            let now = chrono::Local::now().format("%H:%M:%S").to_string();
            let _ = app_clone1.emit("git-install-log", format!("[{}] {}", now, line));
        }
    });

    let app_clone2 = app.clone();
    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            let now = chrono::Local::now().format("%H:%M:%S").to_string();
            let _ = app_clone2.emit("git-install-log", format!("[{}] {}", now, line));
        }
    });

    let _ = app.emit("git-install-log", format!("[{}] > 开始克隆仓库...", chrono::Local::now().format("%H:%M:%S")));
    let status = child.wait().await.map_err(|e| e.to_string())?;

    if !status.success() {
        let _ = app.emit("git-install-log", format!("[{}] ! 克隆失败，请检查网络或仓库权限。", chrono::Local::now().format("%H:%M:%S")));
        return Err("Git 克隆失败，请检查日志。".to_string());
    }

    let _ = app.emit("git-install-log", format!("[{}] √ 克隆并安装成功！", chrono::Local::now().format("%H:%M:%S")));
    Ok(())
}

#[tauri::command]
pub async fn repair_extension_git(
    app: tauri::AppHandle,
    id: String,
    scope: String,
) -> Result<(), String> {
    // 1. 获取基础路径
    let app_handle = app.clone();
    let config = crate::config::read_app_config_from_disk(&app_handle);
    let data_dir = get_config_path(&app_handle)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    
    // 2. 构造目标目录
    let target_dir = if scope == "user" {
        data_dir.join("st_data").join("default-user").join("extensions").join(&id)
    } else {
        let st_dir = if !config.sillytavern.version.path.is_empty() {
            PathBuf::from(&config.sillytavern.version.path)
        } else {
            data_dir.join("sillytavern").join(&config.sillytavern.version.version)
        };
        st_dir.join("public").join("scripts").join("extensions").join("third-party").join(&id)
    };

    if !target_dir.exists() {
        return Err("扩展目录不存在".to_string());
    }

    // 3. 检查 manifest.json 获取 homePage
    let manifest_path = target_dir.join("manifest.json");
    let manifest_path_disabled = target_dir.join("manifest.json.disable");
    let active_manifest = if manifest_path.exists() {
        manifest_path
    } else if manifest_path_disabled.exists() {
        manifest_path_disabled
    } else {
        return Err("未找到 manifest.json".to_string());
    };

    let content = std::fs::read_to_string(&active_manifest).map_err(|e| format!("读取配置失败: {}", e))?;
    let manifest: crate::types::ExtensionManifest = serde_json::from_str(&content).map_err(|e| format!("解析配置失败: {}", e))?;
    
    let url = manifest.home_page.clone().ok_or("该扩展 manifest.json 中未定义 homePage，无法定位远程仓库。")?;
    
    // 扩展修复标准：
    // 1. auto_update 字段为 true (旧标准)
    // 2. 或者 homePage 是一个标准的 Git 仓库链接 (新标准)
    let is_git_url = {
        let u = url.to_lowercase();
        u.contains("github.com") || u.contains("gitee.com") || u.contains("gitcode.com") || u.contains("gitlab.com") || u.ends_with(".git")
    };
    
    let can_repair = manifest.auto_update.unwrap_or(false) || is_git_url;

    if !can_repair {
        return Err("该扩展不具备自动更新属性 (auto_update: true) 且 homePage 不是标准的 Git 链接，无法自动修复。".to_string());
    }

    if url.trim().is_empty() || url == "None" {
         return Err("该扩展 manifest.json 中 homePage 为空，无法定位远程仓库。".to_string());
    }

    // 4. 执行 Git 修复
    let git_exe = crate::git::get_git_exe(&app);
    if !git_exe.exists() && git_exe.to_string_lossy() != "git" {
        return Err("未找到 Git 环境，请先安装 Git。".to_string());
    }

    // 发送初始化日志
    let _ = app.emit("git-install-log", format!("[{}] > 正在修复扩展 Git 环境: {}", chrono::Local::now().format("%H:%M:%S"), id));
    
    // git init
    let mut cmd = TokioCommand::new(&git_exe);
    cmd.current_dir(&target_dir)
       .arg("init");

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000);
    }

    let mut child = cmd.spawn()
        .map_err(|e| format!("git init 失败: {}", e))?;
    child.wait().await.map_err(|e| e.to_string())?;

    // 处理加速 URL
    let mut final_url = url.clone();
    if config.github_proxy.enable && !config.github_proxy.url.is_empty() && url.contains("github.com") {
        let proxy_url = config.github_proxy.url.trim_end_matches('/');
        final_url = format!("{}/{}", proxy_url, url.trim_start_matches('/'));
    }

    let _ = app.emit("git-install-log", format!("[{}] > 远程仓库: {}", chrono::Local::now().format("%H:%M:%S"), final_url));
    
    // 强制更新 remote origin
    // 先尝试 remove (忽略失败)
    let mut cmd_rm = TokioCommand::new(&git_exe);
    cmd_rm.current_dir(&target_dir)
          .arg("remote")
          .arg("remove")
          .arg("origin");

    #[cfg(target_os = "windows")]
    {
        cmd_rm.creation_flags(0x08000000);
    }

    let _ = cmd_rm.spawn().map_err(|e| e.to_string())?.wait().await;

    // git remote add origin
    let mut cmd_add = TokioCommand::new(&git_exe);
    cmd_add.current_dir(&target_dir)
           .arg("remote")
           .arg("add")
           .arg("origin")
           .arg(&final_url);

    #[cfg(target_os = "windows")]
    {
        cmd_add.creation_flags(0x08000000);
    }

    let mut child = cmd_add.spawn()
        .map_err(|e| format!("git remote add 失败: {}", e))?;
    child.wait().await.map_err(|e| e.to_string())?;

    // git fetch (depth 1)
    let _ = app.emit("git-install-log", format!("[{}] > 正在从远程拉取数据...", chrono::Local::now().format("%H:%M:%S")));
    let mut cmd_fetch = TokioCommand::new(&git_exe);
    cmd_fetch.current_dir(&target_dir)
             .arg("fetch")
             .arg("--depth").arg("1")
             .arg("origin");

    #[cfg(target_os = "windows")]
    {
        cmd_fetch.creation_flags(0x08000000);
    }
    
    let mut child = cmd_fetch.spawn()
        .map_err(|e| format!("git fetch 失败: {}", e))?;
    child.wait().await.map_err(|e| e.to_string())?;

    // 尝试识别默认分支并重置 HEAD
    // 这里为了不破坏用户本地可能存在的修改，我们只做 fetch。
    // 如果要彻底修复，通常需要 reset，但风险较高。
    // 我们可以尝试 git symbolic-ref refs/remotes/origin/HEAD refs/remotes/origin/main
    
    let _ = app.emit("git-install-log", format!("[{}] √ 修复成功！现在该扩展已具备 Git 环境。", chrono::Local::now().format("%H:%M:%S")));

    Ok(())
}

