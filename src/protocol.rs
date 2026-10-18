//! JSON text frames only. WebSocket ping/pong is not a presence heartbeat.
use crate::{api::ApiError, *};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub os: PlatformName,
    pub arch: PlatformName,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMetadata {
    pub pod_uid: Option<MetadataText>,
    pub pod_name: Option<MetadataText>,
    pub namespace: Option<MetadataText>,
    pub container_name: Option<MetadataText>,
    pub cluster_name: Option<MetadataText>,
    pub runtime_hint: Option<MetadataText>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationState {
    Starting,
    Running,
    Exited,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    Connected,
    Offline,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
// Keep the small, bounded wire DTO ergonomic; clients must bound their queues.
#[allow(clippy::large_enum_variant)]
pub enum AgentFrame {
    Register {
        protocol_version: u16,
        project_id: ProjectId,
        container_id: ContainerId,
        display_name: DisplayName,
        build_commit: Option<BuildCommit>,
        build_dirty: Option<bool>,
        platform: Platform,
        agent_version: AgentVersion,
        runtime: Option<RuntimeMetadata>,
        application_state: ApplicationState,
    },
    Heartbeat {
        smuggler_session_id: SmugglerSessionId,
        application_state: ApplicationState,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ServerFrame {
    Registered {
        smuggler_session_id: SmugglerSessionId,
        notices: crate::communication::Notices,
        heartbeat_interval_seconds: u64,
        lease_timeout_seconds: u64,
        server_time: DateTime<Utc>,
    },
    HeartbeatAck {
        smuggler_session_id: SmugglerSessionId,
        server_time: DateTime<Utc>,
    },
    Error {
        error: ApiError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("frame exceeds 16384 bytes")]
    TooLarge,
    // Do not expose serde's error: it may include untrusted/secret input.
    #[error("invalid protocol frame")]
    Invalid,
    #[error("unsupported protocol version")]
    UnsupportedVersion,
}
pub fn decode_agent_frame(text: &str) -> Result<AgentFrame, FrameError> {
    let frame: AgentFrame = decode(text)?;
    if matches!(&frame, AgentFrame::Register { protocol_version, .. } if *protocol_version != PROTOCOL_VERSION)
    {
        return Err(FrameError::UnsupportedVersion);
    }
    Ok(frame)
}
pub fn decode_server_frame(text: &str) -> Result<ServerFrame, FrameError> {
    decode(text)
}
fn decode<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, FrameError> {
    if text.len() > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge);
    }
    serde_json::from_str(text).map_err(|_| FrameError::Invalid)
}
pub fn encode_agent_frame(frame: &AgentFrame) -> Result<String, FrameError> {
    let text = encode(frame)?;
    decode_agent_frame(&text)?;
    Ok(text)
}
pub fn encode_server_frame(frame: &ServerFrame) -> Result<String, FrameError> {
    encode(frame)
}
fn encode<T: Serialize>(frame: &T) -> Result<String, FrameError> {
    let text = serde_json::to_string(frame).map_err(|_| FrameError::Invalid)?;
    if text.len() > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge);
    }
    Ok(text)
}
