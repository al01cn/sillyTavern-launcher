use jwalk::WalkDirGeneric;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use sysinfo::Disks;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};
use walkdir::WalkDir as SyncWalkDir;

use crate::config::read_app_config_from_disk;
use crate::utils::get_config_path;

#[derive(Clone, serde::Serialize)]
pub struct ScanProgress {
    pub key: String,
    pub count: usize,
    pub found: usize,
    pub is_done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_path: Option<String>,
}

use crate::types::LocalTavernItem;

static SCAN_CANCEL_FLAG: AtomicBool = AtomicBool::new(false);
static SCAN_RUNNING_FLAG: AtomicBool = AtomicBool::new(false);

#[derive(Deserialize)]
struct PackageJson {
    name: Option<String>,
}

fn is_st_package(path: &Path) -> bool {
    if path.file_name().and_then(|s| s.to_str()) != Some("package.json") {
        return false;
    }
    // 跳过大于 6KB 的 package.json（正常的 SillyTavern package.json 远小于此值）
    if let Ok(md) = fs::metadata(path) {
        if md.len() > 6 * 1024 {
            return false;
        }
    }
    if let Ok(content) = fs::read_to_string(path) {
        // Case-insensitive check: "SillyTavern" or "sillytavern"
        let content_lower = content.to_lowercase();
        if content_lower.contains("\"name\":") && content_lower.contains("\"sillytavern\"") {
            let pkg: Result<PackageJson, _> = serde_json::from_str(&content);
            return pkg.map_or(false, |p| {
                p.name
                    .map_or(false, |n| n.eq_ignore_ascii_case("sillytavern"))
            });
        }
    }
    false
}

/// 黑名单目录名（小写），仅匹配单层文件夹名称
const BLACK_LIST: &[&str] = &[
    // 基础开发与编译相关
    "node_modules",
    "target",
    "dist",
    "build",
    "cache",
    "tmp",
    "temp",
    "site-packages",
    "venv",
    "env",
    "virtualenv",
    "conda",
    "miniconda3",
    "anaconda3",
    "out",
    "bin",
    "lib",
    "pkg",
    "vendor",
    // Windows 系统、驱动、保护目录
    "windows",
    "windows.old",
    "microsoft",
    "microsoft.net",
    "system volume information",
    "$recycle.bin",
    "recovery",
    "documents and settings",
    "perflogs",
    "intel",
    "amd",
    "nvidia",
    "driverstore",
    "boot",
    "efi",
    "msocache",
    // 应用安装、缓存、配置相关
    // 移除了 "users"，因为用户非常有可能将 SillyTavern 放在桌面、下载等用户家目录下
    "program files",
    "program files (x86)",
    "programdata",
    "appdata",
    "application data",
    "localappdata",
    "roaming",
    "local",
    "locallow",
    // 常见的大型游戏平台目录（显著加快扫描速度）
    "steam",
    "steamapps",
    "common",
    "origin games",
    "epic games",
    "ubisoft",
    "battlenet",
    "riot games",
    // 语言环境/包管理器缓存与安装目录
    "go",
    "python",
    "rustup",
    "yarn",
    "npm-cache",
    "pnpm-store",
    // 虚拟机、容器、辅助工具缓存
    "docker",
    "containers",
    "wsl",
    "vmware",
    "virtualbox vms",
    "hyper-v",
    // 常见个人媒体大文件夹（通常不会在这放代码）
    "pictures",
    "videos",
    "music",
    "3d objects",
    "saved games",
    "contacts",
    "searches",
    "onedrive",
    "onedrivetemp",
    "dropbox",
    "google drive",
];

fn should_skip_dir(name: &str) -> bool {
    if name.starts_with('.') {
        return true;
    }
    let name_lower = name.to_lowercase();
    BLACK_LIST.contains(&name_lower.as_str())
}

#[tauri::command]
pub async fn cancel_scan_local_sillytavern() -> Result<(), String> {
    SCAN_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn scan_local_sillytavern(app: AppHandle) -> Result<(), String> {
    if SCAN_RUNNING_FLAG.load(Ordering::SeqCst) {
        return Err("Scan already running".to_string());
    }
    SCAN_CANCEL_FLAG.store(false, Ordering::SeqCst);
    SCAN_RUNNING_FLAG.store(true, Ordering::SeqCst);

    let app_clone = app.clone();
    let app_timer = app.clone();

    // --- Timer Task ---
    tokio::spawn(async move {
        let mut seconds = 0;
        while SCAN_RUNNING_FLAG.load(Ordering::SeqCst) && !SCAN_CANCEL_FLAG.load(Ordering::SeqCst) {
            let mins = seconds / 60;
            let secs = seconds % 60;
            let time_str = format!("{:02}:{:02}", mins, secs);
            let _ = app_timer.emit("scan-local-sillytavern-timer", time_str);
            sleep(Duration::from_secs(1)).await;
            seconds += 1;
        }
    });

    let config = read_app_config_from_disk(&app);
    // 获取安装目录下的 data/sillytavern 路径，用于排除在线下载的版本
    let config_path = get_config_path(&app);
    let data_dir = config_path
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let install_st_dir = data_dir.join("sillytavern");
    // 统一转小写 + 正斜杠，用于 starts_with 比较
    let install_st_dir_normalized = install_st_dir
        .to_string_lossy()
        .to_lowercase()
        .replace('\\', "/");
    let install_st_dir_prefix = if install_st_dir_normalized.ends_with('/') {
        install_st_dir_normalized
    } else {
        format!("{}/", install_st_dir_normalized)
    };
    tracing::info!("扫描排除路径: {}", install_st_dir_prefix);
    tracing::info!("当前工作目录 (cwd): {:?}", std::env::current_dir());
    let install_st_dir_arc = Arc::new(install_st_dir_prefix);
    let _ = app_clone.emit(
        "scan-local-sillytavern-progress",
        ScanProgress {
            key: "versions.scanPreparing".to_string(),
            count: 0,
            found: 0,
            is_done: false,
            current_path: None,
        },
    );

    tokio::task::spawn_blocking(move || {
        let counter = Arc::new(AtomicUsize::new(0));
        let last_emit = Arc::new(std::sync::Mutex::new(std::time::Instant::now()));
        let disks = Disks::new_with_refreshed_list();
        let scan_roots: Vec<PathBuf> = disks
            .iter()
            .map(|d| d.mount_point().to_path_buf())
            .collect();
        for r in &scan_roots {
            tracing::info!("扫描根目录: {:?}", r);
        }

        let existing_paths: Vec<String> = config
            .local_sillytavern_list
            .iter()
            .map(|item| item.path.clone())
            .collect();
        let existing_paths_arc = Arc::new(existing_paths);

        // --- 先尝试 jwalk 并行扫描，10 秒内无进度则切换到 walkdir ---
        let counter_for_timeout = counter.clone();
        let timeout_elapsed = Arc::new(AtomicBool::new(false));
        let jwalk_done = Arc::new(AtomicBool::new(false));

        let use_walkdir = {
            // 启动超时检测线程
            let timeout_flag = timeout_elapsed.clone();
            let done_flag = jwalk_done.clone();
            let timeout_thread = std::thread::spawn(move || {
                // 每 0.5 秒检查一次
                let mut last_count = 0;
                let mut no_progress_ticks = 0;
                loop {
                    if done_flag.load(Ordering::SeqCst) {
                        return; // 正常完成，退出
                    }

                    let current_count = counter_for_timeout.load(Ordering::Relaxed);
                    if current_count == last_count {
                        no_progress_ticks += 1; // 0.5s per tick
                        if no_progress_ticks >= 20 {
                            // 10秒无任何进度
                            timeout_flag.store(true, Ordering::SeqCst);
                            tracing::warn!("jwalk 超时（10 秒无进展），将切换到 walkdir");
                            return;
                        }
                    } else {
                        last_count = current_count;
                        no_progress_ticks = 0;
                    }

                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            });

            // jwalk 扫描（会在 timeout_elapsed 时主动退出）
            let existing_paths_jwalk = existing_paths_arc.clone();
            let install_st_dir_jwalk = install_st_dir_arc.clone();

            let jwalk_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let results: Vec<String> = scan_roots
                    .iter()
                    .flat_map(|root| {
                        if SCAN_CANCEL_FLAG.load(Ordering::SeqCst)
                            || timeout_elapsed.load(Ordering::SeqCst)
                        {
                            return Vec::new();
                        }
                        tracing::info!("[jwalk] 开始扫描磁盘: {:?}", root);
                        let mut found = Vec::new();
                        let counter_clone = counter.clone();
                        let app_progress = app_clone.clone();
                        let existing_paths_ref = existing_paths_jwalk.clone();
                        let install_st_dir_ref = install_st_dir_jwalk.clone();
                        let install_st_dir_ref2 = install_st_dir_jwalk.clone();
                        let last_emit_ref = last_emit.clone();
                        let timeout_ref = timeout_elapsed.clone();
                        let needs_elevation = Arc::new(AtomicBool::new(false));
                        let needs_elevation_ref = needs_elevation.clone();

                        let iter = WalkDirGeneric::<((), bool)>::new(root)
                            .max_depth(10)
                            .process_read_dir(move |_, parent_path, _, children| {
                                // 跳过安装目录下 data/sillytavern/ 及其子目录
                                {
                                    let parent_str_lower: String = parent_path
                                        .to_string_lossy()
                                        .to_lowercase()
                                        .replace('\\', "/");
                                    if parent_str_lower.starts_with(&*install_st_dir_ref) {
                                        children.retain(|_| false);
                                        return;
                                    }
                                }
                                children.retain(|dir_entry_result| match dir_entry_result {
                                    Ok(e) => {
                                        let name = e.file_name().to_string_lossy();
                                        !should_skip_dir(&name)
                                    }
                                    Err(e) => {
                                        let err_msg = e.to_string().to_lowercase();
                                        if err_msg.contains("os error 5")
                                            || err_msg.contains("access is denied")
                                            || err_msg.contains("拒绝访问")
                                            || err_msg.contains("permission denied")
                                        {
                                            needs_elevation_ref.store(true, Ordering::Relaxed);
                                        }
                                        false
                                    }
                                });
                            });

                        for entry in iter {
                            if needs_elevation.load(Ordering::Relaxed)
                                && !crate::elevation::is_elevated()
                            {
                                tracing::warn!("[jwalk] 遇到无权限目录，正在尝试提权...");
                                let _ = crate::elevation::elevate_process(app_progress.clone());
                                break;
                            }
                            if SCAN_CANCEL_FLAG.load(Ordering::SeqCst)
                                || timeout_ref.load(Ordering::SeqCst)
                            {
                                break;
                            }
                            if let Ok(entry) = entry {
                                let path = entry.path();
                                let count = counter_clone.fetch_add(1, Ordering::Relaxed);

                                // 节流：每 200ms 发送一次进度事件
                                let now = std::time::Instant::now();
                                let should_emit = if let Ok(last) = last_emit_ref.lock() {
                                    count % 1000 == 0
                                        || now.duration_since(*last).as_millis() >= 200
                                } else {
                                    count % 1000 == 0
                                };
                                if should_emit {
                                    if let Ok(mut last) = last_emit_ref.lock() {
                                        *last = now;
                                    }
                                    let _ = app_progress.emit(
                                        "scan-local-sillytavern-progress",
                                        ScanProgress {
                                            key: "versions.scanProgress".to_string(),
                                            count,
                                            found: 0,
                                            is_done: false,
                                            current_path: Some(path.to_string_lossy().to_string()),
                                        },
                                    );
                                }

                                if let Some(parent) = process_entry(
                                    &path,
                                    &existing_paths_ref,
                                    &install_st_dir_ref2,
                                    &app_progress,
                                ) {
                                    found.push(parent);
                                }
                            }
                        }
                        found
                    })
                    .collect();
                results
            }));

            // jwalk 迭代完成，通知超时线程退出
            jwalk_done.store(true, Ordering::SeqCst);

            // 等待超时线程结束
            let _ = timeout_thread.join();

            // 如果超时了或者 jwalk 什么都没找到但也没被取消，则用 walkdir 重试
            let timed_out = timeout_elapsed.load(Ordering::SeqCst);
            let cancelled = SCAN_CANCEL_FLAG.load(Ordering::SeqCst);
            let has_panic = jwalk_result.is_err();
            let count_is_zero = counter.load(Ordering::Relaxed) == 0;
            let final_results = jwalk_result.unwrap_or_default();

            if cancelled {
                // 被取消
                let _ = app_clone.emit(
                    "scan-local-sillytavern-progress",
                    ScanProgress {
                        key: "versions.scanCancelled".to_string(),
                        count: counter.load(Ordering::Relaxed),
                        found: final_results.len(),
                        is_done: true,
                        current_path: None,
                    },
                );
                SCAN_RUNNING_FLAG.store(false, Ordering::SeqCst);
                false
            } else if timed_out || has_panic || count_is_zero {
                tracing::warn!(
                    "jwalk 扫描失败 (超时: {}, 崩溃: {}, 扫描数量为0: {})，回退到 walkdir",
                    timed_out,
                    has_panic,
                    count_is_zero
                );
                true
            } else {
                // jwalk 正常完成
                let _ = app_clone.emit(
                    "scan-local-sillytavern-progress",
                    ScanProgress {
                        key: "versions.scanFinished".to_string(),
                        count: counter.load(Ordering::Relaxed),
                        found: final_results.len(),
                        is_done: true,
                        current_path: None,
                    },
                );
                SCAN_RUNNING_FLAG.store(false, Ordering::SeqCst);
                false
            }
        };

        if use_walkdir {
            // --- walkdir fallback: 同步顺序扫描，不会卡死 ---
            tracing::info!("[walkdir] 开始顺序扫描...");
            counter.store(0, Ordering::Relaxed);

            let walkdir_results: Vec<String> = scan_roots
                .iter()
                .flat_map(|root| {
                    if SCAN_CANCEL_FLAG.load(Ordering::SeqCst) {
                        return Vec::new();
                    }
                    tracing::info!("[walkdir] 开始扫描磁盘: {:?}", root);
                    let mut found = Vec::new();

                    for entry in SyncWalkDir::new(root)
                        .max_depth(10)
                        .into_iter()
                        .filter_entry(|e| {
                            // 过滤目录
                            let is_dir = !e.file_type().is_file();
                            if is_dir {
                                let path_str =
                                    e.path().to_string_lossy().to_lowercase().replace('\\', "/");
                                if path_str.starts_with(&*install_st_dir_arc) {
                                    return false;
                                }
                                let name = e.file_name().to_string_lossy();
                                !should_skip_dir(&name)
                            } else {
                                true
                            }
                        })
                    {
                        if SCAN_CANCEL_FLAG.load(Ordering::SeqCst) {
                            break;
                        }
                        let entry = match entry {
                            Ok(e) => e,
                            Err(e) => {
                                let err_msg = e.to_string().to_lowercase();
                                if (err_msg.contains("os error 5")
                                    || err_msg.contains("access is denied")
                                    || err_msg.contains("拒绝访问")
                                    || err_msg.contains("permission denied"))
                                    && !crate::elevation::is_elevated()
                                {
                                    tracing::warn!("[walkdir] 遇到无权限目录，正在尝试提权...");
                                    let _ = crate::elevation::elevate_process(app_clone.clone());
                                }
                                continue;
                            }
                        };
                        let path = entry.path();
                        let count = counter.fetch_add(1, Ordering::Relaxed);

                        // 节流
                        let now = std::time::Instant::now();
                        let should_emit = if let Ok(last) = last_emit.lock() {
                            count % 1000 == 0 || now.duration_since(*last).as_millis() >= 200
                        } else {
                            count % 1000 == 0
                        };
                        if should_emit {
                            if let Ok(mut last) = last_emit.lock() {
                                *last = now;
                            }
                            let _ = app_clone.emit(
                                "scan-local-sillytavern-progress",
                                ScanProgress {
                                    key: "versions.scanProgress".to_string(),
                                    count,
                                    found: 0,
                                    is_done: false,
                                    current_path: Some(path.to_string_lossy().to_string()),
                                },
                            );
                        }

                        if let Some(parent) = process_entry(
                            path,
                            &existing_paths_arc,
                            &install_st_dir_arc,
                            &app_clone,
                        ) {
                            found.push(parent);
                        }
                    }
                    found
                })
                .collect();

            let key = if SCAN_CANCEL_FLAG.load(Ordering::SeqCst) {
                "versions.scanCancelled"
            } else {
                "versions.scanFinished"
            };
            let _ = app_clone.emit(
                "scan-local-sillytavern-progress",
                ScanProgress {
                    key: key.to_string(),
                    count: counter.load(Ordering::Relaxed),
                    found: walkdir_results.len(),
                    is_done: true,
                    current_path: None,
                },
            );
            SCAN_RUNNING_FLAG.store(false, Ordering::SeqCst);
        }
    });

    Ok(())
}

/// 处理单个文件条目：检查是否为 SillyTavern package.json
/// 如果是，返回父目录路径；否则返回 None
fn process_entry(
    path: &Path,
    existing_paths: &[String],
    install_st_dir_prefix: &str,
    app: &AppHandle,
) -> Option<String> {
    if !path.is_file() || !is_st_package(path) {
        return None;
    }

    let parent = path.parent()?;
    let parent_str = parent.to_string_lossy().to_lowercase().replace('\\', "/");
    if parent_str.starts_with(install_st_dir_prefix) {
        return None;
    }

    let mut abs_str = fs::canonicalize(parent)
        .unwrap_or_else(|_| parent.to_path_buf())
        .to_string_lossy()
        .into_owned();
    if abs_str.starts_with(r"\\?\") {
        abs_str = abs_str[4..].to_string();
    }

    // 去重
    let is_existing = existing_paths.iter().any(|p| {
        if let (Ok(p1), Ok(p2)) = (std::fs::canonicalize(p), std::fs::canonicalize(&abs_str)) {
            p1 == p2
        } else {
            p.to_lowercase() == abs_str.to_lowercase()
        }
    });
    if is_existing {
        return None;
    }

    // 获取 version
    let mut version_str = String::from("unknown");
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(v_str) = v.get("version").and_then(|v| v.as_str()) {
                version_str = v_str.to_string();
            }
        }
    }

    let has_node_modules = {
        let nm = parent.join("node_modules");
        nm.exists()
            && std::fs::read_dir(&nm)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false)
    };

    let _ = app.emit(
        "scan-local-sillytavern-found",
        LocalTavernItem {
            path: abs_str.clone(),
            version: version_str,
            has_node_modules,
        },
    );

    Some(abs_str)
}
