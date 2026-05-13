use crate::server::ServerSettingsForClient;
use crate::{
    KeybindingsConfigIssue, ResolvedKeybindingRule, RpcAckLatencyState, ServerProvider,
    WS_RECONNECT_MAX_RETRIES, WsConnectionMetadata, WsConnectionStatus, acknowledge_rpc_request,
    clear_all_tracked_rpc_requests, default_resolved_keybindings,
    get_ws_reconnect_delay_ms_for_retry, initial_rpc_ack_latency_state,
    initial_ws_connection_status, record_ws_connection_attempt, record_ws_connection_closed_at,
    record_ws_connection_errored_at, record_ws_connection_opened_at, track_rpc_request_sent_at,
};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RpcAggregate {
    Auth,
    Filesystem,
    Git,
    Orchestration,
    Server,
    Shell,
    SourceControl,
    Terminal,
    Vcs,
    Workspace,
}

impl RpcAggregate {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::Filesystem => "filesystem",
            Self::Git => "git",
            Self::Orchestration => "orchestration",
            Self::Server => "server",
            Self::Shell => "shell",
            Self::SourceControl => "source-control",
            Self::Terminal => "terminal",
            Self::Vcs => "vcs",
            Self::Workspace => "workspace",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RpcStreamingMode {
    RequestResponse,
    Stream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WsRpcMethod {
    ProjectsList,
    ProjectsAdd,
    ProjectsRemove,
    ProjectsSearchEntries,
    ProjectsWriteFile,
    ShellOpenInEditor,
    FilesystemBrowse,
    VcsPull,
    VcsRefreshStatus,
    VcsListRefs,
    VcsCreateWorktree,
    VcsRemoveWorktree,
    VcsCreateRef,
    VcsSwitchRef,
    VcsInit,
    GitRunStackedAction,
    GitResolvePullRequest,
    GitPreparePullRequestThread,
    TerminalOpen,
    TerminalWrite,
    TerminalResize,
    TerminalClear,
    TerminalRestart,
    TerminalClose,
    ServerGetConfig,
    ServerRefreshProviders,
    ServerUpdateProvider,
    ServerUpsertKeybinding,
    ServerRemoveKeybinding,
    ServerGetSettings,
    ServerUpdateSettings,
    ServerDiscoverSourceControl,
    ServerGetTraceDiagnostics,
    ServerGetProcessDiagnostics,
    ServerSignalProcess,
    SourceControlLookupRepository,
    SourceControlCloneRepository,
    SourceControlPublishRepository,
    SubscribeVcsStatus,
    SubscribeTerminalEvents,
    SubscribeServerConfig,
    SubscribeServerLifecycle,
    SubscribeAuthAccess,
    OrchestrationDispatchCommand,
    OrchestrationGetTurnDiff,
    OrchestrationGetFullThreadDiff,
    OrchestrationReplayEvents,
    OrchestrationGetArchivedShellSnapshot,
    OrchestrationSubscribeShell,
    OrchestrationSubscribeThread,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WsRpcMethodSpec {
    pub method: WsRpcMethod,
    pub wire_name: &'static str,
    pub aggregate: RpcAggregate,
    pub streaming_mode: RpcStreamingMode,
}

impl WsRpcMethod {
    pub fn wire_name(self) -> &'static str {
        self.spec().wire_name
    }

    pub fn aggregate(self) -> RpcAggregate {
        self.spec().aggregate
    }

    pub fn streaming_mode(self) -> RpcStreamingMode {
        self.spec().streaming_mode
    }

    pub fn is_stream(self) -> bool {
        self.streaming_mode() == RpcStreamingMode::Stream
    }

    pub fn spec(self) -> WsRpcMethodSpec {
        WS_RPC_METHOD_SPECS
            .iter()
            .copied()
            .find(|spec| spec.method == self)
            .expect("every WsRpcMethod must have a method spec")
    }
}

pub const WS_RPC_METHOD_COUNT: usize = 50;

pub const WS_RPC_METHOD_SPECS: [WsRpcMethodSpec; WS_RPC_METHOD_COUNT] = [
    spec(
        WsRpcMethod::ProjectsList,
        "projects.list",
        RpcAggregate::Workspace,
        false,
    ),
    spec(
        WsRpcMethod::ProjectsAdd,
        "projects.add",
        RpcAggregate::Workspace,
        false,
    ),
    spec(
        WsRpcMethod::ProjectsRemove,
        "projects.remove",
        RpcAggregate::Workspace,
        false,
    ),
    spec(
        WsRpcMethod::ProjectsSearchEntries,
        "projects.searchEntries",
        RpcAggregate::Workspace,
        false,
    ),
    spec(
        WsRpcMethod::ProjectsWriteFile,
        "projects.writeFile",
        RpcAggregate::Workspace,
        false,
    ),
    spec(
        WsRpcMethod::ShellOpenInEditor,
        "shell.openInEditor",
        RpcAggregate::Shell,
        false,
    ),
    spec(
        WsRpcMethod::FilesystemBrowse,
        "filesystem.browse",
        RpcAggregate::Filesystem,
        false,
    ),
    spec(WsRpcMethod::VcsPull, "vcs.pull", RpcAggregate::Git, false),
    spec(
        WsRpcMethod::VcsRefreshStatus,
        "vcs.refreshStatus",
        RpcAggregate::Vcs,
        false,
    ),
    spec(
        WsRpcMethod::VcsListRefs,
        "vcs.listRefs",
        RpcAggregate::Vcs,
        false,
    ),
    spec(
        WsRpcMethod::VcsCreateWorktree,
        "vcs.createWorktree",
        RpcAggregate::Vcs,
        false,
    ),
    spec(
        WsRpcMethod::VcsRemoveWorktree,
        "vcs.removeWorktree",
        RpcAggregate::Vcs,
        false,
    ),
    spec(
        WsRpcMethod::VcsCreateRef,
        "vcs.createRef",
        RpcAggregate::Vcs,
        false,
    ),
    spec(
        WsRpcMethod::VcsSwitchRef,
        "vcs.switchRef",
        RpcAggregate::Vcs,
        false,
    ),
    spec(WsRpcMethod::VcsInit, "vcs.init", RpcAggregate::Vcs, false),
    spec(
        WsRpcMethod::GitRunStackedAction,
        "git.runStackedAction",
        RpcAggregate::Vcs,
        true,
    ),
    spec(
        WsRpcMethod::GitResolvePullRequest,
        "git.resolvePullRequest",
        RpcAggregate::Git,
        false,
    ),
    spec(
        WsRpcMethod::GitPreparePullRequestThread,
        "git.preparePullRequestThread",
        RpcAggregate::Git,
        false,
    ),
    spec(
        WsRpcMethod::TerminalOpen,
        "terminal.open",
        RpcAggregate::Terminal,
        false,
    ),
    spec(
        WsRpcMethod::TerminalWrite,
        "terminal.write",
        RpcAggregate::Terminal,
        false,
    ),
    spec(
        WsRpcMethod::TerminalResize,
        "terminal.resize",
        RpcAggregate::Terminal,
        false,
    ),
    spec(
        WsRpcMethod::TerminalClear,
        "terminal.clear",
        RpcAggregate::Terminal,
        false,
    ),
    spec(
        WsRpcMethod::TerminalRestart,
        "terminal.restart",
        RpcAggregate::Terminal,
        false,
    ),
    spec(
        WsRpcMethod::TerminalClose,
        "terminal.close",
        RpcAggregate::Terminal,
        false,
    ),
    spec(
        WsRpcMethod::ServerGetConfig,
        "server.getConfig",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::ServerRefreshProviders,
        "server.refreshProviders",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::ServerUpdateProvider,
        "server.updateProvider",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::ServerUpsertKeybinding,
        "server.upsertKeybinding",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::ServerRemoveKeybinding,
        "server.removeKeybinding",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::ServerGetSettings,
        "server.getSettings",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::ServerUpdateSettings,
        "server.updateSettings",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::ServerDiscoverSourceControl,
        "server.discoverSourceControl",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::ServerGetTraceDiagnostics,
        "server.getTraceDiagnostics",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::ServerGetProcessDiagnostics,
        "server.getProcessDiagnostics",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::ServerSignalProcess,
        "server.signalProcess",
        RpcAggregate::Server,
        false,
    ),
    spec(
        WsRpcMethod::SourceControlLookupRepository,
        "sourceControl.lookupRepository",
        RpcAggregate::SourceControl,
        false,
    ),
    spec(
        WsRpcMethod::SourceControlCloneRepository,
        "sourceControl.cloneRepository",
        RpcAggregate::SourceControl,
        false,
    ),
    spec(
        WsRpcMethod::SourceControlPublishRepository,
        "sourceControl.publishRepository",
        RpcAggregate::SourceControl,
        false,
    ),
    spec(
        WsRpcMethod::SubscribeVcsStatus,
        "subscribeVcsStatus",
        RpcAggregate::Vcs,
        true,
    ),
    spec(
        WsRpcMethod::SubscribeTerminalEvents,
        "subscribeTerminalEvents",
        RpcAggregate::Terminal,
        true,
    ),
    spec(
        WsRpcMethod::SubscribeServerConfig,
        "subscribeServerConfig",
        RpcAggregate::Server,
        true,
    ),
    spec(
        WsRpcMethod::SubscribeServerLifecycle,
        "subscribeServerLifecycle",
        RpcAggregate::Server,
        true,
    ),
    spec(
        WsRpcMethod::SubscribeAuthAccess,
        "subscribeAuthAccess",
        RpcAggregate::Auth,
        true,
    ),
    spec(
        WsRpcMethod::OrchestrationDispatchCommand,
        "orchestration.dispatchCommand",
        RpcAggregate::Orchestration,
        false,
    ),
    spec(
        WsRpcMethod::OrchestrationGetTurnDiff,
        "orchestration.getTurnDiff",
        RpcAggregate::Orchestration,
        false,
    ),
    spec(
        WsRpcMethod::OrchestrationGetFullThreadDiff,
        "orchestration.getFullThreadDiff",
        RpcAggregate::Orchestration,
        false,
    ),
    spec(
        WsRpcMethod::OrchestrationReplayEvents,
        "orchestration.replayEvents",
        RpcAggregate::Orchestration,
        false,
    ),
    spec(
        WsRpcMethod::OrchestrationGetArchivedShellSnapshot,
        "orchestration.getArchivedShellSnapshot",
        RpcAggregate::Orchestration,
        false,
    ),
    spec(
        WsRpcMethod::OrchestrationSubscribeShell,
        "orchestration.subscribeShell",
        RpcAggregate::Orchestration,
        true,
    ),
    spec(
        WsRpcMethod::OrchestrationSubscribeThread,
        "orchestration.subscribeThread",
        RpcAggregate::Orchestration,
        true,
    ),
];

const fn spec(
    method: WsRpcMethod,
    wire_name: &'static str,
    aggregate: RpcAggregate,
    is_stream: bool,
) -> WsRpcMethodSpec {
    WsRpcMethodSpec {
        method,
        wire_name,
        aggregate,
        streaming_mode: if is_stream {
            RpcStreamingMode::Stream
        } else {
            RpcStreamingMode::RequestResponse
        },
    }
}

pub fn parse_ws_rpc_method(wire_name: &str) -> Option<WsRpcMethod> {
    WS_RPC_METHOD_SPECS
        .iter()
        .find(|spec| spec.wire_name == wire_name)
        .map(|spec| spec.method)
}

pub fn ws_rpc_methods_for_aggregate(aggregate: RpcAggregate) -> Vec<WsRpcMethod> {
    WS_RPC_METHOD_SPECS
        .iter()
        .filter(|spec| spec.aggregate == aggregate)
        .map(|spec| spec.method)
        .collect()
}

pub fn ws_rpc_stream_methods() -> Vec<WsRpcMethod> {
    WS_RPC_METHOD_SPECS
        .iter()
        .filter(|spec| spec.streaming_mode == RpcStreamingMode::Stream)
        .map(|spec| spec.method)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WsRpcSchemaSpec {
    pub method: WsRpcMethod,
    pub rpc_const: &'static str,
    pub payload_schema: &'static str,
    pub success_schema: Option<&'static str>,
    pub error_schema: Option<&'static str>,
    pub stream: bool,
}

pub const WS_RPC_GROUP_METHOD_COUNT: usize = 47;

pub const WS_RPC_GROUP_METHODS: [WsRpcMethod; WS_RPC_GROUP_METHOD_COUNT] = [
    WsRpcMethod::ServerGetConfig,
    WsRpcMethod::ServerRefreshProviders,
    WsRpcMethod::ServerUpdateProvider,
    WsRpcMethod::ServerUpsertKeybinding,
    WsRpcMethod::ServerRemoveKeybinding,
    WsRpcMethod::ServerGetSettings,
    WsRpcMethod::ServerUpdateSettings,
    WsRpcMethod::ServerDiscoverSourceControl,
    WsRpcMethod::ServerGetTraceDiagnostics,
    WsRpcMethod::ServerGetProcessDiagnostics,
    WsRpcMethod::ServerSignalProcess,
    WsRpcMethod::SourceControlLookupRepository,
    WsRpcMethod::SourceControlCloneRepository,
    WsRpcMethod::SourceControlPublishRepository,
    WsRpcMethod::ProjectsSearchEntries,
    WsRpcMethod::ProjectsWriteFile,
    WsRpcMethod::ShellOpenInEditor,
    WsRpcMethod::FilesystemBrowse,
    WsRpcMethod::SubscribeVcsStatus,
    WsRpcMethod::VcsPull,
    WsRpcMethod::VcsRefreshStatus,
    WsRpcMethod::GitRunStackedAction,
    WsRpcMethod::GitResolvePullRequest,
    WsRpcMethod::GitPreparePullRequestThread,
    WsRpcMethod::VcsListRefs,
    WsRpcMethod::VcsCreateWorktree,
    WsRpcMethod::VcsRemoveWorktree,
    WsRpcMethod::VcsCreateRef,
    WsRpcMethod::VcsSwitchRef,
    WsRpcMethod::VcsInit,
    WsRpcMethod::TerminalOpen,
    WsRpcMethod::TerminalWrite,
    WsRpcMethod::TerminalResize,
    WsRpcMethod::TerminalClear,
    WsRpcMethod::TerminalRestart,
    WsRpcMethod::TerminalClose,
    WsRpcMethod::SubscribeTerminalEvents,
    WsRpcMethod::SubscribeServerConfig,
    WsRpcMethod::SubscribeServerLifecycle,
    WsRpcMethod::SubscribeAuthAccess,
    WsRpcMethod::OrchestrationDispatchCommand,
    WsRpcMethod::OrchestrationGetTurnDiff,
    WsRpcMethod::OrchestrationGetFullThreadDiff,
    WsRpcMethod::OrchestrationReplayEvents,
    WsRpcMethod::OrchestrationGetArchivedShellSnapshot,
    WsRpcMethod::OrchestrationSubscribeShell,
    WsRpcMethod::OrchestrationSubscribeThread,
];

pub const WS_RPC_SCHEMA_SPECS: [WsRpcSchemaSpec; WS_RPC_GROUP_METHOD_COUNT] = [
    schema(
        WsRpcMethod::ServerGetConfig,
        "WsServerGetConfigRpc",
        "Schema.Struct({})",
        Some("ServerConfig"),
        Some("Schema.Union([KeybindingsConfigError, ServerSettingsError])"),
        false,
    ),
    schema(
        WsRpcMethod::ServerRefreshProviders,
        "WsServerRefreshProvidersRpc",
        "Schema.Struct({ instanceId: Schema.optional(ProviderInstanceId) })",
        Some("ServerProviderUpdatedPayload"),
        None,
        false,
    ),
    schema(
        WsRpcMethod::ServerUpdateProvider,
        "WsServerUpdateProviderRpc",
        "ServerProviderUpdateInput",
        Some("ServerProviderUpdatedPayload"),
        Some("ServerProviderUpdateError"),
        false,
    ),
    schema(
        WsRpcMethod::ServerUpsertKeybinding,
        "WsServerUpsertKeybindingRpc",
        "ServerUpsertKeybindingInput",
        Some("ServerUpsertKeybindingResult"),
        Some("KeybindingsConfigError"),
        false,
    ),
    schema(
        WsRpcMethod::ServerRemoveKeybinding,
        "WsServerRemoveKeybindingRpc",
        "ServerRemoveKeybindingInput",
        Some("ServerRemoveKeybindingResult"),
        Some("KeybindingsConfigError"),
        false,
    ),
    schema(
        WsRpcMethod::ServerGetSettings,
        "WsServerGetSettingsRpc",
        "Schema.Struct({})",
        Some("ServerSettings"),
        Some("ServerSettingsError"),
        false,
    ),
    schema(
        WsRpcMethod::ServerUpdateSettings,
        "WsServerUpdateSettingsRpc",
        "Schema.Struct({ patch: ServerSettingsPatch })",
        Some("ServerSettings"),
        Some("ServerSettingsError"),
        false,
    ),
    schema(
        WsRpcMethod::ServerDiscoverSourceControl,
        "WsServerDiscoverSourceControlRpc",
        "Schema.Struct({})",
        Some("SourceControlDiscoveryResult"),
        None,
        false,
    ),
    schema(
        WsRpcMethod::ServerGetTraceDiagnostics,
        "WsServerGetTraceDiagnosticsRpc",
        "Schema.Struct({})",
        Some("ServerTraceDiagnosticsResult"),
        None,
        false,
    ),
    schema(
        WsRpcMethod::ServerGetProcessDiagnostics,
        "WsServerGetProcessDiagnosticsRpc",
        "Schema.Struct({})",
        Some("ServerProcessDiagnosticsResult"),
        None,
        false,
    ),
    schema(
        WsRpcMethod::ServerSignalProcess,
        "WsServerSignalProcessRpc",
        "ServerSignalProcessInput",
        Some("ServerSignalProcessResult"),
        None,
        false,
    ),
    schema(
        WsRpcMethod::SourceControlLookupRepository,
        "WsSourceControlLookupRepositoryRpc",
        "SourceControlRepositoryLookupInput",
        Some("SourceControlRepositoryInfo"),
        Some("SourceControlRepositoryError"),
        false,
    ),
    schema(
        WsRpcMethod::SourceControlCloneRepository,
        "WsSourceControlCloneRepositoryRpc",
        "SourceControlCloneRepositoryInput",
        Some("SourceControlCloneRepositoryResult"),
        Some("SourceControlRepositoryError"),
        false,
    ),
    schema(
        WsRpcMethod::SourceControlPublishRepository,
        "WsSourceControlPublishRepositoryRpc",
        "SourceControlPublishRepositoryInput",
        Some("SourceControlPublishRepositoryResult"),
        Some("SourceControlRepositoryError"),
        false,
    ),
    schema(
        WsRpcMethod::ProjectsSearchEntries,
        "WsProjectsSearchEntriesRpc",
        "ProjectSearchEntriesInput",
        Some("ProjectSearchEntriesResult"),
        Some("ProjectSearchEntriesError"),
        false,
    ),
    schema(
        WsRpcMethod::ProjectsWriteFile,
        "WsProjectsWriteFileRpc",
        "ProjectWriteFileInput",
        Some("ProjectWriteFileResult"),
        Some("ProjectWriteFileError"),
        false,
    ),
    schema(
        WsRpcMethod::ShellOpenInEditor,
        "WsShellOpenInEditorRpc",
        "OpenInEditorInput",
        None,
        Some("OpenError"),
        false,
    ),
    schema(
        WsRpcMethod::FilesystemBrowse,
        "WsFilesystemBrowseRpc",
        "FilesystemBrowseInput",
        Some("FilesystemBrowseResult"),
        Some("FilesystemBrowseError"),
        false,
    ),
    schema(
        WsRpcMethod::SubscribeVcsStatus,
        "WsSubscribeVcsStatusRpc",
        "VcsStatusInput",
        Some("VcsStatusStreamEvent"),
        Some("GitManagerServiceError"),
        true,
    ),
    schema(
        WsRpcMethod::VcsPull,
        "WsVcsPullRpc",
        "VcsPullInput",
        Some("VcsPullResult"),
        Some("GitCommandError"),
        false,
    ),
    schema(
        WsRpcMethod::VcsRefreshStatus,
        "WsVcsRefreshStatusRpc",
        "VcsStatusInput",
        Some("VcsStatusResult"),
        Some("GitManagerServiceError"),
        false,
    ),
    schema(
        WsRpcMethod::GitRunStackedAction,
        "WsGitRunStackedActionRpc",
        "GitRunStackedActionInput",
        Some("GitActionProgressEvent"),
        Some("GitManagerServiceError"),
        true,
    ),
    schema(
        WsRpcMethod::GitResolvePullRequest,
        "WsGitResolvePullRequestRpc",
        "GitPullRequestRefInput",
        Some("GitResolvePullRequestResult"),
        Some("GitManagerServiceError"),
        false,
    ),
    schema(
        WsRpcMethod::GitPreparePullRequestThread,
        "WsGitPreparePullRequestThreadRpc",
        "GitPreparePullRequestThreadInput",
        Some("GitPreparePullRequestThreadResult"),
        Some("GitManagerServiceError"),
        false,
    ),
    schema(
        WsRpcMethod::VcsListRefs,
        "WsVcsListRefsRpc",
        "VcsListRefsInput",
        Some("VcsListRefsResult"),
        Some("GitCommandError"),
        false,
    ),
    schema(
        WsRpcMethod::VcsCreateWorktree,
        "WsVcsCreateWorktreeRpc",
        "VcsCreateWorktreeInput",
        Some("VcsCreateWorktreeResult"),
        Some("GitCommandError"),
        false,
    ),
    schema(
        WsRpcMethod::VcsRemoveWorktree,
        "WsVcsRemoveWorktreeRpc",
        "VcsRemoveWorktreeInput",
        None,
        Some("GitCommandError"),
        false,
    ),
    schema(
        WsRpcMethod::VcsCreateRef,
        "WsVcsCreateRefRpc",
        "VcsCreateRefInput",
        Some("VcsCreateRefResult"),
        Some("GitCommandError"),
        false,
    ),
    schema(
        WsRpcMethod::VcsSwitchRef,
        "WsVcsSwitchRefRpc",
        "VcsSwitchRefInput",
        Some("VcsSwitchRefResult"),
        Some("GitCommandError"),
        false,
    ),
    schema(
        WsRpcMethod::VcsInit,
        "WsVcsInitRpc",
        "VcsInitInput",
        None,
        Some("VcsError"),
        false,
    ),
    schema(
        WsRpcMethod::TerminalOpen,
        "WsTerminalOpenRpc",
        "TerminalOpenInput",
        Some("TerminalSessionSnapshot"),
        Some("TerminalError"),
        false,
    ),
    schema(
        WsRpcMethod::TerminalWrite,
        "WsTerminalWriteRpc",
        "TerminalWriteInput",
        None,
        Some("TerminalError"),
        false,
    ),
    schema(
        WsRpcMethod::TerminalResize,
        "WsTerminalResizeRpc",
        "TerminalResizeInput",
        None,
        Some("TerminalError"),
        false,
    ),
    schema(
        WsRpcMethod::TerminalClear,
        "WsTerminalClearRpc",
        "TerminalClearInput",
        None,
        Some("TerminalError"),
        false,
    ),
    schema(
        WsRpcMethod::TerminalRestart,
        "WsTerminalRestartRpc",
        "TerminalRestartInput",
        Some("TerminalSessionSnapshot"),
        Some("TerminalError"),
        false,
    ),
    schema(
        WsRpcMethod::TerminalClose,
        "WsTerminalCloseRpc",
        "TerminalCloseInput",
        None,
        Some("TerminalError"),
        false,
    ),
    schema(
        WsRpcMethod::SubscribeTerminalEvents,
        "WsSubscribeTerminalEventsRpc",
        "Schema.Struct({})",
        Some("TerminalEvent"),
        None,
        true,
    ),
    schema(
        WsRpcMethod::SubscribeServerConfig,
        "WsSubscribeServerConfigRpc",
        "Schema.Struct({})",
        Some("ServerConfigStreamEvent"),
        Some("Schema.Union([KeybindingsConfigError, ServerSettingsError])"),
        true,
    ),
    schema(
        WsRpcMethod::SubscribeServerLifecycle,
        "WsSubscribeServerLifecycleRpc",
        "Schema.Struct({})",
        Some("ServerLifecycleStreamEvent"),
        None,
        true,
    ),
    schema(
        WsRpcMethod::SubscribeAuthAccess,
        "WsSubscribeAuthAccessRpc",
        "Schema.Struct({})",
        Some("AuthAccessStreamEvent"),
        None,
        true,
    ),
    schema(
        WsRpcMethod::OrchestrationDispatchCommand,
        "WsOrchestrationDispatchCommandRpc",
        "ClientOrchestrationCommand",
        Some("OrchestrationRpcSchemas.dispatchCommand.output"),
        Some("OrchestrationDispatchCommandError"),
        false,
    ),
    schema(
        WsRpcMethod::OrchestrationGetTurnDiff,
        "WsOrchestrationGetTurnDiffRpc",
        "OrchestrationGetTurnDiffInput",
        Some("OrchestrationRpcSchemas.getTurnDiff.output"),
        Some("OrchestrationGetTurnDiffError"),
        false,
    ),
    schema(
        WsRpcMethod::OrchestrationGetFullThreadDiff,
        "WsOrchestrationGetFullThreadDiffRpc",
        "OrchestrationGetFullThreadDiffInput",
        Some("OrchestrationRpcSchemas.getFullThreadDiff.output"),
        Some("OrchestrationGetFullThreadDiffError"),
        false,
    ),
    schema(
        WsRpcMethod::OrchestrationReplayEvents,
        "WsOrchestrationReplayEventsRpc",
        "OrchestrationReplayEventsInput",
        Some("OrchestrationRpcSchemas.replayEvents.output"),
        Some("OrchestrationReplayEventsError"),
        false,
    ),
    schema(
        WsRpcMethod::OrchestrationGetArchivedShellSnapshot,
        "WsOrchestrationGetArchivedShellSnapshotRpc",
        "OrchestrationRpcSchemas.getArchivedShellSnapshot.input",
        Some("OrchestrationRpcSchemas.getArchivedShellSnapshot.output"),
        Some("OrchestrationGetSnapshotError"),
        false,
    ),
    schema(
        WsRpcMethod::OrchestrationSubscribeShell,
        "WsOrchestrationSubscribeShellRpc",
        "OrchestrationRpcSchemas.subscribeShell.input",
        Some("OrchestrationRpcSchemas.subscribeShell.output"),
        Some("OrchestrationGetSnapshotError"),
        true,
    ),
    schema(
        WsRpcMethod::OrchestrationSubscribeThread,
        "WsOrchestrationSubscribeThreadRpc",
        "OrchestrationRpcSchemas.subscribeThread.input",
        Some("OrchestrationRpcSchemas.subscribeThread.output"),
        Some("OrchestrationGetSnapshotError"),
        true,
    ),
];

const fn schema(
    method: WsRpcMethod,
    rpc_const: &'static str,
    payload_schema: &'static str,
    success_schema: Option<&'static str>,
    error_schema: Option<&'static str>,
    stream: bool,
) -> WsRpcSchemaSpec {
    WsRpcSchemaSpec {
        method,
        rpc_const,
        payload_schema,
        success_schema,
        error_schema,
        stream,
    }
}

pub fn ws_rpc_schema_spec(method: WsRpcMethod) -> Option<WsRpcSchemaSpec> {
    WS_RPC_SCHEMA_SPECS
        .iter()
        .copied()
        .find(|spec| spec.method == method)
}

#[derive(Debug, Clone, PartialEq)]
pub struct WsRpcEnvelope {
    pub id: Option<String>,
    pub method: WsRpcMethod,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsRpcEnvelopeError {
    NotObject,
    MissingMethod,
    UnknownMethod(String),
    InvalidId,
}

pub fn parse_ws_rpc_envelope(value: &Value) -> Result<WsRpcEnvelope, WsRpcEnvelopeError> {
    let object = value.as_object().ok_or(WsRpcEnvelopeError::NotObject)?;
    let method_name = object
        .get("method")
        .and_then(Value::as_str)
        .ok_or(WsRpcEnvelopeError::MissingMethod)?;
    let method = parse_ws_rpc_method(method_name)
        .ok_or_else(|| WsRpcEnvelopeError::UnknownMethod(method_name.to_string()))?;
    let id = match object.get("id") {
        None | Some(Value::Null) => None,
        Some(Value::String(id)) => Some(id.clone()),
        Some(_) => return Err(WsRpcEnvelopeError::InvalidId),
    };
    let payload = object.get("payload").cloned().unwrap_or_else(|| json!({}));
    Ok(WsRpcEnvelope {
        id,
        method,
        payload,
    })
}

pub fn build_ws_rpc_success(method: WsRpcMethod, id: Option<&str>, result: Value) -> Value {
    json!({
        "id": id,
        "method": method.wire_name(),
        "success": true,
        "result": result,
    })
}

pub fn build_ws_rpc_failure(method: WsRpcMethod, id: Option<&str>, message: &str) -> Value {
    json!({
        "id": id,
        "method": method.wire_name(),
        "success": false,
        "error": {
            "message": message,
        },
    })
}

pub const APP_ATOM_REGISTRY_FACTORY: &str = "AtomRegistry.make";
pub const APP_ATOM_REGISTRY_PROVIDER: &str = "RegistryContext.Provider";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppAtomRegistryContract {
    pub exported_registry_binding: &'static str,
    pub registry_factory: &'static str,
    pub provider_component: &'static str,
    pub provider_value_binding: &'static str,
    pub reset_disposes_existing_registry: bool,
    pub reset_recreates_registry: bool,
}

pub fn app_atom_registry_contract() -> AppAtomRegistryContract {
    AppAtomRegistryContract {
        exported_registry_binding: "appAtomRegistry",
        registry_factory: APP_ATOM_REGISTRY_FACTORY,
        provider_component: APP_ATOM_REGISTRY_PROVIDER,
        provider_value_binding: "appAtomRegistry",
        reset_disposes_existing_registry: true,
        reset_recreates_registry: true,
    }
}

pub const SERVER_WELCOME_ATOM_LABEL: &str = "server-welcome";
pub const SERVER_CONFIG_ATOM_LABEL: &str = "server-config";
pub const SERVER_CONFIG_UPDATED_ATOM_LABEL: &str = "server-config-updated";
pub const SERVER_PROVIDERS_UPDATED_ATOM_LABEL: &str = "server-providers-updated";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerConfigUpdateSource {
    Snapshot,
    KeybindingsUpdated,
    ProviderStatuses,
    SettingsUpdated,
}

impl ServerConfigUpdateSource {
    pub fn upstream_name(self) -> &'static str {
        match self {
            Self::Snapshot => "snapshot",
            Self::KeybindingsUpdated => "keybindingsUpdated",
            Self::ProviderStatuses => "providerStatuses",
            Self::SettingsUpdated => "settingsUpdated",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WebServerObservabilityConfig {
    pub logs_directory_path: String,
    pub local_tracing_enabled: bool,
    pub otlp_traces_enabled: bool,
    pub otlp_metrics_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WebServerConfig {
    pub available_editors: Vec<String>,
    pub issues: Vec<KeybindingsConfigIssue>,
    pub keybindings: Vec<ResolvedKeybindingRule>,
    pub keybindings_config_path: Option<String>,
    pub observability: Option<WebServerObservabilityConfig>,
    pub providers: Vec<ServerProvider>,
    pub settings: ServerSettingsForClient,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfigUpdatedPayload {
    pub issues: Vec<KeybindingsConfigIssue>,
    pub providers: Vec<ServerProvider>,
    pub settings: ServerSettingsForClient,
}

pub fn to_server_config_updated_payload(config: &WebServerConfig) -> ServerConfigUpdatedPayload {
    ServerConfigUpdatedPayload {
        issues: config.issues.clone(),
        providers: config.providers.clone(),
        settings: config.settings.clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfigUpdatedNotification {
    pub id: usize,
    pub payload: ServerConfigUpdatedPayload,
    pub source: ServerConfigUpdateSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProviderUpdatedPayload {
    pub providers: Vec<ServerProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLifecycleWelcomePayload {
    pub environment_id: String,
    pub cwd: String,
    pub project_name: String,
    pub bootstrap_project_id: Option<String>,
    pub bootstrap_thread_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerConfigStreamEvent {
    Snapshot {
        config: WebServerConfig,
    },
    KeybindingsUpdated {
        keybindings: Vec<ResolvedKeybindingRule>,
        issues: Vec<KeybindingsConfigIssue>,
    },
    ProviderStatuses {
        providers: Vec<ServerProvider>,
    },
    SettingsUpdated {
        settings: ServerSettingsForClient,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerLifecycleStreamEvent {
    Welcome {
        payload: ServerLifecycleWelcomePayload,
    },
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerState {
    pub welcome: Option<ServerLifecycleWelcomePayload>,
    pub config: Option<WebServerConfig>,
    pub config_updated: Option<ServerConfigUpdatedNotification>,
    pub providers_updated: Option<ServerProviderUpdatedPayload>,
    pub next_config_updated_notification_id: usize,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            welcome: None,
            config: None,
            config_updated: None,
            providers_updated: None,
            next_config_updated_notification_id: 1,
        }
    }
}

pub fn select_server_available_editors(config: Option<&WebServerConfig>) -> Vec<String> {
    config
        .map(|config| config.available_editors.clone())
        .unwrap_or_default()
}

pub fn select_server_keybindings(config: Option<&WebServerConfig>) -> Vec<ResolvedKeybindingRule> {
    config
        .map(|config| config.keybindings.clone())
        .unwrap_or_else(default_resolved_keybindings)
}

pub fn select_server_keybindings_config_path(config: Option<&WebServerConfig>) -> Option<String> {
    config.and_then(|config| config.keybindings_config_path.clone())
}

pub fn select_server_observability(
    config: Option<&WebServerConfig>,
) -> Option<WebServerObservabilityConfig> {
    config.and_then(|config| config.observability.clone())
}

pub fn select_server_providers(config: Option<&WebServerConfig>) -> Vec<ServerProvider> {
    config
        .map(|config| config.providers.clone())
        .unwrap_or_default()
}

pub fn select_server_settings(config: Option<&WebServerConfig>) -> ServerSettingsForClient {
    config
        .map(|config| config.settings.clone())
        .unwrap_or_default()
}

pub fn set_server_config_snapshot(state: &ServerState, config: WebServerConfig) -> ServerState {
    let mut next = state.clone();
    next.config = Some(config.clone());
    emit_providers_updated(
        &mut next,
        ServerProviderUpdatedPayload {
            providers: config.providers.clone(),
        },
    );
    emit_server_config_updated(
        &mut next,
        to_server_config_updated_payload(&config),
        ServerConfigUpdateSource::Snapshot,
    );
    next
}

pub fn apply_server_config_event(
    state: &ServerState,
    event: ServerConfigStreamEvent,
) -> ServerState {
    match event {
        ServerConfigStreamEvent::Snapshot { config } => set_server_config_snapshot(state, config),
        ServerConfigStreamEvent::KeybindingsUpdated {
            keybindings,
            issues,
        } => {
            let Some(current) = state.config.as_ref() else {
                return state.clone();
            };
            let mut next_config = current.clone();
            next_config.keybindings = keybindings;
            next_config.issues = issues;
            let mut next = state.clone();
            next.config = Some(next_config.clone());
            emit_server_config_updated(
                &mut next,
                to_server_config_updated_payload(&next_config),
                ServerConfigUpdateSource::KeybindingsUpdated,
            );
            next
        }
        ServerConfigStreamEvent::ProviderStatuses { providers } => {
            apply_providers_updated(state, ServerProviderUpdatedPayload { providers })
        }
        ServerConfigStreamEvent::SettingsUpdated { settings } => {
            apply_settings_updated(state, settings)
        }
    }
}

pub fn apply_providers_updated(
    state: &ServerState,
    payload: ServerProviderUpdatedPayload,
) -> ServerState {
    let mut next = state.clone();
    emit_providers_updated(&mut next, payload.clone());
    let Some(current) = state.config.as_ref() else {
        return next;
    };
    let mut next_config = current.clone();
    next_config.providers = payload.providers;
    next.config = Some(next_config.clone());
    emit_server_config_updated(
        &mut next,
        to_server_config_updated_payload(&next_config),
        ServerConfigUpdateSource::ProviderStatuses,
    );
    next
}

pub fn apply_settings_updated(
    state: &ServerState,
    settings: ServerSettingsForClient,
) -> ServerState {
    let Some(current) = state.config.as_ref() else {
        return state.clone();
    };
    let mut next_config = current.clone();
    next_config.settings = settings;
    let mut next = state.clone();
    next.config = Some(next_config.clone());
    emit_server_config_updated(
        &mut next,
        to_server_config_updated_payload(&next_config),
        ServerConfigUpdateSource::SettingsUpdated,
    );
    next
}

pub fn apply_server_lifecycle_event(
    state: &ServerState,
    event: ServerLifecycleStreamEvent,
) -> ServerState {
    match event {
        ServerLifecycleStreamEvent::Welcome { payload } => {
            let mut next = state.clone();
            next.welcome = Some(payload);
            next
        }
        ServerLifecycleStreamEvent::Other => state.clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerStateSyncPlan {
    pub subscribe_lifecycle: bool,
    pub subscribe_config: bool,
    pub fetch_config_snapshot: bool,
    pub fallback_fetch_ignored_when_disposed_or_config_present: bool,
    pub cleanup_order: Vec<&'static str>,
}

pub fn start_server_state_sync_plan(state: &ServerState) -> ServerStateSyncPlan {
    ServerStateSyncPlan {
        subscribe_lifecycle: true,
        subscribe_config: true,
        fetch_config_snapshot: state.config.is_none(),
        fallback_fetch_ignored_when_disposed_or_config_present: true,
        cleanup_order: vec!["subscribeLifecycle", "subscribeConfig"],
    }
}

pub fn should_apply_server_config_fallback_fetch(disposed: bool, state: &ServerState) -> bool {
    !disposed && state.config.is_none()
}

pub fn reset_server_state_for_tests() -> ServerState {
    ServerState::default()
}

fn emit_providers_updated(state: &mut ServerState, payload: ServerProviderUpdatedPayload) {
    state.providers_updated = Some(payload);
}

fn emit_server_config_updated(
    state: &mut ServerState,
    payload: ServerConfigUpdatedPayload,
    source: ServerConfigUpdateSource,
) {
    let id = state.next_config_updated_notification_id;
    state.config_updated = Some(ServerConfigUpdatedNotification {
        id,
        payload,
        source,
    });
    state.next_config_updated_notification_id += 1;
}

pub const WS_RPC_PROTOCOL_SOCKET_PATH: &str = "/ws";
pub const WS_RPC_PROTOCOL_SERIALIZATION_LAYER: &str = "RpcSerialization.layerJson";
pub const WS_RPC_PROTOCOL_RETRY_TRANSIENT_ERRORS: bool = true;
pub const WS_RPC_PROTOCOL_CONNECT_ERROR_MESSAGE: &str =
    "Unable to connect to the R3 server WebSocket.";
pub const WS_RPC_PROTOCOL_HEARTBEAT_TIMEOUT_MESSAGE: &str = "WebSocket heartbeat timed out.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsRpcSocketUrlError {
    MissingProtocol,
    MissingAuthority,
    UnsupportedProtocol(String),
}

pub fn format_socket_error_message(message: Option<&str>, fallback: &str) -> String {
    if let Some(message) = message {
        if !message.trim().is_empty() {
            return message.to_string();
        }
    }
    fallback.to_string()
}

pub fn resolve_ws_rpc_socket_url(raw_url: &str) -> Result<String, WsRpcSocketUrlError> {
    let Some(protocol_end) = raw_url.find("://") else {
        return Err(WsRpcSocketUrlError::MissingProtocol);
    };
    let protocol = &raw_url[..protocol_end];
    if protocol != "ws" && protocol != "wss" {
        return Err(WsRpcSocketUrlError::UnsupportedProtocol(format!(
            "{protocol}:"
        )));
    }

    let rest = &raw_url[protocol_end + 3..];
    let authority_end = rest
        .find(|ch| ch == '/' || ch == '?' || ch == '#')
        .unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() {
        return Err(WsRpcSocketUrlError::MissingAuthority);
    }

    let suffix = &rest[authority_end..];
    let query_or_hash = if suffix.starts_with('/') {
        suffix
            .find(|ch| ch == '?' || ch == '#')
            .map(|index| &suffix[index..])
            .unwrap_or("")
    } else {
        suffix
    };

    Ok(format!(
        "{protocol}://{authority}{WS_RPC_PROTOCOL_SOCKET_PATH}{query_or_hash}"
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsRpcProtocolLayerContract {
    pub socket_path: &'static str,
    pub serialization_layer: &'static str,
    pub retry_transient_errors: bool,
    pub max_retries: usize,
    pub retry_delays_ms: Vec<u64>,
}

pub fn ws_rpc_protocol_layer_contract() -> WsRpcProtocolLayerContract {
    WsRpcProtocolLayerContract {
        socket_path: WS_RPC_PROTOCOL_SOCKET_PATH,
        serialization_layer: WS_RPC_PROTOCOL_SERIALIZATION_LAYER,
        retry_transient_errors: WS_RPC_PROTOCOL_RETRY_TRANSIENT_ERRORS,
        max_retries: WS_RECONNECT_MAX_RETRIES,
        retry_delays_ms: (0..WS_RECONNECT_MAX_RETRIES)
            .filter_map(|index| get_ws_reconnect_delay_ms_for_retry(index as i64))
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsRpcProtocolState {
    pub connection_status: WsConnectionStatus,
    pub latency_state: RpcAckLatencyState,
}

impl WsRpcProtocolState {
    pub fn initial(online: bool) -> Self {
        Self {
            connection_status: initial_ws_connection_status(online),
            latency_state: initial_rpc_ack_latency_state(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsRpcProtocolCallback {
    Attempt {
        socket_url: String,
    },
    Open,
    Error {
        message: String,
    },
    Close {
        code: i32,
        reason: String,
        intentional: bool,
    },
    HeartbeatPing,
    HeartbeatPong,
    HeartbeatTimeout,
    RequestStart {
        id: String,
        tag: String,
        stream: bool,
    },
    RequestChunk {
        id: String,
        tag: String,
        chunk_count: usize,
    },
    RequestExit {
        id: String,
        tag: String,
        stream: bool,
    },
    RequestInterrupt {
        id: String,
        tag: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsRpcProtocolResponseTag {
    ClientProtocolError,
    Defect,
    Other(String),
}

impl WsRpcProtocolResponseTag {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "ClientProtocolError" => Self::ClientProtocolError,
            "Defect" => Self::Defect,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn clears_tracked_requests(&self) -> bool {
        matches!(self, Self::ClientProtocolError | Self::Defect)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsRpcProtocolEvent {
    Attempt {
        socket_url: String,
        metadata: Option<WsConnectionMetadata>,
    },
    Open {
        metadata: Option<WsConnectionMetadata>,
        now_iso: String,
    },
    Error {
        message: String,
        metadata: Option<WsConnectionMetadata>,
        now_iso: String,
    },
    Close {
        code: i32,
        reason: String,
        intentional: bool,
        metadata: Option<WsConnectionMetadata>,
        now_iso: String,
    },
    HeartbeatPing,
    HeartbeatPong,
    HeartbeatTimeout {
        metadata: Option<WsConnectionMetadata>,
        now_iso: String,
    },
    RequestStart {
        id: String,
        tag: String,
        stream: bool,
        started_at_ms: i64,
        started_at: String,
    },
    RequestChunk {
        id: String,
        tag: String,
        chunk_count: usize,
    },
    RequestExit {
        id: String,
        tag: String,
        stream: bool,
    },
    RequestInterrupt {
        id: String,
        tag: Option<String>,
    },
    ProtocolResponse {
        tag: WsRpcProtocolResponseTag,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsRpcProtocolEventOutcome {
    pub state: WsRpcProtocolState,
    pub callbacks: Vec<WsRpcProtocolCallback>,
    pub cleared_tracked_requests: bool,
}

pub fn apply_ws_rpc_protocol_event(
    current: &WsRpcProtocolState,
    event: WsRpcProtocolEvent,
    active: bool,
) -> WsRpcProtocolEventOutcome {
    if !active {
        return WsRpcProtocolEventOutcome {
            state: current.clone(),
            callbacks: Vec::new(),
            cleared_tracked_requests: false,
        };
    }

    let mut state = current.clone();
    let mut callbacks = Vec::new();
    let mut cleared_tracked_requests = false;

    match event {
        WsRpcProtocolEvent::Attempt {
            socket_url,
            metadata,
        } => {
            state.connection_status = record_ws_connection_attempt(
                &state.connection_status,
                &socket_url,
                metadata.as_ref(),
            );
            callbacks.push(WsRpcProtocolCallback::Attempt { socket_url });
        }
        WsRpcProtocolEvent::Open { metadata, now_iso } => {
            state.connection_status = record_ws_connection_opened_at(
                &state.connection_status,
                metadata.as_ref(),
                &now_iso,
            );
            callbacks.push(WsRpcProtocolCallback::Open);
        }
        WsRpcProtocolEvent::Error {
            message,
            metadata,
            now_iso,
        } => {
            state.latency_state = clear_all_tracked_rpc_requests(&state.latency_state);
            cleared_tracked_requests = true;
            state.connection_status = record_ws_connection_errored_at(
                &state.connection_status,
                Some(&message),
                metadata.as_ref(),
                &now_iso,
            );
            callbacks.push(WsRpcProtocolCallback::Error { message });
        }
        WsRpcProtocolEvent::Close {
            code,
            reason,
            intentional,
            metadata,
            now_iso,
        } => {
            state.latency_state = clear_all_tracked_rpc_requests(&state.latency_state);
            cleared_tracked_requests = true;
            if !intentional {
                state.connection_status = record_ws_connection_closed_at(
                    &state.connection_status,
                    Some(code),
                    Some(&reason),
                    metadata.as_ref(),
                    &now_iso,
                );
            }
            callbacks.push(WsRpcProtocolCallback::Close {
                code,
                reason,
                intentional,
            });
        }
        WsRpcProtocolEvent::HeartbeatPing => {
            callbacks.push(WsRpcProtocolCallback::HeartbeatPing);
        }
        WsRpcProtocolEvent::HeartbeatPong => {
            callbacks.push(WsRpcProtocolCallback::HeartbeatPong);
        }
        WsRpcProtocolEvent::HeartbeatTimeout { metadata, now_iso } => {
            state.latency_state = clear_all_tracked_rpc_requests(&state.latency_state);
            cleared_tracked_requests = true;
            state.connection_status = record_ws_connection_errored_at(
                &state.connection_status,
                Some(WS_RPC_PROTOCOL_HEARTBEAT_TIMEOUT_MESSAGE),
                metadata.as_ref(),
                &now_iso,
            );
            callbacks.push(WsRpcProtocolCallback::HeartbeatTimeout);
        }
        WsRpcProtocolEvent::RequestStart {
            id,
            tag,
            stream,
            started_at_ms,
            started_at,
        } => {
            callbacks.push(WsRpcProtocolCallback::RequestStart {
                id: id.clone(),
                tag: tag.clone(),
                stream,
            });
            state.latency_state = track_rpc_request_sent_at(
                &state.latency_state,
                &id,
                &tag,
                started_at_ms,
                &started_at,
            );
        }
        WsRpcProtocolEvent::RequestChunk {
            id,
            tag,
            chunk_count,
        } => {
            callbacks.push(WsRpcProtocolCallback::RequestChunk {
                id: id.clone(),
                tag,
                chunk_count,
            });
            state.latency_state = acknowledge_rpc_request(&state.latency_state, &id);
        }
        WsRpcProtocolEvent::RequestExit { id, tag, stream } => {
            callbacks.push(WsRpcProtocolCallback::RequestExit {
                id: id.clone(),
                tag,
                stream,
            });
            state.latency_state = acknowledge_rpc_request(&state.latency_state, &id);
        }
        WsRpcProtocolEvent::RequestInterrupt { id, tag } => {
            callbacks.push(WsRpcProtocolCallback::RequestInterrupt {
                id: id.clone(),
                tag,
            });
            state.latency_state = acknowledge_rpc_request(&state.latency_state, &id);
        }
        WsRpcProtocolEvent::ProtocolResponse { tag } => {
            if tag.clears_tracked_requests() {
                state.latency_state = clear_all_tracked_rpc_requests(&state.latency_state);
                cleared_tracked_requests = true;
            }
        }
    }

    WsRpcProtocolEventOutcome {
        state,
        callbacks,
        cleared_tracked_requests,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsRpcDispatchKind {
    Unary,
    Stream,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsRpcDispatchPlan {
    pub method: WsRpcMethod,
    pub wire_name: &'static str,
    pub aggregate: RpcAggregate,
    pub kind: WsRpcDispatchKind,
    pub handler_group: &'static str,
    pub instrumentation: &'static str,
    pub response_channel: &'static str,
}

pub fn ws_rpc_dispatch_plan(envelope: &WsRpcEnvelope) -> WsRpcDispatchPlan {
    let kind = if envelope.method.is_stream() {
        WsRpcDispatchKind::Stream
    } else {
        WsRpcDispatchKind::Unary
    };
    WsRpcDispatchPlan {
        method: envelope.method,
        wire_name: envelope.method.wire_name(),
        aggregate: envelope.method.aggregate(),
        kind,
        handler_group: envelope.method.aggregate().as_str(),
        instrumentation: match kind {
            WsRpcDispatchKind::Unary => "observeRpcEffect",
            WsRpcDispatchKind::Stream => "observeRpcStreamEffect",
        },
        response_channel: match kind {
            WsRpcDispatchKind::Unary => "single response envelope",
            WsRpcDispatchKind::Stream => "stream response envelopes until completion",
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsRpcObservation {
    Effect,
    Stream,
    StreamEffect,
}

impl WsRpcObservation {
    pub fn upstream_name(self) -> &'static str {
        match self {
            Self::Effect => "observeRpcEffect",
            Self::Stream => "observeRpcStream",
            Self::StreamEffect => "observeRpcStreamEffect",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WsRpcServerHandlerSpec {
    pub method: WsRpcMethod,
    pub observation: WsRpcObservation,
    pub handler: &'static str,
    pub side_effect: Option<&'static str>,
}

pub const WS_RPC_SERVER_HANDLER_COUNT: usize = 47;

pub const WS_RPC_SERVER_HANDLER_SPECS: [WsRpcServerHandlerSpec; WS_RPC_SERVER_HANDLER_COUNT] = [
    handler(
        WsRpcMethod::OrchestrationDispatchCommand,
        WsRpcObservation::Effect,
        "normalizeDispatchCommand -> dispatchNormalizedCommand",
        Some("archive stops active session and closes thread terminals"),
    ),
    handler(
        WsRpcMethod::OrchestrationGetTurnDiff,
        WsRpcObservation::Effect,
        "checkpointDiffQuery.getTurnDiff",
        None,
    ),
    handler(
        WsRpcMethod::OrchestrationGetFullThreadDiff,
        WsRpcObservation::Effect,
        "checkpointDiffQuery.getFullThreadDiff",
        None,
    ),
    handler(
        WsRpcMethod::OrchestrationReplayEvents,
        WsRpcObservation::Effect,
        "orchestrationEngine.readEvents -> enrichOrchestrationEvents",
        None,
    ),
    handler(
        WsRpcMethod::OrchestrationSubscribeShell,
        WsRpcObservation::StreamEffect,
        "projectionSnapshotQuery.getShellSnapshot + orchestrationEngine.streamDomainEvents",
        None,
    ),
    handler(
        WsRpcMethod::OrchestrationGetArchivedShellSnapshot,
        WsRpcObservation::Effect,
        "projectionSnapshotQuery.getArchivedShellSnapshot",
        None,
    ),
    handler(
        WsRpcMethod::OrchestrationSubscribeThread,
        WsRpcObservation::StreamEffect,
        "projectionSnapshotQuery.getThreadDetailById + orchestrationEngine.streamDomainEvents",
        None,
    ),
    handler(
        WsRpcMethod::ServerGetConfig,
        WsRpcObservation::Effect,
        "loadServerConfig",
        None,
    ),
    handler(
        WsRpcMethod::ServerRefreshProviders,
        WsRpcObservation::Effect,
        "providerRegistry.refreshInstance or providerRegistry.refresh",
        None,
    ),
    handler(
        WsRpcMethod::ServerUpdateProvider,
        WsRpcObservation::Effect,
        "providerMaintenanceRunner.updateProvider",
        None,
    ),
    handler(
        WsRpcMethod::ServerUpsertKeybinding,
        WsRpcObservation::Effect,
        "keybindings.upsertKeybindingRule",
        None,
    ),
    handler(
        WsRpcMethod::ServerRemoveKeybinding,
        WsRpcObservation::Effect,
        "keybindings.removeKeybindingRule",
        None,
    ),
    handler(
        WsRpcMethod::ServerGetSettings,
        WsRpcObservation::Effect,
        "serverSettings.getSettings -> redactServerSettingsForClient",
        None,
    ),
    handler(
        WsRpcMethod::ServerUpdateSettings,
        WsRpcObservation::Effect,
        "serverSettings.updateSettings -> redactServerSettingsForClient",
        None,
    ),
    handler(
        WsRpcMethod::ServerDiscoverSourceControl,
        WsRpcObservation::Effect,
        "sourceControlDiscovery.discover",
        None,
    ),
    handler(
        WsRpcMethod::ServerGetTraceDiagnostics,
        WsRpcObservation::Effect,
        "TraceDiagnostics.readTraceDiagnostics",
        None,
    ),
    handler(
        WsRpcMethod::ServerGetProcessDiagnostics,
        WsRpcObservation::Effect,
        "processDiagnostics.read",
        None,
    ),
    handler(
        WsRpcMethod::ServerSignalProcess,
        WsRpcObservation::Effect,
        "processDiagnostics.signal",
        None,
    ),
    handler(
        WsRpcMethod::SourceControlLookupRepository,
        WsRpcObservation::Effect,
        "sourceControlRepositories.lookupRepository",
        None,
    ),
    handler(
        WsRpcMethod::SourceControlCloneRepository,
        WsRpcObservation::Effect,
        "sourceControlRepositories.cloneRepository",
        None,
    ),
    handler(
        WsRpcMethod::SourceControlPublishRepository,
        WsRpcObservation::Effect,
        "sourceControlRepositories.publishRepository",
        Some("refreshGitStatus(input.cwd)"),
    ),
    handler(
        WsRpcMethod::ProjectsSearchEntries,
        WsRpcObservation::Effect,
        "workspaceEntries.search",
        None,
    ),
    handler(
        WsRpcMethod::ProjectsWriteFile,
        WsRpcObservation::Effect,
        "workspaceFileSystem.writeFile",
        None,
    ),
    handler(
        WsRpcMethod::ShellOpenInEditor,
        WsRpcObservation::Effect,
        "open.openInEditor",
        None,
    ),
    handler(
        WsRpcMethod::FilesystemBrowse,
        WsRpcObservation::Effect,
        "workspaceEntries.browse",
        None,
    ),
    handler(
        WsRpcMethod::SubscribeVcsStatus,
        WsRpcObservation::Stream,
        "vcsStatusBroadcaster.streamStatus",
        Some("uses automaticGitFetchInterval"),
    ),
    handler(
        WsRpcMethod::VcsRefreshStatus,
        WsRpcObservation::Effect,
        "vcsStatusBroadcaster.refreshStatus",
        None,
    ),
    handler(
        WsRpcMethod::VcsPull,
        WsRpcObservation::Effect,
        "gitWorkflow.pullCurrentBranch",
        Some("refreshGitStatus(input.cwd)"),
    ),
    handler(
        WsRpcMethod::GitRunStackedAction,
        WsRpcObservation::Stream,
        "gitWorkflow.runStackedAction with progress queue",
        Some("refreshGitStatus(input.cwd) before Queue.end"),
    ),
    handler(
        WsRpcMethod::GitResolvePullRequest,
        WsRpcObservation::Effect,
        "gitWorkflow.resolvePullRequest",
        None,
    ),
    handler(
        WsRpcMethod::GitPreparePullRequestThread,
        WsRpcObservation::Effect,
        "gitWorkflow.preparePullRequestThread",
        Some("refreshGitStatus(input.cwd)"),
    ),
    handler(
        WsRpcMethod::VcsListRefs,
        WsRpcObservation::Effect,
        "gitWorkflow.listRefs",
        None,
    ),
    handler(
        WsRpcMethod::VcsCreateWorktree,
        WsRpcObservation::Effect,
        "gitWorkflow.createWorktree",
        Some("refreshGitStatus(input.cwd)"),
    ),
    handler(
        WsRpcMethod::VcsRemoveWorktree,
        WsRpcObservation::Effect,
        "gitWorkflow.removeWorktree",
        Some("refreshGitStatus(input.cwd)"),
    ),
    handler(
        WsRpcMethod::VcsCreateRef,
        WsRpcObservation::Effect,
        "gitWorkflow.createRef",
        Some("refreshGitStatus(input.cwd)"),
    ),
    handler(
        WsRpcMethod::VcsSwitchRef,
        WsRpcObservation::Effect,
        "gitWorkflow.switchRef",
        Some("refreshGitStatus(input.cwd)"),
    ),
    handler(
        WsRpcMethod::VcsInit,
        WsRpcObservation::Effect,
        "vcsProvisioning.initRepository",
        Some("refreshGitStatus(input.cwd)"),
    ),
    handler(
        WsRpcMethod::TerminalOpen,
        WsRpcObservation::Effect,
        "terminalManager.open",
        None,
    ),
    handler(
        WsRpcMethod::TerminalWrite,
        WsRpcObservation::Effect,
        "terminalManager.write",
        None,
    ),
    handler(
        WsRpcMethod::TerminalResize,
        WsRpcObservation::Effect,
        "terminalManager.resize",
        None,
    ),
    handler(
        WsRpcMethod::TerminalClear,
        WsRpcObservation::Effect,
        "terminalManager.clear",
        None,
    ),
    handler(
        WsRpcMethod::TerminalRestart,
        WsRpcObservation::Effect,
        "terminalManager.restart",
        None,
    ),
    handler(
        WsRpcMethod::TerminalClose,
        WsRpcObservation::Effect,
        "terminalManager.close",
        None,
    ),
    handler(
        WsRpcMethod::SubscribeTerminalEvents,
        WsRpcObservation::Stream,
        "terminalManager.subscribe",
        Some("acquireRelease unsubscribe finalizer"),
    ),
    handler(
        WsRpcMethod::SubscribeServerConfig,
        WsRpcObservation::StreamEffect,
        "loadServerConfig snapshot + keybindings/provider/settings streams",
        Some("providerRegistry.refresh forkScoped and 200ms provider debounce"),
    ),
    handler(
        WsRpcMethod::SubscribeServerLifecycle,
        WsRpcObservation::StreamEffect,
        "lifecycleEvents.snapshot + lifecycleEvents.stream",
        None,
    ),
    handler(
        WsRpcMethod::SubscribeAuthAccess,
        WsRpcObservation::StreamEffect,
        "loadAuthAccessSnapshot + bootstrap/session credential streams",
        Some("revisionRef starts at 1 and increments for live events"),
    ),
];

const fn handler(
    method: WsRpcMethod,
    observation: WsRpcObservation,
    handler: &'static str,
    side_effect: Option<&'static str>,
) -> WsRpcServerHandlerSpec {
    WsRpcServerHandlerSpec {
        method,
        observation,
        handler,
        side_effect,
    }
}

pub fn ws_rpc_server_handler_spec(method: WsRpcMethod) -> Option<WsRpcServerHandlerSpec> {
    WS_RPC_SERVER_HANDLER_SPECS
        .iter()
        .copied()
        .find(|spec| spec.method == method)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpRouteAuth {
    Public,
    Authenticated,
    Owner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpRouteKind {
    StaticOrDev,
    WebSocketRpc,
    Json,
    File,
    Proxy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerHttpRouteSpec {
    pub method: &'static str,
    pub path: &'static str,
    pub kind: HttpRouteKind,
    pub auth: HttpRouteAuth,
}

pub const ATTACHMENTS_ROUTE_PREFIX: &str = "/attachments";

pub const SERVER_HTTP_ROUTE_SPECS: &[ServerHttpRouteSpec] = &[
    http(
        "GET",
        "/ws",
        HttpRouteKind::WebSocketRpc,
        HttpRouteAuth::Authenticated,
    ),
    http(
        "GET",
        "/.well-known/t3/environment",
        HttpRouteKind::Json,
        HttpRouteAuth::Public,
    ),
    http(
        "POST",
        "/api/observability/v1/traces",
        HttpRouteKind::Proxy,
        HttpRouteAuth::Authenticated,
    ),
    http(
        "GET",
        "/attachments/*",
        HttpRouteKind::File,
        HttpRouteAuth::Authenticated,
    ),
    http(
        "GET",
        "/api/project-favicon",
        HttpRouteKind::File,
        HttpRouteAuth::Authenticated,
    ),
    http(
        "GET",
        "/api/orchestration/snapshot",
        HttpRouteKind::Json,
        HttpRouteAuth::Owner,
    ),
    http(
        "POST",
        "/api/orchestration/dispatch",
        HttpRouteKind::Json,
        HttpRouteAuth::Owner,
    ),
    http(
        "GET",
        "/api/auth/session",
        HttpRouteKind::Json,
        HttpRouteAuth::Public,
    ),
    http(
        "POST",
        "/api/auth/bootstrap",
        HttpRouteKind::Json,
        HttpRouteAuth::Public,
    ),
    http(
        "POST",
        "/api/auth/bootstrap/bearer",
        HttpRouteKind::Json,
        HttpRouteAuth::Public,
    ),
    http(
        "POST",
        "/api/auth/ws-token",
        HttpRouteKind::Json,
        HttpRouteAuth::Authenticated,
    ),
    http(
        "POST",
        "/api/auth/pairing-token",
        HttpRouteKind::Json,
        HttpRouteAuth::Owner,
    ),
    http(
        "GET",
        "/api/auth/pairing-links",
        HttpRouteKind::Json,
        HttpRouteAuth::Owner,
    ),
    http(
        "POST",
        "/api/auth/pairing-links/revoke",
        HttpRouteKind::Json,
        HttpRouteAuth::Owner,
    ),
    http(
        "GET",
        "/api/auth/clients",
        HttpRouteKind::Json,
        HttpRouteAuth::Owner,
    ),
    http(
        "POST",
        "/api/auth/clients/revoke",
        HttpRouteKind::Json,
        HttpRouteAuth::Owner,
    ),
    http(
        "POST",
        "/api/auth/clients/revoke-others",
        HttpRouteKind::Json,
        HttpRouteAuth::Owner,
    ),
    http(
        "GET",
        "*",
        HttpRouteKind::StaticOrDev,
        HttpRouteAuth::Public,
    ),
];

const fn http(
    method: &'static str,
    path: &'static str,
    kind: HttpRouteKind,
    auth: HttpRouteAuth,
) -> ServerHttpRouteSpec {
    ServerHttpRouteSpec {
        method,
        path,
        kind,
        auth,
    }
}

pub fn find_server_http_route(method: &str, path: &str) -> Option<ServerHttpRouteSpec> {
    SERVER_HTTP_ROUTE_SPECS
        .iter()
        .copied()
        .find(|route| route.method == method && route.path == path)
}

pub fn normalize_attachment_relative_path(raw_relative_path: &str) -> Option<String> {
    let normalized = raw_relative_path.replace('\\', "/");
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains('\0') {
            return None;
        }
        parts.push(part);
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn ports_app_atom_registry_provider_and_reset_contract() {
        assert_eq!(
            app_atom_registry_contract(),
            AppAtomRegistryContract {
                exported_registry_binding: "appAtomRegistry",
                registry_factory: "AtomRegistry.make",
                provider_component: "RegistryContext.Provider",
                provider_value_binding: "appAtomRegistry",
                reset_disposes_existing_registry: true,
                reset_recreates_registry: true,
            }
        );
    }

    fn rpc_test_provider(instance_id: &str, status: crate::ServerProviderState) -> ServerProvider {
        ServerProvider {
            instance_id: instance_id.to_string(),
            driver: "codex".to_string(),
            display_name: None,
            accent_color: None,
            badge_label: None,
            continuation_group_key: None,
            show_interaction_mode_toggle: false,
            enabled: true,
            installed: true,
            version: Some("0.116.0".to_string()),
            status,
            auth: crate::ServerProviderAuth {
                status: crate::ServerProviderAuthStatus::Authenticated,
                kind: None,
                label: None,
                email: None,
            },
            checked_at: "2026-01-01T00:00:00.000Z".to_string(),
            message: None,
            availability: crate::ServerProviderAvailability::Available,
            unavailable_reason: None,
            models: Vec::new(),
            version_advisory: None,
            update_state: None,
        }
    }

    fn rpc_test_server_config() -> WebServerConfig {
        WebServerConfig {
            available_editors: vec!["cursor".to_string()],
            issues: Vec::new(),
            keybindings: Vec::new(),
            keybindings_config_path: Some("/tmp/workspace/.config/keybindings.json".to_string()),
            observability: Some(WebServerObservabilityConfig {
                logs_directory_path: "/tmp/workspace/.config/logs".to_string(),
                local_tracing_enabled: true,
                otlp_traces_enabled: false,
                otlp_metrics_enabled: false,
            }),
            providers: vec![rpc_test_provider(
                "codex",
                crate::ServerProviderState::Ready,
            )],
            settings: ServerSettingsForClient::default(),
        }
    }

    #[test]
    fn server_state_defaults_and_sync_plan_match_upstream_contract() {
        let state = reset_server_state_for_tests();

        assert_eq!(
            select_server_available_editors(state.config.as_ref()),
            Vec::<String>::new()
        );
        assert_eq!(
            select_server_keybindings(state.config.as_ref()),
            default_resolved_keybindings()
        );
        assert_eq!(
            select_server_keybindings_config_path(state.config.as_ref()),
            None
        );
        assert_eq!(select_server_observability(state.config.as_ref()), None);
        assert_eq!(
            select_server_providers(state.config.as_ref()),
            Vec::<ServerProvider>::new()
        );
        assert_eq!(
            select_server_settings(state.config.as_ref()),
            ServerSettingsForClient::default()
        );

        assert_eq!(
            start_server_state_sync_plan(&state),
            ServerStateSyncPlan {
                subscribe_lifecycle: true,
                subscribe_config: true,
                fetch_config_snapshot: true,
                fallback_fetch_ignored_when_disposed_or_config_present: true,
                cleanup_order: vec!["subscribeLifecycle", "subscribeConfig"],
            }
        );
        assert!(should_apply_server_config_fallback_fetch(false, &state));
        assert!(!should_apply_server_config_fallback_fetch(true, &state));
    }

    #[test]
    fn server_state_snapshot_notifications_replay_latest_values() {
        let state = reset_server_state_for_tests();
        let config = rpc_test_server_config();
        let state = set_server_config_snapshot(&state, config.clone());

        assert_eq!(state.config, Some(config.clone()));
        assert_eq!(
            state.providers_updated,
            Some(ServerProviderUpdatedPayload {
                providers: config.providers.clone(),
            })
        );
        assert_eq!(
            state.config_updated,
            Some(ServerConfigUpdatedNotification {
                id: 1,
                payload: ServerConfigUpdatedPayload {
                    issues: Vec::new(),
                    providers: config.providers.clone(),
                    settings: ServerSettingsForClient::default(),
                },
                source: ServerConfigUpdateSource::Snapshot,
            })
        );
        assert_eq!(state.next_config_updated_notification_id, 2);
        assert!(!should_apply_server_config_fallback_fetch(false, &state));
        assert!(!start_server_state_sync_plan(&state).fetch_config_snapshot);
        assert_eq!(
            select_server_available_editors(state.config.as_ref()),
            vec!["cursor".to_string()]
        );
        assert_eq!(
            select_server_keybindings_config_path(state.config.as_ref()).as_deref(),
            Some("/tmp/workspace/.config/keybindings.json")
        );
    }

    #[test]
    fn server_state_merges_keybinding_provider_and_settings_updates() {
        let mut state =
            set_server_config_snapshot(&ServerState::default(), rpc_test_server_config());
        let next_keybindings = default_resolved_keybindings()
            .into_iter()
            .take(1)
            .collect::<Vec<_>>();
        let next_issue = KeybindingsConfigIssue {
            kind: "keybindings.malformed-config".to_string(),
            index: None,
            message: "bad json".to_string(),
        };

        state = apply_server_config_event(
            &state,
            ServerConfigStreamEvent::KeybindingsUpdated {
                keybindings: next_keybindings.clone(),
                issues: vec![next_issue.clone()],
            },
        );
        assert_eq!(state.config.as_ref().unwrap().keybindings, next_keybindings);
        assert_eq!(
            state.config.as_ref().unwrap().issues,
            vec![next_issue.clone()]
        );
        assert_eq!(
            state.config_updated.as_ref().unwrap().source,
            ServerConfigUpdateSource::KeybindingsUpdated
        );
        assert_eq!(state.config_updated.as_ref().unwrap().id, 2);

        let warning_provider = rpc_test_provider("codex", crate::ServerProviderState::Warning);
        state = apply_server_config_event(
            &state,
            ServerConfigStreamEvent::ProviderStatuses {
                providers: vec![warning_provider.clone()],
            },
        );
        assert_eq!(
            state.providers_updated,
            Some(ServerProviderUpdatedPayload {
                providers: vec![warning_provider.clone()],
            })
        );
        assert_eq!(
            state.config.as_ref().unwrap().providers,
            vec![warning_provider.clone()]
        );
        assert_eq!(
            state.config_updated.as_ref().unwrap().source,
            ServerConfigUpdateSource::ProviderStatuses
        );
        assert_eq!(state.config_updated.as_ref().unwrap().id, 3);

        let mut next_settings = ServerSettingsForClient::default();
        next_settings.provider_instances.insert(
            "codex".to_string(),
            crate::server::ProviderInstanceConfig::default(),
        );
        state = apply_server_config_event(
            &state,
            ServerConfigStreamEvent::SettingsUpdated {
                settings: next_settings.clone(),
            },
        );
        assert_eq!(
            state.config.as_ref().unwrap().settings,
            next_settings.clone()
        );
        assert_eq!(
            state.config_updated.as_ref().unwrap().payload,
            ServerConfigUpdatedPayload {
                issues: vec![next_issue],
                providers: vec![warning_provider],
                settings: next_settings,
            }
        );
        assert_eq!(
            state.config_updated.as_ref().unwrap().source,
            ServerConfigUpdateSource::SettingsUpdated
        );
        assert_eq!(state.config_updated.as_ref().unwrap().id, 4);
    }

    #[test]
    fn server_state_ignores_updates_without_snapshot_and_replays_welcome() {
        let state = reset_server_state_for_tests();
        let providers_only = apply_server_config_event(
            &state,
            ServerConfigStreamEvent::ProviderStatuses {
                providers: vec![rpc_test_provider(
                    "codex",
                    crate::ServerProviderState::Ready,
                )],
            },
        );
        assert!(providers_only.config.is_none());
        assert!(providers_only.providers_updated.is_some());
        assert!(providers_only.config_updated.is_none());

        let ignored_keybindings = apply_server_config_event(
            &state,
            ServerConfigStreamEvent::KeybindingsUpdated {
                keybindings: Vec::new(),
                issues: Vec::new(),
            },
        );
        assert_eq!(ignored_keybindings, state);

        let welcome = ServerLifecycleWelcomePayload {
            environment_id: "environment-local".to_string(),
            cwd: "/tmp/workspace".to_string(),
            project_name: "r3-code".to_string(),
            bootstrap_project_id: Some("project-1".to_string()),
            bootstrap_thread_id: Some("thread-1".to_string()),
        };
        let welcomed = apply_server_lifecycle_event(
            &state,
            ServerLifecycleStreamEvent::Welcome {
                payload: welcome.clone(),
            },
        );
        assert_eq!(welcomed.welcome, Some(welcome));
        assert_eq!(
            apply_server_lifecycle_event(&welcomed, ServerLifecycleStreamEvent::Other),
            welcomed
        );
    }

    #[test]
    fn ports_ws_rpc_protocol_url_and_layer_contracts() {
        assert_eq!(
            resolve_ws_rpc_socket_url("ws://localhost:3020").unwrap(),
            "ws://localhost:3020/ws"
        );
        assert_eq!(
            resolve_ws_rpc_socket_url("wss://example.test/api/socket?token=1#frag").unwrap(),
            "wss://example.test/ws?token=1#frag"
        );
        assert_eq!(
            resolve_ws_rpc_socket_url("http://localhost:3020"),
            Err(WsRpcSocketUrlError::UnsupportedProtocol(
                "http:".to_string()
            ))
        );
        assert_eq!(
            resolve_ws_rpc_socket_url("ws:///missing-host"),
            Err(WsRpcSocketUrlError::MissingAuthority)
        );

        assert_eq!(
            format_socket_error_message(Some("  connect failed  "), "fallback"),
            "  connect failed  "
        );
        assert_eq!(
            format_socket_error_message(Some("   "), "fallback"),
            "fallback"
        );

        assert_eq!(
            ws_rpc_protocol_layer_contract(),
            WsRpcProtocolLayerContract {
                socket_path: "/ws",
                serialization_layer: "RpcSerialization.layerJson",
                retry_transient_errors: true,
                max_retries: 7,
                retry_delays_ms: vec![1_000, 2_000, 4_000, 8_000, 16_000, 32_000, 64_000],
            }
        );
    }

    #[test]
    fn ws_rpc_protocol_lifecycle_effects_match_upstream_hooks() {
        let metadata = WsConnectionMetadata {
            connection_label: Some(" Remote ".to_string()),
            version_mismatch_hint: Some(" upgrade client ".to_string()),
        };
        let mut state = WsRpcProtocolState::initial(true);

        let attempted = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::Attempt {
                socket_url: "ws://localhost:3020/ws".to_string(),
                metadata: Some(metadata.clone()),
            },
            true,
        );
        assert_eq!(
            attempted.callbacks,
            vec![WsRpcProtocolCallback::Attempt {
                socket_url: "ws://localhost:3020/ws".to_string(),
            }]
        );
        assert_eq!(
            attempted.state.connection_status.socket_url.as_deref(),
            Some("ws://localhost:3020/ws")
        );
        assert_eq!(
            attempted
                .state
                .connection_status
                .connection_label
                .as_deref(),
            Some("Remote")
        );
        assert_eq!(attempted.state.connection_status.reconnect_attempt_count, 1);
        state = attempted.state;

        let opened = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::Open {
                metadata: Some(metadata.clone()),
                now_iso: "2026-05-13T07:00:00.000Z".to_string(),
            },
            true,
        );
        assert_eq!(opened.callbacks, vec![WsRpcProtocolCallback::Open]);
        assert!(opened.state.connection_status.has_connected);
        assert_eq!(
            opened.state.connection_status.connected_at.as_deref(),
            Some("2026-05-13T07:00:00.000Z")
        );
        state = opened.state;

        let request_started = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::RequestStart {
                id: "rpc-1".to_string(),
                tag: "server.getConfig".to_string(),
                stream: false,
                started_at_ms: 100,
                started_at: "2026-05-13T07:00:01.000Z".to_string(),
            },
            true,
        );
        assert_eq!(
            request_started.callbacks,
            vec![WsRpcProtocolCallback::RequestStart {
                id: "rpc-1".to_string(),
                tag: "server.getConfig".to_string(),
                stream: false,
            }]
        );
        assert_eq!(
            request_started.state.latency_state.pending_requests.len(),
            1
        );
        state = request_started.state;

        let chunk = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::RequestChunk {
                id: "rpc-1".to_string(),
                tag: "server.getConfig".to_string(),
                chunk_count: 1,
            },
            true,
        );
        assert_eq!(chunk.state.latency_state.pending_requests, Vec::new());
        state = chunk.state;

        let request_started = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::RequestStart {
                id: "rpc-2".to_string(),
                tag: "server.getSettings".to_string(),
                stream: false,
                started_at_ms: 200,
                started_at: "2026-05-13T07:00:02.000Z".to_string(),
            },
            true,
        );
        assert_eq!(
            request_started.state.latency_state.pending_requests.len(),
            1
        );
        state = request_started.state;

        let errored = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::Error {
                message: "connect failed".to_string(),
                metadata: Some(metadata),
                now_iso: "2026-05-13T07:00:03.000Z".to_string(),
            },
            true,
        );
        assert!(errored.cleared_tracked_requests);
        assert_eq!(errored.state.latency_state.pending_requests, Vec::new());
        assert_eq!(
            errored.state.connection_status.last_error.as_deref(),
            Some("connect failed Hint: upgrade client")
        );
        assert_eq!(
            errored.callbacks,
            vec![WsRpcProtocolCallback::Error {
                message: "connect failed".to_string(),
            }]
        );

        let inactive = apply_ws_rpc_protocol_event(
            &errored.state,
            WsRpcProtocolEvent::RequestStart {
                id: "rpc-inactive".to_string(),
                tag: "server.getConfig".to_string(),
                stream: false,
                started_at_ms: 300,
                started_at: "2026-05-13T07:00:04.000Z".to_string(),
            },
            false,
        );
        assert_eq!(inactive.state, errored.state);
        assert_eq!(inactive.callbacks, Vec::new());
    }

    #[test]
    fn ws_rpc_protocol_cleanup_matches_upstream_error_and_heartbeat_paths() {
        let mut state = WsRpcProtocolState::initial(true);
        state = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::RequestStart {
                id: "rpc-1".to_string(),
                tag: "subscribeServerConfig".to_string(),
                stream: true,
                started_at_ms: 100,
                started_at: "2026-05-13T07:00:01.000Z".to_string(),
            },
            true,
        )
        .state;
        assert_eq!(state.latency_state.pending_requests, Vec::new());

        state = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::RequestStart {
                id: "rpc-2".to_string(),
                tag: "server.getConfig".to_string(),
                stream: false,
                started_at_ms: 200,
                started_at: "2026-05-13T07:00:02.000Z".to_string(),
            },
            true,
        )
        .state;
        assert_eq!(state.latency_state.pending_requests.len(), 1);

        let ignored_response = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::ProtocolResponse {
                tag: WsRpcProtocolResponseTag::from_tag("Chunk"),
            },
            true,
        );
        assert!(!ignored_response.cleared_tracked_requests);
        assert_eq!(
            ignored_response.state.latency_state.pending_requests.len(),
            1
        );

        let defect = apply_ws_rpc_protocol_event(
            &ignored_response.state,
            WsRpcProtocolEvent::ProtocolResponse {
                tag: WsRpcProtocolResponseTag::from_tag("Defect"),
            },
            true,
        );
        assert!(defect.cleared_tracked_requests);
        assert_eq!(defect.state.latency_state.pending_requests, Vec::new());
        state = defect.state;

        state = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::RequestStart {
                id: "rpc-3".to_string(),
                tag: "server.getSettings".to_string(),
                stream: false,
                started_at_ms: 300,
                started_at: "2026-05-13T07:00:03.000Z".to_string(),
            },
            true,
        )
        .state;

        let timeout = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::HeartbeatTimeout {
                metadata: None,
                now_iso: "2026-05-13T07:00:04.000Z".to_string(),
            },
            true,
        );
        assert!(timeout.cleared_tracked_requests);
        assert_eq!(
            timeout.state.connection_status.last_error.as_deref(),
            Some("WebSocket heartbeat timed out.")
        );
        assert_eq!(
            timeout.callbacks,
            vec![WsRpcProtocolCallback::HeartbeatTimeout]
        );

        state = apply_ws_rpc_protocol_event(
            &timeout.state,
            WsRpcProtocolEvent::RequestStart {
                id: "rpc-4".to_string(),
                tag: "server.getConfig".to_string(),
                stream: false,
                started_at_ms: 400,
                started_at: "2026-05-13T07:00:05.000Z".to_string(),
            },
            true,
        )
        .state;
        let intentional_close = apply_ws_rpc_protocol_event(
            &state,
            WsRpcProtocolEvent::Close {
                code: 1000,
                reason: "done".to_string(),
                intentional: true,
                metadata: None,
                now_iso: "2026-05-13T07:00:06.000Z".to_string(),
            },
            true,
        );
        assert!(intentional_close.cleared_tracked_requests);
        assert_eq!(
            intentional_close.state.connection_status.close_code,
            timeout.state.connection_status.close_code
        );
        assert_eq!(
            intentional_close.callbacks,
            vec![WsRpcProtocolCallback::Close {
                code: 1000,
                reason: "done".to_string(),
                intentional: true,
            }]
        );
    }

    #[test]
    fn ports_upstream_ws_rpc_method_names_exactly() {
        let wire_names = WS_RPC_METHOD_SPECS
            .iter()
            .map(|spec| spec.wire_name)
            .collect::<Vec<_>>();

        assert_eq!(wire_names.len(), WS_RPC_METHOD_COUNT);
        assert_eq!(
            wire_names,
            vec![
                "projects.list",
                "projects.add",
                "projects.remove",
                "projects.searchEntries",
                "projects.writeFile",
                "shell.openInEditor",
                "filesystem.browse",
                "vcs.pull",
                "vcs.refreshStatus",
                "vcs.listRefs",
                "vcs.createWorktree",
                "vcs.removeWorktree",
                "vcs.createRef",
                "vcs.switchRef",
                "vcs.init",
                "git.runStackedAction",
                "git.resolvePullRequest",
                "git.preparePullRequestThread",
                "terminal.open",
                "terminal.write",
                "terminal.resize",
                "terminal.clear",
                "terminal.restart",
                "terminal.close",
                "server.getConfig",
                "server.refreshProviders",
                "server.updateProvider",
                "server.upsertKeybinding",
                "server.removeKeybinding",
                "server.getSettings",
                "server.updateSettings",
                "server.discoverSourceControl",
                "server.getTraceDiagnostics",
                "server.getProcessDiagnostics",
                "server.signalProcess",
                "sourceControl.lookupRepository",
                "sourceControl.cloneRepository",
                "sourceControl.publishRepository",
                "subscribeVcsStatus",
                "subscribeTerminalEvents",
                "subscribeServerConfig",
                "subscribeServerLifecycle",
                "subscribeAuthAccess",
                "orchestration.dispatchCommand",
                "orchestration.getTurnDiff",
                "orchestration.getFullThreadDiff",
                "orchestration.replayEvents",
                "orchestration.getArchivedShellSnapshot",
                "orchestration.subscribeShell",
                "orchestration.subscribeThread",
            ]
        );
        assert_eq!(
            wire_names.iter().copied().collect::<BTreeSet<_>>().len(),
            wire_names.len()
        );
    }

    #[test]
    fn classifies_upstream_streaming_methods() {
        let stream_names = ws_rpc_stream_methods()
            .iter()
            .map(|method| method.wire_name())
            .collect::<Vec<_>>();

        assert_eq!(
            stream_names,
            vec![
                "git.runStackedAction",
                "subscribeVcsStatus",
                "subscribeTerminalEvents",
                "subscribeServerConfig",
                "subscribeServerLifecycle",
                "subscribeAuthAccess",
                "orchestration.subscribeShell",
                "orchestration.subscribeThread",
            ]
        );
        assert!(WsRpcMethod::SubscribeServerConfig.is_stream());
        assert!(!WsRpcMethod::ServerGetConfig.is_stream());
    }

    #[test]
    fn ports_upstream_ws_rpc_group_schema_contracts() {
        let group_names = WS_RPC_GROUP_METHODS
            .iter()
            .map(|method| method.wire_name())
            .collect::<Vec<_>>();

        assert_eq!(group_names.len(), WS_RPC_GROUP_METHOD_COUNT);
        assert_eq!(
            group_names,
            vec![
                "server.getConfig",
                "server.refreshProviders",
                "server.updateProvider",
                "server.upsertKeybinding",
                "server.removeKeybinding",
                "server.getSettings",
                "server.updateSettings",
                "server.discoverSourceControl",
                "server.getTraceDiagnostics",
                "server.getProcessDiagnostics",
                "server.signalProcess",
                "sourceControl.lookupRepository",
                "sourceControl.cloneRepository",
                "sourceControl.publishRepository",
                "projects.searchEntries",
                "projects.writeFile",
                "shell.openInEditor",
                "filesystem.browse",
                "subscribeVcsStatus",
                "vcs.pull",
                "vcs.refreshStatus",
                "git.runStackedAction",
                "git.resolvePullRequest",
                "git.preparePullRequestThread",
                "vcs.listRefs",
                "vcs.createWorktree",
                "vcs.removeWorktree",
                "vcs.createRef",
                "vcs.switchRef",
                "vcs.init",
                "terminal.open",
                "terminal.write",
                "terminal.resize",
                "terminal.clear",
                "terminal.restart",
                "terminal.close",
                "subscribeTerminalEvents",
                "subscribeServerConfig",
                "subscribeServerLifecycle",
                "subscribeAuthAccess",
                "orchestration.dispatchCommand",
                "orchestration.getTurnDiff",
                "orchestration.getFullThreadDiff",
                "orchestration.replayEvents",
                "orchestration.getArchivedShellSnapshot",
                "orchestration.subscribeShell",
                "orchestration.subscribeThread",
            ]
        );
        assert_eq!(
            group_names.iter().copied().collect::<BTreeSet<_>>().len(),
            group_names.len()
        );
        assert_eq!(
            WS_RPC_SCHEMA_SPECS
                .iter()
                .map(|spec| spec.method)
                .collect::<Vec<_>>(),
            WS_RPC_GROUP_METHODS
        );
        assert!(ws_rpc_schema_spec(WsRpcMethod::ProjectsList).is_none());
        assert!(ws_rpc_schema_spec(WsRpcMethod::ProjectsAdd).is_none());
        assert!(ws_rpc_schema_spec(WsRpcMethod::ProjectsRemove).is_none());

        let terminal_resize = ws_rpc_schema_spec(WsRpcMethod::TerminalResize).unwrap();
        assert_eq!(terminal_resize.rpc_const, "WsTerminalResizeRpc");
        assert_eq!(terminal_resize.payload_schema, "TerminalResizeInput");
        assert_eq!(terminal_resize.success_schema, None);
        assert_eq!(terminal_resize.error_schema, Some("TerminalError"));
        assert!(!terminal_resize.stream);

        let server_config = ws_rpc_schema_spec(WsRpcMethod::ServerGetConfig).unwrap();
        assert_eq!(server_config.payload_schema, "Schema.Struct({})");
        assert_eq!(server_config.success_schema, Some("ServerConfig"));
        assert_eq!(
            server_config.error_schema,
            Some("Schema.Union([KeybindingsConfigError, ServerSettingsError])")
        );

        let subscription = ws_rpc_schema_spec(WsRpcMethod::SubscribeTerminalEvents).unwrap();
        assert_eq!(subscription.payload_schema, "Schema.Struct({})");
        assert_eq!(subscription.success_schema, Some("TerminalEvent"));
        assert_eq!(subscription.error_schema, None);
        assert!(subscription.stream);
        assert!(
            WS_RPC_SCHEMA_SPECS
                .iter()
                .all(|spec| spec.stream == spec.method.is_stream())
        );
    }

    #[test]
    fn ports_upstream_ws_rpc_server_handler_map() {
        let handler_names = WS_RPC_SERVER_HANDLER_SPECS
            .iter()
            .map(|spec| spec.method.wire_name())
            .collect::<Vec<_>>();

        assert_eq!(handler_names.len(), WS_RPC_SERVER_HANDLER_COUNT);
        assert_eq!(
            handler_names,
            vec![
                "orchestration.dispatchCommand",
                "orchestration.getTurnDiff",
                "orchestration.getFullThreadDiff",
                "orchestration.replayEvents",
                "orchestration.subscribeShell",
                "orchestration.getArchivedShellSnapshot",
                "orchestration.subscribeThread",
                "server.getConfig",
                "server.refreshProviders",
                "server.updateProvider",
                "server.upsertKeybinding",
                "server.removeKeybinding",
                "server.getSettings",
                "server.updateSettings",
                "server.discoverSourceControl",
                "server.getTraceDiagnostics",
                "server.getProcessDiagnostics",
                "server.signalProcess",
                "sourceControl.lookupRepository",
                "sourceControl.cloneRepository",
                "sourceControl.publishRepository",
                "projects.searchEntries",
                "projects.writeFile",
                "shell.openInEditor",
                "filesystem.browse",
                "subscribeVcsStatus",
                "vcs.refreshStatus",
                "vcs.pull",
                "git.runStackedAction",
                "git.resolvePullRequest",
                "git.preparePullRequestThread",
                "vcs.listRefs",
                "vcs.createWorktree",
                "vcs.removeWorktree",
                "vcs.createRef",
                "vcs.switchRef",
                "vcs.init",
                "terminal.open",
                "terminal.write",
                "terminal.resize",
                "terminal.clear",
                "terminal.restart",
                "terminal.close",
                "subscribeTerminalEvents",
                "subscribeServerConfig",
                "subscribeServerLifecycle",
                "subscribeAuthAccess",
            ]
        );
        assert_eq!(
            handler_names.iter().copied().collect::<BTreeSet<_>>().len(),
            handler_names.len()
        );
        assert!(
            WS_RPC_GROUP_METHODS
                .iter()
                .all(|method| ws_rpc_server_handler_spec(*method).is_some())
        );
        assert!(ws_rpc_server_handler_spec(WsRpcMethod::ProjectsList).is_none());

        let dispatch =
            ws_rpc_server_handler_spec(WsRpcMethod::OrchestrationDispatchCommand).unwrap();
        assert_eq!(dispatch.observation, WsRpcObservation::Effect);
        assert_eq!(
            dispatch.handler,
            "normalizeDispatchCommand -> dispatchNormalizedCommand"
        );
        assert_eq!(
            dispatch.side_effect,
            Some("archive stops active session and closes thread terminals")
        );

        let stacked = ws_rpc_server_handler_spec(WsRpcMethod::GitRunStackedAction).unwrap();
        assert_eq!(stacked.observation.upstream_name(), "observeRpcStream");
        assert_eq!(
            stacked.handler,
            "gitWorkflow.runStackedAction with progress queue"
        );
        assert_eq!(
            stacked.side_effect,
            Some("refreshGitStatus(input.cwd) before Queue.end")
        );

        let config = ws_rpc_server_handler_spec(WsRpcMethod::SubscribeServerConfig).unwrap();
        assert_eq!(config.observation, WsRpcObservation::StreamEffect);
        assert_eq!(
            config.side_effect,
            Some("providerRegistry.refresh forkScoped and 200ms provider debounce")
        );
    }

    #[test]
    fn parses_and_serializes_ws_rpc_envelopes() {
        let envelope = parse_ws_rpc_envelope(&json!({
            "id": "rpc-1",
            "method": "terminal.resize",
            "payload": {
                "terminalId": "terminal-1",
                "cols": 120,
                "rows": 40
            }
        }))
        .unwrap();

        assert_eq!(envelope.id.as_deref(), Some("rpc-1"));
        assert_eq!(envelope.method, WsRpcMethod::TerminalResize);
        assert_eq!(envelope.method.aggregate(), RpcAggregate::Terminal);
        assert_eq!(envelope.payload["cols"], 120);

        assert_eq!(
            build_ws_rpc_success(envelope.method, envelope.id.as_deref(), json!({"ok": true})),
            json!({
                "id": "rpc-1",
                "method": "terminal.resize",
                "success": true,
                "result": {"ok": true},
            })
        );

        assert_eq!(
            ws_rpc_dispatch_plan(&envelope),
            WsRpcDispatchPlan {
                method: WsRpcMethod::TerminalResize,
                wire_name: "terminal.resize",
                aggregate: RpcAggregate::Terminal,
                kind: WsRpcDispatchKind::Unary,
                handler_group: "terminal",
                instrumentation: "observeRpcEffect",
                response_channel: "single response envelope",
            }
        );

        let stream_envelope = parse_ws_rpc_envelope(&json!({
            "method": "subscribeTerminalEvents",
            "payload": { "terminalId": "terminal-1" }
        }))
        .unwrap();
        assert_eq!(
            ws_rpc_dispatch_plan(&stream_envelope),
            WsRpcDispatchPlan {
                method: WsRpcMethod::SubscribeTerminalEvents,
                wire_name: "subscribeTerminalEvents",
                aggregate: RpcAggregate::Terminal,
                kind: WsRpcDispatchKind::Stream,
                handler_group: "terminal",
                instrumentation: "observeRpcStreamEffect",
                response_channel: "stream response envelopes until completion",
            }
        );
    }

    #[test]
    fn ports_server_http_route_table() {
        assert_eq!(
            find_server_http_route("GET", "/ws"),
            Some(ServerHttpRouteSpec {
                method: "GET",
                path: "/ws",
                kind: HttpRouteKind::WebSocketRpc,
                auth: HttpRouteAuth::Authenticated,
            })
        );
        assert_eq!(
            find_server_http_route("POST", "/api/auth/pairing-token")
                .unwrap()
                .auth,
            HttpRouteAuth::Owner
        );
        assert_eq!(
            find_server_http_route("GET", "/.well-known/t3/environment")
                .unwrap()
                .auth,
            HttpRouteAuth::Public
        );
        assert_eq!(ATTACHMENTS_ROUTE_PREFIX, "/attachments");
    }

    #[test]
    fn normalizes_attachment_paths_like_upstream_route_guard() {
        assert_eq!(
            normalize_attachment_relative_path("/thread-1\\image.png").as_deref(),
            Some("thread-1/image.png")
        );
        assert_eq!(normalize_attachment_relative_path("../secret.png"), None);
        assert_eq!(normalize_attachment_relative_path(""), None);
        assert_eq!(
            normalize_attachment_relative_path("safe/../secret.png"),
            None
        );
    }
}
