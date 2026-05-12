pub const APP_NAME: &str = "R3Code";

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopedThreadRef {
    pub environment_id: String,
    pub thread_id: String,
}

impl ScopedThreadRef {
    pub fn new(environment_id: impl Into<String>, thread_id: impl Into<String>) -> Self {
        Self {
            environment_id: environment_id.into(),
            thread_id: thread_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopedProjectRef {
    pub environment_id: String,
    pub project_id: String,
}

impl ScopedProjectRef {
    pub fn new(environment_id: impl Into<String>, project_id: impl Into<String>) -> Self {
        Self {
            environment_id: environment_id.into(),
            project_id: project_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRoute {
    Index,
    Thread(ThreadRouteTarget),
}

impl ChatRoute {
    pub fn renders_chat_view(&self) -> bool {
        matches!(self, Self::Thread(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreadRouteTarget {
    Server { thread_ref: ScopedThreadRef },
    Draft { draft_id: String },
}

pub fn resolve_thread_route_target(
    environment_id: Option<&str>,
    thread_id: Option<&str>,
    draft_id: Option<&str>,
) -> Option<ThreadRouteTarget> {
    match (environment_id, thread_id) {
        (Some(environment_id), Some(thread_id))
            if !environment_id.is_empty() && !thread_id.is_empty() =>
        {
            Some(ThreadRouteTarget::Server {
                thread_ref: ScopedThreadRef::new(environment_id, thread_id),
            })
        }
        _ => draft_id
            .filter(|draft_id| !draft_id.is_empty())
            .map(|draft_id| ThreadRouteTarget::Draft {
                draft_id: draft_id.to_string(),
            }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftThreadEnvMode {
    Local,
    Worktree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchToolbarEnvironmentOption {
    pub environment_id: String,
    pub project_id: String,
    pub label: String,
    pub is_primary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcsRef {
    pub name: String,
    pub current: bool,
    pub is_default: bool,
    pub is_remote: bool,
    pub remote_name: Option<String>,
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchSelectionTarget {
    pub checkout_cwd: String,
    pub next_worktree_path: Option<String>,
    pub reuse_existing_worktree: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchToolbarState {
    pub environment_id: String,
    pub environment_label: String,
    pub environment_is_primary: bool,
    pub show_environment_picker: bool,
    pub effective_env_mode: DraftThreadEnvMode,
    pub env_locked: bool,
    pub env_mode_locked: bool,
    pub active_worktree_path: Option<String>,
    pub workspace_label: &'static str,
    pub branch_label: String,
    pub resolved_active_branch: Option<String>,
}

impl DraftThreadEnvMode {
    pub fn toggled(self) -> Self {
        match self {
            Self::Local => Self::Worktree,
            Self::Worktree => Self::Local,
        }
    }
}

fn normalize_display_label(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn is_generic_local_environment_label(label: &str) -> bool {
    matches!(
        label.trim().to_ascii_lowercase().as_str(),
        "local" | "local environment"
    )
}

pub fn resolve_environment_option_label(
    is_primary: bool,
    environment_id: &str,
    runtime_label: Option<&str>,
    saved_label: Option<&str>,
) -> String {
    let runtime_label = normalize_display_label(runtime_label);
    let saved_label = normalize_display_label(saved_label);

    if is_primary {
        return [runtime_label, saved_label]
            .into_iter()
            .flatten()
            .find(|label| !is_generic_local_environment_label(label))
            .unwrap_or("This device")
            .to_string();
    }

    runtime_label
        .or(saved_label)
        .unwrap_or(environment_id)
        .to_string()
}

pub fn resolve_env_mode_label(mode: DraftThreadEnvMode) -> &'static str {
    match mode {
        DraftThreadEnvMode::Local => "Current checkout",
        DraftThreadEnvMode::Worktree => "New worktree",
    }
}

pub fn resolve_current_workspace_label(active_worktree_path: Option<&str>) -> &'static str {
    if active_worktree_path.is_some() {
        "Current worktree"
    } else {
        resolve_env_mode_label(DraftThreadEnvMode::Local)
    }
}

pub fn resolve_locked_workspace_label(active_worktree_path: Option<&str>) -> &'static str {
    if active_worktree_path.is_some() {
        "Worktree"
    } else {
        "Local checkout"
    }
}

pub fn resolve_effective_env_mode(
    active_worktree_path: Option<&str>,
    has_server_thread: bool,
    draft_thread_env_mode: Option<DraftThreadEnvMode>,
) -> DraftThreadEnvMode {
    if !has_server_thread {
        if active_worktree_path.is_some() {
            return DraftThreadEnvMode::Local;
        }
        return if draft_thread_env_mode == Some(DraftThreadEnvMode::Worktree) {
            DraftThreadEnvMode::Worktree
        } else {
            DraftThreadEnvMode::Local
        };
    }

    if active_worktree_path.is_some() {
        DraftThreadEnvMode::Worktree
    } else {
        DraftThreadEnvMode::Local
    }
}

pub fn resolve_draft_env_mode_after_branch_change(
    next_worktree_path: Option<&str>,
    current_worktree_path: Option<&str>,
    effective_env_mode: DraftThreadEnvMode,
) -> DraftThreadEnvMode {
    if next_worktree_path.is_some() {
        return DraftThreadEnvMode::Worktree;
    }
    if effective_env_mode == DraftThreadEnvMode::Worktree && current_worktree_path.is_none() {
        return DraftThreadEnvMode::Worktree;
    }
    DraftThreadEnvMode::Local
}

pub fn resolve_branch_toolbar_value(
    env_mode: DraftThreadEnvMode,
    active_worktree_path: Option<&str>,
    active_thread_branch: Option<&str>,
    current_git_branch: Option<&str>,
) -> Option<String> {
    if env_mode == DraftThreadEnvMode::Worktree && active_worktree_path.is_none() {
        return active_thread_branch
            .or(current_git_branch)
            .map(str::to_string);
    }
    current_git_branch
        .or(active_thread_branch)
        .map(str::to_string)
}

pub fn branch_toolbar_trigger_label(
    active_worktree_path: Option<&str>,
    effective_env_mode: DraftThreadEnvMode,
    resolved_active_branch: Option<&str>,
) -> String {
    let Some(resolved_active_branch) = resolved_active_branch else {
        return "Select ref".to_string();
    };
    if effective_env_mode == DraftThreadEnvMode::Worktree && active_worktree_path.is_none() {
        return format!("From {resolved_active_branch}");
    }
    resolved_active_branch.to_string()
}

pub fn resolve_branch_selection_target(
    active_project_cwd: &str,
    active_worktree_path: Option<&str>,
    ref_name: &VcsRef,
) -> BranchSelectionTarget {
    if let Some(worktree_path) = ref_name.worktree_path.as_deref() {
        return BranchSelectionTarget {
            checkout_cwd: worktree_path.to_string(),
            next_worktree_path: if worktree_path == active_project_cwd {
                None
            } else {
                Some(worktree_path.to_string())
            },
            reuse_existing_worktree: true,
        };
    }

    let next_worktree_path = if active_worktree_path.is_some() && ref_name.is_default {
        None
    } else {
        active_worktree_path.map(str::to_string)
    };

    BranchSelectionTarget {
        checkout_cwd: next_worktree_path
            .clone()
            .unwrap_or_else(|| active_project_cwd.to_string()),
        next_worktree_path,
        reuse_existing_worktree: false,
    }
}

pub fn derive_local_branch_name_from_remote_ref(branch_name: &str) -> String {
    let Some(first_separator_index) = branch_name.find('/') else {
        return branch_name.to_string();
    };
    if first_separator_index == 0 || first_separator_index == branch_name.len() - 1 {
        return branch_name.to_string();
    }
    branch_name[first_separator_index + 1..].to_string()
}

fn derive_local_branch_name_candidates_from_remote_ref(
    branch_name: &str,
    remote_name: Option<&str>,
) -> Vec<String> {
    let mut candidates = Vec::new();
    let first_slash_candidate = derive_local_branch_name_from_remote_ref(branch_name);
    if !first_slash_candidate.is_empty() {
        candidates.push(first_slash_candidate);
    }

    if let Some(remote_name) = remote_name {
        let remote_prefix = format!("{remote_name}/");
        if branch_name.starts_with(&remote_prefix) && branch_name.len() > remote_prefix.len() {
            let candidate = branch_name[remote_prefix.len()..].to_string();
            if !candidates.iter().any(|existing| existing == &candidate) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

pub fn dedupe_remote_branches_with_local_matches(refs: &[VcsRef]) -> Vec<VcsRef> {
    let local_branch_names = refs
        .iter()
        .filter(|ref_name| !ref_name.is_remote)
        .map(|ref_name| ref_name.name.as_str())
        .collect::<Vec<_>>();

    refs.iter()
        .filter(|ref_name| {
            if !ref_name.is_remote {
                return true;
            }
            if ref_name.remote_name.as_deref() != Some("origin") {
                return true;
            }
            let local_branch_candidates = derive_local_branch_name_candidates_from_remote_ref(
                &ref_name.name,
                ref_name.remote_name.as_deref(),
            );
            !local_branch_candidates.iter().any(|candidate| {
                local_branch_names
                    .iter()
                    .any(|local_name| *local_name == candidate.as_str())
            })
        })
        .cloned()
        .collect()
}

pub fn should_include_branch_picker_item(
    item_value: &str,
    normalized_query: &str,
    create_branch_item_value: Option<&str>,
    checkout_pull_request_item_value: Option<&str>,
) -> bool {
    if normalized_query.is_empty() {
        return true;
    }
    if create_branch_item_value == Some(item_value) {
        return true;
    }
    if checkout_pull_request_item_value == Some(item_value) {
        return true;
    }
    item_value.to_ascii_lowercase().contains(normalized_query)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMode {
    ApprovalRequired,
    AutoAcceptEdits,
    FullAccess,
}

impl Default for RuntimeMode {
    fn default() -> Self {
        Self::FullAccess
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderInteractionMode {
    Default,
    Plan,
}

impl Default for ProviderInteractionMode {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftSessionState {
    pub draft_id: String,
    pub thread_ref: ScopedThreadRef,
    pub project_ref: ScopedProjectRef,
    pub logical_project_key: String,
    pub created_at: String,
    pub runtime_mode: RuntimeMode,
    pub interaction_mode: ProviderInteractionMode,
    pub branch: Option<String>,
    pub worktree_path: Option<String>,
    pub env_mode: DraftThreadEnvMode,
    pub promoted_to: Option<ScopedThreadRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSummary {
    pub id: String,
    pub environment_id: String,
    pub name: String,
    pub path: String,
    pub scripts: Vec<ProjectScript>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectScriptIcon {
    Play,
    Test,
    Lint,
    Configure,
    Build,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectScript {
    pub id: String,
    pub name: String,
    pub command: String,
    pub icon: ProjectScriptIcon,
    pub run_on_worktree_create: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorId {
    Cursor,
    Trae,
    Kiro,
    VsCode,
    VsCodeInsiders,
    VsCodium,
    Zed,
    Antigravity,
    Idea,
    Aqua,
    CLion,
    DataGrip,
    DataSpell,
    GoLand,
    PhpStorm,
    PyCharm,
    Rider,
    RubyMine,
    RustRover,
    WebStorm,
    FileManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorOption {
    pub label: &'static str,
    pub id: EditorId,
}

pub const DEFAULT_PROVIDER_DRIVER_KIND: &str = "codex";
pub const DEFAULT_MODEL: &str = "gpt-5.4";
pub const DEFAULT_GIT_TEXT_GENERATION_MODEL: &str = "gpt-5.4-mini";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerProviderState {
    Ready,
    Warning,
    Error,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerProviderAvailability {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerProviderAuthStatus {
    Authenticated,
    Unauthenticated,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProviderAuth {
    pub status: ServerProviderAuthStatus,
    pub kind: Option<String>,
    pub label: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProviderModel {
    pub slug: String,
    pub name: String,
    pub short_name: Option<String>,
    pub sub_provider: Option<String>,
    pub is_custom: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerProviderVersionAdvisoryStatus {
    Unknown,
    Current,
    BehindLatest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProviderVersionAdvisory {
    pub status: ServerProviderVersionAdvisoryStatus,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub update_command: Option<String>,
    pub can_update: bool,
    pub checked_at: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderVersionAdvisoryEmphasis {
    Normal,
    Strong,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderStatusSummary {
    pub headline: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderVersionAdvisoryPresentation {
    pub detail: String,
    pub update_command: Option<String>,
    pub emphasis: ProviderVersionAdvisoryEmphasis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProvider {
    pub instance_id: String,
    pub driver: String,
    pub display_name: Option<String>,
    pub accent_color: Option<String>,
    pub badge_label: Option<String>,
    pub continuation_group_key: Option<String>,
    pub show_interaction_mode_toggle: bool,
    pub enabled: bool,
    pub installed: bool,
    pub version: Option<String>,
    pub status: ServerProviderState,
    pub auth: ServerProviderAuth,
    pub checked_at: String,
    pub message: Option<String>,
    pub availability: ServerProviderAvailability,
    pub unavailable_reason: Option<String>,
    pub models: Vec<ServerProviderModel>,
    pub version_advisory: Option<ServerProviderVersionAdvisory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInstanceEntry {
    pub instance_id: String,
    pub driver_kind: String,
    pub display_name: String,
    pub accent_color: Option<String>,
    pub continuation_group_key: Option<String>,
    pub enabled: bool,
    pub installed: bool,
    pub status: ServerProviderState,
    pub is_default: bool,
    pub is_available: bool,
    pub models: Vec<ServerProviderModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderModelFavorite {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelPickerSelectedInstance {
    Favorites,
    Instance(String),
}

impl ModelPickerSelectedInstance {
    pub fn instance_id(&self) -> Option<&str> {
        match self {
            Self::Favorites => None,
            Self::Instance(instance_id) => Some(instance_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPickerItem {
    pub slug: String,
    pub name: String,
    pub short_name: Option<String>,
    pub sub_provider: Option<String>,
    pub instance_id: String,
    pub driver_kind: String,
    pub instance_display_name: String,
    pub instance_accent_color: Option<String>,
    pub continuation_group_key: Option<String>,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPickerState {
    pub active_entry: Option<ProviderInstanceEntry>,
    pub trigger_title: String,
    pub trigger_subtitle: Option<String>,
    pub trigger_label: String,
    pub show_instance_badge: bool,
    pub selected_instance: ModelPickerSelectedInstance,
    pub is_locked: bool,
    pub show_locked_instance_sidebar: bool,
    pub show_sidebar: bool,
    pub sidebar_entries: Vec<ProviderInstanceEntry>,
    pub locked_header_label: Option<String>,
    pub filtered_models: Vec<ModelPickerItem>,
}

pub fn default_instance_id_for_driver(driver: &str) -> String {
    driver.to_string()
}

pub fn default_model_by_provider(provider: &str) -> Option<&'static str> {
    match provider {
        "codex" => Some(DEFAULT_MODEL),
        "claudeAgent" => Some("claude-sonnet-4-6"),
        "cursor" => Some("auto"),
        "opencode" => Some("openai/gpt-5"),
        _ => None,
    }
}

pub fn default_git_text_generation_model_by_provider(provider: &str) -> Option<&'static str> {
    match provider {
        "codex" => Some(DEFAULT_GIT_TEXT_GENERATION_MODEL),
        "claudeAgent" => Some("claude-haiku-4-5"),
        "cursor" => Some("composer-2"),
        "opencode" => Some("openai/gpt-5"),
        _ => None,
    }
}

pub fn provider_display_name(driver: &str) -> String {
    match driver {
        "codex" => "Codex".to_string(),
        "claudeAgent" => "Claude".to_string(),
        "cursor" => "Cursor".to_string(),
        "opencode" => "OpenCode".to_string(),
        _ => format_provider_driver_kind_label(driver),
    }
}

pub fn get_provider_summary(provider: Option<&ServerProvider>) -> ProviderStatusSummary {
    let Some(provider) = provider else {
        return ProviderStatusSummary {
            headline: "Checking provider status".to_string(),
            detail: Some(
                "Waiting for the server to report installation and authentication details."
                    .to_string(),
            ),
        };
    };

    if !provider.enabled {
        return ProviderStatusSummary {
            headline: "Disabled".to_string(),
            detail: Some(provider.message.clone().unwrap_or_else(|| {
                format!(
                    "This provider is installed but disabled for new sessions in {}.",
                    APP_NAME
                )
            })),
        };
    }

    if !provider.installed {
        return ProviderStatusSummary {
            headline: "Not found".to_string(),
            detail: Some(
                provider
                    .message
                    .clone()
                    .unwrap_or_else(|| "CLI not detected on PATH.".to_string()),
            ),
        };
    }

    if provider.auth.status == ServerProviderAuthStatus::Authenticated {
        let auth_label = provider
            .auth
            .label
            .as_deref()
            .or(provider.auth.kind.as_deref());
        return ProviderStatusSummary {
            headline: auth_label
                .map(|label| format!("Authenticated · {label}"))
                .unwrap_or_else(|| "Authenticated".to_string()),
            detail: provider.message.clone(),
        };
    }

    if provider.auth.status == ServerProviderAuthStatus::Unauthenticated {
        return ProviderStatusSummary {
            headline: "Not authenticated".to_string(),
            detail: provider.message.clone(),
        };
    }

    if provider.status == ServerProviderState::Warning {
        return ProviderStatusSummary {
            headline: "Needs attention".to_string(),
            detail: Some(provider.message.clone().unwrap_or_else(|| {
                "The provider is installed, but the server could not fully verify it.".to_string()
            })),
        };
    }

    if provider.status == ServerProviderState::Error {
        return ProviderStatusSummary {
            headline: "Unavailable".to_string(),
            detail: Some(
                provider
                    .message
                    .clone()
                    .unwrap_or_else(|| "The provider failed its startup checks.".to_string()),
            ),
        };
    }

    ProviderStatusSummary {
        headline: "Available".to_string(),
        detail: Some(provider.message.clone().unwrap_or_else(|| {
            "Installed and ready, but authentication could not be verified.".to_string()
        })),
    }
}

pub fn get_provider_version_label(version: Option<&str>) -> Option<String> {
    let version = version?;
    if version.is_empty() {
        None
    } else if version.starts_with('v') {
        Some(version.to_string())
    } else {
        Some(format!("v{version}"))
    }
}

pub fn get_provider_version_advisory_presentation(
    advisory: Option<&ServerProviderVersionAdvisory>,
) -> Option<ProviderVersionAdvisoryPresentation> {
    let advisory = advisory?;
    if matches!(
        advisory.status,
        ServerProviderVersionAdvisoryStatus::Current | ServerProviderVersionAdvisoryStatus::Unknown
    ) {
        return None;
    }

    let version_label = get_provider_version_label(advisory.latest_version.as_deref());
    Some(ProviderVersionAdvisoryPresentation {
        detail: advisory.message.clone().unwrap_or_else(|| {
            version_label
                .map(|label| format!("Update available: install {label}."))
                .unwrap_or_else(|| {
                    "Update available: install the latest provider version.".to_string()
                })
        }),
        update_command: advisory.update_command.clone(),
        emphasis: ProviderVersionAdvisoryEmphasis::Normal,
    })
}

pub fn format_provider_driver_kind_label(provider: &str) -> String {
    title_case_words(&split_label_words(provider))
}

pub fn provider_instance_initials(label: &str) -> String {
    let words = split_label_words(label);
    if words.is_empty() {
        return String::new();
    }
    if words.len() == 1 {
        return words[0]
            .chars()
            .take(2)
            .flat_map(char::to_uppercase)
            .collect();
    }
    words
        .iter()
        .take(2)
        .filter_map(|word| word.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

fn split_label_words(value: &str) -> Vec<String> {
    let mut normalized = String::new();
    let mut previous_lowercase = false;
    for ch in value.trim().chars() {
        if ch == '_' || ch == '-' {
            normalized.push(' ');
            previous_lowercase = false;
            continue;
        }
        if ch.is_ascii_uppercase() && previous_lowercase {
            normalized.push(' ');
        }
        previous_lowercase = ch.is_ascii_lowercase();
        normalized.push(ch);
    }
    normalized
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

fn title_case_words(words: &[String]) -> String {
    words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first
                    .to_uppercase()
                    .chain(chars.flat_map(char::to_lowercase))
                    .collect::<String>(),
                None => String::new(),
            }
        })
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn normalize_provider_accent_color(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.len() != 7 || !trimmed.starts_with('#') {
        return None;
    }
    if trimmed[1..].chars().all(|ch| ch.is_ascii_hexdigit()) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn resolve_instance_display_name(
    snapshot: &ServerProvider,
    instance_id: &str,
    driver_kind: &str,
    is_default: bool,
) -> String {
    let trimmed_snapshot_name = snapshot.display_name.as_deref().map(str::trim);
    let kind_label = provider_display_name(driver_kind);
    if let Some(name) = trimmed_snapshot_name.filter(|name| !name.is_empty()) {
        if name != kind_label {
            return name.to_string();
        }
    }
    if !is_default {
        let humanized = title_case_words(&split_label_words(instance_id));
        if !humanized.is_empty() {
            return humanized;
        }
    }
    trimmed_snapshot_name
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .unwrap_or(kind_label)
}

pub fn derive_provider_instance_entries(
    providers: &[ServerProvider],
) -> Vec<ProviderInstanceEntry> {
    providers
        .iter()
        .map(|snapshot| {
            let instance_id = snapshot.instance_id.clone();
            let driver_kind = snapshot.driver.clone();
            let default_id = default_instance_id_for_driver(&driver_kind);
            let is_default = instance_id == default_id;
            ProviderInstanceEntry {
                display_name: resolve_instance_display_name(
                    snapshot,
                    &instance_id,
                    &driver_kind,
                    is_default,
                ),
                accent_color: normalize_provider_accent_color(snapshot.accent_color.as_deref()),
                continuation_group_key: snapshot.continuation_group_key.clone(),
                enabled: snapshot.enabled,
                installed: snapshot.installed,
                status: snapshot.status,
                is_default,
                is_available: snapshot.availability == ServerProviderAvailability::Available,
                models: snapshot.models.clone(),
                instance_id,
                driver_kind,
            }
        })
        .collect()
}

pub fn sort_provider_instance_entries(
    entries: &[ProviderInstanceEntry],
) -> Vec<ProviderInstanceEntry> {
    let mut by_kind = BTreeMap::<String, Vec<ProviderInstanceEntry>>::new();
    let mut kind_order = Vec::<String>::new();
    for entry in entries {
        if !by_kind.contains_key(&entry.driver_kind) {
            kind_order.push(entry.driver_kind.clone());
        }
        by_kind
            .entry(entry.driver_kind.clone())
            .or_default()
            .push(entry.clone());
    }

    let mut sorted = Vec::new();
    for kind in kind_order {
        let Some(bucket) = by_kind.remove(&kind) else {
            continue;
        };
        sorted.extend(bucket.iter().filter(|entry| entry.is_default).cloned());
        sorted.extend(bucket.iter().filter(|entry| !entry.is_default).cloned());
    }
    sorted
}

pub fn get_provider_instance_entry(
    providers: &[ServerProvider],
    instance_id: &str,
) -> Option<ProviderInstanceEntry> {
    derive_provider_instance_entries(providers)
        .into_iter()
        .find(|entry| entry.instance_id == instance_id)
}

pub fn resolve_selectable_provider_instance(
    providers: &[ServerProvider],
    instance_id: Option<&str>,
) -> Option<String> {
    let entries = derive_provider_instance_entries(providers);
    if let Some(instance_id) = instance_id {
        if entries
            .iter()
            .any(|entry| entry.instance_id == instance_id && entry.enabled && entry.is_available)
        {
            return Some(instance_id.to_string());
        }
    }
    entries
        .iter()
        .find(|entry| entry.enabled && entry.is_available)
        .map(|entry| entry.instance_id.clone())
}

pub fn resolve_provider_driver_kind_for_instance_selection(
    providers: &[ServerProvider],
    selection: Option<&str>,
) -> Option<String> {
    derive_provider_instance_entries(providers)
        .into_iter()
        .find(|entry| Some(entry.instance_id.as_str()) == selection)
        .map(|entry| entry.driver_kind)
}

pub fn get_display_model_name(model: &ServerProviderModel, prefer_short_name: bool) -> String {
    if prefer_short_name {
        if let Some(short_name) = model
            .short_name
            .as_deref()
            .filter(|value| !value.is_empty())
        {
            return short_name.to_string();
        }
    }
    model.name.clone()
}

pub fn get_trigger_display_model_name(model: &ServerProviderModel) -> String {
    get_display_model_name(model, true)
}

pub fn get_trigger_display_model_label(model: &ServerProviderModel) -> String {
    let title = get_trigger_display_model_name(model);
    model
        .sub_provider
        .as_deref()
        .filter(|sub_provider| !sub_provider.is_empty())
        .map(|sub_provider| format!("{sub_provider} · {title}"))
        .unwrap_or(title)
}

pub fn provider_model_key(instance_id: &str, slug: &str) -> String {
    format!("{instance_id}:{slug}")
}

pub fn split_instance_model_key(key: &str) -> (String, String) {
    key.split_once(':')
        .map(|(instance_id, slug)| (instance_id.to_string(), slug.to_string()))
        .unwrap_or_else(|| (key.to_string(), String::new()))
}

fn favorite_model_key_set(favorites: &[ProviderModelFavorite]) -> Vec<String> {
    favorites
        .iter()
        .map(|favorite| provider_model_key(&favorite.provider, &favorite.model))
        .collect()
}

fn is_favorite_model_key(favorites: &[String], instance_id: &str, slug: &str) -> bool {
    let key = provider_model_key(instance_id, slug);
    favorites.iter().any(|favorite| favorite == &key)
}

pub fn normalize_search_query(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

pub fn score_subsequence_match(value: &str, query: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(0);
    }

    let value_chars = value.chars().collect::<Vec<_>>();
    let query_chars = query.chars().collect::<Vec<_>>();
    let mut query_index = 0usize;
    let mut first_match_index = None::<usize>;
    let mut previous_match_index = None::<usize>;
    let mut gap_penalty = 0usize;

    for (value_index, value_char) in value_chars.iter().enumerate() {
        if query_index >= query_chars.len() || value_char != &query_chars[query_index] {
            continue;
        }

        if first_match_index.is_none() {
            first_match_index = Some(value_index);
        }
        if let Some(previous) = previous_match_index {
            gap_penalty += value_index.saturating_sub(previous + 1);
        }

        previous_match_index = Some(value_index);
        query_index += 1;
        if query_index == query_chars.len() {
            let first = first_match_index.unwrap_or(0);
            let span_penalty = value_index + 1 - first - query_chars.len();
            let length_penalty = value_chars.len().saturating_sub(query_chars.len()).min(64);
            return Some(first * 2 + gap_penalty * 3 + span_penalty + length_penalty);
        }
    }

    None
}

fn length_penalty(value: &str, query: &str) -> usize {
    value
        .chars()
        .count()
        .saturating_sub(query.chars().count())
        .min(64)
}

fn find_boundary_match_index(value: &str, query: &str) -> Option<usize> {
    [" ", "-", "_", "/"]
        .iter()
        .filter_map(|marker| {
            value
                .find(&format!("{marker}{query}"))
                .map(|index| index + marker.len())
        })
        .min()
}

pub fn score_query_match(
    value: &str,
    query: &str,
    exact_base: usize,
    prefix_base: Option<usize>,
    boundary_base: Option<usize>,
    includes_base: Option<usize>,
    fuzzy_base: Option<usize>,
) -> Option<usize> {
    if value.is_empty() || query.is_empty() {
        return None;
    }
    if value == query {
        return Some(exact_base);
    }
    if let Some(prefix_base) = prefix_base {
        if value.starts_with(query) {
            return Some(prefix_base + length_penalty(value, query));
        }
    }
    if let Some(boundary_base) = boundary_base {
        if let Some(boundary_index) = find_boundary_match_index(value, query) {
            return Some(boundary_base + boundary_index * 2 + length_penalty(value, query));
        }
    }
    if let Some(includes_base) = includes_base {
        if let Some(includes_index) = value.find(query) {
            return Some(includes_base + includes_index * 2 + length_penalty(value, query));
        }
    }
    if let Some(fuzzy_base) = fuzzy_base {
        if let Some(fuzzy_score) = score_subsequence_match(value, query) {
            return Some(fuzzy_base + fuzzy_score);
        }
    }
    None
}

pub fn build_model_picker_search_text(model: &ModelPickerItem) -> String {
    normalize_search_query(
        &[
            model.name.as_str(),
            model.short_name.as_deref().unwrap_or(""),
            model.sub_provider.as_deref().unwrap_or(""),
            model.driver_kind.as_str(),
            model.instance_display_name.as_str(),
        ]
        .iter()
        .filter(|value| !value.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" "),
    )
}

pub fn score_model_picker_search(model: &ModelPickerItem, query: &str) -> Option<isize> {
    const FAVORITE_SCORE_BOOST: isize = 24;
    let tokens = normalize_search_query(query)
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return Some(0);
    }

    let fields = [
        normalize_search_query(&model.name),
        model
            .short_name
            .as_deref()
            .map(normalize_search_query)
            .unwrap_or_default(),
        model
            .sub_provider
            .as_deref()
            .map(normalize_search_query)
            .unwrap_or_default(),
        normalize_search_query(&model.driver_kind),
        normalize_search_query(&model.instance_display_name),
        build_model_picker_search_text(model),
    ];

    let mut score = 0isize;
    for token in tokens {
        let token_score = fields
            .iter()
            .enumerate()
            .filter(|(_, field)| !field.is_empty())
            .filter_map(|(index, field)| {
                let field_base = index * 10;
                score_query_match(
                    field,
                    &token,
                    field_base,
                    Some(field_base + 2),
                    Some(field_base + 4),
                    Some(field_base + 6),
                    (token.len() >= 3).then_some(field_base + 100),
                )
            })
            .min()?;
        score += token_score as isize;
    }

    Some(if model.is_favorite {
        score - FAVORITE_SCORE_BOOST
    } else {
        score
    })
}

pub fn sort_provider_model_items(
    items: &[ModelPickerItem],
    favorite_model_keys: &[String],
    group_favorites: bool,
    instance_order: &[String],
) -> Vec<ModelPickerItem> {
    let instance_rank = instance_order
        .iter()
        .enumerate()
        .map(|(index, instance_id)| (instance_id.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let original_rank = items
        .iter()
        .enumerate()
        .map(|(index, item)| (provider_model_key(&item.instance_id, &item.slug), index))
        .collect::<BTreeMap<_, _>>();
    let mut indexed = items.to_vec();
    indexed.sort_by(|left, right| {
        if group_favorites {
            let left_fav =
                is_favorite_model_key(favorite_model_keys, &left.instance_id, &left.slug);
            let right_fav =
                is_favorite_model_key(favorite_model_keys, &right.instance_id, &right.slug);
            if left_fav != right_fav {
                return right_fav.cmp(&left_fav);
            }
        }

        let left_instance_rank = instance_rank
            .get(&left.instance_id)
            .copied()
            .unwrap_or(usize::MAX);
        let right_instance_rank = instance_rank
            .get(&right.instance_id)
            .copied()
            .unwrap_or(usize::MAX);
        left_instance_rank.cmp(&right_instance_rank).then_with(|| {
            let left_key = provider_model_key(&left.instance_id, &left.slug);
            let right_key = provider_model_key(&right.instance_id, &right.slug);
            original_rank
                .get(&left_key)
                .copied()
                .unwrap_or(usize::MAX)
                .cmp(&original_rank.get(&right_key).copied().unwrap_or(usize::MAX))
        })
    });
    indexed
}

pub fn normalize_model_slug(model: Option<&str>, provider: &str) -> Option<String> {
    let trimmed = model?.trim();
    if trimmed.is_empty() {
        return None;
    }
    let aliased = match provider {
        "codex" => match trimmed {
            "gpt-5-codex" | "5.4" => Some("gpt-5.4"),
            "5.3" | "gpt-5.3" => Some("gpt-5.3-codex"),
            "5.3-spark" | "gpt-5.3-spark" => Some("gpt-5.3-codex-spark"),
            _ => None,
        },
        "claudeAgent" => match trimmed {
            "opus" | "opus-4.7" | "claude-opus-4.7" => Some("claude-opus-4-7"),
            "opus-4.6" | "claude-opus-4.6" | "claude-opus-4-6-20251117" => Some("claude-opus-4-6"),
            "sonnet" | "sonnet-4.6" | "claude-sonnet-4.6" | "claude-sonnet-4-6-20251117" => {
                Some("claude-sonnet-4-6")
            }
            "haiku" | "haiku-4.5" | "claude-haiku-4.5" | "claude-haiku-4-5-20251001" => {
                Some("claude-haiku-4-5")
            }
            _ => None,
        },
        "cursor" => match trimmed {
            "composer" => Some("composer-2"),
            "composer-1" => Some("composer-1.5"),
            "composer-1.5" => Some("composer-1.5"),
            "opus-4.6-thinking" | "opus-4.6" => Some("claude-opus-4-6"),
            "sonnet-4.6-thinking" | "sonnet-4.6" => Some("claude-sonnet-4-6"),
            "opus-4.5-thinking" | "opus-4.5" => Some("claude-opus-4-5"),
            _ => None,
        },
        _ => None,
    };
    Some(aliased.unwrap_or(trimmed).to_string())
}

pub fn resolve_selectable_model(
    provider: &str,
    value: Option<&str>,
    options: &[ServerProviderModel],
) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(direct) = options.iter().find(|option| option.slug == trimmed) {
        return Some(direct.slug.clone());
    }
    if let Some(by_name) = options
        .iter()
        .find(|option| option.name.eq_ignore_ascii_case(trimmed))
    {
        return Some(by_name.slug.clone());
    }
    let normalized = normalize_model_slug(Some(trimmed), provider)?;
    options
        .iter()
        .find(|option| option.slug == normalized)
        .map(|option| option.slug.clone())
}

fn matches_locked_provider(
    entry: &ProviderInstanceEntry,
    locked_provider: Option<&str>,
    locked_continuation_group_key: Option<&str>,
) -> bool {
    let Some(locked_provider) = locked_provider else {
        return true;
    };
    if entry.driver_kind != locked_provider {
        return false;
    }
    locked_continuation_group_key
        .filter(|key| !key.is_empty())
        .map(|key| entry.continuation_group_key.as_deref() == Some(key))
        .unwrap_or(true)
}

pub fn resolve_model_picker_state(
    snapshot: &AppSnapshot,
    search_query: &str,
    selected_instance: Option<ModelPickerSelectedInstance>,
    locked_provider: Option<&str>,
    locked_continuation_group_key: Option<&str>,
) -> ModelPickerState {
    let entries = derive_provider_instance_entries(&snapshot.providers);
    let active_entry = entries
        .iter()
        .find(|entry| entry.instance_id == snapshot.selected_provider_instance_id)
        .cloned();
    let selected_options = active_entry
        .as_ref()
        .map(|entry| entry.models.as_slice())
        .unwrap_or(&[]);
    let selected_model = selected_options
        .iter()
        .find(|option| option.slug == snapshot.selected_model)
        .or_else(|| selected_options.first());
    let trigger_title = selected_model
        .map(get_trigger_display_model_name)
        .unwrap_or_else(|| snapshot.selected_model.clone());
    let trigger_subtitle = selected_model.and_then(|model| model.sub_provider.clone());
    let trigger_label = selected_model
        .map(get_trigger_display_model_label)
        .unwrap_or_else(|| snapshot.selected_model.clone());
    let duplicate_driver_count = active_entry
        .as_ref()
        .map(|active| {
            entries
                .iter()
                .filter(|entry| entry.driver_kind == active.driver_kind)
                .count()
        })
        .unwrap_or(0);
    let show_instance_badge = active_entry
        .as_ref()
        .map(|entry| entry.accent_color.is_some() || duplicate_driver_count > 1)
        .unwrap_or(false);
    let favorite_keys = favorite_model_key_set(&snapshot.model_favorites);
    let selected_instance = selected_instance.unwrap_or_else(|| {
        if locked_provider.is_some() {
            ModelPickerSelectedInstance::Instance(snapshot.selected_provider_instance_id.clone())
        } else if !snapshot.model_favorites.is_empty() {
            ModelPickerSelectedInstance::Favorites
        } else {
            ModelPickerSelectedInstance::Instance(snapshot.selected_provider_instance_id.clone())
        }
    });
    let is_locked = locked_provider.is_some();
    let locked_instance_entries = locked_provider
        .map(|_| {
            entries
                .iter()
                .filter(|entry| {
                    matches_locked_provider(entry, locked_provider, locked_continuation_group_key)
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let show_locked_instance_sidebar = is_locked && locked_instance_entries.len() > 1;
    let is_searching = !search_query.trim().is_empty();
    let show_sidebar = !is_searching && (!is_locked || show_locked_instance_sidebar);
    let sidebar_entries = if show_locked_instance_sidebar {
        locked_instance_entries.clone()
    } else {
        entries.clone()
    };
    let mut flat_models = Vec::<ModelPickerItem>::new();
    for entry in &entries {
        if entry.status != ServerProviderState::Ready {
            continue;
        }
        for model in &entry.models {
            flat_models.push(ModelPickerItem {
                slug: model.slug.clone(),
                name: model.name.clone(),
                short_name: model.short_name.clone(),
                sub_provider: model.sub_provider.clone(),
                instance_id: entry.instance_id.clone(),
                driver_kind: entry.driver_kind.clone(),
                instance_display_name: entry.display_name.clone(),
                instance_accent_color: entry.accent_color.clone(),
                continuation_group_key: entry.continuation_group_key.clone(),
                is_favorite: is_favorite_model_key(&favorite_keys, &entry.instance_id, &model.slug),
            });
        }
    }

    let filtered_models = if is_searching {
        let mut ranked = flat_models
            .into_iter()
            .filter(|model| {
                locked_provider
                    .map(|_| {
                        matches_locked_provider(
                            &ProviderInstanceEntry {
                                instance_id: model.instance_id.clone(),
                                driver_kind: model.driver_kind.clone(),
                                display_name: model.instance_display_name.clone(),
                                accent_color: model.instance_accent_color.clone(),
                                continuation_group_key: model.continuation_group_key.clone(),
                                enabled: true,
                                installed: true,
                                status: ServerProviderState::Ready,
                                is_default: false,
                                is_available: true,
                                models: Vec::new(),
                            },
                            locked_provider,
                            locked_continuation_group_key,
                        )
                    })
                    .unwrap_or(true)
            })
            .filter_map(|model| {
                score_model_picker_search(&model, search_query).map(|score| {
                    let tie_breaker = build_model_picker_search_text(&model);
                    (model, score, tie_breaker)
                })
            })
            .collect::<Vec<_>>();
        ranked.sort_by(
            |(left, left_score, left_tie), (right, right_score, right_tie)| {
                left_score
                    .cmp(right_score)
                    .then_with(|| right.is_favorite.cmp(&left.is_favorite))
                    .then_with(|| left_tie.cmp(right_tie))
            },
        );
        ranked.into_iter().map(|(model, _, _)| model).collect()
    } else {
        let mut result = flat_models
            .into_iter()
            .filter(|model| {
                locked_provider
                    .map(|_| {
                        model.driver_kind == locked_provider.unwrap_or_default()
                            && locked_continuation_group_key
                                .filter(|key| !key.is_empty())
                                .map(|key| model.continuation_group_key.as_deref() == Some(key))
                                .unwrap_or(true)
                    })
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();

        if is_locked {
            if show_locked_instance_sidebar {
                if let Some(instance_id) = selected_instance.instance_id() {
                    result.retain(|model| model.instance_id == instance_id);
                }
            }
        } else if selected_instance == ModelPickerSelectedInstance::Favorites {
            result.retain(|model| model.is_favorite);
        } else if let Some(instance_id) = selected_instance.instance_id() {
            result.retain(|model| model.instance_id == instance_id);
        }

        let instance_order = if selected_instance == ModelPickerSelectedInstance::Favorites {
            entries
                .iter()
                .map(|entry| entry.instance_id.clone())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        sort_provider_model_items(
            &result,
            &favorite_keys,
            selected_instance != ModelPickerSelectedInstance::Favorites,
            &instance_order,
        )
    };

    let locked_header_label = if is_locked && !show_locked_instance_sidebar {
        let matches = entries
            .iter()
            .filter(|entry| {
                matches_locked_provider(entry, locked_provider, locked_continuation_group_key)
            })
            .collect::<Vec<_>>();
        if matches.is_empty() {
            None
        } else {
            matches
                .iter()
                .find(|entry| entry.instance_id == snapshot.selected_provider_instance_id)
                .copied()
                .or_else(|| matches.first().copied())
                .map(|entry| entry.display_name.clone())
        }
    } else {
        None
    };

    ModelPickerState {
        active_entry,
        trigger_title,
        trigger_subtitle,
        trigger_label,
        show_instance_badge,
        selected_instance,
        is_locked,
        show_locked_instance_sidebar,
        show_sidebar,
        sidebar_entries,
        locked_header_label,
        filtered_models,
    }
}

const MAX_SCRIPT_ID_LENGTH: usize = 64;

pub fn command_for_project_script(script_id: &str) -> String {
    format!("script.{script_id}.run")
}

pub fn project_script_id_from_command(command: &str) -> Option<String> {
    let trimmed = command.trim();
    let prefix = "script.";
    let suffix = ".run";
    if !trimmed.starts_with(prefix) || !trimmed.ends_with(suffix) {
        return None;
    }
    let script_id = &trimmed[prefix.len()..trimmed.len() - suffix.len()];
    if script_id.is_empty() {
        None
    } else {
        Some(script_id.to_string())
    }
}

fn normalize_script_id(value: &str) -> String {
    let mut cleaned = String::new();
    let mut last_was_dash = false;
    for ch in value.trim().chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            cleaned.push(ch);
            last_was_dash = false;
        } else if !last_was_dash && !cleaned.is_empty() {
            cleaned.push('-');
            last_was_dash = true;
        }
    }
    while cleaned.ends_with('-') {
        cleaned.pop();
    }
    if cleaned.is_empty() {
        return "script".to_string();
    }
    if cleaned.len() <= MAX_SCRIPT_ID_LENGTH {
        return cleaned;
    }
    let mut truncated = cleaned[..MAX_SCRIPT_ID_LENGTH]
        .trim_end_matches('-')
        .to_string();
    if truncated.is_empty() {
        truncated = "script".to_string();
    }
    truncated
}

pub fn next_project_script_id<I, S>(name: &str, existing_ids: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let taken = existing_ids
        .into_iter()
        .map(|id| id.as_ref().to_string())
        .collect::<Vec<_>>();
    let base_id = normalize_script_id(name);
    if !taken.iter().any(|id| id == &base_id) {
        return base_id;
    }

    for suffix in 2..10_000 {
        let candidate = format!("{base_id}-{suffix}");
        let safe_candidate = if candidate.len() <= MAX_SCRIPT_ID_LENGTH {
            candidate
        } else {
            let suffix_len = suffix.to_string().len();
            let prefix_len = MAX_SCRIPT_ID_LENGTH.saturating_sub(suffix_len + 1).max(1);
            format!("{}-{suffix}", &base_id[..prefix_len])
        };
        if !taken.iter().any(|id| id == &safe_candidate) {
            return safe_candidate;
        }
    }

    base_id
}

pub fn primary_project_script(scripts: &[ProjectScript]) -> Option<&ProjectScript> {
    scripts
        .iter()
        .find(|script| !script.run_on_worktree_create)
        .or_else(|| scripts.first())
}

pub fn setup_project_script(scripts: &[ProjectScript]) -> Option<&ProjectScript> {
    scripts.iter().find(|script| script.run_on_worktree_create)
}

pub fn project_script_cwd(project_cwd: &str, worktree_path: Option<&str>) -> String {
    worktree_path.unwrap_or(project_cwd).to_string()
}

pub fn project_script_runtime_env(
    project_cwd: &str,
    worktree_path: Option<&str>,
    extra_env: &[(&str, &str)],
) -> BTreeMap<String, String> {
    let mut env = BTreeMap::from([("T3CODE_PROJECT_ROOT".to_string(), project_cwd.to_string())]);
    if let Some(worktree_path) = worktree_path {
        env.insert(
            "T3CODE_WORKTREE_PATH".to_string(),
            worktree_path.to_string(),
        );
    }
    for (key, value) in extra_env {
        env.insert((*key).to_string(), (*value).to_string());
    }
    env
}

pub fn should_show_open_in_picker(
    active_project_name: Option<&str>,
    active_thread_environment_id: &str,
    primary_environment_id: Option<&str>,
) -> bool {
    active_project_name.is_some()
        && primary_environment_id
            .map(|primary| primary == active_thread_environment_id)
            .unwrap_or(false)
}

pub fn resolve_editor_options(platform: &str, available_editors: &[EditorId]) -> Vec<EditorOption> {
    editor_options(platform)
        .iter()
        .copied()
        .filter(|option| available_editors.iter().any(|editor| editor == &option.id))
        .collect()
}

fn editor_options(platform: &str) -> Vec<EditorOption> {
    let file_manager_label = if platform.to_ascii_lowercase().contains("win") {
        "Explorer"
    } else if platform.to_ascii_lowercase().contains("mac") {
        "Finder"
    } else {
        "Files"
    };

    vec![
        EditorOption {
            label: "Cursor",
            id: EditorId::Cursor,
        },
        EditorOption {
            label: "Trae",
            id: EditorId::Trae,
        },
        EditorOption {
            label: "Kiro",
            id: EditorId::Kiro,
        },
        EditorOption {
            label: "VS Code",
            id: EditorId::VsCode,
        },
        EditorOption {
            label: "VS Code Insiders",
            id: EditorId::VsCodeInsiders,
        },
        EditorOption {
            label: "VSCodium",
            id: EditorId::VsCodium,
        },
        EditorOption {
            label: "Zed",
            id: EditorId::Zed,
        },
        EditorOption {
            label: "Antigravity",
            id: EditorId::Antigravity,
        },
        EditorOption {
            label: "IntelliJ IDEA",
            id: EditorId::Idea,
        },
        EditorOption {
            label: "Aqua",
            id: EditorId::Aqua,
        },
        EditorOption {
            label: "CLion",
            id: EditorId::CLion,
        },
        EditorOption {
            label: "DataGrip",
            id: EditorId::DataGrip,
        },
        EditorOption {
            label: "DataSpell",
            id: EditorId::DataSpell,
        },
        EditorOption {
            label: "GoLand",
            id: EditorId::GoLand,
        },
        EditorOption {
            label: "PhpStorm",
            id: EditorId::PhpStorm,
        },
        EditorOption {
            label: "PyCharm",
            id: EditorId::PyCharm,
        },
        EditorOption {
            label: "Rider",
            id: EditorId::Rider,
        },
        EditorOption {
            label: "RubyMine",
            id: EditorId::RubyMine,
        },
        EditorOption {
            label: "RustRover",
            id: EditorId::RustRover,
        },
        EditorOption {
            label: "WebStorm",
            id: EditorId::WebStorm,
        },
        EditorOption {
            label: file_manager_label,
            id: EditorId::FileManager,
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSummary {
    pub id: String,
    pub environment_id: String,
    pub project_id: String,
    pub title: String,
    pub project_name: String,
    pub status: ThreadStatus,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    pub latest_user_message_at: Option<String>,
    pub has_pending_approvals: bool,
    pub has_pending_user_input: bool,
    pub has_actionable_proposed_plan: bool,
    pub branch: Option<String>,
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    Idle,
    Running,
    NeedsInput,
    Failed,
}

pub const RECENT_COMMAND_PALETTE_THREAD_LIMIT: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarThreadSortOrder {
    UpdatedAt,
    CreatedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteItemKind {
    Action,
    Submenu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteItem {
    pub kind: CommandPaletteItemKind,
    pub value: String,
    pub search_terms: Vec<String>,
    pub title: String,
    pub description: Option<String>,
    pub timestamp: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteGroup {
    pub value: String,
    pub label: String,
    pub items: Vec<CommandPaletteItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteMode {
    Root,
    RootBrowse,
    Submenu,
    SubmenuBrowse,
}

impl CommandPaletteItem {
    pub fn action(
        value: impl Into<String>,
        search_terms: Vec<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            kind: CommandPaletteItemKind::Action,
            value: value.into(),
            search_terms,
            title: title.into(),
            description: None,
            timestamp: None,
            disabled: false,
        }
    }

    pub fn submenu(
        value: impl Into<String>,
        search_terms: Vec<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            kind: CommandPaletteItemKind::Submenu,
            value: value.into(),
            search_terms,
            title: title.into(),
            description: None,
            timestamp: None,
            disabled: false,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        let description = description.into();
        if !description.is_empty() {
            self.description = Some(description);
        }
        self
    }

    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        let timestamp = timestamp.into();
        if !timestamp.is_empty() {
            self.timestamp = Some(timestamp);
        }
        self
    }
}

pub fn normalize_command_palette_search_text(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn rank_command_palette_search_field(field: &str, normalized_query: &str) -> Option<i32> {
    let normalized_field = normalize_command_palette_search_text(field);
    if normalized_field.is_empty() || !normalized_field.contains(normalized_query) {
        return None;
    }
    if normalized_field == normalized_query {
        return Some(3);
    }
    if normalized_field.starts_with(normalized_query) {
        return Some(2);
    }
    Some(1)
}

fn rank_command_palette_item_match(item: &CommandPaletteItem, normalized_query: &str) -> i32 {
    let terms = item
        .search_terms
        .iter()
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    if terms.is_empty() {
        return 0;
    }

    for (index, field) in terms.iter().enumerate() {
        if let Some(field_rank) = rank_command_palette_search_field(field, normalized_query) {
            return 1_000 - (index as i32 * 100) + field_rank;
        }
    }

    0
}

pub fn filter_command_palette_groups(
    active_groups: &[CommandPaletteGroup],
    query: &str,
    is_in_submenu: bool,
    project_search_items: &[CommandPaletteItem],
    thread_search_items: &[CommandPaletteItem],
) -> Vec<CommandPaletteGroup> {
    let is_actions_filter = query.starts_with('>');
    let search_query = if is_actions_filter {
        &query[1..]
    } else {
        query
    };
    let normalized_query = normalize_command_palette_search_text(search_query);

    if normalized_query.is_empty() {
        if is_actions_filter {
            return active_groups
                .iter()
                .filter(|group| group.value == "actions")
                .cloned()
                .collect();
        }
        return active_groups.to_vec();
    }

    let mut searchable_groups = active_groups
        .iter()
        .filter(|group| {
            if is_actions_filter {
                group.value == "actions"
            } else {
                is_in_submenu || group.value != "recent-threads"
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    if !is_in_submenu && !is_actions_filter {
        if !project_search_items.is_empty() {
            searchable_groups.push(CommandPaletteGroup {
                value: "projects-search".to_string(),
                label: "Projects".to_string(),
                items: project_search_items.to_vec(),
            });
        }
        if !thread_search_items.is_empty() {
            searchable_groups.push(CommandPaletteGroup {
                value: "threads-search".to_string(),
                label: "Threads".to_string(),
                items: thread_search_items.to_vec(),
            });
        }
    }

    searchable_groups
        .into_iter()
        .filter_map(|group| {
            let mut ranked_items = group
                .items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    let haystack =
                        normalize_command_palette_search_text(&item.search_terms.join(" "));
                    if !haystack.contains(&normalized_query) {
                        return None;
                    }
                    Some((
                        index,
                        rank_command_palette_item_match(item, &normalized_query),
                        item.clone(),
                    ))
                })
                .collect::<Vec<_>>();

            ranked_items.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
            let items = ranked_items
                .into_iter()
                .map(|(_, _, item)| item)
                .collect::<Vec<_>>();
            if items.is_empty() {
                None
            } else {
                Some(CommandPaletteGroup { items, ..group })
            }
        })
        .collect()
}

pub fn build_project_action_items(
    projects: &[ProjectSummary],
    value_prefix: &str,
) -> Vec<CommandPaletteItem> {
    projects
        .iter()
        .map(|project| {
            CommandPaletteItem::action(
                format!("{}:{}:{}", value_prefix, project.environment_id, project.id),
                vec![project.name.clone(), project.path.clone()],
                project.name.clone(),
            )
            .with_description(project.path.clone())
        })
        .collect()
}

pub fn build_thread_action_items(
    threads: &[ThreadSummary],
    active_thread_id: Option<&str>,
    projects: &[ProjectSummary],
    sort_order: SidebarThreadSortOrder,
    now_iso: &str,
    limit: Option<usize>,
) -> Vec<CommandPaletteItem> {
    let project_title_by_id = projects
        .iter()
        .map(|project| (project.id.as_str(), project.name.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut sorted_threads = threads
        .iter()
        .filter(|thread| thread.archived_at.is_none())
        .collect::<Vec<_>>();
    sorted_threads.sort_by(|left, right| {
        let left_timestamp = get_thread_sort_timestamp(left, sort_order);
        let right_timestamp = get_thread_sort_timestamp(right, sort_order);
        right_timestamp
            .cmp(&left_timestamp)
            .then(right.id.cmp(&left.id))
    });

    let visible_threads: Box<dyn Iterator<Item = &ThreadSummary> + '_> = if let Some(limit) = limit
    {
        Box::new(sorted_threads.into_iter().take(limit))
    } else {
        Box::new(sorted_threads.into_iter())
    };

    visible_threads
        .map(|thread| {
            let project_title = project_title_by_id
                .get(thread.project_id.as_str())
                .copied()
                .or_else(|| {
                    if thread.project_name.is_empty() {
                        None
                    } else {
                        Some(thread.project_name.as_str())
                    }
                });
            let mut description_parts = Vec::new();
            if let Some(project_title) = project_title {
                description_parts.push(project_title.to_string());
            }
            if let Some(branch) = &thread.branch {
                description_parts.push(format!("#{branch}"));
            }
            if active_thread_id == Some(thread.id.as_str()) {
                description_parts.push("Current thread".to_string());
            }

            let display_timestamp = thread
                .latest_user_message_at
                .as_deref()
                .unwrap_or(thread.updated_at.as_str());

            CommandPaletteItem::action(
                format!("thread:{}", thread.id),
                vec![
                    thread.title.clone(),
                    project_title.unwrap_or_default().to_string(),
                    thread.branch.clone().unwrap_or_default(),
                ],
                thread.title.clone(),
            )
            .with_description(description_parts.join(" · "))
            .with_timestamp(format_relative_time_label_at(display_timestamp, now_iso))
        })
        .collect()
}

pub fn build_root_command_palette_groups(
    action_items: Vec<CommandPaletteItem>,
    recent_thread_items: Vec<CommandPaletteItem>,
) -> Vec<CommandPaletteGroup> {
    let mut groups = Vec::new();
    if !action_items.is_empty() {
        groups.push(CommandPaletteGroup {
            value: "actions".to_string(),
            label: "Actions".to_string(),
            items: action_items,
        });
    }
    if !recent_thread_items.is_empty() {
        groups.push(CommandPaletteGroup {
            value: "recent-threads".to_string(),
            label: "Recent Threads".to_string(),
            items: recent_thread_items,
        });
    }
    groups
}

pub fn get_command_palette_mode(
    current_view_present: bool,
    is_browsing: bool,
) -> CommandPaletteMode {
    match (current_view_present, is_browsing) {
        (true, true) => CommandPaletteMode::SubmenuBrowse,
        (true, false) => CommandPaletteMode::Submenu,
        (false, true) => CommandPaletteMode::RootBrowse,
        (false, false) => CommandPaletteMode::Root,
    }
}

pub fn get_command_palette_input_placeholder(mode: CommandPaletteMode) -> &'static str {
    match mode {
        CommandPaletteMode::Root => "Search commands, projects, and threads...",
        CommandPaletteMode::RootBrowse => "Enter project path (e.g. ~/projects/my-app)",
        CommandPaletteMode::Submenu => "Search...",
        CommandPaletteMode::SubmenuBrowse => "Enter path (e.g. ~/projects/my-app)",
    }
}

fn get_thread_sort_timestamp(thread: &ThreadSummary, sort_order: SidebarThreadSortOrder) -> i64 {
    if sort_order == SidebarThreadSortOrder::CreatedAt {
        return iso_utc_timestamp_seconds(&thread.created_at).unwrap_or(i64::MIN);
    }

    thread
        .latest_user_message_at
        .as_deref()
        .and_then(iso_utc_timestamp_seconds)
        .or_else(|| iso_utc_timestamp_seconds(&thread.updated_at))
        .or_else(|| iso_utc_timestamp_seconds(&thread.created_at))
        .unwrap_or(i64::MIN)
}

pub fn format_relative_time_label_at(iso_date: &str, now_iso: &str) -> String {
    let Some(now_seconds) = iso_utc_timestamp_seconds(now_iso) else {
        return "just now".to_string();
    };
    let Some(date_seconds) = iso_utc_timestamp_seconds(iso_date) else {
        return "just now".to_string();
    };
    let diff = now_seconds.saturating_sub(date_seconds);
    if date_seconds > now_seconds || diff < 60 {
        return "just now".to_string();
    }
    let minutes = diff / 60;
    if minutes < 60 {
        return format!("{minutes}m ago");
    }
    let hours = minutes / 60;
    if hours < 24 {
        return format!("{hours}h ago");
    }
    format!("{}d ago", hours / 24)
}

fn iso_utc_timestamp_seconds(iso: &str) -> Option<i64> {
    let date_time = iso.strip_suffix('Z').unwrap_or(iso);
    let year = date_time.get(0..4)?.parse::<i32>().ok()?;
    let month = date_time.get(5..7)?.parse::<u32>().ok()?;
    let day = date_time.get(8..10)?.parse::<u32>().ok()?;
    let hour = date_time.get(11..13)?.parse::<u32>().ok()?;
    let minute = date_time.get(14..16)?.parse::<u32>().ok()?;
    let second = date_time.get(17..19)?.parse::<u32>().ok()?;

    if date_time.get(4..5) != Some("-")
        || date_time.get(7..8) != Some("-")
        || date_time.get(10..11) != Some("T")
        || date_time.get(13..14) != Some(":")
        || date_time.get(16..17) != Some(":")
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }

    let days = days_from_civil(year, month, day)?;
    Some(days * 86_400 + hour as i64 * 3_600 + minute as i64 * 60 + second as i64)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    let month_days = [31_u32, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let max_day = if month == 2 && is_leap_year(year) {
        29
    } else {
        *month_days.get(month.checked_sub(1)? as usize)?
    };
    if day > max_day {
        return None;
    }

    let year = year - (month <= 2) as i32;
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = month as i32;
    let day = day as i32;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some((era * 146_097 + day_of_era - 719_468) as i64)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSshEnvironmentTarget {
    pub alias: String,
    pub hostname: String,
    pub username: Option<String>,
    pub port: Option<u16>,
}

pub fn format_desktop_ssh_target(target: &DesktopSshEnvironmentTarget) -> String {
    let authority = if let Some(username) = target.username.as_deref() {
        format!("{username}@{}", target.hostname)
    } else {
        target.hostname.clone()
    };
    if let Some(port) = target.port {
        format!("{authority}:{port}")
    } else {
        authority
    }
}

pub fn parse_manual_desktop_ssh_target(
    host: &str,
    username: &str,
    port: &str,
) -> Result<DesktopSshEnvironmentTarget, String> {
    let raw_host = host.trim();
    if raw_host.is_empty() {
        return Err("SSH host or alias is required.".to_string());
    }

    let mut hostname = raw_host.to_string();
    let mut username = trimmed_non_empty(username).map(str::to_string);
    let mut parsed_port = None;
    let mut parsed_port_was_provided = false;

    if let Some(at_index) = hostname.rfind('@') {
        if at_index > 0 {
            let inline_username = hostname[..at_index].trim().to_string();
            hostname = hostname[at_index + 1..].trim().to_string();
            if username.is_none() && !inline_username.is_empty() {
                username = Some(inline_username);
            }
        }
    }

    if let Some((bracketed_host, bracketed_port)) = parse_bracketed_host_port(&hostname) {
        hostname = bracketed_host;
        if let Some(port) = bracketed_port {
            parsed_port = Some(port);
            parsed_port_was_provided = true;
        }
    } else if let Some((host_part, port_part)) = hostname.split_once(':') {
        if !host_part.contains(':')
            && !port_part.contains(':')
            && !port_part.is_empty()
            && port_part.chars().all(|ch| ch.is_ascii_digit())
        {
            let next_hostname = host_part.trim().to_string();
            parsed_port = port_part.parse::<i64>().ok();
            hostname = next_hostname;
            parsed_port_was_provided = true;
        }
    }

    let raw_port = port.trim();
    if !raw_port.is_empty() {
        parsed_port = parse_js_base10_int(raw_port);
        parsed_port_was_provided = true;
    }

    if hostname.is_empty() {
        return Err("SSH host or alias is required.".to_string());
    }

    let port = if parsed_port_was_provided {
        let Some(port) = parsed_port else {
            return Err("SSH port must be between 1 and 65535.".to_string());
        };
        if !(1..=65_535).contains(&port) {
            return Err("SSH port must be between 1 and 65535.".to_string());
        }
        Some(port as u16)
    } else {
        None
    };

    Ok(DesktopSshEnvironmentTarget {
        alias: hostname.clone(),
        hostname,
        username,
        port,
    })
}

fn parse_bracketed_host_port(value: &str) -> Option<(String, Option<i64>)> {
    let rest = value.strip_prefix('[')?;
    let closing_index = rest.find(']')?;
    let hostname = rest[..closing_index].trim().to_string();
    let suffix = &rest[closing_index + 1..];
    if suffix.is_empty() {
        return Some((hostname, None));
    }
    let raw_port = suffix.strip_prefix(':')?;
    if raw_port.is_empty() || !raw_port.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    Some((hostname, raw_port.parse::<i64>().ok()))
}

fn parse_js_base10_int(value: &str) -> Option<i64> {
    let value = value.trim_start();
    let mut sign = 1_i64;
    let mut start = 0_usize;
    if let Some(first) = value.as_bytes().first().copied() {
        if first == b'-' {
            sign = -1;
            start = 1;
        } else if first == b'+' {
            start = 1;
        }
    }

    let digits = value[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<i64>().ok().map(|value| value * sign)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemotePairingFields {
    pub host: String,
    pub pairing_code: String,
}

pub fn parse_pairing_url_fields(input: &str) -> Option<RemotePairingFields> {
    let parsed = ParsedPairingUrl::parse(input.trim())?;
    let token = parsed.pairing_token()?;

    if let Some(host) = parsed.query_param("host") {
        let host = host.trim().to_string();
        if !host.is_empty() && !token.trim().is_empty() {
            return Some(RemotePairingFields {
                host,
                pairing_code: token.trim().to_string(),
            });
        }
    }

    Some(RemotePairingFields {
        host: parsed.origin,
        pairing_code: token.trim().to_string(),
    })
}

pub fn parse_remote_pairing_fields(
    host: &str,
    pairing_code: &str,
) -> Result<RemotePairingFields, String> {
    if let Some(parsed) = parse_pairing_url_fields(host) {
        return Ok(parsed);
    }

    let host = host.trim();
    let pairing_code = pairing_code.trim();
    if host.is_empty() {
        return Err("Enter a backend host.".to_string());
    }
    if pairing_code.is_empty() {
        return Err("Enter a pairing code.".to_string());
    }
    Ok(RemotePairingFields {
        host: host.to_string(),
        pairing_code: pairing_code.to_string(),
    })
}

pub fn format_desktop_ssh_connection_error(error_message: Option<&str>) -> String {
    const FALLBACK: &str = "Failed to connect SSH host.";
    let raw_message = error_message.unwrap_or(FALLBACK);
    let without_ipc_prefix = raw_message
        .strip_prefix("Error invoking remote method 'desktop:ensure-ssh-environment':")
        .map(str::trim_start)
        .unwrap_or(raw_message);
    let without_tagged_prefix =
        strip_ssh_tagged_error_prefix(without_ipc_prefix).unwrap_or(without_ipc_prefix);
    let message = without_tagged_prefix.trim();
    if message.is_empty() {
        FALLBACK.to_string()
    } else {
        message.to_string()
    }
}

fn strip_ssh_tagged_error_prefix(value: &str) -> Option<&str> {
    let suffix = value.strip_prefix("Ssh")?;
    let marker = suffix.find("Error:")?;
    let tag = &suffix[..marker];
    if tag.is_empty() || !tag.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    Some(suffix[marker + "Error:".len()..].trim_start())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvertisedEndpointStatus {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostedHttpsAppCompatibility {
    Compatible,
    Incompatible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvertisedEndpoint {
    pub id: String,
    pub provider_id: String,
    pub label: String,
    pub http_base_url: String,
    pub reachability: String,
    pub status: AdvertisedEndpointStatus,
    pub is_default: bool,
    pub hosted_https_app: HostedHttpsAppCompatibility,
}

pub fn is_tailscale_https_endpoint(endpoint: &AdvertisedEndpoint) -> bool {
    endpoint.id.starts_with("tailscale-magicdns:")
}

pub fn endpoint_default_preference_key(endpoint: &AdvertisedEndpoint) -> String {
    if endpoint.id.starts_with("desktop-loopback:") {
        return "desktop-core:loopback:http".to_string();
    }
    if endpoint.id.starts_with("desktop-lan:") {
        return "desktop-core:lan:http".to_string();
    }
    if endpoint.id.starts_with("tailscale-ip:") {
        return "tailscale:ip:http".to_string();
    }
    if is_tailscale_https_endpoint(endpoint) {
        return "tailscale:magicdns:https".to_string();
    }

    let scheme = ParsedPairingUrl::parse(&endpoint.http_base_url)
        .map(|url| url.scheme)
        .unwrap_or_else(|| "unknown".to_string());
    format!(
        "{}:{}:{}:{}",
        endpoint.provider_id, endpoint.reachability, scheme, endpoint.label
    )
}

pub fn select_pairing_endpoint<'a>(
    endpoints: &'a [AdvertisedEndpoint],
    default_endpoint_key: Option<&str>,
) -> Option<&'a AdvertisedEndpoint> {
    let available = endpoints
        .iter()
        .filter(|endpoint| endpoint.status != AdvertisedEndpointStatus::Unavailable)
        .collect::<Vec<_>>();

    if let Some(default_endpoint_key) = default_endpoint_key {
        if let Some(endpoint) = available
            .iter()
            .copied()
            .find(|endpoint| endpoint_default_preference_key(endpoint) == default_endpoint_key)
        {
            return Some(endpoint);
        }
    }

    available
        .iter()
        .copied()
        .find(|endpoint| endpoint.is_default)
        .or_else(|| {
            available
                .iter()
                .copied()
                .find(|endpoint| endpoint.reachability != "loopback")
        })
        .or_else(|| {
            available.iter().copied().find(|endpoint| {
                endpoint.hosted_https_app == HostedHttpsAppCompatibility::Compatible
            })
        })
}

pub fn resolve_desktop_pairing_url(endpoint_url: &str, credential: &str) -> Option<String> {
    let parsed = ParsedPairingUrl::parse(endpoint_url)?;
    Some(format!(
        "{}/pair#token={}",
        parsed.origin,
        form_url_encode_component(credential)
    ))
}

pub fn resolve_hosted_pairing_url(endpoint_url: &str, credential: &str) -> Option<String> {
    let parsed = ParsedPairingUrl::parse(endpoint_url)?;
    if parsed.scheme != "https" {
        return None;
    }
    Some(format!(
        "https://app.t3.codes/pair?host={}#token={}",
        form_url_encode_component(endpoint_url),
        form_url_encode_component(credential)
    ))
}

pub fn resolve_advertised_endpoint_pairing_url(
    endpoint: &AdvertisedEndpoint,
    credential: &str,
) -> Option<String> {
    if endpoint.hosted_https_app == HostedHttpsAppCompatibility::Compatible {
        return resolve_hosted_pairing_url(&endpoint.http_base_url, credential)
            .or_else(|| resolve_desktop_pairing_url(&endpoint.http_base_url, credential));
    }
    resolve_desktop_pairing_url(&endpoint.http_base_url, credential)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPairingLinkRecord {
    pub id: String,
    pub created_at: String,
}

pub fn sort_desktop_pairing_links(
    links: &[ServerPairingLinkRecord],
) -> Vec<ServerPairingLinkRecord> {
    let mut sorted = links.to_vec();
    sorted.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    sorted
}

pub fn upsert_desktop_pairing_link(
    current: &[ServerPairingLinkRecord],
    next: ServerPairingLinkRecord,
) -> Vec<ServerPairingLinkRecord> {
    let mut updated = current.to_vec();
    if let Some(existing_index) = updated
        .iter()
        .position(|pairing_link| pairing_link.id == next.id)
    {
        updated[existing_index] = next;
    } else {
        updated.push(next);
    }
    sort_desktop_pairing_links(&updated)
}

pub fn remove_desktop_pairing_link(
    current: &[ServerPairingLinkRecord],
    id: &str,
) -> Vec<ServerPairingLinkRecord> {
    current
        .iter()
        .filter(|pairing_link| pairing_link.id != id)
        .cloned()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerClientSessionRecord {
    pub session_id: String,
    pub issued_at: String,
    pub current: bool,
    pub connected: bool,
}

pub fn sort_desktop_client_sessions(
    sessions: &[ServerClientSessionRecord],
) -> Vec<ServerClientSessionRecord> {
    let mut sorted = sessions.to_vec();
    sorted.sort_by(|left, right| {
        right
            .current
            .cmp(&left.current)
            .then(right.connected.cmp(&left.connected))
            .then(right.issued_at.cmp(&left.issued_at))
    });
    sorted
}

pub fn upsert_desktop_client_session(
    current: &[ServerClientSessionRecord],
    next: ServerClientSessionRecord,
) -> Vec<ServerClientSessionRecord> {
    let mut updated = current.to_vec();
    if let Some(existing_index) = updated
        .iter()
        .position(|client_session| client_session.session_id == next.session_id)
    {
        updated[existing_index] = next;
    } else {
        updated.push(next);
    }
    sort_desktop_client_sessions(&updated)
}

pub fn remove_desktop_client_session(
    current: &[ServerClientSessionRecord],
    session_id: &str,
) -> Vec<ServerClientSessionRecord> {
    current
        .iter()
        .filter(|client_session| client_session.session_id != session_id)
        .cloned()
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticsDescriptionInput<'a> {
    pub local_tracing_enabled: bool,
    pub otlp_traces_enabled: bool,
    pub otlp_traces_url: Option<&'a str>,
    pub otlp_metrics_enabled: bool,
    pub otlp_metrics_url: Option<&'a str>,
}

pub fn collapse_otel_signals_url(traces_url: &str, metrics_url: &str) -> Option<String> {
    let traces_suffix = "/traces";
    let metrics_suffix = "/metrics";
    if !traces_url.ends_with(traces_suffix) || !metrics_url.ends_with(metrics_suffix) {
        return None;
    }

    let traces_base = &traces_url[..traces_url.len() - traces_suffix.len()];
    let metrics_base = &metrics_url[..metrics_url.len() - metrics_suffix.len()];
    if traces_base != metrics_base {
        return None;
    }

    Some(format!("{traces_base}/{{traces,metrics}}"))
}

pub fn format_diagnostics_description(input: DiagnosticsDescriptionInput<'_>) -> String {
    let mode = if input.local_tracing_enabled {
        "Local trace file"
    } else {
        "Terminal logs only"
    };
    let traces_url = input
        .otlp_traces_enabled
        .then_some(input.otlp_traces_url)
        .flatten();
    let metrics_url = input
        .otlp_metrics_enabled
        .then_some(input.otlp_metrics_url)
        .flatten();

    match (traces_url, metrics_url) {
        (Some(traces_url), Some(metrics_url)) => {
            if let Some(collapsed_url) = collapse_otel_signals_url(traces_url, metrics_url) {
                format!("{mode}. Exporting OTEL to {collapsed_url}.")
            } else {
                format!(
                    "{mode}. Exporting OTEL traces to {traces_url} and metrics to {metrics_url}."
                )
            }
        }
        (Some(traces_url), None) => format!("{mode}. Exporting OTEL traces to {traces_url}."),
        (None, Some(metrics_url)) => format!("{mode}. Exporting OTEL metrics to {metrics_url}."),
        (None, None) => format!("{mode}."),
    }
}

pub fn format_diagnostics_count(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    grouped.chars().rev().collect()
}

pub fn format_diagnostics_duration_ms(value: f64) -> String {
    if value < 1_000.0 {
        return format!("{} ms", value.round() as i64);
    }
    if value >= 10_000.0 {
        format!("{:.1} s", value / 1_000.0)
    } else {
        format!("{:.2} s", value / 1_000.0)
    }
}

pub fn format_diagnostics_bytes(value: u64) -> String {
    if value < 1024 {
        return format!("{value} B");
    }
    let units = ["KB", "MB", "GB"];
    let mut unit_index = 0_usize;
    let mut next = value as f64 / 1024.0;
    while next >= 1024.0 && unit_index < units.len() - 1 {
        next /= 1024.0;
        unit_index += 1;
    }
    if next >= 10.0 {
        format!("{next:.1} {}", units[unit_index])
    } else {
        format!("{next:.2} {}", units[unit_index])
    }
}

pub fn shorten_trace_id(trace_id: &str) -> String {
    if trace_id.len() <= 32 {
        return trace_id.to_string();
    }
    format!("{}...{}", &trace_id[..18], &trace_id[trace_id.len() - 10..])
}

pub fn is_stale_process_signal_message(message: Option<&str>) -> bool {
    message
        .map(|message| message.contains("not a live descendant"))
        .unwrap_or(false)
}

fn trimmed_non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.is_empty() { None } else { Some(value) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedPairingUrl {
    scheme: String,
    origin: String,
    query: String,
    hash: String,
}

impl ParsedPairingUrl {
    fn parse(input: &str) -> Option<Self> {
        if input.is_empty() {
            return None;
        }
        let url_like_input = if input.starts_with("//") {
            format!("https:{input}")
        } else if has_url_scheme_prefix(input) {
            input.to_string()
        } else {
            format!("https://{input}")
        };

        let scheme_end = url_like_input.find("://")?;
        let scheme = url_like_input[..scheme_end].to_ascii_lowercase();
        if scheme.is_empty() {
            return None;
        }
        let after_scheme = &url_like_input[scheme_end + 3..];
        let authority_end = after_scheme
            .find(|ch| matches!(ch, '/' | '?' | '#'))
            .unwrap_or(after_scheme.len());
        let authority = &after_scheme[..authority_end];
        if authority.is_empty() {
            return None;
        }
        let remainder = &after_scheme[authority_end..];
        let query = extract_url_query(remainder).unwrap_or_default();
        let hash = extract_url_hash(remainder).unwrap_or_default();

        Some(Self {
            scheme: scheme.clone(),
            origin: format!("{scheme}://{authority}"),
            query,
            hash,
        })
    }

    fn query_param(&self, name: &str) -> Option<String> {
        get_url_param(&self.query, name)
    }

    fn hash_param(&self, name: &str) -> Option<String> {
        get_url_param(&self.hash, name)
    }

    fn pairing_token(&self) -> Option<String> {
        self.hash_param("token")
            .filter(|token| !token.trim().is_empty())
            .or_else(|| {
                self.query_param("token")
                    .filter(|token| !token.trim().is_empty())
            })
    }
}

fn has_url_scheme_prefix(input: &str) -> bool {
    let Some((scheme, _)) = input.split_once("://") else {
        return false;
    };
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-'))
}

fn extract_url_query(remainder: &str) -> Option<String> {
    let query_start = remainder.find('?')? + 1;
    let query_end = remainder[query_start..]
        .find('#')
        .map(|index| query_start + index)
        .unwrap_or(remainder.len());
    Some(remainder[query_start..query_end].to_string())
}

fn extract_url_hash(remainder: &str) -> Option<String> {
    let hash_start = remainder.find('#')? + 1;
    Some(remainder[hash_start..].to_string())
}

fn get_url_param(params: &str, name: &str) -> Option<String> {
    params.split('&').find_map(|part| {
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        (percent_decode_form_component(key) == name).then(|| percent_decode_form_component(value))
    })
}

fn percent_decode_form_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0_usize;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let high = hex_value(bytes[index + 1]);
                let low = hex_value(bytes[index + 2]);
                if let (Some(high), Some(low)) = (high, low) {
                    output.push(high * 16 + low);
                    index += 3;
                } else {
                    output.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn form_url_encode_component(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            output.push(byte as char);
        } else if byte == b' ' {
            output.push('+');
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPhase {
    Disconnected,
    Connecting,
    Ready,
    Running,
    Error,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl MessageRole {
    pub fn display_author(self) -> &'static str {
        match self {
            Self::User => "You",
            Self::Assistant => APP_NAME,
            Self::System => "System",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatImageAttachment {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatAttachment {
    Image(ChatImageAttachment),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub text: String,
    pub attachments: Vec<ChatAttachment>,
    pub turn_id: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub streaming: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityTone {
    Thinking,
    Tool,
    Info,
    Error,
    Approval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalRequestKind {
    Command,
    FileRead,
    FileChange,
}

impl ApprovalRequestKind {
    pub fn from_request_type(request_type: &str) -> Option<Self> {
        match request_type {
            "command_execution_approval" | "exec_command_approval" | "dynamic_tool_call" => {
                Some(Self::Command)
            }
            "file_read_approval" => Some(Self::FileRead),
            "file_change_approval" | "apply_patch_approval" => Some(Self::FileChange),
            _ => None,
        }
    }

    pub fn summary(self) -> &'static str {
        match self {
            Self::Command => "Command approval requested",
            Self::FileRead => "File-read approval requested",
            Self::FileChange => "File-change approval requested",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActivityPayload {
    pub request_id: Option<String>,
    pub request_kind: Option<ApprovalRequestKind>,
    pub request_type: Option<String>,
    pub detail: Option<String>,
    pub command: Option<String>,
    pub raw_command: Option<String>,
    pub changed_files: Vec<String>,
    pub title: Option<String>,
    pub item_type: Option<String>,
    pub tool_call_id: Option<String>,
    pub questions: Vec<UserInputQuestion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadActivity {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub tone: ActivityTone,
    pub payload: ActivityPayload,
    pub turn_id: Option<String>,
    pub sequence: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingApproval {
    pub request_id: String,
    pub request_kind: ApprovalRequestKind,
    pub created_at: String,
    pub detail: Option<String>,
}

pub const MAX_VISIBLE_WORK_LOG_ENTRIES: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkLogEntry {
    pub id: String,
    pub activity_kind: String,
    pub created_at: String,
    pub label: String,
    pub detail: Option<String>,
    pub command: Option<String>,
    pub raw_command: Option<String>,
    pub changed_files: Vec<String>,
    pub tone: ActivityTone,
    pub tool_title: Option<String>,
    pub item_type: Option<String>,
    pub request_kind: Option<ApprovalRequestKind>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInputQuestionOption {
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInputQuestion {
    pub id: String,
    pub header: String,
    pub question: String,
    pub options: Vec<UserInputQuestionOption>,
    pub multi_select: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingUserInput {
    pub request_id: String,
    pub created_at: String,
    pub questions: Vec<UserInputQuestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PendingUserInputDraftAnswer {
    pub selected_option_labels: Vec<String>,
    pub custom_answer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingUserInputAnswer {
    Text(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingUserInputProgress {
    pub question_index: usize,
    pub active_question: Option<UserInputQuestion>,
    pub active_draft: Option<PendingUserInputDraftAnswer>,
    pub selected_option_labels: Vec<String>,
    pub custom_answer: String,
    pub resolved_answer: Option<PendingUserInputAnswer>,
    pub using_custom_answer: bool,
    pub answered_question_count: usize,
    pub is_last_question: bool,
    pub is_complete: bool,
    pub can_advance: bool,
}

fn normalize_draft_answer(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_selected_option_labels(value: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for label in value {
        let trimmed = label.trim();
        if trimmed.is_empty() || normalized.iter().any(|entry| entry == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized
}

pub fn resolve_pending_user_input_answer(
    question: &UserInputQuestion,
    draft: Option<&PendingUserInputDraftAnswer>,
) -> Option<PendingUserInputAnswer> {
    if let Some(custom_answer) =
        normalize_draft_answer(draft.and_then(|draft| draft.custom_answer.as_deref()))
    {
        return Some(PendingUserInputAnswer::Text(custom_answer));
    }

    let selected_option_labels = draft
        .map(|draft| normalize_selected_option_labels(&draft.selected_option_labels))
        .unwrap_or_default();
    if question.multi_select {
        return if selected_option_labels.is_empty() {
            None
        } else {
            Some(PendingUserInputAnswer::Multiple(selected_option_labels))
        };
    }

    selected_option_labels
        .first()
        .cloned()
        .map(PendingUserInputAnswer::Text)
}

pub fn set_pending_user_input_custom_answer(
    draft: Option<&PendingUserInputDraftAnswer>,
    custom_answer: impl Into<String>,
) -> PendingUserInputDraftAnswer {
    let custom_answer = custom_answer.into();
    let selected_option_labels = if custom_answer.trim().is_empty() {
        draft
            .map(|draft| normalize_selected_option_labels(&draft.selected_option_labels))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    PendingUserInputDraftAnswer {
        selected_option_labels,
        custom_answer: Some(custom_answer),
    }
}

pub fn toggle_pending_user_input_option_selection(
    question: &UserInputQuestion,
    draft: Option<&PendingUserInputDraftAnswer>,
    option_label: impl Into<String>,
) -> PendingUserInputDraftAnswer {
    let option_label = option_label.into();
    if question.multi_select {
        let mut selected_option_labels = draft
            .map(|draft| normalize_selected_option_labels(&draft.selected_option_labels))
            .unwrap_or_default();
        if selected_option_labels
            .iter()
            .any(|label| label == &option_label)
        {
            selected_option_labels.retain(|label| label != &option_label);
        } else {
            selected_option_labels.push(option_label);
        }

        return PendingUserInputDraftAnswer {
            selected_option_labels,
            custom_answer: Some(String::new()),
        };
    }

    PendingUserInputDraftAnswer {
        selected_option_labels: vec![option_label],
        custom_answer: Some(String::new()),
    }
}

pub fn build_pending_user_input_answers(
    questions: &[UserInputQuestion],
    draft_answers: &BTreeMap<String, PendingUserInputDraftAnswer>,
) -> Option<BTreeMap<String, PendingUserInputAnswer>> {
    let mut answers = BTreeMap::new();

    for question in questions {
        let answer = resolve_pending_user_input_answer(question, draft_answers.get(&question.id))?;
        answers.insert(question.id.clone(), answer);
    }

    Some(answers)
}

pub fn count_answered_pending_user_input_questions(
    questions: &[UserInputQuestion],
    draft_answers: &BTreeMap<String, PendingUserInputDraftAnswer>,
) -> usize {
    questions
        .iter()
        .filter(|question| {
            resolve_pending_user_input_answer(question, draft_answers.get(&question.id)).is_some()
        })
        .count()
}

pub fn find_first_unanswered_pending_user_input_question_index(
    questions: &[UserInputQuestion],
    draft_answers: &BTreeMap<String, PendingUserInputDraftAnswer>,
) -> usize {
    questions
        .iter()
        .position(|question| {
            resolve_pending_user_input_answer(question, draft_answers.get(&question.id)).is_none()
        })
        .unwrap_or_else(|| questions.len().saturating_sub(1))
}

pub fn derive_pending_user_input_progress(
    questions: &[UserInputQuestion],
    draft_answers: &BTreeMap<String, PendingUserInputDraftAnswer>,
    question_index: usize,
) -> PendingUserInputProgress {
    let normalized_question_index = if questions.is_empty() {
        0
    } else {
        question_index.min(questions.len() - 1)
    };
    let active_question = questions.get(normalized_question_index).cloned();
    let active_draft = active_question
        .as_ref()
        .and_then(|question| draft_answers.get(&question.id).cloned());
    let resolved_answer = active_question
        .as_ref()
        .and_then(|question| resolve_pending_user_input_answer(question, active_draft.as_ref()));
    let custom_answer = active_draft
        .as_ref()
        .and_then(|draft| draft.custom_answer.clone())
        .unwrap_or_default();
    let answered_question_count =
        count_answered_pending_user_input_questions(questions, draft_answers);
    let is_last_question = questions.is_empty() || normalized_question_index >= questions.len() - 1;

    PendingUserInputProgress {
        question_index: normalized_question_index,
        active_question,
        selected_option_labels: active_draft
            .as_ref()
            .map(|draft| normalize_selected_option_labels(&draft.selected_option_labels))
            .unwrap_or_default(),
        active_draft,
        using_custom_answer: custom_answer.trim().len() > 0,
        custom_answer,
        can_advance: resolved_answer.is_some(),
        resolved_answer,
        answered_question_count,
        is_last_question,
        is_complete: build_pending_user_input_answers(questions, draft_answers).is_some(),
    }
}

pub const DEFAULT_THREAD_TERMINAL_HEIGHT: u32 = 280;
pub const DEFAULT_THREAD_TERMINAL_ID: &str = "default";
pub const MAX_TERMINALS_PER_GROUP: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadTerminalGroup {
    pub id: String,
    pub terminal_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadTerminalState {
    pub terminal_open: bool,
    pub terminal_height: u32,
    pub terminal_ids: Vec<String>,
    pub running_terminal_ids: Vec<String>,
    pub active_terminal_id: String,
    pub terminal_groups: Vec<ThreadTerminalGroup>,
    pub active_terminal_group_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadTerminalLaunchContext {
    pub cwd: String,
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSessionSnapshot {
    pub thread_id: String,
    pub terminal_id: String,
    pub cwd: String,
    pub worktree_path: Option<String>,
    pub status: String,
    pub pid: Option<u32>,
    pub history: String,
    pub exit_code: Option<i32>,
    pub exit_signal: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEvent {
    Output {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        data: String,
    },
    Activity {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        has_running_subprocess: bool,
    },
    Error {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        message: String,
    },
    Cleared {
        thread_id: String,
        terminal_id: String,
        created_at: String,
    },
    Exited {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        exit_code: Option<i32>,
        exit_signal: Option<String>,
    },
    Started {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        snapshot: TerminalSessionSnapshot,
    },
    Restarted {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        snapshot: TerminalSessionSnapshot,
    },
}

impl TerminalEvent {
    pub fn terminal_id(&self) -> &str {
        match self {
            Self::Output { terminal_id, .. }
            | Self::Activity { terminal_id, .. }
            | Self::Error { terminal_id, .. }
            | Self::Cleared { terminal_id, .. }
            | Self::Exited { terminal_id, .. }
            | Self::Started { terminal_id, .. }
            | Self::Restarted { terminal_id, .. } => terminal_id,
        }
    }

    pub fn created_at(&self) -> &str {
        match self {
            Self::Output { created_at, .. }
            | Self::Activity { created_at, .. }
            | Self::Error { created_at, .. }
            | Self::Cleared { created_at, .. }
            | Self::Exited { created_at, .. }
            | Self::Started { created_at, .. }
            | Self::Restarted { created_at, .. } => created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalEventEntry {
    pub id: u64,
    pub event: TerminalEvent,
}

pub fn terminal_group_id(terminal_id: &str) -> String {
    format!("group-{terminal_id}")
}

pub fn create_default_thread_terminal_state() -> ThreadTerminalState {
    ThreadTerminalState {
        terminal_open: false,
        terminal_height: DEFAULT_THREAD_TERMINAL_HEIGHT,
        terminal_ids: vec![DEFAULT_THREAD_TERMINAL_ID.to_string()],
        running_terminal_ids: Vec::new(),
        active_terminal_id: DEFAULT_THREAD_TERMINAL_ID.to_string(),
        terminal_groups: vec![ThreadTerminalGroup {
            id: terminal_group_id(DEFAULT_THREAD_TERMINAL_ID),
            terminal_ids: vec![DEFAULT_THREAD_TERMINAL_ID.to_string()],
        }],
        active_terminal_group_id: terminal_group_id(DEFAULT_THREAD_TERMINAL_ID),
    }
}

fn normalize_terminal_ids(terminal_ids: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for terminal_id in terminal_ids {
        let trimmed = terminal_id.trim();
        if trimmed.is_empty() || normalized.iter().any(|id| id == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    if normalized.is_empty() {
        normalized.push(DEFAULT_THREAD_TERMINAL_ID.to_string());
    }
    normalized
}

fn normalize_running_terminal_ids(
    running_terminal_ids: &[String],
    terminal_ids: &[String],
) -> Vec<String> {
    let mut normalized = Vec::new();
    for terminal_id in running_terminal_ids {
        let trimmed = terminal_id.trim();
        if trimmed.is_empty()
            || !terminal_ids.iter().any(|id| id == trimmed)
            || normalized.iter().any(|id| id == trimmed)
        {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized
}

fn assign_unique_terminal_group_id(base_id: &str, used_group_ids: &mut Vec<String>) -> String {
    let base_id = if base_id.trim().is_empty() {
        terminal_group_id(DEFAULT_THREAD_TERMINAL_ID)
    } else {
        base_id.trim().to_string()
    };
    let mut candidate = base_id.clone();
    let mut index = 2;
    while used_group_ids.iter().any(|id| id == &candidate) {
        candidate = format!("{base_id}-{index}");
        index += 1;
    }
    used_group_ids.push(candidate.clone());
    candidate
}

fn normalize_terminal_group_ids(terminal_ids: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for terminal_id in terminal_ids {
        let trimmed = terminal_id.trim();
        if trimmed.is_empty() || normalized.iter().any(|id| id == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized
}

fn normalize_terminal_groups(
    terminal_groups: &[ThreadTerminalGroup],
    terminal_ids: &[String],
) -> Vec<ThreadTerminalGroup> {
    let mut assigned_terminal_ids = Vec::<String>::new();
    let mut used_group_ids = Vec::<String>::new();
    let mut next_groups = Vec::<ThreadTerminalGroup>::new();

    for group in terminal_groups {
        let mut group_terminal_ids = Vec::new();
        for terminal_id in normalize_terminal_group_ids(&group.terminal_ids) {
            if !terminal_ids.iter().any(|id| id == &terminal_id)
                || assigned_terminal_ids.iter().any(|id| id == &terminal_id)
            {
                continue;
            }
            assigned_terminal_ids.push(terminal_id.clone());
            group_terminal_ids.push(terminal_id);
        }
        if group_terminal_ids.is_empty() {
            continue;
        }
        let base_group_id = if group.id.trim().is_empty() {
            terminal_group_id(&group_terminal_ids[0])
        } else {
            group.id.trim().to_string()
        };
        next_groups.push(ThreadTerminalGroup {
            id: assign_unique_terminal_group_id(&base_group_id, &mut used_group_ids),
            terminal_ids: group_terminal_ids,
        });
    }

    for terminal_id in terminal_ids {
        if assigned_terminal_ids
            .iter()
            .any(|assigned| assigned == terminal_id)
        {
            continue;
        }
        next_groups.push(ThreadTerminalGroup {
            id: assign_unique_terminal_group_id(
                &terminal_group_id(terminal_id),
                &mut used_group_ids,
            ),
            terminal_ids: vec![terminal_id.clone()],
        });
    }

    if next_groups.is_empty() {
        return vec![ThreadTerminalGroup {
            id: terminal_group_id(DEFAULT_THREAD_TERMINAL_ID),
            terminal_ids: vec![DEFAULT_THREAD_TERMINAL_ID.to_string()],
        }];
    }
    next_groups
}

fn find_terminal_group_index_by_terminal_id(
    terminal_groups: &[ThreadTerminalGroup],
    terminal_id: &str,
) -> Option<usize> {
    terminal_groups
        .iter()
        .position(|group| group.terminal_ids.iter().any(|id| id == terminal_id))
}

pub fn normalize_thread_terminal_state(state: &ThreadTerminalState) -> ThreadTerminalState {
    let terminal_ids = normalize_terminal_ids(&state.terminal_ids);
    let running_terminal_ids =
        normalize_running_terminal_ids(&state.running_terminal_ids, &terminal_ids);
    let active_terminal_id = if terminal_ids
        .iter()
        .any(|terminal_id| terminal_id == &state.active_terminal_id)
    {
        state.active_terminal_id.clone()
    } else {
        terminal_ids[0].clone()
    };
    let terminal_groups = normalize_terminal_groups(&state.terminal_groups, &terminal_ids);
    let active_group_id_from_state = terminal_groups
        .iter()
        .any(|group| group.id == state.active_terminal_group_id)
        .then(|| state.active_terminal_group_id.clone());
    let active_group_id_from_terminal = terminal_groups
        .iter()
        .find(|group| {
            group
                .terminal_ids
                .iter()
                .any(|id| id == &active_terminal_id)
        })
        .map(|group| group.id.clone());

    ThreadTerminalState {
        terminal_open: state.terminal_open,
        terminal_height: if state.terminal_height > 0 {
            state.terminal_height
        } else {
            DEFAULT_THREAD_TERMINAL_HEIGHT
        },
        terminal_ids,
        running_terminal_ids,
        active_terminal_id,
        active_terminal_group_id: active_group_id_from_state
            .or(active_group_id_from_terminal)
            .unwrap_or_else(|| terminal_groups[0].id.clone()),
        terminal_groups,
    }
}

fn upsert_terminal_into_groups(
    state: &ThreadTerminalState,
    terminal_id: &str,
    split: bool,
) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    let terminal_id = terminal_id.trim();
    if terminal_id.is_empty() {
        return normalized;
    }

    let is_new_terminal = !normalized
        .terminal_ids
        .iter()
        .any(|existing| existing == terminal_id);
    let mut terminal_ids = normalized.terminal_ids.clone();
    if is_new_terminal {
        terminal_ids.push(terminal_id.to_string());
    }
    let mut terminal_groups = normalized.terminal_groups.clone();

    if let Some(existing_group_index) =
        find_terminal_group_index_by_terminal_id(&terminal_groups, terminal_id)
    {
        terminal_groups[existing_group_index]
            .terminal_ids
            .retain(|id| id != terminal_id);
        if terminal_groups[existing_group_index]
            .terminal_ids
            .is_empty()
        {
            terminal_groups.remove(existing_group_index);
        }
    }

    if !split {
        let mut used_group_ids = terminal_groups
            .iter()
            .map(|group| group.id.clone())
            .collect::<Vec<_>>();
        let next_group_id =
            assign_unique_terminal_group_id(&terminal_group_id(terminal_id), &mut used_group_ids);
        terminal_groups.push(ThreadTerminalGroup {
            id: next_group_id.clone(),
            terminal_ids: vec![terminal_id.to_string()],
        });
        return normalize_thread_terminal_state(&ThreadTerminalState {
            terminal_open: true,
            terminal_ids,
            active_terminal_id: terminal_id.to_string(),
            terminal_groups,
            active_terminal_group_id: next_group_id,
            ..normalized
        });
    }

    let mut active_group_index = terminal_groups
        .iter()
        .position(|group| group.id == normalized.active_terminal_group_id)
        .or_else(|| {
            find_terminal_group_index_by_terminal_id(
                &terminal_groups,
                &normalized.active_terminal_id,
            )
        });
    if active_group_index.is_none() {
        let mut used_group_ids = terminal_groups
            .iter()
            .map(|group| group.id.clone())
            .collect::<Vec<_>>();
        let group_id = assign_unique_terminal_group_id(
            &terminal_group_id(&normalized.active_terminal_id),
            &mut used_group_ids,
        );
        terminal_groups.push(ThreadTerminalGroup {
            id: group_id,
            terminal_ids: vec![normalized.active_terminal_id.clone()],
        });
        active_group_index = Some(terminal_groups.len() - 1);
    }

    let Some(active_group_index) = active_group_index else {
        return normalized;
    };
    let destination_group = &mut terminal_groups[active_group_index];
    if is_new_terminal
        && !destination_group
            .terminal_ids
            .iter()
            .any(|id| id == terminal_id)
        && destination_group.terminal_ids.len() >= MAX_TERMINALS_PER_GROUP
    {
        return normalized;
    }
    if !destination_group
        .terminal_ids
        .iter()
        .any(|id| id == terminal_id)
    {
        if let Some(anchor_index) = destination_group
            .terminal_ids
            .iter()
            .position(|id| id == &normalized.active_terminal_id)
        {
            destination_group
                .terminal_ids
                .insert(anchor_index + 1, terminal_id.to_string());
        } else {
            destination_group.terminal_ids.push(terminal_id.to_string());
        }
    }
    let active_terminal_group_id = destination_group.id.clone();

    normalize_thread_terminal_state(&ThreadTerminalState {
        terminal_open: true,
        terminal_ids,
        active_terminal_id: terminal_id.to_string(),
        terminal_groups,
        active_terminal_group_id,
        ..normalized
    })
}

pub fn set_thread_terminal_open(state: &ThreadTerminalState, open: bool) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    if normalized.terminal_open == open {
        return normalized;
    }
    ThreadTerminalState {
        terminal_open: open,
        ..normalized
    }
}

pub fn set_thread_terminal_height(state: &ThreadTerminalState, height: u32) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    if height == 0 || normalized.terminal_height == height {
        return normalized;
    }
    ThreadTerminalState {
        terminal_height: height,
        ..normalized
    }
}

pub fn split_thread_terminal(
    state: &ThreadTerminalState,
    terminal_id: &str,
) -> ThreadTerminalState {
    upsert_terminal_into_groups(state, terminal_id, true)
}

pub fn new_thread_terminal(state: &ThreadTerminalState, terminal_id: &str) -> ThreadTerminalState {
    upsert_terminal_into_groups(state, terminal_id, false)
}

pub fn set_thread_active_terminal(
    state: &ThreadTerminalState,
    terminal_id: &str,
) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    if !normalized.terminal_ids.iter().any(|id| id == terminal_id) {
        return normalized;
    }
    let active_terminal_group_id = normalized
        .terminal_groups
        .iter()
        .find(|group| group.terminal_ids.iter().any(|id| id == terminal_id))
        .map(|group| group.id.clone())
        .unwrap_or_else(|| normalized.active_terminal_group_id.clone());
    if normalized.active_terminal_id == terminal_id
        && normalized.active_terminal_group_id == active_terminal_group_id
    {
        return normalized;
    }
    ThreadTerminalState {
        active_terminal_id: terminal_id.to_string(),
        active_terminal_group_id,
        ..normalized
    }
}

pub fn close_thread_terminal(
    state: &ThreadTerminalState,
    terminal_id: &str,
) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    if !normalized.terminal_ids.iter().any(|id| id == terminal_id) {
        return normalized;
    }
    let remaining_terminal_ids = normalized
        .terminal_ids
        .iter()
        .filter(|id| id.as_str() != terminal_id)
        .cloned()
        .collect::<Vec<_>>();
    if remaining_terminal_ids.is_empty() {
        return create_default_thread_terminal_state();
    }
    let closed_terminal_index = normalized
        .terminal_ids
        .iter()
        .position(|id| id == terminal_id)
        .unwrap_or(0);
    let next_active_terminal_id = if normalized.active_terminal_id == terminal_id {
        remaining_terminal_ids
            .get(closed_terminal_index.min(remaining_terminal_ids.len() - 1))
            .cloned()
            .unwrap_or_else(|| remaining_terminal_ids[0].clone())
    } else {
        normalized.active_terminal_id.clone()
    };
    let terminal_groups = normalized
        .terminal_groups
        .iter()
        .filter_map(|group| {
            let terminal_ids = group
                .terminal_ids
                .iter()
                .filter(|id| id.as_str() != terminal_id)
                .cloned()
                .collect::<Vec<_>>();
            (!terminal_ids.is_empty()).then(|| ThreadTerminalGroup {
                id: group.id.clone(),
                terminal_ids,
            })
        })
        .collect::<Vec<_>>();
    let active_terminal_group_id = terminal_groups
        .iter()
        .find(|group| {
            group
                .terminal_ids
                .iter()
                .any(|id| id == &next_active_terminal_id)
        })
        .map(|group| group.id.clone())
        .unwrap_or_else(|| terminal_group_id(&next_active_terminal_id));

    normalize_thread_terminal_state(&ThreadTerminalState {
        terminal_ids: remaining_terminal_ids,
        running_terminal_ids: normalized
            .running_terminal_ids
            .into_iter()
            .filter(|id| id != terminal_id)
            .collect(),
        active_terminal_id: next_active_terminal_id,
        terminal_groups,
        active_terminal_group_id,
        ..normalized
    })
}

pub fn set_thread_terminal_activity(
    state: &ThreadTerminalState,
    terminal_id: &str,
    has_running_subprocess: bool,
) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    if !normalized.terminal_ids.iter().any(|id| id == terminal_id) {
        return normalized;
    }
    let already_running = normalized
        .running_terminal_ids
        .iter()
        .any(|id| id == terminal_id);
    if already_running == has_running_subprocess {
        return normalized;
    }
    let mut running_terminal_ids = normalized.running_terminal_ids.clone();
    if has_running_subprocess {
        running_terminal_ids.push(terminal_id.to_string());
    } else {
        running_terminal_ids.retain(|id| id != terminal_id);
    }
    ThreadTerminalState {
        running_terminal_ids,
        ..normalized
    }
}

pub fn terminal_running_subprocess_from_event(event: &TerminalEvent) -> Option<bool> {
    match event {
        TerminalEvent::Activity {
            has_running_subprocess,
            ..
        } => Some(*has_running_subprocess),
        TerminalEvent::Started { .. }
        | TerminalEvent::Restarted { .. }
        | TerminalEvent::Exited { .. } => Some(false),
        TerminalEvent::Output { .. }
        | TerminalEvent::Error { .. }
        | TerminalEvent::Cleared { .. } => None,
    }
}

pub fn select_terminal_event_entries_after_snapshot(
    entries: &[TerminalEventEntry],
    snapshot_updated_at: &str,
) -> Vec<TerminalEventEntry> {
    entries
        .iter()
        .filter(|entry| entry.event.created_at() > snapshot_updated_at)
        .cloned()
        .collect()
}

pub fn select_pending_terminal_event_entries(
    entries: &[TerminalEventEntry],
    last_applied_terminal_event_id: u64,
) -> Vec<TerminalEventEntry> {
    entries
        .iter()
        .filter(|entry| entry.id > last_applied_terminal_event_id)
        .cloned()
        .collect()
}

fn activity_lifecycle_rank(kind: &str) -> i32 {
    if kind.ends_with(".started") || kind == "tool.started" {
        return 0;
    }
    if kind.ends_with(".progress") || kind.ends_with(".updated") {
        return 1;
    }
    if kind.ends_with(".completed") || kind.ends_with(".resolved") {
        return 2;
    }
    1
}

fn sorted_activities(activities: &[ThreadActivity]) -> Vec<ThreadActivity> {
    let mut ordered = activities.to_vec();
    ordered.sort_by(|left, right| match (left.sequence, right.sequence) {
        (Some(left_sequence), Some(right_sequence)) if left_sequence != right_sequence => {
            left_sequence.cmp(&right_sequence)
        }
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        _ => left
            .created_at
            .cmp(&right.created_at)
            .then_with(|| {
                activity_lifecycle_rank(&left.kind).cmp(&activity_lifecycle_rank(&right.kind))
            })
            .then_with(|| left.id.cmp(&right.id)),
    });
    ordered
}

fn normalize_compact_tool_label(value: &str) -> String {
    let trimmed = value.trim();
    for suffix in [" complete", " completed"] {
        if trimmed.to_ascii_lowercase().ends_with(suffix) {
            return trimmed[..trimmed.len() - suffix.len()].trim().to_string();
        }
    }
    trimmed.to_string()
}

fn is_plan_boundary_tool_activity(activity: &ThreadActivity) -> bool {
    matches!(activity.kind.as_str(), "tool.updated" | "tool.completed")
        && activity
            .payload
            .detail
            .as_deref()
            .map(|detail| detail.starts_with("ExitPlanMode:"))
            .unwrap_or(false)
}

fn is_excluded_work_log_activity(activity: &ThreadActivity) -> bool {
    matches!(
        activity.kind.as_str(),
        "tool.started" | "task.started" | "context-window.updated"
    ) || activity.summary == "Checkpoint captured"
        || is_plan_boundary_tool_activity(activity)
}

fn work_log_tone(activity: &ThreadActivity) -> ActivityTone {
    if activity.kind == "task.progress" {
        ActivityTone::Thinking
    } else if activity.tone == ActivityTone::Approval {
        ActivityTone::Info
    } else {
        activity.tone
    }
}

fn work_log_collapse_key(entry: &WorkLogEntry) -> Option<String> {
    if let Some(tool_call_id) = entry.tool_call_id.as_ref() {
        return Some(format!("tool:{tool_call_id}"));
    }
    let label = normalize_compact_tool_label(entry.tool_title.as_deref().unwrap_or(&entry.label));
    let detail = entry.detail.as_deref().unwrap_or("").trim();
    let item_type = entry.item_type.as_deref().unwrap_or("");
    if label.is_empty() && detail.is_empty() && item_type.is_empty() {
        None
    } else {
        Some(format!("{item_type}\u{1f}{label}\u{1f}{detail}"))
    }
}

fn should_collapse_work_log_entries(previous: &WorkLogEntry, next: &WorkLogEntry) -> bool {
    matches!(previous.activity_kind.as_str(), "tool.updated")
        && matches!(
            next.activity_kind.as_str(),
            "tool.updated" | "tool.completed"
        )
        && work_log_collapse_key(previous).is_some()
        && work_log_collapse_key(previous) == work_log_collapse_key(next)
}

fn merge_work_log_entries(previous: WorkLogEntry, next: WorkLogEntry) -> WorkLogEntry {
    let mut changed_files = previous.changed_files;
    for file in next.changed_files {
        if !changed_files.iter().any(|existing| existing == &file) {
            changed_files.push(file);
        }
    }

    WorkLogEntry {
        detail: next.detail.or(previous.detail),
        command: next.command.or(previous.command),
        raw_command: next.raw_command.or(previous.raw_command),
        changed_files,
        tool_title: next.tool_title.or(previous.tool_title),
        item_type: next.item_type.or(previous.item_type),
        request_kind: next.request_kind.or(previous.request_kind),
        tool_call_id: next.tool_call_id.or(previous.tool_call_id),
        ..next
    }
}

fn thread_activity_to_work_log_entry(activity: ThreadActivity) -> WorkLogEntry {
    let is_task_activity = activity.kind == "task.progress" || activity.kind == "task.completed";
    let label = if is_task_activity {
        activity
            .payload
            .title
            .clone()
            .or_else(|| activity.payload.detail.clone())
            .unwrap_or_else(|| activity.summary.clone())
    } else {
        activity.summary.clone()
    };
    let tone = work_log_tone(&activity);
    let request_kind = activity.payload.request_kind.or_else(|| {
        activity
            .payload
            .request_type
            .as_deref()
            .and_then(ApprovalRequestKind::from_request_type)
    });

    WorkLogEntry {
        id: activity.id,
        activity_kind: activity.kind,
        created_at: activity.created_at,
        label,
        detail: activity.payload.detail,
        command: activity.payload.command,
        raw_command: activity.payload.raw_command,
        changed_files: activity.payload.changed_files,
        tone,
        tool_title: activity.payload.title,
        item_type: activity.payload.item_type,
        request_kind,
        tool_call_id: activity.payload.tool_call_id,
    }
}

pub fn derive_work_log_entries(
    activities: &[ThreadActivity],
    latest_turn_id: Option<&str>,
) -> Vec<WorkLogEntry> {
    let mut collapsed = Vec::<WorkLogEntry>::new();

    for activity in sorted_activities(activities) {
        if latest_turn_id
            .map(|turn_id| activity.turn_id.as_deref() != Some(turn_id))
            .unwrap_or(false)
        {
            continue;
        }
        if is_excluded_work_log_activity(&activity) {
            continue;
        }
        let entry = thread_activity_to_work_log_entry(activity);
        if let Some(previous) = collapsed.pop() {
            if should_collapse_work_log_entries(&previous, &entry) {
                collapsed.push(merge_work_log_entries(previous, entry));
            } else {
                collapsed.push(previous);
                collapsed.push(entry);
            }
        } else {
            collapsed.push(entry);
        }
    }

    collapsed
}

fn is_stale_pending_request_failure_detail(detail: Option<&str>) -> bool {
    let Some(detail) = detail else {
        return false;
    };
    let normalized = detail.to_ascii_lowercase();
    normalized.contains("stale pending approval request")
        || normalized.contains("stale pending user-input request")
        || normalized.contains("unknown pending approval request")
        || normalized.contains("unknown pending permission request")
        || normalized.contains("unknown pending user-input request")
}

pub fn derive_pending_approvals(activities: &[ThreadActivity]) -> Vec<PendingApproval> {
    let mut open_by_request_id = BTreeMap::<String, PendingApproval>::new();

    for activity in sorted_activities(activities) {
        let Some(request_id) = activity.payload.request_id.clone() else {
            continue;
        };
        let request_kind = activity.payload.request_kind.or_else(|| {
            activity
                .payload
                .request_type
                .as_deref()
                .and_then(ApprovalRequestKind::from_request_type)
        });

        match activity.kind.as_str() {
            "approval.requested" => {
                if let Some(request_kind) = request_kind {
                    open_by_request_id.insert(
                        request_id.clone(),
                        PendingApproval {
                            request_id,
                            request_kind,
                            created_at: activity.created_at,
                            detail: activity.payload.detail,
                        },
                    );
                }
            }
            "approval.resolved" => {
                open_by_request_id.remove(&request_id);
            }
            "provider.approval.respond.failed"
                if is_stale_pending_request_failure_detail(activity.payload.detail.as_deref()) =>
            {
                open_by_request_id.remove(&request_id);
            }
            _ => {}
        }
    }

    let mut pending = open_by_request_id.into_values().collect::<Vec<_>>();
    pending.sort_by(|left, right| left.created_at.cmp(&right.created_at));
    pending
}

pub fn derive_pending_user_inputs(activities: &[ThreadActivity]) -> Vec<PendingUserInput> {
    let mut open_by_request_id = BTreeMap::<String, PendingUserInput>::new();

    for activity in sorted_activities(activities) {
        let Some(request_id) = activity.payload.request_id.clone() else {
            continue;
        };

        match activity.kind.as_str() {
            "user-input.requested" => {
                if !activity.payload.questions.is_empty() {
                    open_by_request_id.insert(
                        request_id.clone(),
                        PendingUserInput {
                            request_id,
                            created_at: activity.created_at,
                            questions: activity.payload.questions,
                        },
                    );
                }
            }
            "user-input.resolved" => {
                open_by_request_id.remove(&request_id);
            }
            "provider.user-input.respond.failed"
                if is_stale_pending_request_failure_detail(activity.payload.detail.as_deref()) =>
            {
                open_by_request_id.remove(&request_id);
            }
            _ => {}
        }
    }

    let mut pending = open_by_request_id.into_values().collect::<Vec<_>>();
    pending.sort_by(|left, right| left.created_at.cmp(&right.created_at));
    pending
}

impl ChatMessage {
    pub fn user(
        id: impl Into<String>,
        text: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            role: MessageRole::User,
            text: text.into(),
            attachments: Vec::new(),
            turn_id: None,
            created_at: created_at.into(),
            completed_at: None,
            streaming: false,
        }
    }

    pub fn assistant(
        id: impl Into<String>,
        text: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            role: MessageRole::Assistant,
            text: text.into(),
            attachments: Vec::new(),
            turn_id: None,
            created_at: created_at.into(),
            completed_at: None,
            streaming: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedPlan {
    pub id: String,
    pub turn_id: Option<String>,
    pub plan_markdown: String,
    pub implemented_at: Option<String>,
    pub implementation_thread_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnDiffFileChange {
    pub path: String,
    pub kind: Option<String>,
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TurnDiffStat {
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnDiffTreeNode {
    Directory {
        name: String,
        path: String,
        stat: TurnDiffStat,
        children: Vec<TurnDiffTreeNode>,
    },
    File {
        name: String,
        path: String,
        stat: Option<TurnDiffStat>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffRouteSearch {
    pub diff: Option<String>,
    pub diff_turn_id: Option<String>,
    pub diff_file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffOpenValue {
    String(String),
    Number(i32),
    Bool(bool),
}

impl From<&str> for DiffOpenValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

fn is_diff_open_value(value: Option<&DiffOpenValue>) -> bool {
    matches!(
        value,
        Some(DiffOpenValue::String(value)) if value == "1"
    ) || matches!(
        value,
        Some(DiffOpenValue::Number(1)) | Some(DiffOpenValue::Bool(true))
    )
}

fn normalize_search_string(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn parse_diff_route_search(
    diff: Option<DiffOpenValue>,
    diff_turn_id: Option<&str>,
    diff_file_path: Option<&str>,
) -> DiffRouteSearch {
    if !is_diff_open_value(diff.as_ref()) {
        return DiffRouteSearch::default();
    }
    let diff_turn_id = normalize_search_string(diff_turn_id);
    let diff_file_path = diff_turn_id
        .as_ref()
        .and_then(|_| normalize_search_string(diff_file_path));

    DiffRouteSearch {
        diff: Some("1".to_string()),
        diff_turn_id,
        diff_file_path,
    }
}

pub fn summarize_turn_diff_stats(files: &[TurnDiffFileChange]) -> TurnDiffStat {
    files
        .iter()
        .fold(TurnDiffStat::default(), |mut stat, file| {
            if let (Some(additions), Some(deletions)) = (file.additions, file.deletions) {
                stat.additions += additions;
                stat.deletions += deletions;
            }
            stat
        })
}

pub fn has_non_zero_turn_diff_stat(stat: TurnDiffStat) -> bool {
    stat.additions > 0 || stat.deletions > 0
}

#[derive(Debug, Clone)]
struct MutableDiffDirectory {
    name: String,
    path: String,
    stat: TurnDiffStat,
    directories: BTreeMap<String, MutableDiffDirectory>,
    files: Vec<TurnDiffTreeNode>,
}

fn normalize_diff_path_segments(path: &str) -> Vec<String> {
    path.replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

fn read_turn_diff_stat(file: &TurnDiffFileChange) -> Option<TurnDiffStat> {
    Some(TurnDiffStat {
        additions: file.additions?,
        deletions: file.deletions?,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum DiffSortToken {
    Text(String),
    Number(u128),
}

fn diff_sort_tokens(value: &str) -> Vec<DiffSortToken> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_is_digit = None;

    for character in value.chars() {
        let is_digit = character.is_ascii_digit();
        if current_is_digit == Some(is_digit) {
            current.push(character);
            continue;
        }
        if !current.is_empty() {
            if current_is_digit == Some(true) {
                tokens.push(DiffSortToken::Number(current.parse().unwrap_or(0)));
            } else {
                tokens.push(DiffSortToken::Text(current.to_ascii_lowercase()));
            }
        }
        current.clear();
        current.push(character);
        current_is_digit = Some(is_digit);
    }

    if !current.is_empty() {
        if current_is_digit == Some(true) {
            tokens.push(DiffSortToken::Number(current.parse().unwrap_or(0)));
        } else {
            tokens.push(DiffSortToken::Text(current.to_ascii_lowercase()));
        }
    }

    tokens
}

fn compare_diff_names(left: &str, right: &str) -> std::cmp::Ordering {
    diff_sort_tokens(left)
        .cmp(&diff_sort_tokens(right))
        .then_with(|| left.cmp(right))
}

fn compare_diff_node_name(left: &TurnDiffTreeNode, right: &TurnDiffTreeNode) -> std::cmp::Ordering {
    compare_diff_names(diff_node_name(left), diff_node_name(right))
}

fn diff_node_name(node: &TurnDiffTreeNode) -> &str {
    match node {
        TurnDiffTreeNode::Directory { name, .. } | TurnDiffTreeNode::File { name, .. } => name,
    }
}

fn compact_diff_directory_node(node: TurnDiffTreeNode) -> TurnDiffTreeNode {
    let TurnDiffTreeNode::Directory {
        mut name,
        mut path,
        mut stat,
        mut children,
    } = node
    else {
        return node;
    };

    children = children
        .into_iter()
        .map(|child| match child {
            TurnDiffTreeNode::Directory { .. } => compact_diff_directory_node(child),
            TurnDiffTreeNode::File { .. } => child,
        })
        .collect();

    loop {
        if children.len() != 1 {
            break;
        }
        match children.pop().expect("single child exists") {
            TurnDiffTreeNode::Directory {
                name: child_name,
                path: child_path,
                stat: child_stat,
                children: child_children,
            } => {
                name = format!("{name}/{child_name}");
                path = child_path;
                stat = child_stat;
                children = child_children;
            }
            child @ TurnDiffTreeNode::File { .. } => {
                children.push(child);
                break;
            }
        }
    }

    TurnDiffTreeNode::Directory {
        name,
        path,
        stat,
        children,
    }
}

fn to_turn_diff_tree_nodes(directory: MutableDiffDirectory) -> Vec<TurnDiffTreeNode> {
    let mut directories = directory
        .directories
        .into_values()
        .map(|directory| {
            compact_diff_directory_node(TurnDiffTreeNode::Directory {
                name: directory.name.clone(),
                path: directory.path.clone(),
                stat: directory.stat,
                children: to_turn_diff_tree_nodes(directory),
            })
        })
        .collect::<Vec<_>>();
    let mut files = directory.files;
    directories.sort_by(compare_diff_node_name);
    files.sort_by(compare_diff_node_name);
    directories.extend(files);
    directories
}

pub fn build_turn_diff_tree(files: &[TurnDiffFileChange]) -> Vec<TurnDiffTreeNode> {
    let mut root = MutableDiffDirectory {
        name: String::new(),
        path: String::new(),
        stat: TurnDiffStat::default(),
        directories: BTreeMap::new(),
        files: Vec::new(),
    };

    for file in files {
        let segments = normalize_diff_path_segments(&file.path);
        let Some(file_name) = segments.last().cloned() else {
            continue;
        };
        let file_path = segments.join("/");
        let stat = read_turn_diff_stat(file);
        let mut current = &mut root;
        if let Some(stat) = stat {
            current.stat.additions += stat.additions;
            current.stat.deletions += stat.deletions;
        }
        for segment in &segments[..segments.len().saturating_sub(1)] {
            let next_path = if current.path.is_empty() {
                segment.clone()
            } else {
                format!("{}/{}", current.path, segment)
            };
            current = current
                .directories
                .entry(segment.clone())
                .or_insert_with(|| MutableDiffDirectory {
                    name: segment.clone(),
                    path: next_path,
                    stat: TurnDiffStat::default(),
                    directories: BTreeMap::new(),
                    files: Vec::new(),
                });
            if let Some(stat) = stat {
                current.stat.additions += stat.additions;
                current.stat.deletions += stat.deletions;
            }
        }
        current.files.push(TurnDiffTreeNode::File {
            name: file_name,
            path: file_path,
            stat,
        });
    }

    to_turn_diff_tree_nodes(root)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnDiffSummary {
    pub turn_id: String,
    pub completed_at: String,
    pub status: Option<String>,
    pub files: Vec<TurnDiffFileChange>,
    pub checkpoint_ref: Option<String>,
    pub assistant_message_id: Option<String>,
    pub checkpoint_turn_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSession {
    pub provider: String,
    pub provider_instance_id: Option<String>,
    pub status: SessionPhase,
    pub active_turn_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_error: Option<String>,
    pub orchestration_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadShell {
    pub id: String,
    pub environment_id: String,
    pub codex_thread_id: Option<String>,
    pub project_id: String,
    pub title: String,
    pub runtime_mode: RuntimeMode,
    pub interaction_mode: ProviderInteractionMode,
    pub error: Option<String>,
    pub created_at: String,
    pub archived_at: Option<String>,
    pub updated_at: Option<String>,
    pub branch: Option<String>,
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadTurnState {
    pub latest_turn: Option<LatestTurn>,
    pub pending_source_proposed_plan: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestTurn {
    pub turn_id: String,
    pub state: String,
    pub requested_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub assistant_message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thread {
    pub shell: ThreadShell,
    pub session: Option<ThreadSession>,
    pub latest_turn: Option<LatestTurn>,
    pub pending_source_proposed_plan: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub activities: Vec<ThreadActivity>,
    pub proposed_plans: Vec<ProposedPlan>,
    pub turn_diff_summaries: Vec<TurnDiffSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnvironmentState {
    pub thread_shell_by_id: BTreeMap<String, ThreadShell>,
    pub thread_session_by_id: BTreeMap<String, ThreadSession>,
    pub thread_turn_state_by_id: BTreeMap<String, ThreadTurnState>,
    pub message_ids_by_thread_id: BTreeMap<String, Vec<String>>,
    pub message_by_thread_id: BTreeMap<String, BTreeMap<String, ChatMessage>>,
    pub activity_ids_by_thread_id: BTreeMap<String, Vec<String>>,
    pub activity_by_thread_id: BTreeMap<String, BTreeMap<String, ThreadActivity>>,
    pub proposed_plan_ids_by_thread_id: BTreeMap<String, Vec<String>>,
    pub proposed_plan_by_thread_id: BTreeMap<String, BTreeMap<String, ProposedPlan>>,
    pub turn_diff_ids_by_thread_id: BTreeMap<String, Vec<String>>,
    pub turn_diff_summary_by_thread_id: BTreeMap<String, BTreeMap<String, TurnDiffSummary>>,
}

fn collect_by_ids<T: Clone>(
    ids_by_owner: &BTreeMap<String, Vec<String>>,
    records_by_owner: &BTreeMap<String, BTreeMap<String, T>>,
    owner_id: &str,
) -> Vec<T> {
    let Some(ids) = ids_by_owner.get(owner_id) else {
        return Vec::new();
    };
    let Some(records) = records_by_owner.get(owner_id) else {
        return Vec::new();
    };

    ids.iter()
        .filter_map(|id| records.get(id).cloned())
        .collect()
}

pub fn get_thread_from_environment_state(
    state: &EnvironmentState,
    thread_id: &str,
) -> Option<Thread> {
    let shell = state.thread_shell_by_id.get(thread_id)?.clone();
    let turn_state = state.thread_turn_state_by_id.get(thread_id);

    Some(Thread {
        shell,
        session: state.thread_session_by_id.get(thread_id).cloned(),
        latest_turn: turn_state.and_then(|state| state.latest_turn.clone()),
        pending_source_proposed_plan: turn_state
            .and_then(|state| state.pending_source_proposed_plan.clone()),
        messages: collect_by_ids(
            &state.message_ids_by_thread_id,
            &state.message_by_thread_id,
            thread_id,
        ),
        activities: collect_by_ids(
            &state.activity_ids_by_thread_id,
            &state.activity_by_thread_id,
            thread_id,
        ),
        proposed_plans: collect_by_ids(
            &state.proposed_plan_ids_by_thread_id,
            &state.proposed_plan_by_thread_id,
            thread_id,
        ),
        turn_diff_summaries: collect_by_ids(
            &state.turn_diff_ids_by_thread_id,
            &state.turn_diff_summary_by_thread_id,
            thread_id,
        ),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSnapshot {
    pub route: ChatRoute,
    pub projects: Vec<ProjectSummary>,
    pub threads: Vec<ThreadSummary>,
    pub is_git_repo: bool,
    pub available_environments: Vec<BranchToolbarEnvironmentOption>,
    pub vcs_refs: Vec<VcsRef>,
    pub current_git_branch: Option<String>,
    pub primary_environment_id: Option<String>,
    pub available_editors: Vec<EditorId>,
    pub preferred_editor: Option<EditorId>,
    pub providers: Vec<ServerProvider>,
    pub selected_provider_instance_id: String,
    pub selected_model: String,
    pub model_favorites: Vec<ProviderModelFavorite>,
    pub messages: Vec<ChatMessage>,
    pub activities: Vec<ThreadActivity>,
    pub draft_sessions: Vec<DraftSessionState>,
    pub pending_approvals: Vec<PendingApproval>,
    pub pending_user_inputs: Vec<PendingUserInput>,
    pub pending_user_input_draft_answers: BTreeMap<String, PendingUserInputDraftAnswer>,
    pub active_pending_user_input_question_index: usize,
    pub responding_request_ids: Vec<String>,
    pub terminal_state: ThreadTerminalState,
    pub terminal_launch_context: Option<ThreadTerminalLaunchContext>,
    pub terminal_event_entries: Vec<TerminalEventEntry>,
    pub diff_route: DiffRouteSearch,
    pub turn_diff_summaries: Vec<TurnDiffSummary>,
}

impl AppSnapshot {
    fn reference_environments() -> Vec<BranchToolbarEnvironmentOption> {
        vec![
            BranchToolbarEnvironmentOption {
                environment_id: "local".to_string(),
                project_id: "project-r3code".to_string(),
                label: resolve_environment_option_label(
                    true,
                    "local",
                    Some("Local environment"),
                    Some("Local"),
                ),
                is_primary: true,
            },
            BranchToolbarEnvironmentOption {
                environment_id: "environment-build-box".to_string(),
                project_id: "project-r3code".to_string(),
                label: resolve_environment_option_label(
                    false,
                    "environment-build-box",
                    None,
                    Some("Build box"),
                ),
                is_primary: false,
            },
        ]
    }

    fn reference_vcs_refs() -> Vec<VcsRef> {
        dedupe_remote_branches_with_local_matches(&[
            VcsRef {
                name: "main".to_string(),
                current: true,
                is_default: true,
                is_remote: false,
                remote_name: None,
                worktree_path: None,
            },
            VcsRef {
                name: "feature/parity-branch-toolbar".to_string(),
                current: false,
                is_default: false,
                is_remote: false,
                remote_name: None,
                worktree_path: Some(
                    "C:\\Users\\bunny\\Downloads\\r3code\\.t3\\worktrees\\branch-toolbar"
                        .to_string(),
                ),
            },
            VcsRef {
                name: "origin/main".to_string(),
                current: false,
                is_default: true,
                is_remote: true,
                remote_name: Some("origin".to_string()),
                worktree_path: None,
            },
            VcsRef {
                name: "origin/feature/remote-only".to_string(),
                current: false,
                is_default: false,
                is_remote: true,
                remote_name: Some("origin".to_string()),
                worktree_path: None,
            },
        ])
    }

    fn reference_project_scripts() -> Vec<ProjectScript> {
        vec![
            ProjectScript {
                id: "test".to_string(),
                name: "Test".to_string(),
                command: "cargo test --workspace".to_string(),
                icon: ProjectScriptIcon::Test,
                run_on_worktree_create: false,
            },
            ProjectScript {
                id: "setup".to_string(),
                name: "Setup".to_string(),
                command: "cargo fetch".to_string(),
                icon: ProjectScriptIcon::Configure,
                run_on_worktree_create: true,
            },
        ]
    }

    fn reference_providers() -> Vec<ServerProvider> {
        vec![
            ServerProvider {
                instance_id: "codex".to_string(),
                driver: "codex".to_string(),
                display_name: Some("Codex".to_string()),
                accent_color: None,
                badge_label: None,
                continuation_group_key: Some("codex-default".to_string()),
                show_interaction_mode_toggle: true,
                enabled: true,
                installed: true,
                version: Some("0.49.0".to_string()),
                status: ServerProviderState::Ready,
                auth: ServerProviderAuth {
                    status: ServerProviderAuthStatus::Authenticated,
                    kind: Some("codex".to_string()),
                    label: Some("Codex CLI".to_string()),
                    email: Some("dev@example.com".to_string()),
                },
                checked_at: "2026-03-04T12:00:00.000Z".to_string(),
                message: None,
                availability: ServerProviderAvailability::Available,
                unavailable_reason: None,
                models: vec![
                    ServerProviderModel {
                        slug: "gpt-5.4".to_string(),
                        name: "GPT-5.4".to_string(),
                        short_name: Some("5.4".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                    ServerProviderModel {
                        slug: "gpt-5.4-mini".to_string(),
                        name: "GPT-5.4 Mini".to_string(),
                        short_name: Some("5.4 Mini".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                    ServerProviderModel {
                        slug: "gpt-5.3-codex".to_string(),
                        name: "GPT-5.3 Codex".to_string(),
                        short_name: Some("5.3".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                ],
                version_advisory: Some(ServerProviderVersionAdvisory {
                    status: ServerProviderVersionAdvisoryStatus::BehindLatest,
                    current_version: Some("0.49.0".to_string()),
                    latest_version: Some("0.50.0".to_string()),
                    update_command: Some("npm install -g @openai/codex@latest".to_string()),
                    can_update: true,
                    checked_at: Some("2026-03-04T12:00:00.000Z".to_string()),
                    message: None,
                }),
            },
            ServerProvider {
                instance_id: "codex_personal".to_string(),
                driver: "codex".to_string(),
                display_name: Some("Codex".to_string()),
                accent_color: Some("#2563EB".to_string()),
                badge_label: Some("Personal".to_string()),
                continuation_group_key: Some("codex-personal".to_string()),
                show_interaction_mode_toggle: true,
                enabled: true,
                installed: true,
                version: Some("0.50.0".to_string()),
                status: ServerProviderState::Ready,
                auth: ServerProviderAuth {
                    status: ServerProviderAuthStatus::Authenticated,
                    kind: Some("codex".to_string()),
                    label: Some("Personal".to_string()),
                    email: Some("personal@example.com".to_string()),
                },
                checked_at: "2026-03-04T12:00:00.000Z".to_string(),
                message: None,
                availability: ServerProviderAvailability::Available,
                unavailable_reason: None,
                models: vec![
                    ServerProviderModel {
                        slug: "gpt-5.4".to_string(),
                        name: "GPT-5.4".to_string(),
                        short_name: Some("5.4".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                    ServerProviderModel {
                        slug: "internal-review".to_string(),
                        name: "internal-review".to_string(),
                        short_name: None,
                        sub_provider: Some("OpenAI".to_string()),
                        is_custom: true,
                    },
                ],
                version_advisory: Some(ServerProviderVersionAdvisory {
                    status: ServerProviderVersionAdvisoryStatus::Current,
                    current_version: Some("0.50.0".to_string()),
                    latest_version: Some("0.50.0".to_string()),
                    update_command: None,
                    can_update: false,
                    checked_at: Some("2026-03-04T12:00:00.000Z".to_string()),
                    message: None,
                }),
            },
            ServerProvider {
                instance_id: "claudeAgent".to_string(),
                driver: "claudeAgent".to_string(),
                display_name: Some("Claude".to_string()),
                accent_color: None,
                badge_label: None,
                continuation_group_key: Some("claude-default".to_string()),
                show_interaction_mode_toggle: false,
                enabled: true,
                installed: true,
                version: Some("1.2.3".to_string()),
                status: ServerProviderState::Ready,
                auth: ServerProviderAuth {
                    status: ServerProviderAuthStatus::Authenticated,
                    kind: Some("oauth".to_string()),
                    label: Some("Claude Max".to_string()),
                    email: None,
                },
                checked_at: "2026-03-04T12:00:00.000Z".to_string(),
                message: None,
                availability: ServerProviderAvailability::Available,
                unavailable_reason: None,
                models: vec![
                    ServerProviderModel {
                        slug: "claude-sonnet-4-6".to_string(),
                        name: "Claude Sonnet 4.6".to_string(),
                        short_name: Some("Sonnet 4.6".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                    ServerProviderModel {
                        slug: "claude-haiku-4-5".to_string(),
                        name: "Claude Haiku 4.5".to_string(),
                        short_name: Some("Haiku 4.5".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                ],
                version_advisory: None,
            },
            ServerProvider {
                instance_id: "cursor".to_string(),
                driver: "cursor".to_string(),
                display_name: Some("Cursor".to_string()),
                accent_color: None,
                badge_label: None,
                continuation_group_key: None,
                show_interaction_mode_toggle: false,
                enabled: false,
                installed: false,
                version: None,
                status: ServerProviderState::Disabled,
                auth: ServerProviderAuth {
                    status: ServerProviderAuthStatus::Unknown,
                    kind: None,
                    label: None,
                    email: None,
                },
                checked_at: "2026-03-04T12:00:00.000Z".to_string(),
                message: Some("Cursor CLI not detected on PATH.".to_string()),
                availability: ServerProviderAvailability::Unavailable,
                unavailable_reason: Some("Driver unavailable in this build.".to_string()),
                models: vec![ServerProviderModel {
                    slug: "composer-2".to_string(),
                    name: "Composer 2".to_string(),
                    short_name: Some("Composer".to_string()),
                    sub_provider: None,
                    is_custom: false,
                }],
                version_advisory: None,
            },
            ServerProvider {
                instance_id: "opencode".to_string(),
                driver: "opencode".to_string(),
                display_name: Some("OpenCode".to_string()),
                accent_color: None,
                badge_label: Some("Preview".to_string()),
                continuation_group_key: None,
                show_interaction_mode_toggle: false,
                enabled: true,
                installed: true,
                version: Some("0.8.1".to_string()),
                status: ServerProviderState::Warning,
                auth: ServerProviderAuth {
                    status: ServerProviderAuthStatus::Unknown,
                    kind: None,
                    label: None,
                    email: None,
                },
                checked_at: "2026-03-04T12:00:00.000Z".to_string(),
                message: Some("Server could not verify OpenCode authentication.".to_string()),
                availability: ServerProviderAvailability::Available,
                unavailable_reason: None,
                models: vec![ServerProviderModel {
                    slug: "openai/gpt-5".to_string(),
                    name: "OpenAI GPT-5".to_string(),
                    short_name: Some("GPT-5".to_string()),
                    sub_provider: Some("OpenAI".to_string()),
                    is_custom: false,
                }],
                version_advisory: None,
            },
        ]
    }

    fn reference_model_favorites() -> Vec<ProviderModelFavorite> {
        vec![
            ProviderModelFavorite {
                provider: "codex".to_string(),
                model: "gpt-5.4".to_string(),
            },
            ProviderModelFavorite {
                provider: "claudeAgent".to_string(),
                model: "claude-sonnet-4-6".to_string(),
            },
        ]
    }

    pub fn empty_reference_state() -> Self {
        Self {
            route: ChatRoute::Index,
            projects: Vec::new(),
            threads: Vec::new(),
            is_git_repo: false,
            available_environments: Vec::new(),
            vcs_refs: Vec::new(),
            current_git_branch: None,
            primary_environment_id: None,
            available_editors: Vec::new(),
            preferred_editor: None,
            providers: Self::reference_providers(),
            selected_provider_instance_id: "codex".to_string(),
            selected_model: DEFAULT_GIT_TEXT_GENERATION_MODEL.to_string(),
            model_favorites: Self::reference_model_favorites(),
            messages: Vec::new(),
            activities: Vec::new(),
            draft_sessions: Vec::new(),
            pending_approvals: Vec::new(),
            pending_user_inputs: Vec::new(),
            pending_user_input_draft_answers: BTreeMap::new(),
            active_pending_user_input_question_index: 0,
            responding_request_ids: Vec::new(),
            terminal_state: create_default_thread_terminal_state(),
            terminal_launch_context: None,
            terminal_event_entries: Vec::new(),
            diff_route: DiffRouteSearch::default(),
            turn_diff_summaries: Vec::new(),
        }
    }

    pub fn draft_reference_state() -> Self {
        let draft_id = "draft-r3code-reference".to_string();
        let thread_ref = ScopedThreadRef::new("local", "thread-r3code-reference");
        let project_ref = ScopedProjectRef::new("local", "project-r3code");

        Self {
            route: ChatRoute::Thread(ThreadRouteTarget::Draft {
                draft_id: draft_id.clone(),
            }),
            projects: vec![ProjectSummary {
                id: "project-r3code".to_string(),
                environment_id: "local".to_string(),
                name: "server".to_string(),
                path: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
                scripts: Vec::new(),
            }],
            threads: Vec::new(),
            is_git_repo: false,
            available_environments: Self::reference_environments(),
            vcs_refs: Self::reference_vcs_refs(),
            current_git_branch: Some("main".to_string()),
            primary_environment_id: Some("local".to_string()),
            available_editors: vec![EditorId::VsCode, EditorId::FileManager],
            preferred_editor: Some(EditorId::VsCode),
            providers: Self::reference_providers(),
            selected_provider_instance_id: "codex".to_string(),
            selected_model: DEFAULT_GIT_TEXT_GENERATION_MODEL.to_string(),
            model_favorites: Self::reference_model_favorites(),
            messages: Vec::new(),
            activities: Vec::new(),
            draft_sessions: vec![DraftSessionState {
                draft_id,
                thread_ref,
                project_ref,
                logical_project_key: "local:project-r3code".to_string(),
                created_at: "2026-05-11T00:00:00.000Z".to_string(),
                runtime_mode: RuntimeMode::FullAccess,
                interaction_mode: ProviderInteractionMode::Default,
                branch: None,
                worktree_path: None,
                env_mode: DraftThreadEnvMode::Local,
                promoted_to: None,
            }],
            pending_approvals: Vec::new(),
            pending_user_inputs: Vec::new(),
            pending_user_input_draft_answers: BTreeMap::new(),
            active_pending_user_input_question_index: 0,
            responding_request_ids: Vec::new(),
            terminal_state: create_default_thread_terminal_state(),
            terminal_launch_context: None,
            terminal_event_entries: Vec::new(),
            diff_route: DiffRouteSearch::default(),
            turn_diff_summaries: Vec::new(),
        }
    }

    pub fn mock_reference_state() -> Self {
        Self {
            route: ChatRoute::Thread(ThreadRouteTarget::Server {
                thread_ref: ScopedThreadRef::new("local", "thread-r3code-ui-shell"),
            }),
            projects: vec![ProjectSummary {
                id: "project-r3code".to_string(),
                environment_id: "local".to_string(),
                name: "r3code".to_string(),
                path: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
                scripts: Self::reference_project_scripts(),
            }],
            is_git_repo: true,
            available_environments: Self::reference_environments(),
            vcs_refs: Self::reference_vcs_refs(),
            current_git_branch: Some("main".to_string()),
            primary_environment_id: Some("local".to_string()),
            available_editors: vec![EditorId::VsCode, EditorId::FileManager],
            preferred_editor: Some(EditorId::VsCode),
            providers: Self::reference_providers(),
            selected_provider_instance_id: "codex".to_string(),
            selected_model: DEFAULT_GIT_TEXT_GENERATION_MODEL.to_string(),
            model_favorites: Self::reference_model_favorites(),
            threads: vec![
                ThreadSummary {
                    id: "thread-r3code-ui-shell".to_string(),
                    environment_id: "local".to_string(),
                    project_id: "project-r3code".to_string(),
                    title: "Port R3Code UI shell".to_string(),
                    project_name: "r3code".to_string(),
                    status: ThreadStatus::Running,
                    created_at: "2026-03-04T11:59:00.000Z".to_string(),
                    updated_at: "2026-03-04T12:00:12.000Z".to_string(),
                    archived_at: None,
                    latest_user_message_at: Some("2026-03-04T12:00:09.000Z".to_string()),
                    has_pending_approvals: false,
                    has_pending_user_input: false,
                    has_actionable_proposed_plan: false,
                    branch: Some("main".to_string()),
                    worktree_path: None,
                },
                ThreadSummary {
                    id: "thread-visual-references".to_string(),
                    environment_id: "local".to_string(),
                    project_id: "project-r3code".to_string(),
                    title: "Capture visual references".to_string(),
                    project_name: "r3code".to_string(),
                    status: ThreadStatus::Idle,
                    created_at: "2026-03-03T14:12:00.000Z".to_string(),
                    updated_at: "2026-03-03T14:32:00.000Z".to_string(),
                    archived_at: None,
                    latest_user_message_at: None,
                    has_pending_approvals: false,
                    has_pending_user_input: false,
                    has_actionable_proposed_plan: false,
                    branch: Some("feature/parity-branch-toolbar".to_string()),
                    worktree_path: Some(
                        "C:\\Users\\bunny\\Downloads\\r3code\\.t3\\worktrees\\branch-toolbar"
                            .to_string(),
                    ),
                },
            ],
            messages: vec![
                ChatMessage::user(
                    "msg-user-r3code-ui-shell",
                    "Make the Rust port match the original UI exactly.",
                    "2026-03-04T12:00:09.000Z",
                ),
                ChatMessage::assistant(
                    "msg-assistant-r3code-ui-shell",
                    "Building a static GPUI shell first, then replacing mock data with Rust state.",
                    "2026-03-04T12:00:12.000Z",
                ),
            ],
            activities: Vec::new(),
            draft_sessions: Vec::new(),
            pending_approvals: Vec::new(),
            pending_user_inputs: Vec::new(),
            pending_user_input_draft_answers: BTreeMap::new(),
            active_pending_user_input_question_index: 0,
            responding_request_ids: Vec::new(),
            terminal_state: create_default_thread_terminal_state(),
            terminal_launch_context: None,
            terminal_event_entries: Vec::new(),
            diff_route: DiffRouteSearch::default(),
            turn_diff_summaries: Vec::new(),
        }
    }

    pub fn active_chat_reference_state() -> Self {
        let mut snapshot = Self::mock_reference_state();
        snapshot.turn_diff_summaries = reference_turn_diff_summaries();
        snapshot
    }

    pub fn branch_toolbar_reference_state() -> Self {
        let mut snapshot = Self::draft_reference_state();
        snapshot.is_git_repo = true;
        if let Some(draft) = snapshot.draft_sessions.first_mut() {
            draft.env_mode = DraftThreadEnvMode::Worktree;
            draft.branch = None;
            draft.worktree_path = None;
        }
        snapshot
    }

    pub fn running_turn_reference_state() -> Self {
        let mut snapshot = Self::mock_reference_state();
        if let Some(thread) = snapshot.threads.first_mut() {
            thread.status = ThreadStatus::Running;
        }
        snapshot.messages = vec![ChatMessage::user(
            "msg-user-running-turn",
            "Run the parity harness and fix any failures.",
            "2026-03-04T12:10:00.000Z",
        )];
        snapshot.activities = vec![
            ThreadActivity {
                id: "activity-thinking".to_string(),
                kind: "task.progress".to_string(),
                summary: "Inspecting changed surfaces".to_string(),
                tone: ActivityTone::Thinking,
                payload: ActivityPayload {
                    detail: Some("Reading upstream MessagesTimeline work log behavior".to_string()),
                    ..ActivityPayload::default()
                },
                turn_id: Some("turn-running-1".to_string()),
                sequence: Some(1),
                created_at: "2026-03-04T12:10:02.000Z".to_string(),
            },
            ThreadActivity {
                id: "activity-command".to_string(),
                kind: "tool.completed".to_string(),
                summary: "Ran command".to_string(),
                tone: ActivityTone::Tool,
                payload: ActivityPayload {
                    command: Some("cargo test --workspace".to_string()),
                    title: Some("terminal".to_string()),
                    item_type: Some("command_execution".to_string()),
                    tool_call_id: Some("tool-run-tests".to_string()),
                    ..ActivityPayload::default()
                },
                turn_id: Some("turn-running-1".to_string()),
                sequence: Some(2),
                created_at: "2026-03-04T12:10:08.000Z".to_string(),
            },
            ThreadActivity {
                id: "activity-files".to_string(),
                kind: "tool.completed".to_string(),
                summary: "Edited files".to_string(),
                tone: ActivityTone::Tool,
                payload: ActivityPayload {
                    changed_files: vec![
                        "crates/r3_core/src/lib.rs".to_string(),
                        "crates/r3_ui/src/shell.rs".to_string(),
                    ],
                    title: Some("file change".to_string()),
                    item_type: Some("file_change".to_string()),
                    tool_call_id: Some("tool-edit-files".to_string()),
                    ..ActivityPayload::default()
                },
                turn_id: Some("turn-running-1".to_string()),
                sequence: Some(3),
                created_at: "2026-03-04T12:10:14.000Z".to_string(),
            },
        ];
        snapshot
    }

    pub fn pending_approval_reference_state() -> Self {
        let mut snapshot = Self::mock_reference_state();
        if let Some(thread) = snapshot.threads.first_mut() {
            thread.status = ThreadStatus::NeedsInput;
            thread.has_pending_approvals = true;
        }
        snapshot.pending_approvals = vec![
            PendingApproval {
                request_id: "approval-command-run-tests".to_string(),
                request_kind: ApprovalRequestKind::Command,
                created_at: "2026-03-04T12:00:20.000Z".to_string(),
                detail: Some("cargo test --workspace".to_string()),
            },
            PendingApproval {
                request_id: "approval-file-change".to_string(),
                request_kind: ApprovalRequestKind::FileChange,
                created_at: "2026-03-04T12:00:23.000Z".to_string(),
                detail: Some("Allow editing crates/r3_ui/src/shell.rs".to_string()),
            },
        ];
        snapshot
    }

    pub fn pending_user_input_reference_state() -> Self {
        let mut snapshot = Self::mock_reference_state();
        if let Some(thread) = snapshot.threads.first_mut() {
            thread.status = ThreadStatus::NeedsInput;
            thread.has_pending_user_input = true;
        }
        snapshot.pending_user_inputs = vec![PendingUserInput {
            request_id: "user-input-port-scope".to_string(),
            created_at: "2026-03-04T12:00:24.000Z".to_string(),
            questions: vec![
                UserInputQuestion {
                    id: "surface".to_string(),
                    header: "Surface".to_string(),
                    question: "Which surface should the Rust port match first?".to_string(),
                    options: vec![
                        UserInputQuestionOption {
                            label: "Composer".to_string(),
                            description: "Pending approval and user input states".to_string(),
                        },
                        UserInputQuestionOption {
                            label: "Terminal".to_string(),
                            description: "Drawer and command session state".to_string(),
                        },
                        UserInputQuestionOption {
                            label: "Diff".to_string(),
                            description: "Changed files and line review".to_string(),
                        },
                    ],
                    multi_select: false,
                },
                UserInputQuestion {
                    id: "coverage".to_string(),
                    header: "Coverage".to_string(),
                    question: "Select every state this parity pass should capture.".to_string(),
                    options: vec![
                        UserInputQuestionOption {
                            label: "Light".to_string(),
                            description: "Light theme".to_string(),
                        },
                        UserInputQuestionOption {
                            label: "Dark".to_string(),
                            description: "Dark theme".to_string(),
                        },
                        UserInputQuestionOption {
                            label: "Focused".to_string(),
                            description: "Composer focus state".to_string(),
                        },
                    ],
                    multi_select: true,
                },
            ],
        }];
        snapshot.pending_user_input_draft_answers = BTreeMap::from([(
            "surface".to_string(),
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Composer".to_string()],
                custom_answer: Some(String::new()),
            },
        )]);
        snapshot
    }

    pub fn terminal_drawer_reference_state() -> Self {
        let mut snapshot = Self::mock_reference_state();
        let thread_id = "thread-r3code-ui-shell".to_string();
        snapshot.terminal_state =
            split_thread_terminal(&create_default_thread_terminal_state(), "terminal-2");
        snapshot.terminal_launch_context = Some(ThreadTerminalLaunchContext {
            cwd: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
            worktree_path: None,
        });
        snapshot.terminal_event_entries = vec![
            TerminalEventEntry {
                id: 1,
                event: TerminalEvent::Started {
                    thread_id: thread_id.clone(),
                    terminal_id: "default".to_string(),
                    created_at: "2026-03-04T12:00:14.000Z".to_string(),
                    snapshot: TerminalSessionSnapshot {
                        thread_id: thread_id.clone(),
                        terminal_id: "default".to_string(),
                        cwd: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
                        worktree_path: None,
                        status: "running".to_string(),
                        pid: Some(24012),
                        history: String::new(),
                        exit_code: None,
                        exit_signal: None,
                        updated_at: "2026-03-04T12:00:14.000Z".to_string(),
                    },
                },
            },
            TerminalEventEntry {
                id: 2,
                event: TerminalEvent::Output {
                    thread_id: thread_id.clone(),
                    terminal_id: "default".to_string(),
                    created_at: "2026-03-04T12:00:15.000Z".to_string(),
                    data: "PS C:\\Users\\bunny\\Downloads\\r3code> cargo check --workspace\r\n"
                        .to_string(),
                },
            },
            TerminalEventEntry {
                id: 3,
                event: TerminalEvent::Activity {
                    thread_id: thread_id.clone(),
                    terminal_id: "terminal-2".to_string(),
                    created_at: "2026-03-04T12:00:16.000Z".to_string(),
                    has_running_subprocess: true,
                },
            },
            TerminalEventEntry {
                id: 4,
                event: TerminalEvent::Output {
                    thread_id,
                    terminal_id: "terminal-2".to_string(),
                    created_at: "2026-03-04T12:00:17.000Z".to_string(),
                    data: "Running upstream capture fixture...\r\n".to_string(),
                },
            },
        ];
        snapshot
    }

    pub fn diff_panel_reference_state() -> Self {
        let mut snapshot = Self::active_chat_reference_state();
        snapshot.diff_route = parse_diff_route_search(
            Some(DiffOpenValue::from("1")),
            Some("turn-r3code-ui-shell-2"),
            Some("crates/r3_ui/src/shell.rs"),
        );
        snapshot
    }

    pub fn renders_chat_view(&self) -> bool {
        self.route.renders_chat_view()
    }

    pub fn active_thread_summary(&self) -> Option<&ThreadSummary> {
        self.threads.first()
    }

    pub fn active_thread_title(&self) -> &str {
        self.active_thread_summary()
            .map(|thread| thread.title.as_str())
            .unwrap_or("New thread")
    }

    pub fn active_project_name(&self) -> Option<&str> {
        self.projects.first().map(|project| project.name.as_str())
    }

    pub fn active_project(&self) -> Option<&ProjectSummary> {
        self.projects.first()
    }

    pub fn active_environment_id(&self) -> Option<&str> {
        match &self.route {
            ChatRoute::Thread(ThreadRouteTarget::Server { thread_ref }) => {
                Some(thread_ref.environment_id.as_str())
            }
            ChatRoute::Thread(ThreadRouteTarget::Draft { draft_id }) => self
                .draft_sessions
                .iter()
                .find(|draft| &draft.draft_id == draft_id)
                .map(|draft| draft.thread_ref.environment_id.as_str()),
            ChatRoute::Index => None,
        }
    }

    pub fn open_in_picker_visible(&self) -> bool {
        let Some(active_environment_id) = self.active_environment_id() else {
            return false;
        };
        should_show_open_in_picker(
            self.active_project_name(),
            active_environment_id,
            self.primary_environment_id.as_deref(),
        )
    }

    pub fn active_editor_option(&self, platform: &str) -> Option<EditorOption> {
        let options = resolve_editor_options(platform, &self.available_editors);
        self.preferred_editor
            .and_then(|preferred| {
                options
                    .iter()
                    .copied()
                    .find(|option| option.id == preferred)
            })
            .or_else(|| options.first().copied())
    }

    pub fn active_draft_session(&self) -> Option<&DraftSessionState> {
        match &self.route {
            ChatRoute::Thread(ThreadRouteTarget::Draft { draft_id }) => self
                .draft_sessions
                .iter()
                .find(|draft| &draft.draft_id == draft_id),
            _ => None,
        }
    }

    pub fn active_thread_branch(&self) -> Option<&str> {
        self.active_draft_session()
            .and_then(|draft| draft.branch.as_deref())
            .or_else(|| {
                self.active_thread_summary()
                    .and_then(|thread| thread.branch.as_deref())
            })
    }

    pub fn active_worktree_path(&self) -> Option<&str> {
        self.active_draft_session()
            .and_then(|draft| draft.worktree_path.as_deref())
            .or_else(|| {
                self.active_thread_summary()
                    .and_then(|thread| thread.worktree_path.as_deref())
            })
    }

    pub fn active_branch_toolbar_state(&self) -> Option<BranchToolbarState> {
        if !self.renders_chat_view() || !self.is_git_repo {
            return None;
        }

        let environment = self
            .available_environments
            .first()
            .cloned()
            .unwrap_or_else(|| BranchToolbarEnvironmentOption {
                environment_id: "local".to_string(),
                project_id: "project-local".to_string(),
                label: "This device".to_string(),
                is_primary: true,
            });
        let active_worktree_path = self.active_worktree_path().map(str::to_string);
        let has_server_thread = matches!(
            self.route,
            ChatRoute::Thread(ThreadRouteTarget::Server { .. })
        ) && self.active_thread_summary().is_some();
        let draft_thread_env_mode = self.active_draft_session().map(|draft| draft.env_mode);
        let effective_env_mode = resolve_effective_env_mode(
            active_worktree_path.as_deref(),
            has_server_thread,
            draft_thread_env_mode,
        );
        let resolved_active_branch = resolve_branch_toolbar_value(
            effective_env_mode,
            active_worktree_path.as_deref(),
            self.active_thread_branch(),
            self.current_git_branch.as_deref(),
        );
        let env_mode_locked = has_server_thread && active_worktree_path.is_some();
        let workspace_label = if env_mode_locked {
            resolve_locked_workspace_label(active_worktree_path.as_deref())
        } else if effective_env_mode == DraftThreadEnvMode::Worktree {
            resolve_env_mode_label(DraftThreadEnvMode::Worktree)
        } else {
            resolve_current_workspace_label(active_worktree_path.as_deref())
        };
        let branch_label = branch_toolbar_trigger_label(
            active_worktree_path.as_deref(),
            effective_env_mode,
            resolved_active_branch.as_deref(),
        );

        Some(BranchToolbarState {
            environment_id: environment.environment_id,
            environment_label: environment.label,
            environment_is_primary: environment.is_primary,
            show_environment_picker: self.available_environments.len() > 1,
            effective_env_mode,
            env_locked: false,
            env_mode_locked,
            active_worktree_path,
            workspace_label,
            branch_label,
            resolved_active_branch,
        })
    }

    pub fn set_active_draft_env_mode(&mut self, mode: DraftThreadEnvMode) {
        let ChatRoute::Thread(ThreadRouteTarget::Draft { draft_id }) = &self.route else {
            return;
        };
        if let Some(draft) = self
            .draft_sessions
            .iter_mut()
            .find(|draft| &draft.draft_id == draft_id)
        {
            draft.env_mode = mode;
            if mode == DraftThreadEnvMode::Worktree && draft.branch.is_none() {
                draft.branch = self.current_git_branch.clone();
            }
        }
    }

    pub fn select_branch_for_active_thread(&mut self, branch: impl Into<String>) {
        let branch = branch.into();
        if let Some(ref_name) = self
            .vcs_refs
            .iter()
            .find(|ref_name| ref_name.name == branch)
        {
            self.current_git_branch = Some(if ref_name.is_remote {
                derive_local_branch_name_from_remote_ref(&ref_name.name)
            } else {
                ref_name.name.clone()
            });
            if let ChatRoute::Thread(ThreadRouteTarget::Draft { draft_id }) = &self.route {
                if let Some(draft) = self
                    .draft_sessions
                    .iter_mut()
                    .find(|draft| &draft.draft_id == draft_id)
                {
                    let next_env_mode = resolve_draft_env_mode_after_branch_change(
                        ref_name.worktree_path.as_deref(),
                        draft.worktree_path.as_deref(),
                        draft.env_mode,
                    );
                    draft.branch = self.current_git_branch.clone();
                    draft.worktree_path = ref_name.worktree_path.clone();
                    draft.env_mode = next_env_mode;
                }
            } else if let Some(thread) = self.threads.first_mut() {
                thread.branch = self.current_git_branch.clone();
                thread.worktree_path = ref_name.worktree_path.clone();
            }
        }
    }

    pub fn active_pending_approval(&self) -> Option<&PendingApproval> {
        self.pending_approvals.first()
    }

    pub fn active_pending_user_input(&self) -> Option<&PendingUserInput> {
        self.pending_user_inputs.first()
    }

    pub fn active_pending_user_input_progress(&self) -> Option<PendingUserInputProgress> {
        let prompt = self.active_pending_user_input()?;
        Some(derive_pending_user_input_progress(
            &prompt.questions,
            &self.pending_user_input_draft_answers,
            self.active_pending_user_input_question_index,
        ))
    }

    pub fn is_responding_to_request(&self, request_id: &str) -> bool {
        self.responding_request_ids
            .iter()
            .any(|responding_id| responding_id == request_id)
    }

    pub fn terminal_open(&self) -> bool {
        self.terminal_state.terminal_open
    }

    pub fn work_log_entries(&self) -> Vec<WorkLogEntry> {
        let latest_turn_id = self
            .activities
            .iter()
            .rev()
            .find_map(|activity| activity.turn_id.as_deref());
        derive_work_log_entries(&self.activities, latest_turn_id)
    }

    pub fn diff_open(&self) -> bool {
        self.diff_route.diff.as_deref() == Some("1")
    }

    pub fn ordered_turn_diff_summaries(&self) -> Vec<&TurnDiffSummary> {
        let mut summaries = self.turn_diff_summaries.iter().collect::<Vec<_>>();
        summaries.sort_by(|left, right| {
            right
                .checkpoint_turn_count
                .unwrap_or(0)
                .cmp(&left.checkpoint_turn_count.unwrap_or(0))
                .then_with(|| right.completed_at.cmp(&left.completed_at))
        });
        summaries
    }

    pub fn selected_turn_diff_summary(&self) -> Option<&TurnDiffSummary> {
        let selected_turn_id = self.diff_route.diff_turn_id.as_ref()?;
        self.turn_diff_summaries
            .iter()
            .find(|summary| &summary.turn_id == selected_turn_id)
            .or_else(|| self.ordered_turn_diff_summaries().first().copied())
    }

    pub fn selected_diff_file_path(&self) -> Option<&str> {
        self.diff_route
            .diff_turn_id
            .as_ref()
            .and(self.diff_route.diff_file_path.as_deref())
    }
}

fn reference_turn_diff_summaries() -> Vec<TurnDiffSummary> {
    vec![
        TurnDiffSummary {
            turn_id: "turn-r3code-ui-shell-2".to_string(),
            completed_at: "2026-03-04T12:05:18.000Z".to_string(),
            status: Some("completed".to_string()),
            files: vec![
                TurnDiffFileChange {
                    path: "crates/r3_ui/src/shell.rs".to_string(),
                    kind: Some("modified".to_string()),
                    additions: Some(126),
                    deletions: Some(18),
                },
                TurnDiffFileChange {
                    path: "crates/r3_core/src/lib.rs".to_string(),
                    kind: Some("modified".to_string()),
                    additions: Some(74),
                    deletions: Some(4),
                },
                TurnDiffFileChange {
                    path: "docs/reference/PARITY_PLAN.md".to_string(),
                    kind: Some("modified".to_string()),
                    additions: Some(8),
                    deletions: Some(0),
                },
            ],
            checkpoint_ref: Some("checkpoint-turn-2".to_string()),
            assistant_message_id: Some("msg-assistant-r3code-ui-shell".to_string()),
            checkpoint_turn_count: Some(2),
        },
        TurnDiffSummary {
            turn_id: "turn-r3code-ui-shell-1".to_string(),
            completed_at: "2026-03-04T12:01:42.000Z".to_string(),
            status: Some("completed".to_string()),
            files: vec![
                TurnDiffFileChange {
                    path: "crates/r3_ui/assets/icons/diff.svg".to_string(),
                    kind: Some("added".to_string()),
                    additions: Some(1),
                    deletions: Some(0),
                },
                TurnDiffFileChange {
                    path: "crates/r3_ui/src/assets.rs".to_string(),
                    kind: Some("modified".to_string()),
                    additions: Some(6),
                    deletions: Some(1),
                },
            ],
            checkpoint_ref: Some("checkpoint-turn-1".to_string()),
            assistant_message_id: Some("msg-assistant-r3code-ui-shell".to_string()),
            checkpoint_turn_count: Some(1),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_command_palette_project() -> ProjectSummary {
        ProjectSummary {
            id: "project-1".to_string(),
            environment_id: "environment-local".to_string(),
            name: "Project".to_string(),
            path: "/repo/project".to_string(),
            scripts: Vec::new(),
        }
    }

    fn make_command_palette_thread(
        id: &str,
        title: &str,
        created_at: &str,
        updated_at: &str,
    ) -> ThreadSummary {
        ThreadSummary {
            id: id.to_string(),
            environment_id: "environment-local".to_string(),
            project_id: "project-1".to_string(),
            title: title.to_string(),
            project_name: "Project".to_string(),
            status: ThreadStatus::Idle,
            created_at: created_at.to_string(),
            updated_at: updated_at.to_string(),
            archived_at: None,
            latest_user_message_at: None,
            has_pending_approvals: false,
            has_pending_user_input: false,
            has_actionable_proposed_plan: false,
            branch: None,
            worktree_path: None,
        }
    }

    fn make_server_provider(overrides: impl FnOnce(&mut ServerProvider)) -> ServerProvider {
        let mut provider = ServerProvider {
            instance_id: "codex".to_string(),
            driver: "codex".to_string(),
            display_name: Some("Codex".to_string()),
            accent_color: None,
            badge_label: None,
            continuation_group_key: None,
            show_interaction_mode_toggle: true,
            enabled: true,
            installed: true,
            version: Some("1.2.3".to_string()),
            status: ServerProviderState::Ready,
            auth: ServerProviderAuth {
                status: ServerProviderAuthStatus::Unknown,
                kind: None,
                label: None,
                email: None,
            },
            checked_at: "2026-03-04T12:00:00.000Z".to_string(),
            message: None,
            availability: ServerProviderAvailability::Available,
            unavailable_reason: None,
            models: Vec::new(),
            version_advisory: None,
        };
        overrides(&mut provider);
        provider
    }

    fn advertised_endpoint(
        id: &str,
        http_base_url: &str,
        reachability: &str,
        is_default: bool,
        status: AdvertisedEndpointStatus,
        hosted_https_app: HostedHttpsAppCompatibility,
    ) -> AdvertisedEndpoint {
        AdvertisedEndpoint {
            id: id.to_string(),
            provider_id: "desktop-core".to_string(),
            label: "Local network".to_string(),
            http_base_url: http_base_url.to_string(),
            reachability: reachability.to_string(),
            status,
            is_default,
            hosted_https_app,
        }
    }

    #[test]
    fn parses_manual_desktop_ssh_targets_like_upstream() {
        let target = parse_manual_desktop_ssh_target("alice@example.com:2222", "", "").unwrap();
        assert_eq!(
            target,
            DesktopSshEnvironmentTarget {
                alias: "example.com".to_string(),
                hostname: "example.com".to_string(),
                username: Some("alice".to_string()),
                port: Some(2222),
            }
        );
        assert_eq!(format_desktop_ssh_target(&target), "alice@example.com:2222");

        let explicit_username =
            parse_manual_desktop_ssh_target("alice@example.com", "root", "").unwrap();
        assert_eq!(explicit_username.username.as_deref(), Some("root"));

        let ipv6 = parse_manual_desktop_ssh_target("bob@[fe80::1]:2200", "", "").unwrap();
        assert_eq!(ipv6.hostname, "fe80::1");
        assert_eq!(ipv6.username.as_deref(), Some("bob"));
        assert_eq!(ipv6.port, Some(2200));
        assert_eq!(format_desktop_ssh_target(&ipv6), "bob@fe80::1:2200");

        let explicit_port_parse_int =
            parse_manual_desktop_ssh_target("devbox", "", "22abc").unwrap();
        assert_eq!(explicit_port_parse_int.port, Some(22));
    }

    #[test]
    fn rejects_manual_desktop_ssh_targets_with_upstream_messages() {
        assert_eq!(
            parse_manual_desktop_ssh_target("  ", "", "").unwrap_err(),
            "SSH host or alias is required."
        );
        assert_eq!(
            parse_manual_desktop_ssh_target("alice@:22", "", "").unwrap_err(),
            "SSH host or alias is required."
        );
        assert_eq!(
            parse_manual_desktop_ssh_target("example.com:70000", "", "").unwrap_err(),
            "SSH port must be between 1 and 65535."
        );
        assert_eq!(
            parse_manual_desktop_ssh_target("example.com", "", "nope").unwrap_err(),
            "SSH port must be between 1 and 65535."
        );
    }

    #[test]
    fn parses_remote_pairing_fields_from_urls_and_manual_fields() {
        assert_eq!(
            parse_remote_pairing_fields("https://remote.example.com/pair#token=pairing-token", "")
                .unwrap(),
            RemotePairingFields {
                host: "https://remote.example.com".to_string(),
                pairing_code: "pairing-token".to_string(),
            }
        );
        assert_eq!(
            parse_remote_pairing_fields(
                "https://app.t3.codes/pair?host=https%3A%2F%2Fdesktop.tailnet.ts.net%3A44342%2F#token=pairing-token",
                "",
            )
            .unwrap(),
            RemotePairingFields {
                host: "https://desktop.tailnet.ts.net:44342/".to_string(),
                pairing_code: "pairing-token".to_string(),
            }
        );
        assert_eq!(
            parse_remote_pairing_fields("backend.example.com", "PAIRCODE").unwrap(),
            RemotePairingFields {
                host: "backend.example.com".to_string(),
                pairing_code: "PAIRCODE".to_string(),
            }
        );

        assert_eq!(
            parse_remote_pairing_fields("", "PAIRCODE").unwrap_err(),
            "Enter a backend host."
        );
        assert_eq!(
            parse_remote_pairing_fields("backend.example.com", "").unwrap_err(),
            "Enter a pairing code."
        );
    }

    #[test]
    fn formats_desktop_ssh_connection_errors_like_upstream() {
        assert_eq!(
            format_desktop_ssh_connection_error(Some(
                "Error invoking remote method 'desktop:ensure-ssh-environment': SshConnectionError: bad host"
            )),
            "bad host"
        );
        assert_eq!(
            format_desktop_ssh_connection_error(Some("SshLaunchError: timed out")),
            "timed out"
        );
        assert_eq!(
            format_desktop_ssh_connection_error(Some("   ")),
            "Failed to connect SSH host."
        );
        assert_eq!(
            format_desktop_ssh_connection_error(None),
            "Failed to connect SSH host."
        );
    }

    #[test]
    fn selects_and_resolves_advertised_pairing_endpoints() {
        let loopback = advertised_endpoint(
            "desktop-loopback:127.0.0.1",
            "http://127.0.0.1:8765",
            "loopback",
            true,
            AdvertisedEndpointStatus::Available,
            HostedHttpsAppCompatibility::Incompatible,
        );
        let lan = advertised_endpoint(
            "desktop-lan:192.168.1.44",
            "http://192.168.1.44:8765",
            "lan",
            false,
            AdvertisedEndpointStatus::Available,
            HostedHttpsAppCompatibility::Incompatible,
        );
        let tailscale_https = advertised_endpoint(
            "tailscale-magicdns:desktop.tailnet.ts.net",
            "https://desktop.tailnet.ts.net:8765",
            "tailscale",
            false,
            AdvertisedEndpointStatus::Available,
            HostedHttpsAppCompatibility::Compatible,
        );
        let unavailable_preference = advertised_endpoint(
            "desktop-lan:stale",
            "http://stale.local:8765",
            "lan",
            false,
            AdvertisedEndpointStatus::Unavailable,
            HostedHttpsAppCompatibility::Incompatible,
        );

        assert_eq!(
            endpoint_default_preference_key(&loopback),
            "desktop-core:loopback:http"
        );
        assert_eq!(
            endpoint_default_preference_key(&tailscale_https),
            "tailscale:magicdns:https"
        );

        let endpoints = vec![
            unavailable_preference,
            loopback.clone(),
            lan.clone(),
            tailscale_https.clone(),
        ];
        assert_eq!(
            select_pairing_endpoint(&endpoints, Some("desktop-core:lan:http"))
                .unwrap()
                .id,
            lan.id
        );
        assert_eq!(
            resolve_advertised_endpoint_pairing_url(&lan, "PAIRCODE").unwrap(),
            "http://192.168.1.44:8765/pair#token=PAIRCODE"
        );
        assert_eq!(
            resolve_advertised_endpoint_pairing_url(&tailscale_https, "PAIRCODE").unwrap(),
            "https://app.t3.codes/pair?host=https%3A%2F%2Fdesktop.tailnet.ts.net%3A8765#token=PAIRCODE"
        );
    }

    #[test]
    fn sorts_and_upserts_access_records_like_upstream() {
        let old_link = ServerPairingLinkRecord {
            id: "old".to_string(),
            created_at: "2026-03-01T00:00:00.000Z".to_string(),
        };
        let new_link = ServerPairingLinkRecord {
            id: "new".to_string(),
            created_at: "2026-03-02T00:00:00.000Z".to_string(),
        };
        assert_eq!(
            sort_desktop_pairing_links(&[old_link.clone(), new_link.clone()])
                .into_iter()
                .map(|link| link.id)
                .collect::<Vec<_>>(),
            vec!["new", "old"]
        );
        assert_eq!(
            upsert_desktop_pairing_link(&[old_link], new_link)
                .into_iter()
                .map(|link| link.id)
                .collect::<Vec<_>>(),
            vec!["new", "old"]
        );

        let disconnected_current = ServerClientSessionRecord {
            session_id: "current".to_string(),
            issued_at: "2026-03-01T00:00:00.000Z".to_string(),
            current: true,
            connected: false,
        };
        let connected_other = ServerClientSessionRecord {
            session_id: "other".to_string(),
            issued_at: "2026-03-03T00:00:00.000Z".to_string(),
            current: false,
            connected: true,
        };
        assert_eq!(
            sort_desktop_client_sessions(&[connected_other, disconnected_current])
                .into_iter()
                .map(|session| session.session_id)
                .collect::<Vec<_>>(),
            vec!["current", "other"]
        );
    }

    #[test]
    fn formats_diagnostics_helpers_like_upstream() {
        assert_eq!(format_diagnostics_count(1234567), "1,234,567");
        assert_eq!(format_diagnostics_duration_ms(999.4), "999 ms");
        assert_eq!(format_diagnostics_duration_ms(1500.0), "1.50 s");
        assert_eq!(format_diagnostics_duration_ms(10_000.0), "10.0 s");
        assert_eq!(format_diagnostics_bytes(1023), "1023 B");
        assert_eq!(format_diagnostics_bytes(1536), "1.50 KB");
        assert_eq!(format_diagnostics_bytes(12 * 1024), "12.0 KB");
        assert_eq!(
            shorten_trace_id("0123456789abcdef0123456789abcdef0123456789"),
            "0123456789abcdef01...0123456789"
        );
        assert!(is_stale_process_signal_message(Some(
            "process is not a live descendant"
        )));
        assert!(!is_stale_process_signal_message(None));
    }

    #[test]
    fn formats_diagnostics_settings_description_like_upstream() {
        assert_eq!(
            collapse_otel_signals_url(
                "http://localhost:4318/v1/traces",
                "http://localhost:4318/v1/metrics",
            )
            .as_deref(),
            Some("http://localhost:4318/v1/{traces,metrics}")
        );
        assert_eq!(
            format_diagnostics_description(DiagnosticsDescriptionInput {
                local_tracing_enabled: true,
                otlp_traces_enabled: true,
                otlp_traces_url: Some("http://localhost:4318/v1/traces"),
                otlp_metrics_enabled: true,
                otlp_metrics_url: Some("http://localhost:4318/v1/metrics"),
            }),
            "Local trace file. Exporting OTEL to http://localhost:4318/v1/{traces,metrics}."
        );
        assert_eq!(
            format_diagnostics_description(DiagnosticsDescriptionInput {
                local_tracing_enabled: false,
                otlp_traces_enabled: false,
                otlp_traces_url: Some("http://localhost:4318/v1/traces"),
                otlp_metrics_enabled: false,
                otlp_metrics_url: None,
            }),
            "Terminal logs only."
        );
    }

    #[test]
    fn resolves_server_route_before_draft_route() {
        let target = resolve_thread_route_target(Some("env-1"), Some("thread-1"), Some("draft-1"));

        assert_eq!(
            target,
            Some(ThreadRouteTarget::Server {
                thread_ref: ScopedThreadRef::new("env-1", "thread-1")
            })
        );
    }

    #[test]
    fn resolves_draft_route_when_no_server_thread_params_exist() {
        let target = resolve_thread_route_target(None, None, Some("draft-1"));

        assert_eq!(
            target,
            Some(ThreadRouteTarget::Draft {
                draft_id: "draft-1".to_string()
            })
        );
    }

    #[test]
    fn index_route_does_not_render_chat_view() {
        assert!(!AppSnapshot::empty_reference_state().renders_chat_view());
    }

    #[test]
    fn draft_reference_state_renders_chat_view_with_draft_session() {
        let snapshot = AppSnapshot::draft_reference_state();

        assert!(snapshot.renders_chat_view());
        assert_eq!(snapshot.draft_sessions.len(), 1);
        assert_eq!(snapshot.messages, Vec::new());
        assert_eq!(snapshot.active_thread_title(), "New thread");
        assert_eq!(snapshot.active_project_name(), Some("server"));
    }

    #[test]
    fn mock_reference_state_exposes_active_thread_header_data() {
        let snapshot = AppSnapshot::mock_reference_state();

        assert_eq!(snapshot.active_thread_title(), "Port R3Code UI shell");
        assert_eq!(snapshot.active_project_name(), Some("r3code"));
        assert!(snapshot.turn_diff_summaries.is_empty());
    }

    #[test]
    fn command_palette_builds_recent_threads_with_upstream_sort_and_timestamp_rules() {
        let projects = vec![make_command_palette_project()];
        let threads = vec![
            make_command_palette_thread(
                "thread-older",
                "Older thread",
                "2026-03-23T12:00:00.000Z",
                "2026-03-24T12:00:00.000Z",
            ),
            make_command_palette_thread(
                "thread-newer",
                "Newer thread",
                "2026-03-20T00:00:00.000Z",
                "2026-03-20T00:00:00.000Z",
            ),
        ];

        let items = build_thread_action_items(
            &threads,
            None,
            &projects,
            SidebarThreadSortOrder::UpdatedAt,
            "2026-03-25T12:00:00.000Z",
            None,
        );

        assert_eq!(
            items
                .iter()
                .map(|item| item.value.as_str())
                .collect::<Vec<_>>(),
            vec!["thread:thread-older", "thread:thread-newer"]
        );
        assert_eq!(items[0].timestamp.as_deref(), Some("1d ago"));
        assert_eq!(items[1].timestamp.as_deref(), Some("5d ago"));
    }

    #[test]
    fn command_palette_search_ranks_titles_over_context_and_filters_archived_threads() {
        let projects = vec![make_command_palette_project()];
        let mut context_match = make_command_palette_thread(
            "thread-context-match",
            "Fix navbar spacing",
            "2026-03-02T00:00:00.000Z",
            "2026-03-20T00:00:00.000Z",
        );
        context_match.project_name = "Project".to_string();
        let title_match = make_command_palette_thread(
            "thread-title-match",
            "Project kickoff notes",
            "2026-03-02T00:00:00.000Z",
            "2026-03-19T00:00:00.000Z",
        );
        let mut archived_match = make_command_palette_thread(
            "thread-archived",
            "Archived project thread",
            "2026-03-02T00:00:00.000Z",
            "2026-03-21T00:00:00.000Z",
        );
        archived_match.archived_at = Some("2026-03-22T00:00:00.000Z".to_string());
        let thread_items = build_thread_action_items(
            &[context_match, title_match, archived_match],
            None,
            &projects,
            SidebarThreadSortOrder::UpdatedAt,
            "2026-03-25T12:00:00.000Z",
            None,
        );

        let groups = filter_command_palette_groups(&[], "project", false, &[], &thread_items);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].value, "threads-search");
        assert_eq!(
            groups[0]
                .items
                .iter()
                .map(|item| item.value.as_str())
                .collect::<Vec<_>>(),
            vec!["thread:thread-title-match", "thread:thread-context-match"]
        );
    }

    #[test]
    fn command_palette_filters_action_only_queries_and_injects_projects_and_threads() {
        let action_items = vec![
            CommandPaletteItem::action(
                "action:add-project",
                vec!["add project".to_string(), "folder".to_string()],
                "Add project",
            ),
            CommandPaletteItem::action(
                "action:settings",
                vec!["settings".to_string(), "preferences".to_string()],
                "Open settings",
            ),
        ];
        let root_groups = build_root_command_palette_groups(action_items, Vec::new());
        let projects = vec![make_command_palette_project()];
        let project_items = build_project_action_items(&projects, "project");
        let thread_items = build_thread_action_items(
            &[make_command_palette_thread(
                "thread-1",
                "Project kickoff notes",
                "2026-03-02T00:00:00.000Z",
                "2026-03-19T00:00:00.000Z",
            )],
            None,
            &projects,
            SidebarThreadSortOrder::UpdatedAt,
            "2026-03-25T12:00:00.000Z",
            None,
        );

        let action_groups = filter_command_palette_groups(
            &root_groups,
            ">settings",
            false,
            &project_items,
            &thread_items,
        );
        assert_eq!(action_groups.len(), 1);
        assert_eq!(action_groups[0].value, "actions");
        assert_eq!(action_groups[0].items[0].value, "action:settings");

        let search_groups = filter_command_palette_groups(
            &root_groups,
            "project",
            false,
            &project_items,
            &thread_items,
        );
        assert_eq!(
            search_groups
                .iter()
                .map(|group| group.value.as_str())
                .collect::<Vec<_>>(),
            vec!["actions", "projects-search", "threads-search"]
        );
    }

    #[test]
    fn provider_status_summary_matches_upstream_precedence() {
        let missing = get_provider_summary(None);
        assert_eq!(missing.headline, "Checking provider status");
        assert_eq!(
            missing.detail.as_deref(),
            Some("Waiting for the server to report installation and authentication details.")
        );

        let disabled = make_server_provider(|provider| {
            provider.enabled = false;
            provider.message = None;
        });
        assert_eq!(get_provider_summary(Some(&disabled)).headline, "Disabled");

        let not_found = make_server_provider(|provider| {
            provider.installed = false;
            provider.message = Some("Binary missing.".to_string());
        });
        assert_eq!(get_provider_summary(Some(&not_found)).headline, "Not found");
        assert_eq!(
            get_provider_summary(Some(&not_found)).detail.as_deref(),
            Some("Binary missing.")
        );

        let authenticated = make_server_provider(|provider| {
            provider.auth = ServerProviderAuth {
                status: ServerProviderAuthStatus::Authenticated,
                kind: Some("oauth".to_string()),
                label: Some("Codex Pro".to_string()),
                email: None,
            };
        });
        assert_eq!(
            get_provider_summary(Some(&authenticated)).headline,
            "Authenticated · Codex Pro"
        );

        let unauthenticated = make_server_provider(|provider| {
            provider.auth.status = ServerProviderAuthStatus::Unauthenticated;
        });
        assert_eq!(
            get_provider_summary(Some(&unauthenticated)).headline,
            "Not authenticated"
        );

        let warning = make_server_provider(|provider| {
            provider.status = ServerProviderState::Warning;
            provider.auth.status = ServerProviderAuthStatus::Unknown;
        });
        assert_eq!(
            get_provider_summary(Some(&warning)).headline,
            "Needs attention"
        );

        let error = make_server_provider(|provider| {
            provider.status = ServerProviderState::Error;
            provider.auth.status = ServerProviderAuthStatus::Unknown;
        });
        assert_eq!(get_provider_summary(Some(&error)).headline, "Unavailable");
    }

    #[test]
    fn provider_version_labels_and_advisories_match_upstream_logic() {
        assert_eq!(get_provider_version_label(None), None);
        assert_eq!(
            get_provider_version_label(Some("1.2.3")),
            Some("v1.2.3".to_string())
        );
        assert_eq!(
            get_provider_version_label(Some("v1.2.3")),
            Some("v1.2.3".to_string())
        );

        let current = ServerProviderVersionAdvisory {
            status: ServerProviderVersionAdvisoryStatus::Current,
            current_version: Some("1.2.3".to_string()),
            latest_version: Some("1.2.3".to_string()),
            update_command: None,
            can_update: false,
            checked_at: None,
            message: None,
        };
        assert_eq!(
            get_provider_version_advisory_presentation(Some(&current)),
            None
        );

        let behind = ServerProviderVersionAdvisory {
            status: ServerProviderVersionAdvisoryStatus::BehindLatest,
            current_version: Some("1.2.3".to_string()),
            latest_version: Some("1.2.4".to_string()),
            update_command: Some("npm install -g provider@latest".to_string()),
            can_update: true,
            checked_at: Some("2026-03-04T12:00:00.000Z".to_string()),
            message: None,
        };
        let presentation = get_provider_version_advisory_presentation(Some(&behind)).unwrap();
        assert_eq!(presentation.detail, "Update available: install v1.2.4.");
        assert_eq!(
            presentation.update_command.as_deref(),
            Some("npm install -g provider@latest")
        );

        let custom_message = ServerProviderVersionAdvisory {
            message: Some("Use your package manager to update.".to_string()),
            ..behind
        };
        assert_eq!(
            get_provider_version_advisory_presentation(Some(&custom_message))
                .unwrap()
                .detail,
            "Use your package manager to update."
        );
    }

    #[test]
    fn active_chat_reference_state_links_diff_summary_to_assistant_message() {
        let snapshot = AppSnapshot::active_chat_reference_state();
        let assistant_message = snapshot
            .messages
            .iter()
            .find(|message| message.role == MessageRole::Assistant)
            .unwrap();

        assert_eq!(snapshot.turn_diff_summaries.len(), 2);
        assert!(
            snapshot
                .turn_diff_summaries
                .iter()
                .any(|summary| summary.assistant_message_id.as_deref()
                    == Some(assistant_message.id.as_str()))
        );
    }

    #[test]
    fn branch_toolbar_labels_match_upstream_logic() {
        assert_eq!(
            resolve_env_mode_label(DraftThreadEnvMode::Local),
            "Current checkout"
        );
        assert_eq!(
            resolve_env_mode_label(DraftThreadEnvMode::Worktree),
            "New worktree"
        );
        assert_eq!(resolve_current_workspace_label(None), "Current checkout");
        assert_eq!(
            resolve_current_workspace_label(Some("/repo/.t3/worktrees/feature-a")),
            "Current worktree"
        );
        assert_eq!(resolve_locked_workspace_label(None), "Local checkout");
        assert_eq!(
            resolve_locked_workspace_label(Some("/repo/.t3/worktrees/feature-a")),
            "Worktree"
        );
        assert_eq!(
            resolve_environment_option_label(
                true,
                "environment-local",
                Some("Local environment"),
                Some("Local")
            ),
            "This device"
        );
        assert_eq!(
            resolve_environment_option_label(false, "environment-remote", None, Some("Build box")),
            "Build box"
        );
    }

    #[test]
    fn branch_toolbar_env_mode_and_value_match_upstream_logic() {
        assert_eq!(
            resolve_effective_env_mode(
                Some("/repo/.t3/worktrees/feature-a"),
                false,
                Some(DraftThreadEnvMode::Worktree)
            ),
            DraftThreadEnvMode::Local
        );
        assert_eq!(
            resolve_effective_env_mode(None, false, Some(DraftThreadEnvMode::Worktree)),
            DraftThreadEnvMode::Worktree
        );
        assert_eq!(
            resolve_draft_env_mode_after_branch_change(
                None,
                Some("/repo/.t3/worktrees/feature-a"),
                DraftThreadEnvMode::Worktree
            ),
            DraftThreadEnvMode::Local
        );
        assert_eq!(
            resolve_draft_env_mode_after_branch_change(None, None, DraftThreadEnvMode::Worktree),
            DraftThreadEnvMode::Worktree
        );
        assert_eq!(
            resolve_branch_toolbar_value(DraftThreadEnvMode::Worktree, None, None, Some("main")),
            Some("main".to_string())
        );
        assert_eq!(
            resolve_branch_toolbar_value(
                DraftThreadEnvMode::Worktree,
                None,
                Some("feature/base"),
                Some("main")
            ),
            Some("feature/base".to_string())
        );
        assert_eq!(
            resolve_branch_toolbar_value(
                DraftThreadEnvMode::Local,
                None,
                Some("feature/base"),
                Some("main")
            ),
            Some("main".to_string())
        );
        assert_eq!(
            branch_toolbar_trigger_label(None, DraftThreadEnvMode::Worktree, Some("main")),
            "From main"
        );
    }

    #[test]
    fn branch_selection_target_matches_upstream_worktree_rules() {
        assert_eq!(
            resolve_branch_selection_target(
                "/repo",
                Some("/repo/.t3/worktrees/feature-a"),
                &vcs_ref("feature-b", false, Some("/repo/.t3/worktrees/feature-b"))
            ),
            BranchSelectionTarget {
                checkout_cwd: "/repo/.t3/worktrees/feature-b".to_string(),
                next_worktree_path: Some("/repo/.t3/worktrees/feature-b".to_string()),
                reuse_existing_worktree: true,
            }
        );
        assert_eq!(
            resolve_branch_selection_target(
                "/repo",
                Some("/repo/.t3/worktrees/feature-a"),
                &vcs_ref("main", true, Some("/repo"))
            ),
            BranchSelectionTarget {
                checkout_cwd: "/repo".to_string(),
                next_worktree_path: None,
                reuse_existing_worktree: true,
            }
        );
        assert_eq!(
            resolve_branch_selection_target(
                "/repo",
                Some("/repo/.t3/worktrees/feature-a"),
                &vcs_ref("main", true, None)
            ),
            BranchSelectionTarget {
                checkout_cwd: "/repo".to_string(),
                next_worktree_path: None,
                reuse_existing_worktree: false,
            }
        );
        assert_eq!(
            resolve_branch_selection_target(
                "/repo",
                Some("/repo/.t3/worktrees/feature-a"),
                &vcs_ref("feature-a", false, None)
            ),
            BranchSelectionTarget {
                checkout_cwd: "/repo/.t3/worktrees/feature-a".to_string(),
                next_worktree_path: Some("/repo/.t3/worktrees/feature-a".to_string()),
                reuse_existing_worktree: false,
            }
        );
    }

    #[test]
    fn branch_picker_helpers_match_upstream_filtering() {
        assert_eq!(
            derive_local_branch_name_from_remote_ref("origin/feature/demo"),
            "feature/demo"
        );
        assert_eq!(
            derive_local_branch_name_from_remote_ref("my-org/upstream/feature/demo"),
            "upstream/feature/demo"
        );
        assert_eq!(
            derive_local_branch_name_from_remote_ref("origin/"),
            "origin/"
        );
        assert_eq!(
            dedupe_remote_branches_with_local_matches(&[
                vcs_ref("feature/demo", false, None),
                remote_vcs_ref("origin/feature/demo", "origin"),
                remote_vcs_ref("origin/feature/remote-only", "origin"),
            ])
            .iter()
            .map(|ref_name| ref_name.name.as_str())
            .collect::<Vec<_>>(),
            vec!["feature/demo", "origin/feature/remote-only"]
        );
        assert!(should_include_branch_picker_item(
            "__checkout_pull_request__:1359",
            "gh pr checkout 1359",
            Some("__create_new_branch__:gh pr checkout 1359"),
            Some("__checkout_pull_request__:1359")
        ));
        assert!(should_include_branch_picker_item(
            "__create_new_branch__:feature/demo",
            "feature/demo",
            Some("__create_new_branch__:feature/demo"),
            None
        ));
        assert!(!should_include_branch_picker_item(
            "main",
            "gh pr checkout 1359",
            Some("__create_new_branch__:gh pr checkout 1359"),
            Some("__checkout_pull_request__:1359")
        ));
    }

    #[test]
    fn branch_toolbar_reference_state_exposes_new_worktree_context() {
        let snapshot = AppSnapshot::branch_toolbar_reference_state();
        let toolbar = snapshot.active_branch_toolbar_state().unwrap();

        assert_eq!(toolbar.effective_env_mode, DraftThreadEnvMode::Worktree);
        assert_eq!(toolbar.workspace_label, "New worktree");
        assert_eq!(toolbar.branch_label, "From main");
        assert!(toolbar.show_environment_picker);
    }

    #[test]
    fn project_scripts_helpers_match_upstream_logic() {
        let command = command_for_project_script("lint");
        assert_eq!(command, "script.lint.run");
        assert_eq!(
            project_script_id_from_command(&command),
            Some("lint".to_string())
        );
        assert_eq!(project_script_id_from_command("terminal.toggle"), None);
        assert_eq!(
            next_project_script_id("Run Tests", [] as [&str; 0]),
            "run-tests"
        );
        assert_eq!(
            next_project_script_id("Run Tests", ["run-tests"]),
            "run-tests-2"
        );
        assert_eq!(next_project_script_id("!!!", [] as [&str; 0]), "script");

        let scripts = vec![
            ProjectScript {
                id: "setup".to_string(),
                name: "Setup".to_string(),
                command: "bun install".to_string(),
                icon: ProjectScriptIcon::Configure,
                run_on_worktree_create: true,
            },
            ProjectScript {
                id: "test".to_string(),
                name: "Test".to_string(),
                command: "bun test".to_string(),
                icon: ProjectScriptIcon::Test,
                run_on_worktree_create: false,
            },
        ];

        assert_eq!(primary_project_script(&scripts).unwrap().id, "test");
        assert_eq!(setup_project_script(&scripts).unwrap().id, "setup");
    }

    #[test]
    fn project_script_runtime_context_matches_upstream_logic() {
        let env = project_script_runtime_env("/repo", Some("/repo/worktree-a"), &[]);

        assert_eq!(
            env.get("T3CODE_PROJECT_ROOT").map(String::as_str),
            Some("/repo")
        );
        assert_eq!(
            env.get("T3CODE_WORKTREE_PATH").map(String::as_str),
            Some("/repo/worktree-a")
        );
        assert_eq!(
            project_script_cwd("/repo", Some("/repo/worktree-a")),
            "/repo/worktree-a"
        );
        assert_eq!(project_script_cwd("/repo", None), "/repo");

        let env = project_script_runtime_env(
            "/repo",
            None,
            &[
                ("T3CODE_PROJECT_ROOT", "/custom-root"),
                ("CUSTOM_FLAG", "1"),
            ],
        );
        assert_eq!(
            env.get("T3CODE_PROJECT_ROOT").map(String::as_str),
            Some("/custom-root")
        );
        assert_eq!(env.get("CUSTOM_FLAG").map(String::as_str), Some("1"));
        assert!(!env.contains_key("T3CODE_WORKTREE_PATH"));
    }

    #[test]
    fn open_in_picker_visibility_and_options_match_upstream_logic() {
        assert!(should_show_open_in_picker(
            Some("codething-mvp"),
            "environment-primary",
            Some("environment-primary")
        ));
        assert!(!should_show_open_in_picker(
            Some("codething-mvp"),
            "environment-remote",
            None
        ));
        assert!(!should_show_open_in_picker(
            Some("codething-mvp"),
            "environment-remote",
            Some("environment-primary")
        ));
        assert!(!should_show_open_in_picker(
            None,
            "environment-primary",
            Some("environment-primary")
        ));

        let options = resolve_editor_options(
            "Windows",
            &[
                EditorId::VsCodeInsiders,
                EditorId::VsCodium,
                EditorId::FileManager,
            ],
        );
        assert_eq!(
            options
                .iter()
                .map(|option| option.label)
                .collect::<Vec<_>>(),
            vec!["VS Code Insiders", "VSCodium", "Explorer"]
        );
    }

    #[test]
    fn provider_instance_projection_matches_upstream_logic() {
        let snapshot = AppSnapshot::mock_reference_state();
        let entries = derive_provider_instance_entries(&snapshot.providers);
        let codex = entries
            .iter()
            .find(|entry| entry.instance_id == "codex")
            .unwrap();
        let personal = entries
            .iter()
            .find(|entry| entry.instance_id == "codex_personal")
            .unwrap();
        let cursor = entries
            .iter()
            .find(|entry| entry.instance_id == "cursor")
            .unwrap();

        assert_eq!(codex.display_name, "Codex");
        assert!(codex.is_default);
        assert_eq!(personal.display_name, "Codex Personal");
        assert_eq!(personal.accent_color.as_deref(), Some("#2563EB"));
        assert!(!personal.is_default);
        assert!(!cursor.is_available);
        assert_eq!(provider_instance_initials("Codex Personal"), "CP");
        assert_eq!(normalize_provider_accent_color(Some("not-a-color")), None);

        let sorted = sort_provider_instance_entries(&entries);
        let codex_index = sorted
            .iter()
            .position(|entry| entry.instance_id == "codex")
            .unwrap();
        let personal_index = sorted
            .iter()
            .position(|entry| entry.instance_id == "codex_personal")
            .unwrap();
        assert!(codex_index < personal_index);
    }

    #[test]
    fn model_picker_trigger_filtering_and_locking_match_upstream_logic() {
        let snapshot = AppSnapshot::mock_reference_state();
        let state = resolve_model_picker_state(&snapshot, "", None, None, None);

        assert_eq!(state.trigger_title, "5.4 Mini");
        assert_eq!(state.trigger_label, "5.4 Mini");
        assert!(state.show_instance_badge);
        assert_eq!(
            state.selected_instance,
            ModelPickerSelectedInstance::Favorites
        );
        assert!(state.show_sidebar);
        assert_eq!(
            state
                .filtered_models
                .iter()
                .map(|model| provider_model_key(&model.instance_id, &model.slug))
                .collect::<Vec<_>>(),
            vec!["codex:gpt-5.4", "claudeAgent:claude-sonnet-4-6"]
        );

        let search = resolve_model_picker_state(&snapshot, "sonnet", None, None, None);
        assert!(!search.show_sidebar);
        assert_eq!(search.filtered_models[0].slug, "claude-sonnet-4-6");

        let locked = resolve_model_picker_state(
            &snapshot,
            "",
            Some(ModelPickerSelectedInstance::Instance("codex".to_string())),
            Some("codex"),
            Some("codex-default"),
        );
        assert!(locked.is_locked);
        assert!(!locked.show_locked_instance_sidebar);
        assert_eq!(locked.locked_header_label.as_deref(), Some("Codex"));
        assert!(
            locked
                .filtered_models
                .iter()
                .all(|model| model.instance_id == "codex")
        );
    }

    #[test]
    fn model_picker_search_sorting_and_selection_match_upstream_logic() {
        let snapshot = AppSnapshot::mock_reference_state();
        let (_, slug) = split_instance_model_key("codex:openai/custom:model");
        assert_eq!(slug, "openai/custom:model");

        let codex_models = &snapshot.providers[0].models;
        assert_eq!(
            resolve_selectable_model("codex", Some("5.4"), codex_models),
            Some("gpt-5.4".to_string())
        );
        assert_eq!(
            resolve_selectable_model("codex", Some("GPT-5.3 Codex"), codex_models),
            Some("gpt-5.3-codex".to_string())
        );
        assert_eq!(
            resolve_selectable_provider_instance(&snapshot.providers, Some("missing")),
            Some("codex".to_string())
        );

        let favorites = favorite_model_key_set(&snapshot.model_favorites);
        let state = resolve_model_picker_state(
            &snapshot,
            "",
            Some(ModelPickerSelectedInstance::Instance("codex".to_string())),
            None,
            None,
        );
        let sorted = sort_provider_model_items(&state.filtered_models, &favorites, true, &[]);
        assert_eq!(sorted[0].slug, "gpt-5.4");
        assert!(score_model_picker_search(&sorted[0], "5.4").unwrap() < 10);
        assert_eq!(
            build_model_picker_search_text(&sorted[0]),
            "gpt-5.4 5.4 codex codex"
        );
    }

    #[test]
    fn running_turn_reference_state_exposes_work_log_entries() {
        let snapshot = AppSnapshot::running_turn_reference_state();
        let entries = snapshot.work_log_entries();

        assert_eq!(snapshot.threads[0].status, ThreadStatus::Running);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].tone, ActivityTone::Thinking);
        assert_eq!(
            entries[1].command.as_deref(),
            Some("cargo test --workspace")
        );
        assert_eq!(
            entries[2].changed_files,
            vec!["crates/r3_core/src/lib.rs", "crates/r3_ui/src/shell.rs"]
        );
    }

    #[test]
    fn derives_work_log_entries_with_upstream_filters_and_collapse() {
        let activities = vec![
            ThreadActivity {
                id: "started".to_string(),
                kind: "tool.started".to_string(),
                summary: "Started command".to_string(),
                tone: ActivityTone::Tool,
                payload: ActivityPayload::default(),
                turn_id: Some("turn-1".to_string()),
                sequence: Some(1),
                created_at: "2026-03-04T12:00:01.000Z".to_string(),
            },
            ThreadActivity {
                id: "updated".to_string(),
                kind: "tool.updated".to_string(),
                summary: "Ran command".to_string(),
                tone: ActivityTone::Tool,
                payload: ActivityPayload {
                    command: Some("cargo check".to_string()),
                    title: Some("terminal".to_string()),
                    item_type: Some("command_execution".to_string()),
                    tool_call_id: Some("tool-1".to_string()),
                    ..ActivityPayload::default()
                },
                turn_id: Some("turn-1".to_string()),
                sequence: Some(2),
                created_at: "2026-03-04T12:00:02.000Z".to_string(),
            },
            ThreadActivity {
                id: "completed".to_string(),
                kind: "tool.completed".to_string(),
                summary: "Ran command completed".to_string(),
                tone: ActivityTone::Tool,
                payload: ActivityPayload {
                    detail: Some("Finished in 1s".to_string()),
                    title: Some("terminal".to_string()),
                    item_type: Some("command_execution".to_string()),
                    tool_call_id: Some("tool-1".to_string()),
                    ..ActivityPayload::default()
                },
                turn_id: Some("turn-1".to_string()),
                sequence: Some(3),
                created_at: "2026-03-04T12:00:03.000Z".to_string(),
            },
            ThreadActivity {
                id: "checkpoint".to_string(),
                kind: "tool.completed".to_string(),
                summary: "Checkpoint captured".to_string(),
                tone: ActivityTone::Info,
                payload: ActivityPayload::default(),
                turn_id: Some("turn-1".to_string()),
                sequence: Some(4),
                created_at: "2026-03-04T12:00:04.000Z".to_string(),
            },
            ThreadActivity {
                id: "other-turn".to_string(),
                kind: "task.progress".to_string(),
                summary: "Other turn".to_string(),
                tone: ActivityTone::Thinking,
                payload: ActivityPayload::default(),
                turn_id: Some("turn-2".to_string()),
                sequence: Some(5),
                created_at: "2026-03-04T12:00:05.000Z".to_string(),
            },
        ];

        let entries = derive_work_log_entries(&activities, Some("turn-1"));

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "completed");
        assert_eq!(entries[0].command.as_deref(), Some("cargo check"));
        assert_eq!(entries[0].detail.as_deref(), Some("Finished in 1s"));
    }

    #[test]
    fn pending_approval_reference_state_exposes_first_approval() {
        let snapshot = AppSnapshot::pending_approval_reference_state();
        let approval = snapshot.active_pending_approval().unwrap();

        assert_eq!(snapshot.threads[0].status, ThreadStatus::NeedsInput);
        assert!(snapshot.threads[0].has_pending_approvals);
        assert_eq!(approval.request_kind, ApprovalRequestKind::Command);
        assert_eq!(approval.request_id, "approval-command-run-tests");
        assert!(!snapshot.is_responding_to_request(&approval.request_id));
    }

    #[test]
    fn pending_user_input_reference_state_exposes_active_progress() {
        let snapshot = AppSnapshot::pending_user_input_reference_state();
        let progress = snapshot.active_pending_user_input_progress().unwrap();

        assert_eq!(snapshot.threads[0].status, ThreadStatus::NeedsInput);
        assert!(snapshot.threads[0].has_pending_user_input);
        assert_eq!(progress.question_index, 0);
        assert_eq!(progress.active_question.unwrap().id, "surface");
        assert_eq!(progress.selected_option_labels, vec!["Composer"]);
        assert_eq!(
            progress.resolved_answer,
            Some(PendingUserInputAnswer::Text("Composer".to_string()))
        );
        assert!(progress.can_advance);
        assert!(!progress.is_complete);
    }

    #[test]
    fn terminal_drawer_reference_state_exposes_open_split_terminal() {
        let snapshot = AppSnapshot::terminal_drawer_reference_state();

        assert!(snapshot.terminal_open());
        assert_eq!(
            snapshot.terminal_state.terminal_ids,
            vec!["default", "terminal-2"]
        );
        assert_eq!(snapshot.terminal_state.active_terminal_id, "terminal-2");
        assert_eq!(
            snapshot.terminal_state.terminal_groups,
            vec![ThreadTerminalGroup {
                id: "group-default".to_string(),
                terminal_ids: vec!["default".to_string(), "terminal-2".to_string()],
            }]
        );
        assert_eq!(snapshot.terminal_event_entries.len(), 4);
    }

    #[test]
    fn diff_route_search_matches_upstream_parser_contract() {
        assert_eq!(
            parse_diff_route_search(
                Some(DiffOpenValue::from("1")),
                Some("turn-1"),
                Some("src/app.ts")
            ),
            DiffRouteSearch {
                diff: Some("1".to_string()),
                diff_turn_id: Some("turn-1".to_string()),
                diff_file_path: Some("src/app.ts".to_string()),
            }
        );
        assert_eq!(
            parse_diff_route_search(Some(DiffOpenValue::Number(1)), Some("turn-1"), None),
            DiffRouteSearch {
                diff: Some("1".to_string()),
                diff_turn_id: Some("turn-1".to_string()),
                diff_file_path: None,
            }
        );
        assert_eq!(
            parse_diff_route_search(Some(DiffOpenValue::Bool(true)), Some("turn-1"), None),
            DiffRouteSearch {
                diff: Some("1".to_string()),
                diff_turn_id: Some("turn-1".to_string()),
                diff_file_path: None,
            }
        );
        assert_eq!(
            parse_diff_route_search(
                Some(DiffOpenValue::from("0")),
                Some("turn-1"),
                Some("src/app.ts")
            ),
            DiffRouteSearch::default()
        );
        assert_eq!(
            parse_diff_route_search(Some(DiffOpenValue::from("1")), None, Some("src/app.ts")),
            DiffRouteSearch {
                diff: Some("1".to_string()),
                diff_turn_id: None,
                diff_file_path: None,
            }
        );
        assert_eq!(
            parse_diff_route_search(Some(DiffOpenValue::from("1")), Some("  "), Some("  ")),
            DiffRouteSearch {
                diff: Some("1".to_string()),
                diff_turn_id: None,
                diff_file_path: None,
            }
        );
    }

    #[test]
    fn turn_diff_stats_sum_only_files_with_numeric_values() {
        let stat = summarize_turn_diff_stats(&[
            diff_file("README.md", Some(3), Some(1)),
            diff_file("docs/notes.md", None, None),
            diff_file("src/index.ts", Some(5), Some(2)),
        ]);

        assert_eq!(
            stat,
            TurnDiffStat {
                additions: 8,
                deletions: 3,
            }
        );
        assert!(has_non_zero_turn_diff_stat(stat));
    }

    #[test]
    fn builds_turn_diff_tree_with_aggregated_directory_stats() {
        let tree = build_turn_diff_tree(&[
            diff_file("src/index.ts", Some(2), Some(1)),
            diff_file("src/components/Button.tsx", Some(4), Some(2)),
            diff_file("README.md", Some(1), Some(0)),
        ]);

        assert_eq!(
            tree,
            vec![
                TurnDiffTreeNode::Directory {
                    name: "src".to_string(),
                    path: "src".to_string(),
                    stat: TurnDiffStat {
                        additions: 6,
                        deletions: 3,
                    },
                    children: vec![
                        TurnDiffTreeNode::Directory {
                            name: "components".to_string(),
                            path: "src/components".to_string(),
                            stat: TurnDiffStat {
                                additions: 4,
                                deletions: 2,
                            },
                            children: vec![TurnDiffTreeNode::File {
                                name: "Button.tsx".to_string(),
                                path: "src/components/Button.tsx".to_string(),
                                stat: Some(TurnDiffStat {
                                    additions: 4,
                                    deletions: 2,
                                }),
                            }],
                        },
                        TurnDiffTreeNode::File {
                            name: "index.ts".to_string(),
                            path: "src/index.ts".to_string(),
                            stat: Some(TurnDiffStat {
                                additions: 2,
                                deletions: 1,
                            }),
                        },
                    ],
                },
                TurnDiffTreeNode::File {
                    name: "README.md".to_string(),
                    path: "README.md".to_string(),
                    stat: Some(TurnDiffStat {
                        additions: 1,
                        deletions: 0,
                    }),
                },
            ]
        );
    }

    #[test]
    fn turn_diff_tree_keeps_missing_stats_and_normalizes_windows_paths() {
        let missing_stats = build_turn_diff_tree(&[
            diff_file("docs/notes.md", None, None),
            diff_file("docs/todo.md", Some(1), Some(1)),
        ]);
        assert_eq!(
            missing_stats,
            vec![TurnDiffTreeNode::Directory {
                name: "docs".to_string(),
                path: "docs".to_string(),
                stat: TurnDiffStat {
                    additions: 1,
                    deletions: 1,
                },
                children: vec![
                    TurnDiffTreeNode::File {
                        name: "notes.md".to_string(),
                        path: "docs/notes.md".to_string(),
                        stat: None,
                    },
                    TurnDiffTreeNode::File {
                        name: "todo.md".to_string(),
                        path: "docs/todo.md".to_string(),
                        stat: Some(TurnDiffStat {
                            additions: 1,
                            deletions: 1,
                        }),
                    },
                ],
            }]
        );

        assert_eq!(
            build_turn_diff_tree(&[diff_file("apps\\web\\src\\index.ts", Some(2), Some(1))]),
            vec![TurnDiffTreeNode::Directory {
                name: "apps/web/src".to_string(),
                path: "apps/web/src".to_string(),
                stat: TurnDiffStat {
                    additions: 2,
                    deletions: 1,
                },
                children: vec![TurnDiffTreeNode::File {
                    name: "index.ts".to_string(),
                    path: "apps/web/src/index.ts".to_string(),
                    stat: Some(TurnDiffStat {
                        additions: 2,
                        deletions: 1,
                    }),
                }],
            }]
        );
    }

    #[test]
    fn turn_diff_tree_compacts_directory_chains_and_sorts_numerically() {
        let tree = build_turn_diff_tree(&[
            diff_file("apps/server/src/file10.ts", Some(2), Some(1)),
            diff_file("apps/server/src/file2.ts", Some(4), Some(0)),
            diff_file("apps/server/main.ts", Some(1), Some(0)),
        ]);

        assert_eq!(
            tree,
            vec![TurnDiffTreeNode::Directory {
                name: "apps/server".to_string(),
                path: "apps/server".to_string(),
                stat: TurnDiffStat {
                    additions: 7,
                    deletions: 1,
                },
                children: vec![
                    TurnDiffTreeNode::Directory {
                        name: "src".to_string(),
                        path: "apps/server/src".to_string(),
                        stat: TurnDiffStat {
                            additions: 6,
                            deletions: 1,
                        },
                        children: vec![
                            TurnDiffTreeNode::File {
                                name: "file2.ts".to_string(),
                                path: "apps/server/src/file2.ts".to_string(),
                                stat: Some(TurnDiffStat {
                                    additions: 4,
                                    deletions: 0,
                                }),
                            },
                            TurnDiffTreeNode::File {
                                name: "file10.ts".to_string(),
                                path: "apps/server/src/file10.ts".to_string(),
                                stat: Some(TurnDiffStat {
                                    additions: 2,
                                    deletions: 1,
                                }),
                            },
                        ],
                    },
                    TurnDiffTreeNode::File {
                        name: "main.ts".to_string(),
                        path: "apps/server/main.ts".to_string(),
                        stat: Some(TurnDiffStat {
                            additions: 1,
                            deletions: 0,
                        }),
                    },
                ],
            }]
        );
    }

    #[test]
    fn diff_panel_reference_state_exposes_selected_turn_and_file() {
        let snapshot = AppSnapshot::diff_panel_reference_state();
        let selected = snapshot.selected_turn_diff_summary().unwrap();

        assert!(snapshot.diff_open());
        assert_eq!(selected.turn_id, "turn-r3code-ui-shell-2");
        assert_eq!(
            snapshot.selected_diff_file_path(),
            Some("crates/r3_ui/src/shell.rs")
        );
        assert_eq!(
            snapshot.ordered_turn_diff_summaries()[0].turn_id,
            selected.turn_id
        );
        assert_eq!(
            summarize_turn_diff_stats(&selected.files),
            TurnDiffStat {
                additions: 208,
                deletions: 22,
            }
        );
    }

    #[test]
    fn default_terminal_state_matches_upstream_contract() {
        assert_eq!(
            create_default_thread_terminal_state(),
            ThreadTerminalState {
                terminal_open: false,
                terminal_height: DEFAULT_THREAD_TERMINAL_HEIGHT,
                terminal_ids: vec!["default".to_string()],
                running_terminal_ids: Vec::new(),
                active_terminal_id: "default".to_string(),
                terminal_groups: vec![ThreadTerminalGroup {
                    id: "group-default".to_string(),
                    terminal_ids: vec!["default".to_string()],
                }],
                active_terminal_group_id: "group-default".to_string(),
            }
        );
    }

    #[test]
    fn terminal_split_and_new_group_behaviors_match_upstream_store() {
        let state = create_default_thread_terminal_state();
        let split = split_thread_terminal(&state, "terminal-2");

        assert!(split.terminal_open);
        assert_eq!(split.terminal_ids, vec!["default", "terminal-2"]);
        assert_eq!(split.active_terminal_id, "terminal-2");
        assert_eq!(
            split.terminal_groups,
            vec![ThreadTerminalGroup {
                id: "group-default".to_string(),
                terminal_ids: vec!["default".to_string(), "terminal-2".to_string()],
            }]
        );

        let separate = new_thread_terminal(&state, "terminal-2");
        assert_eq!(separate.active_terminal_id, "terminal-2");
        assert_eq!(separate.active_terminal_group_id, "group-terminal-2");
        assert_eq!(
            separate.terminal_groups,
            vec![
                ThreadTerminalGroup {
                    id: "group-default".to_string(),
                    terminal_ids: vec!["default".to_string()],
                },
                ThreadTerminalGroup {
                    id: "group-terminal-2".to_string(),
                    terminal_ids: vec!["terminal-2".to_string()],
                },
            ]
        );
    }

    #[test]
    fn terminal_split_caps_at_four_per_group() {
        let mut state = create_default_thread_terminal_state();
        for terminal_id in ["terminal-2", "terminal-3", "terminal-4", "terminal-5"] {
            state = split_thread_terminal(&state, terminal_id);
        }

        assert_eq!(
            state.terminal_ids,
            vec!["default", "terminal-2", "terminal-3", "terminal-4"]
        );
        assert_eq!(state.terminal_groups[0].terminal_ids.len(), 4);
    }

    #[test]
    fn terminal_close_keeps_valid_active_terminal() {
        let mut state = create_default_thread_terminal_state();
        state = split_thread_terminal(&state, "terminal-2");
        state = split_thread_terminal(&state, "terminal-3");
        state = close_thread_terminal(&state, "terminal-3");

        assert_eq!(state.active_terminal_id, "terminal-2");
        assert_eq!(state.terminal_ids, vec!["default", "terminal-2"]);
        assert_eq!(
            state.terminal_groups,
            vec![ThreadTerminalGroup {
                id: "group-default".to_string(),
                terminal_ids: vec!["default".to_string(), "terminal-2".to_string()],
            }]
        );
    }

    #[test]
    fn terminal_activity_and_event_filters_match_upstream_helpers() {
        let mut state =
            split_thread_terminal(&create_default_thread_terminal_state(), "terminal-2");
        state = set_thread_terminal_activity(&state, "terminal-2", true);
        assert_eq!(state.running_terminal_ids, vec!["terminal-2"]);
        state = set_thread_terminal_activity(&state, "terminal-2", false);
        assert_eq!(state.running_terminal_ids, Vec::<String>::new());

        let output = TerminalEvent::Output {
            thread_id: "thread-1".to_string(),
            terminal_id: "default".to_string(),
            created_at: "2026-04-02T20:00:00.000Z".to_string(),
            data: "before".to_string(),
        };
        let activity = TerminalEvent::Activity {
            thread_id: "thread-1".to_string(),
            terminal_id: "default".to_string(),
            created_at: "2026-04-02T20:00:01.000Z".to_string(),
            has_running_subprocess: true,
        };
        let exited = TerminalEvent::Exited {
            thread_id: "thread-1".to_string(),
            terminal_id: "default".to_string(),
            created_at: "2026-04-02T20:00:02.000Z".to_string(),
            exit_code: Some(0),
            exit_signal: None,
        };
        assert_eq!(terminal_running_subprocess_from_event(&output), None);
        assert_eq!(
            terminal_running_subprocess_from_event(&activity),
            Some(true)
        );
        assert_eq!(terminal_running_subprocess_from_event(&exited), Some(false));

        let entries = vec![
            TerminalEventEntry {
                id: 1,
                event: output,
            },
            TerminalEventEntry {
                id: 2,
                event: activity,
            },
            TerminalEventEntry {
                id: 3,
                event: exited,
            },
        ];
        assert_eq!(
            select_terminal_event_entries_after_snapshot(&entries, "2026-04-02T20:00:00.500Z")
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
        assert_eq!(
            select_pending_terminal_event_entries(&entries, 1)
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    #[test]
    fn derives_thread_from_environment_state_in_id_order() {
        let thread_id = "thread-browser-test";
        let mut state = EnvironmentState::default();
        state.thread_shell_by_id.insert(
            thread_id.to_string(),
            ThreadShell {
                id: thread_id.to_string(),
                environment_id: "environment-local".to_string(),
                codex_thread_id: None,
                project_id: "project-1".to_string(),
                title: "Browser test thread".to_string(),
                runtime_mode: RuntimeMode::FullAccess,
                interaction_mode: ProviderInteractionMode::Default,
                error: None,
                created_at: "2026-03-04T12:00:00.000Z".to_string(),
                archived_at: None,
                updated_at: Some("2026-03-04T12:00:03.000Z".to_string()),
                branch: Some("main".to_string()),
                worktree_path: None,
            },
        );
        state.thread_session_by_id.insert(
            thread_id.to_string(),
            ThreadSession {
                provider: "codex".to_string(),
                provider_instance_id: Some("codex".to_string()),
                status: SessionPhase::Ready,
                active_turn_id: None,
                created_at: "2026-03-04T12:00:00.000Z".to_string(),
                updated_at: "2026-03-04T12:00:03.000Z".to_string(),
                last_error: None,
                orchestration_status: "ready".to_string(),
            },
        );
        state.message_ids_by_thread_id.insert(
            thread_id.to_string(),
            vec![
                "msg-user".to_string(),
                "msg-missing".to_string(),
                "msg-assistant".to_string(),
            ],
        );
        state.message_by_thread_id.insert(
            thread_id.to_string(),
            BTreeMap::from([
                (
                    "msg-assistant".to_string(),
                    ChatMessage::assistant(
                        "msg-assistant",
                        "assistant filler 0",
                        "2026-03-04T12:00:03.000Z",
                    ),
                ),
                (
                    "msg-user".to_string(),
                    ChatMessage::user("msg-user", "bootstrap", "2026-03-04T12:00:00.000Z"),
                ),
            ]),
        );
        state.activity_ids_by_thread_id.insert(
            thread_id.to_string(),
            vec!["activity-1".to_string(), "activity-2".to_string()],
        );
        state.activity_by_thread_id.insert(
            thread_id.to_string(),
            BTreeMap::from([
                (
                    "activity-1".to_string(),
                    ThreadActivity {
                        id: "activity-1".to_string(),
                        kind: "tool.started".to_string(),
                        summary: "Read file".to_string(),
                        tone: ActivityTone::Tool,
                        payload: ActivityPayload::default(),
                        turn_id: Some("turn-1".to_string()),
                        sequence: Some(1),
                        created_at: "2026-03-04T12:00:01.000Z".to_string(),
                    },
                ),
                (
                    "activity-2".to_string(),
                    ThreadActivity {
                        id: "activity-2".to_string(),
                        kind: "tool.completed".to_string(),
                        summary: "Read file".to_string(),
                        tone: ActivityTone::Tool,
                        payload: ActivityPayload::default(),
                        turn_id: Some("turn-1".to_string()),
                        sequence: Some(2),
                        created_at: "2026-03-04T12:00:02.000Z".to_string(),
                    },
                ),
            ]),
        );
        state
            .turn_diff_ids_by_thread_id
            .insert(thread_id.to_string(), vec!["turn-1".to_string()]);
        state.turn_diff_summary_by_thread_id.insert(
            thread_id.to_string(),
            BTreeMap::from([(
                "turn-1".to_string(),
                TurnDiffSummary {
                    turn_id: "turn-1".to_string(),
                    completed_at: "2026-03-04T12:00:04.000Z".to_string(),
                    status: Some("completed".to_string()),
                    files: vec![TurnDiffFileChange {
                        path: "apps/web/src/components/chat/MessagesTimeline.tsx".to_string(),
                        kind: Some("modified".to_string()),
                        additions: Some(4),
                        deletions: Some(1),
                    }],
                    checkpoint_ref: None,
                    assistant_message_id: Some("msg-assistant".to_string()),
                    checkpoint_turn_count: Some(1),
                },
            )]),
        );

        let thread = get_thread_from_environment_state(&state, thread_id).unwrap();

        assert_eq!(thread.shell.title, "Browser test thread");
        assert_eq!(thread.session.unwrap().status, SessionPhase::Ready);
        assert_eq!(
            thread
                .messages
                .iter()
                .map(|message| message.id.as_str())
                .collect::<Vec<_>>(),
            vec!["msg-user", "msg-assistant"]
        );
        assert_eq!(thread.messages[0].role, MessageRole::User);
        assert_eq!(thread.activities.len(), 2);
        assert_eq!(thread.turn_diff_summaries[0].files[0].additions, Some(4));
    }

    #[test]
    fn missing_thread_shell_returns_none() {
        let state = EnvironmentState::default();

        assert!(get_thread_from_environment_state(&state, "missing-thread").is_none());
    }

    fn vcs_ref(name: &str, is_default: bool, worktree_path: Option<&str>) -> VcsRef {
        VcsRef {
            name: name.to_string(),
            current: false,
            is_default,
            is_remote: false,
            remote_name: None,
            worktree_path: worktree_path.map(str::to_string),
        }
    }

    fn remote_vcs_ref(name: &str, remote_name: &str) -> VcsRef {
        VcsRef {
            name: name.to_string(),
            current: false,
            is_default: false,
            is_remote: true,
            remote_name: Some(remote_name.to_string()),
            worktree_path: None,
        }
    }

    fn diff_file(path: &str, additions: Option<u32>, deletions: Option<u32>) -> TurnDiffFileChange {
        TurnDiffFileChange {
            path: path.to_string(),
            kind: Some("modified".to_string()),
            additions,
            deletions,
        }
    }

    #[test]
    fn message_roles_expose_upstream_display_authors() {
        assert_eq!(MessageRole::User.display_author(), "You");
        assert_eq!(MessageRole::Assistant.display_author(), APP_NAME);
        assert_eq!(MessageRole::System.display_author(), "System");
    }

    fn activity(
        id: &str,
        kind: &str,
        created_at: &str,
        request_id: Option<&str>,
        payload: ActivityPayload,
    ) -> ThreadActivity {
        ThreadActivity {
            id: id.to_string(),
            kind: kind.to_string(),
            summary: kind.to_string(),
            tone: ActivityTone::Info,
            payload: ActivityPayload {
                request_id: request_id.map(str::to_string),
                ..payload
            },
            turn_id: None,
            sequence: None,
            created_at: created_at.to_string(),
        }
    }

    fn user_input_question(id: &str) -> UserInputQuestion {
        UserInputQuestion {
            id: id.to_string(),
            header: "Scope".to_string(),
            question: "What should this change cover?".to_string(),
            options: vec![UserInputQuestionOption {
                label: "Tight".to_string(),
                description: "Touch only the footer layout logic.".to_string(),
            }],
            multi_select: false,
        }
    }

    fn multi_select_question(id: &str) -> UserInputQuestion {
        UserInputQuestion {
            id: id.to_string(),
            header: "Areas".to_string(),
            question: "Which areas should this change cover?".to_string(),
            options: vec![
                UserInputQuestionOption {
                    label: "Server".to_string(),
                    description: "Server".to_string(),
                },
                UserInputQuestionOption {
                    label: "Web".to_string(),
                    description: "Web".to_string(),
                },
            ],
            multi_select: true,
        }
    }

    #[test]
    fn derives_pending_approvals_and_removes_resolved_requests() {
        let activities = vec![
            activity(
                "approval-open",
                "approval.requested",
                "2026-02-23T00:00:01.000Z",
                Some("req-1"),
                ActivityPayload {
                    request_kind: Some(ApprovalRequestKind::Command),
                    detail: Some("bun run lint".to_string()),
                    ..ActivityPayload::default()
                },
            ),
            activity(
                "approval-close",
                "approval.resolved",
                "2026-02-23T00:00:02.000Z",
                Some("req-2"),
                ActivityPayload::default(),
            ),
            activity(
                "approval-closed-request",
                "approval.requested",
                "2026-02-23T00:00:01.500Z",
                Some("req-2"),
                ActivityPayload {
                    request_kind: Some(ApprovalRequestKind::FileChange),
                    ..ActivityPayload::default()
                },
            ),
        ];

        assert_eq!(
            derive_pending_approvals(&activities),
            vec![PendingApproval {
                request_id: "req-1".to_string(),
                request_kind: ApprovalRequestKind::Command,
                created_at: "2026-02-23T00:00:01.000Z".to_string(),
                detail: Some("bun run lint".to_string()),
            }]
        );
    }

    #[test]
    fn derives_pending_approvals_from_canonical_request_type() {
        let activities = vec![activity(
            "approval-open-request-type",
            "approval.requested",
            "2026-02-23T00:00:01.000Z",
            Some("req-request-type"),
            ActivityPayload {
                request_type: Some("command_execution_approval".to_string()),
                detail: Some("pwd".to_string()),
                ..ActivityPayload::default()
            },
        )];

        assert_eq!(
            derive_pending_approvals(&activities),
            vec![PendingApproval {
                request_id: "req-request-type".to_string(),
                request_kind: ApprovalRequestKind::Command,
                created_at: "2026-02-23T00:00:01.000Z".to_string(),
                detail: Some("pwd".to_string()),
            }]
        );
    }

    #[test]
    fn stale_provider_approval_failure_clears_pending_request() {
        let activities = vec![
            activity(
                "approval-open-stale",
                "approval.requested",
                "2026-02-23T00:00:01.000Z",
                Some("req-stale-1"),
                ActivityPayload {
                    request_kind: Some(ApprovalRequestKind::Command),
                    ..ActivityPayload::default()
                },
            ),
            activity(
                "approval-failed-stale",
                "provider.approval.respond.failed",
                "2026-02-23T00:00:02.000Z",
                Some("req-stale-1"),
                ActivityPayload {
                    detail: Some("Unknown pending permission request: req-stale-1".to_string()),
                    ..ActivityPayload::default()
                },
            ),
        ];

        assert_eq!(derive_pending_approvals(&activities), Vec::new());
    }

    #[test]
    fn derives_pending_user_inputs_and_removes_resolved_requests() {
        let activities = vec![
            activity(
                "user-input-open",
                "user-input.requested",
                "2026-02-23T00:00:01.000Z",
                Some("req-user-input-1"),
                ActivityPayload {
                    questions: vec![user_input_question("sandbox_mode")],
                    ..ActivityPayload::default()
                },
            ),
            activity(
                "user-input-resolved",
                "user-input.resolved",
                "2026-02-23T00:00:02.000Z",
                Some("req-user-input-2"),
                ActivityPayload::default(),
            ),
            activity(
                "user-input-open-2",
                "user-input.requested",
                "2026-02-23T00:00:01.500Z",
                Some("req-user-input-2"),
                ActivityPayload {
                    questions: vec![user_input_question("approval")],
                    ..ActivityPayload::default()
                },
            ),
        ];

        assert_eq!(
            derive_pending_user_inputs(&activities),
            vec![PendingUserInput {
                request_id: "req-user-input-1".to_string(),
                created_at: "2026-02-23T00:00:01.000Z".to_string(),
                questions: vec![user_input_question("sandbox_mode")],
            }]
        );
    }

    #[test]
    fn stale_provider_user_input_failure_clears_pending_request() {
        let activities = vec![
            activity(
                "user-input-open-stale",
                "user-input.requested",
                "2026-02-23T00:00:01.000Z",
                Some("req-user-input-stale-1"),
                ActivityPayload {
                    questions: vec![user_input_question("sandbox_mode")],
                    ..ActivityPayload::default()
                },
            ),
            activity(
                "user-input-failed-stale",
                "provider.user-input.respond.failed",
                "2026-02-23T00:00:02.000Z",
                Some("req-user-input-stale-1"),
                ActivityPayload {
                    detail: Some(
                        "Stale pending user-input request: req-user-input-stale-1".to_string(),
                    ),
                    ..ActivityPayload::default()
                },
            ),
        ];

        assert_eq!(derive_pending_user_inputs(&activities), Vec::new());
    }

    #[test]
    fn pending_user_input_answer_prefers_custom_text() {
        let question = user_input_question("scope");
        let draft = PendingUserInputDraftAnswer {
            selected_option_labels: vec!["Tight".to_string()],
            custom_answer: Some("Keep the existing envelope for one release".to_string()),
        };

        assert_eq!(
            resolve_pending_user_input_answer(&question, Some(&draft)),
            Some(PendingUserInputAnswer::Text(
                "Keep the existing envelope for one release".to_string()
            ))
        );
    }

    #[test]
    fn pending_user_input_answer_returns_multi_select_arrays() {
        let question = multi_select_question("areas");
        let draft = PendingUserInputDraftAnswer {
            selected_option_labels: vec!["Server".to_string(), "Web".to_string()],
            custom_answer: None,
        };

        assert_eq!(
            resolve_pending_user_input_answer(&question, Some(&draft)),
            Some(PendingUserInputAnswer::Multiple(vec![
                "Server".to_string(),
                "Web".to_string(),
            ]))
        );
    }

    #[test]
    fn setting_custom_answer_clears_selected_options_when_non_empty() {
        let draft = PendingUserInputDraftAnswer {
            selected_option_labels: vec!["Server".to_string(), "Web".to_string()],
            custom_answer: None,
        };

        assert_eq!(
            set_pending_user_input_custom_answer(Some(&draft), "doesn't matter"),
            PendingUserInputDraftAnswer {
                selected_option_labels: Vec::new(),
                custom_answer: Some("doesn't matter".to_string()),
            }
        );
    }

    #[test]
    fn toggling_pending_user_input_options_matches_select_mode() {
        let multi = multi_select_question("areas");
        let selected = toggle_pending_user_input_option_selection(&multi, None, "Server");
        assert_eq!(
            selected,
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Server".to_string()],
                custom_answer: Some(String::new()),
            }
        );

        let removed = toggle_pending_user_input_option_selection(
            &multi,
            Some(&PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Server".to_string(), "Web".to_string()],
                custom_answer: None,
            }),
            "Server",
        );
        assert_eq!(
            removed,
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Web".to_string()],
                custom_answer: Some(String::new()),
            }
        );

        let single = user_input_question("scope");
        assert_eq!(
            toggle_pending_user_input_option_selection(&single, None, "Tight"),
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Tight".to_string()],
                custom_answer: Some(String::new()),
            }
        );
    }

    #[test]
    fn builds_pending_user_input_answer_map_only_when_complete() {
        let scope = user_input_question("scope");
        let compat = user_input_question("compat");
        let mut answers = BTreeMap::new();
        answers.insert(
            "scope".to_string(),
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Tight".to_string()],
                custom_answer: None,
            },
        );

        assert_eq!(
            build_pending_user_input_answers(&[scope.clone(), compat.clone()], &answers),
            None
        );

        answers.insert(
            "compat".to_string(),
            PendingUserInputDraftAnswer {
                selected_option_labels: Vec::new(),
                custom_answer: Some("Keep the current envelope for one release window".to_string()),
            },
        );

        assert_eq!(
            build_pending_user_input_answers(&[scope, compat], &answers),
            Some(BTreeMap::from([
                (
                    "compat".to_string(),
                    PendingUserInputAnswer::Text(
                        "Keep the current envelope for one release window".to_string()
                    ),
                ),
                (
                    "scope".to_string(),
                    PendingUserInputAnswer::Text("Tight".to_string()),
                ),
            ]))
        );
    }

    #[test]
    fn derives_pending_user_input_question_progress() {
        let questions = vec![user_input_question("scope"), user_input_question("compat")];
        let draft_answers = BTreeMap::from([(
            "scope".to_string(),
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Tight".to_string()],
                custom_answer: None,
            },
        )]);

        let progress = derive_pending_user_input_progress(&questions, &draft_answers, 0);

        assert_eq!(progress.question_index, 0);
        assert_eq!(progress.active_question, Some(questions[0].clone()));
        assert_eq!(progress.selected_option_labels, vec!["Tight"]);
        assert_eq!(
            progress.resolved_answer,
            Some(PendingUserInputAnswer::Text("Tight".to_string()))
        );
        assert_eq!(progress.answered_question_count, 1);
        assert!(!progress.is_last_question);
        assert!(!progress.is_complete);
        assert!(progress.can_advance);
        assert_eq!(
            find_first_unanswered_pending_user_input_question_index(&questions, &draft_answers),
            1
        );
    }

    #[test]
    fn completed_pending_user_input_progress_uses_last_question_index() {
        let questions = vec![user_input_question("scope"), user_input_question("compat")];
        let draft_answers = BTreeMap::from([
            (
                "scope".to_string(),
                PendingUserInputDraftAnswer {
                    selected_option_labels: vec!["Tight".to_string()],
                    custom_answer: None,
                },
            ),
            (
                "compat".to_string(),
                PendingUserInputDraftAnswer {
                    selected_option_labels: Vec::new(),
                    custom_answer: Some("Keep it for one release window".to_string()),
                },
            ),
        ]);

        assert_eq!(
            find_first_unanswered_pending_user_input_question_index(&questions, &draft_answers),
            1
        );
        assert_eq!(
            count_answered_pending_user_input_questions(&questions, &draft_answers),
            2
        );

        let progress = derive_pending_user_input_progress(&questions, &draft_answers, 9);

        assert_eq!(progress.question_index, 1);
        assert!(progress.is_last_question);
        assert!(progress.is_complete);
    }
}
