use std::collections::BTreeMap;

use crate::rpc::WsRpcMethod;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricAttributeInput {
    String(String),
    Number(String),
    Boolean(bool),
    BigInt(String),
    Null,
    Undefined,
    Object,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceAttributeValue {
    String(String),
    Boolean(bool),
    Int(String),
    Double(f64),
    Bytes(String),
    Array(Vec<TraceAttributeValue>),
    Object(BTreeMap<String, TraceAttributeValue>),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OtlpAnyValue {
    StringValue(String),
    BoolValue(bool),
    IntValue(String),
    DoubleValue(f64),
    BytesValue(String),
    ArrayValue(Vec<OtlpAnyValue>),
    KvListValue(Vec<(String, OtlpAnyValue)>),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceRecordEvent {
    pub name: String,
    pub time_unix_nano: String,
    pub attributes: BTreeMap<String, TraceAttributeValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceRecordLink {
    pub trace_id: String,
    pub span_id: String,
    pub attributes: BTreeMap<String, TraceAttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceRecordExit {
    Success,
    Interrupted { cause: String },
    Failure { cause: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectTraceRecord {
    pub name: String,
    pub kind: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub sampled: bool,
    pub start_time_unix_nano: String,
    pub end_time_unix_nano: String,
    pub duration_ms: f64,
    pub attributes: BTreeMap<String, TraceAttributeValue>,
    pub events: Vec<TraceRecordEvent>,
    pub links: Vec<TraceRecordLink>,
    pub exit: TraceRecordExit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SerializableSpanInput {
    pub name: String,
    pub kind: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub sampled: bool,
    pub start_time_unix_nano: u128,
    pub end_time_unix_nano: u128,
    pub attributes: BTreeMap<String, TraceAttributeValue>,
    pub events: Vec<TraceRecordEvent>,
    pub links: Vec<TraceRecordLink>,
    pub exit: TraceRecordExit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OtlpSpanInput {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub kind: u8,
    pub start_time_unix_nano: String,
    pub end_time_unix_nano: String,
    pub attributes: BTreeMap<String, TraceAttributeValue>,
    pub events: Vec<TraceRecordEvent>,
    pub links: Vec<TraceRecordLink>,
    pub status_code: Option<String>,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OtlpTraceRecord {
    pub name: String,
    pub kind: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub sampled: bool,
    pub start_time_unix_nano: String,
    pub end_time_unix_nano: String,
    pub duration_ms: f64,
    pub attributes: BTreeMap<String, TraceAttributeValue>,
    pub resource_attributes: BTreeMap<String, TraceAttributeValue>,
    pub scope_name: Option<String>,
    pub scope_version: Option<String>,
    pub scope_attributes: BTreeMap<String, TraceAttributeValue>,
    pub events: Vec<TraceRecordEvent>,
    pub links: Vec<TraceRecordLink>,
    pub status_code: Option<String>,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservabilityOutcome {
    Success,
    Failure,
    Interrupt,
}

impl ObservabilityOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Interrupt => "interrupt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectExitKind {
    Success,
    Failure,
    InterruptOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: MetricKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    Counter,
    Timer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricUpdatePlan {
    pub metric_name: &'static str,
    pub attributes: BTreeMap<String, String>,
    pub amount: MetricUpdateAmount,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricUpdateAmount {
    Count(u64),
    DurationNanos(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithMetricsPlan {
    pub updates: Vec<MetricUpdatePlan>,
    pub rethrow_failure: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcInstrumentationPlan {
    pub span_name: Option<String>,
    pub span_attributes: BTreeMap<String, String>,
    pub metrics: WithMetricsPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservabilityLayerInput {
    pub mode: String,
    pub server_trace_path: String,
    pub trace_min_level: String,
    pub trace_timing_enabled: bool,
    pub trace_batch_window_ms: u64,
    pub trace_max_bytes: u64,
    pub trace_max_files: u16,
    pub otlp_traces_url: Option<String>,
    pub otlp_metrics_url: Option<String>,
    pub otlp_export_interval_ms: u64,
    pub otlp_service_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceSinkPlan {
    pub file_path: String,
    pub max_bytes: u64,
    pub max_files: u16,
    pub batch_window_ms: u64,
}

pub const TRACE_SINK_FLUSH_BUFFER_THRESHOLD: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceSinkPushDecision {
    pub append_record: bool,
    pub drop_record: bool,
    pub flush_after_push: bool,
    pub buffered_records_after_push: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtlpExportPlan {
    pub url: String,
    pub export_interval: String,
    pub service_name: String,
    pub resource_attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservabilityLayerPlan {
    pub includes_server_logger: bool,
    pub minimum_trace_level: String,
    pub tracer_timing_enabled: bool,
    pub trace_sink: TraceSinkPlan,
    pub local_file_tracer_path: String,
    pub browser_trace_collector_service: &'static str,
    pub otlp_tracer: Option<OtlpExportPlan>,
    pub otlp_metrics: Option<OtlpExportPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTraceCollectorRecordPlan {
    pub accepted_records: usize,
    pub sink_operation: &'static str,
}

pub const RPC_REQUESTS_TOTAL: MetricSpec = MetricSpec {
    name: "t3_rpc_requests_total",
    description: "Total RPC requests handled by the websocket RPC server.",
    kind: MetricKind::Counter,
};

pub const RPC_REQUEST_DURATION: MetricSpec = MetricSpec {
    name: "t3_rpc_request_duration",
    description: "RPC request handling duration.",
    kind: MetricKind::Timer,
};

pub const ORCHESTRATION_COMMANDS_TOTAL: MetricSpec = MetricSpec {
    name: "t3_orchestration_commands_total",
    description: "Total orchestration commands dispatched.",
    kind: MetricKind::Counter,
};

pub const ORCHESTRATION_COMMAND_DURATION: MetricSpec = MetricSpec {
    name: "t3_orchestration_command_duration",
    description: "Orchestration command dispatch duration.",
    kind: MetricKind::Timer,
};

pub const ORCHESTRATION_COMMAND_ACK_DURATION: MetricSpec = MetricSpec {
    name: "t3_orchestration_command_ack_duration",
    description: "Time from orchestration command dispatch to the first committed domain event emitted for that command.",
    kind: MetricKind::Timer,
};

pub const ORCHESTRATION_EVENTS_PROCESSED_TOTAL: MetricSpec = MetricSpec {
    name: "t3_orchestration_events_processed_total",
    description: "Total orchestration intent events processed by runtime reactors.",
    kind: MetricKind::Counter,
};

pub const PROVIDER_SESSIONS_TOTAL: MetricSpec = MetricSpec {
    name: "t3_provider_sessions_total",
    description: "Total provider session lifecycle operations.",
    kind: MetricKind::Counter,
};

pub const PROVIDER_TURNS_TOTAL: MetricSpec = MetricSpec {
    name: "t3_provider_turns_total",
    description: "Total provider turn lifecycle operations.",
    kind: MetricKind::Counter,
};

pub const PROVIDER_TURN_DURATION: MetricSpec = MetricSpec {
    name: "t3_provider_turn_duration",
    description: "Provider turn request duration.",
    kind: MetricKind::Timer,
};

pub const PROVIDER_RUNTIME_EVENTS_TOTAL: MetricSpec = MetricSpec {
    name: "t3_provider_runtime_events_total",
    description: "Total canonical provider runtime events processed.",
    kind: MetricKind::Counter,
};

pub const GIT_COMMANDS_TOTAL: MetricSpec = MetricSpec {
    name: "t3_git_commands_total",
    description: "Total git commands executed by the server runtime.",
    kind: MetricKind::Counter,
};

pub const GIT_COMMAND_DURATION: MetricSpec = MetricSpec {
    name: "t3_git_command_duration",
    description: "Git command execution duration.",
    kind: MetricKind::Timer,
};

pub const TERMINAL_SESSIONS_TOTAL: MetricSpec = MetricSpec {
    name: "t3_terminal_sessions_total",
    description: "Total terminal sessions started.",
    kind: MetricKind::Counter,
};

pub const TERMINAL_RESTARTS_TOTAL: MetricSpec = MetricSpec {
    name: "t3_terminal_restarts_total",
    description: "Total terminal restart requests handled.",
    kind: MetricKind::Counter,
};

pub const METRIC_SPECS: &[MetricSpec] = &[
    RPC_REQUESTS_TOTAL,
    RPC_REQUEST_DURATION,
    ORCHESTRATION_COMMANDS_TOTAL,
    ORCHESTRATION_COMMAND_DURATION,
    ORCHESTRATION_COMMAND_ACK_DURATION,
    ORCHESTRATION_EVENTS_PROCESSED_TOTAL,
    PROVIDER_SESSIONS_TOTAL,
    PROVIDER_TURNS_TOTAL,
    PROVIDER_TURN_DURATION,
    PROVIDER_RUNTIME_EVENTS_TOTAL,
    GIT_COMMANDS_TOTAL,
    GIT_COMMAND_DURATION,
    TERMINAL_SESSIONS_TOTAL,
    TERMINAL_RESTARTS_TOTAL,
];

pub fn compact_metric_attributes(
    attributes: &[(&str, MetricAttributeInput)],
) -> BTreeMap<String, String> {
    let mut compacted = BTreeMap::new();
    for (key, value) in attributes {
        let Some(value) = (match value {
            MetricAttributeInput::String(value) => Some(value.clone()),
            MetricAttributeInput::Number(value) => Some(value.clone()),
            MetricAttributeInput::Boolean(value) => Some(value.to_string()),
            MetricAttributeInput::BigInt(value) => Some(value.clone()),
            MetricAttributeInput::Null
            | MetricAttributeInput::Undefined
            | MetricAttributeInput::Object => None,
        }) else {
            continue;
        };
        compacted.insert((*key).to_string(), value);
    }
    compacted
}

pub fn compact_trace_attributes(
    attributes: &[(&str, Option<TraceAttributeValue>)],
) -> BTreeMap<String, TraceAttributeValue> {
    let mut compacted = BTreeMap::new();
    for (key, value) in attributes {
        if let Some(value) = value {
            compacted.insert((*key).to_string(), value.clone());
        }
    }
    compacted
}

pub fn decode_otlp_value(input: Option<&OtlpAnyValue>) -> TraceAttributeValue {
    match input {
        Some(OtlpAnyValue::StringValue(value)) => TraceAttributeValue::String(value.clone()),
        Some(OtlpAnyValue::BoolValue(value)) => TraceAttributeValue::Boolean(*value),
        Some(OtlpAnyValue::IntValue(value)) => TraceAttributeValue::Int(value.clone()),
        Some(OtlpAnyValue::DoubleValue(value)) => TraceAttributeValue::Double(*value),
        Some(OtlpAnyValue::BytesValue(value)) => TraceAttributeValue::Bytes(value.clone()),
        Some(OtlpAnyValue::ArrayValue(values)) => TraceAttributeValue::Array(
            values
                .iter()
                .map(|value| decode_otlp_value(Some(value)))
                .collect(),
        ),
        Some(OtlpAnyValue::KvListValue(values)) => TraceAttributeValue::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), decode_otlp_value(Some(value))))
                .collect(),
        ),
        Some(OtlpAnyValue::Null) | None => TraceAttributeValue::Null,
    }
}

pub fn normalize_otlp_span_kind(input: u8) -> &'static str {
    match input {
        1 => "internal",
        2 => "server",
        3 => "client",
        4 => "producer",
        5 => "consumer",
        _ => "internal",
    }
}

pub fn span_to_trace_record(span: SerializableSpanInput) -> EffectTraceRecord {
    let duration_ms = span
        .end_time_unix_nano
        .saturating_sub(span.start_time_unix_nano) as f64
        / 1_000_000.0;
    EffectTraceRecord {
        name: span.name,
        kind: span.kind,
        trace_id: span.trace_id,
        span_id: span.span_id,
        parent_span_id: span.parent_span_id,
        sampled: span.sampled,
        start_time_unix_nano: span.start_time_unix_nano.to_string(),
        end_time_unix_nano: span.end_time_unix_nano.to_string(),
        duration_ms,
        attributes: span.attributes,
        events: span.events,
        links: span.links,
        exit: span.exit,
    }
}

pub fn otlp_span_to_trace_record(
    span: OtlpSpanInput,
    resource_attributes: BTreeMap<String, TraceAttributeValue>,
    scope_name: Option<String>,
    scope_version: Option<String>,
    scope_attributes: BTreeMap<String, TraceAttributeValue>,
) -> OtlpTraceRecord {
    let start = span.start_time_unix_nano.parse::<u128>().unwrap_or(0);
    let end = span.end_time_unix_nano.parse::<u128>().unwrap_or(0);
    OtlpTraceRecord {
        name: span.name,
        kind: normalize_otlp_span_kind(span.kind).to_string(),
        trace_id: span.trace_id,
        span_id: span.span_id,
        parent_span_id: span.parent_span_id,
        sampled: true,
        start_time_unix_nano: span.start_time_unix_nano,
        end_time_unix_nano: span.end_time_unix_nano,
        duration_ms: end.saturating_sub(start) as f64 / 1_000_000.0,
        attributes: span.attributes,
        resource_attributes,
        scope_name,
        scope_version,
        scope_attributes,
        events: span.events,
        links: span.links,
        status_code: span.status_code,
        status_message: span.status_message,
    }
}

pub fn outcome_from_exit(exit: EffectExitKind) -> ObservabilityOutcome {
    match exit {
        EffectExitKind::Success => ObservabilityOutcome::Success,
        EffectExitKind::InterruptOnly => ObservabilityOutcome::Interrupt,
        EffectExitKind::Failure => ObservabilityOutcome::Failure,
    }
}

pub fn normalize_model_metric_label(model: Option<&str>) -> Option<&'static str> {
    let normalized = model?.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    if normalized.contains("gpt") {
        return Some("gpt");
    }
    if normalized.contains("claude") {
        return Some("claude");
    }
    if normalized.contains("gemini") {
        return Some("gemini");
    }
    Some("other")
}

pub fn provider_metric_attributes(
    provider: &str,
    extra: &[(&str, MetricAttributeInput)],
) -> BTreeMap<String, String> {
    let mut pairs = vec![(
        "provider",
        MetricAttributeInput::String(provider.to_string()),
    )];
    pairs.extend(extra.iter().cloned());
    compact_metric_attributes(&pairs)
}

pub fn provider_turn_metric_attributes(
    provider: &str,
    model: Option<&str>,
    extra: &[(&str, MetricAttributeInput)],
) -> BTreeMap<String, String> {
    let mut pairs = vec![(
        "provider",
        MetricAttributeInput::String(provider.to_string()),
    )];
    if let Some(model_family) = normalize_model_metric_label(model) {
        pairs.push((
            "modelFamily",
            MetricAttributeInput::String(model_family.to_string()),
        ));
    }
    pairs.extend(extra.iter().cloned());
    compact_metric_attributes(&pairs)
}

pub fn with_metrics_plan(
    counter: Option<&'static str>,
    timer: Option<&'static str>,
    attributes: BTreeMap<String, String>,
    outcome: ObservabilityOutcome,
    elapsed_nanos: u64,
) -> WithMetricsPlan {
    let mut updates = Vec::new();
    if let Some(timer) = timer {
        updates.push(MetricUpdatePlan {
            metric_name: timer,
            attributes: attributes.clone(),
            amount: MetricUpdateAmount::DurationNanos(elapsed_nanos),
        });
    }
    if let Some(counter) = counter {
        let mut counter_attributes = attributes;
        counter_attributes.insert("outcome".to_string(), outcome.as_str().to_string());
        updates.push(MetricUpdatePlan {
            metric_name: counter,
            attributes: counter_attributes,
            amount: MetricUpdateAmount::Count(1),
        });
    }
    WithMetricsPlan {
        updates,
        rethrow_failure: outcome != ObservabilityOutcome::Success,
    }
}

pub fn should_trace_rpc_method(method: WsRpcMethod) -> bool {
    !matches!(
        method,
        WsRpcMethod::ServerGetTraceDiagnostics
            | WsRpcMethod::ServerGetProcessDiagnostics
            | WsRpcMethod::ServerSignalProcess
    )
}

pub fn rpc_span_attributes(
    method: WsRpcMethod,
    extra: &[(&str, MetricAttributeInput)],
) -> BTreeMap<String, String> {
    let mut pairs = vec![
        (
            "rpc.transport",
            MetricAttributeInput::String("websocket".to_string()),
        ),
        (
            "rpc.system",
            MetricAttributeInput::String("effect-rpc".to_string()),
        ),
        (
            "rpc.method",
            MetricAttributeInput::String(method.wire_name().to_string()),
        ),
    ];
    pairs.extend(extra.iter().cloned());
    compact_metric_attributes(&pairs)
}

pub fn observe_rpc_effect_plan(
    method: WsRpcMethod,
    outcome: ObservabilityOutcome,
    elapsed_nanos: u64,
    trace_attributes: &[(&str, MetricAttributeInput)],
) -> RpcInstrumentationPlan {
    let method_name = method.wire_name();
    let metrics = with_metrics_plan(
        Some(RPC_REQUESTS_TOTAL.name),
        Some(RPC_REQUEST_DURATION.name),
        compact_metric_attributes(&[(
            "method",
            MetricAttributeInput::String(method_name.to_string()),
        )]),
        outcome,
        elapsed_nanos,
    );
    RpcInstrumentationPlan {
        span_name: should_trace_rpc_method(method).then(|| format!("ws.rpc.{method_name}")),
        span_attributes: if should_trace_rpc_method(method) {
            rpc_span_attributes(method, trace_attributes)
        } else {
            BTreeMap::new()
        },
        metrics,
    }
}

fn otlp_export_plan(
    input: &ObservabilityLayerInput,
    url: &str,
    resource_attributes: &BTreeMap<String, String>,
) -> OtlpExportPlan {
    OtlpExportPlan {
        url: url.to_string(),
        export_interval: format!("{} millis", input.otlp_export_interval_ms),
        service_name: input.otlp_service_name.clone(),
        resource_attributes: resource_attributes.clone(),
    }
}

pub fn observability_layer_plan(input: &ObservabilityLayerInput) -> ObservabilityLayerPlan {
    let resource_attributes = BTreeMap::from([
        ("service.mode".to_string(), input.mode.clone()),
        ("service.runtime".to_string(), "t3-server".to_string()),
    ]);

    ObservabilityLayerPlan {
        includes_server_logger: true,
        minimum_trace_level: input.trace_min_level.clone(),
        tracer_timing_enabled: input.trace_timing_enabled,
        trace_sink: TraceSinkPlan {
            file_path: input.server_trace_path.clone(),
            max_bytes: input.trace_max_bytes,
            max_files: input.trace_max_files,
            batch_window_ms: input.trace_batch_window_ms,
        },
        local_file_tracer_path: input.server_trace_path.clone(),
        browser_trace_collector_service: "t3/observability/Services/BrowserTraceCollector",
        otlp_tracer: input
            .otlp_traces_url
            .as_deref()
            .map(|url| otlp_export_plan(input, url, &resource_attributes)),
        otlp_metrics: input
            .otlp_metrics_url
            .as_deref()
            .map(|url| otlp_export_plan(input, url, &resource_attributes)),
    }
}

pub fn browser_trace_collector_record_plan(record_count: usize) -> BrowserTraceCollectorRecordPlan {
    BrowserTraceCollectorRecordPlan {
        accepted_records: record_count,
        sink_operation: "push",
    }
}

pub fn trace_sink_push_decision(
    buffered_records: usize,
    record_serializable: bool,
) -> TraceSinkPushDecision {
    if !record_serializable {
        return TraceSinkPushDecision {
            append_record: false,
            drop_record: true,
            flush_after_push: false,
            buffered_records_after_push: buffered_records,
        };
    }

    let next_buffered_records = buffered_records + 1;
    let flush_after_push = next_buffered_records >= TRACE_SINK_FLUSH_BUFFER_THRESHOLD;
    TraceSinkPushDecision {
        append_record: true,
        drop_record: false,
        flush_after_push,
        buffered_records_after_push: if flush_after_push {
            0
        } else {
            next_buffered_records
        },
    }
}

pub fn trace_sink_close_flushes(buffered_records: usize) -> bool {
    buffered_records > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr(value: &str) -> MetricAttributeInput {
        MetricAttributeInput::String(value.to_string())
    }

    #[test]
    fn ports_observability_attribute_contracts() {
        assert_eq!(
            compact_metric_attributes(&[
                ("string", attr("codex")),
                ("number", MetricAttributeInput::Number("42".to_string())),
                ("bool", MetricAttributeInput::Boolean(true)),
                ("bigint", MetricAttributeInput::BigInt("99".to_string())),
                ("null", MetricAttributeInput::Null),
                ("undefined", MetricAttributeInput::Undefined),
                ("object", MetricAttributeInput::Object),
            ]),
            BTreeMap::from([
                ("bigint".to_string(), "99".to_string()),
                ("bool".to_string(), "true".to_string()),
                ("number".to_string(), "42".to_string()),
                ("string".to_string(), "codex".to_string()),
            ])
        );
        assert_eq!(
            compact_trace_attributes(&[
                (
                    "string",
                    Some(TraceAttributeValue::String("codex".to_string()))
                ),
                ("missing", None),
                (
                    "nested",
                    Some(TraceAttributeValue::Object(BTreeMap::from([(
                        "ok".to_string(),
                        TraceAttributeValue::Boolean(true),
                    )]))),
                ),
            ]),
            BTreeMap::from([
                (
                    "nested".to_string(),
                    TraceAttributeValue::Object(BTreeMap::from([(
                        "ok".to_string(),
                        TraceAttributeValue::Boolean(true),
                    )])),
                ),
                (
                    "string".to_string(),
                    TraceAttributeValue::String("codex".to_string()),
                ),
            ])
        );
        assert_eq!(
            decode_otlp_value(Some(&OtlpAnyValue::ArrayValue(vec![
                OtlpAnyValue::StringValue("alpha".to_string()),
                OtlpAnyValue::KvListValue(vec![(
                    "count".to_string(),
                    OtlpAnyValue::IntValue("42".to_string()),
                )]),
            ]))),
            TraceAttributeValue::Array(vec![
                TraceAttributeValue::String("alpha".to_string()),
                TraceAttributeValue::Object(BTreeMap::from([(
                    "count".to_string(),
                    TraceAttributeValue::Int("42".to_string()),
                )])),
            ])
        );
        assert_eq!(decode_otlp_value(None), TraceAttributeValue::Null);
        assert_eq!(normalize_otlp_span_kind(1), "internal");
        assert_eq!(normalize_otlp_span_kind(2), "server");
        assert_eq!(normalize_otlp_span_kind(3), "client");
        assert_eq!(normalize_otlp_span_kind(4), "producer");
        assert_eq!(normalize_otlp_span_kind(5), "consumer");
        assert_eq!(normalize_otlp_span_kind(99), "internal");
        let event = TraceRecordEvent {
            name: "child event".to_string(),
            time_unix_nano: "12000000".to_string(),
            attributes: BTreeMap::from([(
                "effect.logLevel".to_string(),
                TraceAttributeValue::String("INFO".to_string()),
            )]),
        };
        let link = TraceRecordLink {
            trace_id: "trace-linked".to_string(),
            span_id: "span-linked".to_string(),
            attributes: BTreeMap::from([(
                "link.kind".to_string(),
                TraceAttributeValue::String("demo".to_string()),
            )]),
        };
        let record = span_to_trace_record(SerializableSpanInput {
            name: "child-span".to_string(),
            kind: "internal".to_string(),
            trace_id: "trace-1".to_string(),
            span_id: "span-2".to_string(),
            parent_span_id: Some("span-1".to_string()),
            sampled: true,
            start_time_unix_nano: 10_000_000,
            end_time_unix_nano: 13_500_000,
            attributes: BTreeMap::from([(
                "demo.child".to_string(),
                TraceAttributeValue::Boolean(true),
            )]),
            events: vec![event.clone()],
            links: vec![link.clone()],
            exit: TraceRecordExit::Success,
        });
        assert_eq!(record.name, "child-span");
        assert_eq!(record.parent_span_id.as_deref(), Some("span-1"));
        assert_eq!(record.duration_ms, 3.5);
        assert_eq!(record.events, vec![event.clone()]);
        assert_eq!(record.links, vec![link.clone()]);
        assert_eq!(record.exit, TraceRecordExit::Success);

        let otlp_record = otlp_span_to_trace_record(
            OtlpSpanInput {
                name: "otlp-span".to_string(),
                trace_id: "trace-o".to_string(),
                span_id: "span-o".to_string(),
                parent_span_id: None,
                kind: 3,
                start_time_unix_nano: "1000000".to_string(),
                end_time_unix_nano: "4500000".to_string(),
                attributes: BTreeMap::from([(
                    "rpc.method".to_string(),
                    TraceAttributeValue::String("projects.list".to_string()),
                )]),
                events: vec![event],
                links: vec![link],
                status_code: Some("1".to_string()),
                status_message: Some("ok".to_string()),
            },
            BTreeMap::from([(
                "service.name".to_string(),
                TraceAttributeValue::String("t3-server".to_string()),
            )]),
            Some("scope".to_string()),
            Some("1.0.0".to_string()),
            BTreeMap::new(),
        );
        assert_eq!(otlp_record.kind, "client");
        assert_eq!(otlp_record.duration_ms, 3.5);
        assert_eq!(otlp_record.status_message.as_deref(), Some("ok"));
        assert_eq!(
            otlp_record.resource_attributes["service.name"],
            TraceAttributeValue::String("t3-server".to_string())
        );
        assert_eq!(
            outcome_from_exit(EffectExitKind::Success).as_str(),
            "success"
        );
        assert_eq!(
            outcome_from_exit(EffectExitKind::Failure).as_str(),
            "failure"
        );
        assert_eq!(
            outcome_from_exit(EffectExitKind::InterruptOnly).as_str(),
            "interrupt"
        );
        assert_eq!(normalize_model_metric_label(Some("gpt-5.4")), Some("gpt"));
        assert_eq!(
            normalize_model_metric_label(Some(" Claude Sonnet 4 ")),
            Some("claude")
        );
        assert_eq!(
            normalize_model_metric_label(Some("gemini-2.5-pro")),
            Some("gemini")
        );
        assert_eq!(normalize_model_metric_label(Some("llama-3")), Some("other"));
        assert_eq!(normalize_model_metric_label(Some("  ")), None);
    }

    #[test]
    fn ports_observability_metric_contracts() {
        assert_eq!(METRIC_SPECS.len(), 14);
        assert!(METRIC_SPECS.iter().any(|spec| {
            spec.name == "t3_rpc_requests_total" && spec.kind == MetricKind::Counter
        }));
        assert!(METRIC_SPECS.iter().any(|spec| {
            spec.name == "t3_provider_turn_duration" && spec.kind == MetricKind::Timer
        }));
        assert_eq!(
            provider_metric_attributes("codex", &[("runtime", attr("app-server"))]),
            BTreeMap::from([
                ("provider".to_string(), "codex".to_string()),
                ("runtime".to_string(), "app-server".to_string()),
            ])
        );
        assert_eq!(
            provider_turn_metric_attributes(
                "claude",
                Some("claude-sonnet-4"),
                &[("mode", attr("default"))],
            ),
            BTreeMap::from([
                ("mode".to_string(), "default".to_string()),
                ("modelFamily".to_string(), "claude".to_string()),
                ("provider".to_string(), "claude".to_string()),
            ])
        );

        let plan = with_metrics_plan(
            Some("with_metrics_total"),
            Some("with_metrics_duration"),
            BTreeMap::from([("operation".to_string(), "direct".to_string())]),
            ObservabilityOutcome::Failure,
            1_500_000,
        );
        assert!(plan.rethrow_failure);
        assert_eq!(
            plan.updates,
            vec![
                MetricUpdatePlan {
                    metric_name: "with_metrics_duration",
                    attributes: BTreeMap::from([("operation".to_string(), "direct".to_string())]),
                    amount: MetricUpdateAmount::DurationNanos(1_500_000),
                },
                MetricUpdatePlan {
                    metric_name: "with_metrics_total",
                    attributes: BTreeMap::from([
                        ("operation".to_string(), "direct".to_string()),
                        ("outcome".to_string(), "failure".to_string()),
                    ]),
                    amount: MetricUpdateAmount::Count(1),
                },
            ]
        );
    }

    #[test]
    fn ports_rpc_instrumentation_contracts() {
        assert!(should_trace_rpc_method(WsRpcMethod::ProjectsList));
        assert!(!should_trace_rpc_method(
            WsRpcMethod::ServerGetTraceDiagnostics
        ));
        assert!(!should_trace_rpc_method(
            WsRpcMethod::ServerGetProcessDiagnostics
        ));
        assert!(!should_trace_rpc_method(WsRpcMethod::ServerSignalProcess));

        let plan = observe_rpc_effect_plan(
            WsRpcMethod::ProjectsList,
            ObservabilityOutcome::Success,
            2_000_000,
            &[("rpc.aggregate", attr("workspace"))],
        );
        assert_eq!(plan.span_name.as_deref(), Some("ws.rpc.projects.list"));
        assert_eq!(
            plan.span_attributes,
            BTreeMap::from([
                ("rpc.aggregate".to_string(), "workspace".to_string()),
                ("rpc.method".to_string(), "projects.list".to_string()),
                ("rpc.system".to_string(), "effect-rpc".to_string()),
                ("rpc.transport".to_string(), "websocket".to_string()),
            ])
        );
        assert_eq!(
            plan.metrics.updates[0],
            MetricUpdatePlan {
                metric_name: "t3_rpc_request_duration",
                attributes: BTreeMap::from([("method".to_string(), "projects.list".to_string())]),
                amount: MetricUpdateAmount::DurationNanos(2_000_000),
            }
        );
        assert_eq!(
            observe_rpc_effect_plan(
                WsRpcMethod::ServerGetTraceDiagnostics,
                ObservabilityOutcome::Success,
                1,
                &[],
            )
            .span_name,
            None
        );
    }

    #[test]
    fn ports_observability_layer_and_browser_trace_collector_contracts() {
        let plan = observability_layer_plan(&ObservabilityLayerInput {
            mode: "web".to_string(),
            server_trace_path: "/tmp/server.trace.ndjson".to_string(),
            trace_min_level: "Info".to_string(),
            trace_timing_enabled: true,
            trace_batch_window_ms: 200,
            trace_max_bytes: 10 * 1024 * 1024,
            trace_max_files: 10,
            otlp_traces_url: Some("http://localhost:4318/v1/traces".to_string()),
            otlp_metrics_url: None,
            otlp_export_interval_ms: 10_000,
            otlp_service_name: "t3-server".to_string(),
        });

        assert!(plan.includes_server_logger);
        assert_eq!(plan.minimum_trace_level, "Info");
        assert!(plan.tracer_timing_enabled);
        assert_eq!(plan.trace_sink.file_path, "/tmp/server.trace.ndjson");
        assert_eq!(plan.trace_sink.batch_window_ms, 200);
        assert_eq!(
            plan.browser_trace_collector_service,
            "t3/observability/Services/BrowserTraceCollector"
        );
        assert_eq!(
            plan.otlp_tracer,
            Some(OtlpExportPlan {
                url: "http://localhost:4318/v1/traces".to_string(),
                export_interval: "10000 millis".to_string(),
                service_name: "t3-server".to_string(),
                resource_attributes: BTreeMap::from([
                    ("service.mode".to_string(), "web".to_string()),
                    ("service.runtime".to_string(), "t3-server".to_string()),
                ]),
            })
        );
        assert_eq!(plan.otlp_metrics, None);
        assert_eq!(
            browser_trace_collector_record_plan(2),
            BrowserTraceCollectorRecordPlan {
                accepted_records: 2,
                sink_operation: "push",
            }
        );

        let metrics_plan = observability_layer_plan(&ObservabilityLayerInput {
            otlp_traces_url: None,
            otlp_metrics_url: Some("http://localhost:4318/v1/metrics".to_string()),
            ..ObservabilityLayerInput {
                mode: "desktop".to_string(),
                server_trace_path: "/tmp/server.trace.ndjson".to_string(),
                trace_min_level: "Debug".to_string(),
                trace_timing_enabled: false,
                trace_batch_window_ms: 100,
                trace_max_bytes: 1024,
                trace_max_files: 2,
                otlp_traces_url: None,
                otlp_metrics_url: None,
                otlp_export_interval_ms: 500,
                otlp_service_name: "r3-server".to_string(),
            }
        });
        assert_eq!(
            metrics_plan
                .otlp_metrics
                .as_ref()
                .map(|plan| plan.url.as_str()),
            Some("http://localhost:4318/v1/metrics")
        );
        assert_eq!(metrics_plan.otlp_tracer, None);
    }

    #[test]
    fn ports_shared_trace_sink_buffering_contracts() {
        assert_eq!(TRACE_SINK_FLUSH_BUFFER_THRESHOLD, 32);
        assert_eq!(
            trace_sink_push_decision(0, true),
            TraceSinkPushDecision {
                append_record: true,
                drop_record: false,
                flush_after_push: false,
                buffered_records_after_push: 1,
            }
        );
        assert_eq!(
            trace_sink_push_decision(31, true),
            TraceSinkPushDecision {
                append_record: true,
                drop_record: false,
                flush_after_push: true,
                buffered_records_after_push: 0,
            }
        );
        assert_eq!(
            trace_sink_push_decision(2, false),
            TraceSinkPushDecision {
                append_record: false,
                drop_record: true,
                flush_after_push: false,
                buffered_records_after_push: 2,
            }
        );
        assert!(!trace_sink_close_flushes(0));
        assert!(trace_sink_close_flushes(1));
    }
}
