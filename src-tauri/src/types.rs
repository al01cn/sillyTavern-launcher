use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// ─────────────────────────────────────────────
// 进程 / 安装状态
// ─────────────────────────────────────────────

pub struct ProcessState {
    pub kill_tx: Arc<Mutex<Option<tokio::sync::mpsc::Sender<()>>>>,
    pub child_pid: Arc<Mutex<Option<u32>>>,
}

pub struct InstallState {
    pub cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    /// 当前正在运行的 git 子进程 PID（install_extension_git / repair_extension_git 的 git clone/fetch）
    /// 程序退出时可以 kill 掉它，避免留下孤儿进程
    pub git_child_pid: Arc<Mutex<Option<u32>>>,
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
    pub version: LocalTavernItem,
}

/// 代理模式：none=不代理, system=系统代理, custom=自定义
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ProxyMode {
    #[default]
    None,
    System,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct NetworkProxyConfig {
    pub mode: ProxyMode,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct LocalTavernItem {
    pub path: String,
    pub version: String,
    pub has_node_modules: bool,
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
    pub local_sillytavern_list: Vec<LocalTavernItem>,
    pub scan_cpu_cores: Option<usize>,
    pub region_auto_configured: bool,
    pub initial_setup_completed: bool,
    pub enable_animations: bool,
    pub has_scanned_once: bool,
    pub auto_repair_git: bool,
    pub setup_checkpoint: Option<String>,
    /// 优先使用系统 Node（true = 系统优先，false = 内置优先）
    pub use_system_node: bool,
    /// 优先使用系统 Git（true = 系统优先，false = 内置优先）
    pub use_system_git: bool,
    /// 酒馆启动模式：normal / desktop / debug
    pub launch_mode: String,
    /// 启动器网络代理配置
    pub network_proxy: NetworkProxyConfig,
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

impl Default for NetworkProxyConfig {
    fn default() -> Self {
        Self {
            mode: ProxyMode::None,
            host: "127.0.0.1".to_string(),
            port: 7890,
        }
    }
}

impl Default for SillyTavernConfig {
    fn default() -> Self {
        Self {
            version: LocalTavernItem::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            lang: "auto".to_string(),
            theme: "auto".to_string(),
            remember_window_position: true,
            window_position: None,
            github_proxy: GithubProxyConfig::default(),
            sillytavern: SillyTavernConfig::default(),
            npm_registry: "https://registry.npmjs.org/".to_string(),
            local_sillytavern_list: vec![],
            scan_cpu_cores: None,
            region_auto_configured: false,
            initial_setup_completed: false,
            enable_animations: true,
            has_scanned_once: false,
            auto_repair_git: true,
            setup_checkpoint: None,
            use_system_node: true,
            use_system_git: true,
            launch_mode: "normal".to_string(),
            network_proxy: NetworkProxyConfig::default(),
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
// Git 信息
// ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitInfo {
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
    pub is_link: bool,
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
// 对话历史
// ─────────────────────────────────────────────

/// 单条对话消息（解析自 .jsonl 每行）
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub name: String,
    pub mes: String,
    pub is_user: bool,
    pub is_system: bool,
    pub send_date: Option<String>,
}

/// 单个 .jsonl 文件信息
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatFile {
    pub file_name: String,
    /// 所属角色文件夹名（如 default_Seraphina）
    pub char_folder: String,
    pub size: u64,
    pub modified_ms: Option<i64>,
}

/// 按角色分组的对话历史
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatGroup {
    /// 文件夹名（如 default_Seraphina）
    pub char_folder: String,
    /// 显示用角色名（如 Seraphina）
    pub char_name: String,
    pub files: Vec<ChatFile>,
}

/// 批量删除时传入的 {charFolder, fileName} 对
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatDeleteItem {
    pub char_folder: String,
    pub file_name: String,
}

// ─────────────────────────────────────────────
// 资源迁移
// ─────────────────────────────────────────────

/// 可迁移的来源实例
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMigrationSource {
    /// 酒馆根目录路径
    pub tavern_path: String,
    /// 酒馆 data 目录路径（{tavernPath}/data）
    pub data_path: String,
    /// 版本号
    pub version: String,
    /// 显示名称
    pub display: String,
}

/// 冲突文件条目
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFile {
    /// 相对于 data 目录的路径，如 "default-user/characters/foo.png"
    pub rel_path: String,
    /// 来源实例路径（完整绝对路径）
    pub source_full_path: String,
    /// 目标完整路径
    pub dest_full_path: String,
    /// 来源实例显示名
    pub source_display: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 友好的分类名称（角色卡 / 世界书 / 历史聊天记录 / ...）
    pub category: String,
}

/// 迁移进度事件 payload
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrationProgressEvent {
    /// 已完成文件数
    pub done: usize,
    /// 总文件数
    pub total: usize,
    /// 当前正在处理的文件相对路径
    pub current: String,
    /// 是否全部完成
    pub finished: bool,
    /// 错误信息（若有）
    pub error: Option<String>,
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
    pub has_git: bool,
}

// Default Settings JSON
pub const DEFAULT_SETTINGS_JSON: &str = r####"{
    "firstRun": false,
    "accountStorage": {
        "__migrated": "1",
        "LNavOpened": "true",
        "NavOpened": "true",
        "NavLockOn": "true",
        "LNavLockOn": "true"
    },
    "currentVersion": "1.16.0",
    "username": "User",
    "max_context": 8192,
    "main_api": "openai",
    "world_info_settings": {
        "world_info": {
            "globalSelect": []
        },
        "world_info_depth": 2,
        "world_info_min_activations": 0,
        "world_info_min_activations_depth_max": 0,
        "world_info_budget": 100,
        "world_info_include_names": false,
        "world_info_recursive": true,
        "world_info_overflow_alert": false,
        "world_info_case_sensitive": false,
        "world_info_match_whole_words": false,
        "world_info_character_strategy": 1,
        "world_info_budget_cap": 0,
        "world_info_use_group_scoring": false,
        "world_info_max_recursion_steps": 0
    },
    "textgenerationwebui_settings": {
        "temp": 0.7,
        "temperature_last": true,
        "top_p": 0.5,
        "top_k": 40,
        "top_a": 0,
        "tfs": 1,
        "epsilon_cutoff": 0,
        "eta_cutoff": 0,
        "typical_p": 1,
        "min_p": 0,
        "rep_pen": 1.2,
        "rep_pen_range": 0,
        "rep_pen_decay": 0,
        "rep_pen_slope": 1,
        "no_repeat_ngram_size": 0,
        "penalty_alpha": 0,
        "num_beams": 1,
        "length_penalty": 1,
        "min_length": 0,
        "encoder_rep_pen": 1,
        "freq_pen": 0,
        "presence_pen": 0,
        "skew": 0,
        "do_sample": true,
        "early_stopping": false,
        "dynatemp": false,
        "min_temp": 0,
        "max_temp": 2,
        "dynatemp_exponent": 1,
        "smoothing_factor": 0,
        "smoothing_curve": 1,
        "dry_allowed_length": 2,
        "dry_multiplier": 0,
        "dry_base": 1.75,
        "dry_sequence_breakers": "[\"\\n\", \":\", \"\\\"\", \"*\"]",
        "dry_penalty_last_n": 0,
        "max_tokens_second": 0,
        "seed": -1,
        "preset": "Default",
        "add_bos_token": true,
        "stopping_strings": [],
        "ban_eos_token": false,
        "skip_special_tokens": true,
        "include_reasoning": true,
        "streaming": false,
        "mirostat_mode": 0,
        "mirostat_tau": 5,
        "mirostat_eta": 0.1,
        "guidance_scale": 1,
        "negative_prompt": "",
        "grammar_string": "",
        "json_schema": null,
        "json_schema_allow_empty": false,
        "banned_tokens": "",
        "global_banned_tokens": "",
        "send_banned_tokens": true,
        "sampler_priority": [
            "repetition_penalty",
            "presence_penalty",
            "frequency_penalty",
            "dry",
            "temperature",
            "dynamic_temperature",
            "quadratic_sampling",
            "top_n_sigma",
            "top_k",
            "top_p",
            "typical_p",
            "epsilon_cutoff",
            "eta_cutoff",
            "tfs",
            "top_a",
            "min_p",
            "mirostat",
            "xtc",
            "encoder_repetition_penalty",
            "no_repeat_ngram"
        ],
        "samplers": [
            "penalties",
            "dry",
            "top_n_sigma",
            "top_k",
            "typ_p",
            "top_p",
            "min_p",
            "xtc",
            "temperature",
            "adaptive_p"
        ],
        "samplers_priorities": [
            "dry",
            "penalties",
            "no_repeat_ngram",
            "temperature",
            "top_nsigma",
            "top_p_top_k",
            "top_a",
            "min_p",
            "tfs",
            "eta_cutoff",
            "epsilon_cutoff",
            "typical_p",
            "quadratic",
            "xtc"
        ],
        "ignore_eos_token": false,
        "spaces_between_special_tokens": true,
        "speculative_ngram": false,
        "type": "ooba",
        "mancer_model": "mytholite",
        "togetherai_model": "Gryphe/MythoMax-L2-13b",
        "infermaticai_model": "",
        "ollama_model": "",
        "openrouter_model": "openrouter/auto",
        "openrouter_providers": [],
        "openrouter_quantizations": [],
        "vllm_model": "",
        "aphrodite_model": "",
        "dreamgen_model": "lucid-v1-extra-large/text",
        "tabby_model": "",
        "llamacpp_model": "",
        "sampler_order": [
            6,
            0,
            1,
            3,
            4,
            2,
            5
        ],
        "logit_bias": [],
        "n": 1,
        "server_urls": {},
        "custom_model": "",
        "bypass_status_check": false,
        "openrouter_allow_fallbacks": true,
        "xtc_threshold": 0.1,
        "xtc_probability": 0,
        "nsigma": 0,
        "min_keep": 0,
        "featherless_model": "",
        "generic_model": "",
        "extensions": {},
        "adaptive_target": -0.01,
        "adaptive_decay": 0.9
    },
    "swipes": true,
    "horde_settings": {
        "models": [],
        "auto_adjust_response_length": true,
        "auto_adjust_context_length": false,
        "trusted_workers_only": false
    },
    "power_user": {
        "charListGrid": false,
        "tokenizer": 99,
        "token_padding": 64,
        "collapse_newlines": false,
        "pin_examples": false,
        "strip_examples": false,
        "trim_sentences": false,
        "always_force_name2": false,
        "user_prompt_bias": "",
        "show_user_prompt_bias": true,
        "auto_continue": {
            "enabled": false,
            "allow_chat_completions": false,
            "target_length": 400
        },
        "markdown_escape_strings": "",
        "chat_truncation": 100,
        "streaming_fps": 30,
        "smooth_streaming": false,
        "smooth_streaming_no_think": false,
        "smooth_streaming_speed": 50,
        "stream_fade_in": false,
        "fast_ui_mode": true,
        "avatar_style": 0,
        "chat_display": 0,
        "toastr_position": "toast-top-center",
        "chat_width": 50,
        "never_resize_avatars": false,
        "show_card_avatar_urls": false,
        "play_message_sound": false,
        "play_sound_unfocused": true,
        "auto_save_msg_edits": true,
        "confirm_message_delete": true,
        "sort_field": "name",
        "sort_order": "asc",
        "sort_rule": null,
        "font_scale": 1,
        "blur_strength": 10,
        "shadow_width": 2,
        "main_text_color": "rgba(220, 220, 210, 1)",
        "italics_text_color": "rgba(145, 145, 145, 1)",
        "underline_text_color": "rgba(188, 231, 207, 1)",
        "quote_text_color": "rgba(225, 138, 36, 1)",
        "blur_tint_color": "rgba(23, 23, 23, 1)",
        "chat_tint_color": "rgba(23, 23, 23, 1)",
        "user_mes_blur_tint_color": "rgba(0, 0, 0, 0.3)",
        "bot_mes_blur_tint_color": "rgba(60, 60, 60, 0.3)",
        "shadow_color": "rgba(0, 0, 0, 0.5)",
        "border_color": "rgba(0, 0, 0, 0.5)",
        "custom_css": "",
        "waifuMode": false,
        "movingUI": false,
        "movingUIState": {},
        "movingUIPreset": "",
        "noShadows": false,
        "theme": "Default (Dark) 1.7.1",
        "gestures": true,
        "auto_swipe": false,
        "auto_swipe_minimum_length": 0,
        "auto_swipe_blacklist": [],
        "auto_swipe_blacklist_threshold": 2,
        "auto_scroll_chat_to_bottom": true,
        "auto_fix_generated_markdown": false,
        "send_on_enter": 1,
        "console_log_prompts": false,
        "request_token_probabilities": false,
        "show_group_chat_queue": false,
        "allow_name1_display": true,
        "allow_name2_display": true,
        "hotswap_enabled": true,
        "timer_enabled": true,
        "timestamps_enabled": true,
        "timestamp_model_icon": false,
        "mesIDDisplay_enabled": false,
        "hideChatAvatars_enabled": false,
        "max_context_unlocked": false,
        "message_token_count_enabled": false,
        "expand_message_actions": false,
        "enableZenSliders": false,
        "enableLabMode": false,
        "prefer_character_prompt": true,
        "prefer_character_jailbreak": true,
        "quick_continue": true,
        "quick_impersonate": false,
        "continue_on_send": false,
        "trim_spaces": true,
        "relaxed_api_urls": false,
        "world_import_dialog": true,
        "enable_auto_select_input": false,
        "enable_md_hotkeys": false,
        "tag_import_setting": 1,
        "tag_sort_mode": "manual",
        "disable_group_trimming": false,
        "single_line": false,
        "instruct": {
            "enabled": false,
            "preset": "Alpaca",
            "input_sequence": "### Instruction:",
            "input_suffix": "",
            "output_sequence": "### Response:",
            "output_suffix": "",
            "system_sequence": "",
            "system_suffix": "",
            "last_system_sequence": "",
            "first_input_sequence": "",
            "first_output_sequence": "",
            "last_input_sequence": "",
            "last_output_sequence": "",
            "story_string_prefix": "",
            "story_string_suffix": "",
            "stop_sequence": "",
            "wrap": true,
            "macro": true,
            "names_behavior": "force",
            "activation_regex": "",
            "bind_to_context": false,
            "user_alignment_message": "",
            "system_same_as_user": false,
            "sequences_as_stop_strings": true,
            "skip_examples": false
        },
        "context": {
            "preset": "Default",
            "story_string": "{{#if system}}{{system}}\n{{/if}}{{#if description}}{{description}}\n{{/if}}{{#if personality}}{{char}}'s personality: {{personality}}\n{{/if}}{{#if scenario}}Scenario: {{scenario}}\n{{/if}}{{#if persona}}{{persona}}\n{{/if}}",
            "chat_start": "***",
            "example_separator": "***",
            "use_stop_strings": true,
            "names_as_stop_strings": true,
            "story_string_position": 0,
            "story_string_role": 0,
            "story_string_depth": 1
        },
        "instruct_derived": false,
        "context_derived": false,
        "context_size_derived": false,
        "model_templates_mappings": {},
        "chat_template_hash": "",
        "sysprompt": {
            "enabled": true,
            "name": "Neutral - Chat",
            "content": "Write {{char}}'s next reply in a fictional chat between {{char}} and {{user}}.",
            "post_history": ""
        },
        "reasoning": {
            "name": "DeepSeek",
            "auto_parse": false,
            "add_to_prompts": false,
            "auto_expand": false,
            "show_hidden": false,
            "prefix": "<think>\n",
            "suffix": "\n</think>",
            "separator": "\n\n",
            "max_additions": 1
        },
        "personas": {
            "user-default.png": "[Unnamed Persona]",
            "undefined": "User"
        },
        "default_persona": null,
        "persona_descriptions": {
            "user-default.png": {
                "description": "",
                "position": 0,
                "depth": 2,
                "role": 0,
                "lorebook": "",
                "title": ""
            },
            "undefined": {
                "description": "",
                "position": 0,
                "depth": 2,
                "role": 0,
                "lorebook": "",
                "title": ""
            }
        },
        "persona_description": "",
        "persona_description_position": 0,
        "persona_description_role": 0,
        "persona_description_depth": 2,
        "persona_description_lorebook": "",
        "persona_show_notifications": true,
        "persona_sort_order": "asc",
        "custom_stopping_strings": "",
        "custom_stopping_strings_macro": true,
        "fuzzy_search": false,
        "encode_tags": false,
        "experimental_macro_engine": false,
        "servers": [],
        "bogus_folders": false,
        "zoomed_avatar_magnification": false,
        "show_tag_filters": false,
        "aux_field": "character_version",
        "stscript": {
            "matching": "fuzzy",
            "autocomplete": {
                "state": 2,
                "autoHide": false,
                "style": "theme",
                "font": {
                    "scale": 1
                },
                "width": {
                    "left": 1,
                    "right": 1
                },
                "select": 3,
                "showInAllMacroFields": false
            },
            "parser": {
                "flags": {}
            }
        },
        "restore_user_input": true,
        "reduced_motion": true,
        "compact_input_area": false,
        "show_swipe_num_all_messages": false,
        "auto_connect": true,
        "auto_load_chat": true,
        "forbid_external_media": false,
        "external_media_allowed_overrides": [],
        "external_media_forbidden_overrides": [],
        "pin_styles": true,
        "click_to_edit": false,
        "media_display": "list",
        "image_overswipe": "generate"
    },
    "extension_settings": {
        "apiUrl": "http://localhost:5100",
        "apiKey": "",
        "autoConnect": false,
        "notifyUpdates": false,
        "disabledExtensions": [],
        "expressionOverrides": [],
        "memory": {
            "memoryFrozen": false,
            "SkipWIAN": false,
            "source": "extras",
            "prompt": "Ignore previous instructions. Summarize the most important facts and events in the story so far. If a summary already exists in your memory, use that as a base and expand with new facts. Limit the summary to {{words}} words or less. Your response should include nothing but the summary.",
            "template": "[Summary: {{summary}}]",
            "position": 0,
            "role": 0,
            "scan": false,
            "depth": 2,
            "promptWords": 200,
            "promptMinWords": 25,
            "promptMaxWords": 1000,
            "promptWordsStep": 25,
            "promptInterval": 10,
            "promptMinInterval": 0,
            "promptMaxInterval": 250,
            "promptIntervalStep": 1,
            "promptForceWords": 0,
            "promptForceWordsStep": 100,
            "promptMinForceWords": 0,
            "promptMaxForceWords": 10000,
            "overrideResponseLength": 0,
            "overrideResponseLengthMin": 0,
            "overrideResponseLengthMax": 4096,
            "overrideResponseLengthStep": 16,
            "maxMessagesPerRequest": 0,
            "maxMessagesPerRequestMin": 0,
            "maxMessagesPerRequestMax": 250,
            "maxMessagesPerRequestStep": 1,
            "prompt_builder": 0
        },
        "note": {
            "default": "",
            "chara": [],
            "wiAddition": []
        },
        "caption": {
            "refine_mode": false,
            "source": "extras",
            "multimodal_api": "openai",
            "multimodal_model": "gpt-4-turbo",
            "prompt": "What's in this image?",
            "template": "[{{user}} sends {{char}} a picture that contains: {{caption}}]",
            "show_in_chat": false
        },
        "expressions": {
            "api": 99,
            "custom": [],
            "showDefault": false,
            "translate": false,
            "llmPrompt": "Ignore previous instructions. Classify the emotion of the last message. Output just one word, e.g. \"joy\" or \"anger\". Choose only one of the following labels: {{labels}}",
            "allowMultiple": true,
            "rerollIfSame": false,
            "promptType": "raw"
        },
        "connectionManager": {
            "selectedProfile": "",
            "profiles": []
        },
        "dice": {},
        "regex": [],
        "regex_presets": [],
        "character_allowed_regex": [],
        "preset_allowed_regex": {},
        "tts": {
            "voiceMap": "",
            "ttsEnabled": false,
            "currentProvider": "ElevenLabs",
            "auto_generation": true,
            "narrate_user": false,
            "playback_rate": 1,
            "multi_voice_enabled": false,
            "apply_regex": false,
            "regex_pattern": "",
            "ElevenLabs": {}
        },
        "sd": {
            "prompts": {
                "0": "In the next response I want you to provide only a detailed comma-delimited list of keywords and phrases which describe {{char}}. The list must include all of the following items in this order: name, species and race, gender, age, clothing, occupation, physical features and appearances. Do not include descriptions of non-visual qualities such as personality, movements, scents, mental traits, or anything which could not be seen in a still photograph. Do not write in full sentences. Prefix your description with the phrase 'full body portrait,'",
                "1": "Ignore previous instructions and provide a detailed description of {{user}}'s physical appearance from the perspective of {{char}} in the form of a comma-delimited list of keywords and phrases. The list must include all of the following items in this order: name, species and race, gender, age, clothing, occupation, physical features and appearances. Do not include descriptions of non-visual qualities such as personality, movements, scents, mental traits, or anything which could not be seen in a still photograph. Do not write in full sentences. Prefix your description with the phrase 'full body portrait,'. Ignore the rest of the story when crafting this description. Do not reply as {{char}} when writing this description, and do not attempt to continue the story.",
                "2": "Ignore previous instructions and provide a detailed description for all of the following: a brief recap of recent events in the story, {{char}}'s appearance, and {{char}}'s surroundings. Do not reply as {{char}} while writing this description.",
                "3": "Ignore previous instructions and provide ONLY the last chat message string back to me verbatim. Do not write anything after the string. Do not reply as {{char}} when writing this description, and do not attempt to continue the story.",
                "4": "Ignore previous instructions. Your next response must be formatted as a single comma-delimited list of concise keywords.  The list will describe of the visual details included in the last chat message.\n\n    Only mention characters by using pronouns ('he','his','she','her','it','its') or neutral nouns ('male', 'the man', 'female', 'the woman').\n\n    Ignore non-visible things such as feelings, personality traits, thoughts, and spoken dialog.\n\n    Add keywords in this precise order:\n    a keyword to describe the location of the scene,\n    a keyword to mention how many characters of each gender or type are present in the scene (minimum of two characters:\n    {{user}} and {{char}}, example: '2 men ' or '1 man 1 woman ', '1 man 3 robots'),\n\n    keywords to describe the relative physical positioning of the characters to each other (if a commonly known term for the positioning is known use it instead of describing the positioning in detail) + 'POV',\n\n    a single keyword or phrase to describe the primary act taking place in the last chat message,\n\n    keywords to describe {{char}}'s physical appearance and facial expression,\n    keywords to describe {{char}}'s actions,\n    keywords to describe {{user}}'s physical appearance and actions.\n\n    If character actions involve direct physical interaction with another character, mention specifically which body parts interacting and how.\n\n    A correctly formatted example response would be:\n    '(location),(character list by gender),(primary action), (relative character position) POV, (character 1's description and actions), (character 2's description and actions)'",
                "5": "In the next response I want you to provide only a detailed comma-delimited list of keywords and phrases which describe {{char}}. The list must include all of the following items in this order: name, species and race, gender, age, facial features and expressions, occupation, hair and hair accessories (if any), what they are wearing on their upper body (if anything). Do not describe anything below their neck. Do not include descriptions of non-visual qualities such as personality, movements, scents, mental traits, or anything which could not be seen in a still photograph. Do not write in full sentences. Prefix your description with the phrase 'close up facial portrait,'",
                "7": "Ignore previous instructions and provide a detailed description of {{char}}'s surroundings in the form of a comma-delimited list of keywords and phrases. The list must include all of the following items in this order: location, time of day, weather, lighting, and any other relevant details. Do not include descriptions of characters and non-visual qualities such as names, personality, movements, scents, mental traits, or anything which could not be seen in a still photograph. Do not write in full sentences. Prefix your description with the phrase 'background,'. Ignore the rest of the story when crafting this description. Do not reply as {{char}} when writing this description, and do not attempt to continue the story.",
                "8": "Provide an exhaustive comma-separated list of tags describing the appearance of the character on this image in great detail. Start with \"full body portrait\".",
                "9": "Provide an exhaustive comma-separated list of tags describing the appearance of the character on this image in great detail. Start with \"full body portrait\".",
                "10": "Provide an exhaustive comma-separated list of tags describing the appearance of the character on this image in great detail. Start with \"close-up portrait\".",
                "11": "Ignore previous instructions and provide an exhaustive comma-separated list of tags describing the appearance of \"{0}\" in great detail. Start with {{charPrefix}} (sic) if the subject is associated with {{char}}.",
                "-1": "[{{char}} sends a picture that contains: {{prompt}}].",
                "-2": "The text prompt used to generate the image. Must represent an exhaustive description of the desired image that will allow an artist or a photographer to perfectly recreate it."
            },
            "character_prompts": {},
            "character_negative_prompts": {},
            "source": "extras",
            "scale_min": 1,
            "scale_max": 30,
            "scale_step": 0.1,
            "scale": 7,
            "steps_min": 1,
            "steps_max": 150,
            "steps_step": 1,
            "steps": 20,
            "scheduler": "normal",
            "dimension_min": 64,
            "dimension_max": 2048,
            "dimension_step": 64,
            "width": 512,
            "height": 512,
            "prompt_prefix": "best quality, absurdres, aesthetic,",
            "negative_prompt": "lowres, bad anatomy, bad hands, text, error, cropped, worst quality, low quality, normal quality, jpeg artifacts, signature, watermark, username, blurry",
            "sampler": "DDIM",
            "model": "",
            "vae": "",
            "seed": -1,
            "restore_faces": false,
            "enable_hr": false,
            "adetailer_face": false,
            "horde": false,
            "horde_nsfw": false,
            "horde_karras": true,
            "horde_sanitize": true,
            "refine_mode": false,
            "interactive_mode": false,
            "multimodal_captioning": false,
            "snap": false,
            "free_extend": false,
            "function_tool": false,
            "minimal_prompt_processing": false,
            "auto_url": "http://localhost:7860",
            "auto_auth": "",
            "sdcpp_url": "http://127.0.0.1:1234",
            "vlad_url": "http://localhost:7860",
            "vlad_auth": "",
            "drawthings_url": "http://localhost:7860",
            "drawthings_auth": "",
            "hr_upscaler": "Latent",
            "hr_scale": 1,
            "hr_scale_min": 1,
            "hr_scale_max": 4,
            "hr_scale_step": 0.1,
            "denoising_strength": 0.7,
            "denoising_strength_min": 0,
            "denoising_strength_max": 1,
            "denoising_strength_step": 0.01,
            "hr_second_pass_steps": 0,
            "hr_second_pass_steps_min": 0,
            "hr_second_pass_steps_max": 150,
            "hr_second_pass_steps_step": 1,
            "clip_skip_min": 1,
            "clip_skip_max": 12,
            "clip_skip_step": 1,
            "clip_skip": 1,
            "novel_anlas_guard": false,
            "novel_sm": false,
            "novel_sm_dyn": false,
            "novel_decrisper": false,
            "novel_variety_boost": false,
            "openai_style": "vivid",
            "openai_quality": "standard",
            "openai_quality_gpt": "auto",
            "openai_duration": "8",
            "style": "Default",
            "styles": [
                {
                    "name": "Default",
                    "negative": "lowres, bad anatomy, bad hands, text, error, cropped, worst quality, low quality, normal quality, jpeg artifacts, signature, watermark, username, blurry",
                    "prefix": "best quality, absurdres, aesthetic,"
                }
            ],
            "comfy_type": "standard",
            "comfy_url": "http://127.0.0.1:8188",
            "comfy_workflow": "Default_Comfy_Workflow.json",
            "comfy_runpod_url": "",
            "pollinations_enhance": false,
            "wand_visible": false,
            "command_visible": false,
            "interactive_visible": false,
            "tool_visible": false,
            "stability_style_preset": "anime",
            "bfl_upsampling": false,
            "google_api": "makersuite",
            "google_enhance": true,
            "google_duration": 6
        },
        "chromadb": {},
        "translate": {
            "target_language": "en",
            "internal_language": "en",
            "provider": "google",
            "auto_mode": "none",
            "deepl_endpoint": "free"
        },
        "objective": {},
        "quickReply": {},
        "randomizer": {
            "controls": [],
            "fluctuation": 0.1,
            "enabled": false
        },
        "speech_recognition": {},
        "rvc": {},
        "hypebot": {},
        "vectors": {},
        "variables": {
            "global": {}
        },
        "attachments": [],
        "character_attachments": {},
        "disabled_attachments": [],
        "gallery": {
            "folders": {},
            "sort": "dateAsc"
        },
        "SillyTavern-Dialogue-Colorizer": {
            "charColorSettings": {
                "colorizeSource": "avatar_vibrant",
                "staticColor": "#e18a24",
                "colorOverrides": {}
            },
            "personaColorSettings": {
                "colorizeSource": "avatar_vibrant",
                "staticColor": "#e18a24",
                "colorOverrides": {}
            },
            "colorizeTargets": 1,
            "chatBubbleLightness": 0.15
        },
        "EjsTemplate": {
            "enabled": true,
            "generate_enabled": true,
            "generate_loader_enabled": true,
            "render_enabled": true,
            "render_loader_enabled": true,
            "with_context_disabled": false,
            "debug_enabled": false,
            "autosave_enabled": false,
            "preload_worldinfo_enabled": true,
            "code_blocks_enabled": true,
            "raw_message_evaluation_enabled": true,
            "filter_message_enabled": true,
            "cache_enabled": 0,
            "cache_size": 64,
            "cache_hasher": "h32ToString",
            "inject_loader_enabled": false,
            "invert_enabled": true,
            "depth_limit": -1,
            "compile_workers": false,
            "sandbox": false
        },
        "st-input-helper": {
            "enabled": true,
            "buttons": {
                "asterisk": true,
                "quotes": true,
                "parentheses": true,
                "bookQuotes1": true,
                "bookQuotes2": true,
                "bookQuotes3": true,
                "newline": true,
                "user": true,
                "char": true
            },
            "shortcuts": {
                "asterisk": "",
                "quotes": "",
                "parentheses": "",
                "bookQuotes1": "",
                "bookQuotes2": "",
                "bookQuotes3": "",
                "newline": "",
                "user": "",
                "char": ""
            },
            "buttonOrder": [
                "asterisk",
                "quotes",
                "parentheses",
                "bookQuotes1",
                "bookQuotes2",
                "bookQuotes3",
                "newline",
                "user",
                "char"
            ],
            "customSymbols": []
        },
        "quickReplyV2": {
            "isEnabled": false,
            "isCombined": false,
            "isPopout": false,
            "config": {
                "setList": [
                    {
                        "set": "Default",
                        "isVisible": true
                    }
                ]
            },
            "characterConfigs": {}
        },
        "cfg": {
            "global": {
                "guidance_scale": 1,
                "negative_prompt": ""
            },
            "chara": []
        }
    },
    "tags": [
        {
            "id": "9dc2a6f2-ae35-49da-aaa7-ff0d6d266f91",
            "name": "Plain Text",
            "create_date": 1774472307711
        },
        {
            "id": "c1f33656-e0b3-485e-9ee5-655ba400a664",
            "name": "OpenAI",
            "create_date": 1774472307711
        },
        {
            "id": "24bd7d1f-7d99-4eb7-9926-54f54b0cdb5a",
            "name": "W++",
            "create_date": 1774472307711
        },
        {
            "id": "cf10865e-90e8-41c1-9388-a389d1b298f9",
            "name": "Boostyle",
            "create_date": 1774472307711
        },
        {
            "id": "7d94cb42-1460-4639-ada2-4528b1f18eab",
            "name": "PList",
            "create_date": 1774472307711
        },
        {
            "id": "3b54bf92-d0f6-4ae1-983f-1673350baaa5",
            "name": "AliChat",
            "create_date": 1774472307711
        }
    ],
    "tag_map": {
        "default_Seraphina.png": []
    },
    "nai_settings": {
        "min_p": 0,
        "math1_temp": 1,
        "math1_quad": 0,
        "math1_quad_entropy_scale": 0,
        "streaming_novel": false,
        "preamble": "[ Style: chat, complex, sensory, visceral ]",
        "banned_tokens": "",
        "order": [
            1,
            5,
            0,
            2,
            3,
            4
        ],
        "logit_bias": [],
        "extensions": {}
    },
    "kai_settings": {
        "temp": 1,
        "rep_pen": 1,
        "rep_pen_range": 0,
        "top_p": 1,
        "min_p": 0,
        "top_a": 1,
        "top_k": 0,
        "typical": 1,
        "tfs": 1,
        "rep_pen_slope": 0.9,
        "streaming_kobold": false,
        "sampler_order": [
            0,
            1,
            2,
            3,
            4,
            5,
            6
        ],
        "mirostat": 0,
        "mirostat_tau": 5,
        "mirostat_eta": 0.1,
        "use_default_badwordsids": false,
        "grammar": "",
        "seed": -1,
        "preset_settings": "gui",
        "extensions": {}
    },
    "oai_settings": {
        "preset_settings_openai": "Default",
        "temp_openai": 1,
        "freq_pen_openai": 0,
        "pres_pen_openai": 0,
        "top_p_openai": 1,
        "top_k_openai": 0,
        "min_p_openai": 0,
        "top_a_openai": 0,
        "repetition_penalty_openai": 1,
        "stream_openai": false,
        "openai_max_context": 2000000,
        "openai_max_tokens": 300,
        "prompts": [
            {
                "name": "Main Prompt",
                "system_prompt": true,
                "role": "system",
                "content": "Write {{char}}'s next reply in a fictional chat between {{charIfNotGroup}} and {{user}}.",
                "identifier": "main"
            },
            {
                "name": "Auxiliary Prompt",
                "system_prompt": true,
                "role": "system",
                "content": "",
                "identifier": "nsfw"
            },
            {
                "identifier": "dialogueExamples",
                "name": "Chat Examples",
                "system_prompt": true,
                "marker": true
            },
            {
                "name": "Post-History Instructions",
                "system_prompt": true,
                "role": "system",
                "content": "",
                "identifier": "jailbreak"
            },
            {
                "identifier": "chatHistory",
                "name": "Chat History",
                "system_prompt": true,
                "marker": true
            },
            {
                "identifier": "worldInfoAfter",
                "name": "World Info (after)",
                "system_prompt": true,
                "marker": true
            },
            {
                "identifier": "worldInfoBefore",
                "name": "World Info (before)",
                "system_prompt": true,
                "marker": true
            },
            {
                "identifier": "enhanceDefinitions",
                "role": "system",
                "name": "Enhance Definitions",
                "content": "If you have more knowledge of {{char}}, add to the character's lore and personality to enhance them but keep the Character Sheet's definitions absolute.",
                "system_prompt": true,
                "marker": false
            },
            {
                "identifier": "charDescription",
                "name": "Char Description",
                "system_prompt": true,
                "marker": true
            },
            {
                "identifier": "charPersonality",
                "name": "Char Personality",
                "system_prompt": true,
                "marker": true
            },
            {
                "identifier": "scenario",
                "name": "Scenario",
                "system_prompt": true,
                "marker": true
            },
            {
                "identifier": "personaDescription",
                "name": "Persona Description",
                "system_prompt": true,
                "marker": true
            }
        ],
        "prompt_order": [
            {
                "character_id": 100001,
                "order": [
                    {
                        "identifier": "main",
                        "enabled": true
                    },
                    {
                        "identifier": "worldInfoBefore",
                        "enabled": true
                    },
                    {
                        "identifier": "personaDescription",
                        "enabled": true
                    },
                    {
                        "identifier": "charDescription",
                        "enabled": true
                    },
                    {
                        "identifier": "charPersonality",
                        "enabled": true
                    },
                    {
                        "identifier": "scenario",
                        "enabled": true
                    },
                    {
                        "identifier": "enhanceDefinitions",
                        "enabled": false
                    },
                    {
                        "identifier": "nsfw",
                        "enabled": true
                    },
                    {
                        "identifier": "worldInfoAfter",
                        "enabled": true
                    },
                    {
                        "identifier": "dialogueExamples",
                        "enabled": true
                    },
                    {
                        "identifier": "chatHistory",
                        "enabled": true
                    },
                    {
                        "identifier": "jailbreak",
                        "enabled": true
                    }
                ]
            }
        ],
        "send_if_empty": "",
        "impersonation_prompt": "[Write your next reply from the point of view of {{user}}, using the chat history so far as a guideline for the writing style of {{user}}. Don't write as {{char}} or system. Don't describe actions of {{char}}.]",
        "new_chat_prompt": "[Start a new Chat]",
        "new_group_chat_prompt": "[Start a new group chat. Group members: {{group}}]",
        "new_example_chat_prompt": "[Example Chat]",
        "continue_nudge_prompt": "[Continue your last message without repeating its original content.]",
        "bias_preset_selected": "Default (none)",
        "bias_presets": {
            "Default (none)": [],
            "Anti-bond": [
                {
                    "id": "22154f79-dd98-41bc-8e34-87015d6a0eaf",
                    "text": " bond",
                    "value": -50
                },
                {
                    "id": "8ad2d5c4-d8ef-49e4-bc5e-13e7f4690e0f",
                    "text": " future",
                    "value": -50
                },
                {
                    "id": "52a4b280-0956-4940-ac52-4111f83e4046",
                    "text": " bonding",
                    "value": -50
                },
                {
                    "id": "e63037c7-c9d1-4724-ab2d-7756008b433b",
                    "text": " connection",
                    "value": -25
                }
            ]
        },
        "wi_format": "{0}",
        "group_nudge_prompt": "[Write the next reply only as {{char}}.]",
        "scenario_format": "{{scenario}}",
        "personality_format": "{{personality}}",
        "openai_model": "gpt-4-turbo",
        "claude_model": "claude-sonnet-4-5",
        "google_model": "gemini-2.5-pro",
        "vertexai_model": "gemini-2.5-pro",
        "ai21_model": "jamba-large",
        "mistralai_model": "mistral-large-latest",
        "cohere_model": "command-r-plus",
        "perplexity_model": "sonar-pro",
        "groq_model": "llama-3.3-70b-versatile",
        "chutes_model": "deepseek-ai/DeepSeek-V3-0324",
        "chutes_sort_models": "alphabetically",
        "siliconflow_model": "deepseek-ai/DeepSeek-V3",
        "electronhub_model": "gpt-4o-mini",
        "electronhub_sort_models": "alphabetically",
        "electronhub_group_models": false,
        "nanogpt_model": "gpt-4o-mini",
        "deepseek_model": "deepseek-chat",
        "aimlapi_model": "chatgpt-4o-latest",
        "xai_model": "grok-3-beta",
        "pollinations_model": "gemini",
        "cometapi_model": "gpt-4o",
        "moonshot_model": "kimi-latest",
        "fireworks_model": "accounts/fireworks/models/kimi-k2-instruct",
        "zai_model": "glm-4.6",
        "zai_endpoint": "common",
        "azure_base_url": "",
        "azure_deployment_name": "",
        "azure_api_version": "2024-02-15-preview",
        "azure_openai_model": "",
        "custom_model": "gemini-3-pro-preview",
        "custom_url": "http://localhost:8000",
        "custom_include_body": "",
        "custom_exclude_body": "",
        "custom_include_headers": "",
        "openrouter_model": "OR_Website",
        "openrouter_use_fallback": false,
        "openrouter_group_models": false,
        "openrouter_sort_models": "alphabetically",
        "openrouter_providers": [],
        "openrouter_quantizations": [],
        "openrouter_allow_fallbacks": true,
        "openrouter_middleout": "on",
        "reverse_proxy": "",
        "chat_completion_source": "pollinations",
        "chat_completion_source": "openai",
        "max_context_unlocked": true,
        "show_external_models": false,
        "proxy_password": "",
        "assistant_prefill": "",
        "assistant_impersonation": "",
        "use_sysprompt": false,
        "vertexai_auth_mode": "express",
        "vertexai_region": "us-central1",
        "vertexai_express_project_id": "",
        "squash_system_messages": false,
        "media_inlining": true,
        "inline_image_quality": "auto",
        "bypass_status_check": false,
        "continue_prefill": false,
        "function_calling": false,
        "names_behavior": 0,
        "continue_postfix": " ",
        "custom_prompt_post_processing": "",
        "show_thoughts": true,
        "reasoning_effort": "auto",
        "verbosity": "auto",
        "enable_web_search": false,
        "request_images": false,
        "request_image_aspect_ratio": "",
        "request_image_resolution": "",
        "seed": -1,
        "n": 1,
        "bind_preset_to_connection": true,
        "extensions": {}
    },
    "background": {
        "name": "__transparent.png",
        "url": "url(\"backgrounds/__transparent.png\")",
        "fitting": "classic",
        "animation": false,
        "sortOrder": "az",
        "thumbnailColumns": 3
    },
    "proxies": [
        {
            "name": "None",
            "url": "",
            "password": ""
        }
    ],
    "selected_proxy": {
        "name": "None",
        "url": "",
        "password": ""
    }
}"####;
