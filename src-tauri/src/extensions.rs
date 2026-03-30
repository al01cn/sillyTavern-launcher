use std::path::PathBuf;
use crate::types::{ExtensionInfo, ExtensionManifest};
use crate::utils::get_config_path;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;

// ─────────────────────────────────────────────
// 内部辅助函数
// ─────────────────────────────────────────────

/// 判断是否为官方扩展：
/// - 没有 auto_update=true，且
/// - homePage 不是 GitHub / Gitee / GitLab 仓库地址
/// 官方扩展随酒馆本体更新，不需要单独修复 git 环境。
fn is_official_extension(manifest: &ExtensionManifest) -> bool {
    if manifest.auto_update == Some(true) {
        return false;
    }
    if let Some(hp) = &manifest.home_page {
        let lower = hp.to_lowercase();
        if lower.contains("github.com")
            || lower.contains("gitee.com")
            || lower.contains("gitlab.com")
            || lower.ends_with(".git")
        {
            return false;
        }
    }
    true
}

/// 获取酒馆根目录
fn get_st_dir(version: &crate::types::LocalTavernItem, data_dir: &PathBuf) -> Result<PathBuf, String> {
    if version.version.is_empty() && version.path.is_empty() {
        return Err("未指定酒馆版本，无法安装全局扩展".to_string());
    }
    Ok(if version.path.is_empty() {
        data_dir.join("sillytavern").join(&version.version)
    } else {
        PathBuf::from(&version.path)
    })
}

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

    let file = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    // ── 第一步：读取 manifest，判断官方 / 第三方 ──────────────────────────────
    let mut manifest_opt: Option<ExtensionManifest> = None;
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = f.name().to_string();
        if name == "manifest.json" || name.ends_with("/manifest.json") {
            let mut contents = String::new();
            std::io::Read::read_to_string(&mut f, &mut contents).map_err(|e| e.to_string())?;
            manifest_opt = serde_json::from_str::<ExtensionManifest>(&contents).ok();
            break;
        }
    }
    // 重新打开 archive（已被消耗）
    let file2 = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file2).map_err(|e| e.to_string())?;

    let is_official = manifest_opt.as_ref().map(is_official_extension).unwrap_or(false);

    // ── 第二步：根据官方 / 第三方决定安装目录 ─────────────────────────────────
    // 官方扩展：无论用户选择 scope，统一安装到全局 public/scripts/extensions/{id}
    // 第三方扩展：按 scope 安装到对应的 third-party/{id}
    let (target_dir, is_third_party) = if is_official {
        let st_dir = get_st_dir(&version, &data_dir)?;
        let dir = st_dir.join("public").join("scripts").join("extensions");
        (dir, false)
    } else if scope == "user" {
        let dir = data_dir
            .join("st_data")
            .join("default-user")
            .join("extensions")
            .join("third-party");
        (dir, true)
    } else {
        let st_dir = get_st_dir(&version, &data_dir)?;
        let dir = st_dir
            .join("public")
            .join("scripts")
            .join("extensions")
            .join("third-party");
        (dir, true)
    };

    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    }

    // ── 第三步：检测 zip 内是否有公共根目录 ───────────────────────────────────
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

    // 确定解压目标子目录名：单层目录优先用 zip 内根目录名，否则用文件名
    let final_id = if single_root {
        if let Some(root) = root_dir {
            let r = root.trim_end_matches('/');
            if !r.is_empty() { r.to_string() } else { file_stem.clone() }
        } else {
            file_stem.clone()
        }
    } else {
        file_stem.clone()
    };

    let extract_target = target_dir.join(&final_id);

    if !extract_target.exists() {
        std::fs::create_dir_all(&extract_target).map_err(|e| e.to_string())?;
    }

    // ── 第四步：解压文件 ───────────────────────────────────────────────────────
    // 重新打开 archive（检测 root_dir 时已遍历）
    let file3 = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file3).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        // 单层根目录时：去掉第一个组件（根目录名），避免双层嵌套
        let stripped_path = if single_root {
            let mut comps = outpath.components();
            comps.next(); // 跳过根目录组件
            comps.as_path().to_owned()
        } else {
            outpath.clone()
        };

        // 跳过空路径（即根目录本身）
        if stripped_path.as_os_str().is_empty() {
            continue;
        }

        let target_path = extract_target.join(&stripped_path);

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

    // ── 第五步：自动修复（仅第三方扩展）──────────────────────────────────────
    if is_third_party {
        let config = crate::config::read_app_config_from_disk(&app);
        if config.auto_repair_git {
            let repair_scope = if is_official { "global".to_string() } else { scope.clone() };
            let exact_dir = extract_target.to_string_lossy().to_string();
            let dummy_pid = std::sync::Arc::new(tokio::sync::Mutex::new(None::<u32>));
            match repair_extension_git_inner(app, dummy_pid, final_id, repair_scope, Some(exact_dir)).await {
                Ok(true) => tracing::info!("自动修复 Git 环境成功（在线）"),
                Ok(false) => tracing::info!("自动修复 Git 环境完成（离线保底）"),
                Err(e) => tracing::warn!("自动修复 Git 环境失败: {}", e),
            }
        }
    } else {
        tracing::info!("官方扩展，跳过自动修复 Git 环境");
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
        // default-user/extensions/ 和 default-user/extensions/third-party/ 都是官方原定用户级第三方扩展的位置
        // 两个目录都用 is_system=false 扫描，但安装时会根据 manifest 判断是否为官方扩展
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
    let bytes_clone = bytes.clone();

    let (final_id, is_third_party, extract_target_str) = tokio::task::spawn_blocking(move || -> Result<(String, bool, String), String> {
        let data_dir = get_config_path(&app_clone)
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .to_path_buf();

        // ── 第一步：读取 manifest，判断官方 / 第三方 ─────────────────────────
        let reader = std::io::Cursor::new(bytes_clone.clone());
        let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;

        let mut manifest_opt: Option<ExtensionManifest> = None;
        for i in 0..archive.len() {
            let mut f = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = f.name().to_string();
            if name == "manifest.json" || name.ends_with("/manifest.json") {
                let mut contents = String::new();
                std::io::Read::read_to_string(&mut f, &mut contents).map_err(|e| e.to_string())?;
                manifest_opt = serde_json::from_str::<ExtensionManifest>(&contents).ok();
                break;
            }
        }

        let is_official = manifest_opt.as_ref().map(is_official_extension).unwrap_or(false);

        // ── 第二步：根据官方 / 第三方决定安装目录 ────────────────────────────
        let (target_dir, is_third_party) = if is_official {
            let st_dir = get_st_dir(&version, &data_dir)?;
            let dir = st_dir.join("public").join("scripts").join("extensions");
            (dir, false)
        } else if scope_clone == "user" {
            let dir = data_dir
                .join("st_data")
                .join("default-user")
                .join("extensions")
                .join("third-party");
            (dir, true)
        } else {
            let st_dir = get_st_dir(&version, &data_dir)?;
            let dir = st_dir
                .join("public")
                .join("scripts")
                .join("extensions")
                .join("third-party");
            (dir, true)
        };

        if !target_dir.exists() {
            std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
        }

        // ── 第三步：检测 zip 内根目录 ─────────────────────────────────────────
        let reader2 = std::io::Cursor::new(bytes_clone.clone());
        let mut archive = zip::ZipArchive::new(reader2).map_err(|e| e.to_string())?;

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

        // ── 第四步：解压文件 ──────────────────────────────────────────────────
        let reader3 = std::io::Cursor::new(bytes_clone);
        let mut archive = zip::ZipArchive::new(reader3).map_err(|e| e.to_string())?;

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
                let stripped = components.as_path();
                if stripped.as_os_str().is_empty() {
                    continue;
                }
                extract_target.join(stripped)
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

        Ok((final_id, is_third_party, extract_target.to_string_lossy().to_string()))
    })
    .await
    .map_err(|e| e.to_string())??;

    // ── 第五步：自动修复（仅第三方扩展）─────────────────────────────────────
    if is_third_party {
        let config = crate::config::read_app_config_from_disk(&app);
        if config.auto_repair_git {
            let dummy_pid = std::sync::Arc::new(tokio::sync::Mutex::new(None::<u32>));
            match repair_extension_git_inner(app.clone(), dummy_pid, final_id, scope, Some(extract_target_str)).await {
                Ok(true) => tracing::info!("自动修复 Git 环境成功（在线）"),
                Ok(false) => tracing::info!("自动修复 Git 环境完成（离线保底）"),
                Err(e) => {
                    let _ = app.emit("git-install-log", format!(
                        "[{}] ! 自动修复 Git 环境失败: {}",
                        chrono::Local::now().format("%H:%M:%S"), e
                    ));
                }
            }
        }
    } else {
        tracing::info!("官方扩展，跳过自动修复 Git 环境");
    }

    Ok(())
}

#[tauri::command]
pub async fn install_extension_git(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::types::InstallState>,
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

    // 记录 git 子进程 PID，供程序退出时安全终止
    if let Some(pid) = child.id() {
        *state.git_child_pid.lock().await = Some(pid);
        tracing::info!("git clone 子进程 PID={}", pid);
    }
    
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

    // 克隆完成，清除 PID 记录
    *state.git_child_pid.lock().await = None;

    if !status.success() {
        let _ = app.emit("git-install-log", format!("[{}] ! 克隆失败，请检查网络或仓库权限。", chrono::Local::now().format("%H:%M:%S")));
        return Err("Git 克隆失败，请检查日志。".to_string());
    }

    let _ = app.emit("git-install-log", format!("[{}] √ 克隆并安装成功！", chrono::Local::now().format("%H:%M:%S")));
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// 离线保底：手动写最小 .git 结构
//
// 让酒馆认为该目录是合法的 Git 仓库（即使没网也能通过 git status）。
// 写入内容：
//   .git/HEAD                  — 指向 refs/heads/main
//   .git/config                — 包含 [core] 和 [remote "origin"] 两段
//   .git/refs/heads/           — 目录（空也行）
//   .git/refs/remotes/origin/  — 目录
//   .git/packed-refs           — 空文件（git 需要它来跑 log）
// 不需要任何 object，git status / git remote -v 等命令均可正常返回。
// ─────────────────────────────────────────────────────────────────────────────
fn write_offline_git_skeleton(target_dir: &PathBuf, remote_url: &str) -> Result<(), String> {
    let git_dir = target_dir.join(".git");
    std::fs::create_dir_all(git_dir.join("refs").join("heads"))
        .map_err(|e| format!("创建 refs/heads 失败: {}", e))?;
    std::fs::create_dir_all(git_dir.join("refs").join("remotes").join("origin"))
        .map_err(|e| format!("创建 refs/remotes/origin 失败: {}", e))?;
    std::fs::create_dir_all(git_dir.join("objects").join("info"))
        .map_err(|e| format!("创建 objects/info 失败: {}", e))?;
    std::fs::create_dir_all(git_dir.join("objects").join("pack"))
        .map_err(|e| format!("创建 objects/pack 失败: {}", e))?;

    // HEAD：指向 main 分支
    std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")
        .map_err(|e| format!("写入 HEAD 失败: {}", e))?;

    // config
    let config_content = format!(
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = false\n\tbare = false\n\tlogallrefupdates = true\n[remote \"origin\"]\n\turl = {}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n[branch \"main\"]\n\tremote = origin\n\tmerge = refs/heads/main\n",
        remote_url
    );
    std::fs::write(git_dir.join("config"), config_content)
        .map_err(|e| format!("写入 config 失败: {}", e))?;

    // packed-refs（空文件）
    std::fs::write(git_dir.join("packed-refs"), "# pack-refs with: peeled fully-peeled sorted\n")
        .map_err(|e| format!("写入 packed-refs 失败: {}", e))?;

    // ORIG_HEAD 不写，info/exclude 可写可不写
    Ok(())
}

/// 返回值语义：
///   Ok(true)  = 完整修复（git init + fetch 均成功）
///   Ok(false) = 离线保底（无法联网 fetch，已写入最小 .git 结构）
///   Err(msg)  = 修复前置条件不满足，彻底失败
#[tauri::command]
pub async fn repair_extension_git(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::types::InstallState>,
    id: String,
    scope: String,
    // 扩展所在的完整绝对目录路径（优先使用）。
    // 传入此参数可精确定位，避免因 scope 推算路径不准确的问题。
    dir_path: Option<String>,
) -> Result<bool, String> {
    repair_extension_git_inner(app, state.git_child_pid.clone(), id, scope, dir_path).await
}

/// 内部实现，接受 Arc 直接传递（供内部自动修复调用）
async fn repair_extension_git_inner(
    app: tauri::AppHandle,
    git_child_pid: std::sync::Arc<tokio::sync::Mutex<Option<u32>>>,
    id: String,
    scope: String,
    dir_path: Option<String>,
) -> Result<bool, String> {
    // 1. 获取基础路径
    let app_handle = app.clone();
    let config = crate::config::read_app_config_from_disk(&app_handle);
    let data_dir = get_config_path(&app_handle)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    
    // 2. 构造目标目录
    // 优先使用前端传来的精确路径；没有时才按 scope 推算（兼容旧调用）
    let target_dir = if let Some(ref dp) = dir_path {
        PathBuf::from(dp)
    } else if scope == "user" {
        data_dir.join("st_data").join("default-user").join("extensions").join("third-party").join(&id)
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

    // 处理加速 URL
    let mut final_url = url.clone();
    if config.github_proxy.enable && !config.github_proxy.url.is_empty() && url.contains("github.com") {
        let proxy_url = config.github_proxy.url.trim_end_matches('/');
        final_url = format!("{}/{}", proxy_url, url.trim_start_matches('/'));
    }

    // 4. 执行 Git 修复
    let git_exe = crate::git::get_git_exe(&app);
    let has_git = git_exe.exists() || git_exe.to_string_lossy() == "git";

    // 发送初始化日志
    let _ = app.emit("git-install-log", format!("[{}] > 正在修复扩展 Git 环境: {}", chrono::Local::now().format("%H:%M:%S"), id));
    let _ = app.emit("git-install-log", format!("[{}] > 远程仓库: {}", chrono::Local::now().format("%H:%M:%S"), final_url));

    // ── 4-a. 有 Git 可执行文件：尝试在线修复 ────────────────────────────────
    if has_git {
        // git init
        let mut cmd = TokioCommand::new(&git_exe);
        cmd.current_dir(&target_dir).arg("init");
        #[cfg(target_os = "windows")] { cmd.creation_flags(0x08000000); }
        let _ = cmd.spawn().map_err(|e| format!("git init 失败: {}", e))?.wait().await;

        // git remote remove origin（忽略失败）
        let mut cmd_rm = TokioCommand::new(&git_exe);
        cmd_rm.current_dir(&target_dir).arg("remote").arg("remove").arg("origin");
        #[cfg(target_os = "windows")] { cmd_rm.creation_flags(0x08000000); }
        let _ = cmd_rm.spawn().map_err(|e| e.to_string())?.wait().await;

        // git remote add origin
        let mut cmd_add = TokioCommand::new(&git_exe);
        cmd_add.current_dir(&target_dir)
               .arg("remote").arg("add").arg("origin").arg(&final_url);
        #[cfg(target_os = "windows")] { cmd_add.creation_flags(0x08000000); }
        let _ = cmd_add.spawn().map_err(|e| format!("git remote add 失败: {}", e))?.wait().await;

        // git fetch (depth 1)
        let _ = app.emit("git-install-log", format!("[{}] > 正在从远程拉取数据...", chrono::Local::now().format("%H:%M:%S")));
        let mut cmd_fetch = TokioCommand::new(&git_exe);
        cmd_fetch.current_dir(&target_dir)
                 .arg("fetch").arg("--depth").arg("1").arg("origin");
        #[cfg(target_os = "windows")] { cmd_fetch.creation_flags(0x08000000); }

        match cmd_fetch.spawn() {
            Ok(mut child) => {
                // 记录 git fetch 子进程 PID
                if let Some(pid) = child.id() {
                    *git_child_pid.lock().await = Some(pid);
                    tracing::info!("git fetch 子进程 PID={}", pid);
                }
                let wait_result = child.wait().await;
                // 清除 PID 记录
                *git_child_pid.lock().await = None;
                match wait_result {
                    Ok(status) if status.success() => {
                        // ✅ 在线修复成功
                        let _ = app.emit("git-install-log", format!(
                            "[{}] √ 修复成功！现在该扩展已具备完整 Git 环境。",
                            chrono::Local::now().format("%H:%M:%S")
                        ));
                        return Ok(true);
                    }
                    Ok(_) => {
                        let _ = app.emit("git-install-log", format!(
                            "[{}] ! git fetch 返回错误，尝试离线保底修复...",
                            chrono::Local::now().format("%H:%M:%S")
                        ));
                    }
                    Err(e) => {
                        let _ = app.emit("git-install-log", format!(
                            "[{}] ! git fetch 执行失败 ({})，尝试离线保底修复...",
                            chrono::Local::now().format("%H:%M:%S"), e
                        ));
                    }
                }
            }
            Err(e) => {
                let _ = app.emit("git-install-log", format!(
                    "[{}] ! 无法启动 git fetch ({})，尝试离线保底修复...",
                    chrono::Local::now().format("%H:%M:%S"), e
                ));
            }
        }
    } else {
        let _ = app.emit("git-install-log", format!(
            "[{}] ! 未检测到 Git 可执行文件，跳过在线修复，执行离线保底...",
            chrono::Local::now().format("%H:%M:%S")
        ));
    }

    // ── 4-b. 离线保底：手动写最小 .git 结构 ─────────────────────────────────
    let _ = app.emit("git-install-log", format!(
        "[{}] > 正在写入最小 .git 结构（离线保底）...",
        chrono::Local::now().format("%H:%M:%S")
    ));

    // 用原始 url（非加速），保证 remote 记录的是真实仓库地址
    write_offline_git_skeleton(&target_dir, &url).map_err(|e| {
        let _ = app.emit("git-install-log", format!(
            "[{}] ✗ 离线保底失败: {}",
            chrono::Local::now().format("%H:%M:%S"), e
        ));
        e
    })?;

    let _ = app.emit("git-install-log", format!(
        "[{}] ~ 离线保底完成。扩展已可被酒馆识别，联网后可正常更新。",
        chrono::Local::now().format("%H:%M:%S")
    ));

    Ok(false) // false = 离线保底（非完整修复）
}

