use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::config::{get_current_lang, read_app_config_from_disk};
use crate::types::{Lang, NodeInfo, NpmInfo};
use crate::utils::get_config_path;

// ─────────────────────────────────────────────
// 内部辅助：获取 npm install 命令
// ─────────────────────────────────────────────

pub fn get_npm_install_command(data_dir: &Path, registry: &str) -> Option<(PathBuf, Vec<String>)> {
    let node_dir = data_dir.join("node");

    let local_node_path = if cfg!(target_os = "windows") {
        node_dir.join("node.exe")
    } else {
        node_dir.join("bin/node")
    };

    if local_node_path.exists() {
        // 1. 优先使用 npm-cli.js + 本地 node
        let npm_cli_paths = vec![
            node_dir
                .join("node_modules")
                .join("npm")
                .join("bin")
                .join("npm-cli.js"),
            node_dir
                .join("lib")
                .join("node_modules")
                .join("npm")
                .join("bin")
                .join("npm-cli.js"),
        ];

        for cli in npm_cli_paths {
            if cli.exists() {
                return Some((
                    local_node_path.clone(),
                    vec![
                        cli.to_string_lossy().to_string(),
                        "install".to_string(),
                        "--production".to_string(),
                        "--no-audit".to_string(),
                        "--no-fund".to_string(),
                        format!("--registry={}", registry),
                    ],
                ));
            }
        }

        // 2. 尝试 npm.cmd / bin/npm
        let npm_exec = if cfg!(target_os = "windows") {
            node_dir.join("npm.cmd")
        } else {
            node_dir.join("bin/npm")
        };

        if npm_exec.exists() {
            return Some((
                npm_exec,
                vec![
                    "install".to_string(),
                    "--production".to_string(),
                    "--no-audit".to_string(),
                    "--no-fund".to_string(),
                    format!("--registry={}", registry),
                ],
            ));
        }
    }

    // 3. 回退到系统 npm
    let system_npm = if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    };

    let mut command = std::process::Command::new(system_npm);
    command.arg("-v");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }

    if command.stdin(std::process::Stdio::null()).output().is_ok() {
        return Some((
            PathBuf::from(system_npm),
            vec![
                "install".to_string(),
                "--production".to_string(),
                "--no-audit".to_string(),
                "--no-fund".to_string(),
                format!("--registry={}", registry),
            ],
        ));
    }

    None
}

// ─────────────────────────────────────────────
// npm install 执行
// ─────────────────────────────────────────────

pub async fn run_npm_install(app: &AppHandle, target_dir: &Path) -> Result<(), String> {
    use std::process::Stdio;
    use tauri::Emitter;
    use tokio::io::AsyncBufReadExt;
    use tokio::process::Command;

    let data_dir = get_config_path(app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let config = read_app_config_from_disk(app);
    let lang = get_current_lang(app);
    let registry = config.npm_registry;

    match lang {
        Lang::ZhCn => tracing::info!(
            "准备执行 npm install, 目标目录: {:?}, 注册表: {}",
            target_dir,
            registry
        ),
        Lang::EnUs => tracing::info!(
            "Preparing npm install, target: {:?}, registry: {}",
            target_dir,
            registry
        ),
    }

    let package_json = target_dir.join("package.json");
    if !package_json.exists() {
        match lang {
            Lang::ZhCn => tracing::error!("package.json 不存在: {:?}", package_json),
            Lang::EnUs => tracing::error!("package.json does not exist: {:?}", package_json),
        }
        return Err("package.json 文件不存在".to_string());
    }

    let npm_cmd = get_npm_install_command(&data_dir, &registry);

    let emit_progress = |status: &str, progress: f64, log: &str| {
        let _ = app.emit(
            "install-progress",
            crate::types::DownloadProgress {
                status: status.to_string(),
                progress,
                log: log.to_string(),
            },
        );
    };

    if let Some((cmd, mut args)) = npm_cmd {
        args.push("--verbose".to_string());

        match lang {
            Lang::ZhCn => tracing::info!("执行命令: {:?} {:?}", cmd, args),
            Lang::EnUs => tracing::info!("Executing command: {:?} {:?}", cmd, args),
        }
        emit_progress("installing", 0.1, "正在安装依赖，请稍候...");

        // 将本地 node 目录加入 PATH
        let node_bin_dir = data_dir.join("node");
        let path_env = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = std::env::split_paths(&path_env).collect::<Vec<_>>();
        paths.insert(0, node_bin_dir.join("bin"));
        paths.insert(0, node_bin_dir);
        let new_path_env = std::env::join_paths(paths).unwrap_or(path_env);

        let mut command = Command::new(&cmd);
        command
            .args(&args)
            .current_dir(target_dir)
            .env("PATH", new_path_env)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            command.creation_flags(0x08000000);
        }

        let mut child = command.spawn().map_err(|e| {
            tracing::error!("启动 npm 失败: {}", e);
            format!("启动 npm 失败: {}", e)
        })?;

        let mut stdout_reader = child.stdout.take().map(tokio::io::BufReader::new);
        let mut stderr_reader = child.stderr.take().map(tokio::io::BufReader::new);

        let mut stdout_line = String::new();
        let mut stderr_line = String::new();
        let mut last_emit = std::time::Instant::now();
        let mut error_logs = Vec::new();

        loop {
            tokio::select! {
                result = async {
                    stdout_reader.as_mut().unwrap().read_line(&mut stdout_line).await
                }, if stdout_reader.is_some() => {
                    match result {
                        Ok(0) | Err(_) => stdout_reader = None,
                        Ok(_) => {
                            let text = stdout_line.trim_end();
                            if !text.is_empty() {
                                tracing::debug!("NPM_STDOUT: {}", text);
                                if last_emit.elapsed() > std::time::Duration::from_millis(200) {
                                    emit_progress("installing", 0.5, text);
                                    last_emit = std::time::Instant::now();
                                }
                            }
                            stdout_line.clear();
                        }
                    }
                }
                result = async {
                    stderr_reader.as_mut().unwrap().read_line(&mut stderr_line).await
                }, if stderr_reader.is_some() => {
                    match result {
                        Ok(0) | Err(_) => stderr_reader = None,
                        Ok(_) => {
                            let text = stderr_line.trim_end();
                            if !text.is_empty() {
                                tracing::debug!("NPM_STDERR: {}", text);
                                if text.contains("ERR!") || text.contains("error") || text.contains("failed") {
                                    error_logs.push(text.to_string());
                                }
                                if last_emit.elapsed() > std::time::Duration::from_millis(200) {
                                    emit_progress("installing", 0.5, text);
                                    last_emit = std::time::Instant::now();
                                }
                            }
                            stderr_line.clear();
                        }
                    }
                }
                else => break,
            }
        }

        let status = child.wait().await.map_err(|e| {
            match lang {
                Lang::ZhCn => tracing::error!("等待 npm 执行完成时发生错误: {}", e),
                Lang::EnUs => tracing::error!("Error waiting for npm: {}", e),
            }
            e.to_string()
        })?;

        if !status.success() {
            match lang {
                Lang::ZhCn => tracing::error!("npm install 执行失败，退出码: {:?}", status.code()),
                Lang::EnUs => tracing::error!("npm install failed, exit code: {:?}", status.code()),
            }

            if !error_logs.is_empty() {
                match lang {
                    Lang::ZhCn => tracing::error!("npm 错误日志:"),
                    Lang::EnUs => tracing::error!("npm error logs:"),
                }
                for log in &error_logs {
                    tracing::error!("  {}", log);
                }
            }

            // 清理失败的 node_modules
            let node_modules_path = target_dir.join("node_modules");
            if node_modules_path.exists() {
                match lang {
                    Lang::ZhCn => {
                        tracing::info!("清理失败的 node_modules: {:?}", node_modules_path)
                    }
                    Lang::EnUs => {
                        tracing::info!("Cleaning failed node_modules: {:?}", node_modules_path)
                    }
                }
                if let Err(e) = tokio::fs::remove_dir_all(&node_modules_path).await {
                    match lang {
                        Lang::ZhCn => tracing::warn!("清理 node_modules 失败: {}", e),
                        Lang::EnUs => tracing::warn!("Failed to clean node_modules: {}", e),
                    }
                }
            }

            let error_msg = if !error_logs.is_empty() {
                match lang {
                    Lang::ZhCn => format!("npm install 失败: {}", error_logs.join("\n")),
                    Lang::EnUs => format!("npm install failed: {}", error_logs.join("\n")),
                }
            } else {
                match lang {
                    Lang::ZhCn => format!("npm install 失败，退出码: {:?}", status.code()),
                    Lang::EnUs => format!("npm install failed with exit code: {:?}", status.code()),
                }
            };

            return Err(error_msg);
        }

        match lang {
            Lang::ZhCn => tracing::info!("npm install 执行成功"),
            Lang::EnUs => tracing::info!("npm install succeeded"),
        }
    } else {
        match lang {
            Lang::ZhCn => {
                tracing::warn!("未找到 npm，跳过依赖安装");
                return Err(
                    "未找到 npm，跳过依赖安装。请确保已安装 Node.js 或在设置中配置了正确的环境。"
                        .to_string(),
                );
            }
            Lang::EnUs => {
                tracing::warn!("npm not found, skipping dependency installation");
                return Err("npm not found. Please ensure Node.js is installed or environment is correctly configured in settings.".to_string());
            }
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────
// Tauri commands
// ─────────────────────────────────────────────

#[tauri::command]
pub async fn check_nodejs(app: AppHandle) -> Result<NodeInfo, String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!("检查 Node.js 环境"),
        Lang::EnUs => tracing::info!("Checking Node.js environment"),
    }
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let node_dir = data_dir.join("node");

    let local_node_path = if cfg!(target_os = "windows") {
        node_dir.join("node.exe")
    } else {
        node_dir.join("bin/node")
    };

    if local_node_path.exists() {
        let mut command = std::process::Command::new(&local_node_path);
        command.arg("-v");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }

        if let Ok(output) = command.stdin(std::process::Stdio::null()).output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                match lang {
                    Lang::ZhCn => tracing::info!("找到本地 Node.js: {}", version),
                    Lang::EnUs => tracing::info!("Found local Node.js: {}", version),
                }
                return Ok(NodeInfo {
                    version: Some(version),
                    path: Some(local_node_path.to_string_lossy().to_string()),
                    source: "local".to_string(),
                });
            }
        }
    }

    let cmd = "node";
    let mut command = std::process::Command::new(cmd);
    command.arg("-v");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }

    if let Ok(output) = command.stdin(std::process::Stdio::null()).output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

            let path_cmd = if cfg!(target_os = "windows") {
                "where"
            } else {
                "which"
            };
            let mut node_path = "system".to_string();

            let mut path_command = std::process::Command::new(path_cmd);
            path_command.arg("node");
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
                            node_path = trimmed.to_string();
                        }
                    }
                }
            }

            match lang {
                Lang::ZhCn => tracing::info!("找到系统 Node.js: {}", version),
                Lang::EnUs => tracing::info!("Found system Node.js: {}", version),
            }
            return Ok(NodeInfo {
                version: Some(version),
                path: Some(node_path),
                source: "system".to_string(),
            });
        }
    }

    match lang {
        Lang::ZhCn => tracing::warn!("未找到 Node.js 环境"),
        Lang::EnUs => tracing::warn!("Node.js environment not found"),
    }
    Ok(NodeInfo {
        version: None,
        path: None,
        source: "none".to_string(),
    })
}

#[tauri::command]
pub async fn check_npm(app: AppHandle) -> Result<NpmInfo, String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!("检查 NPM 环境"),
        Lang::EnUs => tracing::info!("Checking NPM environment"),
    }
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let node_dir = data_dir.join("node");

    let local_node_path = if cfg!(target_os = "windows") {
        node_dir.join("node.exe")
    } else {
        node_dir.join("bin/node")
    };

    if local_node_path.exists() {
        let npm_cmd = if cfg!(target_os = "windows") {
            node_dir.join("npm.cmd")
        } else {
            node_dir.join("bin/npm")
        };

        if npm_cmd.exists() {
            let mut command = std::process::Command::new(&npm_cmd);
            command.arg("-v");
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x08000000);
            }

            if let Ok(output) = command.stdin(std::process::Stdio::null()).output() {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    match lang {
                        Lang::ZhCn => tracing::info!("找到本地 NPM (cmd/bin): {}", version),
                        Lang::EnUs => tracing::info!("Found local NPM (cmd/bin): {}", version),
                    }
                    return Ok(NpmInfo {
                        version: Some(version),
                        path: Some(npm_cmd.to_string_lossy().to_string()),
                        source: "local".to_string(),
                    });
                }
            }
        }

        let npm_cli = if cfg!(target_os = "windows") {
            node_dir
                .join("node_modules")
                .join("npm")
                .join("bin")
                .join("npm-cli.js")
        } else {
            node_dir.join("lib/node_modules/npm/bin/npm-cli.js")
        };

        let npm_cli_flat = node_dir.join("node_modules/npm/bin/npm-cli.js");

        let target_cli = if npm_cli.exists() {
            Some(npm_cli)
        } else if npm_cli_flat.exists() {
            Some(npm_cli_flat)
        } else {
            None
        };

        if let Some(cli) = target_cli {
            let mut command = std::process::Command::new(&local_node_path);
            command.arg(&cli).arg("-v");

            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x08000000);
            }

            if let Ok(output) = command.stdin(std::process::Stdio::null()).output() {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    tracing::info!("找到本地 NPM (cli.js): {}", version);
                    return Ok(NpmInfo {
                        version: Some(version),
                        path: Some(cli.to_string_lossy().to_string()),
                        source: "local".to_string(),
                    });
                }
            }
        }
    }

    // 系统 npm
    let cmd = if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    };

    let mut command = std::process::Command::new(cmd);
    command.arg("-v");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }

    if let Ok(output) = command.stdin(std::process::Stdio::null()).output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

            let path_cmd = if cfg!(target_os = "windows") {
                "where"
            } else {
                "which"
            };
            let mut npm_path = "system".to_string();

            let mut path_command = std::process::Command::new(path_cmd);
            path_command.arg("npm");
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
                            npm_path = trimmed.to_string();
                        }
                    }
                }
            }

            match lang {
                Lang::ZhCn => tracing::info!("找到系统 NPM: {}", version),
                Lang::EnUs => tracing::info!("Found system NPM: {}", version),
            }
            return Ok(NpmInfo {
                version: Some(version),
                path: Some(npm_path),
                source: "system".to_string(),
            });
        }
    }

    match lang {
        Lang::ZhCn => tracing::warn!("未找到 NPM 环境"),
        Lang::EnUs => tracing::warn!("NPM environment not found"),
    }
    Ok(NpmInfo {
        version: None,
        path: None,
        source: "none".to_string(),
    })
}

#[tauri::command]
pub async fn install_nodejs(app: AppHandle) -> Result<(), String> {
    use futures_util::StreamExt;
    use tauri::Emitter;

    let lang = get_current_lang(&app);
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let node_os = match os {
        "windows" => "win",
        "linux" => "linux",
        "macos" => "darwin",
        _ => return Err(format!("Unsupported OS: {}", os)),
    };

    let node_arch = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => return Err(format!("Unsupported Arch: {}", arch)),
    };

    let filename = format!("node-v22.12.0-{}-{}.zip", node_os, node_arch);
    let url = format!("https://npmmirror.com/mirrors/node/v22.12.0/{}", filename);

    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let node_dir = data_dir.join("node");

    match lang {
        Lang::ZhCn => tracing::info!(
            "开始安装 Node.js, OS: {}, Arch: {}, URL: {}",
            node_os,
            node_arch,
            url
        ),
        Lang::EnUs => tracing::info!(
            "Starting Node.js install, OS: {}, Arch: {}, URL: {}",
            node_os,
            node_arch,
            url
        ),
    }

    let emit_progress = |status: &str, progress: f64, log: &str| {
        let _ = app.emit(
            "download-progress",
            crate::types::DownloadProgress {
                status: status.to_string(),
                progress,
                log: log.to_string(),
            },
        );
    };

    emit_progress(
        "downloading",
        0.0,
        &match lang {
            Lang::ZhCn => format!("开始下载 Node.js: {}", filename),
            Lang::EnUs => format!("Starting Node.js download: {}", filename),
        },
    );

    let temp_dir = std::env::temp_dir();
    let temp_zip_path = temp_dir.join(&filename);

    let client = reqwest::Client::builder()
        .user_agent("sillyTavern-launcher")
        .build()
        .map_err(|e| {
            match lang {
                Lang::ZhCn => tracing::error!("创建 HTTP 客户端失败: {}", e),
                Lang::EnUs => tracing::error!("Failed to create HTTP client: {}", e),
            }
            e.to_string()
        })?;

    let response = client.get(&url).send().await.map_err(|e| {
        match lang {
            Lang::ZhCn => tracing::error!("请求 Node.js 下载失败: {}", e),
            Lang::EnUs => tracing::error!("Node.js download request failed: {}", e),
        }
        e.to_string()
    })?;
    let total_size = response.content_length().unwrap_or(0);
    match lang {
        Lang::ZhCn => tracing::info!("Node.js 下载开始，总大小: {} 字节", total_size),
        Lang::EnUs => tracing::info!("Node.js download started, total size: {} bytes", total_size),
    }

    let mut file = tokio::fs::File::create(&temp_zip_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        use tokio::io::AsyncWriteExt;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let progress = if total_size > 0 {
            (downloaded as f64) / (total_size as f64)
        } else {
            0.0
        };

        let mb_downloaded = downloaded as f64 / 1_048_576.0;
        emit_progress(
            "downloading",
            progress,
            &match lang {
                Lang::ZhCn => format!("已下载: {:.2} MB", mb_downloaded),
                Lang::EnUs => format!("Downloaded: {:.2} MB", mb_downloaded),
            },
        );
    }

    emit_progress(
        "extracting",
        0.0,
        &match lang {
            Lang::ZhCn => "下载完成，正在解压...".to_string(),
            Lang::EnUs => "Download complete, extracting...".to_string(),
        },
    );

    let app_clone = app.clone();
    let temp_zip_path_clone = temp_zip_path.clone();
    let node_dir_clone = node_dir.clone();
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

        if node_dir_clone.exists() {
            std::fs::remove_dir_all(&node_dir_clone).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir_all(&node_dir_clone).map_err(|e| e.to_string())?;

        let file = std::fs::File::open(&temp_zip_path_clone).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let total_files = archive.len();

        for i in 0..total_files {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let mut components = outpath.components();
            components.next();
            let stripped_path: PathBuf = components.collect();

            if stripped_path.as_os_str().is_empty() {
                continue;
            }

            let target_path = node_dir_clone.join(&stripped_path);

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

    let _ = tokio::fs::remove_file(temp_zip_path).await;
    emit_progress(
        "done",
        1.0,
        &match lang {
            Lang::ZhCn => "Node.js 安装完成".to_string(),
            Lang::EnUs => "Node.js installation complete".to_string(),
        },
    );

    Ok(())
}
