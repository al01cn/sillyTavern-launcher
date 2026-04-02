use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use crate::config::{get_current_lang, read_app_config_from_disk, write_app_config_to_disk};
use crate::git::get_git_exe;
use crate::node::run_npm_install;
use crate::types::InstallState;
use crate::types::{
    DownloadProgress, InstalledVersionInfo, Lang, ProcessState, Release, TavernBackupsChatConfig,
    TavernBackupsCommonConfig, TavernBackupsConfig, TavernBasicAuthUser, TavernCacheBusterConfig,
    TavernConfigPayload, TavernCorsConfig, TavernDualStackAddress, TavernDualStackProtocol,
    TavernExtensionsConfig, TavernHostWhitelistConfig, TavernLoggingConfig,
    TavernPerformanceConfig, TavernRequestProxyConfig, TavernSslConfig, TavernSsoConfig,
    TavernThumbnailsConfig, TavernThumbnailsDimensionsConfig,
};
use crate::utils::get_config_path;

// ─── Git config 全局 URL 重写 ────────────────────────────────────────────────

/// 获取当前将要使用的 Node.js 版本号（major, minor, patch）
/// 返回 None 表示找不到 node 或解析失败
fn get_actual_node_version(node_path: &std::path::Path) -> Option<(u32, u32, u32)> {
    let mut cmd = std::process::Command::new(node_path);
    cmd.arg("-v").stdin(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let ver = String::from_utf8_lossy(&output.stdout);
    let ver = ver.trim().trim_start_matches('v');
    let parts: Vec<u32> = ver
        .split('.')
        .take(3)
        .filter_map(|s| s.split('-').next()?.parse().ok())
        .collect();
    if parts.len() >= 3 {
        Some((parts[0], parts[1], parts[2]))
    } else if parts.len() == 2 {
        Some((parts[0], parts[1], 0))
    } else if parts.len() == 1 {
        Some((parts[0], 0, 0))
    } else {
        None
    }
}

/// --import 拦截器要求 Node.js >= 18.19.0
fn node_supports_import(node_path: &std::path::Path) -> bool {
    match get_actual_node_version(node_path) {
        Some((major, minor, _patch)) => {
            // >= 18.19.0：minor >= 19 即满足（18.19.x 及以上所有小版本均可）
            major > 18 || (major == 18 && minor >= 19)
        }
        None => false,
    }
}

/// 设置全局 git config URL 重写（加速用）
/// 执行: git config --global url."<proxy>/https://github.com/".insteadOf "https://github.com/"
/// git_exe: 使用内置 MinGit 或系统 git 的完整路径，避免依赖系统 PATH
pub fn set_git_global_proxy(git_exe: &std::path::Path, proxy_url: &str) {
    let key = format!(
        "url.{}/https://github.com/.insteadOf",
        proxy_url.trim_end_matches('/')
    );
    tracing::info!(
        "正在设置全局 git proxy, git={}, key={}",
        git_exe.display(),
        key
    );
    let mut cmd = std::process::Command::new(git_exe);
    cmd.args(["config", "--global", &key, "https://github.com/"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    match cmd.output() {
        Ok(out) if out.status.success() => tracing::info!("已设置全局 git proxy: {}", proxy_url),
        Ok(out) => tracing::warn!(
            "设置 git proxy 失败(exit={}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => tracing::warn!("运行 git config 失败: {}", e),
    }
}

/// 还原全局 git config URL 重写（移除代理设置）
/// 执行: git config --global --unset url."<proxy>/https://github.com/".insteadOf
/// git_exe: 使用内置 MinGit 或系统 git 的完整路径，避免依赖系统 PATH
pub fn unset_git_global_proxy(git_exe: &std::path::Path, proxy_url: &str) {
    let key = format!(
        "url.{}/https://github.com/.insteadOf",
        proxy_url.trim_end_matches('/')
    );
    tracing::info!(
        "正在还原全局 git proxy, git={}, key={}",
        git_exe.display(),
        key
    );
    let mut cmd = std::process::Command::new(git_exe);
    cmd.args(["config", "--global", "--unset", &key])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    match cmd.output() {
        Ok(out) if out.status.success() => tracing::info!("已还原全局 git proxy 设置"),
        // exit code 5 = key not found, 正常情况
        Ok(out) if out.status.code() == Some(5) => tracing::info!("git proxy 设置不存在，无需还原"),
        Ok(out) => tracing::warn!(
            "还原 git proxy 失败(exit={}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => tracing::warn!("运行 git config --unset 失败: {}", e),
    }
}

// ─── 默认配置模板 ────────────────────────────────────────────────────────────

/// SillyTavern 默认配置模板（YAML 格式）
/// 用于在配置文件不存在时自动创建
pub const DEFAULT_CONFIG_TEMPLATE: &str = r#"dataRoot: ./
listen: true
listenAddress:
  ipv4: 0.0.0.0
  ipv6: '[::]'
protocol:
  ipv4: true
  ipv6: true
dnsPreferIPv6: false
browserLaunch:
  enabled: true
  browser: default
  hostname: auto
  port: -1
  avoidLocalhost: false
port: 11451
heartbeatInterval: 0
ssl:
  enabled: false
  certPath: ./certs/cert.pem
  keyPath: ./certs/privkey.pem
  keyPassphrase: ''
whitelistMode: true
enableForwardedWhitelist: true
whitelist:
- ::1
- 127.0.0.1
whitelistDockerHosts: true
basicAuthMode: false
basicAuthUser:
  username: user
  password: password
enableCorsProxy: false
cors:
  enabled: false
  origin:
  - 'null'
  methods:
  - OPTIONS
  allowedHeaders: []
  exposedHeaders: []
  credentials: false
  maxAge: null
requestProxy:
  enabled: false
  url: socks5://username:password@example.com:1080
  bypass:
  - localhost
  - 127.0.0.1
  - ::1
enableUserAccounts: false
enableDiscreetLogin: false
perUserBasicAuth: false
sso:
  autheliaAuth: false
  authentikAuth: false
hostWhitelist:
  enabled: false
  scan: true
  hosts: []
sessionTimeout: -1
disableCsrfProtection: false
securityOverride: false
logging:
  enableAccessLog: true
  minLogLevel: 0
rateLimiting:
  preferRealIpHeader: false
backups:
  common:
    numberOfBackups: 50
  chat:
    enabled: true
    checkIntegrity: true
    maxTotalBackups: -1
    throttleInterval: 10000
thumbnails:
  enabled: true
  format: jpg
  quality: 95
  dimensions:
    bg:
    - 160
    - 90
    avatar:
    - 96
    - 144
    persona:
    - 96
    - 144
performance:
  lazyLoadCharacters: false
  memoryCacheCapacity: 100mb
  useDiskCache: true
cacheBuster:
  enabled: false
  userAgentPattern: ''
allowKeysExposure: false
skipContentCheck: false
whitelistImportDomains:
- localhost
- cdn.discordapp.com
- files.catbox.moe
- raw.githubusercontent.com
- ghfast.top
requestOverrides: []
extensions:
  enabled: true
  autoUpdate: true
  models:
    autoDownload: true
    classification: Cohee/distilbert-base-uncased-go-emotions-onnx
    captioning: Xenova/vit-gpt2-image-captioning
    embedding: Cohee/jina-embeddings-v2-base-en
    speechToText: Xenova/whisper-small
    textToSpeech: Xenova/speecht5_tts
enableDownloadableTokenizers: true
promptPlaceholder: '[Start a new chat]'
openai:
  randomizeUserId: false
  captionSystemPrompt: ''
deepl:
  formality: default
mistral:
  enablePrefix: false
ollama:
  keepAlive: -1
  batchSize: -1
claude:
  enableSystemPromptCache: false
  cachingAtDepth: -1
  extendedTTL: false
gemini:
  apiVersion: v1beta
  thoughtSignatures: true
  enableSystemPromptCache: false
  image:
    personGeneration: allow_adult
enableServerPlugins: false
enableServerPluginsAutoUpdate: true
"#;

// ─── GitHub Releases ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn fetch_sillytavern_releases() -> Result<Vec<Release>, String> {
    let client = reqwest::Client::builder()
        .user_agent("sillyTavern-launcher")
        .build()
        .map_err(|e| e.to_string())?;
    let url = "https://api.github.com/repos/SillyTavern/SillyTavern/releases";
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("GitHub API Error: {}", response.status()));
    }
    let releases: Vec<Release> = response.json().await.map_err(|e| e.to_string())?;
    Ok(releases)
}

// ─── 版本列表 ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_installed_sillytavern_versions(app: AppHandle) -> Result<Vec<String>, String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!("获取已安装的酒馆版本列表"),
        Lang::EnUs => tracing::info!("Getting installed SillyTavern version list"),
    }
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let start = std::time::Instant::now();
        let data_dir = get_config_path(&app_clone)
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        let st_dir = data_dir.join("sillytavern");
        if !st_dir.exists() {
            match lang {
                Lang::ZhCn => {
                    tracing::info!("酒馆目录不存在，返回空列表, 耗时: {:?}", start.elapsed())
                }
                Lang::EnUs => tracing::info!(
                    "SillyTavern directory not found, elapsed: {:?}",
                    start.elapsed()
                ),
            }
            return Ok(vec![]);
        }
        let mut versions = Vec::new();
        if let Ok(entries) = fs::read_dir(&st_dir) {
            for entry in entries.flatten() {
                if let Ok(_ft) = entry.file_type() {
                    if entry.path().is_dir() {
                        if let Ok(name) = entry.file_name().into_string() {
                            if !name.starts_with('.') {
                                versions.push(name);
                            }
                        }
                    }
                }
            }
        }
        match lang {
            Lang::ZhCn => tracing::info!(
                "找到已安装的版本: {:?}, 耗时: {:?}",
                versions,
                start.elapsed()
            ),
            Lang::EnUs => tracing::info!(
                "Found versions: {:?}, elapsed: {:?}",
                versions,
                start.elapsed()
            ),
        }
        Ok(versions)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_installed_versions_info(
    app: AppHandle,
) -> Result<Vec<InstalledVersionInfo>, String> {
    let lang = get_current_lang(&app);
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let start = std::time::Instant::now();
        let data_dir = get_config_path(&app_clone)
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        let st_dir = data_dir.join("sillytavern");
        if !st_dir.exists() {
            return Ok(vec![]);
        }
        let mut versions = Vec::new();
        if let Ok(entries) = fs::read_dir(&st_dir) {
            for entry in entries.flatten() {
                if let Ok(_ft) = entry.file_type() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(name) = entry.file_name().into_string() {
                            if !name.starts_with('.') {
                                let nm = path.join("node_modules");
                                let has_node_modules = nm.exists()
                                    && fs::read_dir(&nm)
                                        .map(|mut d| d.next().is_some())
                                        .unwrap_or(false);

                                let mut is_link = false;
                                if let Ok(m) = fs::symlink_metadata(&path) {
                                    if m.file_type().is_symlink() {
                                        is_link = true;
                                    } else {
                                        #[cfg(target_os = "windows")]
                                        {
                                            use std::os::windows::fs::MetadataExt;
                                            if m.file_attributes() & 0x400 != 0 {
                                                is_link = true;
                                            }
                                        }
                                    }
                                }

                                versions.push(InstalledVersionInfo {
                                    version: name,
                                    has_node_modules,
                                    is_link,
                                });
                            }
                        }
                    }
                }
            }
        }
        match lang {
            Lang::ZhCn => tracing::info!("获取到版本详细信息, 耗时: {:?}", start.elapsed()),
            Lang::EnUs => tracing::info!("Got version info, elapsed: {:?}", start.elapsed()),
        }
        Ok(versions)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─── 版本切换 ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn switch_sillytavern_version(
    app: AppHandle,
    version: crate::types::LocalTavernItem,
) -> Result<(), String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!("切换酒馆版本到: {}", version.version),
        Lang::EnUs => tracing::info!("Switching version to: {}", version.version),
    }
    let mut config = read_app_config_from_disk(&app);

    let version_to_save = version.clone();

    let version_dir = if version_to_save.path.is_empty() {
        let data_dir = get_config_path(&app)
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        data_dir.join("sillytavern").join(&version_to_save.version)
    } else {
        PathBuf::from(&version_to_save.path)
    };

    if !version_dir.exists() {
        match lang {
            Lang::ZhCn => {
                tracing::error!("版本 {} 的路径不存在", version_to_save.version);
                return Err(format!("版本 {} 的路径不存在", version_to_save.version));
            }
            Lang::EnUs => {
                tracing::error!("Version path for {} not found", version_to_save.version);
                return Err(format!(
                    "Version path for {} not found",
                    version_to_save.version
                ));
            }
        }
    }
    config.sillytavern.version = version_to_save;
    write_app_config_to_disk(&app, &config)
}

// ─── 取消安装 ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn cancel_install(state: tauri::State<'_, InstallState>) {
    state
        .cancel_flag
        .store(true, std::sync::atomic::Ordering::Relaxed);
}

// ─── 安装版本 ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn install_sillytavern_version(
    app: AppHandle,
    state: tauri::State<'_, InstallState>,
    version: String,
    url: String,
) -> Result<(), String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!("开始安装酒馆版本: {}，URL: {}", version, url),
        Lang::EnUs => tracing::info!("Installing version: {}, URL: {}", version, url),
    }
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let st_dir = data_dir.join("sillytavern").join(&version);

    if st_dir.exists() {
        match lang {
            Lang::ZhCn => tracing::info!("版本 {} 已存在，跳过安装", version),
            Lang::EnUs => tracing::info!("Version {} already exists, skipping", version),
        }
        return Ok(());
    }

    fs::create_dir_all(&st_dir).map_err(|e| {
        match lang {
            Lang::ZhCn => tracing::error!("创建目录失败: {}", e),
            Lang::EnUs => tracing::error!("Failed to create dir: {}", e),
        }
        e.to_string()
    })?;
    state
        .cancel_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);

    let emit = |status: &str, progress: f64, log: &str| {
        let _ = app.emit(
            "install-progress",
            DownloadProgress {
                status: status.to_string(),
                progress,
                log: log.to_string(),
            },
        );
    };

    emit(
        "downloading",
        0.0,
        &match lang {
            Lang::ZhCn => format!("准备下载版本 {}...", version),
            Lang::EnUs => format!("Preparing to download version {}...", version),
        },
    );

    let temp_zip = std::env::temp_dir().join(format!("sillytavern_{}.zip", version));
    let client = reqwest::Client::builder()
        .user_agent("sillyTavern-launcher")
        .build()
        .map_err(|e| e.to_string())?;

    let proxy = crate::utils::GithubProxy::new(&app).await;
    let (fastest_url, response) = proxy.get_fastest_stream(client, &url).await.map_err(|e| {
        match lang {
            Lang::ZhCn => tracing::error!("请求下载失败: {}", e),
            Lang::EnUs => tracing::error!("Download failed: {}", e),
        }
        e
    })?;

    match lang {
        Lang::ZhCn => tracing::info!("使用下载节点: {}", fastest_url),
        Lang::EnUs => tracing::info!("Using download mirror: {}", fastest_url),
    }

    let total_size = response.content_length().unwrap_or(0);

    let mut file = tokio::fs::File::create(&temp_zip)
        .await
        .map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut last_emit = std::time::Instant::now();

    while let Some(item) = stream.next().await {
        if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = tokio::fs::remove_file(&temp_zip).await;
            let _ = tokio::fs::remove_dir_all(&st_dir).await;
            emit(
                "error",
                0.0,
                match lang {
                    Lang::ZhCn => "下载已取消",
                    Lang::EnUs => "Download cancelled",
                },
            );
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
                downloaded as f64 / total_size as f64
            } else {
                0.0
            };
            let mb_downloaded = downloaded as f64 / 1_048_576.0;
            let mb_total = total_size as f64 / 1_048_576.0;
            emit(
                "downloading",
                progress,
                &match lang {
                    Lang::ZhCn => {
                        if total_size > 0 {
                            format!("已下载: {:.2} MB / {:.2} MB", mb_downloaded, mb_total)
                        } else {
                            format!("已下载: {:.2} MB", mb_downloaded)
                        }
                    }
                    Lang::EnUs => {
                        if total_size > 0 {
                            format!("Downloaded: {:.2} MB / {:.2} MB", mb_downloaded, mb_total)
                        } else {
                            format!("Downloaded: {:.2} MB", mb_downloaded)
                        }
                    }
                },
            );
            last_emit = std::time::Instant::now();
        }
    }

    drop(file);

    emit(
        "extracting",
        0.0,
        match lang {
            Lang::ZhCn => "下载完成，准备解压...",
            Lang::EnUs => "Download complete, extracting...",
        },
    );

    let cancel_flag = state.cancel_flag.clone();
    let app_clone = app.clone();
    let temp_zip_clone = temp_zip.clone();
    let st_dir_clone = st_dir.clone();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let emit2 = |status: &str, progress: f64, log: &str| {
            let _ = app_clone.emit(
                "install-progress",
                DownloadProgress {
                    status: status.to_string(),
                    progress,
                    log: log.to_string(),
                },
            );
        };
        let file = fs::File::open(&temp_zip_clone).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let total = archive.len();
        for i in 0..total {
            if i % 10 == 0 && cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = fs::remove_file(&temp_zip_clone);
                let _ = fs::remove_dir_all(&st_dir_clone);
                emit2(
                    "error",
                    0.0,
                    match lang {
                        Lang::ZhCn => "解压已取消",
                        Lang::EnUs => "Extraction cancelled",
                    },
                );
                return Err(match lang {
                    Lang::ZhCn => "解压已取消".to_string(),
                    Lang::EnUs => "Extraction cancelled".to_string(),
                });
            }
            let mut f = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match f.enclosed_name() {
                Some(p) => p.to_owned(),
                None => continue,
            };
            let mut comps = outpath.components();
            comps.next();
            let stripped: PathBuf = comps.collect();
            if stripped.as_os_str().is_empty() {
                continue;
            }
            let target = st_dir_clone.join(&stripped);
            if (*f.name()).ends_with('/') {
                fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = target.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let mut out = fs::File::create(&target).map_err(|e| e.to_string())?;
                io::copy(&mut f, &mut out).map_err(|e| e.to_string())?;
            }
            if i % 500 == 0 || i == total - 1 {
                emit2(
                    "extracting",
                    i as f64 / total as f64,
                    &match lang {
                        Lang::ZhCn => format!("解压中: {}/{} 文件...", i + 1, total),
                        Lang::EnUs => format!("Extracting: {}/{} files...", i + 1, total),
                    },
                );
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    let _ = fs::remove_file(&temp_zip);

    emit(
        "installing",
        0.0,
        match lang {
            Lang::ZhCn => "正在安装依赖 (npm install)... 这可能需要几分钟",
            Lang::EnUs => "Installing dependencies (npm install)... this may take a few minutes",
        },
    );

    let app2 = app.clone();
    let st_dir2 = st_dir.clone();
    let version_clone = version.clone();
    tokio::spawn(async move {
        if let Err(e) = run_npm_install(&app2, &st_dir2).await {
            let _ = app2.emit(
                "install-progress",
                DownloadProgress {
                    status: "error".to_string(),
                    progress: 0.0,
                    log: match lang {
                        Lang::ZhCn => format!("安装依赖失败: {}", e),
                        Lang::EnUs => format!("Failed to install dependencies: {}", e),
                    },
                },
            );
        } else {
            let _ = generate_default_settings_for_version(&app2, &version_clone);
            let _ = app2.emit(
                "install-progress",
                DownloadProgress {
                    status: "done".to_string(),
                    progress: 1.0,
                    log: match lang {
                        Lang::ZhCn => "安装完成！".to_string(),
                        Lang::EnUs => "Installation complete!".to_string(),
                    },
                },
            );
        }
    });

    Ok(())
}

// ─── 单独安装依赖 ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn check_local_tavern_dependencies(
    _app: AppHandle,
    path: String,
) -> Result<bool, String> {
    let st_dir = PathBuf::from(&path);
    let nm = st_dir.join("node_modules");
    let has_nm = nm.exists()
        && std::fs::read_dir(&nm)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false);
    Ok(has_nm)
}

#[tauri::command]
pub async fn install_sillytavern_dependencies(
    app: AppHandle,
    version: String,
) -> Result<(), String> {
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();

    // Support installing dependencies for local path
    let st_dir = if version.contains('/') || version.contains('\\') || version.contains(':') {
        PathBuf::from(&version)
    } else {
        data_dir.join("sillytavern").join(&version)
    };

    if !st_dir.exists() {
        return Err(format!("Version/Path {} not found", version));
    }
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    let version_clone = version.clone();
    tokio::spawn(async move {
        if let Err(e) = run_npm_install(&app2, &st_dir).await {
            let _ = app2.emit(
                "install-progress",
                DownloadProgress {
                    status: "error".to_string(),
                    progress: 0.0,
                    log: match lang {
                        Lang::ZhCn => format!("安装依赖失败: {}", e),
                        Lang::EnUs => format!("Failed to install dependencies: {}", e),
                    },
                },
            );
        } else {
            // Get actual version from package.json if it is a local path
            let actual_version = if version_clone.contains('/')
                || version_clone.contains('\\')
                || version_clone.contains(':')
            {
                if let Ok(content) = std::fs::read_to_string(st_dir.join("package.json")) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                        v.get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string()
                    } else {
                        "unknown".to_string()
                    }
                } else {
                    "unknown".to_string()
                }
            } else {
                version_clone.clone()
            };

            let _ = generate_default_settings_for_version(&app2, &actual_version);
            let _ = app2.emit(
                "install-progress",
                DownloadProgress {
                    status: "done".to_string(),
                    progress: 1.0,
                    log: match lang {
                        Lang::ZhCn => "依赖安装完成！".to_string(),
                        Lang::EnUs => "Dependency installation complete!".to_string(),
                    },
                },
            );
        }
    });
    Ok(())
}

// ─── 删除版本 ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn delete_sillytavern_version(app: AppHandle, version: String) -> Result<(), String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!("准备删除酒馆版本: {}", version),
        Lang::EnUs => tracing::info!("Deleting version: {}", version),
    }
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let version_dir = data_dir.join("sillytavern").join(&version);

    if !version_dir.exists() {
        return match lang {
            Lang::ZhCn => Err(format!("版本 {} 不存在", version)),
            Lang::EnUs => Err(format!("Version {} not found", version)),
        };
    }
    if version.trim().is_empty()
        || version.contains("..")
        || version.contains('/')
        || version.contains('\\')
    {
        return match lang {
            Lang::ZhCn => Err("无效的版本号".to_string()),
            Lang::EnUs => Err("Invalid version number".to_string()),
        };
    }

    let app2 = app.clone();
    let vdir = version_dir.clone();
    let vc = version.clone();

    let result = tokio::task::spawn_blocking(move || {
        let meta = fs::symlink_metadata(&vdir).ok();
        let mut is_link = false;
        if let Some(m) = meta {
            if m.file_type().is_symlink() {
                is_link = true;
            } else {
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::fs::MetadataExt;
                    let attrs = m.file_attributes();
                    if attrs & 0x400 != 0 {
                        is_link = true;
                    }
                }
            }
        }

        let _ = app2.emit(
            "install-progress",
            DownloadProgress {
                status: "deleting".to_string(),
                progress: 0.1,
                log: match lang {
                    Lang::ZhCn => {
                        if is_link {
                            format!("开始解绑版本 {}...", vc)
                        } else {
                            format!("开始删除版本 {}...", vc)
                        }
                    }
                    Lang::EnUs => {
                        if is_link {
                            format!("Unbinding version {}...", vc)
                        } else {
                            format!("Deleting version {}...", vc)
                        }
                    }
                },
            },
        );
        std::thread::sleep(std::time::Duration::from_millis(100));

        if is_link {
            #[cfg(target_os = "windows")]
            let _ = fs::remove_dir(&vdir).or_else(|_| fs::remove_file(&vdir));
            #[cfg(not(target_os = "windows"))]
            let _ = fs::remove_file(&vdir);
        } else {
            let mut samples = Vec::new();
            if let Ok(entries) = fs::read_dir(&vdir) {
                for e in entries.flatten() {
                    if let Ok(n) = e.file_name().into_string() {
                        samples.push(n);
                    }
                }
            }
            let total = samples.len();
            for (i, name) in samples.iter().enumerate() {
                std::thread::sleep(std::time::Duration::from_millis(15));
                let _ = app2.emit(
                    "install-progress",
                    DownloadProgress {
                        status: "deleting".to_string(),
                        progress: 0.3 + 0.5 * (i as f64 / total as f64),
                        log: match lang {
                            Lang::ZhCn => format!("已删除：{}/{}", vc, name),
                            Lang::EnUs => format!("Deleted: {}/{}", vc, name),
                        },
                    },
                );
            }

            fn fast_remove(dir: &Path) -> io::Result<()> {
                if dir.is_dir() {
                    if let Ok(entries) = fs::read_dir(dir) {
                        for e in entries.flatten() {
                            let p = e.path();
                            if let Ok(_ft) = e.file_type() {
                                if p.is_dir() {
                                    let _ = fast_remove(&p);
                                } else if fs::remove_file(&p).is_err() {
                                    #[cfg(target_os = "windows")]
                                    {
                                        if let Ok(mut perms) =
                                            fs::metadata(&p).map(|m| m.permissions())
                                        {
                                            if perms.readonly() {
                                                perms.set_readonly(false);
                                                let _ = fs::set_permissions(&p, perms);
                                                let _ = fs::remove_file(&p);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let _ = fs::remove_dir(dir);
                }
                Ok(())
            }
            let _ = fast_remove(&vdir);
            if vdir.exists() {
                fs::remove_dir_all(&vdir)?;
            }
        }
        let _ = app2.emit(
            "install-progress",
            DownloadProgress {
                status: "deleting".to_string(),
                progress: 1.0,
                log: match lang {
                    Lang::ZhCn => {
                        if is_link {
                            format!("版本 {} 解绑成功", vc)
                        } else {
                            format!("版本 {} 已全部删除", vc)
                        }
                    }
                    Lang::EnUs => {
                        if is_link {
                            format!("Version {} unbound", vc)
                        } else {
                            format!("Version {} deleted", vc)
                        }
                    }
                },
            },
        );
        Ok::<(), io::Error>(())
    })
    .await;

    match result {
        Ok(Ok(_)) => {
            match lang {
                Lang::ZhCn => tracing::info!("版本 {} 删除成功", version),
                Lang::EnUs => tracing::info!("Version {} deleted", version),
            }
            Ok(())
        }
        Ok(Err(e)) => match lang {
            Lang::ZhCn => Err(format!("删除失败: {}", e)),
            Lang::EnUs => Err(format!("Deletion failed: {}", e)),
        },
        Err(e) => match lang {
            Lang::ZhCn => Err(format!("任务执行失败: {}", e)),
            Lang::EnUs => Err(format!("Task failed: {}", e)),
        },
    }
}

// ─── 检查 ST 是否为空 ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn check_sillytavern_empty(app: AppHandle) -> Result<bool, String> {
    let lang = get_current_lang(&app);
    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let st_dir = data_dir.join("sillytavern");
    if !st_dir.exists() {
        return Ok(true);
    }
    let entries = match fs::read_dir(&st_dir) {
        Ok(e) => e,
        Err(_) => return Ok(true),
    };
    let mut has_valid = false;
    for entry in entries {
        if let Ok(entry) = entry {
            let n = entry.file_name();
            let s = n.to_string_lossy();
            if s != ".gitkeep" && s != ".DS_Store" {
                has_valid = true;
                break;
            }
        }
    }
    match lang {
        Lang::ZhCn => tracing::info!("酒馆目录检查结果: isEmpty={}", !has_valid),
        Lang::EnUs => tracing::info!("SillyTavern directory isEmpty={}", !has_valid),
    }
    Ok(!has_valid)
}

// ─── 链接已有版本 ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn link_existing_sillytavern(
    app: AppHandle,
    package_json_path: String,
) -> Result<String, String> {
    let lang = get_current_lang(&app);
    let pkg_path = PathBuf::from(&package_json_path);
    if !pkg_path.exists() {
        return Err(match lang {
            Lang::ZhCn => "文件不存在".to_string(),
            Lang::EnUs => "File does not exist".to_string(),
        });
    }

    let target_dir = pkg_path
        .parent()
        .ok_or_else(|| match lang {
            Lang::ZhCn => "无法获取父目录".to_string(),
            Lang::EnUs => "Cannot get parent directory".to_string(),
        })?
        .to_path_buf();

    let content = fs::read_to_string(&pkg_path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let version = v
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| match lang {
            Lang::ZhCn => "在 package.json 中未找到 version 字段".to_string(),
            Lang::EnUs => "Version field not found in package.json".to_string(),
        })?
        .to_string();

    if version.trim().is_empty()
        || version.contains("..")
        || version.contains('/')
        || version.contains('\\')
    {
        return Err(match lang {
            Lang::ZhCn => "无效的版本号".to_string(),
            Lang::EnUs => "Invalid version number".to_string(),
        });
    }

    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let st_dir = data_dir.join("sillytavern");
    if !st_dir.exists() {
        fs::create_dir_all(&st_dir).map_err(|e| e.to_string())?;
    }

    let link_path = st_dir.join(&version);
    if link_path.exists() {
        let meta = fs::symlink_metadata(&link_path).ok();
        let mut is_link = false;
        if let Some(m) = meta {
            if m.file_type().is_symlink() {
                is_link = true;
            } else {
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::fs::MetadataExt;
                    let attrs = m.file_attributes();
                    if attrs & 0x400 != 0 {
                        is_link = true;
                    }
                }
            }
        }

        if is_link {
            #[cfg(target_os = "windows")]
            let _ = fs::remove_dir(&link_path).or_else(|_| fs::remove_file(&link_path));
            #[cfg(not(target_os = "windows"))]
            let _ = fs::remove_file(&link_path);
        } else {
            return Err(match lang {
                Lang::ZhCn => "该版本已存在且不是链接目录，请先手动删除".to_string(),
                Lang::EnUs => {
                    "Version already exists and is not a link, please delete it first".to_string()
                }
            });
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let status = std::process::Command::new("cmd")
            .args(&[
                "/C",
                "mklink",
                "/J",
                link_path.to_str().unwrap(),
                target_dir.to_str().unwrap(),
            ])
            .creation_flags(0x08000000)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(match lang {
                Lang::ZhCn => "创建目录链接失败".to_string(),
                Lang::EnUs => "Failed to create directory junction".to_string(),
            });
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(&target_dir, &link_path).map_err(|e| match lang {
            Lang::ZhCn => format!("创建软链接失败: {}", e),
            Lang::EnUs => format!("Failed to create symlink: {}", e),
        })?;
    }

    Ok(version)
}

// ─── ST 当前版本 ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_tavern_version(app: AppHandle) -> Result<crate::types::LocalTavernItem, String> {
    let _lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let config = read_app_config_from_disk(&app2);
        let ver_item = config.sillytavern.version;
        if ver_item.version.is_empty() {
            return Err("未设置".to_string());
        }
        let ver_dir = if ver_item.path.is_empty() {
            let data_dir = get_config_path(&app2)
                .parent()
                .unwrap_or(&PathBuf::from("."))
                .to_path_buf();
            data_dir.join("sillytavern").join(&ver_item.version)
        } else {
            PathBuf::from(&ver_item.path)
        };
        if !ver_dir.exists() {
            return Err("未安装".to_string());
        }
        let pkg = ver_dir.join("package.json");
        if pkg.exists() {
            if let Ok(content) = fs::read_to_string(&pkg) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(v) = parsed.get("version").and_then(|v| v.as_str()) {
                        let mut item = ver_item.clone();
                        item.version = v.to_string();
                        return Ok(item);
                    }
                }
            }
        }
        Ok(ver_item)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─── ST 配置文件路径 ────────────────────────────────────────────────────────────

fn get_st_config_path(app: &AppHandle, _version: &str) -> Result<PathBuf, String> {
    let data_dir = get_config_path(app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let st_data = data_dir.join("st_data");

    // 自动创建全局数据目录
    if !st_data.exists() {
        std::fs::create_dir_all(&st_data).map_err(|e| format!("无法创建全局数据目录：{}", e))?;
    }

    let global = st_data.join("config.yaml");

    // 如果全局配置不存在，直接使用模板创建
    if !global.exists() {
        std::fs::write(&global, DEFAULT_CONFIG_TEMPLATE)
            .map_err(|e| format!("无法创建默认配置文件：{}", e))?;
    }

    Ok(global)
}

/// 获取全局配置文件路径（不需要版本号）
fn get_st_global_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = get_config_path(app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let st_data = data_dir.join("st_data");

    // 自动创建全局数据目录
    if !st_data.exists() {
        std::fs::create_dir_all(&st_data).map_err(|e| format!("无法创建全局数据目录：{}", e))?;
    }

    let global = st_data.join("config.yaml");

    // 如果全局配置不存在，直接使用模板创建
    if !global.exists() {
        std::fs::write(&global, DEFAULT_CONFIG_TEMPLATE)
            .map_err(|e| format!("无法创建默认配置文件：{}", e))?;
    }

    Ok(global)
}

// ─── ST Config YAML 读写 ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn read_sillytavern_config(app: AppHandle, version: String) -> Result<String, String> {
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let path = get_st_config_path(&app2, &version)?;
        fs::read_to_string(&path).map_err(|e| match lang {
            Lang::ZhCn => format!("读取失败: {}", e),
            Lang::EnUs => format!("Read failed: {}", e),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn write_sillytavern_config(
    app: AppHandle,
    version: String,
    content: String,
) -> Result<(), String> {
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let path = get_st_config_path(&app2, &version)?;
        fs::write(&path, content).map_err(|e| match lang {
            Lang::ZhCn => format!("写入失败: {}", e),
            Lang::EnUs => format!("Write failed: {}", e),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn get_sillytavern_config_path(app: AppHandle, version: String) -> Result<String, String> {
    let path = get_st_config_path(&app, &version)?;
    Ok(path.to_string_lossy().to_string())
}

// ─── ST 高级配置解析 ────────────────────────────────────────────────────────────

fn parse_tavern_config_payload(yaml_str: &str) -> Result<TavernConfigPayload, String> {
    let root: serde_yaml::Value =
        serde_yaml::from_str(yaml_str).map_err(|e| format!("解析配置失败: {}", e))?;
    let mapping = root
        .as_mapping()
        .ok_or("配置文件格式无效，根节点必须是对象".to_string())?;

    let get_bool = |key: &str, default: bool| {
        mapping
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(default)
    };
    let get_i64 = |key: &str, default: i64| {
        mapping
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(serde_yaml::Value::as_i64)
            .unwrap_or(default)
    };
    let parse_str_seq = |value: Option<&serde_yaml::Value>, default: Vec<String>| -> Vec<String> {
        value
            .and_then(serde_yaml::Value::as_sequence)
            .map(|seq| {
                seq.iter()
                    .filter_map(serde_yaml::Value::as_str)
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or(default)
    };
    let parse_i64_seq = |value: Option<&serde_yaml::Value>, default: Vec<i64>| -> Vec<i64> {
        value
            .and_then(serde_yaml::Value::as_sequence)
            .map(|seq| {
                seq.iter()
                    .filter_map(serde_yaml::Value::as_i64)
                    .collect::<Vec<_>>()
            })
            .filter(|s| !s.is_empty())
            .unwrap_or(default)
    };
    let key = |s: &str| serde_yaml::Value::String(s.to_string());
    let sub = |k: &str| {
        mapping
            .get(serde_yaml::Value::String(k.to_string()))
            .and_then(serde_yaml::Value::as_mapping)
    };

    let listen_address = sub("listenAddress")
        .map(|m| TavernDualStackAddress {
            ipv4: m
                .get(key("ipv4"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("0.0.0.0")
                .to_string(),
            ipv6: m
                .get(key("ipv6"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("[::]")
                .to_string(),
        })
        .unwrap_or(TavernDualStackAddress {
            ipv4: "0.0.0.0".to_string(),
            ipv6: "[::]".to_string(),
        });

    let protocol = sub("protocol")
        .map(|m| TavernDualStackProtocol {
            ipv4: m
                .get(key("ipv4"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true),
            ipv6: m
                .get(key("ipv6"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
        })
        .unwrap_or(TavernDualStackProtocol {
            ipv4: true,
            ipv6: false,
        });

    let whitelist = parse_str_seq(
        mapping.get(key("whitelist")),
        vec!["::1".to_string(), "127.0.0.1".to_string()],
    );

    let basic_auth_user = sub("basicAuthUser")
        .map(|m| TavernBasicAuthUser {
            username: m
                .get(key("username"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("user")
                .to_string(),
            password: m
                .get(key("password"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("password")
                .to_string(),
        })
        .unwrap_or(TavernBasicAuthUser {
            username: "user".to_string(),
            password: "password".to_string(),
        });

    let cors = sub("cors")
        .map(|m| TavernCorsConfig {
            enabled: m
                .get(key("enabled"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true),
            origin: parse_str_seq(m.get(key("origin")), vec!["null".to_string()]),
            methods: parse_str_seq(m.get(key("methods")), vec!["OPTIONS".to_string()]),
            allowed_headers: parse_str_seq(m.get(key("allowedHeaders")), vec![]),
            exposed_headers: parse_str_seq(m.get(key("exposedHeaders")), vec![]),
            credentials: m
                .get(key("credentials"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
            max_age: m.get(key("maxAge")).and_then(serde_yaml::Value::as_i64),
        })
        .unwrap_or(TavernCorsConfig {
            enabled: true,
            origin: vec!["null".to_string()],
            methods: vec!["OPTIONS".to_string()],
            allowed_headers: vec![],
            exposed_headers: vec![],
            credentials: false,
            max_age: None,
        });

    let request_proxy = sub("requestProxy")
        .map(|m| TavernRequestProxyConfig {
            enabled: m
                .get(key("enabled"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
            url: m
                .get(key("url"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("")
                .to_string(),
            bypass: m
                .get(key("bypass"))
                .and_then(serde_yaml::Value::as_sequence)
                .map(|s| {
                    s.iter()
                        .filter_map(serde_yaml::Value::as_str)
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default(),
        })
        .unwrap_or(TavernRequestProxyConfig {
            enabled: false,
            url: "".to_string(),
            bypass: vec![],
        });

    let backups = sub("backups")
        .map(|item| {
            let common = item
                .get(key("common"))
                .and_then(serde_yaml::Value::as_mapping);
            let chat = item
                .get(key("chat"))
                .and_then(serde_yaml::Value::as_mapping);
            TavernBackupsConfig {
                common: TavernBackupsCommonConfig {
                    number_of_backups: common
                        .and_then(|x| {
                            x.get(key("numberOfBackups"))
                                .and_then(serde_yaml::Value::as_i64)
                        })
                        .unwrap_or(50),
                },
                chat: TavernBackupsChatConfig {
                    enabled: chat
                        .and_then(|x| x.get(key("enabled")).and_then(serde_yaml::Value::as_bool))
                        .unwrap_or(true),
                    check_integrity: chat
                        .and_then(|x| {
                            x.get(key("checkIntegrity"))
                                .and_then(serde_yaml::Value::as_bool)
                        })
                        .unwrap_or(true),
                    max_total_backups: chat
                        .and_then(|x| {
                            x.get(key("maxTotalBackups"))
                                .and_then(serde_yaml::Value::as_i64)
                        })
                        .unwrap_or(-1),
                    throttle_interval: chat
                        .and_then(|x| {
                            x.get(key("throttleInterval"))
                                .and_then(serde_yaml::Value::as_i64)
                        })
                        .unwrap_or(10000),
                },
            }
        })
        .unwrap_or(TavernBackupsConfig {
            common: TavernBackupsCommonConfig {
                number_of_backups: 50,
            },
            chat: TavernBackupsChatConfig {
                enabled: true,
                check_integrity: true,
                max_total_backups: -1,
                throttle_interval: 10000,
            },
        });

    let thumbnails = sub("thumbnails")
        .map(|item| {
            let dims = item
                .get(key("dimensions"))
                .and_then(serde_yaml::Value::as_mapping);
            TavernThumbnailsConfig {
                enabled: item
                    .get(key("enabled"))
                    .and_then(serde_yaml::Value::as_bool)
                    .unwrap_or(true),
                format: item
                    .get(key("format"))
                    .and_then(serde_yaml::Value::as_str)
                    .unwrap_or("jpg")
                    .to_string(),
                quality: item
                    .get(key("quality"))
                    .and_then(serde_yaml::Value::as_i64)
                    .unwrap_or(95),
                dimensions: TavernThumbnailsDimensionsConfig {
                    bg: parse_i64_seq(dims.and_then(|x| x.get(key("bg"))), vec![160, 90]),
                    avatar: parse_i64_seq(dims.and_then(|x| x.get(key("avatar"))), vec![96, 144]),
                    persona: parse_i64_seq(dims.and_then(|x| x.get(key("persona"))), vec![96, 144]),
                },
            }
        })
        .unwrap_or(TavernThumbnailsConfig {
            enabled: true,
            format: "jpg".to_string(),
            quality: 95,
            dimensions: TavernThumbnailsDimensionsConfig {
                bg: vec![160, 90],
                avatar: vec![96, 144],
                persona: vec![96, 144],
            },
        });

    let (browser_launch_enabled, browser_type) = sub("browserLaunch")
        .map(|m| {
            let e = m
                .get(key("enabled"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true);
            let b = m
                .get(key("browser"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("default")
                .to_string();
            (e, b)
        })
        .unwrap_or((true, "default".to_string()));

    // SSL/TLS 配置
    let ssl = sub("ssl")
        .map(|m| TavernSslConfig {
            enabled: m
                .get(key("enabled"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
            cert_path: m
                .get(key("certPath"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("./certs/cert.pem")
                .to_string(),
            key_path: m
                .get(key("keyPath"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("./certs/privkey.pem")
                .to_string(),
            key_passphrase: m
                .get(key("keyPassphrase"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("")
                .to_string(),
        })
        .unwrap_or(TavernSslConfig::default());

    // DNS 和网络高级选项
    let dns_prefer_ipv6 = get_bool("dnsPreferIPv6", false);
    let heartbeat_interval = get_i64("heartbeatInterval", 0);

    let host_whitelist = sub("hostWhitelist")
        .map(|m| TavernHostWhitelistConfig {
            enabled: m
                .get(key("enabled"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
            scan: m
                .get(key("scan"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true),
            hosts: parse_str_seq(m.get(key("hosts")), vec![]),
        })
        .unwrap_or(TavernHostWhitelistConfig::default());

    let whitelist_import_domains =
        parse_str_seq(mapping.get(key("whitelistImportDomains")), vec![]);

    // 会话和安全
    let session_timeout = get_i64("sessionTimeout", -1);
    let disable_csrf_protection = get_bool("disableCsrfProtection", false);
    let security_override = get_bool("securityOverride", false);
    let allow_keys_exposure = get_bool("allowKeysExposure", false);
    let skip_content_check = get_bool("skipContentCheck", false);

    // 日志
    let logging = sub("logging")
        .map(|m| TavernLoggingConfig {
            enable_access_log: m
                .get(key("enableAccessLog"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true),
            min_log_level: m
                .get(key("minLogLevel"))
                .and_then(serde_yaml::Value::as_i64)
                .unwrap_or(0),
        })
        .unwrap_or(TavernLoggingConfig::default());

    // 性能
    let performance = sub("performance")
        .map(|m| TavernPerformanceConfig {
            lazy_load_characters: m
                .get(key("lazyLoadCharacters"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
            memory_cache_capacity: m
                .get(key("memoryCacheCapacity"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("100mb")
                .to_string(),
            use_disk_cache: m
                .get(key("useDiskCache"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true),
        })
        .unwrap_or(TavernPerformanceConfig::default());

    // 缓存清除
    let cache_buster = sub("cacheBuster")
        .map(|m| TavernCacheBusterConfig {
            enabled: m
                .get(key("enabled"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
            user_agent_pattern: m
                .get(key("userAgentPattern"))
                .and_then(serde_yaml::Value::as_str)
                .unwrap_or("")
                .to_string(),
        })
        .unwrap_or(TavernCacheBusterConfig::default());

    // SSO
    let sso = sub("sso")
        .map(|m| TavernSsoConfig {
            authelia_auth: m
                .get(key("autheliaAuth"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
            authentik_auth: m
                .get(key("authentikAuth"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false),
        })
        .unwrap_or(TavernSsoConfig::default());

    // 扩展
    let extensions = sub("extensions")
        .map(|m| TavernExtensionsConfig {
            enabled: m
                .get(key("enabled"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true),
            auto_update: m
                .get(key("autoUpdate"))
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true),
        })
        .unwrap_or(TavernExtensionsConfig::default());

    // 服务器插件
    let enable_server_plugins = get_bool("enableServerPlugins", false);
    let enable_server_plugins_auto_update = get_bool("enableServerPluginsAutoUpdate", true);

    // 其他
    let enable_cors_proxy = get_bool("enableCorsProxy", false);
    let prompt_placeholder = mapping
        .get(key("promptPlaceholder"))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or("[Start a new chat]")
        .to_string();
    let enable_downloadable_tokenizers = get_bool("enableDownloadableTokenizers", true);

    Ok(TavernConfigPayload {
        port: get_i64("port", 8000),
        listen: get_bool("listen", false),
        listen_address,
        protocol,
        basic_auth_mode: get_bool("basicAuthMode", false),
        enable_user_accounts: get_bool("enableUserAccounts", false),
        enable_discreet_login: get_bool("enableDiscreetLogin", false),
        per_user_basic_auth: get_bool("perUserBasicAuth", false),
        basic_auth_user,
        whitelist_mode: get_bool("whitelistMode", true),
        whitelist,
        cors,
        request_proxy,
        backups,
        thumbnails,
        browser_launch_enabled,
        browser_type,
        ssl,
        dns_prefer_ipv6,
        heartbeat_interval,
        host_whitelist,
        whitelist_import_domains,
        session_timeout,
        disable_csrf_protection,
        security_override,
        allow_keys_exposure,
        skip_content_check,
        logging,
        performance,
        cache_buster,
        sso,
        extensions,
        enable_server_plugins,
        enable_server_plugins_auto_update,
        enable_cors_proxy,
        prompt_placeholder,
        enable_downloadable_tokenizers,
    })
}

fn upsert(m: &mut serde_yaml::Mapping, k: &str, v: serde_yaml::Value) {
    m.insert(serde_yaml::Value::String(k.to_string()), v);
}

fn child_map<'a>(
    m: &'a mut serde_yaml::Mapping,
    k: &str,
) -> Result<&'a mut serde_yaml::Mapping, String> {
    let ck = serde_yaml::Value::String(k.to_string());
    if !m.contains_key(&ck) {
        m.insert(
            ck.clone(),
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
    }
    m.get_mut(&ck)
        .and_then(serde_yaml::Value::as_mapping_mut)
        .ok_or(format!("{} 配置格式无效", k))
}

#[tauri::command]
pub async fn get_sillytavern_config_options(
    app: AppHandle,
    version: String,
) -> Result<TavernConfigPayload, String> {
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let path = get_st_config_path(&app2, &version)?;
        let content = fs::read_to_string(&path).map_err(|e| match lang {
            Lang::ZhCn => format!("读取失败: {}", e),
            Lang::EnUs => format!("Read failed: {}", e),
        })?;
        parse_tavern_config_payload(&content)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_sillytavern_config_options(
    app: AppHandle,
    version: String,
    config: TavernConfigPayload,
) -> Result<TavernConfigPayload, String> {
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let path = get_st_config_path(&app2, &version)?;
        let content = fs::read_to_string(&path).map_err(|e| match lang {
            Lang::ZhCn => format!("读取失败: {}", e),
            Lang::EnUs => format!("Read failed: {}", e),
        })?;
        let mut root: serde_yaml::Value =
            serde_yaml::from_str(&content).map_err(|e| match lang {
                Lang::ZhCn => format!("解析配置失败: {}", e),
                Lang::EnUs => format!("Parse failed: {}", e),
            })?;
        let m = root
            .as_mapping_mut()
            .ok_or("配置文件格式无效，根节点必须是对象".to_string())?;

        upsert(
            m,
            "port",
            serde_yaml::Value::Number(serde_yaml::Number::from(config.port)),
        );
        upsert(m, "listen", serde_yaml::Value::Bool(config.listen));
        {
            let la = child_map(m, "listenAddress")?;
            upsert(
                la,
                "ipv4",
                serde_yaml::Value::String(config.listen_address.ipv4.clone()),
            );
            upsert(
                la,
                "ipv6",
                serde_yaml::Value::String(config.listen_address.ipv6.clone()),
            );
        }
        {
            let p = child_map(m, "protocol")?;
            upsert(p, "ipv4", serde_yaml::Value::Bool(config.protocol.ipv4));
            upsert(p, "ipv6", serde_yaml::Value::Bool(config.protocol.ipv6));
        }
        upsert(
            m,
            "basicAuthMode",
            serde_yaml::Value::Bool(config.basic_auth_mode),
        );
        upsert(
            m,
            "enableUserAccounts",
            serde_yaml::Value::Bool(config.enable_user_accounts),
        );
        upsert(
            m,
            "enableDiscreetLogin",
            serde_yaml::Value::Bool(config.enable_discreet_login),
        );
        upsert(
            m,
            "perUserBasicAuth",
            serde_yaml::Value::Bool(config.per_user_basic_auth),
        );
        {
            let bau = child_map(m, "basicAuthUser")?;
            upsert(
                bau,
                "username",
                serde_yaml::Value::String(config.basic_auth_user.username.clone()),
            );
            upsert(
                bau,
                "password",
                serde_yaml::Value::String(config.basic_auth_user.password.clone()),
            );
        }
        upsert(
            m,
            "whitelistMode",
            serde_yaml::Value::Bool(config.whitelist_mode),
        );
        upsert(
            m,
            "whitelist",
            serde_yaml::Value::Sequence(
                config
                    .whitelist
                    .iter()
                    .map(|s| serde_yaml::Value::String(s.clone()))
                    .collect(),
            ),
        );
        {
            let c = child_map(m, "cors")?;
            upsert(c, "enabled", serde_yaml::Value::Bool(config.cors.enabled));
            upsert(
                c,
                "origin",
                serde_yaml::Value::Sequence(
                    config
                        .cors
                        .origin
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
            upsert(
                c,
                "methods",
                serde_yaml::Value::Sequence(
                    config
                        .cors
                        .methods
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
            upsert(
                c,
                "allowedHeaders",
                serde_yaml::Value::Sequence(
                    config
                        .cors
                        .allowed_headers
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
            upsert(
                c,
                "exposedHeaders",
                serde_yaml::Value::Sequence(
                    config
                        .cors
                        .exposed_headers
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
            upsert(
                c,
                "credentials",
                serde_yaml::Value::Bool(config.cors.credentials),
            );
            upsert(
                c,
                "maxAge",
                config
                    .cors
                    .max_age
                    .map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(v)))
                    .unwrap_or(serde_yaml::Value::Null),
            );
        }
        {
            let rp = child_map(m, "requestProxy")?;
            upsert(
                rp,
                "enabled",
                serde_yaml::Value::Bool(config.request_proxy.enabled),
            );
            upsert(
                rp,
                "url",
                serde_yaml::Value::String(config.request_proxy.url.clone()),
            );
            upsert(
                rp,
                "bypass",
                serde_yaml::Value::Sequence(
                    config
                        .request_proxy
                        .bypass
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
        {
            let bk = child_map(m, "backups")?;
            {
                let bc = child_map(bk, "common")?;
                upsert(
                    bc,
                    "numberOfBackups",
                    serde_yaml::Value::Number(serde_yaml::Number::from(
                        config.backups.common.number_of_backups,
                    )),
                );
            }
            {
                let bch = child_map(bk, "chat")?;
                upsert(
                    bch,
                    "enabled",
                    serde_yaml::Value::Bool(config.backups.chat.enabled),
                );
                upsert(
                    bch,
                    "checkIntegrity",
                    serde_yaml::Value::Bool(config.backups.chat.check_integrity),
                );
                upsert(
                    bch,
                    "maxTotalBackups",
                    serde_yaml::Value::Number(serde_yaml::Number::from(
                        config.backups.chat.max_total_backups,
                    )),
                );
                upsert(
                    bch,
                    "throttleInterval",
                    serde_yaml::Value::Number(serde_yaml::Number::from(
                        config.backups.chat.throttle_interval,
                    )),
                );
            }
        }
        {
            let th = child_map(m, "thumbnails")?;
            upsert(
                th,
                "enabled",
                serde_yaml::Value::Bool(config.thumbnails.enabled),
            );
            upsert(
                th,
                "format",
                serde_yaml::Value::String(config.thumbnails.format.clone()),
            );
            upsert(
                th,
                "quality",
                serde_yaml::Value::Number(serde_yaml::Number::from(config.thumbnails.quality)),
            );
            {
                let d = child_map(th, "dimensions")?;
                upsert(
                    d,
                    "bg",
                    serde_yaml::Value::Sequence(
                        config
                            .thumbnails
                            .dimensions
                            .bg
                            .iter()
                            .map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(*v)))
                            .collect(),
                    ),
                );
                upsert(
                    d,
                    "avatar",
                    serde_yaml::Value::Sequence(
                        config
                            .thumbnails
                            .dimensions
                            .avatar
                            .iter()
                            .map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(*v)))
                            .collect(),
                    ),
                );
                upsert(
                    d,
                    "persona",
                    serde_yaml::Value::Sequence(
                        config
                            .thumbnails
                            .dimensions
                            .persona
                            .iter()
                            .map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(*v)))
                            .collect(),
                    ),
                );
            }
        }
        {
            let bl = child_map(m, "browserLaunch")?;
            upsert(
                bl,
                "enabled",
                serde_yaml::Value::Bool(config.browser_launch_enabled),
            );
            upsert(
                bl,
                "browser",
                serde_yaml::Value::String(config.browser_type.clone()),
            );
        }

        let new_content = serde_yaml::to_string(&root).map_err(|e| match lang {
            Lang::ZhCn => format!("序列化配置失败: {}", e),
            Lang::EnUs => format!("Serialize failed: {}", e),
        })?;
        fs::write(&path, new_content).map_err(|e| match lang {
            Lang::ZhCn => format!("写入失败: {}", e),
            Lang::EnUs => format!("Write failed: {}", e),
        })?;
        Ok(config)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn open_sillytavern_config_file(app: AppHandle, version: String) -> Result<(), String> {
    let path = get_st_config_path(&app, &version)?;
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(path);
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
        cmd.spawn().map_err(|e| format!("打开失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开失败: {}", e))?;
    }
    Ok(())
}

// ─── 全局配置操作（不需要版本号） ────────────────────────────────────────────────

#[tauri::command]
pub async fn get_sillytavern_global_config_options(
    app: AppHandle,
) -> Result<TavernConfigPayload, String> {
    let lang = get_current_lang(&app);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let path = get_st_global_config_path(&app2)?;
        let content = fs::read_to_string(&path).map_err(|e| match lang {
            Lang::ZhCn => format!("读取失败：{}", e),
            Lang::EnUs => format!("Read failed: {}", e),
        })?;
        parse_tavern_config_payload(&content)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_sillytavern_global_config_options(
    app: AppHandle,
    config: TavernConfigPayload,
) -> Result<TavernConfigPayload, String> {
    let lang = get_current_lang(&app);
    match lang {
        Lang::ZhCn => tracing::info!(
            "开始保存全局酒馆配置，dnsPreferIPv6={}",
            config.dns_prefer_ipv6
        ),
        Lang::EnUs => tracing::info!(
            "Starting to save global tavern config, dnsPreferIPv6={}",
            config.dns_prefer_ipv6
        ),
    }
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let path = get_st_global_config_path(&app2)?;
        let content = fs::read_to_string(&path).map_err(|e| match lang {
            Lang::ZhCn => format!("读取失败：{}", e),
            Lang::EnUs => format!("Read failed: {}", e),
        })?;
        let mut root: serde_yaml::Value =
            serde_yaml::from_str(&content).map_err(|e| match lang {
                Lang::ZhCn => format!("解析配置失败：{}", e),
                Lang::EnUs => format!("Parse failed: {}", e),
            })?;
        let m = root
            .as_mapping_mut()
            .ok_or("配置文件格式无效，根节点必须是对象".to_string())?;

        upsert(
            m,
            "port",
            serde_yaml::Value::Number(serde_yaml::Number::from(config.port)),
        );
        upsert(m, "listen", serde_yaml::Value::Bool(config.listen));
        {
            let la = child_map(m, "listenAddress")?;
            upsert(
                la,
                "ipv4",
                serde_yaml::Value::String(config.listen_address.ipv4.clone()),
            );
            upsert(
                la,
                "ipv6",
                serde_yaml::Value::String(config.listen_address.ipv6.clone()),
            );
        }
        {
            let p = child_map(m, "protocol")?;
            upsert(p, "ipv4", serde_yaml::Value::Bool(config.protocol.ipv4));
            upsert(p, "ipv6", serde_yaml::Value::Bool(config.protocol.ipv6));
        }
        upsert(
            m,
            "basicAuthMode",
            serde_yaml::Value::Bool(config.basic_auth_mode),
        );
        upsert(
            m,
            "enableUserAccounts",
            serde_yaml::Value::Bool(config.enable_user_accounts),
        );
        upsert(
            m,
            "enableDiscreetLogin",
            serde_yaml::Value::Bool(config.enable_discreet_login),
        );
        upsert(
            m,
            "perUserBasicAuth",
            serde_yaml::Value::Bool(config.per_user_basic_auth),
        );
        {
            let bau = child_map(m, "basicAuthUser")?;
            upsert(
                bau,
                "username",
                serde_yaml::Value::String(config.basic_auth_user.username.clone()),
            );
            upsert(
                bau,
                "password",
                serde_yaml::Value::String(config.basic_auth_user.password.clone()),
            );
        }
        upsert(
            m,
            "whitelistMode",
            serde_yaml::Value::Bool(config.whitelist_mode),
        );
        upsert(
            m,
            "whitelist",
            serde_yaml::Value::Sequence(
                config
                    .whitelist
                    .iter()
                    .map(|s| serde_yaml::Value::String(s.clone()))
                    .collect(),
            ),
        );
        {
            let c = child_map(m, "cors")?;
            upsert(c, "enabled", serde_yaml::Value::Bool(config.cors.enabled));
            upsert(
                c,
                "origin",
                serde_yaml::Value::Sequence(
                    config
                        .cors
                        .origin
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
            upsert(
                c,
                "methods",
                serde_yaml::Value::Sequence(
                    config
                        .cors
                        .methods
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
            upsert(
                c,
                "allowedHeaders",
                serde_yaml::Value::Sequence(
                    config
                        .cors
                        .allowed_headers
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
            upsert(
                c,
                "exposedHeaders",
                serde_yaml::Value::Sequence(
                    config
                        .cors
                        .exposed_headers
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
            upsert(
                c,
                "credentials",
                serde_yaml::Value::Bool(config.cors.credentials),
            );
            upsert(
                c,
                "maxAge",
                config
                    .cors
                    .max_age
                    .map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(v)))
                    .unwrap_or(serde_yaml::Value::Null),
            );
        }
        {
            let rp = child_map(m, "requestProxy")?;
            upsert(
                rp,
                "enabled",
                serde_yaml::Value::Bool(config.request_proxy.enabled),
            );
            upsert(
                rp,
                "url",
                serde_yaml::Value::String(config.request_proxy.url.clone()),
            );
            upsert(
                rp,
                "bypass",
                serde_yaml::Value::Sequence(
                    config
                        .request_proxy
                        .bypass
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
        {
            let bk = child_map(m, "backups")?;
            {
                let bc = child_map(bk, "common")?;
                upsert(
                    bc,
                    "numberOfBackups",
                    serde_yaml::Value::Number(serde_yaml::Number::from(
                        config.backups.common.number_of_backups,
                    )),
                );
            }
            {
                let bch = child_map(bk, "chat")?;
                upsert(
                    bch,
                    "enabled",
                    serde_yaml::Value::Bool(config.backups.chat.enabled),
                );
                upsert(
                    bch,
                    "checkIntegrity",
                    serde_yaml::Value::Bool(config.backups.chat.check_integrity),
                );
                upsert(
                    bch,
                    "maxTotalBackups",
                    serde_yaml::Value::Number(serde_yaml::Number::from(
                        config.backups.chat.max_total_backups,
                    )),
                );
                upsert(
                    bch,
                    "throttleInterval",
                    serde_yaml::Value::Number(serde_yaml::Number::from(
                        config.backups.chat.throttle_interval,
                    )),
                );
            }
        }
        {
            let th = child_map(m, "thumbnails")?;
            upsert(
                th,
                "enabled",
                serde_yaml::Value::Bool(config.thumbnails.enabled),
            );
            upsert(
                th,
                "format",
                serde_yaml::Value::String(config.thumbnails.format.clone()),
            );
            upsert(
                th,
                "quality",
                serde_yaml::Value::Number(serde_yaml::Number::from(config.thumbnails.quality)),
            );
            {
                let d = child_map(th, "dimensions")?;
                upsert(
                    d,
                    "bg",
                    serde_yaml::Value::Sequence(
                        config
                            .thumbnails
                            .dimensions
                            .bg
                            .iter()
                            .map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(*v)))
                            .collect(),
                    ),
                );
                upsert(
                    d,
                    "avatar",
                    serde_yaml::Value::Sequence(
                        config
                            .thumbnails
                            .dimensions
                            .avatar
                            .iter()
                            .map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(*v)))
                            .collect(),
                    ),
                );
                upsert(
                    d,
                    "persona",
                    serde_yaml::Value::Sequence(
                        config
                            .thumbnails
                            .dimensions
                            .persona
                            .iter()
                            .map(|v| serde_yaml::Value::Number(serde_yaml::Number::from(*v)))
                            .collect(),
                    ),
                );
            }
        }
        {
            let bl = child_map(m, "browserLaunch")?;
            upsert(
                bl,
                "enabled",
                serde_yaml::Value::Bool(config.browser_launch_enabled),
            );
            upsert(
                bl,
                "browser",
                serde_yaml::Value::String(config.browser_type.clone()),
            );
        }

        // SSL/TLS 配置
        {
            let ssl = child_map(m, "ssl")?;
            upsert(ssl, "enabled", serde_yaml::Value::Bool(config.ssl.enabled));
            upsert(
                ssl,
                "certPath",
                serde_yaml::Value::String(config.ssl.cert_path.clone()),
            );
            upsert(
                ssl,
                "keyPath",
                serde_yaml::Value::String(config.ssl.key_path.clone()),
            );
            upsert(
                ssl,
                "keyPassphrase",
                serde_yaml::Value::String(config.ssl.key_passphrase.clone()),
            );
        }

        // DNS 和网络高级选项
        upsert(
            m,
            "dnsPreferIPv6",
            serde_yaml::Value::Bool(config.dns_prefer_ipv6),
        );
        upsert(
            m,
            "heartbeatInterval",
            serde_yaml::Value::Number(serde_yaml::Number::from(config.heartbeat_interval)),
        );
        {
            let hw = child_map(m, "hostWhitelist")?;
            upsert(
                hw,
                "enabled",
                serde_yaml::Value::Bool(config.host_whitelist.enabled),
            );
            upsert(
                hw,
                "scan",
                serde_yaml::Value::Bool(config.host_whitelist.scan),
            );
            upsert(
                hw,
                "hosts",
                serde_yaml::Value::Sequence(
                    config
                        .host_whitelist
                        .hosts
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
        upsert(
            m,
            "whitelistImportDomains",
            serde_yaml::Value::Sequence(
                config
                    .whitelist_import_domains
                    .iter()
                    .map(|s| serde_yaml::Value::String(s.clone()))
                    .collect(),
            ),
        );

        // 会话和安全
        upsert(
            m,
            "sessionTimeout",
            serde_yaml::Value::Number(serde_yaml::Number::from(config.session_timeout)),
        );
        upsert(
            m,
            "disableCsrfProtection",
            serde_yaml::Value::Bool(config.disable_csrf_protection),
        );
        upsert(
            m,
            "securityOverride",
            serde_yaml::Value::Bool(config.security_override),
        );
        upsert(
            m,
            "allowKeysExposure",
            serde_yaml::Value::Bool(config.allow_keys_exposure),
        );
        upsert(
            m,
            "skipContentCheck",
            serde_yaml::Value::Bool(config.skip_content_check),
        );

        // 日志
        {
            let log = child_map(m, "logging")?;
            upsert(
                log,
                "enableAccessLog",
                serde_yaml::Value::Bool(config.logging.enable_access_log),
            );
            upsert(
                log,
                "minLogLevel",
                serde_yaml::Value::Number(serde_yaml::Number::from(config.logging.min_log_level)),
            );
        }

        // 性能
        {
            let perf = child_map(m, "performance")?;
            upsert(
                perf,
                "lazyLoadCharacters",
                serde_yaml::Value::Bool(config.performance.lazy_load_characters),
            );
            upsert(
                perf,
                "memoryCacheCapacity",
                serde_yaml::Value::String(config.performance.memory_cache_capacity.clone()),
            );
            upsert(
                perf,
                "useDiskCache",
                serde_yaml::Value::Bool(config.performance.use_disk_cache),
            );
        }

        // 缓存清除
        {
            let cb = child_map(m, "cacheBuster")?;
            upsert(
                cb,
                "enabled",
                serde_yaml::Value::Bool(config.cache_buster.enabled),
            );
            upsert(
                cb,
                "userAgentPattern",
                serde_yaml::Value::String(config.cache_buster.user_agent_pattern.clone()),
            );
        }

        // SSO
        {
            let sso = child_map(m, "sso")?;
            upsert(
                sso,
                "autheliaAuth",
                serde_yaml::Value::Bool(config.sso.authelia_auth),
            );
            upsert(
                sso,
                "authentikAuth",
                serde_yaml::Value::Bool(config.sso.authentik_auth),
            );
        }

        // 扩展
        {
            let ext = child_map(m, "extensions")?;
            upsert(
                ext,
                "enabled",
                serde_yaml::Value::Bool(config.extensions.enabled),
            );
            upsert(
                ext,
                "autoUpdate",
                serde_yaml::Value::Bool(config.extensions.auto_update),
            );
        }

        // 服务器插件
        upsert(
            m,
            "enableServerPlugins",
            serde_yaml::Value::Bool(config.enable_server_plugins),
        );
        upsert(
            m,
            "enableServerPluginsAutoUpdate",
            serde_yaml::Value::Bool(config.enable_server_plugins_auto_update),
        );

        // 其他
        upsert(
            m,
            "enableCorsProxy",
            serde_yaml::Value::Bool(config.enable_cors_proxy),
        );
        upsert(
            m,
            "promptPlaceholder",
            serde_yaml::Value::String(config.prompt_placeholder.clone()),
        );
        upsert(
            m,
            "enableDownloadableTokenizers",
            serde_yaml::Value::Bool(config.enable_downloadable_tokenizers),
        );

        let new_content = serde_yaml::to_string(&root).map_err(|e| match lang {
            Lang::ZhCn => format!("序列化配置失败：{}", e),
            Lang::EnUs => format!("Serialize failed: {}", e),
        })?;
        // 直接写入字符串，不需要引用
        fs::write(&path, new_content).map_err(|e| match lang {
            Lang::ZhCn => format!("写入失败：{}", e),
            Lang::EnUs => format!("Write failed: {}", e),
        })?;
        match lang {
            Lang::ZhCn => tracing::info!("全局酒馆配置保存成功到：{:?}", path),
            Lang::EnUs => tracing::info!("Global tavern config saved successfully to: {:?}", path),
        }
        Ok(config)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn open_sillytavern_global_config_file(app: AppHandle) -> Result<(), String> {
    let path = get_st_global_config_path(&app)?;
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(path);
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
        cmd.spawn().map_err(|e| format!("打开失败：{}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开失败：{}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开失败：{}", e))?;
    }
    Ok(())
}

// ─── 配置迁移 ────────────────────────────────────────────────────────────────

/// 返回每个本地酒馆实例下 default/config.yaml 的路径（仅返回实际存在的文件）
#[tauri::command]
pub async fn list_config_migration_sources(
    app: AppHandle,
) -> Result<Vec<serde_json::Value>, String> {
    let lang = get_current_lang(&app);
    let config = crate::config::get_app_config(app.clone()).await?;
    let mut results: Vec<serde_json::Value> = Vec::new();

    for item in &config.local_sillytavern_list {
        if item.path.is_empty() {
            continue;
        }
        let tavern_dir = PathBuf::from(&item.path);
        let config_path = tavern_dir.join("default").join("config.yaml");
        if config_path.exists() {
            let display = match lang {
                Lang::ZhCn => format!("{} ({})", item.version, item.path),
                Lang::EnUs => format!("{} ({})", item.version, item.path),
            };
            results.push(serde_json::json!({
                "path": config_path.to_string_lossy(),
                "tavernPath": item.path,
                "version": item.version,
                "display": display,
            }));
        }
    }

    Ok(results)
}

/// 将指定的 config.yaml 覆盖到 st_data/config.yaml
#[tauri::command]
pub async fn migrate_tavern_config(app: AppHandle, source_path: String) -> Result<(), String> {
    let lang = get_current_lang(&app);
    let src = PathBuf::from(&source_path);

    if !src.exists() {
        return match lang {
            Lang::ZhCn => Err(format!("源配置文件不存在：{}", source_path)),
            Lang::EnUs => Err(format!("Source config file not found: {}", source_path)),
        };
    }

    let dest = get_st_global_config_path(&app)?;

    // 确保目标目录存在
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| match lang {
                Lang::ZhCn => format!("无法创建目标目录：{}", e),
                Lang::EnUs => format!("Failed to create target directory: {}", e),
            })?;
        }
    }

    std::fs::copy(&src, &dest).map_err(|e| match lang {
        Lang::ZhCn => format!("配置迁移失败：{}", e),
        Lang::EnUs => format!("Config migration failed: {}", e),
    })?;

    tracing::info!("配置迁移成功: {:?} -> {:?}", src, dest);
    Ok(())
}

// ─── 资源迁移 ─────────────────────────────────────────────────────────────────

use crate::types::{ConflictFile, MigrationProgressEvent, ResourceMigrationSource};

/// 黑名单：迁移时始终跳过的目录/文件（相对于 data/）
/// 注意：扩展目录不在此处硬编码，由前端 exclude_categories 动态控制
fn is_resource_blacklisted(rel: &str) -> bool {
    // 统一用 `/` 分隔符比较
    let norm = rel.replace('\\', "/");
    norm.starts_with("_webpack/")
        || norm == "_webpack"
        || norm == "cookie-secret.txt"
        || norm.starts_with("default-user/characters/Seraphina/")
        || norm == "default-user/characters/Seraphina"
        // 顶层 extensions/（老版本可能存在的全局扩展目录）
        || norm.starts_with("extensions/")
        || norm == "extensions"
}

/// 根据 exclude_categories 判断某个文件是否应跳过
/// `excluded` 是用户选择排除的「友好类别名称」集合（与 infer_category 返回值一致）
fn is_category_excluded(rel: &str, excluded: &std::collections::HashSet<String>) -> bool {
    if excluded.is_empty() {
        return false;
    }
    let category = infer_category(rel);
    excluded.contains(&category)
}

/// 从相对路径推断友好类别名称
fn infer_category(rel: &str) -> String {
    let norm = rel.replace('\\', "/");
    let parts: Vec<&str> = norm.splitn(4, '/').collect();
    // 路径格式通常是 default-user/{subdir}/... 或顶层文件
    let subdir = if parts.len() >= 2 && parts[0] == "default-user" {
        parts[1]
    } else {
        parts[0]
    };
    match subdir {
        "characters" => "角色卡".to_string(),
        "worlds" => "世界书".to_string(),
        "backgrounds" => "聊天背景".to_string(),
        "chats" => "历史聊天记录".to_string(),
        "backups" => "备份".to_string(),
        "user-avatars" => "用户头像".to_string(),
        "personas" => "角色扮演人设".to_string(),
        "themes" => "主题".to_string(),
        "movingUI" => "移动UI布局".to_string(),
        "QuickReplies" => "快捷回复".to_string(),
        "assets" => "资源文件".to_string(),
        "context" => "上下文模板".to_string(),
        "instruct" => "指令模板".to_string(),
        "sysprompt" => "系统提示词".to_string(),
        "openai_histories" => "OpenAI对话历史".to_string(),
        "vectors" => "向量数据".to_string(),
        _ => subdir.to_string(),
    }
}

/// 扫描所有本地酒馆实例，返回拥有 data 目录的来源列表
#[tauri::command]
pub async fn list_resource_migration_sources(
    app: AppHandle,
) -> Result<Vec<ResourceMigrationSource>, String> {
    let config = crate::config::get_app_config(app.clone()).await?;
    let mut results = Vec::new();

    for item in &config.local_sillytavern_list {
        if item.path.is_empty() {
            continue;
        }
        let data_path = PathBuf::from(&item.path).join("data");
        if data_path.exists() && data_path.is_dir() {
            let display = if item.version.is_empty() {
                item.path.clone()
            } else {
                format!("{} ({})", item.version, item.path)
            };
            results.push(ResourceMigrationSource {
                tavern_path: item.path.clone(),
                data_path: data_path.to_string_lossy().to_string(),
                version: item.version.clone(),
                display,
            });
        }
    }

    Ok(results)
}

/// 扫描给定来源的 data 目录，返回与目标 st_data 冲突的文件列表
/// `source_paths` 是用户勾选的多个 data 目录的绝对路径
/// `exclude_categories_per_source` 按 source_paths 顺序，每个来源各自要排除的分类列表
/// `priority_source_path` 可选，标记优先级来源（扫描时不影响逻辑，仅用于参数对齐）
#[tauri::command]
pub async fn scan_migration_conflicts(
    app: AppHandle,
    source_paths: Vec<String>,
    source_displays: Vec<String>,
    exclude_categories_per_source: Option<Vec<Vec<String>>>,
    priority_source_path: Option<String>,
) -> Result<Vec<ConflictFile>, String> {
    use std::collections::HashSet;
    let _ = priority_source_path; // 扫描时不影响逻辑

    // 按 index 构建每个来源的排除集合；若不足则用空集合补足
    let per_source_excluded: Vec<HashSet<String>> = {
        let raw = exclude_categories_per_source.unwrap_or_default();
        (0..source_paths.len())
            .map(|i| {
                raw.get(i)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect()
            })
            .collect()
    };

    let dest_root = {
        let data_dir = get_config_path(&app)
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        data_dir.join("st_data")
    };

    let mut conflicts: Vec<ConflictFile> = Vec::new();

    for (idx, (src_path_str, display)) in
        source_paths.iter().zip(source_displays.iter()).enumerate()
    {
        let src_root = PathBuf::from(src_path_str);
        if !src_root.exists() {
            continue;
        }
        let excluded = &per_source_excluded[idx];

        // 递归遍历 src_root 下所有文件
        let walker = walkdir::WalkDir::new(&src_root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            let full_src = entry.path().to_path_buf();
            let rel = match full_src.strip_prefix(&src_root) {
                Ok(r) => r.to_string_lossy().to_string(),
                Err(_) => continue,
            };

            // 跳过黑名单
            if is_resource_blacklisted(&rel) {
                continue;
            }
            // 跳过该来源用户自定义排除分类
            if is_category_excluded(&rel, excluded) {
                continue;
            }

            let full_dest = dest_root.join(&rel);
            if full_dest.exists() {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let category = infer_category(&rel);
                conflicts.push(ConflictFile {
                    rel_path: rel,
                    source_full_path: full_src.to_string_lossy().to_string(),
                    dest_full_path: full_dest.to_string_lossy().to_string(),
                    source_display: display.clone(),
                    size,
                    category,
                });
            }
        }
    }

    Ok(conflicts)
}

/// 执行资源迁移
/// - `source_paths` / `source_displays`：选中的来源列表
/// - `overwrite_rel_paths`：用户已确认可以覆盖的相对路径集合
/// - `skip_rel_paths`：用户选择跳过的相对路径集合（冲突文件中未选覆盖的）
/// - `exclude_categories_per_source`：按 source_paths 顺序，每个来源各自要排除的分类列表
/// - `priority_source_path`：优先级来源 dataPath，该来源的文件将最后执行，确保最终结果以该来源为准
///
/// settings.json 始终进行 JSON 深度合并（不直接覆盖）。
/// 迁移进度通过 `resource-migration-progress` 事件推送。
#[tauri::command]
pub async fn execute_resource_migration(
    app: AppHandle,
    source_paths: Vec<String>,
    _source_displays: Vec<String>,
    overwrite_rel_paths: Vec<String>,
    skip_rel_paths: Vec<String>,
    exclude_categories_per_source: Option<Vec<Vec<String>>>,
    priority_source_path: Option<String>,
) -> Result<(), String> {
    use std::collections::HashSet;
    use tauri::Emitter;

    // 按 index 构建每个来源的排除集合
    let per_source_excluded: Vec<HashSet<String>> = {
        let raw = exclude_categories_per_source.unwrap_or_default();
        (0..source_paths.len())
            .map(|i| {
                raw.get(i)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect()
            })
            .collect()
    };

    let dest_root = {
        let data_dir = get_config_path(&app)
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        data_dir.join("st_data")
    };

    // 确保目标根目录存在
    if !dest_root.exists() {
        std::fs::create_dir_all(&dest_root).map_err(|e| format!("无法创建目标目录：{}", e))?;
    }

    let overwrite_set: HashSet<String> = overwrite_rel_paths.into_iter().collect();
    let skip_set: HashSet<String> = skip_rel_paths.into_iter().collect();

    // 先统计总文件数（用于进度显示）
    let mut all_files: Vec<(PathBuf, PathBuf, String)> = Vec::new(); // (src, dest, rel)

    for (idx, src_path_str) in source_paths.iter().enumerate() {
        let src_root = PathBuf::from(src_path_str);
        if !src_root.exists() {
            continue;
        }
        let excluded = &per_source_excluded[idx];
        let walker = walkdir::WalkDir::new(&src_root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            let full_src = entry.path().to_path_buf();
            let rel = match full_src.strip_prefix(&src_root) {
                Ok(r) => r.to_string_lossy().to_string(),
                Err(_) => continue,
            };
            if is_resource_blacklisted(&rel) {
                continue;
            }
            if is_category_excluded(&rel, excluded) {
                continue;
            }
            let full_dest = dest_root.join(&rel);
            all_files.push((full_src, full_dest, rel));
        }
    }

    // ── 优先级排序：将 priority_source_path 来源的文件移到末尾，确保其最后执行 ──
    if let Some(ref priority_path) = priority_source_path {
        let priority_root = PathBuf::from(priority_path);
        // 分离：非优先级 + 优先级
        let (non_priority, priority_files): (Vec<_>, Vec<_>) = all_files
            .into_iter()
            .partition(|(full_src, _, _)| !full_src.starts_with(&priority_root));
        all_files = non_priority;
        all_files.extend(priority_files);
    }

    let total = all_files.len();
    let mut done = 0usize;

    let emit_progress = |done: usize, current: &str, finished: bool, error: Option<String>| {
        let _ = app.emit(
            "resource-migration-progress",
            MigrationProgressEvent {
                done,
                total,
                current: current.to_string(),
                finished,
                error,
            },
        );
    };

    for (full_src, full_dest, rel) in &all_files {
        // 先发进度
        emit_progress(done, rel, false, None);

        // 目标已存在时的处理
        if full_dest.exists() {
            // settings.json 特殊处理：深度合并
            let norm_rel = rel.replace('\\', "/");
            if norm_rel == "default-user/settings.json" {
                if let Err(e) = merge_settings_json(full_src, full_dest) {
                    emit_progress(done, rel, false, Some(e.clone()));
                    // 合并失败不终止，继续
                }
                done += 1;
                continue;
            }

            // 用户选择跳过
            if skip_set.contains(rel) {
                done += 1;
                continue;
            }
            // 用户未选覆盖 → 跳过
            if !overwrite_set.contains(rel) {
                done += 1;
                continue;
            }
        }

        // 确保父目录存在
        if let Some(parent) = full_dest.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    emit_progress(done, rel, false, Some(format!("创建目录失败：{}", e)));
                    done += 1;
                    continue;
                }
            }
        }

        if let Err(e) = std::fs::copy(full_src, full_dest) {
            emit_progress(done, rel, false, Some(format!("复制失败 {}: {}", rel, e)));
        }

        done += 1;
    }

    emit_progress(done, "", true, None);
    Ok(())
}

/// 深度合并两个 settings.json（src 的值递归合并到 dest）
fn merge_settings_json(src: &PathBuf, dest: &PathBuf) -> Result<(), String> {
    let src_text =
        std::fs::read_to_string(src).map_err(|e| format!("读取源 settings.json 失败：{}", e))?;
    let dest_text =
        std::fs::read_to_string(dest).map_err(|e| format!("读取目标 settings.json 失败：{}", e))?;

    let src_val: serde_json::Value =
        serde_json::from_str(&src_text).map_err(|e| format!("解析源 settings.json 失败：{}", e))?;
    let mut dest_val: serde_json::Value = serde_json::from_str(&dest_text)
        .map_err(|e| format!("解析目标 settings.json 失败：{}", e))?;

    json_merge(&mut dest_val, &src_val);

    let merged = serde_json::to_string_pretty(&dest_val)
        .map_err(|e| format!("序列化合并结果失败：{}", e))?;
    std::fs::write(dest, merged).map_err(|e| format!("写入合并后 settings.json 失败：{}", e))?;
    Ok(())
}

/// 递归合并 JSON：src 中的字段覆盖/追加到 dest
fn json_merge(dest: &mut serde_json::Value, src: &serde_json::Value) {
    match (dest, src) {
        (serde_json::Value::Object(d), serde_json::Value::Object(s)) => {
            for (k, v) in s {
                let entry = d.entry(k.clone()).or_insert(serde_json::Value::Null);
                json_merge(entry, v);
            }
        }
        (dest, src) => {
            *dest = src.clone();
        }
    }
}

// ─── 启动 / 停止 / 状态 ────────────────────────────────────────────────────────

pub fn generate_default_settings_for_version(app: &AppHandle, version: &str) -> Result<(), String> {
    let lang = get_current_lang(app);

    let data_dir = get_config_path(app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let st_data = data_dir.join("st_data");

    if !st_data.exists() {
        std::fs::create_dir_all(&st_data).map_err(|e| match lang {
            Lang::ZhCn => format!("无法创建全局数据目录：{}", e),
            Lang::EnUs => format!("Failed to create global data directory: {}", e),
        })?;
    }

    // 初始化 default-user/settings.json
    let default_user_dir = st_data.join("default-user");
    if !default_user_dir.exists() {
        std::fs::create_dir_all(&default_user_dir).map_err(|e| match lang {
            Lang::ZhCn => format!("无法创建 default-user 目录：{}", e),
            Lang::EnUs => format!("Failed to create default-user directory: {}", e),
        })?;
    }
    let default_settings_path = default_user_dir.join("settings.json");
    if !default_settings_path.exists() {
        if let Ok(mut settings) =
            serde_json::from_str::<serde_json::Value>(crate::types::DEFAULT_SETTINGS_JSON)
        {
            if let Some(obj) = settings.as_object_mut() {
                obj.insert("currentVersion".to_string(), serde_json::json!(version));
            }
            let mut buf = Vec::new();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
            let content_to_write = if serde::Serialize::serialize(&settings, &mut ser).is_ok() {
                String::from_utf8(buf)
                    .unwrap_or_else(|_| crate::types::DEFAULT_SETTINGS_JSON.to_string())
            } else {
                crate::types::DEFAULT_SETTINGS_JSON.to_string()
            };
            std::fs::write(&default_settings_path, content_to_write).map_err(|e| match lang {
                Lang::ZhCn => format!("无法写入 settings.json：{}", e),
                Lang::EnUs => format!("Failed to write settings.json: {}", e),
            })?;
        } else {
            std::fs::write(&default_settings_path, crate::types::DEFAULT_SETTINGS_JSON).map_err(
                |e| match lang {
                    Lang::ZhCn => format!("无法写入 settings.json：{}", e),
                    Lang::EnUs => format!("Failed to write settings.json: {}", e),
                },
            )?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn start_sillytavern(
    app: AppHandle,
    state: tauri::State<'_, ProcessState>,
) -> Result<(), String> {
    let lang = get_current_lang(&app);
    let mut kill_tx_guard = state.kill_tx.lock().await;
    if kill_tx_guard.is_some() {
        return match lang {
            Lang::ZhCn => Err("进程已经在运行中了".to_string()),
            Lang::EnUs => Err("Process is already running".to_string()),
        };
    }

    // 检查端口是否被占用 (默认端口 11451)
    let port = 11451u16;
    let ipv4_addr = format!("127.0.0.1:{}", port);
    let ipv6_addr = format!("[::]:{}", port);

    let port_in_use = match tokio::net::TcpListener::bind(&ipv6_addr).await {
        Ok(listener) => {
            drop(listener);
            false
        }
        Err(_) => true,
    } || match tokio::net::TcpListener::bind(&ipv4_addr).await {
        Ok(listener) => {
            drop(listener);
            false
        }
        Err(_) => true,
    };

    if port_in_use {
        // 端口被占用,尝试停止可能残留的进程
        tracing::warn!(
            "Port {} is in use, trying to cleanup existing process",
            port
        );
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let mut cmd = std::process::Command::new("powershell");
            cmd.args(["-Command", &format!("Get-NetTCPConnection -LocalPort {} -ErrorAction SilentlyContinue | ForEach-Object {{ Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }}", port)])
               .creation_flags(0x08000000);
            let _ = cmd.output();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("sh")
                .args([
                    "-c",
                    &format!("lsof -ti:{} | xargs kill -9 2>/dev/null || true", port),
                ])
                .output();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("sh")
                .args([
                    "-c",
                    &format!("lsof -ti:{} | xargs kill -9 2>/dev/null || true", port),
                ])
                .output();
        }

        // 等待一下让进程终止
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 再次检查端口
        let still_in_use = match tokio::net::TcpListener::bind(&ipv6_addr).await {
            Ok(listener) => {
                drop(listener);
                false
            }
            Err(_) => true,
        } || match tokio::net::TcpListener::bind(&ipv4_addr).await {
            Ok(listener) => {
                drop(listener);
                false
            }
            Err(_) => true,
        };

        if still_in_use {
            return match lang {
                Lang::ZhCn => Err(format!("端口 {} 仍被占用,请手动检查并关闭占用进程", port)),
                Lang::EnUs => Err(format!(
                    "Port {} is still in use. Please check and close the process manually.",
                    port
                )),
            };
        }
    }

    let config = read_app_config_from_disk(&app);
    let version_item = config.sillytavern.version;
    if version_item.version.is_empty() {
        return match lang {
            Lang::ZhCn => Err("未选择酒馆版本，请先在版本页面选择或安装".to_string()),
            Lang::EnUs => Err("No version selected".to_string()),
        };
    }

    let data_dir = get_config_path(&app)
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let st_dir = if version_item.path.is_empty() {
        data_dir.join("sillytavern").join(&version_item.version)
    } else {
        PathBuf::from(&version_item.path)
    };
    let st_data = data_dir.join("st_data");

    if !st_data.exists() {
        std::fs::create_dir_all(&st_data).map_err(|e| format!("无法创建全局数据目录：{}", e))?;
    }

    if !st_dir.exists() {
        return match lang {
            Lang::ZhCn => Err(format!("版本 {} 的目录不存在", version_item.version)),
            Lang::EnUs => Err(format!(
                "Directory for version {} not found",
                version_item.version
            )),
        };
    }

    let mut node_path = if cfg!(target_os = "windows") {
        data_dir.join("node").join("node.exe")
    } else {
        data_dir.join("node").join("bin/node")
    };
    if !node_path.exists() {
        node_path = PathBuf::from("node");
    }

    let server_js = st_dir.join("server.js");
    if !server_js.exists() {
        return match lang {
            Lang::ZhCn => Err("找不到 server.js，酒馆文件可能损坏".to_string()),
            Lang::EnUs => Err("server.js not found".to_string()),
        };
    }

    // ─── 启动前依赖完整性检查 & 精准修复 ────────────────────────────────────
    // 读取酒馆自身的 package.json，对比 dependencies 与 node_modules 实际状况
    let node_modules = st_dir.join("node_modules");
    let pkg_json_path = st_dir.join("package.json");

    // 从 package.json 解析出所有 dependencies 的包名
    let required_packages: Vec<String> = if pkg_json_path.exists() {
        match std::fs::read_to_string(&pkg_json_path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        {
            Some(json) => {
                let mut pkgs = Vec::new();
                if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
                    pkgs.extend(deps.keys().cloned());
                }
                pkgs
            }
            None => Vec::new(),
        }
    } else {
        // 没有 package.json 时回退到检查核心包
        vec!["express".to_string(), "socket.io".to_string()]
    };

    // 找出 node_modules 中缺失的包
    let missing_packages: Vec<String> = required_packages
        .iter()
        .filter(|pkg| !node_modules.join(pkg.as_str()).exists())
        .cloned()
        .collect();

    if !missing_packages.is_empty() {
        let pkg_list = missing_packages.join(", ");
        let repair_msg = match lang {
            Lang::ZhCn => format!(
                "INFO: 检测到以下依赖缺失，正在自动安装（请耐心等待）：{}",
                pkg_list
            ),
            Lang::EnUs => format!(
                "INFO: Missing dependencies detected, auto-installing: {}",
                pkg_list
            ),
        };
        let _ = app.emit("process-log", repair_msg);

        match crate::node::run_npm_install_packages(&app, &st_dir, &missing_packages).await {
            Ok(()) => {
                let ok_msg = match lang {
                    Lang::ZhCn => "INFO: 缺失依赖已安装完成，继续启动酒馆...".to_string(),
                    Lang::EnUs => {
                        "INFO: Missing dependencies installed. Continuing startup...".to_string()
                    }
                };
                let _ = app.emit("process-log", ok_msg);
            }
            Err(e) => {
                let err_msg = match lang {
                    Lang::ZhCn => format!("ERROR: 依赖安装失败，启动中止：{}", e),
                    Lang::EnUs => format!("ERROR: Dependency installation failed, aborting: {}", e),
                };
                let _ = app.emit("process-log", err_msg.clone());
                return Err(err_msg);
            }
        }
    }
    // ─────────────────────────────────────────────────────────────────────────

    let global_cfg = st_data.join("config.yaml");
    // 使用 display() 而不是 canonicalize() 来避免 \\?\ 前缀
    let global_cfg_str = global_cfg.to_string_lossy().to_string();
    let st_data_str = st_data.to_string_lossy().to_string();

    // 调试日志: 打印配置路径和数据路径
    tracing::info!("SillyTavern dataRoot: {}", st_data_str);
    tracing::info!("SillyTavern configPath: {}", global_cfg_str);
    tracing::info!("SillyTavern global_cfg exists: {}", global_cfg.exists());

    let mut std_cmd = std::process::Command::new(&node_path);

    // ─── launch_mode 处理 ────────────────────────────────────────────────────
    let launch_mode = config.launch_mode.as_str();
    tracing::info!("SillyTavern launch_mode: {}", launch_mode);

    // DEBUG 模式：node --inspect server.js
    if launch_mode == "debug" {
        std_cmd.arg("--inspect");
    }

    // 判断实际 Node 版本是否 >= 18.19.0（--import 参数的最低支持版本）
    let import_supported = node_supports_import(&node_path);
    let node_ver_str = get_actual_node_version(&node_path)
        .map(|(a, b, c)| format!("v{}.{}.{}", a, b, c))
        .unwrap_or_else(|| "unknown".to_string());

    tracing::info!(
        "Node.js 版本检查: 实际 {}, --import 支持(>=18.19.0)={}",
        node_ver_str,
        import_supported
    );

    // --import 拦截脚本：仅在 GitHub 加速开启 且 Node >= 18.19.0 时才注入
    // 加速关闭 或 Node 版本过低 → 跳过（低版本不支持 ESM --import 参数）
    if config.github_proxy.enable && !config.github_proxy.url.is_empty() && import_supported {
        let proxy_url = config.github_proxy.url.trim_end_matches('/');
        // 创建临时的拦截脚本文件
        let interceptor_script = format!(
            r#"
// GitHub URL 拦截器 - 自动重写 GitHub 链接到镜像
import https from 'https';
import http from 'http';
const originalHttpsRequest = https.request;
const originalHttpsGet = https.get;
const originalHttpRequest = http.request;
const originalHttpGet = http.get;

const PROXY_URL = '{}';

function rewriteGitHubUrl(url) {{
    if (!url || typeof url !== 'string') return url;

    // 不重写 API 请求
    if (url.includes('api.github.com')) return url;

    // 只重写 GitHub 相关的 URL
    if (url.includes('github.com') || url.includes('raw.githubusercontent.com')) {{
        return PROXY_URL + '/' + url;
    }}

    return url;
}}

// 拦截 https.request
https.request = function(url, options, callback) {{
    let req;
    if (typeof url === 'string') {{
        const rewrittenUrl = rewriteGitHubUrl(url);
        req = originalHttpsRequest.call(https, rewrittenUrl, options, callback);
    }} else if (url && typeof url === 'object') {{
        if (url.href) {{
            const newUrl = Object.assign({{}}, url);
            newUrl.href = rewriteGitHubUrl(url.href);
            // 同步修改 host/hostname 等其他可能被使用的字段
            if (newUrl.href !== url.href) {{
                try {{
                    const parsed = new URL(newUrl.href);
                    newUrl.host = parsed.host;
                    newUrl.hostname = parsed.hostname;
                    newUrl.pathname = parsed.pathname;
                    newUrl.protocol = parsed.protocol;
                    newUrl.port = parsed.port;
                }} catch (e) {{}}
            }}
            req = originalHttpsRequest.call(https, newUrl, options, callback);
        }} else {{
            req = originalHttpsRequest.call(https, url, options, callback);
        }}
    }} else {{
        req = originalHttpsRequest.call(https, url, options, callback);
    }}

    // 拦截 request 的 write 方法，确保 URL 也被重写
    const originalWrite = req.write;
    req.write = function(chunk, encoding, callback) {{
        if (chunk && typeof chunk === 'string') {{
            try {{
                const data = JSON.parse(chunk);
                if (data.url && typeof data.url === 'string') {{
                    const rewrittenUrl = rewriteGitHubUrl(data.url);
                    if (rewrittenUrl !== data.url) {{
                        data.url = rewrittenUrl;
                        chunk = JSON.stringify(data);
                    }}
                }}
            }} catch (e) {{
                // 忽略解析错误
            }}
        }}
        return originalWrite.call(req, chunk, encoding, callback);
    }};

    return req;
}};

// 拦截 https.get
https.get = function(url, options, callback) {{
    if (typeof url === 'string') {{
        const rewrittenUrl = rewriteGitHubUrl(url);
        return originalHttpsGet.call(https, rewrittenUrl, options, callback);
    }} else if (url && typeof url === 'object') {{
        if (url.href) {{
            const newUrl = Object.assign({{}}, url);
            newUrl.href = rewriteGitHubUrl(url.href);
            if (newUrl.href !== url.href) {{
                try {{
                    const parsed = new URL(newUrl.href);
                    newUrl.host = parsed.host;
                    newUrl.hostname = parsed.hostname;
                    newUrl.pathname = parsed.pathname;
                    newUrl.protocol = parsed.protocol;
                    newUrl.port = parsed.port;
                }} catch (e) {{}}
            }}
            return originalHttpsGet.call(https, newUrl, options, callback);
        }}
        return originalHttpsGet.call(https, url, options, callback);
    }}
    return originalHttpsGet.call(https, url, options, callback);
}};

// 拦截 http.request（部分 GitHub 资源可能使用 HTTP）
http.request = function(url, options, callback) {{
    if (typeof url === 'string') {{
        const rewrittenUrl = rewriteGitHubUrl(url);
        return originalHttpRequest.call(http, rewrittenUrl, options, callback);
    }} else if (url && typeof url === 'object') {{
        if (url.href) {{
            const newUrl = Object.assign({{}}, url);
            newUrl.href = rewriteGitHubUrl(url.href);
            if (newUrl.href !== url.href) {{
                try {{
                    const parsed = new URL(newUrl.href);
                    newUrl.host = parsed.host;
                    newUrl.hostname = parsed.hostname;
                    newUrl.pathname = parsed.pathname;
                    newUrl.protocol = parsed.protocol;
                    newUrl.port = parsed.port;
                }} catch (e) {{}}
            }}
            return originalHttpRequest.call(http, newUrl, options, callback);
        }}
        return originalHttpRequest.call(http, url, options, callback);
    }}
    return originalHttpRequest.call(http, url, options, callback);
}};

// 拦截 http.get
http.get = function(url, options, callback) {{
    if (typeof url === 'string') {{
        const rewrittenUrl = rewriteGitHubUrl(url);
        return originalHttpGet.call(http, rewrittenUrl, options, callback);
    }} else if (url && typeof url === 'object') {{
        if (url.href) {{
            const newUrl = Object.assign({{}}, url);
            newUrl.href = rewriteGitHubUrl(url.href);
            if (newUrl.href !== url.href) {{
                try {{
                    const parsed = new URL(newUrl.href);
                    newUrl.host = parsed.host;
                    newUrl.hostname = parsed.hostname;
                    newUrl.pathname = parsed.pathname;
                    newUrl.protocol = parsed.protocol;
                    newUrl.port = parsed.port;
                }} catch (e) {{}}
            }}
            return originalHttpGet.call(http, newUrl, options, callback);
        }}
        return originalHttpGet.call(http, url, options, callback);
    }}
    return originalHttpGet.call(http, url, options, callback);
}};

console.log('[GitHub Proxy] URL interceptor loaded, proxy:', PROXY_URL);
"#,
            proxy_url
        );

        let interceptor_path = data_dir.join("github-proxy-interceptor.js");
        std::fs::write(&interceptor_path, interceptor_script)
            .map_err(|e| format!("Failed to create interceptor script: {}", e))?;

        // Windows 路径需要转换为 file:// URL 格式
        let interceptor_path_str = interceptor_path.to_string_lossy().to_string();
        #[cfg(target_os = "windows")]
        let interceptor_url = format!("file:///{}", interceptor_path_str.replace('\\', "/"));
        #[cfg(not(target_os = "windows"))]
        let interceptor_url = format!("file://{}", interceptor_path_str);

        // 使用 --import 参数在启动时加载拦截脚本 (ESM 模式)
        std_cmd.arg("--import").arg(&interceptor_url);
    }

    std_cmd.arg(&server_js);
    std_cmd.arg("--dataRoot").arg(&st_data_str);

    // 强制指定 configPath
    std_cmd.arg("--configPath").arg(&global_cfg_str);
    tracing::info!("SillyTavern will use config path: {}", global_cfg_str);

    // 桌面程序模式：禁止酒馆自动打开浏览器，由 Launcher 创建子窗口来展示
    if launch_mode == "desktop" {
        std_cmd.arg("--browserLaunchEnabled").arg("false");
    }

    // 局域网/公网服务模式：禁止自动打开浏览器
    if launch_mode == "lan" || launch_mode == "public" {
        std_cmd.arg("--browserLaunchEnabled").arg("false");
    }

    let path_env = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = std::env::split_paths(&path_env).collect::<Vec<_>>();

    let node_bin_dir = data_dir.join("node");
    if node_bin_dir.exists() {
        paths.insert(0, node_bin_dir.join("bin"));
        paths.insert(0, node_bin_dir);
    }

    // 检查系统是否有 Git，如果没有才添加 MinGit 到 PATH
    let mut git_check_cmd = std::process::Command::new("git");
    git_check_cmd
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        git_check_cmd.creation_flags(0x08000000);
    }
    let system_git_exists = git_check_cmd.status().is_ok();

    if !system_git_exists {
        let git_dir = data_dir.join("git");
        let git_bin_dir = if cfg!(target_os = "windows") {
            git_dir.join("cmd")
        } else {
            git_dir.join("bin")
        };
        if git_bin_dir.exists() {
            paths.insert(0, git_bin_dir);
        }
    }

    let new_path_env = std::env::join_paths(paths).unwrap_or(path_env);

    std_cmd
        .current_dir(&st_dir)
        .env("SILLYTAVERN_DATA_DIR", &st_data_str)
        .env("PATH", new_path_env);

    // 局域网/公网服务模式：通过环境变量设置白名单
    match launch_mode {
        "lan" => {
            // 仅允许局域网 IP 段访问
            std_cmd.env(
                "SILLYTAVERN_WHITELIST",
                r#"["127.0.0.1", "::1", "10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"]"#,
            );
            tracing::info!("局域网服务模式：已设置局域网白名单");
        }
        "public" => {
            // 允许所有 IP 访问（公网）
            std_cmd.env(
                "SILLYTAVERN_WHITELIST",
                r#"["127.0.0.1", "0.0.0.0/0", "::/0"]"#,
            );
            tracing::info!("公网服务模式：已开放全网访问白名单");
        }
        _ => {}
    }

    // ─── 网络代理环境变量设置 ─────────────────────────────────────────────────
    // 根据 network_proxy 配置设置 HTTP_PROXY / HTTPS_PROXY / NO_PROXY
    let proxy_config = &config.network_proxy;
    match proxy_config.mode {
        crate::types::ProxyMode::None => {
            tracing::info!("网络代理：未启用代理");
        }
        crate::types::ProxyMode::System => {
            // 读取系统代理设置
            if let Some((server, enabled)) = crate::config::read_windows_system_proxy() {
                if enabled {
                    // 解析 proxy server 地址，处理多协议格式如 "http=host:port;https=host:port"
                    let proxy_addr = if server.contains('=') {
                        server
                            .split(';')
                            .find(|s| s.starts_with("http="))
                            .map(|s| s.trim_start_matches("http="))
                            .unwrap_or(&server)
                            .to_string()
                    } else {
                        server.clone()
                    };
                    let proxy_url = format!("http://{}", proxy_addr);
                    std_cmd.env("HTTP_PROXY", &proxy_url);
                    std_cmd.env("HTTPS_PROXY", &proxy_url);
                    std_cmd.env("http_proxy", &proxy_url);
                    std_cmd.env("https_proxy", &proxy_url);
                    tracing::info!("网络代理（系统代理）：已设置 HTTP_PROXY={}", proxy_url);
                } else {
                    tracing::info!("网络代理：系统代理已配置但未启用");
                }
            } else {
                tracing::info!("网络代理：未检测到系统代理设置");
            }
        }
        crate::types::ProxyMode::Custom => {
            // 使用用户自定义代理配置
            let proxy_url = format!("http://{}:{}", proxy_config.host, proxy_config.port);
            std_cmd.env("HTTP_PROXY", &proxy_url);
            std_cmd.env("HTTPS_PROXY", &proxy_url);
            std_cmd.env("http_proxy", &proxy_url);
            std_cmd.env("https_proxy", &proxy_url);
            tracing::info!("网络代理（自定义）：已设置 HTTP_PROXY={}", proxy_url);
        }
    }

    // GitHub 加速策略（二选一，互斥）：
    // - Node >= 18.19.0：已通过 --import 拦截器处理，不再需要全局 git config
    // - Node < 18.19.0（不支持 --import）：改用全局 git config URL 重写做加速
    //   停止服务或软件关闭时会自动还原此配置
    if config.github_proxy.enable && !config.github_proxy.url.is_empty() && !import_supported {
        let proxy_url = config.github_proxy.url.trim_end_matches('/');
        let git_exe = get_git_exe(&app);
        tracing::info!(
            "GitHub 加速(git config 模式): proxy={}, git={}",
            proxy_url,
            git_exe.display()
        );
        set_git_global_proxy(&git_exe, proxy_url);
    }

    std_cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        std_cmd.creation_flags(0x08000000);
    }

    // 打印完整的启动命令
    if let Some(program) = std_cmd.get_program().to_str() {
        let args: Vec<String> = std_cmd
            .get_args()
            .filter_map(|a| a.to_str())
            .map(|s| s.to_string())
            .collect();
        tracing::info!("SillyTavern start command: {} {}", program, args.join(" "));
    }

    let mut cmd = tokio::process::Command::from(std_cmd);
    cmd.kill_on_drop(true);
    let mut child = cmd.spawn().map_err(|e| match lang {
        Lang::ZhCn => format!("启动进程失败: {}", e),
        Lang::EnUs => format!("Failed to start: {}", e),
    })?;

    *state.child_pid.lock().await = child.id();

    if let Some(pid) = child.id() {
        let msg = match lang {
            Lang::ZhCn => format!("INFO: 启动成功! 进程PID: {}", pid),
            Lang::EnUs => format!("INFO: Started successfully! Process PID: {}", pid),
        };
        let _ = app.emit("process-log", msg);
    }

    let stdout = child.stdout.take().ok_or("无法获取标准输出")?;
    let stderr = child.stderr.take().ok_or("无法获取标准错误")?;

    let is_desktop_mode = launch_mode == "desktop";
    let is_network_mode = launch_mode == "lan" || launch_mode == "public";
    let network_mode_str = launch_mode.to_string();

    let app1 = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        // 用于防止重复触发
        let mut desktop_window_opened = false;
        let mut network_port_sent = false;
        while let Ok(Some(line)) = reader.next_line().await {
            tracing::info!("ST_STDOUT: {}", line);
            let _ = app1.emit("process-log", format!("INFO: {}", line));

            // 桌面程序模式：检测酒馆启动成功后输出的访问地址
            if is_desktop_mode && !desktop_window_opened {
                let url_opt = extract_tavern_url(&line);
                if let Some(url) = url_opt {
                    tracing::info!("桌面模式检测到酒馆地址: {}", url);
                    desktop_window_opened = true;
                    let _ = app1.emit("tavern-desktop-ready", url);
                }
            }

            // 局域网/公网服务模式：提取端口，通知前端显示二维码弹窗
            if is_network_mode && !network_port_sent {
                let url_opt = extract_tavern_url(&line);
                if let Some(url) = url_opt {
                    // 提取端口号
                    let port = url
                        .rsplit(':')
                        .next()
                        .and_then(|p| p.trim_end_matches('/').parse::<u16>().ok())
                        .unwrap_or(8000);
                    tracing::info!(
                        "网络服务模式（{}）检测到酒馆端口: {}",
                        network_mode_str,
                        port
                    );
                    network_port_sent = true;
                    // 发送事件：{mode: "lan"|"public", port: 8000}
                    let payload = serde_json::json!({
                        "mode": network_mode_str,
                        "port": port,
                    });
                    let _ = app1.emit("tavern-network-ready", payload);
                }
            }
        }
    });

    let app2 = app.clone();
    let st_dir_for_repair = st_dir.clone();
    tokio::spawn(async move {
        use regex::Regex;
        // 匹配 Node.js 模块缺失错误中的包名：
        //   Cannot find package 'yargs' imported from ...
        //   Cannot find module 'express'
        //   Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'xxx'
        let re_package =
            Regex::new(r#"Cannot find (?:package|module) ['"`]([^'"`@/][^'"`]*)['"`]"#).unwrap();
        // 同时处理 scoped 包 @scope/pkg
        let re_scoped =
            Regex::new(r#"Cannot find (?:package|module) ['"`](@[^'"`]+)['"`]"#).unwrap();

        let mut missing_pkgs: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            tracing::error!("ST_STDERR: {}", line);
            let _ = app2.emit("process-log", format!("ERROR: {}", line));

            // 检测 MODULE_NOT_FOUND 错误
            if line.contains("ERR_MODULE_NOT_FOUND")
                || line.contains("Cannot find package")
                || line.contains("Cannot find module")
            {
                // 优先匹配 scoped 包（@scope/pkg），再匹配普通包
                let pkg_name = re_scoped
                    .captures(&line)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .or_else(|| {
                        re_package
                            .captures(&line)
                            .and_then(|c| c.get(1))
                            .map(|m| m.as_str().to_string())
                    });

                if let Some(pkg) = pkg_name {
                    // 过滤掉内置模块和相对路径导入
                    let is_builtin = matches!(
                        pkg.as_str(),
                        "path"
                            | "fs"
                            | "os"
                            | "url"
                            | "util"
                            | "events"
                            | "stream"
                            | "crypto"
                            | "http"
                            | "https"
                            | "net"
                            | "tls"
                            | "zlib"
                            | "child_process"
                            | "cluster"
                            | "readline"
                            | "buffer"
                            | "assert"
                            | "process"
                            | "querystring"
                            | "string_decoder"
                            | "timers"
                            | "vm"
                            | "worker_threads"
                            | "perf_hooks"
                            | "module"
                            | "domain"
                            | "dns"
                            | "dgram"
                            | "constants"
                    );
                    let is_relative = pkg.starts_with('.') || pkg.starts_with('/');

                    if !is_builtin && !is_relative && !pkg.is_empty() {
                        if missing_pkgs.insert(pkg.clone()) {
                            tracing::warn!("检测到运行时缺失包: {}", pkg);
                        }
                    }
                }
            }
        }

        // stderr 读完后，如果有缺失包，通知前端修复
        if !missing_pkgs.is_empty() {
            let pkgs: Vec<String> = missing_pkgs.into_iter().collect();
            tracing::warn!("运行时缺失包汇总，触发修复: {:?}", pkgs);
            let payload = serde_json::json!({
                "packages": pkgs,
                "st_dir": st_dir_for_repair.to_string_lossy(),
            });
            let _ = app2.emit("tavern-missing-dep", payload);
        }
    });

    let (kill_tx, mut kill_rx) = tokio::sync::mpsc::channel::<()>(1);
    *kill_tx_guard = Some(kill_tx);

    let app3 = app.clone();
    let kill_tx_arc = state.inner().kill_tx.clone();
    let child_pid_arc = state.inner().child_pid.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = child.wait() => { let _ = app3.emit("process-log", "INFO: 进程已退出".to_string()); }
            _ = kill_rx.recv() => { let _ = child.kill().await; let _ = app3.emit("process-log", "INFO: 进程已被终止".to_string()); }
        }
        *kill_tx_arc.lock().await = None;
        *child_pid_arc.lock().await = None;
        let _ = app3.emit("process-exit", ());
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_sillytavern(
    app: tauri::AppHandle,
    state: tauri::State<'_, ProcessState>,
) -> Result<(), String> {
    let mut guard = state.kill_tx.lock().await;
    let mut pid_guard = state.child_pid.lock().await;

    if guard.is_none() && pid_guard.is_none() {
        return Ok(());
    }

    tracing::info!("尝试停止酒馆...");
    if let Some(tx) = guard.take() {
        let _ = tx.send(()).await;
    }

    // 直接清理进程树（尤其是退出软件时可能因为异步不执行完毕）
    if let Some(pid) = pid_guard.take() {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let mut cmd = std::process::Command::new("taskkill");
            cmd.args(["/F", "/PID", &pid.to_string(), "/T"])
                .creation_flags(0x08000000);
            let _ = cmd.output();
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .output();
        }
    }

    // 还原全局 git config：
    // 只有在「加速开 + Node < 18.19.0」时才设置过全局 git config，才需要还原
    // （加速开 + Node >= 18.19.0 时用的是 --import 拦截器，没有动全局 git config）
    let config = read_app_config_from_disk(&app);
    if config.github_proxy.enable && !config.github_proxy.url.is_empty() {
        use tauri::Manager;
        let data_dir = app.path().app_data_dir().unwrap_or_default();
        let node_path = if cfg!(target_os = "windows") {
            data_dir.join("node").join("node.exe")
        } else {
            data_dir.join("node").join("bin/node")
        };
        let node_path = if node_path.exists() {
            node_path
        } else {
            std::path::PathBuf::from("node")
        };

        if !node_supports_import(&node_path) {
            let proxy_url = config.github_proxy.url.trim_end_matches('/').to_string();
            let git_exe = get_git_exe(&app);
            unset_git_global_proxy(&git_exe, &proxy_url);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn check_sillytavern_status(
    state: tauri::State<'_, ProcessState>,
) -> Result<bool, String> {
    Ok(state.kill_tx.lock().await.is_some())
}

/// 从酒馆启动日志中提取 HTTP 访问地址
/// 酒馆启动成功时通常会输出含 http://localhost:PORT 的行
fn extract_tavern_url(line: &str) -> Option<String> {
    // 匹配行中出现的 http://localhost:PORT 或 http://127.0.0.1:PORT 形式的 URL
    // 例如：
    //   "SillyTavern is listening on: http://localhost:11451"
    //   "Open http://localhost:11451 in your browser"
    //   "Listening on port 11451"（仅端口号形式，需要补全）
    let lower = line.to_ascii_lowercase();

    // 尝试直接找 http:// 开头的 URL
    if let Some(start) = line.find("http://") {
        let rest = &line[start..];
        // 取到第一个空格或引号为止
        let end = rest
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
            .unwrap_or(rest.len());
        let url = &rest[..end];
        if url.starts_with("http://localhost")
            || url.starts_with("http://127.0.0.1")
            || url.starts_with("http://0.0.0.0")
        {
            // 把 0.0.0.0 替换成 localhost
            let normalized = url.replace("http://0.0.0.0", "http://localhost");
            return Some(normalized);
        }
    }

    // 尝试找 "listening on port XXXX" 格式（只有端口号）
    if lower.contains("listening on port") || lower.contains("server is running") {
        // 找数字端口
        let port_re: Option<u16> = lower
            .split_whitespace()
            .filter_map(|tok| tok.trim_matches(':').parse::<u16>().ok())
            .find(|&p| p > 1024 && p < 65535);
        if let Some(port) = port_re {
            return Some(format!("http://localhost:{}", port));
        }
    }

    None
}

/// 桌面程序模式：创建并打开子窗口访问酒馆
/// 子窗口关闭时，自动停止酒馆服务
#[tauri::command]
pub async fn open_tavern_desktop_window(
    app: AppHandle,
    state: tauri::State<'_, ProcessState>,
    url: String,
) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    tracing::info!("打开桌面程序模式窗口: {}", url);

    // 如果已经有同名窗口，直接聚焦
    if let Some(existing) = app.get_webview_window("sillytavern-desktop") {
        let _ = existing.set_focus();
        return Ok(());
    }

    let tavern_url = WebviewUrl::External(url.parse().map_err(|e| format!("URL 解析失败: {}", e))?);

    let window = WebviewWindowBuilder::new(&app, "sillytavern-desktop", tavern_url)
        .title("SillyTavern Desktop")
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .focused(true)
        .center()
        .build()
        .map_err(|e| format!("创建子窗口失败: {}", e))?;

    // 监听子窗口关闭事件，关闭时自动停止酒馆服务
    let app_clone = app.clone();
    let kill_tx_arc = state.inner().kill_tx.clone();
    let child_pid_arc = state.inner().child_pid.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            tracing::info!("桌面程序窗口已关闭，正在停止酒馆服务...");
            let app2 = app_clone.clone();
            let kill_tx2 = kill_tx_arc.clone();
            let child_pid2 = child_pid_arc.clone();
            tauri::async_runtime::spawn(async move {
                // 先通知前端：这是主动停止，不要当作异常退出
                let _ = app2.emit("process-intentional-stop", ());

                // 发送 kill 信号
                {
                    let mut guard = kill_tx2.lock().await;
                    if let Some(tx) = guard.take() {
                        let _ = tx.send(()).await;
                    }
                }
                // 强制 kill 进程树
                if let Some(pid) = child_pid2.lock().await.take() {
                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;
                        let mut cmd = std::process::Command::new("taskkill");
                        cmd.args(["/F", "/PID", &pid.to_string(), "/T"])
                            .creation_flags(0x08000000);
                        let _ = cmd.output();
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let _ = std::process::Command::new("kill")
                            .args(["-9", &pid.to_string()])
                            .output();
                    }
                }
                tracing::info!("桌面程序模式：酒馆服务已停止");
                // process-exit 会由 stdout 监听任务自然触发，无需手动 emit
            });
        }
    });

    Ok(())
}

/// 获取本机局域网 IPv4 / IPv6 地址列表
#[tauri::command]
pub async fn get_local_ip_addresses() -> Result<serde_json::Value, String> {
    use std::net::{IpAddr, UdpSocket};

    #[derive(Debug, Clone)]
    struct LanInterface {
        name: String,
        desc: Option<String>,
        iface_type: Option<String>,
        is_up: bool,
        is_virtual_guess: bool,
        ipv4_addrs: Vec<String>,
        ipv6_addrs: Vec<String>,
    }

    let mut interfaces: Vec<LanInterface> = Vec::new();

    // ─── 基础探测：通过 UDP 连接探测当前默认出站接口 ────────────────────────────
    let mut primary_ipv4: Option<String> = None;
    let mut primary_ipv6: Option<String> = None;

    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if !ip.starts_with("127.") {
                    primary_ipv4 = Some(ip);
                }
            }
        }
    }

    if let Ok(socket) = UdpSocket::bind("[::]:0") {
        if socket.connect("[2001:4860:4860::8888]:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                if let IpAddr::V6(v6) = addr.ip() {
                    let s = v6.to_string();
                    if !s.starts_with("::1") && !s.starts_with("fe80") {
                        primary_ipv6 = Some(format!("[{}]", s));
                    }
                }
            }
        }
    }

    // ─── get_if_addrs 枚举所有网卡 ───────────────────────────────────────────────
    let mut ifaddr_map: std::collections::HashMap<String, LanInterface> = std::collections::HashMap::new();
    if let Ok(all_ifaces) = get_if_addrs::get_if_addrs() {
        for iface in all_ifaces {
            let name = iface.name.clone();
            let entry = ifaddr_map.entry(name.clone()).or_insert_with(|| LanInterface {
                name: name.clone(),
                desc: None,
                iface_type: None,
                is_up: true,
                is_virtual_guess: false,
                ipv4_addrs: Vec::new(),
                ipv6_addrs: Vec::new(),
            });
            let ip = iface.addr.ip();
            match ip {
                IpAddr::V4(v4) => {
                    let ip_str = v4.to_string();
                    if !ip_str.starts_with("127.") && !ip_str.starts_with("169.254") {
                        if entry.ipv4_addrs.iter().all(|x| x != &ip_str) {
                            entry.ipv4_addrs.push(ip_str);
                        }
                    }
                }
                IpAddr::V6(v6) => {
                    let s = v6.to_string();
                    if !s.starts_with("::1") && !s.starts_with("fe80") {
                        let formatted = format!("[{}]", s);
                        if entry.ipv6_addrs.iter().all(|x| x != &formatted) {
                            entry.ipv6_addrs.push(formatted);
                        }
                    }
                }
            }
        }
    }

    // ─── Windows 下获取网卡描述 / 状态 / InterfaceAlias ─────────────────────────
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use std::process::{Command, Stdio};

        fn run_powershell_json(script: &str) -> Option<serde_json::Value> {
            let output = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", script])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .ok()?;
            serde_json::from_slice(&output.stdout).ok()
        }

        if let Some(value) = run_powershell_json("Get-NetAdapter | Select-Object -Property Name, InterfaceDescription, Status, InterfaceAlias, ifIndex | ConvertTo-Json") {
            match value {
                serde_json::Value::Array(arr) => {
                    for item in arr {
                        if let Some(name) = item.get("Name").and_then(|v| v.as_str()) {
                            let desc = item
                                .get("InterfaceDescription")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let iface_type = item
                                .get("InterfaceAlias")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let status = item
                                .get("Status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string();

                            let entry = ifaddr_map.entry(name.to_string()).or_insert_with(|| LanInterface {
                                name: name.to_string(),
                                desc: None,
                                iface_type: None,
                                is_up: true,
                                is_virtual_guess: false,
                                ipv4_addrs: Vec::new(),
                                ipv6_addrs: Vec::new(),
                            });

                            entry.desc = desc.or_else(|| entry.desc.clone());
                            entry.iface_type = iface_type.or_else(|| entry.iface_type.clone());
                            entry.is_up = status.eq_ignore_ascii_case("up") || status.eq_ignore_ascii_case("connected");
                        }
                    }
                }
                serde_json::Value::Object(obj) => {
                    if let Some(name) = obj.get("Name").and_then(|v| v.as_str()) {
                        let desc = obj
                            .get("InterfaceDescription")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let iface_type = obj
                            .get("InterfaceAlias")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let status = obj
                            .get("Status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        let entry = ifaddr_map.entry(name.to_string()).or_insert_with(|| LanInterface {
                            name: name.to_string(),
                            desc: None,
                            iface_type: None,
                            is_up: true,
                            is_virtual_guess: false,
                            ipv4_addrs: Vec::new(),
                            ipv6_addrs: Vec::new(),
                        });

                        entry.desc = desc.or_else(|| entry.desc.clone());
                        entry.iface_type = iface_type.or_else(|| entry.iface_type.clone());
                        entry.is_up = status.eq_ignore_ascii_case("up") || status.eq_ignore_ascii_case("connected");
                    }
                }
                _ => {}
            }
        }
    }

    // ─── 构建最终接口列表 ──────────────────────────────────────────────────────
    for (_, mut iface) in ifaddr_map.into_iter() {
        iface.is_virtual_guess = is_virtual_adapter(iface.desc.as_deref(), iface.name.as_str());
        if iface.ipv4_addrs.is_empty() && iface.ipv6_addrs.is_empty() {
            continue;
        }
        interfaces.push(iface);
    }

    // 如果 UDP 探测有结果但接口列表为空，直接返回探测值
    if interfaces.is_empty() {
        let mut fallback_ipv4 = Vec::new();
        let mut fallback_ipv6 = Vec::new();
        if let Some(v4) = primary_ipv4 {
            fallback_ipv4.push(v4);
        }
        if let Some(v6) = primary_ipv6 {
            fallback_ipv6.push(v6);
        }
        return Ok(serde_json::json!({
            "ipv4": fallback_ipv4,
            "ipv6": fallback_ipv6,
            "interfaces": Vec::<serde_json::Value>::new(),
        }));
    }

    // 首选网卡：优先不是虚拟网卡、状态 Up、含 IPv4 地址的第一项；其次 IPv6；失败则取第一项
    let mut preferred_index: Option<usize> = None;
    for (idx, iface) in interfaces.iter().enumerate() {
        let name_lower = iface.name.to_lowercase();
        let is_blacklisted_name = name_lower.contains("虚拟") || name_lower.contains("virtual");
        if !iface.ipv4_addrs.is_empty() && !iface.is_virtual_guess && iface.is_up && !is_blacklisted_name {
            preferred_index = Some(idx);
            break;
        }
    }
    if preferred_index.is_none() {
        for (idx, iface) in interfaces.iter().enumerate() {
            let name_lower = iface.name.to_lowercase();
            let is_blacklisted_name = name_lower.contains("虚拟") || name_lower.contains("virtual");
            if !iface.ipv6_addrs.is_empty() && !iface.is_virtual_guess && iface.is_up && !is_blacklisted_name {
                preferred_index = Some(idx);
                break;
            }
        }
    }
    if preferred_index.is_none() && !interfaces.is_empty() {
        preferred_index = Some(0);
    }

    let mut ipv4_list: Vec<String> = Vec::new();
    let mut ipv6_list: Vec<String> = Vec::new();
    if let Some(index) = preferred_index {
        if let Some(preferred) = interfaces.get(index) {
            if let Some(ip) = preferred.ipv4_addrs.first() {
                ipv4_list.push(ip.clone());
            }
            if let Some(ip) = preferred.ipv6_addrs.first() {
                ipv6_list.push(ip.clone());
            }
        }
    }

    Ok(serde_json::json!({
        "ipv4": ipv4_list,
        "ipv6": ipv6_list,
        "interfaces": interfaces
            .into_iter()
            .map(|iface| serde_json::json!({
                "name": iface.name,
                "desc": iface.desc,
                "alias": iface.iface_type,
                "is_up": iface.is_up,
                "is_virtual": iface.is_virtual_guess,
                "ipv4": iface.ipv4_addrs,
                "ipv6": iface.ipv6_addrs,
            }))
            .collect::<Vec<_>>(),
    }))
}

fn is_virtual_adapter(desc: Option<&str>, name: &str) -> bool {
    let desc_lower = desc.unwrap_or("").to_lowercase();
    let name_lower = name.to_lowercase();
    let keywords = [
        "virtual",
        "vmware",
        "hyper-v",
        "loopback",
        "vnic",
        "vpn",
        "packet",
        "adapter for loopback",
        "docker",
        "wintap",
        "tap-",
        "pnet",
    ];
    for key in keywords {
        if desc_lower.contains(key) || name_lower.contains(key) {
            return true;
        }
    }
    false
}

/// 从本机网卡探测 GUA IPv6（全局单播地址，2xxx:: / 3xxx::）
/// 很多国内用户的 IPv6 就是公网地址，直接从网卡取即可
fn detect_local_gua_ipv6() -> Option<String> {
    use std::net::{IpAddr, UdpSocket};

    // 方法1：UDP 探测出站 IPv6（连接到 Google DNS，不实际发包）
    if let Ok(socket) = UdpSocket::bind("[::]:0") {
        if socket.connect("[2001:4860:4860::8888]:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                if let IpAddr::V6(v6) = addr.ip() {
                    let s = v6.to_string();
                    // GUA: 2xxx:: / 3xxx:: 开头，排除回环、本地链路、ULA
                    if is_global_unicast_v6(&s) {
                        return Some(format!("[{}]", s));
                    }
                }
            }
        }
    }

    // 方法2：Windows 下枚举网卡
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        if let Ok(out) = std::process::Command::new("powershell")
            .args([
                "-Command",
                "Get-NetIPAddress -AddressFamily IPv6 | Where-Object { $_.IPAddress -match '^2[0-9a-fA-F]' -or $_.IPAddress -match '^3[0-9a-fA-F]' } | Select-Object -ExpandProperty IPAddress",
            ])
            .creation_flags(0x08000000)
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                let ip = line.trim();
                if !ip.is_empty() && is_global_unicast_v6(ip) {
                    return Some(format!("[{}]", ip));
                }
            }
        }
    }

    None
}

/// 判断是否为全局单播 IPv6（2000::/3，即 2xxx:: 或 3xxx::）
fn is_global_unicast_v6(s: &str) -> bool {
    if s.starts_with("::1") || s.starts_with("fe80") || s.starts_with("fc") || s.starts_with("fd") {
        return false;
    }
    // 2000::/3：第一个字节高3位为 001，即 0x20~0x3F
    if let Some(first_hex) = s.split(':').next() {
        if let Ok(val) = u16::from_str_radix(first_hex, 16) {
            return val >= 0x2000 && val <= 0x3FFF;
        }
    }
    false
}

/// 尝试从单个 URL 获取纯文本 IP 字符串，超时 4s
async fn fetch_ip_text(client: &reqwest::Client, url: &str) -> Option<String> {
    let resp = client.get(url).send().await.ok()?;
    let text = resp.text().await.ok()?;
    let t = text.trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

/// 判断字符串是否像 IPv4
fn looks_like_ipv4(s: &str) -> bool {
    s.contains('.') && !s.contains(':')
}

/// 判断字符串是否像 IPv6
fn looks_like_ipv6(s: &str) -> bool {
    s.contains(':')
}

/// 格式化 IPv6 为带方括号形式（已有括号则原样返回）
fn fmt_ipv6(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('[') {
        s.to_string()
    } else {
        format!("[{}]", s)
    }
}

/// 获取公网 IP
///
/// 优先级：
///   IPv4: 4.ipw.cn → api4.ipify.org → ipv4.icanhazip.com → ipv4.ip.sb → ident.me
///   IPv6: 6.ipw.cn → api6.ipify.org → ipv6.icanhazip.com → ipv6.ip.sb → v6.ident.me
///         → 全部失败时 fallback 到本地网卡 GUA IPv6
///
/// 注意：preferred 不再由本函数决定，改由 check_network_availability 命令通过 itdog 可用性检测得出
#[tauri::command]
pub async fn get_public_ip_addresses() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?;

    // ── IPv4：依次尝试，首次成功即用 ────────────────────────────────────────
    let ipv4_apis: &[&str] = &[
        "https://4.ipw.cn",
        "https://api4.ipify.org",
        "https://ipv4.icanhazip.com",
        "https://ipv4.ip.sb",
        "https://ipv4.ident.me",
    ];
    let mut ipv4: Option<String> = None;
    for api in ipv4_apis {
        if let Some(t) = fetch_ip_text(&client, api).await {
            if looks_like_ipv4(&t) {
                ipv4 = Some(t);
                break;
            }
        }
    }

    // ── IPv6：依次尝试，全失败则 fallback 本地 GUA ──────────────────────────
    let ipv6_apis: &[&str] = &[
        "https://6.ipw.cn",
        "https://api6.ipify.org",
        "https://ipv6.icanhazip.com",
        "https://ipv6.ip.sb",
        "https://v6.ident.me",
    ];
    let mut ipv6: Option<String> = None;
    for api in ipv6_apis {
        if let Some(t) = fetch_ip_text(&client, api).await {
            if looks_like_ipv6(&t) {
                ipv6 = Some(fmt_ipv6(&t));
                break;
            }
        }
    }
    // 所有外部 API 全失败 → 本地网卡 GUA IPv6（国内 ISP 分配的 IPv6 通常就是公网地址）
    if ipv6.is_none() {
        ipv6 = detect_local_gua_ipv6();
    }

    Ok(serde_json::json!({
        "ipv4": ipv4,
        "ipv6": ipv6,
    }))
}

// ─── itdog TCPing 可用性检测（Chrome Headless 方案）──────────────────────────

/// 查找系统中安装的 Google Chrome 可执行文件路径（Windows）
fn find_chrome_executable() -> Option<std::path::PathBuf> {
    let candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    ];
    for path in &candidates {
        let p = std::path::Path::new(path);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }
    // 用户级安装（LOCALAPPDATA）
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let p = std::path::PathBuf::from(&local).join("Google\\Chrome\\Application\\chrome.exe");
        if p.exists() {
            return Some(p);
        }
    }
    // 注册表查询（CREATE_NO_WINDOW = 0x08000000，避免弹出命令行窗口）
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;
    if let Ok(out) = {
        let mut cmd = std::process::Command::new("reg");
        cmd.args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe",
            "/ve",
        ]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);
        cmd.output()
    } {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.contains("REG_SZ") {
                    if let Some(pos) = line.rfind("REG_SZ") {
                        let path_str = line[pos + 6..].trim();
                        let p = std::path::Path::new(path_str);
                        if p.exists() {
                            return Some(p.to_path_buf());
                        }
                    }
                }
            }
        }
    }
    None
}

/// 用 Chrome Headless + CDP 跑 itdog TCPing 检测，返回 (total, timeout_count)
fn itdog_tcping_chrome(
    chrome_path: &std::path::Path,
    ip_with_port: &str,
    is_ipv6: bool,
    progress_cb: impl Fn(&str),
) -> (u32, u32) {
    use headless_chrome::{Browser, LaunchOptions};

    let url_str = if is_ipv6 {
        format!("https://www.itdog.cn/tcping_ipv6/{}", ip_with_port)
    } else {
        format!("https://www.itdog.cn/tcping/{}", ip_with_port)
    };
    let proto = if is_ipv6 { "IPv6" } else { "IPv4" };

    tracing::info!("[itdog] {} Chrome Headless 加载: {}", proto, url_str);
    progress_cb("injecting");

    let browser = match Browser::new(
        match LaunchOptions::default_builder()
            .path(Some(chrome_path.to_path_buf()))
            .headless(true)
            .build()
        {
            Ok(opts) => opts,
            Err(e) => {
                tracing::warn!("[itdog] {} LaunchOptions 构建失败: {}", proto, e);
                return (0, 0);
            }
        },
    ) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("[itdog] {} Chrome 启动失败: {}", proto, e);
            return (0, 0);
        }
    };

    let tab = match browser.new_tab() {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("[itdog] {} 新建 Tab 失败: {}", proto, e);
            return (0, 0);
        }
    };

    if let Err(e) = tab.navigate_to(&url_str) {
        tracing::warn!("[itdog] {} 页面导航失败: {}", proto, e);
        return (0, 0);
    }
    if let Err(e) = tab.wait_until_navigated() {
        tracing::warn!("[itdog] {} 等待页面加载失败: {}", proto, e);
        return (0, 0);
    }

    // 等待 itdog 自身脚本初始化
    std::thread::sleep(std::time::Duration::from_secs(3));

    tracing::info!("[itdog] {} 调用 check_form()...", proto);
    let _ = tab.evaluate("if(typeof check_form==='function')check_form();", false);

    progress_cb("waiting");

    let poll_interval = std::time::Duration::from_millis(1000);
    let max_wait = std::time::Duration::from_secs(10);
    let start = std::time::Instant::now();

    loop {
        std::thread::sleep(poll_interval);

        let total_val = tab
            .evaluate(
                "typeof window.check_node_num!=='undefined'?Number(window.check_node_num):0",
                false,
            )
            .ok()
            .and_then(|v| v.value)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u32;
        let finished_val = tab
            .evaluate(
                "typeof window.time_out_num!=='undefined'?Number(window.time_out_num):0",
                false,
            )
            .ok()
            .and_then(|v| v.value)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u32;

        tracing::info!("[itdog] {} 进度: {}/{}", proto, finished_val, total_val);

        // finished_val = window.time_out_num = 超时节点数（同时也是检测结束的计数）
        // 完成条件：total > 0 且超时计数 >= 总节点数
        if total_val > 0 && finished_val >= total_val {
            tracing::info!(
                "[itdog] {} 完成: total={}, timeout={}",
                proto,
                total_val,
                finished_val
            );
            return (total_val, finished_val);
        }

        if start.elapsed() >= max_wait {
            if total_val > 0 {
                // 已有节点数，用现有超时数（未必跑完，但有参考价值）
                tracing::warn!(
                    "[itdog] {} 等待超时（10s），使用现有数据: total={}, timeout={}",
                    proto,
                    total_val,
                    finished_val
                );
                return (total_val, finished_val);
            } else {
                tracing::warn!("[itdog] {} 等待超时（10s），未获取到任何节点数据", proto);
                return (0, 0);
            }
        }
    }
}

// ─── check_network_availability（Chrome headless 方案见上方 itdog_tcping_chrome）──

// (旧 WebView 方案已移除，Chrome headless 方案见上方 itdog_tcping_chrome)

/// 检测公网 IPv4/IPv6 可用性
///
/// 策略：
///   1. 检测系统是否安装了 Google Chrome
///   2. 有 Chrome → 用 Chrome Headless + itdog TCPing 检测
///   3. 无 Chrome → 返回 { no_chrome: true, itdog_url_v4, itdog_url_v6 }，前端引导用户手动查看
///
/// ipv4_host / ipv6_host：不带括号的 IP 字符串（如 1.2.3.4 或 2001:db8::1）
/// port：酒馆端口
#[tauri::command]
pub async fn check_network_availability(
    app: AppHandle,
    ipv4_host: Option<String>,
    ipv6_host: Option<String>,
    port: u16,
) -> Result<serde_json::Value, String> {
    let ipv4_target = ipv4_host.as_deref().map(|h| format!("{}:{}", h, port));
    let ipv6_target = ipv6_host.as_deref().map(|h| {
        let clean = h.trim_matches(|c| c == '[' || c == ']');
        format!("[{}]:{}", clean, port)
    });

    tracing::info!(
        "[itdog] check_network_availability 开始: ipv4={:?}, ipv6={:?}, port={}",
        ipv4_target,
        ipv6_target,
        port
    );

    // ── 检测 Chrome 是否存在 ──────────────────────────────────────────────────
    let chrome_path = find_chrome_executable();
    if chrome_path.is_none() {
        tracing::info!("[itdog] 未检测到 Chrome，返回 no_chrome 模式");
        let itdog_url_v4 = ipv4_target
            .as_deref()
            .map(|t| format!("https://www.itdog.cn/tcping/{}", t));
        let itdog_url_v6 = ipv6_target
            .as_deref()
            .map(|t| format!("https://www.itdog.cn/tcping_ipv6/{}", t));
        return Ok(serde_json::json!({
            "no_chrome": true,
            "itdog_url_v4": itdog_url_v4,
            "itdog_url_v6": itdog_url_v6,
        }));
    }
    let chrome_path = chrome_path.unwrap();
    tracing::info!("[itdog] 找到 Chrome: {}", chrome_path.display());

    // ── IPv4 检测 ──────────────────────────────────────────────────────────────
    if ipv4_target.is_some() {
        let _ = app.emit(
            "itdog-check-progress",
            serde_json::json!({
                "phase": "start", "proto": "IPv4", "ip": ipv4_target.as_deref().unwrap_or(""),
            }),
        );
    }
    let v4_result: (u32, u32) = if let Some(ref t) = ipv4_target {
        let t = t.clone();
        let app2 = app.clone();
        let chrome2 = chrome_path.clone();
        tracing::info!("[itdog] 开始 IPv4 检测: {}", t);
        tokio::task::spawn_blocking(move || {
            itdog_tcping_chrome(&chrome2, &t, false, move |phase| {
                let _ = app2.emit(
                    "itdog-check-progress",
                    serde_json::json!({
                        "phase": phase, "proto": "IPv4",
                    }),
                );
            })
        })
        .await
        .unwrap_or((0, 0))
    } else {
        tracing::info!("[itdog] 跳过 IPv4 检测（无 IPv4 地址）");
        (0, 0)
    };

    // ── IPv6 检测 ──────────────────────────────────────────────────────────────
    if ipv6_target.is_some() {
        let _ = app.emit(
            "itdog-check-progress",
            serde_json::json!({
                "phase": "start", "proto": "IPv6", "ip": ipv6_target.as_deref().unwrap_or(""),
            }),
        );
    }
    let v6_result: (u32, u32) = if let Some(ref t) = ipv6_target {
        let t = t.clone();
        let app2 = app.clone();
        let chrome2 = chrome_path.clone();
        tracing::info!("[itdog] 开始 IPv6 检测: {}", t);
        tokio::task::spawn_blocking(move || {
            itdog_tcping_chrome(&chrome2, &t, true, move |phase| {
                let _ = app2.emit(
                    "itdog-check-progress",
                    serde_json::json!({
                        "phase": phase, "proto": "IPv6",
                    }),
                );
            })
        })
        .await
        .unwrap_or((0, 0))
    } else {
        tracing::info!("[itdog] 跳过 IPv6 检测（无 IPv6 地址）");
        (0, 0)
    };

    let (v4_total, v4_timeout) = v4_result;
    let (v6_total, v6_timeout) = v6_result;

    // total > 0 时：rate = timeout / total（0 超时 → 0.0 = 100% 可用）
    // total = 0 时：表示检测完全无数据（Chrome 未加载出页面等），设 rate = 1.0 表示不可用
    let v4_rate = if v4_total > 0 {
        v4_timeout as f64 / v4_total as f64
    } else {
        1.0
    };
    let v6_rate = if v6_total > 0 {
        v6_timeout as f64 / v6_total as f64
    } else {
        1.0
    };

    // preferred：超时率更低的协议；若都没数据则不推荐
    let preferred: Option<String> = if ipv4_host.is_some() && ipv6_host.is_some() {
        if v4_total == 0 && v6_total == 0 {
            None
        } else if v4_total == 0 {
            Some("ipv6".to_string())
        } else if v6_total == 0 {
            Some("ipv4".to_string())
        } else if v4_rate <= v6_rate {
            Some("ipv4".to_string())
        } else {
            Some("ipv6".to_string())
        }
    } else if ipv4_host.is_some() {
        Some("ipv4".to_string())
    } else if ipv6_host.is_some() {
        Some("ipv6".to_string())
    } else {
        None
    };

    tracing::info!("[itdog] 检测汇总: IPv4 total={} timeout={} rate={:.1}%, IPv6 total={} timeout={} rate={:.1}%, preferred={:?}",
        v4_total, v4_timeout, v4_rate * 100.0,
        v6_total, v6_timeout, v6_rate * 100.0,
        preferred
    );

    // 发送 done 事件
    if ipv4_target.is_some() {
        let _ = app.emit(
            "itdog-check-progress",
            serde_json::json!({
                "phase": "done", "proto": "IPv4", "total": v4_total, "timeout": v4_timeout,
            }),
        );
    }
    if ipv6_target.is_some() {
        let _ = app.emit(
            "itdog-check-progress",
            serde_json::json!({
                "phase": "done", "proto": "IPv6", "total": v6_total, "timeout": v6_timeout,
            }),
        );
    }

    Ok(serde_json::json!({
        "no_chrome": false,
        "preferred": preferred,
        "ipv4_total": v4_total,
        "ipv4_timeout": v4_timeout,
        "ipv4_timeout_rate": v4_rate,
        "ipv6_total": v6_total,
        "ipv6_timeout": v6_timeout,
        "ipv6_timeout_rate": v6_rate,
    }))
}

/// 修复运行时缺失的 npm 包。
/// 前端收到 `tavern-missing-dep` 事件后调用此命令，安装成功后 emit `tavern-dep-repaired`。
#[tauri::command]
pub async fn repair_missing_deps(
    app: AppHandle,
    packages: Vec<String>,
    st_dir: String,
) -> Result<(), String> {
    use crate::config::get_current_lang;
    use crate::types::Lang;
    use tauri::Emitter;

    let lang = get_current_lang(&app);
    let dir = std::path::PathBuf::from(&st_dir);

    if packages.is_empty() {
        return Ok(());
    }

    let pkg_list = packages.join(", ");
    let msg = match lang {
        Lang::ZhCn => format!("INFO: 正在安装运行时缺失包，请稍候：{}", pkg_list),
        Lang::EnUs => format!("INFO: Installing runtime missing packages: {}", pkg_list),
    };
    let _ = app.emit("process-log", msg);

    crate::node::run_npm_install_packages(&app, &dir, &packages).await?;

    let ok_msg = match lang {
        Lang::ZhCn => "INFO: 缺失包修复完成，即将自动重启酒馆...".to_string(),
        Lang::EnUs => "INFO: Missing packages repaired. Auto-restarting SillyTavern...".to_string(),
    };
    let _ = app.emit("process-log", ok_msg);
    let _ = app.emit("tavern-dep-repaired", ());

    Ok(())
}
