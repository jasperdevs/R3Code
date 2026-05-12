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
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSummary {
    pub title: String,
    pub project_name: String,
    pub status: ThreadStatus,
    pub latest_user_message_at: Option<String>,
    pub has_pending_approvals: bool,
    pub has_pending_user_input: bool,
    pub has_actionable_proposed_plan: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    Idle,
    Running,
    NeedsInput,
    Failed,
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
    pub messages: Vec<ChatMessage>,
    pub draft_sessions: Vec<DraftSessionState>,
}

impl AppSnapshot {
    pub fn empty_reference_state() -> Self {
        Self {
            route: ChatRoute::Index,
            projects: Vec::new(),
            threads: Vec::new(),
            messages: Vec::new(),
            draft_sessions: Vec::new(),
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
                name: "server".to_string(),
                path: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
            }],
            threads: Vec::new(),
            messages: Vec::new(),
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
        }
    }

    pub fn mock_reference_state() -> Self {
        Self {
            route: ChatRoute::Thread(ThreadRouteTarget::Server {
                thread_ref: ScopedThreadRef::new("local", "thread-r3code-ui-shell"),
            }),
            projects: vec![ProjectSummary {
                name: "r3code".to_string(),
                path: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
            }],
            threads: vec![
                ThreadSummary {
                    title: "Port R3Code UI shell".to_string(),
                    project_name: "r3code".to_string(),
                    status: ThreadStatus::Running,
                    latest_user_message_at: Some("2026-03-04T12:00:09.000Z".to_string()),
                    has_pending_approvals: false,
                    has_pending_user_input: false,
                    has_actionable_proposed_plan: false,
                },
                ThreadSummary {
                    title: "Capture visual references".to_string(),
                    project_name: "r3code".to_string(),
                    status: ThreadStatus::Idle,
                    latest_user_message_at: None,
                    has_pending_approvals: false,
                    has_pending_user_input: false,
                    has_actionable_proposed_plan: false,
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
            draft_sessions: Vec::new(),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
