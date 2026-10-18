//! Shared v1 wire contract. No authorization, persistence, process supervision,
//! or presence aggregates live here. Map these DTOs explicitly at domain edges.
pub mod api;
pub mod communication;
pub mod ids;
pub mod protocol;
pub mod values;

pub use ids::*;
pub use values::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid wire value")]
pub struct InvalidValue;

pub const API_PREFIX: &str = "/api/v1";
pub const AGENT_CONNECT_PATH: &str = "/api/v1/agent/connect";
pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_FRAME_BYTES: usize = 16 * 1024;
pub const REGISTRATION_TIMEOUT_SECONDS: u64 = 5;
pub const HEARTBEAT_INTERVAL_SECONDS: u64 = 15;
pub const LEASE_TIMEOUT_SECONDS: u64 = 45;
pub const OFFLINE_RETENTION_SECONDS: u64 = 24 * 60 * 60;
pub const SNAPSHOT_POLL_SECONDS: u64 = 3;
pub const DEFAULT_PAGE_SIZE: u16 = 50;
pub const MAX_PAGE_SIZE: u16 = 200;
pub const MAX_CONTAINERS_PER_PROJECT: usize = 10_000;
pub const MAX_SESSIONS_PER_PROJECT: usize = 1_000;
pub const HEARTBEAT_ACK_TIMEOUT_SECONDS: u64 = 15;
pub const MAX_HTTP_REQUEST_BYTES: usize = 16 * 1024;
pub const MAX_SECRET_BYTES: usize = 512;
pub const DEVICE_CODE_LIFETIME_SECONDS: u64 = 600;
pub const DEVICE_POLL_INTERVAL_SECONDS: u64 = 5;
pub const CLI_CREDENTIAL_LIFETIME_SECONDS: u64 = 30 * 24 * 60 * 60;
