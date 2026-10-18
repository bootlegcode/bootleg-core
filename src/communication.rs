//! Portable client attribution and server-authored presentation metadata.
//! Adapters extract headers; these types neither authenticate clients nor render UI.
use crate::{
    AgentVersion, InvalidValue,
    values::{MessageText, NoticeId},
};
use schemars::{JsonSchema, Schema, json_schema};
use serde::{Deserialize, Serialize};

pub const CLIENT_HEADER: &str = "Bootleg-Client";
pub const CLIENT_VERSION_HEADER: &str = "Bootleg-Client-Version";
pub const PROTOCOL_VERSION_HEADER: &str = "Bootleg-Protocol-Version";
pub const CLIENT_BOOTSTRAP_PATH: &str = "/api/v1/client/bootstrap";
pub const MAX_NOTICES: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    Cli,
    Smuggler,
}
impl ClientKind {
    pub fn header_value(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Smuggler => "smuggler",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientMetadata {
    pub client: ClientKind,
    pub version: AgentVersion,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ClientMetadataError {
    #[error("missing or invalid client metadata")]
    Invalid,
    #[error("unsupported protocol version")]
    UnsupportedProtocol,
}
impl ClientMetadata {
    /// Call only after rejecting duplicate headers. Inputs are mandatory on
    /// every native-client HTTP request, including WebSocket upgrades.
    pub fn from_headers(
        client: &str,
        version: &str,
        protocol: &str,
    ) -> Result<Self, ClientMetadataError> {
        let client = match client {
            "cli" => ClientKind::Cli,
            "smuggler" => ClientKind::Smuggler,
            _ => return Err(ClientMetadataError::Invalid),
        };
        if version.is_empty()
            || !version
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+'))
        {
            return Err(ClientMetadataError::Invalid);
        }
        let version = version.parse().map_err(|_| ClientMetadataError::Invalid)?;
        if protocol != "1" {
            return Err(ClientMetadataError::UnsupportedProtocol);
        }
        Ok(Self { client, version })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

/// Server-authored plain text. Severity affects presentation only, never exit
/// status or retry policy. The stable ID is for deduplication, not text lookup.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Notice {
    pub id: NoticeId,
    pub level: MessageLevel,
    pub message: MessageText,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "Vec<Notice>", into = "Vec<Notice>")]
pub struct Notices(Vec<Notice>);
impl Notices {
    pub fn as_slice(&self) -> &[Notice] {
        &self.0
    }
}
impl TryFrom<Vec<Notice>> for Notices {
    type Error = InvalidValue;
    fn try_from(value: Vec<Notice>) -> Result<Self, Self::Error> {
        if value.len() > MAX_NOTICES
            || value
                .iter()
                .enumerate()
                .any(|(i, n)| value[..i].iter().any(|p| p.id == n.id))
        {
            return Err(InvalidValue);
        }
        Ok(Self(value))
    }
}
impl From<Notices> for Vec<Notice> {
    fn from(value: Notices) -> Self {
        value.0
    }
}
impl JsonSchema for Notices {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Notices".into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({"type":"array", "maxItems":MAX_NOTICES,
            "items":generator.subschema_for::<Notice>(),
            "description":"Notice IDs must be unique within this response."})
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BootstrapResponse {
    pub protocol_version: u16,
    pub notices: Notices,
}
