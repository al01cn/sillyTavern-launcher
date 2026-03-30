use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;

use crate::types::{ChatFile, ChatGroup, ChatMessage};
use crate::utils::get_config_path;

// ─────────────────────────────────────────────
// 内部辅助：对话历史根目录
// st_data/default-user/chats/{角色}/{文件.jsonl}
// ─────────────────────────────────────────────

fn get_chats_dir(app: &AppHandle) -> PathBuf {
    let data_dir = get_config_path(app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();

    data_dir
        .join("st_data")
        .join("default-user")
        .join("chats")
}

// ─────────────────────────────────────────────
// list_chats
// 返回按角色分组的对话历史列表
// ─────────────────────────────────────────────

#[tauri::command]
pub async fn list_chats(app: AppHandle) -> Result<Vec<ChatGroup>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let chats_dir = get_chats_dir(&app_clone);
        if !chats_dir.exists() {
            return Ok(Vec::new());
        }

        let mut groups: Vec<ChatGroup> = Vec::new();

        let char_entries = fs::read_dir(&chats_dir).map_err(|e| e.to_string())?;
        for char_entry in char_entries {
            let char_entry = match char_entry {
                Ok(v) => v,
                Err(_) => continue,
            };
            let char_file_type = match char_entry.file_type() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !char_file_type.is_dir() {
                continue;
            }
            let char_folder = match char_entry.file_name().into_string() {
                Ok(v) => v,
                Err(_) => continue,
            };

            // 从文件夹名解析角色名：default_Seraphina → Seraphina
            // 约定：前缀 "default_" 去掉后即为角色名
            let char_name = if char_folder.starts_with("default_") {
                char_folder[8..].to_string()
            } else {
                char_folder.clone()
            };

            let char_dir = chats_dir.join(&char_folder);
            let file_entries = match fs::read_dir(&char_dir) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let mut files: Vec<ChatFile> = Vec::new();
            for file_entry in file_entries {
                let file_entry = match file_entry {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let file_path = file_entry.path();
                let ext_ok = file_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.eq_ignore_ascii_case("jsonl"))
                    .unwrap_or(false);
                if !ext_ok {
                    continue;
                }
                let file_name = match file_path.file_name().and_then(|s| s.to_str()) {
                    Some(v) => v.to_string(),
                    None => continue,
                };
                let meta = match file_entry.metadata() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let modified_ms = meta.modified().ok().and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_millis() as i64)
                });

                files.push(ChatFile {
                    file_name,
                    char_folder: char_folder.clone(),
                    size: meta.len(),
                    modified_ms,
                });
            }

            if files.is_empty() {
                continue;
            }

            // 按修改时间倒序
            files.sort_by(|a, b| {
                b.modified_ms
                    .unwrap_or(0)
                    .cmp(&a.modified_ms.unwrap_or(0))
            });

            groups.push(ChatGroup {
                char_folder,
                char_name,
                files,
            });
        }

        // 按角色名字母顺序排序
        groups.sort_by(|a, b| a.char_name.to_lowercase().cmp(&b.char_name.to_lowercase()));

        Ok(groups)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─────────────────────────────────────────────
// read_chat
// 读取单个 .jsonl 文件，解析并返回消息列表
// ─────────────────────────────────────────────

#[tauri::command]
pub async fn read_chat(
    app: AppHandle,
    char_folder: String,
    file_name: String,
) -> Result<Vec<ChatMessage>, String> {
    // 安全校验
    if char_folder.trim().is_empty()
        || char_folder.contains("..")
        || char_folder.contains('/')
        || char_folder.contains('\\')
    {
        return Err("角色目录名不合法".to_string());
    }
    if file_name.trim().is_empty()
        || file_name.contains("..")
        || file_name.contains('/')
        || file_name.contains('\\')
    {
        return Err("文件名不合法".to_string());
    }
    if !file_name.to_lowercase().ends_with(".jsonl") {
        return Err("仅支持 .jsonl 文件".to_string());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let file_path = get_chats_dir(&app_clone)
            .join(&char_folder)
            .join(&file_name);

        if !file_path.exists() {
            return Err("文件不存在".to_string());
        }

        let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
        let mut messages: Vec<ChatMessage> = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let val: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // 第一行通常是元数据（包含 user_name、character_name 但没有 mes 字段），跳过
            if line_idx == 0 && val.get("mes").is_none() {
                continue;
            }

            let name = val
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let mes = val
                .get("mes")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let is_user = val
                .get("is_user")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let is_system = val
                .get("is_system")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let send_date = val
                .get("send_date")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if mes.is_empty() {
                continue;
            }

            messages.push(ChatMessage {
                name,
                mes,
                is_user,
                is_system,
                send_date,
            });
        }

        Ok(messages)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─────────────────────────────────────────────
// delete_chats
// 批量删除对话记录（传入 char_folder + file_name 对）
// ─────────────────────────────────────────────

#[tauri::command]
pub async fn delete_chats(
    app: AppHandle,
    items: Vec<crate::types::ChatDeleteItem>,
) -> Result<(), String> {
    if items.is_empty() {
        return Ok(());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let chats_dir = get_chats_dir(&app_clone);
        let mut errors: Vec<String> = Vec::new();

        for item in items {
            let char_folder = &item.char_folder;
            let file_name = &item.file_name;

            // 安全校验
            if char_folder.trim().is_empty()
                || char_folder.contains("..")
                || char_folder.contains('/')
                || char_folder.contains('\\')
                || file_name.trim().is_empty()
                || file_name.contains("..")
                || file_name.contains('/')
                || file_name.contains('\\')
            {
                errors.push(format!("路径不合法: {}/{}", char_folder, file_name));
                continue;
            }

            let file_path = chats_dir.join(char_folder).join(file_name);
            if file_path.exists() {
                if let Err(e) = fs::remove_file(&file_path) {
                    errors.push(format!("无法删除 {}/{}: {}", char_folder, file_name, e));
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
