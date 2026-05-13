use serde_json::{Value, json};

use crate::{
    ChatAttachment, ChatImageAttachment, ProjectScript, ProjectionProjectRow, ProjectionThreadRow,
    ProposedPlan, ProviderInteractionMode, RuntimeMode, TurnDiffFileChange,
    attachments::{attachment_relative_path, create_attachment_id, parse_base64_data_url},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrchestrationReadModel {
    pub snapshot_sequence: i64,
    pub projects: Vec<ProjectionProjectRow>,
    pub threads: Vec<ProjectionThreadRow>,
    pub proposed_plans: Vec<OrchestrationProposedPlanRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationProposedPlanRef {
    pub thread_id: String,
    pub plan: ProposedPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestrationRecoveryReason {
    Bootstrap,
    SequenceGap,
    Resubscribe,
    ReplayFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestrationRecoveryPhaseKind {
    Snapshot,
    Replay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrchestrationRecoveryPhase {
    pub kind: OrchestrationRecoveryPhaseKind,
    pub reason: OrchestrationRecoveryReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OrchestrationRecoveryState {
    pub latest_sequence: u64,
    pub highest_observed_sequence: u64,
    pub bootstrapped: bool,
    pub pending_replay: bool,
    pub in_flight: Option<OrchestrationRecoveryPhase>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayRecoveryCompletion {
    pub replay_made_progress: bool,
    pub should_replay: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayRetryTracker {
    pub attempts: u32,
    pub latest_sequence: u64,
    pub highest_observed_sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayRetryDecision {
    pub should_retry: bool,
    pub delay_ms: u64,
    pub tracker: Option<ReplayRetryTracker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainEventClassification {
    Ignore,
    Defer,
    Recover,
    Apply,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrchestrationRecoveryCoordinator {
    state: OrchestrationRecoveryState,
    replay_start_sequence: Option<u64>,
}

impl OrchestrationRecoveryCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> OrchestrationRecoveryState {
        self.state
    }

    fn observe_sequence(&mut self, sequence: u64) {
        self.state.highest_observed_sequence = self.state.highest_observed_sequence.max(sequence);
    }

    fn resolve_replay_need_after_recovery(&mut self) -> bool {
        let pending_replay_before_reset = self.state.pending_replay;
        let observed_ahead = self.state.highest_observed_sequence > self.state.latest_sequence;
        let should_replay = pending_replay_before_reset || observed_ahead;
        self.state.pending_replay = false;
        should_replay
    }

    pub fn classify_domain_event(&mut self, sequence: u64) -> DomainEventClassification {
        self.observe_sequence(sequence);
        if sequence <= self.state.latest_sequence {
            return DomainEventClassification::Ignore;
        }
        if !self.state.bootstrapped || self.state.in_flight.is_some() {
            self.state.pending_replay = true;
            return DomainEventClassification::Defer;
        }
        if sequence != self.state.latest_sequence + 1 {
            self.state.pending_replay = true;
            return DomainEventClassification::Recover;
        }
        DomainEventClassification::Apply
    }

    pub fn mark_event_batch_applied(&mut self, events: &[u64]) -> Vec<u64> {
        let mut next_events = events
            .iter()
            .copied()
            .filter(|sequence| *sequence > self.state.latest_sequence)
            .collect::<Vec<_>>();
        next_events.sort_unstable();
        if let Some(last) = next_events.last().copied() {
            self.state.latest_sequence = last;
            self.state.highest_observed_sequence = self
                .state
                .highest_observed_sequence
                .max(self.state.latest_sequence);
        }
        next_events
    }

    pub fn begin_snapshot_recovery(&mut self, reason: OrchestrationRecoveryReason) -> bool {
        if self.state.in_flight.is_some() {
            self.state.pending_replay = true;
            return false;
        }
        self.state.in_flight = Some(OrchestrationRecoveryPhase {
            kind: OrchestrationRecoveryPhaseKind::Snapshot,
            reason,
        });
        true
    }

    pub fn complete_snapshot_recovery(&mut self, snapshot_sequence: u64) -> bool {
        self.state.latest_sequence = self.state.latest_sequence.max(snapshot_sequence);
        self.state.highest_observed_sequence = self
            .state
            .highest_observed_sequence
            .max(self.state.latest_sequence);
        self.state.bootstrapped = true;
        self.state.in_flight = None;
        self.resolve_replay_need_after_recovery()
    }

    pub fn fail_snapshot_recovery(&mut self) {
        self.state.in_flight = None;
    }

    pub fn begin_replay_recovery(&mut self, reason: OrchestrationRecoveryReason) -> bool {
        if !self.state.bootstrapped
            || self.state.in_flight.is_some_and(|phase| {
                phase.kind == OrchestrationRecoveryPhaseKind::Snapshot
                    || phase.kind == OrchestrationRecoveryPhaseKind::Replay
            })
        {
            self.state.pending_replay = true;
            return false;
        }
        self.state.pending_replay = false;
        self.replay_start_sequence = Some(self.state.latest_sequence);
        self.state.in_flight = Some(OrchestrationRecoveryPhase {
            kind: OrchestrationRecoveryPhaseKind::Replay,
            reason,
        });
        true
    }

    pub fn complete_replay_recovery(&mut self) -> ReplayRecoveryCompletion {
        let replay_made_progress = self
            .replay_start_sequence
            .is_some_and(|start| self.state.latest_sequence > start);
        self.replay_start_sequence = None;
        self.state.in_flight = None;
        ReplayRecoveryCompletion {
            replay_made_progress,
            should_replay: self.resolve_replay_need_after_recovery(),
        }
    }

    pub fn fail_replay_recovery(&mut self) {
        self.replay_start_sequence = None;
        self.state.bootstrapped = false;
        self.state.in_flight = None;
    }
}

pub fn derive_replay_retry_decision(
    previous_tracker: Option<ReplayRetryTracker>,
    completion: ReplayRecoveryCompletion,
    recovery_state: OrchestrationRecoveryState,
    base_delay_ms: u64,
    max_no_progress_retries: u32,
) -> ReplayRetryDecision {
    if !completion.should_replay {
        return ReplayRetryDecision {
            should_retry: false,
            delay_ms: 0,
            tracker: None,
        };
    }
    if completion.replay_made_progress {
        return ReplayRetryDecision {
            should_retry: true,
            delay_ms: 0,
            tracker: None,
        };
    }

    let same_frontier = previous_tracker.is_some_and(|tracker| {
        tracker.latest_sequence == recovery_state.latest_sequence
            && tracker.highest_observed_sequence == recovery_state.highest_observed_sequence
    });
    let attempts = if same_frontier {
        previous_tracker
            .map(|tracker| tracker.attempts + 1)
            .unwrap_or(1)
    } else {
        1
    };

    if attempts > max_no_progress_retries {
        return ReplayRetryDecision {
            should_retry: false,
            delay_ms: 0,
            tracker: None,
        };
    }

    ReplayRetryDecision {
        should_retry: true,
        delay_ms: base_delay_ms * 2_u64.pow(attempts.saturating_sub(1)),
        tracker: Some(ReplayRetryTracker {
            attempts,
            latest_sequence: recovery_state.latest_sequence,
            highest_observed_sequence: recovery_state.highest_observed_sequence,
        }),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedOrchestrationEvent {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationCommandInvariantError {
    pub command_type: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrchestrationCommand {
    ProjectCreate {
        command_id: String,
        project_id: String,
        title: String,
        workspace_root: String,
        default_model_selection: Option<Value>,
        created_at: String,
    },
    ProjectMetaUpdate {
        command_id: String,
        project_id: String,
        title: Option<String>,
        workspace_root: Option<String>,
        default_model_selection: Option<Value>,
        scripts: Option<Vec<ProjectScript>>,
    },
    ProjectDelete {
        command_id: String,
        project_id: String,
        force: bool,
    },
    ThreadCreate {
        command_id: String,
        thread_id: String,
        project_id: String,
        title: String,
        model_selection: Value,
        runtime_mode: RuntimeMode,
        interaction_mode: ProviderInteractionMode,
        branch: Option<String>,
        worktree_path: Option<String>,
        created_at: String,
    },
    ThreadDelete {
        command_id: String,
        thread_id: String,
    },
    ThreadArchive {
        command_id: String,
        thread_id: String,
    },
    ThreadUnarchive {
        command_id: String,
        thread_id: String,
    },
    ThreadMetaUpdate {
        command_id: String,
        thread_id: String,
        title: Option<String>,
        model_selection: Option<Value>,
        branch: Option<String>,
        worktree_path: Option<String>,
    },
    ThreadRuntimeModeSet {
        command_id: String,
        thread_id: String,
        runtime_mode: RuntimeMode,
    },
    ThreadInteractionModeSet {
        command_id: String,
        thread_id: String,
        interaction_mode: ProviderInteractionMode,
    },
    ThreadTurnStart {
        command_id: String,
        thread_id: String,
        message_id: String,
        text: String,
        attachments: Vec<ChatAttachment>,
        model_selection: Option<Value>,
        title_seed: Option<String>,
        source_proposed_plan: Option<SourceProposedPlanRef>,
        created_at: String,
    },
    ThreadTurnInterrupt {
        command_id: String,
        thread_id: String,
        turn_id: Option<String>,
        created_at: String,
    },
    ThreadApprovalRespond {
        command_id: String,
        thread_id: String,
        request_id: String,
        decision: String,
        created_at: String,
    },
    ThreadUserInputRespond {
        command_id: String,
        thread_id: String,
        request_id: String,
        answers: Value,
        created_at: String,
    },
    ThreadCheckpointRevert {
        command_id: String,
        thread_id: String,
        turn_count: u32,
        created_at: String,
    },
    ThreadSessionStop {
        command_id: String,
        thread_id: String,
        created_at: String,
    },
    ThreadSessionSet {
        command_id: String,
        thread_id: String,
        session: Value,
        created_at: String,
    },
    ThreadMessageAssistantDelta {
        command_id: String,
        thread_id: String,
        message_id: String,
        delta: String,
        turn_id: Option<String>,
        created_at: String,
    },
    ThreadMessageAssistantComplete {
        command_id: String,
        thread_id: String,
        message_id: String,
        turn_id: Option<String>,
        created_at: String,
    },
    ThreadProposedPlanUpsert {
        command_id: String,
        thread_id: String,
        proposed_plan: ProposedPlan,
        created_at: String,
    },
    ThreadTurnDiffComplete {
        command_id: String,
        thread_id: String,
        turn_id: String,
        checkpoint_turn_count: u32,
        checkpoint_ref: Option<String>,
        status: String,
        files: Vec<TurnDiffFileChange>,
        assistant_message_id: Option<String>,
        completed_at: String,
        created_at: String,
    },
    ThreadRevertComplete {
        command_id: String,
        thread_id: String,
        turn_count: u32,
        created_at: String,
    },
    ThreadActivityAppend {
        command_id: String,
        thread_id: String,
        activity: Value,
        created_at: String,
    },
}

impl OrchestrationCommand {
    pub fn command_id(&self) -> &str {
        match self {
            Self::ProjectCreate { command_id, .. }
            | Self::ProjectMetaUpdate { command_id, .. }
            | Self::ProjectDelete { command_id, .. }
            | Self::ThreadCreate { command_id, .. }
            | Self::ThreadDelete { command_id, .. }
            | Self::ThreadArchive { command_id, .. }
            | Self::ThreadUnarchive { command_id, .. }
            | Self::ThreadMetaUpdate { command_id, .. }
            | Self::ThreadRuntimeModeSet { command_id, .. }
            | Self::ThreadInteractionModeSet { command_id, .. }
            | Self::ThreadTurnStart { command_id, .. }
            | Self::ThreadTurnInterrupt { command_id, .. }
            | Self::ThreadApprovalRespond { command_id, .. }
            | Self::ThreadUserInputRespond { command_id, .. }
            | Self::ThreadCheckpointRevert { command_id, .. }
            | Self::ThreadSessionStop { command_id, .. }
            | Self::ThreadSessionSet { command_id, .. }
            | Self::ThreadMessageAssistantDelta { command_id, .. }
            | Self::ThreadMessageAssistantComplete { command_id, .. }
            | Self::ThreadProposedPlanUpsert { command_id, .. }
            | Self::ThreadTurnDiffComplete { command_id, .. }
            | Self::ThreadRevertComplete { command_id, .. }
            | Self::ThreadActivityAppend { command_id, .. } => command_id,
        }
    }

    pub fn aggregate_kind_and_id(&self) -> (&'static str, &str) {
        match self {
            Self::ProjectCreate { project_id, .. }
            | Self::ProjectMetaUpdate { project_id, .. }
            | Self::ProjectDelete { project_id, .. } => ("project", project_id),
            Self::ThreadCreate { thread_id, .. }
            | Self::ThreadDelete { thread_id, .. }
            | Self::ThreadArchive { thread_id, .. }
            | Self::ThreadUnarchive { thread_id, .. }
            | Self::ThreadMetaUpdate { thread_id, .. }
            | Self::ThreadRuntimeModeSet { thread_id, .. }
            | Self::ThreadInteractionModeSet { thread_id, .. }
            | Self::ThreadTurnStart { thread_id, .. }
            | Self::ThreadTurnInterrupt { thread_id, .. }
            | Self::ThreadApprovalRespond { thread_id, .. }
            | Self::ThreadUserInputRespond { thread_id, .. }
            | Self::ThreadCheckpointRevert { thread_id, .. }
            | Self::ThreadSessionStop { thread_id, .. }
            | Self::ThreadSessionSet { thread_id, .. }
            | Self::ThreadMessageAssistantDelta { thread_id, .. }
            | Self::ThreadMessageAssistantComplete { thread_id, .. }
            | Self::ThreadProposedPlanUpsert { thread_id, .. }
            | Self::ThreadTurnDiffComplete { thread_id, .. }
            | Self::ThreadRevertComplete { thread_id, .. }
            | Self::ThreadActivityAppend { thread_id, .. } => ("thread", thread_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProposedPlanRef {
    pub thread_id: String,
    pub plan_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderCommandIntent {
    RuntimeModeSet {
        thread_id: String,
        runtime_mode: RuntimeMode,
    },
    TurnStart {
        thread_id: String,
        message_id: String,
        runtime_mode: RuntimeMode,
        interaction_mode: ProviderInteractionMode,
    },
    TurnInterrupt {
        thread_id: String,
        turn_id: Option<String>,
    },
    ApprovalRespond {
        thread_id: String,
        request_id: String,
        decision: String,
    },
    UserInputRespond {
        thread_id: String,
        request_id: String,
        answers: Value,
    },
    SessionStop {
        thread_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadDeletionCleanupAction {
    StopProviderSession,
    CloseThreadTerminalsAndDeleteHistory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSessionStatus {
    Connecting,
    Ready,
    Running,
    Error,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSessionModelSwitch {
    InSession,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAdapterCapabilities {
    pub session_model_switch: ProviderSessionModelSwitch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInstanceRoutingInfo {
    pub instance_id: String,
    pub driver_kind: String,
    pub enabled: bool,
    pub continuation_key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderSessionStartInput {
    pub thread_id: String,
    pub provider: Option<String>,
    pub provider_instance_id: String,
    pub cwd: Option<String>,
    pub model_selection: Option<Value>,
    pub resume_cursor: Option<Value>,
    pub approval_policy: Option<String>,
    pub sandbox_mode: Option<String>,
    pub runtime_mode: RuntimeMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderSession {
    pub provider: String,
    pub provider_instance_id: Option<String>,
    pub status: ProviderSessionStatus,
    pub runtime_mode: RuntimeMode,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub thread_id: String,
    pub resume_cursor: Option<Value>,
    pub active_turn_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderSendTurnInput {
    pub thread_id: String,
    pub input: Option<String>,
    pub attachments: Vec<ChatAttachment>,
    pub model_selection: Option<Value>,
    pub interaction_mode: ProviderInteractionMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderTurnStartResult {
    pub thread_id: String,
    pub turn_id: String,
    pub resume_cursor: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInterruptTurnInput {
    pub thread_id: String,
    pub turn_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRespondToRequestInput {
    pub thread_id: String,
    pub request_id: String,
    pub decision: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderRespondToUserInputInput {
    pub thread_id: String,
    pub request_id: String,
    pub answers: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderStopSessionInput {
    pub thread_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRollbackConversationInput {
    pub thread_id: String,
    pub num_turns: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderRuntimeBinding {
    pub thread_id: String,
    pub provider: String,
    pub provider_instance_id: Option<String>,
    pub adapter_key: Option<String>,
    pub status: Option<String>,
    pub resume_cursor: Option<Value>,
    pub runtime_payload: Option<Value>,
    pub runtime_mode: Option<RuntimeMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderServiceResolvedValueSource {
    Request,
    Persisted,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderServiceStartSessionPlan {
    pub adapter_input: ProviderSessionStartInput,
    pub resume_cursor_source: ProviderServiceResolvedValueSource,
    pub cwd_source: ProviderServiceResolvedValueSource,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderServiceRuntimePayloadExtra {
    pub model_selection: Option<Value>,
    pub last_runtime_event: Option<String>,
    pub last_runtime_event_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderServiceAdapterSessionProbe {
    pub provider: String,
    pub provider_instance_id: String,
    pub has_session: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderServiceAdapterRegistryEntry {
    pub instance_info: ProviderInstanceRoutingInfo,
    pub capabilities: ProviderAdapterCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderServiceStopSessionCall {
    pub provider: String,
    pub provider_instance_id: String,
    pub input: ProviderStopSessionInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderServiceStopAllCall {
    pub provider: String,
    pub provider_instance_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderServiceAdapterCall {
    StartSession {
        provider: String,
        provider_instance_id: String,
        input: ProviderSessionStartInput,
    },
    SendTurn {
        provider: String,
        provider_instance_id: String,
        input: ProviderSendTurnInput,
    },
    InterruptTurn {
        provider: String,
        provider_instance_id: String,
        input: ProviderInterruptTurnInput,
    },
    RespondToRequest {
        provider: String,
        provider_instance_id: String,
        input: ProviderRespondToRequestInput,
    },
    RespondToUserInput {
        provider: String,
        provider_instance_id: String,
        input: ProviderRespondToUserInputInput,
    },
    StopSession {
        provider: String,
        provider_instance_id: String,
        input: ProviderStopSessionInput,
    },
    RollbackConversation {
        provider: String,
        provider_instance_id: String,
        input: ProviderRollbackConversationInput,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderServiceRecoveryPlan {
    AdoptExisting {
        session: ProviderSession,
    },
    Resume {
        adapter_input: ProviderSessionStartInput,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderServiceRoutableSessionPlan {
    Active {
        provider: String,
        instance_id: String,
        thread_id: String,
    },
    Inactive {
        provider: String,
        instance_id: String,
        thread_id: String,
    },
    Recover(ProviderServiceRecoveryPlan),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderServiceRollbackConversationPlan {
    Noop,
    Route {
        route: ProviderServiceRoutableSessionPlan,
        num_turns: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderServiceStopAllPlan {
    pub active_session_bindings: Vec<ProviderRuntimeBinding>,
    pub stop_all_calls: Vec<ProviderServiceStopAllCall>,
    pub stopped_bindings: Vec<ProviderRuntimeBinding>,
    pub session_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderServiceRuntimeEventEnvelope {
    pub provider: String,
    pub provider_instance_id: Option<String>,
    pub thread_id: String,
    pub event: ProviderRuntimeEventInput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderServiceRuntimeEventFanoutPlan {
    pub canonical_events: Vec<ProviderServiceRuntimeEventEnvelope>,
    pub log_thread_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderServiceStartSessionExecutionPlan {
    pub start_call: ProviderServiceAdapterCall,
    pub resume_cursor_source: ProviderServiceResolvedValueSource,
    pub cwd_source: ProviderServiceResolvedValueSource,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderServiceStartSessionCompletionPlan {
    pub binding: ProviderRuntimeBinding,
    pub stop_stale_calls: Vec<ProviderServiceStopSessionCall>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderServicePlanError {
    MissingProviderInstanceId {
        operation: String,
        provider: Option<String>,
    },
    ProviderInstanceMismatch {
        requested_instance_id: String,
        resolved_instance_id: String,
    },
    ProviderDriverMismatch {
        instance_id: String,
        instance_driver: String,
        requested_provider: String,
    },
    ProviderInstanceDisabled {
        instance_id: String,
    },
    EmptySendTurnInput {
        operation: String,
    },
    CannotRouteThreadWithoutBinding {
        operation: String,
        thread_id: String,
    },
    CannotRecoverThreadWithoutResumeState {
        operation: String,
        thread_id: String,
    },
    ListSessionsProviderMismatch {
        thread_id: String,
        session_provider: String,
        binding_provider: String,
    },
    ListSessionsInstanceMismatch {
        thread_id: String,
        session_instance_id: Option<String>,
        binding_instance_id: String,
    },
    ProviderInstanceNotFound {
        operation: String,
        instance_id: String,
    },
    RuntimeEventProviderMismatch {
        source_provider: String,
        event_provider: String,
        provider_instance_id: String,
    },
    RuntimeEventInstanceMismatch {
        source_instance_id: String,
        event_instance_id: String,
    },
    StartSessionAdapterProviderMismatch {
        expected_provider: String,
        actual_provider: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderServiceRequest {
    EnsureSessionForRuntimeMode {
        thread_id: String,
        runtime_mode: RuntimeMode,
    },
    BuildAndSendTurn {
        thread_id: String,
        message_id: String,
        runtime_mode: RuntimeMode,
        interaction_mode: ProviderInteractionMode,
    },
    InterruptTurn(ProviderInterruptTurnInput),
    RespondToRequest(ProviderRespondToRequestInput),
    RespondToUserInput(ProviderRespondToUserInputInput),
    StopSession(ProviderStopSessionInput),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreadDeletionCleanupRequest {
    StopProviderSession(ProviderStopSessionInput),
    CloseThreadTerminalsAndDeleteHistory { thread_id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderRuntimeEventInput {
    pub event_type: String,
    pub event_id: String,
    pub created_at: String,
    pub turn_id: Option<String>,
    pub request_id: Option<String>,
    pub item_id: Option<String>,
    pub payload: Value,
    pub session_sequence: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderRuntimeActivity {
    pub id: String,
    pub created_at: String,
    pub tone: String,
    pub kind: String,
    pub summary: String,
    pub payload: Value,
    pub turn_id: Option<String>,
    pub sequence: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRuntimeBufferedAssistantText {
    pub message_id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRuntimeSessionContext {
    pub thread_id: String,
    pub provider: String,
    pub provider_instance_id: Option<String>,
    pub runtime_mode: RuntimeMode,
    pub active_turn_id: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderRuntimeIngestionCommandPlanContext {
    pub thread_id: String,
    pub session_context: Option<ProviderRuntimeSessionContext>,
    pub existing_proposed_plan_created_at: Option<String>,
    pub next_checkpoint_turn_count: Option<u32>,
    pub assistant_completion_has_projected_message: Option<bool>,
    pub assistant_completion_projected_text_is_empty: Option<bool>,
    pub turn_completion_assistant_message_ids: Vec<String>,
    pub pause_assistant_message_ids: Vec<String>,
    pub pause_assistant_buffered_texts: Vec<ProviderRuntimeBufferedAssistantText>,
    pub turn_completion_assistant_buffered_texts: Vec<ProviderRuntimeBufferedAssistantText>,
    pub turn_completion_proposed_plan_markdown: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderRuntimeIngestionInput {
    Runtime {
        context: ProviderRuntimeIngestionCommandPlanContext,
        event: ProviderRuntimeEventInput,
    },
    DomainTurnStartRequested {
        event_id: String,
        thread_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProviderRuntimeIngestionDrain {
    pub runtime_events: Vec<(
        ProviderRuntimeIngestionCommandPlanContext,
        ProviderRuntimeEventInput,
    )>,
    pub drained_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProviderRuntimeIngestionQueueState {
    pub pending_count: usize,
    pub is_idle: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProviderRuntimeIngestionQueue {
    pending: Vec<ProviderRuntimeIngestionInput>,
}

impl ProviderRuntimeIngestionQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue_runtime(
        &mut self,
        context: ProviderRuntimeIngestionCommandPlanContext,
        event: ProviderRuntimeEventInput,
    ) {
        self.pending
            .push(ProviderRuntimeIngestionInput::Runtime { context, event });
    }

    pub fn enqueue_domain_turn_start_requested(
        &mut self,
        event_id: impl Into<String>,
        thread_id: impl Into<String>,
    ) {
        self.pending
            .push(ProviderRuntimeIngestionInput::DomainTurnStartRequested {
                event_id: event_id.into(),
                thread_id: thread_id.into(),
            });
    }

    pub fn state(&self) -> ProviderRuntimeIngestionQueueState {
        ProviderRuntimeIngestionQueueState {
            pending_count: self.pending.len(),
            is_idle: self.pending.is_empty(),
        }
    }

    pub fn drain(&mut self) -> ProviderRuntimeIngestionDrain {
        let drained_count = self.pending.len();
        let runtime_events = self
            .pending
            .drain(..)
            .filter_map(|input| match input {
                ProviderRuntimeIngestionInput::Runtime { context, event } => Some((context, event)),
                ProviderRuntimeIngestionInput::DomainTurnStartRequested { .. } => None,
            })
            .collect();
        ProviderRuntimeIngestionDrain {
            runtime_events,
            drained_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestrationReactorComponent {
    ProviderRuntimeIngestion,
    ProviderCommandReactor,
    CheckpointReactor,
    ThreadDeletionReactor,
}

pub fn orchestration_reactor_start_order() -> [OrchestrationReactorComponent; 4] {
    [
        OrchestrationReactorComponent::ProviderRuntimeIngestion,
        OrchestrationReactorComponent::ProviderCommandReactor,
        OrchestrationReactorComponent::CheckpointReactor,
        OrchestrationReactorComponent::ThreadDeletionReactor,
    ]
}

pub fn normalize_runtime_turn_state(value: Option<&str>) -> &'static str {
    match value {
        Some("failed") => "failed",
        Some("interrupted") => "interrupted",
        Some("cancelled") => "cancelled",
        Some("completed") => "completed",
        _ => "completed",
    }
}

pub fn orchestration_session_status_from_runtime_state(value: &str) -> Option<&'static str> {
    match value {
        "starting" => Some("starting"),
        "running" | "waiting" => Some("running"),
        "ready" => Some("ready"),
        "interrupted" => Some("interrupted"),
        "stopped" => Some("stopped"),
        "error" => Some("error"),
        _ => None,
    }
}

pub fn request_kind_from_canonical_request_type(value: Option<&str>) -> Option<&'static str> {
    match value {
        Some("command_execution_approval") | Some("exec_command_approval") => Some("command"),
        Some("file_read_approval") => Some("file-read"),
        Some("file_change_approval") | Some("apply_patch_approval") => Some("file-change"),
        _ => None,
    }
}

pub fn truncate_runtime_detail(value: &str, limit: usize) -> String {
    if value.len() > limit {
        format!("{}...", &value[..limit.saturating_sub(3)])
    } else {
        value.to_string()
    }
}

pub fn normalize_proposed_plan_markdown(plan_markdown: Option<&str>) -> Option<String> {
    plan_markdown
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn has_renderable_assistant_text(text: Option<&str>) -> bool {
    text.map(str::trim)
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

pub fn proposed_plan_id_for_turn(thread_id: &str, turn_id: &str) -> String {
    format!("plan:{thread_id}:turn:{turn_id}")
}

pub fn proposed_plan_id_from_runtime_event(
    thread_id: &str,
    turn_id: Option<&str>,
    item_id: Option<&str>,
    event_id: &str,
) -> String {
    if let Some(turn_id) = turn_id {
        return proposed_plan_id_for_turn(thread_id, turn_id);
    }
    if let Some(item_id) = item_id {
        return format!("plan:{thread_id}:item:{item_id}");
    }
    format!("plan:{thread_id}:event:{event_id}")
}

pub fn assistant_segment_base_key_from_runtime_event(
    item_id: Option<&str>,
    turn_id: Option<&str>,
    event_id: &str,
) -> String {
    item_id.or(turn_id).unwrap_or(event_id).to_string()
}

pub fn assistant_segment_message_id(base_key: &str, segment_index: usize) -> String {
    if segment_index == 0 {
        format!("assistant:{base_key}")
    } else {
        format!("assistant:{base_key}:segment:{segment_index}")
    }
}

pub fn is_tool_lifecycle_item_type(value: &str) -> bool {
    matches!(
        value,
        "command_execution"
            | "file_change"
            | "mcp_tool_call"
            | "dynamic_tool_call"
            | "collab_agent_tool_call"
            | "web_search"
            | "image_view"
    )
}

pub fn provider_runtime_event_to_activities(
    event: &ProviderRuntimeEventInput,
) -> Vec<ProviderRuntimeActivity> {
    match event.event_type.as_str() {
        "request.opened" => {
            let request_type = event.payload.get("requestType").and_then(Value::as_str);
            if request_type == Some("tool_user_input") {
                return Vec::new();
            }
            let request_kind = request_kind_from_canonical_request_type(request_type);
            let mut payload = json!({
                "requestId": event.request_id,
                "requestType": request_type,
            });
            insert_optional(
                &mut payload,
                "requestKind",
                request_kind.map(|value| json!(value)),
            );
            insert_optional(
                &mut payload,
                "detail",
                event
                    .payload
                    .get("detail")
                    .and_then(Value::as_str)
                    .map(|detail| json!(truncate_runtime_detail(detail, 180))),
            );
            vec![runtime_activity(
                event,
                "approval",
                "approval.requested",
                match request_kind {
                    Some("command") => "Command approval requested",
                    Some("file-read") => "File-read approval requested",
                    Some("file-change") => "File-change approval requested",
                    _ => "Approval requested",
                },
                payload,
            )]
        }
        "request.resolved" => {
            let request_type = event.payload.get("requestType").and_then(Value::as_str);
            if request_type == Some("tool_user_input") {
                return Vec::new();
            }
            let request_kind = request_kind_from_canonical_request_type(request_type);
            let mut payload = json!({
                "requestId": event.request_id,
                "requestType": request_type,
            });
            insert_optional(
                &mut payload,
                "requestKind",
                request_kind.map(|value| json!(value)),
            );
            insert_optional(
                &mut payload,
                "decision",
                event
                    .payload
                    .get("decision")
                    .and_then(Value::as_str)
                    .map(|value| json!(value)),
            );
            vec![runtime_activity(
                event,
                "approval",
                "approval.resolved",
                "Approval resolved",
                payload,
            )]
        }
        "runtime.error" => vec![runtime_activity(
            event,
            "error",
            "runtime.error",
            "Runtime error",
            json!({
                "message": truncate_runtime_detail(
                    event.payload.get("message").and_then(Value::as_str).unwrap_or(""),
                    180,
                ),
            }),
        )],
        "runtime.warning" => {
            let mut payload = json!({
                "message": truncate_runtime_detail(
                    event.payload.get("message").and_then(Value::as_str).unwrap_or(""),
                    180,
                ),
            });
            insert_optional(&mut payload, "detail", event.payload.get("detail").cloned());
            vec![runtime_activity(
                event,
                "info",
                "runtime.warning",
                "Runtime warning",
                payload,
            )]
        }
        "turn.plan.updated" => {
            let mut payload = json!({
                "plan": event.payload.get("plan").cloned().unwrap_or(Value::Null),
            });
            insert_optional(
                &mut payload,
                "explanation",
                event.payload.get("explanation").cloned(),
            );
            vec![runtime_activity(
                event,
                "info",
                "turn.plan.updated",
                "Plan updated",
                payload,
            )]
        }
        "user-input.requested" => vec![runtime_activity(
            event,
            "info",
            "user-input.requested",
            "User input requested",
            json!({
                "requestId": event.request_id,
                "questions": event.payload.get("questions").cloned().unwrap_or_else(|| json!([])),
            }),
        )],
        "user-input.resolved" => vec![runtime_activity(
            event,
            "info",
            "user-input.resolved",
            "User input submitted",
            json!({
                "requestId": event.request_id,
                "answers": event.payload.get("answers").cloned().unwrap_or(Value::Null),
            }),
        )],
        "task.started" => {
            let task_type = event.payload.get("taskType").and_then(Value::as_str);
            let mut payload = json!({
                "taskId": event.payload.get("taskId").cloned().unwrap_or(Value::Null),
            });
            insert_optional(
                &mut payload,
                "taskType",
                task_type.map(|value| json!(value)),
            );
            insert_optional(
                &mut payload,
                "detail",
                event
                    .payload
                    .get("description")
                    .and_then(Value::as_str)
                    .map(|detail| json!(truncate_runtime_detail(detail, 180))),
            );
            let summary = match task_type {
                Some("plan") => "Plan task started".to_string(),
                Some(value) => format!("{value} task started"),
                None => "Task started".to_string(),
            };
            vec![runtime_activity(
                event,
                "info",
                "task.started",
                &summary,
                payload,
            )]
        }
        "task.progress" => {
            let detail = event
                .payload
                .get("summary")
                .or_else(|| event.payload.get("description"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let mut payload = json!({
                "taskId": event.payload.get("taskId").cloned().unwrap_or(Value::Null),
                "detail": truncate_runtime_detail(detail, 180),
            });
            insert_optional(
                &mut payload,
                "summary",
                event
                    .payload
                    .get("summary")
                    .and_then(Value::as_str)
                    .map(|summary| json!(truncate_runtime_detail(summary, 180))),
            );
            insert_optional(
                &mut payload,
                "lastToolName",
                event.payload.get("lastToolName").cloned(),
            );
            insert_optional(&mut payload, "usage", event.payload.get("usage").cloned());
            vec![runtime_activity(
                event,
                "info",
                "task.progress",
                "Reasoning update",
                payload,
            )]
        }
        "task.completed" => {
            let status = event.payload.get("status").and_then(Value::as_str);
            let mut payload = json!({
                "taskId": event.payload.get("taskId").cloned().unwrap_or(Value::Null),
                "status": status,
            });
            insert_optional(
                &mut payload,
                "detail",
                event
                    .payload
                    .get("summary")
                    .and_then(Value::as_str)
                    .map(|summary| json!(truncate_runtime_detail(summary, 180))),
            );
            insert_optional(&mut payload, "usage", event.payload.get("usage").cloned());
            vec![runtime_activity(
                event,
                if status == Some("failed") {
                    "error"
                } else {
                    "info"
                },
                "task.completed",
                match status {
                    Some("failed") => "Task failed",
                    Some("stopped") => "Task stopped",
                    _ => "Task completed",
                },
                payload,
            )]
        }
        "thread.state.changed" => {
            if event.payload.get("state").and_then(Value::as_str) != Some("compacted") {
                return Vec::new();
            }
            let mut payload = json!({
                "state": "compacted",
            });
            insert_optional(&mut payload, "detail", event.payload.get("detail").cloned());
            vec![runtime_activity(
                event,
                "info",
                "context-compaction",
                "Context compacted",
                payload,
            )]
        }
        "thread.token-usage.updated" => {
            let used_tokens = event
                .payload
                .pointer("/usage/usedTokens")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            if used_tokens <= 0 {
                return Vec::new();
            }
            vec![runtime_activity(
                event,
                "info",
                "context-window.updated",
                "Context window updated",
                event.payload.get("usage").cloned().unwrap_or(Value::Null),
            )]
        }
        "item.started" | "item.updated" | "item.completed" => {
            let item_type = event.payload.get("itemType").and_then(Value::as_str);
            if !item_type.map(is_tool_lifecycle_item_type).unwrap_or(false) {
                return Vec::new();
            }
            let title = event.payload.get("title").and_then(Value::as_str);
            let mut payload = json!({
                "itemType": item_type,
            });
            insert_optional(&mut payload, "status", event.payload.get("status").cloned());
            insert_optional(
                &mut payload,
                "detail",
                event
                    .payload
                    .get("detail")
                    .and_then(Value::as_str)
                    .map(|detail| json!(truncate_runtime_detail(detail, 180))),
            );
            insert_optional(&mut payload, "data", event.payload.get("data").cloned());
            let (kind, summary) = match event.event_type.as_str() {
                "item.started" => (
                    "tool.started",
                    format!("{} started", title.unwrap_or("Tool")),
                ),
                "item.updated" => ("tool.updated", title.unwrap_or("Tool updated").to_string()),
                _ => ("tool.completed", title.unwrap_or("Tool").to_string()),
            };
            vec![runtime_activity(event, "tool", kind, &summary, payload)]
        }
        _ => Vec::new(),
    }
}

pub fn provider_runtime_activity_to_json(activity: &ProviderRuntimeActivity) -> Value {
    let mut value = json!({
        "id": activity.id,
        "createdAt": activity.created_at,
        "tone": activity.tone,
        "kind": activity.kind,
        "summary": activity.summary,
        "payload": activity.payload,
        "turnId": activity.turn_id,
    });
    insert_optional(
        &mut value,
        "sequence",
        activity.sequence.map(|sequence| json!(sequence)),
    );
    value
}

pub fn provider_runtime_activity_to_thread_activity_append_command(
    thread_id: &str,
    command_id: &str,
    activity: &ProviderRuntimeActivity,
) -> OrchestrationCommand {
    OrchestrationCommand::ThreadActivityAppend {
        command_id: command_id.to_string(),
        thread_id: thread_id.to_string(),
        activity: provider_runtime_activity_to_json(activity),
        created_at: activity.created_at.clone(),
    }
}

pub fn provider_runtime_command_id(event: &ProviderRuntimeEventInput, tag: &str) -> String {
    format!("provider:{}:{tag}", event.event_id)
}

pub fn provider_runtime_event_to_orchestration_commands(
    context: &ProviderRuntimeIngestionCommandPlanContext,
    event: &ProviderRuntimeEventInput,
) -> Vec<OrchestrationCommand> {
    let mut commands = Vec::new();
    if let Some(session_context) = context.session_context.as_ref() {
        if let Some(command) = provider_runtime_lifecycle_session_command(
            session_context,
            &provider_runtime_command_id(event, "thread-session-set"),
            event,
        ) {
            commands.push(command);
        }
    }
    if let Some(command) = provider_runtime_assistant_delta_command(
        &context.thread_id,
        &provider_runtime_command_id(event, "assistant-delta"),
        event,
    ) {
        commands.push(command);
    }
    let assistant_fallback_delta = provider_runtime_assistant_completion_fallback_delta_command(
        &context.thread_id,
        &provider_runtime_command_id(event, "assistant-delta-finalize"),
        event,
        context
            .assistant_completion_projected_text_is_empty
            .unwrap_or(false),
    );
    if let Some(command) = assistant_fallback_delta.as_ref() {
        commands.push(command.clone());
    }
    if let Some(command) = provider_runtime_assistant_complete_command(
        &context.thread_id,
        &provider_runtime_command_id(event, "assistant-complete"),
        event,
    ) {
        if context
            .assistant_completion_has_projected_message
            .unwrap_or(true)
            || assistant_fallback_delta.is_some()
        {
            commands.push(command);
        }
    }
    if matches!(
        event.event_type.as_str(),
        "request.opened" | "user-input.requested"
    ) {
        let delta_command_tag = if event.event_type == "request.opened" {
            "assistant-delta-flush-on-request-opened"
        } else {
            "assistant-delta-flush-on-user-input-requested"
        };
        for buffered_text in &context.pause_assistant_buffered_texts {
            if has_renderable_assistant_text(Some(&buffered_text.text)) {
                commands.push(OrchestrationCommand::ThreadMessageAssistantDelta {
                    command_id: provider_runtime_command_id(event, delta_command_tag),
                    thread_id: context.thread_id.clone(),
                    message_id: buffered_text.message_id.clone(),
                    delta: buffered_text.text.clone(),
                    turn_id: event.turn_id.clone(),
                    created_at: event.created_at.clone(),
                });
            }
        }
        let complete_command_tag = if event.event_type == "request.opened" {
            "assistant-complete-on-request-opened"
        } else {
            "assistant-complete-on-user-input-requested"
        };
        for message_id in &context.pause_assistant_message_ids {
            commands.push(OrchestrationCommand::ThreadMessageAssistantComplete {
                command_id: provider_runtime_command_id(event, complete_command_tag),
                thread_id: context.thread_id.clone(),
                message_id: message_id.clone(),
                turn_id: event.turn_id.clone(),
                created_at: event.created_at.clone(),
            });
        }
    }
    if event.event_type == "turn.completed" {
        for buffered_text in &context.turn_completion_assistant_buffered_texts {
            if has_renderable_assistant_text(Some(&buffered_text.text)) {
                commands.push(OrchestrationCommand::ThreadMessageAssistantDelta {
                    command_id: provider_runtime_command_id(
                        event,
                        "assistant-delta-finalize-fallback",
                    ),
                    thread_id: context.thread_id.clone(),
                    message_id: buffered_text.message_id.clone(),
                    delta: buffered_text.text.clone(),
                    turn_id: event.turn_id.clone(),
                    created_at: event.created_at.clone(),
                });
            }
        }
        for message_id in &context.turn_completion_assistant_message_ids {
            commands.push(OrchestrationCommand::ThreadMessageAssistantComplete {
                command_id: provider_runtime_command_id(event, "assistant-complete-finalize"),
                thread_id: context.thread_id.clone(),
                message_id: message_id.clone(),
                turn_id: event.turn_id.clone(),
                created_at: event.created_at.clone(),
            });
        }
    }
    if let Some(command) = provider_runtime_proposed_plan_complete_command(
        &context.thread_id,
        &provider_runtime_command_id(event, "proposed-plan-upsert"),
        event,
        context.existing_proposed_plan_created_at.as_deref(),
    ) {
        commands.push(command);
    }
    if let Some(command) = provider_runtime_turn_completion_proposed_plan_command(
        &context.thread_id,
        &provider_runtime_command_id(event, "proposed-plan-upsert"),
        event,
        context.turn_completion_proposed_plan_markdown.as_deref(),
        context.existing_proposed_plan_created_at.as_deref(),
    ) {
        commands.push(command);
    }
    if let Some(command) = provider_runtime_thread_meta_update_command(
        &context.thread_id,
        &provider_runtime_command_id(event, "thread-meta-update"),
        event,
    ) {
        commands.push(command);
    }
    if let Some(checkpoint_turn_count) = context.next_checkpoint_turn_count {
        if let Some(command) = provider_runtime_turn_diff_complete_command(
            &context.thread_id,
            &provider_runtime_command_id(event, "thread-turn-diff-complete"),
            event,
            checkpoint_turn_count,
        ) {
            commands.push(command);
        }
    }
    for activity in provider_runtime_event_to_activities(event) {
        commands.push(provider_runtime_activity_to_thread_activity_append_command(
            &context.thread_id,
            &provider_runtime_command_id(event, "thread-activity-append"),
            &activity,
        ));
    }
    commands
}

pub fn provider_runtime_lifecycle_session_command(
    context: &ProviderRuntimeSessionContext,
    command_id: &str,
    event: &ProviderRuntimeEventInput,
) -> Option<OrchestrationCommand> {
    if event.event_type == "runtime.error"
        && context.active_turn_id.is_some()
        && event.turn_id.is_some()
        && context.active_turn_id != event.turn_id
    {
        return None;
    }
    let next_active_turn_id = match event.event_type.as_str() {
        "turn.started" => event.turn_id.clone(),
        "turn.completed" | "session.exited" | "runtime.error" => None,
        _ => context.active_turn_id.clone(),
    };
    let status = match event.event_type.as_str() {
        "session.state.changed" => {
            orchestration_session_status_from_runtime_state(event.payload.get("state")?.as_str()?)?
        }
        "turn.started" => "running",
        "session.exited" => "stopped",
        "turn.completed" => {
            if normalize_runtime_turn_state(event.payload.get("state").and_then(Value::as_str))
                == "failed"
            {
                "error"
            } else {
                "ready"
            }
        }
        "runtime.error" => "error",
        "session.started" | "thread.started" => {
            if context.active_turn_id.is_some() {
                "running"
            } else {
                "ready"
            }
        }
        _ => return None,
    };
    let last_error = if event.event_type == "session.state.changed"
        && event.payload.get("state").and_then(Value::as_str) == Some("error")
    {
        Some(
            event
                .payload
                .get("reason")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| context.last_error.clone())
                .unwrap_or_else(|| "Provider session error".to_string()),
        )
    } else if event.event_type == "turn.completed"
        && normalize_runtime_turn_state(event.payload.get("state").and_then(Value::as_str))
            == "failed"
    {
        Some(
            event
                .payload
                .get("errorMessage")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| context.last_error.clone())
                .unwrap_or_else(|| "Turn failed".to_string()),
        )
    } else if event.event_type == "runtime.error" {
        Some(
            event
                .payload
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| context.last_error.clone())
                .unwrap_or_else(|| "Runtime error".to_string()),
        )
    } else if status == "ready" {
        None
    } else {
        context.last_error.clone()
    };
    let mut session = json!({
        "threadId": context.thread_id,
        "status": status,
        "providerName": context.provider,
        "runtimeMode": runtime_mode_to_t3(context.runtime_mode),
        "activeTurnId": next_active_turn_id,
        "lastError": last_error,
        "updatedAt": event.created_at,
    });
    insert_optional(
        &mut session,
        "providerInstanceId",
        context
            .provider_instance_id
            .as_ref()
            .map(|provider_instance_id| json!(provider_instance_id)),
    );
    Some(OrchestrationCommand::ThreadSessionSet {
        command_id: command_id.to_string(),
        thread_id: context.thread_id.clone(),
        session,
        created_at: event.created_at.clone(),
    })
}

pub fn provider_runtime_assistant_delta_command(
    thread_id: &str,
    command_id: &str,
    event: &ProviderRuntimeEventInput,
) -> Option<OrchestrationCommand> {
    if event.event_type != "content.delta" {
        return None;
    }
    if event.payload.get("streamKind").and_then(Value::as_str) != Some("assistant_text") {
        return None;
    }
    let delta = event.payload.get("delta")?.as_str()?;
    if delta.is_empty() {
        return None;
    }
    Some(OrchestrationCommand::ThreadMessageAssistantDelta {
        command_id: command_id.to_string(),
        thread_id: thread_id.to_string(),
        message_id: assistant_segment_message_id(
            &assistant_segment_base_key_from_runtime_event(
                event.item_id.as_deref(),
                event.turn_id.as_deref(),
                &event.event_id,
            ),
            0,
        ),
        delta: delta.to_string(),
        turn_id: event.turn_id.clone(),
        created_at: event.created_at.clone(),
    })
}

pub fn provider_runtime_assistant_complete_command(
    thread_id: &str,
    command_id: &str,
    event: &ProviderRuntimeEventInput,
) -> Option<OrchestrationCommand> {
    if event.event_type != "item.completed" {
        return None;
    }
    if event.payload.get("itemType").and_then(Value::as_str) != Some("assistant_message") {
        return None;
    }
    Some(OrchestrationCommand::ThreadMessageAssistantComplete {
        command_id: command_id.to_string(),
        thread_id: thread_id.to_string(),
        message_id: assistant_segment_message_id(
            &assistant_segment_base_key_from_runtime_event(
                event.item_id.as_deref(),
                event.turn_id.as_deref(),
                &event.event_id,
            ),
            0,
        ),
        turn_id: event.turn_id.clone(),
        created_at: event.created_at.clone(),
    })
}

pub fn provider_runtime_assistant_completion_fallback_delta_command(
    thread_id: &str,
    command_id: &str,
    event: &ProviderRuntimeEventInput,
    projected_text_is_empty: bool,
) -> Option<OrchestrationCommand> {
    if !projected_text_is_empty {
        return None;
    }
    if event.event_type != "item.completed" {
        return None;
    }
    if event.payload.get("itemType").and_then(Value::as_str) != Some("assistant_message") {
        return None;
    }
    let fallback_text = event.payload.get("detail")?.as_str()?;
    if !has_renderable_assistant_text(Some(fallback_text)) {
        return None;
    }
    Some(OrchestrationCommand::ThreadMessageAssistantDelta {
        command_id: command_id.to_string(),
        thread_id: thread_id.to_string(),
        message_id: assistant_segment_message_id(
            &assistant_segment_base_key_from_runtime_event(
                event.item_id.as_deref(),
                event.turn_id.as_deref(),
                &event.event_id,
            ),
            0,
        ),
        delta: fallback_text.to_string(),
        turn_id: event.turn_id.clone(),
        created_at: event.created_at.clone(),
    })
}

pub fn provider_runtime_proposed_plan_complete_command(
    thread_id: &str,
    command_id: &str,
    event: &ProviderRuntimeEventInput,
    existing_plan_created_at: Option<&str>,
) -> Option<OrchestrationCommand> {
    if event.event_type != "turn.proposed.completed" {
        return None;
    }
    let plan_markdown = normalize_proposed_plan_markdown(
        event.payload.get("planMarkdown").and_then(Value::as_str),
    )?;
    let plan_id = proposed_plan_id_from_runtime_event(
        thread_id,
        event.turn_id.as_deref(),
        event.item_id.as_deref(),
        &event.event_id,
    );
    Some(OrchestrationCommand::ThreadProposedPlanUpsert {
        command_id: command_id.to_string(),
        thread_id: thread_id.to_string(),
        proposed_plan: ProposedPlan {
            id: plan_id,
            turn_id: event.turn_id.clone(),
            plan_markdown,
            implemented_at: None,
            implementation_thread_id: None,
            created_at: existing_plan_created_at
                .unwrap_or(event.created_at.as_str())
                .to_string(),
            updated_at: event.created_at.clone(),
        },
        created_at: event.created_at.clone(),
    })
}

pub fn provider_runtime_turn_completion_proposed_plan_command(
    thread_id: &str,
    command_id: &str,
    event: &ProviderRuntimeEventInput,
    plan_markdown: Option<&str>,
    existing_plan_created_at: Option<&str>,
) -> Option<OrchestrationCommand> {
    if event.event_type != "turn.completed" {
        return None;
    }
    let turn_id = event.turn_id.clone()?;
    let plan_markdown = normalize_proposed_plan_markdown(plan_markdown)?;
    Some(OrchestrationCommand::ThreadProposedPlanUpsert {
        command_id: command_id.to_string(),
        thread_id: thread_id.to_string(),
        proposed_plan: ProposedPlan {
            id: proposed_plan_id_for_turn(thread_id, &turn_id),
            turn_id: Some(turn_id),
            plan_markdown,
            implemented_at: None,
            implementation_thread_id: None,
            created_at: existing_plan_created_at
                .unwrap_or(event.created_at.as_str())
                .to_string(),
            updated_at: event.created_at.clone(),
        },
        created_at: event.created_at.clone(),
    })
}

pub fn provider_runtime_turn_diff_complete_command(
    thread_id: &str,
    command_id: &str,
    event: &ProviderRuntimeEventInput,
    checkpoint_turn_count: u32,
) -> Option<OrchestrationCommand> {
    if event.event_type != "turn.diff.updated" {
        return None;
    }
    let turn_id = event.turn_id.clone()?;
    Some(OrchestrationCommand::ThreadTurnDiffComplete {
        command_id: command_id.to_string(),
        thread_id: thread_id.to_string(),
        turn_id,
        checkpoint_turn_count,
        checkpoint_ref: Some(format!("provider-diff:{}", event.event_id)),
        status: "missing".to_string(),
        files: Vec::new(),
        assistant_message_id: Some(assistant_segment_message_id(
            &assistant_segment_base_key_from_runtime_event(
                event.item_id.as_deref(),
                event.turn_id.as_deref(),
                &event.event_id,
            ),
            0,
        )),
        completed_at: event.created_at.clone(),
        created_at: event.created_at.clone(),
    })
}

pub fn provider_runtime_thread_meta_update_command(
    thread_id: &str,
    command_id: &str,
    event: &ProviderRuntimeEventInput,
) -> Option<OrchestrationCommand> {
    if event.event_type != "thread.metadata.updated" {
        return None;
    }
    let title = event
        .payload
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    Some(OrchestrationCommand::ThreadMetaUpdate {
        command_id: command_id.to_string(),
        thread_id: thread_id.to_string(),
        title: Some(title.to_string()),
        model_selection: None,
        branch: None,
        worktree_path: None,
    })
}

pub fn map_provider_session_status_to_orchestration_status(
    status: ProviderSessionStatus,
) -> &'static str {
    match status {
        ProviderSessionStatus::Connecting => "starting",
        ProviderSessionStatus::Running => "running",
        ProviderSessionStatus::Error => "error",
        ProviderSessionStatus::Closed => "stopped",
        ProviderSessionStatus::Ready => "ready",
    }
}

pub fn map_provider_session_status_to_runtime_status(
    status: ProviderSessionStatus,
) -> &'static str {
    match status {
        ProviderSessionStatus::Connecting => "starting",
        ProviderSessionStatus::Error => "error",
        ProviderSessionStatus::Closed => "stopped",
        ProviderSessionStatus::Ready | ProviderSessionStatus::Running => "running",
    }
}

pub fn provider_service_runtime_payload_from_session(
    session: &ProviderSession,
    extra: Option<&ProviderServiceRuntimePayloadExtra>,
) -> Value {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "cwd".to_string(),
        session
            .cwd
            .as_ref()
            .map_or(Value::Null, |cwd| Value::String(cwd.clone())),
    );
    payload.insert(
        "model".to_string(),
        session
            .model
            .as_ref()
            .map_or(Value::Null, |model| Value::String(model.clone())),
    );
    payload.insert(
        "activeTurnId".to_string(),
        session
            .active_turn_id
            .as_ref()
            .map_or(Value::Null, |turn_id| Value::String(turn_id.clone())),
    );
    payload.insert(
        "lastError".to_string(),
        session
            .last_error
            .as_ref()
            .map_or(Value::Null, |last_error| Value::String(last_error.clone())),
    );

    if let Some(extra) = extra {
        if let Some(model_selection) = &extra.model_selection {
            payload.insert("modelSelection".to_string(), model_selection.clone());
        }
        if let Some(last_runtime_event) = &extra.last_runtime_event {
            payload.insert(
                "lastRuntimeEvent".to_string(),
                Value::String(last_runtime_event.clone()),
            );
        }
        if let Some(last_runtime_event_at) = &extra.last_runtime_event_at {
            payload.insert(
                "lastRuntimeEventAt".to_string(),
                Value::String(last_runtime_event_at.clone()),
            );
        }
    }

    Value::Object(payload)
}

pub fn provider_service_session_binding_upsert(
    session: &ProviderSession,
    thread_id: &str,
    extra: Option<&ProviderServiceRuntimePayloadExtra>,
) -> Result<ProviderRuntimeBinding, ProviderServicePlanError> {
    let provider_instance_id = provider_service_require_instance_id(
        "ProviderService.upsertSessionBinding",
        Some(session.provider.as_str()),
        session.provider_instance_id.as_deref(),
    )?;

    Ok(ProviderRuntimeBinding {
        thread_id: thread_id.to_string(),
        provider: session.provider.clone(),
        provider_instance_id: Some(provider_instance_id),
        adapter_key: None,
        status: Some(map_provider_session_status_to_runtime_status(session.status).to_string()),
        resume_cursor: session.resume_cursor.clone(),
        runtime_payload: Some(provider_service_runtime_payload_from_session(
            session, extra,
        )),
        runtime_mode: Some(session.runtime_mode),
    })
}

pub fn provider_service_send_turn_binding_upsert(
    input: &ProviderSendTurnInput,
    turn: &ProviderTurnStartResult,
    provider: &str,
    provider_instance_id: &str,
    now_iso: &str,
) -> ProviderRuntimeBinding {
    let mut runtime_payload = serde_json::Map::new();
    if let Some(model_selection) = &input.model_selection {
        runtime_payload.insert("modelSelection".to_string(), model_selection.clone());
    }
    runtime_payload.insert(
        "activeTurnId".to_string(),
        Value::String(turn.turn_id.clone()),
    );
    runtime_payload.insert(
        "lastRuntimeEvent".to_string(),
        Value::String("provider.sendTurn".to_string()),
    );
    runtime_payload.insert(
        "lastRuntimeEventAt".to_string(),
        Value::String(now_iso.to_string()),
    );

    ProviderRuntimeBinding {
        thread_id: input.thread_id.clone(),
        provider: provider.to_string(),
        provider_instance_id: Some(provider_instance_id.to_string()),
        adapter_key: None,
        status: Some("running".to_string()),
        resume_cursor: turn.resume_cursor.clone(),
        runtime_payload: Some(Value::Object(runtime_payload)),
        runtime_mode: None,
    }
}

pub fn provider_service_stop_session_binding_upsert(
    thread_id: &str,
    provider: &str,
    provider_instance_id: &str,
) -> ProviderRuntimeBinding {
    ProviderRuntimeBinding {
        thread_id: thread_id.to_string(),
        provider: provider.to_string(),
        provider_instance_id: Some(provider_instance_id.to_string()),
        adapter_key: None,
        status: Some("stopped".to_string()),
        resume_cursor: None,
        runtime_payload: Some(json!({ "activeTurnId": null })),
        runtime_mode: None,
    }
}

pub fn provider_service_list_sessions_merge(
    active_sessions: &[ProviderSession],
    persisted_bindings: &[ProviderRuntimeBinding],
) -> Result<Vec<ProviderSession>, ProviderServicePlanError> {
    let bindings_by_thread_id: std::collections::HashMap<&str, &ProviderRuntimeBinding> =
        persisted_bindings
            .iter()
            .map(|binding| (binding.thread_id.as_str(), binding))
            .collect();

    active_sessions
        .iter()
        .map(|session| {
            let Some(binding) = bindings_by_thread_id.get(session.thread_id.as_str()) else {
                return Ok(session.clone());
            };
            let binding_instance_id = provider_service_require_instance_id(
                "ProviderService.listSessions",
                Some(binding.provider.as_str()),
                binding.provider_instance_id.as_deref(),
            )?;

            if binding.provider != session.provider {
                return Err(ProviderServicePlanError::ListSessionsProviderMismatch {
                    thread_id: session.thread_id.clone(),
                    session_provider: session.provider.clone(),
                    binding_provider: binding.provider.clone(),
                });
            }

            if session.provider_instance_id.as_deref() != Some(binding_instance_id.as_str()) {
                return Err(ProviderServicePlanError::ListSessionsInstanceMismatch {
                    thread_id: session.thread_id.clone(),
                    session_instance_id: session.provider_instance_id.clone(),
                    binding_instance_id,
                });
            }

            let mut merged = session.clone();
            if merged.resume_cursor.is_none() && binding.resume_cursor.is_some() {
                merged.resume_cursor = binding.resume_cursor.clone();
            }
            if let Some(runtime_mode) = binding.runtime_mode {
                merged.runtime_mode = runtime_mode;
            }
            Ok(merged)
        })
        .collect()
}

pub fn provider_service_require_instance_id(
    operation: &str,
    provider: Option<&str>,
    provider_instance_id: Option<&str>,
) -> Result<String, ProviderServicePlanError> {
    provider_instance_id.map(str::to_string).ok_or_else(|| {
        ProviderServicePlanError::MissingProviderInstanceId {
            operation: operation.to_string(),
            provider: provider.map(str::to_string),
        }
    })
}

pub fn provider_service_persisted_model_selection(
    runtime_payload: Option<&Value>,
) -> Option<Value> {
    match runtime_payload {
        Some(Value::Object(payload)) => payload.get("modelSelection").cloned(),
        _ => None,
    }
}

pub fn provider_service_persisted_cwd(runtime_payload: Option<&Value>) -> Option<String> {
    match runtime_payload {
        Some(Value::Object(payload)) => payload
            .get("cwd")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|cwd| !cwd.is_empty())
            .map(str::to_string),
        _ => None,
    }
}

pub fn provider_service_start_session_plan(
    input: &ProviderSessionStartInput,
    instance_info: &ProviderInstanceRoutingInfo,
    persisted_binding: Option<&ProviderRuntimeBinding>,
) -> Result<ProviderServiceStartSessionPlan, ProviderServicePlanError> {
    if input.provider_instance_id != instance_info.instance_id {
        return Err(ProviderServicePlanError::ProviderInstanceMismatch {
            requested_instance_id: input.provider_instance_id.clone(),
            resolved_instance_id: instance_info.instance_id.clone(),
        });
    }

    if let Some(requested_provider) = &input.provider
        && requested_provider != &instance_info.driver_kind
    {
        return Err(ProviderServicePlanError::ProviderDriverMismatch {
            instance_id: instance_info.instance_id.clone(),
            instance_driver: instance_info.driver_kind.clone(),
            requested_provider: requested_provider.clone(),
        });
    }

    if !instance_info.enabled {
        return Err(ProviderServicePlanError::ProviderInstanceDisabled {
            instance_id: instance_info.instance_id.clone(),
        });
    }

    let persisted_for_same_instance = persisted_binding.filter(|binding| {
        binding.provider_instance_id.as_deref() == Some(instance_info.instance_id.as_str())
    });

    let (cwd, cwd_source) = match &input.cwd {
        Some(cwd) => (
            Some(cwd.clone()),
            ProviderServiceResolvedValueSource::Request,
        ),
        None => match persisted_for_same_instance
            .and_then(|binding| provider_service_persisted_cwd(binding.runtime_payload.as_ref()))
        {
            Some(cwd) => (Some(cwd), ProviderServiceResolvedValueSource::Persisted),
            None => (None, ProviderServiceResolvedValueSource::None),
        },
    };

    let (resume_cursor, resume_cursor_source) = match &input.resume_cursor {
        Some(cursor) => (
            Some(cursor.clone()),
            ProviderServiceResolvedValueSource::Request,
        ),
        None => match persisted_for_same_instance.and_then(|binding| binding.resume_cursor.clone())
        {
            Some(cursor) => (Some(cursor), ProviderServiceResolvedValueSource::Persisted),
            None => (None, ProviderServiceResolvedValueSource::None),
        },
    };

    Ok(ProviderServiceStartSessionPlan {
        adapter_input: ProviderSessionStartInput {
            thread_id: input.thread_id.clone(),
            provider: Some(instance_info.driver_kind.clone()),
            provider_instance_id: instance_info.instance_id.clone(),
            cwd,
            model_selection: input.model_selection.clone(),
            resume_cursor,
            approval_policy: input.approval_policy.clone(),
            sandbox_mode: input.sandbox_mode.clone(),
            runtime_mode: input.runtime_mode,
        },
        resume_cursor_source,
        cwd_source,
    })
}

pub fn provider_service_start_session_execution_plan(
    input: &ProviderSessionStartInput,
    instance_info: &ProviderInstanceRoutingInfo,
    persisted_binding: Option<&ProviderRuntimeBinding>,
) -> Result<ProviderServiceStartSessionExecutionPlan, ProviderServicePlanError> {
    let plan = provider_service_start_session_plan(input, instance_info, persisted_binding)?;
    Ok(ProviderServiceStartSessionExecutionPlan {
        start_call: ProviderServiceAdapterCall::StartSession {
            provider: instance_info.driver_kind.clone(),
            provider_instance_id: instance_info.instance_id.clone(),
            input: plan.adapter_input,
        },
        resume_cursor_source: plan.resume_cursor_source,
        cwd_source: plan.cwd_source,
    })
}

pub fn provider_service_start_session_completion_plan(
    session: &ProviderSession,
    thread_id: &str,
    expected_provider: &str,
    current_instance_id: &str,
    model_selection: Option<Value>,
    stale_session_probes: &[ProviderServiceAdapterSessionProbe],
) -> Result<ProviderServiceStartSessionCompletionPlan, ProviderServicePlanError> {
    if session.provider != expected_provider {
        return Err(
            ProviderServicePlanError::StartSessionAdapterProviderMismatch {
                expected_provider: expected_provider.to_string(),
                actual_provider: session.provider.clone(),
            },
        );
    }

    let mut session_with_instance = session.clone();
    session_with_instance.provider_instance_id = Some(current_instance_id.to_string());
    let binding = provider_service_session_binding_upsert(
        &session_with_instance,
        thread_id,
        Some(&ProviderServiceRuntimePayloadExtra {
            model_selection,
            last_runtime_event: None,
            last_runtime_event_at: None,
        }),
    )?;

    Ok(ProviderServiceStartSessionCompletionPlan {
        binding,
        stop_stale_calls: provider_service_stop_stale_session_calls(
            thread_id,
            current_instance_id,
            stale_session_probes,
        ),
    })
}

pub fn provider_service_send_turn_plan(
    input: &ProviderSendTurnInput,
) -> Result<ProviderSendTurnInput, ProviderServicePlanError> {
    let has_input_text = input
        .input
        .as_deref()
        .map(str::trim)
        .is_some_and(|text| !text.is_empty());
    if !has_input_text && input.attachments.is_empty() {
        return Err(ProviderServicePlanError::EmptySendTurnInput {
            operation: "ProviderService.sendTurn".to_string(),
        });
    }

    Ok(input.clone())
}

pub fn provider_service_recover_session_plan(
    binding: &ProviderRuntimeBinding,
    operation: &str,
    has_active_session: bool,
    active_session: Option<&ProviderSession>,
) -> Result<ProviderServiceRecoveryPlan, ProviderServicePlanError> {
    let instance_id = provider_service_require_instance_id(
        operation,
        Some(binding.provider.as_str()),
        binding.provider_instance_id.as_deref(),
    )?;

    if has_active_session
        && let Some(session) = active_session
        && session.thread_id == binding.thread_id
    {
        let mut adopted = session.clone();
        adopted.provider_instance_id = Some(instance_id);
        return Ok(ProviderServiceRecoveryPlan::AdoptExisting { session: adopted });
    }

    let resume_cursor = binding
        .resume_cursor
        .clone()
        .and_then(|cursor| if cursor.is_null() { None } else { Some(cursor) });
    let Some(resume_cursor) = resume_cursor else {
        return Err(
            ProviderServicePlanError::CannotRecoverThreadWithoutResumeState {
                operation: operation.to_string(),
                thread_id: binding.thread_id.clone(),
            },
        );
    };

    Ok(ProviderServiceRecoveryPlan::Resume {
        adapter_input: ProviderSessionStartInput {
            thread_id: binding.thread_id.clone(),
            provider: Some(binding.provider.clone()),
            provider_instance_id: provider_service_require_instance_id(
                operation,
                Some(binding.provider.as_str()),
                binding.provider_instance_id.as_deref(),
            )?,
            cwd: provider_service_persisted_cwd(binding.runtime_payload.as_ref()),
            model_selection: provider_service_persisted_model_selection(
                binding.runtime_payload.as_ref(),
            ),
            resume_cursor: Some(resume_cursor),
            approval_policy: None,
            sandbox_mode: None,
            runtime_mode: binding.runtime_mode.unwrap_or(RuntimeMode::FullAccess),
        },
    })
}

pub fn provider_service_routable_session_plan(
    thread_id: &str,
    binding: Option<&ProviderRuntimeBinding>,
    operation: &str,
    allow_recovery: bool,
    has_active_session: bool,
    active_session: Option<&ProviderSession>,
) -> Result<ProviderServiceRoutableSessionPlan, ProviderServicePlanError> {
    let Some(binding) = binding else {
        return Err(ProviderServicePlanError::CannotRouteThreadWithoutBinding {
            operation: operation.to_string(),
            thread_id: thread_id.to_string(),
        });
    };
    let instance_id = provider_service_require_instance_id(
        operation,
        Some(binding.provider.as_str()),
        binding.provider_instance_id.as_deref(),
    )?;

    if has_active_session {
        return Ok(ProviderServiceRoutableSessionPlan::Active {
            provider: binding.provider.clone(),
            instance_id,
            thread_id: binding.thread_id.clone(),
        });
    }

    if !allow_recovery {
        return Ok(ProviderServiceRoutableSessionPlan::Inactive {
            provider: binding.provider.clone(),
            instance_id,
            thread_id: binding.thread_id.clone(),
        });
    }

    provider_service_recover_session_plan(binding, operation, has_active_session, active_session)
        .map(ProviderServiceRoutableSessionPlan::Recover)
}

pub fn provider_service_rollback_conversation_plan(
    input: &ProviderRollbackConversationInput,
    binding: Option<&ProviderRuntimeBinding>,
    has_active_session: bool,
    active_session: Option<&ProviderSession>,
) -> Result<ProviderServiceRollbackConversationPlan, ProviderServicePlanError> {
    if input.num_turns == 0 {
        return Ok(ProviderServiceRollbackConversationPlan::Noop);
    }

    let route = provider_service_routable_session_plan(
        input.thread_id.as_str(),
        binding,
        "ProviderService.rollbackConversation",
        true,
        has_active_session,
        active_session,
    )?;

    Ok(ProviderServiceRollbackConversationPlan::Route {
        route,
        num_turns: input.num_turns,
    })
}

fn provider_service_route_identity(
    route: &ProviderServiceRoutableSessionPlan,
) -> Result<(String, String, String), ProviderServicePlanError> {
    match route {
        ProviderServiceRoutableSessionPlan::Active {
            provider,
            instance_id,
            thread_id,
        }
        | ProviderServiceRoutableSessionPlan::Inactive {
            provider,
            instance_id,
            thread_id,
        } => Ok((provider.clone(), instance_id.clone(), thread_id.clone())),
        ProviderServiceRoutableSessionPlan::Recover(
            ProviderServiceRecoveryPlan::AdoptExisting { session },
        ) => Ok((
            session.provider.clone(),
            provider_service_require_instance_id(
                "ProviderService.route",
                Some(session.provider.as_str()),
                session.provider_instance_id.as_deref(),
            )?,
            session.thread_id.clone(),
        )),
        ProviderServiceRoutableSessionPlan::Recover(ProviderServiceRecoveryPlan::Resume {
            adapter_input,
        }) => Ok((
            adapter_input.provider.clone().ok_or_else(|| {
                ProviderServicePlanError::MissingProviderInstanceId {
                    operation: "ProviderService.route".to_string(),
                    provider: None,
                }
            })?,
            adapter_input.provider_instance_id.clone(),
            adapter_input.thread_id.clone(),
        )),
    }
}

fn provider_service_recovery_prefix_calls(
    route: &ProviderServiceRoutableSessionPlan,
) -> Result<Vec<ProviderServiceAdapterCall>, ProviderServicePlanError> {
    match route {
        ProviderServiceRoutableSessionPlan::Recover(ProviderServiceRecoveryPlan::Resume {
            adapter_input,
        }) => Ok(vec![ProviderServiceAdapterCall::StartSession {
            provider: adapter_input.provider.clone().ok_or_else(|| {
                ProviderServicePlanError::MissingProviderInstanceId {
                    operation: "ProviderService.startSession".to_string(),
                    provider: None,
                }
            })?,
            provider_instance_id: adapter_input.provider_instance_id.clone(),
            input: adapter_input.clone(),
        }]),
        _ => Ok(Vec::new()),
    }
}

pub fn provider_service_send_turn_adapter_calls(
    input: &ProviderSendTurnInput,
    binding: Option<&ProviderRuntimeBinding>,
    has_active_session: bool,
    active_session: Option<&ProviderSession>,
) -> Result<Vec<ProviderServiceAdapterCall>, ProviderServicePlanError> {
    let input = provider_service_send_turn_plan(input)?;
    let route = provider_service_routable_session_plan(
        input.thread_id.as_str(),
        binding,
        "ProviderService.sendTurn",
        true,
        has_active_session,
        active_session,
    )?;
    let (provider, provider_instance_id, _) = provider_service_route_identity(&route)?;
    let mut calls = provider_service_recovery_prefix_calls(&route)?;
    calls.push(ProviderServiceAdapterCall::SendTurn {
        provider,
        provider_instance_id,
        input,
    });
    Ok(calls)
}

pub fn provider_service_interrupt_turn_adapter_calls(
    input: &ProviderInterruptTurnInput,
    binding: Option<&ProviderRuntimeBinding>,
    has_active_session: bool,
    active_session: Option<&ProviderSession>,
) -> Result<Vec<ProviderServiceAdapterCall>, ProviderServicePlanError> {
    let route = provider_service_routable_session_plan(
        input.thread_id.as_str(),
        binding,
        "ProviderService.interruptTurn",
        true,
        has_active_session,
        active_session,
    )?;
    let (provider, provider_instance_id, _) = provider_service_route_identity(&route)?;
    let mut calls = provider_service_recovery_prefix_calls(&route)?;
    calls.push(ProviderServiceAdapterCall::InterruptTurn {
        provider,
        provider_instance_id,
        input: input.clone(),
    });
    Ok(calls)
}

pub fn provider_service_respond_to_request_adapter_calls(
    input: &ProviderRespondToRequestInput,
    binding: Option<&ProviderRuntimeBinding>,
    has_active_session: bool,
    active_session: Option<&ProviderSession>,
) -> Result<Vec<ProviderServiceAdapterCall>, ProviderServicePlanError> {
    let route = provider_service_routable_session_plan(
        input.thread_id.as_str(),
        binding,
        "ProviderService.respondToRequest",
        true,
        has_active_session,
        active_session,
    )?;
    let (provider, provider_instance_id, _) = provider_service_route_identity(&route)?;
    let mut calls = provider_service_recovery_prefix_calls(&route)?;
    calls.push(ProviderServiceAdapterCall::RespondToRequest {
        provider,
        provider_instance_id,
        input: input.clone(),
    });
    Ok(calls)
}

pub fn provider_service_respond_to_user_input_adapter_calls(
    input: &ProviderRespondToUserInputInput,
    binding: Option<&ProviderRuntimeBinding>,
    has_active_session: bool,
    active_session: Option<&ProviderSession>,
) -> Result<Vec<ProviderServiceAdapterCall>, ProviderServicePlanError> {
    let route = provider_service_routable_session_plan(
        input.thread_id.as_str(),
        binding,
        "ProviderService.respondToUserInput",
        true,
        has_active_session,
        active_session,
    )?;
    let (provider, provider_instance_id, _) = provider_service_route_identity(&route)?;
    let mut calls = provider_service_recovery_prefix_calls(&route)?;
    calls.push(ProviderServiceAdapterCall::RespondToUserInput {
        provider,
        provider_instance_id,
        input: input.clone(),
    });
    Ok(calls)
}

pub fn provider_service_stop_session_adapter_calls(
    input: &ProviderStopSessionInput,
    binding: Option<&ProviderRuntimeBinding>,
    has_active_session: bool,
) -> Result<Vec<ProviderServiceAdapterCall>, ProviderServicePlanError> {
    let route = provider_service_routable_session_plan(
        input.thread_id.as_str(),
        binding,
        "ProviderService.stopSession",
        false,
        has_active_session,
        None,
    )?;
    let ProviderServiceRoutableSessionPlan::Active { .. } = route else {
        return Ok(Vec::new());
    };
    let (provider, provider_instance_id, _) = provider_service_route_identity(&route)?;
    Ok(vec![ProviderServiceAdapterCall::StopSession {
        provider,
        provider_instance_id,
        input: input.clone(),
    }])
}

pub fn provider_service_rollback_conversation_adapter_calls(
    input: &ProviderRollbackConversationInput,
    binding: Option<&ProviderRuntimeBinding>,
    has_active_session: bool,
    active_session: Option<&ProviderSession>,
) -> Result<Vec<ProviderServiceAdapterCall>, ProviderServicePlanError> {
    let plan = provider_service_rollback_conversation_plan(
        input,
        binding,
        has_active_session,
        active_session,
    )?;
    let ProviderServiceRollbackConversationPlan::Route { route, .. } = plan else {
        return Ok(Vec::new());
    };
    let (provider, provider_instance_id, _) = provider_service_route_identity(&route)?;
    let mut calls = provider_service_recovery_prefix_calls(&route)?;
    calls.push(ProviderServiceAdapterCall::RollbackConversation {
        provider,
        provider_instance_id,
        input: input.clone(),
    });
    Ok(calls)
}

pub fn provider_service_stop_stale_session_calls(
    thread_id: &str,
    current_instance_id: &str,
    probes: &[ProviderServiceAdapterSessionProbe],
) -> Vec<ProviderServiceStopSessionCall> {
    probes
        .iter()
        .filter(|probe| probe.provider_instance_id != current_instance_id && probe.has_session)
        .map(|probe| ProviderServiceStopSessionCall {
            provider: probe.provider.clone(),
            provider_instance_id: probe.provider_instance_id.clone(),
            input: ProviderStopSessionInput {
                thread_id: thread_id.to_string(),
            },
        })
        .collect()
}

pub fn provider_service_instance_info_for_instance_id(
    instance_id: &str,
    registry_entries: &[ProviderServiceAdapterRegistryEntry],
) -> Result<ProviderInstanceRoutingInfo, ProviderServicePlanError> {
    registry_entries
        .iter()
        .find(|entry| entry.instance_info.instance_id == instance_id)
        .map(|entry| entry.instance_info.clone())
        .ok_or_else(|| ProviderServicePlanError::ProviderInstanceNotFound {
            operation: "ProviderService.getInstanceInfo".to_string(),
            instance_id: instance_id.to_string(),
        })
}

pub fn provider_service_capabilities_for_instance_id(
    instance_id: &str,
    registry_entries: &[ProviderServiceAdapterRegistryEntry],
) -> Result<ProviderAdapterCapabilities, ProviderServicePlanError> {
    registry_entries
        .iter()
        .find(|entry| entry.instance_info.instance_id == instance_id)
        .map(|entry| entry.capabilities.clone())
        .ok_or_else(|| ProviderServicePlanError::ProviderInstanceNotFound {
            operation: "ProviderService.getCapabilities".to_string(),
            instance_id: instance_id.to_string(),
        })
}

pub fn provider_service_correlate_runtime_event_instance(
    source_provider: &str,
    source_instance_id: &str,
    event_provider: &str,
    event_instance_id: Option<&str>,
) -> Result<String, ProviderServicePlanError> {
    if event_provider != source_provider {
        return Err(ProviderServicePlanError::RuntimeEventProviderMismatch {
            source_provider: source_provider.to_string(),
            event_provider: event_provider.to_string(),
            provider_instance_id: source_instance_id.to_string(),
        });
    }
    if let Some(event_instance_id) = event_instance_id
        && event_instance_id != source_instance_id
    {
        return Err(ProviderServicePlanError::RuntimeEventInstanceMismatch {
            source_instance_id: source_instance_id.to_string(),
            event_instance_id: event_instance_id.to_string(),
        });
    }

    Ok(source_instance_id.to_string())
}

pub fn provider_service_runtime_event_fanout_plan(
    source_provider: &str,
    source_instance_id: &str,
    events: &[ProviderServiceRuntimeEventEnvelope],
) -> Result<ProviderServiceRuntimeEventFanoutPlan, ProviderServicePlanError> {
    let canonical_events = events
        .iter()
        .map(|event| {
            let provider_instance_id = provider_service_correlate_runtime_event_instance(
                source_provider,
                source_instance_id,
                event.provider.as_str(),
                event.provider_instance_id.as_deref(),
            )?;
            Ok(ProviderServiceRuntimeEventEnvelope {
                provider: event.provider.clone(),
                provider_instance_id: Some(provider_instance_id),
                thread_id: event.thread_id.clone(),
                event: event.event.clone(),
            })
        })
        .collect::<Result<Vec<_>, ProviderServicePlanError>>()?;
    let log_thread_ids = canonical_events
        .iter()
        .map(|event| event.thread_id.clone())
        .collect();

    Ok(ProviderServiceRuntimeEventFanoutPlan {
        canonical_events,
        log_thread_ids,
    })
}

pub fn provider_service_stop_all_plan(
    thread_ids: &[String],
    registry_entries: &[ProviderServiceAdapterRegistryEntry],
    active_sessions: &[ProviderSession],
    bindings: &[ProviderRuntimeBinding],
    now_iso: &str,
) -> Result<ProviderServiceStopAllPlan, ProviderServicePlanError> {
    let active_session_bindings = active_sessions
        .iter()
        .map(|session| {
            provider_service_session_binding_upsert(
                session,
                session.thread_id.as_str(),
                Some(&ProviderServiceRuntimePayloadExtra {
                    model_selection: None,
                    last_runtime_event: Some("provider.stopAll".to_string()),
                    last_runtime_event_at: Some(now_iso.to_string()),
                }),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let stop_all_calls = registry_entries
        .iter()
        .map(|entry| ProviderServiceStopAllCall {
            provider: entry.instance_info.driver_kind.clone(),
            provider_instance_id: entry.instance_info.instance_id.clone(),
        })
        .collect();

    let stopped_bindings = bindings
        .iter()
        .map(|binding| {
            let provider_instance_id = provider_service_require_instance_id(
                "ProviderService.stopAll",
                Some(binding.provider.as_str()),
                binding.provider_instance_id.as_deref(),
            )?;
            Ok(ProviderRuntimeBinding {
                thread_id: binding.thread_id.clone(),
                provider: binding.provider.clone(),
                provider_instance_id: Some(provider_instance_id),
                adapter_key: binding.adapter_key.clone(),
                status: Some("stopped".to_string()),
                resume_cursor: None,
                runtime_payload: Some(json!({
                    "activeTurnId": null,
                    "lastRuntimeEvent": "provider.stopAll",
                    "lastRuntimeEventAt": now_iso,
                })),
                runtime_mode: binding.runtime_mode,
            })
        })
        .collect::<Result<Vec<_>, ProviderServicePlanError>>()?;

    Ok(ProviderServiceStopAllPlan {
        active_session_bindings,
        stop_all_calls,
        stopped_bindings,
        session_count: thread_ids.len(),
    })
}

pub const CHECKPOINT_REFS_PREFIX: &str = "refs/t3/checkpoints";
pub const PROVIDER_SEND_TURN_MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointDiffStatus {
    Ready,
    Missing,
    Error,
}

impl CheckpointDiffStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Missing => "missing",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrchestrationRuntimeReceipt {
    CheckpointBaselineCaptured {
        thread_id: String,
        checkpoint_turn_count: u32,
        checkpoint_ref: String,
        created_at: String,
    },
    CheckpointDiffFinalized {
        thread_id: String,
        turn_id: String,
        checkpoint_turn_count: u32,
        checkpoint_ref: String,
        status: CheckpointDiffStatus,
        created_at: String,
    },
    TurnProcessingQuiesced {
        thread_id: String,
        turn_id: String,
        checkpoint_turn_count: u32,
        created_at: String,
    },
}

impl OrchestrationRuntimeReceipt {
    pub fn receipt_type(&self) -> &'static str {
        match self {
            Self::CheckpointBaselineCaptured { .. } => "checkpoint.baseline.captured",
            Self::CheckpointDiffFinalized { .. } => "checkpoint.diff.finalized",
            Self::TurnProcessingQuiesced { .. } => "turn.processing.quiesced",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeReceiptBusLayerKind {
    Live,
    Test,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeReceiptBusLayerPlan {
    pub layer_kind: RuntimeReceiptBusLayerKind,
    pub publish_retains_receipts: bool,
    pub stream_events_for_test: &'static str,
}

pub fn runtime_receipt_bus_layer_plan(
    layer_kind: RuntimeReceiptBusLayerKind,
) -> RuntimeReceiptBusLayerPlan {
    match layer_kind {
        RuntimeReceiptBusLayerKind::Live => RuntimeReceiptBusLayerPlan {
            layer_kind,
            publish_retains_receipts: false,
            stream_events_for_test: "empty",
        },
        RuntimeReceiptBusLayerKind::Test => RuntimeReceiptBusLayerPlan {
            layer_kind,
            publish_retains_receipts: true,
            stream_events_for_test: "pubsub",
        },
    }
}

pub fn checkpoint_ref_for_thread_turn(thread_id: &str, turn_count: u32) -> String {
    format!(
        "{CHECKPOINT_REFS_PREFIX}/{}/turn/{turn_count}",
        base64url_no_pad(thread_id.as_bytes())
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionCheckpointSummary {
    pub turn_id: String,
    pub checkpoint_turn_count: u32,
    pub checkpoint_ref: String,
    pub status: String,
    pub files: Vec<TurnDiffFileChange>,
    pub assistant_message_id: Option<String>,
    pub completed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionThreadCheckpointContext {
    pub thread_id: String,
    pub project_id: String,
    pub workspace_root: String,
    pub worktree_path: Option<String>,
    pub checkpoints: Vec<ProjectionCheckpointSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionFullThreadDiffContext {
    pub thread_id: String,
    pub project_id: String,
    pub workspace_root: String,
    pub worktree_path: Option<String>,
    pub latest_checkpoint_turn_count: u32,
    pub to_checkpoint_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationGetTurnDiffInput {
    pub thread_id: String,
    pub from_turn_count: u32,
    pub to_turn_count: u32,
    pub ignore_whitespace: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationGetFullThreadDiffInput {
    pub thread_id: String,
    pub to_turn_count: u32,
    pub ignore_whitespace: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationCheckpointDiffResult {
    pub thread_id: String,
    pub from_turn_count: u32,
    pub to_turn_count: u32,
    pub diff: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointDiffQueryPlan {
    pub thread_id: String,
    pub from_turn_count: u32,
    pub to_turn_count: u32,
    pub cwd: String,
    pub from_checkpoint_ref: String,
    pub to_checkpoint_ref: String,
    pub fallback_from_to_head: bool,
    pub ignore_whitespace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointDiffQueryDecision {
    Empty(OrchestrationCheckpointDiffResult),
    Diff(CheckpointDiffQueryPlan),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointDiffQueryError {
    Invariant {
        operation: &'static str,
        detail: String,
    },
    Unavailable {
        thread_id: String,
        turn_count: u32,
        detail: String,
    },
}

impl CheckpointDiffQueryError {
    pub fn message(&self) -> String {
        match self {
            Self::Invariant { operation, detail } => {
                format!("Checkpoint invariant violation in {operation}: {detail}")
            }
            Self::Unavailable {
                thread_id,
                turn_count,
                detail,
            } => {
                format!("Checkpoint unavailable for thread {thread_id} turn {turn_count}: {detail}")
            }
        }
    }
}

pub fn build_checkpoint_diff_result(
    thread_id: &str,
    from_turn_count: u32,
    to_turn_count: u32,
    diff: impl Into<String>,
) -> OrchestrationCheckpointDiffResult {
    OrchestrationCheckpointDiffResult {
        thread_id: thread_id.to_string(),
        from_turn_count,
        to_turn_count,
        diff: diff.into(),
    }
}

fn checkpoint_diff_workspace_cwd(
    workspace_root: &str,
    worktree_path: Option<&str>,
) -> Option<String> {
    worktree_path
        .filter(|path| !path.is_empty())
        .or_else(|| (!workspace_root.is_empty()).then_some(workspace_root))
        .map(str::to_string)
}

fn checkpoint_ref_from_context(
    context: &ProjectionThreadCheckpointContext,
    turn_count: u32,
) -> Option<String> {
    context
        .checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint_turn_count == turn_count)
        .map(|checkpoint| checkpoint.checkpoint_ref.clone())
}

pub fn resolve_checkpoint_turn_diff_query(
    input: &OrchestrationGetTurnDiffInput,
    context: Option<&ProjectionThreadCheckpointContext>,
) -> Result<CheckpointDiffQueryDecision, CheckpointDiffQueryError> {
    const OPERATION: &str = "CheckpointDiffQuery.getTurnDiff";
    let ignore_whitespace = input.ignore_whitespace.unwrap_or(true);

    if input.from_turn_count == input.to_turn_count {
        return Ok(CheckpointDiffQueryDecision::Empty(
            build_checkpoint_diff_result(
                &input.thread_id,
                input.from_turn_count,
                input.to_turn_count,
                "",
            ),
        ));
    }

    let context = context.ok_or_else(|| CheckpointDiffQueryError::Invariant {
        operation: OPERATION,
        detail: format!("Thread '{}' not found.", input.thread_id),
    })?;

    let max_turn_count = context
        .checkpoints
        .iter()
        .map(|checkpoint| checkpoint.checkpoint_turn_count)
        .max()
        .unwrap_or(0);
    if input.to_turn_count > max_turn_count {
        return Err(CheckpointDiffQueryError::Unavailable {
            thread_id: input.thread_id.clone(),
            turn_count: input.to_turn_count,
            detail: format!(
                "Turn diff range exceeds current turn count: requested {}, current {}.",
                input.to_turn_count, max_turn_count
            ),
        });
    }

    let cwd =
        checkpoint_diff_workspace_cwd(&context.workspace_root, context.worktree_path.as_deref())
            .ok_or_else(|| CheckpointDiffQueryError::Invariant {
                operation: OPERATION,
                detail: format!(
                    "Workspace path missing for thread '{}' when computing turn diff.",
                    input.thread_id
                ),
            })?;

    let from_checkpoint_ref = if input.from_turn_count == 0 {
        Some(checkpoint_ref_for_thread_turn(&input.thread_id, 0))
    } else {
        checkpoint_ref_from_context(context, input.from_turn_count)
    }
    .ok_or_else(|| CheckpointDiffQueryError::Unavailable {
        thread_id: input.thread_id.clone(),
        turn_count: input.from_turn_count,
        detail: format!(
            "Checkpoint ref is unavailable for turn {}.",
            input.from_turn_count
        ),
    })?;

    let to_checkpoint_ref =
        checkpoint_ref_from_context(context, input.to_turn_count).ok_or_else(|| {
            CheckpointDiffQueryError::Unavailable {
                thread_id: input.thread_id.clone(),
                turn_count: input.to_turn_count,
                detail: format!(
                    "Checkpoint ref is unavailable for turn {}.",
                    input.to_turn_count
                ),
            }
        })?;

    Ok(CheckpointDiffQueryDecision::Diff(CheckpointDiffQueryPlan {
        thread_id: input.thread_id.clone(),
        from_turn_count: input.from_turn_count,
        to_turn_count: input.to_turn_count,
        cwd,
        from_checkpoint_ref,
        to_checkpoint_ref,
        fallback_from_to_head: false,
        ignore_whitespace,
    }))
}

pub fn resolve_checkpoint_full_thread_diff_query(
    input: &OrchestrationGetFullThreadDiffInput,
    context: Option<&ProjectionFullThreadDiffContext>,
) -> Result<CheckpointDiffQueryDecision, CheckpointDiffQueryError> {
    const OPERATION: &str = "CheckpointDiffQuery.getFullThreadDiff";
    let ignore_whitespace = input.ignore_whitespace.unwrap_or(true);

    if input.to_turn_count == 0 {
        return Ok(CheckpointDiffQueryDecision::Empty(
            build_checkpoint_diff_result(&input.thread_id, 0, 0, ""),
        ));
    }

    let context = context.ok_or_else(|| CheckpointDiffQueryError::Invariant {
        operation: OPERATION,
        detail: format!("Thread '{}' not found.", input.thread_id),
    })?;

    if input.to_turn_count > context.latest_checkpoint_turn_count {
        return Err(CheckpointDiffQueryError::Unavailable {
            thread_id: input.thread_id.clone(),
            turn_count: input.to_turn_count,
            detail: format!(
                "Turn diff range exceeds current turn count: requested {}, current {}.",
                input.to_turn_count, context.latest_checkpoint_turn_count
            ),
        });
    }

    let cwd =
        checkpoint_diff_workspace_cwd(&context.workspace_root, context.worktree_path.as_deref())
            .ok_or_else(|| CheckpointDiffQueryError::Invariant {
                operation: OPERATION,
                detail: format!(
                    "Workspace path missing for thread '{}' when computing full thread diff.",
                    input.thread_id
                ),
            })?;

    let to_checkpoint_ref =
        context
            .to_checkpoint_ref
            .clone()
            .ok_or_else(|| CheckpointDiffQueryError::Unavailable {
                thread_id: input.thread_id.clone(),
                turn_count: input.to_turn_count,
                detail: format!(
                    "Checkpoint ref is unavailable for turn {}.",
                    input.to_turn_count
                ),
            })?;

    Ok(CheckpointDiffQueryDecision::Diff(CheckpointDiffQueryPlan {
        thread_id: input.thread_id.clone(),
        from_turn_count: 0,
        to_turn_count: input.to_turn_count,
        cwd,
        from_checkpoint_ref: checkpoint_ref_for_thread_turn(&input.thread_id, 0),
        to_checkpoint_ref,
        fallback_from_to_head: false,
        ignore_whitespace,
    }))
}

pub fn checkpoint_status_from_runtime(status: Option<&str>) -> CheckpointDiffStatus {
    match status {
        Some("failed") => CheckpointDiffStatus::Error,
        Some("cancelled") | Some("interrupted") => CheckpointDiffStatus::Missing,
        Some("completed") | _ => CheckpointDiffStatus::Ready,
    }
}

pub fn checkpoint_reactor_domain_event_is_enqueued(event_type: &str) -> bool {
    matches!(
        event_type,
        "thread.turn-start-requested"
            | "thread.message-sent"
            | "thread.checkpoint-revert-requested"
            | "thread.turn-diff-completed"
    )
}

pub fn checkpoint_reactor_runtime_event_is_enqueued(event_type: &str) -> bool {
    matches!(event_type, "turn.started" | "turn.completed")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadWorkspaceCwdInput {
    pub thread_project_id: String,
    pub thread_worktree_path: Option<String>,
    pub projects: Vec<ThreadWorkspaceProject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadWorkspaceProject {
    pub project_id: String,
    pub workspace_root: String,
}

pub fn resolve_thread_workspace_cwd(input: &ThreadWorkspaceCwdInput) -> Option<String> {
    input.thread_worktree_path.clone().or_else(|| {
        input
            .projects
            .iter()
            .find(|project| project.project_id == input.thread_project_id)
            .map(|project| project.workspace_root.clone())
    })
}

pub fn checkpoint_reactor_resolve_checkpoint_cwd(
    from_session_cwd: Option<&str>,
    from_thread_cwd: Option<&str>,
    prefer_session_runtime: bool,
) -> Option<String> {
    if prefer_session_runtime {
        from_session_cwd.or(from_thread_cwd)
    } else {
        from_thread_cwd.or(from_session_cwd)
    }
    .map(str::to_string)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointReactorAction {
    CaptureBaseline {
        checkpoint_ref: String,
    },
    CaptureTurnDiff {
        from_checkpoint_ref: String,
        target_checkpoint_ref: String,
        status: CheckpointDiffStatus,
    },
    RefreshLocalGitStatus {
        cwd: String,
    },
    RollbackConversation {
        num_turns: u32,
    },
    DeleteStaleCheckpointRefs {
        checkpoint_refs: Vec<String>,
    },
    AppendRevertFailureActivity {
        detail: String,
    },
}

pub fn checkpoint_reactor_baseline_capture_action(
    thread_id: &str,
    current_turn_count: u32,
    baseline_exists: bool,
) -> Option<CheckpointReactorAction> {
    (!baseline_exists).then(|| CheckpointReactorAction::CaptureBaseline {
        checkpoint_ref: checkpoint_ref_for_thread_turn(thread_id, current_turn_count),
    })
}

pub fn checkpoint_reactor_turn_completion_action(
    thread_id: &str,
    turn_count: u32,
    runtime_status: Option<&str>,
    has_real_checkpoint_for_turn: bool,
) -> Option<CheckpointReactorAction> {
    (!has_real_checkpoint_for_turn).then(|| CheckpointReactorAction::CaptureTurnDiff {
        from_checkpoint_ref: checkpoint_ref_for_thread_turn(
            thread_id,
            turn_count.saturating_sub(1),
        ),
        target_checkpoint_ref: checkpoint_ref_for_thread_turn(thread_id, turn_count),
        status: checkpoint_status_from_runtime(runtime_status),
    })
}

pub fn checkpoint_reactor_revert_actions(
    thread_id: &str,
    requested_turn_count: u32,
    current_turn_count: u32,
    session_cwd: Option<&str>,
    is_git_workspace: bool,
    known_checkpoint_ref: Option<&str>,
) -> Vec<CheckpointReactorAction> {
    if session_cwd.is_none() {
        return vec![CheckpointReactorAction::AppendRevertFailureActivity {
            detail: "No active provider session with workspace cwd is bound to this thread."
                .to_string(),
        }];
    }
    if !is_git_workspace {
        return vec![CheckpointReactorAction::AppendRevertFailureActivity {
            detail: "Checkpoints are unavailable because this project is not a git repository."
                .to_string(),
        }];
    }
    if requested_turn_count > current_turn_count {
        return vec![CheckpointReactorAction::AppendRevertFailureActivity {
            detail: format!(
                "Checkpoint turn count {requested_turn_count} exceeds current turn count {current_turn_count}."
            ),
        }];
    }
    let Some(target_checkpoint_ref) = known_checkpoint_ref.map(str::to_string).or_else(|| {
        (requested_turn_count == 0).then(|| checkpoint_ref_for_thread_turn(thread_id, 0))
    }) else {
        return vec![CheckpointReactorAction::AppendRevertFailureActivity {
            detail: format!(
                "Checkpoint ref for turn {requested_turn_count} is unavailable in read model."
            ),
        }];
    };
    let rolled_back_turns = current_turn_count.saturating_sub(requested_turn_count);
    let mut actions = vec![CheckpointReactorAction::CaptureBaseline {
        checkpoint_ref: target_checkpoint_ref,
    }];
    if rolled_back_turns > 0 {
        actions.push(CheckpointReactorAction::RollbackConversation {
            num_turns: rolled_back_turns,
        });
        actions.push(CheckpointReactorAction::DeleteStaleCheckpointRefs {
            checkpoint_refs: ((requested_turn_count + 1)..=current_turn_count)
                .map(|turn_count| checkpoint_ref_for_thread_turn(thread_id, turn_count))
                .collect(),
        });
    }
    actions
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationLayerCompositionPlan {
    pub event_infrastructure_layers: Vec<&'static str>,
    pub projection_pipeline_layers: Vec<&'static str>,
    pub infrastructure_layers: Vec<&'static str>,
    pub live_layers: Vec<&'static str>,
}

pub fn orchestration_layer_composition_plan() -> OrchestrationLayerCompositionPlan {
    OrchestrationLayerCompositionPlan {
        event_infrastructure_layers: vec![
            "OrchestrationEventStoreLive",
            "OrchestrationCommandReceiptRepositoryLive",
        ],
        projection_pipeline_layers: vec!["OrchestrationProjectionPipelineLive"],
        infrastructure_layers: vec![
            "OrchestrationProjectionSnapshotQueryLive",
            "OrchestrationEventInfrastructureLayerLive",
            "OrchestrationProjectionPipelineLayerLive",
        ],
        live_layers: vec![
            "OrchestrationInfrastructureLayerLive",
            "OrchestrationEngineLive",
        ],
    }
}

pub fn orchestration_schema_aliases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("ProjectCreatedPayload", "ProjectCreatedPayload"),
        ("ProjectMetaUpdatedPayload", "ProjectMetaUpdatedPayload"),
        ("ProjectDeletedPayload", "ProjectDeletedPayload"),
        ("ThreadCreatedPayload", "ThreadCreatedPayload"),
        ("ThreadArchivedPayload", "ThreadArchivedPayload"),
        ("ThreadMetaUpdatedPayload", "ThreadMetaUpdatedPayload"),
        ("ThreadRuntimeModeSetPayload", "ThreadRuntimeModeSetPayload"),
        (
            "ThreadInteractionModeSetPayload",
            "ThreadInteractionModeSetPayload",
        ),
        ("ThreadDeletedPayload", "ThreadDeletedPayload"),
        ("ThreadUnarchivedPayload", "ThreadUnarchivedPayload"),
        ("MessageSentPayloadSchema", "ThreadMessageSentPayload"),
        (
            "ThreadProposedPlanUpsertedPayload",
            "ThreadProposedPlanUpsertedPayload",
        ),
        ("ThreadSessionSetPayload", "ThreadSessionSetPayload"),
        (
            "ThreadTurnDiffCompletedPayload",
            "ThreadTurnDiffCompletedPayload",
        ),
        ("ThreadRevertedPayload", "ThreadRevertedPayload"),
        (
            "ThreadActivityAppendedPayload",
            "ThreadActivityAppendedPayload",
        ),
        (
            "ThreadTurnStartRequestedPayload",
            "ThreadTurnStartRequestedPayload",
        ),
        (
            "ThreadTurnInterruptRequestedPayload",
            "ThreadTurnInterruptRequestedPayload",
        ),
        (
            "ThreadApprovalResponseRequestedPayload",
            "ThreadApprovalResponseRequestedPayload",
        ),
        (
            "ThreadCheckpointRevertRequestedPayload",
            "ThreadCheckpointRevertRequestedPayload",
        ),
        (
            "ThreadSessionStopRequestedPayload",
            "ThreadSessionStopRequestedPayload",
        ),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientImageAttachmentDataUrl {
    pub name: String,
    pub data_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedAttachmentPersistPlan {
    pub attachment: ChatAttachment,
    pub relative_path: String,
    pub source_base64: String,
    pub byte_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizerError {
    pub message: String,
}

pub fn normalize_turn_start_attachment_plan(
    thread_id: &str,
    attachment: &ClientImageAttachmentDataUrl,
) -> Result<NormalizedAttachmentPersistPlan, NormalizerError> {
    let parsed = parse_base64_data_url(&attachment.data_url).ok_or_else(|| NormalizerError {
        message: format!(
            "Invalid image attachment payload for '{}'.",
            attachment.name
        ),
    })?;
    if !parsed.mime_type.starts_with("image/") {
        return Err(NormalizerError {
            message: format!(
                "Invalid image attachment payload for '{}'.",
                attachment.name
            ),
        });
    }
    let byte_len = decoded_base64_byte_len(&parsed.base64).ok_or_else(|| NormalizerError {
        message: format!(
            "Image attachment '{}' is empty or too large.",
            attachment.name
        ),
    })?;
    if byte_len == 0 || byte_len > PROVIDER_SEND_TURN_MAX_IMAGE_BYTES {
        return Err(NormalizerError {
            message: format!(
                "Image attachment '{}' is empty or too large.",
                attachment.name
            ),
        });
    }
    let attachment_id = create_attachment_id(thread_id).ok_or_else(|| NormalizerError {
        message: "Failed to create a safe attachment id.".to_string(),
    })?;
    let normalized = ChatAttachment::Image(ChatImageAttachment {
        id: attachment_id,
        name: attachment.name.clone(),
        mime_type: parsed.mime_type,
        size_bytes: byte_len as u64,
        preview_url: None,
    });
    let relative_path = attachment_relative_path(&normalized);
    Ok(NormalizedAttachmentPersistPlan {
        attachment: normalized,
        relative_path,
        source_base64: parsed.base64,
        byte_len,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizeDispatchCommandPlan {
    ProjectCreate {
        workspace_root: String,
        create_workspace_root_if_missing: bool,
    },
    ProjectMetaUpdate {
        workspace_root: String,
    },
    ThreadTurnStart {
        attachments: Vec<NormalizedAttachmentPersistPlan>,
    },
    Passthrough {
        command_type: String,
    },
}

pub fn normalize_dispatch_command_plan(
    command_type: &str,
    workspace_root: Option<&str>,
    create_workspace_root_if_missing: Option<bool>,
    thread_id: Option<&str>,
    attachments: &[ClientImageAttachmentDataUrl],
) -> Result<NormalizeDispatchCommandPlan, NormalizerError> {
    match command_type {
        "project.create" => Ok(NormalizeDispatchCommandPlan::ProjectCreate {
            workspace_root: workspace_root.unwrap_or_default().to_string(),
            create_workspace_root_if_missing: create_workspace_root_if_missing.unwrap_or(false),
        }),
        "project.meta.update" if workspace_root.is_some() => {
            Ok(NormalizeDispatchCommandPlan::ProjectMetaUpdate {
                workspace_root: workspace_root.unwrap_or_default().to_string(),
            })
        }
        "thread.turn.start" => Ok(NormalizeDispatchCommandPlan::ThreadTurnStart {
            attachments: attachments
                .iter()
                .map(|attachment| {
                    normalize_turn_start_attachment_plan(thread_id.unwrap_or_default(), attachment)
                })
                .collect::<Result<Vec<_>, _>>()?,
        }),
        _ => Ok(NormalizeDispatchCommandPlan::Passthrough {
            command_type: command_type.to_string(),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestrationHttpRouteKind {
    Snapshot,
    Dispatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationHttpRoutePlan {
    pub kind: OrchestrationHttpRouteKind,
    pub method: &'static str,
    pub path: &'static str,
    pub owner_session_required: bool,
    pub success_status: u16,
    pub invalid_request_status: Option<u16>,
}

pub fn orchestration_http_route_plans() -> Vec<OrchestrationHttpRoutePlan> {
    vec![
        OrchestrationHttpRoutePlan {
            kind: OrchestrationHttpRouteKind::Snapshot,
            method: "GET",
            path: "/api/orchestration/snapshot",
            owner_session_required: true,
            success_status: 200,
            invalid_request_status: None,
        },
        OrchestrationHttpRoutePlan {
            kind: OrchestrationHttpRouteKind::Dispatch,
            method: "POST",
            path: "/api/orchestration/dispatch",
            owner_session_required: true,
            success_status: 200,
            invalid_request_status: Some(400),
        },
    ]
}

pub fn orchestration_http_error_status(error_tag: &str) -> u16 {
    match error_tag {
        "OrchestrationGetSnapshotError" => 500,
        "OrchestrationDispatchCommandError" => 400,
        _ => 500,
    }
}

fn decoded_base64_byte_len(base64: &str) -> Option<usize> {
    if base64.is_empty() || base64.len() % 4 == 1 {
        return None;
    }
    let padding = base64
        .as_bytes()
        .iter()
        .rev()
        .take_while(|byte| **byte == b'=')
        .count()
        .min(2);
    Some((base64.len() / 4) * 3 + ((base64.len() % 4) * 3) / 4 - padding)
}

fn base64url_no_pad(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | b2 as u32;
        output.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        }
        if chunk.len() > 2 {
            output.push(TABLE[(n & 0x3f) as usize] as char);
        }
    }
    output
}

pub fn provider_service_request_for_intent(
    intent: &ProviderCommandIntent,
) -> ProviderServiceRequest {
    match intent {
        ProviderCommandIntent::RuntimeModeSet {
            thread_id,
            runtime_mode,
        } => ProviderServiceRequest::EnsureSessionForRuntimeMode {
            thread_id: thread_id.clone(),
            runtime_mode: *runtime_mode,
        },
        ProviderCommandIntent::TurnStart {
            thread_id,
            message_id,
            runtime_mode,
            interaction_mode,
        } => ProviderServiceRequest::BuildAndSendTurn {
            thread_id: thread_id.clone(),
            message_id: message_id.clone(),
            runtime_mode: *runtime_mode,
            interaction_mode: *interaction_mode,
        },
        ProviderCommandIntent::TurnInterrupt { thread_id, turn_id } => {
            ProviderServiceRequest::InterruptTurn(ProviderInterruptTurnInput {
                thread_id: thread_id.clone(),
                turn_id: turn_id.clone(),
            })
        }
        ProviderCommandIntent::ApprovalRespond {
            thread_id,
            request_id,
            decision,
        } => ProviderServiceRequest::RespondToRequest(ProviderRespondToRequestInput {
            thread_id: thread_id.clone(),
            request_id: request_id.clone(),
            decision: decision.clone(),
        }),
        ProviderCommandIntent::UserInputRespond {
            thread_id,
            request_id,
            answers,
        } => ProviderServiceRequest::RespondToUserInput(ProviderRespondToUserInputInput {
            thread_id: thread_id.clone(),
            request_id: request_id.clone(),
            answers: answers.clone(),
        }),
        ProviderCommandIntent::SessionStop { thread_id } => {
            ProviderServiceRequest::StopSession(ProviderStopSessionInput {
                thread_id: thread_id.clone(),
            })
        }
    }
}

pub fn thread_deletion_cleanup_requests(
    thread_id: &str,
    actions: &[ThreadDeletionCleanupAction],
) -> Vec<ThreadDeletionCleanupRequest> {
    actions
        .iter()
        .map(|action| match action {
            ThreadDeletionCleanupAction::StopProviderSession => {
                ThreadDeletionCleanupRequest::StopProviderSession(ProviderStopSessionInput {
                    thread_id: thread_id.to_string(),
                })
            }
            ThreadDeletionCleanupAction::CloseThreadTerminalsAndDeleteHistory => {
                ThreadDeletionCleanupRequest::CloseThreadTerminalsAndDeleteHistory {
                    thread_id: thread_id.to_string(),
                }
            }
        })
        .collect()
}

pub fn provider_error_label(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_string()
}

pub fn provider_error_label_from_instance_hint(
    instance_id: Option<&str>,
    model_selection_instance_id: Option<&str>,
    session_provider: Option<&str>,
) -> String {
    provider_error_label(
        instance_id
            .or(model_selection_instance_id)
            .or(session_provider),
    )
}

pub fn provider_command_intent_for_event(
    event: &PlannedOrchestrationEvent,
) -> Option<ProviderCommandIntent> {
    match event.event_type.as_str() {
        "thread.runtime-mode-set" => Some(ProviderCommandIntent::RuntimeModeSet {
            thread_id: event.payload.get("threadId")?.as_str()?.to_string(),
            runtime_mode: runtime_mode_from_t3(event.payload.get("runtimeMode")?.as_str()?)?,
        }),
        "thread.turn-start-requested" => Some(ProviderCommandIntent::TurnStart {
            thread_id: event.payload.get("threadId")?.as_str()?.to_string(),
            message_id: event.payload.get("messageId")?.as_str()?.to_string(),
            runtime_mode: runtime_mode_from_t3(event.payload.get("runtimeMode")?.as_str()?)?,
            interaction_mode: interaction_mode_from_t3(
                event.payload.get("interactionMode")?.as_str()?,
            )?,
        }),
        "thread.turn-interrupt-requested" => Some(ProviderCommandIntent::TurnInterrupt {
            thread_id: event.payload.get("threadId")?.as_str()?.to_string(),
            turn_id: event
                .payload
                .get("turnId")
                .and_then(Value::as_str)
                .map(str::to_string),
        }),
        "thread.approval-response-requested" => Some(ProviderCommandIntent::ApprovalRespond {
            thread_id: event.payload.get("threadId")?.as_str()?.to_string(),
            request_id: event.payload.get("requestId")?.as_str()?.to_string(),
            decision: event.payload.get("decision")?.as_str()?.to_string(),
        }),
        "thread.user-input-response-requested" => Some(ProviderCommandIntent::UserInputRespond {
            thread_id: event.payload.get("threadId")?.as_str()?.to_string(),
            request_id: event.payload.get("requestId")?.as_str()?.to_string(),
            answers: event.payload.get("answers")?.clone(),
        }),
        "thread.session-stop-requested" => Some(ProviderCommandIntent::SessionStop {
            thread_id: event.payload.get("threadId")?.as_str()?.to_string(),
        }),
        _ => None,
    }
}

pub fn thread_deletion_cleanup_actions_for_event(
    event: &PlannedOrchestrationEvent,
) -> Option<(String, Vec<ThreadDeletionCleanupAction>)> {
    if event.event_type != "thread.deleted" {
        return None;
    }
    Some((
        event.payload.get("threadId")?.as_str()?.to_string(),
        vec![
            ThreadDeletionCleanupAction::StopProviderSession,
            ThreadDeletionCleanupAction::CloseThreadTerminalsAndDeleteHistory,
        ],
    ))
}

pub fn decide_orchestration_command(
    command: &OrchestrationCommand,
    read_model: &OrchestrationReadModel,
    now_iso: &str,
) -> Result<Vec<PlannedOrchestrationEvent>, OrchestrationCommandInvariantError> {
    match command {
        OrchestrationCommand::ProjectCreate {
            command_id,
            project_id,
            title,
            workspace_root,
            default_model_selection,
            created_at,
        } => {
            require_project_absent(read_model, "project.create", project_id)?;
            Ok(vec![event(
                command_id,
                "project",
                project_id,
                created_at,
                "project.created",
                json!({
                    "projectId": project_id,
                    "title": title,
                    "workspaceRoot": workspace_root,
                    "defaultModelSelection": default_model_selection.clone().unwrap_or(Value::Null),
                    "scripts": [],
                    "createdAt": created_at,
                    "updatedAt": created_at,
                }),
                json!({}),
                None,
                0,
            )])
        }
        OrchestrationCommand::ProjectMetaUpdate {
            command_id,
            project_id,
            title,
            workspace_root,
            default_model_selection,
            scripts,
        } => {
            require_project(read_model, "project.meta.update", project_id)?;
            let mut payload = json!({
                "projectId": project_id,
                "updatedAt": now_iso,
            });
            insert_optional(
                &mut payload,
                "title",
                title.as_ref().map(|value| json!(value)),
            );
            insert_optional(
                &mut payload,
                "workspaceRoot",
                workspace_root.as_ref().map(|value| json!(value)),
            );
            insert_optional(
                &mut payload,
                "defaultModelSelection",
                default_model_selection.clone(),
            );
            insert_optional(
                &mut payload,
                "scripts",
                scripts.as_ref().map(project_scripts_to_json),
            );
            Ok(vec![event(
                command_id,
                "project",
                project_id,
                now_iso,
                "project.meta-updated",
                payload,
                json!({}),
                None,
                0,
            )])
        }
        OrchestrationCommand::ProjectDelete {
            command_id,
            project_id,
            force,
        } => {
            require_project(read_model, "project.delete", project_id)?;
            let active_threads = read_model
                .threads
                .iter()
                .filter(|thread| thread.project_id == *project_id && thread.deleted_at.is_none())
                .collect::<Vec<_>>();
            if !active_threads.is_empty() && !force {
                return Err(invariant(
                    "project.delete",
                    format!(
                        "Project '{project_id}' is not empty and cannot be deleted without force=true."
                    ),
                ));
            }
            let mut events = active_threads
                .iter()
                .enumerate()
                .map(|(index, thread)| {
                    event(
                        command_id,
                        "thread",
                        &thread.thread_id,
                        now_iso,
                        "thread.deleted",
                        json!({
                            "threadId": thread.thread_id,
                            "deletedAt": now_iso,
                        }),
                        json!({}),
                        None,
                        index,
                    )
                })
                .collect::<Vec<_>>();
            events.push(event(
                command_id,
                "project",
                project_id,
                now_iso,
                "project.deleted",
                json!({
                    "projectId": project_id,
                    "deletedAt": now_iso,
                }),
                json!({}),
                None,
                events.len(),
            ));
            Ok(events)
        }
        OrchestrationCommand::ThreadCreate {
            command_id,
            thread_id,
            project_id,
            title,
            model_selection,
            runtime_mode,
            interaction_mode,
            branch,
            worktree_path,
            created_at,
        } => {
            require_project(read_model, "thread.create", project_id)?;
            require_thread_absent(read_model, "thread.create", thread_id)?;
            Ok(vec![event(
                command_id,
                "thread",
                thread_id,
                created_at,
                "thread.created",
                json!({
                    "threadId": thread_id,
                    "projectId": project_id,
                    "title": title,
                    "modelSelection": model_selection,
                    "runtimeMode": runtime_mode_to_t3(*runtime_mode),
                    "interactionMode": interaction_mode_to_t3(*interaction_mode),
                    "branch": branch,
                    "worktreePath": worktree_path,
                    "createdAt": created_at,
                    "updatedAt": created_at,
                }),
                json!({}),
                None,
                0,
            )])
        }
        OrchestrationCommand::ThreadDelete {
            command_id,
            thread_id,
        } => {
            require_thread(read_model, "thread.delete", thread_id)?;
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                now_iso,
                "thread.deleted",
                json!({ "threadId": thread_id, "deletedAt": now_iso }),
            )])
        }
        OrchestrationCommand::ThreadArchive {
            command_id,
            thread_id,
        } => {
            let thread = require_thread(read_model, "thread.archive", thread_id)?;
            if thread.archived_at.is_some() {
                return Err(invariant(
                    "thread.archive",
                    format!("Thread '{thread_id}' is already archived."),
                ));
            }
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                now_iso,
                "thread.archived",
                json!({ "threadId": thread_id, "archivedAt": now_iso, "updatedAt": now_iso }),
            )])
        }
        OrchestrationCommand::ThreadUnarchive {
            command_id,
            thread_id,
        } => {
            let thread = require_thread(read_model, "thread.unarchive", thread_id)?;
            if thread.archived_at.is_none() {
                return Err(invariant(
                    "thread.unarchive",
                    format!("Thread '{thread_id}' is not archived."),
                ));
            }
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                now_iso,
                "thread.unarchived",
                json!({ "threadId": thread_id, "updatedAt": now_iso }),
            )])
        }
        OrchestrationCommand::ThreadMetaUpdate {
            command_id,
            thread_id,
            title,
            model_selection,
            branch,
            worktree_path,
        } => {
            require_thread(read_model, "thread.meta.update", thread_id)?;
            let mut payload = json!({ "threadId": thread_id, "updatedAt": now_iso });
            insert_optional(
                &mut payload,
                "title",
                title.as_ref().map(|value| json!(value)),
            );
            insert_optional(&mut payload, "modelSelection", model_selection.clone());
            insert_optional(
                &mut payload,
                "branch",
                branch.as_ref().map(|value| json!(value)),
            );
            insert_optional(
                &mut payload,
                "worktreePath",
                worktree_path.as_ref().map(|value| json!(value)),
            );
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                now_iso,
                "thread.meta-updated",
                payload,
            )])
        }
        OrchestrationCommand::ThreadRuntimeModeSet {
            command_id,
            thread_id,
            runtime_mode,
        } => {
            require_thread(read_model, "thread.runtime-mode.set", thread_id)?;
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                now_iso,
                "thread.runtime-mode-set",
                json!({
                    "threadId": thread_id,
                    "runtimeMode": runtime_mode_to_t3(*runtime_mode),
                    "updatedAt": now_iso,
                }),
            )])
        }
        OrchestrationCommand::ThreadInteractionModeSet {
            command_id,
            thread_id,
            interaction_mode,
        } => {
            require_thread(read_model, "thread.interaction-mode.set", thread_id)?;
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                now_iso,
                "thread.interaction-mode-set",
                json!({
                    "threadId": thread_id,
                    "interactionMode": interaction_mode_to_t3(*interaction_mode),
                    "updatedAt": now_iso,
                }),
            )])
        }
        OrchestrationCommand::ThreadTurnStart {
            command_id,
            thread_id,
            message_id,
            text,
            attachments,
            model_selection,
            title_seed,
            source_proposed_plan,
            created_at,
        } => {
            let target_thread = require_thread(read_model, "thread.turn.start", thread_id)?;
            if let Some(source) = source_proposed_plan {
                let source_thread =
                    require_thread(read_model, "thread.turn.start", &source.thread_id)?;
                let plan_exists = read_model.proposed_plans.iter().any(|entry| {
                    entry.thread_id == source.thread_id && entry.plan.id == source.plan_id
                });
                if !plan_exists {
                    return Err(invariant(
                        "thread.turn.start",
                        format!(
                            "Proposed plan '{}' does not exist on thread '{}'.",
                            source.plan_id, source.thread_id
                        ),
                    ));
                }
                if source_thread.project_id != target_thread.project_id {
                    return Err(invariant(
                        "thread.turn.start",
                        format!(
                            "Proposed plan '{}' belongs to thread '{}' in a different project.",
                            source.plan_id, source.thread_id
                        ),
                    ));
                }
            }
            let user_event = event(
                command_id,
                "thread",
                thread_id,
                created_at,
                "thread.message-sent",
                json!({
                    "threadId": thread_id,
                    "messageId": message_id,
                    "role": "user",
                    "text": text,
                    "attachments": chat_attachments_to_json(attachments),
                    "turnId": null,
                    "streaming": false,
                    "createdAt": created_at,
                    "updatedAt": created_at,
                }),
                json!({}),
                None,
                0,
            );
            let mut turn_payload = json!({
                "threadId": thread_id,
                "messageId": message_id,
                "runtimeMode": runtime_mode_to_t3(target_thread.runtime_mode),
                "interactionMode": interaction_mode_to_t3(target_thread.interaction_mode),
                "createdAt": created_at,
            });
            insert_optional(&mut turn_payload, "modelSelection", model_selection.clone());
            insert_optional(
                &mut turn_payload,
                "titleSeed",
                title_seed.as_ref().map(|value| json!(value)),
            );
            insert_optional(
                &mut turn_payload,
                "sourceProposedPlan",
                source_proposed_plan.as_ref().map(
                    |source| json!({ "threadId": source.thread_id, "planId": source.plan_id }),
                ),
            );
            let turn_event = event(
                command_id,
                "thread",
                thread_id,
                created_at,
                "thread.turn-start-requested",
                turn_payload,
                json!({}),
                Some(user_event.event_id.clone()),
                1,
            );
            Ok(vec![user_event, turn_event])
        }
        OrchestrationCommand::ThreadTurnInterrupt {
            command_id,
            thread_id,
            turn_id,
            created_at,
        } => {
            require_thread(read_model, "thread.turn.interrupt", thread_id)?;
            let mut payload = json!({ "threadId": thread_id, "createdAt": created_at });
            insert_optional(
                &mut payload,
                "turnId",
                turn_id.as_ref().map(|value| json!(value)),
            );
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                created_at,
                "thread.turn-interrupt-requested",
                payload,
            )])
        }
        OrchestrationCommand::ThreadApprovalRespond {
            command_id,
            thread_id,
            request_id,
            decision,
            created_at,
        } => {
            require_thread(read_model, "thread.approval.respond", thread_id)?;
            Ok(vec![event(
                command_id,
                "thread",
                thread_id,
                created_at,
                "thread.approval-response-requested",
                json!({
                    "threadId": thread_id,
                    "requestId": request_id,
                    "decision": decision,
                    "createdAt": created_at,
                }),
                json!({ "requestId": request_id }),
                None,
                0,
            )])
        }
        OrchestrationCommand::ThreadUserInputRespond {
            command_id,
            thread_id,
            request_id,
            answers,
            created_at,
        } => {
            require_thread(read_model, "thread.user-input.respond", thread_id)?;
            Ok(vec![event(
                command_id,
                "thread",
                thread_id,
                created_at,
                "thread.user-input-response-requested",
                json!({
                    "threadId": thread_id,
                    "requestId": request_id,
                    "answers": answers,
                    "createdAt": created_at,
                }),
                json!({ "requestId": request_id }),
                None,
                0,
            )])
        }
        OrchestrationCommand::ThreadCheckpointRevert {
            command_id,
            thread_id,
            turn_count,
            created_at,
        } => {
            require_thread(read_model, "thread.checkpoint.revert", thread_id)?;
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                created_at,
                "thread.checkpoint-revert-requested",
                json!({ "threadId": thread_id, "turnCount": turn_count, "createdAt": created_at }),
            )])
        }
        OrchestrationCommand::ThreadSessionStop {
            command_id,
            thread_id,
            created_at,
        } => {
            require_thread(read_model, "thread.session.stop", thread_id)?;
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                created_at,
                "thread.session-stop-requested",
                json!({ "threadId": thread_id, "createdAt": created_at }),
            )])
        }
        OrchestrationCommand::ThreadSessionSet {
            command_id,
            thread_id,
            session,
            created_at,
        } => {
            require_thread(read_model, "thread.session.set", thread_id)?;
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                created_at,
                "thread.session-set",
                json!({ "threadId": thread_id, "session": session }),
            )])
        }
        OrchestrationCommand::ThreadMessageAssistantDelta {
            command_id,
            thread_id,
            message_id,
            delta,
            turn_id,
            created_at,
        } => {
            require_thread(read_model, "thread.message.assistant.delta", thread_id)?;
            Ok(vec![assistant_message_event(
                command_id, thread_id, message_id, delta, turn_id, created_at, true,
            )])
        }
        OrchestrationCommand::ThreadMessageAssistantComplete {
            command_id,
            thread_id,
            message_id,
            turn_id,
            created_at,
        } => {
            require_thread(read_model, "thread.message.assistant.complete", thread_id)?;
            Ok(vec![assistant_message_event(
                command_id, thread_id, message_id, "", turn_id, created_at, false,
            )])
        }
        OrchestrationCommand::ThreadProposedPlanUpsert {
            command_id,
            thread_id,
            proposed_plan,
            created_at,
        } => {
            require_thread(read_model, "thread.proposed-plan.upsert", thread_id)?;
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                created_at,
                "thread.proposed-plan-upserted",
                json!({
                    "threadId": thread_id,
                    "proposedPlan": proposed_plan_to_json(proposed_plan),
                }),
            )])
        }
        OrchestrationCommand::ThreadTurnDiffComplete {
            command_id,
            thread_id,
            turn_id,
            checkpoint_turn_count,
            checkpoint_ref,
            status,
            files,
            assistant_message_id,
            completed_at,
            created_at,
        } => {
            require_thread(read_model, "thread.turn.diff.complete", thread_id)?;
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                created_at,
                "thread.turn-diff-completed",
                json!({
                    "threadId": thread_id,
                    "turnId": turn_id,
                    "checkpointTurnCount": checkpoint_turn_count,
                    "checkpointRef": checkpoint_ref,
                    "status": status,
                    "files": turn_diff_files_to_json(files),
                    "assistantMessageId": assistant_message_id,
                    "completedAt": completed_at,
                }),
            )])
        }
        OrchestrationCommand::ThreadRevertComplete {
            command_id,
            thread_id,
            turn_count,
            created_at,
        } => {
            require_thread(read_model, "thread.revert.complete", thread_id)?;
            Ok(vec![thread_simple_event(
                command_id,
                thread_id,
                created_at,
                "thread.reverted",
                json!({ "threadId": thread_id, "turnCount": turn_count }),
            )])
        }
        OrchestrationCommand::ThreadActivityAppend {
            command_id,
            thread_id,
            activity,
            created_at,
        } => {
            require_thread(read_model, "thread.activity.append", thread_id)?;
            let metadata = activity
                .get("payload")
                .and_then(|payload| payload.get("requestId"))
                .and_then(Value::as_str)
                .map(|request_id| json!({ "requestId": request_id }))
                .unwrap_or_else(|| json!({}));
            Ok(vec![event(
                command_id,
                "thread",
                thread_id,
                created_at,
                "thread.activity-appended",
                json!({ "threadId": thread_id, "activity": activity }),
                metadata,
                None,
                0,
            )])
        }
    }
}

fn event(
    command_id: &str,
    aggregate_kind: &str,
    aggregate_id: &str,
    occurred_at: &str,
    event_type: &str,
    payload: Value,
    metadata: Value,
    causation_event_id: Option<String>,
    ordinal: usize,
) -> PlannedOrchestrationEvent {
    PlannedOrchestrationEvent {
        event_id: format!("{command_id}:{event_type}:{ordinal}"),
        aggregate_kind: aggregate_kind.to_string(),
        aggregate_id: aggregate_id.to_string(),
        event_type: event_type.to_string(),
        occurred_at: occurred_at.to_string(),
        command_id: Some(command_id.to_string()),
        causation_event_id,
        correlation_id: Some(command_id.to_string()),
        payload,
        metadata,
    }
}

fn thread_simple_event(
    command_id: &str,
    thread_id: &str,
    occurred_at: &str,
    event_type: &str,
    payload: Value,
) -> PlannedOrchestrationEvent {
    event(
        command_id,
        "thread",
        thread_id,
        occurred_at,
        event_type,
        payload,
        json!({}),
        None,
        0,
    )
}

fn assistant_message_event(
    command_id: &str,
    thread_id: &str,
    message_id: &str,
    text: &str,
    turn_id: &Option<String>,
    created_at: &str,
    streaming: bool,
) -> PlannedOrchestrationEvent {
    thread_simple_event(
        command_id,
        thread_id,
        created_at,
        "thread.message-sent",
        json!({
            "threadId": thread_id,
            "messageId": message_id,
            "role": "assistant",
            "text": text,
            "turnId": turn_id,
            "streaming": streaming,
            "createdAt": created_at,
            "updatedAt": created_at,
        }),
    )
}

fn insert_optional(payload: &mut Value, key: &str, value: Option<Value>) {
    if let (Some(object), Some(value)) = (payload.as_object_mut(), value) {
        object.insert(key.to_string(), value);
    }
}

fn invariant(
    command_type: impl Into<String>,
    detail: impl Into<String>,
) -> OrchestrationCommandInvariantError {
    OrchestrationCommandInvariantError {
        command_type: command_type.into(),
        detail: detail.into(),
    }
}

fn require_project<'a>(
    read_model: &'a OrchestrationReadModel,
    command_type: &str,
    project_id: &str,
) -> Result<&'a ProjectionProjectRow, OrchestrationCommandInvariantError> {
    read_model
        .projects
        .iter()
        .find(|project| project.project_id == project_id && project.deleted_at.is_none())
        .ok_or_else(|| {
            invariant(
                command_type,
                format!("Project '{project_id}' does not exist."),
            )
        })
}

fn require_project_absent(
    read_model: &OrchestrationReadModel,
    command_type: &str,
    project_id: &str,
) -> Result<(), OrchestrationCommandInvariantError> {
    if read_model
        .projects
        .iter()
        .any(|project| project.project_id == project_id && project.deleted_at.is_none())
    {
        Err(invariant(
            command_type,
            format!("Project '{project_id}' already exists."),
        ))
    } else {
        Ok(())
    }
}

fn require_thread<'a>(
    read_model: &'a OrchestrationReadModel,
    command_type: &str,
    thread_id: &str,
) -> Result<&'a ProjectionThreadRow, OrchestrationCommandInvariantError> {
    read_model
        .threads
        .iter()
        .find(|thread| thread.thread_id == thread_id && thread.deleted_at.is_none())
        .ok_or_else(|| {
            invariant(
                command_type,
                format!("Thread '{thread_id}' does not exist."),
            )
        })
}

fn require_thread_absent(
    read_model: &OrchestrationReadModel,
    command_type: &str,
    thread_id: &str,
) -> Result<(), OrchestrationCommandInvariantError> {
    if read_model
        .threads
        .iter()
        .any(|thread| thread.thread_id == thread_id && thread.deleted_at.is_none())
    {
        Err(invariant(
            command_type,
            format!("Thread '{thread_id}' already exists."),
        ))
    } else {
        Ok(())
    }
}

fn runtime_mode_to_t3(mode: RuntimeMode) -> &'static str {
    match mode {
        RuntimeMode::ApprovalRequired => "approval-required",
        RuntimeMode::AutoAcceptEdits => "auto-accept-edits",
        RuntimeMode::FullAccess => "full-access",
    }
}

fn interaction_mode_to_t3(mode: ProviderInteractionMode) -> &'static str {
    match mode {
        ProviderInteractionMode::Default => "default",
        ProviderInteractionMode::Plan => "plan",
    }
}

fn runtime_mode_from_t3(value: &str) -> Option<RuntimeMode> {
    match value {
        "approval-required" => Some(RuntimeMode::ApprovalRequired),
        "auto-accept-edits" => Some(RuntimeMode::AutoAcceptEdits),
        "full-access" => Some(RuntimeMode::FullAccess),
        _ => None,
    }
}

fn interaction_mode_from_t3(value: &str) -> Option<ProviderInteractionMode> {
    match value {
        "default" => Some(ProviderInteractionMode::Default),
        "plan" => Some(ProviderInteractionMode::Plan),
        _ => None,
    }
}

fn runtime_activity(
    event: &ProviderRuntimeEventInput,
    tone: &str,
    kind: &str,
    summary: &str,
    payload: Value,
) -> ProviderRuntimeActivity {
    ProviderRuntimeActivity {
        id: event.event_id.clone(),
        created_at: event.created_at.clone(),
        tone: tone.to_string(),
        kind: kind.to_string(),
        summary: summary.to_string(),
        payload,
        turn_id: event.turn_id.clone(),
        sequence: event.session_sequence,
    }
}

fn project_scripts_to_json(scripts: &Vec<ProjectScript>) -> Value {
    Value::Array(
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
    )
}

fn project_script_icon_to_t3(icon: crate::ProjectScriptIcon) -> &'static str {
    match icon {
        crate::ProjectScriptIcon::Play => "play",
        crate::ProjectScriptIcon::Test => "test",
        crate::ProjectScriptIcon::Lint => "lint",
        crate::ProjectScriptIcon::Configure => "configure",
        crate::ProjectScriptIcon::Build => "build",
        crate::ProjectScriptIcon::Debug => "debug",
    }
}

fn chat_attachments_to_json(attachments: &[ChatAttachment]) -> Value {
    Value::Array(
        attachments
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
            .collect(),
    )
}

fn proposed_plan_to_json(plan: &ProposedPlan) -> Value {
    json!({
        "id": plan.id,
        "turnId": plan.turn_id,
        "planMarkdown": plan.plan_markdown,
        "implementedAt": plan.implemented_at,
        "implementationThreadId": plan.implementation_thread_id,
        "createdAt": plan.created_at,
        "updatedAt": plan.updated_at,
    })
}

fn turn_diff_files_to_json(files: &[TurnDiffFileChange]) -> Value {
    Value::Array(
        files
            .iter()
            .map(|file| {
                json!({
                    "path": file.path,
                    "kind": file.kind,
                    "additions": file.additions,
                    "deletions": file.deletions,
                })
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(project_id: &str) -> ProjectionProjectRow {
        ProjectionProjectRow {
            project_id: project_id.to_string(),
            title: format!("Project {project_id}"),
            workspace_root: format!("/repo/{project_id}"),
            scripts: Vec::new(),
            created_at: "2026-03-04T12:00:00.000Z".to_string(),
            updated_at: "2026-03-04T12:00:00.000Z".to_string(),
            deleted_at: None,
        }
    }

    fn thread(thread_id: &str, project_id: &str) -> ProjectionThreadRow {
        ProjectionThreadRow {
            thread_id: thread_id.to_string(),
            project_id: project_id.to_string(),
            title: format!("Thread {thread_id}"),
            runtime_mode: RuntimeMode::FullAccess,
            interaction_mode: ProviderInteractionMode::Plan,
            branch: Some("main".to_string()),
            worktree_path: Some(format!("/repo/{project_id}")),
            created_at: "2026-03-04T12:00:00.000Z".to_string(),
            updated_at: "2026-03-04T12:00:00.000Z".to_string(),
            archived_at: None,
            latest_user_message_at: None,
            pending_approval_count: 0,
            pending_user_input_count: 0,
            has_actionable_proposed_plan: false,
            deleted_at: None,
        }
    }

    fn checkpoint(turn_count: u32) -> ProjectionCheckpointSummary {
        ProjectionCheckpointSummary {
            turn_id: format!("turn-{turn_count}"),
            checkpoint_turn_count: turn_count,
            checkpoint_ref: checkpoint_ref_for_thread_turn("thread-1", turn_count),
            status: "ready".to_string(),
            files: Vec::new(),
            assistant_message_id: None,
            completed_at: "2026-01-01T00:00:00.000Z".to_string(),
        }
    }

    #[test]
    fn orchestration_recovery_coordinator_matches_upstream_state_machine() {
        let mut coordinator = OrchestrationRecoveryCoordinator::new();
        assert!(coordinator.begin_snapshot_recovery(OrchestrationRecoveryReason::Bootstrap));
        assert_eq!(
            coordinator.classify_domain_event(4),
            DomainEventClassification::Defer
        );
        assert!(coordinator.complete_snapshot_recovery(2));
        assert_eq!(
            coordinator.state(),
            OrchestrationRecoveryState {
                latest_sequence: 2,
                highest_observed_sequence: 4,
                bootstrapped: true,
                pending_replay: false,
                in_flight: None,
            }
        );

        let mut gap = OrchestrationRecoveryCoordinator::new();
        gap.begin_snapshot_recovery(OrchestrationRecoveryReason::Bootstrap);
        gap.complete_snapshot_recovery(3);
        assert_eq!(
            gap.classify_domain_event(5),
            DomainEventClassification::Recover
        );
        assert!(gap.begin_replay_recovery(OrchestrationRecoveryReason::SequenceGap));
        assert_eq!(
            gap.state().in_flight,
            Some(OrchestrationRecoveryPhase {
                kind: OrchestrationRecoveryPhaseKind::Replay,
                reason: OrchestrationRecoveryReason::SequenceGap,
            })
        );

        let mut live = OrchestrationRecoveryCoordinator::new();
        live.begin_snapshot_recovery(OrchestrationRecoveryReason::Bootstrap);
        live.complete_snapshot_recovery(3);
        assert_eq!(
            live.classify_domain_event(4),
            DomainEventClassification::Apply
        );
        assert_eq!(live.mark_event_batch_applied(&[4]), vec![4]);
        assert_eq!(live.state().latest_sequence, 4);

        gap.classify_domain_event(7);
        assert_eq!(gap.mark_event_batch_applied(&[4, 5, 6]), vec![4, 5, 6]);
        assert_eq!(
            gap.complete_replay_recovery(),
            ReplayRecoveryCompletion {
                replay_made_progress: true,
                should_replay: true,
            }
        );

        let mut no_progress = OrchestrationRecoveryCoordinator::new();
        no_progress.begin_snapshot_recovery(OrchestrationRecoveryReason::Bootstrap);
        no_progress.complete_snapshot_recovery(3);
        no_progress.classify_domain_event(5);
        no_progress.begin_replay_recovery(OrchestrationRecoveryReason::SequenceGap);
        assert_eq!(
            no_progress.complete_replay_recovery(),
            ReplayRecoveryCompletion {
                replay_made_progress: false,
                should_replay: true,
            }
        );

        let mut failed = OrchestrationRecoveryCoordinator::new();
        failed.begin_snapshot_recovery(OrchestrationRecoveryReason::Bootstrap);
        failed.complete_snapshot_recovery(3);
        failed.begin_replay_recovery(OrchestrationRecoveryReason::SequenceGap);
        failed.fail_replay_recovery();
        assert!(!failed.state().bootstrapped);
        assert!(failed.begin_snapshot_recovery(OrchestrationRecoveryReason::ReplayFailed));
    }

    #[test]
    fn orchestration_replay_retry_decision_matches_upstream_backoff() {
        assert_eq!(
            derive_replay_retry_decision(
                Some(ReplayRetryTracker {
                    attempts: 2,
                    latest_sequence: 3,
                    highest_observed_sequence: 5,
                }),
                ReplayRecoveryCompletion {
                    replay_made_progress: true,
                    should_replay: true,
                },
                OrchestrationRecoveryState {
                    latest_sequence: 5,
                    highest_observed_sequence: 5,
                    ..OrchestrationRecoveryState::default()
                },
                100,
                3,
            ),
            ReplayRetryDecision {
                should_retry: true,
                delay_ms: 0,
                tracker: None,
            }
        );

        let no_progress = ReplayRecoveryCompletion {
            replay_made_progress: false,
            should_replay: true,
        };
        let frontier = OrchestrationRecoveryState {
            latest_sequence: 3,
            highest_observed_sequence: 5,
            ..OrchestrationRecoveryState::default()
        };
        let first = derive_replay_retry_decision(None, no_progress, frontier, 100, 3);
        let second = derive_replay_retry_decision(first.tracker, no_progress, frontier, 100, 3);
        let third = derive_replay_retry_decision(second.tracker, no_progress, frontier, 100, 3);
        let fourth = derive_replay_retry_decision(third.tracker, no_progress, frontier, 100, 3);

        assert_eq!(first.delay_ms, 100);
        assert_eq!(first.tracker.unwrap().attempts, 1);
        assert_eq!(second.delay_ms, 200);
        assert_eq!(second.tracker.unwrap().attempts, 2);
        assert_eq!(third.delay_ms, 400);
        assert_eq!(third.tracker.unwrap().attempts, 3);
        assert_eq!(
            fourth,
            ReplayRetryDecision {
                should_retry: false,
                delay_ms: 0,
                tracker: None,
            }
        );

        assert_eq!(
            derive_replay_retry_decision(
                third.tracker,
                no_progress,
                OrchestrationRecoveryState {
                    latest_sequence: 3,
                    highest_observed_sequence: 6,
                    ..OrchestrationRecoveryState::default()
                },
                100,
                3,
            ),
            ReplayRetryDecision {
                should_retry: true,
                delay_ms: 100,
                tracker: Some(ReplayRetryTracker {
                    attempts: 1,
                    latest_sequence: 3,
                    highest_observed_sequence: 6,
                }),
            }
        );
    }

    #[test]
    fn ports_runtime_receipt_bus_and_checkpoint_reactor_contracts() {
        assert_eq!(
            runtime_receipt_bus_layer_plan(RuntimeReceiptBusLayerKind::Live),
            RuntimeReceiptBusLayerPlan {
                layer_kind: RuntimeReceiptBusLayerKind::Live,
                publish_retains_receipts: false,
                stream_events_for_test: "empty",
            }
        );
        assert_eq!(
            runtime_receipt_bus_layer_plan(RuntimeReceiptBusLayerKind::Test).stream_events_for_test,
            "pubsub"
        );

        let receipt = OrchestrationRuntimeReceipt::CheckpointDiffFinalized {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            checkpoint_turn_count: 1,
            checkpoint_ref: checkpoint_ref_for_thread_turn("thread-1", 1),
            status: CheckpointDiffStatus::Ready,
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
        };
        assert_eq!(receipt.receipt_type(), "checkpoint.diff.finalized");
        assert_eq!(
            checkpoint_ref_for_thread_turn("thread-1", 2),
            "refs/t3/checkpoints/dGhyZWFkLTE/turn/2"
        );
        assert_eq!(
            checkpoint_status_from_runtime(Some("failed")),
            CheckpointDiffStatus::Error
        );
        assert_eq!(
            checkpoint_status_from_runtime(Some("interrupted")),
            CheckpointDiffStatus::Missing
        );
        assert_eq!(
            checkpoint_status_from_runtime(Some("completed")),
            CheckpointDiffStatus::Ready
        );
        assert!(checkpoint_reactor_domain_event_is_enqueued(
            "thread.checkpoint-revert-requested"
        ));
        assert!(checkpoint_reactor_domain_event_is_enqueued(
            "thread.turn-diff-completed"
        ));
        assert!(!checkpoint_reactor_domain_event_is_enqueued(
            "thread.created"
        ));
        assert!(checkpoint_reactor_runtime_event_is_enqueued("turn.started"));
        assert!(checkpoint_reactor_runtime_event_is_enqueued(
            "turn.completed"
        ));
        assert!(!checkpoint_reactor_runtime_event_is_enqueued(
            "checkpoint.captured"
        ));
    }

    #[test]
    fn ports_checkpoint_reactor_cwd_baseline_completion_and_revert_plans() {
        let workspace = ThreadWorkspaceCwdInput {
            thread_project_id: "project-1".to_string(),
            thread_worktree_path: None,
            projects: vec![ThreadWorkspaceProject {
                project_id: "project-1".to_string(),
                workspace_root: "C:/repo/project-1".to_string(),
            }],
        };
        assert_eq!(
            resolve_thread_workspace_cwd(&workspace).as_deref(),
            Some("C:/repo/project-1")
        );
        assert_eq!(
            checkpoint_reactor_resolve_checkpoint_cwd(Some("C:/session"), Some("C:/thread"), true)
                .as_deref(),
            Some("C:/session")
        );
        assert_eq!(
            checkpoint_reactor_resolve_checkpoint_cwd(Some("C:/session"), Some("C:/thread"), false)
                .as_deref(),
            Some("C:/thread")
        );
        assert_eq!(
            checkpoint_reactor_baseline_capture_action("thread-1", 0, false),
            Some(CheckpointReactorAction::CaptureBaseline {
                checkpoint_ref: "refs/t3/checkpoints/dGhyZWFkLTE/turn/0".to_string(),
            })
        );
        assert_eq!(
            checkpoint_reactor_baseline_capture_action("thread-1", 0, true),
            None
        );
        assert_eq!(
            checkpoint_reactor_turn_completion_action("thread-1", 2, Some("cancelled"), false),
            Some(CheckpointReactorAction::CaptureTurnDiff {
                from_checkpoint_ref: "refs/t3/checkpoints/dGhyZWFkLTE/turn/1".to_string(),
                target_checkpoint_ref: "refs/t3/checkpoints/dGhyZWFkLTE/turn/2".to_string(),
                status: CheckpointDiffStatus::Missing,
            })
        );
        assert_eq!(
            checkpoint_reactor_turn_completion_action("thread-1", 2, Some("completed"), true),
            None
        );
        assert_eq!(
            checkpoint_reactor_revert_actions(
                "thread-1",
                1,
                3,
                Some("C:/repo/project-1"),
                true,
                Some("refs/t3/checkpoints/dGhyZWFkLTE/turn/1"),
            ),
            vec![
                CheckpointReactorAction::CaptureBaseline {
                    checkpoint_ref: "refs/t3/checkpoints/dGhyZWFkLTE/turn/1".to_string(),
                },
                CheckpointReactorAction::RollbackConversation { num_turns: 2 },
                CheckpointReactorAction::DeleteStaleCheckpointRefs {
                    checkpoint_refs: vec![
                        "refs/t3/checkpoints/dGhyZWFkLTE/turn/2".to_string(),
                        "refs/t3/checkpoints/dGhyZWFkLTE/turn/3".to_string(),
                    ],
                },
            ]
        );
        assert_eq!(
            checkpoint_reactor_revert_actions("thread-1", 1, 1, None, true, None),
            vec![CheckpointReactorAction::AppendRevertFailureActivity {
                detail: "No active provider session with workspace cwd is bound to this thread."
                    .to_string(),
            }]
        );
    }

    #[test]
    fn ports_checkpoint_diff_query_contracts() {
        let context = ProjectionThreadCheckpointContext {
            thread_id: "thread-1".to_string(),
            project_id: "project-1".to_string(),
            workspace_root: "/tmp/workspace".to_string(),
            worktree_path: Some("/tmp/worktree".to_string()),
            checkpoints: vec![checkpoint(1), checkpoint(4)],
        };

        assert_eq!(
            resolve_checkpoint_turn_diff_query(
                &OrchestrationGetTurnDiffInput {
                    thread_id: "thread-1".to_string(),
                    from_turn_count: 0,
                    to_turn_count: 1,
                    ignore_whitespace: None,
                },
                Some(&context),
            )
            .unwrap(),
            CheckpointDiffQueryDecision::Diff(CheckpointDiffQueryPlan {
                thread_id: "thread-1".to_string(),
                from_turn_count: 0,
                to_turn_count: 1,
                cwd: "/tmp/worktree".to_string(),
                from_checkpoint_ref: checkpoint_ref_for_thread_turn("thread-1", 0),
                to_checkpoint_ref: checkpoint_ref_for_thread_turn("thread-1", 1),
                fallback_from_to_head: false,
                ignore_whitespace: true,
            })
        );

        assert_eq!(
            resolve_checkpoint_turn_diff_query(
                &OrchestrationGetTurnDiffInput {
                    thread_id: "thread-1".to_string(),
                    from_turn_count: 2,
                    to_turn_count: 2,
                    ignore_whitespace: Some(false),
                },
                None,
            )
            .unwrap(),
            CheckpointDiffQueryDecision::Empty(build_checkpoint_diff_result("thread-1", 2, 2, ""))
        );

        let full_context = ProjectionFullThreadDiffContext {
            thread_id: "thread-1".to_string(),
            project_id: "project-1".to_string(),
            workspace_root: "/tmp/workspace".to_string(),
            worktree_path: None,
            latest_checkpoint_turn_count: 4,
            to_checkpoint_ref: Some(checkpoint_ref_for_thread_turn("thread-1", 4)),
        };
        assert_eq!(
            resolve_checkpoint_full_thread_diff_query(
                &OrchestrationGetFullThreadDiffInput {
                    thread_id: "thread-1".to_string(),
                    to_turn_count: 4,
                    ignore_whitespace: Some(false),
                },
                Some(&full_context),
            )
            .unwrap(),
            CheckpointDiffQueryDecision::Diff(CheckpointDiffQueryPlan {
                thread_id: "thread-1".to_string(),
                from_turn_count: 0,
                to_turn_count: 4,
                cwd: "/tmp/workspace".to_string(),
                from_checkpoint_ref: checkpoint_ref_for_thread_turn("thread-1", 0),
                to_checkpoint_ref: checkpoint_ref_for_thread_turn("thread-1", 4),
                fallback_from_to_head: false,
                ignore_whitespace: false,
            })
        );
        assert_eq!(
            resolve_checkpoint_full_thread_diff_query(
                &OrchestrationGetFullThreadDiffInput {
                    thread_id: "thread-1".to_string(),
                    to_turn_count: 0,
                    ignore_whitespace: None,
                },
                None,
            )
            .unwrap(),
            CheckpointDiffQueryDecision::Empty(build_checkpoint_diff_result("thread-1", 0, 0, ""))
        );

        assert_eq!(
            resolve_checkpoint_turn_diff_query(
                &OrchestrationGetTurnDiffInput {
                    thread_id: "thread-missing".to_string(),
                    from_turn_count: 0,
                    to_turn_count: 1,
                    ignore_whitespace: None,
                },
                None,
            )
            .unwrap_err()
            .message(),
            "Checkpoint invariant violation in CheckpointDiffQuery.getTurnDiff: Thread 'thread-missing' not found."
        );

        assert_eq!(
            resolve_checkpoint_turn_diff_query(
                &OrchestrationGetTurnDiffInput {
                    thread_id: "thread-1".to_string(),
                    from_turn_count: 0,
                    to_turn_count: 5,
                    ignore_whitespace: None,
                },
                Some(&context),
            )
            .unwrap_err()
            .message(),
            "Checkpoint unavailable for thread thread-1 turn 5: Turn diff range exceeds current turn count: requested 5, current 4."
        );

        let missing_workspace = ProjectionFullThreadDiffContext {
            workspace_root: String::new(),
            to_checkpoint_ref: Some(checkpoint_ref_for_thread_turn("thread-1", 1)),
            latest_checkpoint_turn_count: 1,
            ..full_context
        };
        assert_eq!(
            resolve_checkpoint_full_thread_diff_query(
                &OrchestrationGetFullThreadDiffInput {
                    thread_id: "thread-1".to_string(),
                    to_turn_count: 1,
                    ignore_whitespace: None,
                },
                Some(&missing_workspace),
            )
            .unwrap_err()
            .message(),
            "Checkpoint invariant violation in CheckpointDiffQuery.getFullThreadDiff: Workspace path missing for thread 'thread-1' when computing full thread diff."
        );

        let missing_ref = ProjectionFullThreadDiffContext {
            to_checkpoint_ref: None,
            ..missing_workspace
        };
        assert_eq!(
            resolve_checkpoint_full_thread_diff_query(
                &OrchestrationGetFullThreadDiffInput {
                    thread_id: "thread-1".to_string(),
                    to_turn_count: 1,
                    ignore_whitespace: None,
                },
                Some(&missing_ref),
            )
            .unwrap_err()
            .message(),
            "Checkpoint invariant violation in CheckpointDiffQuery.getFullThreadDiff: Workspace path missing for thread 'thread-1' when computing full thread diff."
        );

        let missing_to_ref = ProjectionFullThreadDiffContext {
            workspace_root: "/tmp/workspace".to_string(),
            to_checkpoint_ref: None,
            ..missing_ref
        };
        assert_eq!(
            resolve_checkpoint_full_thread_diff_query(
                &OrchestrationGetFullThreadDiffInput {
                    thread_id: "thread-1".to_string(),
                    to_turn_count: 1,
                    ignore_whitespace: None,
                },
                Some(&missing_to_ref),
            )
            .unwrap_err()
            .message(),
            "Checkpoint unavailable for thread thread-1 turn 1: Checkpoint ref is unavailable for turn 1."
        );
    }

    #[test]
    fn ports_normalizer_schema_runtime_layer_and_http_contracts() {
        let attachment = ClientImageAttachmentDataUrl {
            name: "Screenshot.PNG".to_string(),
            data_url: "data:image/png;base64,SGVsbG8=".to_string(),
        };
        let normalized =
            normalize_turn_start_attachment_plan("Thread 1", &attachment).expect("attachment");
        assert_eq!(normalized.byte_len, 5);
        assert_eq!(normalized.source_base64, "SGVsbG8=");
        assert!(normalized.relative_path.ends_with(".png"));
        let ChatAttachment::Image(image) = normalized.attachment;
        assert_eq!(image.name, "Screenshot.PNG");
        assert_eq!(image.mime_type, "image/png");
        assert_eq!(image.size_bytes, 5);
        assert!(image.id.starts_with("thread-1-"));

        assert_eq!(
            normalize_turn_start_attachment_plan(
                "thread-1",
                &ClientImageAttachmentDataUrl {
                    name: "bad.txt".to_string(),
                    data_url: "data:text/plain;base64,SGVsbG8=".to_string(),
                },
            )
            .unwrap_err()
            .message,
            "Invalid image attachment payload for 'bad.txt'."
        );

        assert!(matches!(
            normalize_dispatch_command_plan(
                "project.create",
                Some("C:/repo"),
                Some(true),
                None,
                &[]
            )
            .unwrap(),
            NormalizeDispatchCommandPlan::ProjectCreate {
                create_workspace_root_if_missing: true,
                ..
            }
        ));
        assert!(matches!(
            normalize_dispatch_command_plan(
                "thread.turn.start",
                None,
                None,
                Some("thread-1"),
                &[ClientImageAttachmentDataUrl {
                    name: "shot.jpg".to_string(),
                    data_url: "data:image/jpeg;base64,AA==".to_string(),
                }]
            )
            .unwrap(),
            NormalizeDispatchCommandPlan::ThreadTurnStart { attachments }
                if attachments.len() == 1
        ));

        let aliases = orchestration_schema_aliases();
        assert!(aliases.contains(&("ProjectCreatedPayload", "ProjectCreatedPayload")));
        assert!(aliases.contains(&("MessageSentPayloadSchema", "ThreadMessageSentPayload")));
        assert!(aliases.contains(&(
            "ThreadCheckpointRevertRequestedPayload",
            "ThreadCheckpointRevertRequestedPayload"
        )));

        let layer_plan = orchestration_layer_composition_plan();
        assert_eq!(
            layer_plan.event_infrastructure_layers,
            vec![
                "OrchestrationEventStoreLive",
                "OrchestrationCommandReceiptRepositoryLive"
            ]
        );
        assert!(layer_plan.live_layers.contains(&"OrchestrationEngineLive"));

        assert_eq!(
            orchestration_http_route_plans(),
            vec![
                OrchestrationHttpRoutePlan {
                    kind: OrchestrationHttpRouteKind::Snapshot,
                    method: "GET",
                    path: "/api/orchestration/snapshot",
                    owner_session_required: true,
                    success_status: 200,
                    invalid_request_status: None,
                },
                OrchestrationHttpRoutePlan {
                    kind: OrchestrationHttpRouteKind::Dispatch,
                    method: "POST",
                    path: "/api/orchestration/dispatch",
                    owner_session_required: true,
                    success_status: 200,
                    invalid_request_status: Some(400),
                },
            ]
        );
        assert_eq!(
            orchestration_http_error_status("OrchestrationDispatchCommandError"),
            400
        );
        assert_eq!(
            orchestration_http_error_status("OrchestrationGetSnapshotError"),
            500
        );
    }

    #[test]
    fn decides_project_and_thread_create_events_with_upstream_payload_shape() {
        let read_model = OrchestrationReadModel {
            projects: vec![project("project-1")],
            ..OrchestrationReadModel::default()
        };
        let events = decide_orchestration_command(
            &OrchestrationCommand::ThreadCreate {
                command_id: "cmd-thread".to_string(),
                thread_id: "thread-1".to_string(),
                project_id: "project-1".to_string(),
                title: "Thread 1".to_string(),
                model_selection: json!({ "instanceId": "codex", "model": "gpt-5.4" }),
                runtime_mode: RuntimeMode::FullAccess,
                interaction_mode: ProviderInteractionMode::Plan,
                branch: None,
                worktree_path: None,
                created_at: "2026-03-04T12:00:01.000Z".to_string(),
            },
            &read_model,
            "2026-03-04T12:00:02.000Z",
        )
        .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "thread.created");
        assert_eq!(events[0].aggregate_kind, "thread");
        assert_eq!(events[0].payload["runtimeMode"], "full-access");
        assert_eq!(events[0].payload["interactionMode"], "plan");
        assert_eq!(events[0].correlation_id.as_deref(), Some("cmd-thread"));
    }

    #[test]
    fn decides_turn_start_as_user_message_then_turn_request() {
        let read_model = OrchestrationReadModel {
            projects: vec![project("project-1")],
            threads: vec![thread("thread-1", "project-1")],
            ..OrchestrationReadModel::default()
        };
        let events = decide_orchestration_command(
            &OrchestrationCommand::ThreadTurnStart {
                command_id: "cmd-turn".to_string(),
                thread_id: "thread-1".to_string(),
                message_id: "message-1".to_string(),
                text: "hello".to_string(),
                attachments: Vec::new(),
                model_selection: Some(json!({ "instanceId": "codex", "model": "gpt-5.4" })),
                title_seed: Some("hello".to_string()),
                source_proposed_plan: None,
                created_at: "2026-03-04T12:00:01.000Z".to_string(),
            },
            &read_model,
            "2026-03-04T12:00:02.000Z",
        )
        .unwrap();

        assert_eq!(
            events
                .iter()
                .map(|event| event.event_type.as_str())
                .collect::<Vec<_>>(),
            vec!["thread.message-sent", "thread.turn-start-requested"]
        );
        assert_eq!(
            events[1].causation_event_id.as_deref(),
            Some(events[0].event_id.as_str())
        );
        assert_eq!(events[1].payload["runtimeMode"], "full-access");
        assert_eq!(events[1].payload["interactionMode"], "plan");
        assert_eq!(events[1].payload["modelSelection"]["model"], "gpt-5.4");
    }

    #[test]
    fn refuses_deleting_non_empty_project_without_force() {
        let read_model = OrchestrationReadModel {
            projects: vec![project("project-1")],
            threads: vec![thread("thread-1", "project-1")],
            ..OrchestrationReadModel::default()
        };
        let error = decide_orchestration_command(
            &OrchestrationCommand::ProjectDelete {
                command_id: "cmd-delete".to_string(),
                project_id: "project-1".to_string(),
                force: false,
            },
            &read_model,
            "2026-03-04T12:00:02.000Z",
        )
        .unwrap_err();

        assert_eq!(error.command_type, "project.delete");
        assert!(error.detail.contains("force=true"));
    }

    #[test]
    fn orchestration_reactor_start_order_matches_upstream_composite_reactor() {
        assert_eq!(
            orchestration_reactor_start_order(),
            [
                OrchestrationReactorComponent::ProviderRuntimeIngestion,
                OrchestrationReactorComponent::ProviderCommandReactor,
                OrchestrationReactorComponent::CheckpointReactor,
                OrchestrationReactorComponent::ThreadDeletionReactor,
            ]
        );
    }

    #[test]
    fn provider_runtime_ingestion_helpers_match_upstream_normalization() {
        assert_eq!(normalize_runtime_turn_state(Some("failed")), "failed");
        assert_eq!(normalize_runtime_turn_state(Some("unknown")), "completed");
        assert_eq!(normalize_runtime_turn_state(None), "completed");
        assert_eq!(
            orchestration_session_status_from_runtime_state("waiting"),
            Some("running")
        );
        assert_eq!(
            orchestration_session_status_from_runtime_state("stopped"),
            Some("stopped")
        );
        assert_eq!(
            request_kind_from_canonical_request_type(Some("exec_command_approval")),
            Some("command")
        );
        assert_eq!(
            request_kind_from_canonical_request_type(Some("apply_patch_approval")),
            Some("file-change")
        );
        assert_eq!(
            request_kind_from_canonical_request_type(Some("other")),
            None
        );
        assert_eq!(
            normalize_proposed_plan_markdown(Some("  - do it\n ")).as_deref(),
            Some("- do it")
        );
        assert!(normalize_proposed_plan_markdown(Some("   ")).is_none());
        assert!(has_renderable_assistant_text(Some(" hello ")));
        assert!(!has_renderable_assistant_text(Some("   ")));
        assert_eq!(
            proposed_plan_id_from_runtime_event(
                "thread-1",
                Some("turn-1"),
                Some("item-1"),
                "event-1"
            ),
            "plan:thread-1:turn:turn-1"
        );
        assert_eq!(
            proposed_plan_id_from_runtime_event("thread-1", None, Some("item-1"), "event-1"),
            "plan:thread-1:item:item-1"
        );
        assert_eq!(
            assistant_segment_base_key_from_runtime_event(None, Some("turn-1"), "event-1"),
            "turn-1"
        );
        assert_eq!(
            assistant_segment_message_id("item-1", 0),
            "assistant:item-1"
        );
        assert_eq!(
            assistant_segment_message_id("item-1", 2),
            "assistant:item-1:segment:2"
        );
        assert_eq!(truncate_runtime_detail("abcdef", 5), "ab...");

        let approval = provider_runtime_event_to_activities(&ProviderRuntimeEventInput {
            event_type: "request.opened".to_string(),
            event_id: "event-approval".to_string(),
            created_at: "2026-03-04T12:00:00.000Z".to_string(),
            turn_id: Some("turn-1".to_string()),
            request_id: Some("approval-1".to_string()),
            item_id: None,
            payload: json!({
                "requestType": "exec_command_approval",
                "detail": "run cargo test",
            }),
            session_sequence: Some(7),
        });
        assert_eq!(approval.len(), 1);
        assert_eq!(approval[0].tone, "approval");
        assert_eq!(approval[0].summary, "Command approval requested");
        assert_eq!(approval[0].payload["requestKind"], "command");
        assert_eq!(approval[0].sequence, Some(7));
        let command = provider_runtime_activity_to_thread_activity_append_command(
            "thread-1",
            "provider:event-approval:activity",
            &approval[0],
        );
        match command {
            OrchestrationCommand::ThreadActivityAppend {
                command_id,
                thread_id,
                activity,
                created_at,
            } => {
                assert_eq!(command_id, "provider:event-approval:activity");
                assert_eq!(thread_id, "thread-1");
                assert_eq!(created_at, "2026-03-04T12:00:00.000Z");
                assert_eq!(activity["kind"], "approval.requested");
                assert_eq!(activity["sequence"], 7);
            }
            other => panic!("expected activity append command, got {other:?}"),
        }

        let ignored_tool_user_input =
            provider_runtime_event_to_activities(&ProviderRuntimeEventInput {
                event_type: "request.opened".to_string(),
                event_id: "event-ignored".to_string(),
                created_at: "2026-03-04T12:00:01.000Z".to_string(),
                turn_id: None,
                request_id: Some("input-1".to_string()),
                item_id: None,
                payload: json!({
                    "requestType": "tool_user_input",
                }),
                session_sequence: None,
            });
        assert!(ignored_tool_user_input.is_empty());

        let tool = provider_runtime_event_to_activities(&ProviderRuntimeEventInput {
            event_type: "item.completed".to_string(),
            event_id: "event-tool".to_string(),
            created_at: "2026-03-04T12:00:02.000Z".to_string(),
            turn_id: Some("turn-1".to_string()),
            request_id: None,
            item_id: Some("item-1".to_string()),
            payload: json!({
                "itemType": "command_execution",
                "title": "Cargo test",
                "detail": "tests passed",
            }),
            session_sequence: None,
        });
        assert_eq!(tool[0].kind, "tool.completed");
        assert_eq!(tool[0].summary, "Cargo test");

        assert!(
            provider_runtime_event_to_activities(&ProviderRuntimeEventInput {
                event_type: "thread.token-usage.updated".to_string(),
                event_id: "event-usage-empty".to_string(),
                created_at: "2026-03-04T12:00:03.000Z".to_string(),
                turn_id: None,
                request_id: None,
                item_id: None,
                payload: json!({ "usage": { "usedTokens": 0 } }),
                session_sequence: None,
            })
            .is_empty()
        );

        let session_context = ProviderRuntimeSessionContext {
            thread_id: "thread-1".to_string(),
            provider: "codex".to_string(),
            provider_instance_id: Some("codex".to_string()),
            runtime_mode: RuntimeMode::FullAccess,
            active_turn_id: None,
            last_error: None,
        };
        let started_command = provider_runtime_lifecycle_session_command(
            &session_context,
            "provider:event-turn-started:thread-session-set",
            &ProviderRuntimeEventInput {
                event_type: "turn.started".to_string(),
                event_id: "event-turn-started".to_string(),
                created_at: "2026-03-04T12:00:04.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: None,
                payload: json!({}),
                session_sequence: None,
            },
        )
        .unwrap();
        match started_command {
            OrchestrationCommand::ThreadSessionSet { session, .. } => {
                assert_eq!(session["status"], "running");
                assert_eq!(session["activeTurnId"], "turn-1");
                assert_eq!(session["providerInstanceId"], "codex");
            }
            other => panic!("expected session set command, got {other:?}"),
        }

        let failed_command = provider_runtime_lifecycle_session_command(
            &ProviderRuntimeSessionContext {
                active_turn_id: Some("turn-1".to_string()),
                ..session_context
            },
            "provider:event-turn-completed:thread-session-set",
            &ProviderRuntimeEventInput {
                event_type: "turn.completed".to_string(),
                event_id: "event-turn-completed".to_string(),
                created_at: "2026-03-04T12:00:05.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: None,
                payload: json!({
                    "state": "failed",
                    "errorMessage": "tool crashed",
                }),
                session_sequence: None,
            },
        )
        .unwrap();
        match failed_command {
            OrchestrationCommand::ThreadSessionSet { session, .. } => {
                assert_eq!(session["status"], "error");
                assert!(session["activeTurnId"].is_null());
                assert_eq!(session["lastError"], "tool crashed");
            }
            other => panic!("expected session set command, got {other:?}"),
        }

        let assistant_delta = provider_runtime_assistant_delta_command(
            "thread-1",
            "provider:event-delta:assistant-delta",
            &ProviderRuntimeEventInput {
                event_type: "content.delta".to_string(),
                event_id: "event-delta".to_string(),
                created_at: "2026-03-04T12:00:06.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: Some("assistant-item".to_string()),
                payload: json!({
                    "streamKind": "assistant_text",
                    "delta": "hello",
                }),
                session_sequence: None,
            },
        )
        .unwrap();
        match assistant_delta {
            OrchestrationCommand::ThreadMessageAssistantDelta {
                message_id,
                delta,
                turn_id,
                ..
            } => {
                assert_eq!(message_id, "assistant:assistant-item");
                assert_eq!(delta, "hello");
                assert_eq!(turn_id.as_deref(), Some("turn-1"));
            }
            other => panic!("expected assistant delta command, got {other:?}"),
        }

        let assistant_complete = provider_runtime_assistant_complete_command(
            "thread-1",
            "provider:event-assistant-complete:assistant-complete",
            &ProviderRuntimeEventInput {
                event_type: "item.completed".to_string(),
                event_id: "event-assistant-complete".to_string(),
                created_at: "2026-03-04T12:00:07.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: Some("assistant-item".to_string()),
                payload: json!({
                    "itemType": "assistant_message",
                    "detail": "hello",
                }),
                session_sequence: None,
            },
        )
        .unwrap();
        match assistant_complete {
            OrchestrationCommand::ThreadMessageAssistantComplete {
                message_id,
                turn_id,
                created_at,
                ..
            } => {
                assert_eq!(message_id, "assistant:assistant-item");
                assert_eq!(turn_id.as_deref(), Some("turn-1"));
                assert_eq!(created_at, "2026-03-04T12:00:07.000Z");
            }
            other => panic!("expected assistant complete command, got {other:?}"),
        }

        let proposed_plan = provider_runtime_proposed_plan_complete_command(
            "thread-1",
            "provider:event-proposed:proposed-plan",
            &ProviderRuntimeEventInput {
                event_type: "turn.proposed.completed".to_string(),
                event_id: "event-proposed".to_string(),
                created_at: "2026-03-04T12:00:08.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: Some("plan-item".to_string()),
                payload: json!({
                    "planMarkdown": "  - inspect\n- port\n ",
                }),
                session_sequence: None,
            },
            Some("2026-03-04T12:00:01.000Z"),
        )
        .unwrap();
        match proposed_plan {
            OrchestrationCommand::ThreadProposedPlanUpsert {
                proposed_plan,
                created_at,
                ..
            } => {
                assert_eq!(proposed_plan.id, "plan:thread-1:turn:turn-1");
                assert_eq!(proposed_plan.turn_id.as_deref(), Some("turn-1"));
                assert_eq!(proposed_plan.plan_markdown, "- inspect\n- port");
                assert_eq!(proposed_plan.created_at, "2026-03-04T12:00:01.000Z");
                assert_eq!(proposed_plan.updated_at, "2026-03-04T12:00:08.000Z");
                assert_eq!(created_at, "2026-03-04T12:00:08.000Z");
            }
            other => panic!("expected proposed plan command, got {other:?}"),
        }

        let diff_complete = provider_runtime_turn_diff_complete_command(
            "thread-1",
            "provider:event-diff:turn-diff",
            &ProviderRuntimeEventInput {
                event_type: "turn.diff.updated".to_string(),
                event_id: "event-diff".to_string(),
                created_at: "2026-03-04T12:00:09.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: Some("diff-item".to_string()),
                payload: json!({}),
                session_sequence: None,
            },
            4,
        )
        .unwrap();
        match diff_complete {
            OrchestrationCommand::ThreadTurnDiffComplete {
                turn_id,
                checkpoint_turn_count,
                checkpoint_ref,
                status,
                files,
                assistant_message_id,
                completed_at,
                ..
            } => {
                assert_eq!(turn_id, "turn-1");
                assert_eq!(checkpoint_turn_count, 4);
                assert_eq!(checkpoint_ref.as_deref(), Some("provider-diff:event-diff"));
                assert_eq!(status, "missing");
                assert!(files.is_empty());
                assert_eq!(assistant_message_id.as_deref(), Some("assistant:diff-item"));
                assert_eq!(completed_at, "2026-03-04T12:00:09.000Z");
            }
            other => panic!("expected turn diff command, got {other:?}"),
        }

        let plan_context = ProviderRuntimeIngestionCommandPlanContext {
            thread_id: "thread-1".to_string(),
            session_context: Some(ProviderRuntimeSessionContext {
                thread_id: "thread-1".to_string(),
                provider: "codex".to_string(),
                provider_instance_id: Some("codex".to_string()),
                runtime_mode: RuntimeMode::FullAccess,
                active_turn_id: None,
                last_error: None,
            }),
            existing_proposed_plan_created_at: Some("2026-03-04T12:00:01.000Z".to_string()),
            next_checkpoint_turn_count: Some(5),
            assistant_completion_has_projected_message: None,
            assistant_completion_projected_text_is_empty: None,
            turn_completion_assistant_message_ids: Vec::new(),
            pause_assistant_message_ids: Vec::new(),
            pause_assistant_buffered_texts: Vec::new(),
            turn_completion_assistant_buffered_texts: Vec::new(),
            turn_completion_proposed_plan_markdown: None,
        };
        let planned = provider_runtime_event_to_orchestration_commands(
            &plan_context,
            &ProviderRuntimeEventInput {
                event_type: "turn.proposed.completed".to_string(),
                event_id: "event-planned-proposed".to_string(),
                created_at: "2026-03-04T12:00:10.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: None,
                payload: json!({ "planMarkdown": "  - planned\n" }),
                session_sequence: None,
            },
        );
        assert_eq!(planned.len(), 1);
        match &planned[0] {
            OrchestrationCommand::ThreadProposedPlanUpsert {
                command_id,
                proposed_plan,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-proposed:proposed-plan-upsert"
                );
                assert_eq!(proposed_plan.plan_markdown, "- planned");
                assert_eq!(proposed_plan.created_at, "2026-03-04T12:00:01.000Z");
                assert_eq!(proposed_plan.updated_at, "2026-03-04T12:00:10.000Z");
            }
            other => panic!("expected planned proposed plan command, got {other:?}"),
        }

        let planned = provider_runtime_event_to_orchestration_commands(
            &plan_context,
            &ProviderRuntimeEventInput {
                event_type: "turn.diff.updated".to_string(),
                event_id: "event-planned-diff".to_string(),
                created_at: "2026-03-04T12:00:11.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: Some("diff-item".to_string()),
                payload: json!({}),
                session_sequence: None,
            },
        );
        assert_eq!(planned.len(), 1);
        match &planned[0] {
            OrchestrationCommand::ThreadTurnDiffComplete {
                command_id,
                checkpoint_turn_count,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-diff:thread-turn-diff-complete"
                );
                assert_eq!(*checkpoint_turn_count, 5);
            }
            other => panic!("expected planned diff command, got {other:?}"),
        }

        let planned = provider_runtime_event_to_orchestration_commands(
            &plan_context,
            &ProviderRuntimeEventInput {
                event_type: "request.opened".to_string(),
                event_id: "event-planned-activity".to_string(),
                created_at: "2026-03-04T12:00:12.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: Some("approval-planned".to_string()),
                item_id: None,
                payload: json!({
                    "requestType": "exec_command_approval",
                    "detail": "run tests",
                }),
                session_sequence: Some(8),
            },
        );
        assert_eq!(planned.len(), 1);
        match &planned[0] {
            OrchestrationCommand::ThreadActivityAppend {
                command_id,
                activity,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-activity:thread-activity-append"
                );
                assert_eq!(activity["kind"], "approval.requested");
                assert_eq!(activity["sequence"], 8);
            }
            other => panic!("expected planned activity command, got {other:?}"),
        }

        let planned = provider_runtime_event_to_orchestration_commands(
            &plan_context,
            &ProviderRuntimeEventInput {
                event_type: "runtime.error".to_string(),
                event_id: "event-planned-error".to_string(),
                created_at: "2026-03-04T12:00:13.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: None,
                payload: json!({ "message": "provider crashed" }),
                session_sequence: Some(9),
            },
        );
        assert_eq!(planned.len(), 2);
        match &planned[0] {
            OrchestrationCommand::ThreadSessionSet {
                command_id,
                session,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-error:thread-session-set"
                );
                assert_eq!(session["status"], "error");
                assert!(session["activeTurnId"].is_null());
                assert_eq!(session["lastError"], "provider crashed");
            }
            other => panic!("expected planned runtime error session command, got {other:?}"),
        }
        match &planned[1] {
            OrchestrationCommand::ThreadActivityAppend {
                command_id,
                activity,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-error:thread-activity-append"
                );
                assert_eq!(activity["kind"], "runtime.error");
                assert_eq!(activity["payload"]["message"], "provider crashed");
            }
            other => panic!("expected planned runtime error activity command, got {other:?}"),
        }

        let planned = provider_runtime_event_to_orchestration_commands(
            &ProviderRuntimeIngestionCommandPlanContext {
                thread_id: "thread-1".to_string(),
                session_context: Some(ProviderRuntimeSessionContext {
                    thread_id: "thread-1".to_string(),
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
                event_id: "event-stale-error".to_string(),
                created_at: "2026-03-04T12:00:13.500Z".to_string(),
                turn_id: Some("turn-stale".to_string()),
                request_id: None,
                item_id: None,
                payload: json!({ "message": "stale crash" }),
                session_sequence: None,
            },
        );
        assert_eq!(planned.len(), 1);
        match &planned[0] {
            OrchestrationCommand::ThreadActivityAppend { activity, .. } => {
                assert_eq!(activity["kind"], "runtime.error");
            }
            other => panic!("expected stale runtime error activity only, got {other:?}"),
        }

        let planned = provider_runtime_event_to_orchestration_commands(
            &plan_context,
            &ProviderRuntimeEventInput {
                event_type: "thread.metadata.updated".to_string(),
                event_id: "event-planned-title".to_string(),
                created_at: "2026-03-04T12:00:14.000Z".to_string(),
                turn_id: None,
                request_id: None,
                item_id: None,
                payload: json!({ "name": "  Renamed thread  " }),
                session_sequence: None,
            },
        );
        assert_eq!(planned.len(), 1);
        match &planned[0] {
            OrchestrationCommand::ThreadMetaUpdate {
                command_id, title, ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-title:thread-meta-update"
                );
                assert_eq!(title.as_deref(), Some("Renamed thread"));
            }
            other => panic!("expected planned thread meta command, got {other:?}"),
        }

        let planned = provider_runtime_event_to_orchestration_commands(
            &ProviderRuntimeIngestionCommandPlanContext {
                assistant_completion_has_projected_message: Some(false),
                assistant_completion_projected_text_is_empty: Some(true),
                ..plan_context.clone()
            },
            &ProviderRuntimeEventInput {
                event_type: "item.completed".to_string(),
                event_id: "event-planned-assistant-fallback".to_string(),
                created_at: "2026-03-04T12:00:15.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: Some("assistant-fallback".to_string()),
                payload: json!({
                    "itemType": "assistant_message",
                    "detail": "fallback text",
                }),
                session_sequence: None,
            },
        );
        assert_eq!(planned.len(), 2);
        match &planned[0] {
            OrchestrationCommand::ThreadMessageAssistantDelta {
                command_id,
                message_id,
                delta,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-assistant-fallback:assistant-delta-finalize"
                );
                assert_eq!(message_id, "assistant:assistant-fallback");
                assert_eq!(delta, "fallback text");
            }
            other => panic!("expected planned fallback delta command, got {other:?}"),
        }
        match &planned[1] {
            OrchestrationCommand::ThreadMessageAssistantComplete {
                command_id,
                message_id,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-assistant-fallback:assistant-complete"
                );
                assert_eq!(message_id, "assistant:assistant-fallback");
            }
            other => panic!("expected planned fallback complete command, got {other:?}"),
        }

        let planned = provider_runtime_event_to_orchestration_commands(
            &ProviderRuntimeIngestionCommandPlanContext {
                assistant_completion_has_projected_message: Some(false),
                assistant_completion_projected_text_is_empty: Some(true),
                ..plan_context
            },
            &ProviderRuntimeEventInput {
                event_type: "item.completed".to_string(),
                event_id: "event-planned-empty-complete".to_string(),
                created_at: "2026-03-04T12:00:16.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: Some("assistant-empty".to_string()),
                payload: json!({
                    "itemType": "assistant_message",
                    "detail": "   ",
                }),
                session_sequence: None,
            },
        );
        assert!(planned.is_empty());

        let planned = provider_runtime_event_to_orchestration_commands(
            &ProviderRuntimeIngestionCommandPlanContext {
                thread_id: "thread-1".to_string(),
                session_context: None,
                existing_proposed_plan_created_at: None,
                next_checkpoint_turn_count: None,
                assistant_completion_has_projected_message: None,
                assistant_completion_projected_text_is_empty: None,
                turn_completion_assistant_message_ids: vec!["assistant:active".to_string()],
                pause_assistant_message_ids: Vec::new(),
                pause_assistant_buffered_texts: Vec::new(),
                turn_completion_assistant_buffered_texts: vec![
                    ProviderRuntimeBufferedAssistantText {
                        message_id: "assistant:active".to_string(),
                        text: "final chunk".to_string(),
                    },
                ],
                turn_completion_proposed_plan_markdown: None,
            },
            &ProviderRuntimeEventInput {
                event_type: "turn.completed".to_string(),
                event_id: "event-planned-turn-complete".to_string(),
                created_at: "2026-03-04T12:00:17.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: None,
                payload: json!({ "state": "completed" }),
                session_sequence: None,
            },
        );
        assert_eq!(planned.len(), 2);
        match &planned[0] {
            OrchestrationCommand::ThreadMessageAssistantDelta {
                command_id,
                message_id,
                delta,
                turn_id,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-turn-complete:assistant-delta-finalize-fallback"
                );
                assert_eq!(message_id, "assistant:active");
                assert_eq!(delta, "final chunk");
                assert_eq!(turn_id.as_deref(), Some("turn-1"));
            }
            other => panic!("expected planned turn-complete delta command, got {other:?}"),
        }
        match &planned[1] {
            OrchestrationCommand::ThreadMessageAssistantComplete {
                command_id,
                message_id,
                turn_id,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-turn-complete:assistant-complete-finalize"
                );
                assert_eq!(message_id, "assistant:active");
                assert_eq!(turn_id.as_deref(), Some("turn-1"));
            }
            other => panic!("expected planned turn-complete assistant command, got {other:?}"),
        }

        let planned = provider_runtime_event_to_orchestration_commands(
            &ProviderRuntimeIngestionCommandPlanContext {
                thread_id: "thread-1".to_string(),
                session_context: None,
                existing_proposed_plan_created_at: None,
                next_checkpoint_turn_count: None,
                assistant_completion_has_projected_message: None,
                assistant_completion_projected_text_is_empty: None,
                turn_completion_assistant_message_ids: Vec::new(),
                pause_assistant_message_ids: vec!["assistant:paused".to_string()],
                pause_assistant_buffered_texts: vec![ProviderRuntimeBufferedAssistantText {
                    message_id: "assistant:paused".to_string(),
                    text: "pending approval".to_string(),
                }],
                turn_completion_assistant_buffered_texts: Vec::new(),
                turn_completion_proposed_plan_markdown: None,
            },
            &ProviderRuntimeEventInput {
                event_type: "request.opened".to_string(),
                event_id: "event-planned-pause".to_string(),
                created_at: "2026-03-04T12:00:18.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: Some("approval-1".to_string()),
                item_id: None,
                payload: json!({
                    "requestType": "exec_command_approval",
                    "detail": "run tests",
                }),
                session_sequence: None,
            },
        );
        assert_eq!(planned.len(), 3);
        match &planned[0] {
            OrchestrationCommand::ThreadMessageAssistantDelta {
                command_id,
                message_id,
                delta,
                turn_id,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-pause:assistant-delta-flush-on-request-opened"
                );
                assert_eq!(message_id, "assistant:paused");
                assert_eq!(delta, "pending approval");
                assert_eq!(turn_id.as_deref(), Some("turn-1"));
            }
            other => panic!("expected planned pause delta command, got {other:?}"),
        }
        match &planned[1] {
            OrchestrationCommand::ThreadMessageAssistantComplete {
                command_id,
                message_id,
                turn_id,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-pause:assistant-complete-on-request-opened"
                );
                assert_eq!(message_id, "assistant:paused");
                assert_eq!(turn_id.as_deref(), Some("turn-1"));
            }
            other => panic!("expected planned pause assistant command, got {other:?}"),
        }
        match &planned[2] {
            OrchestrationCommand::ThreadActivityAppend { activity, .. } => {
                assert_eq!(activity["kind"], "approval.requested");
            }
            other => panic!("expected planned pause activity command, got {other:?}"),
        }

        let planned = provider_runtime_event_to_orchestration_commands(
            &ProviderRuntimeIngestionCommandPlanContext {
                thread_id: "thread-1".to_string(),
                session_context: None,
                existing_proposed_plan_created_at: Some("2026-03-04T12:00:01.000Z".to_string()),
                next_checkpoint_turn_count: None,
                assistant_completion_has_projected_message: None,
                assistant_completion_projected_text_is_empty: None,
                turn_completion_assistant_message_ids: Vec::new(),
                pause_assistant_message_ids: Vec::new(),
                pause_assistant_buffered_texts: Vec::new(),
                turn_completion_assistant_buffered_texts: Vec::new(),
                turn_completion_proposed_plan_markdown: Some("  - final plan\n ".to_string()),
            },
            &ProviderRuntimeEventInput {
                event_type: "turn.completed".to_string(),
                event_id: "event-planned-turn-plan".to_string(),
                created_at: "2026-03-04T12:00:19.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: None,
                payload: json!({ "state": "completed" }),
                session_sequence: None,
            },
        );
        assert_eq!(planned.len(), 1);
        match &planned[0] {
            OrchestrationCommand::ThreadProposedPlanUpsert {
                command_id,
                proposed_plan,
                ..
            } => {
                assert_eq!(
                    command_id,
                    "provider:event-planned-turn-plan:proposed-plan-upsert"
                );
                assert_eq!(proposed_plan.id, "plan:thread-1:turn:turn-1");
                assert_eq!(proposed_plan.plan_markdown, "- final plan");
                assert_eq!(proposed_plan.created_at, "2026-03-04T12:00:01.000Z");
                assert_eq!(proposed_plan.updated_at, "2026-03-04T12:00:19.000Z");
            }
            other => panic!("expected planned turn-complete proposed plan command, got {other:?}"),
        }

        let mut queue = ProviderRuntimeIngestionQueue::new();
        assert!(queue.state().is_idle);
        queue.enqueue_domain_turn_start_requested("domain-turn-start", "thread-1");
        queue.enqueue_runtime(
            ProviderRuntimeIngestionCommandPlanContext {
                thread_id: "thread-1".to_string(),
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
            ProviderRuntimeEventInput {
                event_type: "content.delta".to_string(),
                event_id: "event-queued-delta".to_string(),
                created_at: "2026-03-04T12:00:20.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: Some("assistant-queued".to_string()),
                payload: json!({
                    "streamKind": "assistant_text",
                    "delta": "queued",
                }),
                session_sequence: Some(10),
            },
        );
        assert_eq!(queue.state().pending_count, 2);
        assert!(!queue.state().is_idle);
        let drained = queue.drain();
        assert_eq!(drained.drained_count, 2);
        assert_eq!(drained.runtime_events.len(), 1);
        assert_eq!(drained.runtime_events[0].1.event_id, "event-queued-delta");
        assert!(queue.state().is_idle);
    }

    #[test]
    fn provider_reactor_extracts_intents_and_error_labels_like_upstream() {
        assert_eq!(
            provider_error_label_from_instance_hint(
                Some("codex_personal"),
                Some("codex"),
                Some("codex")
            ),
            "codex_personal"
        );
        assert_eq!(provider_error_label(Some("   ")), "unknown");

        let read_model = OrchestrationReadModel {
            projects: vec![project("project-1")],
            threads: vec![thread("thread-1", "project-1")],
            ..OrchestrationReadModel::default()
        };
        let events = decide_orchestration_command(
            &OrchestrationCommand::ThreadTurnStart {
                command_id: "cmd-turn".to_string(),
                thread_id: "thread-1".to_string(),
                message_id: "message-1".to_string(),
                text: "hello".to_string(),
                attachments: Vec::new(),
                model_selection: None,
                title_seed: None,
                source_proposed_plan: None,
                created_at: "2026-03-04T12:00:01.000Z".to_string(),
            },
            &read_model,
            "2026-03-04T12:00:02.000Z",
        )
        .unwrap();

        assert_eq!(provider_command_intent_for_event(&events[0]), None);
        assert_eq!(
            provider_command_intent_for_event(&events[1]),
            Some(ProviderCommandIntent::TurnStart {
                thread_id: "thread-1".to_string(),
                message_id: "message-1".to_string(),
                runtime_mode: RuntimeMode::FullAccess,
                interaction_mode: ProviderInteractionMode::Plan,
            })
        );
        assert_eq!(
            provider_service_request_for_intent(&ProviderCommandIntent::TurnStart {
                thread_id: "thread-1".to_string(),
                message_id: "message-1".to_string(),
                runtime_mode: RuntimeMode::FullAccess,
                interaction_mode: ProviderInteractionMode::Plan,
            }),
            ProviderServiceRequest::BuildAndSendTurn {
                thread_id: "thread-1".to_string(),
                message_id: "message-1".to_string(),
                runtime_mode: RuntimeMode::FullAccess,
                interaction_mode: ProviderInteractionMode::Plan,
            }
        );
        assert_eq!(
            provider_service_request_for_intent(&ProviderCommandIntent::ApprovalRespond {
                thread_id: "thread-1".to_string(),
                request_id: "approval-1".to_string(),
                decision: "accept".to_string(),
            }),
            ProviderServiceRequest::RespondToRequest(ProviderRespondToRequestInput {
                thread_id: "thread-1".to_string(),
                request_id: "approval-1".to_string(),
                decision: "accept".to_string(),
            })
        );
        assert_eq!(
            map_provider_session_status_to_orchestration_status(ProviderSessionStatus::Connecting),
            "starting"
        );
        assert_eq!(
            map_provider_session_status_to_orchestration_status(ProviderSessionStatus::Closed),
            "stopped"
        );
    }

    #[test]
    fn provider_service_start_session_plan_matches_upstream_resolution_rules() {
        let input = ProviderSessionStartInput {
            thread_id: "thread-1".to_string(),
            provider: Some("codex".to_string()),
            provider_instance_id: "codex-main".to_string(),
            cwd: None,
            model_selection: Some(json!({ "model": "gpt-5.4", "provider": "openai" })),
            resume_cursor: None,
            approval_policy: Some("on-request".to_string()),
            sandbox_mode: Some("workspace-write".to_string()),
            runtime_mode: RuntimeMode::FullAccess,
        };
        let instance_info = ProviderInstanceRoutingInfo {
            instance_id: "codex-main".to_string(),
            driver_kind: "codex".to_string(),
            enabled: true,
            continuation_key: "codex-main".to_string(),
        };
        let persisted = ProviderRuntimeBinding {
            thread_id: "thread-1".to_string(),
            provider: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            adapter_key: None,
            status: Some("running".to_string()),
            resume_cursor: Some(json!({ "opaque": "resume-thread-1" })),
            runtime_payload: Some(json!({
                "cwd": "  C:/work/r3code  ",
                "modelSelection": { "model": "persisted-model" }
            })),
            runtime_mode: Some(RuntimeMode::AutoAcceptEdits),
        };

        let plan = provider_service_start_session_plan(&input, &instance_info, Some(&persisted))
            .expect("start session plan");

        assert_eq!(plan.adapter_input.provider.as_deref(), Some("codex"));
        assert_eq!(plan.adapter_input.provider_instance_id, "codex-main");
        assert_eq!(plan.adapter_input.cwd.as_deref(), Some("C:/work/r3code"));
        assert_eq!(
            plan.adapter_input.resume_cursor,
            Some(json!({ "opaque": "resume-thread-1" }))
        );
        assert_eq!(
            plan.cwd_source,
            ProviderServiceResolvedValueSource::Persisted
        );
        assert_eq!(
            plan.resume_cursor_source,
            ProviderServiceResolvedValueSource::Persisted
        );
        assert_eq!(
            provider_service_persisted_model_selection(persisted.runtime_payload.as_ref()),
            Some(json!({ "model": "persisted-model" }))
        );

        let request_values_win = provider_service_start_session_plan(
            &ProviderSessionStartInput {
                cwd: Some("C:/override".to_string()),
                resume_cursor: Some(json!({ "opaque": "request-cursor" })),
                ..input.clone()
            },
            &instance_info,
            Some(&persisted),
        )
        .expect("request values should win");
        assert_eq!(
            request_values_win.cwd_source,
            ProviderServiceResolvedValueSource::Request
        );
        assert_eq!(
            request_values_win.resume_cursor_source,
            ProviderServiceResolvedValueSource::Request
        );
        assert_eq!(
            request_values_win.adapter_input.resume_cursor,
            Some(json!({ "opaque": "request-cursor" }))
        );

        let other_instance_persisted = ProviderRuntimeBinding {
            provider_instance_id: Some("claude-main".to_string()),
            ..persisted
        };
        let no_reuse = provider_service_start_session_plan(
            &ProviderSessionStartInput {
                cwd: None,
                resume_cursor: None,
                ..input.clone()
            },
            &instance_info,
            Some(&other_instance_persisted),
        )
        .expect("other instance binding should not be reused");
        assert_eq!(
            no_reuse.cwd_source,
            ProviderServiceResolvedValueSource::None
        );
        assert_eq!(
            no_reuse.resume_cursor_source,
            ProviderServiceResolvedValueSource::None
        );

        assert_eq!(
            provider_service_start_session_plan(
                &ProviderSessionStartInput {
                    provider: Some("claudeAgent".to_string()),
                    ..input.clone()
                },
                &instance_info,
                None,
            ),
            Err(ProviderServicePlanError::ProviderDriverMismatch {
                instance_id: "codex-main".to_string(),
                instance_driver: "codex".to_string(),
                requested_provider: "claudeAgent".to_string(),
            })
        );
        assert_eq!(
            provider_service_start_session_plan(
                &input,
                &ProviderInstanceRoutingInfo {
                    enabled: false,
                    ..instance_info.clone()
                },
                None,
            ),
            Err(ProviderServicePlanError::ProviderInstanceDisabled {
                instance_id: "codex-main".to_string(),
            })
        );
        assert_eq!(
            provider_service_require_instance_id(
                "ProviderService.startSession",
                Some("codex"),
                None
            ),
            Err(ProviderServicePlanError::MissingProviderInstanceId {
                operation: "ProviderService.startSession".to_string(),
                provider: Some("codex".to_string()),
            })
        );
    }

    #[test]
    fn provider_service_start_session_execution_and_completion_match_upstream_flow() {
        let input = ProviderSessionStartInput {
            thread_id: "thread-1".to_string(),
            provider: Some("codex".to_string()),
            provider_instance_id: "codex-main".to_string(),
            cwd: Some("C:/work/r3code".to_string()),
            model_selection: Some(json!({ "provider": "openai", "model": "gpt-5.4" })),
            resume_cursor: None,
            approval_policy: None,
            sandbox_mode: None,
            runtime_mode: RuntimeMode::FullAccess,
        };
        let instance_info = ProviderInstanceRoutingInfo {
            instance_id: "codex-main".to_string(),
            driver_kind: "codex".to_string(),
            enabled: true,
            continuation_key: "codex-main".to_string(),
        };

        assert_eq!(
            provider_service_start_session_execution_plan(&input, &instance_info, None)
                .expect("execution plan"),
            ProviderServiceStartSessionExecutionPlan {
                start_call: ProviderServiceAdapterCall::StartSession {
                    provider: "codex".to_string(),
                    provider_instance_id: "codex-main".to_string(),
                    input: input.clone(),
                },
                resume_cursor_source: ProviderServiceResolvedValueSource::None,
                cwd_source: ProviderServiceResolvedValueSource::Request,
            }
        );

        let session = ProviderSession {
            provider: "codex".to_string(),
            provider_instance_id: None,
            status: ProviderSessionStatus::Ready,
            runtime_mode: RuntimeMode::FullAccess,
            cwd: Some("C:/work/r3code".to_string()),
            model: Some("gpt-5.4".to_string()),
            thread_id: "thread-1".to_string(),
            resume_cursor: Some(json!({ "opaque": "resume-thread-1" })),
            active_turn_id: None,
            created_at: "2026-03-04T12:00:00.000Z".to_string(),
            updated_at: "2026-03-04T12:00:01.000Z".to_string(),
            last_error: None,
        };
        let completion = provider_service_start_session_completion_plan(
            &session,
            "thread-1",
            "codex",
            "codex-main",
            Some(json!({ "provider": "openai", "model": "gpt-5.4" })),
            &[ProviderServiceAdapterSessionProbe {
                provider: "claudeAgent".to_string(),
                provider_instance_id: "claude-main".to_string(),
                has_session: true,
            }],
        )
        .expect("completion plan");
        assert_eq!(
            completion.binding.provider_instance_id.as_deref(),
            Some("codex-main")
        );
        assert_eq!(completion.binding.status.as_deref(), Some("running"));
        assert_eq!(
            completion.stop_stale_calls,
            vec![ProviderServiceStopSessionCall {
                provider: "claudeAgent".to_string(),
                provider_instance_id: "claude-main".to_string(),
                input: ProviderStopSessionInput {
                    thread_id: "thread-1".to_string(),
                },
            }]
        );
        assert_eq!(
            provider_service_start_session_completion_plan(
                &ProviderSession {
                    provider: "claudeAgent".to_string(),
                    ..session
                },
                "thread-1",
                "codex",
                "codex-main",
                None,
                &[],
            ),
            Err(
                ProviderServicePlanError::StartSessionAdapterProviderMismatch {
                    expected_provider: "codex".to_string(),
                    actual_provider: "claudeAgent".to_string(),
                }
            )
        );
    }

    #[test]
    fn provider_service_send_turn_plan_rejects_empty_turns() {
        let empty = ProviderSendTurnInput {
            thread_id: "thread-1".to_string(),
            input: Some("  ".to_string()),
            attachments: Vec::new(),
            model_selection: None,
            interaction_mode: ProviderInteractionMode::Default,
        };
        assert_eq!(
            provider_service_send_turn_plan(&empty),
            Err(ProviderServicePlanError::EmptySendTurnInput {
                operation: "ProviderService.sendTurn".to_string(),
            })
        );

        let text = ProviderSendTurnInput {
            input: Some("hello".to_string()),
            ..empty
        };
        assert_eq!(provider_service_send_turn_plan(&text), Ok(text));
    }

    #[test]
    fn provider_service_session_binding_upsert_matches_runtime_payload_shape() {
        let session = ProviderSession {
            provider: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            status: ProviderSessionStatus::Ready,
            runtime_mode: RuntimeMode::FullAccess,
            cwd: Some("C:/work/r3code".to_string()),
            model: Some("gpt-5.4".to_string()),
            thread_id: "thread-1".to_string(),
            resume_cursor: Some(json!({ "opaque": "resume-thread-1" })),
            active_turn_id: Some("turn-1".to_string()),
            created_at: "2026-03-04T12:00:00.000Z".to_string(),
            updated_at: "2026-03-04T12:00:01.000Z".to_string(),
            last_error: None,
        };

        let binding = provider_service_session_binding_upsert(
            &session,
            "thread-1",
            Some(&ProviderServiceRuntimePayloadExtra {
                model_selection: Some(json!({ "provider": "openai", "model": "gpt-5.4" })),
                last_runtime_event: Some("provider.sendTurn".to_string()),
                last_runtime_event_at: Some("2026-03-04T12:00:02.000Z".to_string()),
            }),
        )
        .expect("binding upsert");

        assert_eq!(binding.thread_id, "thread-1");
        assert_eq!(binding.provider, "codex");
        assert_eq!(binding.provider_instance_id.as_deref(), Some("codex-main"));
        assert_eq!(binding.status.as_deref(), Some("running"));
        assert_eq!(binding.runtime_mode, Some(RuntimeMode::FullAccess));
        assert_eq!(
            binding.runtime_payload,
            Some(json!({
                "cwd": "C:/work/r3code",
                "model": "gpt-5.4",
                "activeTurnId": "turn-1",
                "lastError": null,
                "modelSelection": { "provider": "openai", "model": "gpt-5.4" },
                "lastRuntimeEvent": "provider.sendTurn",
                "lastRuntimeEventAt": "2026-03-04T12:00:02.000Z"
            }))
        );

        assert_eq!(
            provider_service_session_binding_upsert(
                &ProviderSession {
                    provider_instance_id: None,
                    ..session
                },
                "thread-1",
                None,
            ),
            Err(ProviderServicePlanError::MissingProviderInstanceId {
                operation: "ProviderService.upsertSessionBinding".to_string(),
                provider: Some("codex".to_string()),
            })
        );
        assert_eq!(
            map_provider_session_status_to_runtime_status(ProviderSessionStatus::Connecting),
            "starting"
        );
        assert_eq!(
            map_provider_session_status_to_runtime_status(ProviderSessionStatus::Closed),
            "stopped"
        );
    }

    #[test]
    fn provider_service_send_and_stop_session_binding_upserts_match_upstream_payloads() {
        let send_input = ProviderSendTurnInput {
            thread_id: "thread-1".to_string(),
            input: Some("hello".to_string()),
            attachments: Vec::new(),
            model_selection: Some(json!({ "provider": "openai", "model": "gpt-5.4" })),
            interaction_mode: ProviderInteractionMode::Plan,
        };
        let turn = ProviderTurnStartResult {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            resume_cursor: Some(json!({ "opaque": "resume-after-turn" })),
        };

        assert_eq!(
            provider_service_send_turn_binding_upsert(
                &send_input,
                &turn,
                "codex",
                "codex-main",
                "2026-03-04T12:00:02.000Z",
            ),
            ProviderRuntimeBinding {
                thread_id: "thread-1".to_string(),
                provider: "codex".to_string(),
                provider_instance_id: Some("codex-main".to_string()),
                adapter_key: None,
                status: Some("running".to_string()),
                resume_cursor: Some(json!({ "opaque": "resume-after-turn" })),
                runtime_payload: Some(json!({
                    "modelSelection": { "provider": "openai", "model": "gpt-5.4" },
                    "activeTurnId": "turn-1",
                    "lastRuntimeEvent": "provider.sendTurn",
                    "lastRuntimeEventAt": "2026-03-04T12:00:02.000Z"
                })),
                runtime_mode: None,
            }
        );

        assert_eq!(
            provider_service_stop_session_binding_upsert("thread-1", "codex", "codex-main"),
            ProviderRuntimeBinding {
                thread_id: "thread-1".to_string(),
                provider: "codex".to_string(),
                provider_instance_id: Some("codex-main".to_string()),
                adapter_key: None,
                status: Some("stopped".to_string()),
                resume_cursor: None,
                runtime_payload: Some(json!({ "activeTurnId": null })),
                runtime_mode: None,
            }
        );
    }

    #[test]
    fn provider_service_list_sessions_merge_matches_binding_overlay_and_mismatch_rules() {
        let session = ProviderSession {
            provider: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            status: ProviderSessionStatus::Ready,
            runtime_mode: RuntimeMode::FullAccess,
            cwd: Some("C:/work/r3code".to_string()),
            model: Some("gpt-5.4".to_string()),
            thread_id: "thread-1".to_string(),
            resume_cursor: None,
            active_turn_id: None,
            created_at: "2026-03-04T12:00:00.000Z".to_string(),
            updated_at: "2026-03-04T12:00:01.000Z".to_string(),
            last_error: None,
        };
        let binding = ProviderRuntimeBinding {
            thread_id: "thread-1".to_string(),
            provider: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            adapter_key: None,
            status: Some("running".to_string()),
            resume_cursor: Some(json!({ "opaque": "persisted-resume" })),
            runtime_payload: None,
            runtime_mode: Some(RuntimeMode::AutoAcceptEdits),
        };

        let merged = provider_service_list_sessions_merge(&[session.clone()], &[binding.clone()])
            .expect("merged sessions");
        assert_eq!(merged.len(), 1);
        assert_eq!(
            merged[0].resume_cursor,
            Some(json!({ "opaque": "persisted-resume" }))
        );
        assert_eq!(merged[0].runtime_mode, RuntimeMode::AutoAcceptEdits);

        let session_cursor_wins = provider_service_list_sessions_merge(
            &[ProviderSession {
                resume_cursor: Some(json!({ "opaque": "adapter-resume" })),
                ..session.clone()
            }],
            &[binding.clone()],
        )
        .expect("session resume cursor should win");
        assert_eq!(
            session_cursor_wins[0].resume_cursor,
            Some(json!({ "opaque": "adapter-resume" }))
        );

        assert_eq!(
            provider_service_list_sessions_merge(
                &[session.clone()],
                &[ProviderRuntimeBinding {
                    provider: "claudeAgent".to_string(),
                    ..binding.clone()
                }],
            ),
            Err(ProviderServicePlanError::ListSessionsProviderMismatch {
                thread_id: "thread-1".to_string(),
                session_provider: "codex".to_string(),
                binding_provider: "claudeAgent".to_string(),
            })
        );

        assert_eq!(
            provider_service_list_sessions_merge(
                &[ProviderSession {
                    provider_instance_id: Some("codex-other".to_string()),
                    ..session.clone()
                }],
                &[binding.clone()],
            ),
            Err(ProviderServicePlanError::ListSessionsInstanceMismatch {
                thread_id: "thread-1".to_string(),
                session_instance_id: Some("codex-other".to_string()),
                binding_instance_id: "codex-main".to_string(),
            })
        );

        assert_eq!(
            provider_service_list_sessions_merge(
                &[session],
                &[ProviderRuntimeBinding {
                    provider_instance_id: None,
                    ..binding
                }],
            ),
            Err(ProviderServicePlanError::MissingProviderInstanceId {
                operation: "ProviderService.listSessions".to_string(),
                provider: Some("codex".to_string()),
            })
        );
    }

    #[test]
    fn provider_service_rollback_conversation_plan_matches_noop_and_recovery_rules() {
        let input = ProviderRollbackConversationInput {
            thread_id: "thread-1".to_string(),
            num_turns: 0,
        };
        assert_eq!(
            provider_service_rollback_conversation_plan(&input, None, false, None),
            Ok(ProviderServiceRollbackConversationPlan::Noop)
        );

        let binding = ProviderRuntimeBinding {
            thread_id: "thread-1".to_string(),
            provider: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            adapter_key: None,
            status: Some("stopped".to_string()),
            resume_cursor: Some(json!({ "opaque": "resume-thread-1" })),
            runtime_payload: Some(json!({ "cwd": "C:/work/r3code" })),
            runtime_mode: Some(RuntimeMode::FullAccess),
        };

        assert_eq!(
            provider_service_rollback_conversation_plan(
                &ProviderRollbackConversationInput {
                    num_turns: 2,
                    ..input.clone()
                },
                Some(&binding),
                false,
                None,
            ),
            Ok(ProviderServiceRollbackConversationPlan::Route {
                route: ProviderServiceRoutableSessionPlan::Recover(
                    ProviderServiceRecoveryPlan::Resume {
                        adapter_input: ProviderSessionStartInput {
                            thread_id: "thread-1".to_string(),
                            provider: Some("codex".to_string()),
                            provider_instance_id: "codex-main".to_string(),
                            cwd: Some("C:/work/r3code".to_string()),
                            model_selection: None,
                            resume_cursor: Some(json!({ "opaque": "resume-thread-1" })),
                            approval_policy: None,
                            sandbox_mode: None,
                            runtime_mode: RuntimeMode::FullAccess,
                        },
                    },
                ),
                num_turns: 2,
            })
        );

        assert_eq!(
            provider_service_rollback_conversation_plan(
                &ProviderRollbackConversationInput {
                    num_turns: 1,
                    ..input
                },
                None,
                false,
                None,
            ),
            Err(ProviderServicePlanError::CannotRouteThreadWithoutBinding {
                operation: "ProviderService.rollbackConversation".to_string(),
                thread_id: "thread-1".to_string(),
            })
        );
    }

    #[test]
    fn provider_service_stop_stale_session_calls_skip_current_and_inactive_instances() {
        assert_eq!(
            provider_service_stop_stale_session_calls(
                "thread-1",
                "claude-main",
                &[
                    ProviderServiceAdapterSessionProbe {
                        provider: "codex".to_string(),
                        provider_instance_id: "codex-main".to_string(),
                        has_session: true,
                    },
                    ProviderServiceAdapterSessionProbe {
                        provider: "claudeAgent".to_string(),
                        provider_instance_id: "claude-main".to_string(),
                        has_session: true,
                    },
                    ProviderServiceAdapterSessionProbe {
                        provider: "cursor".to_string(),
                        provider_instance_id: "cursor-main".to_string(),
                        has_session: false,
                    },
                ],
            ),
            vec![ProviderServiceStopSessionCall {
                provider: "codex".to_string(),
                provider_instance_id: "codex-main".to_string(),
                input: ProviderStopSessionInput {
                    thread_id: "thread-1".to_string(),
                },
            }]
        );
    }

    #[test]
    fn provider_service_adapter_call_plans_match_routing_and_recovery_order() {
        let binding = ProviderRuntimeBinding {
            thread_id: "thread-1".to_string(),
            provider: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            adapter_key: None,
            status: Some("stopped".to_string()),
            resume_cursor: Some(json!({ "opaque": "resume-thread-1" })),
            runtime_payload: Some(json!({ "cwd": "C:/work/r3code" })),
            runtime_mode: Some(RuntimeMode::FullAccess),
        };
        let send_input = ProviderSendTurnInput {
            thread_id: "thread-1".to_string(),
            input: Some("hello".to_string()),
            attachments: Vec::new(),
            model_selection: None,
            interaction_mode: ProviderInteractionMode::Default,
        };

        assert_eq!(
            provider_service_send_turn_adapter_calls(&send_input, Some(&binding), false, None)
                .expect("send calls"),
            vec![
                ProviderServiceAdapterCall::StartSession {
                    provider: "codex".to_string(),
                    provider_instance_id: "codex-main".to_string(),
                    input: ProviderSessionStartInput {
                        thread_id: "thread-1".to_string(),
                        provider: Some("codex".to_string()),
                        provider_instance_id: "codex-main".to_string(),
                        cwd: Some("C:/work/r3code".to_string()),
                        model_selection: None,
                        resume_cursor: Some(json!({ "opaque": "resume-thread-1" })),
                        approval_policy: None,
                        sandbox_mode: None,
                        runtime_mode: RuntimeMode::FullAccess,
                    },
                },
                ProviderServiceAdapterCall::SendTurn {
                    provider: "codex".to_string(),
                    provider_instance_id: "codex-main".to_string(),
                    input: send_input.clone(),
                },
            ]
        );

        let active_session = ProviderSession {
            provider: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            status: ProviderSessionStatus::Running,
            runtime_mode: RuntimeMode::FullAccess,
            cwd: None,
            model: None,
            thread_id: "thread-1".to_string(),
            resume_cursor: None,
            active_turn_id: Some("turn-1".to_string()),
            created_at: "2026-03-04T12:00:00.000Z".to_string(),
            updated_at: "2026-03-04T12:00:01.000Z".to_string(),
            last_error: None,
        };
        assert_eq!(
            provider_service_interrupt_turn_adapter_calls(
                &ProviderInterruptTurnInput {
                    thread_id: "thread-1".to_string(),
                    turn_id: Some("turn-1".to_string()),
                },
                Some(&binding),
                true,
                Some(&active_session),
            )
            .expect("interrupt calls"),
            vec![ProviderServiceAdapterCall::InterruptTurn {
                provider: "codex".to_string(),
                provider_instance_id: "codex-main".to_string(),
                input: ProviderInterruptTurnInput {
                    thread_id: "thread-1".to_string(),
                    turn_id: Some("turn-1".to_string()),
                },
            }]
        );
        assert_eq!(
            provider_service_stop_session_adapter_calls(
                &ProviderStopSessionInput {
                    thread_id: "thread-1".to_string(),
                },
                Some(&binding),
                false,
            )
            .expect("inactive stop should not call adapter"),
            Vec::<ProviderServiceAdapterCall>::new()
        );
        assert_eq!(
            provider_service_rollback_conversation_adapter_calls(
                &ProviderRollbackConversationInput {
                    thread_id: "thread-1".to_string(),
                    num_turns: 0,
                },
                Some(&binding),
                false,
                None,
            )
            .expect("zero rollback is noop"),
            Vec::<ProviderServiceAdapterCall>::new()
        );
    }

    #[test]
    fn provider_service_registry_read_contracts_return_instance_info_and_capabilities() {
        let entries = vec![
            ProviderServiceAdapterRegistryEntry {
                instance_info: ProviderInstanceRoutingInfo {
                    instance_id: "codex-main".to_string(),
                    driver_kind: "codex".to_string(),
                    enabled: true,
                    continuation_key: "codex-main".to_string(),
                },
                capabilities: ProviderAdapterCapabilities {
                    session_model_switch: ProviderSessionModelSwitch::InSession,
                },
            },
            ProviderServiceAdapterRegistryEntry {
                instance_info: ProviderInstanceRoutingInfo {
                    instance_id: "cursor-main".to_string(),
                    driver_kind: "cursor".to_string(),
                    enabled: false,
                    continuation_key: "cursor-main".to_string(),
                },
                capabilities: ProviderAdapterCapabilities {
                    session_model_switch: ProviderSessionModelSwitch::Unsupported,
                },
            },
        ];

        assert_eq!(
            provider_service_instance_info_for_instance_id("cursor-main", &entries)
                .expect("instance info")
                .driver_kind,
            "cursor"
        );
        assert_eq!(
            provider_service_capabilities_for_instance_id("codex-main", &entries),
            Ok(ProviderAdapterCapabilities {
                session_model_switch: ProviderSessionModelSwitch::InSession,
            })
        );
        assert_eq!(
            provider_service_capabilities_for_instance_id("missing", &entries),
            Err(ProviderServicePlanError::ProviderInstanceNotFound {
                operation: "ProviderService.getCapabilities".to_string(),
                instance_id: "missing".to_string(),
            })
        );
        assert_eq!(
            provider_service_instance_info_for_instance_id("missing", &entries),
            Err(ProviderServicePlanError::ProviderInstanceNotFound {
                operation: "ProviderService.getInstanceInfo".to_string(),
                instance_id: "missing".to_string(),
            })
        );
    }

    #[test]
    fn provider_service_runtime_event_correlation_injects_and_guards_instance_id() {
        assert_eq!(
            provider_service_correlate_runtime_event_instance("codex", "codex-main", "codex", None,),
            Ok("codex-main".to_string())
        );
        assert_eq!(
            provider_service_correlate_runtime_event_instance(
                "codex",
                "codex-main",
                "codex",
                Some("codex-main"),
            ),
            Ok("codex-main".to_string())
        );
        assert_eq!(
            provider_service_correlate_runtime_event_instance(
                "codex",
                "codex-main",
                "claudeAgent",
                None,
            ),
            Err(ProviderServicePlanError::RuntimeEventProviderMismatch {
                source_provider: "codex".to_string(),
                event_provider: "claudeAgent".to_string(),
                provider_instance_id: "codex-main".to_string(),
            })
        );
        assert_eq!(
            provider_service_correlate_runtime_event_instance(
                "codex",
                "codex-main",
                "codex",
                Some("codex-other"),
            ),
            Err(ProviderServicePlanError::RuntimeEventInstanceMismatch {
                source_instance_id: "codex-main".to_string(),
                event_instance_id: "codex-other".to_string(),
            })
        );
    }

    #[test]
    fn provider_service_runtime_event_fanout_plan_canonicalizes_events_for_logging_and_pubsub() {
        let event = ProviderServiceRuntimeEventEnvelope {
            provider: "codex".to_string(),
            provider_instance_id: None,
            thread_id: "thread-1".to_string(),
            event: ProviderRuntimeEventInput {
                event_type: "turn.started".to_string(),
                event_id: "event-1".to_string(),
                created_at: "2026-03-04T12:00:00.000Z".to_string(),
                turn_id: Some("turn-1".to_string()),
                request_id: None,
                item_id: None,
                payload: json!({ "state": "running" }),
                session_sequence: Some(1),
            },
        };

        let plan =
            provider_service_runtime_event_fanout_plan("codex", "codex-main", &[event.clone()])
                .expect("fanout plan");
        assert_eq!(plan.log_thread_ids, vec!["thread-1"]);
        assert_eq!(
            plan.canonical_events,
            vec![ProviderServiceRuntimeEventEnvelope {
                provider_instance_id: Some("codex-main".to_string()),
                ..event
            }]
        );

        assert_eq!(
            provider_service_runtime_event_fanout_plan(
                "codex",
                "codex-main",
                &[ProviderServiceRuntimeEventEnvelope {
                    provider_instance_id: Some("codex-other".to_string()),
                    ..plan.canonical_events[0].clone()
                }],
            ),
            Err(ProviderServicePlanError::RuntimeEventInstanceMismatch {
                source_instance_id: "codex-main".to_string(),
                event_instance_id: "codex-other".to_string(),
            })
        );
    }

    #[test]
    fn provider_service_stop_all_plan_matches_shutdown_reconciliation() {
        let entries = vec![ProviderServiceAdapterRegistryEntry {
            instance_info: ProviderInstanceRoutingInfo {
                instance_id: "codex-main".to_string(),
                driver_kind: "codex".to_string(),
                enabled: true,
                continuation_key: "codex-main".to_string(),
            },
            capabilities: ProviderAdapterCapabilities {
                session_model_switch: ProviderSessionModelSwitch::InSession,
            },
        }];
        let active_session = ProviderSession {
            provider: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            status: ProviderSessionStatus::Running,
            runtime_mode: RuntimeMode::FullAccess,
            cwd: Some("C:/work/r3code".to_string()),
            model: Some("gpt-5.4".to_string()),
            thread_id: "thread-1".to_string(),
            resume_cursor: Some(json!({ "opaque": "resume-active" })),
            active_turn_id: Some("turn-1".to_string()),
            created_at: "2026-03-04T12:00:00.000Z".to_string(),
            updated_at: "2026-03-04T12:00:01.000Z".to_string(),
            last_error: None,
        };
        let binding = ProviderRuntimeBinding {
            thread_id: "thread-1".to_string(),
            provider: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            adapter_key: Some("codex-main".to_string()),
            status: Some("running".to_string()),
            resume_cursor: Some(json!({ "opaque": "resume-active" })),
            runtime_payload: Some(json!({ "cwd": "C:/work/r3code" })),
            runtime_mode: Some(RuntimeMode::FullAccess),
        };

        let plan = provider_service_stop_all_plan(
            &["thread-1".to_string()],
            &entries,
            &[active_session],
            &[binding],
            "2026-03-04T12:00:02.000Z",
        )
        .expect("stop all plan");

        assert_eq!(plan.session_count, 1);
        assert_eq!(
            plan.stop_all_calls,
            vec![ProviderServiceStopAllCall {
                provider: "codex".to_string(),
                provider_instance_id: "codex-main".to_string(),
            }]
        );
        assert_eq!(
            plan.active_session_bindings[0].runtime_payload,
            Some(json!({
                "cwd": "C:/work/r3code",
                "model": "gpt-5.4",
                "activeTurnId": "turn-1",
                "lastError": null,
                "lastRuntimeEvent": "provider.stopAll",
                "lastRuntimeEventAt": "2026-03-04T12:00:02.000Z"
            }))
        );
        assert_eq!(
            plan.stopped_bindings,
            vec![ProviderRuntimeBinding {
                thread_id: "thread-1".to_string(),
                provider: "codex".to_string(),
                provider_instance_id: Some("codex-main".to_string()),
                adapter_key: Some("codex-main".to_string()),
                status: Some("stopped".to_string()),
                resume_cursor: None,
                runtime_payload: Some(json!({
                    "activeTurnId": null,
                    "lastRuntimeEvent": "provider.stopAll",
                    "lastRuntimeEventAt": "2026-03-04T12:00:02.000Z"
                })),
                runtime_mode: Some(RuntimeMode::FullAccess),
            }]
        );
    }

    #[test]
    fn provider_service_routable_session_plan_matches_recovery_rules() {
        let binding = ProviderRuntimeBinding {
            thread_id: "thread-1".to_string(),
            provider: "codex".to_string(),
            provider_instance_id: Some("codex-main".to_string()),
            adapter_key: None,
            status: Some("running".to_string()),
            resume_cursor: Some(json!({ "opaque": "resume-thread-1" })),
            runtime_payload: Some(json!({
                "cwd": "C:/work/r3code",
                "modelSelection": { "provider": "openai", "model": "gpt-5.4" }
            })),
            runtime_mode: Some(RuntimeMode::AutoAcceptEdits),
        };
        let active_session = ProviderSession {
            provider: "codex".to_string(),
            provider_instance_id: None,
            status: ProviderSessionStatus::Ready,
            runtime_mode: RuntimeMode::FullAccess,
            cwd: Some("C:/active".to_string()),
            model: Some("gpt-5.4".to_string()),
            thread_id: "thread-1".to_string(),
            resume_cursor: None,
            active_turn_id: None,
            created_at: "2026-03-04T12:00:00.000Z".to_string(),
            updated_at: "2026-03-04T12:00:01.000Z".to_string(),
            last_error: None,
        };

        assert_eq!(
            provider_service_routable_session_plan(
                "thread-missing",
                None,
                "ProviderService.sendTurn",
                true,
                false,
                None,
            ),
            Err(ProviderServicePlanError::CannotRouteThreadWithoutBinding {
                operation: "ProviderService.sendTurn".to_string(),
                thread_id: "thread-missing".to_string(),
            })
        );
        assert_eq!(
            provider_service_routable_session_plan(
                "thread-1",
                Some(&binding),
                "ProviderService.stopSession",
                false,
                false,
                None,
            ),
            Ok(ProviderServiceRoutableSessionPlan::Inactive {
                provider: "codex".to_string(),
                instance_id: "codex-main".to_string(),
                thread_id: "thread-1".to_string(),
            })
        );
        assert_eq!(
            provider_service_routable_session_plan(
                "thread-1",
                Some(&binding),
                "ProviderService.sendTurn",
                true,
                true,
                Some(&active_session),
            ),
            Ok(ProviderServiceRoutableSessionPlan::Active {
                provider: "codex".to_string(),
                instance_id: "codex-main".to_string(),
                thread_id: "thread-1".to_string(),
            })
        );

        let recovery = provider_service_recover_session_plan(
            &binding,
            "ProviderService.sendTurn",
            false,
            None,
        )
        .expect("resume recovery plan");
        assert_eq!(
            recovery,
            ProviderServiceRecoveryPlan::Resume {
                adapter_input: ProviderSessionStartInput {
                    thread_id: "thread-1".to_string(),
                    provider: Some("codex".to_string()),
                    provider_instance_id: "codex-main".to_string(),
                    cwd: Some("C:/work/r3code".to_string()),
                    model_selection: Some(json!({ "provider": "openai", "model": "gpt-5.4" })),
                    resume_cursor: Some(json!({ "opaque": "resume-thread-1" })),
                    approval_policy: None,
                    sandbox_mode: None,
                    runtime_mode: RuntimeMode::AutoAcceptEdits,
                },
            }
        );

        assert_eq!(
            provider_service_recover_session_plan(
                &binding,
                "ProviderService.sendTurn",
                true,
                Some(&active_session),
            ),
            Ok(ProviderServiceRecoveryPlan::AdoptExisting {
                session: ProviderSession {
                    provider_instance_id: Some("codex-main".to_string()),
                    ..active_session
                }
            })
        );

        let missing_resume = ProviderRuntimeBinding {
            resume_cursor: None,
            ..binding
        };
        assert_eq!(
            provider_service_recover_session_plan(
                &missing_resume,
                "ProviderService.sendTurn",
                false,
                None,
            ),
            Err(
                ProviderServicePlanError::CannotRecoverThreadWithoutResumeState {
                    operation: "ProviderService.sendTurn".to_string(),
                    thread_id: "thread-1".to_string(),
                }
            )
        );
    }

    #[test]
    fn thread_deletion_reactor_maps_deleted_events_to_cleanup_actions() {
        let event = thread_simple_event(
            "cmd-delete",
            "thread-1",
            "2026-03-04T12:00:02.000Z",
            "thread.deleted",
            json!({
                "threadId": "thread-1",
                "deletedAt": "2026-03-04T12:00:02.000Z",
            }),
        );

        assert_eq!(
            thread_deletion_cleanup_actions_for_event(&event),
            Some((
                "thread-1".to_string(),
                vec![
                    ThreadDeletionCleanupAction::StopProviderSession,
                    ThreadDeletionCleanupAction::CloseThreadTerminalsAndDeleteHistory,
                ],
            ))
        );
        assert_eq!(
            thread_deletion_cleanup_requests(
                "thread-1",
                &[
                    ThreadDeletionCleanupAction::StopProviderSession,
                    ThreadDeletionCleanupAction::CloseThreadTerminalsAndDeleteHistory,
                ],
            ),
            vec![
                ThreadDeletionCleanupRequest::StopProviderSession(ProviderStopSessionInput {
                    thread_id: "thread-1".to_string(),
                }),
                ThreadDeletionCleanupRequest::CloseThreadTerminalsAndDeleteHistory {
                    thread_id: "thread-1".to_string(),
                },
            ]
        );
    }
}
