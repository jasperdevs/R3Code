use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryIdentifierSource {
    CodexAccountId,
    ClaudeUserId,
    AnonymousId,
}

impl TelemetryIdentifierSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CodexAccountId => "codex.account_id",
            Self::ClaudeUserId => "claude.userID",
            Self::AnonymousId => "anonymous-id",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryIdentifierPlan {
    pub source: TelemetryIdentifierSource,
    pub raw_identifier: String,
    pub hash_algorithm: &'static str,
    pub persist_anonymous_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryEnvConfig {
    pub posthog_key: String,
    pub posthog_host: String,
    pub enabled: bool,
    pub flush_batch_size: usize,
    pub max_buffered_events: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferedAnalyticsEvent {
    pub event: String,
    pub properties: BTreeMap<String, String>,
    pub captured_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsEnqueueResult {
    pub buffer: Vec<BufferedAnalyticsEvent>,
    pub size: usize,
    pub dropped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsBatchPayloadEvent {
    pub event: String,
    pub distinct_id: String,
    pub properties: BTreeMap<String, String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsBatchPayload {
    pub api_key: String,
    pub batch: Vec<AnalyticsBatchPayloadEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsFlushPlan {
    pub batches: Vec<Vec<BufferedAnalyticsEvent>>,
    pub remaining: Vec<BufferedAnalyticsEvent>,
}

pub const DEFAULT_POSTHOG_KEY: &str = "phc_XOWci4oZP4VvLiEyrFqkFjP4CZn55mjYYBMREK5Wd6m";
pub const DEFAULT_POSTHOG_HOST: &str = "https://us.i.posthog.com";
pub const DEFAULT_TELEMETRY_FLUSH_BATCH_SIZE: usize = 20;
pub const DEFAULT_TELEMETRY_MAX_BUFFERED_EVENTS: usize = 1_000;
pub const ANALYTICS_SERVICE_TAG: &str = "t3/telemetry/Services/AnalyticsService";

pub fn default_telemetry_env_config() -> TelemetryEnvConfig {
    TelemetryEnvConfig {
        posthog_key: DEFAULT_POSTHOG_KEY.to_string(),
        posthog_host: DEFAULT_POSTHOG_HOST.to_string(),
        enabled: true,
        flush_batch_size: DEFAULT_TELEMETRY_FLUSH_BATCH_SIZE,
        max_buffered_events: DEFAULT_TELEMETRY_MAX_BUFFERED_EVENTS,
    }
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn telemetry_identifier_plan(
    codex_account_id: Result<Option<&str>, &str>,
    claude_user_id: Result<Option<&str>, &str>,
    anonymous_id: Result<Option<&str>, &str>,
    generated_anonymous_id: &str,
) -> Option<TelemetryIdentifierPlan> {
    if let Ok(Some(codex_account_id)) = codex_account_id {
        if let Some(raw_identifier) = non_empty(Some(codex_account_id)) {
            return Some(TelemetryIdentifierPlan {
                source: TelemetryIdentifierSource::CodexAccountId,
                raw_identifier,
                hash_algorithm: "sha256",
                persist_anonymous_id: None,
            });
        }
    }
    if let Ok(Some(claude_user_id)) = claude_user_id {
        if let Some(raw_identifier) = non_empty(Some(claude_user_id)) {
            return Some(TelemetryIdentifierPlan {
                source: TelemetryIdentifierSource::ClaudeUserId,
                raw_identifier,
                hash_algorithm: "sha256",
                persist_anonymous_id: None,
            });
        }
    }
    match anonymous_id {
        Ok(Some(anonymous_id)) => {
            non_empty(Some(anonymous_id)).map(|raw_identifier| TelemetryIdentifierPlan {
                source: TelemetryIdentifierSource::AnonymousId,
                raw_identifier,
                hash_algorithm: "sha256",
                persist_anonymous_id: None,
            })
        }
        Ok(None) | Err(_) => {
            non_empty(Some(generated_anonymous_id)).map(|raw_identifier| TelemetryIdentifierPlan {
                source: TelemetryIdentifierSource::AnonymousId,
                raw_identifier: raw_identifier.clone(),
                hash_algorithm: "sha256",
                persist_anonymous_id: Some(raw_identifier),
            })
        }
    }
}

pub fn analytics_client_type(server_mode: &str) -> &'static str {
    if server_mode == "desktop" {
        "desktop-app"
    } else {
        "cli-web-client"
    }
}

pub fn enqueue_buffered_analytics_event(
    current: &[BufferedAnalyticsEvent],
    event: &str,
    properties: BTreeMap<String, String>,
    captured_at: &str,
    max_buffered_events: usize,
) -> AnalyticsEnqueueResult {
    let mut appended = current.to_vec();
    appended.push(BufferedAnalyticsEvent {
        event: event.to_string(),
        properties,
        captured_at: captured_at.to_string(),
    });
    let dropped = appended.len() > max_buffered_events;
    let buffer = if dropped {
        appended[appended.len() - max_buffered_events..].to_vec()
    } else {
        appended
    };
    AnalyticsEnqueueResult {
        size: buffer.len(),
        dropped,
        buffer,
    }
}

pub fn plan_analytics_flush(
    current: &[BufferedAnalyticsEvent],
    flush_batch_size: usize,
) -> AnalyticsFlushPlan {
    let mut batches = Vec::new();
    let mut cursor = 0usize;
    let batch_size = flush_batch_size.max(1);
    while cursor < current.len() {
        let end = (cursor + batch_size).min(current.len());
        batches.push(current[cursor..end].to_vec());
        cursor = end;
    }
    AnalyticsFlushPlan {
        batches,
        remaining: Vec::new(),
    }
}

pub fn analytics_batch_payload(
    config: &TelemetryEnvConfig,
    events: &[BufferedAnalyticsEvent],
    distinct_id: &str,
    platform: &str,
    wsl_distro_name: Option<&str>,
    arch: &str,
    server_version: &str,
    server_mode: &str,
) -> Option<AnalyticsBatchPayload> {
    if !config.enabled || distinct_id.trim().is_empty() {
        return None;
    }
    Some(AnalyticsBatchPayload {
        api_key: config.posthog_key.clone(),
        batch: events
            .iter()
            .map(|event| {
                let mut properties = event.properties.clone();
                properties.insert("$process_person_profile".to_string(), "false".to_string());
                properties.insert("platform".to_string(), platform.to_string());
                if let Some(wsl) = non_empty(wsl_distro_name) {
                    properties.insert("wsl".to_string(), wsl);
                }
                properties.insert("arch".to_string(), arch.to_string());
                properties.insert("t3CodeVersion".to_string(), server_version.to_string());
                properties.insert(
                    "clientType".to_string(),
                    analytics_client_type(server_mode).to_string(),
                );
                AnalyticsBatchPayloadEvent {
                    event: event.event.clone(),
                    distinct_id: distinct_id.to_string(),
                    properties,
                    timestamp: event.captured_at.clone(),
                }
            })
            .collect(),
    })
}

pub fn analytics_batch_url(posthog_host: &str) -> String {
    format!("{}/batch/", posthog_host.trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(index: usize) -> BTreeMap<String, String> {
        BTreeMap::from([("index".to_string(), index.to_string())])
    }

    #[test]
    fn ports_telemetry_identifier_priority_and_anonymous_id_contracts() {
        assert_eq!(
            TelemetryIdentifierSource::CodexAccountId.as_str(),
            "codex.account_id"
        );
        assert_eq!(
            telemetry_identifier_plan(
                Ok(Some(" codex-account ")),
                Ok(Some("claude-user")),
                Ok(Some("anonymous")),
                "generated",
            ),
            Some(TelemetryIdentifierPlan {
                source: TelemetryIdentifierSource::CodexAccountId,
                raw_identifier: "codex-account".to_string(),
                hash_algorithm: "sha256",
                persist_anonymous_id: None,
            })
        );
        assert_eq!(
            telemetry_identifier_plan(Err("missing"), Ok(Some(" claude-user ")), Ok(None), "gen"),
            Some(TelemetryIdentifierPlan {
                source: TelemetryIdentifierSource::ClaudeUserId,
                raw_identifier: "claude-user".to_string(),
                hash_algorithm: "sha256",
                persist_anonymous_id: None,
            })
        );
        assert_eq!(
            telemetry_identifier_plan(Err("missing"), Err("missing"), Ok(None), "generated-id")
                .unwrap()
                .persist_anonymous_id
                .as_deref(),
            Some("generated-id")
        );
    }

    #[test]
    fn ports_analytics_service_buffer_flush_and_payload_contracts() {
        assert_eq!(
            default_telemetry_env_config(),
            TelemetryEnvConfig {
                posthog_key: DEFAULT_POSTHOG_KEY.to_string(),
                posthog_host: DEFAULT_POSTHOG_HOST.to_string(),
                enabled: true,
                flush_batch_size: 20,
                max_buffered_events: 1_000,
            }
        );
        assert_eq!(
            ANALYTICS_SERVICE_TAG,
            "t3/telemetry/Services/AnalyticsService"
        );
        assert_eq!(analytics_client_type("desktop"), "desktop-app");
        assert_eq!(analytics_client_type("web"), "cli-web-client");

        let mut buffer = Vec::new();
        for index in 0..5 {
            buffer = enqueue_buffered_analytics_event(
                &buffer,
                "test.flush.drain",
                props(index),
                &format!("2026-01-01T00:00:0{index}.000Z"),
                3,
            )
            .buffer;
        }
        assert_eq!(
            buffer
                .iter()
                .map(|event| event.properties["index"].as_str())
                .collect::<Vec<_>>(),
            vec!["2", "3", "4"]
        );

        let flush_plan = plan_analytics_flush(&buffer, 2);
        assert_eq!(flush_plan.batches.len(), 2);
        assert_eq!(flush_plan.batches[0].len(), 2);
        assert!(flush_plan.remaining.is_empty());

        let payload = analytics_batch_payload(
            &TelemetryEnvConfig {
                posthog_key: "phc_test_key".to_string(),
                posthog_host: String::new(),
                enabled: true,
                flush_batch_size: 20,
                max_buffered_events: 1_000,
            },
            &buffer,
            "hashed-id",
            "win32",
            Some("Ubuntu"),
            "x64",
            "0.0.23",
            "web",
        )
        .unwrap();
        assert_eq!(payload.api_key, "phc_test_key");
        assert_eq!(payload.batch.len(), 3);
        assert_eq!(payload.batch[0].distinct_id, "hashed-id");
        assert_eq!(
            payload.batch[0]
                .properties
                .get("clientType")
                .map(String::as_str),
            Some("cli-web-client")
        );
        assert_eq!(
            payload.batch[0]
                .properties
                .get("$process_person_profile")
                .map(String::as_str),
            Some("false")
        );
        assert_eq!(
            analytics_batch_url("https://us.i.posthog.com/"),
            "https://us.i.posthog.com/batch/"
        );
        assert!(
            analytics_batch_payload(
                &TelemetryEnvConfig {
                    enabled: false,
                    ..default_telemetry_env_config()
                },
                &buffer,
                "hashed-id",
                "linux",
                None,
                "arm64",
                "0.0.23",
                "desktop",
            )
            .is_none()
        );
    }
}
