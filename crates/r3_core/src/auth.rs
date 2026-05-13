use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

pub const SESSION_COOKIE_NAME: &str = "t3_session";
pub const PAIRING_TOKEN_ALPHABET: &str = "23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
pub const PAIRING_TOKEN_LENGTH: usize = 12;
pub const DEFAULT_SESSION_TTL_MS: i64 = 30 * 24 * 60 * 60 * 1000;
pub const DEFAULT_WEBSOCKET_TOKEN_TTL_MS: i64 = 5 * 60 * 1000;
pub const AUTHORIZATION_PREFIX: &str = "Bearer ";
pub const WEBSOCKET_TOKEN_QUERY_PARAM: &str = "wsToken";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerMode {
    Web,
    Desktop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerAuthPolicyKind {
    DesktopManagedLocal,
    LoopbackBrowser,
    RemoteReachable,
    UnsafeNoAuth,
}

impl ServerAuthPolicyKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DesktopManagedLocal => "desktop-managed-local",
            Self::LoopbackBrowser => "loopback-browser",
            Self::RemoteReachable => "remote-reachable",
            Self::UnsafeNoAuth => "unsafe-no-auth",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerAuthBootstrapMethod {
    DesktopBootstrap,
    OneTimeToken,
}

impl ServerAuthBootstrapMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DesktopBootstrap => "desktop-bootstrap",
            Self::OneTimeToken => "one-time-token",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerAuthSessionMethod {
    BrowserSessionCookie,
    BearerSessionToken,
}

impl ServerAuthSessionMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BrowserSessionCookie => "browser-session-cookie",
            Self::BearerSessionToken => "bearer-session-token",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthSessionRole {
    Owner,
    Client,
}

impl AuthSessionRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Client => "client",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthClientMetadataDeviceType {
    Desktop,
    Mobile,
    Tablet,
    Bot,
    Unknown,
}

impl AuthClientMetadataDeviceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Mobile => "mobile",
            Self::Tablet => "tablet",
            Self::Bot => "bot",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerAuthDescriptor {
    pub policy: ServerAuthPolicyKind,
    pub bootstrap_methods: Vec<ServerAuthBootstrapMethod>,
    pub session_methods: Vec<ServerAuthSessionMethod>,
    pub session_cookie_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthClientMetadata {
    pub label: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: AuthClientMetadataDeviceType,
    pub os: Option<String>,
    pub browser: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthPairingLink {
    pub id: String,
    pub credential: String,
    pub role: AuthSessionRole,
    pub subject: String,
    pub label: Option<String>,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthClientSession {
    pub session_id: String,
    pub subject: String,
    pub role: AuthSessionRole,
    pub method: ServerAuthSessionMethod,
    pub client: AuthClientMetadata,
    pub issued_at: String,
    pub expires_at: String,
    pub last_connected_at: Option<String>,
    pub connected: bool,
    pub current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthAccessSnapshot {
    pub pairing_links: Vec<AuthPairingLink>,
    pub client_sessions: Vec<AuthClientSession>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthAccessStreamEvent {
    Snapshot {
        revision: u64,
        payload: AuthAccessSnapshot,
    },
    PairingLinkUpserted {
        revision: u64,
        payload: AuthPairingLink,
    },
    PairingLinkRemoved {
        revision: u64,
        id: String,
    },
    ClientUpserted {
        revision: u64,
        payload: AuthClientSession,
    },
    ClientRemoved {
        revision: u64,
        session_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapCredentialChange {
    PairingLinkUpserted { pairing_link: AuthPairingLink },
    PairingLinkRemoved { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCredentialChange {
    ClientUpserted { client_session: AuthClientSession },
    ClientRemoved { session_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthAccessChange {
    Bootstrap(BootstrapCredentialChange),
    Session(SessionCredentialChange),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConnectedSessionCounts {
    pub counts: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedSessionTransition {
    pub counts: ConnectedSessionCounts,
    pub was_disconnected: bool,
    pub should_set_last_connected_at: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionClaims {
    pub session_id: String,
    pub subject: String,
    pub role: AuthSessionRole,
    pub method: ServerAuthSessionMethod,
    pub issued_at_ms: i64,
    pub expires_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketClaims {
    pub session_id: String,
    pub issued_at_ms: i64,
    pub expires_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSessionCredentialRecord {
    pub session_id: String,
    pub subject: String,
    pub role: AuthSessionRole,
    pub method: ServerAuthSessionMethod,
    pub client: AuthClientMetadata,
    pub issued_at_ms: i64,
    pub expires_at_ms: i64,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCredentialIssueInput {
    pub session_id: String,
    pub issued_at_ms: i64,
    pub ttl_ms: Option<i64>,
    pub subject: Option<String>,
    pub method: Option<ServerAuthSessionMethod>,
    pub role: Option<AuthSessionRole>,
    pub client: Option<AuthClientMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedSessionCredential {
    pub session_id: String,
    pub token: String,
    pub method: ServerAuthSessionMethod,
    pub client: AuthClientMetadata,
    pub expires_at_ms: i64,
    pub role: AuthSessionRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCredentialIssuePlan {
    pub issued: IssuedSessionCredential,
    pub claims: SessionClaims,
    pub persisted_session: AuthSessionCredentialRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketCredentialIssueInput {
    pub session_id: String,
    pub issued_at_ms: i64,
    pub ttl_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedWebSocketCredential {
    pub token: String,
    pub expires_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedSessionCredential {
    pub session_id: String,
    pub token: String,
    pub method: ServerAuthSessionMethod,
    pub client: AuthClientMetadata,
    pub expires_at_ms: Option<i64>,
    pub subject: String,
    pub role: AuthSessionRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedTokenParts {
    pub encoded_payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretStorePathPlan {
    pub secrets_dir: String,
    pub secret_path: String,
    pub temp_path: String,
    pub directory_mode: u32,
    pub file_mode: u32,
    pub make_directory_recursive: bool,
    pub write_sequence: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretStoreReadDecision {
    MissingReturnsNull,
    Fail { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretStoreConcurrentCreateDecision {
    ReturnCreatedSecret,
    FailAfterMissingConcurrentCreate { message: String },
    PropagatePersistError { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSecretStoreError {
    pub message: String,
}

impl From<String> for FileSecretStoreError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedAuthSessionVerificationInput {
    pub session_id: String,
    pub expires_at_ms: i64,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistedAuthSessionVerification {
    Accepted,
    Unknown { message: &'static str },
    Expired { message: &'static str },
    Revoked { message: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthTokenVerificationError {
    Token(AuthError),
    Persisted(PersistedAuthSessionVerification),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthCredentialSelection {
    Credential(String),
    Missing { message: &'static str, status: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketUpgradeCredentialSelection {
    WebSocketToken(String),
    Fallback(AuthCredentialSelection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthOwnerAction {
    CreatePairingCredential,
    ManageNetworkAccess,
    RevokeClientSession,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthAccessDecision {
    Allowed,
    Denied { message: &'static str, status: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapCredentialErrorInput {
    pub status: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapCredentialAuthError {
    pub message: &'static str,
    pub status: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    MalformedToken,
    InvalidBase64,
    InvalidUtf8,
    InvalidSignature,
    InvalidClaims,
    ExpiredToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthHttpRoutePlan {
    pub layer: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub auth: &'static str,
    pub request_schema: Option<&'static str>,
    pub success_response: &'static str,
    pub success_status: u16,
    pub sets_session_cookie: bool,
    pub cors_headers: bool,
    pub invalid_payload_message: Option<&'static str>,
}

pub const AUTH_HTTP_ROUTE_COUNT: usize = 10;

pub fn auth_http_route_plans() -> Vec<AuthHttpRoutePlan> {
    vec![
        AuthHttpRoutePlan {
            layer: "authSessionRouteLayer",
            method: "GET",
            path: "/api/auth/session",
            auth: "session-state",
            request_schema: None,
            success_response: "serverAuth.getSessionState(request)",
            success_status: 200,
            sets_session_cookie: false,
            cors_headers: true,
            invalid_payload_message: None,
        },
        AuthHttpRoutePlan {
            layer: "authBootstrapRouteLayer",
            method: "POST",
            path: "/api/auth/bootstrap",
            auth: "bootstrap-credential",
            request_schema: Some("AuthBootstrapInput"),
            success_response: "AuthBootstrapResult response body",
            success_status: 200,
            sets_session_cookie: true,
            cors_headers: true,
            invalid_payload_message: Some("Invalid bootstrap payload."),
        },
        AuthHttpRoutePlan {
            layer: "authBearerBootstrapRouteLayer",
            method: "POST",
            path: "/api/auth/bootstrap/bearer",
            auth: "bootstrap-credential",
            request_schema: Some("AuthBootstrapInput"),
            success_response: "AuthBearerBootstrapResult",
            success_status: 200,
            sets_session_cookie: false,
            cors_headers: true,
            invalid_payload_message: Some("Invalid bootstrap payload."),
        },
        AuthHttpRoutePlan {
            layer: "authWebSocketTokenRouteLayer",
            method: "POST",
            path: "/api/auth/ws-token",
            auth: "authenticated-session",
            request_schema: None,
            success_response: "AuthWebSocketTokenResult",
            success_status: 200,
            sets_session_cookie: false,
            cors_headers: true,
            invalid_payload_message: None,
        },
        AuthHttpRoutePlan {
            layer: "authPairingCredentialRouteLayer",
            method: "POST",
            path: "/api/auth/pairing-token",
            auth: "owner-session",
            request_schema: Some("optional AuthCreatePairingCredentialInput"),
            success_response: "Issued pairing credential",
            success_status: 200,
            sets_session_cookie: false,
            cors_headers: false,
            invalid_payload_message: Some("Invalid pairing credential payload."),
        },
        AuthHttpRoutePlan {
            layer: "authPairingLinksRouteLayer",
            method: "GET",
            path: "/api/auth/pairing-links",
            auth: "owner-session",
            request_schema: None,
            success_response: "serverAuth.listPairingLinks()",
            success_status: 200,
            sets_session_cookie: false,
            cors_headers: false,
            invalid_payload_message: None,
        },
        AuthHttpRoutePlan {
            layer: "authPairingLinksRevokeRouteLayer",
            method: "POST",
            path: "/api/auth/pairing-links/revoke",
            auth: "owner-session",
            request_schema: Some("AuthRevokePairingLinkInput"),
            success_response: "{ revoked }",
            success_status: 200,
            sets_session_cookie: false,
            cors_headers: false,
            invalid_payload_message: Some("Invalid revoke pairing link payload."),
        },
        AuthHttpRoutePlan {
            layer: "authClientsRouteLayer",
            method: "GET",
            path: "/api/auth/clients",
            auth: "owner-session",
            request_schema: None,
            success_response: "serverAuth.listClientSessions(currentSessionId)",
            success_status: 200,
            sets_session_cookie: false,
            cors_headers: false,
            invalid_payload_message: None,
        },
        AuthHttpRoutePlan {
            layer: "authClientsRevokeRouteLayer",
            method: "POST",
            path: "/api/auth/clients/revoke",
            auth: "owner-session",
            request_schema: Some("AuthRevokeClientSessionInput"),
            success_response: "{ revoked }",
            success_status: 200,
            sets_session_cookie: false,
            cors_headers: false,
            invalid_payload_message: Some("Invalid revoke client payload."),
        },
        AuthHttpRoutePlan {
            layer: "authClientsRevokeOthersRouteLayer",
            method: "POST",
            path: "/api/auth/clients/revoke-others",
            auth: "owner-session",
            request_schema: None,
            success_response: "{ revokedCount }",
            success_status: 200,
            sets_session_cookie: false,
            cors_headers: false,
            invalid_payload_message: None,
        },
    ]
}

pub fn resolve_session_cookie_name(mode: ServerMode, port: u16) -> String {
    match mode {
        ServerMode::Desktop => format!("{SESSION_COOKIE_NAME}_{port}"),
        ServerMode::Web => SESSION_COOKIE_NAME.to_string(),
    }
}

pub fn parse_bearer_token(authorization_header: Option<&str>) -> Option<String> {
    let header = authorization_header?;
    if !header.starts_with(AUTHORIZATION_PREFIX) {
        return None;
    }
    let token = header[AUTHORIZATION_PREFIX.len()..].trim();
    (!token.is_empty()).then(|| token.to_string())
}

pub fn select_http_auth_credential(
    cookie_token: Option<&str>,
    authorization_header: Option<&str>,
) -> AuthCredentialSelection {
    cookie_token
        .filter(|token| !token.trim().is_empty())
        .map(|token| AuthCredentialSelection::Credential(token.to_string()))
        .or_else(|| {
            parse_bearer_token(authorization_header).map(AuthCredentialSelection::Credential)
        })
        .unwrap_or(AuthCredentialSelection::Missing {
            message: "Authentication required.",
            status: 401,
        })
}

pub fn select_websocket_upgrade_credential(
    request_url: Option<&str>,
    cookie_token: Option<&str>,
    authorization_header: Option<&str>,
) -> WebSocketUpgradeCredentialSelection {
    if let Some(token) = request_url.and_then(extract_websocket_query_token) {
        return WebSocketUpgradeCredentialSelection::WebSocketToken(token);
    }
    WebSocketUpgradeCredentialSelection::Fallback(select_http_auth_credential(
        cookie_token,
        authorization_header,
    ))
}

pub fn map_bootstrap_credential_auth_error(
    cause: BootstrapCredentialErrorInput,
) -> BootstrapCredentialAuthError {
    if cause.status == 500 {
        BootstrapCredentialAuthError {
            message: "Failed to validate bootstrap credential.",
            status: 500,
        }
    } else {
        BootstrapCredentialAuthError {
            message: "Invalid bootstrap credential.",
            status: 401,
        }
    }
}

pub fn require_owner_session(
    role: AuthSessionRole,
    action: AuthOwnerAction,
    current_session_id: Option<&str>,
    target_session_id: Option<&str>,
) -> AuthAccessDecision {
    if action == AuthOwnerAction::RevokeClientSession && current_session_id == target_session_id {
        return AuthAccessDecision::Denied {
            message: "Use revoke other clients to keep the current owner session active.",
            status: 403,
        };
    }
    if role == AuthSessionRole::Owner {
        return AuthAccessDecision::Allowed;
    }
    AuthAccessDecision::Denied {
        message: match action {
            AuthOwnerAction::CreatePairingCredential => {
                "Only owner sessions can create pairing credentials."
            }
            AuthOwnerAction::ManageNetworkAccess | AuthOwnerAction::RevokeClientSession => {
                "Only owner sessions can manage network access."
            }
        },
        status: 403,
    }
}

pub fn auth_pairing_request_has_body(
    content_length: Option<&str>,
    transfer_encoding: Option<&str>,
) -> bool {
    if let Some(header) = content_length {
        if let Ok(length) = header.parse::<u64>() {
            return length > 0;
        }
    }
    transfer_encoding.is_some()
}

pub fn resolve_secret_path(secrets_dir: &str, name: &str) -> String {
    let separator = if secrets_dir.ends_with('/') || secrets_dir.ends_with('\\') {
        ""
    } else {
        "/"
    };
    format!("{secrets_dir}{separator}{name}.bin")
}

pub fn secret_store_path_plan(secrets_dir: &str, name: &str, temp_id: &str) -> SecretStorePathPlan {
    let secret_path = resolve_secret_path(secrets_dir, name);
    SecretStorePathPlan {
        secrets_dir: secrets_dir.to_string(),
        temp_path: format!("{secret_path}.{temp_id}.tmp"),
        secret_path,
        directory_mode: 0o700,
        file_mode: 0o600,
        make_directory_recursive: true,
        write_sequence: vec![
            "makeDirectory(secretsDir, recursive)",
            "chmod(secretsDir, 0o700)",
            "writeFile(tempPath, value)",
            "chmod(tempPath, 0o600)",
            "rename(tempPath, secretPath)",
            "chmod(secretPath, 0o600)",
        ],
    }
}

pub fn secret_store_read_decision(
    name: &str,
    platform_error_tag: Option<&str>,
) -> SecretStoreReadDecision {
    match platform_error_tag {
        Some("NotFound") => SecretStoreReadDecision::MissingReturnsNull,
        Some(_) => SecretStoreReadDecision::Fail {
            message: format!("Failed to read secret {name}."),
        },
        None => SecretStoreReadDecision::Fail {
            message: format!("Failed to read secret {name}."),
        },
    }
}

pub fn secret_store_concurrent_create_decision(
    name: &str,
    persist_error_tag: Option<&str>,
    read_after_create_found: bool,
) -> SecretStoreConcurrentCreateDecision {
    match (persist_error_tag, read_after_create_found) {
        (Some("AlreadyExists"), true) => SecretStoreConcurrentCreateDecision::ReturnCreatedSecret,
        (Some("AlreadyExists"), false) => {
            SecretStoreConcurrentCreateDecision::FailAfterMissingConcurrentCreate {
                message: format!("Failed to read secret {name} after concurrent creation."),
            }
        }
        _ => SecretStoreConcurrentCreateDecision::PropagatePersistError {
            message: format!("Failed to persist secret {name}."),
        },
    }
}

pub fn read_file_secret(
    secrets_dir: impl AsRef<Path>,
    name: &str,
) -> Result<Option<Vec<u8>>, FileSecretStoreError> {
    let secret_path = secrets_dir.as_ref().join(format!("{name}.bin"));
    match fs::read(&secret_path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(FileSecretStoreError {
            message: format!("Failed to read secret {name}."),
        }),
    }
}

pub fn write_file_secret(
    secrets_dir: impl AsRef<Path>,
    name: &str,
    value: &[u8],
    temp_id: &str,
) -> Result<(), FileSecretStoreError> {
    let secrets_dir = secrets_dir.as_ref();
    let secret_path = secrets_dir.join(format!("{name}.bin"));
    let temp_path = secrets_dir.join(format!("{name}.bin.{temp_id}.tmp"));
    fs::create_dir_all(secrets_dir).map_err(|_| FileSecretStoreError {
        message: format!(
            "Failed to secure secrets directory {}.",
            secrets_dir.display()
        ),
    })?;
    fs::write(&temp_path, value).map_err(|_| FileSecretStoreError {
        message: format!("Failed to persist secret {name}."),
    })?;
    if let Err(error) = fs::rename(&temp_path, &secret_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(FileSecretStoreError {
            message: if error.kind() == io::ErrorKind::AlreadyExists {
                format!("Failed to persist secret {name}.")
            } else {
                format!("Failed to persist secret {name}.")
            },
        });
    }
    Ok(())
}

pub fn get_or_create_file_secret(
    secrets_dir: impl AsRef<Path>,
    name: &str,
    generated_value: &[u8],
) -> Result<Vec<u8>, FileSecretStoreError> {
    let secrets_dir = secrets_dir.as_ref();
    if let Some(existing) = read_file_secret(secrets_dir, name)? {
        return Ok(existing);
    }

    fs::create_dir_all(secrets_dir).map_err(|_| FileSecretStoreError {
        message: format!(
            "Failed to secure secrets directory {}.",
            secrets_dir.display()
        ),
    })?;
    let secret_path = secrets_dir.join(format!("{name}.bin"));
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&secret_path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return read_file_secret(secrets_dir, name)?.ok_or_else(|| FileSecretStoreError {
                message: format!("Failed to read secret {name} after concurrent creation."),
            });
        }
        Err(_) => {
            return Err(FileSecretStoreError {
                message: format!("Failed to persist secret {name}."),
            });
        }
    };
    file.write_all(generated_value)
        .and_then(|()| file.sync_all())
        .map_err(|_| FileSecretStoreError {
            message: format!("Failed to persist secret {name}."),
        })?;
    Ok(generated_value.to_vec())
}

pub fn remove_file_secret(
    secrets_dir: impl AsRef<Path>,
    name: &str,
) -> Result<(), FileSecretStoreError> {
    let secret_path = secrets_dir.as_ref().join(format!("{name}.bin"));
    match fs::remove_file(&secret_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(FileSecretStoreError {
            message: format!("Failed to remove secret {name}."),
        }),
    }
}

pub fn verify_persisted_session_state(
    row: Option<&PersistedAuthSessionVerificationInput>,
    now_ms: i64,
    websocket: bool,
) -> PersistedAuthSessionVerification {
    let Some(row) = row else {
        return PersistedAuthSessionVerification::Unknown {
            message: if websocket {
                "Unknown websocket session."
            } else {
                "Unknown session token."
            },
        };
    };
    if websocket && row.expires_at_ms <= now_ms {
        return PersistedAuthSessionVerification::Expired {
            message: "Websocket session expired.",
        };
    }
    if row.revoked_at.is_some() {
        return PersistedAuthSessionVerification::Revoked {
            message: if websocket {
                "Websocket session revoked."
            } else {
                "Session token revoked."
            },
        };
    }
    PersistedAuthSessionVerification::Accepted
}

pub fn default_auth_client_metadata() -> AuthClientMetadata {
    AuthClientMetadata {
        label: None,
        ip_address: None,
        user_agent: None,
        device_type: AuthClientMetadataDeviceType::Unknown,
        os: None,
        browser: None,
    }
}

pub fn issue_session_credential_plan(
    input: SessionCredentialIssueInput,
    secret: &[u8],
) -> SessionCredentialIssuePlan {
    let expires_at_ms = input.issued_at_ms + input.ttl_ms.unwrap_or(DEFAULT_SESSION_TTL_MS);
    let subject = input.subject.unwrap_or_else(|| "browser".to_string());
    let role = input.role.unwrap_or(AuthSessionRole::Client);
    let method = input
        .method
        .unwrap_or(ServerAuthSessionMethod::BrowserSessionCookie);
    let client = input.client.unwrap_or_else(default_auth_client_metadata);
    let claims = SessionClaims {
        session_id: input.session_id,
        subject: subject.clone(),
        role,
        method,
        issued_at_ms: input.issued_at_ms,
        expires_at_ms,
    };
    let token = sign_session_claims(&claims, secret);
    SessionCredentialIssuePlan {
        issued: IssuedSessionCredential {
            session_id: claims.session_id.clone(),
            token,
            method,
            client: client.clone(),
            expires_at_ms,
            role,
        },
        persisted_session: AuthSessionCredentialRecord {
            session_id: claims.session_id.clone(),
            subject,
            role,
            method,
            client,
            issued_at_ms: claims.issued_at_ms,
            expires_at_ms,
            revoked_at: None,
        },
        claims,
    }
}

pub fn issue_websocket_credential(
    input: WebSocketCredentialIssueInput,
    secret: &[u8],
) -> IssuedWebSocketCredential {
    let expires_at_ms = input.issued_at_ms + input.ttl_ms.unwrap_or(DEFAULT_WEBSOCKET_TOKEN_TTL_MS);
    let claims = WebSocketClaims {
        session_id: input.session_id,
        issued_at_ms: input.issued_at_ms,
        expires_at_ms,
    };
    IssuedWebSocketCredential {
        token: sign_websocket_claims(&claims, secret),
        expires_at_ms,
    }
}

pub fn is_loopback_host(host: Option<&str>) -> bool {
    let host = host.unwrap_or_default();
    host.is_empty()
        || host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host == "[::1]"
        || host.starts_with("127.")
}

pub fn is_wildcard_host(host: Option<&str>) -> bool {
    matches!(host, Some("0.0.0.0" | "::" | "[::]"))
}

pub fn make_server_auth_descriptor(
    mode: ServerMode,
    host: Option<&str>,
    port: u16,
) -> ServerAuthDescriptor {
    let is_remote_reachable = is_wildcard_host(host) || !is_loopback_host(host);
    let policy = match (mode, is_remote_reachable) {
        (ServerMode::Desktop, false) => ServerAuthPolicyKind::DesktopManagedLocal,
        (ServerMode::Web, false) => ServerAuthPolicyKind::LoopbackBrowser,
        (_, true) => ServerAuthPolicyKind::RemoteReachable,
    };

    let bootstrap_methods = match (mode, policy) {
        (ServerMode::Desktop, ServerAuthPolicyKind::DesktopManagedLocal) => {
            vec![ServerAuthBootstrapMethod::DesktopBootstrap]
        }
        (ServerMode::Desktop, ServerAuthPolicyKind::RemoteReachable) => vec![
            ServerAuthBootstrapMethod::DesktopBootstrap,
            ServerAuthBootstrapMethod::OneTimeToken,
        ],
        _ => vec![ServerAuthBootstrapMethod::OneTimeToken],
    };

    ServerAuthDescriptor {
        policy,
        bootstrap_methods,
        session_methods: vec![
            ServerAuthSessionMethod::BrowserSessionCookie,
            ServerAuthSessionMethod::BearerSessionToken,
        ],
        session_cookie_name: resolve_session_cookie_name(mode, port),
    }
}

pub fn normalize_ip_address(value: Option<&str>) -> Option<String> {
    let normalized = normalize_non_empty_string(value)?;
    normalized
        .strip_prefix("::ffff:")
        .map(ToString::to_string)
        .or(Some(normalized))
}

pub fn derive_auth_client_metadata(
    user_agent: Option<&str>,
    remote_address: Option<&str>,
    label: Option<&str>,
) -> AuthClientMetadata {
    let user_agent = normalize_non_empty_string(user_agent);
    let ip_address = normalize_ip_address(remote_address);
    AuthClientMetadata {
        label: normalize_non_empty_string(label),
        ip_address,
        os: infer_os(user_agent.as_deref()).map(ToString::to_string),
        browser: infer_browser(user_agent.as_deref()).map(ToString::to_string),
        device_type: infer_device_type(user_agent.as_deref()),
        user_agent,
    }
}

pub fn infer_device_type(user_agent: Option<&str>) -> AuthClientMetadataDeviceType {
    let Some(normalized) = user_agent.map(|value| value.to_ascii_lowercase()) else {
        return AuthClientMetadataDeviceType::Unknown;
    };
    if contains_any(
        &normalized,
        &["bot", "crawler", "spider", "slurp", "curl", "wget"],
    ) {
        AuthClientMetadataDeviceType::Bot
    } else if contains_any(&normalized, &["ipad", "tablet"]) {
        AuthClientMetadataDeviceType::Tablet
    } else if normalized.contains("iphone")
        || normalized.contains("mobile")
        || (normalized.contains("android") && normalized.contains("mobile"))
    {
        AuthClientMetadataDeviceType::Mobile
    } else {
        AuthClientMetadataDeviceType::Desktop
    }
}

pub fn infer_browser(user_agent: Option<&str>) -> Option<&'static str> {
    let normalized = user_agent?.to_ascii_lowercase();
    if normalized.contains("edg/") {
        Some("Edge")
    } else if normalized.contains("opr/") {
        Some("Opera")
    } else if normalized.contains("firefox/") {
        Some("Firefox")
    } else if normalized.contains("electron/") {
        Some("Electron")
    } else if normalized.contains("chrome/") || normalized.contains("crios/") {
        Some("Chrome")
    } else if normalized.contains("safari/") && !normalized.contains("chrome/") {
        Some("Safari")
    } else {
        None
    }
}

pub fn infer_os(user_agent: Option<&str>) -> Option<&'static str> {
    let normalized = user_agent?.to_ascii_lowercase();
    if contains_any(&normalized, &["iphone", "ipad", "ipod"]) {
        Some("iOS")
    } else if normalized.contains("android") {
        Some("Android")
    } else if normalized.contains("mac os x") || normalized.contains("macintosh") {
        Some("macOS")
    } else if normalized.contains("windows nt") {
        Some("Windows")
    } else if normalized.contains("linux") {
        Some("Linux")
    } else {
        None
    }
}

pub fn pairing_token_from_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(PAIRING_TOKEN_LENGTH)
        .map(|byte| {
            PAIRING_TOKEN_ALPHABET
                .as_bytes()
                .get((byte & 31) as usize)
                .copied()
                .unwrap_or(b'2') as char
        })
        .collect()
}

pub fn encode_session_claims_payload(claims: &SessionClaims) -> String {
    base64_url_encode(
        format!(
            "{{\"v\":1,\"kind\":\"session\",\"sid\":{},\"sub\":{},\"role\":\"{}\",\"method\":\"{}\",\"iat\":{},\"exp\":{}}}",
            json_string(&claims.session_id),
            json_string(&claims.subject),
            claims.role.as_str(),
            claims.method.as_str(),
            claims.issued_at_ms,
            claims.expires_at_ms,
        )
        .as_bytes(),
    )
}

pub fn encode_websocket_claims_payload(claims: &WebSocketClaims) -> String {
    base64_url_encode(
        format!(
            "{{\"v\":1,\"kind\":\"websocket\",\"sid\":{},\"iat\":{},\"exp\":{}}}",
            json_string(&claims.session_id),
            claims.issued_at_ms,
            claims.expires_at_ms,
        )
        .as_bytes(),
    )
}

pub fn join_signed_token(parts: &SignedTokenParts) -> String {
    format!("{}.{}", parts.encoded_payload, parts.signature)
}

pub fn sign_payload(payload: &str, secret: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts keys of any length");
    mac.update(payload.as_bytes());
    base64_url_encode(&mac.finalize().into_bytes())
}

pub fn verify_payload_signature(payload: &str, signature: &str, secret: &[u8]) -> bool {
    let Ok(signature_bytes) = base64_url_decode(signature) else {
        return false;
    };
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts keys of any length");
    mac.update(payload.as_bytes());
    mac.verify_slice(&signature_bytes).is_ok()
}

pub fn sign_session_claims(claims: &SessionClaims, secret: &[u8]) -> String {
    let encoded_payload = encode_session_claims_payload(claims);
    join_signed_token(&SignedTokenParts {
        signature: sign_payload(&encoded_payload, secret),
        encoded_payload,
    })
}

pub fn sign_websocket_claims(claims: &WebSocketClaims, secret: &[u8]) -> String {
    let encoded_payload = encode_websocket_claims_payload(claims);
    join_signed_token(&SignedTokenParts {
        signature: sign_payload(&encoded_payload, secret),
        encoded_payload,
    })
}

pub fn verify_signed_token_signature(
    token: &str,
    secret: &[u8],
) -> Result<SignedTokenParts, AuthError> {
    let parts = split_signed_token(token)?;
    if verify_payload_signature(&parts.encoded_payload, &parts.signature, secret) {
        Ok(parts)
    } else {
        Err(AuthError::InvalidSignature)
    }
}

pub fn decode_session_claims_payload(encoded_payload: &str) -> Result<SessionClaims, AuthError> {
    let decoded = base64_url_decode_utf8(encoded_payload)?;
    let value: Value = serde_json::from_str(&decoded).map_err(|_| AuthError::InvalidClaims)?;
    if read_u64_claim(&value, "v")? != 1 || read_string_claim(&value, "kind")? != "session" {
        return Err(AuthError::InvalidClaims);
    }
    Ok(SessionClaims {
        session_id: read_string_claim(&value, "sid")?.to_string(),
        subject: read_string_claim(&value, "sub")?.to_string(),
        role: parse_auth_session_role(read_string_claim(&value, "role")?)?,
        method: parse_auth_session_method(read_string_claim(&value, "method")?)?,
        issued_at_ms: read_i64_claim(&value, "iat")?,
        expires_at_ms: read_i64_claim(&value, "exp")?,
    })
}

pub fn decode_websocket_claims_payload(
    encoded_payload: &str,
) -> Result<WebSocketClaims, AuthError> {
    let decoded = base64_url_decode_utf8(encoded_payload)?;
    let value: Value = serde_json::from_str(&decoded).map_err(|_| AuthError::InvalidClaims)?;
    if read_u64_claim(&value, "v")? != 1 || read_string_claim(&value, "kind")? != "websocket" {
        return Err(AuthError::InvalidClaims);
    }
    Ok(WebSocketClaims {
        session_id: read_string_claim(&value, "sid")?.to_string(),
        issued_at_ms: read_i64_claim(&value, "iat")?,
        expires_at_ms: read_i64_claim(&value, "exp")?,
    })
}

pub fn verify_signed_session_claims(
    token: &str,
    secret: &[u8],
    now_ms: i64,
) -> Result<SessionClaims, AuthError> {
    let parts = verify_signed_token_signature(token, secret)?;
    let claims = decode_session_claims_payload(&parts.encoded_payload)?;
    if claims.expires_at_ms <= now_ms {
        return Err(AuthError::ExpiredToken);
    }
    Ok(claims)
}

pub fn verify_signed_websocket_claims(
    token: &str,
    secret: &[u8],
    now_ms: i64,
) -> Result<WebSocketClaims, AuthError> {
    let parts = verify_signed_token_signature(token, secret)?;
    let claims = decode_websocket_claims_payload(&parts.encoded_payload)?;
    if claims.expires_at_ms <= now_ms {
        return Err(AuthError::ExpiredToken);
    }
    Ok(claims)
}

pub fn verify_signed_session_against_persisted_state(
    token: &str,
    secret: &[u8],
    now_ms: i64,
    row: Option<&PersistedAuthSessionVerificationInput>,
) -> Result<SessionClaims, AuthTokenVerificationError> {
    let claims = verify_signed_session_claims(token, secret, now_ms)
        .map_err(AuthTokenVerificationError::Token)?;
    let row = row.filter(|row| row.session_id == claims.session_id);
    match verify_persisted_session_state(row, now_ms, false) {
        PersistedAuthSessionVerification::Accepted => Ok(claims),
        decision => Err(AuthTokenVerificationError::Persisted(decision)),
    }
}

pub fn verify_signed_websocket_against_persisted_state(
    token: &str,
    secret: &[u8],
    now_ms: i64,
    row: Option<&PersistedAuthSessionVerificationInput>,
) -> Result<WebSocketClaims, AuthTokenVerificationError> {
    let claims = verify_signed_websocket_claims(token, secret, now_ms)
        .map_err(AuthTokenVerificationError::Token)?;
    let row = row.filter(|row| row.session_id == claims.session_id);
    match verify_persisted_session_state(row, now_ms, true) {
        PersistedAuthSessionVerification::Accepted => Ok(claims),
        decision => Err(AuthTokenVerificationError::Persisted(decision)),
    }
}

pub fn verify_session_credential_record(
    token: &str,
    secret: &[u8],
    now_ms: i64,
    row: Option<&AuthSessionCredentialRecord>,
) -> Result<VerifiedSessionCredential, AuthTokenVerificationError> {
    let claims = verify_signed_session_claims(token, secret, now_ms)
        .map_err(AuthTokenVerificationError::Token)?;
    let row = row.filter(|row| row.session_id == claims.session_id);
    let persisted = row.map(persisted_auth_session_input_from_credential_record);
    match verify_persisted_session_state(persisted.as_ref(), now_ms, false) {
        PersistedAuthSessionVerification::Accepted => {
            let row = row.expect("accepted persisted session has a row");
            Ok(VerifiedSessionCredential {
                session_id: claims.session_id,
                token: token.to_string(),
                method: claims.method,
                client: row.client.clone(),
                expires_at_ms: Some(claims.expires_at_ms),
                subject: claims.subject,
                role: claims.role,
            })
        }
        decision => Err(AuthTokenVerificationError::Persisted(decision)),
    }
}

pub fn verify_websocket_credential_record(
    token: &str,
    secret: &[u8],
    now_ms: i64,
    row: Option<&AuthSessionCredentialRecord>,
) -> Result<VerifiedSessionCredential, AuthTokenVerificationError> {
    let claims = verify_signed_websocket_claims(token, secret, now_ms)
        .map_err(AuthTokenVerificationError::Token)?;
    let row = row.filter(|row| row.session_id == claims.session_id);
    let persisted = row.map(persisted_auth_session_input_from_credential_record);
    match verify_persisted_session_state(persisted.as_ref(), now_ms, true) {
        PersistedAuthSessionVerification::Accepted => {
            let row = row.expect("accepted persisted session has a row");
            Ok(VerifiedSessionCredential {
                session_id: row.session_id.clone(),
                token: token.to_string(),
                method: row.method,
                client: row.client.clone(),
                expires_at_ms: Some(row.expires_at_ms),
                subject: row.subject.clone(),
                role: row.role,
            })
        }
        decision => Err(AuthTokenVerificationError::Persisted(decision)),
    }
}

pub fn split_signed_token(token: &str) -> Result<SignedTokenParts, AuthError> {
    let mut parts = token.split('.');
    let Some(encoded_payload) = parts.next().filter(|part| !part.is_empty()) else {
        return Err(AuthError::MalformedToken);
    };
    let Some(signature) = parts.next().filter(|part| !part.is_empty()) else {
        return Err(AuthError::MalformedToken);
    };
    if parts.next().is_some() {
        return Err(AuthError::MalformedToken);
    }
    Ok(SignedTokenParts {
        encoded_payload: encoded_payload.to_string(),
        signature: signature.to_string(),
    })
}

pub fn base64_url_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::new();
    let mut index = 0;
    while index < input.len() {
        let b0 = input[index];
        let b1 = input.get(index + 1).copied().unwrap_or(0);
        let b2 = input.get(index + 2).copied().unwrap_or(0);
        output.push(TABLE[(b0 >> 2) as usize] as char);
        output.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if index + 1 < input.len() {
            output.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        }
        if index + 2 < input.len() {
            output.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        }
        index += 3;
    }
    output
}

pub fn base64_url_decode_utf8(input: &str) -> Result<String, AuthError> {
    let bytes = base64_url_decode(input)?;
    String::from_utf8(bytes).map_err(|_| AuthError::InvalidUtf8)
}

pub fn auth_access_event_type(event: &AuthAccessStreamEvent) -> &'static str {
    match event {
        AuthAccessStreamEvent::Snapshot { .. } => "snapshot",
        AuthAccessStreamEvent::PairingLinkUpserted { .. } => "pairingLinkUpserted",
        AuthAccessStreamEvent::PairingLinkRemoved { .. } => "pairingLinkRemoved",
        AuthAccessStreamEvent::ClientUpserted { .. } => "clientUpserted",
        AuthAccessStreamEvent::ClientRemoved { .. } => "clientRemoved",
    }
}

pub fn auth_access_event_revision(event: &AuthAccessStreamEvent) -> u64 {
    match event {
        AuthAccessStreamEvent::Snapshot { revision, .. }
        | AuthAccessStreamEvent::PairingLinkUpserted { revision, .. }
        | AuthAccessStreamEvent::PairingLinkRemoved { revision, .. }
        | AuthAccessStreamEvent::ClientUpserted { revision, .. }
        | AuthAccessStreamEvent::ClientRemoved { revision, .. } => *revision,
    }
}

pub fn auth_access_stream_event_from_change(
    change: AuthAccessChange,
    revision: u64,
    current_session_id: &str,
) -> AuthAccessStreamEvent {
    match change {
        AuthAccessChange::Bootstrap(BootstrapCredentialChange::PairingLinkUpserted {
            pairing_link,
        }) => AuthAccessStreamEvent::PairingLinkUpserted {
            revision,
            payload: pairing_link,
        },
        AuthAccessChange::Bootstrap(BootstrapCredentialChange::PairingLinkRemoved { id }) => {
            AuthAccessStreamEvent::PairingLinkRemoved { revision, id }
        }
        AuthAccessChange::Session(SessionCredentialChange::ClientUpserted {
            mut client_session,
        }) => {
            client_session.current = client_session.session_id == current_session_id;
            AuthAccessStreamEvent::ClientUpserted {
                revision,
                payload: client_session,
            }
        }
        AuthAccessChange::Session(SessionCredentialChange::ClientRemoved { session_id }) => {
            AuthAccessStreamEvent::ClientRemoved {
                revision,
                session_id,
            }
        }
    }
}

pub fn session_credential_change_type(change: &SessionCredentialChange) -> &'static str {
    match change {
        SessionCredentialChange::ClientUpserted { .. } => "clientUpserted",
        SessionCredentialChange::ClientRemoved { .. } => "clientRemoved",
    }
}

pub fn mark_connected_session_count(
    current: &ConnectedSessionCounts,
    session_id: &str,
) -> ConnectedSessionTransition {
    let mut counts = current.counts.clone();
    let was_disconnected = !counts.contains_key(session_id);
    let next_count = counts.get(session_id).copied().unwrap_or(0) + 1;
    counts.insert(session_id.to_string(), next_count);
    ConnectedSessionTransition {
        counts: ConnectedSessionCounts { counts },
        was_disconnected,
        should_set_last_connected_at: was_disconnected,
    }
}

pub fn mark_disconnected_session_count(
    current: &ConnectedSessionCounts,
    session_id: &str,
) -> ConnectedSessionCounts {
    let mut counts = current.counts.clone();
    let remaining = counts
        .get(session_id)
        .copied()
        .unwrap_or(0)
        .saturating_sub(1);
    if remaining > 0 {
        counts.insert(session_id.to_string(), remaining);
    } else {
        counts.remove(session_id);
    }
    ConnectedSessionCounts { counts }
}

pub fn is_session_connected(current: &ConnectedSessionCounts, session_id: &str) -> bool {
    current.counts.contains_key(session_id)
}

fn normalize_non_empty_string(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

fn read_string_claim<'a>(value: &'a Value, key: &str) -> Result<&'a str, AuthError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or(AuthError::InvalidClaims)
}

fn read_i64_claim(value: &Value, key: &str) -> Result<i64, AuthError> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .ok_or(AuthError::InvalidClaims)
}

fn read_u64_claim(value: &Value, key: &str) -> Result<u64, AuthError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or(AuthError::InvalidClaims)
}

fn parse_auth_session_role(value: &str) -> Result<AuthSessionRole, AuthError> {
    match value {
        "owner" => Ok(AuthSessionRole::Owner),
        "client" => Ok(AuthSessionRole::Client),
        _ => Err(AuthError::InvalidClaims),
    }
}

fn parse_auth_session_method(value: &str) -> Result<ServerAuthSessionMethod, AuthError> {
    match value {
        "browser-session-cookie" => Ok(ServerAuthSessionMethod::BrowserSessionCookie),
        "bearer-session-token" => Ok(ServerAuthSessionMethod::BearerSessionToken),
        _ => Err(AuthError::InvalidClaims),
    }
}

fn persisted_auth_session_input_from_credential_record(
    row: &AuthSessionCredentialRecord,
) -> PersistedAuthSessionVerificationInput {
    PersistedAuthSessionVerificationInput {
        session_id: row.session_id.clone(),
        expires_at_ms: row.expires_at_ms,
        revoked_at: row.revoked_at.clone(),
    }
}

fn extract_websocket_query_token(request_url: &str) -> Option<String> {
    let query = request_url
        .split_once('?')?
        .1
        .split('#')
        .next()
        .unwrap_or("");
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == WEBSOCKET_TOKEN_QUERY_PARAM {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn base64_url_decode(input: &str) -> Result<Vec<u8>, AuthError> {
    let mut sextets = Vec::new();
    for byte in input.bytes() {
        let sextet = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => return Err(AuthError::InvalidBase64),
        };
        sextets.push(sextet);
    }

    let mut output = Vec::new();
    let mut index = 0;
    while index < sextets.len() {
        let s0 = sextets[index];
        let Some(&s1) = sextets.get(index + 1) else {
            return Err(AuthError::InvalidBase64);
        };
        let s2 = sextets.get(index + 2).copied();
        let s3 = sextets.get(index + 3).copied();

        output.push((s0 << 2) | (s1 >> 4));
        if let Some(s2) = s2 {
            output.push(((s1 & 0b0000_1111) << 4) | (s2 >> 2));
        }
        if let (Some(s2), Some(s3)) = (s2, s3) {
            output.push(((s2 & 0b0000_0011) << 6) | s3);
        }
        index += 4;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_session_cookie_names_and_auth_policy_descriptor() {
        let desktop = make_server_auth_descriptor(ServerMode::Desktop, Some("127.0.0.1"), 3773);
        assert_eq!(desktop.policy, ServerAuthPolicyKind::DesktopManagedLocal);
        assert_eq!(
            desktop.bootstrap_methods,
            vec![ServerAuthBootstrapMethod::DesktopBootstrap]
        );
        assert_eq!(desktop.session_cookie_name, "t3_session_3773");

        let exposed_desktop =
            make_server_auth_descriptor(ServerMode::Desktop, Some("0.0.0.0"), 3773);
        assert_eq!(
            exposed_desktop.bootstrap_methods,
            vec![
                ServerAuthBootstrapMethod::DesktopBootstrap,
                ServerAuthBootstrapMethod::OneTimeToken
            ]
        );
        assert_eq!(
            exposed_desktop.session_methods,
            vec![
                ServerAuthSessionMethod::BrowserSessionCookie,
                ServerAuthSessionMethod::BearerSessionToken
            ]
        );

        let web = make_server_auth_descriptor(ServerMode::Web, Some("localhost"), 3773);
        assert_eq!(web.policy, ServerAuthPolicyKind::LoopbackBrowser);
        assert_eq!(web.session_cookie_name, "t3_session");

        let remote = make_server_auth_descriptor(ServerMode::Web, Some("192.168.1.50"), 3773);
        assert_eq!(remote.policy, ServerAuthPolicyKind::RemoteReachable);
    }

    #[test]
    fn ports_auth_http_route_contracts() {
        let routes = auth_http_route_plans();
        assert_eq!(routes.len(), AUTH_HTTP_ROUTE_COUNT);
        assert_eq!(
            routes.iter().map(|route| route.layer).collect::<Vec<_>>(),
            vec![
                "authSessionRouteLayer",
                "authBootstrapRouteLayer",
                "authBearerBootstrapRouteLayer",
                "authWebSocketTokenRouteLayer",
                "authPairingCredentialRouteLayer",
                "authPairingLinksRouteLayer",
                "authPairingLinksRevokeRouteLayer",
                "authClientsRouteLayer",
                "authClientsRevokeRouteLayer",
                "authClientsRevokeOthersRouteLayer",
            ]
        );

        let browser_bootstrap = routes
            .iter()
            .find(|route| route.layer == "authBootstrapRouteLayer")
            .unwrap();
        assert_eq!(browser_bootstrap.path, "/api/auth/bootstrap");
        assert_eq!(browser_bootstrap.request_schema, Some("AuthBootstrapInput"));
        assert!(browser_bootstrap.sets_session_cookie);
        assert!(browser_bootstrap.cors_headers);
        assert_eq!(
            browser_bootstrap.invalid_payload_message,
            Some("Invalid bootstrap payload.")
        );

        let ws_token = routes
            .iter()
            .find(|route| route.layer == "authWebSocketTokenRouteLayer")
            .unwrap();
        assert_eq!(ws_token.path, "/api/auth/ws-token");
        assert_eq!(ws_token.auth, "authenticated-session");
        assert_eq!(ws_token.success_response, "AuthWebSocketTokenResult");
        assert!(ws_token.cors_headers);

        let pairing = routes
            .iter()
            .find(|route| route.layer == "authPairingCredentialRouteLayer")
            .unwrap();
        assert_eq!(pairing.auth, "owner-session");
        assert_eq!(
            pairing.request_schema,
            Some("optional AuthCreatePairingCredentialInput")
        );
        assert!(!pairing.cors_headers);
        assert_eq!(
            pairing.invalid_payload_message,
            Some("Invalid pairing credential payload.")
        );

        assert!(auth_pairing_request_has_body(Some("1"), None));
        assert!(!auth_pairing_request_has_body(Some("0"), None));
        assert!(auth_pairing_request_has_body(
            Some("not-a-number"),
            Some("chunked")
        ));
        assert!(!auth_pairing_request_has_body(Some("not-a-number"), None));
    }

    #[test]
    fn ports_http_and_websocket_auth_credential_selection() {
        assert_eq!(
            parse_bearer_token(Some("Bearer token-1")).as_deref(),
            Some("token-1")
        );
        assert_eq!(
            parse_bearer_token(Some("Bearer    token-2   ")).as_deref(),
            Some("token-2")
        );
        assert_eq!(parse_bearer_token(Some("Basic token-1")), None);
        assert_eq!(parse_bearer_token(Some("Bearer    ")), None);

        assert_eq!(
            select_http_auth_credential(Some("cookie-token"), Some("Bearer bearer-token")),
            AuthCredentialSelection::Credential("cookie-token".to_string())
        );
        assert_eq!(
            select_http_auth_credential(None, Some("Bearer bearer-token")),
            AuthCredentialSelection::Credential("bearer-token".to_string())
        );
        assert_eq!(
            select_http_auth_credential(Some("   "), None),
            AuthCredentialSelection::Missing {
                message: "Authentication required.",
                status: 401,
            }
        );

        assert_eq!(
            select_websocket_upgrade_credential(
                Some("http://127.0.0.1:3773/ws?wsToken=ws-token"),
                Some("cookie-token"),
                Some("Bearer bearer-token"),
            ),
            WebSocketUpgradeCredentialSelection::WebSocketToken("ws-token".to_string())
        );
        assert_eq!(
            select_websocket_upgrade_credential(
                Some("http://127.0.0.1:3773/ws?wsToken=   "),
                Some("cookie-token"),
                None,
            ),
            WebSocketUpgradeCredentialSelection::Fallback(AuthCredentialSelection::Credential(
                "cookie-token".to_string()
            ))
        );
        assert_eq!(
            select_websocket_upgrade_credential(
                Some("http://127.0.0.1:3773/ws?other=1"),
                None,
                Some("Bearer bearer-token"),
            ),
            WebSocketUpgradeCredentialSelection::Fallback(AuthCredentialSelection::Credential(
                "bearer-token".to_string()
            ))
        );
    }

    #[test]
    fn ports_bootstrap_error_mapping_and_owner_access_checks() {
        assert_eq!(
            map_bootstrap_credential_auth_error(BootstrapCredentialErrorInput { status: 401 }),
            BootstrapCredentialAuthError {
                message: "Invalid bootstrap credential.",
                status: 401,
            }
        );
        assert_eq!(
            map_bootstrap_credential_auth_error(BootstrapCredentialErrorInput { status: 500 }),
            BootstrapCredentialAuthError {
                message: "Failed to validate bootstrap credential.",
                status: 500,
            }
        );

        assert_eq!(
            require_owner_session(
                AuthSessionRole::Owner,
                AuthOwnerAction::ManageNetworkAccess,
                Some("owner"),
                Some("client"),
            ),
            AuthAccessDecision::Allowed
        );
        assert_eq!(
            require_owner_session(
                AuthSessionRole::Client,
                AuthOwnerAction::CreatePairingCredential,
                Some("client"),
                None,
            ),
            AuthAccessDecision::Denied {
                message: "Only owner sessions can create pairing credentials.",
                status: 403,
            }
        );
        assert_eq!(
            require_owner_session(
                AuthSessionRole::Client,
                AuthOwnerAction::ManageNetworkAccess,
                Some("client"),
                None,
            ),
            AuthAccessDecision::Denied {
                message: "Only owner sessions can manage network access.",
                status: 403,
            }
        );
        assert_eq!(
            require_owner_session(
                AuthSessionRole::Owner,
                AuthOwnerAction::RevokeClientSession,
                Some("owner"),
                Some("owner"),
            ),
            AuthAccessDecision::Denied {
                message: "Use revoke other clients to keep the current owner session active.",
                status: 403,
            }
        );
    }

    #[test]
    fn ports_server_secret_store_filesystem_contracts() {
        let plan = secret_store_path_plan("C:/r3-home/secrets", "server-signing-key", "uuid-1");
        assert_eq!(
            plan.secret_path,
            "C:/r3-home/secrets/server-signing-key.bin"
        );
        assert_eq!(
            plan.temp_path,
            "C:/r3-home/secrets/server-signing-key.bin.uuid-1.tmp"
        );
        assert_eq!(plan.directory_mode, 0o700);
        assert_eq!(plan.file_mode, 0o600);
        assert!(plan.make_directory_recursive);
        assert_eq!(
            plan.write_sequence,
            vec![
                "makeDirectory(secretsDir, recursive)",
                "chmod(secretsDir, 0o700)",
                "writeFile(tempPath, value)",
                "chmod(tempPath, 0o600)",
                "rename(tempPath, secretPath)",
                "chmod(secretPath, 0o600)",
            ]
        );
        assert_eq!(
            resolve_secret_path("C:/r3-home/secrets/", "session-signing-key"),
            "C:/r3-home/secrets/session-signing-key.bin"
        );
        assert_eq!(
            secret_store_read_decision("server-signing-key", Some("NotFound")),
            SecretStoreReadDecision::MissingReturnsNull
        );
        assert_eq!(
            secret_store_read_decision("server-signing-key", Some("PermissionDenied")),
            SecretStoreReadDecision::Fail {
                message: "Failed to read secret server-signing-key.".to_string()
            }
        );
        assert_eq!(
            secret_store_concurrent_create_decision(
                "server-signing-key",
                Some("AlreadyExists"),
                true
            ),
            SecretStoreConcurrentCreateDecision::ReturnCreatedSecret
        );
        assert_eq!(
            secret_store_concurrent_create_decision(
                "server-signing-key",
                Some("AlreadyExists"),
                false
            ),
            SecretStoreConcurrentCreateDecision::FailAfterMissingConcurrentCreate {
                message: "Failed to read secret server-signing-key after concurrent creation."
                    .to_string()
            }
        );
        assert_eq!(
            secret_store_concurrent_create_decision(
                "server-signing-key",
                Some("PermissionDenied"),
                false
            ),
            SecretStoreConcurrentCreateDecision::PropagatePersistError {
                message: "Failed to persist secret server-signing-key.".to_string()
            }
        );
    }

    #[test]
    fn file_secret_store_round_trips_with_temp_rename_semantics() {
        let dir =
            std::env::temp_dir().join(format!("r3code-auth-secret-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(read_file_secret(&dir, "server-signing-key").unwrap(), None);
        write_file_secret(&dir, "server-signing-key", &[1, 2, 3, 4], "temp-1").unwrap();
        assert_eq!(
            read_file_secret(&dir, "server-signing-key").unwrap(),
            Some(vec![1, 2, 3, 4])
        );
        assert!(!dir.join("server-signing-key.bin.temp-1.tmp").exists());

        write_file_secret(&dir, "server-signing-key", &[5, 6], "temp-2").unwrap();
        assert_eq!(
            read_file_secret(&dir, "server-signing-key").unwrap(),
            Some(vec![5, 6])
        );
        assert_eq!(
            get_or_create_file_secret(&dir, "server-signing-key", &[9, 9]).unwrap(),
            vec![5, 6]
        );

        remove_file_secret(&dir, "server-signing-key").unwrap();
        assert_eq!(read_file_secret(&dir, "server-signing-key").unwrap(), None);
        assert_eq!(
            get_or_create_file_secret(&dir, "server-signing-key", &[7, 8, 9]).unwrap(),
            vec![7, 8, 9]
        );
        assert_eq!(
            read_file_secret(&dir, "server-signing-key").unwrap(),
            Some(vec![7, 8, 9])
        );
        remove_file_secret(&dir, "server-signing-key").unwrap();

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ports_persisted_session_verification_decisions() {
        let active = PersistedAuthSessionVerificationInput {
            session_id: "session-1".to_string(),
            expires_at_ms: 2_000,
            revoked_at: None,
        };
        assert_eq!(
            verify_persisted_session_state(Some(&active), 1_500, false),
            PersistedAuthSessionVerification::Accepted
        );
        assert_eq!(
            verify_persisted_session_state(None, 1_500, false),
            PersistedAuthSessionVerification::Unknown {
                message: "Unknown session token."
            }
        );
        assert_eq!(
            verify_persisted_session_state(None, 1_500, true),
            PersistedAuthSessionVerification::Unknown {
                message: "Unknown websocket session."
            }
        );

        let expired = PersistedAuthSessionVerificationInput {
            expires_at_ms: 2_000,
            ..active.clone()
        };
        assert_eq!(
            verify_persisted_session_state(Some(&expired), 2_000, false),
            PersistedAuthSessionVerification::Accepted
        );
        assert_eq!(
            verify_persisted_session_state(Some(&expired), 2_000, true),
            PersistedAuthSessionVerification::Expired {
                message: "Websocket session expired."
            }
        );

        let revoked = PersistedAuthSessionVerificationInput {
            revoked_at: Some("2026-03-04T12:00:00.000Z".to_string()),
            ..active
        };
        assert_eq!(
            verify_persisted_session_state(Some(&revoked), 1_500, false),
            PersistedAuthSessionVerification::Revoked {
                message: "Session token revoked."
            }
        );
        assert_eq!(
            verify_persisted_session_state(Some(&revoked), 1_500, true),
            PersistedAuthSessionVerification::Revoked {
                message: "Websocket session revoked."
            }
        );
    }

    #[test]
    fn verifies_signed_session_tokens_against_repository_state_like_upstream() {
        let session = SessionClaims {
            session_id: "session-1".to_string(),
            subject: "desktop-bootstrap".to_string(),
            role: AuthSessionRole::Owner,
            method: ServerAuthSessionMethod::BrowserSessionCookie,
            issued_at_ms: 1000,
            expires_at_ms: 3000,
        };
        let token = sign_session_claims(&session, b"secret");
        let active = PersistedAuthSessionVerificationInput {
            session_id: "session-1".to_string(),
            expires_at_ms: 1500,
            revoked_at: None,
        };
        assert_eq!(
            verify_signed_session_against_persisted_state(&token, b"secret", 2_000, Some(&active))
                .unwrap(),
            session
        );
        assert_eq!(
            verify_signed_session_against_persisted_state(&token, b"secret", 2_000, None),
            Err(AuthTokenVerificationError::Persisted(
                PersistedAuthSessionVerification::Unknown {
                    message: "Unknown session token."
                }
            ))
        );

        let mismatched = PersistedAuthSessionVerificationInput {
            session_id: "session-2".to_string(),
            ..active.clone()
        };
        assert_eq!(
            verify_signed_session_against_persisted_state(
                &token,
                b"secret",
                2_000,
                Some(&mismatched)
            ),
            Err(AuthTokenVerificationError::Persisted(
                PersistedAuthSessionVerification::Unknown {
                    message: "Unknown session token."
                }
            ))
        );

        let revoked = PersistedAuthSessionVerificationInput {
            revoked_at: Some("2026-03-04T12:00:00.000Z".to_string()),
            ..active
        };
        assert_eq!(
            verify_signed_session_against_persisted_state(&token, b"secret", 2_000, Some(&revoked)),
            Err(AuthTokenVerificationError::Persisted(
                PersistedAuthSessionVerification::Revoked {
                    message: "Session token revoked."
                }
            ))
        );
        assert_eq!(
            verify_signed_session_against_persisted_state(&token, b"wrong", 2_000, Some(&revoked)),
            Err(AuthTokenVerificationError::Token(
                AuthError::InvalidSignature
            ))
        );
    }

    #[test]
    fn verifies_signed_websocket_tokens_against_repository_state_like_upstream() {
        let websocket = WebSocketClaims {
            session_id: "session-1".to_string(),
            issued_at_ms: 1000,
            expires_at_ms: 3000,
        };
        let token = sign_websocket_claims(&websocket, b"secret");
        let active = PersistedAuthSessionVerificationInput {
            session_id: "session-1".to_string(),
            expires_at_ms: 4000,
            revoked_at: None,
        };
        assert_eq!(
            verify_signed_websocket_against_persisted_state(
                &token,
                b"secret",
                2_000,
                Some(&active)
            )
            .unwrap(),
            websocket
        );
        assert_eq!(
            verify_signed_websocket_against_persisted_state(&token, b"secret", 2_000, None),
            Err(AuthTokenVerificationError::Persisted(
                PersistedAuthSessionVerification::Unknown {
                    message: "Unknown websocket session."
                }
            ))
        );

        let expired_parent = PersistedAuthSessionVerificationInput {
            expires_at_ms: 2_000,
            ..active.clone()
        };
        assert_eq!(
            verify_signed_websocket_against_persisted_state(
                &token,
                b"secret",
                2_000,
                Some(&expired_parent)
            ),
            Err(AuthTokenVerificationError::Persisted(
                PersistedAuthSessionVerification::Expired {
                    message: "Websocket session expired."
                }
            ))
        );

        let revoked = PersistedAuthSessionVerificationInput {
            revoked_at: Some("2026-03-04T12:00:00.000Z".to_string()),
            ..active
        };
        assert_eq!(
            verify_signed_websocket_against_persisted_state(
                &token,
                b"secret",
                2_000,
                Some(&revoked)
            ),
            Err(AuthTokenVerificationError::Persisted(
                PersistedAuthSessionVerification::Revoked {
                    message: "Websocket session revoked."
                }
            ))
        );
        assert_eq!(
            verify_signed_websocket_against_persisted_state(
                &token,
                b"wrong",
                2_000,
                Some(&revoked)
            ),
            Err(AuthTokenVerificationError::Token(
                AuthError::InvalidSignature
            ))
        );
    }

    #[test]
    fn issues_session_credentials_with_upstream_defaults_and_persistence_row() {
        let plan = issue_session_credential_plan(
            SessionCredentialIssueInput {
                session_id: "session-1".to_string(),
                issued_at_ms: 1_000,
                ttl_ms: None,
                subject: None,
                method: None,
                role: None,
                client: None,
            },
            b"secret",
        );

        assert_eq!(
            plan.claims,
            SessionClaims {
                session_id: "session-1".to_string(),
                subject: "browser".to_string(),
                role: AuthSessionRole::Client,
                method: ServerAuthSessionMethod::BrowserSessionCookie,
                issued_at_ms: 1_000,
                expires_at_ms: 1_000 + DEFAULT_SESSION_TTL_MS,
            }
        );
        assert_eq!(plan.issued.session_id, "session-1");
        assert_eq!(
            plan.issued.method,
            ServerAuthSessionMethod::BrowserSessionCookie
        );
        assert_eq!(plan.issued.client, default_auth_client_metadata());
        assert_eq!(plan.issued.role, AuthSessionRole::Client);
        assert_eq!(plan.persisted_session.subject, "browser");
        assert_eq!(
            plan.persisted_session.client,
            default_auth_client_metadata()
        );
        assert_eq!(plan.persisted_session.issued_at_ms, 1_000);
        assert_eq!(plan.persisted_session.revoked_at, None);
        assert_eq!(
            verify_signed_session_claims(&plan.issued.token, b"secret", 1_001).unwrap(),
            plan.claims
        );
    }

    #[test]
    fn issues_session_credentials_with_upstream_overrides() {
        let client = AuthClientMetadata {
            label: Some("Desktop".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("R3Code".to_string()),
            device_type: AuthClientMetadataDeviceType::Desktop,
            os: Some("Windows".to_string()),
            browser: None,
        };
        let plan = issue_session_credential_plan(
            SessionCredentialIssueInput {
                session_id: "session-owner".to_string(),
                issued_at_ms: 2_000,
                ttl_ms: Some(5_000),
                subject: Some("desktop-bootstrap".to_string()),
                method: Some(ServerAuthSessionMethod::BearerSessionToken),
                role: Some(AuthSessionRole::Owner),
                client: Some(client.clone()),
            },
            b"secret",
        );

        assert_eq!(plan.issued.expires_at_ms, 7_000);
        assert_eq!(plan.issued.role, AuthSessionRole::Owner);
        assert_eq!(plan.issued.client, client);
        assert_eq!(plan.persisted_session.subject, "desktop-bootstrap");
        assert_eq!(
            plan.persisted_session.method,
            ServerAuthSessionMethod::BearerSessionToken
        );
        assert_eq!(
            verify_signed_session_claims(&plan.issued.token, b"secret", 6_999).unwrap(),
            SessionClaims {
                session_id: "session-owner".to_string(),
                subject: "desktop-bootstrap".to_string(),
                role: AuthSessionRole::Owner,
                method: ServerAuthSessionMethod::BearerSessionToken,
                issued_at_ms: 2_000,
                expires_at_ms: 7_000,
            }
        );
    }

    #[test]
    fn issues_websocket_credentials_with_upstream_default_and_override_ttls() {
        let default_token = issue_websocket_credential(
            WebSocketCredentialIssueInput {
                session_id: "session-1".to_string(),
                issued_at_ms: 1_000,
                ttl_ms: None,
            },
            b"secret",
        );
        assert_eq!(
            default_token.expires_at_ms,
            1_000 + DEFAULT_WEBSOCKET_TOKEN_TTL_MS
        );
        assert_eq!(
            verify_signed_websocket_claims(&default_token.token, b"secret", 1_001).unwrap(),
            WebSocketClaims {
                session_id: "session-1".to_string(),
                issued_at_ms: 1_000,
                expires_at_ms: 1_000 + DEFAULT_WEBSOCKET_TOKEN_TTL_MS,
            }
        );

        let short_token = issue_websocket_credential(
            WebSocketCredentialIssueInput {
                session_id: "session-1".to_string(),
                issued_at_ms: 1_000,
                ttl_ms: Some(5_000),
            },
            b"secret",
        );
        assert_eq!(short_token.expires_at_ms, 6_000);
        assert_eq!(
            verify_signed_websocket_claims(&short_token.token, b"secret", 5_999).unwrap(),
            WebSocketClaims {
                session_id: "session-1".to_string(),
                issued_at_ms: 1_000,
                expires_at_ms: 6_000,
            }
        );
    }

    #[test]
    fn assembles_verified_session_credentials_from_claims_and_row_like_upstream() {
        let claims = SessionClaims {
            session_id: "session-1".to_string(),
            subject: "claim-subject".to_string(),
            role: AuthSessionRole::Owner,
            method: ServerAuthSessionMethod::BearerSessionToken,
            issued_at_ms: 1000,
            expires_at_ms: 3000,
        };
        let token = sign_session_claims(&claims, b"secret");
        let row = AuthSessionCredentialRecord {
            session_id: "session-1".to_string(),
            subject: "row-subject".to_string(),
            role: AuthSessionRole::Client,
            method: ServerAuthSessionMethod::BrowserSessionCookie,
            client: AuthClientMetadata {
                label: Some("Desktop".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: None,
                device_type: AuthClientMetadataDeviceType::Desktop,
                os: Some("Windows".to_string()),
                browser: None,
            },
            issued_at_ms: 1000,
            expires_at_ms: 1500,
            revoked_at: None,
        };

        assert_eq!(
            verify_session_credential_record(&token, b"secret", 2_000, Some(&row)).unwrap(),
            VerifiedSessionCredential {
                session_id: "session-1".to_string(),
                token: token.clone(),
                method: ServerAuthSessionMethod::BearerSessionToken,
                client: row.client.clone(),
                expires_at_ms: Some(3000),
                subject: "claim-subject".to_string(),
                role: AuthSessionRole::Owner,
            }
        );
    }

    #[test]
    fn assembles_verified_websocket_credentials_from_row_like_upstream() {
        let claims = WebSocketClaims {
            session_id: "session-1".to_string(),
            issued_at_ms: 1000,
            expires_at_ms: 3000,
        };
        let token = sign_websocket_claims(&claims, b"secret");
        let row = AuthSessionCredentialRecord {
            session_id: "session-1".to_string(),
            subject: "row-subject".to_string(),
            role: AuthSessionRole::Client,
            method: ServerAuthSessionMethod::BrowserSessionCookie,
            client: AuthClientMetadata {
                label: Some("Phone".to_string()),
                ip_address: Some("10.0.0.5".to_string()),
                user_agent: None,
                device_type: AuthClientMetadataDeviceType::Mobile,
                os: Some("iOS".to_string()),
                browser: Some("Safari".to_string()),
            },
            issued_at_ms: 1000,
            expires_at_ms: 4000,
            revoked_at: None,
        };

        assert_eq!(
            verify_websocket_credential_record(&token, b"secret", 2_000, Some(&row)).unwrap(),
            VerifiedSessionCredential {
                session_id: "session-1".to_string(),
                token: token.clone(),
                method: ServerAuthSessionMethod::BrowserSessionCookie,
                client: row.client.clone(),
                expires_at_ms: Some(4000),
                subject: "row-subject".to_string(),
                role: AuthSessionRole::Client,
            }
        );
    }

    #[test]
    fn derives_client_metadata_like_upstream_auth_utils() {
        let metadata = derive_auth_client_metadata(
            Some(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) t3code/0.0.15 Chrome/136.0.7103.93 Electron/36.3.2 Safari/537.36",
            ),
            Some("::ffff:127.0.0.1"),
            None,
        );

        assert_eq!(metadata.browser.as_deref(), Some("Electron"));
        assert_eq!(metadata.device_type, AuthClientMetadataDeviceType::Desktop);
        assert_eq!(metadata.ip_address.as_deref(), Some("127.0.0.1"));
        assert_eq!(metadata.os.as_deref(), Some("macOS"));

        let bot = derive_auth_client_metadata(Some("curl/8.0"), None, Some("deploy-bot"));
        assert_eq!(bot.device_type, AuthClientMetadataDeviceType::Bot);
        assert_eq!(bot.label.as_deref(), Some("deploy-bot"));
    }

    #[test]
    fn ports_pairing_token_alphabet_and_length() {
        let token = pairing_token_from_bytes(&[0, 1, 2, 3, 4, 5, 30, 31, 32, 33, 254, 255]);
        assert_eq!(token.len(), PAIRING_TOKEN_LENGTH);
        assert!(token.chars().all(|ch| PAIRING_TOKEN_ALPHABET.contains(ch)));
        assert_eq!(token, "234567YZ23YZ");
    }

    #[test]
    fn encodes_base64url_payloads_without_padding() {
        let encoded = base64_url_encode("hello?".as_bytes());
        assert_eq!(encoded, "aGVsbG8_");
        assert_eq!(base64_url_decode_utf8(&encoded).unwrap(), "hello?");
    }

    #[test]
    fn signs_session_and_websocket_tokens_like_upstream_hmac_sha256() {
        assert_eq!(
            sign_payload("payload", b"secret"),
            "uC_LeRrOxXhZuYm0MKgmSIzi5Hn9-SMmvQoug3WkK6Q"
        );
        assert!(verify_payload_signature(
            "payload",
            "uC_LeRrOxXhZuYm0MKgmSIzi5Hn9-SMmvQoug3WkK6Q",
            b"secret"
        ));
        assert!(!verify_payload_signature(
            "payload",
            "uC_LeRrOxXhZuYm0MKgmSIzi5Hn9-SMmvQoug3WkK6Q",
            b"wrong"
        ));

        let session = SessionClaims {
            session_id: "session-1".to_string(),
            subject: "desktop-bootstrap".to_string(),
            role: AuthSessionRole::Owner,
            method: ServerAuthSessionMethod::BrowserSessionCookie,
            issued_at_ms: 1000,
            expires_at_ms: 2000,
        };
        let signed_session = sign_session_claims(&session, b"secret");
        let verified_session = verify_signed_token_signature(&signed_session, b"secret").unwrap();
        assert_eq!(
            base64_url_decode_utf8(&verified_session.encoded_payload).unwrap(),
            "{\"v\":1,\"kind\":\"session\",\"sid\":\"session-1\",\"sub\":\"desktop-bootstrap\",\"role\":\"owner\",\"method\":\"browser-session-cookie\",\"iat\":1000,\"exp\":2000}"
        );
        assert_eq!(
            decode_session_claims_payload(&verified_session.encoded_payload).unwrap(),
            session
        );
        assert_eq!(
            verify_signed_session_claims(&signed_session, b"secret", 1500).unwrap(),
            session
        );
        assert_eq!(
            verify_signed_session_claims(&signed_session, b"secret", 2000),
            Err(AuthError::ExpiredToken)
        );
        assert_eq!(
            verify_signed_token_signature(&signed_session, b"wrong"),
            Err(AuthError::InvalidSignature)
        );

        let websocket = sign_websocket_claims(
            &WebSocketClaims {
                session_id: "session-1".to_string(),
                issued_at_ms: 1000,
                expires_at_ms: 2000,
            },
            b"secret",
        );
        let verified_websocket = verify_signed_token_signature(&websocket, b"secret").unwrap();
        assert_eq!(
            base64_url_decode_utf8(&verified_websocket.encoded_payload).unwrap(),
            "{\"v\":1,\"kind\":\"websocket\",\"sid\":\"session-1\",\"iat\":1000,\"exp\":2000}"
        );
        assert_eq!(
            decode_websocket_claims_payload(&verified_websocket.encoded_payload).unwrap(),
            WebSocketClaims {
                session_id: "session-1".to_string(),
                issued_at_ms: 1000,
                expires_at_ms: 2000,
            }
        );
        assert_eq!(
            verify_signed_websocket_claims(&websocket, b"secret", 2000),
            Err(AuthError::ExpiredToken)
        );
        assert_eq!(
            decode_websocket_claims_payload(&encode_session_claims_payload(&session)),
            Err(AuthError::InvalidClaims)
        );
    }

    #[test]
    fn builds_session_and_websocket_claim_payloads_in_upstream_shape() {
        let session = encode_session_claims_payload(&SessionClaims {
            session_id: "session-1".to_string(),
            subject: "desktop-bootstrap".to_string(),
            role: AuthSessionRole::Owner,
            method: ServerAuthSessionMethod::BrowserSessionCookie,
            issued_at_ms: 1000,
            expires_at_ms: 2000,
        });
        let decoded = base64_url_decode_utf8(&session).unwrap();
        assert_eq!(
            decoded,
            "{\"v\":1,\"kind\":\"session\",\"sid\":\"session-1\",\"sub\":\"desktop-bootstrap\",\"role\":\"owner\",\"method\":\"browser-session-cookie\",\"iat\":1000,\"exp\":2000}"
        );

        let websocket = encode_websocket_claims_payload(&WebSocketClaims {
            session_id: "session-1".to_string(),
            issued_at_ms: 1000,
            expires_at_ms: 2000,
        });
        assert_eq!(
            base64_url_decode_utf8(&websocket).unwrap(),
            "{\"v\":1,\"kind\":\"websocket\",\"sid\":\"session-1\",\"iat\":1000,\"exp\":2000}"
        );
    }

    #[test]
    fn splits_signed_session_tokens_and_rejects_malformed_values() {
        let parts = SignedTokenParts {
            encoded_payload: "payload".to_string(),
            signature: "signature".to_string(),
        };
        let token = join_signed_token(&parts);
        assert_eq!(token, "payload.signature");
        assert_eq!(split_signed_token(&token).unwrap(), parts);
        assert_eq!(
            split_signed_token("not-a-session-token"),
            Err(AuthError::MalformedToken)
        );
        assert_eq!(split_signed_token("a.b.c"), Err(AuthError::MalformedToken));
    }

    #[test]
    fn exposes_auth_access_stream_event_contract_names() {
        let event = AuthAccessStreamEvent::ClientRemoved {
            revision: 7,
            session_id: "session-1".to_string(),
        };
        assert_eq!(auth_access_event_type(&event), "clientRemoved");
        assert_eq!(auth_access_event_revision(&event), 7);

        let change = SessionCredentialChange::ClientRemoved {
            session_id: "session-1".to_string(),
        };
        assert_eq!(session_credential_change_type(&change), "clientRemoved");
    }

    #[test]
    fn converts_auth_access_changes_to_revisioned_stream_events_like_upstream_ws() {
        let pairing_link = AuthPairingLink {
            id: "pairing-1".to_string(),
            credential: "TOKEN".to_string(),
            role: AuthSessionRole::Client,
            subject: "client".to_string(),
            label: None,
            created_at: "2026-03-04T12:00:00.000Z".to_string(),
            expires_at: "2026-03-04T12:10:00.000Z".to_string(),
        };
        assert_eq!(
            auth_access_stream_event_from_change(
                AuthAccessChange::Bootstrap(BootstrapCredentialChange::PairingLinkUpserted {
                    pairing_link: pairing_link.clone()
                }),
                2,
                "session-current",
            ),
            AuthAccessStreamEvent::PairingLinkUpserted {
                revision: 2,
                payload: pairing_link,
            }
        );
        assert_eq!(
            auth_access_stream_event_from_change(
                AuthAccessChange::Bootstrap(BootstrapCredentialChange::PairingLinkRemoved {
                    id: "pairing-1".to_string(),
                }),
                3,
                "session-current",
            ),
            AuthAccessStreamEvent::PairingLinkRemoved {
                revision: 3,
                id: "pairing-1".to_string(),
            }
        );

        let client_session = AuthClientSession {
            session_id: "session-current".to_string(),
            subject: "browser".to_string(),
            role: AuthSessionRole::Owner,
            method: ServerAuthSessionMethod::BrowserSessionCookie,
            client: default_auth_client_metadata(),
            issued_at: "2026-03-04T12:00:00.000Z".to_string(),
            expires_at: "2026-04-03T12:00:00.000Z".to_string(),
            last_connected_at: None,
            connected: true,
            current: false,
        };
        assert_eq!(
            auth_access_stream_event_from_change(
                AuthAccessChange::Session(SessionCredentialChange::ClientUpserted {
                    client_session: client_session.clone(),
                }),
                4,
                "session-current",
            ),
            AuthAccessStreamEvent::ClientUpserted {
                revision: 4,
                payload: AuthClientSession {
                    current: true,
                    ..client_session
                },
            }
        );
        assert_eq!(
            auth_access_stream_event_from_change(
                AuthAccessChange::Session(SessionCredentialChange::ClientRemoved {
                    session_id: "session-current".to_string(),
                }),
                5,
                "session-current",
            ),
            AuthAccessStreamEvent::ClientRemoved {
                revision: 5,
                session_id: "session-current".to_string(),
            }
        );
    }

    #[test]
    fn ports_connected_session_reference_count_transitions() {
        let empty = ConnectedSessionCounts::default();
        let first = mark_connected_session_count(&empty, "session-1");
        assert!(first.was_disconnected);
        assert!(first.should_set_last_connected_at);
        assert_eq!(first.counts.counts.get("session-1"), Some(&1));
        assert!(is_session_connected(&first.counts, "session-1"));

        let second = mark_connected_session_count(&first.counts, "session-1");
        assert!(!second.was_disconnected);
        assert!(!second.should_set_last_connected_at);
        assert_eq!(second.counts.counts.get("session-1"), Some(&2));

        let still_connected = mark_disconnected_session_count(&second.counts, "session-1");
        assert_eq!(still_connected.counts.get("session-1"), Some(&1));
        assert!(is_session_connected(&still_connected, "session-1"));

        let disconnected = mark_disconnected_session_count(&still_connected, "session-1");
        assert_eq!(disconnected.counts.get("session-1"), None);
        assert!(!is_session_connected(&disconnected, "session-1"));

        let missing_disconnect = mark_disconnected_session_count(&disconnected, "session-1");
        assert_eq!(missing_disconnect, disconnected);
    }
}
