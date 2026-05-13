use std::{error::Error, fmt, path::Path};

use rusqlite::{Connection, Row, Rows, params};
use serde_json::{Value, json};

use crate::{
    ActivityPayload, ActivityTone, ApprovalRequestKind, ChatAttachment, ChatImageAttachment,
    ChatMessage, EnvironmentState, MessageRole, ProjectScript, ProjectScriptIcon,
    ProjectionLatestTurnRow, ProjectionProjectRow, ProjectionShellSnapshotInput,
    ProjectionThreadRow, ProjectionThreadSessionRow, ProposedPlan, ProviderInteractionMode,
    RuntimeMode, Thread, ThreadActivity, TurnDiffFileChange, TurnDiffSummary, UserInputQuestion,
    UserInputQuestionOption, derive_pending_user_inputs, get_thread_from_environment_state,
    orchestration::{
        OrchestrationCommand, OrchestrationCommandInvariantError, OrchestrationProposedPlanRef,
        OrchestrationReadModel, PlannedOrchestrationEvent, ProviderCommandIntent,
        ProviderRuntimeBinding, ProviderRuntimeEventInput,
        ProviderRuntimeIngestionCommandPlanContext, ProviderRuntimeIngestionQueue,
        ProviderServiceRequest, ThreadDeletionCleanupAction, ThreadDeletionCleanupRequest,
        decide_orchestration_command, provider_command_intent_for_event,
        provider_runtime_event_to_orchestration_commands, provider_service_request_for_intent,
        thread_deletion_cleanup_actions_for_event, thread_deletion_cleanup_requests,
    },
};

pub const PROJECTION_SQLITE_MIGRATION_NAMES: &[&str] = &[
    "001_OrchestrationEvents",
    "002_OrchestrationCommandReceipts",
    "003_CheckpointDiffBlobs",
    "004_ProviderSessionRuntime",
    "005_Projections",
    "006_ProjectionThreadSessionRuntimeModeColumns",
    "007_ProjectionThreadMessageAttachments",
    "008_ProjectionThreadActivitySequence",
    "009_ProviderSessionRuntimeMode",
    "010_ProjectionThreadsRuntimeMode",
    "011_OrchestrationThreadCreatedRuntimeMode",
    "012_ProjectionThreadsInteractionMode",
    "013_ProjectionThreadProposedPlans",
    "014_ProjectionThreadProposedPlanImplementation",
    "015_ProjectionTurnsSourceProposedPlan",
    "016_CanonicalizeModelSelections",
    "017_ProjectionThreadsArchivedAt",
    "018_ProjectionThreadsArchivedAtIndex",
    "019_ProjectionSnapshotLookupIndexes",
    "020_AuthAccessManagement",
    "021_AuthSessionClientMetadata",
    "022_AuthSessionLastConnectedAt",
    "023_ProjectionThreadShellSummary",
    "024_BackfillProjectionThreadShellSummary",
    "025_CleanupInvalidProjectionPendingApprovals",
    "026_CanonicalizeModelSelectionOptions",
    "027_ProviderSessionRuntimeInstanceId",
    "028_ProjectionThreadSessionInstanceId",
    "029_ProjectionThreadDetailOrderingIndexes",
    "030_ProjectionThreadShellArchiveIndexes",
];

const DEFAULT_MODEL_SELECTION_JSON: &str = r#"{"provider":"codex","model":"gpt-5.4"}"#;
pub const ORCHESTRATION_PROJECTOR_NAMES: &[&str] = &[
    "projection.projects",
    "projection.threads",
    "projection.thread-messages",
    "projection.thread-proposed-plans",
    "projection.thread-activities",
    "projection.thread-sessions",
    "projection.thread-turns",
    "projection.checkpoints",
    "projection.pending-approvals",
];

#[derive(Debug)]
pub enum ProjectionPersistenceError {
    Sqlite(rusqlite::Error),
    Json(serde_json::Error),
    InvalidProjectionEventPayload(String),
    OrchestrationCommandInvariant(OrchestrationCommandInvariantError),
    InvalidProjectScript(String),
    InvalidRuntimeMode(String),
    InvalidInteractionMode(String),
    InvalidProviderRuntimeBinding(String),
}

impl fmt::Display for ProjectionPersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(formatter, "sqlite error: {error}"),
            Self::Json(error) => write!(formatter, "json error: {error}"),
            Self::InvalidProjectionEventPayload(message) => {
                write!(formatter, "invalid projection event payload: {message}")
            }
            Self::OrchestrationCommandInvariant(error) => {
                write!(
                    formatter,
                    "orchestration command invariant failed for {}: {}",
                    error.command_type, error.detail
                )
            }
            Self::InvalidProjectScript(message) => {
                write!(formatter, "invalid project script: {message}")
            }
            Self::InvalidRuntimeMode(value) => write!(formatter, "invalid runtime mode: {value}"),
            Self::InvalidInteractionMode(value) => {
                write!(formatter, "invalid interaction mode: {value}")
            }
            Self::InvalidProviderRuntimeBinding(message) => {
                write!(formatter, "invalid provider runtime binding: {message}")
            }
        }
    }
}

impl Error for ProjectionPersistenceError {}

impl From<rusqlite::Error> for ProjectionPersistenceError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<serde_json::Error> for ProjectionPersistenceError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<OrchestrationCommandInvariantError> for ProjectionPersistenceError {
    fn from(error: OrchestrationCommandInvariantError) -> Self {
        Self::OrchestrationCommandInvariant(error)
    }
}

pub type ProjectionPersistenceResult<T> = Result<T, ProjectionPersistenceError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceErrorTag {
    PersistenceSqlError,
    PersistenceDecodeError,
    ProviderSessionRepositoryValidationError,
    ProviderSessionRepositoryPersistenceError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistenceTaggedErrorPlan {
    pub tag: PersistenceErrorTag,
    pub operation: String,
    pub detail: Option<String>,
    pub issue: Option<String>,
    pub message: String,
}

pub fn persistence_sql_error_plan(operation: &str) -> PersistenceTaggedErrorPlan {
    let detail = format!("Failed to execute {operation}");
    PersistenceTaggedErrorPlan {
        tag: PersistenceErrorTag::PersistenceSqlError,
        operation: operation.to_string(),
        detail: Some(detail.clone()),
        issue: None,
        message: format!("SQL error in {operation}: {detail}"),
    }
}

pub fn persistence_decode_error_plan(operation: &str, issue: &str) -> PersistenceTaggedErrorPlan {
    PersistenceTaggedErrorPlan {
        tag: PersistenceErrorTag::PersistenceDecodeError,
        operation: operation.to_string(),
        detail: None,
        issue: Some(issue.to_string()),
        message: format!("Decode error in {operation}: {issue}"),
    }
}

pub fn persistence_decode_cause_error_plan(operation: &str) -> PersistenceTaggedErrorPlan {
    persistence_decode_error_plan(operation, &format!("Failed to execute {operation}"))
}

pub fn provider_session_repository_validation_error_plan(
    operation: &str,
    issue: &str,
) -> PersistenceTaggedErrorPlan {
    PersistenceTaggedErrorPlan {
        tag: PersistenceErrorTag::ProviderSessionRepositoryValidationError,
        operation: operation.to_string(),
        detail: None,
        issue: Some(issue.to_string()),
        message: format!("Provider session repository validation failed in {operation}: {issue}"),
    }
}

pub fn provider_session_repository_persistence_error_plan(
    operation: &str,
    detail: &str,
) -> PersistenceTaggedErrorPlan {
    PersistenceTaggedErrorPlan {
        tag: PersistenceErrorTag::ProviderSessionRepositoryPersistenceError,
        operation: operation.to_string(),
        detail: Some(detail.to_string()),
        issue: None,
        message: format!("Provider session repository persistence error in {operation}: {detail}"),
    }
}

pub fn is_persistence_error_tag(tag: PersistenceErrorTag) -> bool {
    matches!(
        tag,
        PersistenceErrorTag::PersistenceSqlError | PersistenceErrorTag::PersistenceDecodeError
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistenceMigrationEntry {
    pub id: u32,
    pub name: &'static str,
    pub module_name: &'static str,
}

pub fn persistence_migration_entries() -> Vec<PersistenceMigrationEntry> {
    PROJECTION_SQLITE_MIGRATION_NAMES
        .iter()
        .filter_map(|migration_name| {
            let (id, name) = migration_name.split_once('_')?;
            Some(PersistenceMigrationEntry {
                id: id.parse().ok()?,
                name,
                module_name: migration_name,
            })
        })
        .collect()
}

pub fn migration_record_key(entry: PersistenceMigrationEntry) -> String {
    format!("{}_{}", entry.id, entry.name)
}

pub fn migration_loader_record_keys(through_id: Option<u32>) -> Vec<String> {
    persistence_migration_entries()
        .into_iter()
        .filter(|entry| through_id.is_none_or(|through_id| entry.id <= through_id))
        .map(migration_record_key)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqliteRuntime {
    Bun,
    Node,
}

pub fn sqlite_runtime_from_bun_version_present(bun_version_present: bool) -> SqliteRuntime {
    if bun_version_present {
        SqliteRuntime::Bun
    } else {
        SqliteRuntime::Node
    }
}

pub fn node_sqlite_statement_columns_supported(node_version: &str) -> bool {
    let mut parts = node_version
        .split('.')
        .filter_map(|part| part.parse::<u32>().ok());
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    (major == 22 && minor >= 16) || (major == 23 && minor >= 11) || major >= 24
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSqliteClientConfigPlan {
    pub filename: String,
    pub readonly: bool,
    pub allow_extension: bool,
    pub prepare_cache_size: usize,
    pub prepare_cache_ttl: &'static str,
    pub span_attributes: Vec<(String, String)>,
    pub transform_result_names: bool,
    pub transform_query_names: bool,
}

pub fn node_sqlite_client_config_plan(
    filename: &str,
    readonly: Option<bool>,
    allow_extension: Option<bool>,
    span_attributes: &[(&str, &str)],
    transform_result_names: bool,
    transform_query_names: bool,
) -> NodeSqliteClientConfigPlan {
    NodeSqliteClientConfigPlan {
        filename: filename.to_string(),
        readonly: readonly.unwrap_or(false),
        allow_extension: allow_extension.unwrap_or(false),
        prepare_cache_size: 200,
        prepare_cache_ttl: "10 minutes",
        span_attributes: span_attributes
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .chain([("db.system.name".to_string(), "sqlite".to_string())])
            .collect(),
        transform_result_names,
        transform_query_names,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlitePersistenceLayerPlan {
    pub runtime: SqliteRuntime,
    pub filename: String,
    pub db_dir: Option<String>,
    pub setup_statements: Vec<&'static str>,
    pub span_attributes: Vec<(String, String)>,
    pub migration_keys: Vec<String>,
}

pub fn sqlite_persistence_layer_plan(
    db_path: &str,
    runtime: SqliteRuntime,
    through_migration: Option<u32>,
) -> SqlitePersistenceLayerPlan {
    let path = Path::new(db_path);
    let db_dir = path.parent().map(|parent| parent.display().to_string());
    let db_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(db_path)
        .to_string();
    SqlitePersistenceLayerPlan {
        runtime,
        filename: db_path.to_string(),
        db_dir,
        setup_statements: vec!["PRAGMA journal_mode = WAL;", "PRAGMA foreign_keys = ON;"],
        span_attributes: vec![
            ("db.name".to_string(), db_name),
            ("service.name".to_string(), "t3-server".to_string()),
        ],
        migration_keys: migration_loader_record_keys(through_migration),
    }
}

pub fn projection_checkpoint_repository_operation_names() -> Vec<(&'static str, &'static str)> {
    vec![
        ("upsert", "ProjectionCheckpointRepository.upsert:query"),
        (
            "listByThreadId",
            "ProjectionCheckpointRepository.listByThreadId:query",
        ),
        (
            "getByThreadAndTurnCount",
            "ProjectionCheckpointRepository.getByThreadAndTurnCount:query",
        ),
        (
            "deleteByThreadId",
            "ProjectionCheckpointRepository.deleteByThreadId:query",
        ),
    ]
}

pub fn provider_session_runtime_repository_operation_names() -> Vec<(&'static str, &'static str)> {
    vec![
        ("upsert", "ProviderSessionRuntimeRepository.upsert:query"),
        (
            "getByThreadId",
            "ProviderSessionRuntimeRepository.getByThreadId:query",
        ),
        ("list", "ProviderSessionRuntimeRepository.list:query"),
        (
            "deleteByThreadId",
            "ProviderSessionRuntimeRepository.deleteByThreadId:query",
        ),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionPendingApprovalRow {
    pub request_id: String,
    pub thread_id: String,
    pub turn_id: Option<String>,
    pub status: String,
    pub decision: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewOrchestrationEventRow {
    pub event_id: String,
    pub aggregate_kind: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub occurred_at: String,
    pub command_id: Option<String>,
    pub causation_event_id: Option<String>,
    pub correlation_id: Option<String>,
    pub payload: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrchestrationEventRow {
    pub sequence: i64,
    pub stream_version: i64,
    pub event_id: String,
    pub aggregate_kind: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub occurred_at: String,
    pub command_id: Option<String>,
    pub causation_event_id: Option<String>,
    pub correlation_id: Option<String>,
    pub actor_kind: String,
    pub payload: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationCommandReceiptRow {
    pub command_id: String,
    pub aggregate_kind: String,
    pub aggregate_id: String,
    pub accepted_at: String,
    pub result_sequence: i64,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionProviderServiceRequest {
    pub event: OrchestrationEventRow,
    pub request: ProviderServiceRequest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionThreadCleanupRequest {
    pub event: OrchestrationEventRow,
    pub request: ThreadDeletionCleanupRequest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderSessionRuntimeRow {
    pub thread_id: String,
    pub provider_name: String,
    pub provider_instance_id: Option<String>,
    pub adapter_key: String,
    pub runtime_mode: RuntimeMode,
    pub status: String,
    pub last_seen_at: String,
    pub resume_cursor: Option<Value>,
    pub runtime_payload: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSessionClientMetadataRow {
    pub label: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: String,
    pub os: Option<String>,
    pub browser: Option<String>,
}

impl Default for AuthSessionClientMetadataRow {
    fn default() -> Self {
        Self {
            label: None,
            ip_address: None,
            user_agent: None,
            device_type: "unknown".to_string(),
            os: None,
            browser: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSessionRow {
    pub session_id: String,
    pub subject: String,
    pub role: String,
    pub method: String,
    pub client: AuthSessionClientMetadataRow,
    pub issued_at: String,
    pub expires_at: String,
    pub last_connected_at: Option<String>,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthPairingLinkRow {
    pub id: String,
    pub credential: String,
    pub method: String,
    pub role: String,
    pub subject: String,
    pub label: Option<String>,
    pub created_at: String,
    pub expires_at: String,
    pub consumed_at: Option<String>,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProjectionReactorBatch {
    pub high_water_sequence: i64,
    pub provider_requests: Vec<ProjectionProviderServiceRequest>,
    pub thread_cleanup_requests: Vec<ProjectionThreadCleanupRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionStateRow {
    pub projector: String,
    pub last_applied_sequence: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectionTurnRow {
    thread_id: String,
    turn_id: Option<String>,
    pending_message_id: Option<String>,
    source_proposed_plan_thread_id: Option<String>,
    source_proposed_plan_id: Option<String>,
    assistant_message_id: Option<String>,
    state: String,
    requested_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    checkpoint_turn_count: Option<u32>,
    checkpoint_ref: Option<String>,
    checkpoint_status: Option<String>,
    checkpoint_files: Vec<TurnDiffFileChange>,
}

pub struct ProjectionSqliteStore {
    connection: Connection,
}

impl ProjectionSqliteStore {
    pub fn open(path: impl AsRef<Path>) -> ProjectionPersistenceResult<Self> {
        let store = Self {
            connection: Connection::open(path)?,
        };
        store.apply_migrations()?;
        Ok(store)
    }

    pub fn open_in_memory() -> ProjectionPersistenceResult<Self> {
        let store = Self {
            connection: Connection::open_in_memory()?,
        };
        store.apply_migrations()?;
        Ok(store)
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn apply_migrations(&self) -> ProjectionPersistenceResult<()> {
        self.connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS orchestration_events (
              sequence INTEGER PRIMARY KEY AUTOINCREMENT,
              event_id TEXT NOT NULL UNIQUE,
              aggregate_kind TEXT NOT NULL,
              stream_id TEXT NOT NULL,
              stream_version INTEGER NOT NULL,
              event_type TEXT NOT NULL,
              occurred_at TEXT NOT NULL,
              command_id TEXT,
              causation_event_id TEXT,
              correlation_id TEXT,
              actor_kind TEXT NOT NULL,
              payload_json TEXT NOT NULL,
              metadata_json TEXT NOT NULL
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_orch_events_stream_version
              ON orchestration_events(aggregate_kind, stream_id, stream_version);
            CREATE INDEX IF NOT EXISTS idx_orch_events_stream_sequence
              ON orchestration_events(aggregate_kind, stream_id, sequence);
            CREATE INDEX IF NOT EXISTS idx_orch_events_command_id
              ON orchestration_events(command_id);
            CREATE INDEX IF NOT EXISTS idx_orch_events_correlation_id
              ON orchestration_events(correlation_id);

            CREATE TABLE IF NOT EXISTS orchestration_command_receipts (
              command_id TEXT PRIMARY KEY,
              aggregate_kind TEXT NOT NULL,
              aggregate_id TEXT NOT NULL,
              accepted_at TEXT NOT NULL,
              result_sequence INTEGER NOT NULL,
              status TEXT NOT NULL,
              error TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_orch_command_receipts_aggregate
              ON orchestration_command_receipts(aggregate_kind, aggregate_id);
            CREATE INDEX IF NOT EXISTS idx_orch_command_receipts_sequence
              ON orchestration_command_receipts(result_sequence);

            CREATE TABLE IF NOT EXISTS provider_session_runtime (
              thread_id TEXT PRIMARY KEY,
              provider_name TEXT NOT NULL,
              provider_instance_id TEXT,
              adapter_key TEXT NOT NULL,
              runtime_mode TEXT NOT NULL DEFAULT 'full-access',
              status TEXT NOT NULL,
              last_seen_at TEXT NOT NULL,
              resume_cursor_json TEXT,
              runtime_payload_json TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_provider_session_runtime_status
              ON provider_session_runtime(status);
            CREATE INDEX IF NOT EXISTS idx_provider_session_runtime_provider
              ON provider_session_runtime(provider_name);
            CREATE INDEX IF NOT EXISTS idx_provider_session_runtime_instance
              ON provider_session_runtime(provider_instance_id);

            CREATE TABLE IF NOT EXISTS auth_pairing_links (
              id TEXT PRIMARY KEY,
              credential TEXT NOT NULL UNIQUE,
              method TEXT NOT NULL,
              role TEXT NOT NULL,
              subject TEXT NOT NULL,
              label TEXT,
              created_at TEXT NOT NULL,
              expires_at TEXT NOT NULL,
              consumed_at TEXT,
              revoked_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_auth_pairing_links_active
              ON auth_pairing_links(revoked_at, consumed_at, expires_at);

            CREATE TABLE IF NOT EXISTS auth_sessions (
              session_id TEXT PRIMARY KEY,
              subject TEXT NOT NULL,
              role TEXT NOT NULL,
              method TEXT NOT NULL,
              client_label TEXT,
              client_ip_address TEXT,
              client_user_agent TEXT,
              client_device_type TEXT NOT NULL DEFAULT 'unknown',
              client_os TEXT,
              client_browser TEXT,
              issued_at TEXT NOT NULL,
              expires_at TEXT NOT NULL,
              last_connected_at TEXT,
              revoked_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_auth_sessions_active
              ON auth_sessions(revoked_at, expires_at, issued_at);

            CREATE TABLE IF NOT EXISTS projection_projects (
              project_id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              workspace_root TEXT NOT NULL,
              default_model_selection_json TEXT,
              scripts_json TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              deleted_at TEXT
            );

            CREATE TABLE IF NOT EXISTS projection_threads (
              thread_id TEXT PRIMARY KEY,
              project_id TEXT NOT NULL,
              title TEXT NOT NULL,
              model_selection_json TEXT NOT NULL,
              runtime_mode TEXT NOT NULL DEFAULT 'full-access',
              interaction_mode TEXT NOT NULL DEFAULT 'default',
              branch TEXT,
              worktree_path TEXT,
              latest_turn_id TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              archived_at TEXT,
              latest_user_message_at TEXT,
              pending_approval_count INTEGER NOT NULL DEFAULT 0,
              pending_user_input_count INTEGER NOT NULL DEFAULT 0,
              has_actionable_proposed_plan INTEGER NOT NULL DEFAULT 0,
              deleted_at TEXT
            );

            CREATE TABLE IF NOT EXISTS projection_thread_sessions (
              thread_id TEXT PRIMARY KEY,
              status TEXT NOT NULL,
              provider_name TEXT,
              provider_instance_id TEXT,
              provider_session_id TEXT,
              provider_thread_id TEXT,
              runtime_mode TEXT NOT NULL DEFAULT 'full-access',
              active_turn_id TEXT,
              last_error TEXT,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS projection_thread_messages (
              message_id TEXT PRIMARY KEY,
              thread_id TEXT NOT NULL,
              turn_id TEXT,
              role TEXT NOT NULL,
              text TEXT NOT NULL,
              attachments_json TEXT,
              is_streaming INTEGER NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS projection_thread_activities (
              activity_id TEXT PRIMARY KEY,
              thread_id TEXT NOT NULL,
              turn_id TEXT,
              tone TEXT NOT NULL,
              kind TEXT NOT NULL,
              summary TEXT NOT NULL,
              payload_json TEXT NOT NULL,
              sequence INTEGER,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS projection_thread_proposed_plans (
              plan_id TEXT PRIMARY KEY,
              thread_id TEXT NOT NULL,
              turn_id TEXT,
              plan_markdown TEXT NOT NULL,
              implemented_at TEXT,
              implementation_thread_id TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS projection_turns (
              row_id INTEGER PRIMARY KEY AUTOINCREMENT,
              thread_id TEXT NOT NULL,
              turn_id TEXT,
              pending_message_id TEXT,
              source_proposed_plan_thread_id TEXT,
              source_proposed_plan_id TEXT,
              assistant_message_id TEXT,
              state TEXT NOT NULL,
              requested_at TEXT NOT NULL,
              started_at TEXT,
              completed_at TEXT,
              checkpoint_turn_count INTEGER,
              checkpoint_ref TEXT,
              checkpoint_status TEXT,
              checkpoint_files_json TEXT NOT NULL,
              UNIQUE (thread_id, turn_id),
              UNIQUE (thread_id, checkpoint_turn_count)
            );

            CREATE TABLE IF NOT EXISTS projection_pending_approvals (
              request_id TEXT PRIMARY KEY,
              thread_id TEXT NOT NULL,
              turn_id TEXT,
              status TEXT NOT NULL,
              decision TEXT,
              created_at TEXT NOT NULL,
              resolved_at TEXT
            );

            CREATE TABLE IF NOT EXISTS projection_state (
              projector TEXT PRIMARY KEY,
              last_applied_sequence INTEGER NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_projection_projects_updated_at
              ON projection_projects(updated_at);
            CREATE INDEX IF NOT EXISTS idx_projection_projects_workspace_root_deleted_at
              ON projection_projects(workspace_root, deleted_at);
            CREATE INDEX IF NOT EXISTS idx_projection_threads_project_id
              ON projection_threads(project_id);
            CREATE INDEX IF NOT EXISTS idx_projection_threads_project_archived_at
              ON projection_threads(project_id, archived_at);
            CREATE INDEX IF NOT EXISTS idx_projection_threads_project_deleted_created
              ON projection_threads(project_id, deleted_at, created_at);
            CREATE INDEX IF NOT EXISTS idx_projection_threads_shell_active
              ON projection_threads(deleted_at, archived_at, project_id, created_at, thread_id);
            CREATE INDEX IF NOT EXISTS idx_projection_threads_shell_archived
              ON projection_threads(deleted_at, archived_at, project_id, thread_id);
            CREATE INDEX IF NOT EXISTS idx_projection_thread_sessions_instance
              ON projection_thread_sessions(provider_instance_id);
            CREATE INDEX IF NOT EXISTS idx_projection_thread_sessions_provider_session
              ON projection_thread_sessions(provider_session_id);
            CREATE INDEX IF NOT EXISTS idx_projection_thread_messages_thread_created_id
              ON projection_thread_messages(thread_id, created_at, message_id);
            CREATE INDEX IF NOT EXISTS idx_projection_thread_activities_thread_sequence_created_id
              ON projection_thread_activities(thread_id, sequence, created_at, activity_id);
            CREATE INDEX IF NOT EXISTS idx_projection_thread_proposed_plans_thread_created
              ON projection_thread_proposed_plans(thread_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_projection_turns_thread_requested
              ON projection_turns(thread_id, requested_at);
            CREATE INDEX IF NOT EXISTS idx_projection_turns_thread_checkpoint_completed
              ON projection_turns(thread_id, checkpoint_turn_count, completed_at);
            CREATE INDEX IF NOT EXISTS idx_projection_pending_approvals_thread_status
              ON projection_pending_approvals(thread_id, status);
            "#,
        )?;
        Ok(())
    }

    pub fn table_column_names(&self, table: &str) -> ProjectionPersistenceResult<Vec<String>> {
        let mut statement = self
            .connection
            .prepare(&format!("PRAGMA table_info({table})"))?;
        let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ProjectionPersistenceError::from)
    }

    pub fn table_index_names(&self, table: &str) -> ProjectionPersistenceResult<Vec<String>> {
        let mut statement = self
            .connection
            .prepare(&format!("PRAGMA index_list({table})"))?;
        let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ProjectionPersistenceError::from)
    }

    pub fn upsert_provider_session_runtime(
        &self,
        row: &ProviderSessionRuntimeRow,
    ) -> ProjectionPersistenceResult<()> {
        let resume_cursor_json = row
            .resume_cursor
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let runtime_payload_json = row
            .runtime_payload
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        self.connection.execute(
            r#"
            INSERT INTO provider_session_runtime (
              thread_id,
              provider_name,
              provider_instance_id,
              adapter_key,
              runtime_mode,
              status,
              last_seen_at,
              resume_cursor_json,
              runtime_payload_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT (thread_id)
            DO UPDATE SET
              provider_name = excluded.provider_name,
              provider_instance_id = excluded.provider_instance_id,
              adapter_key = excluded.adapter_key,
              runtime_mode = excluded.runtime_mode,
              status = excluded.status,
              last_seen_at = excluded.last_seen_at,
              resume_cursor_json = excluded.resume_cursor_json,
              runtime_payload_json = excluded.runtime_payload_json
            "#,
            params![
                row.thread_id.as_str(),
                row.provider_name.as_str(),
                row.provider_instance_id.as_deref(),
                row.adapter_key.as_str(),
                runtime_mode_to_t3(row.runtime_mode),
                row.status.as_str(),
                row.last_seen_at.as_str(),
                resume_cursor_json.as_deref(),
                runtime_payload_json.as_deref(),
            ],
        )?;
        Ok(())
    }

    pub fn get_provider_session_runtime_by_thread_id(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Option<ProviderSessionRuntimeRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              thread_id,
              provider_name,
              provider_instance_id,
              adapter_key,
              runtime_mode,
              status,
              last_seen_at,
              resume_cursor_json,
              runtime_payload_json
            FROM provider_session_runtime
            WHERE thread_id = ?1
            "#,
        )?;
        let mut rows = statement.query(params![thread_id])?;
        rows.next()?
            .map(provider_session_runtime_row_from_sql)
            .transpose()
    }

    pub fn list_provider_session_runtimes(
        &self,
    ) -> ProjectionPersistenceResult<Vec<ProviderSessionRuntimeRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              thread_id,
              provider_name,
              provider_instance_id,
              adapter_key,
              runtime_mode,
              status,
              last_seen_at,
              resume_cursor_json,
              runtime_payload_json
            FROM provider_session_runtime
            ORDER BY last_seen_at ASC, thread_id ASC
            "#,
        )?;
        let mut rows = statement.query([])?;
        collect_projection_rows(&mut rows, provider_session_runtime_row_from_sql)
    }

    pub fn delete_provider_session_runtime_by_thread_id(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            "DELETE FROM provider_session_runtime WHERE thread_id = ?1",
            params![thread_id],
        )?;
        Ok(())
    }

    pub fn insert_auth_session(&self, row: &AuthSessionRow) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            r#"
            INSERT INTO auth_sessions (
              session_id,
              subject,
              role,
              method,
              client_label,
              client_ip_address,
              client_user_agent,
              client_device_type,
              client_os,
              client_browser,
              issued_at,
              expires_at,
              last_connected_at,
              revoked_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                row.session_id,
                row.subject,
                row.role,
                row.method,
                row.client.label,
                row.client.ip_address,
                row.client.user_agent,
                row.client.device_type,
                row.client.os,
                row.client.browser,
                row.issued_at,
                row.expires_at,
                row.last_connected_at,
                row.revoked_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_auth_session_by_id(
        &self,
        session_id: &str,
    ) -> ProjectionPersistenceResult<Option<AuthSessionRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              session_id,
              subject,
              role,
              method,
              client_label,
              client_ip_address,
              client_user_agent,
              client_device_type,
              client_os,
              client_browser,
              issued_at,
              expires_at,
              last_connected_at,
              revoked_at
            FROM auth_sessions
            WHERE session_id = ?1
            "#,
        )?;
        let mut rows = statement.query(params![session_id])?;
        rows.next()?
            .map(auth_session_row_from_sql)
            .transpose()
            .map_err(ProjectionPersistenceError::from)
    }

    pub fn list_active_auth_sessions(
        &self,
        now: &str,
    ) -> ProjectionPersistenceResult<Vec<AuthSessionRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              session_id,
              subject,
              role,
              method,
              client_label,
              client_ip_address,
              client_user_agent,
              client_device_type,
              client_os,
              client_browser,
              issued_at,
              expires_at,
              last_connected_at,
              revoked_at
            FROM auth_sessions
            WHERE revoked_at IS NULL
              AND expires_at > ?1
            ORDER BY issued_at DESC, session_id DESC
            "#,
        )?;
        let mut rows = statement.query(params![now])?;
        collect_projection_rows(&mut rows, auth_session_row_from_sql)
    }

    pub fn set_auth_session_last_connected_at(
        &self,
        session_id: &str,
        last_connected_at: &str,
    ) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            r#"
            UPDATE auth_sessions
            SET last_connected_at = ?2
            WHERE session_id = ?1
              AND revoked_at IS NULL
            "#,
            params![session_id, last_connected_at],
        )?;
        Ok(())
    }

    pub fn revoke_auth_session(
        &self,
        session_id: &str,
        revoked_at: &str,
    ) -> ProjectionPersistenceResult<bool> {
        let updated = self.connection.execute(
            r#"
            UPDATE auth_sessions
            SET revoked_at = ?2
            WHERE session_id = ?1
              AND revoked_at IS NULL
            "#,
            params![session_id, revoked_at],
        )?;
        Ok(updated > 0)
    }

    pub fn revoke_other_auth_sessions(
        &self,
        current_session_id: &str,
        revoked_at: &str,
    ) -> ProjectionPersistenceResult<Vec<String>> {
        let active_before = self.list_active_auth_sessions(revoked_at)?;
        self.connection.execute(
            r#"
            UPDATE auth_sessions
            SET revoked_at = ?2
            WHERE session_id <> ?1
              AND revoked_at IS NULL
            "#,
            params![current_session_id, revoked_at],
        )?;
        Ok(active_before
            .into_iter()
            .filter(|row| row.session_id != current_session_id)
            .map(|row| row.session_id)
            .collect())
    }

    pub fn insert_auth_pairing_link(
        &self,
        row: &AuthPairingLinkRow,
    ) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            r#"
            INSERT INTO auth_pairing_links (
              id,
              credential,
              method,
              role,
              subject,
              label,
              created_at,
              expires_at,
              consumed_at,
              revoked_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                row.id,
                row.credential,
                row.method,
                row.role,
                row.subject,
                row.label,
                row.created_at,
                row.expires_at,
                row.consumed_at,
                row.revoked_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_auth_pairing_link_by_credential(
        &self,
        credential: &str,
    ) -> ProjectionPersistenceResult<Option<AuthPairingLinkRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              id,
              credential,
              method,
              role,
              subject,
              label,
              created_at,
              expires_at,
              consumed_at,
              revoked_at
            FROM auth_pairing_links
            WHERE credential = ?1
            "#,
        )?;
        let mut rows = statement.query(params![credential])?;
        rows.next()?
            .map(auth_pairing_link_row_from_sql)
            .transpose()
            .map_err(ProjectionPersistenceError::from)
    }

    pub fn list_active_auth_pairing_links(
        &self,
        now: &str,
    ) -> ProjectionPersistenceResult<Vec<AuthPairingLinkRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              id,
              credential,
              method,
              role,
              subject,
              label,
              created_at,
              expires_at,
              consumed_at,
              revoked_at
            FROM auth_pairing_links
            WHERE revoked_at IS NULL
              AND consumed_at IS NULL
              AND expires_at > ?1
            ORDER BY created_at DESC, id DESC
            "#,
        )?;
        let mut rows = statement.query(params![now])?;
        collect_projection_rows(&mut rows, auth_pairing_link_row_from_sql)
    }

    pub fn consume_available_auth_pairing_link(
        &self,
        credential: &str,
        consumed_at: &str,
        now: &str,
    ) -> ProjectionPersistenceResult<Option<AuthPairingLinkRow>> {
        let updated = self.connection.execute(
            r#"
            UPDATE auth_pairing_links
            SET consumed_at = ?2
            WHERE credential = ?1
              AND revoked_at IS NULL
              AND consumed_at IS NULL
              AND expires_at > ?3
            "#,
            params![credential, consumed_at, now],
        )?;
        if updated == 0 {
            return Ok(None);
        }
        self.get_auth_pairing_link_by_credential(credential)
    }

    pub fn revoke_auth_pairing_link(
        &self,
        id: &str,
        revoked_at: &str,
    ) -> ProjectionPersistenceResult<bool> {
        let updated = self.connection.execute(
            r#"
            UPDATE auth_pairing_links
            SET revoked_at = ?2
            WHERE id = ?1
              AND revoked_at IS NULL
              AND consumed_at IS NULL
            "#,
            params![id, revoked_at],
        )?;
        Ok(updated > 0)
    }

    pub fn upsert_provider_runtime_binding(
        &self,
        binding: &ProviderRuntimeBinding,
        last_seen_at: &str,
    ) -> ProjectionPersistenceResult<ProviderSessionRuntimeRow> {
        let existing = self.get_provider_session_runtime_by_thread_id(&binding.thread_id)?;
        let provider_changed = existing
            .as_ref()
            .is_some_and(|row| row.provider_name != binding.provider);
        let provider_instance_id = binding.provider_instance_id.clone().or_else(|| {
            if provider_changed {
                None
            } else {
                existing
                    .as_ref()
                    .and_then(|row| row.provider_instance_id.clone())
            }
        });
        let Some(provider_instance_id) = provider_instance_id else {
            return Err(ProjectionPersistenceError::InvalidProviderRuntimeBinding(
                "providerInstanceId is required for provider session runtime bindings.".to_string(),
            ));
        };

        let runtime_payload = merge_provider_runtime_payload(
            existing
                .as_ref()
                .and_then(|row| row.runtime_payload.clone()),
            binding.runtime_payload.clone(),
        );
        let row = ProviderSessionRuntimeRow {
            thread_id: binding.thread_id.clone(),
            provider_name: binding.provider.clone(),
            provider_instance_id: Some(provider_instance_id),
            adapter_key: binding.adapter_key.clone().unwrap_or_else(|| {
                if provider_changed {
                    binding.provider.clone()
                } else {
                    existing
                        .as_ref()
                        .map(|row| row.adapter_key.clone())
                        .unwrap_or_else(|| binding.provider.clone())
                }
            }),
            runtime_mode: binding
                .runtime_mode
                .or_else(|| existing.as_ref().map(|row| row.runtime_mode))
                .unwrap_or(RuntimeMode::FullAccess),
            status: binding
                .status
                .clone()
                .or_else(|| existing.as_ref().map(|row| row.status.clone()))
                .unwrap_or_else(|| "running".to_string()),
            last_seen_at: last_seen_at.to_string(),
            resume_cursor: binding
                .resume_cursor
                .clone()
                .or_else(|| existing.as_ref().and_then(|row| row.resume_cursor.clone())),
            runtime_payload,
        };
        self.upsert_provider_session_runtime(&row)?;
        Ok(row)
    }

    pub fn get_provider_runtime_binding_by_thread_id(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Option<ProviderRuntimeBinding>> {
        self.get_provider_session_runtime_by_thread_id(thread_id)?
            .map(|row| provider_runtime_binding_from_session_runtime(&row))
            .transpose()
    }

    pub fn list_provider_runtime_bindings(
        &self,
    ) -> ProjectionPersistenceResult<Vec<ProviderRuntimeBinding>> {
        self.list_provider_session_runtimes()?
            .into_iter()
            .map(|row| provider_runtime_binding_from_session_runtime(&row))
            .collect()
    }

    pub fn append_event(
        &self,
        event: &NewOrchestrationEventRow,
    ) -> ProjectionPersistenceResult<OrchestrationEventRow> {
        let payload_json = serde_json::to_string(&event.payload)?;
        let metadata_json = serde_json::to_string(&event.metadata)?;
        let actor_kind = infer_actor_kind(event);
        self.connection.execute(
            r#"
            INSERT INTO orchestration_events (
              event_id,
              aggregate_kind,
              stream_id,
              stream_version,
              event_type,
              occurred_at,
              command_id,
              causation_event_id,
              correlation_id,
              actor_kind,
              payload_json,
              metadata_json
            )
            VALUES (
              ?1,
              ?2,
              ?3,
              COALESCE(
                (
                  SELECT stream_version + 1
                  FROM orchestration_events
                  WHERE aggregate_kind = ?2
                    AND stream_id = ?3
                  ORDER BY stream_version DESC
                  LIMIT 1
                ),
                0
              ),
              ?4,
              ?5,
              ?6,
              ?7,
              ?8,
              ?9,
              ?10,
              ?11
            )
            "#,
            params![
                event.event_id.as_str(),
                event.aggregate_kind.as_str(),
                event.aggregate_id.as_str(),
                event.event_type.as_str(),
                event.occurred_at.as_str(),
                event.command_id.as_deref(),
                event.causation_event_id.as_deref(),
                event.correlation_id.as_deref(),
                actor_kind,
                payload_json,
                metadata_json
            ],
        )?;
        self.get_event_by_id(&event.event_id)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows.into())
    }

    pub fn get_event_by_id(
        &self,
        event_id: &str,
    ) -> ProjectionPersistenceResult<Option<OrchestrationEventRow>> {
        let mut statement = self.connection.prepare(&format!(
            "{} WHERE event_id = ?1",
            orchestration_event_select_sql()
        ))?;
        let mut rows = statement.query(params![event_id])?;
        rows.next()?
            .map(orchestration_event_row_from_sql)
            .transpose()
    }

    pub fn read_events_from_sequence(
        &self,
        sequence_exclusive: i64,
        limit: Option<i64>,
    ) -> ProjectionPersistenceResult<Vec<OrchestrationEventRow>> {
        let normalized_limit = limit.unwrap_or(1_000).max(0);
        if normalized_limit == 0 {
            return Ok(Vec::new());
        }
        let mut statement = self.connection.prepare(&format!(
            "{} WHERE sequence > ?1 ORDER BY sequence ASC LIMIT ?2",
            orchestration_event_select_sql()
        ))?;
        let mut rows = statement.query(params![sequence_exclusive.max(0), normalized_limit])?;
        collect_projection_rows(&mut rows, orchestration_event_row_from_sql)
    }

    pub fn read_all_events(&self) -> ProjectionPersistenceResult<Vec<OrchestrationEventRow>> {
        self.read_events_from_sequence(0, Some(i64::MAX))
    }

    pub fn provider_command_intents_from_sequence(
        &self,
        sequence_exclusive: i64,
    ) -> ProjectionPersistenceResult<Vec<(OrchestrationEventRow, ProviderCommandIntent)>> {
        Ok(self
            .read_events_from_sequence(sequence_exclusive, Some(i64::MAX))?
            .into_iter()
            .filter_map(|event| {
                let planned_event = planned_event_from_row(&event);
                provider_command_intent_for_event(&planned_event).map(|intent| (event, intent))
            })
            .collect())
    }

    pub fn thread_deletion_cleanup_actions_from_sequence(
        &self,
        sequence_exclusive: i64,
    ) -> ProjectionPersistenceResult<Vec<(OrchestrationEventRow, Vec<ThreadDeletionCleanupAction>)>>
    {
        Ok(self
            .read_events_from_sequence(sequence_exclusive, Some(i64::MAX))?
            .into_iter()
            .filter_map(|event| {
                let planned_event = planned_event_from_row(&event);
                thread_deletion_cleanup_actions_for_event(&planned_event)
                    .map(|(_, actions)| (event, actions))
            })
            .collect())
    }

    pub fn reactor_batch_from_sequence(
        &self,
        sequence_exclusive: i64,
    ) -> ProjectionPersistenceResult<ProjectionReactorBatch> {
        let provider_intents = self.provider_command_intents_from_sequence(sequence_exclusive)?;
        let cleanup_actions =
            self.thread_deletion_cleanup_actions_from_sequence(sequence_exclusive)?;
        let mut batch = ProjectionReactorBatch {
            high_water_sequence: sequence_exclusive.max(0),
            provider_requests: Vec::with_capacity(provider_intents.len()),
            thread_cleanup_requests: Vec::new(),
        };

        for (event, intent) in provider_intents {
            batch.high_water_sequence = batch.high_water_sequence.max(event.sequence);
            batch
                .provider_requests
                .push(ProjectionProviderServiceRequest {
                    request: provider_service_request_for_intent(&intent),
                    event,
                });
        }

        for (event, actions) in cleanup_actions {
            batch.high_water_sequence = batch.high_water_sequence.max(event.sequence);
            let thread_id = required_payload_string(&event.payload, "threadId")?;
            for request in thread_deletion_cleanup_requests(&thread_id, &actions) {
                batch
                    .thread_cleanup_requests
                    .push(ProjectionThreadCleanupRequest {
                        event: event.clone(),
                        request,
                    });
            }
        }

        Ok(batch)
    }

    pub fn upsert_command_receipt(
        &self,
        receipt: &OrchestrationCommandReceiptRow,
    ) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            r#"
            INSERT INTO orchestration_command_receipts (
              command_id,
              aggregate_kind,
              aggregate_id,
              accepted_at,
              result_sequence,
              status,
              error
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT (command_id)
            DO UPDATE SET
              aggregate_kind = excluded.aggregate_kind,
              aggregate_id = excluded.aggregate_id,
              accepted_at = excluded.accepted_at,
              result_sequence = excluded.result_sequence,
              status = excluded.status,
              error = excluded.error
            "#,
            params![
                receipt.command_id.as_str(),
                receipt.aggregate_kind.as_str(),
                receipt.aggregate_id.as_str(),
                receipt.accepted_at.as_str(),
                receipt.result_sequence,
                receipt.status.as_str(),
                receipt.error.as_deref()
            ],
        )?;
        Ok(())
    }

    pub fn get_command_receipt(
        &self,
        command_id: &str,
    ) -> ProjectionPersistenceResult<Option<OrchestrationCommandReceiptRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              command_id,
              aggregate_kind,
              aggregate_id,
              accepted_at,
              result_sequence,
              status,
              error
            FROM orchestration_command_receipts
            WHERE command_id = ?1
            "#,
        )?;
        let mut rows = statement.query(params![command_id])?;
        rows.next()?.map(command_receipt_row_from_sql).transpose()
    }

    pub fn upsert_projection_state(
        &self,
        row: &ProjectionStateRow,
    ) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            r#"
            INSERT INTO projection_state (
              projector,
              last_applied_sequence,
              updated_at
            )
            VALUES (?1, ?2, ?3)
            ON CONFLICT (projector)
            DO UPDATE SET
              last_applied_sequence = excluded.last_applied_sequence,
              updated_at = excluded.updated_at
            "#,
            params![
                row.projector.as_str(),
                row.last_applied_sequence,
                row.updated_at.as_str()
            ],
        )?;
        Ok(())
    }

    pub fn get_projection_state(
        &self,
        projector: &str,
    ) -> ProjectionPersistenceResult<Option<ProjectionStateRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              projector,
              last_applied_sequence,
              updated_at
            FROM projection_state
            WHERE projector = ?1
            "#,
        )?;
        let mut rows = statement.query(params![projector])?;
        rows.next()?.map(projection_state_row_from_sql).transpose()
    }

    pub fn list_projection_states(&self) -> ProjectionPersistenceResult<Vec<ProjectionStateRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              projector,
              last_applied_sequence,
              updated_at
            FROM projection_state
            ORDER BY projector ASC
            "#,
        )?;
        let mut rows = statement.query([])?;
        collect_projection_rows(&mut rows, projection_state_row_from_sql)
    }

    pub fn apply_pending_projection_events(&self) -> ProjectionPersistenceResult<usize> {
        let mut last_applied_sequence = i64::MAX;
        for projector in ORCHESTRATION_PROJECTOR_NAMES {
            let sequence = self
                .get_projection_state(projector)?
                .map(|state| state.last_applied_sequence)
                .unwrap_or(0);
            last_applied_sequence = last_applied_sequence.min(sequence);
        }
        if last_applied_sequence == i64::MAX {
            last_applied_sequence = 0;
        }

        let events = self.read_events_from_sequence(last_applied_sequence, None)?;
        for event in &events {
            self.apply_projection_event(event)?;
            for projector in ORCHESTRATION_PROJECTOR_NAMES {
                self.upsert_projection_state(&ProjectionStateRow {
                    projector: (*projector).to_string(),
                    last_applied_sequence: event.sequence,
                    updated_at: event.occurred_at.clone(),
                })?;
            }
        }
        Ok(events.len())
    }

    pub fn execute_orchestration_command(
        &self,
        command: &OrchestrationCommand,
        now_iso: &str,
    ) -> ProjectionPersistenceResult<Vec<OrchestrationEventRow>> {
        let read_model = self.load_orchestration_read_model()?;
        let planned_events = decide_orchestration_command(command, &read_model, now_iso)?;
        let mut appended_events = Vec::with_capacity(planned_events.len());
        for planned_event in planned_events {
            let event = self.append_event(&planned_event_to_new_row(planned_event))?;
            appended_events.push(event);
        }
        let result_sequence = appended_events
            .last()
            .map(|event| event.sequence)
            .unwrap_or_else(|| read_model.snapshot_sequence);
        let (aggregate_kind, aggregate_id) = command.aggregate_kind_and_id();
        self.upsert_command_receipt(&OrchestrationCommandReceiptRow {
            command_id: command.command_id().to_string(),
            aggregate_kind: aggregate_kind.to_string(),
            aggregate_id: aggregate_id.to_string(),
            accepted_at: now_iso.to_string(),
            result_sequence,
            status: "succeeded".to_string(),
            error: None,
        })?;
        self.apply_pending_projection_events()?;
        Ok(appended_events)
    }

    pub fn ingest_provider_runtime_event(
        &self,
        context: &ProviderRuntimeIngestionCommandPlanContext,
        event: &ProviderRuntimeEventInput,
        now_iso: &str,
    ) -> ProjectionPersistenceResult<Vec<OrchestrationEventRow>> {
        let commands = provider_runtime_event_to_orchestration_commands(context, event);
        let mut appended_events = Vec::new();
        for command in commands {
            appended_events.extend(self.execute_orchestration_command(&command, now_iso)?);
        }
        Ok(appended_events)
    }

    pub fn ingest_provider_runtime_events(
        &self,
        inputs: &[(
            ProviderRuntimeIngestionCommandPlanContext,
            ProviderRuntimeEventInput,
        )],
        now_iso: &str,
    ) -> ProjectionPersistenceResult<Vec<OrchestrationEventRow>> {
        let mut appended_events = Vec::new();
        for (context, event) in inputs {
            appended_events.extend(self.ingest_provider_runtime_event(context, event, now_iso)?);
        }
        Ok(appended_events)
    }

    pub fn drain_provider_runtime_ingestion_queue(
        &self,
        queue: &mut ProviderRuntimeIngestionQueue,
        now_iso: &str,
    ) -> ProjectionPersistenceResult<Vec<OrchestrationEventRow>> {
        let drained = queue.drain();
        self.ingest_provider_runtime_events(&drained.runtime_events, now_iso)
    }

    fn load_orchestration_read_model(&self) -> ProjectionPersistenceResult<OrchestrationReadModel> {
        let projects = self.list_projects()?;
        let threads = self.list_threads()?;
        let mut proposed_plans = Vec::new();
        for thread in &threads {
            for plan in self.list_proposed_plans_by_thread(&thread.thread_id)? {
                proposed_plans.push(OrchestrationProposedPlanRef {
                    thread_id: thread.thread_id.clone(),
                    plan,
                });
            }
        }
        let snapshot_sequence = self
            .read_events_from_sequence(0, Some(i64::MAX))?
            .last()
            .map(|event| event.sequence)
            .unwrap_or(0);
        Ok(OrchestrationReadModel {
            snapshot_sequence,
            projects,
            threads,
            proposed_plans,
        })
    }

    fn apply_projection_event(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        match event.event_type.as_str() {
            "project.created" => self.apply_project_created(event),
            "project.meta-updated" => self.apply_project_meta_updated(event),
            "project.deleted" => self.apply_project_deleted(event),
            "thread.created" => self.apply_thread_created(event),
            "thread.meta-updated" => self.apply_thread_meta_updated(event),
            "thread.runtime-mode-set" => self.apply_thread_runtime_mode_set(event),
            "thread.interaction-mode-set" => self.apply_thread_interaction_mode_set(event),
            "thread.archived" => self.apply_thread_archived(event),
            "thread.unarchived" => self.apply_thread_unarchived(event),
            "thread.deleted" => self.apply_thread_deleted(event),
            "thread.message-sent" => self.apply_thread_message_sent(event),
            "thread.session-set" => self.apply_thread_session_set(event),
            "thread.turn-start-requested" => self.apply_thread_turn_start_requested(event),
            "thread.turn-interrupt-requested" => self.apply_thread_turn_interrupt_requested(event),
            "thread.proposed-plan-upserted" => self.apply_thread_proposed_plan_upserted(event),
            "thread.turn-diff-completed" => self.apply_thread_turn_diff_completed(event),
            "thread.reverted" => self.apply_thread_reverted(event),
            "thread.activity-appended" => self.apply_thread_activity_appended(event),
            "thread.approval-response-requested" => {
                self.apply_thread_approval_response_requested(event)
            }
            _ => Ok(()),
        }
    }

    fn apply_project_created(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        self.upsert_project(&ProjectionProjectRow {
            project_id: required_payload_string(&event.payload, "projectId")?,
            title: required_payload_string(&event.payload, "title")?,
            workspace_root: required_payload_string(&event.payload, "workspaceRoot")?,
            scripts: payload_project_scripts(&event.payload)?,
            created_at: required_payload_string(&event.payload, "createdAt")?,
            updated_at: required_payload_string(&event.payload, "updatedAt")?,
            deleted_at: None,
        })
    }

    fn apply_project_meta_updated(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let project_id = required_payload_string(&event.payload, "projectId")?;
        let Some(mut project) = self.get_project(&project_id)? else {
            return Ok(());
        };
        if let Some(title) = optional_payload_string(&event.payload, "title")? {
            project.title = title;
        }
        if let Some(workspace_root) = optional_payload_string(&event.payload, "workspaceRoot")? {
            project.workspace_root = workspace_root;
        }
        if event.payload.get("scripts").is_some() {
            project.scripts = payload_project_scripts(&event.payload)?;
        }
        project.updated_at = required_payload_string(&event.payload, "updatedAt")?;
        self.upsert_project(&project)
    }

    fn apply_project_deleted(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let project_id = required_payload_string(&event.payload, "projectId")?;
        let Some(mut project) = self.get_project(&project_id)? else {
            return Ok(());
        };
        let deleted_at = required_payload_string(&event.payload, "deletedAt")?;
        project.deleted_at = Some(deleted_at.clone());
        project.updated_at = deleted_at;
        self.upsert_project(&project)
    }

    fn apply_thread_created(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        self.upsert_thread(&ProjectionThreadRow {
            thread_id: required_payload_string(&event.payload, "threadId")?,
            project_id: required_payload_string(&event.payload, "projectId")?,
            title: required_payload_string(&event.payload, "title")?,
            runtime_mode: runtime_mode_from_t3(&required_payload_string(
                &event.payload,
                "runtimeMode",
            )?)?,
            interaction_mode: interaction_mode_from_t3(
                &optional_payload_string(&event.payload, "interactionMode")?
                    .unwrap_or_else(|| "default".to_string()),
            )?,
            branch: optional_payload_string(&event.payload, "branch")?,
            worktree_path: optional_payload_string(&event.payload, "worktreePath")?,
            created_at: required_payload_string(&event.payload, "createdAt")?,
            updated_at: required_payload_string(&event.payload, "updatedAt")?,
            archived_at: None,
            latest_user_message_at: None,
            pending_approval_count: 0,
            pending_user_input_count: 0,
            has_actionable_proposed_plan: false,
            deleted_at: None,
        })
    }

    fn apply_thread_meta_updated(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let Some(mut thread) = self.get_thread(&thread_id)? else {
            return Ok(());
        };
        if let Some(title) = optional_payload_string(&event.payload, "title")? {
            thread.title = title;
        }
        if let Some(branch) = optional_payload_string(&event.payload, "branch")? {
            thread.branch = Some(branch);
        }
        if event.payload.get("branch").is_some_and(Value::is_null) {
            thread.branch = None;
        }
        if let Some(worktree_path) = optional_payload_string(&event.payload, "worktreePath")? {
            thread.worktree_path = Some(worktree_path);
        }
        if event
            .payload
            .get("worktreePath")
            .is_some_and(Value::is_null)
        {
            thread.worktree_path = None;
        }
        thread.updated_at = required_payload_string(&event.payload, "updatedAt")?;
        self.upsert_thread(&thread)
    }

    fn apply_thread_runtime_mode_set(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let Some(mut thread) = self.get_thread(&thread_id)? else {
            return Ok(());
        };
        thread.runtime_mode =
            runtime_mode_from_t3(&required_payload_string(&event.payload, "runtimeMode")?)?;
        thread.updated_at = required_payload_string(&event.payload, "updatedAt")?;
        self.upsert_thread(&thread)
    }

    fn apply_thread_interaction_mode_set(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let Some(mut thread) = self.get_thread(&thread_id)? else {
            return Ok(());
        };
        thread.interaction_mode =
            interaction_mode_from_t3(&required_payload_string(&event.payload, "interactionMode")?)?;
        thread.updated_at = required_payload_string(&event.payload, "updatedAt")?;
        self.upsert_thread(&thread)
    }

    fn apply_thread_archived(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let Some(mut thread) = self.get_thread(&thread_id)? else {
            return Ok(());
        };
        thread.archived_at = Some(required_payload_string(&event.payload, "archivedAt")?);
        thread.updated_at = required_payload_string(&event.payload, "updatedAt")?;
        self.upsert_thread(&thread)
    }

    fn apply_thread_unarchived(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let Some(mut thread) = self.get_thread(&thread_id)? else {
            return Ok(());
        };
        thread.archived_at = None;
        thread.updated_at = required_payload_string(&event.payload, "updatedAt")?;
        self.upsert_thread(&thread)
    }

    fn apply_thread_deleted(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let Some(mut thread) = self.get_thread(&thread_id)? else {
            return Ok(());
        };
        let deleted_at = required_payload_string(&event.payload, "deletedAt")?;
        thread.deleted_at = Some(deleted_at.clone());
        thread.updated_at = deleted_at;
        self.upsert_thread(&thread)
    }

    fn apply_thread_message_sent(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let Some(mut thread) = self.get_thread(&thread_id)? else {
            return Ok(());
        };
        let role = message_role_from_t3(&required_payload_string(&event.payload, "role")?)?;
        let message_id = required_payload_string(&event.payload, "messageId")?;
        let existing_message = self.get_message(&message_id)?;
        let payload_text = required_payload_string(&event.payload, "text")?;
        let streaming = event
            .payload
            .get("streaming")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let text = existing_message
            .as_ref()
            .map(|message| {
                if streaming {
                    format!("{}{}", message.text, payload_text)
                } else if payload_text.is_empty() {
                    message.text.clone()
                } else {
                    payload_text.clone()
                }
            })
            .unwrap_or(payload_text);
        let attachments = if event.payload.get("attachments").is_some() {
            payload_chat_attachments(&event.payload)?
        } else {
            existing_message
                .as_ref()
                .map(|message| message.attachments.clone())
                .unwrap_or_default()
        };
        let message = ChatMessage {
            id: message_id,
            role,
            text,
            attachments,
            turn_id: optional_payload_string(&event.payload, "turnId")?,
            created_at: existing_message
                .as_ref()
                .map(|message| message.created_at.clone())
                .unwrap_or(required_payload_string(&event.payload, "createdAt")?),
            completed_at: optional_payload_string(&event.payload, "updatedAt")?,
            streaming,
        };
        if role == MessageRole::User {
            thread.latest_user_message_at = Some(message.created_at.clone());
        }
        thread.updated_at = event.occurred_at.clone();
        self.upsert_thread(&thread)?;
        if role == MessageRole::Assistant {
            if let Some(turn_id) = message.turn_id.as_deref() {
                let existing_turn = self.get_projection_turn(&thread_id, turn_id)?;
                let state = if let Some(existing_turn) = existing_turn.as_ref() {
                    if message.streaming {
                        existing_turn.state.clone()
                    } else if existing_turn.state == "interrupted" || existing_turn.state == "error"
                    {
                        existing_turn.state.clone()
                    } else {
                        "completed".to_string()
                    }
                } else if message.streaming {
                    "running".to_string()
                } else {
                    "completed".to_string()
                };
                let completed_at = if message.streaming {
                    existing_turn
                        .as_ref()
                        .and_then(|turn| turn.completed_at.clone())
                } else {
                    existing_turn
                        .as_ref()
                        .and_then(|turn| turn.completed_at.clone())
                        .or_else(|| message.completed_at.clone())
                };
                self.upsert_projection_turn(&ProjectionTurnRow {
                    thread_id: thread_id.clone(),
                    turn_id: Some(turn_id.to_string()),
                    pending_message_id: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.pending_message_id.clone()),
                    source_proposed_plan_thread_id: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.source_proposed_plan_thread_id.clone()),
                    source_proposed_plan_id: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.source_proposed_plan_id.clone()),
                    state,
                    requested_at: existing_turn
                        .as_ref()
                        .map(|turn| turn.requested_at.clone())
                        .unwrap_or_else(|| message.created_at.clone()),
                    started_at: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.started_at.clone())
                        .or_else(|| Some(message.created_at.clone())),
                    completed_at,
                    assistant_message_id: Some(message.id.clone()),
                    checkpoint_turn_count: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.checkpoint_turn_count),
                    checkpoint_ref: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.checkpoint_ref.clone()),
                    checkpoint_status: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.checkpoint_status.clone()),
                    checkpoint_files: existing_turn
                        .as_ref()
                        .map(|turn| turn.checkpoint_files.clone())
                        .unwrap_or_default(),
                })?;
                self.set_thread_latest_turn(&thread_id, Some(turn_id))?;
            }
        }
        self.upsert_message(&thread_id, &message)
    }

    fn apply_thread_turn_start_requested(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let source_proposed_plan = event.payload.get("sourceProposedPlan");
        self.replace_pending_turn_start(&ProjectionTurnRow {
            thread_id,
            turn_id: None,
            pending_message_id: Some(required_payload_string(&event.payload, "messageId")?),
            source_proposed_plan_thread_id: source_proposed_plan
                .and_then(|value| value.get("threadId"))
                .and_then(Value::as_str)
                .map(str::to_string),
            source_proposed_plan_id: source_proposed_plan
                .and_then(|value| value.get("planId"))
                .and_then(Value::as_str)
                .map(str::to_string),
            assistant_message_id: None,
            state: "pending".to_string(),
            requested_at: required_payload_string(&event.payload, "createdAt")?,
            started_at: None,
            completed_at: None,
            checkpoint_turn_count: None,
            checkpoint_ref: None,
            checkpoint_status: None,
            checkpoint_files: Vec::new(),
        })
    }

    fn apply_thread_session_set(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let session = required_payload_object(&event.payload, "session")?;
        let provider_name = optional_payload_string(session, "providerName")?
            .or(optional_payload_string(session, "provider")?)
            .unwrap_or_else(|| "codex".to_string());
        let active_turn_id = optional_payload_string(session, "activeTurnId")?;
        self.upsert_thread_session(&ProjectionThreadSessionRow {
            thread_id: thread_id.clone(),
            status: required_payload_string(session, "status")?,
            provider_name,
            provider_instance_id: optional_payload_string(session, "providerInstanceId")?,
            active_turn_id: active_turn_id.clone(),
            last_error: optional_payload_string(session, "lastError")?,
            updated_at: required_payload_string(session, "updatedAt")?,
        })?;
        if let Some(mut thread) = self.get_thread(&thread_id)? {
            thread.updated_at = event.occurred_at.clone();
            self.upsert_thread(&thread)?;
        }
        self.set_thread_latest_turn(&thread_id, active_turn_id.as_deref())?;
        if required_payload_string(session, "status")? == "running" {
            if let Some(turn_id) = active_turn_id.as_deref() {
                let existing_turn = self.get_projection_turn(&thread_id, turn_id)?;
                let pending_turn_start = self.get_pending_turn_start(&thread_id)?;
                let preserve_state = existing_turn
                    .as_ref()
                    .is_some_and(|turn| turn.state == "completed" || turn.state == "error");
                self.upsert_projection_turn(&ProjectionTurnRow {
                    thread_id: thread_id.clone(),
                    turn_id: Some(turn_id.to_string()),
                    pending_message_id: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.pending_message_id.clone())
                        .or_else(|| {
                            pending_turn_start
                                .as_ref()
                                .and_then(|turn| turn.pending_message_id.clone())
                        }),
                    source_proposed_plan_thread_id: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.source_proposed_plan_thread_id.clone())
                        .or_else(|| {
                            pending_turn_start
                                .as_ref()
                                .and_then(|turn| turn.source_proposed_plan_thread_id.clone())
                        }),
                    source_proposed_plan_id: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.source_proposed_plan_id.clone())
                        .or_else(|| {
                            pending_turn_start
                                .as_ref()
                                .and_then(|turn| turn.source_proposed_plan_id.clone())
                        }),
                    assistant_message_id: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.assistant_message_id.clone()),
                    state: if preserve_state {
                        existing_turn
                            .as_ref()
                            .map(|turn| turn.state.clone())
                            .unwrap_or_else(|| "running".to_string())
                    } else {
                        "running".to_string()
                    },
                    requested_at: existing_turn
                        .as_ref()
                        .map(|turn| turn.requested_at.clone())
                        .or_else(|| {
                            pending_turn_start
                                .as_ref()
                                .map(|turn| turn.requested_at.clone())
                        })
                        .unwrap_or_else(|| event.occurred_at.clone()),
                    started_at: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.started_at.clone())
                        .or_else(|| {
                            pending_turn_start
                                .as_ref()
                                .map(|turn| turn.requested_at.clone())
                        })
                        .or_else(|| Some(event.occurred_at.clone())),
                    completed_at: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.completed_at.clone()),
                    checkpoint_turn_count: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.checkpoint_turn_count),
                    checkpoint_ref: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.checkpoint_ref.clone()),
                    checkpoint_status: existing_turn
                        .as_ref()
                        .and_then(|turn| turn.checkpoint_status.clone()),
                    checkpoint_files: existing_turn
                        .as_ref()
                        .map(|turn| turn.checkpoint_files.clone())
                        .unwrap_or_default(),
                })?;
                self.delete_pending_turn_start(&thread_id)?;
            }
        }
        Ok(())
    }

    fn apply_thread_proposed_plan_upserted(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let proposed_plan = required_payload_object(&event.payload, "proposedPlan")?;
        let plan = ProposedPlan {
            id: required_payload_string(proposed_plan, "id")?,
            turn_id: optional_payload_string(proposed_plan, "turnId")?,
            plan_markdown: required_payload_string(proposed_plan, "planMarkdown")?,
            implemented_at: optional_payload_string(proposed_plan, "implementedAt")?,
            implementation_thread_id: optional_payload_string(
                proposed_plan,
                "implementationThreadId",
            )?,
            created_at: required_payload_string(proposed_plan, "createdAt")?,
            updated_at: required_payload_string(proposed_plan, "updatedAt")?,
        };
        let is_actionable = plan.implemented_at.is_none();
        self.upsert_proposed_plan(&thread_id, &plan)?;
        if let Some(mut thread) = self.get_thread(&thread_id)? {
            thread.updated_at = event.occurred_at.clone();
            thread.has_actionable_proposed_plan = is_actionable;
            self.upsert_thread(&thread)?;
        }
        Ok(())
    }

    fn apply_thread_turn_diff_completed(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let status = required_payload_string(&event.payload, "status")?;
        let summary = TurnDiffSummary {
            turn_id: required_payload_string(&event.payload, "turnId")?,
            completed_at: required_payload_string(&event.payload, "completedAt")?,
            status: Some(status.clone()),
            files: payload_turn_diff_files(&event.payload)?,
            checkpoint_ref: optional_payload_string(&event.payload, "checkpointRef")?,
            assistant_message_id: optional_payload_string(&event.payload, "assistantMessageId")?,
            checkpoint_turn_count: optional_payload_u32(&event.payload, "checkpointTurnCount")?,
        };
        let existing_turn = self.get_projection_turn(&thread_id, &summary.turn_id)?;
        self.upsert_projection_turn(&ProjectionTurnRow {
            thread_id: thread_id.clone(),
            turn_id: Some(summary.turn_id.clone()),
            pending_message_id: existing_turn
                .as_ref()
                .and_then(|turn| turn.pending_message_id.clone()),
            source_proposed_plan_thread_id: existing_turn
                .as_ref()
                .and_then(|turn| turn.source_proposed_plan_thread_id.clone()),
            source_proposed_plan_id: existing_turn
                .as_ref()
                .and_then(|turn| turn.source_proposed_plan_id.clone()),
            assistant_message_id: summary.assistant_message_id.clone(),
            state: if status == "error" {
                "error".to_string()
            } else {
                "completed".to_string()
            },
            requested_at: existing_turn
                .as_ref()
                .map(|turn| turn.requested_at.clone())
                .unwrap_or_else(|| summary.completed_at.clone()),
            started_at: existing_turn
                .as_ref()
                .and_then(|turn| turn.started_at.clone())
                .or_else(|| Some(summary.completed_at.clone())),
            completed_at: Some(summary.completed_at.clone()),
            checkpoint_turn_count: summary.checkpoint_turn_count,
            checkpoint_ref: summary.checkpoint_ref.clone(),
            checkpoint_status: summary.status.clone(),
            checkpoint_files: summary.files.clone(),
        })?;
        if let Some(mut thread) = self.get_thread(&thread_id)? {
            thread.updated_at = event.occurred_at.clone();
            self.upsert_thread(&thread)?;
        }
        self.set_thread_latest_turn(&thread_id, Some(&summary.turn_id))?;
        Ok(())
    }

    fn apply_thread_turn_interrupt_requested(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let Some(turn_id) = optional_payload_string(&event.payload, "turnId")? else {
            return Ok(());
        };
        let created_at = required_payload_string(&event.payload, "createdAt")?;
        let existing_turn = self.get_projection_turn(&thread_id, &turn_id)?;
        self.upsert_projection_turn(&ProjectionTurnRow {
            thread_id: thread_id.clone(),
            turn_id: Some(turn_id.clone()),
            pending_message_id: existing_turn
                .as_ref()
                .and_then(|turn| turn.pending_message_id.clone()),
            source_proposed_plan_thread_id: existing_turn
                .as_ref()
                .and_then(|turn| turn.source_proposed_plan_thread_id.clone()),
            source_proposed_plan_id: existing_turn
                .as_ref()
                .and_then(|turn| turn.source_proposed_plan_id.clone()),
            assistant_message_id: existing_turn
                .as_ref()
                .and_then(|turn| turn.assistant_message_id.clone()),
            state: "interrupted".to_string(),
            requested_at: existing_turn
                .as_ref()
                .map(|turn| turn.requested_at.clone())
                .unwrap_or_else(|| created_at.clone()),
            started_at: existing_turn
                .as_ref()
                .and_then(|turn| turn.started_at.clone())
                .or_else(|| Some(created_at.clone())),
            completed_at: existing_turn
                .as_ref()
                .and_then(|turn| turn.completed_at.clone())
                .or_else(|| Some(created_at)),
            checkpoint_turn_count: existing_turn
                .as_ref()
                .and_then(|turn| turn.checkpoint_turn_count),
            checkpoint_ref: existing_turn
                .as_ref()
                .and_then(|turn| turn.checkpoint_ref.clone()),
            checkpoint_status: existing_turn
                .as_ref()
                .and_then(|turn| turn.checkpoint_status.clone()),
            checkpoint_files: existing_turn
                .as_ref()
                .map(|turn| turn.checkpoint_files.clone())
                .unwrap_or_default(),
        })?;
        self.set_thread_latest_turn(&thread_id, Some(&turn_id))
    }

    fn apply_thread_reverted(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let turn_count = event
            .payload
            .get("turnCount")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                ProjectionPersistenceError::InvalidProjectionEventPayload(
                    "missing turnCount number".to_string(),
                )
            })? as u32;
        let kept_turns = self
            .list_projection_turns_by_thread(&thread_id)?
            .into_iter()
            .filter(|turn| {
                turn.turn_id.is_some()
                    && turn
                        .checkpoint_turn_count
                        .is_some_and(|checkpoint_turn_count| checkpoint_turn_count <= turn_count)
            })
            .collect::<Vec<_>>();
        self.delete_turns_by_thread(&thread_id)?;
        for turn in &kept_turns {
            self.upsert_projection_turn(turn)?;
        }
        self.prune_thread_detail_rows_after_revert(&thread_id, &kept_turns)?;
        let latest_turn_id = kept_turns
            .iter()
            .filter_map(|turn| Some((turn.checkpoint_turn_count?, turn.turn_id.as_deref()?)))
            .max_by_key(|(checkpoint_turn_count, _)| *checkpoint_turn_count)
            .map(|(_, turn_id)| turn_id.to_string());
        self.set_thread_latest_turn(&thread_id, latest_turn_id.as_deref())?;
        if let Some(mut thread) = self.get_thread(&thread_id)? {
            thread.updated_at = event.occurred_at.clone();
            self.upsert_thread(&thread)?;
            self.set_thread_latest_turn(&thread_id, latest_turn_id.as_deref())?;
        }
        Ok(())
    }

    fn apply_thread_activity_appended(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id = required_payload_string(&event.payload, "threadId")?;
        let activity_value = required_payload_object(&event.payload, "activity")?;
        let activity_payload = activity_value
            .get("payload")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let activity = ThreadActivity {
            id: required_payload_string(activity_value, "id")?,
            kind: required_payload_string(activity_value, "kind")?,
            summary: required_payload_string(activity_value, "summary")?,
            tone: activity_tone_from_t3(&required_payload_string(activity_value, "tone")?)?,
            payload: decode_activity_payload(&serde_json::to_string(&activity_payload)?)?,
            turn_id: optional_payload_string(activity_value, "turnId")?,
            sequence: optional_payload_i32(activity_value, "sequence")?,
            created_at: required_payload_string(activity_value, "createdAt")?,
        };
        self.upsert_activity(&thread_id, &activity)?;
        self.project_pending_approval_from_activity(&thread_id, &activity, event)?;
        Ok(())
    }

    fn apply_thread_approval_response_requested(
        &self,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let request_id = required_payload_string(&event.payload, "requestId")?;
        let existing = self.get_pending_approval(&request_id)?;
        let fallback_thread_id = required_payload_string(&event.payload, "threadId")?;
        let fallback_created_at = optional_payload_string(&event.payload, "createdAt")?
            .unwrap_or_else(|| event.occurred_at.clone());
        self.upsert_pending_approval(&ProjectionPendingApprovalRow {
            request_id,
            thread_id: existing
                .as_ref()
                .map(|row| row.thread_id.clone())
                .unwrap_or(fallback_thread_id),
            turn_id: existing.as_ref().and_then(|row| row.turn_id.clone()),
            status: "resolved".to_string(),
            decision: optional_payload_string(&event.payload, "decision")?,
            created_at: existing
                .as_ref()
                .map(|row| row.created_at.clone())
                .unwrap_or_else(|| fallback_created_at.clone()),
            resolved_at: Some(fallback_created_at),
        })
    }

    fn project_pending_approval_from_activity(
        &self,
        thread_id: &str,
        activity: &ThreadActivity,
        event: &OrchestrationEventRow,
    ) -> ProjectionPersistenceResult<()> {
        let request_id = activity.payload.request_id.clone().or_else(|| {
            event
                .metadata
                .get("requestId")
                .and_then(Value::as_str)
                .map(str::to_string)
        });
        let Some(request_id) = request_id else {
            return Ok(());
        };
        match activity.kind.as_str() {
            "approval.requested" => self.upsert_pending_approval(&ProjectionPendingApprovalRow {
                request_id,
                thread_id: thread_id.to_string(),
                turn_id: activity.turn_id.clone(),
                status: "pending".to_string(),
                decision: None,
                created_at: activity.created_at.clone(),
                resolved_at: None,
            }),
            "approval.resolved" => {
                let existing = self.get_pending_approval(&request_id)?;
                self.upsert_pending_approval(&ProjectionPendingApprovalRow {
                    request_id,
                    thread_id: existing
                        .as_ref()
                        .map(|row| row.thread_id.clone())
                        .unwrap_or_else(|| thread_id.to_string()),
                    turn_id: existing
                        .as_ref()
                        .and_then(|row| row.turn_id.clone())
                        .or_else(|| activity.turn_id.clone()),
                    status: "resolved".to_string(),
                    decision: activity.payload.detail.clone(),
                    created_at: existing
                        .as_ref()
                        .map(|row| row.created_at.clone())
                        .unwrap_or_else(|| activity.created_at.clone()),
                    resolved_at: Some(activity.created_at.clone()),
                })
            }
            "provider.approval.respond.failed"
                if is_stale_pending_approval_failure_detail(activity.payload.detail.as_deref()) =>
            {
                let Some(existing) = self.get_pending_approval(&request_id)? else {
                    return Ok(());
                };
                if existing.status == "resolved" {
                    return Ok(());
                }
                self.upsert_pending_approval(&ProjectionPendingApprovalRow {
                    request_id,
                    thread_id: existing.thread_id,
                    turn_id: existing.turn_id,
                    status: "resolved".to_string(),
                    decision: None,
                    created_at: existing.created_at,
                    resolved_at: Some(activity.created_at.clone()),
                })
            }
            _ => Ok(()),
        }
    }

    pub fn upsert_project(&self, row: &ProjectionProjectRow) -> ProjectionPersistenceResult<()> {
        let scripts_json = encode_project_scripts(&row.scripts)?;
        self.connection.execute(
            r#"
            INSERT INTO projection_projects (
              project_id,
              title,
              workspace_root,
              default_model_selection_json,
              scripts_json,
              created_at,
              updated_at,
              deleted_at
            )
            VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7)
            ON CONFLICT (project_id)
            DO UPDATE SET
              title = excluded.title,
              workspace_root = excluded.workspace_root,
              default_model_selection_json = excluded.default_model_selection_json,
              scripts_json = excluded.scripts_json,
              created_at = excluded.created_at,
              updated_at = excluded.updated_at,
              deleted_at = excluded.deleted_at
            "#,
            params![
                row.project_id.as_str(),
                row.title.as_str(),
                row.workspace_root.as_str(),
                scripts_json,
                row.created_at.as_str(),
                row.updated_at.as_str(),
                row.deleted_at.as_deref()
            ],
        )?;
        Ok(())
    }

    pub fn get_project(
        &self,
        project_id: &str,
    ) -> ProjectionPersistenceResult<Option<ProjectionProjectRow>> {
        let sql = project_select_sql("WHERE project_id = ?1");
        let mut statement = self.connection.prepare(&sql)?;
        let mut rows = statement.query(params![project_id])?;
        rows.next()?.map(project_row_from_sql).transpose()
    }

    pub fn list_projects(&self) -> ProjectionPersistenceResult<Vec<ProjectionProjectRow>> {
        let sql = project_select_sql("ORDER BY created_at ASC, project_id ASC");
        let mut statement = self.connection.prepare(&sql)?;
        let mut rows = statement.query([])?;
        collect_projection_rows(&mut rows, project_row_from_sql)
    }

    pub fn delete_project_by_id(&self, project_id: &str) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            "DELETE FROM projection_projects WHERE project_id = ?1",
            params![project_id],
        )?;
        Ok(())
    }

    pub fn upsert_thread(&self, row: &ProjectionThreadRow) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            r#"
            INSERT INTO projection_threads (
              thread_id,
              project_id,
              title,
              model_selection_json,
              runtime_mode,
              interaction_mode,
              branch,
              worktree_path,
              latest_turn_id,
              created_at,
              updated_at,
              archived_at,
              latest_user_message_at,
              pending_approval_count,
              pending_user_input_count,
              has_actionable_proposed_plan,
              deleted_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT (thread_id)
            DO UPDATE SET
              project_id = excluded.project_id,
              title = excluded.title,
              model_selection_json = excluded.model_selection_json,
              runtime_mode = excluded.runtime_mode,
              interaction_mode = excluded.interaction_mode,
              branch = excluded.branch,
              worktree_path = excluded.worktree_path,
              latest_turn_id = excluded.latest_turn_id,
              created_at = excluded.created_at,
              updated_at = excluded.updated_at,
              archived_at = excluded.archived_at,
              latest_user_message_at = excluded.latest_user_message_at,
              pending_approval_count = excluded.pending_approval_count,
              pending_user_input_count = excluded.pending_user_input_count,
              has_actionable_proposed_plan = excluded.has_actionable_proposed_plan,
              deleted_at = excluded.deleted_at
            "#,
            params![
                row.thread_id.as_str(),
                row.project_id.as_str(),
                row.title.as_str(),
                DEFAULT_MODEL_SELECTION_JSON,
                runtime_mode_to_t3(row.runtime_mode),
                interaction_mode_to_t3(row.interaction_mode),
                row.branch.as_deref(),
                row.worktree_path.as_deref(),
                Option::<String>::None,
                row.created_at.as_str(),
                row.updated_at.as_str(),
                row.archived_at.as_deref(),
                row.latest_user_message_at.as_deref(),
                row.pending_approval_count,
                row.pending_user_input_count,
                if row.has_actionable_proposed_plan {
                    1
                } else {
                    0
                },
                row.deleted_at.as_deref()
            ],
        )?;
        Ok(())
    }

    pub fn set_thread_latest_turn(
        &self,
        thread_id: &str,
        turn_id: Option<&str>,
    ) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            "UPDATE projection_threads SET latest_turn_id = ?2 WHERE thread_id = ?1",
            params![thread_id, turn_id],
        )?;
        Ok(())
    }

    pub fn get_thread(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Option<ProjectionThreadRow>> {
        let sql = thread_select_sql("WHERE thread_id = ?1");
        let mut statement = self.connection.prepare(&sql)?;
        let mut rows = statement.query(params![thread_id])?;
        rows.next()?.map(thread_row_from_sql).transpose()
    }

    pub fn list_threads_by_project(
        &self,
        project_id: &str,
    ) -> ProjectionPersistenceResult<Vec<ProjectionThreadRow>> {
        let sql = thread_select_sql("WHERE project_id = ?1 ORDER BY created_at ASC, thread_id ASC");
        let mut statement = self.connection.prepare(&sql)?;
        let mut rows = statement.query(params![project_id])?;
        collect_projection_rows(&mut rows, thread_row_from_sql)
    }

    pub fn list_threads(&self) -> ProjectionPersistenceResult<Vec<ProjectionThreadRow>> {
        let sql = thread_select_sql("ORDER BY created_at ASC, thread_id ASC");
        let mut statement = self.connection.prepare(&sql)?;
        let mut rows = statement.query([])?;
        collect_projection_rows(&mut rows, thread_row_from_sql)
    }

    pub fn delete_thread_by_id(&self, thread_id: &str) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            "DELETE FROM projection_threads WHERE thread_id = ?1",
            params![thread_id],
        )?;
        Ok(())
    }

    pub fn upsert_thread_session(
        &self,
        row: &ProjectionThreadSessionRow,
    ) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            r#"
            INSERT INTO projection_thread_sessions (
              thread_id,
              status,
              provider_name,
              provider_instance_id,
              provider_session_id,
              provider_thread_id,
              runtime_mode,
              active_turn_id,
              last_error,
              updated_at
            )
            VALUES (?1, ?2, ?3, ?4, NULL, NULL, 'full-access', ?5, ?6, ?7)
            ON CONFLICT (thread_id)
            DO UPDATE SET
              status = excluded.status,
              provider_name = excluded.provider_name,
              provider_instance_id = excluded.provider_instance_id,
              provider_session_id = excluded.provider_session_id,
              provider_thread_id = excluded.provider_thread_id,
              runtime_mode = excluded.runtime_mode,
              active_turn_id = excluded.active_turn_id,
              last_error = excluded.last_error,
              updated_at = excluded.updated_at
            "#,
            params![
                row.thread_id.as_str(),
                row.status.as_str(),
                row.provider_name.as_str(),
                row.provider_instance_id.as_deref(),
                row.active_turn_id.as_deref(),
                row.last_error.as_deref(),
                row.updated_at.as_str()
            ],
        )?;
        Ok(())
    }

    pub fn get_thread_session(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Option<ProjectionThreadSessionRow>> {
        let sql = thread_session_select_sql("WHERE sessions.thread_id = ?1");
        let mut statement = self.connection.prepare(&sql)?;
        let mut rows = statement.query(params![thread_id])?;
        rows.next()?.map(thread_session_row_from_sql).transpose()
    }

    pub fn delete_thread_session_by_thread_id(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            "DELETE FROM projection_thread_sessions WHERE thread_id = ?1",
            params![thread_id],
        )?;
        Ok(())
    }

    pub fn upsert_turn(&self, row: &ProjectionLatestTurnRow) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            r#"
            INSERT INTO projection_turns (
              thread_id,
              turn_id,
              pending_message_id,
              source_proposed_plan_thread_id,
              source_proposed_plan_id,
              assistant_message_id,
              state,
              requested_at,
              started_at,
              completed_at,
              checkpoint_turn_count,
              checkpoint_ref,
              checkpoint_status,
              checkpoint_files_json
            )
            VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, NULL, NULL, '[]')
            ON CONFLICT (thread_id, turn_id)
            DO UPDATE SET
              source_proposed_plan_thread_id = excluded.source_proposed_plan_thread_id,
              source_proposed_plan_id = excluded.source_proposed_plan_id,
              assistant_message_id = excluded.assistant_message_id,
              state = excluded.state,
              requested_at = excluded.requested_at,
              started_at = excluded.started_at,
              completed_at = excluded.completed_at
            "#,
            params![
                row.thread_id.as_str(),
                row.turn_id.as_str(),
                row.source_proposed_plan_thread_id.as_deref(),
                row.source_proposed_plan_id.as_deref(),
                row.assistant_message_id.as_deref(),
                row.state.as_str(),
                row.requested_at.as_str(),
                row.started_at.as_deref(),
                row.completed_at.as_deref()
            ],
        )?;
        Ok(())
    }

    fn upsert_projection_turn(&self, row: &ProjectionTurnRow) -> ProjectionPersistenceResult<()> {
        let checkpoint_files_json = encode_turn_diff_files(&row.checkpoint_files)?;
        self.connection.execute(
            r#"
            INSERT INTO projection_turns (
              thread_id,
              turn_id,
              pending_message_id,
              source_proposed_plan_thread_id,
              source_proposed_plan_id,
              assistant_message_id,
              state,
              requested_at,
              started_at,
              completed_at,
              checkpoint_turn_count,
              checkpoint_ref,
              checkpoint_status,
              checkpoint_files_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT (thread_id, turn_id)
            DO UPDATE SET
              pending_message_id = excluded.pending_message_id,
              source_proposed_plan_thread_id = excluded.source_proposed_plan_thread_id,
              source_proposed_plan_id = excluded.source_proposed_plan_id,
              assistant_message_id = excluded.assistant_message_id,
              state = excluded.state,
              requested_at = excluded.requested_at,
              started_at = excluded.started_at,
              completed_at = excluded.completed_at,
              checkpoint_turn_count = excluded.checkpoint_turn_count,
              checkpoint_ref = excluded.checkpoint_ref,
              checkpoint_status = excluded.checkpoint_status,
              checkpoint_files_json = excluded.checkpoint_files_json
            "#,
            params![
                row.thread_id.as_str(),
                row.turn_id.as_deref(),
                row.pending_message_id.as_deref(),
                row.source_proposed_plan_thread_id.as_deref(),
                row.source_proposed_plan_id.as_deref(),
                row.assistant_message_id.as_deref(),
                row.state.as_str(),
                row.requested_at.as_str(),
                row.started_at.as_deref(),
                row.completed_at.as_deref(),
                row.checkpoint_turn_count,
                row.checkpoint_ref.as_deref(),
                row.checkpoint_status.as_deref(),
                checkpoint_files_json
            ],
        )?;
        Ok(())
    }

    fn replace_pending_turn_start(
        &self,
        row: &ProjectionTurnRow,
    ) -> ProjectionPersistenceResult<()> {
        self.delete_pending_turn_start(&row.thread_id)?;
        self.upsert_projection_turn(row)
    }

    fn get_projection_turn(
        &self,
        thread_id: &str,
        turn_id: &str,
    ) -> ProjectionPersistenceResult<Option<ProjectionTurnRow>> {
        let mut statement = self.connection.prepare(&format!(
            "{} WHERE thread_id = ?1 AND turn_id = ?2",
            projection_turn_select_sql()
        ))?;
        let mut rows = statement.query(params![thread_id, turn_id])?;
        rows.next()?.map(projection_turn_row_from_sql).transpose()
    }

    fn get_pending_turn_start(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Option<ProjectionTurnRow>> {
        let mut statement = self.connection.prepare(&format!(
            "{} WHERE thread_id = ?1 AND turn_id IS NULL ORDER BY requested_at DESC LIMIT 1",
            projection_turn_select_sql()
        ))?;
        let mut rows = statement.query(params![thread_id])?;
        rows.next()?.map(projection_turn_row_from_sql).transpose()
    }

    fn list_projection_turns_by_thread(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Vec<ProjectionTurnRow>> {
        let mut statement = self.connection.prepare(&format!(
            "{} WHERE thread_id = ?1 ORDER BY requested_at ASC, row_id ASC",
            projection_turn_select_sql()
        ))?;
        let mut rows = statement.query(params![thread_id])?;
        collect_projection_rows(&mut rows, projection_turn_row_from_sql)
    }

    fn delete_pending_turn_start(&self, thread_id: &str) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            "DELETE FROM projection_turns WHERE thread_id = ?1 AND turn_id IS NULL",
            params![thread_id],
        )?;
        Ok(())
    }

    fn delete_turns_by_thread(&self, thread_id: &str) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            "DELETE FROM projection_turns WHERE thread_id = ?1",
            params![thread_id],
        )?;
        Ok(())
    }

    fn prune_thread_detail_rows_after_revert(
        &self,
        thread_id: &str,
        kept_turns: &[ProjectionTurnRow],
    ) -> ProjectionPersistenceResult<()> {
        let kept_turn_ids = kept_turns
            .iter()
            .filter_map(|turn| turn.turn_id.as_deref())
            .collect::<Vec<_>>();
        if kept_turn_ids.is_empty() {
            self.connection.execute(
                "DELETE FROM projection_thread_messages WHERE thread_id = ?1 AND role != 'system'",
                params![thread_id],
            )?;
            self.connection.execute(
                "DELETE FROM projection_thread_activities WHERE thread_id = ?1",
                params![thread_id],
            )?;
            self.connection.execute(
                "DELETE FROM projection_thread_proposed_plans WHERE thread_id = ?1",
                params![thread_id],
            )?;
            return Ok(());
        }

        let placeholders = std::iter::repeat_n("?", kept_turn_ids.len())
            .collect::<Vec<_>>()
            .join(", ");
        let mut message_params: Vec<&dyn rusqlite::ToSql> =
            Vec::with_capacity(1 + kept_turn_ids.len());
        message_params.push(&thread_id);
        message_params.extend(
            kept_turn_ids
                .iter()
                .map(|turn_id| turn_id as &dyn rusqlite::ToSql),
        );
        self.connection.execute(
            &format!(
                "DELETE FROM projection_thread_messages
                 WHERE thread_id = ? AND role != 'system'
                   AND (turn_id IS NULL OR turn_id NOT IN ({placeholders}))"
            ),
            message_params.as_slice(),
        )?;

        let mut activity_params: Vec<&dyn rusqlite::ToSql> =
            Vec::with_capacity(1 + kept_turn_ids.len());
        activity_params.push(&thread_id);
        activity_params.extend(
            kept_turn_ids
                .iter()
                .map(|turn_id| turn_id as &dyn rusqlite::ToSql),
        );
        self.connection.execute(
            &format!(
                "DELETE FROM projection_thread_activities
                 WHERE thread_id = ?
                   AND (turn_id IS NULL OR turn_id NOT IN ({placeholders}))"
            ),
            activity_params.as_slice(),
        )?;

        let mut plan_params: Vec<&dyn rusqlite::ToSql> =
            Vec::with_capacity(1 + kept_turn_ids.len());
        plan_params.push(&thread_id);
        plan_params.extend(
            kept_turn_ids
                .iter()
                .map(|turn_id| turn_id as &dyn rusqlite::ToSql),
        );
        self.connection.execute(
            &format!(
                "DELETE FROM projection_thread_proposed_plans
                 WHERE thread_id = ?
                   AND (turn_id IS NULL OR turn_id NOT IN ({placeholders}))"
            ),
            plan_params.as_slice(),
        )?;
        Ok(())
    }

    pub fn upsert_message(
        &self,
        thread_id: &str,
        message: &ChatMessage,
    ) -> ProjectionPersistenceResult<()> {
        let attachments_json = if message.attachments.is_empty() {
            None
        } else {
            Some(encode_chat_attachments(&message.attachments)?)
        };
        let updated_at = message
            .completed_at
            .as_deref()
            .unwrap_or(&message.created_at);
        self.connection.execute(
            r#"
            INSERT INTO projection_thread_messages (
              message_id,
              thread_id,
              turn_id,
              role,
              text,
              attachments_json,
              is_streaming,
              created_at,
              updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT (message_id)
            DO UPDATE SET
              thread_id = excluded.thread_id,
              turn_id = excluded.turn_id,
              role = excluded.role,
              text = excluded.text,
              attachments_json = excluded.attachments_json,
              is_streaming = excluded.is_streaming,
              created_at = excluded.created_at,
              updated_at = excluded.updated_at
            "#,
            params![
                message.id.as_str(),
                thread_id,
                message.turn_id.as_deref(),
                message_role_to_t3(message.role),
                message.text.as_str(),
                attachments_json.as_deref(),
                if message.streaming { 1 } else { 0 },
                message.created_at.as_str(),
                updated_at
            ],
        )?;
        Ok(())
    }

    pub fn get_message(
        &self,
        message_id: &str,
    ) -> ProjectionPersistenceResult<Option<ChatMessage>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              message_id,
              role,
              text,
              attachments_json,
              turn_id,
              created_at,
              updated_at,
              is_streaming
            FROM projection_thread_messages
            WHERE message_id = ?1
            "#,
        )?;
        let mut rows = statement.query(params![message_id])?;
        rows.next()?.map(chat_message_row_from_sql).transpose()
    }

    pub fn upsert_activity(
        &self,
        thread_id: &str,
        activity: &ThreadActivity,
    ) -> ProjectionPersistenceResult<()> {
        let payload_json = encode_activity_payload(&activity.payload)?;
        self.connection.execute(
            r#"
            INSERT INTO projection_thread_activities (
              activity_id,
              thread_id,
              turn_id,
              tone,
              kind,
              summary,
              payload_json,
              sequence,
              created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT (activity_id)
            DO UPDATE SET
              thread_id = excluded.thread_id,
              turn_id = excluded.turn_id,
              tone = excluded.tone,
              kind = excluded.kind,
              summary = excluded.summary,
              payload_json = excluded.payload_json,
              sequence = excluded.sequence,
              created_at = excluded.created_at
            "#,
            params![
                activity.id.as_str(),
                thread_id,
                activity.turn_id.as_deref(),
                activity_tone_to_t3(activity.tone),
                activity.kind.as_str(),
                activity.summary.as_str(),
                payload_json,
                activity.sequence,
                activity.created_at.as_str()
            ],
        )?;
        self.refresh_thread_pending_counts(thread_id)?;
        Ok(())
    }

    pub fn upsert_proposed_plan(
        &self,
        thread_id: &str,
        plan: &ProposedPlan,
    ) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            r#"
            INSERT INTO projection_thread_proposed_plans (
              plan_id,
              thread_id,
              turn_id,
              plan_markdown,
              implemented_at,
              implementation_thread_id,
              created_at,
              updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT (plan_id)
            DO UPDATE SET
              thread_id = excluded.thread_id,
              turn_id = excluded.turn_id,
              plan_markdown = excluded.plan_markdown,
              implemented_at = excluded.implemented_at,
              implementation_thread_id = excluded.implementation_thread_id,
              created_at = excluded.created_at,
              updated_at = excluded.updated_at
            "#,
            params![
                plan.id.as_str(),
                thread_id,
                plan.turn_id.as_deref(),
                plan.plan_markdown.as_str(),
                plan.implemented_at.as_deref(),
                plan.implementation_thread_id.as_deref(),
                plan.created_at.as_str(),
                plan.updated_at.as_str()
            ],
        )?;
        Ok(())
    }

    pub fn upsert_checkpoint_summary(
        &self,
        thread_id: &str,
        summary: &TurnDiffSummary,
    ) -> ProjectionPersistenceResult<()> {
        let files_json = encode_turn_diff_files(&summary.files)?;
        self.connection.execute(
            r#"
            INSERT INTO projection_turns (
              thread_id,
              turn_id,
              pending_message_id,
              source_proposed_plan_thread_id,
              source_proposed_plan_id,
              assistant_message_id,
              state,
              requested_at,
              started_at,
              completed_at,
              checkpoint_turn_count,
              checkpoint_ref,
              checkpoint_status,
              checkpoint_files_json
            )
            VALUES (?1, ?2, NULL, NULL, NULL, ?3, ?4, ?5, NULL, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT (thread_id, turn_id)
            DO UPDATE SET
              assistant_message_id = excluded.assistant_message_id,
              completed_at = excluded.completed_at,
              checkpoint_turn_count = excluded.checkpoint_turn_count,
              checkpoint_ref = excluded.checkpoint_ref,
              checkpoint_status = excluded.checkpoint_status,
              checkpoint_files_json = excluded.checkpoint_files_json
            "#,
            params![
                thread_id,
                summary.turn_id.as_str(),
                summary.assistant_message_id.as_deref(),
                summary.status.as_deref().unwrap_or("completed"),
                summary.completed_at.as_str(),
                summary.checkpoint_turn_count,
                summary.checkpoint_ref.as_deref(),
                summary.status.as_deref(),
                files_json
            ],
        )?;
        Ok(())
    }

    pub fn upsert_pending_approval(
        &self,
        row: &ProjectionPendingApprovalRow,
    ) -> ProjectionPersistenceResult<()> {
        self.connection.execute(
            r#"
            INSERT INTO projection_pending_approvals (
              request_id,
              thread_id,
              turn_id,
              status,
              decision,
              created_at,
              resolved_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT (request_id)
            DO UPDATE SET
              thread_id = excluded.thread_id,
              turn_id = excluded.turn_id,
              status = excluded.status,
              decision = excluded.decision,
              created_at = excluded.created_at,
              resolved_at = excluded.resolved_at
            "#,
            params![
                row.request_id.as_str(),
                row.thread_id.as_str(),
                row.turn_id.as_deref(),
                row.status.as_str(),
                row.decision.as_deref(),
                row.created_at.as_str(),
                row.resolved_at.as_deref()
            ],
        )?;
        self.refresh_thread_pending_counts(&row.thread_id)?;
        Ok(())
    }

    pub fn list_pending_approvals_by_thread(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Vec<ProjectionPendingApprovalRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              request_id,
              thread_id,
              turn_id,
              status,
              decision,
              created_at,
              resolved_at
            FROM projection_pending_approvals
            WHERE thread_id = ?1
            ORDER BY created_at ASC, request_id ASC
            "#,
        )?;
        let mut rows = statement.query(params![thread_id])?;
        collect_projection_rows(&mut rows, pending_approval_row_from_sql)
    }

    pub fn get_pending_approval(
        &self,
        request_id: &str,
    ) -> ProjectionPersistenceResult<Option<ProjectionPendingApprovalRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              request_id,
              thread_id,
              turn_id,
              status,
              decision,
              created_at,
              resolved_at
            FROM projection_pending_approvals
            WHERE request_id = ?1
            "#,
        )?;
        let mut rows = statement.query(params![request_id])?;
        rows.next()?.map(pending_approval_row_from_sql).transpose()
    }

    pub fn delete_pending_approval_by_request_id(
        &self,
        request_id: &str,
    ) -> ProjectionPersistenceResult<()> {
        let thread_id: Option<String> = self
            .connection
            .query_row(
                "SELECT thread_id FROM projection_pending_approvals WHERE request_id = ?1",
                params![request_id],
                |row| row.get(0),
            )
            .unwrap_or(None);
        self.connection.execute(
            "DELETE FROM projection_pending_approvals WHERE request_id = ?1",
            params![request_id],
        )?;
        if let Some(thread_id) = thread_id {
            self.refresh_thread_pending_counts(&thread_id)?;
        }
        Ok(())
    }

    pub fn refresh_thread_pending_counts(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<()> {
        let pending_approval_count: u32 = self.connection.query_row(
            r#"
            SELECT COUNT(*)
            FROM projection_pending_approvals
            WHERE thread_id = ?1
              AND status = 'pending'
            "#,
            params![thread_id],
            |row| row.get(0),
        )?;
        let pending_user_input_count =
            derive_pending_user_inputs(&self.list_activities_by_thread(thread_id)?).len() as u32;
        self.connection.execute(
            r#"
            UPDATE projection_threads
            SET
              pending_approval_count = ?2,
              pending_user_input_count = ?3
            WHERE thread_id = ?1
            "#,
            params![thread_id, pending_approval_count, pending_user_input_count],
        )?;
        Ok(())
    }

    pub fn load_thread_detail(
        &self,
        environment_id: impl Into<String>,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Option<Thread>> {
        let shell =
            crate::build_projection_shell_snapshot(self.load_shell_snapshot_input(environment_id)?);
        let mut state = shell.environment_state;
        self.load_thread_detail_into_state(&mut state, thread_id)?;
        Ok(get_thread_from_environment_state(&state, thread_id))
    }

    pub fn load_shell_snapshot_input(
        &self,
        environment_id: impl Into<String>,
    ) -> ProjectionPersistenceResult<ProjectionShellSnapshotInput> {
        Ok(ProjectionShellSnapshotInput {
            environment_id: environment_id.into(),
            projects: self.list_projects()?,
            threads: self.list_threads()?,
            sessions: self.list_shell_sessions()?,
            latest_turns: self.list_shell_latest_turns()?,
        })
    }

    fn list_shell_sessions(&self) -> ProjectionPersistenceResult<Vec<ProjectionThreadSessionRow>> {
        let mut statement = self.connection.prepare(&format!(
            "{} INNER JOIN projection_threads threads ON threads.thread_id = sessions.thread_id
            WHERE threads.deleted_at IS NULL
            ORDER BY sessions.thread_id ASC",
            thread_session_select_sql_from("projection_thread_sessions sessions")
        ))?;
        let mut rows = statement.query([])?;
        collect_projection_rows(&mut rows, thread_session_row_from_sql)
    }

    fn list_shell_latest_turns(&self) -> ProjectionPersistenceResult<Vec<ProjectionLatestTurnRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              turns.thread_id,
              turns.turn_id,
              turns.state,
              turns.requested_at,
              turns.started_at,
              turns.completed_at,
              turns.assistant_message_id,
              turns.source_proposed_plan_thread_id,
              turns.source_proposed_plan_id
            FROM projection_threads threads
            JOIN projection_turns turns
              ON turns.thread_id = threads.thread_id
              AND turns.turn_id = threads.latest_turn_id
            WHERE threads.deleted_at IS NULL
              AND threads.latest_turn_id IS NOT NULL
            ORDER BY turns.thread_id ASC
            "#,
        )?;
        let mut rows = statement.query([])?;
        collect_projection_rows(&mut rows, latest_turn_row_from_sql)
    }

    fn load_thread_detail_into_state(
        &self,
        state: &mut EnvironmentState,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<()> {
        let messages = self.list_messages_by_thread(thread_id)?;
        state.message_ids_by_thread_id.insert(
            thread_id.to_string(),
            messages.iter().map(|message| message.id.clone()).collect(),
        );
        state.message_by_thread_id.insert(
            thread_id.to_string(),
            messages
                .into_iter()
                .map(|message| (message.id.clone(), message))
                .collect(),
        );

        let activities = self.list_activities_by_thread(thread_id)?;
        state.activity_ids_by_thread_id.insert(
            thread_id.to_string(),
            activities
                .iter()
                .map(|activity| activity.id.clone())
                .collect(),
        );
        state.activity_by_thread_id.insert(
            thread_id.to_string(),
            activities
                .into_iter()
                .map(|activity| (activity.id.clone(), activity))
                .collect(),
        );

        let plans = self.list_proposed_plans_by_thread(thread_id)?;
        state.proposed_plan_ids_by_thread_id.insert(
            thread_id.to_string(),
            plans.iter().map(|plan| plan.id.clone()).collect(),
        );
        state.proposed_plan_by_thread_id.insert(
            thread_id.to_string(),
            plans
                .into_iter()
                .map(|plan| (plan.id.clone(), plan))
                .collect(),
        );

        let summaries = self.list_checkpoint_summaries_by_thread(thread_id)?;
        state.turn_diff_ids_by_thread_id.insert(
            thread_id.to_string(),
            summaries
                .iter()
                .map(|summary| summary.turn_id.clone())
                .collect(),
        );
        state.turn_diff_summary_by_thread_id.insert(
            thread_id.to_string(),
            summaries
                .into_iter()
                .map(|summary| (summary.turn_id.clone(), summary))
                .collect(),
        );
        Ok(())
    }

    fn list_messages_by_thread(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Vec<ChatMessage>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              message_id,
              role,
              text,
              attachments_json,
              turn_id,
              created_at,
              updated_at,
              is_streaming
            FROM projection_thread_messages
            WHERE thread_id = ?1
            ORDER BY created_at ASC, message_id ASC
            "#,
        )?;
        let mut rows = statement.query(params![thread_id])?;
        collect_projection_rows(&mut rows, chat_message_row_from_sql)
    }

    fn list_activities_by_thread(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Vec<ThreadActivity>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              activity_id,
              kind,
              summary,
              tone,
              payload_json,
              turn_id,
              sequence,
              created_at
            FROM projection_thread_activities
            WHERE thread_id = ?1
            ORDER BY sequence ASC, created_at ASC, activity_id ASC
            "#,
        )?;
        let mut rows = statement.query(params![thread_id])?;
        collect_projection_rows(&mut rows, thread_activity_row_from_sql)
    }

    fn list_proposed_plans_by_thread(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Vec<ProposedPlan>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              plan_id,
              turn_id,
              plan_markdown,
              implemented_at,
              implementation_thread_id,
              created_at,
              updated_at
            FROM projection_thread_proposed_plans
            WHERE thread_id = ?1
            ORDER BY created_at ASC, plan_id ASC
            "#,
        )?;
        let mut rows = statement.query(params![thread_id])?;
        collect_projection_rows(&mut rows, proposed_plan_row_from_sql)
    }

    fn list_checkpoint_summaries_by_thread(
        &self,
        thread_id: &str,
    ) -> ProjectionPersistenceResult<Vec<TurnDiffSummary>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
              turn_id,
              completed_at,
              checkpoint_status,
              checkpoint_files_json,
              checkpoint_ref,
              assistant_message_id,
              checkpoint_turn_count
            FROM projection_turns
            WHERE thread_id = ?1
              AND checkpoint_turn_count IS NOT NULL
            ORDER BY checkpoint_turn_count ASC
            "#,
        )?;
        let mut rows = statement.query(params![thread_id])?;
        collect_projection_rows(&mut rows, turn_diff_summary_row_from_sql)
    }
}

fn collect_projection_rows<T>(
    rows: &mut Rows<'_>,
    mapper: fn(&Row<'_>) -> ProjectionPersistenceResult<T>,
) -> ProjectionPersistenceResult<Vec<T>> {
    let mut output = Vec::new();
    while let Some(row) = rows.next()? {
        output.push(mapper(row)?);
    }
    Ok(output)
}

fn orchestration_event_select_sql() -> &'static str {
    r#"
    SELECT
      sequence,
      stream_version,
      event_id,
      aggregate_kind,
      stream_id,
      event_type,
      occurred_at,
      command_id,
      causation_event_id,
      correlation_id,
      actor_kind,
      payload_json,
      metadata_json
    FROM orchestration_events
    "#
}

fn orchestration_event_row_from_sql(
    row: &Row<'_>,
) -> ProjectionPersistenceResult<OrchestrationEventRow> {
    let payload_json: String = row.get(11)?;
    let metadata_json: String = row.get(12)?;
    Ok(OrchestrationEventRow {
        sequence: row.get(0)?,
        stream_version: row.get(1)?,
        event_id: row.get(2)?,
        aggregate_kind: row.get(3)?,
        aggregate_id: row.get(4)?,
        event_type: row.get(5)?,
        occurred_at: row.get(6)?,
        command_id: row.get(7)?,
        causation_event_id: row.get(8)?,
        correlation_id: row.get(9)?,
        actor_kind: row.get(10)?,
        payload: serde_json::from_str(&payload_json)?,
        metadata: serde_json::from_str(&metadata_json)?,
    })
}

fn command_receipt_row_from_sql(
    row: &Row<'_>,
) -> ProjectionPersistenceResult<OrchestrationCommandReceiptRow> {
    Ok(OrchestrationCommandReceiptRow {
        command_id: row.get(0)?,
        aggregate_kind: row.get(1)?,
        aggregate_id: row.get(2)?,
        accepted_at: row.get(3)?,
        result_sequence: row.get(4)?,
        status: row.get(5)?,
        error: row.get(6)?,
    })
}

fn provider_session_runtime_row_from_sql(
    row: &Row<'_>,
) -> ProjectionPersistenceResult<ProviderSessionRuntimeRow> {
    let runtime_mode: String = row.get(4)?;
    let resume_cursor_json: Option<String> = row.get(7)?;
    let runtime_payload_json: Option<String> = row.get(8)?;
    Ok(ProviderSessionRuntimeRow {
        thread_id: row.get(0)?,
        provider_name: row.get(1)?,
        provider_instance_id: row.get(2)?,
        adapter_key: row.get(3)?,
        runtime_mode: runtime_mode_from_t3(&runtime_mode)?,
        status: row.get(5)?,
        last_seen_at: row.get(6)?,
        resume_cursor: resume_cursor_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?,
        runtime_payload: runtime_payload_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?,
    })
}

fn auth_session_row_from_sql(row: &Row<'_>) -> ProjectionPersistenceResult<AuthSessionRow> {
    Ok(AuthSessionRow {
        session_id: row.get(0)?,
        subject: row.get(1)?,
        role: row.get(2)?,
        method: row.get(3)?,
        client: AuthSessionClientMetadataRow {
            label: row.get(4)?,
            ip_address: row.get(5)?,
            user_agent: row.get(6)?,
            device_type: row.get(7)?,
            os: row.get(8)?,
            browser: row.get(9)?,
        },
        issued_at: row.get(10)?,
        expires_at: row.get(11)?,
        last_connected_at: row.get(12)?,
        revoked_at: row.get(13)?,
    })
}

fn auth_pairing_link_row_from_sql(
    row: &Row<'_>,
) -> ProjectionPersistenceResult<AuthPairingLinkRow> {
    Ok(AuthPairingLinkRow {
        id: row.get(0)?,
        credential: row.get(1)?,
        method: row.get(2)?,
        role: row.get(3)?,
        subject: row.get(4)?,
        label: row.get(5)?,
        created_at: row.get(6)?,
        expires_at: row.get(7)?,
        consumed_at: row.get(8)?,
        revoked_at: row.get(9)?,
    })
}

fn provider_runtime_binding_from_session_runtime(
    row: &ProviderSessionRuntimeRow,
) -> ProjectionPersistenceResult<ProviderRuntimeBinding> {
    if row.thread_id.trim().is_empty() {
        return Err(ProjectionPersistenceError::InvalidProviderRuntimeBinding(
            "threadId must be a non-empty string.".to_string(),
        ));
    }
    Ok(ProviderRuntimeBinding {
        thread_id: row.thread_id.clone(),
        provider: row.provider_name.clone(),
        provider_instance_id: Some(
            row.provider_instance_id
                .clone()
                .unwrap_or_else(|| row.provider_name.clone()),
        ),
        adapter_key: Some(row.adapter_key.clone()),
        status: Some(row.status.clone()),
        resume_cursor: row.resume_cursor.clone(),
        runtime_payload: row.runtime_payload.clone(),
        runtime_mode: Some(row.runtime_mode),
    })
}

fn merge_provider_runtime_payload(existing: Option<Value>, next: Option<Value>) -> Option<Value> {
    match (existing, next) {
        (existing, None) => existing,
        (Some(Value::Object(mut existing)), Some(Value::Object(next))) => {
            existing.extend(next);
            Some(Value::Object(existing))
        }
        (_, Some(Value::Null)) => None,
        (_, Some(next)) => Some(next),
    }
}

fn planned_event_to_new_row(event: PlannedOrchestrationEvent) -> NewOrchestrationEventRow {
    NewOrchestrationEventRow {
        event_id: event.event_id,
        aggregate_kind: event.aggregate_kind,
        aggregate_id: event.aggregate_id,
        event_type: event.event_type,
        occurred_at: event.occurred_at,
        command_id: event.command_id,
        causation_event_id: event.causation_event_id,
        correlation_id: event.correlation_id,
        payload: event.payload,
        metadata: event.metadata,
    }
}

fn planned_event_from_row(event: &OrchestrationEventRow) -> PlannedOrchestrationEvent {
    PlannedOrchestrationEvent {
        event_id: event.event_id.clone(),
        aggregate_kind: event.aggregate_kind.clone(),
        aggregate_id: event.aggregate_id.clone(),
        event_type: event.event_type.clone(),
        occurred_at: event.occurred_at.clone(),
        command_id: event.command_id.clone(),
        causation_event_id: event.causation_event_id.clone(),
        correlation_id: event.correlation_id.clone(),
        payload: event.payload.clone(),
        metadata: event.metadata.clone(),
    }
}

fn projection_state_row_from_sql(row: &Row<'_>) -> ProjectionPersistenceResult<ProjectionStateRow> {
    Ok(ProjectionStateRow {
        projector: row.get(0)?,
        last_applied_sequence: row.get(1)?,
        updated_at: row.get(2)?,
    })
}

fn infer_actor_kind(event: &NewOrchestrationEventRow) -> &'static str {
    if event
        .command_id
        .as_deref()
        .map(|command_id| command_id.starts_with("provider:"))
        .unwrap_or(false)
    {
        return "provider";
    }
    if event
        .command_id
        .as_deref()
        .map(|command_id| command_id.starts_with("server:"))
        .unwrap_or(false)
    {
        return "server";
    }
    if event.metadata.get("providerTurnId").is_some()
        || event.metadata.get("providerItemId").is_some()
        || event.metadata.get("adapterKey").is_some()
    {
        return "provider";
    }
    if event.command_id.is_none() {
        return "server";
    }
    "client"
}

fn required_payload_string(value: &Value, key: &str) -> ProjectionPersistenceResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            ProjectionPersistenceError::InvalidProjectionEventPayload(format!(
                "missing {key} string"
            ))
        })
}

fn optional_payload_string(
    value: &Value,
    key: &str,
) -> ProjectionPersistenceResult<Option<String>> {
    let Some(field) = value.get(key) else {
        return Ok(None);
    };
    if field.is_null() {
        return Ok(None);
    }
    field
        .as_str()
        .map(|value| Some(value.to_string()))
        .ok_or_else(|| {
            ProjectionPersistenceError::InvalidProjectionEventPayload(format!(
                "{key} must be a string"
            ))
        })
}

fn required_payload_object<'a>(
    value: &'a Value,
    key: &str,
) -> ProjectionPersistenceResult<&'a Value> {
    value
        .get(key)
        .filter(|field| field.is_object())
        .ok_or_else(|| {
            ProjectionPersistenceError::InvalidProjectionEventPayload(format!(
                "missing {key} object"
            ))
        })
}

fn optional_payload_u32(value: &Value, key: &str) -> ProjectionPersistenceResult<Option<u32>> {
    let Some(field) = value.get(key) else {
        return Ok(None);
    };
    if field.is_null() {
        return Ok(None);
    }
    field
        .as_u64()
        .map(|value| Some(value as u32))
        .ok_or_else(|| {
            ProjectionPersistenceError::InvalidProjectionEventPayload(format!(
                "{key} must be a number"
            ))
        })
}

fn optional_payload_i32(value: &Value, key: &str) -> ProjectionPersistenceResult<Option<i32>> {
    let Some(field) = value.get(key) else {
        return Ok(None);
    };
    if field.is_null() {
        return Ok(None);
    }
    field
        .as_i64()
        .map(|value| Some(value as i32))
        .ok_or_else(|| {
            ProjectionPersistenceError::InvalidProjectionEventPayload(format!(
                "{key} must be a number"
            ))
        })
}

fn payload_project_scripts(value: &Value) -> ProjectionPersistenceResult<Vec<ProjectScript>> {
    let scripts = value
        .get("scripts")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    decode_project_scripts(&serde_json::to_string(&scripts)?)
}

fn payload_chat_attachments(value: &Value) -> ProjectionPersistenceResult<Vec<ChatAttachment>> {
    let Some(attachments) = value.get("attachments") else {
        return Ok(Vec::new());
    };
    if attachments.is_null() {
        return Ok(Vec::new());
    }
    decode_chat_attachments(&serde_json::to_string(attachments)?)
}

fn payload_turn_diff_files(value: &Value) -> ProjectionPersistenceResult<Vec<TurnDiffFileChange>> {
    let files = value
        .get("files")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    decode_turn_diff_files(&serde_json::to_string(&files)?)
}

fn is_stale_pending_approval_failure_detail(detail: Option<&str>) -> bool {
    let Some(detail) = detail.map(str::to_ascii_lowercase) else {
        return false;
    };
    detail.contains("stale pending approval request")
        || detail.contains("unknown pending approval request")
        || detail.contains("unknown pending permission request")
}

fn project_select_sql(suffix: &str) -> String {
    format!(
        r#"
        SELECT
          project_id,
          title,
          workspace_root,
          scripts_json,
          created_at,
          updated_at,
          deleted_at
        FROM projection_projects
        {suffix}
        "#
    )
}

fn project_row_from_sql(row: &Row<'_>) -> ProjectionPersistenceResult<ProjectionProjectRow> {
    let scripts_json: String = row.get(3)?;
    Ok(ProjectionProjectRow {
        project_id: row.get(0)?,
        title: row.get(1)?,
        workspace_root: row.get(2)?,
        scripts: decode_project_scripts(&scripts_json)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        deleted_at: row.get(6)?,
    })
}

fn thread_select_sql(suffix: &str) -> String {
    format!(
        r#"
        SELECT
          thread_id,
          project_id,
          title,
          runtime_mode,
          interaction_mode,
          branch,
          worktree_path,
          created_at,
          updated_at,
          archived_at,
          latest_user_message_at,
          pending_approval_count,
          pending_user_input_count,
          has_actionable_proposed_plan,
          deleted_at
        FROM projection_threads
        {suffix}
        "#
    )
}

fn thread_row_from_sql(row: &Row<'_>) -> ProjectionPersistenceResult<ProjectionThreadRow> {
    let runtime_mode: String = row.get(3)?;
    let interaction_mode: String = row.get(4)?;
    let has_actionable_proposed_plan: i64 = row.get(13)?;
    Ok(ProjectionThreadRow {
        thread_id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        runtime_mode: runtime_mode_from_t3(&runtime_mode)?,
        interaction_mode: interaction_mode_from_t3(&interaction_mode)?,
        branch: row.get(5)?,
        worktree_path: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        archived_at: row.get(9)?,
        latest_user_message_at: row.get(10)?,
        pending_approval_count: row.get(11)?,
        pending_user_input_count: row.get(12)?,
        has_actionable_proposed_plan: has_actionable_proposed_plan > 0,
        deleted_at: row.get(14)?,
    })
}

fn thread_session_select_sql(suffix: &str) -> String {
    format!(
        "{} {suffix}",
        thread_session_select_sql_from("projection_thread_sessions sessions")
    )
}

fn thread_session_select_sql_from(from: &str) -> String {
    format!(
        r#"
        SELECT
          sessions.thread_id,
          sessions.status,
          sessions.provider_name,
          sessions.provider_instance_id,
          sessions.active_turn_id,
          sessions.last_error,
          sessions.updated_at
        FROM {from}
        "#
    )
}

fn thread_session_row_from_sql(
    row: &Row<'_>,
) -> ProjectionPersistenceResult<ProjectionThreadSessionRow> {
    Ok(ProjectionThreadSessionRow {
        thread_id: row.get(0)?,
        status: row.get(1)?,
        provider_name: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        provider_instance_id: row.get(3)?,
        active_turn_id: row.get(4)?,
        last_error: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn projection_turn_select_sql() -> &'static str {
    r#"
    SELECT
      thread_id,
      turn_id,
      pending_message_id,
      source_proposed_plan_thread_id,
      source_proposed_plan_id,
      assistant_message_id,
      state,
      requested_at,
      started_at,
      completed_at,
      checkpoint_turn_count,
      checkpoint_ref,
      checkpoint_status,
      checkpoint_files_json
    FROM projection_turns
    "#
}

fn projection_turn_row_from_sql(row: &Row<'_>) -> ProjectionPersistenceResult<ProjectionTurnRow> {
    let checkpoint_files_json: String = row.get(13)?;
    Ok(ProjectionTurnRow {
        thread_id: row.get(0)?,
        turn_id: row.get(1)?,
        pending_message_id: row.get(2)?,
        source_proposed_plan_thread_id: row.get(3)?,
        source_proposed_plan_id: row.get(4)?,
        assistant_message_id: row.get(5)?,
        state: row.get(6)?,
        requested_at: row.get(7)?,
        started_at: row.get(8)?,
        completed_at: row.get(9)?,
        checkpoint_turn_count: row.get(10)?,
        checkpoint_ref: row.get(11)?,
        checkpoint_status: row.get(12)?,
        checkpoint_files: decode_turn_diff_files(&checkpoint_files_json)?,
    })
}

fn latest_turn_row_from_sql(row: &Row<'_>) -> ProjectionPersistenceResult<ProjectionLatestTurnRow> {
    Ok(ProjectionLatestTurnRow {
        thread_id: row.get(0)?,
        turn_id: row.get(1)?,
        state: row.get(2)?,
        requested_at: row.get(3)?,
        started_at: row.get(4)?,
        completed_at: row.get(5)?,
        assistant_message_id: row.get(6)?,
        source_proposed_plan_thread_id: row.get(7)?,
        source_proposed_plan_id: row.get(8)?,
    })
}

fn chat_message_row_from_sql(row: &Row<'_>) -> ProjectionPersistenceResult<ChatMessage> {
    let role: String = row.get(1)?;
    let attachments_json: Option<String> = row.get(3)?;
    let created_at: String = row.get(5)?;
    let updated_at: String = row.get(6)?;
    let streaming: i64 = row.get(7)?;
    Ok(ChatMessage {
        id: row.get(0)?,
        role: message_role_from_t3(&role)?,
        text: row.get(2)?,
        attachments: attachments_json
            .as_deref()
            .map(decode_chat_attachments)
            .transpose()?
            .unwrap_or_default(),
        turn_id: row.get(4)?,
        completed_at: (updated_at != created_at).then_some(updated_at),
        created_at,
        streaming: streaming > 0,
    })
}

fn thread_activity_row_from_sql(row: &Row<'_>) -> ProjectionPersistenceResult<ThreadActivity> {
    let tone: String = row.get(3)?;
    let payload_json: String = row.get(4)?;
    Ok(ThreadActivity {
        id: row.get(0)?,
        kind: row.get(1)?,
        summary: row.get(2)?,
        tone: activity_tone_from_t3(&tone)?,
        payload: decode_activity_payload(&payload_json)?,
        turn_id: row.get(5)?,
        sequence: row.get(6)?,
        created_at: row.get(7)?,
    })
}

fn proposed_plan_row_from_sql(row: &Row<'_>) -> ProjectionPersistenceResult<ProposedPlan> {
    Ok(ProposedPlan {
        id: row.get(0)?,
        turn_id: row.get(1)?,
        plan_markdown: row.get(2)?,
        implemented_at: row.get(3)?,
        implementation_thread_id: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn turn_diff_summary_row_from_sql(row: &Row<'_>) -> ProjectionPersistenceResult<TurnDiffSummary> {
    let files_json: String = row.get(3)?;
    Ok(TurnDiffSummary {
        turn_id: row.get(0)?,
        completed_at: row.get(1)?,
        status: row.get(2)?,
        files: decode_turn_diff_files(&files_json)?,
        checkpoint_ref: row.get(4)?,
        assistant_message_id: row.get(5)?,
        checkpoint_turn_count: row.get(6)?,
    })
}

fn pending_approval_row_from_sql(
    row: &Row<'_>,
) -> ProjectionPersistenceResult<ProjectionPendingApprovalRow> {
    Ok(ProjectionPendingApprovalRow {
        request_id: row.get(0)?,
        thread_id: row.get(1)?,
        turn_id: row.get(2)?,
        status: row.get(3)?,
        decision: row.get(4)?,
        created_at: row.get(5)?,
        resolved_at: row.get(6)?,
    })
}

fn encode_project_scripts(scripts: &[ProjectScript]) -> ProjectionPersistenceResult<String> {
    let value = Value::Array(
        scripts
            .iter()
            .map(|script| {
                json!({
                    "id": script.id,
                    "name": script.name,
                    "command": script.command,
                    "icon": project_script_icon_to_t3(script.icon),
                    "runOnWorktreeCreate": script.run_on_worktree_create,
                })
            })
            .collect(),
    );
    serde_json::to_string(&value).map_err(ProjectionPersistenceError::from)
}

fn decode_project_scripts(json: &str) -> ProjectionPersistenceResult<Vec<ProjectScript>> {
    let value: Value = serde_json::from_str(json)?;
    let Some(items) = value.as_array() else {
        return Err(ProjectionPersistenceError::InvalidProjectScript(
            "scripts_json must be an array".to_string(),
        ));
    };

    items
        .iter()
        .map(|item| {
            Ok(ProjectScript {
                id: required_string(item, "id")?,
                name: required_string(item, "name")?,
                command: required_string(item, "command")?,
                icon: project_script_icon_from_t3(&required_string(item, "icon")?)?,
                run_on_worktree_create: item
                    .get("runOnWorktreeCreate")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| {
                        ProjectionPersistenceError::InvalidProjectScript(
                            "missing runOnWorktreeCreate boolean".to_string(),
                        )
                    })?,
            })
        })
        .collect()
}

fn encode_chat_attachments(attachments: &[ChatAttachment]) -> ProjectionPersistenceResult<String> {
    serde_json::to_string(
        &attachments
            .iter()
            .map(|attachment| match attachment {
                ChatAttachment::Image(image) => json!({
                    "type": "image",
                    "id": image.id,
                    "name": image.name,
                    "mimeType": image.mime_type,
                    "sizeBytes": image.size_bytes,
                }),
            })
            .collect::<Vec<_>>(),
    )
    .map_err(ProjectionPersistenceError::from)
}

fn decode_chat_attachments(json: &str) -> ProjectionPersistenceResult<Vec<ChatAttachment>> {
    let value: Value = serde_json::from_str(json)?;
    let Some(items) = value.as_array() else {
        return Err(ProjectionPersistenceError::InvalidProjectScript(
            "attachments_json must be an array".to_string(),
        ));
    };
    items
        .iter()
        .map(|item| {
            let item_type = required_string(item, "type")?;
            match item_type.as_str() {
                "image" => Ok(ChatAttachment::Image(ChatImageAttachment {
                    id: required_string(item, "id")?,
                    name: required_string(item, "name")?,
                    mime_type: required_string(item, "mimeType")?,
                    size_bytes: item
                        .get("sizeBytes")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| {
                            ProjectionPersistenceError::InvalidProjectScript(
                                "missing sizeBytes integer".to_string(),
                            )
                        })?,
                    preview_url: None,
                })),
                _ => Err(ProjectionPersistenceError::InvalidProjectScript(format!(
                    "unknown attachment type {item_type}"
                ))),
            }
        })
        .collect()
}

fn encode_activity_payload(payload: &ActivityPayload) -> ProjectionPersistenceResult<String> {
    serde_json::to_string(&json!({
        "requestId": payload.request_id,
        "requestType": payload.request_type,
        "detail": payload.detail,
        "command": payload.command,
        "rawCommand": payload.raw_command,
        "changedFiles": payload.changed_files,
        "title": payload.title,
        "itemType": payload.item_type,
        "toolCallId": payload.tool_call_id,
        "questions": payload.questions.iter().map(encode_user_input_question).collect::<Vec<_>>(),
    }))
    .map_err(ProjectionPersistenceError::from)
}

fn decode_activity_payload(json: &str) -> ProjectionPersistenceResult<ActivityPayload> {
    let value: Value = serde_json::from_str(json)?;
    let request_type = value
        .get("requestType")
        .and_then(Value::as_str)
        .map(str::to_string);
    Ok(ActivityPayload {
        request_id: value
            .get("requestId")
            .and_then(Value::as_str)
            .map(str::to_string),
        request_kind: request_type
            .as_deref()
            .and_then(ApprovalRequestKind::from_request_type),
        request_type,
        detail: value
            .get("detail")
            .or_else(|| value.get("message"))
            .and_then(Value::as_str)
            .map(str::to_string),
        command: value
            .get("command")
            .and_then(Value::as_str)
            .map(str::to_string),
        raw_command: value
            .get("rawCommand")
            .and_then(Value::as_str)
            .map(str::to_string),
        changed_files: value
            .get("changedFiles")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        title: value
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string),
        item_type: value
            .get("itemType")
            .and_then(Value::as_str)
            .map(str::to_string),
        tool_call_id: value
            .get("toolCallId")
            .and_then(Value::as_str)
            .map(str::to_string),
        questions: value
            .get("questions")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(decode_user_input_question)
                    .collect::<ProjectionPersistenceResult<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default(),
    })
}

fn encode_user_input_question(question: &UserInputQuestion) -> Value {
    json!({
        "id": question.id,
        "header": question.header,
        "question": question.question,
        "multiSelect": question.multi_select,
        "options": question.options.iter().map(|option| {
            json!({
                "label": option.label,
                "description": option.description,
            })
        }).collect::<Vec<_>>(),
    })
}

fn decode_user_input_question(value: &Value) -> ProjectionPersistenceResult<UserInputQuestion> {
    Ok(UserInputQuestion {
        id: required_string(value, "id")?,
        header: required_string(value, "header")?,
        question: required_string(value, "question")?,
        options: value
            .get("options")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|item| {
                        Ok(UserInputQuestionOption {
                            label: required_string(item, "label")?,
                            description: required_string(item, "description")?,
                        })
                    })
                    .collect::<ProjectionPersistenceResult<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default(),
        multi_select: value
            .get("multiSelect")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn encode_turn_diff_files(files: &[TurnDiffFileChange]) -> ProjectionPersistenceResult<String> {
    serde_json::to_string(
        &files
            .iter()
            .map(|file| {
                json!({
                    "path": file.path,
                    "kind": file.kind,
                    "additions": file.additions,
                    "deletions": file.deletions,
                })
            })
            .collect::<Vec<_>>(),
    )
    .map_err(ProjectionPersistenceError::from)
}

fn decode_turn_diff_files(json: &str) -> ProjectionPersistenceResult<Vec<TurnDiffFileChange>> {
    let value: Value = serde_json::from_str(json)?;
    let Some(items) = value.as_array() else {
        return Err(ProjectionPersistenceError::InvalidProjectScript(
            "checkpoint_files_json must be an array".to_string(),
        ));
    };
    items
        .iter()
        .map(|item| {
            Ok(TurnDiffFileChange {
                path: required_string(item, "path")?,
                kind: item.get("kind").and_then(Value::as_str).map(str::to_string),
                additions: item
                    .get("additions")
                    .and_then(Value::as_u64)
                    .map(|value| value as u32),
                deletions: item
                    .get("deletions")
                    .and_then(Value::as_u64)
                    .map(|value| value as u32),
            })
        })
        .collect()
}

fn required_string(value: &Value, key: &str) -> ProjectionPersistenceResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            ProjectionPersistenceError::InvalidProjectScript(format!(
                "missing non-empty {key} string"
            ))
        })
}

fn project_script_icon_to_t3(icon: ProjectScriptIcon) -> &'static str {
    match icon {
        ProjectScriptIcon::Play => "play",
        ProjectScriptIcon::Test => "test",
        ProjectScriptIcon::Lint => "lint",
        ProjectScriptIcon::Configure => "configure",
        ProjectScriptIcon::Build => "build",
        ProjectScriptIcon::Debug => "debug",
    }
}

fn project_script_icon_from_t3(value: &str) -> ProjectionPersistenceResult<ProjectScriptIcon> {
    match value {
        "play" => Ok(ProjectScriptIcon::Play),
        "test" => Ok(ProjectScriptIcon::Test),
        "lint" => Ok(ProjectScriptIcon::Lint),
        "configure" => Ok(ProjectScriptIcon::Configure),
        "build" => Ok(ProjectScriptIcon::Build),
        "debug" => Ok(ProjectScriptIcon::Debug),
        _ => Err(ProjectionPersistenceError::InvalidProjectScript(format!(
            "unknown icon {value}"
        ))),
    }
}

fn message_role_to_t3(role: MessageRole) -> &'static str {
    match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    }
}

fn message_role_from_t3(value: &str) -> ProjectionPersistenceResult<MessageRole> {
    match value {
        "user" => Ok(MessageRole::User),
        "assistant" => Ok(MessageRole::Assistant),
        "system" => Ok(MessageRole::System),
        _ => Err(ProjectionPersistenceError::InvalidProjectScript(format!(
            "unknown message role {value}"
        ))),
    }
}

fn activity_tone_to_t3(tone: ActivityTone) -> &'static str {
    match tone {
        ActivityTone::Thinking => "thinking",
        ActivityTone::Tool => "tool",
        ActivityTone::Info => "info",
        ActivityTone::Error => "error",
        ActivityTone::Approval => "approval",
    }
}

fn activity_tone_from_t3(value: &str) -> ProjectionPersistenceResult<ActivityTone> {
    match value {
        "thinking" => Ok(ActivityTone::Thinking),
        "tool" => Ok(ActivityTone::Tool),
        "info" => Ok(ActivityTone::Info),
        "error" => Ok(ActivityTone::Error),
        "approval" => Ok(ActivityTone::Approval),
        _ => Err(ProjectionPersistenceError::InvalidProjectScript(format!(
            "unknown activity tone {value}"
        ))),
    }
}

fn runtime_mode_to_t3(mode: RuntimeMode) -> &'static str {
    match mode {
        RuntimeMode::ApprovalRequired => "approval-required",
        RuntimeMode::AutoAcceptEdits => "auto-accept-edits",
        RuntimeMode::FullAccess => "full-access",
    }
}

fn runtime_mode_from_t3(value: &str) -> ProjectionPersistenceResult<RuntimeMode> {
    match value {
        "approval-required" => Ok(RuntimeMode::ApprovalRequired),
        "auto-accept-edits" => Ok(RuntimeMode::AutoAcceptEdits),
        "full-access" => Ok(RuntimeMode::FullAccess),
        _ => Err(ProjectionPersistenceError::InvalidRuntimeMode(
            value.to_string(),
        )),
    }
}

fn interaction_mode_to_t3(mode: ProviderInteractionMode) -> &'static str {
    match mode {
        ProviderInteractionMode::Default => "default",
        ProviderInteractionMode::Plan => "plan",
    }
}

fn interaction_mode_from_t3(value: &str) -> ProjectionPersistenceResult<ProviderInteractionMode> {
    match value {
        "default" => Ok(ProviderInteractionMode::Default),
        "plan" => Ok(ProviderInteractionMode::Plan),
        _ => Err(ProjectionPersistenceError::InvalidInteractionMode(
            value.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ThreadStatus, build_projection_shell_snapshot, get_thread_from_environment_state,
        orchestration::{
            ProviderRuntimeBufferedAssistantText, ProviderRuntimeEventInput,
            ProviderRuntimeIngestionCommandPlanContext, ProviderRuntimeIngestionQueue,
            ProviderRuntimeSessionContext, ProviderStopSessionInput, SourceProposedPlanRef,
            provider_runtime_activity_to_thread_activity_append_command,
            provider_runtime_assistant_complete_command, provider_runtime_assistant_delta_command,
            provider_runtime_event_to_activities, provider_runtime_lifecycle_session_command,
            provider_runtime_proposed_plan_complete_command,
            provider_runtime_turn_diff_complete_command,
        },
    };

    fn project(project_id: &str, created_at: &str) -> ProjectionProjectRow {
        ProjectionProjectRow {
            project_id: project_id.to_string(),
            title: format!("Project {project_id}"),
            workspace_root: format!("/repo/{project_id}"),
            scripts: vec![ProjectScript {
                id: "setup".to_string(),
                name: "Setup".to_string(),
                command: "npm install".to_string(),
                icon: ProjectScriptIcon::Configure,
                run_on_worktree_create: true,
            }],
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            deleted_at: None,
        }
    }

    fn thread(thread_id: &str, project_id: &str, created_at: &str) -> ProjectionThreadRow {
        ProjectionThreadRow {
            thread_id: thread_id.to_string(),
            project_id: project_id.to_string(),
            title: format!("Thread {thread_id}"),
            runtime_mode: RuntimeMode::AutoAcceptEdits,
            interaction_mode: ProviderInteractionMode::Plan,
            branch: Some("main".to_string()),
            worktree_path: Some(format!("/repo/{project_id}")),
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            archived_at: None,
            latest_user_message_at: Some("2026-03-04T12:00:04.000Z".to_string()),
            pending_approval_count: 0,
            pending_user_input_count: 0,
            has_actionable_proposed_plan: true,
            deleted_at: None,
        }
    }

    #[test]
    fn ports_persistence_error_and_sqlite_layer_contracts() {
        assert_eq!(
            persistence_sql_error_plan("ProjectionThreads.upsert"),
            PersistenceTaggedErrorPlan {
                tag: PersistenceErrorTag::PersistenceSqlError,
                operation: "ProjectionThreads.upsert".to_string(),
                detail: Some("Failed to execute ProjectionThreads.upsert".to_string()),
                issue: None,
                message:
                    "SQL error in ProjectionThreads.upsert: Failed to execute ProjectionThreads.upsert"
                        .to_string(),
            }
        );
        assert_eq!(
            persistence_decode_error_plan("ProjectionThreads.decode", "bad row").message,
            "Decode error in ProjectionThreads.decode: bad row"
        );
        assert_eq!(
            persistence_decode_cause_error_plan("ProjectionThreads.read").issue,
            Some("Failed to execute ProjectionThreads.read".to_string())
        );
        assert_eq!(
            provider_session_repository_validation_error_plan(
                "ProviderSessionRuntime.upsert",
                "missing thread id"
            )
            .message,
            "Provider session repository validation failed in ProviderSessionRuntime.upsert: missing thread id"
        );
        assert_eq!(
            provider_session_repository_persistence_error_plan(
                "ProviderSessionRuntime.upsert",
                "database closed"
            )
            .message,
            "Provider session repository persistence error in ProviderSessionRuntime.upsert: database closed"
        );
        assert!(is_persistence_error_tag(
            PersistenceErrorTag::PersistenceSqlError
        ));
        assert!(!is_persistence_error_tag(
            PersistenceErrorTag::ProviderSessionRepositoryPersistenceError
        ));

        assert_eq!(
            sqlite_runtime_from_bun_version_present(true),
            SqliteRuntime::Bun
        );
        assert_eq!(
            sqlite_runtime_from_bun_version_present(false),
            SqliteRuntime::Node
        );
        assert!(!node_sqlite_statement_columns_supported("22.15.0"));
        assert!(node_sqlite_statement_columns_supported("22.16.0"));
        assert!(node_sqlite_statement_columns_supported("23.11.0"));
        assert!(node_sqlite_statement_columns_supported("24.0.0"));

        let node_plan = node_sqlite_client_config_plan(
            ":memory:",
            None,
            None,
            &[("service.name", "t3-server")],
            true,
            false,
        );
        assert_eq!(node_plan.filename, ":memory:");
        assert!(!node_plan.readonly);
        assert!(!node_plan.allow_extension);
        assert_eq!(node_plan.prepare_cache_size, 200);
        assert_eq!(node_plan.prepare_cache_ttl, "10 minutes");
        assert!(
            node_plan
                .span_attributes
                .contains(&("db.system.name".to_string(), "sqlite".to_string()))
        );
        assert!(node_plan.transform_result_names);
        assert!(!node_plan.transform_query_names);

        let layer_plan = sqlite_persistence_layer_plan(
            "C:/Users/bunny/AppData/Roaming/r3code/r3.sqlite",
            SqliteRuntime::Node,
            Some(2),
        );
        assert_eq!(layer_plan.runtime, SqliteRuntime::Node);
        assert_eq!(
            layer_plan.setup_statements,
            vec!["PRAGMA journal_mode = WAL;", "PRAGMA foreign_keys = ON;"]
        );
        assert_eq!(
            layer_plan.span_attributes,
            vec![
                ("db.name".to_string(), "r3.sqlite".to_string()),
                ("service.name".to_string(), "t3-server".to_string())
            ]
        );
        assert_eq!(
            layer_plan.migration_keys,
            vec![
                "1_OrchestrationEvents".to_string(),
                "2_OrchestrationCommandReceipts".to_string()
            ]
        );
    }

    #[test]
    fn ports_migration_entry_ordering_and_filtering_contracts() {
        let entries = persistence_migration_entries();
        assert_eq!(entries.len(), 30);
        assert_eq!(
            entries.first(),
            Some(&PersistenceMigrationEntry {
                id: 1,
                name: "OrchestrationEvents",
                module_name: "001_OrchestrationEvents",
            })
        );
        assert_eq!(
            entries.get(2),
            Some(&PersistenceMigrationEntry {
                id: 3,
                name: "CheckpointDiffBlobs",
                module_name: "003_CheckpointDiffBlobs",
            })
        );
        assert_eq!(
            entries.get(25),
            Some(&PersistenceMigrationEntry {
                id: 26,
                name: "CanonicalizeModelSelectionOptions",
                module_name: "026_CanonicalizeModelSelectionOptions",
            })
        );
        assert_eq!(
            entries.last(),
            Some(&PersistenceMigrationEntry {
                id: 30,
                name: "ProjectionThreadShellArchiveIndexes",
                module_name: "030_ProjectionThreadShellArchiveIndexes",
            })
        );
        assert_eq!(
            migration_loader_record_keys(Some(4)),
            vec![
                "1_OrchestrationEvents".to_string(),
                "2_OrchestrationCommandReceipts".to_string(),
                "3_CheckpointDiffBlobs".to_string(),
                "4_ProviderSessionRuntime".to_string(),
            ]
        );
        assert_eq!(migration_loader_record_keys(None).len(), 30);
        assert_eq!(
            projection_checkpoint_repository_operation_names(),
            vec![
                ("upsert", "ProjectionCheckpointRepository.upsert:query"),
                (
                    "listByThreadId",
                    "ProjectionCheckpointRepository.listByThreadId:query"
                ),
                (
                    "getByThreadAndTurnCount",
                    "ProjectionCheckpointRepository.getByThreadAndTurnCount:query"
                ),
                (
                    "deleteByThreadId",
                    "ProjectionCheckpointRepository.deleteByThreadId:query"
                ),
            ]
        );
        assert_eq!(
            provider_session_runtime_repository_operation_names(),
            vec![
                ("upsert", "ProviderSessionRuntimeRepository.upsert:query"),
                (
                    "getByThreadId",
                    "ProviderSessionRuntimeRepository.getByThreadId:query"
                ),
                ("list", "ProviderSessionRuntimeRepository.list:query"),
                (
                    "deleteByThreadId",
                    "ProviderSessionRuntimeRepository.deleteByThreadId:query"
                ),
            ]
        );
    }

    #[test]
    fn applies_t3_projection_schema_columns() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();

        assert!(
            store
                .table_column_names("projection_projects")
                .unwrap()
                .contains(&"default_model_selection_json".to_string())
        );
        assert!(
            store
                .table_column_names("projection_threads")
                .unwrap()
                .contains(&"has_actionable_proposed_plan".to_string())
        );
        assert!(
            store
                .table_column_names("projection_thread_sessions")
                .unwrap()
                .contains(&"provider_instance_id".to_string())
        );
        assert!(
            store
                .table_column_names("provider_session_runtime")
                .unwrap()
                .contains(&"provider_instance_id".to_string())
        );
        assert!(
            store
                .table_column_names("auth_pairing_links")
                .unwrap()
                .contains(&"label".to_string())
        );
        assert!(
            store
                .table_column_names("auth_sessions")
                .unwrap()
                .contains(&"client_device_type".to_string())
        );
        assert!(
            store
                .table_column_names("auth_sessions")
                .unwrap()
                .contains(&"last_connected_at".to_string())
        );
        assert!(
            store
                .table_column_names("projection_turns")
                .unwrap()
                .contains(&"source_proposed_plan_id".to_string())
        );
        assert!(
            store
                .table_column_names("projection_thread_messages")
                .unwrap()
                .contains(&"attachments_json".to_string())
        );
        assert!(
            store
                .table_column_names("projection_thread_proposed_plans")
                .unwrap()
                .contains(&"implementation_thread_id".to_string())
        );
        assert!(
            store
                .table_column_names("orchestration_events")
                .unwrap()
                .contains(&"stream_version".to_string())
        );
        assert!(
            store
                .table_column_names("orchestration_command_receipts")
                .unwrap()
                .contains(&"result_sequence".to_string())
        );
        let project_indexes = store.table_index_names("projection_projects").unwrap();
        assert!(
            project_indexes
                .contains(&"idx_projection_projects_workspace_root_deleted_at".to_string())
        );
        let thread_indexes = store.table_index_names("projection_threads").unwrap();
        assert!(thread_indexes.contains(&"idx_projection_threads_project_archived_at".to_string()));
        assert!(
            thread_indexes.contains(&"idx_projection_threads_project_deleted_created".to_string())
        );
        let message_indexes = store
            .table_index_names("projection_thread_messages")
            .unwrap();
        assert!(
            message_indexes
                .contains(&"idx_projection_thread_messages_thread_created_id".to_string())
        );
        let activity_indexes = store
            .table_index_names("projection_thread_activities")
            .unwrap();
        assert!(
            activity_indexes.contains(
                &"idx_projection_thread_activities_thread_sequence_created_id".to_string()
            )
        );
        let provider_runtime_indexes = store.table_index_names("provider_session_runtime").unwrap();
        assert!(
            provider_runtime_indexes.contains(&"idx_provider_session_runtime_instance".to_string())
        );
        assert!(
            store
                .table_index_names("auth_pairing_links")
                .unwrap()
                .contains(&"idx_auth_pairing_links_active".to_string())
        );
        assert!(
            store
                .table_index_names("auth_sessions")
                .unwrap()
                .contains(&"idx_auth_sessions_active".to_string())
        );
    }

    #[test]
    fn projection_sqlite_store_round_trips_provider_session_runtime_rows() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        let first = ProviderSessionRuntimeRow {
            thread_id: "thread-b".to_string(),
            provider_name: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            adapter_key: "codex-main".to_string(),
            runtime_mode: RuntimeMode::FullAccess,
            status: "running".to_string(),
            last_seen_at: "2026-03-04T12:00:02.000Z".to_string(),
            resume_cursor: Some(json!({ "opaque": "resume-b" })),
            runtime_payload: Some(json!({
                "cwd": "C:/work/r3code",
                "activeTurnId": "turn-1"
            })),
        };
        let second = ProviderSessionRuntimeRow {
            thread_id: "thread-a".to_string(),
            provider_name: "claudeAgent".to_string(),
            provider_instance_id: Some("claude-main".to_string()),
            adapter_key: "claude-main".to_string(),
            runtime_mode: RuntimeMode::AutoAcceptEdits,
            status: "stopped".to_string(),
            last_seen_at: "2026-03-04T12:00:01.000Z".to_string(),
            resume_cursor: None,
            runtime_payload: Some(json!({ "activeTurnId": null })),
        };

        store.upsert_provider_session_runtime(&first).unwrap();
        store.upsert_provider_session_runtime(&second).unwrap();

        assert_eq!(
            store
                .get_provider_session_runtime_by_thread_id("thread-b")
                .unwrap(),
            Some(first.clone())
        );
        assert_eq!(
            store
                .list_provider_session_runtimes()
                .unwrap()
                .into_iter()
                .map(|row| row.thread_id)
                .collect::<Vec<_>>(),
            vec!["thread-a", "thread-b"]
        );

        let updated = ProviderSessionRuntimeRow {
            status: "error".to_string(),
            last_seen_at: "2026-03-04T12:00:03.000Z".to_string(),
            resume_cursor: Some(json!({ "opaque": "resume-updated" })),
            runtime_payload: Some(json!({ "lastError": "failed" })),
            ..first
        };
        store.upsert_provider_session_runtime(&updated).unwrap();
        assert_eq!(
            store
                .get_provider_session_runtime_by_thread_id("thread-b")
                .unwrap(),
            Some(updated)
        );

        store
            .delete_provider_session_runtime_by_thread_id("thread-b")
            .unwrap();
        assert_eq!(
            store
                .get_provider_session_runtime_by_thread_id("thread-b")
                .unwrap(),
            None
        );
    }

    #[test]
    fn projection_sqlite_store_ports_auth_session_repository_behaviour() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        let first = AuthSessionRow {
            session_id: "session-first".to_string(),
            subject: "owner".to_string(),
            role: "owner".to_string(),
            method: "browser-session-cookie".to_string(),
            client: AuthSessionClientMetadataRow {
                label: Some("Desktop".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("R3Code".to_string()),
                device_type: "desktop".to_string(),
                os: Some("Windows".to_string()),
                browser: None,
            },
            issued_at: "2026-03-04T12:00:01.000Z".to_string(),
            expires_at: "2026-03-04T13:00:00.000Z".to_string(),
            last_connected_at: None,
            revoked_at: None,
        };
        let second = AuthSessionRow {
            session_id: "session-second".to_string(),
            subject: "client".to_string(),
            role: "client".to_string(),
            method: "bearer-session-token".to_string(),
            issued_at: "2026-03-04T12:00:02.000Z".to_string(),
            expires_at: "2026-03-04T13:00:00.000Z".to_string(),
            client: AuthSessionClientMetadataRow::default(),
            last_connected_at: None,
            revoked_at: None,
        };
        let expired = AuthSessionRow {
            session_id: "session-expired".to_string(),
            issued_at: "2026-03-04T11:00:00.000Z".to_string(),
            expires_at: "2026-03-04T11:59:00.000Z".to_string(),
            ..second.clone()
        };

        store.insert_auth_session(&first).unwrap();
        store.insert_auth_session(&second).unwrap();
        store.insert_auth_session(&expired).unwrap();

        assert_eq!(
            store.get_auth_session_by_id("session-first").unwrap(),
            Some(first.clone())
        );
        assert_eq!(
            store
                .list_active_auth_sessions("2026-03-04T12:10:00.000Z")
                .unwrap()
                .into_iter()
                .map(|row| row.session_id)
                .collect::<Vec<_>>(),
            vec!["session-second", "session-first"]
        );

        store
            .set_auth_session_last_connected_at("session-first", "2026-03-04T12:20:00.000Z")
            .unwrap();
        assert_eq!(
            store
                .get_auth_session_by_id("session-first")
                .unwrap()
                .unwrap()
                .last_connected_at
                .as_deref(),
            Some("2026-03-04T12:20:00.000Z")
        );

        assert!(
            store
                .revoke_auth_session("session-first", "2026-03-04T12:30:00.000Z")
                .unwrap()
        );
        assert!(
            !store
                .revoke_auth_session("session-first", "2026-03-04T12:31:00.000Z")
                .unwrap()
        );
        assert_eq!(
            store
                .revoke_other_auth_sessions("session-first", "2026-03-04T12:40:00.000Z")
                .unwrap(),
            vec!["session-second"]
        );
        assert!(
            store
                .list_active_auth_sessions("2026-03-04T12:41:00.000Z")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn projection_sqlite_store_ports_auth_pairing_link_repository_behaviour() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        let first = AuthPairingLinkRow {
            id: "pairing-first".to_string(),
            credential: "credential-first".to_string(),
            method: "desktop-bootstrap".to_string(),
            role: "owner".to_string(),
            subject: "owner".to_string(),
            label: Some("Main desktop".to_string()),
            created_at: "2026-03-04T12:00:01.000Z".to_string(),
            expires_at: "2026-03-04T13:00:00.000Z".to_string(),
            consumed_at: None,
            revoked_at: None,
        };
        let second = AuthPairingLinkRow {
            id: "pairing-second".to_string(),
            credential: "credential-second".to_string(),
            method: "one-time-token".to_string(),
            role: "client".to_string(),
            subject: "client".to_string(),
            label: None,
            created_at: "2026-03-04T12:00:02.000Z".to_string(),
            expires_at: "2026-03-04T13:00:00.000Z".to_string(),
            consumed_at: None,
            revoked_at: None,
        };

        store.insert_auth_pairing_link(&first).unwrap();
        store.insert_auth_pairing_link(&second).unwrap();

        assert_eq!(
            store
                .list_active_auth_pairing_links("2026-03-04T12:10:00.000Z")
                .unwrap()
                .into_iter()
                .map(|row| row.id)
                .collect::<Vec<_>>(),
            vec!["pairing-second", "pairing-first"]
        );

        let consumed = store
            .consume_available_auth_pairing_link(
                "credential-first",
                "2026-03-04T12:20:00.000Z",
                "2026-03-04T12:20:00.000Z",
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            consumed.consumed_at.as_deref(),
            Some("2026-03-04T12:20:00.000Z")
        );
        assert_eq!(
            store
                .consume_available_auth_pairing_link(
                    "credential-first",
                    "2026-03-04T12:21:00.000Z",
                    "2026-03-04T12:21:00.000Z",
                )
                .unwrap(),
            None
        );

        assert!(
            store
                .revoke_auth_pairing_link("pairing-second", "2026-03-04T12:30:00.000Z")
                .unwrap()
        );
        assert!(
            !store
                .revoke_auth_pairing_link("pairing-second", "2026-03-04T12:31:00.000Z")
                .unwrap()
        );
        assert!(
            store
                .list_active_auth_pairing_links("2026-03-04T12:31:00.000Z")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn projection_sqlite_store_maps_provider_runtime_bindings_like_session_directory() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        let inserted = store
            .upsert_provider_runtime_binding(
                &ProviderRuntimeBinding {
                    thread_id: "thread-1".to_string(),
                    provider: "codex".to_string(),
                    provider_instance_id: Some("codex-main".to_string()),
                    adapter_key: None,
                    status: Some("running".to_string()),
                    resume_cursor: Some(json!({ "opaque": "resume-1" })),
                    runtime_payload: Some(json!({
                        "cwd": "C:/work/r3code",
                        "activeTurnId": "turn-1"
                    })),
                    runtime_mode: Some(RuntimeMode::FullAccess),
                },
                "2026-03-04T12:00:00.000Z",
            )
            .unwrap();
        assert_eq!(inserted.adapter_key, "codex");
        assert_eq!(
            store
                .get_provider_runtime_binding_by_thread_id("thread-1")
                .unwrap(),
            Some(ProviderRuntimeBinding {
                thread_id: "thread-1".to_string(),
                provider: "codex".to_string(),
                provider_instance_id: Some("codex-main".to_string()),
                adapter_key: Some("codex".to_string()),
                status: Some("running".to_string()),
                resume_cursor: Some(json!({ "opaque": "resume-1" })),
                runtime_payload: Some(json!({
                    "cwd": "C:/work/r3code",
                    "activeTurnId": "turn-1"
                })),
                runtime_mode: Some(RuntimeMode::FullAccess),
            })
        );

        let updated = store
            .upsert_provider_runtime_binding(
                &ProviderRuntimeBinding {
                    thread_id: "thread-1".to_string(),
                    provider: "codex".to_string(),
                    provider_instance_id: None,
                    adapter_key: None,
                    status: Some("stopped".to_string()),
                    resume_cursor: None,
                    runtime_payload: Some(json!({
                        "activeTurnId": null,
                        "lastRuntimeEvent": "provider.stopSession"
                    })),
                    runtime_mode: None,
                },
                "2026-03-04T12:00:01.000Z",
            )
            .unwrap();
        assert_eq!(updated.provider_instance_id.as_deref(), Some("codex-main"));
        assert_eq!(updated.resume_cursor, Some(json!({ "opaque": "resume-1" })));
        assert_eq!(
            updated.runtime_payload,
            Some(json!({
                "cwd": "C:/work/r3code",
                "activeTurnId": null,
                "lastRuntimeEvent": "provider.stopSession"
            }))
        );

        store
            .upsert_provider_session_runtime(&ProviderSessionRuntimeRow {
                thread_id: "legacy-thread".to_string(),
                provider_name: "claudeAgent".to_string(),
                provider_instance_id: None,
                adapter_key: "claudeAgent".to_string(),
                runtime_mode: RuntimeMode::FullAccess,
                status: "running".to_string(),
                last_seen_at: "2026-03-04T12:00:02.000Z".to_string(),
                resume_cursor: None,
                runtime_payload: None,
            })
            .unwrap();
        assert_eq!(
            store
                .get_provider_runtime_binding_by_thread_id("legacy-thread")
                .unwrap()
                .and_then(|binding| binding.provider_instance_id),
            Some("claudeAgent".to_string())
        );

        let missing_instance = store.upsert_provider_runtime_binding(
            &ProviderRuntimeBinding {
                thread_id: "new-thread".to_string(),
                provider: "codex".to_string(),
                provider_instance_id: None,
                adapter_key: None,
                status: None,
                resume_cursor: None,
                runtime_payload: None,
                runtime_mode: None,
            },
            "2026-03-04T12:00:03.000Z",
        );
        assert!(matches!(
            missing_instance,
            Err(ProjectionPersistenceError::InvalidProviderRuntimeBinding(_))
        ));

        assert_eq!(
            store
                .list_provider_runtime_bindings()
                .unwrap()
                .into_iter()
                .map(|binding| binding.thread_id)
                .collect::<Vec<_>>(),
            vec!["thread-1", "legacy-thread"]
        );
    }

    #[test]
    fn projection_sqlite_store_appends_events_with_stream_versions_and_actor_kind() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        let first = store
            .append_event(&NewOrchestrationEventRow {
                event_id: "thread-event-1".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.message-appended".to_string(),
                occurred_at: "2026-03-04T12:00:00.000Z".to_string(),
                command_id: Some("client:append-message".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:append-message".to_string()),
                payload: json!({ "messageId": "message-first" }),
                metadata: json!({}),
            })
            .unwrap();
        let second = store
            .append_event(&NewOrchestrationEventRow {
                event_id: "thread-event-2".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.provider-item-added".to_string(),
                occurred_at: "2026-03-04T12:01:00.000Z".to_string(),
                command_id: Some("provider:append-item".to_string()),
                causation_event_id: Some("thread-event-1".to_string()),
                correlation_id: Some("client:append-message".to_string()),
                payload: json!({ "providerItemId": "provider-item-first" }),
                metadata: json!({ "providerTurnId": "turn-provider" }),
            })
            .unwrap();
        let server = store
            .append_event(&NewOrchestrationEventRow {
                event_id: "project-event-1".to_string(),
                aggregate_kind: "project".to_string(),
                aggregate_id: "project-first".to_string(),
                event_type: "project.created".to_string(),
                occurred_at: "2026-03-04T12:02:00.000Z".to_string(),
                command_id: None,
                causation_event_id: None,
                correlation_id: None,
                payload: json!({ "title": "Project" }),
                metadata: json!({}),
            })
            .unwrap();

        assert_eq!(first.sequence, 1);
        assert_eq!(first.stream_version, 0);
        assert_eq!(first.actor_kind, "client");
        assert_eq!(second.sequence, 2);
        assert_eq!(second.stream_version, 1);
        assert_eq!(second.actor_kind, "provider");
        assert_eq!(server.sequence, 3);
        assert_eq!(server.stream_version, 0);
        assert_eq!(server.actor_kind, "server");

        let first_two = store.read_events_from_sequence(0, Some(2)).unwrap();
        assert_eq!(
            first_two
                .iter()
                .map(|event| event.event_id.as_str())
                .collect::<Vec<_>>(),
            vec!["thread-event-1", "thread-event-2"]
        );
        assert!(
            store
                .read_events_from_sequence(0, Some(0))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn projection_sqlite_store_upserts_command_receipts() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        store
            .upsert_command_receipt(&OrchestrationCommandReceiptRow {
                command_id: "client:create-project".to_string(),
                aggregate_kind: "project".to_string(),
                aggregate_id: "project-first".to_string(),
                accepted_at: "2026-03-04T12:00:00.000Z".to_string(),
                result_sequence: 1,
                status: "accepted".to_string(),
                error: None,
            })
            .unwrap();
        let accepted = store
            .get_command_receipt("client:create-project")
            .unwrap()
            .unwrap();
        assert_eq!(accepted.result_sequence, 1);
        assert_eq!(accepted.status, "accepted");
        assert_eq!(accepted.error, None);

        store
            .upsert_command_receipt(&OrchestrationCommandReceiptRow {
                command_id: "client:create-project".to_string(),
                aggregate_kind: "project".to_string(),
                aggregate_id: "project-first".to_string(),
                accepted_at: "2026-03-04T12:00:01.000Z".to_string(),
                result_sequence: 2,
                status: "failed".to_string(),
                error: Some("boom".to_string()),
            })
            .unwrap();
        let failed = store
            .get_command_receipt("client:create-project")
            .unwrap()
            .unwrap();
        assert_eq!(failed.result_sequence, 2);
        assert_eq!(failed.status, "failed");
        assert_eq!(failed.error, Some("boom".to_string()));
    }

    #[test]
    fn projection_sqlite_store_projects_events_into_shell_and_detail_rows() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        let now = "2026-03-04T12:00:00.000Z";
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-project".to_string(),
                aggregate_kind: "project".to_string(),
                aggregate_id: "project-first".to_string(),
                event_type: "project.created".to_string(),
                occurred_at: now.to_string(),
                command_id: Some("client:create-project".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:create-project".to_string()),
                payload: json!({
                    "projectId": "project-first",
                    "title": "Project First",
                    "workspaceRoot": "/repo/project-first",
                    "defaultModelSelection": null,
                    "scripts": [],
                    "createdAt": now,
                    "updatedAt": now,
                }),
                metadata: json!({}),
            })
            .unwrap();
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-thread".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.created".to_string(),
                occurred_at: now.to_string(),
                command_id: Some("client:create-thread".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:create-thread".to_string()),
                payload: json!({
                    "threadId": "thread-first",
                    "projectId": "project-first",
                    "title": "Thread First",
                    "modelSelection": {
                        "instanceId": "codex",
                        "model": "gpt-5.4"
                    },
                    "runtimeMode": "full-access",
                    "interactionMode": "plan",
                    "branch": "main",
                    "worktreePath": "/repo/project-first",
                    "createdAt": now,
                    "updatedAt": now,
                }),
                metadata: json!({}),
            })
            .unwrap();
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-message".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.message-sent".to_string(),
                occurred_at: "2026-03-04T12:00:01.000Z".to_string(),
                command_id: Some("client:send-message".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:send-message".to_string()),
                payload: json!({
                    "threadId": "thread-first",
                    "messageId": "message-first",
                    "role": "user",
                    "text": "hello",
                    "attachments": [],
                    "turnId": null,
                    "streaming": false,
                    "createdAt": "2026-03-04T12:00:01.000Z",
                    "updatedAt": "2026-03-04T12:00:01.000Z",
                }),
                metadata: json!({}),
            })
            .unwrap();
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-turn-start".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.turn-start-requested".to_string(),
                occurred_at: "2026-03-04T12:00:01.500Z".to_string(),
                command_id: Some("server:start-turn".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:send-message".to_string()),
                payload: json!({
                    "threadId": "thread-first",
                    "messageId": "message-first",
                    "sourceProposedPlan": {
                        "threadId": "source-thread",
                        "planId": "source-plan"
                    },
                    "createdAt": "2026-03-04T12:00:01.500Z"
                }),
                metadata: json!({}),
            })
            .unwrap();
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-session".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.session-set".to_string(),
                occurred_at: "2026-03-04T12:00:02.000Z".to_string(),
                command_id: Some("server:set-session".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:send-message".to_string()),
                payload: json!({
                    "threadId": "thread-first",
                    "session": {
                        "status": "running",
                        "providerName": "Codex",
                        "providerInstanceId": "codex",
                        "runtimeMode": "full-access",
                        "activeTurnId": "turn-1",
                        "lastError": null,
                        "updatedAt": "2026-03-04T12:00:02.000Z"
                    }
                }),
                metadata: json!({}),
            })
            .unwrap();
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-assistant-message".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.message-sent".to_string(),
                occurred_at: "2026-03-04T12:00:03.000Z".to_string(),
                command_id: Some("provider:message".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:send-message".to_string()),
                payload: json!({
                    "threadId": "thread-first",
                    "messageId": "message-assistant",
                    "role": "assistant",
                    "text": "hi back",
                    "turnId": "turn-1",
                    "streaming": false,
                    "createdAt": "2026-03-04T12:00:03.000Z",
                    "updatedAt": "2026-03-04T12:00:03.000Z",
                }),
                metadata: json!({}),
            })
            .unwrap();
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-stream-start".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.message-sent".to_string(),
                occurred_at: "2026-03-04T12:00:03.500Z".to_string(),
                command_id: Some("provider:stream-start".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:send-message".to_string()),
                payload: json!({
                    "threadId": "thread-first",
                    "messageId": "message-stream",
                    "role": "assistant",
                    "text": "hel",
                    "turnId": "turn-1",
                    "streaming": true,
                    "createdAt": "2026-03-04T12:00:03.500Z",
                    "updatedAt": "2026-03-04T12:00:03.500Z",
                }),
                metadata: json!({}),
            })
            .unwrap();
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-stream-complete".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.message-sent".to_string(),
                occurred_at: "2026-03-04T12:00:03.600Z".to_string(),
                command_id: Some("provider:stream-complete".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:send-message".to_string()),
                payload: json!({
                    "threadId": "thread-first",
                    "messageId": "message-stream",
                    "role": "assistant",
                    "text": "",
                    "turnId": "turn-1",
                    "streaming": false,
                    "createdAt": "2026-03-04T12:00:03.600Z",
                    "updatedAt": "2026-03-04T12:00:03.600Z",
                }),
                metadata: json!({}),
            })
            .unwrap();
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-plan".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.proposed-plan-upserted".to_string(),
                occurred_at: "2026-03-04T12:00:04.000Z".to_string(),
                command_id: Some("provider:plan".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:send-message".to_string()),
                payload: json!({
                    "threadId": "thread-first",
                    "proposedPlan": {
                        "id": "plan-1",
                        "turnId": "turn-1",
                        "planMarkdown": "Do the thing",
                        "implementedAt": null,
                        "implementationThreadId": null,
                        "createdAt": "2026-03-04T12:00:04.000Z",
                        "updatedAt": "2026-03-04T12:00:04.000Z"
                    }
                }),
                metadata: json!({}),
            })
            .unwrap();
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-diff".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.turn-diff-completed".to_string(),
                occurred_at: "2026-03-04T12:00:05.000Z".to_string(),
                command_id: Some("provider:diff".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:send-message".to_string()),
                payload: json!({
                    "threadId": "thread-first",
                    "turnId": "turn-1",
                    "checkpointTurnCount": 1,
                    "checkpointRef": "checkpoint-1",
                    "status": "ready",
                    "files": [{ "path": "src/main.rs", "kind": "modified", "additions": 2, "deletions": 1 }],
                    "assistantMessageId": "message-assistant",
                    "completedAt": "2026-03-04T12:00:05.000Z"
                }),
                metadata: json!({}),
            })
            .unwrap();
        store
            .append_event(&NewOrchestrationEventRow {
                event_id: "evt-activity".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-first".to_string(),
                event_type: "thread.activity-appended".to_string(),
                occurred_at: "2026-03-04T12:00:06.000Z".to_string(),
                command_id: Some("provider:approval".to_string()),
                causation_event_id: None,
                correlation_id: Some("client:send-message".to_string()),
                payload: json!({
                    "threadId": "thread-first",
                    "activity": {
                        "id": "activity-approval",
                        "kind": "approval.requested",
                        "summary": "Approval requested",
                        "tone": "approval",
                        "payload": {
                            "requestId": "approval-1",
                            "requestType": "exec_command_approval",
                            "detail": "Run cargo test"
                        },
                        "turnId": "turn-1",
                        "sequence": 1,
                        "createdAt": "2026-03-04T12:00:06.000Z"
                    }
                }),
                metadata: json!({}),
            })
            .unwrap();

        assert_eq!(store.apply_pending_projection_events().unwrap(), 11);
        assert_eq!(store.apply_pending_projection_events().unwrap(), 0);

        let snapshot = store
            .load_shell_snapshot_input("environment-local")
            .unwrap();
        assert_eq!(snapshot.projects[0].title, "Project First");
        assert_eq!(snapshot.threads[0].title, "Thread First");
        assert_eq!(snapshot.threads[0].runtime_mode, RuntimeMode::FullAccess);
        assert_eq!(
            snapshot.threads[0].interaction_mode,
            ProviderInteractionMode::Plan
        );
        assert_eq!(
            snapshot.threads[0].latest_user_message_at.as_deref(),
            Some("2026-03-04T12:00:01.000Z")
        );
        assert_eq!(snapshot.threads[0].pending_approval_count, 1);
        assert!(snapshot.threads[0].has_actionable_proposed_plan);
        assert_eq!(
            snapshot.sessions[0].active_turn_id.as_deref(),
            Some("turn-1")
        );
        assert_eq!(snapshot.latest_turns[0].turn_id, "turn-1");
        assert_eq!(
            snapshot.latest_turns[0].requested_at,
            "2026-03-04T12:00:01.500Z"
        );
        assert_eq!(
            snapshot.latest_turns[0].source_proposed_plan_id.as_deref(),
            Some("source-plan")
        );

        let detail = store
            .load_thread_detail("environment-local", "thread-first")
            .unwrap()
            .unwrap();
        assert_eq!(detail.messages[0].id, "message-first");
        assert_eq!(detail.messages[0].text, "hello");
        assert_eq!(detail.messages[1].id, "message-assistant");
        assert_eq!(detail.messages[2].id, "message-stream");
        assert_eq!(detail.messages[2].text, "hel");
        assert_eq!(detail.proposed_plans[0].id, "plan-1");
        assert_eq!(detail.activities[0].id, "activity-approval");
        assert_eq!(detail.turn_diff_summaries[0].turn_id, "turn-1");
        assert_eq!(
            store
                .list_pending_approvals_by_thread("thread-first")
                .unwrap()[0]
                .request_id,
            "approval-1"
        );
        assert_eq!(
            store
                .list_projection_states()
                .unwrap()
                .iter()
                .map(|state| (state.projector.as_str(), state.last_applied_sequence))
                .collect::<Vec<_>>(),
            vec![
                ("projection.checkpoints", 11),
                ("projection.pending-approvals", 11),
                ("projection.projects", 11),
                ("projection.thread-activities", 11),
                ("projection.thread-messages", 11),
                ("projection.thread-proposed-plans", 11),
                ("projection.thread-sessions", 11),
                ("projection.thread-turns", 11),
                ("projection.threads", 11),
            ]
        );
    }

    #[test]
    fn projection_sqlite_store_projects_interrupts_and_reverts_from_events() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        let now = "2026-03-04T13:00:00.000Z";
        for event in [
            NewOrchestrationEventRow {
                event_id: "revert-project".to_string(),
                aggregate_kind: "project".to_string(),
                aggregate_id: "project-revert".to_string(),
                event_type: "project.created".to_string(),
                occurred_at: now.to_string(),
                command_id: Some("client:create-project".to_string()),
                causation_event_id: None,
                correlation_id: None,
                payload: json!({
                    "projectId": "project-revert",
                    "title": "Project Revert",
                    "workspaceRoot": "/repo/project-revert",
                    "defaultModelSelection": null,
                    "scripts": [],
                    "createdAt": now,
                    "updatedAt": now,
                }),
                metadata: json!({}),
            },
            NewOrchestrationEventRow {
                event_id: "revert-thread".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-revert".to_string(),
                event_type: "thread.created".to_string(),
                occurred_at: now.to_string(),
                command_id: Some("client:create-thread".to_string()),
                causation_event_id: None,
                correlation_id: None,
                payload: json!({
                    "threadId": "thread-revert",
                    "projectId": "project-revert",
                    "title": "Thread Revert",
                    "modelSelection": null,
                    "runtimeMode": "full-access",
                    "interactionMode": "default",
                    "branch": null,
                    "worktreePath": null,
                    "createdAt": now,
                    "updatedAt": now,
                }),
                metadata: json!({}),
            },
            NewOrchestrationEventRow {
                event_id: "interrupt-turn".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-revert".to_string(),
                event_type: "thread.turn-interrupt-requested".to_string(),
                occurred_at: "2026-03-04T13:00:01.000Z".to_string(),
                command_id: Some("client:interrupt".to_string()),
                causation_event_id: None,
                correlation_id: None,
                payload: json!({
                    "threadId": "thread-revert",
                    "turnId": "turn-1",
                    "createdAt": "2026-03-04T13:00:01.000Z",
                }),
                metadata: json!({}),
            },
            NewOrchestrationEventRow {
                event_id: "checkpoint-one".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-revert".to_string(),
                event_type: "thread.turn-diff-completed".to_string(),
                occurred_at: "2026-03-04T13:00:02.000Z".to_string(),
                command_id: Some("provider:checkpoint-one".to_string()),
                causation_event_id: None,
                correlation_id: None,
                payload: json!({
                    "threadId": "thread-revert",
                    "turnId": "turn-1",
                    "checkpointTurnCount": 1,
                    "checkpointRef": "checkpoint-one",
                    "status": "ready",
                    "files": [],
                    "assistantMessageId": null,
                    "completedAt": "2026-03-04T13:00:02.000Z",
                }),
                metadata: json!({}),
            },
            NewOrchestrationEventRow {
                event_id: "message-two".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-revert".to_string(),
                event_type: "thread.message-sent".to_string(),
                occurred_at: "2026-03-04T13:00:03.000Z".to_string(),
                command_id: Some("provider:message-two".to_string()),
                causation_event_id: None,
                correlation_id: None,
                payload: json!({
                    "threadId": "thread-revert",
                    "messageId": "message-two",
                    "role": "assistant",
                    "text": "second turn",
                    "turnId": "turn-2",
                    "streaming": false,
                    "createdAt": "2026-03-04T13:00:03.000Z",
                    "updatedAt": "2026-03-04T13:00:03.000Z",
                }),
                metadata: json!({}),
            },
            NewOrchestrationEventRow {
                event_id: "activity-two".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-revert".to_string(),
                event_type: "thread.activity-appended".to_string(),
                occurred_at: "2026-03-04T13:00:03.500Z".to_string(),
                command_id: Some("provider:activity-two".to_string()),
                causation_event_id: None,
                correlation_id: None,
                payload: json!({
                    "threadId": "thread-revert",
                    "activity": {
                        "id": "activity-two",
                        "kind": "tool.completed",
                        "summary": "Tool completed",
                        "tone": "tool",
                        "payload": {},
                        "turnId": "turn-2",
                        "sequence": 2,
                        "createdAt": "2026-03-04T13:00:03.500Z"
                    }
                }),
                metadata: json!({}),
            },
            NewOrchestrationEventRow {
                event_id: "checkpoint-two".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-revert".to_string(),
                event_type: "thread.turn-diff-completed".to_string(),
                occurred_at: "2026-03-04T13:00:04.000Z".to_string(),
                command_id: Some("provider:checkpoint-two".to_string()),
                causation_event_id: None,
                correlation_id: None,
                payload: json!({
                    "threadId": "thread-revert",
                    "turnId": "turn-2",
                    "checkpointTurnCount": 2,
                    "checkpointRef": "checkpoint-two",
                    "status": "ready",
                    "files": [],
                    "assistantMessageId": "message-two",
                    "completedAt": "2026-03-04T13:00:04.000Z",
                }),
                metadata: json!({}),
            },
            NewOrchestrationEventRow {
                event_id: "revert-one".to_string(),
                aggregate_kind: "thread".to_string(),
                aggregate_id: "thread-revert".to_string(),
                event_type: "thread.reverted".to_string(),
                occurred_at: "2026-03-04T13:00:05.000Z".to_string(),
                command_id: Some("client:revert".to_string()),
                causation_event_id: None,
                correlation_id: None,
                payload: json!({
                    "threadId": "thread-revert",
                    "turnCount": 1,
                }),
                metadata: json!({}),
            },
        ] {
            store.append_event(&event).unwrap();
        }

        assert_eq!(store.apply_pending_projection_events().unwrap(), 8);
        let snapshot = store
            .load_shell_snapshot_input("environment-local")
            .unwrap();
        assert_eq!(snapshot.latest_turns[0].turn_id, "turn-1");
        assert_eq!(snapshot.latest_turns[0].state, "completed");
        let detail = store
            .load_thread_detail("environment-local", "thread-revert")
            .unwrap()
            .unwrap();
        assert!(detail.messages.is_empty());
        assert!(detail.activities.is_empty());
        assert_eq!(detail.turn_diff_summaries.len(), 1);
        assert_eq!(detail.turn_diff_summaries[0].turn_id, "turn-1");
    }

    #[test]
    fn projection_sqlite_store_executes_decided_commands_into_events_receipts_and_projections() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        store
            .execute_orchestration_command(
                &OrchestrationCommand::ProjectCreate {
                    command_id: "cmd-project".to_string(),
                    project_id: "project-engine".to_string(),
                    title: "Project Engine".to_string(),
                    workspace_root: "/repo/project-engine".to_string(),
                    default_model_selection: None,
                    created_at: "2026-03-04T14:00:00.000Z".to_string(),
                },
                "2026-03-04T14:00:00.000Z",
            )
            .unwrap();
        store
            .execute_orchestration_command(
                &OrchestrationCommand::ThreadCreate {
                    command_id: "cmd-thread".to_string(),
                    thread_id: "thread-engine".to_string(),
                    project_id: "project-engine".to_string(),
                    title: "Thread Engine".to_string(),
                    model_selection: json!({ "instanceId": "codex", "model": "gpt-5.4" }),
                    runtime_mode: RuntimeMode::FullAccess,
                    interaction_mode: ProviderInteractionMode::Plan,
                    branch: Some("main".to_string()),
                    worktree_path: Some("/repo/project-engine".to_string()),
                    created_at: "2026-03-04T14:00:01.000Z".to_string(),
                },
                "2026-03-04T14:00:01.000Z",
            )
            .unwrap();
        let turn_events = store
            .execute_orchestration_command(
                &OrchestrationCommand::ThreadTurnStart {
                    command_id: "cmd-turn".to_string(),
                    thread_id: "thread-engine".to_string(),
                    message_id: "message-engine".to_string(),
                    text: "hello engine".to_string(),
                    attachments: Vec::new(),
                    model_selection: None,
                    title_seed: None,
                    source_proposed_plan: None::<SourceProposedPlanRef>,
                    created_at: "2026-03-04T14:00:02.000Z".to_string(),
                },
                "2026-03-04T14:00:02.000Z",
            )
            .unwrap();

        assert_eq!(
            turn_events
                .iter()
                .map(|event| event.event_type.as_str())
                .collect::<Vec<_>>(),
            vec!["thread.message-sent", "thread.turn-start-requested"]
        );
        assert_eq!(
            store
                .get_command_receipt("cmd-turn")
                .unwrap()
                .unwrap()
                .status,
            "succeeded"
        );
        let detail = store
            .load_thread_detail("environment-local", "thread-engine")
            .unwrap()
            .unwrap();
        assert_eq!(detail.messages[0].text, "hello engine");
        assert_eq!(
            store
                .get_pending_turn_start("thread-engine")
                .unwrap()
                .unwrap()
                .pending_message_id
                .as_deref(),
            Some("message-engine")
        );
        let provider_intents = store.provider_command_intents_from_sequence(0).unwrap();
        assert_eq!(provider_intents.len(), 1);
        assert_eq!(
            provider_intents[0].0.event_type,
            "thread.turn-start-requested"
        );
        assert_eq!(
            provider_intents[0].1,
            ProviderCommandIntent::TurnStart {
                thread_id: "thread-engine".to_string(),
                message_id: "message-engine".to_string(),
                runtime_mode: RuntimeMode::FullAccess,
                interaction_mode: ProviderInteractionMode::Plan,
            }
        );
        assert!(
            store
                .provider_command_intents_from_sequence(provider_intents[0].0.sequence)
                .unwrap()
                .is_empty()
        );
        let reactor_batch = store.reactor_batch_from_sequence(0).unwrap();
        assert_eq!(reactor_batch.high_water_sequence, turn_events[1].sequence);
        assert_eq!(reactor_batch.provider_requests.len(), 1);
        assert_eq!(
            reactor_batch.provider_requests[0].request,
            ProviderServiceRequest::BuildAndSendTurn {
                thread_id: "thread-engine".to_string(),
                message_id: "message-engine".to_string(),
                runtime_mode: RuntimeMode::FullAccess,
                interaction_mode: ProviderInteractionMode::Plan,
            }
        );
        assert!(reactor_batch.thread_cleanup_requests.is_empty());
    }

    #[test]
    fn projection_sqlite_store_maps_persisted_thread_delete_to_cleanup_actions() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        store
            .execute_orchestration_command(
                &OrchestrationCommand::ProjectCreate {
                    command_id: "cmd-project-cleanup".to_string(),
                    project_id: "project-cleanup".to_string(),
                    title: "Project Cleanup".to_string(),
                    workspace_root: "/repo/project-cleanup".to_string(),
                    default_model_selection: None,
                    created_at: "2026-03-04T15:00:00.000Z".to_string(),
                },
                "2026-03-04T15:00:00.000Z",
            )
            .unwrap();
        store
            .execute_orchestration_command(
                &OrchestrationCommand::ThreadCreate {
                    command_id: "cmd-thread-cleanup".to_string(),
                    thread_id: "thread-cleanup".to_string(),
                    project_id: "project-cleanup".to_string(),
                    title: "Thread Cleanup".to_string(),
                    model_selection: json!({ "instanceId": "codex", "model": "gpt-5.4" }),
                    runtime_mode: RuntimeMode::AutoAcceptEdits,
                    interaction_mode: ProviderInteractionMode::Default,
                    branch: None,
                    worktree_path: Some("/repo/project-cleanup".to_string()),
                    created_at: "2026-03-04T15:00:01.000Z".to_string(),
                },
                "2026-03-04T15:00:01.000Z",
            )
            .unwrap();
        store
            .execute_orchestration_command(
                &OrchestrationCommand::ThreadDelete {
                    command_id: "cmd-thread-delete-cleanup".to_string(),
                    thread_id: "thread-cleanup".to_string(),
                },
                "2026-03-04T15:00:02.000Z",
            )
            .unwrap();

        let cleanup_actions = store
            .thread_deletion_cleanup_actions_from_sequence(0)
            .unwrap();
        assert_eq!(cleanup_actions.len(), 1);
        assert_eq!(cleanup_actions[0].0.event_type, "thread.deleted");
        assert_eq!(
            cleanup_actions[0].1,
            vec![
                ThreadDeletionCleanupAction::StopProviderSession,
                ThreadDeletionCleanupAction::CloseThreadTerminalsAndDeleteHistory,
            ]
        );
        assert!(
            store
                .thread_deletion_cleanup_actions_from_sequence(cleanup_actions[0].0.sequence)
                .unwrap()
                .is_empty()
        );
        let reactor_batch = store.reactor_batch_from_sequence(0).unwrap();
        assert_eq!(
            reactor_batch.high_water_sequence,
            cleanup_actions[0].0.sequence
        );
        assert!(reactor_batch.provider_requests.is_empty());
        assert_eq!(
            reactor_batch
                .thread_cleanup_requests
                .iter()
                .map(|entry| entry.request.clone())
                .collect::<Vec<_>>(),
            vec![
                ThreadDeletionCleanupRequest::StopProviderSession(ProviderStopSessionInput {
                    thread_id: "thread-cleanup".to_string(),
                }),
                ThreadDeletionCleanupRequest::CloseThreadTerminalsAndDeleteHistory {
                    thread_id: "thread-cleanup".to_string(),
                },
            ]
        );
    }

    #[test]
    fn projection_sqlite_store_projects_provider_runtime_activity_commands() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        store
            .execute_orchestration_command(
                &OrchestrationCommand::ProjectCreate {
                    command_id: "cmd-project-runtime-activity".to_string(),
                    project_id: "project-runtime-activity".to_string(),
                    title: "Project Runtime Activity".to_string(),
                    workspace_root: "/repo/project-runtime-activity".to_string(),
                    default_model_selection: None,
                    created_at: "2026-03-04T16:00:00.000Z".to_string(),
                },
                "2026-03-04T16:00:00.000Z",
            )
            .unwrap();
        store
            .execute_orchestration_command(
                &OrchestrationCommand::ThreadCreate {
                    command_id: "cmd-thread-runtime-activity".to_string(),
                    thread_id: "thread-runtime-activity".to_string(),
                    project_id: "project-runtime-activity".to_string(),
                    title: "Thread Runtime Activity".to_string(),
                    model_selection: json!({ "instanceId": "codex", "model": "gpt-5.4" }),
                    runtime_mode: RuntimeMode::FullAccess,
                    interaction_mode: ProviderInteractionMode::Default,
                    branch: None,
                    worktree_path: Some("/repo/project-runtime-activity".to_string()),
                    created_at: "2026-03-04T16:00:01.000Z".to_string(),
                },
                "2026-03-04T16:00:01.000Z",
            )
            .unwrap();

        let activities = provider_runtime_event_to_activities(&ProviderRuntimeEventInput {
            event_type: "request.opened".to_string(),
            event_id: "runtime-approval-opened".to_string(),
            created_at: "2026-03-04T16:00:02.000Z".to_string(),
            turn_id: Some("turn-runtime".to_string()),
            request_id: Some("approval-runtime".to_string()),
            item_id: None,
            payload: json!({
                "requestType": "exec_command_approval",
                "detail": "run cargo test",
            }),
            session_sequence: Some(9),
        });
        store
            .execute_orchestration_command(
                &provider_runtime_activity_to_thread_activity_append_command(
                    "thread-runtime-activity",
                    "provider:runtime-approval-opened:activity",
                    &activities[0],
                ),
                "2026-03-04T16:00:02.000Z",
            )
            .unwrap();

        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert_eq!(detail.activities.len(), 1);
        assert_eq!(detail.activities[0].kind, "approval.requested");
        assert_eq!(detail.activities[0].sequence, Some(9));
        assert_eq!(
            detail.activities[0].payload.request_id.as_deref(),
            Some("approval-runtime")
        );
        assert_eq!(
            store
                .get_pending_approval("approval-runtime")
                .unwrap()
                .unwrap()
                .status,
            "pending"
        );

        store
            .execute_orchestration_command(
                &provider_runtime_lifecycle_session_command(
                    &ProviderRuntimeSessionContext {
                        thread_id: "thread-runtime-activity".to_string(),
                        provider: "codex".to_string(),
                        provider_instance_id: Some("codex".to_string()),
                        runtime_mode: RuntimeMode::FullAccess,
                        active_turn_id: None,
                        last_error: None,
                    },
                    "provider:runtime-turn-started:thread-session-set",
                    &ProviderRuntimeEventInput {
                        event_type: "turn.started".to_string(),
                        event_id: "runtime-turn-started".to_string(),
                        created_at: "2026-03-04T16:00:03.000Z".to_string(),
                        turn_id: Some("turn-runtime".to_string()),
                        request_id: None,
                        item_id: None,
                        payload: json!({}),
                        session_sequence: None,
                    },
                )
                .unwrap(),
                "2026-03-04T16:00:03.000Z",
            )
            .unwrap();
        let session = store
            .get_thread_session("thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert_eq!(session.status, "running");
        assert_eq!(session.active_turn_id.as_deref(), Some("turn-runtime"));
        assert_eq!(session.provider_name, "codex");
        assert_eq!(session.provider_instance_id.as_deref(), Some("codex"));

        store
            .execute_orchestration_command(
                &provider_runtime_assistant_delta_command(
                    "thread-runtime-activity",
                    "provider:runtime-assistant-delta:assistant-delta",
                    &ProviderRuntimeEventInput {
                        event_type: "content.delta".to_string(),
                        event_id: "runtime-assistant-delta".to_string(),
                        created_at: "2026-03-04T16:00:04.000Z".to_string(),
                        turn_id: Some("turn-runtime".to_string()),
                        request_id: None,
                        item_id: Some("assistant-runtime".to_string()),
                        payload: json!({
                            "streamKind": "assistant_text",
                            "delta": "working",
                        }),
                        session_sequence: None,
                    },
                )
                .unwrap(),
                "2026-03-04T16:00:04.000Z",
            )
            .unwrap();
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        let assistant = detail
            .messages
            .iter()
            .find(|message| message.id == "assistant:assistant-runtime")
            .unwrap();
        assert_eq!(assistant.text, "working");
        assert_eq!(assistant.turn_id.as_deref(), Some("turn-runtime"));
        assert!(assistant.streaming);

        store
            .execute_orchestration_command(
                &provider_runtime_assistant_complete_command(
                    "thread-runtime-activity",
                    "provider:runtime-assistant-complete:assistant-complete",
                    &ProviderRuntimeEventInput {
                        event_type: "item.completed".to_string(),
                        event_id: "runtime-assistant-complete".to_string(),
                        created_at: "2026-03-04T16:00:05.000Z".to_string(),
                        turn_id: Some("turn-runtime".to_string()),
                        request_id: None,
                        item_id: Some("assistant-runtime".to_string()),
                        payload: json!({
                            "itemType": "assistant_message",
                            "detail": "working",
                        }),
                        session_sequence: None,
                    },
                )
                .unwrap(),
                "2026-03-04T16:00:05.000Z",
            )
            .unwrap();
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        let assistant = detail
            .messages
            .iter()
            .find(|message| message.id == "assistant:assistant-runtime")
            .unwrap();
        assert_eq!(assistant.text, "working");
        assert!(!assistant.streaming);

        store
            .execute_orchestration_command(
                &provider_runtime_proposed_plan_complete_command(
                    "thread-runtime-activity",
                    "provider:runtime-proposed-complete:proposed-plan",
                    &ProviderRuntimeEventInput {
                        event_type: "turn.proposed.completed".to_string(),
                        event_id: "runtime-proposed-complete".to_string(),
                        created_at: "2026-03-04T16:00:06.000Z".to_string(),
                        turn_id: Some("turn-runtime".to_string()),
                        request_id: None,
                        item_id: Some("plan-runtime".to_string()),
                        payload: json!({
                            "planMarkdown": "  - keep parity\n ",
                        }),
                        session_sequence: None,
                    },
                    None,
                )
                .unwrap(),
                "2026-03-04T16:00:06.000Z",
            )
            .unwrap();
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert_eq!(detail.proposed_plans.len(), 1);
        assert_eq!(
            detail.proposed_plans[0].id,
            "plan:thread-runtime-activity:turn:turn-runtime"
        );
        assert_eq!(detail.proposed_plans[0].plan_markdown, "- keep parity");

        store
            .execute_orchestration_command(
                &provider_runtime_turn_diff_complete_command(
                    "thread-runtime-activity",
                    "provider:runtime-diff-updated:turn-diff",
                    &ProviderRuntimeEventInput {
                        event_type: "turn.diff.updated".to_string(),
                        event_id: "runtime-diff-updated".to_string(),
                        created_at: "2026-03-04T16:00:07.000Z".to_string(),
                        turn_id: Some("turn-runtime".to_string()),
                        request_id: None,
                        item_id: Some("assistant-runtime".to_string()),
                        payload: json!({}),
                        session_sequence: None,
                    },
                    1,
                )
                .unwrap(),
                "2026-03-04T16:00:07.000Z",
            )
            .unwrap();
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert_eq!(detail.turn_diff_summaries.len(), 1);
        assert_eq!(detail.turn_diff_summaries[0].turn_id, "turn-runtime");
        assert_eq!(
            detail.turn_diff_summaries[0].status.as_deref(),
            Some("missing")
        );
        assert!(detail.turn_diff_summaries[0].files.is_empty());
        assert_eq!(
            detail.turn_diff_summaries[0].checkpoint_ref.as_deref(),
            Some("provider-diff:runtime-diff-updated")
        );
        assert_eq!(
            detail.turn_diff_summaries[0]
                .assistant_message_id
                .as_deref(),
            Some("assistant:assistant-runtime")
        );
        assert_eq!(detail.turn_diff_summaries[0].checkpoint_turn_count, Some(1));

        let runtime_error_events = store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: Some(ProviderRuntimeSessionContext {
                        thread_id: "thread-runtime-activity".to_string(),
                        provider: "codex".to_string(),
                        provider_instance_id: Some("codex".to_string()),
                        runtime_mode: RuntimeMode::FullAccess,
                        active_turn_id: Some("turn-runtime".to_string()),
                        last_error: None,
                    }),
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: None,
                    assistant_completion_projected_text_is_empty: None,
                    turn_completion_assistant_message_ids: Vec::new(),
                    pause_assistant_message_ids: Vec::new(),
                    pause_assistant_buffered_texts: Vec::new(),
                    turn_completion_assistant_buffered_texts: Vec::new(),
                    turn_completion_proposed_plan_markdown: None,
                },
                &ProviderRuntimeEventInput {
                    event_type: "runtime.error".to_string(),
                    event_id: "runtime-error".to_string(),
                    created_at: "2026-03-04T16:00:08.000Z".to_string(),
                    turn_id: Some("turn-runtime".to_string()),
                    request_id: None,
                    item_id: None,
                    payload: json!({ "message": "provider crashed" }),
                    session_sequence: Some(10),
                },
                "2026-03-04T16:00:08.000Z",
            )
            .unwrap();
        assert_eq!(runtime_error_events.len(), 2);
        let session = store
            .get_thread_session("thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert_eq!(session.status, "error");
        assert_eq!(session.last_error.as_deref(), Some("provider crashed"));
        assert!(session.active_turn_id.is_none());
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert!(
            detail
                .activities
                .iter()
                .any(|activity| activity.kind == "runtime.error"
                    && activity.payload.detail.as_deref() == Some("provider crashed"))
        );

        let stale_runtime_error_events = store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: Some(ProviderRuntimeSessionContext {
                        thread_id: "thread-runtime-activity".to_string(),
                        provider: "codex".to_string(),
                        provider_instance_id: Some("codex".to_string()),
                        runtime_mode: RuntimeMode::FullAccess,
                        active_turn_id: Some("turn-active".to_string()),
                        last_error: None,
                    }),
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: None,
                    assistant_completion_projected_text_is_empty: None,
                    turn_completion_assistant_message_ids: Vec::new(),
                    pause_assistant_message_ids: Vec::new(),
                    pause_assistant_buffered_texts: Vec::new(),
                    turn_completion_assistant_buffered_texts: Vec::new(),
                    turn_completion_proposed_plan_markdown: None,
                },
                &ProviderRuntimeEventInput {
                    event_type: "runtime.error".to_string(),
                    event_id: "runtime-error-stale".to_string(),
                    created_at: "2026-03-04T16:00:08.500Z".to_string(),
                    turn_id: Some("turn-stale".to_string()),
                    request_id: None,
                    item_id: None,
                    payload: json!({ "message": "stale crash" }),
                    session_sequence: Some(10),
                },
                "2026-03-04T16:00:08.500Z",
            )
            .unwrap();
        assert_eq!(stale_runtime_error_events.len(), 1);
        assert_eq!(
            stale_runtime_error_events[0].event_type,
            "thread.activity-appended"
        );

        let title_events = store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: None,
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: None,
                    assistant_completion_projected_text_is_empty: None,
                    turn_completion_assistant_message_ids: Vec::new(),
                    pause_assistant_message_ids: Vec::new(),
                    pause_assistant_buffered_texts: Vec::new(),
                    turn_completion_assistant_buffered_texts: Vec::new(),
                    turn_completion_proposed_plan_markdown: None,
                },
                &ProviderRuntimeEventInput {
                    event_type: "thread.metadata.updated".to_string(),
                    event_id: "runtime-title".to_string(),
                    created_at: "2026-03-04T16:00:09.000Z".to_string(),
                    turn_id: None,
                    request_id: None,
                    item_id: None,
                    payload: json!({ "name": "  Runtime title  " }),
                    session_sequence: None,
                },
                "2026-03-04T16:00:09.000Z",
            )
            .unwrap();
        assert_eq!(title_events.len(), 1);
        let thread = store
            .get_thread("thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert_eq!(thread.title, "Runtime title");

        let fallback_events = store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: None,
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: Some(false),
                    assistant_completion_projected_text_is_empty: Some(true),
                    turn_completion_assistant_message_ids: Vec::new(),
                    pause_assistant_message_ids: Vec::new(),
                    pause_assistant_buffered_texts: Vec::new(),
                    turn_completion_assistant_buffered_texts: Vec::new(),
                    turn_completion_proposed_plan_markdown: None,
                },
                &ProviderRuntimeEventInput {
                    event_type: "item.completed".to_string(),
                    event_id: "runtime-assistant-fallback".to_string(),
                    created_at: "2026-03-04T16:00:10.000Z".to_string(),
                    turn_id: Some("turn-fallback".to_string()),
                    request_id: None,
                    item_id: Some("assistant-fallback".to_string()),
                    payload: json!({
                        "itemType": "assistant_message",
                        "detail": "fallback text",
                    }),
                    session_sequence: None,
                },
                "2026-03-04T16:00:10.000Z",
            )
            .unwrap();
        assert_eq!(fallback_events.len(), 2);
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        let assistant = detail
            .messages
            .iter()
            .find(|message| message.id == "assistant:assistant-fallback")
            .unwrap();
        assert_eq!(assistant.text, "fallback text");
        assert_eq!(assistant.turn_id.as_deref(), Some("turn-fallback"));
        assert!(!assistant.streaming);

        store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: None,
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: None,
                    assistant_completion_projected_text_is_empty: None,
                    turn_completion_assistant_message_ids: Vec::new(),
                    pause_assistant_message_ids: Vec::new(),
                    pause_assistant_buffered_texts: Vec::new(),
                    turn_completion_assistant_buffered_texts: Vec::new(),
                    turn_completion_proposed_plan_markdown: None,
                },
                &ProviderRuntimeEventInput {
                    event_type: "content.delta".to_string(),
                    event_id: "runtime-turn-complete-delta".to_string(),
                    created_at: "2026-03-04T16:00:10.500Z".to_string(),
                    turn_id: Some("turn-complete".to_string()),
                    request_id: None,
                    item_id: Some("turn-complete".to_string()),
                    payload: json!({
                        "streamKind": "assistant_text",
                        "delta": "streamed",
                    }),
                    session_sequence: None,
                },
                "2026-03-04T16:00:10.500Z",
            )
            .unwrap();
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert!(
            detail
                .messages
                .iter()
                .find(|message| message.id == "assistant:turn-complete")
                .unwrap()
                .streaming
        );
        let turn_complete_events = store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: None,
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: None,
                    assistant_completion_projected_text_is_empty: None,
                    turn_completion_assistant_message_ids: vec![
                        "assistant:turn-complete".to_string(),
                    ],
                    pause_assistant_message_ids: Vec::new(),
                    pause_assistant_buffered_texts: Vec::new(),
                    turn_completion_assistant_buffered_texts: vec![
                        ProviderRuntimeBufferedAssistantText {
                            message_id: "assistant:turn-complete".to_string(),
                            text: " final".to_string(),
                        },
                    ],
                    turn_completion_proposed_plan_markdown: None,
                },
                &ProviderRuntimeEventInput {
                    event_type: "turn.completed".to_string(),
                    event_id: "runtime-turn-complete".to_string(),
                    created_at: "2026-03-04T16:00:10.750Z".to_string(),
                    turn_id: Some("turn-complete".to_string()),
                    request_id: None,
                    item_id: None,
                    payload: json!({ "state": "completed" }),
                    session_sequence: None,
                },
                "2026-03-04T16:00:10.750Z",
            )
            .unwrap();
        assert_eq!(turn_complete_events.len(), 2);
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        let assistant = detail
            .messages
            .iter()
            .find(|message| message.id == "assistant:turn-complete")
            .unwrap();
        assert_eq!(assistant.text, "streamed final");
        assert!(!assistant.streaming);

        let turn_plan_events = store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: None,
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: None,
                    assistant_completion_projected_text_is_empty: None,
                    turn_completion_assistant_message_ids: Vec::new(),
                    pause_assistant_message_ids: Vec::new(),
                    pause_assistant_buffered_texts: Vec::new(),
                    turn_completion_assistant_buffered_texts: Vec::new(),
                    turn_completion_proposed_plan_markdown: Some("  - ship parity\n ".to_string()),
                },
                &ProviderRuntimeEventInput {
                    event_type: "turn.completed".to_string(),
                    event_id: "runtime-turn-plan".to_string(),
                    created_at: "2026-03-04T16:00:10.800Z".to_string(),
                    turn_id: Some("turn-plan".to_string()),
                    request_id: None,
                    item_id: None,
                    payload: json!({ "state": "completed" }),
                    session_sequence: None,
                },
                "2026-03-04T16:00:10.800Z",
            )
            .unwrap();
        assert_eq!(turn_plan_events.len(), 1);
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        let proposed_plan = detail
            .proposed_plans
            .iter()
            .find(|plan| plan.id == "plan:thread-runtime-activity:turn:turn-plan")
            .unwrap();
        assert_eq!(proposed_plan.turn_id.as_deref(), Some("turn-plan"));
        assert_eq!(proposed_plan.plan_markdown, "- ship parity");

        store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: None,
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: None,
                    assistant_completion_projected_text_is_empty: None,
                    turn_completion_assistant_message_ids: Vec::new(),
                    pause_assistant_message_ids: Vec::new(),
                    pause_assistant_buffered_texts: Vec::new(),
                    turn_completion_assistant_buffered_texts: Vec::new(),
                    turn_completion_proposed_plan_markdown: None,
                },
                &ProviderRuntimeEventInput {
                    event_type: "content.delta".to_string(),
                    event_id: "runtime-pause-delta".to_string(),
                    created_at: "2026-03-04T16:00:10.875Z".to_string(),
                    turn_id: Some("turn-pause".to_string()),
                    request_id: None,
                    item_id: Some("pause".to_string()),
                    payload: json!({
                        "streamKind": "assistant_text",
                        "delta": "needs approval",
                    }),
                    session_sequence: None,
                },
                "2026-03-04T16:00:10.875Z",
            )
            .unwrap();
        let pause_events = store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: None,
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: None,
                    assistant_completion_projected_text_is_empty: None,
                    turn_completion_assistant_message_ids: Vec::new(),
                    pause_assistant_message_ids: vec!["assistant:pause".to_string()],
                    pause_assistant_buffered_texts: vec![ProviderRuntimeBufferedAssistantText {
                        message_id: "assistant:pause".to_string(),
                        text: " and wait".to_string(),
                    }],
                    turn_completion_assistant_buffered_texts: Vec::new(),
                    turn_completion_proposed_plan_markdown: None,
                },
                &ProviderRuntimeEventInput {
                    event_type: "request.opened".to_string(),
                    event_id: "runtime-pause-request".to_string(),
                    created_at: "2026-03-04T16:00:10.900Z".to_string(),
                    turn_id: Some("turn-pause".to_string()),
                    request_id: Some("approval-pause".to_string()),
                    item_id: None,
                    payload: json!({
                        "requestType": "exec_command_approval",
                        "detail": "run tests",
                    }),
                    session_sequence: Some(13),
                },
                "2026-03-04T16:00:10.900Z",
            )
            .unwrap();
        assert_eq!(pause_events.len(), 3);
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        let assistant = detail
            .messages
            .iter()
            .find(|message| message.id == "assistant:pause")
            .unwrap();
        assert_eq!(assistant.text, "needs approval and wait");
        assert!(!assistant.streaming);
        assert_eq!(
            store
                .get_pending_approval("approval-pause")
                .unwrap()
                .unwrap()
                .status,
            "pending"
        );

        let user_input_requested = store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: None,
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: None,
                    assistant_completion_projected_text_is_empty: None,
                    turn_completion_assistant_message_ids: Vec::new(),
                    pause_assistant_message_ids: Vec::new(),
                    pause_assistant_buffered_texts: Vec::new(),
                    turn_completion_assistant_buffered_texts: Vec::new(),
                    turn_completion_proposed_plan_markdown: None,
                },
                &ProviderRuntimeEventInput {
                    event_type: "user-input.requested".to_string(),
                    event_id: "runtime-user-input-requested".to_string(),
                    created_at: "2026-03-04T16:00:11.000Z".to_string(),
                    turn_id: Some("turn-fallback".to_string()),
                    request_id: Some("user-input-runtime".to_string()),
                    item_id: None,
                    payload: json!({
                        "questions": [{
                            "id": "scope",
                            "header": "Scope",
                            "question": "What should be ported next?",
                            "multiSelect": false,
                            "options": [{
                                "label": "Runtime",
                                "description": "Continue runtime parity."
                            }]
                        }]
                    }),
                    session_sequence: Some(11),
                },
                "2026-03-04T16:00:11.000Z",
            )
            .unwrap();
        assert_eq!(user_input_requested.len(), 1);
        let thread = store
            .get_thread("thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert_eq!(thread.pending_user_input_count, 1);
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        let pending_user_inputs = derive_pending_user_inputs(&detail.activities);
        assert_eq!(pending_user_inputs.len(), 1);
        assert_eq!(pending_user_inputs[0].request_id, "user-input-runtime");
        assert_eq!(pending_user_inputs[0].questions[0].id, "scope");

        let user_input_resolved = store
            .ingest_provider_runtime_event(
                &ProviderRuntimeIngestionCommandPlanContext {
                    thread_id: "thread-runtime-activity".to_string(),
                    session_context: None,
                    existing_proposed_plan_created_at: None,
                    next_checkpoint_turn_count: None,
                    assistant_completion_has_projected_message: None,
                    assistant_completion_projected_text_is_empty: None,
                    turn_completion_assistant_message_ids: Vec::new(),
                    pause_assistant_message_ids: Vec::new(),
                    pause_assistant_buffered_texts: Vec::new(),
                    turn_completion_assistant_buffered_texts: Vec::new(),
                    turn_completion_proposed_plan_markdown: None,
                },
                &ProviderRuntimeEventInput {
                    event_type: "user-input.resolved".to_string(),
                    event_id: "runtime-user-input-resolved".to_string(),
                    created_at: "2026-03-04T16:00:12.000Z".to_string(),
                    turn_id: Some("turn-fallback".to_string()),
                    request_id: Some("user-input-runtime".to_string()),
                    item_id: None,
                    payload: json!({ "answers": { "scope": "Runtime" } }),
                    session_sequence: Some(12),
                },
                "2026-03-04T16:00:12.000Z",
            )
            .unwrap();
        assert_eq!(user_input_resolved.len(), 1);
        let thread = store
            .get_thread("thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert_eq!(thread.pending_user_input_count, 0);
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-activity")
            .unwrap()
            .unwrap();
        assert!(derive_pending_user_inputs(&detail.activities).is_empty());
    }

    #[test]
    fn projection_sqlite_store_ingests_provider_runtime_events_in_order() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        store
            .execute_orchestration_command(
                &OrchestrationCommand::ProjectCreate {
                    command_id: "cmd-project-runtime-batch".to_string(),
                    project_id: "project-runtime-batch".to_string(),
                    title: "Project Runtime Batch".to_string(),
                    workspace_root: "/repo/project-runtime-batch".to_string(),
                    default_model_selection: None,
                    created_at: "2026-03-04T17:00:00.000Z".to_string(),
                },
                "2026-03-04T17:00:00.000Z",
            )
            .unwrap();
        store
            .execute_orchestration_command(
                &OrchestrationCommand::ThreadCreate {
                    command_id: "cmd-thread-runtime-batch".to_string(),
                    thread_id: "thread-runtime-batch".to_string(),
                    project_id: "project-runtime-batch".to_string(),
                    title: "Thread Runtime Batch".to_string(),
                    model_selection: json!({ "instanceId": "codex", "model": "gpt-5.4" }),
                    runtime_mode: RuntimeMode::FullAccess,
                    interaction_mode: ProviderInteractionMode::Default,
                    branch: None,
                    worktree_path: Some("/repo/project-runtime-batch".to_string()),
                    created_at: "2026-03-04T17:00:01.000Z".to_string(),
                },
                "2026-03-04T17:00:01.000Z",
            )
            .unwrap();

        let context = ProviderRuntimeIngestionCommandPlanContext {
            thread_id: "thread-runtime-batch".to_string(),
            session_context: None,
            existing_proposed_plan_created_at: None,
            next_checkpoint_turn_count: None,
            assistant_completion_has_projected_message: None,
            assistant_completion_projected_text_is_empty: None,
            turn_completion_assistant_message_ids: Vec::new(),
            pause_assistant_message_ids: Vec::new(),
            pause_assistant_buffered_texts: Vec::new(),
            turn_completion_assistant_buffered_texts: Vec::new(),
            turn_completion_proposed_plan_markdown: None,
        };
        let events = store
            .ingest_provider_runtime_events(
                &[
                    (
                        context.clone(),
                        ProviderRuntimeEventInput {
                            event_type: "content.delta".to_string(),
                            event_id: "runtime-batch-delta".to_string(),
                            created_at: "2026-03-04T17:00:02.000Z".to_string(),
                            turn_id: Some("turn-batch".to_string()),
                            request_id: None,
                            item_id: Some("assistant-batch".to_string()),
                            payload: json!({
                                "streamKind": "assistant_text",
                                "delta": "queued",
                            }),
                            session_sequence: Some(1),
                        },
                    ),
                    (
                        ProviderRuntimeIngestionCommandPlanContext {
                            assistant_completion_has_projected_message: Some(true),
                            ..context
                        },
                        ProviderRuntimeEventInput {
                            event_type: "item.completed".to_string(),
                            event_id: "runtime-batch-complete".to_string(),
                            created_at: "2026-03-04T17:00:03.000Z".to_string(),
                            turn_id: Some("turn-batch".to_string()),
                            request_id: None,
                            item_id: Some("assistant-batch".to_string()),
                            payload: json!({
                                "itemType": "assistant_message",
                            }),
                            session_sequence: Some(2),
                        },
                    ),
                ],
                "2026-03-04T17:00:04.000Z",
            )
            .unwrap();
        assert_eq!(events.len(), 2);
        assert!(events[0].sequence < events[1].sequence);
        assert_eq!(
            events[0].command_id.as_deref(),
            Some("provider:runtime-batch-delta:assistant-delta")
        );
        assert_eq!(
            events[1].command_id.as_deref(),
            Some("provider:runtime-batch-complete:assistant-complete")
        );
        assert!(
            store
                .get_command_receipt("provider:runtime-batch-complete:assistant-complete")
                .unwrap()
                .is_some()
        );
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-batch")
            .unwrap()
            .unwrap();
        let assistant = detail
            .messages
            .iter()
            .find(|message| message.id == "assistant:assistant-batch")
            .unwrap();
        assert_eq!(assistant.text, "queued");
        assert!(!assistant.streaming);

        let mut queue = ProviderRuntimeIngestionQueue::new();
        queue.enqueue_domain_turn_start_requested("domain-batch-turn", "thread-runtime-batch");
        queue.enqueue_runtime(
            ProviderRuntimeIngestionCommandPlanContext {
                thread_id: "thread-runtime-batch".to_string(),
                session_context: None,
                existing_proposed_plan_created_at: None,
                next_checkpoint_turn_count: None,
                assistant_completion_has_projected_message: None,
                assistant_completion_projected_text_is_empty: None,
                turn_completion_assistant_message_ids: Vec::new(),
                pause_assistant_message_ids: Vec::new(),
                pause_assistant_buffered_texts: Vec::new(),
                turn_completion_assistant_buffered_texts: Vec::new(),
                turn_completion_proposed_plan_markdown: Some("- queue plan".to_string()),
            },
            ProviderRuntimeEventInput {
                event_type: "turn.completed".to_string(),
                event_id: "runtime-queue-turn-plan".to_string(),
                created_at: "2026-03-04T17:00:05.000Z".to_string(),
                turn_id: Some("turn-queue-plan".to_string()),
                request_id: None,
                item_id: None,
                payload: json!({ "state": "completed" }),
                session_sequence: Some(3),
            },
        );
        assert_eq!(queue.state().pending_count, 2);
        let events = store
            .drain_provider_runtime_ingestion_queue(&mut queue, "2026-03-04T17:00:06.000Z")
            .unwrap();
        assert_eq!(events.len(), 1);
        assert!(queue.state().is_idle);
        let detail = store
            .load_thread_detail("environment-local", "thread-runtime-batch")
            .unwrap()
            .unwrap();
        assert!(detail.proposed_plans.iter().any(|plan| plan.id
            == "plan:thread-runtime-batch:turn:turn-queue-plan"
            && plan.plan_markdown == "- queue plan"));
    }

    #[test]
    fn projection_sqlite_store_round_trips_shell_snapshot_rows() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        store
            .upsert_project(&project("project-second", "2026-03-04T12:00:02.000Z"))
            .unwrap();
        store
            .upsert_project(&project("project-first", "2026-03-04T12:00:01.000Z"))
            .unwrap();
        store
            .upsert_thread(&thread(
                "thread-first",
                "project-first",
                "2026-03-04T12:00:03.000Z",
            ))
            .unwrap();
        store
            .upsert_thread_session(&ProjectionThreadSessionRow {
                thread_id: "thread-first".to_string(),
                status: "running".to_string(),
                provider_name: "codex".to_string(),
                provider_instance_id: Some("codex".to_string()),
                active_turn_id: Some("turn-1".to_string()),
                last_error: None,
                updated_at: "2026-03-04T12:00:05.000Z".to_string(),
            })
            .unwrap();
        store
            .upsert_turn(&ProjectionLatestTurnRow {
                thread_id: "thread-first".to_string(),
                turn_id: "turn-1".to_string(),
                state: "completed".to_string(),
                requested_at: "2026-03-04T12:00:04.000Z".to_string(),
                started_at: Some("2026-03-04T12:00:05.000Z".to_string()),
                completed_at: Some("2026-03-04T12:00:06.000Z".to_string()),
                assistant_message_id: Some("msg-assistant".to_string()),
                source_proposed_plan_thread_id: Some("thread-plan".to_string()),
                source_proposed_plan_id: Some("plan-1".to_string()),
            })
            .unwrap();
        store
            .set_thread_latest_turn("thread-first", Some("turn-1"))
            .unwrap();

        let input = store
            .load_shell_snapshot_input("environment-local")
            .unwrap();
        assert_eq!(
            input
                .projects
                .iter()
                .map(|project| project.project_id.as_str())
                .collect::<Vec<_>>(),
            vec!["project-first", "project-second"]
        );
        assert_eq!(
            input.projects[0].scripts[0].icon,
            ProjectScriptIcon::Configure
        );
        assert_eq!(input.threads[0].runtime_mode, RuntimeMode::AutoAcceptEdits);
        assert_eq!(
            input.threads[0].interaction_mode,
            ProviderInteractionMode::Plan
        );

        let snapshot = build_projection_shell_snapshot(input);
        assert_eq!(snapshot.threads[0].status, ThreadStatus::Running);
        assert!(snapshot.threads[0].has_actionable_proposed_plan);

        let thread =
            get_thread_from_environment_state(&snapshot.environment_state, "thread-first").unwrap();
        assert_eq!(thread.latest_turn.unwrap().state, "completed");
        assert_eq!(
            thread.pending_source_proposed_plan.as_deref(),
            Some("plan-1")
        );
    }

    #[test]
    fn projection_sqlite_store_loads_thread_detail_rows() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        store
            .upsert_project(&project("project-first", "2026-03-04T12:00:01.000Z"))
            .unwrap();
        store
            .upsert_thread(&thread(
                "thread-first",
                "project-first",
                "2026-03-04T12:00:02.000Z",
            ))
            .unwrap();
        store
            .upsert_message(
                "thread-first",
                &ChatMessage {
                    id: "msg-user".to_string(),
                    role: MessageRole::User,
                    text: "inspect this".to_string(),
                    attachments: vec![ChatAttachment::Image(ChatImageAttachment {
                        id: "image-1".to_string(),
                        name: "screen.png".to_string(),
                        mime_type: "image/png".to_string(),
                        size_bytes: 42,
                        preview_url: None,
                    })],
                    turn_id: Some("turn-1".to_string()),
                    created_at: "2026-03-04T12:00:03.000Z".to_string(),
                    completed_at: None,
                    streaming: false,
                },
            )
            .unwrap();
        store
            .upsert_activity(
                "thread-first",
                &ThreadActivity {
                    id: "activity-1".to_string(),
                    kind: "tool.completed".to_string(),
                    summary: "Read file".to_string(),
                    tone: ActivityTone::Tool,
                    payload: ActivityPayload {
                        command: Some("rg Projection".to_string()),
                        questions: vec![UserInputQuestion {
                            id: "q1".to_string(),
                            header: "Choice".to_string(),
                            question: "Pick one".to_string(),
                            options: vec![UserInputQuestionOption {
                                label: "A".to_string(),
                                description: "First".to_string(),
                            }],
                            multi_select: false,
                        }],
                        ..ActivityPayload::default()
                    },
                    turn_id: Some("turn-1".to_string()),
                    sequence: Some(1),
                    created_at: "2026-03-04T12:00:04.000Z".to_string(),
                },
            )
            .unwrap();
        store
            .upsert_proposed_plan(
                "thread-first",
                &ProposedPlan {
                    id: "plan-1".to_string(),
                    turn_id: Some("turn-1".to_string()),
                    plan_markdown: "- Port detail rows".to_string(),
                    implemented_at: None,
                    implementation_thread_id: None,
                    created_at: "2026-03-04T12:00:05.000Z".to_string(),
                    updated_at: "2026-03-04T12:00:05.000Z".to_string(),
                },
            )
            .unwrap();
        store
            .upsert_checkpoint_summary(
                "thread-first",
                &TurnDiffSummary {
                    turn_id: "turn-1".to_string(),
                    completed_at: "2026-03-04T12:00:06.000Z".to_string(),
                    status: Some("completed".to_string()),
                    files: vec![TurnDiffFileChange {
                        path: "crates/r3_core/src/persistence.rs".to_string(),
                        kind: Some("modified".to_string()),
                        additions: Some(20),
                        deletions: Some(1),
                    }],
                    checkpoint_ref: Some("checkpoint-1".to_string()),
                    assistant_message_id: Some("msg-assistant".to_string()),
                    checkpoint_turn_count: Some(1),
                },
            )
            .unwrap();

        let thread = store
            .load_thread_detail("environment-local", "thread-first")
            .unwrap()
            .unwrap();

        assert_eq!(thread.messages[0].id, "msg-user");
        assert_eq!(thread.messages[0].attachments.len(), 1);
        assert_eq!(
            thread.activities[0].payload.command.as_deref(),
            Some("rg Projection")
        );
        assert_eq!(
            thread.activities[0].payload.questions[0].options[0].label,
            "A"
        );
        assert_eq!(thread.proposed_plans[0].id, "plan-1");
        assert_eq!(
            thread.turn_diff_summaries[0].files[0].path,
            "crates/r3_core/src/persistence.rs"
        );
    }

    #[test]
    fn projection_sqlite_store_persists_pending_approval_and_user_input_counts() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        store
            .upsert_project(&project("project-first", "2026-03-04T12:00:01.000Z"))
            .unwrap();
        store
            .upsert_thread(&thread(
                "thread-first",
                "project-first",
                "2026-03-04T12:00:02.000Z",
            ))
            .unwrap();

        store
            .upsert_pending_approval(&ProjectionPendingApprovalRow {
                request_id: "approval-1".to_string(),
                thread_id: "thread-first".to_string(),
                turn_id: Some("turn-1".to_string()),
                status: "pending".to_string(),
                decision: None,
                created_at: "2026-03-04T12:00:03.000Z".to_string(),
                resolved_at: None,
            })
            .unwrap();

        let thread_row = store.get_thread("thread-first").unwrap().unwrap();
        assert_eq!(thread_row.pending_approval_count, 1);
        assert_eq!(
            store
                .list_pending_approvals_by_thread("thread-first")
                .unwrap()[0]
                .request_id,
            "approval-1"
        );

        store
            .upsert_pending_approval(&ProjectionPendingApprovalRow {
                request_id: "approval-1".to_string(),
                thread_id: "thread-first".to_string(),
                turn_id: Some("turn-1".to_string()),
                status: "resolved".to_string(),
                decision: Some("accept".to_string()),
                created_at: "2026-03-04T12:00:03.000Z".to_string(),
                resolved_at: Some("2026-03-04T12:00:04.000Z".to_string()),
            })
            .unwrap();
        assert_eq!(
            store
                .get_pending_approval("approval-1")
                .unwrap()
                .unwrap()
                .decision
                .as_deref(),
            Some("accept")
        );
        assert_eq!(
            store
                .get_thread("thread-first")
                .unwrap()
                .unwrap()
                .pending_approval_count,
            0
        );

        store
            .upsert_activity(
                "thread-first",
                &ThreadActivity {
                    id: "activity-user-input".to_string(),
                    kind: "user-input.requested".to_string(),
                    summary: "Input requested".to_string(),
                    tone: ActivityTone::Info,
                    payload: ActivityPayload {
                        request_id: Some("input-1".to_string()),
                        questions: vec![UserInputQuestion {
                            id: "scope".to_string(),
                            header: "Scope".to_string(),
                            question: "What now?".to_string(),
                            options: Vec::new(),
                            multi_select: false,
                        }],
                        ..ActivityPayload::default()
                    },
                    turn_id: Some("turn-1".to_string()),
                    sequence: Some(1),
                    created_at: "2026-03-04T12:00:05.000Z".to_string(),
                },
            )
            .unwrap();
        assert_eq!(
            store
                .get_thread("thread-first")
                .unwrap()
                .unwrap()
                .pending_user_input_count,
            1
        );

        store
            .upsert_pending_approval(&ProjectionPendingApprovalRow {
                request_id: "approval-stale".to_string(),
                thread_id: "thread-first".to_string(),
                turn_id: Some("turn-1".to_string()),
                status: "pending".to_string(),
                decision: None,
                created_at: "2026-03-04T12:00:06.000Z".to_string(),
                resolved_at: None,
            })
            .unwrap();
        store
            .upsert_activity(
                "thread-first",
                &ThreadActivity {
                    id: "activity-stale-approval".to_string(),
                    kind: "provider.approval.respond.failed".to_string(),
                    summary: "Approval response failed".to_string(),
                    tone: ActivityTone::Error,
                    payload: ActivityPayload {
                        request_id: Some("approval-stale".to_string()),
                        detail: Some("unknown pending approval request".to_string()),
                        ..ActivityPayload::default()
                    },
                    turn_id: Some("turn-1".to_string()),
                    sequence: Some(2),
                    created_at: "2026-03-04T12:00:07.000Z".to_string(),
                },
            )
            .unwrap();
        store
            .project_pending_approval_from_activity(
                "thread-first",
                &store
                    .list_activities_by_thread("thread-first")
                    .unwrap()
                    .into_iter()
                    .find(|activity| activity.id == "activity-stale-approval")
                    .unwrap(),
                &OrchestrationEventRow {
                    sequence: 1,
                    stream_version: 0,
                    event_id: "event-stale".to_string(),
                    aggregate_kind: "thread".to_string(),
                    aggregate_id: "thread-first".to_string(),
                    event_type: "thread.activity-appended".to_string(),
                    occurred_at: "2026-03-04T12:00:07.000Z".to_string(),
                    command_id: None,
                    causation_event_id: None,
                    correlation_id: None,
                    actor_kind: "server".to_string(),
                    payload: json!({}),
                    metadata: json!({}),
                },
            )
            .unwrap();
        assert_eq!(
            store
                .get_pending_approval("approval-stale")
                .unwrap()
                .unwrap()
                .status,
            "resolved"
        );
    }

    #[test]
    fn projection_sqlite_store_preserves_deleted_rows_for_mapper_filtering() {
        let store = ProjectionSqliteStore::open_in_memory().unwrap();
        let mut deleted_thread = thread(
            "thread-deleted",
            "project-first",
            "2026-03-04T12:00:03.000Z",
        );
        deleted_thread.deleted_at = Some("2026-03-04T12:00:04.000Z".to_string());
        store
            .upsert_project(&project("project-first", "2026-03-04T12:00:01.000Z"))
            .unwrap();
        store.upsert_thread(&deleted_thread).unwrap();

        let input = store
            .load_shell_snapshot_input("environment-local")
            .unwrap();
        assert_eq!(
            input.threads[0].deleted_at.as_deref(),
            Some("2026-03-04T12:00:04.000Z")
        );

        let snapshot = build_projection_shell_snapshot(input);
        assert!(snapshot.threads.is_empty());
    }
}
