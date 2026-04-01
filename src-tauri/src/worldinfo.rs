use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;

use crate::types::WorldInfoFile;
use crate::utils::get_config_path;

// ─────────────────────────────────────────────
// 内部辅助：世界书目录
// ─────────────────────────────────────────────

fn get_worlds_dir(app: &AppHandle) -> PathBuf {
    let data_dir = get_config_path(app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();

    let primary = data_dir.join("st_data").join("worlds");
    if primary.exists() {
        return primary;
    }

    let fallback = data_dir.join("st_data").join("default-user").join("worlds");
    if fallback.exists() {
        return fallback;
    }

    fallback
}

// ─────────────────────────────────────────────
// Tauri commands
// ─────────────────────────────────────────────

#[tauri::command]
pub async fn list_world_infos(app: AppHandle) -> Result<Vec<WorldInfoFile>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let dir = get_worlds_dir(&app_clone);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();
        let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;
        for entry in entries {
            let entry = match entry {
                Ok(v) => v,
                Err(_) => continue,
            };
            let file_type = match entry.file_type() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !file_type.is_file() {
                continue;
            }
            let path = entry.path();
            let ext_ok = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("json"))
                .unwrap_or(false);
            if !ext_ok {
                continue;
            }
            let file_name = match path.file_name().and_then(|s| s.to_str()) {
                Some(v) => v.to_string(),
                None => continue,
            };

            let meta = match entry.metadata() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let modified_ms = meta.modified().ok().and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_millis() as i64)
            });

            result.push(WorldInfoFile {
                file_name,
                size: meta.len(),
                modified_ms,
            });
        }

        result.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn read_world_info(app: AppHandle, file_name: String) -> Result<String, String> {
    if file_name.trim().is_empty() {
        return Err("文件名不能为空".to_string());
    }
    if file_name.contains("..") || file_name.contains('/') || file_name.contains('\\') {
        return Err("文件名不合法".to_string());
    }
    if !file_name.to_lowercase().ends_with(".json") {
        return Err("仅支持 .json 文件".to_string());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let dir = get_worlds_dir(&app_clone);
        let file_path = dir.join(&file_name);
        if !file_path.exists() {
            return Err("文件不存在".to_string());
        }
        fs::read_to_string(&file_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_world_infos(app: AppHandle, file_names: Vec<String>) -> Result<(), String> {
    if file_names.is_empty() {
        return Ok(());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let dir = get_worlds_dir(&app_clone);
        let mut errors = Vec::new();

        for file_name in file_names {
            if file_name.trim().is_empty()
                || file_name.contains("..")
                || file_name.contains('/')
                || file_name.contains('\\')
            {
                errors.push(format!("文件名不合法: {}", file_name));
                continue;
            }

            let file_path = dir.join(&file_name);
            if file_path.exists() {
                if let Err(e) = fs::remove_file(&file_path) {
                    errors.push(format!("无法删除 {}: {}", file_name, e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_world_info(app: AppHandle, source_path: String) -> Result<(), String> {
    if source_path.trim().is_empty() {
        return Err("源路径不能为空".to_string());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let source = std::path::PathBuf::from(&source_path);
        if !source.exists() || !source.is_file() {
            return Err("源文件不存在或不是文件".to_string());
        }

        let ext = source
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "json" {
            return Err("只支持导入 json 格式的世界书文件".to_string());
        }

        let file_name = source
            .file_name()
            .ok_or("无效的文件名")?
            .to_string_lossy()
            .to_string();

        let dir = get_worlds_dir(&app_clone);
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        }

        let target_path = dir.join(&file_name);
        if target_path.exists() {
            return Err("同名世界书文件已存在，请重命名后再导入".to_string());
        }

        fs::copy(&source, &target_path).map_err(|e| format!("复制文件失败: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_world_info_from_bytes(
    app: AppHandle,
    bytes: Vec<u8>,
    filename: String,
) -> Result<(), String> {
    // 1. 安全和格式校验
    if filename.trim().is_empty() {
        return Err("文件名不能为空".to_string());
    }
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err("文件名不合法".to_string());
    }
    if !filename.to_lowercase().ends_with(".json") {
        return Err("只支持导入 json 格式的世界书".to_string());
    }

    let app_clone = app.clone();

    // 2. 放入 blocking 线程池执行写入
    tokio::task::spawn_blocking(move || {
        // 修正了这里的函数调用，改为 get_worlds_dir
        let dir = get_worlds_dir(&app_clone);
        if !dir.exists() {
            std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let target_path = dir.join(&filename);

        if target_path.exists() {
            return Err("同名世界书已存在，请重命名后再导入".to_string());
        }

        std::fs::write(&target_path, bytes).map_err(|e| format!("写入文件失败: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
