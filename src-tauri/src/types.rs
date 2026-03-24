use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// ─────────────────────────────────────────────
// 进程 / 安装状态
// ─────────────────────────────────────────────

pub struct ProcessState {
    pub kill_tx: Arc<Mutex<Option<tokio::sync::mpsc::Sender<()>>>>,
}

pub struct InstallState {
    pub cancel_flag: Arc<std::sync::atomic::AtomicBool>,
}

// ─────────────────────────────────────────────
// 下载进度
// ─────────────────────────────────────────────

#[derive(Clone, Serialize)]
pub struct DownloadProgress {
    pub status: String,
    pub progress: f64,
    pub log: String,
}

// ─────────────────────────────────────────────
// 应用配置
// ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct GithubProxyConfig {
    pub enable: bool,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct SillyTavernConfig {
    pub version: String,
    pub use_global_config: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub lang: String,
    pub theme: String,
    pub remember_window_position: bool,
    pub window_position: Option<WindowPosition>,
    pub github_proxy: GithubProxyConfig,
    pub sillytavern: SillyTavernConfig,
    pub npm_registry: String,
}

impl Default for WindowPosition {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Default for GithubProxyConfig {
    fn default() -> Self {
        Self {
            enable: false,
            url: "https://ghfast.top/".to_string(),
        }
    }
}

impl Default for SillyTavernConfig {
    fn default() -> Self {
        Self {
            version: "".to_string(),
            use_global_config: true,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            lang: "auto".to_string(),
            theme: "auto".to_string(),
            remember_window_position: false,
            window_position: None,
            github_proxy: GithubProxyConfig::default(),
            sillytavern: SillyTavernConfig::default(),
            npm_registry: "https://registry.npmjs.org/".to_string(),
        }
    }
}

// ─────────────────────────────────────────────
// 语言
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    ZhCn,
    EnUs,
}

// ─────────────────────────────────────────────
// Node / Npm 信息
// ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeInfo {
    pub version: Option<String>,
    pub path: Option<String>,
    pub source: String, // "system", "local", or "none"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NpmInfo {
    pub version: Option<String>,
    pub path: Option<String>,
    pub source: String, // "system", "local", or "none"
}

// ─────────────────────────────────────────────
// GitHub 相关
// ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyItem {
    pub url: String,
    pub server: String,
    pub ip: String,
    pub location: String,
    pub latency: u32,
    pub speed: f64,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyResponse {
    pub code: u32,
    pub msg: String,
    pub data: Vec<ProxyItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub created_at: String,
    pub published_at: String,
    pub zipball_url: String,
    pub assets: Vec<ReleaseAsset>,
}

// ─────────────────────────────────────────────
// SillyTavern 高级配置结构体
// ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct TavernConfigPayload {
    pub port: i64,
    pub listen: bool,
    pub listen_address: TavernDualStackAddress,
    pub protocol: TavernDualStackProtocol,
    pub basic_auth_mode: bool,
    pub enable_user_accounts: bool,
    pub enable_discreet_login: bool,
    pub per_user_basic_auth: bool,
    pub basic_auth_user: TavernBasicAuthUser,
    pub whitelist_mode: bool,
    pub whitelist: Vec<String>,
    pub cors: TavernCorsConfig,
    pub request_proxy: TavernRequestProxyConfig,
    pub backups: TavernBackupsConfig,
    pub thumbnails: TavernThumbnailsConfig,
    pub browser_launch_enabled: bool,
    pub browser_type: String,
    
    // SSL/TLS 配置
    pub ssl: TavernSslConfig,
    
    // DNS 和网络高级选项
    #[serde(rename = "dnsPreferIPv6")]
    pub dns_prefer_ipv6: bool,
    pub heartbeat_interval: i64,
    pub host_whitelist: TavernHostWhitelistConfig,
    pub whitelist_import_domains: Vec<String>,
    
    // 会话和安全
    pub session_timeout: i64,
    pub disable_csrf_protection: bool,
    pub security_override: bool,
    pub allow_keys_exposure: bool,
    pub skip_content_check: bool,
    
    // 日志
    pub logging: TavernLoggingConfig,
    
    // 性能
    pub performance: TavernPerformanceConfig,
    
    // 缓存清除
    pub cache_buster: TavernCacheBusterConfig,
    
    // SSO
    pub sso: TavernSsoConfig,
    
    // 扩展
    pub extensions: TavernExtensionsConfig,
    
    // 服务器插件
    pub enable_server_plugins: bool,
    pub enable_server_plugins_auto_update: bool,
    
    // 其他
    pub enable_cors_proxy: bool,
    pub prompt_placeholder: String,
    pub enable_downloadable_tokenizers: bool,
}

impl Default for TavernConfigPayload {
    fn default() -> Self {
        Self {
            port: 8000,
            listen: false,
            listen_address: TavernDualStackAddress::default(),
            protocol: TavernDualStackProtocol::default(),
            basic_auth_mode: false,
            enable_user_accounts: false,
            enable_discreet_login: false,
            per_user_basic_auth: false,
            basic_auth_user: TavernBasicAuthUser::default(),
            whitelist_mode: true,
            whitelist: vec!["::1".to_string(), "127.0.0.1".to_string()],
            cors: TavernCorsConfig::default(),
            request_proxy: TavernRequestProxyConfig::default(),
            backups: TavernBackupsConfig::default(),
            thumbnails: TavernThumbnailsConfig::default(),
            browser_launch_enabled: true,
            browser_type: "default".to_string(),
            ssl: TavernSslConfig::default(),
            dns_prefer_ipv6: false,
            heartbeat_interval: 0,
            host_whitelist: TavernHostWhitelistConfig::default(),
            whitelist_import_domains: vec![],
            session_timeout: -1,
            disable_csrf_protection: false,
            security_override: false,
            allow_keys_exposure: false,
            skip_content_check: false,
            logging: TavernLoggingConfig::default(),
            performance: TavernPerformanceConfig::default(),
            cache_buster: TavernCacheBusterConfig::default(),
            sso: TavernSsoConfig::default(),
            extensions: TavernExtensionsConfig::default(),
            enable_server_plugins: false,
            enable_server_plugins_auto_update: true,
            enable_cors_proxy: false,
            prompt_placeholder: "[Start a new chat]".to_string(),
            enable_downloadable_tokenizers: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernDualStackAddress {
    pub ipv4: String,
    pub ipv6: String,
}

impl Default for TavernDualStackAddress {
    fn default() -> Self {
        Self {
            ipv4: "0.0.0.0".to_string(),
            ipv6: "[::]".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernDualStackProtocol {
    pub ipv4: bool,
    pub ipv6: bool,
}

impl Default for TavernDualStackProtocol {
    fn default() -> Self {
        Self {
            ipv4: true,
            ipv6: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernCorsConfig {
    pub enabled: bool,
    pub origin: Vec<String>,
    pub methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub credentials: bool,
    pub max_age: Option<i64>,
}

impl Default for TavernCorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            origin: vec!["null".to_string()],
            methods: vec!["OPTIONS".to_string()],
            allowed_headers: vec![],
            exposed_headers: vec![],
            credentials: false,
            max_age: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernRequestProxyConfig {
    pub enabled: bool,
    pub url: String,
    pub bypass: Vec<String>,
}

impl Default for TavernRequestProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: "".to_string(),
            bypass: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernBasicAuthUser {
    pub username: String,
    pub password: String,
}

impl Default for TavernBasicAuthUser {
    fn default() -> Self {
        Self {
            username: "user".to_string(),
            password: "password".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernBackupsConfig {
    pub common: TavernBackupsCommonConfig,
    pub chat: TavernBackupsChatConfig,
}

impl Default for TavernBackupsConfig {
    fn default() -> Self {
        Self {
            common: TavernBackupsCommonConfig::default(),
            chat: TavernBackupsChatConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernBackupsCommonConfig {
    pub number_of_backups: i64,
}

impl Default for TavernBackupsCommonConfig {
    fn default() -> Self {
        Self {
            number_of_backups: 50,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernBackupsChatConfig {
    pub enabled: bool,
    pub check_integrity: bool,
    pub max_total_backups: i64,
    pub throttle_interval: i64,
}

impl Default for TavernBackupsChatConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_integrity: true,
            max_total_backups: -1,
            throttle_interval: 10000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernThumbnailsConfig {
    pub enabled: bool,
    pub format: String,
    pub quality: i64,
    pub dimensions: TavernThumbnailsDimensionsConfig,
}

impl Default for TavernThumbnailsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: "jpg".to_string(),
            quality: 95,
            dimensions: TavernThumbnailsDimensionsConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernThumbnailsDimensionsConfig {
    pub bg: Vec<i64>,
    pub avatar: Vec<i64>,
    pub persona: Vec<i64>,
}

impl Default for TavernThumbnailsDimensionsConfig {
    fn default() -> Self {
        Self {
            bg: vec![160, 90],
            avatar: vec![96, 144],
            persona: vec![96, 144],
        }
    }
}

// ─────────────────────────────────────────────
// 新增配置结构体（高级配置）
// ─────────────────────────────────────────────

// SSL/TLS 配置
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TavernSslConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
    pub key_passphrase: String,
}

// 主机白名单配置
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TavernHostWhitelistConfig {
    pub enabled: bool,
    pub scan: bool,
    pub hosts: Vec<String>,
}

// 日志配置
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TavernLoggingConfig {
    pub enable_access_log: bool,
    pub min_log_level: i64,
}

// 性能配置
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TavernPerformanceConfig {
    pub lazy_load_characters: bool,
    pub memory_cache_capacity: String,
    pub use_disk_cache: bool,
}

// 缓存清除配置
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TavernCacheBusterConfig {
    pub enabled: bool,
    pub user_agent_pattern: String,
}

// SSO 配置
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TavernSsoConfig {
    pub authelia_auth: bool,
    pub authentik_auth: bool,
}

// 扩展配置
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TavernExtensionsConfig {
    pub enabled: bool,
    pub auto_update: bool,
}

// ─────────────────────────────────────────────
// 版本信息
// ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstalledVersionInfo {
    pub version: String,
    pub has_node_modules: bool,
}

// ─────────────────────────────────────────────
// 角色卡 / 世界书
// ─────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterCardFile {
    pub file_name: String,
    pub size: u64,
    pub modified_ms: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldInfoFile {
    pub file_name: String,
    pub size: u64,
    pub modified_ms: Option<i64>,
}

// ─────────────────────────────────────────────
// 扩展
// ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ExtensionManifest {
    #[serde(rename = "display_name", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(rename = "homePage", default)]
    pub home_page: Option<String>,
    #[serde(default)]
    pub auto_update: Option<bool>,
    #[serde(default)]
    pub minimum_client_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtensionInfo {
    pub id: String,
    pub manifest: ExtensionManifest,
    pub dir_path: String,
    pub enabled: bool,
    pub is_system: bool,
    pub scope: String, // "global" or "user"
}
