use std::{collections::BTreeMap, path::Path};

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopAppStageLabel {
    Dev,
    Nightly,
    Alpha,
}

impl DesktopAppStageLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "Dev",
            Self::Nightly => "Nightly",
            Self::Alpha => "Alpha",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopAppBranding {
    pub base_name: String,
    pub stage_label: DesktopAppStageLabel,
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopRuntimeArch {
    Arm64,
    X64,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopRuntimeInfo {
    pub host_arch: DesktopRuntimeArch,
    pub app_arch: DesktopRuntimeArch,
    pub running_under_arm64_translation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopUpdateChannel {
    Latest,
    Nightly,
}

impl DesktopUpdateChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Latest => "latest",
            Self::Nightly => "nightly",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DesktopConfig {
    pub app_data_directory: Option<String>,
    pub xdg_config_home: Option<String>,
    pub r3_home: Option<String>,
    pub dev_server_url: Option<String>,
    pub dev_remote_r3_server_entry_path: Option<String>,
    pub configured_backend_port: Option<u16>,
    pub commit_hash_override: Option<String>,
    pub desktop_lan_host_override: Option<String>,
    pub desktop_https_endpoint_urls: Vec<String>,
    pub otlp_traces_url: Option<String>,
    pub otlp_export_interval_ms: i64,
    pub app_image_path: Option<String>,
    pub disable_auto_update: bool,
    pub mock_updates: bool,
    pub mock_update_server_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MakeDesktopEnvironmentInput {
    pub dirname: String,
    pub home_directory: String,
    pub platform: String,
    pub process_arch: String,
    pub app_version: String,
    pub app_path: String,
    pub is_packaged: bool,
    pub resources_path: String,
    pub running_under_arm64_translation: bool,
    pub config: DesktopConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEnvironment {
    pub dirname: String,
    pub platform: String,
    pub process_arch: String,
    pub is_packaged: bool,
    pub is_development: bool,
    pub app_version: String,
    pub app_path: String,
    pub resources_path: String,
    pub home_directory: String,
    pub app_data_directory: String,
    pub base_dir: String,
    pub state_dir: String,
    pub desktop_settings_path: String,
    pub client_settings_path: String,
    pub saved_environment_registry_path: String,
    pub server_settings_path: String,
    pub log_dir: String,
    pub root_dir: String,
    pub app_root: String,
    pub backend_entry_path: String,
    pub backend_cwd: String,
    pub preload_path: String,
    pub app_update_yml_path: String,
    pub branding: DesktopAppBranding,
    pub display_name: String,
    pub app_user_model_id: String,
    pub linux_desktop_entry_name: String,
    pub linux_wm_class: String,
    pub user_data_dir_name: String,
    pub legacy_user_data_dir_name: String,
    pub runtime_info: DesktopRuntimeInfo,
    pub development_dock_icon_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopBackendExposure {
    pub port: u16,
    pub bind_host: String,
    pub http_base_url: String,
    pub tailscale_serve_enabled: bool,
    pub tailscale_serve_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopBackendBootstrapConfig {
    pub mode: String,
    pub no_browser: bool,
    pub port: u16,
    pub r3_home: String,
    pub host: String,
    pub desktop_bootstrap_token: String,
    pub tailscale_serve_enabled: bool,
    pub tailscale_serve_port: Option<u16>,
    pub otlp_traces_url: Option<String>,
    pub otlp_metrics_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopBackendStartConfig {
    pub executable_path: String,
    pub entry_path: String,
    pub cwd: String,
    pub env_patch: BTreeMap<String, Option<String>>,
    pub bootstrap: DesktopBackendBootstrapConfig,
    pub http_base_url: String,
    pub capture_output: bool,
}

pub const SSH_PASSWORD_PROMPT_CANCELLED_RESULT: &str = "ssh-password-prompt-cancelled";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopIpcRegistrationKind {
    Invoke,
    Sync,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopIpcChannelSpec {
    pub constant_name: &'static str,
    pub channel: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopIpcHandlerSpec {
    pub method_name: &'static str,
    pub channel: &'static str,
    pub kind: DesktopIpcRegistrationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopPreloadBridgeCallKind {
    SendSync,
    Invoke,
    On,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopPreloadBridgeMethodSpec {
    pub method_name: &'static str,
    pub channel: &'static str,
    pub call_kind: DesktopPreloadBridgeCallKind,
    pub validates_object_result: bool,
    pub validates_object_event: bool,
    pub validates_string_event: bool,
    pub unwraps_ssh_cancelled_result: bool,
    pub returns_unsubscribe: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopServerExposureMode {
    LocalOnly,
    NetworkAccessible,
}

impl DesktopServerExposureMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalOnly => "local-only",
            Self::NetworkAccessible => "network-accessible",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSettings {
    pub server_exposure_mode: DesktopServerExposureMode,
    pub tailscale_serve_enabled: bool,
    pub tailscale_serve_port: u16,
    pub update_channel: DesktopUpdateChannel,
    pub update_channel_configured_by_user: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSettingsDocument {
    pub server_exposure_mode: Option<DesktopServerExposureMode>,
    pub tailscale_serve_enabled: Option<bool>,
    pub tailscale_serve_port: Option<i64>,
    pub update_channel: Option<DesktopUpdateChannel>,
    pub update_channel_configured_by_user: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSettingsChange {
    pub settings: DesktopSettings,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSshTargetRecord {
    pub alias: String,
    pub hostname: String,
    pub username: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedSavedEnvironmentRecord {
    pub environment_id: String,
    pub label: String,
    pub http_base_url: String,
    pub ws_base_url: String,
    pub created_at: String,
    pub last_connected_at: Option<String>,
    pub desktop_ssh: Option<DesktopSshTargetRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedEnvironmentStorageRecord {
    pub environment_id: String,
    pub label: String,
    pub http_base_url: String,
    pub ws_base_url: String,
    pub created_at: String,
    pub last_connected_at: Option<String>,
    pub desktop_ssh: Option<DesktopSshTargetRecord>,
    pub encrypted_bearer_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedEnvironmentRegistryDocument {
    pub version: u32,
    pub records: Vec<SavedEnvironmentStorageRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMenuItemSpec {
    pub label: Option<&'static str>,
    pub role: Option<&'static str>,
    pub accelerator: Option<&'static str>,
    pub action: Option<&'static str>,
    pub separator: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopTopLevelMenuSpec {
    pub label: Option<String>,
    pub role: Option<&'static str>,
    pub submenu: Vec<DesktopMenuItemSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopTitleBarStyle {
    Hidden,
    HiddenInset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopTitleBarOverlay {
    pub color: String,
    pub height: u16,
    pub symbol_color: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopWindowTitleBarOptions {
    pub title_bar_style: DesktopTitleBarStyle,
    pub traffic_light_position: Option<(u16, u16)>,
    pub title_bar_overlay: Option<DesktopTitleBarOverlay>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMainWindowOptions {
    pub width: u16,
    pub height: u16,
    pub min_width: u16,
    pub min_height: u16,
    pub show: bool,
    pub auto_hide_menu_bar: bool,
    pub background_color: String,
    pub title: String,
    pub title_bar: DesktopWindowTitleBarOptions,
    pub context_isolation: bool,
    pub node_integration: bool,
    pub sandbox: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopNetworkInterfaceInfo {
    pub address: String,
    pub family: String,
    pub internal: bool,
    pub netmask: Option<String>,
    pub mac: Option<String>,
    pub cidr: Option<String>,
    pub scope_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDesktopServerExposure {
    pub mode: DesktopServerExposureMode,
    pub bind_host: String,
    pub local_http_url: String,
    pub local_ws_url: String,
    pub endpoint_url: Option<String>,
    pub advertised_host: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopServerExposureState {
    pub mode: DesktopServerExposureMode,
    pub endpoint_url: Option<String>,
    pub advertised_host: Option<String>,
    pub tailscale_serve_enabled: bool,
    pub tailscale_serve_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopServerExposureRuntimeState {
    pub requested_mode: DesktopServerExposureMode,
    pub mode: DesktopServerExposureMode,
    pub port: u16,
    pub bind_host: String,
    pub local_http_url: String,
    pub local_ws_url: String,
    pub http_base_url: String,
    pub endpoint_url: Option<String>,
    pub advertised_host: Option<String>,
    pub tailscale_serve_enabled: bool,
    pub tailscale_serve_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDesktopServerExposureRuntimeState {
    pub state: DesktopServerExposureRuntimeState,
    pub unavailable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopServerExposureBackendConfig {
    pub port: u16,
    pub bind_host: String,
    pub http_base_url: String,
    pub tailscale_serve_enabled: bool,
    pub tailscale_serve_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopServerExposureChange {
    pub state: DesktopServerExposureState,
    pub requires_relaunch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopAdvertisedEndpointProvider {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: &'static str,
    pub is_addon: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopAdvertisedEndpointCompatibility {
    pub hosted_https_app: &'static str,
    pub desktop_app: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopAdvertisedEndpoint {
    pub id: String,
    pub label: &'static str,
    pub provider: DesktopAdvertisedEndpointProvider,
    pub http_base_url: String,
    pub ws_base_url: String,
    pub reachability: &'static str,
    pub compatibility: DesktopAdvertisedEndpointCompatibility,
    pub source: &'static str,
    pub status: &'static str,
    pub is_default: Option<bool>,
    pub description: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopBackendSnapshot {
    pub desired_running: bool,
    pub ready: bool,
    pub active_pid: Option<u32>,
    pub restart_attempt: u32,
    pub restart_scheduled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TailscaleStatus {
    pub magic_dns_name: Option<String>,
    pub tailnet_ipv4_addresses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TailscaleCommandSpec {
    pub command: &'static str,
    pub args: Vec<String>,
    pub shell_on_windows: bool,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopStateFlags {
    pub backend_ready: bool,
    pub quitting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopBackendChildLogPhase {
    Start,
    End,
}

impl DesktopBackendChildLogPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "START",
            Self::End => "END",
        }
    }

    pub fn message_suffix(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopBackendOutputStream {
    Stdout,
    Stderr,
}

impl DesktopBackendOutputStream {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }

    pub fn level(self) -> &'static str {
        match self {
            Self::Stdout => "INFO",
            Self::Stderr => "ERROR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopBackendChildLogRecord {
    pub message: String,
    pub level: &'static str,
    pub timestamp: String,
    pub annotations: BTreeMap<String, Value>,
    pub spans: BTreeMap<String, Value>,
    pub fiber_id: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopComponentLogAnnotation {
    pub component: String,
    pub annotations: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopRotatingLogFileWriterConfig {
    pub file_path: String,
    pub max_bytes: u64,
    pub max_files: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopLogFileWriterConfigurationError {
    pub option: &'static str,
    pub value: i64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopBackendPortSelection {
    pub port: u16,
    pub selected_by_scan: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopBackendPortUnavailableError {
    pub start_port: u16,
    pub max_port: u16,
    pub hosts: Vec<&'static str>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopDevelopmentBackendPortRequiredError {
    pub message: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopFatalStartupErrorPlan {
    pub log_stage: String,
    pub log_message: String,
    pub dialog_title: &'static str,
    pub dialog_body: String,
    pub should_show_dialog: bool,
    pub request_shutdown: bool,
    pub quit_app: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopShutdownFlags {
    pub requested: bool,
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopBeforeQuitPlan {
    pub prevent_default: bool,
    pub set_quitting: bool,
    pub request_shutdown: bool,
    pub await_complete: bool,
    pub mark_quit_allowed: bool,
    pub quit_after_shutdown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSignalQuitPlan {
    pub set_quitting: bool,
    pub request_shutdown: bool,
    pub await_complete: bool,
    pub quit_app: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopRelaunchPlan {
    pub reason: String,
    pub set_quitting: bool,
    pub request_shutdown: bool,
    pub await_complete: bool,
    pub relaunch: bool,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopIconPaths {
    pub ico: Option<String>,
    pub icns: Option<String>,
    pub png: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronAppMetadata {
    pub app_version: String,
    pub app_path: String,
    pub is_packaged: bool,
    pub resources_path: String,
    pub running_under_arm64_translation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronAppAppendSwitchCommand {
    pub switch_name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronScopedListenerSpec {
    pub event_name: String,
    pub acquire_method: &'static str,
    pub release_method: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronOpenDialogOptions {
    pub properties: Vec<&'static str>,
    pub default_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronConfirmDialogOptions {
    pub dialog_type: &'static str,
    pub buttons: Vec<&'static str>,
    pub default_id: u8,
    pub cancel_id: u8,
    pub no_link: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronThemeSourceChange {
    pub theme: String,
    pub property: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopShellCommandSpec {
    pub command: String,
    pub args: Vec<String>,
    pub shell: bool,
    pub timeout_ms: u64,
    pub terminate_grace_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronSchemePrivilegeSpec {
    pub scheme: &'static str,
    pub standard: bool,
    pub secure: bool,
    pub support_fetch_api: bool,
    pub cors_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElectronProtocolFileResponse {
    Path(String),
    Error(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronContextMenuItem {
    pub id: Option<String>,
    pub label: Option<String>,
    pub destructive: bool,
    pub disabled: bool,
    pub children: Vec<ElectronContextMenuItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronMenuTemplateItem {
    pub label: String,
    pub enabled: bool,
    pub destructive: bool,
    pub separator: bool,
    pub click_id: Option<String>,
    pub children: Vec<ElectronMenuTemplateItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronMenuPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronUpdaterFeedUrl {
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectronUpdaterCommandKind {
    SetFeedUrl,
    SetAutoDownload,
    SetAutoInstallOnAppQuit,
    SetChannel,
    SetAllowPrerelease,
    SetAllowDowngrade,
    SetDisableDifferentialDownload,
    CheckForUpdates,
    DownloadUpdate,
    QuitAndInstall,
}

impl ElectronUpdaterCommandKind {
    pub fn method_name(self) -> &'static str {
        match self {
            Self::SetFeedUrl => "setFeedURL",
            Self::SetAutoDownload => "autoDownload",
            Self::SetAutoInstallOnAppQuit => "autoInstallOnAppQuit",
            Self::SetChannel => "channel",
            Self::SetAllowPrerelease => "allowPrerelease",
            Self::SetAllowDowngrade => "allowDowngrade",
            Self::SetDisableDifferentialDownload => "disableDifferentialDownload",
            Self::CheckForUpdates => "checkForUpdates",
            Self::DownloadUpdate => "downloadUpdate",
            Self::QuitAndInstall => "quitAndInstall",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronUpdaterPropertyCommand {
    pub kind: ElectronUpdaterCommandKind,
    pub bool_value: Option<bool>,
    pub string_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronUpdaterQuitAndInstallOptions {
    pub is_silent: bool,
    pub is_force_run_after: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronUpdaterStateSnapshot {
    pub allow_downgrade: bool,
    pub allow_prerelease: bool,
    pub auto_download: bool,
    pub auto_install_on_app_quit: bool,
    pub channel: String,
    pub disable_differential_download: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectronUpdaterErrorKind {
    CheckForUpdates,
    DownloadUpdate,
    QuitAndInstall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopUpdateStatus {
    Disabled,
    Idle,
    Checking,
    UpToDate,
    Available,
    Downloading,
    Downloaded,
    Error,
}

impl DesktopUpdateStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Idle => "idle",
            Self::Checking => "checking",
            Self::UpToDate => "up-to-date",
            Self::Available => "available",
            Self::Downloading => "downloading",
            Self::Downloaded => "downloaded",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopUpdateAction {
    Check,
    Download,
    Install,
}

impl DesktopUpdateAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Download => "download",
            Self::Install => "install",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopUpdateState {
    pub enabled: bool,
    pub status: DesktopUpdateStatus,
    pub channel: DesktopUpdateChannel,
    pub current_version: String,
    pub host_arch: DesktopRuntimeArch,
    pub app_arch: DesktopRuntimeArch,
    pub running_under_arm64_translation: bool,
    pub available_version: Option<String>,
    pub downloaded_version: Option<String>,
    pub download_percent: Option<f64>,
    pub checked_at: Option<String>,
    pub message: Option<String>,
    pub error_context: Option<DesktopUpdateAction>,
    pub can_retry: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DesktopUpdateInFlightFlags {
    pub check: bool,
    pub download: bool,
    pub install: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopUpdateDisabledReasonInput {
    pub is_development: bool,
    pub is_packaged: bool,
    pub platform: String,
    pub app_image: Option<String>,
    pub disabled_by_env: bool,
    pub has_update_feed_config: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopUpdatesConfigurePlan {
    pub feed_url: Option<ElectronUpdaterFeedUrl>,
    pub enabled: bool,
    pub initial_state_status: DesktopUpdateStatus,
    pub updater_configured: bool,
    pub auto_download: bool,
    pub auto_install_on_app_quit: bool,
    pub allow_prerelease: bool,
    pub allow_downgrade: bool,
    pub disable_differential_download: bool,
    pub listener_events: Vec<&'static str>,
    pub startup_delay_ms: u64,
    pub poll_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopUpdateSetChannelPlan {
    pub accepted: bool,
    pub active_action: Option<DesktopUpdateAction>,
    pub persist_settings: bool,
    pub reset_state: bool,
    pub apply_auto_updater_channel: bool,
    pub temporarily_allow_downgrade: bool,
    pub check_reason: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopUpdateActionResultPlan {
    pub accepted: bool,
    pub completed: bool,
    pub stop_backend: bool,
    pub destroy_windows: bool,
    pub quit_and_install: Option<ElectronUpdaterQuitAndInstallOptions>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopUpdateStateQueryOptions {
    pub query_key: Vec<&'static str>,
    pub stale_time: f64,
    pub refetch_on_mount: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronWindowState {
    pub id: u32,
    pub destroyed: bool,
    pub minimized: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronWindowRevealPlan {
    pub window_id: u32,
    pub restore: bool,
    pub show: bool,
    pub app_focus_steal: bool,
    pub focus: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectronWindowSendAllPlan {
    pub channel: String,
    pub args_len: usize,
    pub target_window_ids: Vec<u32>,
}

pub const DESKTOP_IPC_CHANNEL_SPECS: &[DesktopIpcChannelSpec] = &[
    channel("PICK_FOLDER_CHANNEL", "desktop:pick-folder"),
    channel("CONFIRM_CHANNEL", "desktop:confirm"),
    channel("SET_THEME_CHANNEL", "desktop:set-theme"),
    channel("CONTEXT_MENU_CHANNEL", "desktop:context-menu"),
    channel("OPEN_EXTERNAL_CHANNEL", "desktop:open-external"),
    channel("MENU_ACTION_CHANNEL", "desktop:menu-action"),
    channel("UPDATE_STATE_CHANNEL", "desktop:update-state"),
    channel("UPDATE_GET_STATE_CHANNEL", "desktop:update-get-state"),
    channel("UPDATE_SET_CHANNEL_CHANNEL", "desktop:update-set-channel"),
    channel("UPDATE_DOWNLOAD_CHANNEL", "desktop:update-download"),
    channel("UPDATE_INSTALL_CHANNEL", "desktop:update-install"),
    channel("UPDATE_CHECK_CHANNEL", "desktop:update-check"),
    channel("GET_APP_BRANDING_CHANNEL", "desktop:get-app-branding"),
    channel(
        "GET_LOCAL_ENVIRONMENT_BOOTSTRAP_CHANNEL",
        "desktop:get-local-environment-bootstrap",
    ),
    channel("GET_CLIENT_SETTINGS_CHANNEL", "desktop:get-client-settings"),
    channel("SET_CLIENT_SETTINGS_CHANNEL", "desktop:set-client-settings"),
    channel(
        "GET_SAVED_ENVIRONMENT_REGISTRY_CHANNEL",
        "desktop:get-saved-environment-registry",
    ),
    channel(
        "SET_SAVED_ENVIRONMENT_REGISTRY_CHANNEL",
        "desktop:set-saved-environment-registry",
    ),
    channel(
        "GET_SAVED_ENVIRONMENT_SECRET_CHANNEL",
        "desktop:get-saved-environment-secret",
    ),
    channel(
        "SET_SAVED_ENVIRONMENT_SECRET_CHANNEL",
        "desktop:set-saved-environment-secret",
    ),
    channel(
        "REMOVE_SAVED_ENVIRONMENT_SECRET_CHANNEL",
        "desktop:remove-saved-environment-secret",
    ),
    channel("DISCOVER_SSH_HOSTS_CHANNEL", "desktop:discover-ssh-hosts"),
    channel(
        "ENSURE_SSH_ENVIRONMENT_CHANNEL",
        "desktop:ensure-ssh-environment",
    ),
    channel(
        "DISCONNECT_SSH_ENVIRONMENT_CHANNEL",
        "desktop:disconnect-ssh-environment",
    ),
    channel(
        "FETCH_SSH_ENVIRONMENT_DESCRIPTOR_CHANNEL",
        "desktop:fetch-ssh-environment-descriptor",
    ),
    channel(
        "BOOTSTRAP_SSH_BEARER_SESSION_CHANNEL",
        "desktop:bootstrap-ssh-bearer-session",
    ),
    channel(
        "FETCH_SSH_SESSION_STATE_CHANNEL",
        "desktop:fetch-ssh-session-state",
    ),
    channel(
        "ISSUE_SSH_WEBSOCKET_TOKEN_CHANNEL",
        "desktop:issue-ssh-websocket-token",
    ),
    channel("SSH_PASSWORD_PROMPT_CHANNEL", "desktop:ssh-password-prompt"),
    channel(
        "RESOLVE_SSH_PASSWORD_PROMPT_CHANNEL",
        "desktop:resolve-ssh-password-prompt",
    ),
    channel(
        "GET_SERVER_EXPOSURE_STATE_CHANNEL",
        "desktop:get-server-exposure-state",
    ),
    channel(
        "SET_SERVER_EXPOSURE_MODE_CHANNEL",
        "desktop:set-server-exposure-mode",
    ),
    channel(
        "SET_TAILSCALE_SERVE_ENABLED_CHANNEL",
        "desktop:set-tailscale-serve-enabled",
    ),
    channel(
        "GET_ADVERTISED_ENDPOINTS_CHANNEL",
        "desktop:get-advertised-endpoints",
    ),
];

pub const DESKTOP_PRELOAD_WORLD_KEY: &str = "desktopBridge";
pub const DESKTOP_PRELOAD_BRIDGE_METHOD_SPECS: &[DesktopPreloadBridgeMethodSpec] = &[
    preload_method(
        "getAppBranding",
        "desktop:get-app-branding",
        DesktopPreloadBridgeCallKind::SendSync,
        true,
        false,
        false,
        false,
        false,
    ),
    preload_method(
        "getLocalEnvironmentBootstrap",
        "desktop:get-local-environment-bootstrap",
        DesktopPreloadBridgeCallKind::SendSync,
        true,
        false,
        false,
        false,
        false,
    ),
    preload_invoke("getClientSettings", "desktop:get-client-settings"),
    preload_invoke("setClientSettings", "desktop:set-client-settings"),
    preload_invoke(
        "getSavedEnvironmentRegistry",
        "desktop:get-saved-environment-registry",
    ),
    preload_invoke(
        "setSavedEnvironmentRegistry",
        "desktop:set-saved-environment-registry",
    ),
    preload_invoke(
        "getSavedEnvironmentSecret",
        "desktop:get-saved-environment-secret",
    ),
    preload_invoke(
        "setSavedEnvironmentSecret",
        "desktop:set-saved-environment-secret",
    ),
    preload_invoke(
        "removeSavedEnvironmentSecret",
        "desktop:remove-saved-environment-secret",
    ),
    preload_invoke("discoverSshHosts", "desktop:discover-ssh-hosts"),
    preload_method(
        "ensureSshEnvironment",
        "desktop:ensure-ssh-environment",
        DesktopPreloadBridgeCallKind::Invoke,
        false,
        false,
        false,
        true,
        false,
    ),
    preload_invoke(
        "disconnectSshEnvironment",
        "desktop:disconnect-ssh-environment",
    ),
    preload_invoke(
        "fetchSshEnvironmentDescriptor",
        "desktop:fetch-ssh-environment-descriptor",
    ),
    preload_invoke(
        "bootstrapSshBearerSession",
        "desktop:bootstrap-ssh-bearer-session",
    ),
    preload_invoke("fetchSshSessionState", "desktop:fetch-ssh-session-state"),
    preload_invoke(
        "issueSshWebSocketToken",
        "desktop:issue-ssh-websocket-token",
    ),
    preload_method(
        "onSshPasswordPrompt",
        "desktop:ssh-password-prompt",
        DesktopPreloadBridgeCallKind::On,
        false,
        true,
        false,
        false,
        true,
    ),
    preload_invoke(
        "resolveSshPasswordPrompt",
        "desktop:resolve-ssh-password-prompt",
    ),
    preload_invoke(
        "getServerExposureState",
        "desktop:get-server-exposure-state",
    ),
    preload_invoke("setServerExposureMode", "desktop:set-server-exposure-mode"),
    preload_invoke(
        "setTailscaleServeEnabled",
        "desktop:set-tailscale-serve-enabled",
    ),
    preload_invoke("getAdvertisedEndpoints", "desktop:get-advertised-endpoints"),
    preload_invoke("pickFolder", "desktop:pick-folder"),
    preload_invoke("confirm", "desktop:confirm"),
    preload_invoke("setTheme", "desktop:set-theme"),
    preload_invoke("showContextMenu", "desktop:context-menu"),
    preload_invoke("openExternal", "desktop:open-external"),
    preload_method(
        "onMenuAction",
        "desktop:menu-action",
        DesktopPreloadBridgeCallKind::On,
        false,
        false,
        true,
        false,
        true,
    ),
    preload_invoke("getUpdateState", "desktop:update-get-state"),
    preload_invoke("setUpdateChannel", "desktop:update-set-channel"),
    preload_invoke("checkForUpdate", "desktop:update-check"),
    preload_invoke("downloadUpdate", "desktop:update-download"),
    preload_invoke("installUpdate", "desktop:update-install"),
    preload_method(
        "onUpdateState",
        "desktop:update-state",
        DesktopPreloadBridgeCallKind::On,
        false,
        true,
        false,
        false,
        true,
    ),
];

pub const DESKTOP_IPC_HANDLER_INSTALL_ORDER: &[DesktopIpcHandlerSpec] = &[
    sync_handler("getAppBranding", "desktop:get-app-branding"),
    sync_handler(
        "getLocalEnvironmentBootstrap",
        "desktop:get-local-environment-bootstrap",
    ),
    invoke_handler("getClientSettings", "desktop:get-client-settings"),
    invoke_handler("setClientSettings", "desktop:set-client-settings"),
    invoke_handler(
        "getSavedEnvironmentRegistry",
        "desktop:get-saved-environment-registry",
    ),
    invoke_handler(
        "setSavedEnvironmentRegistry",
        "desktop:set-saved-environment-registry",
    ),
    invoke_handler(
        "getSavedEnvironmentSecret",
        "desktop:get-saved-environment-secret",
    ),
    invoke_handler(
        "setSavedEnvironmentSecret",
        "desktop:set-saved-environment-secret",
    ),
    invoke_handler(
        "removeSavedEnvironmentSecret",
        "desktop:remove-saved-environment-secret",
    ),
    invoke_handler("discoverSshHosts", "desktop:discover-ssh-hosts"),
    invoke_handler("ensureSshEnvironment", "desktop:ensure-ssh-environment"),
    invoke_handler(
        "disconnectSshEnvironment",
        "desktop:disconnect-ssh-environment",
    ),
    invoke_handler(
        "fetchSshEnvironmentDescriptor",
        "desktop:fetch-ssh-environment-descriptor",
    ),
    invoke_handler(
        "bootstrapSshBearerSession",
        "desktop:bootstrap-ssh-bearer-session",
    ),
    invoke_handler("fetchSshSessionState", "desktop:fetch-ssh-session-state"),
    invoke_handler(
        "issueSshWebSocketToken",
        "desktop:issue-ssh-websocket-token",
    ),
    invoke_handler(
        "resolveSshPasswordPrompt",
        "desktop:resolve-ssh-password-prompt",
    ),
    invoke_handler(
        "getServerExposureState",
        "desktop:get-server-exposure-state",
    ),
    invoke_handler("setServerExposureMode", "desktop:set-server-exposure-mode"),
    invoke_handler(
        "setTailscaleServeEnabled",
        "desktop:set-tailscale-serve-enabled",
    ),
    invoke_handler("getAdvertisedEndpoints", "desktop:get-advertised-endpoints"),
    invoke_handler("pickFolder", "desktop:pick-folder"),
    invoke_handler("confirm", "desktop:confirm"),
    invoke_handler("setTheme", "desktop:set-theme"),
    invoke_handler("showContextMenu", "desktop:context-menu"),
    invoke_handler("openExternal", "desktop:open-external"),
    invoke_handler("getUpdateState", "desktop:update-get-state"),
    invoke_handler("setUpdateChannel", "desktop:update-set-channel"),
    invoke_handler("downloadUpdate", "desktop:update-download"),
    invoke_handler("installUpdate", "desktop:update-install"),
    invoke_handler("checkForUpdate", "desktop:update-check"),
];

pub const DEFAULT_TAILSCALE_SERVE_PORT: u16 = 443;
pub const DESKTOP_LOOPBACK_HOST: &str = "127.0.0.1";
pub const DESKTOP_LAN_BIND_HOST: &str = "0.0.0.0";
pub const DESKTOP_TITLEBAR_HEIGHT: u16 = 40;
pub const DESKTOP_TITLEBAR_COLOR: &str = "#01000000";
pub const DESKTOP_TITLEBAR_LIGHT_SYMBOL_COLOR: &str = "#1f2937";
pub const DESKTOP_TITLEBAR_DARK_SYMBOL_COLOR: &str = "#f8fafc";
pub const DESKTOP_BACKEND_INITIAL_RESTART_DELAY_MS: u64 = 500;
pub const DESKTOP_BACKEND_MAX_RESTART_DELAY_MS: u64 = 10_000;
pub const DESKTOP_BACKEND_READINESS_TIMEOUT_MS: u64 = 60_000;
pub const DESKTOP_BACKEND_READINESS_INTERVAL_MS: u64 = 100;
pub const DESKTOP_BACKEND_READINESS_REQUEST_TIMEOUT_MS: u64 = 1_000;
pub const DESKTOP_BACKEND_TERMINATE_GRACE_MS: u64 = 2_000;
pub const DESKTOP_BACKEND_READINESS_PATH: &str = "/.well-known/t3/environment";
pub const TAILSCALE_STATUS_TIMEOUT_MS: u64 = 1_500;
pub const TAILSCALE_SERVE_TIMEOUT_MS: u64 = 10_000;
pub const TAILSCALE_PROBE_TIMEOUT_MS: u64 = 2_500;
pub const DESKTOP_LOG_FILE_MAX_BYTES: u64 = 10 * 1024 * 1024;
pub const DESKTOP_LOG_FILE_MAX_FILES: u16 = 10;
pub const DESKTOP_BACKEND_CHILD_LOG_FIBER_ID: &str = "#backend-child";
pub const DESKTOP_TRACE_BATCH_WINDOW_MS: u64 = 200;
pub const DEFAULT_DESKTOP_BACKEND_PORT: u16 = 3773;
pub const MAX_DESKTOP_TCP_PORT: u16 = 65_535;
pub const DESKTOP_BACKEND_PORT_PROBE_HOSTS: &[&str] = &["127.0.0.1", "0.0.0.0", "::"];
pub const DESKTOP_RELAUNCH_DEVELOPMENT_EXIT_CODE: i32 = 75;
pub const DESKTOP_RELAUNCH_PACKAGED_EXIT_CODE: i32 = 0;
pub const DESKTOP_AUTO_UPDATE_STARTUP_DELAY_MS: u64 = 15_000;
pub const DESKTOP_AUTO_UPDATE_POLL_INTERVAL_MS: u64 = 240_000;
pub const DESKTOP_UPDATE_LISTENER_EVENTS: &[&str] = &[
    "checking-for-update",
    "update-available",
    "update-not-available",
    "error",
    "download-progress",
    "update-downloaded",
];
pub const DESKTOP_LOGIN_SHELL_ENV_NAMES: &[&str] = &[
    "PATH",
    "SSH_AUTH_SOCK",
    "HOMEBREW_PREFIX",
    "HOMEBREW_CELLAR",
    "HOMEBREW_REPOSITORY",
    "XDG_CONFIG_HOME",
    "XDG_DATA_HOME",
];
pub const DESKTOP_WINDOWS_PROFILE_ENV_NAMES: &[&str] = &["PATH", "FNM_DIR", "FNM_MULTISHELL_PATH"];
pub const DESKTOP_WINDOWS_SHELL_CANDIDATES: &[&str] = &["pwsh.exe", "powershell.exe"];
pub const DESKTOP_LOGIN_SHELL_TIMEOUT_MS: u64 = 5_000;
pub const DESKTOP_LAUNCHCTL_TIMEOUT_MS: u64 = 2_000;
pub const DESKTOP_PROCESS_TERMINATE_GRACE_MS: u64 = 1_000;
pub const DESKTOP_SCHEME: &str = "t3";
pub const ELECTRON_PROTOCOL_REGISTRATION_ERROR_MESSAGE: &str =
    "Failed to register t3: file protocol.";
pub const ELECTRON_PROTOCOL_STATIC_BUNDLE_MISSING_MESSAGE: &str =
    "Desktop static bundle missing. Build apps/server (with bundled client) first.";
pub const ELECTRON_SAFE_STORAGE_AVAILABILITY_ERROR_MESSAGE: &str =
    "Electron safe storage failed to check encryption availability.";
pub const ELECTRON_SAFE_STORAGE_ENCRYPT_ERROR_MESSAGE: &str =
    "Electron safe storage failed to encrypt a string.";
pub const ELECTRON_SAFE_STORAGE_DECRYPT_ERROR_MESSAGE: &str =
    "Electron safe storage failed to decrypt a string.";
pub const ELECTRON_UPDATER_CHECK_FOR_UPDATES_ERROR_MESSAGE: &str =
    "Electron updater failed to check for updates.";
pub const ELECTRON_UPDATER_DOWNLOAD_UPDATE_ERROR_MESSAGE: &str =
    "Electron updater failed to download the update.";
pub const ELECTRON_UPDATER_QUIT_AND_INSTALL_ERROR_MESSAGE: &str =
    "Electron updater failed to quit and install the update.";
pub const ELECTRON_WINDOW_CREATE_ERROR_MESSAGE: &str = "Failed to create Electron BrowserWindow.";

pub const DESKTOP_CORE_ENDPOINT_PROVIDER: DesktopAdvertisedEndpointProvider =
    DesktopAdvertisedEndpointProvider {
        id: "desktop-core",
        label: "Desktop",
        kind: "core",
        is_addon: false,
    };

pub const DESKTOP_MANUAL_ENDPOINT_PROVIDER: DesktopAdvertisedEndpointProvider =
    DesktopAdvertisedEndpointProvider {
        id: "manual",
        label: "Manual",
        kind: "manual",
        is_addon: false,
    };

pub const TAILSCALE_ENDPOINT_PROVIDER: DesktopAdvertisedEndpointProvider =
    DesktopAdvertisedEndpointProvider {
        id: "tailscale",
        label: "Tailscale",
        kind: "private-network",
        is_addon: true,
    };

pub fn is_nightly_desktop_version(version: &str) -> bool {
    let Some((_, suffix)) = version.rsplit_once("-nightly.") else {
        return false;
    };
    let mut parts = suffix.split('.');
    let Some(date) = parts.next() else {
        return false;
    };
    let Some(build) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && date.len() == 8
        && date.chars().all(|character| character.is_ascii_digit())
        && !build.is_empty()
        && build.chars().all(|character| character.is_ascii_digit())
}

pub fn default_desktop_settings() -> DesktopSettings {
    DesktopSettings {
        server_exposure_mode: DesktopServerExposureMode::LocalOnly,
        tailscale_serve_enabled: false,
        tailscale_serve_port: DEFAULT_TAILSCALE_SERVE_PORT,
        update_channel: DesktopUpdateChannel::Latest,
        update_channel_configured_by_user: false,
    }
}

pub fn resolve_default_desktop_settings(app_version: &str) -> DesktopSettings {
    DesktopSettings {
        update_channel: resolve_default_desktop_update_channel(app_version),
        ..default_desktop_settings()
    }
}

pub fn normalize_tailscale_serve_port(value: Option<i64>) -> u16 {
    match value {
        Some(value) if (1..=65_535).contains(&value) => value as u16,
        _ => DEFAULT_TAILSCALE_SERVE_PORT,
    }
}

pub fn normalize_desktop_settings_document(
    parsed: &DesktopSettingsDocument,
    app_version: &str,
) -> DesktopSettings {
    let default_settings = resolve_default_desktop_settings(app_version);
    let is_legacy_settings = parsed.update_channel_configured_by_user.is_none();
    let update_channel_configured_by_user = parsed.update_channel_configured_by_user == Some(true)
        || (is_legacy_settings && parsed.update_channel == Some(DesktopUpdateChannel::Nightly));

    DesktopSettings {
        server_exposure_mode: match parsed.server_exposure_mode {
            Some(DesktopServerExposureMode::NetworkAccessible) => {
                DesktopServerExposureMode::NetworkAccessible
            }
            _ => DesktopServerExposureMode::LocalOnly,
        },
        tailscale_serve_enabled: parsed.tailscale_serve_enabled == Some(true),
        tailscale_serve_port: normalize_tailscale_serve_port(parsed.tailscale_serve_port),
        update_channel: if update_channel_configured_by_user {
            parsed
                .update_channel
                .unwrap_or(default_settings.update_channel)
        } else {
            default_settings.update_channel
        },
        update_channel_configured_by_user,
    }
}

pub fn normalize_optional_desktop_host(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn is_usable_lan_ipv4_address(address: &str) -> bool {
    !address.starts_with("127.") && !address.starts_with("169.254.")
}

pub fn resolve_lan_advertised_host(
    network_interfaces: &BTreeMap<String, Vec<DesktopNetworkInterfaceInfo>>,
    explicit_host: Option<&str>,
) -> Option<String> {
    if let Some(host) = normalize_optional_desktop_host(explicit_host) {
        return Some(host);
    }

    network_interfaces.values().find_map(|addresses| {
        addresses.iter().find_map(|address| {
            (!address.internal
                && address.family == "IPv4"
                && is_usable_lan_ipv4_address(&address.address))
            .then(|| address.address.clone())
        })
    })
}

pub fn resolve_desktop_server_exposure(
    mode: DesktopServerExposureMode,
    port: u16,
    network_interfaces: &BTreeMap<String, Vec<DesktopNetworkInterfaceInfo>>,
    advertised_host_override: Option<&str>,
) -> ResolvedDesktopServerExposure {
    let local_http_url = format!("http://{DESKTOP_LOOPBACK_HOST}:{port}");
    let local_ws_url = format!("ws://{DESKTOP_LOOPBACK_HOST}:{port}");

    if mode == DesktopServerExposureMode::LocalOnly {
        return ResolvedDesktopServerExposure {
            mode,
            bind_host: DESKTOP_LOOPBACK_HOST.to_string(),
            local_http_url,
            local_ws_url,
            endpoint_url: None,
            advertised_host: None,
        };
    }

    let advertised_host = resolve_lan_advertised_host(network_interfaces, advertised_host_override);
    ResolvedDesktopServerExposure {
        mode,
        bind_host: DESKTOP_LAN_BIND_HOST.to_string(),
        local_http_url,
        local_ws_url,
        endpoint_url: advertised_host
            .as_ref()
            .map(|host| format!("http://{host}:{port}")),
        advertised_host,
    }
}

pub fn desktop_server_exposure_runtime_state_from_resolved(
    requested_mode: DesktopServerExposureMode,
    settings: &DesktopSettings,
    exposure: ResolvedDesktopServerExposure,
    port: u16,
) -> DesktopServerExposureRuntimeState {
    DesktopServerExposureRuntimeState {
        requested_mode,
        mode: exposure.mode,
        port,
        bind_host: exposure.bind_host,
        local_http_url: exposure.local_http_url.clone(),
        local_ws_url: exposure.local_ws_url,
        http_base_url: exposure.local_http_url,
        endpoint_url: exposure.endpoint_url,
        advertised_host: exposure.advertised_host,
        tailscale_serve_enabled: settings.tailscale_serve_enabled,
        tailscale_serve_port: settings.tailscale_serve_port,
    }
}

pub fn initial_desktop_server_exposure_runtime_state() -> DesktopServerExposureRuntimeState {
    let settings = default_desktop_settings();
    let network_interfaces = BTreeMap::new();
    desktop_server_exposure_runtime_state_from_resolved(
        settings.server_exposure_mode,
        &settings,
        resolve_desktop_server_exposure(
            settings.server_exposure_mode,
            0,
            &network_interfaces,
            None,
        ),
        0,
    )
}

pub fn desktop_server_exposure_contract_state(
    state: &DesktopServerExposureRuntimeState,
) -> DesktopServerExposureState {
    DesktopServerExposureState {
        mode: state.mode,
        endpoint_url: state.endpoint_url.clone(),
        advertised_host: state.advertised_host.clone(),
        tailscale_serve_enabled: state.tailscale_serve_enabled,
        tailscale_serve_port: state.tailscale_serve_port,
    }
}

pub fn desktop_server_exposure_backend_config(
    state: &DesktopServerExposureRuntimeState,
) -> DesktopServerExposureBackendConfig {
    DesktopServerExposureBackendConfig {
        port: state.port,
        bind_host: state.bind_host.clone(),
        http_base_url: state.http_base_url.clone(),
        tailscale_serve_enabled: state.tailscale_serve_enabled,
        tailscale_serve_port: state.tailscale_serve_port,
    }
}

pub fn resolved_exposure_from_runtime_state(
    state: &DesktopServerExposureRuntimeState,
) -> ResolvedDesktopServerExposure {
    ResolvedDesktopServerExposure {
        mode: state.mode,
        bind_host: state.bind_host.clone(),
        local_http_url: state.local_http_url.clone(),
        local_ws_url: state.local_ws_url.clone(),
        endpoint_url: state.endpoint_url.clone(),
        advertised_host: state.advertised_host.clone(),
    }
}

pub fn resolve_desktop_server_exposure_runtime_state(
    requested_mode: DesktopServerExposureMode,
    settings: &DesktopSettings,
    port: u16,
    network_interfaces: &BTreeMap<String, Vec<DesktopNetworkInterfaceInfo>>,
    advertised_host_override: Option<&str>,
) -> ResolvedDesktopServerExposureRuntimeState {
    let requested_exposure = resolve_desktop_server_exposure(
        requested_mode,
        port,
        network_interfaces,
        advertised_host_override,
    );
    let unavailable = requested_mode == DesktopServerExposureMode::NetworkAccessible
        && requested_exposure.endpoint_url.is_none();
    let exposure = if unavailable {
        resolve_desktop_server_exposure(
            DesktopServerExposureMode::LocalOnly,
            port,
            network_interfaces,
            advertised_host_override,
        )
    } else {
        requested_exposure
    };

    ResolvedDesktopServerExposureRuntimeState {
        state: desktop_server_exposure_runtime_state_from_resolved(
            requested_mode,
            settings,
            exposure,
            port,
        ),
        unavailable,
    }
}

pub fn requires_desktop_backend_relaunch(
    previous: &DesktopServerExposureRuntimeState,
    next: &DesktopServerExposureRuntimeState,
) -> bool {
    previous.port != next.port
        || previous.bind_host != next.bind_host
        || previous.local_http_url != next.local_http_url
}

pub fn set_desktop_server_exposure_runtime_mode(
    previous: &DesktopServerExposureRuntimeState,
    current_settings: &DesktopSettings,
    mode: DesktopServerExposureMode,
    network_interfaces: &BTreeMap<String, Vec<DesktopNetworkInterfaceInfo>>,
    advertised_host_override: Option<&str>,
) -> Result<DesktopServerExposureChange, u16> {
    let settings_change = set_desktop_server_exposure_mode(current_settings, mode);
    let resolved = resolve_desktop_server_exposure_runtime_state(
        mode,
        &settings_change.settings,
        previous.port,
        network_interfaces,
        advertised_host_override,
    );

    if resolved.unavailable {
        return Err(previous.port);
    }

    Ok(DesktopServerExposureChange {
        requires_relaunch: settings_change.changed
            || requires_desktop_backend_relaunch(previous, &resolved.state),
        state: desktop_server_exposure_contract_state(&resolved.state),
    })
}

pub fn set_desktop_runtime_tailscale_serve_enabled(
    current: &DesktopServerExposureRuntimeState,
    settings: &DesktopSettings,
    enabled: bool,
    port: Option<i64>,
) -> DesktopServerExposureChange {
    let settings_change = set_desktop_tailscale_serve(settings, enabled, port);
    let next = DesktopServerExposureRuntimeState {
        tailscale_serve_enabled: settings_change.settings.tailscale_serve_enabled,
        tailscale_serve_port: settings_change.settings.tailscale_serve_port,
        ..current.clone()
    };
    DesktopServerExposureChange {
        state: desktop_server_exposure_contract_state(&next),
        requires_relaunch: settings_change.changed,
    }
}

pub fn normalize_desktop_endpoint_http_base_url(raw_value: &str) -> Option<String> {
    let raw_value = raw_value.trim();
    let scheme_end = raw_value.find("://")?;
    let raw_scheme = raw_value[..scheme_end].to_ascii_lowercase();
    let scheme = match raw_scheme.as_str() {
        "http" | "ws" => "http",
        "https" | "wss" => "https",
        _ => return None,
    };
    let after_scheme = &raw_value[scheme_end + 3..];
    let authority_end = after_scheme
        .find(|ch| matches!(ch, '/' | '?' | '#'))
        .unwrap_or(after_scheme.len());
    let authority = after_scheme[..authority_end].trim();
    if authority.is_empty() {
        return None;
    }
    Some(format!("{scheme}://{authority}/"))
}

pub fn derive_desktop_endpoint_ws_base_url(http_base_url: &str) -> Option<String> {
    let http_base_url = normalize_desktop_endpoint_http_base_url(http_base_url)?;
    if let Some(rest) = http_base_url.strip_prefix("https://") {
        Some(format!("wss://{rest}"))
    } else {
        http_base_url
            .strip_prefix("http://")
            .map(|rest| format!("ws://{rest}"))
    }
}

pub fn classify_desktop_endpoint_hosted_https_compatibility(
    http_base_url: &str,
    fallback: &'static str,
) -> Option<&'static str> {
    let normalized = normalize_desktop_endpoint_http_base_url(http_base_url)?;
    if normalized.starts_with("http://") {
        return Some("mixed-content-blocked");
    }
    Some(if fallback == "mixed-content-blocked" {
        "unknown"
    } else {
        fallback
    })
}

fn desktop_endpoint(
    id: String,
    label: &'static str,
    provider: DesktopAdvertisedEndpointProvider,
    http_base_url: &str,
    reachability: &'static str,
    hosted_https_compatibility: Option<&'static str>,
    source: &'static str,
    status: &'static str,
    is_default: Option<bool>,
    description: Option<&'static str>,
) -> Option<DesktopAdvertisedEndpoint> {
    let http_base_url = normalize_desktop_endpoint_http_base_url(http_base_url)?;
    let hosted_https_app = hosted_https_compatibility
        .or_else(|| classify_desktop_endpoint_hosted_https_compatibility(&http_base_url, "unknown"))
        .unwrap_or("unknown");
    Some(DesktopAdvertisedEndpoint {
        id,
        label,
        provider,
        ws_base_url: derive_desktop_endpoint_ws_base_url(&http_base_url)?,
        http_base_url,
        reachability,
        compatibility: DesktopAdvertisedEndpointCompatibility {
            hosted_https_app,
            desktop_app: "compatible",
        },
        source,
        status,
        is_default,
        description,
    })
}

pub fn resolve_desktop_core_advertised_endpoints(
    port: u16,
    exposure: &ResolvedDesktopServerExposure,
    custom_endpoint_urls: &[String],
) -> Vec<DesktopAdvertisedEndpoint> {
    let mut endpoints = vec![
        desktop_endpoint(
            format!("desktop-loopback:{port}"),
            "This machine",
            DESKTOP_CORE_ENDPOINT_PROVIDER,
            &exposure.local_http_url,
            "loopback",
            None,
            "desktop-core",
            "available",
            None,
            Some("Loopback endpoint for this desktop app."),
        )
        .expect("loopback endpoint should always be valid"),
    ];

    if let Some(endpoint_url) = &exposure.endpoint_url {
        if let Some(endpoint) = desktop_endpoint(
            format!("desktop-lan:{endpoint_url}"),
            "Local network",
            DESKTOP_CORE_ENDPOINT_PROVIDER,
            endpoint_url,
            "lan",
            None,
            "desktop-core",
            "available",
            Some(true),
            Some("Reachable from devices on the same network."),
        ) {
            endpoints.push(endpoint);
        }
    }

    for custom_endpoint_url in custom_endpoint_urls {
        let Some(normalized) = normalize_desktop_endpoint_http_base_url(custom_endpoint_url) else {
            continue;
        };
        let is_https = normalized.starts_with("https://");
        if let Some(endpoint) = desktop_endpoint(
            format!("manual:{custom_endpoint_url}"),
            if is_https {
                "Custom HTTPS"
            } else {
                "Custom endpoint"
            },
            DESKTOP_MANUAL_ENDPOINT_PROVIDER,
            custom_endpoint_url,
            "public",
            is_https.then_some("compatible"),
            "user",
            "unknown",
            None,
            Some(if is_https {
                "User-configured HTTPS endpoint for this desktop backend."
            } else {
                "User-configured endpoint for this desktop backend."
            }),
        ) {
            endpoints.push(endpoint);
        }
    }

    endpoints
}

pub fn is_tailscale_ipv4_address(address: &str) -> bool {
    let parts = address.split('.').collect::<Vec<_>>();
    if parts.len() != 4 {
        return false;
    }
    let octets = parts
        .iter()
        .map(|part| part.parse::<u16>())
        .collect::<Result<Vec<_>, _>>();
    let Ok(octets) = octets else {
        return false;
    };
    octets.iter().all(|octet| *octet <= 255) && octets[0] == 100 && (64..=127).contains(&octets[1])
}

fn normalize_tailscale_magic_dns_name(status_json: &Value) -> Option<String> {
    let dns_name = status_json
        .get("Self")
        .and_then(Value::as_object)
        .and_then(|self_status| self_status.get("DNSName"))
        .and_then(Value::as_str)?;
    let normalized = dns_name.trim().trim_end_matches('.');
    (!normalized.is_empty()).then(|| normalized.to_string())
}

pub fn parse_tailscale_magic_dns_name(
    raw_status_json: &str,
) -> Result<Option<String>, serde_json::Error> {
    serde_json::from_str::<Value>(raw_status_json)
        .map(|value| normalize_tailscale_magic_dns_name(&value))
}

pub fn parse_tailscale_status(raw_status_json: &str) -> Result<TailscaleStatus, serde_json::Error> {
    let value = serde_json::from_str::<Value>(raw_status_json)?;
    let tailnet_ipv4_addresses = value
        .get("Self")
        .and_then(Value::as_object)
        .and_then(|self_status| self_status.get("TailscaleIPs"))
        .and_then(Value::as_array)
        .map(|addresses| {
            addresses
                .iter()
                .filter_map(Value::as_str)
                .filter(|address| is_tailscale_ipv4_address(address))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(TailscaleStatus {
        magic_dns_name: normalize_tailscale_magic_dns_name(&value),
        tailnet_ipv4_addresses,
    })
}

pub fn build_tailscale_https_base_url(magic_dns_name: &str, serve_port: Option<u16>) -> String {
    let serve_port = serve_port.unwrap_or(DEFAULT_TAILSCALE_SERVE_PORT);
    if serve_port == DEFAULT_TAILSCALE_SERVE_PORT {
        format!("https://{magic_dns_name}/")
    } else {
        format!("https://{magic_dns_name}:{serve_port}/")
    }
}

pub fn tailscale_status_command_spec() -> TailscaleCommandSpec {
    TailscaleCommandSpec {
        command: "tailscale",
        args: vec!["status".to_string(), "--json".to_string()],
        shell_on_windows: true,
        timeout_ms: TAILSCALE_STATUS_TIMEOUT_MS,
    }
}

pub fn ensure_tailscale_serve_command_spec(
    local_port: u16,
    serve_port: Option<u16>,
    local_host: Option<&str>,
) -> TailscaleCommandSpec {
    let serve_port = serve_port.unwrap_or(DEFAULT_TAILSCALE_SERVE_PORT);
    let local_host = local_host.unwrap_or(DESKTOP_LOOPBACK_HOST);
    TailscaleCommandSpec {
        command: "tailscale",
        args: vec![
            "serve".to_string(),
            "--bg".to_string(),
            format!("--https={serve_port}"),
            format!("http://{local_host}:{local_port}"),
        ],
        shell_on_windows: true,
        timeout_ms: TAILSCALE_SERVE_TIMEOUT_MS,
    }
}

pub fn disable_tailscale_serve_command_spec(serve_port: Option<u16>) -> TailscaleCommandSpec {
    let serve_port = serve_port.unwrap_or(DEFAULT_TAILSCALE_SERVE_PORT);
    TailscaleCommandSpec {
        command: "tailscale",
        args: vec![
            "serve".to_string(),
            format!("--https={serve_port}"),
            "off".to_string(),
        ],
        shell_on_windows: true,
        timeout_ms: TAILSCALE_SERVE_TIMEOUT_MS,
    }
}

pub fn resolve_tailscale_ip_advertised_endpoints(
    port: u16,
    network_interfaces: &BTreeMap<String, Vec<DesktopNetworkInterfaceInfo>>,
) -> Vec<DesktopAdvertisedEndpoint> {
    let mut seen = Vec::<String>::new();
    let mut endpoints = Vec::new();

    for addresses in network_interfaces.values() {
        for address in addresses {
            if address.internal
                || address.family != "IPv4"
                || !is_tailscale_ipv4_address(&address.address)
                || seen.contains(&address.address)
            {
                continue;
            }
            seen.push(address.address.clone());
            if let Some(endpoint) = desktop_endpoint(
                format!("tailscale-ip:http://{}:{port}", address.address),
                "Tailscale IP",
                TAILSCALE_ENDPOINT_PROVIDER,
                &format!("http://{}:{port}", address.address),
                "private-network",
                None,
                "desktop-addon",
                "available",
                None,
                Some("Reachable from devices on the same Tailnet."),
            ) {
                endpoints.push(endpoint);
            }
        }
    }

    endpoints
}

pub fn resolve_tailscale_magic_dns_advertised_endpoint(
    dns_name: Option<&str>,
    serve_enabled: bool,
    serve_port: Option<u16>,
    probe_reachable: Option<bool>,
) -> Option<DesktopAdvertisedEndpoint> {
    let dns_name = dns_name?;
    let http_base_url = build_tailscale_https_base_url(dns_name, serve_port);
    let is_reachable = serve_enabled && probe_reachable.unwrap_or(false);
    desktop_endpoint(
        format!("tailscale-magicdns:{http_base_url}"),
        "Tailscale HTTPS",
        TAILSCALE_ENDPOINT_PROVIDER,
        &http_base_url,
        "private-network",
        Some(if is_reachable {
            "compatible"
        } else {
            "requires-configuration"
        }),
        "desktop-addon",
        if is_reachable {
            "available"
        } else {
            "unavailable"
        },
        None,
        Some(if is_reachable {
            "HTTPS endpoint served by Tailscale Serve."
        } else {
            "MagicDNS hostname. Configure Tailscale Serve for HTTPS access."
        }),
    )
}

pub fn resolve_tailscale_advertised_endpoints_from_status(
    port: u16,
    serve_enabled: bool,
    serve_port: Option<u16>,
    network_interfaces: &BTreeMap<String, Vec<DesktopNetworkInterfaceInfo>>,
    status_json: Option<&str>,
    probe_reachable: Option<bool>,
) -> Result<Vec<DesktopAdvertisedEndpoint>, serde_json::Error> {
    let mut endpoints = resolve_tailscale_ip_advertised_endpoints(port, network_interfaces);
    let dns_name = status_json
        .map(parse_tailscale_magic_dns_name)
        .transpose()?
        .flatten();
    if let Some(endpoint) = resolve_tailscale_magic_dns_advertised_endpoint(
        dns_name.as_deref(),
        serve_enabled,
        serve_port,
        probe_reachable,
    ) {
        endpoints.push(endpoint);
    }
    Ok(endpoints)
}

pub fn desktop_backend_readiness_url(http_base_url: &str) -> Option<String> {
    let base = normalize_desktop_endpoint_http_base_url(http_base_url)?;
    Some(format!(
        "{}{}",
        base.trim_end_matches('/'),
        DESKTOP_BACKEND_READINESS_PATH
    ))
}

pub fn desktop_backend_restart_delay_ms(attempt: u32) -> u64 {
    let multiplier = 2_u64.saturating_pow(attempt);
    DESKTOP_BACKEND_INITIAL_RESTART_DELAY_MS
        .saturating_mul(multiplier)
        .min(DESKTOP_BACKEND_MAX_RESTART_DELAY_MS)
}

pub fn default_desktop_state_flags() -> DesktopStateFlags {
    DesktopStateFlags {
        backend_ready: false,
        quitting: false,
    }
}

pub fn desktop_component_log_annotation(
    component: &str,
    annotations: BTreeMap<String, Value>,
) -> DesktopComponentLogAnnotation {
    let mut merged = BTreeMap::from([(
        "component".to_string(),
        Value::String(component.to_string()),
    )]);
    merged.extend(annotations);
    DesktopComponentLogAnnotation {
        component: component.to_string(),
        annotations: merged,
    }
}

pub fn sanitize_desktop_log_value(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn desktop_backend_child_log_record(
    message: String,
    level: &'static str,
    timestamp: &str,
    annotations: BTreeMap<String, Value>,
) -> DesktopBackendChildLogRecord {
    DesktopBackendChildLogRecord {
        message,
        level,
        timestamp: timestamp.to_string(),
        annotations,
        spans: BTreeMap::new(),
        fiber_id: DESKTOP_BACKEND_CHILD_LOG_FIBER_ID,
    }
}

pub fn desktop_backend_child_session_boundary_log_record(
    phase: DesktopBackendChildLogPhase,
    details: &str,
    run_id: Option<&str>,
    timestamp: &str,
) -> DesktopBackendChildLogRecord {
    desktop_backend_child_log_record(
        format!("backend child process session {}", phase.message_suffix()),
        "INFO",
        timestamp,
        BTreeMap::from([
            (
                "component".to_string(),
                Value::String("desktop-backend-child".to_string()),
            ),
            (
                "runId".to_string(),
                Value::String(run_id.unwrap_or("unknown").to_string()),
            ),
            (
                "phase".to_string(),
                Value::String(phase.as_str().to_string()),
            ),
            (
                "details".to_string(),
                Value::String(sanitize_desktop_log_value(details)),
            ),
        ]),
    )
}

pub fn desktop_backend_child_output_log_record(
    stream_name: DesktopBackendOutputStream,
    text: &str,
    run_id: Option<&str>,
    timestamp: &str,
) -> DesktopBackendChildLogRecord {
    desktop_backend_child_log_record(
        "backend child process output".to_string(),
        stream_name.level(),
        timestamp,
        BTreeMap::from([
            (
                "component".to_string(),
                Value::String("desktop-backend-child".to_string()),
            ),
            (
                "runId".to_string(),
                Value::String(run_id.unwrap_or("unknown").to_string()),
            ),
            (
                "stream".to_string(),
                Value::String(stream_name.as_str().to_string()),
            ),
            ("text".to_string(), Value::String(text.to_string())),
        ]),
    )
}

pub fn desktop_rotating_log_file_writer_config(
    file_path: &str,
    max_bytes: Option<i64>,
    max_files: Option<i64>,
) -> Result<DesktopRotatingLogFileWriterConfig, DesktopLogFileWriterConfigurationError> {
    let max_bytes = max_bytes.unwrap_or(DESKTOP_LOG_FILE_MAX_BYTES as i64);
    if max_bytes < 1 {
        return Err(DesktopLogFileWriterConfigurationError {
            option: "maxBytes",
            value: max_bytes,
            message: format!("maxBytes must be >= 1 (received {max_bytes})"),
        });
    }
    let max_files = max_files.unwrap_or(DESKTOP_LOG_FILE_MAX_FILES as i64);
    if max_files < 1 {
        return Err(DesktopLogFileWriterConfigurationError {
            option: "maxFiles",
            value: max_files,
            message: format!("maxFiles must be >= 1 (received {max_files})"),
        });
    }

    Ok(DesktopRotatingLogFileWriterConfig {
        file_path: file_path.to_string(),
        max_bytes: max_bytes as u64,
        max_files: max_files as u16,
    })
}

pub fn desktop_rotating_log_backup_path(file_path: &str, index: u16) -> String {
    format!("{file_path}.{index}")
}

pub fn desktop_rotating_log_rotation_order(
    file_path: &str,
    max_files: u16,
) -> Vec<(String, String)> {
    if max_files <= 1 {
        return Vec::new();
    }
    (1..max_files)
        .rev()
        .map(|index| {
            (
                desktop_rotating_log_backup_path(file_path, index),
                desktop_rotating_log_backup_path(file_path, index + 1),
            )
        })
        .collect()
}

pub fn resolve_desktop_backend_port<F>(
    configured_port: Option<u16>,
    can_listen_on_host: F,
) -> Result<DesktopBackendPortSelection, DesktopBackendPortUnavailableError>
where
    F: Fn(u16, &str) -> bool,
{
    if let Some(port) = configured_port {
        return Ok(DesktopBackendPortSelection {
            port,
            selected_by_scan: false,
        });
    }

    for port in DEFAULT_DESKTOP_BACKEND_PORT..=MAX_DESKTOP_TCP_PORT {
        if DESKTOP_BACKEND_PORT_PROBE_HOSTS
            .iter()
            .all(|host| can_listen_on_host(port, host))
        {
            return Ok(DesktopBackendPortSelection {
                port,
                selected_by_scan: true,
            });
        }
    }

    Err(DesktopBackendPortUnavailableError {
        start_port: DEFAULT_DESKTOP_BACKEND_PORT,
        max_port: MAX_DESKTOP_TCP_PORT,
        hosts: DESKTOP_BACKEND_PORT_PROBE_HOSTS.to_vec(),
        message: format!(
            "No desktop backend port is available on hosts {} between {} and {}.",
            DESKTOP_BACKEND_PORT_PROBE_HOSTS.join(", "),
            DEFAULT_DESKTOP_BACKEND_PORT,
            MAX_DESKTOP_TCP_PORT
        ),
    })
}

pub fn validate_desktop_development_backend_port(
    is_development: bool,
    configured_port: Option<u16>,
) -> Result<(), DesktopDevelopmentBackendPortRequiredError> {
    if is_development && configured_port.is_none() {
        Err(DesktopDevelopmentBackendPortRequiredError {
            message: "T3CODE_PORT is required in desktop development.",
        })
    } else {
        Ok(())
    }
}

pub fn desktop_fatal_startup_error_plan(
    stage: &str,
    message: &str,
    stack: Option<&str>,
    was_quitting: bool,
) -> DesktopFatalStartupErrorPlan {
    let detail = stack
        .filter(|stack| !stack.is_empty())
        .map(|stack| format!("\n{stack}"))
        .unwrap_or_default();
    DesktopFatalStartupErrorPlan {
        log_stage: stage.to_string(),
        log_message: message.to_string(),
        dialog_title: "T3 Code failed to start",
        dialog_body: format!("Stage: {stage}\n{message}{detail}"),
        should_show_dialog: !was_quitting,
        request_shutdown: true,
        quit_app: true,
    }
}

pub fn default_desktop_shutdown_flags() -> DesktopShutdownFlags {
    DesktopShutdownFlags {
        requested: false,
        complete: false,
    }
}

pub fn desktop_shutdown_request(flags: &DesktopShutdownFlags) -> DesktopShutdownFlags {
    DesktopShutdownFlags {
        requested: true,
        complete: flags.complete,
    }
}

pub fn desktop_shutdown_mark_complete(flags: &DesktopShutdownFlags) -> DesktopShutdownFlags {
    DesktopShutdownFlags {
        requested: flags.requested,
        complete: true,
    }
}

pub fn desktop_before_quit_plan(quit_allowed: bool) -> DesktopBeforeQuitPlan {
    if quit_allowed {
        DesktopBeforeQuitPlan {
            prevent_default: false,
            set_quitting: true,
            request_shutdown: false,
            await_complete: false,
            mark_quit_allowed: false,
            quit_after_shutdown: false,
        }
    } else {
        DesktopBeforeQuitPlan {
            prevent_default: true,
            set_quitting: true,
            request_shutdown: true,
            await_complete: true,
            mark_quit_allowed: true,
            quit_after_shutdown: true,
        }
    }
}

pub fn desktop_signal_quit_plan(was_quitting: bool) -> DesktopSignalQuitPlan {
    DesktopSignalQuitPlan {
        set_quitting: !was_quitting,
        request_shutdown: !was_quitting,
        await_complete: !was_quitting,
        quit_app: !was_quitting,
    }
}

pub fn desktop_window_all_closed_should_quit(platform: &str, quitting: bool) -> bool {
    platform != "darwin" && !quitting
}

pub fn desktop_relaunch_plan(is_development: bool, reason: &str) -> DesktopRelaunchPlan {
    DesktopRelaunchPlan {
        reason: reason.to_string(),
        set_quitting: true,
        request_shutdown: true,
        await_complete: true,
        relaunch: !is_development,
        exit_code: if is_development {
            DESKTOP_RELAUNCH_DEVELOPMENT_EXIT_CODE
        } else {
            DESKTOP_RELAUNCH_PACKAGED_EXIT_CODE
        },
    }
}

pub fn desktop_main_layer_order() -> Vec<&'static str> {
    vec![
        "desktopEnvironmentLayer",
        "electronLayer",
        "desktopFoundationLayer",
        "desktopSshLayer",
        "desktopServerExposureLayer",
        "desktopWindowLayer",
        "desktopBackendLayer",
        "desktopApplicationLayer",
        "desktopRuntimeLayer",
        "DesktopApp.program",
    ]
}

pub fn desktop_resource_path_candidates(
    environment: &DesktopEnvironment,
    file_name: &str,
) -> Vec<String> {
    vec![
        join_path(&[&environment.dirname, "..", "resources", file_name]),
        join_path(&[&environment.dirname, "..", "prod-resources", file_name]),
        join_path(&[&environment.resources_path, "resources", file_name]),
        join_path(&[&environment.resources_path, file_name]),
    ]
}

pub fn resolve_desktop_resource_path<F>(
    environment: &DesktopEnvironment,
    file_name: &str,
    exists: F,
) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    desktop_resource_path_candidates(environment, file_name)
        .into_iter()
        .find(|candidate| exists(candidate))
}

pub fn resolve_desktop_icon_path<F>(
    environment: &DesktopEnvironment,
    ext: &str,
    process_platform: &str,
    exists: F,
) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    if environment.is_development && process_platform == "darwin" && ext == "png" {
        let development_dock_icon_path = environment.development_dock_icon_path.clone();
        if exists(&development_dock_icon_path) {
            return Some(development_dock_icon_path);
        }
    }
    resolve_desktop_resource_path(environment, &format!("icon.{ext}"), exists)
}

pub fn resolve_desktop_icon_paths<F>(
    environment: &DesktopEnvironment,
    process_platform: &str,
    exists: F,
) -> DesktopIconPaths
where
    F: Fn(&str) -> bool + Copy,
{
    DesktopIconPaths {
        ico: resolve_desktop_icon_path(environment, "ico", process_platform, exists),
        icns: resolve_desktop_icon_path(environment, "icns", process_platform, exists),
        png: resolve_desktop_icon_path(environment, "png", process_platform, exists),
    }
}

pub fn electron_app_metadata(
    app_version: &str,
    app_path: &str,
    is_packaged: bool,
    resources_path: &str,
    running_under_arm64_translation: bool,
) -> ElectronAppMetadata {
    ElectronAppMetadata {
        app_version: app_version.to_string(),
        app_path: app_path.to_string(),
        is_packaged,
        resources_path: resources_path.to_string(),
        running_under_arm64_translation,
    }
}

pub fn electron_app_append_switch_command(
    switch_name: &str,
    value: Option<&str>,
) -> ElectronAppAppendSwitchCommand {
    ElectronAppAppendSwitchCommand {
        switch_name: switch_name.to_string(),
        value: value.map(str::to_string),
    }
}

pub fn electron_app_listener_spec(event_name: &str) -> ElectronScopedListenerSpec {
    ElectronScopedListenerSpec {
        event_name: event_name.to_string(),
        acquire_method: "app.on",
        release_method: "app.removeListener",
    }
}

pub fn electron_theme_listener_spec() -> ElectronScopedListenerSpec {
    ElectronScopedListenerSpec {
        event_name: "updated".to_string(),
        acquire_method: "nativeTheme.on",
        release_method: "nativeTheme.removeListener",
    }
}

pub fn electron_theme_source_change(theme: &str) -> ElectronThemeSourceChange {
    ElectronThemeSourceChange {
        theme: theme.to_string(),
        property: "nativeTheme.themeSource",
    }
}

pub fn electron_pick_folder_options(default_path: Option<&str>) -> ElectronOpenDialogOptions {
    ElectronOpenDialogOptions {
        properties: vec!["openDirectory", "createDirectory"],
        default_path: default_path.map(str::to_string),
    }
}

pub fn electron_confirm_dialog_options(message: &str) -> Option<ElectronConfirmDialogOptions> {
    let message = message.trim();
    if message.is_empty() {
        return None;
    }
    Some(ElectronConfirmDialogOptions {
        dialog_type: "question",
        buttons: vec!["No", "Yes"],
        default_id: 0,
        cancel_id: 0,
        no_link: true,
        message: message.to_string(),
    })
}

pub fn electron_confirm_response_is_yes(response: i64) -> bool {
    const CONFIRM_BUTTON_INDEX: i64 = 1;
    response == CONFIRM_BUTTON_INDEX
}

pub fn parse_safe_external_url(raw_url: Option<&str>) -> Option<String> {
    let raw_url = raw_url?.trim();
    let scheme_end = raw_url.find(':')?;
    let scheme = raw_url[..=scheme_end].to_ascii_lowercase();
    if scheme != "http:" && scheme != "https:" {
        return None;
    }
    let after_scheme = raw_url.get(scheme_end + 1..)?;
    if !after_scheme.starts_with("//") || after_scheme.len() <= 2 {
        return None;
    }
    Some(raw_url.to_string())
}

pub fn trim_non_empty_shell_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn desktop_shell_path_delimiter(platform: &str) -> &'static str {
    if platform == "win32" { ";" } else { ":" }
}

pub fn read_desktop_env_path(env: &BTreeMap<String, String>) -> Option<String> {
    ["PATH", "Path", "path"]
        .into_iter()
        .find_map(|name| trim_non_empty_shell_value(env.get(name).map(String::as_str)))
}

pub fn desktop_shell_path_comparison_key(entry: &str, platform: &str) -> String {
    let normalized = entry.trim().trim_matches('"').to_string();
    if platform == "win32" {
        normalized.to_lowercase()
    } else {
        normalized
    }
}

pub fn merge_desktop_shell_paths(platform: &str, values: &[Option<String>]) -> Option<String> {
    let delimiter = desktop_shell_path_delimiter(platform);
    let mut entries = Vec::<String>::new();
    let mut seen = Vec::<String>::new();

    for value in values.iter().flatten() {
        for entry in value.split(delimiter) {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            let key = desktop_shell_path_comparison_key(trimmed, platform);
            if key.is_empty() || seen.contains(&key) {
                continue;
            }
            seen.push(key);
            entries.push(trimmed.to_string());
        }
    }

    (!entries.is_empty()).then(|| entries.join(delimiter))
}

pub fn list_desktop_login_shell_candidates(
    platform: &str,
    env: &BTreeMap<String, String>,
    user_shell: Option<&str>,
) -> Vec<String> {
    let fallback = match platform {
        "darwin" => Some("/bin/zsh"),
        "linux" => Some("/bin/bash"),
        _ => None,
    };
    let candidates = [
        trim_non_empty_shell_value(env.get("SHELL").map(String::as_str)),
        trim_non_empty_shell_value(user_shell),
        trim_non_empty_shell_value(fallback),
    ];
    let mut seen = Vec::<String>::new();
    let mut output = Vec::<String>::new();
    for candidate in candidates.into_iter().flatten() {
        if seen.contains(&candidate) {
            continue;
        }
        seen.push(candidate.clone());
        output.push(candidate);
    }
    output
}

pub fn known_desktop_windows_cli_dirs(env: &BTreeMap<String, String>) -> Vec<String> {
    let mut dirs = Vec::new();
    if let Some(appdata) = trim_non_empty_shell_value(env.get("APPDATA").map(String::as_str)) {
        dirs.push(format!("{appdata}\\npm"));
    }
    if let Some(localappdata) =
        trim_non_empty_shell_value(env.get("LOCALAPPDATA").map(String::as_str))
    {
        dirs.push(format!("{localappdata}\\Programs\\nodejs"));
        dirs.push(format!("{localappdata}\\Volta\\bin"));
        dirs.push(format!("{localappdata}\\pnpm"));
    }
    if let Some(userprofile) =
        trim_non_empty_shell_value(env.get("USERPROFILE").map(String::as_str))
    {
        dirs.push(format!("{userprofile}\\.bun\\bin"));
        dirs.push(format!("{userprofile}\\scoop\\shims"));
    }
    dirs
}

pub fn desktop_shell_env_start_marker(name: &str) -> String {
    format!("__T3CODE_ENV_{name}_START__")
}

pub fn desktop_shell_env_end_marker(name: &str) -> String {
    format!("__T3CODE_ENV_{name}_END__")
}

pub fn capture_desktop_posix_environment_command(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| {
            [
                format!("printf '%s\\n' '{}'", desktop_shell_env_start_marker(name)),
                format!("printenv {name} || true"),
                format!("printf '%s\\n' '{}'", desktop_shell_env_end_marker(name)),
            ]
            .join("; ")
        })
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn capture_desktop_windows_environment_command(names: &[&str]) -> String {
    let mut parts = vec!["$ErrorActionPreference = 'Stop'".to_string()];
    for name in names {
        parts.push(format!(
            "Write-Output '{}'",
            desktop_shell_env_start_marker(name)
        ));
        parts.push(format!(
            "$value = [Environment]::GetEnvironmentVariable('{name}')"
        ));
        parts.push(
            "if ($null -ne $value -and $value.Length -gt 0) { Write-Output $value }".to_string(),
        );
        parts.push(format!(
            "Write-Output '{}'",
            desktop_shell_env_end_marker(name)
        ));
    }
    parts.join("; ")
}

pub fn extract_desktop_shell_environment(output: &str, names: &[&str]) -> BTreeMap<String, String> {
    let mut environment = BTreeMap::new();
    for name in names {
        let start_marker = desktop_shell_env_start_marker(name);
        let end_marker = desktop_shell_env_end_marker(name);
        let Some(start) = output.find(&start_marker) else {
            continue;
        };
        let value_start = start + start_marker.len();
        let Some(end_offset) = output[value_start..].find(&end_marker) else {
            continue;
        };
        let end = value_start + end_offset;
        let value = output[value_start..end]
            .trim_start_matches(&['\r', '\n'][..])
            .trim_end_matches(&['\r', '\n'][..]);
        if !value.is_empty() {
            environment.insert((*name).to_string(), value.to_string());
        }
    }
    environment
}

pub fn desktop_login_shell_command_spec(
    shell: &str,
    names: &[&str],
) -> Option<DesktopShellCommandSpec> {
    (!names.is_empty()).then(|| DesktopShellCommandSpec {
        command: shell.to_string(),
        args: vec![
            "-ilc".to_string(),
            capture_desktop_posix_environment_command(names),
        ],
        shell: false,
        timeout_ms: DESKTOP_LOGIN_SHELL_TIMEOUT_MS,
        terminate_grace_ms: DESKTOP_PROCESS_TERMINATE_GRACE_MS,
    })
}

pub fn desktop_launchctl_path_command_spec() -> DesktopShellCommandSpec {
    DesktopShellCommandSpec {
        command: "/bin/launchctl".to_string(),
        args: vec!["getenv".to_string(), "PATH".to_string()],
        shell: false,
        timeout_ms: DESKTOP_LAUNCHCTL_TIMEOUT_MS,
        terminate_grace_ms: DESKTOP_PROCESS_TERMINATE_GRACE_MS,
    }
}

pub fn desktop_windows_environment_command_specs(
    names: &[&str],
    load_profile: bool,
) -> Vec<DesktopShellCommandSpec> {
    if names.is_empty() {
        return Vec::new();
    }
    let mut args = vec!["-NoLogo".to_string()];
    if !load_profile {
        args.push("-NoProfile".to_string());
    }
    args.extend([
        "-NonInteractive".to_string(),
        "-Command".to_string(),
        capture_desktop_windows_environment_command(names),
    ]);

    DESKTOP_WINDOWS_SHELL_CANDIDATES
        .iter()
        .map(|command| DesktopShellCommandSpec {
            command: (*command).to_string(),
            args: args.clone(),
            shell: true,
            timeout_ms: DESKTOP_LOGIN_SHELL_TIMEOUT_MS,
            terminate_grace_ms: DESKTOP_PROCESS_TERMINATE_GRACE_MS,
        })
        .collect()
}

pub fn install_desktop_posix_environment_patch(
    platform: &str,
    current_env: &BTreeMap<String, String>,
    shell_environment: &BTreeMap<String, String>,
    launchctl_path: Option<String>,
) -> BTreeMap<String, String> {
    let mut patch = BTreeMap::new();
    let shell_path = trim_non_empty_shell_value(shell_environment.get("PATH").map(String::as_str))
        .or(launchctl_path);
    if let Some(path) =
        merge_desktop_shell_paths(platform, &[shell_path, read_desktop_env_path(current_env)])
    {
        patch.insert("PATH".to_string(), path);
    }
    if !current_env.contains_key("SSH_AUTH_SOCK") {
        if let Some(value) =
            trim_non_empty_shell_value(shell_environment.get("SSH_AUTH_SOCK").map(String::as_str))
        {
            patch.insert("SSH_AUTH_SOCK".to_string(), value);
        }
    }
    for name in [
        "HOMEBREW_PREFIX",
        "HOMEBREW_CELLAR",
        "HOMEBREW_REPOSITORY",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
    ] {
        if !current_env.contains_key(name) {
            if let Some(value) =
                trim_non_empty_shell_value(shell_environment.get(name).map(String::as_str))
            {
                patch.insert(name.to_string(), value);
            }
        }
    }
    patch
}

pub fn install_desktop_windows_environment_patch(
    current_env: &BTreeMap<String, String>,
    no_profile_environment: &BTreeMap<String, String>,
    profile_environment: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let known_cli_path = known_desktop_windows_cli_dirs(current_env).join(";");
    let mut patch = BTreeMap::new();
    if let Some(path) = merge_desktop_shell_paths(
        "win32",
        &[
            trim_non_empty_shell_value(profile_environment.get("PATH").map(String::as_str)),
            trim_non_empty_shell_value(Some(&known_cli_path)),
            trim_non_empty_shell_value(no_profile_environment.get("PATH").map(String::as_str)),
            read_desktop_env_path(current_env),
        ],
    ) {
        patch.insert("PATH".to_string(), path);
    }
    for name in ["FNM_DIR", "FNM_MULTISHELL_PATH"] {
        if !current_env.contains_key(name) {
            if let Some(value) =
                trim_non_empty_shell_value(profile_environment.get(name).map(String::as_str))
            {
                patch.insert(name.to_string(), value);
            }
        }
    }
    patch
}

pub fn desktop_scheme_privilege_spec() -> ElectronSchemePrivilegeSpec {
    ElectronSchemePrivilegeSpec {
        scheme: DESKTOP_SCHEME,
        standard: true,
        secure: true,
        support_fetch_api: true,
        cors_enabled: true,
    }
}

pub fn normalize_desktop_protocol_pathname(raw_path: &str) -> Option<String> {
    let mut segments = Vec::new();
    for segment in raw_path.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return None;
        }
        segments.push(segment);
    }
    Some(segments.join("/"))
}

pub fn desktop_static_dir_candidates(environment: &DesktopEnvironment) -> Vec<String> {
    vec![
        join_path(&[&environment.app_root, "apps", "server", "dist", "client"]),
        join_path(&[&environment.app_root, "apps", "web", "dist"]),
    ]
}

fn request_url_pathname(request_url: &str) -> Option<String> {
    let scheme_end = request_url.find("://")?;
    let after_scheme = &request_url[scheme_end + 3..];
    let path_start = after_scheme.find('/').map(|index| scheme_end + 3 + index)?;
    let raw_path = &request_url[path_start..];
    let path_end = raw_path
        .find(|ch| matches!(ch, '?' | '#'))
        .unwrap_or(raw_path.len());
    Some(percent_decode_path_component(&raw_path[..path_end]))
}

fn percent_decode_path_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_nibble(bytes[index + 1]), hex_nibble(bytes[index + 2]))
            {
                output.push(high * 16 + low);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub fn is_desktop_static_asset_request(request_url: &str) -> bool {
    request_url_pathname(request_url)
        .and_then(|path| path.rsplit('/').next().map(str::to_string))
        .and_then(|file_name| file_name.rsplit_once('.').map(|(_, ext)| ext.to_string()))
        .is_some_and(|ext| !ext.is_empty())
}

pub fn resolve_desktop_static_path<F>(static_root: &str, request_url: &str, exists: F) -> String
where
    F: Fn(&str) -> bool,
{
    let fallback_index = join_path(&[static_root, "index.html"]);
    let Some(raw_path) = request_url_pathname(request_url) else {
        return fallback_index;
    };
    let Some(normalized_path) = normalize_desktop_protocol_pathname(&raw_path) else {
        return fallback_index;
    };
    let requested_path = if normalized_path.is_empty() {
        "index.html".to_string()
    } else {
        normalized_path
    };
    let resolved_path = join_path(&[static_root, &requested_path]);
    if Path::new(&resolved_path).extension().is_some() {
        return resolved_path;
    }
    let nested_index = join_path(&[&resolved_path, "index.html"]);
    if exists(&nested_index) {
        nested_index
    } else {
        fallback_index
    }
}

pub fn desktop_protocol_file_response(
    static_root: &str,
    request_url: &str,
    exists: bool,
    resolved_candidate: &str,
) -> ElectronProtocolFileResponse {
    let fallback_index = join_path(&[static_root, "index.html"]);
    let static_root_prefix = format!("{}{}", static_root.trim_end_matches(['/', '\\']), "\\");
    let normalized_candidate = resolved_candidate.replace('/', "\\");
    let normalized_root = static_root.replace('/', "\\");
    let is_in_root = normalized_candidate == fallback_index.replace('/', "\\")
        || normalized_candidate.starts_with(&static_root_prefix)
        || normalized_candidate.starts_with(&format!("{normalized_root}\\"));
    if !is_in_root || !exists {
        if is_desktop_static_asset_request(request_url) {
            ElectronProtocolFileResponse::Error(-6)
        } else {
            ElectronProtocolFileResponse::Path(fallback_index)
        }
    } else {
        ElectronProtocolFileResponse::Path(resolved_candidate.to_string())
    }
}

pub fn normalize_electron_context_menu_items(
    source: &[ElectronContextMenuItem],
) -> Vec<ElectronContextMenuItem> {
    let mut normalized = Vec::new();
    for item in source {
        let (Some(id), Some(label)) = (&item.id, &item.label) else {
            continue;
        };
        let children = normalize_electron_context_menu_items(&item.children);
        if !item.children.is_empty() && children.is_empty() {
            continue;
        }
        normalized.push(ElectronContextMenuItem {
            id: Some(id.clone()),
            label: Some(label.clone()),
            destructive: item.destructive,
            disabled: item.disabled,
            children,
        });
    }
    normalized
}

pub fn normalize_electron_menu_position(x: f64, y: f64) -> Option<ElectronMenuPosition> {
    (x.is_finite() && y.is_finite() && x >= 0.0 && y >= 0.0).then(|| ElectronMenuPosition {
        x: x.floor() as i32,
        y: y.floor() as i32,
    })
}

pub fn build_electron_context_menu_template(
    entries: &[ElectronContextMenuItem],
) -> Vec<ElectronMenuTemplateItem> {
    let mut template = Vec::new();
    let mut has_inserted_destructive_separator = false;
    for item in normalize_electron_context_menu_items(entries) {
        if item.destructive && !has_inserted_destructive_separator && !template.is_empty() {
            template.push(ElectronMenuTemplateItem {
                label: String::new(),
                enabled: true,
                destructive: false,
                separator: true,
                click_id: None,
                children: Vec::new(),
            });
            has_inserted_destructive_separator = true;
        }
        let children = build_electron_context_menu_template(&item.children);
        template.push(ElectronMenuTemplateItem {
            label: item.label.unwrap_or_default(),
            enabled: !item.disabled,
            destructive: item.destructive,
            separator: false,
            click_id: if children.is_empty() { item.id } else { None },
            children,
        });
    }
    template
}

pub fn electron_updater_listener_spec(event_name: &str) -> ElectronScopedListenerSpec {
    ElectronScopedListenerSpec {
        event_name: event_name.to_string(),
        acquire_method: "autoUpdater.on",
        release_method: "autoUpdater.removeListener",
    }
}

pub fn electron_updater_set_feed_url_command(url: &str) -> ElectronUpdaterPropertyCommand {
    ElectronUpdaterPropertyCommand {
        kind: ElectronUpdaterCommandKind::SetFeedUrl,
        bool_value: None,
        string_value: Some(url.to_string()),
    }
}

pub fn electron_updater_set_bool_command(
    kind: ElectronUpdaterCommandKind,
    value: bool,
) -> Option<ElectronUpdaterPropertyCommand> {
    matches!(
        kind,
        ElectronUpdaterCommandKind::SetAutoDownload
            | ElectronUpdaterCommandKind::SetAutoInstallOnAppQuit
            | ElectronUpdaterCommandKind::SetAllowPrerelease
            | ElectronUpdaterCommandKind::SetAllowDowngrade
            | ElectronUpdaterCommandKind::SetDisableDifferentialDownload
    )
    .then_some(ElectronUpdaterPropertyCommand {
        kind,
        bool_value: Some(value),
        string_value: None,
    })
}

pub fn electron_updater_set_channel_command(channel: &str) -> ElectronUpdaterPropertyCommand {
    ElectronUpdaterPropertyCommand {
        kind: ElectronUpdaterCommandKind::SetChannel,
        bool_value: None,
        string_value: Some(channel.to_string()),
    }
}

pub fn electron_updater_quit_and_install_options(
    is_silent: bool,
    is_force_run_after: bool,
) -> ElectronUpdaterQuitAndInstallOptions {
    ElectronUpdaterQuitAndInstallOptions {
        is_silent,
        is_force_run_after,
    }
}

pub fn electron_updater_error_message(kind: ElectronUpdaterErrorKind) -> &'static str {
    match kind {
        ElectronUpdaterErrorKind::CheckForUpdates => {
            ELECTRON_UPDATER_CHECK_FOR_UPDATES_ERROR_MESSAGE
        }
        ElectronUpdaterErrorKind::DownloadUpdate => ELECTRON_UPDATER_DOWNLOAD_UPDATE_ERROR_MESSAGE,
        ElectronUpdaterErrorKind::QuitAndInstall => ELECTRON_UPDATER_QUIT_AND_INSTALL_ERROR_MESSAGE,
    }
}

pub fn parse_desktop_app_update_yml(raw: &str) -> Option<BTreeMap<String, String>> {
    let mut entries = BTreeMap::new();
    for line in raw.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty()
            || value.is_empty()
            || !key
                .chars()
                .all(|character| character == '_' || character.is_ascii_alphanumeric())
        {
            continue;
        }
        entries.insert(key.to_string(), value.to_string());
    }
    entries.contains_key("provider").then_some(entries)
}

pub fn desktop_update_query_key_all() -> Vec<&'static str> {
    vec!["desktop", "update"]
}

pub fn desktop_update_query_key_state() -> Vec<&'static str> {
    vec!["desktop", "update", "state"]
}

pub fn desktop_update_state_query_options() -> DesktopUpdateStateQueryOptions {
    DesktopUpdateStateQueryOptions {
        query_key: desktop_update_query_key_state(),
        stale_time: f64::INFINITY,
        refetch_on_mount: "always",
    }
}

pub fn create_initial_desktop_update_state(
    current_version: &str,
    runtime_info: &DesktopRuntimeInfo,
    channel: DesktopUpdateChannel,
) -> DesktopUpdateState {
    DesktopUpdateState {
        enabled: false,
        status: DesktopUpdateStatus::Disabled,
        channel,
        current_version: current_version.to_string(),
        host_arch: runtime_info.host_arch,
        app_arch: runtime_info.app_arch,
        running_under_arm64_translation: runtime_info.running_under_arm64_translation,
        available_version: None,
        downloaded_version: None,
        download_percent: None,
        checked_at: None,
        message: None,
        error_context: None,
        can_retry: false,
    }
}

pub fn create_base_desktop_update_state(
    current_version: &str,
    runtime_info: &DesktopRuntimeInfo,
    channel: DesktopUpdateChannel,
    enabled: bool,
) -> DesktopUpdateState {
    DesktopUpdateState {
        enabled,
        status: if enabled {
            DesktopUpdateStatus::Idle
        } else {
            DesktopUpdateStatus::Disabled
        },
        ..create_initial_desktop_update_state(current_version, runtime_info, channel)
    }
}

pub fn reduce_desktop_update_state_on_check_start(
    state: &DesktopUpdateState,
    checked_at: &str,
) -> DesktopUpdateState {
    DesktopUpdateState {
        status: DesktopUpdateStatus::Checking,
        checked_at: Some(checked_at.to_string()),
        message: None,
        download_percent: None,
        error_context: None,
        can_retry: false,
        ..state.clone()
    }
}

pub fn reduce_desktop_update_state_on_check_failure(
    state: &DesktopUpdateState,
    message: &str,
    checked_at: &str,
) -> DesktopUpdateState {
    DesktopUpdateState {
        status: DesktopUpdateStatus::Error,
        message: Some(message.to_string()),
        checked_at: Some(checked_at.to_string()),
        download_percent: None,
        error_context: Some(DesktopUpdateAction::Check),
        can_retry: true,
        ..state.clone()
    }
}

pub fn reduce_desktop_update_state_on_update_available(
    state: &DesktopUpdateState,
    version: &str,
    checked_at: &str,
) -> DesktopUpdateState {
    DesktopUpdateState {
        status: DesktopUpdateStatus::Available,
        available_version: Some(version.to_string()),
        downloaded_version: None,
        download_percent: None,
        checked_at: Some(checked_at.to_string()),
        message: None,
        error_context: None,
        can_retry: false,
        ..state.clone()
    }
}

pub fn reduce_desktop_update_state_on_no_update(
    state: &DesktopUpdateState,
    checked_at: &str,
) -> DesktopUpdateState {
    DesktopUpdateState {
        status: DesktopUpdateStatus::UpToDate,
        available_version: None,
        downloaded_version: None,
        download_percent: None,
        checked_at: Some(checked_at.to_string()),
        message: None,
        error_context: None,
        can_retry: false,
        ..state.clone()
    }
}

pub fn reduce_desktop_update_state_on_download_start(
    state: &DesktopUpdateState,
) -> DesktopUpdateState {
    DesktopUpdateState {
        status: DesktopUpdateStatus::Downloading,
        download_percent: Some(0.0),
        message: None,
        error_context: None,
        can_retry: false,
        ..state.clone()
    }
}

pub fn next_status_after_desktop_update_download_failure(
    state: &DesktopUpdateState,
) -> DesktopUpdateStatus {
    if state.available_version.is_some() {
        DesktopUpdateStatus::Available
    } else {
        DesktopUpdateStatus::Error
    }
}

pub fn reduce_desktop_update_state_on_download_failure(
    state: &DesktopUpdateState,
    message: &str,
) -> DesktopUpdateState {
    DesktopUpdateState {
        status: next_status_after_desktop_update_download_failure(state),
        message: Some(message.to_string()),
        download_percent: None,
        error_context: Some(DesktopUpdateAction::Download),
        can_retry: state.available_version.is_some(),
        ..state.clone()
    }
}

pub fn reduce_desktop_update_state_on_download_progress(
    state: &DesktopUpdateState,
    percent: f64,
) -> DesktopUpdateState {
    DesktopUpdateState {
        status: DesktopUpdateStatus::Downloading,
        download_percent: Some(percent),
        message: None,
        error_context: None,
        can_retry: false,
        ..state.clone()
    }
}

pub fn reduce_desktop_update_state_on_download_complete(
    state: &DesktopUpdateState,
    version: &str,
) -> DesktopUpdateState {
    DesktopUpdateState {
        status: DesktopUpdateStatus::Downloaded,
        available_version: Some(version.to_string()),
        downloaded_version: Some(version.to_string()),
        download_percent: Some(100.0),
        message: None,
        error_context: None,
        can_retry: true,
        ..state.clone()
    }
}

pub fn reduce_desktop_update_state_on_install_failure(
    state: &DesktopUpdateState,
    message: &str,
) -> DesktopUpdateState {
    DesktopUpdateState {
        status: DesktopUpdateStatus::Downloaded,
        message: Some(message.to_string()),
        error_context: Some(DesktopUpdateAction::Install),
        can_retry: true,
        ..state.clone()
    }
}

pub fn desktop_update_disabled_reason(
    input: &DesktopUpdateDisabledReasonInput,
) -> Option<&'static str> {
    if !input.has_update_feed_config {
        return Some("Automatic updates are not available because no update feed is configured.");
    }
    if input.is_development || !input.is_packaged {
        return Some("Automatic updates are only available in packaged production builds.");
    }
    if input.disabled_by_env {
        return Some("Automatic updates are disabled by the T3CODE_DISABLE_AUTO_UPDATE setting.");
    }
    if input.platform == "linux" && input.app_image.is_none() {
        return Some("Automatic updates on Linux require running the AppImage build.");
    }
    None
}

pub fn desktop_update_active_action(
    flags: DesktopUpdateInFlightFlags,
) -> Option<DesktopUpdateAction> {
    if flags.install {
        Some(DesktopUpdateAction::Install)
    } else if flags.download {
        Some(DesktopUpdateAction::Download)
    } else if flags.check {
        Some(DesktopUpdateAction::Check)
    } else {
        None
    }
}

pub fn desktop_update_action_in_progress_message(action: DesktopUpdateAction) -> String {
    format!(
        "Cannot change update tracks while an update {} action is in progress.",
        action.as_str()
    )
}

pub fn is_arm64_host_running_intel_desktop_build(runtime_info: &DesktopRuntimeInfo) -> bool {
    runtime_info.host_arch == DesktopRuntimeArch::Arm64
        && runtime_info.app_arch == DesktopRuntimeArch::X64
}

pub fn resolve_desktop_update_button_action(
    state: &DesktopUpdateState,
) -> Option<DesktopUpdateAction> {
    if state.downloaded_version.is_some() {
        return Some(DesktopUpdateAction::Install);
    }
    if state.status == DesktopUpdateStatus::Available {
        return Some(DesktopUpdateAction::Download);
    }
    if state.status == DesktopUpdateStatus::Error
        && state.error_context == Some(DesktopUpdateAction::Download)
        && state.available_version.is_some()
    {
        return Some(DesktopUpdateAction::Download);
    }
    None
}

pub fn should_show_desktop_update_button(state: Option<&DesktopUpdateState>) -> bool {
    let Some(state) = state else {
        return false;
    };
    if !state.enabled {
        return false;
    }
    state.status == DesktopUpdateStatus::Downloading
        || resolve_desktop_update_button_action(state).is_some()
}

pub fn should_show_arm64_intel_build_warning(state: Option<&DesktopUpdateState>) -> bool {
    state.is_some_and(|state| {
        state.host_arch == DesktopRuntimeArch::Arm64 && state.app_arch == DesktopRuntimeArch::X64
    })
}

pub fn is_desktop_update_button_disabled(state: Option<&DesktopUpdateState>) -> bool {
    state.is_some_and(|state| state.status == DesktopUpdateStatus::Downloading)
}

pub fn get_arm64_intel_build_warning_description(state: &DesktopUpdateState) -> String {
    if !should_show_arm64_intel_build_warning(Some(state)) {
        return "This install is using the correct architecture.".to_string();
    }

    match resolve_desktop_update_button_action(state) {
        Some(DesktopUpdateAction::Download) => "This Mac has Apple Silicon, but R3Code is still running the Intel build under Rosetta. Download the available update to switch to the native Apple Silicon build.".to_string(),
        Some(DesktopUpdateAction::Install) => "This Mac has Apple Silicon, but R3Code is still running the Intel build under Rosetta. Restart to install the downloaded Apple Silicon build.".to_string(),
        _ => "This Mac has Apple Silicon, but R3Code is still running the Intel build under Rosetta. The next app update will replace it with the native Apple Silicon build.".to_string(),
    }
}

pub fn get_desktop_update_button_tooltip(state: &DesktopUpdateState) -> String {
    if state.error_context == Some(DesktopUpdateAction::Download)
        && state.available_version.is_some()
        && state.message.is_some()
    {
        return format!(
            "Download failed for {}. Click to retry.",
            state.available_version.as_deref().unwrap()
        );
    }
    if state.error_context == Some(DesktopUpdateAction::Install)
        && state.downloaded_version.is_some()
        && state.message.is_some()
    {
        return format!(
            "Install failed for {}. Click to retry.",
            state.downloaded_version.as_deref().unwrap()
        );
    }

    match state.status {
        DesktopUpdateStatus::Available => format!(
            "Update {} ready to download",
            state.available_version.as_deref().unwrap_or("available")
        ),
        DesktopUpdateStatus::Downloading => {
            let progress = state
                .download_percent
                .map(|percent| format!(" ({}%)", percent.floor() as i64))
                .unwrap_or_default();
            format!("Downloading update{progress}")
        }
        DesktopUpdateStatus::Downloaded => format!(
            "Update {} downloaded. Click to restart and install.",
            state
                .downloaded_version
                .as_deref()
                .or(state.available_version.as_deref())
                .unwrap_or("ready")
        ),
        DesktopUpdateStatus::Error => state
            .message
            .clone()
            .unwrap_or_else(|| "Update failed".to_string()),
        _ => "Up to date".to_string(),
    }
}

pub fn get_desktop_update_install_confirmation_message(
    available_version: Option<&str>,
    downloaded_version: Option<&str>,
) -> String {
    let version = downloaded_version.or(available_version);
    format!(
        "Install update{} and restart R3Code?\n\nAny running tasks will be interrupted. Make sure you're ready before continuing.",
        version
            .map(|version| format!(" {version}"))
            .unwrap_or_default()
    )
}

pub fn get_desktop_update_action_error(
    accepted: bool,
    completed: bool,
    state: &DesktopUpdateState,
) -> Option<String> {
    if !accepted || completed {
        return None;
    }
    let message = state.message.as_deref()?.trim();
    (!message.is_empty()).then(|| message.to_string())
}

pub fn should_toast_desktop_update_action_result(
    accepted: bool,
    completed: bool,
    state: &DesktopUpdateState,
) -> bool {
    get_desktop_update_action_error(accepted, completed, state).is_some()
}

pub fn can_check_for_desktop_update(state: Option<&DesktopUpdateState>) -> bool {
    let Some(state) = state else {
        return false;
    };
    state.enabled
        && state.status != DesktopUpdateStatus::Checking
        && state.status != DesktopUpdateStatus::Downloading
        && state.status != DesktopUpdateStatus::Downloaded
        && state.status != DesktopUpdateStatus::Disabled
}

pub fn should_broadcast_desktop_update_download_progress(
    state: &DesktopUpdateState,
    next_percent: f64,
) -> bool {
    if state.status != DesktopUpdateStatus::Downloading {
        return true;
    }
    let Some(current_percent) = state.download_percent else {
        return true;
    };
    let previous_step = (current_percent / 10.0).floor() as i32;
    let next_step = (next_percent / 10.0).floor() as i32;
    next_step != previous_step || next_percent == 100.0
}

pub fn desktop_updates_configure_plan(
    environment: &DesktopEnvironment,
    settings: &DesktopSettings,
    has_update_feed_config: bool,
) -> DesktopUpdatesConfigurePlan {
    let disabled_reason = desktop_update_disabled_reason(&DesktopUpdateDisabledReasonInput {
        is_development: environment.is_development,
        is_packaged: environment.is_packaged,
        platform: environment.platform.clone(),
        app_image: None,
        disabled_by_env: false,
        has_update_feed_config,
    });
    let enabled = disabled_reason.is_none();
    let allow_prerelease = settings.update_channel == DesktopUpdateChannel::Nightly;
    DesktopUpdatesConfigurePlan {
        feed_url: None,
        enabled,
        initial_state_status: if enabled {
            DesktopUpdateStatus::Idle
        } else {
            DesktopUpdateStatus::Disabled
        },
        updater_configured: enabled,
        auto_download: false,
        auto_install_on_app_quit: false,
        allow_prerelease,
        allow_downgrade: allow_prerelease,
        disable_differential_download: is_arm64_host_running_intel_desktop_build(
            &environment.runtime_info,
        ),
        listener_events: DESKTOP_UPDATE_LISTENER_EVENTS.to_vec(),
        startup_delay_ms: DESKTOP_AUTO_UPDATE_STARTUP_DELAY_MS,
        poll_interval_ms: DESKTOP_AUTO_UPDATE_POLL_INTERVAL_MS,
    }
}

pub fn desktop_updates_mock_feed_url(port: u16) -> ElectronUpdaterFeedUrl {
    ElectronUpdaterFeedUrl {
        url: format!("http://localhost:{port}"),
    }
}

pub fn desktop_update_set_channel_plan(
    current_channel: DesktopUpdateChannel,
    next_channel: DesktopUpdateChannel,
    active_flags: DesktopUpdateInFlightFlags,
    enabled: bool,
    updater_configured: bool,
) -> DesktopUpdateSetChannelPlan {
    if let Some(action) = desktop_update_active_action(active_flags) {
        return DesktopUpdateSetChannelPlan {
            accepted: false,
            active_action: Some(action),
            persist_settings: false,
            reset_state: false,
            apply_auto_updater_channel: false,
            temporarily_allow_downgrade: false,
            check_reason: None,
        };
    }
    if current_channel == next_channel {
        return DesktopUpdateSetChannelPlan {
            accepted: true,
            active_action: None,
            persist_settings: false,
            reset_state: false,
            apply_auto_updater_channel: false,
            temporarily_allow_downgrade: false,
            check_reason: None,
        };
    }
    DesktopUpdateSetChannelPlan {
        accepted: true,
        active_action: None,
        persist_settings: true,
        reset_state: true,
        apply_auto_updater_channel: enabled && updater_configured,
        temporarily_allow_downgrade: enabled && updater_configured,
        check_reason: (enabled && updater_configured).then_some("channel-change"),
    }
}

pub fn desktop_update_download_plan(
    updater_configured: bool,
    download_in_flight: bool,
    state: &DesktopUpdateState,
) -> DesktopUpdateActionResultPlan {
    let accepted =
        updater_configured && !download_in_flight && state.status == DesktopUpdateStatus::Available;
    DesktopUpdateActionResultPlan {
        accepted,
        completed: accepted,
        stop_backend: false,
        destroy_windows: false,
        quit_and_install: None,
    }
}

pub fn desktop_update_install_plan(
    quitting: bool,
    updater_configured: bool,
    state: &DesktopUpdateState,
) -> DesktopUpdateActionResultPlan {
    let accepted =
        !quitting && updater_configured && state.status == DesktopUpdateStatus::Downloaded;
    DesktopUpdateActionResultPlan {
        accepted,
        completed: false,
        stop_backend: accepted,
        destroy_windows: accepted,
        quit_and_install: accepted.then(|| electron_updater_quit_and_install_options(true, true)),
    }
}

fn electron_window_live(window: &ElectronWindowState) -> bool {
    !window.destroyed
}

pub fn electron_window_current_main_or_first(
    main_window_id: Option<u32>,
    windows: &[ElectronWindowState],
) -> Option<u32> {
    main_window_id
        .and_then(|id| {
            windows
                .iter()
                .find(|window| window.id == id && electron_window_live(window))
                .map(|window| window.id)
        })
        .or_else(|| {
            windows
                .iter()
                .find(|window| electron_window_live(window))
                .map(|window| window.id)
        })
}

pub fn electron_window_focused_main_or_first(
    focused_window_id: Option<u32>,
    main_window_id: Option<u32>,
    windows: &[ElectronWindowState],
) -> Option<u32> {
    focused_window_id
        .and_then(|id| {
            windows
                .iter()
                .find(|window| window.id == id && electron_window_live(window))
                .map(|window| window.id)
        })
        .or_else(|| electron_window_current_main_or_first(main_window_id, windows))
}

pub fn electron_window_clear_main(
    current_main_window_id: Option<u32>,
    requested_window_id: Option<u32>,
) -> Option<u32> {
    match (current_main_window_id, requested_window_id) {
        (None, _) => None,
        (Some(current), Some(requested)) if current != requested => Some(current),
        (Some(_), _) => None,
    }
}

pub fn electron_window_reveal_plan(
    window: &ElectronWindowState,
    platform: &str,
) -> Option<ElectronWindowRevealPlan> {
    electron_window_live(window).then(|| ElectronWindowRevealPlan {
        window_id: window.id,
        restore: window.minimized,
        show: !window.visible,
        app_focus_steal: platform == "darwin",
        focus: true,
    })
}

pub fn electron_window_send_all_plan(
    windows: &[ElectronWindowState],
    channel: &str,
    args_len: usize,
) -> ElectronWindowSendAllPlan {
    ElectronWindowSendAllPlan {
        channel: channel.to_string(),
        args_len,
        target_window_ids: windows
            .iter()
            .filter(|window| electron_window_live(window))
            .map(|window| window.id)
            .collect(),
    }
}

pub fn electron_window_destroy_all_targets(windows: &[ElectronWindowState]) -> Vec<u32> {
    windows.iter().map(|window| window.id).collect()
}

pub fn electron_window_sync_all_appearance_targets(windows: &[ElectronWindowState]) -> Vec<u32> {
    windows
        .iter()
        .filter(|window| electron_window_live(window))
        .map(|window| window.id)
        .collect()
}

pub fn unwrap_desktop_preload_ensure_ssh_environment_result(
    result: &Value,
) -> Result<Value, String> {
    if result
        .as_object()
        .and_then(|object| object.get("type"))
        .and_then(Value::as_str)
        == Some(SSH_PASSWORD_PROMPT_CANCELLED_RESULT)
    {
        let message = result
            .as_object()
            .and_then(|object| object.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("SSH authentication cancelled.");
        Err(message.to_string())
    } else {
        Ok(result.clone())
    }
}

pub fn desktop_settings_document_delta(
    settings: &DesktopSettings,
    defaults: &DesktopSettings,
) -> DesktopSettingsDocument {
    DesktopSettingsDocument {
        server_exposure_mode: (settings.server_exposure_mode != defaults.server_exposure_mode)
            .then_some(settings.server_exposure_mode),
        tailscale_serve_enabled: (settings.tailscale_serve_enabled
            != defaults.tailscale_serve_enabled)
            .then_some(settings.tailscale_serve_enabled),
        tailscale_serve_port: (settings.tailscale_serve_port != defaults.tailscale_serve_port)
            .then_some(settings.tailscale_serve_port as i64),
        update_channel: (settings.update_channel != defaults.update_channel)
            .then_some(settings.update_channel),
        update_channel_configured_by_user: (settings.update_channel_configured_by_user
            != defaults.update_channel_configured_by_user)
            .then_some(settings.update_channel_configured_by_user),
    }
}

pub fn set_desktop_server_exposure_mode(
    settings: &DesktopSettings,
    requested_mode: DesktopServerExposureMode,
) -> DesktopSettingsChange {
    if settings.server_exposure_mode == requested_mode {
        return DesktopSettingsChange {
            settings: settings.clone(),
            changed: false,
        };
    }
    let mut next = settings.clone();
    next.server_exposure_mode = requested_mode;
    DesktopSettingsChange {
        settings: next,
        changed: true,
    }
}

pub fn set_desktop_tailscale_serve(
    settings: &DesktopSettings,
    enabled: bool,
    port: Option<i64>,
) -> DesktopSettingsChange {
    let normalized_port = port
        .map(|port| normalize_tailscale_serve_port(Some(port)))
        .unwrap_or(settings.tailscale_serve_port);
    if settings.tailscale_serve_enabled == enabled
        && settings.tailscale_serve_port == normalized_port
    {
        return DesktopSettingsChange {
            settings: settings.clone(),
            changed: false,
        };
    }
    let mut next = settings.clone();
    next.tailscale_serve_enabled = enabled;
    next.tailscale_serve_port = normalized_port;
    DesktopSettingsChange {
        settings: next,
        changed: true,
    }
}

pub fn set_desktop_update_channel(
    settings: &DesktopSettings,
    requested_channel: DesktopUpdateChannel,
) -> DesktopSettingsChange {
    if settings.update_channel == requested_channel {
        return DesktopSettingsChange {
            settings: settings.clone(),
            changed: false,
        };
    }
    let mut next = settings.clone();
    next.update_channel = requested_channel;
    next.update_channel_configured_by_user = true;
    DesktopSettingsChange {
        settings: next,
        changed: true,
    }
}

pub fn normalize_saved_environment_registry_document(
    version: Option<u32>,
    records: Option<Vec<SavedEnvironmentStorageRecord>>,
) -> SavedEnvironmentRegistryDocument {
    SavedEnvironmentRegistryDocument {
        version: version.unwrap_or(1),
        records: records.unwrap_or_default(),
    }
}

pub fn to_persisted_saved_environment_record(
    record: &SavedEnvironmentStorageRecord,
) -> PersistedSavedEnvironmentRecord {
    PersistedSavedEnvironmentRecord {
        environment_id: record.environment_id.clone(),
        label: record.label.clone(),
        http_base_url: record.http_base_url.clone(),
        ws_base_url: record.ws_base_url.clone(),
        created_at: record.created_at.clone(),
        last_connected_at: record.last_connected_at.clone(),
        desktop_ssh: record.desktop_ssh.clone(),
    }
}

pub fn to_saved_environment_storage_record(
    record: &PersistedSavedEnvironmentRecord,
    encrypted_bearer_token: Option<String>,
) -> SavedEnvironmentStorageRecord {
    SavedEnvironmentStorageRecord {
        environment_id: record.environment_id.clone(),
        label: record.label.clone(),
        http_base_url: record.http_base_url.clone(),
        ws_base_url: record.ws_base_url.clone(),
        created_at: record.created_at.clone(),
        last_connected_at: record.last_connected_at.clone(),
        desktop_ssh: record.desktop_ssh.clone(),
        encrypted_bearer_token,
    }
}

pub fn preserve_existing_saved_environment_secrets(
    current_document: &SavedEnvironmentRegistryDocument,
    records: &[PersistedSavedEnvironmentRecord],
) -> SavedEnvironmentRegistryDocument {
    let encrypted_by_id = current_document
        .records
        .iter()
        .filter_map(|record| {
            record
                .encrypted_bearer_token
                .as_ref()
                .map(|token| (record.environment_id.as_str(), token.as_str()))
        })
        .collect::<BTreeMap<_, _>>();

    SavedEnvironmentRegistryDocument {
        version: current_document.version,
        records: records
            .iter()
            .map(|record| {
                to_saved_environment_storage_record(
                    record,
                    encrypted_by_id
                        .get(record.environment_id.as_str())
                        .map(|token| (*token).to_string()),
                )
            })
            .collect(),
    }
}

pub fn desktop_initial_window_background_color(should_use_dark_colors: bool) -> &'static str {
    if should_use_dark_colors {
        "#0a0a0a"
    } else {
        "#ffffff"
    }
}

pub fn desktop_window_title_bar_options(
    platform: &str,
    should_use_dark_colors: bool,
) -> DesktopWindowTitleBarOptions {
    if platform == "darwin" {
        return DesktopWindowTitleBarOptions {
            title_bar_style: DesktopTitleBarStyle::HiddenInset,
            traffic_light_position: Some((16, 18)),
            title_bar_overlay: None,
        };
    }

    DesktopWindowTitleBarOptions {
        title_bar_style: DesktopTitleBarStyle::Hidden,
        traffic_light_position: None,
        title_bar_overlay: Some(DesktopTitleBarOverlay {
            color: DESKTOP_TITLEBAR_COLOR.to_string(),
            height: DESKTOP_TITLEBAR_HEIGHT,
            symbol_color: if should_use_dark_colors {
                DESKTOP_TITLEBAR_DARK_SYMBOL_COLOR
            } else {
                DESKTOP_TITLEBAR_LIGHT_SYMBOL_COLOR
            }
            .to_string(),
        }),
    }
}

pub fn desktop_main_window_options(
    platform: &str,
    display_name: &str,
    should_use_dark_colors: bool,
) -> DesktopMainWindowOptions {
    DesktopMainWindowOptions {
        width: 1100,
        height: 780,
        min_width: 840,
        min_height: 620,
        show: false,
        auto_hide_menu_bar: true,
        background_color: desktop_initial_window_background_color(should_use_dark_colors)
            .to_string(),
        title: display_name.to_string(),
        title_bar: desktop_window_title_bar_options(platform, should_use_dark_colors),
        context_isolation: true,
        node_integration: false,
        sandbox: true,
    }
}

pub fn desktop_application_menu_template(
    platform: &str,
    app_name: &str,
) -> Vec<DesktopTopLevelMenuSpec> {
    let mut template = Vec::new();

    if platform == "darwin" {
        template.push(DesktopTopLevelMenuSpec {
            label: Some(app_name.to_string()),
            role: None,
            submenu: vec![
                role_item("about"),
                action_item("Check for Updates...", None, "check-for-updates"),
                separator_item(),
                action_item("Settings...", Some("CmdOrCtrl+,"), "open-settings"),
                separator_item(),
                role_item("services"),
                separator_item(),
                role_item("hide"),
                role_item("hideOthers"),
                role_item("unhide"),
                separator_item(),
                role_item("quit"),
            ],
        });
    }

    let mut file_submenu = Vec::new();
    if platform != "darwin" {
        file_submenu.push(action_item(
            "Settings...",
            Some("CmdOrCtrl+,"),
            "open-settings",
        ));
        file_submenu.push(separator_item());
    }
    file_submenu.push(role_item(if platform == "darwin" {
        "close"
    } else {
        "quit"
    }));

    template.push(DesktopTopLevelMenuSpec {
        label: Some("File".to_string()),
        role: None,
        submenu: file_submenu,
    });
    template.push(DesktopTopLevelMenuSpec {
        label: None,
        role: Some("editMenu"),
        submenu: Vec::new(),
    });
    template.push(DesktopTopLevelMenuSpec {
        label: Some("View".to_string()),
        role: None,
        submenu: vec![
            role_item("reload"),
            role_item("forceReload"),
            role_item("toggleDevTools"),
            separator_item(),
            role_item("resetZoom"),
            menu_item(Some("zoomIn"), None, Some("CmdOrCtrl+="), None, false, true),
            menu_item(
                Some("zoomIn"),
                None,
                Some("CmdOrCtrl+Plus"),
                None,
                false,
                false,
            ),
            role_item("zoomOut"),
            separator_item(),
            role_item("togglefullscreen"),
        ],
    });
    template.push(DesktopTopLevelMenuSpec {
        label: None,
        role: Some("windowMenu"),
        submenu: Vec::new(),
    });
    template.push(DesktopTopLevelMenuSpec {
        label: None,
        role: Some("help"),
        submenu: vec![action_item(
            "Check for Updates...",
            None,
            "check-for-updates",
        )],
    });

    template
}

pub fn resolve_default_desktop_update_channel(version: &str) -> DesktopUpdateChannel {
    if is_nightly_desktop_version(version) {
        DesktopUpdateChannel::Nightly
    } else {
        DesktopUpdateChannel::Latest
    }
}

pub fn resolve_desktop_app_stage_label(
    is_development: bool,
    app_version: &str,
) -> DesktopAppStageLabel {
    if is_development {
        DesktopAppStageLabel::Dev
    } else if is_nightly_desktop_version(app_version) {
        DesktopAppStageLabel::Nightly
    } else {
        DesktopAppStageLabel::Alpha
    }
}

pub fn resolve_desktop_app_branding(is_development: bool, app_version: &str) -> DesktopAppBranding {
    let stage_label = resolve_desktop_app_stage_label(is_development, app_version);
    let base_name = "R3Code".to_string();
    DesktopAppBranding {
        display_name: format!("{} ({})", base_name, stage_label.as_str()),
        base_name,
        stage_label,
    }
}

pub fn normalize_desktop_arch(arch: &str) -> DesktopRuntimeArch {
    match arch {
        "arm64" => DesktopRuntimeArch::Arm64,
        "x64" => DesktopRuntimeArch::X64,
        _ => DesktopRuntimeArch::Other,
    }
}

pub fn resolve_desktop_runtime_info(
    platform: &str,
    process_arch: &str,
    running_under_arm64_translation: bool,
) -> DesktopRuntimeInfo {
    let app_arch = normalize_desktop_arch(process_arch);
    if platform != "darwin" {
        return DesktopRuntimeInfo {
            host_arch: app_arch,
            app_arch,
            running_under_arm64_translation: false,
        };
    }
    let host_arch = if app_arch == DesktopRuntimeArch::Arm64 || running_under_arm64_translation {
        DesktopRuntimeArch::Arm64
    } else {
        app_arch
    };
    DesktopRuntimeInfo {
        host_arch,
        app_arch,
        running_under_arm64_translation,
    }
}

pub fn parse_desktop_config_from_env(env: &BTreeMap<String, String>) -> DesktopConfig {
    DesktopConfig {
        app_data_directory: trim_env(env.get("APPDATA")),
        xdg_config_home: trim_env(env.get("XDG_CONFIG_HOME")),
        r3_home: trim_env(env.get("R3CODE_HOME")).or_else(|| trim_env(env.get("T3CODE_HOME"))),
        dev_server_url: trim_env(env.get("VITE_DEV_SERVER_URL")),
        dev_remote_r3_server_entry_path: trim_env(
            env.get("R3CODE_DEV_REMOTE_R3_SERVER_ENTRY_PATH"),
        )
        .or_else(|| trim_env(env.get("T3CODE_DEV_REMOTE_T3_SERVER_ENTRY_PATH"))),
        configured_backend_port: trim_env(env.get("R3CODE_PORT"))
            .or_else(|| trim_env(env.get("T3CODE_PORT")))
            .and_then(|value| value.parse::<u16>().ok()),
        commit_hash_override: trim_env(env.get("R3CODE_COMMIT_HASH"))
            .or_else(|| trim_env(env.get("T3CODE_COMMIT_HASH"))),
        desktop_lan_host_override: trim_env(env.get("R3CODE_DESKTOP_LAN_HOST"))
            .or_else(|| trim_env(env.get("T3CODE_DESKTOP_LAN_HOST"))),
        desktop_https_endpoint_urls: comma_separated_env(env.get("R3CODE_DESKTOP_HTTPS_ENDPOINTS"))
            .or_else(|| comma_separated_env(env.get("T3CODE_DESKTOP_HTTPS_ENDPOINTS")))
            .unwrap_or_default(),
        otlp_traces_url: trim_env(env.get("R3CODE_OTLP_TRACES_URL"))
            .or_else(|| trim_env(env.get("T3CODE_OTLP_TRACES_URL"))),
        otlp_export_interval_ms: trim_env(env.get("R3CODE_OTLP_EXPORT_INTERVAL_MS"))
            .or_else(|| trim_env(env.get("T3CODE_OTLP_EXPORT_INTERVAL_MS")))
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(10_000),
        app_image_path: trim_env(env.get("APPIMAGE")),
        disable_auto_update: parse_env_bool(env.get("R3CODE_DISABLE_AUTO_UPDATE"))
            || parse_env_bool(env.get("T3CODE_DISABLE_AUTO_UPDATE")),
        mock_updates: parse_env_bool(env.get("R3CODE_DESKTOP_MOCK_UPDATES"))
            || parse_env_bool(env.get("T3CODE_DESKTOP_MOCK_UPDATES")),
        mock_update_server_port: trim_env(env.get("R3CODE_DESKTOP_MOCK_UPDATE_SERVER_PORT"))
            .or_else(|| trim_env(env.get("T3CODE_DESKTOP_MOCK_UPDATE_SERVER_PORT")))
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(3000),
    }
}

pub fn make_desktop_environment(input: MakeDesktopEnvironmentInput) -> DesktopEnvironment {
    let is_development = input.config.dev_server_url.is_some();
    let app_data_directory = if input.platform == "win32" {
        input
            .config
            .app_data_directory
            .clone()
            .unwrap_or_else(|| join_path(&[&input.home_directory, "AppData", "Roaming"]))
    } else if input.platform == "darwin" {
        join_path(&[&input.home_directory, "Library", "Application Support"])
    } else {
        input
            .config
            .xdg_config_home
            .clone()
            .unwrap_or_else(|| join_path(&[&input.home_directory, ".config"]))
    };
    let base_dir = input
        .config
        .r3_home
        .clone()
        .unwrap_or_else(|| join_path(&[&input.home_directory, ".r3"]));
    let root_dir = normalize_join(&[&input.dirname, "../../.."]);
    let app_root = if input.is_packaged {
        input.app_path.clone()
    } else {
        root_dir.clone()
    };
    let state_dir = join_path(&[&base_dir, if is_development { "dev" } else { "userdata" }]);
    let branding = resolve_desktop_app_branding(is_development, &input.app_version);
    let display_name = branding.display_name.clone();
    let user_data_dir_name = if is_development {
        "r3code-dev"
    } else {
        "r3code"
    }
    .to_string();
    let legacy_user_data_dir_name = if is_development {
        "T3 Code (Dev)"
    } else {
        "T3 Code (Alpha)"
    }
    .to_string();

    DesktopEnvironment {
        dirname: input.dirname.clone(),
        platform: input.platform.clone(),
        process_arch: input.process_arch.clone(),
        is_packaged: input.is_packaged,
        is_development,
        app_version: input.app_version.clone(),
        app_path: input.app_path.clone(),
        resources_path: input.resources_path.clone(),
        home_directory: input.home_directory.clone(),
        app_data_directory,
        base_dir: base_dir.clone(),
        state_dir: state_dir.clone(),
        desktop_settings_path: join_path(&[&state_dir, "desktop-settings.json"]),
        client_settings_path: join_path(&[&state_dir, "client-settings.json"]),
        saved_environment_registry_path: join_path(&[&state_dir, "saved-environments.json"]),
        server_settings_path: join_path(&[&state_dir, "settings.json"]),
        log_dir: join_path(&[&state_dir, "logs"]),
        root_dir: root_dir.clone(),
        app_root: app_root.clone(),
        backend_entry_path: join_path(&[&app_root, "apps/server/dist/bin.mjs"]),
        backend_cwd: if input.is_packaged {
            input.home_directory.clone()
        } else {
            app_root.clone()
        },
        preload_path: join_path(&[&input.dirname, "preload.cjs"]),
        app_update_yml_path: if input.is_packaged {
            join_path(&[&input.resources_path, "app-update.yml"])
        } else {
            join_path(&[&input.app_path, "dev-app-update.yml"])
        },
        branding,
        display_name,
        app_user_model_id: if is_development {
            "com.r3code.r3code.dev"
        } else {
            "com.r3code.r3code"
        }
        .to_string(),
        linux_desktop_entry_name: if is_development {
            "r3code-dev.desktop"
        } else {
            "r3code.desktop"
        }
        .to_string(),
        linux_wm_class: if is_development {
            "r3code-dev"
        } else {
            "r3code"
        }
        .to_string(),
        user_data_dir_name,
        legacy_user_data_dir_name,
        runtime_info: resolve_desktop_runtime_info(
            &input.platform,
            &input.process_arch,
            input.running_under_arm64_translation,
        ),
        development_dock_icon_path: join_path(&[
            &root_dir,
            "assets",
            "dev",
            "blueprint-macos-1024.png",
        ]),
    }
}

pub fn desktop_backend_env_patch() -> BTreeMap<String, Option<String>> {
    [
        "R3CODE_PORT",
        "R3CODE_MODE",
        "R3CODE_NO_BROWSER",
        "R3CODE_HOST",
        "R3CODE_DESKTOP_WS_URL",
        "R3CODE_DESKTOP_LAN_ACCESS",
        "R3CODE_DESKTOP_LAN_HOST",
        "R3CODE_DESKTOP_HTTPS_ENDPOINTS",
        "R3CODE_TAILSCALE_SERVE",
        "R3CODE_TAILSCALE_SERVE_PORT",
    ]
    .into_iter()
    .map(|name| (name.to_string(), None))
    .collect()
}

pub fn resolve_desktop_backend_start_config(
    environment: &DesktopEnvironment,
    exposure: &DesktopBackendExposure,
    executable_path: &str,
    bootstrap_token: &str,
    otlp_traces_url: Option<String>,
    otlp_metrics_url: Option<String>,
) -> DesktopBackendStartConfig {
    let mut env_patch = desktop_backend_env_patch();
    env_patch.insert("ELECTRON_RUN_AS_NODE".to_string(), Some("1".to_string()));
    DesktopBackendStartConfig {
        executable_path: executable_path.to_string(),
        entry_path: environment.backend_entry_path.clone(),
        cwd: environment.backend_cwd.clone(),
        env_patch,
        bootstrap: DesktopBackendBootstrapConfig {
            mode: "desktop".to_string(),
            no_browser: true,
            port: exposure.port,
            r3_home: environment.base_dir.clone(),
            host: exposure.bind_host.clone(),
            desktop_bootstrap_token: bootstrap_token.to_string(),
            tailscale_serve_enabled: exposure.tailscale_serve_enabled,
            tailscale_serve_port: exposure.tailscale_serve_port,
            otlp_traces_url,
            otlp_metrics_url,
        },
        http_base_url: exposure.http_base_url.clone(),
        capture_output: true,
    }
}

pub fn normalize_desktop_commit_hash(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if (7..=40).contains(&trimmed.len())
        && trimmed
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        Some(trimmed.chars().take(12).collect::<String>().to_lowercase())
    } else {
        None
    }
}

fn trim_env(value: Option<&String>) -> Option<String> {
    value
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn comma_separated_env(value: Option<&String>) -> Option<Vec<String>> {
    trim_env(value).map(|value| {
        value
            .split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(str::to_string)
            .collect()
    })
}

fn parse_env_bool(value: Option<&String>) -> bool {
    value
        .map(|value| {
            matches!(
                value.trim().to_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn join_path(parts: &[&str]) -> String {
    let mut output = Path::new(parts[0]).to_path_buf();
    for part in &parts[1..] {
        output.push(part);
    }
    output.to_string_lossy().to_string()
}

fn normalize_join(parts: &[&str]) -> String {
    let joined = join_path(parts);
    dunce_like_normalize(Path::new(&joined))
}

fn dunce_like_normalize(path: &Path) -> String {
    let mut output = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                output.pop();
            }
            std::path::Component::CurDir => {}
            other => output.push(other.as_os_str().to_string_lossy().to_string()),
        }
    }
    if output.is_empty() {
        ".".to_string()
    } else {
        output.join(std::path::MAIN_SEPARATOR_STR)
    }
}

const fn channel(constant_name: &'static str, channel: &'static str) -> DesktopIpcChannelSpec {
    DesktopIpcChannelSpec {
        constant_name,
        channel,
    }
}

const fn preload_invoke(
    method_name: &'static str,
    channel: &'static str,
) -> DesktopPreloadBridgeMethodSpec {
    preload_method(
        method_name,
        channel,
        DesktopPreloadBridgeCallKind::Invoke,
        false,
        false,
        false,
        false,
        false,
    )
}

const fn preload_method(
    method_name: &'static str,
    channel: &'static str,
    call_kind: DesktopPreloadBridgeCallKind,
    validates_object_result: bool,
    validates_object_event: bool,
    validates_string_event: bool,
    unwraps_ssh_cancelled_result: bool,
    returns_unsubscribe: bool,
) -> DesktopPreloadBridgeMethodSpec {
    DesktopPreloadBridgeMethodSpec {
        method_name,
        channel,
        call_kind,
        validates_object_result,
        validates_object_event,
        validates_string_event,
        unwraps_ssh_cancelled_result,
        returns_unsubscribe,
    }
}

const fn invoke_handler(method_name: &'static str, channel: &'static str) -> DesktopIpcHandlerSpec {
    DesktopIpcHandlerSpec {
        method_name,
        channel,
        kind: DesktopIpcRegistrationKind::Invoke,
    }
}

const fn sync_handler(method_name: &'static str, channel: &'static str) -> DesktopIpcHandlerSpec {
    DesktopIpcHandlerSpec {
        method_name,
        channel,
        kind: DesktopIpcRegistrationKind::Sync,
    }
}

fn menu_item(
    role: Option<&'static str>,
    label: Option<&'static str>,
    accelerator: Option<&'static str>,
    action: Option<&'static str>,
    separator: bool,
    visible: bool,
) -> DesktopMenuItemSpec {
    DesktopMenuItemSpec {
        label,
        role,
        accelerator,
        action,
        separator,
        visible,
    }
}

fn role_item(role: &'static str) -> DesktopMenuItemSpec {
    menu_item(Some(role), None, None, None, false, true)
}

fn action_item(
    label: &'static str,
    accelerator: Option<&'static str>,
    action: &'static str,
) -> DesktopMenuItemSpec {
    menu_item(None, Some(label), accelerator, Some(action), false, true)
}

fn separator_item() -> DesktopMenuItemSpec {
    menu_item(None, None, None, None, true, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input(config: DesktopConfig) -> MakeDesktopEnvironmentInput {
        MakeDesktopEnvironmentInput {
            dirname: "C:/repo/apps/desktop/dist".to_string(),
            home_directory: "C:/Users/bunny".to_string(),
            platform: "win32".to_string(),
            process_arch: "x64".to_string(),
            app_version: "0.1.0".to_string(),
            app_path: "C:/Program Files/R3Code".to_string(),
            is_packaged: false,
            resources_path: "C:/Program Files/R3Code/resources".to_string(),
            running_under_arm64_translation: false,
            config,
        }
    }

    #[test]
    fn resolves_update_channels_from_nightly_versions() {
        assert!(is_nightly_desktop_version("0.2.0-nightly.20260512.1"));
        assert!(!is_nightly_desktop_version("0.2.0-nightly.2026051.1"));
        assert_eq!(
            resolve_default_desktop_update_channel("0.2.0-nightly.20260512.1"),
            DesktopUpdateChannel::Nightly
        );
        assert_eq!(
            resolve_default_desktop_update_channel("0.2.0"),
            DesktopUpdateChannel::Latest
        );
    }

    #[test]
    fn resolves_r3code_branding_and_runtime_info() {
        assert_eq!(
            resolve_desktop_app_branding(true, "0.1.0").display_name,
            "R3Code (Dev)"
        );
        assert_eq!(
            resolve_desktop_app_branding(false, "0.1.0-nightly.20260512.1").display_name,
            "R3Code (Nightly)"
        );
        assert_eq!(
            resolve_desktop_runtime_info("darwin", "x64", true),
            DesktopRuntimeInfo {
                host_arch: DesktopRuntimeArch::Arm64,
                app_arch: DesktopRuntimeArch::X64,
                running_under_arm64_translation: true,
            }
        );
    }

    #[test]
    fn parses_desktop_config_with_r3_names_and_legacy_t3_fallbacks() {
        let env = BTreeMap::from([
            ("R3CODE_HOME".to_string(), " C:/r3 ".to_string()),
            ("R3CODE_PORT".to_string(), "4567".to_string()),
            (
                "R3CODE_DESKTOP_HTTPS_ENDPOINTS".to_string(),
                "https://one, https://two".to_string(),
            ),
            ("R3CODE_DISABLE_AUTO_UPDATE".to_string(), "true".to_string()),
            (
                "T3CODE_COMMIT_HASH".to_string(),
                "abcdef1234567890".to_string(),
            ),
        ]);

        let config = parse_desktop_config_from_env(&env);

        assert_eq!(config.r3_home.as_deref(), Some("C:/r3"));
        assert_eq!(config.configured_backend_port, Some(4567));
        assert_eq!(
            config.desktop_https_endpoint_urls,
            vec!["https://one", "https://two"]
        );
        assert!(config.disable_auto_update);
        assert_eq!(
            config.commit_hash_override.as_deref(),
            Some("abcdef1234567890")
        );
    }

    #[test]
    fn builds_desktop_environment_paths_like_upstream_with_r3_names() {
        let environment = make_desktop_environment(base_input(DesktopConfig {
            dev_server_url: Some("http://localhost:5173".to_string()),
            ..DesktopConfig::default()
        }));

        assert!(environment.is_development);
        assert_eq!(environment.display_name, "R3Code (Dev)");
        assert_eq!(environment.user_data_dir_name, "r3code-dev");
        assert_eq!(environment.app_user_model_id, "com.r3code.r3code.dev");
        assert!(
            environment
                .backend_entry_path
                .replace('\\', "/")
                .ends_with("apps/server/dist/bin.mjs")
        );
        assert!(
            environment
                .desktop_settings_path
                .replace('\\', "/")
                .ends_with(".r3/dev/desktop-settings.json")
        );
    }

    #[test]
    fn builds_backend_start_config_with_sanitized_child_env() {
        let environment = make_desktop_environment(base_input(DesktopConfig::default()));
        let exposure = DesktopBackendExposure {
            port: 3487,
            bind_host: "127.0.0.1".to_string(),
            http_base_url: "http://127.0.0.1:3487".to_string(),
            tailscale_serve_enabled: true,
            tailscale_serve_port: Some(443),
        };

        let start = resolve_desktop_backend_start_config(
            &environment,
            &exposure,
            "C:/R3Code/R3Code.exe",
            "bootstrap-token",
            Some("http://trace".to_string()),
            None,
        );

        assert_eq!(start.executable_path, "C:/R3Code/R3Code.exe");
        assert_eq!(
            start.env_patch.get("ELECTRON_RUN_AS_NODE"),
            Some(&Some("1".to_string()))
        );
        assert_eq!(start.env_patch.get("R3CODE_PORT"), Some(&None));
        assert_eq!(start.bootstrap.mode, "desktop");
        assert!(start.bootstrap.no_browser);
        assert_eq!(start.bootstrap.r3_home, environment.base_dir);
        assert_eq!(start.bootstrap.desktop_bootstrap_token, "bootstrap-token");
        assert_eq!(
            start.bootstrap.otlp_traces_url.as_deref(),
            Some("http://trace")
        );
        assert!(start.capture_output);
    }

    #[test]
    fn normalizes_embedded_commit_hash_for_about_panel() {
        assert_eq!(
            normalize_desktop_commit_hash(" ABCDEF1234567890 "),
            Some("abcdef123456".to_string())
        );
        assert_eq!(normalize_desktop_commit_hash("not-a-hash"), None);
    }

    #[test]
    fn ports_desktop_ipc_channels_and_handler_registration_order() {
        assert_eq!(
            SSH_PASSWORD_PROMPT_CANCELLED_RESULT,
            "ssh-password-prompt-cancelled"
        );
        assert_eq!(DESKTOP_IPC_CHANNEL_SPECS.len(), 34);
        assert_eq!(
            DESKTOP_IPC_CHANNEL_SPECS
                .iter()
                .map(|spec| spec.channel)
                .collect::<Vec<_>>(),
            vec![
                "desktop:pick-folder",
                "desktop:confirm",
                "desktop:set-theme",
                "desktop:context-menu",
                "desktop:open-external",
                "desktop:menu-action",
                "desktop:update-state",
                "desktop:update-get-state",
                "desktop:update-set-channel",
                "desktop:update-download",
                "desktop:update-install",
                "desktop:update-check",
                "desktop:get-app-branding",
                "desktop:get-local-environment-bootstrap",
                "desktop:get-client-settings",
                "desktop:set-client-settings",
                "desktop:get-saved-environment-registry",
                "desktop:set-saved-environment-registry",
                "desktop:get-saved-environment-secret",
                "desktop:set-saved-environment-secret",
                "desktop:remove-saved-environment-secret",
                "desktop:discover-ssh-hosts",
                "desktop:ensure-ssh-environment",
                "desktop:disconnect-ssh-environment",
                "desktop:fetch-ssh-environment-descriptor",
                "desktop:bootstrap-ssh-bearer-session",
                "desktop:fetch-ssh-session-state",
                "desktop:issue-ssh-websocket-token",
                "desktop:ssh-password-prompt",
                "desktop:resolve-ssh-password-prompt",
                "desktop:get-server-exposure-state",
                "desktop:set-server-exposure-mode",
                "desktop:set-tailscale-serve-enabled",
                "desktop:get-advertised-endpoints",
            ]
        );

        assert_eq!(DESKTOP_IPC_HANDLER_INSTALL_ORDER.len(), 31);
        assert_eq!(
            DESKTOP_IPC_HANDLER_INSTALL_ORDER
                .iter()
                .take(2)
                .map(|spec| (spec.method_name, spec.channel, spec.kind))
                .collect::<Vec<_>>(),
            vec![
                (
                    "getAppBranding",
                    "desktop:get-app-branding",
                    DesktopIpcRegistrationKind::Sync
                ),
                (
                    "getLocalEnvironmentBootstrap",
                    "desktop:get-local-environment-bootstrap",
                    DesktopIpcRegistrationKind::Sync
                ),
            ]
        );
        assert_eq!(
            DESKTOP_IPC_HANDLER_INSTALL_ORDER
                .iter()
                .rev()
                .take(3)
                .map(|spec| spec.method_name)
                .collect::<Vec<_>>(),
            vec!["checkForUpdate", "installUpdate", "downloadUpdate"]
        );
    }

    #[test]
    fn ports_desktop_preload_bridge_contracts() {
        assert_eq!(DESKTOP_PRELOAD_WORLD_KEY, "desktopBridge");
        assert_eq!(DESKTOP_PRELOAD_BRIDGE_METHOD_SPECS.len(), 34);
        assert_eq!(
            DESKTOP_PRELOAD_BRIDGE_METHOD_SPECS[0],
            DesktopPreloadBridgeMethodSpec {
                method_name: "getAppBranding",
                channel: "desktop:get-app-branding",
                call_kind: DesktopPreloadBridgeCallKind::SendSync,
                validates_object_result: true,
                validates_object_event: false,
                validates_string_event: false,
                unwraps_ssh_cancelled_result: false,
                returns_unsubscribe: false,
            }
        );
        let ensure = DESKTOP_PRELOAD_BRIDGE_METHOD_SPECS
            .iter()
            .find(|method| method.method_name == "ensureSshEnvironment")
            .unwrap();
        assert_eq!(ensure.channel, "desktop:ensure-ssh-environment");
        assert!(ensure.unwraps_ssh_cancelled_result);
        let menu_action = DESKTOP_PRELOAD_BRIDGE_METHOD_SPECS
            .iter()
            .find(|method| method.method_name == "onMenuAction")
            .unwrap();
        assert_eq!(menu_action.call_kind, DesktopPreloadBridgeCallKind::On);
        assert!(menu_action.validates_string_event);
        assert!(menu_action.returns_unsubscribe);
        let update_state = DESKTOP_PRELOAD_BRIDGE_METHOD_SPECS
            .iter()
            .find(|method| method.method_name == "onUpdateState")
            .unwrap();
        assert_eq!(update_state.channel, "desktop:update-state");
        assert!(update_state.validates_object_event);
        assert!(update_state.returns_unsubscribe);

        let ok = serde_json::json!({ "environmentId": "local" });
        assert_eq!(
            unwrap_desktop_preload_ensure_ssh_environment_result(&ok).unwrap(),
            ok
        );
        let cancelled = serde_json::json!({
            "type": "ssh-password-prompt-cancelled",
            "message": "No password."
        });
        assert_eq!(
            unwrap_desktop_preload_ensure_ssh_environment_result(&cancelled).unwrap_err(),
            "No password."
        );
        let cancelled_without_message = serde_json::json!({
            "type": "ssh-password-prompt-cancelled"
        });
        assert_eq!(
            unwrap_desktop_preload_ensure_ssh_environment_result(&cancelled_without_message)
                .unwrap_err(),
            "SSH authentication cancelled."
        );
    }

    #[test]
    fn ports_desktop_settings_defaults_migrations_and_updates() {
        assert_eq!(
            resolve_default_desktop_settings("0.0.17-nightly.20260415.1"),
            DesktopSettings {
                server_exposure_mode: DesktopServerExposureMode::LocalOnly,
                tailscale_serve_enabled: false,
                tailscale_serve_port: 443,
                update_channel: DesktopUpdateChannel::Nightly,
                update_channel_configured_by_user: false,
            }
        );

        let loaded = normalize_desktop_settings_document(
            &DesktopSettingsDocument {
                server_exposure_mode: Some(DesktopServerExposureMode::NetworkAccessible),
                tailscale_serve_enabled: Some(true),
                tailscale_serve_port: Some(8443),
                update_channel: Some(DesktopUpdateChannel::Latest),
                update_channel_configured_by_user: Some(true),
            },
            "0.0.17-nightly.20260415.1",
        );
        assert_eq!(
            loaded.server_exposure_mode,
            DesktopServerExposureMode::NetworkAccessible
        );
        assert_eq!(loaded.tailscale_serve_port, 8443);
        assert_eq!(loaded.update_channel, DesktopUpdateChannel::Latest);
        assert!(loaded.update_channel_configured_by_user);

        let legacy = normalize_desktop_settings_document(
            &DesktopSettingsDocument {
                server_exposure_mode: Some(DesktopServerExposureMode::LocalOnly),
                tailscale_serve_enabled: None,
                tailscale_serve_port: Some(0),
                update_channel: Some(DesktopUpdateChannel::Latest),
                update_channel_configured_by_user: None,
            },
            "0.0.17-nightly.20260415.1",
        );
        assert_eq!(legacy.tailscale_serve_port, 443);
        assert_eq!(legacy.update_channel, DesktopUpdateChannel::Nightly);
        assert!(!legacy.update_channel_configured_by_user);

        let defaults = default_desktop_settings();
        assert!(
            !set_desktop_server_exposure_mode(&defaults, DesktopServerExposureMode::LocalOnly)
                .changed
        );
        let exposed = set_desktop_server_exposure_mode(
            &defaults,
            DesktopServerExposureMode::NetworkAccessible,
        );
        assert!(exposed.changed);
        assert_eq!(
            desktop_settings_document_delta(&exposed.settings, &defaults),
            DesktopSettingsDocument {
                server_exposure_mode: Some(DesktopServerExposureMode::NetworkAccessible),
                tailscale_serve_enabled: None,
                tailscale_serve_port: None,
                update_channel: None,
                update_channel_configured_by_user: None,
            }
        );

        let tailscale = set_desktop_tailscale_serve(&defaults, true, Some(94_443));
        assert!(tailscale.changed);
        assert_eq!(tailscale.settings.tailscale_serve_port, 443);

        let update = set_desktop_update_channel(&defaults, DesktopUpdateChannel::Nightly);
        assert!(update.changed);
        assert!(update.settings.update_channel_configured_by_user);
    }

    #[test]
    fn ports_saved_environment_registry_secret_preservation() {
        let ssh = DesktopSshTargetRecord {
            alias: "devbox".to_string(),
            hostname: "devbox.example.com".to_string(),
            username: Some("julius".to_string()),
            port: Some(22),
        };
        let record = PersistedSavedEnvironmentRecord {
            environment_id: "environment-1".to_string(),
            label: "Remote environment".to_string(),
            http_base_url: "https://remote.example.com/".to_string(),
            ws_base_url: "wss://remote.example.com/".to_string(),
            created_at: "2026-04-09T00:00:00.000Z".to_string(),
            last_connected_at: Some("2026-04-09T01:00:00.000Z".to_string()),
            desktop_ssh: Some(ssh.clone()),
        };
        let document = normalize_saved_environment_registry_document(
            None,
            Some(vec![SavedEnvironmentStorageRecord {
                environment_id: "environment-1".to_string(),
                label: "Old label".to_string(),
                http_base_url: "https://old.example.com/".to_string(),
                ws_base_url: "wss://old.example.com/".to_string(),
                created_at: "2026-04-01T00:00:00.000Z".to_string(),
                last_connected_at: None,
                desktop_ssh: Some(ssh),
                encrypted_bearer_token: Some("encrypted-token".to_string()),
            }]),
        );

        let rewritten = preserve_existing_saved_environment_secrets(&document, &[record.clone()]);
        assert_eq!(rewritten.version, 1);
        assert_eq!(rewritten.records.len(), 1);
        assert_eq!(
            rewritten.records[0].encrypted_bearer_token.as_deref(),
            Some("encrypted-token")
        );
        assert_eq!(
            to_persisted_saved_environment_record(&rewritten.records[0]),
            record
        );

        let storage = to_saved_environment_storage_record(&record, None);
        assert!(storage.encrypted_bearer_token.is_none());
    }

    #[test]
    fn ports_desktop_window_options_and_native_menu_template() {
        assert_eq!(
            desktop_main_window_options("darwin", "R3Code (Dev)", false),
            DesktopMainWindowOptions {
                width: 1100,
                height: 780,
                min_width: 840,
                min_height: 620,
                show: false,
                auto_hide_menu_bar: true,
                background_color: "#ffffff".to_string(),
                title: "R3Code (Dev)".to_string(),
                title_bar: DesktopWindowTitleBarOptions {
                    title_bar_style: DesktopTitleBarStyle::HiddenInset,
                    traffic_light_position: Some((16, 18)),
                    title_bar_overlay: None,
                },
                context_isolation: true,
                node_integration: false,
                sandbox: true,
            }
        );

        let linux_dark = desktop_main_window_options("linux", "R3Code (Dev)", true);
        assert_eq!(linux_dark.background_color, "#0a0a0a");
        assert_eq!(
            linux_dark.title_bar.title_bar_overlay,
            Some(DesktopTitleBarOverlay {
                color: "#01000000".to_string(),
                height: 40,
                symbol_color: "#f8fafc".to_string(),
            })
        );

        let linux_menu = desktop_application_menu_template("linux", "R3Code");
        assert_eq!(
            linux_menu
                .iter()
                .map(|item| (item.label.as_deref(), item.role))
                .collect::<Vec<_>>(),
            vec![
                (Some("File"), None),
                (None, Some("editMenu")),
                (Some("View"), None),
                (None, Some("windowMenu")),
                (None, Some("help")),
            ]
        );
        assert_eq!(
            linux_menu[0].submenu[0],
            DesktopMenuItemSpec {
                label: Some("Settings..."),
                role: None,
                accelerator: Some("CmdOrCtrl+,"),
                action: Some("open-settings"),
                separator: false,
                visible: true,
            }
        );
        assert_eq!(linux_menu[0].submenu[2].role, Some("quit"));

        let darwin_menu = desktop_application_menu_template("darwin", "R3Code");
        assert_eq!(darwin_menu[0].label.as_deref(), Some("R3Code"));
        assert_eq!(darwin_menu[0].submenu[0].role, Some("about"));
        assert_eq!(darwin_menu[1].submenu[0].role, Some("close"));
        assert!(
            darwin_menu[3]
                .submenu
                .iter()
                .any(|item| item.role == Some("zoomIn") && item.visible == false)
        );
    }

    #[test]
    fn ports_desktop_server_exposure_resolution_and_runtime_state() {
        let empty_network_interfaces = BTreeMap::new();
        let lan_network_interfaces = BTreeMap::from([(
            "en0".to_string(),
            vec![
                DesktopNetworkInterfaceInfo {
                    address: "127.0.0.1".to_string(),
                    family: "IPv4".to_string(),
                    internal: false,
                    netmask: None,
                    mac: None,
                    cidr: None,
                    scope_id: None,
                },
                DesktopNetworkInterfaceInfo {
                    address: "192.168.1.20".to_string(),
                    family: "IPv4".to_string(),
                    internal: false,
                    netmask: None,
                    mac: None,
                    cidr: None,
                    scope_id: None,
                },
            ],
        )]);

        let unavailable = resolve_desktop_server_exposure_runtime_state(
            DesktopServerExposureMode::NetworkAccessible,
            &set_desktop_server_exposure_mode(
                &default_desktop_settings(),
                DesktopServerExposureMode::NetworkAccessible,
            )
            .settings,
            4173,
            &empty_network_interfaces,
            None,
        );
        assert!(unavailable.unavailable);
        assert_eq!(unavailable.state.mode, DesktopServerExposureMode::LocalOnly);
        assert_eq!(
            unavailable.state.requested_mode,
            DesktopServerExposureMode::NetworkAccessible
        );
        assert_eq!(unavailable.state.bind_host, "127.0.0.1");
        assert!(unavailable.state.endpoint_url.is_none());

        let resolved = resolve_desktop_server_exposure_runtime_state(
            DesktopServerExposureMode::NetworkAccessible,
            &set_desktop_server_exposure_mode(
                &default_desktop_settings(),
                DesktopServerExposureMode::NetworkAccessible,
            )
            .settings,
            4173,
            &lan_network_interfaces,
            None,
        );
        assert!(!resolved.unavailable);
        assert_eq!(resolved.state.bind_host, "0.0.0.0");
        assert_eq!(
            desktop_server_exposure_contract_state(&resolved.state),
            DesktopServerExposureState {
                mode: DesktopServerExposureMode::NetworkAccessible,
                endpoint_url: Some("http://192.168.1.20:4173".to_string()),
                advertised_host: Some("192.168.1.20".to_string()),
                tailscale_serve_enabled: false,
                tailscale_serve_port: 443,
            }
        );
        assert_eq!(
            desktop_server_exposure_backend_config(&resolved.state),
            DesktopServerExposureBackendConfig {
                port: 4173,
                bind_host: "0.0.0.0".to_string(),
                http_base_url: "http://127.0.0.1:4173".to_string(),
                tailscale_serve_enabled: false,
                tailscale_serve_port: 443,
            }
        );

        let overridden = resolve_desktop_server_exposure(
            DesktopServerExposureMode::NetworkAccessible,
            4173,
            &lan_network_interfaces,
            Some(" 10.0.0.7 "),
        );
        assert_eq!(overridden.advertised_host.as_deref(), Some("10.0.0.7"));
        assert_eq!(
            overridden.endpoint_url.as_deref(),
            Some("http://10.0.0.7:4173")
        );

        let change = set_desktop_server_exposure_runtime_mode(
            &initial_desktop_server_exposure_runtime_state(),
            &default_desktop_settings(),
            DesktopServerExposureMode::NetworkAccessible,
            &lan_network_interfaces,
            None,
        )
        .unwrap();
        assert!(change.requires_relaunch);
        assert_eq!(
            change.state.advertised_host.as_deref(),
            Some("192.168.1.20")
        );

        assert_eq!(
            set_desktop_server_exposure_runtime_mode(
                &initial_desktop_server_exposure_runtime_state(),
                &default_desktop_settings(),
                DesktopServerExposureMode::NetworkAccessible,
                &empty_network_interfaces,
                None,
            ),
            Err(0)
        );
    }

    #[test]
    fn ports_desktop_core_advertised_endpoint_contracts() {
        let exposure = ResolvedDesktopServerExposure {
            mode: DesktopServerExposureMode::NetworkAccessible,
            bind_host: "0.0.0.0".to_string(),
            local_http_url: "http://127.0.0.1:3773".to_string(),
            local_ws_url: "ws://127.0.0.1:3773".to_string(),
            endpoint_url: Some("http://192.168.1.20:3773".to_string()),
            advertised_host: Some("192.168.1.20".to_string()),
        };

        let endpoints = resolve_desktop_core_advertised_endpoints(
            3773,
            &exposure,
            &[
                "https://desktop.example.ts.net".to_string(),
                "http://desktop.example.test:3773/path?q=1#hash".to_string(),
                "not-a-url".to_string(),
            ],
        );

        assert_eq!(
            endpoints
                .iter()
                .map(|endpoint| endpoint.http_base_url.as_str())
                .collect::<Vec<_>>(),
            vec![
                "http://127.0.0.1:3773/",
                "http://192.168.1.20:3773/",
                "https://desktop.example.ts.net/",
                "http://desktop.example.test:3773/",
            ]
        );
        assert_eq!(endpoints[0].id, "desktop-loopback:3773");
        assert_eq!(endpoints[0].provider, DESKTOP_CORE_ENDPOINT_PROVIDER);
        assert_eq!(endpoints[0].ws_base_url, "ws://127.0.0.1:3773/");
        assert_eq!(
            endpoints[0].compatibility.hosted_https_app,
            "mixed-content-blocked"
        );
        assert_eq!(endpoints[1].id, "desktop-lan:http://192.168.1.20:3773");
        assert_eq!(endpoints[1].is_default, Some(true));
        assert_eq!(endpoints[2].label, "Custom HTTPS");
        assert_eq!(endpoints[2].provider, DESKTOP_MANUAL_ENDPOINT_PROVIDER);
        assert_eq!(endpoints[2].ws_base_url, "wss://desktop.example.ts.net/");
        assert_eq!(endpoints[2].compatibility.hosted_https_app, "compatible");
        assert_eq!(endpoints[3].label, "Custom endpoint");
        assert_eq!(
            endpoints[3].description,
            Some("User-configured endpoint for this desktop backend.")
        );
        assert_eq!(endpoints[3].status, "unknown");
    }

    #[test]
    fn ports_tailscale_status_and_command_contracts() {
        let status_json = r#"{"Self":{"DNSName":"desktop.tail.ts.net.","TailscaleIPs":["100.100.100.100","fd7a:115c:a1e0::1","192.168.1.20"]}}"#;

        assert!(is_tailscale_ipv4_address("100.64.0.1"));
        assert!(is_tailscale_ipv4_address("100.127.255.254"));
        assert!(!is_tailscale_ipv4_address("100.128.0.1"));
        assert!(!is_tailscale_ipv4_address("192.168.1.44"));
        assert_eq!(
            parse_tailscale_magic_dns_name(status_json)
                .unwrap()
                .as_deref(),
            Some("desktop.tail.ts.net")
        );
        assert_eq!(parse_tailscale_magic_dns_name("{}").unwrap(), None);
        assert!(parse_tailscale_magic_dns_name("not-json").is_err());
        assert_eq!(
            parse_tailscale_status(status_json).unwrap(),
            TailscaleStatus {
                magic_dns_name: Some("desktop.tail.ts.net".to_string()),
                tailnet_ipv4_addresses: vec!["100.100.100.100".to_string()],
            }
        );
        assert_eq!(
            build_tailscale_https_base_url("desktop.tail.ts.net", None),
            "https://desktop.tail.ts.net/"
        );
        assert_eq!(
            build_tailscale_https_base_url("desktop.tail.ts.net", Some(8443)),
            "https://desktop.tail.ts.net:8443/"
        );
        assert_eq!(
            tailscale_status_command_spec(),
            TailscaleCommandSpec {
                command: "tailscale",
                args: vec!["status".to_string(), "--json".to_string()],
                shell_on_windows: true,
                timeout_ms: 1_500,
            }
        );
        assert_eq!(
            ensure_tailscale_serve_command_spec(13_773, Some(8443), None).args,
            vec![
                "serve".to_string(),
                "--bg".to_string(),
                "--https=8443".to_string(),
                "http://127.0.0.1:13773".to_string(),
            ]
        );
        assert_eq!(
            disable_tailscale_serve_command_spec(Some(8443)).args,
            vec![
                "serve".to_string(),
                "--https=8443".to_string(),
                "off".to_string(),
            ]
        );
    }

    #[test]
    fn ports_tailscale_advertised_endpoint_contracts() {
        let network_interfaces = BTreeMap::from([(
            "tailscale0".to_string(),
            vec![
                DesktopNetworkInterfaceInfo {
                    address: "100.100.100.100".to_string(),
                    family: "IPv4".to_string(),
                    internal: false,
                    netmask: Some("255.192.0.0".to_string()),
                    mac: Some("00:00:00:00:00:00".to_string()),
                    cidr: Some("100.100.100.100/10".to_string()),
                    scope_id: None,
                },
                DesktopNetworkInterfaceInfo {
                    address: "100.100.100.100".to_string(),
                    family: "IPv4".to_string(),
                    internal: false,
                    netmask: None,
                    mac: None,
                    cidr: None,
                    scope_id: None,
                },
            ],
        )]);

        let endpoints = resolve_tailscale_advertised_endpoints_from_status(
            3773,
            false,
            None,
            &network_interfaces,
            Some(r#"{"Self":{"DNSName":"desktop.tail.ts.net."}}"#),
            None,
        )
        .unwrap();

        assert_eq!(endpoints.len(), 2);
        assert_eq!(
            endpoints[0],
            DesktopAdvertisedEndpoint {
                id: "tailscale-ip:http://100.100.100.100:3773".to_string(),
                label: "Tailscale IP",
                provider: TAILSCALE_ENDPOINT_PROVIDER,
                http_base_url: "http://100.100.100.100:3773/".to_string(),
                ws_base_url: "ws://100.100.100.100:3773/".to_string(),
                reachability: "private-network",
                compatibility: DesktopAdvertisedEndpointCompatibility {
                    hosted_https_app: "mixed-content-blocked",
                    desktop_app: "compatible",
                },
                source: "desktop-addon",
                status: "available",
                is_default: None,
                description: Some("Reachable from devices on the same Tailnet."),
            }
        );
        assert_eq!(
            endpoints[1].id,
            "tailscale-magicdns:https://desktop.tail.ts.net/"
        );
        assert_eq!(endpoints[1].label, "Tailscale HTTPS");
        assert_eq!(endpoints[1].provider, TAILSCALE_ENDPOINT_PROVIDER);
        assert_eq!(endpoints[1].http_base_url, "https://desktop.tail.ts.net/");
        assert_eq!(endpoints[1].ws_base_url, "wss://desktop.tail.ts.net/");
        assert_eq!(
            endpoints[1].compatibility.hosted_https_app,
            "requires-configuration"
        );
        assert_eq!(endpoints[1].status, "unavailable");

        let serve_endpoint = resolve_tailscale_advertised_endpoints_from_status(
            3773,
            true,
            Some(8443),
            &BTreeMap::new(),
            Some(r#"{"Self":{"DNSName":"desktop.tail.ts.net."}}"#),
            Some(true),
        )
        .unwrap()
        .remove(0);
        assert_eq!(
            serve_endpoint.id,
            "tailscale-magicdns:https://desktop.tail.ts.net:8443/"
        );
        assert_eq!(
            serve_endpoint.http_base_url,
            "https://desktop.tail.ts.net:8443/"
        );
        assert_eq!(serve_endpoint.compatibility.hosted_https_app, "compatible");
        assert_eq!(serve_endpoint.status, "available");
        assert_eq!(
            serve_endpoint.description,
            Some("HTTPS endpoint served by Tailscale Serve.")
        );
    }

    #[test]
    fn ports_desktop_backend_manager_contract_constants() {
        assert_eq!(
            DESKTOP_BACKEND_READINESS_PATH,
            "/.well-known/t3/environment"
        );
        assert_eq!(
            desktop_backend_readiness_url("http://127.0.0.1:3773").unwrap(),
            "http://127.0.0.1:3773/.well-known/t3/environment"
        );
        assert_eq!(desktop_backend_restart_delay_ms(0), 500);
        assert_eq!(desktop_backend_restart_delay_ms(1), 1_000);
        assert_eq!(desktop_backend_restart_delay_ms(8), 10_000);
        assert_eq!(DESKTOP_BACKEND_READINESS_TIMEOUT_MS, 60_000);
        assert_eq!(DESKTOP_BACKEND_READINESS_INTERVAL_MS, 100);
        assert_eq!(DESKTOP_BACKEND_READINESS_REQUEST_TIMEOUT_MS, 1_000);
        assert_eq!(DESKTOP_BACKEND_TERMINATE_GRACE_MS, 2_000);
        assert_eq!(
            DesktopBackendSnapshot {
                desired_running: true,
                ready: false,
                active_pid: Some(123),
                restart_attempt: 2,
                restart_scheduled: true,
            },
            DesktopBackendSnapshot {
                desired_running: true,
                ready: false,
                active_pid: Some(123),
                restart_attempt: 2,
                restart_scheduled: true,
            }
        );
    }

    #[test]
    fn ports_desktop_state_and_observability_log_contracts() {
        assert_eq!(
            default_desktop_state_flags(),
            DesktopStateFlags {
                backend_ready: false,
                quitting: false,
            }
        );
        assert_eq!(DESKTOP_LOG_FILE_MAX_BYTES, 10 * 1024 * 1024);
        assert_eq!(DESKTOP_LOG_FILE_MAX_FILES, 10);
        assert_eq!(DESKTOP_BACKEND_CHILD_LOG_FIBER_ID, "#backend-child");
        assert_eq!(DESKTOP_TRACE_BATCH_WINDOW_MS, 200);

        let annotated = desktop_component_log_annotation(
            "desktop-window",
            BTreeMap::from([("runId".to_string(), Value::String("run-1".to_string()))]),
        );
        assert_eq!(
            annotated.annotations,
            BTreeMap::from([
                (
                    "component".to_string(),
                    Value::String("desktop-window".to_string())
                ),
                ("runId".to_string(), Value::String("run-1".to_string())),
            ])
        );
        assert_eq!(
            sanitize_desktop_log_value(" pid=123 \n\t port=3773   cwd=/repo "),
            "pid=123 port=3773 cwd=/repo"
        );

        let boundary = desktop_backend_child_session_boundary_log_record(
            DesktopBackendChildLogPhase::Start,
            "pid=123\nport=3773",
            Some("test-run"),
            "2026-05-12T20:00:00.000Z",
        );
        assert_eq!(boundary.message, "backend child process session start");
        assert_eq!(boundary.level, "INFO");
        assert_eq!(boundary.fiber_id, "#backend-child");
        assert_eq!(
            boundary.annotations.get("phase"),
            Some(&Value::String("START".to_string()))
        );
        assert_eq!(
            boundary.annotations.get("details"),
            Some(&Value::String("pid=123 port=3773".to_string()))
        );

        let output = desktop_backend_child_output_log_record(
            DesktopBackendOutputStream::Stderr,
            "bad server\n",
            Some("test-run"),
            "2026-05-12T20:00:01.000Z",
        );
        assert_eq!(output.message, "backend child process output");
        assert_eq!(output.level, "ERROR");
        assert_eq!(
            output.annotations.get("stream"),
            Some(&Value::String("stderr".to_string()))
        );
        assert_eq!(
            output.annotations.get("text"),
            Some(&Value::String("bad server\n".to_string()))
        );

        assert_eq!(
            desktop_rotating_log_file_writer_config("C:/logs/server-child.log", None, None)
                .unwrap(),
            DesktopRotatingLogFileWriterConfig {
                file_path: "C:/logs/server-child.log".to_string(),
                max_bytes: 10 * 1024 * 1024,
                max_files: 10,
            }
        );
        assert_eq!(
            desktop_rotating_log_file_writer_config("log", Some(0), None)
                .unwrap_err()
                .message,
            "maxBytes must be >= 1 (received 0)"
        );
        assert_eq!(
            desktop_rotating_log_file_writer_config("log", None, Some(0))
                .unwrap_err()
                .message,
            "maxFiles must be >= 1 (received 0)"
        );
        assert_eq!(
            desktop_rotating_log_rotation_order("server-child.log", 4),
            vec![
                (
                    "server-child.log.3".to_string(),
                    "server-child.log.4".to_string()
                ),
                (
                    "server-child.log.2".to_string(),
                    "server-child.log.3".to_string()
                ),
                (
                    "server-child.log.1".to_string(),
                    "server-child.log.2".to_string()
                ),
            ]
        );
    }

    #[test]
    fn ports_desktop_app_bootstrap_and_lifecycle_contracts() {
        assert_eq!(DEFAULT_DESKTOP_BACKEND_PORT, 3773);
        assert_eq!(MAX_DESKTOP_TCP_PORT, 65_535);
        assert_eq!(
            DESKTOP_BACKEND_PORT_PROBE_HOSTS,
            &["127.0.0.1", "0.0.0.0", "::"]
        );
        assert_eq!(
            resolve_desktop_backend_port(Some(4888), |_, _| false).unwrap(),
            DesktopBackendPortSelection {
                port: 4888,
                selected_by_scan: false,
            }
        );
        assert_eq!(
            resolve_desktop_backend_port(None, |port, host| {
                port == 3775 && DESKTOP_BACKEND_PORT_PROBE_HOSTS.contains(&host)
            })
            .unwrap(),
            DesktopBackendPortSelection {
                port: 3775,
                selected_by_scan: true,
            }
        );
        assert_eq!(
            resolve_desktop_backend_port(None, |_, _| false)
                .unwrap_err()
                .message,
            "No desktop backend port is available on hosts 127.0.0.1, 0.0.0.0, :: between 3773 and 65535."
        );
        assert_eq!(
            validate_desktop_development_backend_port(true, None)
                .unwrap_err()
                .message,
            "T3CODE_PORT is required in desktop development."
        );
        assert!(validate_desktop_development_backend_port(true, Some(3773)).is_ok());

        let fatal = desktop_fatal_startup_error_plan(
            "bootstrap",
            "database unavailable",
            Some("stack line"),
            false,
        );
        assert_eq!(fatal.dialog_title, "T3 Code failed to start");
        assert_eq!(
            fatal.dialog_body,
            "Stage: bootstrap\ndatabase unavailable\nstack line"
        );
        assert!(fatal.should_show_dialog);
        assert!(fatal.request_shutdown);
        assert!(fatal.quit_app);
        assert!(
            !desktop_fatal_startup_error_plan("whenReady", "bad", None, true).should_show_dialog
        );

        let flags = default_desktop_shutdown_flags();
        assert_eq!(
            desktop_shutdown_mark_complete(&desktop_shutdown_request(&flags)),
            DesktopShutdownFlags {
                requested: true,
                complete: true,
            }
        );
        assert_eq!(
            desktop_before_quit_plan(false),
            DesktopBeforeQuitPlan {
                prevent_default: true,
                set_quitting: true,
                request_shutdown: true,
                await_complete: true,
                mark_quit_allowed: true,
                quit_after_shutdown: true,
            }
        );
        assert_eq!(
            desktop_before_quit_plan(true),
            DesktopBeforeQuitPlan {
                prevent_default: false,
                set_quitting: true,
                request_shutdown: false,
                await_complete: false,
                mark_quit_allowed: false,
                quit_after_shutdown: false,
            }
        );
        assert_eq!(
            desktop_signal_quit_plan(false),
            DesktopSignalQuitPlan {
                set_quitting: true,
                request_shutdown: true,
                await_complete: true,
                quit_app: true,
            }
        );
        assert!(!desktop_signal_quit_plan(true).quit_app);
        assert!(desktop_window_all_closed_should_quit("win32", false));
        assert!(!desktop_window_all_closed_should_quit("darwin", false));
        assert!(!desktop_window_all_closed_should_quit("linux", true));
        assert_eq!(
            desktop_relaunch_plan(true, "update").exit_code,
            DESKTOP_RELAUNCH_DEVELOPMENT_EXIT_CODE
        );
        assert!(!desktop_relaunch_plan(true, "update").relaunch);
        assert_eq!(
            desktop_relaunch_plan(false, "update"),
            DesktopRelaunchPlan {
                reason: "update".to_string(),
                set_quitting: true,
                request_shutdown: true,
                await_complete: true,
                relaunch: true,
                exit_code: 0,
            }
        );
        assert_eq!(
            desktop_main_layer_order(),
            vec![
                "desktopEnvironmentLayer",
                "electronLayer",
                "desktopFoundationLayer",
                "desktopSshLayer",
                "desktopServerExposureLayer",
                "desktopWindowLayer",
                "desktopBackendLayer",
                "desktopApplicationLayer",
                "desktopRuntimeLayer",
                "DesktopApp.program",
            ]
        );
    }

    #[test]
    fn ports_desktop_assets_and_electron_wrapper_contracts() {
        let environment = make_desktop_environment(base_input(DesktopConfig {
            dev_server_url: Some("http://localhost:5173".to_string()),
            ..DesktopConfig::default()
        }));
        let candidates = desktop_resource_path_candidates(&environment, "icon.png")
            .into_iter()
            .map(|path| path.replace('\\', "/"))
            .collect::<Vec<_>>();
        assert!(
            candidates[0].ends_with("apps/desktop/dist/../resources/icon.png"),
            "{candidates:?}"
        );
        assert!(
            candidates[1].ends_with("apps/desktop/dist/../prod-resources/icon.png"),
            "{candidates:?}"
        );
        assert!(
            candidates[2].ends_with("resources/resources/icon.png"),
            "{candidates:?}"
        );
        assert!(
            candidates[3].ends_with("resources/icon.png"),
            "{candidates:?}"
        );

        let existing_png = environment.development_dock_icon_path.clone();
        assert_eq!(
            resolve_desktop_icon_path(&environment, "png", "darwin", |path| path == existing_png),
            Some(existing_png.clone())
        );
        assert_eq!(
            resolve_desktop_resource_path(&environment, "icon.ico", |path| {
                path.replace('\\', "/").ends_with("resources/icon.ico")
            })
            .map(|path| path.replace('\\', "/"))
            .unwrap()
            .ends_with("resources/icon.ico"),
            true
        );
        assert_eq!(
            resolve_desktop_icon_paths(&environment, "win32", |path| {
                path.replace('\\', "/").ends_with("resources/icon.ico")
            })
            .ico
            .is_some(),
            true
        );

        assert_eq!(
            electron_app_metadata("1.2.3", "/app", true, "/resources", false),
            ElectronAppMetadata {
                app_version: "1.2.3".to_string(),
                app_path: "/app".to_string(),
                is_packaged: true,
                resources_path: "/resources".to_string(),
                running_under_arm64_translation: false,
            }
        );
        assert_eq!(
            electron_app_listener_spec("activate"),
            ElectronScopedListenerSpec {
                event_name: "activate".to_string(),
                acquire_method: "app.on",
                release_method: "app.removeListener",
            }
        );
        assert_eq!(
            electron_app_append_switch_command("class", Some("r3code")),
            ElectronAppAppendSwitchCommand {
                switch_name: "class".to_string(),
                value: Some("r3code".to_string()),
            }
        );
        assert_eq!(
            electron_pick_folder_options(Some("C:/repo")),
            ElectronOpenDialogOptions {
                properties: vec!["openDirectory", "createDirectory"],
                default_path: Some("C:/repo".to_string()),
            }
        );
        assert_eq!(electron_confirm_dialog_options("   "), None);
        assert_eq!(
            electron_confirm_dialog_options(" Delete worktree? ").unwrap(),
            ElectronConfirmDialogOptions {
                dialog_type: "question",
                buttons: vec!["No", "Yes"],
                default_id: 0,
                cancel_id: 0,
                no_link: true,
                message: "Delete worktree?".to_string(),
            }
        );
        assert!(!electron_confirm_response_is_yes(0));
        assert!(electron_confirm_response_is_yes(1));
        assert_eq!(
            parse_safe_external_url(Some("https://example.com/path")).as_deref(),
            Some("https://example.com/path")
        );
        assert_eq!(parse_safe_external_url(Some("file:///etc/passwd")), None);
        assert_eq!(parse_safe_external_url(None), None);
        assert_eq!(
            electron_theme_listener_spec(),
            ElectronScopedListenerSpec {
                event_name: "updated".to_string(),
                acquire_method: "nativeTheme.on",
                release_method: "nativeTheme.removeListener",
            }
        );
        assert_eq!(
            electron_theme_source_change("dark"),
            ElectronThemeSourceChange {
                theme: "dark".to_string(),
                property: "nativeTheme.themeSource",
            }
        );
    }

    #[test]
    fn ports_desktop_shell_environment_contracts() {
        assert_eq!(
            DESKTOP_LOGIN_SHELL_ENV_NAMES,
            &[
                "PATH",
                "SSH_AUTH_SOCK",
                "HOMEBREW_PREFIX",
                "HOMEBREW_CELLAR",
                "HOMEBREW_REPOSITORY",
                "XDG_CONFIG_HOME",
                "XDG_DATA_HOME",
            ]
        );
        assert_eq!(
            DESKTOP_WINDOWS_PROFILE_ENV_NAMES,
            &["PATH", "FNM_DIR", "FNM_MULTISHELL_PATH"]
        );
        assert_eq!(
            DESKTOP_WINDOWS_SHELL_CANDIDATES,
            &["pwsh.exe", "powershell.exe"]
        );
        assert_eq!(desktop_shell_path_delimiter("win32"), ";");
        assert_eq!(desktop_shell_path_delimiter("darwin"), ":");

        let env = BTreeMap::from([
            ("SHELL".to_string(), "/opt/homebrew/bin/nu".to_string()),
            ("PATH".to_string(), "/usr/bin".to_string()),
        ]);
        assert_eq!(
            list_desktop_login_shell_candidates("darwin", &env, Some("/bin/zsh")),
            vec!["/opt/homebrew/bin/nu", "/bin/zsh"]
        );
        assert_eq!(
            list_desktop_login_shell_candidates("linux", &BTreeMap::new(), None),
            vec!["/bin/bash"]
        );
        assert_eq!(
            merge_desktop_shell_paths(
                "darwin",
                &[
                    Some("/opt/homebrew/bin:/usr/bin".to_string()),
                    Some("/Users/test/.local/bin:/usr/bin".to_string()),
                ],
            )
            .as_deref(),
            Some("/opt/homebrew/bin:/usr/bin:/Users/test/.local/bin")
        );
        assert_eq!(
            merge_desktop_shell_paths(
                "win32",
                &[
                    Some("C:\\Tools;C:\\Windows\\System32".to_string()),
                    Some("c:\\tools;C:\\Node".to_string()),
                ],
            )
            .as_deref(),
            Some("C:\\Tools;C:\\Windows\\System32;C:\\Node")
        );

        let command = capture_desktop_posix_environment_command(&["PATH", "SSH_AUTH_SOCK"]);
        assert!(command.contains("__T3CODE_ENV_PATH_START__"));
        assert!(command.contains("printenv PATH || true"));
        assert!(command.contains("__T3CODE_ENV_SSH_AUTH_SOCK_END__"));
        let output = [
            "__T3CODE_ENV_PATH_START__",
            "/opt/homebrew/bin:/usr/bin",
            "__T3CODE_ENV_PATH_END__",
            "__T3CODE_ENV_SSH_AUTH_SOCK_START__",
            "/tmp/secretive.sock",
            "__T3CODE_ENV_SSH_AUTH_SOCK_END__",
        ]
        .join("\n");
        assert_eq!(
            extract_desktop_shell_environment(&output, &["PATH", "SSH_AUTH_SOCK"]),
            BTreeMap::from([
                ("PATH".to_string(), "/opt/homebrew/bin:/usr/bin".to_string()),
                (
                    "SSH_AUTH_SOCK".to_string(),
                    "/tmp/secretive.sock".to_string()
                ),
            ])
        );

        let login_spec = desktop_login_shell_command_spec("/bin/zsh", &["PATH"]).unwrap();
        assert_eq!(login_spec.command, "/bin/zsh");
        assert_eq!(login_spec.args[0], "-ilc");
        assert_eq!(login_spec.timeout_ms, 5_000);
        assert_eq!(
            desktop_launchctl_path_command_spec().args,
            vec!["getenv", "PATH"]
        );
        let windows_specs = desktop_windows_environment_command_specs(&["PATH"], false);
        assert_eq!(
            windows_specs
                .iter()
                .map(|spec| spec.command.as_str())
                .collect::<Vec<_>>(),
            vec!["pwsh.exe", "powershell.exe"]
        );
        assert!(windows_specs[0].args.contains(&"-NoProfile".to_string()));
        assert!(windows_specs[0].shell);

        let current_env = BTreeMap::from([(
            "PATH".to_string(),
            "/Users/test/.local/bin:/usr/bin".to_string(),
        )]);
        let shell_environment = BTreeMap::from([
            ("PATH".to_string(), "/opt/homebrew/bin:/usr/bin".to_string()),
            (
                "SSH_AUTH_SOCK".to_string(),
                "/tmp/secretive.sock".to_string(),
            ),
            ("HOMEBREW_PREFIX".to_string(), "/opt/homebrew".to_string()),
        ]);
        assert_eq!(
            install_desktop_posix_environment_patch(
                "darwin",
                &current_env,
                &shell_environment,
                None,
            ),
            BTreeMap::from([
                (
                    "PATH".to_string(),
                    "/opt/homebrew/bin:/usr/bin:/Users/test/.local/bin".to_string(),
                ),
                (
                    "SSH_AUTH_SOCK".to_string(),
                    "/tmp/secretive.sock".to_string()
                ),
                ("HOMEBREW_PREFIX".to_string(), "/opt/homebrew".to_string()),
            ])
        );

        let windows_env = BTreeMap::from([
            ("PATH".to_string(), "C:\\Windows\\System32".to_string()),
            (
                "APPDATA".to_string(),
                "C:\\Users\\testuser\\AppData\\Roaming".to_string(),
            ),
            (
                "LOCALAPPDATA".to_string(),
                "C:\\Users\\testuser\\AppData\\Local".to_string(),
            ),
            ("USERPROFILE".to_string(), "C:\\Users\\testuser".to_string()),
        ]);
        let no_profile = BTreeMap::from([(
            "PATH".to_string(),
            "C:\\Custom\\Bin;C:\\Windows\\System32".to_string(),
        )]);
        let profile = BTreeMap::from([
            (
                "PATH".to_string(),
                "C:\\Profile\\Node;C:\\Windows\\System32".to_string(),
            ),
            (
                "FNM_DIR".to_string(),
                "C:\\Users\\testuser\\AppData\\Roaming\\fnm".to_string(),
            ),
            (
                "FNM_MULTISHELL_PATH".to_string(),
                "C:\\Users\\testuser\\AppData\\Local\\fnm_multishells\\123".to_string(),
            ),
        ]);
        let patch = install_desktop_windows_environment_patch(&windows_env, &no_profile, &profile);
        assert_eq!(
            patch.get("PATH").unwrap(),
            &[
                "C:\\Profile\\Node",
                "C:\\Windows\\System32",
                "C:\\Users\\testuser\\AppData\\Roaming\\npm",
                "C:\\Users\\testuser\\AppData\\Local\\Programs\\nodejs",
                "C:\\Users\\testuser\\AppData\\Local\\Volta\\bin",
                "C:\\Users\\testuser\\AppData\\Local\\pnpm",
                "C:\\Users\\testuser\\.bun\\bin",
                "C:\\Users\\testuser\\scoop\\shims",
                "C:\\Custom\\Bin",
            ]
            .join(";")
        );
        assert_eq!(
            patch.get("FNM_MULTISHELL_PATH").unwrap(),
            "C:\\Users\\testuser\\AppData\\Local\\fnm_multishells\\123"
        );
    }

    #[test]
    fn ports_electron_protocol_menu_and_safe_storage_contracts() {
        assert_eq!(DESKTOP_SCHEME, "t3");
        assert_eq!(
            desktop_scheme_privilege_spec(),
            ElectronSchemePrivilegeSpec {
                scheme: "t3",
                standard: true,
                secure: true,
                support_fetch_api: true,
                cors_enabled: true,
            }
        );
        assert_eq!(
            ELECTRON_PROTOCOL_REGISTRATION_ERROR_MESSAGE,
            "Failed to register t3: file protocol."
        );
        assert_eq!(
            ELECTRON_PROTOCOL_STATIC_BUNDLE_MISSING_MESSAGE,
            "Desktop static bundle missing. Build apps/server (with bundled client) first."
        );
        assert_eq!(
            normalize_desktop_protocol_pathname("/settings/./general").as_deref(),
            Some("settings/general")
        );
        assert_eq!(normalize_desktop_protocol_pathname("/../secret"), None);

        let environment = make_desktop_environment(base_input(DesktopConfig::default()));
        let static_candidates = desktop_static_dir_candidates(&environment)
            .into_iter()
            .map(|path| path.replace('\\', "/"))
            .collect::<Vec<_>>();
        assert!(
            static_candidates[0].ends_with("apps/server/dist/client"),
            "{static_candidates:?}"
        );
        assert!(
            static_candidates[1].ends_with("apps/web/dist"),
            "{static_candidates:?}"
        );

        let static_root = "C:/app/apps/server/dist/client";
        assert_eq!(
            resolve_desktop_static_path(static_root, "t3://app/settings", |path| {
                path.replace('\\', "/").ends_with("settings/index.html")
            })
            .replace('\\', "/"),
            "C:/app/apps/server/dist/client/settings/index.html"
        );
        assert_eq!(
            resolve_desktop_static_path(static_root, "t3://app/../secret", |_| true)
                .replace('\\', "/"),
            "C:/app/apps/server/dist/client/index.html"
        );
        assert!(is_desktop_static_asset_request("t3://app/assets/app.js"));
        assert!(!is_desktop_static_asset_request("t3://app/settings"));
        assert_eq!(
            desktop_protocol_file_response(
                static_root,
                "t3://app/assets/missing.js",
                false,
                "C:/app/apps/server/dist/client/assets/missing.js",
            ),
            ElectronProtocolFileResponse::Error(-6)
        );
        let fallback_response = desktop_protocol_file_response(
            static_root,
            "t3://app/settings",
            false,
            "C:/outside/settings",
        );
        assert_eq!(
            match fallback_response {
                ElectronProtocolFileResponse::Path(path) => {
                    ElectronProtocolFileResponse::Path(path.replace('\\', "/"))
                }
                response => response,
            },
            ElectronProtocolFileResponse::Path(
                "C:/app/apps/server/dist/client/index.html".to_string(),
            )
        );

        assert_eq!(
            normalize_electron_menu_position(10.8, 20.2),
            Some(ElectronMenuPosition { x: 10, y: 20 })
        );
        assert_eq!(normalize_electron_menu_position(-1.0, 20.0), None);
        let menu_items = vec![
            ElectronContextMenuItem {
                id: Some("copy".to_string()),
                label: Some("Copy".to_string()),
                destructive: false,
                disabled: false,
                children: Vec::new(),
            },
            ElectronContextMenuItem {
                id: Some("delete".to_string()),
                label: Some("Delete".to_string()),
                destructive: true,
                disabled: false,
                children: Vec::new(),
            },
            ElectronContextMenuItem {
                id: None,
                label: Some("Invalid".to_string()),
                destructive: false,
                disabled: false,
                children: Vec::new(),
            },
        ];
        let template = build_electron_context_menu_template(&menu_items);
        assert_eq!(template.len(), 3);
        assert_eq!(template[0].label, "Copy");
        assert_eq!(template[0].click_id.as_deref(), Some("copy"));
        assert!(template[1].separator);
        assert_eq!(template[2].label, "Delete");
        assert!(template[2].destructive);

        assert_eq!(
            ELECTRON_SAFE_STORAGE_AVAILABILITY_ERROR_MESSAGE,
            "Electron safe storage failed to check encryption availability."
        );
        assert_eq!(
            ELECTRON_SAFE_STORAGE_ENCRYPT_ERROR_MESSAGE,
            "Electron safe storage failed to encrypt a string."
        );
        assert_eq!(
            ELECTRON_SAFE_STORAGE_DECRYPT_ERROR_MESSAGE,
            "Electron safe storage failed to decrypt a string."
        );
    }

    #[test]
    fn ports_electron_updater_and_window_contracts() {
        assert_eq!(
            electron_updater_listener_spec("update-available"),
            ElectronScopedListenerSpec {
                event_name: "update-available".to_string(),
                acquire_method: "autoUpdater.on",
                release_method: "autoUpdater.removeListener",
            }
        );
        assert_eq!(
            electron_updater_set_feed_url_command("https://updates.example/latest"),
            ElectronUpdaterPropertyCommand {
                kind: ElectronUpdaterCommandKind::SetFeedUrl,
                bool_value: None,
                string_value: Some("https://updates.example/latest".to_string()),
            }
        );
        assert_eq!(
            electron_updater_set_bool_command(ElectronUpdaterCommandKind::SetAutoDownload, false)
                .unwrap(),
            ElectronUpdaterPropertyCommand {
                kind: ElectronUpdaterCommandKind::SetAutoDownload,
                bool_value: Some(false),
                string_value: None,
            }
        );
        assert_eq!(
            electron_updater_set_bool_command(ElectronUpdaterCommandKind::CheckForUpdates, true),
            None
        );
        assert_eq!(
            electron_updater_set_channel_command("nightly"),
            ElectronUpdaterPropertyCommand {
                kind: ElectronUpdaterCommandKind::SetChannel,
                bool_value: None,
                string_value: Some("nightly".to_string()),
            }
        );
        assert_eq!(
            electron_updater_quit_and_install_options(true, false),
            ElectronUpdaterQuitAndInstallOptions {
                is_silent: true,
                is_force_run_after: false,
            }
        );
        assert_eq!(
            ElectronUpdaterCommandKind::SetDisableDifferentialDownload.method_name(),
            "disableDifferentialDownload"
        );
        assert_eq!(
            electron_updater_error_message(ElectronUpdaterErrorKind::CheckForUpdates),
            "Electron updater failed to check for updates."
        );
        assert_eq!(
            electron_updater_error_message(ElectronUpdaterErrorKind::DownloadUpdate),
            "Electron updater failed to download the update."
        );
        assert_eq!(
            electron_updater_error_message(ElectronUpdaterErrorKind::QuitAndInstall),
            "Electron updater failed to quit and install the update."
        );
        assert_eq!(
            ELECTRON_WINDOW_CREATE_ERROR_MESSAGE,
            "Failed to create Electron BrowserWindow."
        );

        let windows = vec![
            ElectronWindowState {
                id: 1,
                destroyed: true,
                minimized: false,
                visible: true,
            },
            ElectronWindowState {
                id: 2,
                destroyed: false,
                minimized: true,
                visible: false,
            },
            ElectronWindowState {
                id: 3,
                destroyed: false,
                minimized: false,
                visible: true,
            },
        ];
        assert_eq!(
            electron_window_current_main_or_first(Some(3), &windows),
            Some(3)
        );
        assert_eq!(
            electron_window_current_main_or_first(Some(1), &windows),
            Some(2)
        );
        assert_eq!(
            electron_window_focused_main_or_first(Some(1), Some(3), &windows),
            Some(3)
        );
        assert_eq!(
            electron_window_focused_main_or_first(Some(2), Some(3), &windows),
            Some(2)
        );
        assert_eq!(electron_window_clear_main(Some(3), Some(2)), Some(3));
        assert_eq!(electron_window_clear_main(Some(3), Some(3)), None);
        assert_eq!(electron_window_clear_main(Some(3), None), None);
        assert_eq!(
            electron_window_reveal_plan(&windows[1], "darwin"),
            Some(ElectronWindowRevealPlan {
                window_id: 2,
                restore: true,
                show: true,
                app_focus_steal: true,
                focus: true,
            })
        );
        assert_eq!(electron_window_reveal_plan(&windows[0], "darwin"), None);
        assert_eq!(
            electron_window_send_all_plan(&windows, "desktop:update-state", 2),
            ElectronWindowSendAllPlan {
                channel: "desktop:update-state".to_string(),
                args_len: 2,
                target_window_ids: vec![2, 3],
            }
        );
        assert_eq!(electron_window_destroy_all_targets(&windows), vec![1, 2, 3]);
        assert_eq!(
            electron_window_sync_all_appearance_targets(&windows),
            vec![2, 3]
        );
    }

    #[test]
    fn ports_desktop_updates_runtime_contracts() {
        assert_eq!(desktop_update_query_key_all(), vec!["desktop", "update"]);
        assert_eq!(
            desktop_update_query_key_state(),
            vec!["desktop", "update", "state"]
        );
        assert_eq!(
            desktop_update_state_query_options(),
            DesktopUpdateStateQueryOptions {
                query_key: vec!["desktop", "update", "state"],
                stale_time: f64::INFINITY,
                refetch_on_mount: "always",
            }
        );

        let runtime = DesktopRuntimeInfo {
            host_arch: DesktopRuntimeArch::X64,
            app_arch: DesktopRuntimeArch::X64,
            running_under_arm64_translation: false,
        };
        let initial =
            create_initial_desktop_update_state("1.0.0", &runtime, DesktopUpdateChannel::Latest);
        assert_eq!(initial.status, DesktopUpdateStatus::Disabled);
        assert_eq!(initial.current_version, "1.0.0");
        assert_eq!(initial.host_arch, DesktopRuntimeArch::X64);

        let enabled =
            create_base_desktop_update_state("1.0.0", &runtime, DesktopUpdateChannel::Latest, true);
        let checked_at = "2026-03-04T00:00:00.000Z";
        let checking = reduce_desktop_update_state_on_check_start(
            &DesktopUpdateState {
                status: DesktopUpdateStatus::Error,
                message: Some("network".to_string()),
                error_context: Some(DesktopUpdateAction::Check),
                can_retry: true,
                ..enabled.clone()
            },
            checked_at,
        );
        assert_eq!(checking.status, DesktopUpdateStatus::Checking);
        assert_eq!(checking.message, None);
        assert_eq!(checking.error_context, None);
        assert!(!checking.can_retry);

        let failed = reduce_desktop_update_state_on_check_failure(
            &checking,
            "network unavailable",
            checked_at,
        );
        assert_eq!(failed.status, DesktopUpdateStatus::Error);
        assert_eq!(failed.error_context, Some(DesktopUpdateAction::Check));
        assert!(failed.can_retry);

        let available =
            reduce_desktop_update_state_on_update_available(&checking, "1.1.0", checked_at);
        assert_eq!(available.status, DesktopUpdateStatus::Available);
        assert_eq!(available.available_version.as_deref(), Some("1.1.0"));
        assert!(should_show_desktop_update_button(Some(&available)));
        assert_eq!(
            resolve_desktop_update_button_action(&available),
            Some(DesktopUpdateAction::Download)
        );
        assert_eq!(
            get_desktop_update_button_tooltip(&available),
            "Update 1.1.0 ready to download"
        );
        assert!(can_check_for_desktop_update(Some(&available)));

        let downloading = reduce_desktop_update_state_on_download_start(&available);
        assert_eq!(downloading.download_percent, Some(0.0));
        let progress = reduce_desktop_update_state_on_download_progress(&downloading, 55.5);
        assert_eq!(progress.download_percent, Some(55.5));
        assert!(should_show_desktop_update_button(Some(&progress)));
        assert!(is_desktop_update_button_disabled(Some(&progress)));
        assert_eq!(
            get_desktop_update_button_tooltip(&progress),
            "Downloading update (55%)"
        );
        assert!(!can_check_for_desktop_update(Some(&progress)));
        assert!(
            !should_broadcast_desktop_update_download_progress(&progress, 59.9),
            "same 10% bucket should not rebroadcast"
        );
        assert!(should_broadcast_desktop_update_download_progress(
            &progress, 60.0
        ));
        assert!(should_broadcast_desktop_update_download_progress(
            &progress, 100.0
        ));

        let download_failed =
            reduce_desktop_update_state_on_download_failure(&progress, "checksum mismatch");
        assert_eq!(download_failed.status, DesktopUpdateStatus::Available);
        assert_eq!(
            download_failed.error_context,
            Some(DesktopUpdateAction::Download)
        );
        assert!(download_failed.can_retry);
        assert!(should_show_desktop_update_button(Some(&download_failed)));
        assert_eq!(
            resolve_desktop_update_button_action(&download_failed),
            Some(DesktopUpdateAction::Download)
        );
        assert_eq!(
            get_desktop_update_button_tooltip(&download_failed),
            "Download failed for 1.1.0. Click to retry."
        );
        assert_eq!(
            get_desktop_update_action_error(true, false, &download_failed).as_deref(),
            Some("checksum mismatch")
        );
        assert!(should_toast_desktop_update_action_result(
            true,
            false,
            &download_failed
        ));
        assert!(!should_toast_desktop_update_action_result(
            false,
            false,
            &download_failed
        ));

        let downloaded = reduce_desktop_update_state_on_download_complete(&progress, "1.1.0");
        assert_eq!(downloaded.status, DesktopUpdateStatus::Downloaded);
        assert_eq!(downloaded.downloaded_version.as_deref(), Some("1.1.0"));
        assert_eq!(
            resolve_desktop_update_button_action(&downloaded),
            Some(DesktopUpdateAction::Install)
        );
        assert_eq!(
            get_desktop_update_install_confirmation_message(
                downloaded.available_version.as_deref(),
                Some("1.1.1")
            ),
            "Install update 1.1.1 and restart R3Code?\n\nAny running tasks will be interrupted. Make sure you're ready before continuing."
        );
        assert!(!can_check_for_desktop_update(Some(&downloaded)));
        let install_failed = reduce_desktop_update_state_on_install_failure(
            &downloaded,
            "backend shutdown timed out",
        );
        assert_eq!(install_failed.status, DesktopUpdateStatus::Downloaded);
        assert_eq!(
            install_failed.error_context,
            Some(DesktopUpdateAction::Install)
        );
        assert!(should_show_desktop_update_button(Some(&install_failed)));
        assert_eq!(
            resolve_desktop_update_button_action(&install_failed),
            Some(DesktopUpdateAction::Install)
        );
        assert_eq!(
            get_desktop_update_button_tooltip(&install_failed),
            "Install failed for 1.1.0. Click to retry."
        );
        let up_to_date = reduce_desktop_update_state_on_no_update(&install_failed, checked_at);
        assert_eq!(up_to_date.status, DesktopUpdateStatus::UpToDate);
        assert_eq!(up_to_date.available_version, None);
        assert_eq!(up_to_date.downloaded_version, None);
        assert!(!should_show_desktop_update_button(Some(&up_to_date)));
        assert_eq!(resolve_desktop_update_button_action(&up_to_date), None);
        assert_eq!(get_desktop_update_button_tooltip(&up_to_date), "Up to date");
        assert!(can_check_for_desktop_update(Some(&up_to_date)));
        assert!(!can_check_for_desktop_update(None));
        assert!(!can_check_for_desktop_update(Some(&DesktopUpdateState {
            enabled: false,
            status: DesktopUpdateStatus::Disabled,
            ..enabled.clone()
        })));

        assert_eq!(
            parse_desktop_app_update_yml(
                "provider: generic\nurl: https://updates.example\nignored\nchannel: latest"
            )
            .unwrap()
            .get("url")
            .map(String::as_str),
            Some("https://updates.example")
        );
        assert_eq!(
            parse_desktop_app_update_yml("url: https://updates.example"),
            None
        );

        assert_eq!(
            desktop_update_disabled_reason(&DesktopUpdateDisabledReasonInput {
                is_development: false,
                is_packaged: true,
                platform: "darwin".to_string(),
                app_image: None,
                disabled_by_env: false,
                has_update_feed_config: false,
            }),
            Some("Automatic updates are not available because no update feed is configured.")
        );
        assert_eq!(
            desktop_update_disabled_reason(&DesktopUpdateDisabledReasonInput {
                is_development: false,
                is_packaged: true,
                platform: "linux".to_string(),
                app_image: None,
                disabled_by_env: false,
                has_update_feed_config: true,
            }),
            Some("Automatic updates on Linux require running the AppImage build.")
        );
        assert_eq!(
            desktop_update_action_in_progress_message(DesktopUpdateAction::Download),
            "Cannot change update tracks while an update download action is in progress."
        );
        assert_eq!(
            desktop_update_active_action(DesktopUpdateInFlightFlags {
                check: true,
                download: true,
                install: false,
            }),
            Some(DesktopUpdateAction::Download)
        );

        let arm_runtime = DesktopRuntimeInfo {
            host_arch: DesktopRuntimeArch::Arm64,
            app_arch: DesktopRuntimeArch::X64,
            running_under_arm64_translation: false,
        };
        assert!(is_arm64_host_running_intel_desktop_build(&arm_runtime));
        let arm_update_state = DesktopUpdateState {
            host_arch: DesktopRuntimeArch::Arm64,
            app_arch: DesktopRuntimeArch::X64,
            running_under_arm64_translation: true,
            ..available.clone()
        };
        assert!(should_show_arm64_intel_build_warning(Some(
            &arm_update_state
        )));
        assert!(
            get_arm64_intel_build_warning_description(&arm_update_state)
                .contains("Download the available update")
        );
        assert_eq!(
            get_desktop_update_install_confirmation_message(None, None),
            "Install update and restart R3Code?\n\nAny running tasks will be interrupted. Make sure you're ready before continuing."
        );
        let mut environment_input = base_input(DesktopConfig {
            mock_updates: true,
            ..DesktopConfig::default()
        });
        environment_input.is_packaged = true;
        let environment = make_desktop_environment(environment_input);
        let mut settings = default_desktop_settings();
        settings.update_channel = DesktopUpdateChannel::Nightly;
        let configure = desktop_updates_configure_plan(&environment, &settings, true);
        assert!(configure.enabled);
        assert_eq!(configure.listener_events, DESKTOP_UPDATE_LISTENER_EVENTS);
        assert_eq!(configure.startup_delay_ms, 15_000);
        assert_eq!(configure.poll_interval_ms, 240_000);
        assert!(configure.allow_prerelease);
        assert!(configure.allow_downgrade);
        assert_eq!(
            desktop_updates_mock_feed_url(4141),
            ElectronUpdaterFeedUrl {
                url: "http://localhost:4141".to_string(),
            }
        );

        assert_eq!(
            desktop_update_set_channel_plan(
                DesktopUpdateChannel::Latest,
                DesktopUpdateChannel::Nightly,
                DesktopUpdateInFlightFlags {
                    check: true,
                    download: false,
                    install: false,
                },
                true,
                true,
            ),
            DesktopUpdateSetChannelPlan {
                accepted: false,
                active_action: Some(DesktopUpdateAction::Check),
                persist_settings: false,
                reset_state: false,
                apply_auto_updater_channel: false,
                temporarily_allow_downgrade: false,
                check_reason: None,
            }
        );
        assert_eq!(
            desktop_update_set_channel_plan(
                DesktopUpdateChannel::Latest,
                DesktopUpdateChannel::Nightly,
                DesktopUpdateInFlightFlags::default(),
                true,
                true,
            ),
            DesktopUpdateSetChannelPlan {
                accepted: true,
                active_action: None,
                persist_settings: true,
                reset_state: true,
                apply_auto_updater_channel: true,
                temporarily_allow_downgrade: true,
                check_reason: Some("channel-change"),
            }
        );
        assert_eq!(
            desktop_update_download_plan(true, false, &available),
            DesktopUpdateActionResultPlan {
                accepted: true,
                completed: true,
                stop_backend: false,
                destroy_windows: false,
                quit_and_install: None,
            }
        );
        assert_eq!(
            desktop_update_install_plan(false, true, &downloaded),
            DesktopUpdateActionResultPlan {
                accepted: true,
                completed: false,
                stop_backend: true,
                destroy_windows: true,
                quit_and_install: Some(ElectronUpdaterQuitAndInstallOptions {
                    is_silent: true,
                    is_force_run_after: true,
                }),
            }
        );
    }
}
