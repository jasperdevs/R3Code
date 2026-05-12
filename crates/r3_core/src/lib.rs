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
    pub pending_approvals: Vec<PendingApproval>,
    pub pending_user_inputs: Vec<PendingUserInput>,
    pub pending_user_input_draft_answers: BTreeMap<String, PendingUserInputDraftAnswer>,
    pub active_pending_user_input_question_index: usize,
    pub responding_request_ids: Vec<String>,
    pub terminal_state: ThreadTerminalState,
    pub terminal_launch_context: Option<ThreadTerminalLaunchContext>,
    pub terminal_event_entries: Vec<TerminalEventEntry>,
}

impl AppSnapshot {
    pub fn empty_reference_state() -> Self {
        Self {
            route: ChatRoute::Index,
            projects: Vec::new(),
            threads: Vec::new(),
            messages: Vec::new(),
            draft_sessions: Vec::new(),
            pending_approvals: Vec::new(),
            pending_user_inputs: Vec::new(),
            pending_user_input_draft_answers: BTreeMap::new(),
            active_pending_user_input_question_index: 0,
            responding_request_ids: Vec::new(),
            terminal_state: create_default_thread_terminal_state(),
            terminal_launch_context: None,
            terminal_event_entries: Vec::new(),
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
            pending_approvals: Vec::new(),
            pending_user_inputs: Vec::new(),
            pending_user_input_draft_answers: BTreeMap::new(),
            active_pending_user_input_question_index: 0,
            responding_request_ids: Vec::new(),
            terminal_state: create_default_thread_terminal_state(),
            terminal_launch_context: None,
            terminal_event_entries: Vec::new(),
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
            pending_approvals: Vec::new(),
            pending_user_inputs: Vec::new(),
            pending_user_input_draft_answers: BTreeMap::new(),
            active_pending_user_input_question_index: 0,
            responding_request_ids: Vec::new(),
            terminal_state: create_default_thread_terminal_state(),
            terminal_launch_context: None,
            terminal_event_entries: Vec::new(),
        }
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
