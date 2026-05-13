use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerNetworkInterfaceInfo {
    pub address: String,
    pub family: String,
    pub internal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessServeAccessInfo {
    pub connection_string: String,
    pub token: String,
    pub pairing_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectedUint8StreamText {
    pub text: String,
    pub truncated: bool,
    pub bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicWriteFileStringPlan {
    pub file_path: String,
    pub contents_len: usize,
    pub target_directory: String,
    pub temp_directory_prefix: String,
    pub temp_file_suffix: &'static str,
    pub make_target_directory_recursive: bool,
    pub scoped_temp_directory: bool,
    pub final_operation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AuthClientMetadata {
    pub label: Option<String>,
    pub device_type: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedPairingCredential {
    pub id: String,
    pub credential: String,
    pub label: Option<String>,
    pub role: String,
    pub subject: String,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedBearerSession {
    pub session_id: String,
    pub token: String,
    pub method: String,
    pub role: String,
    pub subject: String,
    pub client: AuthClientMetadata,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthClientSession {
    pub session_id: String,
    pub method: String,
    pub role: String,
    pub subject: String,
    pub client: AuthClientMetadata,
    pub connected: bool,
    pub issued_at: String,
    pub expires_at: String,
    pub last_connected_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEnvironmentVariable {
    pub name: String,
    pub value: String,
    pub sensitive: bool,
    pub value_redacted: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProviderInstanceConfig {
    pub environment: Vec<ProviderEnvironmentVariable>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerSettingsForClient {
    pub provider_instances: BTreeMap<String, ProviderInstanceConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerRuntimeMode {
    Web,
    Desktop,
}

impl ServerRuntimeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Desktop => "desktop",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupPresentation {
    Browser,
    Headless,
}

impl StartupPresentation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Headless => "headless",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerDerivedPaths {
    pub state_dir: String,
    pub db_path: String,
    pub keybindings_config_path: String,
    pub settings_path: String,
    pub provider_status_cache_dir: String,
    pub worktrees_dir: String,
    pub attachments_dir: String,
    pub logs_dir: String,
    pub server_log_path: String,
    pub server_trace_path: String,
    pub provider_logs_dir: String,
    pub provider_event_log_path: String,
    pub terminal_logs_dir: String,
    pub anonymous_id_path: String,
    pub environment_id_path: String,
    pub server_runtime_state_path: String,
    pub secrets_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerModelSelection {
    pub instance_id: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerWelcomeBase {
    pub cwd: String,
    pub project_name: String,
}

pub const SERVER_PACKAGE_NAME: &str = "r3";
pub const UPSTREAM_SERVER_PACKAGE_NAME: &str = "t3";
pub const SERVER_PACKAGE_VERSION: &str = "0.0.23";
pub const SERVER_PACKAGE_BIN_ENTRY: &str = "./dist/bin.mjs";
pub const SERVER_NODE_ENGINES: &str = "^22.16 || ^23.11 || >=24.10";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPackageMetadata {
    pub name: &'static str,
    pub upstream_name: &'static str,
    pub version: &'static str,
    pub license: &'static str,
    pub repository_url: &'static str,
    pub repository_directory: &'static str,
    pub bin_name: &'static str,
    pub bin_entry: &'static str,
    pub module_type: &'static str,
    pub files: Vec<&'static str>,
    pub engines_node: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerBuildToolConfig {
    pub entry: Vec<&'static str>,
    pub formats: Vec<&'static str>,
    pub out_dir: &'static str,
    pub sourcemap: bool,
    pub clean: bool,
    pub banner: &'static str,
    pub no_external_prefixes: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerTestRuntimeConfig {
    pub file_parallelism: bool,
    pub test_timeout_ms: u64,
    pub hook_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerBuildCommandPlan {
    pub command: &'static str,
    pub args: Vec<&'static str>,
    pub cwd_relative: &'static str,
    pub windows_shell: bool,
    pub client_source: &'static str,
    pub client_target: &'static str,
    pub dev_icon_overrides_after_client_copy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPublishCommandPlan {
    pub required_assets: Vec<&'static str>,
    pub package_backup_suffix: &'static str,
    pub stripped_fields: Vec<&'static str>,
    pub resolved_fields: Vec<&'static str>,
    pub publish_command: &'static str,
    pub publish_args: Vec<&'static str>,
    pub optional_args: Vec<&'static str>,
    pub publish_icon_overrides_with_restore: bool,
    pub windows_shell: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCliScriptPlan {
    pub root_command: &'static str,
    pub description: &'static str,
    pub subcommands: Vec<&'static str>,
    pub provided_layers: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerRuntimeStartupErrorPlan {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerCommandReadinessState {
    Pending,
    Ready,
    Failed(ServerRuntimeStartupErrorPlan),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCommandGateEnqueuePlan {
    pub run_immediately: bool,
    pub queue_until_ready: bool,
    pub error: Option<ServerRuntimeStartupErrorPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLifecycleEvent {
    pub version: u32,
    pub event_type: String,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerLifecycleSnapshotState {
    pub sequence: u64,
    pub events: Vec<ServerLifecycleEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedServerRuntimeState {
    pub version: u32,
    pub pid: u32,
    pub host: Option<String>,
    pub port: u16,
    pub origin: String,
    pub started_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliServerPrecedencePlan {
    pub no_browser: bool,
    pub auto_bootstrap_project_from_cwd: bool,
    pub log_websocket_events: bool,
    pub tailscale_serve_enabled: bool,
    pub tailscale_serve_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerEnvironmentLabelInput {
    pub cwd_base_name: String,
    pub platform: String,
    pub hostname: Option<String>,
    pub macos_computer_name: Option<String>,
    pub linux_machine_info: Option<String>,
    pub linux_hostnamectl_pretty: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionEnvironmentPlatformOs {
    Darwin,
    Linux,
    Windows,
    Unknown,
}

impl ExecutionEnvironmentPlatformOs {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Darwin => "darwin",
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionEnvironmentPlatformArch {
    Arm64,
    X64,
    Other,
}

impl ExecutionEnvironmentPlatformArch {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Arm64 => "arm64",
            Self::X64 => "x64",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEnvironmentPlatform {
    pub os: ExecutionEnvironmentPlatformOs,
    pub arch: ExecutionEnvironmentPlatformArch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEnvironmentCapabilities {
    pub repository_identity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEnvironmentDescriptor {
    pub environment_id: String,
    pub label: String,
    pub platform: ExecutionEnvironmentPlatform,
    pub server_version: String,
    pub capabilities: ExecutionEnvironmentCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerEnvironmentIdPlan {
    pub environment_id: String,
    pub persist_contents: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerEnvironmentIdReadError {
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MakeServerEnvironmentInput {
    pub cwd: String,
    pub environment_id: String,
    pub node_platform: String,
    pub node_arch: String,
    pub server_version: String,
    pub hostname: Option<String>,
    pub macos_computer_name: Option<String>,
    pub linux_machine_info: Option<String>,
    pub linux_hostnamectl_pretty: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLoggerLayerPlan {
    pub minimum_log_level: String,
    pub logger_names: Vec<&'static str>,
    pub merge_with_existing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCommandSpec {
    pub path: Vec<&'static str>,
    pub description: &'static str,
    pub handler: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCommandRunPlan {
    pub startup_presentation: StartupPresentation,
    pub force_auto_bootstrap_project_from_cwd: Option<bool>,
}

pub const DEFAULT_SERVER_MODEL: &str = "gpt-5.4";
pub const DEFAULT_PROVIDER_INTERACTION_MODE: &str = "default";
pub const UPSTREAM_SERVER_VERSION: &str = "0.0.23";
pub const R3_CLI_NAME: &str = "r3";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapReadOutcome {
    Some(serde_json::Value),
    None,
    Error { message: &'static str },
}

pub const DEFAULT_SERVER_PORT: u16 = 3773;
pub const DEFAULT_TRACE_MAX_BYTES: u64 = 10 * 1024 * 1024;
pub const DEFAULT_TRACE_MAX_FILES: u16 = 10;
pub const DEFAULT_TRACE_BATCH_WINDOW_MS: u64 = 200;
pub const DEFAULT_OTLP_EXPORT_INTERVAL_MS: u64 = 10_000;
pub const DEFAULT_OTLP_SERVICE_NAME: &str = "t3-server";

pub fn expand_home_path(value: &str, home_directory: &str) -> String {
    if value.is_empty() {
        return value.to_string();
    }
    if value == "~" {
        return home_directory.to_string();
    }
    if let Some(rest) = value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"))
    {
        return std::path::Path::new(home_directory)
            .join(rest)
            .to_string_lossy()
            .into_owned();
    }
    value.to_string()
}

pub fn resolve_base_dir(raw: Option<&str>, home_directory: &str) -> String {
    let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return join_server_path(&[home_directory, ".t3"]);
    };
    let expanded = expand_home_path(raw, home_directory);
    std::path::Path::new(&expanded)
        .to_string_lossy()
        .into_owned()
}

pub fn collect_uint8_stream_text(
    chunks: &[Vec<u8>],
    max_bytes: Option<usize>,
    truncated_marker: Option<&str>,
) -> CollectedUint8StreamText {
    let max_bytes = max_bytes.unwrap_or(usize::MAX);
    let truncated_marker = truncated_marker.unwrap_or("");
    let mut bytes = 0usize;
    let mut collected = Vec::new();
    let mut truncated = false;

    for chunk in chunks {
        if truncated {
            break;
        }
        let remaining_bytes = max_bytes.saturating_sub(bytes);
        if remaining_bytes == 0 {
            truncated = true;
            break;
        }
        let take = chunk.len().min(remaining_bytes);
        collected.extend_from_slice(&chunk[..take]);
        bytes += take;
        truncated = chunk.len() > remaining_bytes;
    }

    let mut text = String::from_utf8_lossy(&collected).into_owned();
    if truncated {
        text.push_str(truncated_marker);
    }
    CollectedUint8StreamText {
        text,
        truncated,
        bytes,
    }
}

pub fn derive_server_paths(base_dir: &str, dev_url: Option<&str>) -> ServerDerivedPaths {
    let state_dir =
        join_server_path(&[base_dir, if dev_url.is_some() { "dev" } else { "userdata" }]);
    let logs_dir = join_server_path(&[&state_dir, "logs"]);
    let provider_logs_dir = join_server_path(&[&logs_dir, "provider"]);
    let provider_status_cache_dir = join_server_path(&[base_dir, "caches"]);
    ServerDerivedPaths {
        state_dir: state_dir.clone(),
        db_path: join_server_path(&[&state_dir, "state.sqlite"]),
        keybindings_config_path: join_server_path(&[&state_dir, "keybindings.json"]),
        settings_path: join_server_path(&[&state_dir, "settings.json"]),
        provider_status_cache_dir,
        worktrees_dir: join_server_path(&[base_dir, "worktrees"]),
        attachments_dir: join_server_path(&[&state_dir, "attachments"]),
        logs_dir: logs_dir.clone(),
        server_log_path: join_server_path(&[&logs_dir, "server.log"]),
        server_trace_path: join_server_path(&[&logs_dir, "server.trace.ndjson"]),
        provider_logs_dir: provider_logs_dir.clone(),
        provider_event_log_path: join_server_path(&[&provider_logs_dir, "events.log"]),
        terminal_logs_dir: join_server_path(&[&logs_dir, "terminals"]),
        anonymous_id_path: join_server_path(&[&state_dir, "anonymous-id"]),
        environment_id_path: join_server_path(&[&state_dir, "environment-id"]),
        server_runtime_state_path: join_server_path(&[&state_dir, "server-runtime.json"]),
        secrets_dir: join_server_path(&[&state_dir, "secrets"]),
    }
}

fn join_server_path(parts: &[&str]) -> String {
    let mut output = std::path::PathBuf::new();
    for part in parts {
        output.push(part);
    }
    output.to_string_lossy().into_owned()
}

pub fn server_directories_to_ensure(paths: &ServerDerivedPaths) -> Vec<String> {
    vec![
        paths.state_dir.clone(),
        paths.logs_dir.clone(),
        paths.provider_logs_dir.clone(),
        paths.terminal_logs_dir.clone(),
        paths.attachments_dir.clone(),
        paths.worktrees_dir.clone(),
        parent_directory(&paths.keybindings_config_path),
        parent_directory(&paths.settings_path),
        paths.provider_status_cache_dir.clone(),
        parent_directory(&paths.anonymous_id_path),
        parent_directory(&paths.server_runtime_state_path),
    ]
}

pub fn auto_bootstrap_default_model_selection() -> ServerModelSelection {
    ServerModelSelection {
        instance_id: "codex".to_string(),
        model: DEFAULT_SERVER_MODEL.to_string(),
    }
}

pub fn server_package_metadata() -> ServerPackageMetadata {
    ServerPackageMetadata {
        name: SERVER_PACKAGE_NAME,
        upstream_name: UPSTREAM_SERVER_PACKAGE_NAME,
        version: SERVER_PACKAGE_VERSION,
        license: "MIT",
        repository_url: "https://github.com/pingdotgg/t3code",
        repository_directory: "apps/server",
        bin_name: SERVER_PACKAGE_NAME,
        bin_entry: SERVER_PACKAGE_BIN_ENTRY,
        module_type: "module",
        files: vec!["dist"],
        engines_node: SERVER_NODE_ENGINES,
    }
}

pub fn server_package_script_commands() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("dev", "node --watch src/bin.ts"),
        ("build", "node scripts/cli.ts build"),
        ("build:bundle", "tsdown"),
        ("start", "node dist/bin.mjs"),
        ("typecheck", "tsc --noEmit"),
        ("test", "vitest run"),
        (
            "test:process-reaper",
            "vitest run src/server.test.ts src/provider/Layers/ClaudeAdapter.test.ts src/provider/Layers/ProviderSessionDirectory.test.ts src/provider/Layers/ProviderSessionReaper.test.ts src/provider/Layers/CodexAdapter.test.ts",
        ),
    ])
}

pub fn server_runtime_dependencies() -> Vec<&'static str> {
    vec![
        "@anthropic-ai/claude-agent-sdk",
        "@effect/platform-bun",
        "@effect/platform-node",
        "@effect/sql-sqlite-bun",
        "@opencode-ai/sdk",
        "@pierre/diffs",
        "effect",
        "node-pty",
        "open",
    ]
}

pub fn server_workspace_dev_dependencies() -> Vec<&'static str> {
    vec![
        "@t3tools/contracts",
        "@t3tools/shared",
        "@t3tools/tailscale",
        "@t3tools/web",
        "effect-acp",
        "effect-codex-app-server",
    ]
}

pub fn server_build_tool_config() -> ServerBuildToolConfig {
    ServerBuildToolConfig {
        entry: vec!["src/bin.ts"],
        formats: vec!["esm", "cjs"],
        out_dir: "dist",
        sourcemap: true,
        clean: true,
        banner: "#!/usr/bin/env node\n",
        no_external_prefixes: vec!["@t3tools/", "effect-acp"],
    }
}

pub fn server_test_runtime_config() -> ServerTestRuntimeConfig {
    ServerTestRuntimeConfig {
        file_parallelism: false,
        test_timeout_ms: 60_000,
        hook_timeout_ms: 60_000,
    }
}

pub fn server_build_command_plan(verbose: bool, windows: bool) -> ServerBuildCommandPlan {
    ServerBuildCommandPlan {
        command: "node",
        args: vec!["--run", "build:bundle"],
        cwd_relative: "apps/server",
        windows_shell: windows,
        client_source: "apps/web/dist",
        client_target: "apps/server/dist/client",
        dev_icon_overrides_after_client_copy: true,
    }
    .with_verbose_stdio(verbose)
}

impl ServerBuildCommandPlan {
    fn with_verbose_stdio(self, _verbose: bool) -> Self {
        self
    }
}

pub fn server_publish_command_plan(
    access: &'static str,
    tag: &'static str,
    provenance: bool,
    dry_run: bool,
    windows: bool,
) -> ServerPublishCommandPlan {
    let mut publish_args = vec!["publish", "--access", access, "--tag", tag];
    let mut optional_args = Vec::new();
    if provenance {
        optional_args.push("--provenance");
    }
    if dry_run {
        optional_args.push("--dry-run");
    }
    publish_args.extend(optional_args.iter().copied());

    ServerPublishCommandPlan {
        required_assets: vec!["dist/bin.mjs", "dist/client/index.html"],
        package_backup_suffix: ".bak",
        stripped_fields: vec!["devDependencies", "scripts"],
        resolved_fields: vec!["dependencies", "overrides"],
        publish_command: "npm",
        publish_args,
        optional_args,
        publish_icon_overrides_with_restore: true,
        windows_shell: windows,
    }
}

pub fn server_cli_script_plan() -> ServerCliScriptPlan {
    ServerCliScriptPlan {
        root_command: "cli",
        description: "T3 server build & publish CLI.",
        subcommands: vec!["build", "publish"],
        provided_layers: vec![
            "Logger.consolePretty",
            "@effect/platform-node/NodeServices.layer",
            "@effect/platform-node/NodeRuntime.runMain",
        ],
    }
}

pub fn resolve_startup_welcome_base(cwd: &str) -> ServerWelcomeBase {
    let project_name = cwd
        .split(['/', '\\'])
        .filter(|segment| !segment.is_empty())
        .next_back()
        .unwrap_or("project");
    ServerWelcomeBase {
        cwd: cwd.to_string(),
        project_name: project_name.to_string(),
    }
}

pub fn server_command_gate_enqueue_plan(
    state: &ServerCommandReadinessState,
) -> ServerCommandGateEnqueuePlan {
    match state {
        ServerCommandReadinessState::Ready => ServerCommandGateEnqueuePlan {
            run_immediately: true,
            queue_until_ready: false,
            error: None,
        },
        ServerCommandReadinessState::Pending => ServerCommandGateEnqueuePlan {
            run_immediately: false,
            queue_until_ready: true,
            error: None,
        },
        ServerCommandReadinessState::Failed(error) => ServerCommandGateEnqueuePlan {
            run_immediately: false,
            queue_until_ready: false,
            error: Some(error.clone()),
        },
    }
}

pub fn server_runtime_startup_error(message: &str) -> ServerRuntimeStartupErrorPlan {
    ServerRuntimeStartupErrorPlan {
        message: message.to_string(),
    }
}

pub fn publish_server_lifecycle_event(
    current: &ServerLifecycleSnapshotState,
    version: u32,
    event_type: &str,
) -> (ServerLifecycleEvent, ServerLifecycleSnapshotState) {
    let next_sequence = current.sequence + 1;
    let next_event = ServerLifecycleEvent {
        version,
        event_type: event_type.to_string(),
        sequence: next_sequence,
    };
    let mut events = vec![next_event.clone()];
    events.extend(
        current
            .events
            .iter()
            .filter(|event| event.event_type != event_type)
            .cloned(),
    );
    (
        next_event,
        ServerLifecycleSnapshotState {
            sequence: next_sequence,
            events,
        },
    )
}

pub fn runtime_origin_for_config(host: Option<&str>, port: u16) -> String {
    let hostname = match host {
        Some(host) if !is_wildcard_host(Some(host)) => format_host_for_url(host),
        _ => "127.0.0.1".to_string(),
    };
    format!("http://{hostname}:{port}")
}

pub fn make_persisted_server_runtime_state(
    pid: u32,
    host: Option<&str>,
    port: u16,
    started_at: &str,
) -> PersistedServerRuntimeState {
    PersistedServerRuntimeState {
        version: 1,
        pid,
        host: host.map(str::to_string),
        port,
        origin: runtime_origin_for_config(host, port),
        started_at: started_at.to_string(),
    }
}

pub fn resolve_option_precedence<T: Clone>(values: &[Option<T>], default: T) -> T {
    values.iter().find_map(Clone::clone).unwrap_or(default)
}

pub fn resolve_cli_server_precedence_plan(
    mode: ServerRuntimeMode,
    startup_presentation: StartupPresentation,
    dev_url: Option<&str>,
    no_browser_flag: Option<bool>,
    no_browser_env: Option<bool>,
    no_browser_bootstrap: Option<bool>,
    force_auto_bootstrap_project_from_cwd: Option<bool>,
    auto_bootstrap_flag: Option<bool>,
    auto_bootstrap_env: Option<bool>,
    log_websocket_events_flag: Option<bool>,
    log_websocket_events_env: Option<bool>,
    tailscale_serve_enabled_flag: Option<bool>,
    tailscale_serve_enabled_env: Option<bool>,
    tailscale_serve_enabled_bootstrap: Option<bool>,
    tailscale_serve_port_flag: Option<u16>,
    tailscale_serve_port_env: Option<u16>,
    tailscale_serve_port_bootstrap: Option<u16>,
) -> CliServerPrecedencePlan {
    let is_headless = startup_presentation == StartupPresentation::Headless;
    CliServerPrecedencePlan {
        no_browser: resolve_option_precedence(
            &[
                is_headless.then_some(true),
                no_browser_flag,
                no_browser_env,
                no_browser_bootstrap,
            ],
            mode == ServerRuntimeMode::Desktop,
        ),
        auto_bootstrap_project_from_cwd: resolve_option_precedence(
            &[
                force_auto_bootstrap_project_from_cwd,
                is_headless.then_some(false),
                auto_bootstrap_flag,
                auto_bootstrap_env,
            ],
            mode == ServerRuntimeMode::Web,
        ),
        log_websocket_events: resolve_option_precedence(
            &[log_websocket_events_flag, log_websocket_events_env],
            dev_url.is_some(),
        ),
        tailscale_serve_enabled: resolve_option_precedence(
            &[
                tailscale_serve_enabled_flag,
                tailscale_serve_enabled_env,
                tailscale_serve_enabled_bootstrap,
            ],
            false,
        ),
        tailscale_serve_port: resolve_option_precedence(
            &[
                tailscale_serve_port_flag,
                tailscale_serve_port_env,
                tailscale_serve_port_bootstrap,
            ],
            443,
        ),
    }
}

pub fn parse_duration_shorthand_ms(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let split_at = trimmed
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(trimmed.len());
    if split_at == 0 {
        return None;
    }
    let amount = trimmed[..split_at].parse::<u64>().ok()?;
    let unit = trimmed[split_at..].to_ascii_lowercase();
    match unit.as_str() {
        "ms" => Some(amount),
        "s" => Some(amount * 1_000),
        "m" => Some(amount * 60_000),
        "h" => Some(amount * 60 * 60_000),
        "d" => Some(amount * 24 * 60 * 60_000),
        "w" => Some(amount * 7 * 24 * 60 * 60_000),
        _ => None,
    }
}

pub fn normalize_environment_label(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn parse_machine_info_value(raw: &str, key: &str) -> Option<String> {
    for line in raw.split(['\r', '\n']) {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || !trimmed.starts_with(&format!("{key}="))
        {
            continue;
        }
        let value = trimmed[key.len() + 1..].trim();
        if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            return normalize_environment_label(Some(&value[1..value.len() - 1]));
        }
        return normalize_environment_label(Some(value));
    }
    None
}

pub fn resolve_server_environment_label(input: &ServerEnvironmentLabelInput) -> String {
    if input.platform == "darwin" {
        if let Some(label) = normalize_environment_label(input.macos_computer_name.as_deref()) {
            return label;
        }
    }
    if input.platform == "linux" {
        if let Some(machine_info) = input.linux_machine_info.as_deref() {
            if let Some(label) = parse_machine_info_value(machine_info, "PRETTY_HOSTNAME") {
                return label;
            }
        }
        if let Some(label) = normalize_environment_label(input.linux_hostnamectl_pretty.as_deref())
        {
            return label;
        }
    }
    normalize_environment_label(input.hostname.as_deref())
        .or_else(|| normalize_environment_label(Some(&input.cwd_base_name)))
        .unwrap_or_else(|| "T3 environment".to_string())
}

pub fn normalize_persisted_environment_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub fn resolve_server_environment_id_plan(
    persisted_read: Result<Option<&str>, &str>,
    generated_environment_id: &str,
) -> Result<ServerEnvironmentIdPlan, ServerEnvironmentIdReadError> {
    match persisted_read {
        Ok(Some(raw)) => {
            if let Some(environment_id) = normalize_persisted_environment_id(raw) {
                Ok(ServerEnvironmentIdPlan {
                    environment_id,
                    persist_contents: None,
                })
            } else {
                Ok(ServerEnvironmentIdPlan {
                    environment_id: generated_environment_id.to_string(),
                    persist_contents: Some(format!("{generated_environment_id}\n")),
                })
            }
        }
        Ok(None) => Ok(ServerEnvironmentIdPlan {
            environment_id: generated_environment_id.to_string(),
            persist_contents: Some(format!("{generated_environment_id}\n")),
        }),
        Err(detail) => Err(ServerEnvironmentIdReadError {
            detail: detail.to_string(),
        }),
    }
}

pub fn execution_platform_os_from_node_platform(value: &str) -> ExecutionEnvironmentPlatformOs {
    match value {
        "darwin" => ExecutionEnvironmentPlatformOs::Darwin,
        "linux" => ExecutionEnvironmentPlatformOs::Linux,
        "win32" => ExecutionEnvironmentPlatformOs::Windows,
        _ => ExecutionEnvironmentPlatformOs::Unknown,
    }
}

pub fn execution_platform_arch_from_node_arch(value: &str) -> ExecutionEnvironmentPlatformArch {
    match value {
        "arm64" => ExecutionEnvironmentPlatformArch::Arm64,
        "x64" => ExecutionEnvironmentPlatformArch::X64,
        _ => ExecutionEnvironmentPlatformArch::Other,
    }
}

fn path_base_name(path: &str) -> String {
    path.split(['/', '\\'])
        .filter(|segment| !segment.is_empty())
        .next_back()
        .unwrap_or(path)
        .trim()
        .to_string()
}

pub fn make_server_environment_descriptor(
    input: &MakeServerEnvironmentInput,
) -> ExecutionEnvironmentDescriptor {
    let cwd_base_name = path_base_name(&input.cwd);
    let label = resolve_server_environment_label(&ServerEnvironmentLabelInput {
        cwd_base_name,
        platform: input.node_platform.clone(),
        hostname: input.hostname.clone(),
        macos_computer_name: input.macos_computer_name.clone(),
        linux_machine_info: input.linux_machine_info.clone(),
        linux_hostnamectl_pretty: input.linux_hostnamectl_pretty.clone(),
    });
    ExecutionEnvironmentDescriptor {
        environment_id: input.environment_id.clone(),
        label,
        platform: ExecutionEnvironmentPlatform {
            os: execution_platform_os_from_node_platform(&input.node_platform),
            arch: execution_platform_arch_from_node_arch(&input.node_arch),
        },
        server_version: input.server_version.clone(),
        capabilities: ExecutionEnvironmentCapabilities {
            repository_identity: true,
        },
    }
}

pub fn server_logger_layer_plan(log_level: &str) -> ServerLoggerLayerPlan {
    ServerLoggerLayerPlan {
        minimum_log_level: log_level.to_string(),
        logger_names: vec!["consolePretty", "tracerLogger"],
        merge_with_existing: false,
    }
}

pub fn server_cli_command_specs() -> Vec<CliCommandSpec> {
    vec![
        CliCommandSpec {
            path: vec![R3_CLI_NAME],
            description: "Run the R3Code server.",
            handler: "runServerCommand",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "start"],
            description: "Run the R3Code server.",
            handler: "runServerCommand",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "serve"],
            description: "Run the R3Code server without opening a browser and print headless pairing details.",
            handler: "runServerCommand(headless)",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "auth"],
            description: "Manage the local auth control plane for headless deployments.",
            handler: "authCommand",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "auth", "pairing", "create"],
            description: "Issue a new client pairing token.",
            handler: "AuthControlPlane.createPairingLink",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "auth", "pairing", "list"],
            description: "List active client pairing tokens without revealing their secrets.",
            handler: "AuthControlPlane.listPairingLinks",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "auth", "pairing", "revoke"],
            description: "Revoke an active client pairing token.",
            handler: "AuthControlPlane.revokePairingLink",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "auth", "session", "issue"],
            description: "Issue a bearer session token for headless or remote clients.",
            handler: "AuthControlPlane.issueSession",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "auth", "session", "list"],
            description: "List active sessions without revealing bearer tokens.",
            handler: "AuthControlPlane.listSessions",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "auth", "session", "revoke"],
            description: "Revoke an active session.",
            handler: "AuthControlPlane.revokeSession",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "project"],
            description: "Manage projects.",
            handler: "projectCommand",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "project", "add"],
            description: "Add a project.",
            handler: "project.create",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "project", "remove"],
            description: "Remove a project.",
            handler: "project.delete",
        },
        CliCommandSpec {
            path: vec![R3_CLI_NAME, "project", "rename"],
            description: "Rename a project.",
            handler: "project.meta.update",
        },
    ]
}

pub fn server_cli_subcommands() -> Vec<&'static str> {
    vec!["start", "serve", "auth", "project"]
}

pub fn server_command_run_plan(command: Option<&str>) -> ServerCommandRunPlan {
    match command {
        Some("serve") => ServerCommandRunPlan {
            startup_presentation: StartupPresentation::Headless,
            force_auto_bootstrap_project_from_cwd: Some(false),
        },
        _ => ServerCommandRunPlan {
            startup_presentation: StartupPresentation::Browser,
            force_auto_bootstrap_project_from_cwd: None,
        },
    }
}

pub fn project_cli_allows_dev_url(command: &str) -> bool {
    !matches!(command, "add" | "remove" | "rename")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerJavaScriptRuntime {
    Bun,
    Node,
}

pub fn server_runtime_from_bun_present(bun_present: bool) -> ServerJavaScriptRuntime {
    if bun_present {
        ServerJavaScriptRuntime::Bun
    } else {
        ServerJavaScriptRuntime::Node
    }
}

pub fn pty_adapter_layer_name(runtime: ServerJavaScriptRuntime) -> &'static str {
    match runtime {
        ServerJavaScriptRuntime::Bun => "terminal/Layers/BunPTY.layer",
        ServerJavaScriptRuntime::Node => "terminal/Layers/NodePTY.layer",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpServerLayerPlan {
    pub runtime: ServerJavaScriptRuntime,
    pub layer_name: &'static str,
    pub host: Option<String>,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticRequestPathDecision {
    ServeRelativePath(String),
    Invalid { message: &'static str, status: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticAndDevRouteDecision {
    BadRequest {
        body: &'static str,
        status: u16,
    },
    Redirect {
        location: String,
        status: u16,
    },
    ServiceUnavailable {
        body: &'static str,
        status: u16,
    },
    InvalidStaticPath {
        body: &'static str,
        status: u16,
    },
    ServeFile {
        relative_path: String,
        status: u16,
        content_type: &'static str,
    },
    FallbackIndex {
        relative_path: &'static str,
        status: u16,
        content_type: &'static str,
    },
    NotFound {
        body: &'static str,
        status: u16,
    },
    InternalServerError {
        body: &'static str,
        status: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticAndDevRouteFileResponse {
    BadRequest {
        body: &'static str,
        status: u16,
    },
    Redirect {
        location: String,
        status: u16,
    },
    ServiceUnavailable {
        body: &'static str,
        status: u16,
    },
    InvalidStaticPath {
        body: &'static str,
        status: u16,
    },
    File {
        relative_path: String,
        bytes: Vec<u8>,
        status: u16,
        content_type: &'static str,
    },
    FallbackIndex {
        relative_path: &'static str,
        bytes: Vec<u8>,
        status: u16,
        content_type: &'static str,
    },
    NotFound {
        body: &'static str,
        status: u16,
    },
    InternalServerError {
        body: &'static str,
        status: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectFaviconRouteDecision {
    BadRequest {
        body: &'static str,
        status: u16,
    },
    FallbackSvg {
        body: &'static str,
        status: u16,
        content_type: &'static str,
        cache_control: &'static str,
    },
    File {
        path: String,
        status: u16,
        cache_control: &'static str,
    },
    InternalServerError {
        body: &'static str,
        status: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectFaviconRouteFileResponse {
    BadRequest {
        body: &'static str,
        status: u16,
    },
    FallbackSvg {
        body: &'static str,
        status: u16,
        content_type: &'static str,
        cache_control: &'static str,
    },
    File {
        path: PathBuf,
        bytes: Vec<u8>,
        status: u16,
        cache_control: &'static str,
    },
    InternalServerError {
        body: &'static str,
        status: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OtlpTracesProxyDecision {
    NoExportConfigured { status: u16 },
    Export { url: String },
    ExportSucceeded { status: u16 },
    ExportFailed { body: &'static str, status: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserApiCorsRouteDecision {
    Preflight {
        status: u16,
        headers: BTreeMap<&'static str, String>,
    },
    ApplyHeaders {
        headers: BTreeMap<&'static str, String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserApiCorsLayerPlan {
    pub allowed_methods: Vec<&'static str>,
    pub allowed_headers: Vec<&'static str>,
    pub max_age_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerEnvironmentRouteResponse {
    pub descriptor: ExecutionEnvironmentDescriptor,
    pub status: u16,
    pub headers: BTreeMap<String, String>,
}

pub const BROWSER_API_CORS_ALLOWED_METHODS: &[&str] = &["GET", "POST", "OPTIONS"];
pub const BROWSER_API_CORS_ALLOWED_HEADERS: &[&str] =
    &["authorization", "b3", "traceparent", "content-type"];
pub const BROWSER_API_CORS_MAX_AGE_SECONDS: u32 = 600;
pub const OTLP_TRACES_PROXY_PATH: &str = "/api/observability/v1/traces";
pub const PROJECT_FAVICON_CACHE_CONTROL: &str = "public, max-age=3600";
pub const FALLBACK_PROJECT_FAVICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="#6b728080" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" data-fallback="project-favicon"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-8l-2-2H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2Z"/></svg>"##;

pub fn http_server_layer_plan(
    runtime: ServerJavaScriptRuntime,
    host: Option<&str>,
    port: u16,
) -> HttpServerLayerPlan {
    match runtime {
        ServerJavaScriptRuntime::Bun => HttpServerLayerPlan {
            runtime,
            layer_name: "@effect/platform-bun/BunHttpServer.layer",
            host: host.map(str::to_string),
            port,
        },
        ServerJavaScriptRuntime::Node => HttpServerLayerPlan {
            runtime,
            layer_name: "@effect/platform-node/NodeHttpServer.layer(NodeHttp.createServer)",
            host: host.map(str::to_string),
            port,
        },
    }
}

pub fn platform_services_layer_name(runtime: ServerJavaScriptRuntime) -> &'static str {
    match runtime {
        ServerJavaScriptRuntime::Bun => "@effect/platform-bun/BunServices.layer",
        ServerJavaScriptRuntime::Node => "@effect/platform-node/NodeServices.layer",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLayerGroupPlan {
    pub name: &'static str,
    pub layers: Vec<&'static str>,
}

pub fn server_reactor_layer_plan() -> ServerLayerGroupPlan {
    ServerLayerGroupPlan {
        name: "ReactorLayerLive",
        layers: vec![
            "OrchestrationReactorLive",
            "ProviderRuntimeIngestionLive",
            "ProviderCommandReactorLive",
            "CheckpointReactorLive",
            "ThreadDeletionReactorLive",
            "RuntimeReceiptBusLive",
        ],
    }
}

pub fn server_provider_layer_plan() -> ServerLayerGroupPlan {
    ServerLayerGroupPlan {
        name: "ProviderLayerLive",
        layers: vec![
            "ProviderServiceLive",
            "ProviderAdapterRegistryLive",
            "ProviderSessionDirectoryLive",
            "ProviderSessionRuntimeRepositoryLive",
        ],
    }
}

pub fn server_runtime_core_dependencies_plan() -> ServerLayerGroupPlan {
    ServerLayerGroupPlan {
        name: "RuntimeCoreDependenciesLive",
        layers: vec![
            "ReactorLayerLive",
            "CheckpointingLayerLive",
            "SourceControlProviderRegistryLayerLive",
            "GitLayerLive",
            "VcsLayerLive",
            "ProviderRuntimeLayerLive",
            "TerminalLayerLive",
            "PersistenceLayerLive",
            "KeybindingsLive",
            "ProviderRegistryLive",
            "ProviderInstanceRegistryHydrationLive",
            "ProviderEventLoggersLive",
            "OpenCodeRuntimeLive",
            "ServerSettingsLive",
            "WorkspaceLayerLive",
            "ProjectFaviconResolverLive",
            "RepositoryIdentityResolverLive",
            "ServerEnvironmentLive",
            "AuthLayerLive",
        ],
    }
}

pub fn server_runtime_dependencies_plan() -> ServerLayerGroupPlan {
    ServerLayerGroupPlan {
        name: "RuntimeDependenciesLive",
        layers: vec![
            "RuntimeCoreDependenciesLive",
            "ProcessDiagnostics.layer",
            "TraceDiagnostics.layer",
            "AnalyticsServiceLayerLive",
            "OpenLive",
            "ServerLifecycleEventsLive",
            "@t3tools/shared/Net.layer",
        ],
    }
}

pub fn server_routes_layer_names() -> Vec<&'static str> {
    vec![
        "authBearerBootstrapRouteLayer",
        "authBootstrapRouteLayer",
        "authClientsRevokeOthersRouteLayer",
        "authClientsRevokeRouteLayer",
        "authClientsRouteLayer",
        "authPairingLinksRevokeRouteLayer",
        "authPairingLinksRouteLayer",
        "authPairingCredentialRouteLayer",
        "authSessionRouteLayer",
        "authWebSocketTokenRouteLayer",
        "attachmentsRouteLayer",
        "orchestrationDispatchRouteLayer",
        "orchestrationSnapshotRouteLayer",
        "otlpTracesProxyRouteLayer",
        "projectFaviconRouteLayer",
        "serverEnvironmentRouteLayer",
        "staticAndDevRouteLayer",
        "websocketRpcRouteLayer",
        "browserApiCorsLayer",
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketRpcRoutePlan {
    pub method: &'static str,
    pub path: &'static str,
    pub authenticate_upgrade: &'static str,
    pub rpc_http_effect: &'static str,
    pub disable_tracing: bool,
    pub session_lifecycle_scope: &'static str,
    pub auth_error_handler: &'static str,
    pub provided_layers: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketRpcSessionLifecyclePlan {
    pub session_id: String,
    pub acquire_effect: &'static str,
    pub use_effect: &'static str,
    pub release_effect: &'static str,
}

pub fn websocket_rpc_route_plan() -> WebSocketRpcRoutePlan {
    WebSocketRpcRoutePlan {
        method: "GET",
        path: "/ws",
        authenticate_upgrade: "serverAuth.authenticateWebSocketUpgrade(request)",
        rpc_http_effect: "RpcServer.toHttpEffectWebsocket(WsRpcGroup)",
        disable_tracing: true,
        session_lifecycle_scope: "acquireUseRelease markConnected -> rpcWebSocketHttpEffect -> markDisconnected",
        auth_error_handler: "respondToAuthError",
        provided_layers: vec![
            "makeWsRpcLayer(session.sessionId)",
            "RpcSerialization.layerJson",
            "ProviderMaintenanceRunner.layer",
            "SourceControlDiscoveryLayer.layer",
            "SourceControlProviderRegistry.layer",
            "AzureDevOpsCli.layer",
            "BitbucketApi.layer",
            "GitHubCli.layer",
            "GitLabCli.layer",
            "GitVcsDriver.layer",
            "VcsDriverRegistry.layer",
            "VcsProjectConfig.layer",
            "VcsProcess.layer",
        ],
    }
}

pub fn websocket_rpc_session_lifecycle_plan(session_id: &str) -> WebSocketRpcSessionLifecyclePlan {
    WebSocketRpcSessionLifecyclePlan {
        session_id: session_id.to_string(),
        acquire_effect: "sessions.markConnected(session.sessionId)",
        use_effect: "rpcWebSocketHttpEffect",
        release_effect: "sessions.markDisconnected(session.sessionId)",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerApplicationLayerPlan {
    pub fix_path_before_layer_build: bool,
    pub route_disable_logger: bool,
    pub mark_http_listening: bool,
    pub persist_runtime_state: bool,
    pub clear_runtime_state_on_release: bool,
    pub tailscale_serve_enabled: bool,
    pub tailscale_local_host: Option<&'static str>,
    pub provided_layers: Vec<&'static str>,
}

pub fn server_application_layer_plan(
    log_websocket_events: bool,
    tailscale_serve_enabled: bool,
) -> ServerApplicationLayerPlan {
    ServerApplicationLayerPlan {
        fix_path_before_layer_build: true,
        route_disable_logger: !log_websocket_events,
        mark_http_listening: true,
        persist_runtime_state: true,
        clear_runtime_state_on_release: true,
        tailscale_serve_enabled,
        tailscale_local_host: tailscale_serve_enabled.then_some("127.0.0.1"),
        provided_layers: vec![
            "RuntimeServicesLive",
            "HttpServerLive",
            "ObservabilityLive",
            "FetchHttpClient.layer",
            "VcsProcess.layer",
            "PlatformServicesLive",
        ],
    }
}

pub fn encode_persisted_server_runtime_state(state: &PersistedServerRuntimeState) -> String {
    let mut value = serde_json::json!({
        "version": state.version,
        "pid": state.pid,
        "port": state.port,
        "origin": state.origin,
        "startedAt": state.started_at,
    });
    if let Some(host) = &state.host {
        value["host"] = serde_json::json!(host);
    }
    format!("{}\n", serde_json::to_string(&value).unwrap())
}

pub fn resolve_bootstrap_fd_path(fd: i32, platform: &str) -> Option<String> {
    match platform {
        "linux" => Some(format!("/proc/self/fd/{fd}")),
        "win32" => None,
        _ => Some(format!("/dev/fd/{fd}")),
    }
}

pub fn is_unavailable_bootstrap_fd_error_code(code: &str) -> bool {
    matches!(code, "EBADF" | "ENOENT")
}

pub fn is_bootstrap_fd_path_duplication_error_code(code: &str) -> bool {
    matches!(code, "ENXIO" | "EINVAL" | "EPERM")
}

pub fn decode_bootstrap_envelope_line(line: Option<&str>) -> BootstrapReadOutcome {
    let Some(line) = line else {
        return BootstrapReadOutcome::None;
    };
    match serde_json::from_str::<serde_json::Value>(line) {
        Ok(value) => BootstrapReadOutcome::Some(value),
        Err(_) => BootstrapReadOutcome::Error {
            message: "Failed to decode bootstrap envelope.",
        },
    }
}

fn parent_directory(path: &str) -> String {
    std::path::Path::new(path)
        .parent()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string())
}

pub fn plan_atomic_write_file_string(file_path: &str, contents: &str) -> AtomicWriteFileStringPlan {
    let path = std::path::Path::new(file_path);
    let target_directory = path
        .parent()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());
    let basename = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    AtomicWriteFileStringPlan {
        file_path: file_path.to_string(),
        contents_len: contents.len(),
        target_directory,
        temp_directory_prefix: format!("{basename}."),
        temp_file_suffix: ".tmp",
        make_target_directory_recursive: true,
        scoped_temp_directory: true,
        final_operation: "rename",
    }
}

pub fn is_loopback_host(host: Option<&str>) -> bool {
    match host {
        None | Some("") => true,
        Some("localhost" | "127.0.0.1" | "::1" | "[::1]") => true,
        Some(host) => host.starts_with("127."),
    }
}

pub fn is_loopback_hostname_for_dev_redirect(hostname: &str) -> bool {
    let normalized = hostname
        .trim()
        .to_ascii_lowercase()
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .map(str::to_string)
        .unwrap_or_else(|| hostname.trim().to_ascii_lowercase());
    matches!(normalized.as_str(), "127.0.0.1" | "::1" | "localhost")
}

pub fn resolve_dev_redirect_url(dev_url: &str, request_url: &str) -> String {
    let origin = url_origin(dev_url);
    let request_path = url_path_search_hash(request_url);
    format!("{origin}{request_path}")
}

pub fn normalize_static_request_path(pathname: &str) -> StaticRequestPathDecision {
    let static_request_path = if pathname == "/" {
        "/index.html"
    } else {
        pathname
    };
    let raw_static_relative_path = static_request_path
        .trim_start_matches(['/', '\\'])
        .replace('\\', "/");
    let has_raw_leading_parent_segment = raw_static_relative_path.starts_with("..");
    let static_relative_path = normalize_relative_url_path(&raw_static_relative_path)
        .trim_start_matches('/')
        .to_string();
    let has_path_traversal_segment = static_relative_path.starts_with("..");
    if static_relative_path.is_empty()
        || has_raw_leading_parent_segment
        || has_path_traversal_segment
        || static_relative_path.contains('\0')
    {
        return StaticRequestPathDecision::Invalid {
            message: "Invalid static file path",
            status: 400,
        };
    }
    if path_has_extension(&static_relative_path) {
        StaticRequestPathDecision::ServeRelativePath(static_relative_path)
    } else {
        StaticRequestPathDecision::ServeRelativePath(format!("{static_relative_path}/index.html"))
    }
}

pub fn static_and_dev_route_decision(
    request_url: Option<&str>,
    dev_url: Option<&str>,
    static_dir_configured: bool,
    file_exists: bool,
    file_read_ok: bool,
    index_read_ok: bool,
) -> StaticAndDevRouteDecision {
    let Some(request_url) = request_url else {
        return StaticAndDevRouteDecision::BadRequest {
            body: "Bad Request",
            status: 400,
        };
    };
    if let Some(dev_url) = dev_url.filter(|url| !url.is_empty()) {
        if is_loopback_hostname_for_dev_redirect(&url_hostname(request_url)) {
            return StaticAndDevRouteDecision::Redirect {
                location: resolve_dev_redirect_url(dev_url, request_url),
                status: 302,
            };
        }
    }
    if !static_dir_configured {
        return StaticAndDevRouteDecision::ServiceUnavailable {
            body: "No static directory configured and no dev URL set.",
            status: 503,
        };
    }
    let pathname = url_pathname(request_url);
    let relative_path = match normalize_static_request_path(&pathname) {
        StaticRequestPathDecision::ServeRelativePath(relative_path) => relative_path,
        StaticRequestPathDecision::Invalid { message, status } => {
            return StaticAndDevRouteDecision::InvalidStaticPath {
                body: message,
                status,
            };
        }
    };
    if file_exists {
        if !file_read_ok {
            return StaticAndDevRouteDecision::InternalServerError {
                body: "Internal Server Error",
                status: 500,
            };
        }
        return StaticAndDevRouteDecision::ServeFile {
            content_type: static_content_type_for_path(&relative_path),
            relative_path,
            status: 200,
        };
    }
    if index_read_ok {
        return StaticAndDevRouteDecision::FallbackIndex {
            relative_path: "index.html",
            status: 200,
            content_type: "text/html; charset=utf-8",
        };
    }
    StaticAndDevRouteDecision::NotFound {
        body: "Not Found",
        status: 404,
    }
}

pub fn static_and_dev_route_file_response(
    request_url: Option<&str>,
    dev_url: Option<&str>,
    static_dir: Option<&Path>,
) -> StaticAndDevRouteFileResponse {
    let Some(request_url) = request_url else {
        return StaticAndDevRouteFileResponse::BadRequest {
            body: "Bad Request",
            status: 400,
        };
    };
    if let Some(dev_url) = dev_url.filter(|url| !url.is_empty()) {
        if is_loopback_hostname_for_dev_redirect(&url_hostname(request_url)) {
            return StaticAndDevRouteFileResponse::Redirect {
                location: resolve_dev_redirect_url(dev_url, request_url),
                status: 302,
            };
        }
    }
    let Some(static_dir) = static_dir else {
        return StaticAndDevRouteFileResponse::ServiceUnavailable {
            body: "No static directory configured and no dev URL set.",
            status: 503,
        };
    };
    let pathname = url_pathname(request_url);
    let relative_path = match normalize_static_request_path(&pathname) {
        StaticRequestPathDecision::ServeRelativePath(relative_path) => relative_path,
        StaticRequestPathDecision::Invalid { message, status } => {
            return StaticAndDevRouteFileResponse::InvalidStaticPath {
                body: message,
                status,
            };
        }
    };
    let static_root = normalize_filesystem_path(&absolutize_filesystem_path(static_dir));
    let file_path = normalize_filesystem_path(&static_root.join(Path::new(&relative_path)));
    if !file_path.starts_with(&static_root) {
        return StaticAndDevRouteFileResponse::InvalidStaticPath {
            body: "Invalid static file path",
            status: 400,
        };
    }
    if fs::metadata(&file_path)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
    {
        return match fs::read(&file_path) {
            Ok(bytes) => StaticAndDevRouteFileResponse::File {
                content_type: static_content_type_for_path(&relative_path),
                relative_path,
                bytes,
                status: 200,
            },
            Err(_) => StaticAndDevRouteFileResponse::InternalServerError {
                body: "Internal Server Error",
                status: 500,
            },
        };
    }
    let fallback_index_path = normalize_filesystem_path(&static_root.join("index.html"));
    match fs::read(fallback_index_path) {
        Ok(bytes) => StaticAndDevRouteFileResponse::FallbackIndex {
            relative_path: "index.html",
            bytes,
            status: 200,
            content_type: "text/html; charset=utf-8",
        },
        Err(_) => StaticAndDevRouteFileResponse::NotFound {
            body: "Not Found",
            status: 404,
        },
    }
}

fn absolutize_filesystem_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn normalize_filesystem_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

pub fn static_content_type_for_path(path: &str) -> &'static str {
    match path
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript",
        Some("css") => "text/css",
        Some("json") | Some("map") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}

pub fn browser_api_cors_headers() -> BTreeMap<&'static str, String> {
    BTreeMap::from([
        ("access-control-allow-origin", "*".to_string()),
        (
            "access-control-allow-methods",
            BROWSER_API_CORS_ALLOWED_METHODS.join(", "),
        ),
        (
            "access-control-allow-headers",
            BROWSER_API_CORS_ALLOWED_HEADERS.join(", "),
        ),
    ])
}

pub fn browser_api_cors_layer_plan() -> BrowserApiCorsLayerPlan {
    BrowserApiCorsLayerPlan {
        allowed_methods: BROWSER_API_CORS_ALLOWED_METHODS.to_vec(),
        allowed_headers: BROWSER_API_CORS_ALLOWED_HEADERS.to_vec(),
        max_age_seconds: BROWSER_API_CORS_MAX_AGE_SECONDS,
    }
}

pub fn apply_browser_api_cors_headers(
    mut headers: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    for (key, value) in browser_api_cors_headers() {
        headers.insert(key.to_string(), value);
    }
    headers
}

pub fn server_environment_route_response(
    descriptor: ExecutionEnvironmentDescriptor,
) -> ServerEnvironmentRouteResponse {
    ServerEnvironmentRouteResponse {
        descriptor,
        status: 200,
        headers: apply_browser_api_cors_headers(BTreeMap::new()),
    }
}

pub fn browser_api_cors_route_decision(method: &str) -> BrowserApiCorsRouteDecision {
    let mut headers = browser_api_cors_headers();
    if method.eq_ignore_ascii_case("OPTIONS") {
        headers.insert(
            "access-control-max-age",
            BROWSER_API_CORS_MAX_AGE_SECONDS.to_string(),
        );
        return BrowserApiCorsRouteDecision::Preflight {
            status: 204,
            headers,
        };
    }
    BrowserApiCorsRouteDecision::ApplyHeaders { headers }
}

pub fn project_favicon_route_decision(
    request_url: Option<&str>,
    resolved_favicon_path: Option<&str>,
    file_response_failed: bool,
) -> ProjectFaviconRouteDecision {
    let Some(request_url) = request_url else {
        return ProjectFaviconRouteDecision::BadRequest {
            body: "Bad Request",
            status: 400,
        };
    };
    if query_param(request_url, "cwd")
        .filter(|value| !value.is_empty())
        .is_none()
    {
        return ProjectFaviconRouteDecision::BadRequest {
            body: "Missing cwd parameter",
            status: 400,
        };
    }
    let Some(path) = resolved_favicon_path.filter(|path| !path.is_empty()) else {
        return ProjectFaviconRouteDecision::FallbackSvg {
            body: FALLBACK_PROJECT_FAVICON_SVG,
            status: 200,
            content_type: "image/svg+xml",
            cache_control: PROJECT_FAVICON_CACHE_CONTROL,
        };
    };
    if file_response_failed {
        return ProjectFaviconRouteDecision::InternalServerError {
            body: "Internal Server Error",
            status: 500,
        };
    }
    ProjectFaviconRouteDecision::File {
        path: path.to_string(),
        status: 200,
        cache_control: PROJECT_FAVICON_CACHE_CONTROL,
    }
}

pub fn project_favicon_route_file_response(
    request_url: Option<&str>,
    resolved_favicon_path: Option<&Path>,
) -> ProjectFaviconRouteFileResponse {
    let Some(request_url) = request_url else {
        return ProjectFaviconRouteFileResponse::BadRequest {
            body: "Bad Request",
            status: 400,
        };
    };
    if query_param(request_url, "cwd")
        .filter(|value| !value.is_empty())
        .is_none()
    {
        return ProjectFaviconRouteFileResponse::BadRequest {
            body: "Missing cwd parameter",
            status: 400,
        };
    }
    let Some(path) = resolved_favicon_path else {
        return ProjectFaviconRouteFileResponse::FallbackSvg {
            body: FALLBACK_PROJECT_FAVICON_SVG,
            status: 200,
            content_type: "image/svg+xml",
            cache_control: PROJECT_FAVICON_CACHE_CONTROL,
        };
    };
    match fs::read(path) {
        Ok(bytes) => ProjectFaviconRouteFileResponse::File {
            path: path.to_path_buf(),
            bytes,
            status: 200,
            cache_control: PROJECT_FAVICON_CACHE_CONTROL,
        },
        Err(_) => ProjectFaviconRouteFileResponse::InternalServerError {
            body: "Internal Server Error",
            status: 500,
        },
    }
}

pub fn otlp_traces_proxy_export_decision(otlp_traces_url: Option<&str>) -> OtlpTracesProxyDecision {
    match otlp_traces_url.filter(|url| !url.is_empty()) {
        Some(url) => OtlpTracesProxyDecision::Export {
            url: url.to_string(),
        },
        None => OtlpTracesProxyDecision::NoExportConfigured { status: 204 },
    }
}

pub fn otlp_traces_proxy_response_after_export(export_status_ok: bool) -> OtlpTracesProxyDecision {
    if export_status_ok {
        OtlpTracesProxyDecision::ExportSucceeded { status: 204 }
    } else {
        OtlpTracesProxyDecision::ExportFailed {
            body: "Trace export failed.",
            status: 502,
        }
    }
}

pub fn is_wildcard_host(host: Option<&str>) -> bool {
    matches!(host, Some("0.0.0.0" | "::" | "[::]"))
}

pub fn format_host_for_url(host: &str) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    }
}

pub fn normalize_server_host(host: &str) -> &str {
    host.strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host)
}

fn is_ipv4_family(family: &str) -> bool {
    family == "IPv4" || family == "4"
}

fn is_ipv6_family(family: &str) -> bool {
    family == "IPv6" || family == "6"
}

fn url_origin(url: &str) -> &str {
    let Some(scheme_index) = url.find("://") else {
        return url
            .split(['/', '?', '#'])
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or(url);
    };
    let authority_start = scheme_index + 3;
    let path_offset = url[authority_start..]
        .find(['/', '?', '#'])
        .map(|index| authority_start + index)
        .unwrap_or(url.len());
    &url[..path_offset]
}

fn url_path_search_hash(url: &str) -> String {
    let path_start = url
        .find("://")
        .map(|scheme_index| {
            let authority_start = scheme_index + 3;
            url[authority_start..]
                .find(['/', '?', '#'])
                .map(|index| authority_start + index)
                .unwrap_or(url.len())
        })
        .unwrap_or(0);
    let tail = &url[path_start..];
    if tail.is_empty() {
        "/".to_string()
    } else if tail.starts_with('?') || tail.starts_with('#') {
        format!("/{tail}")
    } else {
        tail.to_string()
    }
}

fn url_hostname(url: &str) -> String {
    let origin = url_origin(url);
    let authority = origin
        .split_once("://")
        .map(|(_, authority)| authority)
        .unwrap_or(origin);
    let host_port = authority
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(authority);
    if let Some(rest) = host_port.strip_prefix('[') {
        return rest
            .split_once(']')
            .map(|(host, _)| host.to_string())
            .unwrap_or_else(|| host_port.to_string());
    }
    host_port
        .split_once(':')
        .map(|(host, _)| host.to_string())
        .unwrap_or_else(|| host_port.to_string())
}

fn url_pathname(url: &str) -> String {
    let path_search_hash = url_path_search_hash(url);
    let pathname = path_search_hash
        .split(['?', '#'])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("/");
    pathname.to_string()
}

fn query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1.split('#').next().unwrap_or("");
    for pair in query.split('&') {
        let (pair_key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if pair_key == key {
            return Some(value.to_string());
        }
    }
    None
}

fn normalize_relative_url_path(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if segments.last().is_some_and(|last| *last != "..") {
                    segments.pop();
                } else {
                    segments.push("..");
                }
            }
            _ => segments.push(segment),
        }
    }
    segments.join("/")
}

fn path_has_extension(path: &str) -> bool {
    path.rsplit('/').next().is_some_and(|name| {
        name.rsplit_once('.')
            .is_some_and(|(stem, ext)| !stem.is_empty() && !ext.is_empty())
    })
}

pub fn resolve_headless_connection_host(
    host: Option<&str>,
    interfaces: &BTreeMap<String, Vec<ServerNetworkInterfaceInfo>>,
) -> String {
    let Some(host) = host.filter(|host| !host.is_empty()) else {
        return "localhost".to_string();
    };
    if !is_wildcard_host(Some(host)) {
        return normalize_server_host(host).to_string();
    }

    let entries = interfaces.values().flat_map(|entries| entries.iter());
    if let Some(ipv4) = entries
        .clone()
        .find(|entry| !entry.internal && is_ipv4_family(&entry.family))
    {
        return ipv4.address.clone();
    }
    entries
        .filter(|entry| !entry.internal && is_ipv6_family(&entry.family))
        .map(|entry| normalize_server_host(&entry.address).to_string())
        .next()
        .unwrap_or_else(|| "localhost".to_string())
}

pub fn resolve_headless_connection_string(
    host: Option<&str>,
    port: u16,
    interfaces: &BTreeMap<String, Vec<ServerNetworkInterfaceInfo>>,
) -> String {
    let connection_host = resolve_headless_connection_host(host, interfaces);
    format!("http://{}:{port}", format_host_for_url(&connection_host))
}

pub fn resolve_listening_port(address_port: Option<u16>, fallback_port: u16) -> u16 {
    address_port.unwrap_or(fallback_port)
}

pub fn build_pairing_url(connection_string: &str, token: &str) -> String {
    let base = connection_string
        .split(['?', '#'])
        .next()
        .unwrap_or(connection_string)
        .trim_end_matches('/');
    format!("{base}/pair#token={}", percent_encode_query_value(token))
}

fn percent_encode_query_value(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            b' ' => encoded.push('+'),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

pub fn render_terminal_qr_code(value: &str, margin: usize) -> String {
    crate::shared::render_terminal_qr_code(value, margin)
        .expect("pairing URL must fit in a QR code")
}

pub fn format_headless_serve_output(access_info: &HeadlessServeAccessInfo) -> String {
    [
        "T3 Code server is ready.".to_string(),
        format!("Connection string: {}", access_info.connection_string),
        format!("Token: {}", access_info.token),
        format!("Pairing URL: {}", access_info.pairing_url),
        String::new(),
        render_terminal_qr_code(&access_info.pairing_url, 2),
        String::new(),
    ]
    .join("\n")
}

fn serialize_optional_fields(values: &[Option<String>]) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| value.as_ref())
        .filter(|value| !value.is_empty())
        .cloned()
        .collect()
}

pub fn format_auth_client_metadata(metadata: &AuthClientMetadata) -> String {
    let details = serialize_optional_fields(&[
        metadata.label.clone(),
        metadata
            .device_type
            .as_ref()
            .filter(|value| value.as_str() != "unknown")
            .cloned(),
        metadata.os.clone(),
        metadata.browser.clone(),
        metadata.ip_address.clone(),
    ]);
    if details.is_empty() {
        "unlabeled client".to_string()
    } else {
        details.join(" | ")
    }
}

pub fn format_issued_pairing_credential(
    credential: &IssuedPairingCredential,
    json_output: bool,
    base_url: Option<&str>,
) -> String {
    let pair_url = base_url
        .filter(|base_url| !base_url.is_empty())
        .map(|base_url| build_pairing_url(base_url, &credential.credential));
    if json_output {
        let mut output = serde_json::json!({
            "id": credential.id,
            "credential": credential.credential,
            "role": credential.role,
            "expiresAt": credential.expires_at,
        });
        if let Some(label) = &credential.label {
            output["label"] = serde_json::json!(label);
        }
        if let Some(pair_url) = pair_url {
            output["pairUrl"] = serde_json::json!(pair_url);
        }
        return format!("{}\n", serde_json::to_string_pretty(&output).unwrap());
    }
    [
        format!("Issued client pairing token {}.", credential.id),
        format!("Token: {}", credential.credential),
        pair_url
            .map(|url| format!("Pair URL: {url}"))
            .unwrap_or_default(),
        format!("Expires at: {}", credential.expires_at),
    ]
    .into_iter()
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>()
    .join("\n")
        + "\n"
}

pub fn format_pairing_credential_list(
    credentials: &[IssuedPairingCredential],
    json_output: bool,
) -> String {
    if json_output {
        let values = credentials
            .iter()
            .map(|credential| {
                let mut value = serde_json::json!({
                    "id": credential.id,
                    "role": credential.role,
                    "createdAt": credential.created_at,
                    "expiresAt": credential.expires_at,
                });
                if let Some(label) = &credential.label {
                    value["label"] = serde_json::json!(label);
                }
                value
            })
            .collect::<Vec<_>>();
        return format!("{}\n", serde_json::to_string_pretty(&values).unwrap());
    }
    if credentials.is_empty() {
        return "No active pairing credentials.\n".to_string();
    }
    credentials
        .iter()
        .map(|credential| {
            [
                format!(
                    "{}{}",
                    credential.id,
                    credential
                        .label
                        .as_ref()
                        .map(|label| format!(" ({label})"))
                        .unwrap_or_default()
                ),
                format!("  role: {}", credential.role),
                format!("  created: {}", credential.created_at),
                format!("  expires: {}", credential.expires_at),
            ]
            .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
        + "\n"
}

pub fn format_issued_session(
    session: &IssuedBearerSession,
    json_output: bool,
    token_only: bool,
) -> String {
    if token_only {
        return format!("{}\n", session.token);
    }
    if json_output {
        return format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "sessionId": session.session_id,
                "token": session.token,
                "method": session.method,
                "role": session.role,
                "subject": session.subject,
                "client": {
                    "label": session.client.label,
                    "deviceType": session.client.device_type,
                    "os": session.client.os,
                    "browser": session.client.browser,
                    "ipAddress": session.client.ip_address,
                },
                "expiresAt": session.expires_at,
            }))
            .unwrap()
        );
    }
    [
        format!(
            "Issued {} bearer session {}.",
            session.role, session.session_id
        ),
        format!("Token: {}", session.token),
        format!("Subject: {}", session.subject),
        format!("Client: {}", format_auth_client_metadata(&session.client)),
        format!("Expires at: {}", session.expires_at),
    ]
    .join("\n")
        + "\n"
}

pub fn format_session_list(sessions: &[AuthClientSession], json_output: bool) -> String {
    if json_output {
        let values = sessions
            .iter()
            .map(|session| {
                serde_json::json!({
                    "sessionId": session.session_id,
                    "method": session.method,
                    "role": session.role,
                    "subject": session.subject,
                    "client": {
                        "label": session.client.label,
                        "deviceType": session.client.device_type,
                        "os": session.client.os,
                        "browser": session.client.browser,
                        "ipAddress": session.client.ip_address,
                    },
                    "connected": session.connected,
                    "issuedAt": session.issued_at,
                    "expiresAt": session.expires_at,
                    "lastConnectedAt": session.last_connected_at,
                })
            })
            .collect::<Vec<_>>();
        return format!("{}\n", serde_json::to_string_pretty(&values).unwrap());
    }
    if sessions.is_empty() {
        return "No active sessions.\n".to_string();
    }
    sessions
        .iter()
        .map(|session| {
            [
                format!(
                    "{} [{}]{}",
                    session.session_id,
                    session.role,
                    if session.connected { " connected" } else { "" }
                ),
                format!("  method: {}", session.method),
                format!("  subject: {}", session.subject),
                format!("  client: {}", format_auth_client_metadata(&session.client)),
                format!("  issued: {}", session.issued_at),
                format!(
                    "  last connected: {}",
                    session.last_connected_at.as_deref().unwrap_or("never")
                ),
                format!("  expires: {}", session.expires_at),
            ]
            .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
        + "\n"
}

pub fn provider_environment_secret_name(instance_id: &str, name: &str) -> String {
    format!(
        "provider-env-{}-{}",
        base64url_no_pad(instance_id.as_bytes()),
        base64url_no_pad(name.as_bytes())
    )
}

fn base64url_no_pad(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::new();
    let mut index = 0;
    while index < bytes.len() {
        let first = bytes[index];
        let second = bytes.get(index + 1).copied();
        let third = bytes.get(index + 2).copied();
        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(
            TABLE[(((first & 0b0000_0011) << 4) | (second.unwrap_or(0) >> 4)) as usize] as char,
        );
        if let Some(second) = second {
            output.push(
                TABLE[(((second & 0b0000_1111) << 2) | (third.unwrap_or(0) >> 6)) as usize] as char,
            );
        }
        if let Some(third) = third {
            output.push(TABLE[(third & 0b0011_1111) as usize] as char);
        }
        index += 3;
    }
    output
}

pub fn redact_provider_environment_variable(
    variable: &ProviderEnvironmentVariable,
) -> ProviderEnvironmentVariable {
    if !variable.sensitive {
        return ProviderEnvironmentVariable {
            value_redacted: None,
            ..variable.clone()
        };
    }
    ProviderEnvironmentVariable {
        value: String::new(),
        value_redacted: (variable.value_redacted == Some(true) || !variable.value.is_empty())
            .then_some(true),
        ..variable.clone()
    }
}

pub fn redact_server_settings_for_client(
    settings: &ServerSettingsForClient,
) -> ServerSettingsForClient {
    ServerSettingsForClient {
        provider_instances: settings
            .provider_instances
            .iter()
            .map(|(instance_id, instance)| {
                (
                    instance_id.clone(),
                    ProviderInstanceConfig {
                        environment: instance
                            .environment
                            .iter()
                            .map(redact_provider_environment_variable)
                            .collect(),
                    },
                )
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_static_test_dir(name: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("r3code-static-{name}-{unique}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn ports_server_expand_home_path() {
        let home = if cfg!(windows) {
            "C:\\Users\\bunny"
        } else {
            "/Users/bunny"
        };
        assert_eq!(expand_home_path("", home), "");
        assert_eq!(expand_home_path("/absolute/path", home), "/absolute/path");
        assert_eq!(expand_home_path("relative/path", home), "relative/path");
        assert_eq!(expand_home_path("some~weird~path", home), "some~weird~path");
        assert_eq!(expand_home_path("~", home), home);
        assert_eq!(
            expand_home_path("~/.codex-work", home),
            std::path::Path::new(home)
                .join(".codex-work")
                .to_string_lossy()
                .into_owned()
        );
        assert_eq!(
            expand_home_path("~\\.codex", home),
            std::path::Path::new(home)
                .join(".codex")
                .to_string_lossy()
                .into_owned()
        );
        assert_eq!(expand_home_path("~alice/foo", home), "~alice/foo");
        assert_eq!(
            resolve_base_dir(None, home),
            std::path::Path::new(home)
                .join(".t3")
                .to_string_lossy()
                .into_owned()
        );
        assert_eq!(
            resolve_base_dir(Some(" ~/.r3  "), home),
            std::path::Path::new(home)
                .join(".r3")
                .to_string_lossy()
                .into_owned()
        );
    }

    #[test]
    fn ports_server_stream_text_collection_and_atomic_write_plan() {
        assert_eq!(
            collect_uint8_stream_text(&[b"hello ".to_vec(), b"world".to_vec()], None, None),
            CollectedUint8StreamText {
                text: "hello world".to_string(),
                bytes: 11,
                truncated: false,
            }
        );
        assert_eq!(
            collect_uint8_stream_text(
                &[b"abcdef".to_vec(), b"ghij".to_vec()],
                Some(5),
                Some("[truncated]"),
            ),
            CollectedUint8StreamText {
                text: "abcde[truncated]".to_string(),
                bytes: 5,
                truncated: true,
            }
        );

        let plan = plan_atomic_write_file_string("C:/state/settings.json", "{\"ok\":true}");
        assert_eq!(plan.file_path, "C:/state/settings.json");
        assert_eq!(plan.contents_len, 11);
        assert!(
            plan.target_directory
                .replace('\\', "/")
                .ends_with("C:/state")
        );
        assert_eq!(plan.temp_directory_prefix, "settings.json.");
        assert_eq!(plan.temp_file_suffix, ".tmp");
        assert!(plan.make_target_directory_recursive);
        assert!(plan.scoped_temp_directory);
        assert_eq!(plan.final_operation, "rename");
    }

    #[test]
    fn ports_server_cli_auth_formatting_contracts() {
        let pairing = IssuedPairingCredential {
            id: "pairing-1".to_string(),
            credential: "secret-pairing-token".to_string(),
            label: Some("Phone".to_string()),
            role: "client".to_string(),
            subject: "one-time-token".to_string(),
            created_at: "2026-04-08T09:00:00.000Z".to_string(),
            expires_at: "2026-04-08T10:00:00.000Z".to_string(),
        };
        let issued_pairing =
            format_issued_pairing_credential(&pairing, false, Some("https://example.com"));
        assert!(issued_pairing.contains("secret-pairing-token"));
        assert!(issued_pairing.contains("https://example.com/pair#token=secret-pairing-token"));

        let listed_pairings = format_pairing_credential_list(&[pairing], false);
        assert!(listed_pairings.contains("pairing-1"));
        assert!(listed_pairings.contains("(Phone)"));
        assert!(!listed_pairings.contains("secret-pairing-token"));
        assert_eq!(
            format_pairing_credential_list(&[], false),
            "No active pairing credentials.\n"
        );

        let client = AuthClientMetadata {
            label: Some("deploy-bot".to_string()),
            device_type: Some("bot".to_string()),
            ..AuthClientMetadata::default()
        };
        assert_eq!(format_auth_client_metadata(&client), "deploy-bot | bot");
        assert_eq!(
            format_auth_client_metadata(&AuthClientMetadata::default()),
            "unlabeled client"
        );
        let issued_session = IssuedBearerSession {
            session_id: "session-1".to_string(),
            token: "secret-session-token".to_string(),
            method: "bearer-session-token".to_string(),
            role: "owner".to_string(),
            subject: "cli-issued-session".to_string(),
            client: client.clone(),
            expires_at: "2026-04-08T10:00:00.000Z".to_string(),
        };
        let issued_output = format_issued_session(&issued_session, false, false);
        assert!(issued_output.contains("secret-session-token"));
        assert_eq!(
            format_issued_session(&issued_session, false, true),
            "secret-session-token\n"
        );

        let listed_output = format_session_list(
            &[AuthClientSession {
                session_id: "session-1".to_string(),
                method: "bearer-session-token".to_string(),
                role: "owner".to_string(),
                subject: "cli-issued-session".to_string(),
                client,
                connected: false,
                issued_at: "2026-04-08T09:00:00.000Z".to_string(),
                expires_at: "2026-04-08T10:00:00.000Z".to_string(),
                last_connected_at: None,
            }],
            false,
        );
        assert!(listed_output.contains("session-1 [owner]"));
        assert!(listed_output.contains("last connected: never"));
        assert!(!listed_output.contains("secret-session-token"));
        assert_eq!(format_session_list(&[], false), "No active sessions.\n");
    }

    #[test]
    fn ports_server_config_derived_paths_and_defaults() {
        assert_eq!(DEFAULT_SERVER_PORT, 3773);
        assert_eq!(DEFAULT_TRACE_MAX_BYTES, 10 * 1024 * 1024);
        assert_eq!(DEFAULT_TRACE_MAX_FILES, 10);
        assert_eq!(DEFAULT_TRACE_BATCH_WINDOW_MS, 200);
        assert_eq!(DEFAULT_OTLP_EXPORT_INTERVAL_MS, 10_000);
        assert_eq!(DEFAULT_OTLP_SERVICE_NAME, "t3-server");
        assert_eq!(ServerRuntimeMode::Web.as_str(), "web");
        assert_eq!(ServerRuntimeMode::Desktop.as_str(), "desktop");
        assert_eq!(StartupPresentation::Browser.as_str(), "browser");
        assert_eq!(StartupPresentation::Headless.as_str(), "headless");

        let paths = derive_server_paths("C:/r3-home", None);
        let normalized_state = paths.state_dir.replace('\\', "/");
        assert!(normalized_state.ends_with("C:/r3-home/userdata"));
        assert!(
            paths
                .db_path
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/state.sqlite")
        );
        assert!(
            paths
                .keybindings_config_path
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/keybindings.json")
        );
        assert!(
            paths
                .settings_path
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/settings.json")
        );
        assert!(
            paths
                .provider_status_cache_dir
                .replace('\\', "/")
                .ends_with("C:/r3-home/caches")
        );
        assert!(
            paths
                .worktrees_dir
                .replace('\\', "/")
                .ends_with("C:/r3-home/worktrees")
        );
        assert!(
            paths
                .attachments_dir
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/attachments")
        );
        assert!(
            paths
                .server_log_path
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/logs/server.log")
        );
        assert!(
            paths
                .server_trace_path
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/logs/server.trace.ndjson")
        );
        assert!(
            paths
                .provider_event_log_path
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/logs/provider/events.log")
        );
        assert!(
            paths
                .terminal_logs_dir
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/logs/terminals")
        );
        assert!(
            paths
                .anonymous_id_path
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/anonymous-id")
        );
        assert!(
            paths
                .environment_id_path
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/environment-id")
        );
        assert!(
            paths
                .server_runtime_state_path
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/server-runtime.json")
        );
        assert!(
            paths
                .secrets_dir
                .replace('\\', "/")
                .ends_with("C:/r3-home/userdata/secrets")
        );

        let dev_paths = derive_server_paths("C:/r3-home", Some("http://127.0.0.1:5173"));
        assert!(
            dev_paths
                .state_dir
                .replace('\\', "/")
                .ends_with("C:/r3-home/dev")
        );

        let ensure = server_directories_to_ensure(&paths)
            .into_iter()
            .map(|path| path.replace('\\', "/"))
            .collect::<Vec<_>>();
        assert!(
            ensure
                .iter()
                .any(|path| path.ends_with("C:/r3-home/userdata"))
        );
        assert!(
            ensure
                .iter()
                .any(|path| path.ends_with("C:/r3-home/userdata/logs"))
        );
        assert!(
            ensure
                .iter()
                .any(|path| path.ends_with("C:/r3-home/userdata/attachments"))
        );
        assert!(
            ensure
                .iter()
                .any(|path| path.ends_with("C:/r3-home/worktrees"))
        );
        assert!(
            ensure
                .iter()
                .any(|path| path.ends_with("C:/r3-home/caches"))
        );
    }

    #[test]
    fn ports_server_bootstrap_envelope_contracts() {
        assert_eq!(
            resolve_bootstrap_fd_path(3, "linux").as_deref(),
            Some("/proc/self/fd/3")
        );
        assert_eq!(
            resolve_bootstrap_fd_path(3, "darwin").as_deref(),
            Some("/dev/fd/3")
        );
        assert_eq!(resolve_bootstrap_fd_path(3, "win32"), None);
        assert!(is_unavailable_bootstrap_fd_error_code("EBADF"));
        assert!(is_unavailable_bootstrap_fd_error_code("ENOENT"));
        assert!(!is_unavailable_bootstrap_fd_error_code("EACCES"));
        assert!(is_bootstrap_fd_path_duplication_error_code("ENXIO"));
        assert!(is_bootstrap_fd_path_duplication_error_code("EINVAL"));
        assert!(is_bootstrap_fd_path_duplication_error_code("EPERM"));
        assert!(!is_bootstrap_fd_path_duplication_error_code("ENOENT"));

        assert_eq!(
            decode_bootstrap_envelope_line(None),
            BootstrapReadOutcome::None
        );
        assert_eq!(
            decode_bootstrap_envelope_line(Some("{\"mode\":\"desktop\"}")),
            BootstrapReadOutcome::Some(serde_json::json!({ "mode": "desktop" }))
        );
        assert_eq!(
            decode_bootstrap_envelope_line(Some("not json")),
            BootstrapReadOutcome::Error {
                message: "Failed to decode bootstrap envelope.",
            }
        );
    }

    #[test]
    fn ports_server_runtime_startup_command_gate_contracts() {
        assert_eq!(
            auto_bootstrap_default_model_selection(),
            ServerModelSelection {
                instance_id: "codex".to_string(),
                model: "gpt-5.4".to_string(),
            }
        );
        assert_eq!(DEFAULT_PROVIDER_INTERACTION_MODE, "default");
        assert_eq!(
            resolve_startup_welcome_base("/tmp/startup-project"),
            ServerWelcomeBase {
                cwd: "/tmp/startup-project".to_string(),
                project_name: "startup-project".to_string(),
            }
        );
        assert_eq!(
            resolve_startup_welcome_base("C:\\work\\r3code"),
            ServerWelcomeBase {
                cwd: "C:\\work\\r3code".to_string(),
                project_name: "r3code".to_string(),
            }
        );
        assert_eq!(
            resolve_startup_welcome_base("/"),
            ServerWelcomeBase {
                cwd: "/".to_string(),
                project_name: "project".to_string(),
            }
        );
        assert_eq!(
            server_command_gate_enqueue_plan(&ServerCommandReadinessState::Pending),
            ServerCommandGateEnqueuePlan {
                run_immediately: false,
                queue_until_ready: true,
                error: None,
            }
        );
        assert_eq!(
            server_command_gate_enqueue_plan(&ServerCommandReadinessState::Ready),
            ServerCommandGateEnqueuePlan {
                run_immediately: true,
                queue_until_ready: false,
                error: None,
            }
        );
        let error = server_runtime_startup_error("startup failed");
        assert_eq!(
            server_command_gate_enqueue_plan(&ServerCommandReadinessState::Failed(error.clone())),
            ServerCommandGateEnqueuePlan {
                run_immediately: false,
                queue_until_ready: false,
                error: Some(error),
            }
        );
    }

    #[test]
    fn ports_server_lifecycle_events_and_runtime_state_contracts() {
        let initial = ServerLifecycleSnapshotState::default();
        let (welcome, after_welcome) = publish_server_lifecycle_event(&initial, 1, "welcome");
        assert_eq!(welcome.sequence, 1);
        assert_eq!(after_welcome.sequence, 1);
        assert_eq!(
            after_welcome
                .events
                .iter()
                .map(|event| event.event_type.as_str())
                .collect::<Vec<_>>(),
            vec!["welcome"]
        );
        let (ready, after_ready) = publish_server_lifecycle_event(&after_welcome, 1, "ready");
        assert_eq!(ready.sequence, 2);
        assert_eq!(after_ready.sequence, 2);
        assert_eq!(
            after_ready
                .events
                .iter()
                .map(|event| event.event_type.as_str())
                .collect::<Vec<_>>(),
            vec!["ready", "welcome"]
        );
        let (_, after_second_welcome) = publish_server_lifecycle_event(&after_ready, 1, "welcome");
        assert_eq!(after_second_welcome.sequence, 3);
        assert_eq!(
            after_second_welcome
                .events
                .iter()
                .map(|event| (event.event_type.as_str(), event.sequence))
                .collect::<Vec<_>>(),
            vec![("welcome", 3), ("ready", 2)]
        );

        assert_eq!(
            runtime_origin_for_config(Some("0.0.0.0"), 3773),
            "http://127.0.0.1:3773"
        );
        assert_eq!(
            runtime_origin_for_config(Some("::1"), 3773),
            "http://[::1]:3773"
        );
        let state = make_persisted_server_runtime_state(
            1234,
            Some("127.0.0.1"),
            3773,
            "2026-01-01T00:00:00.000Z",
        );
        assert_eq!(state.version, 1);
        assert_eq!(state.origin, "http://127.0.0.1:3773");
        assert_eq!(
            encode_persisted_server_runtime_state(&state),
            "{\"host\":\"127.0.0.1\",\"origin\":\"http://127.0.0.1:3773\",\"pid\":1234,\"port\":3773,\"startedAt\":\"2026-01-01T00:00:00.000Z\",\"version\":1}\n"
        );
    }

    #[test]
    fn ports_server_cli_config_precedence_contracts() {
        assert_eq!(
            resolve_option_precedence(&[None, Some("env"), Some("bootstrap")], "default"),
            "env"
        );
        assert_eq!(
            resolve_cli_server_precedence_plan(
                ServerRuntimeMode::Web,
                StartupPresentation::Browser,
                Some("http://127.0.0.1:5173"),
                Some(false),
                Some(true),
                Some(true),
                None,
                Some(false),
                Some(true),
                Some(false),
                Some(true),
                None,
                Some(true),
                Some(false),
                None,
                Some(8443),
                Some(443),
            ),
            CliServerPrecedencePlan {
                no_browser: false,
                auto_bootstrap_project_from_cwd: false,
                log_websocket_events: false,
                tailscale_serve_enabled: true,
                tailscale_serve_port: 8443,
            }
        );
        assert_eq!(
            resolve_cli_server_precedence_plan(
                ServerRuntimeMode::Web,
                StartupPresentation::Headless,
                None,
                Some(false),
                Some(false),
                Some(false),
                None,
                Some(true),
                Some(true),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ),
            CliServerPrecedencePlan {
                no_browser: true,
                auto_bootstrap_project_from_cwd: false,
                log_websocket_events: false,
                tailscale_serve_enabled: false,
                tailscale_serve_port: 443,
            }
        );
        assert_eq!(
            resolve_cli_server_precedence_plan(
                ServerRuntimeMode::Desktop,
                StartupPresentation::Browser,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .no_browser,
            true
        );
        assert_eq!(parse_duration_shorthand_ms("5m"), Some(300_000));
        assert_eq!(parse_duration_shorthand_ms("1h"), Some(3_600_000));
        assert_eq!(parse_duration_shorthand_ms("30d"), Some(2_592_000_000));
        assert_eq!(parse_duration_shorthand_ms(""), None);
        assert_eq!(parse_duration_shorthand_ms("wat"), None);
    }

    #[test]
    fn ports_server_environment_label_contracts() {
        assert_eq!(
            normalize_environment_label(Some("  Julius's MacBook Pro \n")).as_deref(),
            Some("Julius's MacBook Pro")
        );
        assert_eq!(normalize_environment_label(Some("   ")), None);
        assert_eq!(
            parse_machine_info_value(
                "PRETTY_HOSTNAME=\"Build Agent 01\"\nICON_NAME=\"computer-vm\"\n",
                "PRETTY_HOSTNAME",
            )
            .as_deref(),
            Some("Build Agent 01")
        );
        assert_eq!(
            parse_machine_info_value("PRETTY_HOSTNAME='Runner 02'\n", "PRETTY_HOSTNAME").as_deref(),
            Some("Runner 02")
        );
        assert_eq!(
            resolve_server_environment_label(&ServerEnvironmentLabelInput {
                cwd_base_name: "t3code".to_string(),
                platform: "win32".to_string(),
                hostname: Some("macbook-pro".to_string()),
                macos_computer_name: None,
                linux_machine_info: None,
                linux_hostnamectl_pretty: None,
            }),
            "macbook-pro"
        );
        assert_eq!(
            resolve_server_environment_label(&ServerEnvironmentLabelInput {
                cwd_base_name: "t3code".to_string(),
                platform: "darwin".to_string(),
                hostname: Some("macbook-pro".to_string()),
                macos_computer_name: Some(" Julius's MacBook Pro \n".to_string()),
                linux_machine_info: None,
                linux_hostnamectl_pretty: None,
            }),
            "Julius's MacBook Pro"
        );
        assert_eq!(
            resolve_server_environment_label(&ServerEnvironmentLabelInput {
                cwd_base_name: "t3code".to_string(),
                platform: "linux".to_string(),
                hostname: Some("buildbox".to_string()),
                macos_computer_name: None,
                linux_machine_info: Some(
                    "PRETTY_HOSTNAME=\"Build Agent 01\"\nICON_NAME=\"computer-vm\"\n".to_string(),
                ),
                linux_hostnamectl_pretty: Some("CI Runner\n".to_string()),
            }),
            "Build Agent 01"
        );
        assert_eq!(
            resolve_server_environment_label(&ServerEnvironmentLabelInput {
                cwd_base_name: "t3code".to_string(),
                platform: "linux".to_string(),
                hostname: Some("   ".to_string()),
                macos_computer_name: None,
                linux_machine_info: None,
                linux_hostnamectl_pretty: Some(" ".to_string()),
            }),
            "t3code"
        );
    }

    #[test]
    fn ports_server_environment_descriptor_contracts() {
        assert_eq!(UPSTREAM_SERVER_VERSION, "0.0.23");
        assert_eq!(
            normalize_persisted_environment_id(" persisted-env \n").as_deref(),
            Some("persisted-env")
        );
        assert_eq!(normalize_persisted_environment_id(" \n\t"), None);
        assert_eq!(
            resolve_server_environment_id_plan(Ok(Some("persisted-env\n")), "generated-env")
                .unwrap(),
            ServerEnvironmentIdPlan {
                environment_id: "persisted-env".to_string(),
                persist_contents: None,
            }
        );
        assert_eq!(
            resolve_server_environment_id_plan(Ok(None), "generated-env").unwrap(),
            ServerEnvironmentIdPlan {
                environment_id: "generated-env".to_string(),
                persist_contents: Some("generated-env\n".to_string()),
            }
        );
        assert_eq!(
            resolve_server_environment_id_plan(Ok(Some("  ")), "generated-env").unwrap(),
            ServerEnvironmentIdPlan {
                environment_id: "generated-env".to_string(),
                persist_contents: Some("generated-env\n".to_string()),
            }
        );
        assert_eq!(
            resolve_server_environment_id_plan(Err("permission denied"), "generated-env")
                .unwrap_err(),
            ServerEnvironmentIdReadError {
                detail: "permission denied".to_string(),
            }
        );

        let descriptor = make_server_environment_descriptor(&MakeServerEnvironmentInput {
            cwd: "C:/work/r3code".to_string(),
            environment_id: "env-123".to_string(),
            node_platform: "win32".to_string(),
            node_arch: "x64".to_string(),
            server_version: UPSTREAM_SERVER_VERSION.to_string(),
            hostname: Some("windows-box".to_string()),
            macos_computer_name: None,
            linux_machine_info: None,
            linux_hostnamectl_pretty: None,
        });
        assert_eq!(descriptor.environment_id, "env-123");
        assert_eq!(descriptor.label, "windows-box");
        assert_eq!(descriptor.platform.os.as_str(), "windows");
        assert_eq!(descriptor.platform.arch.as_str(), "x64");
        assert_eq!(descriptor.server_version, "0.0.23");
        assert!(descriptor.capabilities.repository_identity);

        let linux_descriptor = make_server_environment_descriptor(&MakeServerEnvironmentInput {
            cwd: "/home/bunny/t3code".to_string(),
            environment_id: "env-linux".to_string(),
            node_platform: "linux".to_string(),
            node_arch: "arm64".to_string(),
            server_version: UPSTREAM_SERVER_VERSION.to_string(),
            hostname: Some("fallback-host".to_string()),
            macos_computer_name: None,
            linux_machine_info: Some("PRETTY_HOSTNAME=\"CI Runner\"\n".to_string()),
            linux_hostnamectl_pretty: None,
        });
        assert_eq!(linux_descriptor.label, "CI Runner");
        assert_eq!(
            linux_descriptor.platform.os,
            ExecutionEnvironmentPlatformOs::Linux
        );
        assert_eq!(
            linux_descriptor.platform.arch,
            ExecutionEnvironmentPlatformArch::Arm64
        );
        assert_eq!(
            execution_platform_os_from_node_platform("freebsd"),
            ExecutionEnvironmentPlatformOs::Unknown
        );
        assert_eq!(
            execution_platform_arch_from_node_arch("ia32"),
            ExecutionEnvironmentPlatformArch::Other
        );
        assert_eq!(
            server_environment_route_response(descriptor.clone()),
            ServerEnvironmentRouteResponse {
                descriptor,
                status: 200,
                headers: BTreeMap::from([
                    ("access-control-allow-origin".to_string(), "*".to_string()),
                    (
                        "access-control-allow-methods".to_string(),
                        "GET, POST, OPTIONS".to_string()
                    ),
                    (
                        "access-control-allow-headers".to_string(),
                        "authorization, b3, traceparent, content-type".to_string()
                    ),
                ]),
            }
        );
    }

    #[test]
    fn ports_server_logger_layer_contracts() {
        assert_eq!(
            server_logger_layer_plan("Debug"),
            ServerLoggerLayerPlan {
                minimum_log_level: "Debug".to_string(),
                logger_names: vec!["consolePretty", "tracerLogger"],
                merge_with_existing: false,
            }
        );
        assert_eq!(server_logger_layer_plan("Error").minimum_log_level, "Error");
    }

    #[test]
    fn ports_server_layer_composition_contracts() {
        assert_eq!(
            server_runtime_from_bun_present(true),
            ServerJavaScriptRuntime::Bun
        );
        assert_eq!(
            server_runtime_from_bun_present(false),
            ServerJavaScriptRuntime::Node
        );
        assert_eq!(
            pty_adapter_layer_name(ServerJavaScriptRuntime::Bun),
            "terminal/Layers/BunPTY.layer"
        );
        assert_eq!(
            pty_adapter_layer_name(ServerJavaScriptRuntime::Node),
            "terminal/Layers/NodePTY.layer"
        );
        assert_eq!(
            http_server_layer_plan(ServerJavaScriptRuntime::Node, Some("127.0.0.1"), 3773),
            HttpServerLayerPlan {
                runtime: ServerJavaScriptRuntime::Node,
                layer_name: "@effect/platform-node/NodeHttpServer.layer(NodeHttp.createServer)",
                host: Some("127.0.0.1".to_string()),
                port: 3773,
            }
        );
        assert_eq!(
            http_server_layer_plan(ServerJavaScriptRuntime::Bun, None, 3773).layer_name,
            "@effect/platform-bun/BunHttpServer.layer"
        );
        assert_eq!(
            platform_services_layer_name(ServerJavaScriptRuntime::Node),
            "@effect/platform-node/NodeServices.layer"
        );

        assert_eq!(
            server_reactor_layer_plan().layers,
            vec![
                "OrchestrationReactorLive",
                "ProviderRuntimeIngestionLive",
                "ProviderCommandReactorLive",
                "CheckpointReactorLive",
                "ThreadDeletionReactorLive",
                "RuntimeReceiptBusLive",
            ]
        );
        assert_eq!(
            server_provider_layer_plan().layers,
            vec![
                "ProviderServiceLive",
                "ProviderAdapterRegistryLive",
                "ProviderSessionDirectoryLive",
                "ProviderSessionRuntimeRepositoryLive",
            ]
        );
        assert!(
            server_runtime_core_dependencies_plan()
                .layers
                .contains(&"ProviderInstanceRegistryHydrationLive")
        );
        assert!(
            server_runtime_core_dependencies_plan()
                .layers
                .contains(&"ProviderEventLoggersLive")
        );
        assert!(
            server_runtime_dependencies_plan()
                .layers
                .contains(&"AnalyticsServiceLayerLive")
        );

        let routes = server_routes_layer_names();
        assert!(routes.contains(&"authBootstrapRouteLayer"));
        assert!(routes.contains(&"attachmentsRouteLayer"));
        assert!(routes.contains(&"orchestrationDispatchRouteLayer"));
        assert!(routes.contains(&"orchestrationSnapshotRouteLayer"));
        assert!(routes.contains(&"websocketRpcRouteLayer"));
        assert_eq!(routes.last(), Some(&"browserApiCorsLayer"));

        assert_eq!(
            websocket_rpc_route_plan(),
            WebSocketRpcRoutePlan {
                method: "GET",
                path: "/ws",
                authenticate_upgrade: "serverAuth.authenticateWebSocketUpgrade(request)",
                rpc_http_effect: "RpcServer.toHttpEffectWebsocket(WsRpcGroup)",
                disable_tracing: true,
                session_lifecycle_scope: "acquireUseRelease markConnected -> rpcWebSocketHttpEffect -> markDisconnected",
                auth_error_handler: "respondToAuthError",
                provided_layers: vec![
                    "makeWsRpcLayer(session.sessionId)",
                    "RpcSerialization.layerJson",
                    "ProviderMaintenanceRunner.layer",
                    "SourceControlDiscoveryLayer.layer",
                    "SourceControlProviderRegistry.layer",
                    "AzureDevOpsCli.layer",
                    "BitbucketApi.layer",
                    "GitHubCli.layer",
                    "GitLabCli.layer",
                    "GitVcsDriver.layer",
                    "VcsDriverRegistry.layer",
                    "VcsProjectConfig.layer",
                    "VcsProcess.layer",
                ],
            }
        );

        assert_eq!(
            websocket_rpc_session_lifecycle_plan("session-1"),
            WebSocketRpcSessionLifecyclePlan {
                session_id: "session-1".to_string(),
                acquire_effect: "sessions.markConnected(session.sessionId)",
                use_effect: "rpcWebSocketHttpEffect",
                release_effect: "sessions.markDisconnected(session.sessionId)",
            }
        );

        assert_eq!(
            server_application_layer_plan(false, true),
            ServerApplicationLayerPlan {
                fix_path_before_layer_build: true,
                route_disable_logger: true,
                mark_http_listening: true,
                persist_runtime_state: true,
                clear_runtime_state_on_release: true,
                tailscale_serve_enabled: true,
                tailscale_local_host: Some("127.0.0.1"),
                provided_layers: vec![
                    "RuntimeServicesLive",
                    "HttpServerLive",
                    "ObservabilityLive",
                    "FetchHttpClient.layer",
                    "VcsProcess.layer",
                    "PlatformServicesLive",
                ],
            }
        );
        assert!(!server_application_layer_plan(true, false).route_disable_logger);
        assert_eq!(
            server_application_layer_plan(true, false).tailscale_local_host,
            None
        );
    }

    #[test]
    fn ports_server_cli_command_topology_contracts() {
        assert_eq!(R3_CLI_NAME, "r3");
        assert_eq!(
            server_cli_subcommands(),
            vec!["start", "serve", "auth", "project"]
        );
        let specs = server_cli_command_specs();
        assert_eq!(
            specs
                .iter()
                .filter(|spec| spec.path.len() == 2)
                .map(|spec| spec.path[1])
                .collect::<Vec<_>>(),
            vec!["start", "serve", "auth", "project"]
        );
        assert!(specs.iter().any(|spec| {
            spec.path == vec![R3_CLI_NAME, "auth", "pairing", "create"]
                && spec.handler == "AuthControlPlane.createPairingLink"
        }));
        assert!(specs.iter().any(|spec| {
            spec.path == vec![R3_CLI_NAME, "auth", "session", "issue"]
                && spec.handler == "AuthControlPlane.issueSession"
        }));
        assert!(specs.iter().any(|spec| {
            spec.path == vec![R3_CLI_NAME, "project", "add"] && spec.handler == "project.create"
        }));
        assert_eq!(
            server_command_run_plan(Some("serve")),
            ServerCommandRunPlan {
                startup_presentation: StartupPresentation::Headless,
                force_auto_bootstrap_project_from_cwd: Some(false),
            }
        );
        assert_eq!(
            server_command_run_plan(Some("start")),
            ServerCommandRunPlan {
                startup_presentation: StartupPresentation::Browser,
                force_auto_bootstrap_project_from_cwd: None,
            }
        );
        assert!(!project_cli_allows_dev_url("add"));
        assert!(!project_cli_allows_dev_url("remove"));
        assert!(!project_cli_allows_dev_url("rename"));
    }

    #[test]
    fn ports_server_package_build_publish_and_test_config_contracts() {
        assert_eq!(
            server_package_metadata(),
            ServerPackageMetadata {
                name: "r3",
                upstream_name: "t3",
                version: "0.0.23",
                license: "MIT",
                repository_url: "https://github.com/pingdotgg/t3code",
                repository_directory: "apps/server",
                bin_name: "r3",
                bin_entry: "./dist/bin.mjs",
                module_type: "module",
                files: vec!["dist"],
                engines_node: "^22.16 || ^23.11 || >=24.10",
            }
        );
        assert_eq!(
            server_package_script_commands()["build"],
            "node scripts/cli.ts build"
        );
        assert_eq!(server_package_script_commands()["build:bundle"], "tsdown");
        assert!(server_runtime_dependencies().contains(&"node-pty"));
        assert!(server_runtime_dependencies().contains(&"open"));
        assert!(server_workspace_dev_dependencies().contains(&"effect-codex-app-server"));

        assert_eq!(
            server_build_tool_config(),
            ServerBuildToolConfig {
                entry: vec!["src/bin.ts"],
                formats: vec!["esm", "cjs"],
                out_dir: "dist",
                sourcemap: true,
                clean: true,
                banner: "#!/usr/bin/env node\n",
                no_external_prefixes: vec!["@t3tools/", "effect-acp"],
            }
        );
        assert_eq!(
            server_test_runtime_config(),
            ServerTestRuntimeConfig {
                file_parallelism: false,
                test_timeout_ms: 60_000,
                hook_timeout_ms: 60_000,
            }
        );

        let build = server_build_command_plan(false, true);
        assert_eq!(build.command, "node");
        assert_eq!(build.args, vec!["--run", "build:bundle"]);
        assert_eq!(build.client_source, "apps/web/dist");
        assert_eq!(build.client_target, "apps/server/dist/client");
        assert!(build.dev_icon_overrides_after_client_copy);
        assert!(build.windows_shell);

        let publish = server_publish_command_plan("public", "latest", true, true, true);
        assert_eq!(
            publish.required_assets,
            vec!["dist/bin.mjs", "dist/client/index.html"]
        );
        assert_eq!(
            publish.publish_args,
            vec![
                "publish",
                "--access",
                "public",
                "--tag",
                "latest",
                "--provenance",
                "--dry-run"
            ]
        );
        assert!(publish.publish_icon_overrides_with_restore);
        assert_eq!(publish.stripped_fields, vec!["devDependencies", "scripts"]);
        assert_eq!(
            server_cli_script_plan().provided_layers,
            vec![
                "Logger.consolePretty",
                "@effect/platform-node/NodeServices.layer",
                "@effect/platform-node/NodeRuntime.runMain",
            ]
        );
    }

    #[test]
    fn ports_server_settings_secret_redaction_contracts() {
        assert_eq!(
            provider_environment_secret_name("codex_personal", "OPENROUTER_API_KEY"),
            "provider-env-Y29kZXhfcGVyc29uYWw-T1BFTlJPVVRFUl9BUElfS0VZ"
        );
        assert_eq!(
            redact_provider_environment_variable(&ProviderEnvironmentVariable {
                name: "ANTHROPIC_BASE_URL".to_string(),
                value: "https://openrouter.ai/api".to_string(),
                sensitive: false,
                value_redacted: Some(true),
            }),
            ProviderEnvironmentVariable {
                name: "ANTHROPIC_BASE_URL".to_string(),
                value: "https://openrouter.ai/api".to_string(),
                sensitive: false,
                value_redacted: None,
            }
        );
        assert_eq!(
            redact_provider_environment_variable(&ProviderEnvironmentVariable {
                name: "OPENROUTER_API_KEY".to_string(),
                value: "sk-or-secret".to_string(),
                sensitive: true,
                value_redacted: None,
            }),
            ProviderEnvironmentVariable {
                name: "OPENROUTER_API_KEY".to_string(),
                value: String::new(),
                sensitive: true,
                value_redacted: Some(true),
            }
        );
        assert_eq!(
            redact_provider_environment_variable(&ProviderEnvironmentVariable {
                name: "EMPTY_SECRET".to_string(),
                value: String::new(),
                sensitive: true,
                value_redacted: None,
            })
            .value_redacted,
            None
        );

        let settings = ServerSettingsForClient {
            provider_instances: BTreeMap::from([(
                "codex_personal".to_string(),
                ProviderInstanceConfig {
                    environment: vec![
                        ProviderEnvironmentVariable {
                            name: "OPENROUTER_API_KEY".to_string(),
                            value: "sk-or-secret".to_string(),
                            sensitive: true,
                            value_redacted: None,
                        },
                        ProviderEnvironmentVariable {
                            name: "ANTHROPIC_BASE_URL".to_string(),
                            value: "https://openrouter.ai/api".to_string(),
                            sensitive: false,
                            value_redacted: Some(true),
                        },
                    ],
                },
            )]),
        };
        let redacted = redact_server_settings_for_client(&settings);
        let environment = &redacted.provider_instances["codex_personal"].environment;
        assert_eq!(environment[0].value, "");
        assert_eq!(environment[0].value_redacted, Some(true));
        assert_eq!(environment[1].value, "https://openrouter.ai/api");
        assert_eq!(environment[1].value_redacted, None);
    }

    #[test]
    fn ports_server_startup_access_contracts() {
        assert!(is_loopback_host(None));
        assert!(is_loopback_host(Some("127.12.0.1")));
        assert!(is_loopback_hostname_for_dev_redirect(" LOCALHOST "));
        assert!(is_loopback_hostname_for_dev_redirect("[::1]"));
        assert!(!is_loopback_hostname_for_dev_redirect("127.12.0.1"));
        assert!(is_wildcard_host(Some("0.0.0.0")));
        assert_eq!(format_host_for_url("::1"), "[::1]");
        assert_eq!(normalize_server_host("[::1]"), "::1");
        assert_eq!(
            resolve_dev_redirect_url(
                "http://localhost:5173/base?old=1#old",
                "http://127.0.0.1:3773/projects/demo?thread=1#turn"
            ),
            "http://localhost:5173/projects/demo?thread=1#turn"
        );
        assert_eq!(
            resolve_dev_redirect_url("http://localhost:5173", "http://127.0.0.1:3773?x=1"),
            "http://localhost:5173/?x=1"
        );
        assert_eq!(
            normalize_static_request_path("/"),
            StaticRequestPathDecision::ServeRelativePath("index.html".to_string())
        );
        assert_eq!(
            normalize_static_request_path("/assets/app.js"),
            StaticRequestPathDecision::ServeRelativePath("assets/app.js".to_string())
        );
        assert_eq!(
            normalize_static_request_path("/docs"),
            StaticRequestPathDecision::ServeRelativePath("docs/index.html".to_string())
        );
        assert_eq!(
            normalize_static_request_path("/docs/../guide"),
            StaticRequestPathDecision::ServeRelativePath("guide/index.html".to_string())
        );
        assert_eq!(
            normalize_static_request_path("/../secret"),
            StaticRequestPathDecision::Invalid {
                message: "Invalid static file path",
                status: 400,
            }
        );
        assert_eq!(
            normalize_static_request_path("/bad\0path"),
            StaticRequestPathDecision::Invalid {
                message: "Invalid static file path",
                status: 400,
            }
        );
        assert_eq!(
            static_and_dev_route_decision(
                Some("http://127.0.0.1:3773/projects/demo?thread=1#turn"),
                Some("http://localhost:5173/base"),
                false,
                false,
                false,
                false,
            ),
            StaticAndDevRouteDecision::Redirect {
                location: "http://localhost:5173/projects/demo?thread=1#turn".to_string(),
                status: 302,
            }
        );
        assert_eq!(
            static_and_dev_route_decision(None, None, false, false, false, false),
            StaticAndDevRouteDecision::BadRequest {
                body: "Bad Request",
                status: 400,
            }
        );
        assert_eq!(
            static_and_dev_route_decision(
                Some("http://example.com:3773/assets/app.js"),
                None,
                false,
                false,
                false,
                false,
            ),
            StaticAndDevRouteDecision::ServiceUnavailable {
                body: "No static directory configured and no dev URL set.",
                status: 503,
            }
        );
        assert_eq!(
            static_and_dev_route_decision(
                Some("http://example.com:3773/../secret"),
                None,
                true,
                false,
                false,
                false,
            ),
            StaticAndDevRouteDecision::InvalidStaticPath {
                body: "Invalid static file path",
                status: 400,
            }
        );
        assert_eq!(
            static_and_dev_route_decision(
                Some("http://example.com:3773/assets/app.js"),
                None,
                true,
                true,
                true,
                false,
            ),
            StaticAndDevRouteDecision::ServeFile {
                relative_path: "assets/app.js".to_string(),
                status: 200,
                content_type: "text/javascript",
            }
        );
        assert_eq!(
            static_and_dev_route_decision(
                Some("http://example.com:3773/missing"),
                None,
                true,
                false,
                false,
                true,
            ),
            StaticAndDevRouteDecision::FallbackIndex {
                relative_path: "index.html",
                status: 200,
                content_type: "text/html; charset=utf-8",
            }
        );
        assert_eq!(
            static_and_dev_route_decision(
                Some("http://example.com:3773/missing"),
                None,
                true,
                false,
                false,
                false,
            ),
            StaticAndDevRouteDecision::NotFound {
                body: "Not Found",
                status: 404,
            }
        );
        assert_eq!(
            static_and_dev_route_decision(
                Some("http://example.com:3773/assets/app.js"),
                Some("http://localhost:5173"),
                false,
                false,
                false,
                false,
            ),
            StaticAndDevRouteDecision::ServiceUnavailable {
                body: "No static directory configured and no dev URL set.",
                status: 503,
            }
        );
        assert_eq!(
            static_and_dev_route_decision(
                Some("http://example.com:3773/assets/app.js"),
                None,
                true,
                true,
                false,
                false,
            ),
            StaticAndDevRouteDecision::InternalServerError {
                body: "Internal Server Error",
                status: 500,
            }
        );
        let static_dir = make_static_test_dir("serve-file");
        std::fs::create_dir_all(static_dir.join("assets")).unwrap();
        std::fs::write(static_dir.join("assets/app.js"), b"console.log('r3');").unwrap();
        assert_eq!(
            static_and_dev_route_file_response(
                Some("http://example.com:3773/assets/app.js"),
                None,
                Some(&static_dir),
            ),
            StaticAndDevRouteFileResponse::File {
                relative_path: "assets/app.js".to_string(),
                bytes: b"console.log('r3');".to_vec(),
                status: 200,
                content_type: "text/javascript",
            }
        );
        let _ = std::fs::remove_dir_all(&static_dir);

        let static_dir = make_static_test_dir("fallback-index");
        std::fs::write(static_dir.join("index.html"), b"<main>fallback</main>").unwrap();
        assert_eq!(
            static_and_dev_route_file_response(
                Some("http://example.com:3773/missing-route"),
                None,
                Some(&static_dir),
            ),
            StaticAndDevRouteFileResponse::FallbackIndex {
                relative_path: "index.html",
                bytes: b"<main>fallback</main>".to_vec(),
                status: 200,
                content_type: "text/html; charset=utf-8",
            }
        );
        let _ = std::fs::remove_dir_all(&static_dir);

        let static_dir = make_static_test_dir("not-found");
        assert_eq!(
            static_and_dev_route_file_response(
                Some("http://example.com:3773/missing-route"),
                None,
                Some(&static_dir),
            ),
            StaticAndDevRouteFileResponse::NotFound {
                body: "Not Found",
                status: 404,
            }
        );
        let _ = std::fs::remove_dir_all(&static_dir);

        assert_eq!(
            static_and_dev_route_file_response(
                Some("http://127.0.0.1:3773/projects/demo?thread=1"),
                Some("http://localhost:5173/base"),
                None,
            ),
            StaticAndDevRouteFileResponse::Redirect {
                location: "http://localhost:5173/projects/demo?thread=1".to_string(),
                status: 302,
            }
        );
        assert_eq!(
            static_and_dev_route_file_response(
                Some("http://example.com:3773/assets/app.js"),
                Some("http://localhost:5173"),
                None,
            ),
            StaticAndDevRouteFileResponse::ServiceUnavailable {
                body: "No static directory configured and no dev URL set.",
                status: 503,
            }
        );
        assert_eq!(
            static_and_dev_route_file_response(
                Some("http://example.com:3773/../secret"),
                None,
                Some(Path::new(".")),
            ),
            StaticAndDevRouteFileResponse::InvalidStaticPath {
                body: "Invalid static file path",
                status: 400,
            }
        );
        assert_eq!(static_content_type_for_path("style.css"), "text/css");
        assert_eq!(static_content_type_for_path("icon.svg"), "image/svg+xml");
        assert_eq!(
            static_content_type_for_path("asset.unknown"),
            "application/octet-stream"
        );
        assert_eq!(BROWSER_API_CORS_ALLOWED_METHODS, ["GET", "POST", "OPTIONS"]);
        assert_eq!(
            BROWSER_API_CORS_ALLOWED_HEADERS,
            ["authorization", "b3", "traceparent", "content-type"]
        );
        assert_eq!(BROWSER_API_CORS_MAX_AGE_SECONDS, 600);
        assert_eq!(
            browser_api_cors_headers(),
            BTreeMap::from([
                ("access-control-allow-origin", "*".to_string()),
                (
                    "access-control-allow-methods",
                    "GET, POST, OPTIONS".to_string(),
                ),
                (
                    "access-control-allow-headers",
                    "authorization, b3, traceparent, content-type".to_string(),
                ),
            ])
        );
        assert_eq!(
            browser_api_cors_layer_plan(),
            BrowserApiCorsLayerPlan {
                allowed_methods: vec!["GET", "POST", "OPTIONS"],
                allowed_headers: vec!["authorization", "b3", "traceparent", "content-type"],
                max_age_seconds: 600,
            }
        );
        assert_eq!(
            apply_browser_api_cors_headers(BTreeMap::from([(
                "content-type".to_string(),
                "application/json".to_string()
            )])),
            BTreeMap::from([
                ("content-type".to_string(), "application/json".to_string()),
                ("access-control-allow-origin".to_string(), "*".to_string()),
                (
                    "access-control-allow-methods".to_string(),
                    "GET, POST, OPTIONS".to_string()
                ),
                (
                    "access-control-allow-headers".to_string(),
                    "authorization, b3, traceparent, content-type".to_string()
                ),
            ])
        );
        assert_eq!(
            browser_api_cors_route_decision("OPTIONS"),
            BrowserApiCorsRouteDecision::Preflight {
                status: 204,
                headers: BTreeMap::from([
                    ("access-control-allow-origin", "*".to_string()),
                    (
                        "access-control-allow-methods",
                        "GET, POST, OPTIONS".to_string(),
                    ),
                    (
                        "access-control-allow-headers",
                        "authorization, b3, traceparent, content-type".to_string(),
                    ),
                    ("access-control-max-age", "600".to_string()),
                ]),
            }
        );
        assert_eq!(
            browser_api_cors_route_decision("get"),
            BrowserApiCorsRouteDecision::ApplyHeaders {
                headers: browser_api_cors_headers(),
            }
        );
        assert!(FALLBACK_PROJECT_FAVICON_SVG.contains(r#"data-fallback="project-favicon""#));
        assert_eq!(PROJECT_FAVICON_CACHE_CONTROL, "public, max-age=3600");
        assert_eq!(
            project_favicon_route_decision(None, None, false),
            ProjectFaviconRouteDecision::BadRequest {
                body: "Bad Request",
                status: 400,
            }
        );
        assert_eq!(
            project_favicon_route_decision(
                Some("http://localhost:3773/api/project-favicon"),
                None,
                false
            ),
            ProjectFaviconRouteDecision::BadRequest {
                body: "Missing cwd parameter",
                status: 400,
            }
        );
        assert_eq!(
            project_favicon_route_decision(
                Some("http://localhost:3773/api/project-favicon?cwd=C:/repo"),
                None,
                false,
            ),
            ProjectFaviconRouteDecision::FallbackSvg {
                body: FALLBACK_PROJECT_FAVICON_SVG,
                status: 200,
                content_type: "image/svg+xml",
                cache_control: PROJECT_FAVICON_CACHE_CONTROL,
            }
        );
        assert_eq!(
            project_favicon_route_decision(
                Some("http://localhost:3773/api/project-favicon?cwd=C:/repo"),
                Some("C:/repo/favicon.ico"),
                false,
            ),
            ProjectFaviconRouteDecision::File {
                path: "C:/repo/favicon.ico".to_string(),
                status: 200,
                cache_control: PROJECT_FAVICON_CACHE_CONTROL,
            }
        );
        assert_eq!(
            project_favicon_route_decision(
                Some("http://localhost:3773/api/project-favicon?cwd=C:/repo"),
                Some("C:/repo/favicon.ico"),
                true,
            ),
            ProjectFaviconRouteDecision::InternalServerError {
                body: "Internal Server Error",
                status: 500,
            }
        );
        let favicon_dir = make_static_test_dir("favicon");
        let favicon_path = favicon_dir.join("favicon.ico");
        std::fs::write(&favicon_path, [0_u8, 1, 2, 3]).unwrap();
        assert_eq!(
            project_favicon_route_file_response(
                Some("http://localhost:3773/api/project-favicon?cwd=C:/repo"),
                Some(&favicon_path),
            ),
            ProjectFaviconRouteFileResponse::File {
                path: favicon_path.clone(),
                bytes: vec![0, 1, 2, 3],
                status: 200,
                cache_control: PROJECT_FAVICON_CACHE_CONTROL,
            }
        );
        assert_eq!(
            project_favicon_route_file_response(
                Some("http://localhost:3773/api/project-favicon?cwd=C:/repo"),
                Some(&favicon_dir.join("missing.ico")),
            ),
            ProjectFaviconRouteFileResponse::InternalServerError {
                body: "Internal Server Error",
                status: 500,
            }
        );
        let _ = std::fs::remove_dir_all(&favicon_dir);
        assert_eq!(
            project_favicon_route_file_response(
                Some("http://localhost:3773/api/project-favicon?cwd=C:/repo"),
                None,
            ),
            ProjectFaviconRouteFileResponse::FallbackSvg {
                body: FALLBACK_PROJECT_FAVICON_SVG,
                status: 200,
                content_type: "image/svg+xml",
                cache_control: PROJECT_FAVICON_CACHE_CONTROL,
            }
        );
        assert_eq!(OTLP_TRACES_PROXY_PATH, "/api/observability/v1/traces");
        assert_eq!(
            otlp_traces_proxy_export_decision(None),
            OtlpTracesProxyDecision::NoExportConfigured { status: 204 }
        );
        assert_eq!(
            otlp_traces_proxy_export_decision(Some("http://localhost:4318/v1/traces")),
            OtlpTracesProxyDecision::Export {
                url: "http://localhost:4318/v1/traces".to_string(),
            }
        );
        assert_eq!(
            otlp_traces_proxy_response_after_export(true),
            OtlpTracesProxyDecision::ExportSucceeded { status: 204 }
        );
        assert_eq!(
            otlp_traces_proxy_response_after_export(false),
            OtlpTracesProxyDecision::ExportFailed {
                body: "Trace export failed.",
                status: 502,
            }
        );

        assert_eq!(
            resolve_headless_connection_string(None, 3773, &BTreeMap::new()),
            "http://localhost:3773"
        );
        assert_eq!(
            resolve_headless_connection_string(Some("127.0.0.1"), 3773, &BTreeMap::new()),
            "http://127.0.0.1:3773"
        );
        assert_eq!(
            resolve_headless_connection_string(Some("::1"), 3773, &BTreeMap::new()),
            "http://[::1]:3773"
        );

        let interfaces = BTreeMap::from([
            (
                "en0".to_string(),
                vec![ServerNetworkInterfaceInfo {
                    address: "192.168.1.42".to_string(),
                    family: "IPv4".to_string(),
                    internal: false,
                }],
            ),
            (
                "lo0".to_string(),
                vec![ServerNetworkInterfaceInfo {
                    address: "127.0.0.1".to_string(),
                    family: "IPv4".to_string(),
                    internal: true,
                }],
            ),
        ]);
        assert_eq!(
            resolve_headless_connection_string(Some("0.0.0.0"), 3773, &interfaces),
            "http://192.168.1.42:3773"
        );
        assert_eq!(resolve_listening_port(Some(4123), 3773), 4123);
        assert_eq!(resolve_listening_port(None, 3773), 3773);
        assert_eq!(
            build_pairing_url("http://192.168.1.42:3773", "PAIRCODE"),
            "http://192.168.1.42:3773/pair#token=PAIRCODE"
        );

        let qr_code = render_terminal_qr_code("http://192.168.1.42:3773/pair#token=PAIRCODE", 2);
        assert!(qr_code.contains('█') || qr_code.contains('▀') || qr_code.contains('▄'));
        assert!(qr_code.split('\n').count() > 10);
        assert_eq!(qr_code.lines().next().unwrap_or("").chars().count(), 37);
        assert_eq!(qr_code.lines().count(), 19);

        let output = format_headless_serve_output(&HeadlessServeAccessInfo {
            connection_string: "http://192.168.1.42:3773".to_string(),
            token: "PAIRCODE".to_string(),
            pairing_url: "http://192.168.1.42:3773/pair#token=PAIRCODE".to_string(),
        });
        assert!(output.contains("Connection string: http://192.168.1.42:3773"));
        assert!(output.contains("Token: PAIRCODE"));
        assert!(output.contains("Pairing URL: http://192.168.1.42:3773/pair#token=PAIRCODE"));
    }
}
