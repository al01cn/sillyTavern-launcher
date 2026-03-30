use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use crate::config::{get_current_lang, read_app_config_from_disk};
use crate::types::{Lang, GitInfo, DownloadProgress};
use crate::utils::get_config_path;

/// 内置 MinGit 的可执行文件路径
fn local_git_path(app: &AppHandle) -> PathBuf {
    let data_dir = get_config_path(app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let git_dir = data_dir.join("git");
    if cfg!(target_os = "windows") {
        git_dir.join("cmd/git.exe")
    } else {
        git_dir.join("bin/git")
    }
}

/// 检测系统 Git 是否存在，返回 PathBuf（"git"）或 None
fn has_system_git() -> bool {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    { use std::os::windows::process::CommandExt; cmd.creation_flags(0x08000000); }
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

/// 获取可用的 git 可执行文件路径。
/// 按 use_system_git 配置决定优先级（系统优先 or 内置优先），均不可用时回退到 "git"。
pub fn get_git_exe(app: &AppHandle) -> PathBuf {
    let use_system = read_app_config_from_disk(app).use_system_git;

    let local = local_git_path(app);
    let local_exists = local.exists();

    if use_system {
        // 系统优先
        if has_system_git() {
            return PathBuf::from("git");
        }
        if local_exists { return local; }
    } else {
        // 内置优先
        if local_exists { return local; }
        if has_system_git() {
            return PathBuf::from("git");
        }
    }
    PathBuf::from("git") // 都没有，让调用方报错
}

/// 同时检测系统 Git 和内置 Git，用于前端展示切换按钮
#[tauri::command]
pub async fn check_git_both(app: AppHandle) -> Result<serde_json::Value, String> {
    let local_path = local_git_path(&app);

    // ── 系统 Git ──
    let system_git: Option<GitInfo> = {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("--version").stdin(std::process::Stdio::null());
        #[cfg(target_os = "windows")]
        { use std::os::windows::process::CommandExt; cmd.creation_flags(0x08000000); }
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let ver_raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let ver = ver_raw.replace("git version ", "").trim().to_string();

                // 取路径
                let path_cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
                let mut pc = std::process::Command::new(path_cmd);
                pc.arg("git").stdin(std::process::Stdio::null());
                #[cfg(target_os = "windows")]
                { use std::os::windows::process::CommandExt; pc.creation_flags(0x08000000); }
                let mut git_path = "system".to_string();
                if let Ok(po) = pc.output() {
                    if po.status.success() {
                        let ps = String::from_utf8_lossy(&po.stdout);
                        if let Some(l) = ps.lines().next() {
                            let t = l.trim();
                            if !t.is_empty() { git_path = t.replace('\\', "/"); }
                        }
                    }
                }
                // 排除内置路径被误识别为系统 Git
                let local_norm = local_path.to_string_lossy().replace('\\', "/").to_lowercase();
                let found_norm = git_path.to_lowercase();
                if found_norm != local_norm {
                    Some(GitInfo { version: Some(ver), path: Some(git_path), source: "system".to_string() })
                } else {
                    None
                }
            } else { None }
        } else { None }
    };

    // ── 内置 MinGit ──
    let local_git: Option<GitInfo> = if local_path.exists() {
        let mut cmd = std::process::Command::new(&local_path);
        cmd.arg("--version").stdin(std::process::Stdio::null());
        #[cfg(target_os = "windows")]
        { use std::os::windows::process::CommandExt; cmd.creation_flags(0x08000000); }
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let ver_raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let ver = ver_raw.replace("git version ", "").trim().to_string();
                Some(GitInfo {
                    version: Some(ver),
                    path: Some(local_path.to_string_lossy().replace('\\', "/")),
                    source: "local".to_string(),
                })
            } else { None }
        } else { None }
    } else { None };

    Ok(serde_json::json!({ "system": system_git, "local": local_git }))
}



#[tauri::command]
pub async fn check_git(app: AppHandle) -> Result<GitInfo, String> {
    // 直接复用 get_git_exe 获取实际可用的 git 可执行文件路径，
    // 保证检测结果与实际使用（clone/install 等操作）完全一致。
    let git_exe = get_git_exe(&app);
    let git_exe_str = git_exe.to_string_lossy().to_string();

    // 判断来源：是系统 Git（"git"）还是内置 MinGit（绝对路径）
    let is_system = git_exe_str == "git";

    let mut command = std::process::Command::new(&git_exe);
    command.arg("--version")
        .stdin(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }

    if let Ok(output) = command.output() {
        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let version = version_output.replace("git version ", "").trim().to_string();

            let (path, source) = if is_system {
                // 系统 Git：通过 where/which 获取实际路径
                let path_cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
                let mut path_command = std::process::Command::new(path_cmd);
                path_command.arg("git").stdin(std::process::Stdio::null());
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    path_command.creation_flags(0x08000000);
                }
                let git_path = if let Ok(path_output) = path_command.output() {
                    if path_output.status.success() {
                        let path_str = String::from_utf8_lossy(&path_output.stdout);
                        path_str.lines().next()
                            .map(|l| l.trim().replace('\\', "/"))
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| "system".to_string())
                    } else {
                        "system".to_string()
                    }
                } else {
                    "system".to_string()
                };
                (git_path, "system".to_string())
            } else {
                // 内置 MinGit：直接用绝对路径
                (git_exe_str.replace('\\', "/"), "local".to_string())
            };

            return Ok(GitInfo {
                version: Some(version),
                path: Some(path),
                source,
            });
        }
    }

    // get_git_exe 找不到可用 git（系统无 git 且内置 MinGit 不存在）
    Ok(GitInfo {
        version: None,
        path: None,
        source: "none".to_string(),
    })
}

#[tauri::command]
pub async fn install_git(app: AppHandle) -> Result<(), String> {
    let lang = get_current_lang(&app);
    let os = std::env::consts::OS;

    let emit_progress = |status: &str, progress: f64, log: &str| {
        let _ = app.emit(
            "download-progress",
            DownloadProgress {
                status: status.to_string(),
                progress,
                log: log.to_string(),
            },
        );
    };

    if os == "windows" {
        // Try MinGit first
        let app_clone_mingit = app.clone();
        let mingit_result = async move {
            let app = app_clone_mingit;
            let emit_progress = |status: &str, progress: f64, log: &str| {
                let _ = app.emit(
                    "download-progress",
                    DownloadProgress {
                        status: status.to_string(),
                        progress,
                        log: log.to_string(),
                    },
                );
            };

            emit_progress("downloading", 0.0, &match lang {
                Lang::ZhCn => "开始下载 MinGit (轻量版 Git)...".to_string(),
                Lang::EnUs => "Downloading MinGit (Lightweight Git)...".to_string(),
            });

            let target_url = "https://github.com/git-for-windows/git/releases/download/v2.53.0.windows.2/MinGit-2.53.0.2-64-bit.zip".to_string();

            let temp_dir = std::env::temp_dir();
            let zip_path = temp_dir.join("MinGit.zip");

            let client = reqwest::Client::builder()
                .user_agent("sillyTavern-launcher")
                .build()
                .map_err(|e| e.to_string())?;

            let proxy = crate::utils::GithubProxy::new(&app).await;
            let (fastest_url, response) = proxy.get_fastest_stream(client, &target_url).await.map_err(|e| e.to_string())?;
            
            match lang {
                Lang::ZhCn => tracing::info!("使用下载节点: {}", fastest_url),
                Lang::EnUs => tracing::info!("Using download mirror: {}", fastest_url),
            }

            let total_size = response.content_length().unwrap_or(0);

            let mut file = tokio::fs::File::create(&zip_path).await.map_err(|e| e.to_string())?;
            let mut downloaded: u64 = 0;
            use futures_util::StreamExt;
            let mut stream = response.bytes_stream();
            let mut last_emit = std::time::Instant::now();

            while let Some(item) = stream.next().await {
                let chunk = item.map_err(|e| e.to_string())?;
                use tokio::io::AsyncWriteExt;
                file.write_all(&chunk).await.map_err(|e| e.to_string())?;
                downloaded += chunk.len() as u64;

                if last_emit.elapsed() > std::time::Duration::from_millis(150) || downloaded == total_size {
                    let progress = if total_size > 0 {
                        (downloaded as f64) / (total_size as f64)
                    } else {
                        0.0
                    };

                    let mb_downloaded = downloaded as f64 / 1_048_576.0;
                    let mb_total = total_size as f64 / 1_048_576.0;
                    emit_progress("downloading", progress, &match lang {
                        Lang::ZhCn => if total_size > 0 { format!("已下载: {:.2} MB / {:.2} MB", mb_downloaded, mb_total) } else { format!("已下载: {:.2} MB", mb_downloaded) },
                        Lang::EnUs => if total_size > 0 { format!("Downloaded: {:.2} MB / {:.2} MB", mb_downloaded, mb_total) } else { format!("Downloaded: {:.2} MB", mb_downloaded) },
                    });
                    last_emit = std::time::Instant::now();
                }
            }

            drop(file);

            emit_progress("extracting", 0.0, &match lang {
                Lang::ZhCn => "下载完成，正在解压 MinGit...".to_string(),
                Lang::EnUs => "Download complete, extracting MinGit...".to_string(),
            });

            let data_dir = get_config_path(&app)
                .parent()
                .unwrap_or(&std::path::PathBuf::from("."))
                .to_path_buf();
            let git_dir = data_dir.join("git");

            if git_dir.exists() {
                let _ = tokio::fs::remove_dir_all(&git_dir).await;
            }
            let _ = tokio::fs::create_dir_all(&git_dir).await;

            let git_dir_clone = git_dir.clone();
            let zip_path_clone = zip_path.clone();
            let app_clone = app.clone();
            let lang_clone = lang;

            let _extract_result = tokio::task::spawn_blocking(move || -> Result<(), String> {
                let emit_progress = |status: &str, progress: f64, log: &str| {
                    let _ = app_clone.emit(
                        "download-progress",
                        crate::types::DownloadProgress {
                            status: status.to_string(),
                            progress,
                            log: log.to_string(),
                        },
                    );
                };

                let file = std::fs::File::open(&zip_path_clone).map_err(|e| e.to_string())?;
                let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
                let total_files = archive.len();

                for i in 0..total_files {
                    let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
                    let outpath = match file.enclosed_name() {
                        Some(path) => path.to_owned(),
                        None => continue,
                    };

                    let target_path = git_dir_clone.join(&outpath);

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

                    if i % 50 == 0 || i == total_files - 1 {
                        let progress = (i as f64) / (total_files as f64);
                        emit_progress(
                            "extracting",
                            progress,
                            &match lang_clone {
                                Lang::ZhCn => format!("解压中: {}/{} 文件...", i + 1, total_files),
                                Lang::EnUs => format!("Extracting: {}/{} files...", i + 1, total_files),
                            },
                        );
                    }
                }
                Ok(())
            })
            .await
            .map_err(|e| e.to_string())??;

            let _ = tokio::fs::remove_file(zip_path).await;

            emit_progress("done", 1.0, &match lang {
                Lang::ZhCn => "Git 安装完成".to_string(),
                Lang::EnUs => "Git installation complete".to_string(),
            });

            Ok::<(), String>(())
        }.await;

        if mingit_result.is_ok() {
            return Ok(());
        }

        tracing::warn!("MinGit install failed: {:?}, falling back to full git...", mingit_result);
        
        let emit_progress = |status: &str, progress: f64, log: &str| {
            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    status: status.to_string(),
                    progress,
                    log: log.to_string(),
                },
            );
        };
        
        emit_progress("installing", 0.0, &match lang {
            Lang::ZhCn => "MinGit 安装失败，准备回退到完整版 Git...".to_string(),
            Lang::EnUs => "MinGit installation failed, falling back to full Git...".to_string(),
        });

        // check winget
        let mut winget_cmd = std::process::Command::new("winget");
        winget_cmd.arg("--version");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            winget_cmd.creation_flags(0x08000000);
        }

        let winget_exists = winget_cmd.stdin(std::process::Stdio::null()).output().is_ok();

        if winget_exists {
            emit_progress("installing", 0.0, &match lang {
                Lang::ZhCn => "开始使用 winget 安装 Git...".to_string(),
                Lang::EnUs => "Starting to install Git using winget...".to_string(),
            });

            let mut original_url: Option<String> = None;
            if lang == Lang::ZhCn {
                // Get original winget source URL before modifying
                let mut list_cmd = std::process::Command::new("winget");
                list_cmd.args(&["source", "list", "-n", "winget"]);
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    list_cmd.creation_flags(0x08000000);
                }
                if let Ok(output) = list_cmd.stdin(std::process::Stdio::null()).output() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for line in output_str.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for part in parts {
                            if part.starts_with("https://") {
                                original_url = Some(part.to_string());
                            }
                        }
                    }
                }

                emit_progress("installing", 0.1, "正在配置 winget 国内镜像源...");
                // winget source remove winget
                let mut remove_cmd = std::process::Command::new("winget");
                remove_cmd.args(&["source", "remove", "winget"]);
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    remove_cmd.creation_flags(0x08000000);
                }
                let _ = remove_cmd.stdin(std::process::Stdio::null()).output();

                // winget source add winget https://mirrors.ustc.edu.cn/winget-source --trust-level trusted
                let mut add_cmd = std::process::Command::new("winget");
                add_cmd.args(&["source", "add", "winget", "https://mirrors.ustc.edu.cn/winget-source", "--trust-level", "trusted"]);
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    add_cmd.creation_flags(0x08000000);
                }
                let _ = add_cmd.stdin(std::process::Stdio::null()).output();
            }

            emit_progress("installing", 0.3, &match lang {
                Lang::ZhCn => "正在执行安装 (最多等待1分钟)...".to_string(),
                Lang::EnUs => "Executing installation (waiting up to 1 minute)...".to_string(),
            });

            // winget install --id Git.Git -e --source winget
            let mut install_cmd = tokio::process::Command::new("winget");
            install_cmd.args(&["install", "--id", "Git.Git", "-e", "--source", "winget"]);
            install_cmd.kill_on_drop(true);
            #[cfg(target_os = "windows")]
            {
                install_cmd.creation_flags(0x08000000);
            }

            let install_future = install_cmd.output();
            let timeout_result = tokio::time::timeout(std::time::Duration::from_secs(60), install_future).await;

            if lang == Lang::ZhCn {
                emit_progress("installing", 0.9, "正在还原 winget 镜像源...");
                let should_reset_to_default = match &original_url {
                    Some(url) => url.contains("cdn.winget.microsoft.com") || url.contains("storeedgefd.dsx.mp.microsoft.com"),
                    None => true,
                };

                if should_reset_to_default {
                    let mut reset_cmd = std::process::Command::new("winget");
                    reset_cmd.args(&["source", "reset", "-n", "winget", "--force"]);
                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;
                        reset_cmd.creation_flags(0x08000000);
                    }
                    let _ = reset_cmd.stdin(std::process::Stdio::null()).output();
                } else if let Some(url) = original_url {
                    if url != "https://mirrors.ustc.edu.cn/winget-source" {
                        // remove current and add the original custom one back
                        let mut remove_cmd = std::process::Command::new("winget");
                        remove_cmd.args(&["source", "remove", "winget"]);
                        #[cfg(target_os = "windows")]
                        {
                            use std::os::windows::process::CommandExt;
                            remove_cmd.creation_flags(0x08000000);
                        }
                        let _ = remove_cmd.stdin(std::process::Stdio::null()).output();

                        let mut add_cmd = std::process::Command::new("winget");
                        add_cmd.args(&["source", "add", "winget", &url, "--trust-level", "trusted"]);
                        #[cfg(target_os = "windows")]
                        {
                            use std::os::windows::process::CommandExt;
                            add_cmd.creation_flags(0x08000000);
                        }
                        let _ = add_cmd.stdin(std::process::Stdio::null()).output();
                    }
                }
            }

            match timeout_result {
                Ok(Ok(output)) => {
                    if output.status.success() {
                        emit_progress("done", 1.0, &match lang {
                            Lang::ZhCn => "Git 安装完成".to_string(),
                            Lang::EnUs => "Git installation complete".to_string(),
                        });
                        return Ok(());
                    } else {
                        tracing::warn!("winget install failed: {}", String::from_utf8_lossy(&output.stderr));
                        // fallback to portable git
                    }
                }
                Ok(Err(e)) => {
                    tracing::warn!("winget execute failed: {}", e);
                    // fallback to portable git
                }
                Err(_) => {
                    tracing::warn!("winget install timed out after 60 seconds");
                    // fallback to portable git
                }
            }
        }

        // Portable Git Fallback
        emit_progress("downloading", 0.0, &match lang {
            Lang::ZhCn => "未找到 winget，准备下载便携版 Git...".to_string(),
            Lang::EnUs => "winget not found, preparing to download portable Git...".to_string(),
        });

        let target_url = "https://github.com/git-for-windows/git/releases/download/v2.45.2.windows.1/PortableGit-2.45.2-64-bit.7z.exe".to_string();

        let temp_dir = std::env::temp_dir();
        let exe_path = temp_dir.join("PortableGit.exe");

        let client = reqwest::Client::builder()
            .user_agent("sillyTavern-launcher")
            .build()
            .map_err(|e| e.to_string())?;

        let proxy = crate::utils::GithubProxy::new(&app).await;
        let (fastest_url, response) = proxy.get_fastest_stream(client, &target_url).await.map_err(|e| e.to_string())?;

        match lang {
            Lang::ZhCn => tracing::info!("使用下载节点: {}", fastest_url),
            Lang::EnUs => tracing::info!("Using download mirror: {}", fastest_url),
        }

        let total_size = response.content_length().unwrap_or(0);

        let mut file = tokio::fs::File::create(&exe_path).await.map_err(|e| e.to_string())?;
        let mut downloaded: u64 = 0;
        use futures_util::StreamExt;
        let mut stream = response.bytes_stream();
        let mut last_emit = std::time::Instant::now();

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| e.to_string())?;
            use tokio::io::AsyncWriteExt;
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            downloaded += chunk.len() as u64;

            if last_emit.elapsed() > std::time::Duration::from_millis(150) || downloaded == total_size {
                let progress = if total_size > 0 {
                    (downloaded as f64) / (total_size as f64)
                } else {
                    0.0
                };

                let mb_downloaded = downloaded as f64 / 1_048_576.0;
                let mb_total = total_size as f64 / 1_048_576.0;
                emit_progress("downloading", progress, &match lang {
                    Lang::ZhCn => if total_size > 0 { format!("已下载: {:.2} MB / {:.2} MB", mb_downloaded, mb_total) } else { format!("已下载: {:.2} MB", mb_downloaded) },
                    Lang::EnUs => if total_size > 0 { format!("Downloaded: {:.2} MB / {:.2} MB", mb_downloaded, mb_total) } else { format!("Downloaded: {:.2} MB", mb_downloaded) },
                });
                last_emit = std::time::Instant::now();
            }
        }

        drop(file); // Ensure the downloaded file handle is closed and flushed!

        emit_progress("extracting", 0.0, &match lang {
            Lang::ZhCn => "下载完成，正在解压 (文件较多，这可能需要几分钟的时间)...".to_string(),
            Lang::EnUs => "Download complete, extracting (this may take a few minutes)...".to_string(),
        });

        let data_dir = get_config_path(&app)
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        let git_dir = data_dir.join("git");

        if git_dir.exists() {
            let _ = tokio::fs::remove_dir_all(&git_dir).await;
        }
        let _ = tokio::fs::create_dir_all(&git_dir).await;

        let git_dir_clone = git_dir.clone();
        let exe_path_clone = exe_path.clone();
        let temp_dir_clone = temp_dir.clone();
        let app_clone = app.clone();
        let is_zh_cn = lang == Lang::ZhCn;

        let extract_task = tokio::task::spawn_blocking(move || -> std::io::Result<()> {
            let mut f = std::fs::File::open(&exe_path_clone)?;
            let mut data = Vec::new();
            use std::io::Read;
            f.read_to_end(&mut data)?;

            // 7z file header magic
            let magic = b"7z\xBC\xAF\x27\x1C";

            let pos = data
                .windows(magic.len())
                .position(|w| w == magic)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Cannot find 7z header"))?;

            let inner_7z_path = temp_dir_clone.join("inner_git.7z");
            {
                use std::io::Write;
                let mut out = std::fs::File::create(&inner_7z_path)?;
                out.write_all(&data[pos..])?;
            }

            let mut last_log = std::time::Instant::now();
            let mut extracted_count = 0;

            sevenz_rust::decompress_file_with_extract_fn(&inner_7z_path, git_dir_clone, |entry, reader, dest| {
                extracted_count += 1;
                if last_log.elapsed() > std::time::Duration::from_millis(150) {
                    use tauri::Emitter;
                    let msg = if is_zh_cn {
                        format!("正在解压: {} (已解压 {} 个文件)...", entry.name(), extracted_count)
                    } else {
                        format!("Extracting: {} ({} files extracted)...", entry.name(), extracted_count)
                    };
                    let _ = app_clone.emit("download-progress", crate::types::DownloadProgress {
                        status: "extracting".to_string(),
                        progress: 0.0,
                        log: msg,
                    });
                    last_log = std::time::Instant::now();
                }
                sevenz_rust::default_entry_extract_fn(entry, reader, dest)
            }).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })?;

            let _ = std::fs::remove_file(&inner_7z_path);
            Ok(())
        });

        if let Err(e) = extract_task.await.map_err(|e| e.to_string())? {
            return Err(format!("Extracting failed: {}", e));
        }

        let _ = tokio::fs::remove_file(exe_path).await;

        emit_progress("done", 1.0, &match lang {
            Lang::ZhCn => "Git 安装完成".to_string(),
            Lang::EnUs => "Git installation complete".to_string(),
        });

        return Ok(());
    } else if os == "macos" {
        emit_progress("installing", 0.1, &match lang {
            Lang::ZhCn => "正在通过 Homebrew 安装 Git...".to_string(),
            Lang::EnUs => "Installing Git via Homebrew...".to_string(),
        });

        let mut brew_cmd = tokio::process::Command::new("brew");
        brew_cmd.args(&["install", "git"]);
        let output = brew_cmd.output().await.map_err(|e| e.to_string())?;
        if output.status.success() {
            emit_progress("done", 1.0, &match lang {
                Lang::ZhCn => "Git 安装完成".to_string(),
                Lang::EnUs => "Git installation complete".to_string(),
            });
            return Ok(());
        } else {
            return Err(format!("Brew install failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
    } else if os == "linux" {
        emit_progress("installing", 0.1, &match lang {
            Lang::ZhCn => "正在通过系统包管理器安装 Git...".to_string(),
            Lang::EnUs => "Installing Git via system package manager...".to_string(),
        });

        let pkg_managers = [
            ("apt-get", vec!["install", "-y", "git"]),
            ("pacman", vec!["-S", "--noconfirm", "git"]),
            ("dnf", vec!["install", "-y", "git"]),
            ("yum", vec!["install", "-y", "git"]),
            ("zypper", vec!["install", "-y", "git"]),
        ];

        for (pkg, args) in pkg_managers.iter() {
            let mut check_cmd = std::process::Command::new(pkg);
            check_cmd.arg("--version");
            if check_cmd.stdin(std::process::Stdio::null()).output().is_ok() {
                let mut pk_cmd = tokio::process::Command::new(pkg);
                pk_cmd.args(args);
                let output = pk_cmd.output().await.map_err(|e| e.to_string())?;
                if output.status.success() {
                    emit_progress("done", 1.0, &match lang {
                        Lang::ZhCn => "Git 安装完成".to_string(),
                        Lang::EnUs => "Git installation complete".to_string(),
                    });
                    return Ok(());
                } else {
                    return Err(format!("{} install failed: {}", pkg, String::from_utf8_lossy(&output.stderr)));
                }
            }
        }
        return Err("No supported package manager found on this Linux system".to_string());
    }

    Err(format!("Unsupported OS: {}", os))
}