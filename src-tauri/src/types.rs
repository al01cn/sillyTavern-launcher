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
#[serde(rename_all = "camelCase")]
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernDualStackAddress {
    pub ipv4: String,
    pub ipv6: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernDualStackProtocol {
    pub ipv4: bool,
    pub ipv6: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernRequestProxyConfig {
    pub enabled: bool,
    pub url: String,
    pub bypass: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernBasicAuthUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernBackupsConfig {
    pub common: TavernBackupsCommonConfig,
    pub chat: TavernBackupsChatConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernBackupsCommonConfig {
    pub number_of_backups: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernBackupsChatConfig {
    pub enabled: bool,
    pub check_integrity: bool,
    pub max_total_backups: i64,
    pub throttle_interval: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernThumbnailsConfig {
    pub enabled: bool,
    pub format: String,
    pub quality: i64,
    pub dimensions: TavernThumbnailsDimensionsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TavernThumbnailsDimensionsConfig {
    pub bg: Vec<i64>,
    pub avatar: Vec<i64>,
    pub persona: Vec<i64>,
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
