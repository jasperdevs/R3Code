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
