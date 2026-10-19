//! HTTPS JSON DTOs. See docs/contract-v1.md for authorization and HTTP semantics.
use crate::{protocol::*, *};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Unauthorized,
    Forbidden,
    NotFound,
    Validation,
    RateLimited,
    ServiceUnavailable,
    ProtocolIncompatible,
    Conflict,
    QuotaExceeded,
    AuthorizationPending,
    SlowDown,
    AccessDenied,
    ExpiredToken,
    InvalidGrant,
}
impl ErrorCode {
    pub fn http_status(self) -> u16 {
        match self {
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Validation
            | Self::ProtocolIncompatible
            | Self::AuthorizationPending
            | Self::SlowDown
            | Self::AccessDenied
            | Self::ExpiredToken
            | Self::InvalidGrant => 400,
            Self::Conflict => 409,
            Self::QuotaExceeded | Self::RateLimited => 429,
            Self::ServiceUnavailable => 503,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiError {
    pub code: ErrorCode,
    /// Server-authored display text; codes drive behavior, not local text lookup.
    pub message: MessageText,
    pub level: crate::communication::MessageLevel,
    pub request_id: RequestId,
    pub retry_after_seconds: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ErrorResponse {
    pub error: ApiError,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeviceCodeRequest {
    /// Suggested label only; the approving user may edit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_name: Option<DisplayName>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeviceCodeResponse {
    pub device_grant_id: DeviceGrantId,
    pub device_code: SecretString,
    pub user_code: SecretString,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeviceTokenRequest {
    pub device_code: SecretString,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum TokenType {
    Bearer,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeviceTokenResponse {
    pub cli_credential_id: CliCredentialId,
    pub access_token: SecretString,
    pub token_type: TokenType,
    pub expires_in: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IdentityResponse {
    pub user_id: UserId,
    pub email: String,
    pub cli_credential_id: CliCredentialId,
    pub expires_at: DateTime<Utc>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrgRole {
    Owner,
    Admin,
    Member,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    pub org_id: OrgId,
    pub name: DisplayName,
    pub role: OrgRole,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub project_id: ProjectId,
    pub org_id: OrgId,
    pub name: DisplayName,
    pub url: Option<String>,
    pub active: bool,
    pub end_to_end_encryption: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateProjectRequest {
    pub name: DisplayName,
    pub url: Option<String>,
}
/// Secret-bearing mutation response: never use as ordinary CLI JSON output.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectTokenResponse {
    pub project: Project,
    pub project_token: SecretString,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RotateProjectTokenRequest {}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<Cursor>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PageQuery {
    #[serde(default = "default_page_size")]
    pub limit: u16,
    pub cursor: Option<Cursor>,
}
fn default_page_size() -> u16 {
    DEFAULT_PAGE_SIZE
}
impl Default for PageQuery {
    fn default() -> Self {
        Self {
            limit: DEFAULT_PAGE_SIZE,
            cursor: None,
        }
    }
}
impl PageQuery {
    pub fn validate(&self) -> Result<(), InvalidValue> {
        if self.limit == 0 || self.limit > MAX_PAGE_SIZE {
            Err(InvalidValue)
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerSnapshot {
    pub project_id: ProjectId,
    pub container_id: ContainerId,
    pub display_name: DisplayName,
    pub build_commit: Option<BuildCommit>,
    pub build_dirty: Option<bool>,
    pub platform: Platform,
    pub agent_version: AgentVersion,
    pub runtime: Option<RuntimeMetadata>,
    pub application_state: ApplicationState,
    pub smuggler_session_id: SmugglerSessionId,
    pub presence: PresenceStatus,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InstancesResponse {
    pub items: Vec<ContainerSnapshot>,
    pub next_cursor: Option<Cursor>,
    pub server_time: DateTime<Utc>,
}
