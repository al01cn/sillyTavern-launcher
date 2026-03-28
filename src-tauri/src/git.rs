use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

pub fn get_git_exe(app: &AppHandle) -> PathBuf {
    let data_dir = get_config_path(app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let git_dir = data_dir.join("git");

    if cfg!(target_os = "windows") {
        let local = git_dir.join("cmd/git.exe");
        if local.exists() {
            local
        } else {
            PathBuf::from("git")
        }
    } else {
        let local = git_dir.join("bin/git");
        if local.exists() {
            local
        } else {
            PathBuf::from("git")
        }
    }
}
use crate::config::get_current_lang;
use crate::types::{Lang, GitInfo, DownloadProgress};
use crate::utils::get_config_path;

#[tauri::command]
pub async fn check_git(app: AppHandle) -> Result<GitInfo, String> {
    let _lang = get_current_lang(&app);
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let git_dir = data_dir.join("git");

    let local_git_path = if cfg!(target_os = "windows") {
        git_dir.join("cmd/git.exe")
    } else {
        git_dir.join("bin/git")
    };

    if local_git_path.exists() {
        let mut command = std::process::Command::new(&local_git_path);
        command.arg("--version");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }

        if let Ok(output) = command.stdin(std::process::Stdio::null()).output() {
            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let version = version_output.replace("git version ", "").trim().to_string();
                return Ok(GitInfo {
                    version: Some(version),
                    path: Some(local_git_path.to_string_lossy().replace('\\', "/")),
                    source: "local".to_string(),
                });
            }
        }
    }

    let cmd = "git";
    let mut command = std::process::Command::new(cmd);
    command.arg("--version");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }

    if let Ok(output) = command.stdin(std::process::Stdio::null()).output() {
        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let version = version_output.replace("git version ", "").trim().to_string();

            let path_cmd = if cfg!(target_os = "windows") {
                "where"
            } else {
                "which"
            };
            let mut git_path = "system".to_string();

            let mut path_command = std::process::Command::new(path_cmd);
            path_command.arg("git");
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                path_command.creation_flags(0x08000000);
            }

            if let Ok(path_output) = path_command.stdin(std::process::Stdio::null()).output() {
                if path_output.status.success() {
                    let path_str = String::from_utf8_lossy(&path_output.stdout);
                    if let Some(first_line) = path_str.lines().next() {
                        let trimmed = first_line.trim();
                        if !trimmed.is_empty() {
                            git_path = trimmed.replace('\\', "/");
                        }
                    }
                }
            }

            return Ok(GitInfo {
                version: Some(version),
                path: Some(git_path),
                source: "system".to_string(),
            });
        }
    }

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