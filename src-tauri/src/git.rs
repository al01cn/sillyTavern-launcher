use crate::config::{get_current_lang, read_app_config_from_disk};
use crate::types::{DownloadProgress, GitInfo, Lang};
use crate::utils::get_config_path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};

/// Git/Node 安装取消标志（全局，跨 git.rs 和 node.rs 共用）
pub static INSTALL_CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

/// 终止占用指定路径的进程（Windows 使用 handle.exe 或 PowerShell）
#[cfg(target_os = "windows")]
async fn kill_processes_using_path(path: &PathBuf) -> Result<(), String> {
    let path_str = path.to_string_lossy();

    // 使用 PowerShell 查找并终止占用进程
    let ps_script = format!(
        "$path = '{}'; Get-Process | Where-Object {{ $_.Path -like \"$path*\" -or $_.Modules.FileName -like \"$path*\" }} | ForEach-Object {{ Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }}",
        path_str.replace("'", "''")
    );

    let mut cmd = tokio::process::Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-Command", &ps_script])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(0x08000000);

    let _ = cmd.status().await;

    // 额外尝试：使用 taskkill 终止 git.exe 进程
    let mut taskkill_cmd = tokio::process::Command::new("taskkill");
    taskkill_cmd
        .args(["/F", "/IM", "git.exe", "/T"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(0x08000000);
    let _ = taskkill_cmd.status().await;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
async fn kill_processes_using_path(_path: &PathBuf) -> Result<(), String> {
    // 非 Windows 平台暂不实现
    Ok(())
}

/// 取消当前 Git 或 Node.js 安装/下载
#[tauri::command]
pub fn cancel_git_node_install() -> Result<(), String> {
    INSTALL_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

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

/// 检测系统 Git 是否存在
pub fn has_system_git() -> bool {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
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
        if local_exists {
            return local;
        }
    } else {
        // 内置优先
        if local_exists {
            return local;
        }
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
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let ver_raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let ver = ver_raw.replace("git version ", "").trim().to_string();

                // 取路径
                let path_cmd = if cfg!(target_os = "windows") {
                    "where"
                } else {
                    "which"
                };
                let mut pc = std::process::Command::new(path_cmd);
                pc.arg("git").stdin(std::process::Stdio::null());
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    pc.creation_flags(0x08000000);
                }
                let mut git_path = "system".to_string();
                if let Ok(po) = pc.output() {
                    if po.status.success() {
                        let ps = String::from_utf8_lossy(&po.stdout);
                        if let Some(l) = ps.lines().next() {
                            let t = l.trim();
                            if !t.is_empty() {
                                git_path = t.replace('\\', "/");
                            }
                        }
                    }
                }
                // 排除内置路径被误识别为系统 Git
                let local_norm = local_path
                    .to_string_lossy()
                    .replace('\\', "/")
                    .to_lowercase();
                let found_norm = git_path.to_lowercase();
                if found_norm != local_norm {
                    Some(GitInfo {
                        version: Some(ver),
                        path: Some(git_path),
                        source: "system".to_string(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    // ── 内置 MinGit ──
    let local_git: Option<GitInfo> = if local_path.exists() {
        let mut cmd = std::process::Command::new(&local_path);
        cmd.arg("--version").stdin(std::process::Stdio::null());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let ver_raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let ver = ver_raw.replace("git version ", "").trim().to_string();
                Some(GitInfo {
                    version: Some(ver),
                    path: Some(local_path.to_string_lossy().replace('\\', "/")),
                    source: "local".to_string(),
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

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
    command.arg("--version").stdin(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }

    if let Ok(output) = command.output() {
        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let version = version_output
                .replace("git version ", "")
                .trim()
                .to_string();

            let (path, source) = if is_system {
                // 系统 Git：通过 where/which 获取实际路径
                let path_cmd = if cfg!(target_os = "windows") {
                    "where"
                } else {
                    "which"
                };
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
                        path_str
                            .lines()
                            .next()
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

    // 重置取消标志
    INSTALL_CANCEL_FLAG.store(false, Ordering::SeqCst);

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

            // 五阶回退下载策略：npmmirror → 清华镜像 → 华为镜像 → 加速地址 → 直连
            let github_url = "https://github.com/git-for-windows/git/releases/download/v2.53.0.windows.2/MinGit-2.53.0.2-64-bit.zip";
            let npmmirror_url = "https://registry.npmmirror.com/-/binary/git-for-windows/v2.53.0.windows.2/MinGit-2.53.0.2-64-bit.zip";
            let tsinghua_mirror = "https://mirrors.tuna.tsinghua.edu.cn/github-release/git-for-windows/git/Git%20for%20Windows%202.53.0%282%29/MinGit-2.53.0.2-64-bit.zip";
            let huawei_mirror = "https://mirrors.huaweicloud.com/git-for-windows/v2.53.0.windows.2/MinGit-2.53.0.2-64-bit.zip";

            let temp_dir = std::env::temp_dir();
            let zip_path = temp_dir.join("MinGit.zip");

            // 清理可能存在的旧临时文件（避免"文件被占用"错误）
            if zip_path.exists() {
                let _ = tokio::fs::remove_file(&zip_path).await;
            }

            let client = reqwest::Client::builder()
                .user_agent("sillyTavern-launcher")
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .map_err(|e| e.to_string())?;

            let mut download_result: Option<(String, reqwest::Response)> = None;

            // 尝试1：npmmirror（主用）
            match lang {
                Lang::ZhCn => tracing::info!("尝试从 npmmirror 下载 MinGit..."),
                Lang::EnUs => tracing::info!("Trying npmmirror for MinGit..."),
            }
            match client.get(npmmirror_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match lang {
                        Lang::ZhCn => tracing::info!("npmmirror 可用，开始下载"),
                        Lang::EnUs => tracing::info!("npmmirror available, downloading"),
                    }
                    download_result = Some((npmmirror_url.to_string(), resp));
                }
                Ok(resp) => {
                    tracing::warn!("npmmirror 返回状态: {}", resp.status());
                }
                Err(e) => {
                    tracing::warn!("npmmirror 下载失败: {}", e);
                }
            }

            // 尝试2：清华镜像（第一备用）
            if download_result.is_none() {
                match lang {
                    Lang::ZhCn => tracing::info!("尝试从清华镜像下载 MinGit..."),
                    Lang::EnUs => tracing::info!("Trying Tsinghua mirror for MinGit..."),
                }
                match client.get(tsinghua_mirror).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        match lang {
                            Lang::ZhCn => tracing::info!("清华镜像可用，开始下载"),
                            Lang::EnUs => tracing::info!("Tsinghua mirror available, downloading"),
                        }
                        download_result = Some((tsinghua_mirror.to_string(), resp));
                    }
                    Ok(resp) => {
                        tracing::warn!("清华镜像返回状态: {}", resp.status());
                    }
                    Err(e) => {
                        tracing::warn!("清华镜像下载失败: {}", e);
                    }
                }
            }

            // 尝试3：华为镜像（第二备用）
            if download_result.is_none() {
                match lang {
                    Lang::ZhCn => tracing::info!("尝试从华为镜像下载 MinGit..."),
                    Lang::EnUs => tracing::info!("Trying Huawei mirror for MinGit..."),
                }
                match client.get(huawei_mirror).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        match lang {
                            Lang::ZhCn => tracing::info!("华为镜像可用，开始下载"),
                            Lang::EnUs => tracing::info!("Huawei mirror available, downloading"),
                        }
                        download_result = Some((huawei_mirror.to_string(), resp));
                    }
                    Ok(resp) => {
                        tracing::warn!("华为镜像返回状态: {}", resp.status());
                    }
                    Err(e) => {
                        tracing::warn!("华为镜像下载失败: {}", e);
                    }
                }
            }

            // 尝试4：加速地址
            if download_result.is_none() {
                match lang {
                    Lang::ZhCn => tracing::info!("尝试使用加速地址下载..."),
                    Lang::EnUs => tracing::info!("Trying accelerated download..."),
                }
                let proxy = crate::utils::GithubProxy::new(&app).await;
                match proxy.get_fastest_stream(client.clone(), github_url).await {
                    Ok((url, resp)) => {
                        match lang {
                            Lang::ZhCn => tracing::info!("使用加速节点: {}", url),
                            Lang::EnUs => tracing::info!("Using accelerated mirror: {}", url),
                        }
                        download_result = Some((url, resp));
                    }
                    Err(e) => {
                        tracing::warn!("加速地址下载失败: {}", e);
                    }
                }
            }

            // 尝试5：直连 GitHub
            if download_result.is_none() {
                match lang {
                    Lang::ZhCn => tracing::info!("尝试直连 GitHub 下载..."),
                    Lang::EnUs => tracing::info!("Trying direct GitHub download..."),
                }
                match client.get(github_url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        match lang {
                            Lang::ZhCn => tracing::info!("直连 GitHub 可用"),
                            Lang::EnUs => tracing::info!("Direct GitHub connection available"),
                        }
                        download_result = Some((github_url.to_string(), resp));
                    }
                    Ok(resp) => {
                        tracing::warn!("直连 GitHub 返回状态: {}", resp.status());
                    }
                    Err(e) => {
                        tracing::warn!("直连 GitHub 失败: {}", e);
                    }
                }
            }

            // 都失败，提示用户检查网络
            let (fastest_url, response) = match download_result {
                Some(result) => result,
                None => {
                    return Err(match lang {
                        Lang::ZhCn => "网络连接失败，无法下载 MinGit。请检查网络设置或稍后重试。".to_string(),
                        Lang::EnUs => "Network connection failed. Unable to download MinGit. Please check your network settings and try again.".to_string(),
                    });
                }
            };

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
                // 检查取消标志
                if INSTALL_CANCEL_FLAG.load(Ordering::SeqCst) {
                    drop(file);
                    let _ = tokio::fs::remove_file(&zip_path).await;
                    emit_progress("cancelled", 0.0, match lang {
                        Lang::ZhCn => "下载已取消",
                        Lang::EnUs => "Download cancelled",
                    });
                    return Err(match lang {
                        Lang::ZhCn => "下载已取消".to_string(),
                        Lang::EnUs => "Download cancelled".to_string(),
                    });
                }
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

            // 如果 git 目录已存在，先终止占用进程，然后删除旧目录
            if git_dir.exists() {
                // 终止占用 git 目录的进程
                let _ = kill_processes_using_path(&git_dir).await;
                // 等待一下让进程完全退出
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                // 删除旧目录
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

        // MinGit 安装失败，直接返回错误（不再回退到 winget/PortableGit）
        if let Err(e) = mingit_result {
            return Err(e);
        }

        return Ok(());
    } else if os == "macos" {
        emit_progress(
            "installing",
            0.1,
            &match lang {
                Lang::ZhCn => "正在通过 Homebrew 安装 Git...".to_string(),
                Lang::EnUs => "Installing Git via Homebrew...".to_string(),
            },
        );

        let mut brew_cmd = tokio::process::Command::new("brew");
        brew_cmd.args(&["install", "git"]);
        let output = brew_cmd.output().await.map_err(|e| e.to_string())?;
        if output.status.success() {
            emit_progress(
                "done",
                1.0,
                &match lang {
                    Lang::ZhCn => "Git 安装完成".to_string(),
                    Lang::EnUs => "Git installation complete".to_string(),
                },
            );
            return Ok(());
        } else {
            return Err(format!(
                "Brew install failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    } else if os == "linux" {
        emit_progress(
            "installing",
            0.1,
            &match lang {
                Lang::ZhCn => "正在通过系统包管理器安装 Git...".to_string(),
                Lang::EnUs => "Installing Git via system package manager...".to_string(),
            },
        );

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
            if check_cmd
                .stdin(std::process::Stdio::null())
                .output()
                .is_ok()
            {
                let mut pk_cmd = tokio::process::Command::new(pkg);
                pk_cmd.args(args);
                let output = pk_cmd.output().await.map_err(|e| e.to_string())?;
                if output.status.success() {
                    emit_progress(
                        "done",
                        1.0,
                        &match lang {
                            Lang::ZhCn => "Git 安装完成".to_string(),
                            Lang::EnUs => "Git installation complete".to_string(),
                        },
                    );
                    return Ok(());
                } else {
                    return Err(format!(
                        "{} install failed: {}",
                        pkg,
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        }
        return Err("No supported package manager found on this Linux system".to_string());
    }

    Err(format!("Unsupported OS: {}", os))
}
