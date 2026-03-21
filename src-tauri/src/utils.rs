use chrono::Local;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::types::AppConfig;

// ─────────────────────────────────────────────
// 日志初始化
// ─────────────────────────────────────────────

pub fn init_logger(data_dir: &Path) {
    let logs_dir = data_dir.join("logs");
    if !logs_dir.exists() {
        let _ = fs::create_dir_all(&logs_dir);
    }

    // 每天保留最新的日志内容，删除非今天的旧日志
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_file_name = format!("{}.log", today);

    if let Ok(entries) = fs::read_dir(&logs_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.ends_with(".log") && file_name != log_file_name {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    let file_appender = tracing_appender::rolling::never(&logs_dir, &log_file_name);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 将 _guard 泄漏到全局，防止 drop 导致最后日志丢失
    Box::leak(Box::new(_guard));

    let subscriber = tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

// ─────────────────────────────────────────────
// 标准目录布局
// ─────────────────────────────────────────────

pub fn ensure_file_with_default(path: &Path, content: &str) -> io::Result<()> {
    if !path.exists() {
        fs::write(path, content)?;
    }
    Ok(())
}

pub fn ensure_standard_layout(base_dir: &Path) -> io::Result<()> {
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

    ensure_file_with_default(&config_path, &default_config_str)?;

    Ok(())
}

// ─────────────────────────────────────────────
// 配置文件路径
// ─────────────────────────────────────────────

pub fn get_config_path(_app: &tauri::AppHandle) -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if path.ends_with("src-tauri") {
        path.pop();
    }

    path.join("data/config.json")
}
