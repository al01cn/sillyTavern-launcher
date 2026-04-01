use futures_util::future::select_ok;
use reqwest::Client;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Registry};

use crate::types::AppConfig;

// ─────────────────────────────────────────────
// GitHub 加速下载代理机制
// ─────────────────────────────────────────────

#[derive(Clone)]
pub struct GithubProxy {
    mirrors: Vec<String>,
}

impl GithubProxy {
    pub async fn new(app: &tauri::AppHandle) -> Self {
        let config = crate::config::read_app_config_from_disk(app);
        let mut mirrors = vec!["".to_string()]; // 原始链接

        // GitHub 加速开关开启时，或设置了网络代理（代理和加速不冲突）时启用加速
        let use_accelerate = config.github_proxy.enable
            || config.network_proxy.mode != crate::types::ProxyMode::None;

        if use_accelerate {
            // 用户设置的默认代理节点
            if !config.github_proxy.url.is_empty() {
                mirrors.push(config.github_proxy.url.clone());
            }

            // 获取 API 上的代理节点，补充作为备选（如果上面没有）
            // 我们不进行延迟测试，只提取前三个返回的节点以供并发使用
            let client = Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap_or_default();

            if let Ok(response) = client.get("https://api.akams.cn/github").send().await {
                if let Ok(proxy_resp) = response.json::<crate::types::ProxyResponse>().await {
                    if proxy_resp.code == 200 {
                        let mut proxies = proxy_resp.data;
                        proxies.sort_by(|a, b| a.latency.cmp(&b.latency));
                        for p in proxies.into_iter().take(3) {
                            if !mirrors.contains(&p.url) {
                                mirrors.push(p.url);
                            }
                        }
                    }
                }
            }
        }

        Self { mirrors }
    }

    pub fn build_urls(&self, url: &str) -> Vec<String> {
        if url.contains("api.github.com") {
            return vec![url.to_string()];
        }

        self.mirrors
            .iter()
            .map(|m| format!("{}{}", m, url))
            .collect()
    }

    pub async fn get_fastest_stream(
        &self,
        client: Client,
        url: &str,
    ) -> Result<(String, reqwest::Response), String> {
        if !url.contains("github.com") || url.contains("api.github.com") {
            // 普通 URL 或者 GitHub API，不走并发镜像请求
            let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                return Ok((url.to_string(), resp));
            } else {
                return Err(format!("HTTP Error: {}", resp.status()));
            }
        }

        let urls = self.build_urls(url);
        let futures = urls.into_iter().map(|u| {
            let client = client.clone();
            Box::pin(async move {
                let resp = client.get(&u).send().await.map_err(|e| e.to_string())?;
                if resp.status().is_success() {
                    Ok((u, resp))
                } else {
                    Err(format!("HTTP Error: {}", resp.status()))
                }
            })
        });

        match select_ok(futures).await {
            Ok(((fastest_url, response), _remaining)) => Ok((fastest_url, response)),
            Err(e) => Err(format!("All download attempts failed: {}", e)),
        }
    }
}

// ─────────────────────────────────────────────
// 日志初始化
// ─────────────────────────────────────────────

pub fn init_logger(data_dir: &Path) {
    let logs_dir = data_dir.join("logs");
    if !logs_dir.exists() {
        let _ = fs::create_dir_all(&logs_dir);
    }

    // 清除所有的旧日志（包括以前日期的.log或现有的launcher.log）
    if let Ok(entries) = fs::read_dir(&logs_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.ends_with(".log") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    // 创建单文件覆盖日志 launcher.log
    let log_file_path = logs_dir.join("launcher.log");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_file_path)
        .unwrap_or_else(|e| panic!("Failed to open log file {:?}: {}", log_file_path, e));

    let (non_blocking, _guard) = tracing_appender::non_blocking(file);

    // 将 _guard 泄漏到全局，防止 drop 导致最后日志丢失
    Box::leak(Box::new(_guard));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false);

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false);

    // 环境级过滤：默认 info，本应用 debug，从而过滤无关底层网络杂乱日志
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sillytavern_launcher_lib=debug"));

    let subscriber = Registry::default()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer);

    let _ = tracing::subscriber::set_global_default(subscriber);

    // 捕获 Panic 并记录到日志
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown".to_string()
        };

        let location = if let Some(location) = panic_info.location() {
            format!("{}:{}", location.file(), location.line())
        } else {
            "unknown location".to_string()
        };

        tracing::error!("程序发生致命崩溃 (Panic) @ {}: {}", location, payload);
    }));
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

    // 首次启动时生成 config.yaml
    let global_cfg = st_data_dir.join("config.yaml");
    if !global_cfg.exists() {
        let _ = fs::write(&global_cfg, crate::sillytavern::DEFAULT_CONFIG_TEMPLATE);
    }

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
