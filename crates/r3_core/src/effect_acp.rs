use std::collections::BTreeSet;

use serde_json::{Value, json};

pub const ACP_SCHEMA_RELEASE: &str = "v0.11.3";
pub const ACP_PROTOCOL_VERSION: u8 = 1;

pub const ACP_AGENT_METHODS: &[&str] = &[
    "authenticate",
    "initialize",
    "logout",
    "session/cancel",
    "session/close",
    "session/fork",
    "session/list",
    "session/load",
    "session/new",
    "session/prompt",
    "session/resume",
    "session/set_config_option",
    "session/set_mode",
    "session/set_model",
];

pub const ACP_CLIENT_METHODS: &[&str] = &[
    "fs/read_text_file",
    "fs/write_text_file",
    "session/elicitation",
    "session/elicitation/complete",
    "session/request_permission",
    "session/update",
    "terminal/create",
    "terminal/kill",
    "terminal/output",
    "terminal/release",
    "terminal/wait_for_exit",
];

pub const ACP_AGENT_RPC_METHODS: &[&str] = &[
    "initialize",
    "authenticate",
    "logout",
    "session/new",
    "session/load",
    "session/list",
    "session/fork",
    "session/resume",
    "session/close",
    "session/prompt",
    "session/set_model",
    "session/set_config_option",
];

pub const ACP_CLIENT_RPC_METHODS: &[&str] = &[
    "fs/read_text_file",
    "fs/write_text_file",
    "session/request_permission",
    "session/elicitation",
    "terminal/create",
    "terminal/output",
    "terminal/release",
    "terminal/wait_for_exit",
    "terminal/kill",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpMockAgentFlags {
    pub request_log_path: Option<String>,
    pub exit_log_path: Option<String>,
    pub emit_tool_calls: bool,
    pub emit_interleaved_assistant_tool_calls: bool,
    pub emit_generic_tool_placeholders: bool,
    pub emit_ask_question: bool,
    pub fail_set_config_option: bool,
    pub exit_on_set_config_option: bool,
    pub prompt_response_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpSessionModeContract {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpMockAgentPlan {
    pub session_id: &'static str,
    pub default_mode_id: &'static str,
    pub default_model_id: &'static str,
    pub default_reasoning: &'static str,
    pub default_context: &'static str,
    pub protocol_version: u8,
    pub load_session_capability: bool,
    pub supports_request_logging: bool,
    pub handles_unknown_mode_set_ext_request: bool,
    pub stdio_layer: &'static str,
    pub modes: Vec<AcpSessionModeContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpMockConfigOption {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub option_type: &'static str,
    pub current_value: String,
    pub values: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorAcpProbePlan {
    pub target_model_default: &'static str,
    pub prompt_default: &'static str,
    pub agent_bin_default: &'static str,
    pub prompt_wait_ms_default: u64,
    pub request_timeout_ms_default: u64,
    pub request_methods: Vec<&'static str>,
    pub handled_client_methods: Vec<&'static str>,
    pub selection_strategy: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpProtocolErrorShape {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpError {
    Spawn { command: Option<String> },
    ProcessExited { code: Option<i32> },
    ProtocolParse { detail: String },
    Transport { detail: String },
    Request(AcpRequestError),
}

impl AcpError {
    pub fn message(&self) -> String {
        match self {
            Self::Spawn {
                command: Some(command),
            } => {
                format!("Failed to spawn ACP process for command: {command}")
            }
            Self::Spawn { command: None } => "Failed to spawn ACP process".to_string(),
            Self::ProcessExited { code: Some(code) } => {
                format!("ACP process exited with code {code}")
            }
            Self::ProcessExited { code: None } => "ACP process exited".to_string(),
            Self::ProtocolParse { detail } => {
                format!("Failed to parse ACP protocol message: {detail}")
            }
            Self::Transport { detail } => detail.clone(),
            Self::Request(error) => error.error_message.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpRequestError {
    pub code: i32,
    pub error_message: String,
    pub data: Option<Value>,
}

impl AcpRequestError {
    pub fn from_protocol_error(error: AcpProtocolErrorShape) -> Self {
        Self {
            code: error.code,
            error_message: error.message,
            data: error.data,
        }
    }

    pub fn parse_error(message: Option<&str>, data: Option<Value>) -> Self {
        Self::new(-32700, message.unwrap_or("Parse error"), data)
    }

    pub fn invalid_request(message: Option<&str>, data: Option<Value>) -> Self {
        Self::new(-32600, message.unwrap_or("Invalid request"), data)
    }

    pub fn method_not_found(method: &str) -> Self {
        Self::new(-32601, &format!("Method not found: {method}"), None)
    }

    pub fn invalid_params(message: Option<&str>, data: Option<Value>) -> Self {
        Self::new(-32602, message.unwrap_or("Invalid params"), data)
    }

    pub fn internal_error(message: Option<&str>, data: Option<Value>) -> Self {
        Self::new(-32603, message.unwrap_or("Internal error"), data)
    }

    pub fn auth_required(message: Option<&str>, data: Option<Value>) -> Self {
        Self::new(-32000, message.unwrap_or("Authentication required"), data)
    }

    pub fn resource_not_found(message: Option<&str>, data: Option<Value>) -> Self {
        Self::new(-32002, message.unwrap_or("Resource not found"), data)
    }

    pub fn to_protocol_error(&self) -> AcpProtocolErrorShape {
        AcpProtocolErrorShape {
            code: self.code,
            message: self.error_message.clone(),
            data: self.data.clone(),
        }
    }

    fn new(code: i32, message: &str, data: Option<Value>) -> Self {
        Self {
            code,
            error_message: message.to_string(),
            data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpIncomingNotificationKind {
    SessionUpdate,
    ElicitationComplete,
    ExtNotification,
}

pub fn classify_acp_notification(method: &str) -> AcpIncomingNotificationKind {
    match method {
        "session/update" => AcpIncomingNotificationKind::SessionUpdate,
        "session/elicitation/complete" => AcpIncomingNotificationKind::ElicitationComplete,
        _ => AcpIncomingNotificationKind::ExtNotification,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpDecodedRoute {
    ServerRequest,
    ClientResponseOrProtocol,
    ClientNotification(AcpIncomingNotificationKind),
    ExtensionRequest,
    ExtensionResponsePending,
}

pub fn route_acp_decoded_message(
    tag: &str,
    id: &str,
    server_request_methods: &[&str],
    pending_extension_request_ids: &[&str],
) -> AcpDecodedRoute {
    if id.is_empty() {
        return AcpDecodedRoute::ClientNotification(classify_acp_notification(tag));
    }
    if pending_extension_request_ids.contains(&id) {
        return AcpDecodedRoute::ExtensionResponsePending;
    }
    if server_request_methods.contains(&tag) {
        return AcpDecodedRoute::ServerRequest;
    }
    if tag.is_empty() {
        AcpDecodedRoute::ClientResponseOrProtocol
    } else {
        AcpDecodedRoute::ExtensionRequest
    }
}

pub fn next_acp_extension_request_id(current: u64) -> (String, u64) {
    (current.to_string(), current + 1)
}

pub fn acp_notification_message(method: &str, payload: Value) -> Value {
    json!({
        "_tag": "Request",
        "id": "",
        "tag": method,
        "payload": payload,
        "headers": [],
    })
}

pub fn acp_request_message(request_id: &str, method: &str, payload: Value) -> Value {
    json!({
        "_tag": "Request",
        "id": request_id,
        "tag": method,
        "payload": payload,
        "headers": [],
    })
}

pub fn acp_success_response(request_id: &str, value: Value) -> Value {
    json!({
        "_tag": "Exit",
        "requestId": request_id,
        "exit": {
            "_tag": "Success",
            "value": value,
        },
    })
}

pub fn acp_error_response(request_id: &str, error: &AcpRequestError) -> Value {
    json!({
        "_tag": "Exit",
        "requestId": request_id,
        "exit": {
            "_tag": "Failure",
            "cause": [{
                "_tag": "Fail",
                "error": {
                    "code": error.code,
                    "message": error.error_message,
                    "data": error.data,
                },
            }],
        },
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpTerminalPlan {
    pub session_id: String,
    pub terminal_id: String,
    pub output_method: &'static str,
    pub wait_for_exit_method: &'static str,
    pub kill_method: &'static str,
    pub release_method: &'static str,
}

pub fn make_acp_terminal_plan(session_id: &str, terminal_id: &str) -> AcpTerminalPlan {
    AcpTerminalPlan {
        session_id: session_id.to_string(),
        terminal_id: terminal_id.to_string(),
        output_method: "terminal/output",
        wait_for_exit_method: "terminal/wait_for_exit",
        kill_method: "terminal/kill",
        release_method: "terminal/release",
    }
}

pub fn acp_mock_agent_plan() -> AcpMockAgentPlan {
    AcpMockAgentPlan {
        session_id: "mock-session-1",
        default_mode_id: "ask",
        default_model_id: "default",
        default_reasoning: "medium",
        default_context: "272k",
        protocol_version: ACP_PROTOCOL_VERSION,
        load_session_capability: true,
        supports_request_logging: true,
        handles_unknown_mode_set_ext_request: true,
        stdio_layer: "EffectAcpAgent.layerStdio",
        modes: vec![
            AcpSessionModeContract {
                id: "ask",
                name: "Ask",
                description: "Request permission before making any changes",
            },
            AcpSessionModeContract {
                id: "architect",
                name: "Architect",
                description: "Design and plan software systems without implementation",
            },
            AcpSessionModeContract {
                id: "code",
                name: "Code",
                description: "Write and modify code with full tool access",
            },
        ],
    }
}

pub fn acp_mock_agent_flags_from_env(env: &[(String, String)]) -> AcpMockAgentFlags {
    let get = |name: &str| {
        env.iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    };
    let is_one = |name: &str| get(name).as_deref() == Some("1");

    AcpMockAgentFlags {
        request_log_path: get("T3_ACP_REQUEST_LOG_PATH"),
        exit_log_path: get("T3_ACP_EXIT_LOG_PATH"),
        emit_tool_calls: is_one("T3_ACP_EMIT_TOOL_CALLS"),
        emit_interleaved_assistant_tool_calls: is_one(
            "T3_ACP_EMIT_INTERLEAVED_ASSISTANT_TOOL_CALLS",
        ),
        emit_generic_tool_placeholders: is_one("T3_ACP_EMIT_GENERIC_TOOL_PLACEHOLDERS"),
        emit_ask_question: is_one("T3_ACP_EMIT_ASK_QUESTION"),
        fail_set_config_option: is_one("T3_ACP_FAIL_SET_CONFIG_OPTION"),
        exit_on_set_config_option: is_one("T3_ACP_EXIT_ON_SET_CONFIG_OPTION"),
        prompt_response_text: get("T3_ACP_PROMPT_RESPONSE_TEXT"),
    }
}

pub fn acp_mock_config_options_for_model(
    parameterized_model_picker: bool,
    current_mode_id: &str,
    current_model_id: &str,
    current_reasoning: &str,
    current_context: &str,
    current_fast: bool,
) -> Vec<AcpMockConfigOption> {
    if !parameterized_model_picker {
        return vec![AcpMockConfigOption {
            id: "model",
            name: "Model",
            category: "model",
            option_type: "select",
            current_value: current_model_id.to_string(),
            values: vec![
                "default",
                "composer-2",
                "composer-2[fast=true]",
                "gpt-5.3-codex[reasoning=medium,fast=false]",
            ],
        }];
    }

    let mut options = vec![
        AcpMockConfigOption {
            id: "mode",
            name: "Mode",
            category: "mode",
            option_type: "select",
            current_value: current_mode_id.to_string(),
            values: vec!["ask", "architect", "code"],
        },
        AcpMockConfigOption {
            id: "model",
            name: "Model",
            category: "model",
            option_type: "select",
            current_value: current_model_id.to_string(),
            values: vec!["default", "composer-2", "gpt-5.4", "claude-opus-4-6"],
        },
    ];

    match current_model_id {
        "gpt-5.4" => {
            options.push(AcpMockConfigOption {
                id: "reasoning",
                name: "Reasoning",
                category: "thought_level",
                option_type: "select",
                current_value: current_reasoning.to_string(),
                values: vec!["none", "low", "medium", "high", "extra-high"],
            });
            options.push(AcpMockConfigOption {
                id: "context",
                name: "Context",
                category: "model_config",
                option_type: "select",
                current_value: current_context.to_string(),
                values: vec!["272k", "1m"],
            });
            options.push(AcpMockConfigOption {
                id: "fast",
                name: "Fast",
                category: "model_config",
                option_type: "select",
                current_value: current_fast.to_string(),
                values: vec!["false", "true"],
            });
        }
        "composer-2" => options.push(AcpMockConfigOption {
            id: "fast",
            name: "Fast",
            category: "model_config",
            option_type: "select",
            current_value: current_fast.to_string(),
            values: vec!["false", "true"],
        }),
        "claude-opus-4-6" => {
            options.push(AcpMockConfigOption {
                id: "reasoning",
                name: "Reasoning",
                category: "thought_level",
                option_type: "select",
                current_value: current_reasoning.to_string(),
                values: vec!["low", "medium", "high"],
            });
            options.push(AcpMockConfigOption {
                id: "thinking",
                name: "Thinking",
                category: "model_config",
                option_type: "boolean",
                current_value: "true".to_string(),
                values: Vec::new(),
            });
        }
        _ => {}
    }

    options
}

pub fn cursor_acp_model_mismatch_probe_plan() -> CursorAcpProbePlan {
    CursorAcpProbePlan {
        target_model_default: "gpt-5.4",
        prompt_default: "helo",
        agent_bin_default: "agent",
        prompt_wait_ms_default: 4_000,
        request_timeout_ms_default: 20_000,
        request_methods: vec![
            "initialize",
            "session/new",
            "session/set_config_option",
            "session/prompt",
        ],
        handled_client_methods: vec![
            "session/update",
            "session/request_permission",
            "cursor/ask_question",
        ],
        selection_strategy: vec!["model", "reasoning", "context", "fast"],
    }
}

pub fn acp_package_exports() -> Vec<&'static str> {
    vec![
        "./client",
        "./agent",
        "./schema",
        "./rpc",
        "./protocol",
        "./terminal",
        "./errors",
    ]
}

pub fn acp_build_entrypoints() -> Vec<&'static str> {
    vec![
        "src/client.ts",
        "src/agent.ts",
        "src/_generated/schema.gen.ts",
        "src/rpc.ts",
        "src/protocol.ts",
        "src/terminal.ts",
    ]
}

pub const CODEX_APP_SERVER_PROTOCOL_REF: &str = "07b695190f30a450e4921f71f77473e564395c59";

pub const CODEX_CLIENT_REQUEST_METHODS: &[&str] = &[
    "initialize",
    "thread/start",
    "thread/resume",
    "thread/fork",
    "thread/archive",
    "thread/unsubscribe",
    "thread/name/set",
    "thread/metadata/update",
    "thread/unarchive",
    "thread/compact/start",
    "thread/shellCommand",
    "thread/approveGuardianDeniedAction",
    "thread/rollback",
    "thread/list",
    "thread/loaded/list",
    "thread/read",
    "thread/inject_items",
    "skills/list",
    "hooks/list",
    "marketplace/add",
    "marketplace/remove",
    "marketplace/upgrade",
    "plugin/list",
    "plugin/read",
    "plugin/skill/read",
    "plugin/share/save",
    "plugin/share/updateTargets",
    "plugin/share/list",
    "plugin/share/delete",
    "app/list",
    "fs/readFile",
    "fs/writeFile",
    "fs/createDirectory",
    "fs/getMetadata",
    "fs/readDirectory",
    "fs/remove",
    "fs/copy",
    "fs/watch",
    "fs/unwatch",
    "skills/config/write",
    "plugin/install",
    "plugin/uninstall",
    "turn/start",
    "turn/steer",
    "turn/interrupt",
    "review/start",
    "model/list",
    "modelProvider/capabilities/read",
    "experimentalFeature/list",
    "experimentalFeature/enablement/set",
    "mcpServer/oauth/login",
    "config/mcpServer/reload",
    "mcpServerStatus/list",
    "mcpServer/resource/read",
    "mcpServer/tool/call",
    "windowsSandbox/setupStart",
    "windowsSandbox/readiness",
    "account/login/start",
    "account/login/cancel",
    "account/logout",
    "account/rateLimits/read",
    "account/sendAddCreditsNudgeEmail",
    "feedback/upload",
    "command/exec",
    "command/exec/write",
    "command/exec/terminate",
    "command/exec/resize",
    "config/read",
    "externalAgentConfig/detect",
    "externalAgentConfig/import",
    "config/value/write",
    "config/batchWrite",
    "configRequirements/read",
    "account/read",
    "getConversationSummary",
    "gitDiffToRemote",
    "getAuthStatus",
    "fuzzyFileSearch",
];

pub const CODEX_CLIENT_NOTIFICATION_METHODS: &[&str] = &["initialized"];

pub const CODEX_SERVER_REQUEST_METHODS: &[&str] = &[
    "item/commandExecution/requestApproval",
    "item/fileChange/requestApproval",
    "item/tool/requestUserInput",
    "mcpServer/elicitation/request",
    "item/permissions/requestApproval",
    "item/tool/call",
    "account/chatgptAuthTokens/refresh",
    "applyPatchApproval",
    "execCommandApproval",
];

pub const CODEX_SERVER_NOTIFICATION_METHODS: &[&str] = &[
    "error",
    "thread/started",
    "thread/status/changed",
    "thread/archived",
    "thread/unarchived",
    "thread/closed",
    "skills/changed",
    "thread/name/updated",
    "thread/goal/updated",
    "thread/goal/cleared",
    "thread/tokenUsage/updated",
    "turn/started",
    "hook/started",
    "turn/completed",
    "hook/completed",
    "turn/diff/updated",
    "turn/plan/updated",
    "item/started",
    "item/autoApprovalReview/started",
    "item/autoApprovalReview/completed",
    "item/completed",
    "rawResponseItem/completed",
    "item/agentMessage/delta",
    "item/plan/delta",
    "command/exec/outputDelta",
    "process/outputDelta",
    "process/exited",
    "item/commandExecution/outputDelta",
    "item/commandExecution/terminalInteraction",
    "item/fileChange/outputDelta",
    "item/fileChange/patchUpdated",
    "serverRequest/resolved",
    "item/mcpToolCall/progress",
    "mcpServer/oauthLogin/completed",
    "mcpServer/startupStatus/updated",
    "account/updated",
    "account/rateLimits/updated",
    "app/list/updated",
    "remoteControl/status/changed",
    "externalAgentConfig/import/completed",
    "fs/changed",
    "item/reasoning/summaryTextDelta",
    "item/reasoning/summaryPartAdded",
    "item/reasoning/textDelta",
    "thread/compacted",
    "model/rerouted",
    "model/verification",
    "warning",
    "guardianWarning",
    "deprecationNotice",
    "configWarning",
    "fuzzyFileSearch/sessionUpdated",
    "fuzzyFileSearch/sessionCompleted",
    "thread/realtime/started",
    "thread/realtime/itemAdded",
    "thread/realtime/transcript/delta",
    "thread/realtime/transcript/done",
    "thread/realtime/outputAudio/delta",
    "thread/realtime/sdp",
    "thread/realtime/error",
    "thread/realtime/closed",
    "windows/worldWritableWarning",
    "windowsSandbox/setupCompleted",
    "account/login/completed",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexAppServerMessageRoute {
    Request,
    Notification,
    Response,
    Unknown,
}

pub fn classify_codex_app_server_wire_message(value: &Value) -> CodexAppServerMessageRoute {
    let Some(object) = value.as_object() else {
        return CodexAppServerMessageRoute::Unknown;
    };
    let has_method = object.get("method").and_then(Value::as_str).is_some();
    let has_id = object.contains_key("id");
    let has_result_or_error = object.contains_key("result") || object.contains_key("error");
    match (has_method, has_id, has_result_or_error) {
        (true, true, _) => CodexAppServerMessageRoute::Request,
        (true, false, _) => CodexAppServerMessageRoute::Notification,
        (false, true, true) => CodexAppServerMessageRoute::Response,
        _ => CodexAppServerMessageRoute::Unknown,
    }
}

pub fn codex_app_server_request_message(
    request_id: u64,
    method: &str,
    params: Option<Value>,
) -> Value {
    let mut message = json!({
        "id": request_id,
        "method": method,
    });
    if let Some(params) = params {
        message["params"] = params;
    }
    message
}

pub fn codex_app_server_notification_message(method: &str, params: Option<Value>) -> Value {
    let mut message = json!({
        "method": method,
    });
    if let Some(params) = params {
        message["params"] = params;
    }
    message
}

pub fn codex_app_server_response_message(
    request_id: Value,
    result: Option<Value>,
    error: Option<Value>,
) -> Value {
    let mut message = json!({ "id": request_id });
    if let Some(result) = result {
        message["result"] = result;
    }
    if let Some(error) = error {
        message["error"] = error;
    }
    message
}

pub fn codex_app_server_package_exports() -> Vec<&'static str> {
    vec!["./client", "./schema", "./rpc", "./protocol", "./errors"]
}

pub fn validate_unique_methods(methods: &[&str]) -> bool {
    let set = methods.iter().copied().collect::<BTreeSet<_>>();
    set.len() == methods.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_acp_meta_rpc_errors_protocol_and_terminal_contracts() {
        assert_eq!(ACP_SCHEMA_RELEASE, "v0.11.3");
        assert_eq!(ACP_PROTOCOL_VERSION, 1);
        assert!(ACP_AGENT_METHODS.contains(&"session/set_mode"));
        assert_eq!(ACP_AGENT_RPC_METHODS.len(), 12);
        assert_eq!(ACP_CLIENT_RPC_METHODS.len(), 9);
        assert_eq!(
            acp_package_exports(),
            vec![
                "./client",
                "./agent",
                "./schema",
                "./rpc",
                "./protocol",
                "./terminal",
                "./errors"
            ]
        );

        assert_eq!(
            AcpError::Spawn {
                command: Some("cursor-agent".to_string())
            }
            .message(),
            "Failed to spawn ACP process for command: cursor-agent"
        );
        assert_eq!(
            AcpError::ProtocolParse {
                detail: "bad json".to_string()
            }
            .message(),
            "Failed to parse ACP protocol message: bad json"
        );
        assert_eq!(
            AcpRequestError::method_not_found("x").to_protocol_error(),
            AcpProtocolErrorShape {
                code: -32601,
                message: "Method not found: x".to_string(),
                data: None,
            }
        );
        assert_eq!(AcpRequestError::auth_required(None, None).code, -32000);
        assert_eq!(
            classify_acp_notification("session/elicitation/complete"),
            AcpIncomingNotificationKind::ElicitationComplete
        );
        assert_eq!(
            route_acp_decoded_message("session/update", "", ACP_AGENT_RPC_METHODS, &[]),
            AcpDecodedRoute::ClientNotification(AcpIncomingNotificationKind::SessionUpdate)
        );
        assert_eq!(
            route_acp_decoded_message("initialize", "1", ACP_AGENT_RPC_METHODS, &[]),
            AcpDecodedRoute::ServerRequest
        );
        assert_eq!(
            route_acp_decoded_message("ext/demo", "7", ACP_AGENT_RPC_METHODS, &[]),
            AcpDecodedRoute::ExtensionRequest
        );
        assert_eq!(next_acp_extension_request_id(1), ("1".to_string(), 2));
        assert_eq!(
            acp_notification_message("session/cancel", json!({"sessionId": "s1"}))["id"],
            ""
        );
        assert_eq!(
            make_acp_terminal_plan("s1", "t1").wait_for_exit_method,
            "terminal/wait_for_exit"
        );
    }

    #[test]
    fn ports_acp_mock_agent_and_cursor_probe_contracts() {
        let plan = acp_mock_agent_plan();
        assert_eq!(plan.session_id, "mock-session-1");
        assert_eq!(plan.default_mode_id, "ask");
        assert_eq!(plan.default_reasoning, "medium");
        assert_eq!(plan.default_context, "272k");
        assert!(plan.load_session_capability);
        assert!(plan.supports_request_logging);
        assert!(plan.handles_unknown_mode_set_ext_request);
        assert_eq!(
            plan.modes.iter().map(|mode| mode.id).collect::<Vec<_>>(),
            vec!["ask", "architect", "code"]
        );

        let flags = acp_mock_agent_flags_from_env(&[
            ("T3_ACP_EMIT_TOOL_CALLS".to_string(), "1".to_string()),
            (
                "T3_ACP_EMIT_GENERIC_TOOL_PLACEHOLDERS".to_string(),
                "1".to_string(),
            ),
            (
                "T3_ACP_PROMPT_RESPONSE_TEXT".to_string(),
                "custom".to_string(),
            ),
        ]);
        assert!(flags.emit_tool_calls);
        assert!(flags.emit_generic_tool_placeholders);
        assert_eq!(flags.prompt_response_text, Some("custom".to_string()));
        assert!(!flags.exit_on_set_config_option);

        let options =
            acp_mock_config_options_for_model(true, "ask", "gpt-5.4", "medium", "272k", false);
        assert_eq!(
            options.iter().map(|option| option.id).collect::<Vec<_>>(),
            vec!["mode", "model", "reasoning", "context", "fast"]
        );
        assert_eq!(
            options[2].values,
            vec!["none", "low", "medium", "high", "extra-high"]
        );
        assert_eq!(
            acp_mock_config_options_for_model(false, "ask", "default", "medium", "272k", false)[0]
                .values,
            vec![
                "default",
                "composer-2",
                "composer-2[fast=true]",
                "gpt-5.3-codex[reasoning=medium,fast=false]"
            ]
        );

        let probe = cursor_acp_model_mismatch_probe_plan();
        assert_eq!(probe.target_model_default, "gpt-5.4");
        assert_eq!(probe.prompt_default, "helo");
        assert_eq!(probe.prompt_wait_ms_default, 4_000);
        assert_eq!(probe.request_timeout_ms_default, 20_000);
        assert!(probe.request_methods.contains(&"session/set_config_option"));
        assert!(
            probe
                .handled_client_methods
                .contains(&"cursor/ask_question")
        );
        assert_eq!(
            probe.selection_strategy,
            vec!["model", "reasoning", "context", "fast"]
        );
    }

    #[test]
    fn ports_codex_app_server_meta_and_wire_message_contracts() {
        assert_eq!(
            CODEX_APP_SERVER_PROTOCOL_REF,
            "07b695190f30a450e4921f71f77473e564395c59"
        );
        assert_eq!(CODEX_CLIENT_REQUEST_METHODS.len(), 78);
        assert_eq!(CODEX_SERVER_REQUEST_METHODS.len(), 9);
        assert_eq!(CODEX_SERVER_NOTIFICATION_METHODS.len(), 64);
        assert!(CODEX_CLIENT_REQUEST_METHODS.contains(&"thread/start"));
        assert!(CODEX_CLIENT_REQUEST_METHODS.contains(&"fuzzyFileSearch"));
        assert!(CODEX_SERVER_NOTIFICATION_METHODS.contains(&"thread/realtime/sdp"));
        assert!(validate_unique_methods(CODEX_CLIENT_REQUEST_METHODS));
        assert!(validate_unique_methods(CODEX_SERVER_NOTIFICATION_METHODS));
        assert_eq!(
            codex_app_server_package_exports(),
            vec!["./client", "./schema", "./rpc", "./protocol", "./errors"]
        );

        let request =
            codex_app_server_request_message(1, "thread/start", Some(json!({"cwd": "."})));
        assert_eq!(
            classify_codex_app_server_wire_message(&request),
            CodexAppServerMessageRoute::Request
        );
        let notification = codex_app_server_notification_message("initialized", None);
        assert_eq!(
            classify_codex_app_server_wire_message(&notification),
            CodexAppServerMessageRoute::Notification
        );
        let response = codex_app_server_response_message(json!(1), Some(json!({})), None);
        assert_eq!(
            classify_codex_app_server_wire_message(&response),
            CodexAppServerMessageRoute::Response
        );
    }
}
