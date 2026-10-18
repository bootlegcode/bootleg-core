//! Deterministic serialization examples, not fake network services.
use bootleg_core::{api::*, communication::*, protocol::*, *};
use serde::Serialize;
use std::{fs, str::FromStr};
fn write(name: &str, value: &impl Serialize) {
    fs::write(
        format!("fixtures/v1/{name}.json"),
        serde_json::to_string_pretty(value).unwrap() + "\n",
    )
    .unwrap();
}
fn main() {
    fs::create_dir_all("fixtures/v1").unwrap();
    let notices: Notices = vec![Notice {
        id: "maintenance-2026-01-02-v1".parse().unwrap(),
        level: MessageLevel::Warning,
        message: "Scheduled maintenance tomorrow at 02:00 UTC."
            .parse()
            .unwrap(),
    }]
    .try_into()
    .unwrap();
    write(
        "bootstrap",
        &BootstrapResponse {
            protocol_version: 1,
            notices: notices.clone(),
        },
    );
    // UUID 00000000-0000-4000-8000-000000000000, encoded by mti convention.
    let suffix = "00000000008008000000000000";
    let container = ContainerId::from_str(&format!("container_{suffix}")).unwrap();
    let session = SmugglerSessionId::from_str(&format!("smuggler_session_{suffix}")).unwrap();
    let org = OrgId::from_str(&format!("org_{suffix}")).unwrap();
    let grant = DeviceGrantId::from_str(&format!("device_grant_{suffix}")).unwrap();
    let credential = CliCredentialId::from_str(&format!("cli_credential_{suffix}")).unwrap();
    let now = "2026-01-01T00:00:00Z".parse().unwrap();
    let platform = Platform {
        os: "linux".parse().unwrap(),
        arch: "aarch64".parse().unwrap(),
    };
    let register = AgentFrame::Register {
        protocol_version: 1,
        project_id: ProjectId::new(42).unwrap(),
        container_id: container.clone(),
        display_name: "api / web".parse().unwrap(),
        build_commit: None,
        build_dirty: None,
        platform: platform.clone(),
        agent_version: "0.1.0".parse().unwrap(),
        runtime: None,
        application_state: ApplicationState::Running,
    };
    write("register", &register);
    write(
        "registered",
        &ServerFrame::Registered {
            smuggler_session_id: session.clone(),
            notices,
            heartbeat_interval_seconds: 15,
            lease_timeout_seconds: 45,
            server_time: now,
        },
    );
    write(
        "heartbeat",
        &AgentFrame::Heartbeat {
            smuggler_session_id: session.clone(),
            application_state: ApplicationState::Running,
        },
    );
    write(
        "heartbeat-ack",
        &ServerFrame::HeartbeatAck {
            smuggler_session_id: session.clone(),
            server_time: now,
        },
    );
    write(
        "instances",
        &InstancesResponse {
            items: vec![ContainerSnapshot {
                project_id: ProjectId::new(42).unwrap(),
                container_id: container,
                display_name: "api / web".parse().unwrap(),
                build_commit: None,
                build_dirty: None,
                platform,
                agent_version: "0.1.0".parse().unwrap(),
                runtime: None,
                application_state: ApplicationState::Running,
                smuggler_session_id: session,
                presence: PresenceStatus::Connected,
                first_seen_at: now,
                last_seen_at: now,
            }],
            next_cursor: None,
            server_time: now,
        },
    );
    write("device-code-request", &DeviceCodeRequest {});
    write(
        "device-code",
        &DeviceCodeResponse {
            device_grant_id: grant,
            device_code: SecretString::new("fixture-only-device-secret".into()),
            user_code: SecretString::new("ABCD-EFGH".into()),
            verification_uri: "https://bootleg.example/device".into(),
            expires_in: DEVICE_CODE_LIFETIME_SECONDS,
            interval: DEVICE_POLL_INTERVAL_SECONDS,
        },
    );
    write(
        "device-token-request",
        &DeviceTokenRequest {
            device_code: SecretString::new("fixture-only-device-secret".into()),
        },
    );
    write(
        "device-token",
        &DeviceTokenResponse {
            cli_credential_id: credential.clone(),
            access_token: SecretString::new("fixture-only-user-bearer".into()),
            token_type: TokenType::Bearer,
            expires_in: CLI_CREDENTIAL_LIFETIME_SECONDS,
        },
    );
    write(
        "identity",
        &IdentityResponse {
            user_id: format!("user_{suffix}").parse().unwrap(),
            email: "dev@example.com".into(),
            cli_credential_id: credential,
            expires_at: now,
        },
    );
    write(
        "orgs",
        &Page {
            items: vec![Organization {
                org_id: org.clone(),
                name: "Example".parse().unwrap(),
                role: OrgRole::Owner,
            }],
            next_cursor: None,
        },
    );
    let project = Project {
        project_id: ProjectId::new(42).unwrap(),
        org_id: org,
        name: "Example".parse().unwrap(),
        url: None,
        active: true,
        end_to_end_encryption: false,
    };
    write(
        "projects",
        &Page {
            items: vec![project.clone()],
            next_cursor: None,
        },
    );
    write(
        "create-project",
        &CreateProjectRequest {
            name: "Example".parse().unwrap(),
            url: None,
        },
    );
    write("rotate-project-token", &RotateProjectTokenRequest {});
    write(
        "project-token",
        &ProjectTokenResponse {
            project,
            project_token: SecretString::new(format!("project_{suffix}")),
        },
    );
    write("page-query", &PageQuery::default());
    for (name, code) in [
        ("unauthorized", ErrorCode::Unauthorized),
        ("forbidden", ErrorCode::Forbidden),
        ("not-found", ErrorCode::NotFound),
        ("validation", ErrorCode::Validation),
        ("rate-limited", ErrorCode::RateLimited),
        ("service-unavailable", ErrorCode::ServiceUnavailable),
        ("protocol-incompatible", ErrorCode::ProtocolIncompatible),
        ("conflict", ErrorCode::Conflict),
        ("quota-exceeded", ErrorCode::QuotaExceeded),
        ("authorization-pending", ErrorCode::AuthorizationPending),
        ("slow-down", ErrorCode::SlowDown),
        ("access-denied", ErrorCode::AccessDenied),
        ("expired-token", ErrorCode::ExpiredToken),
        ("invalid-grant", ErrorCode::InvalidGrant),
    ] {
        let error = ApiError {
            level: if matches!(code, ErrorCode::AuthorizationPending) {
                MessageLevel::Info
            } else if matches!(code, ErrorCode::SlowDown) {
                MessageLevel::Warning
            } else {
                MessageLevel::Error
            },
            code,
            message: "Request could not be completed".parse().unwrap(),
            request_id: "req-fixture-1".parse().unwrap(),
            retry_after_seconds: if matches!(
                code,
                ErrorCode::RateLimited | ErrorCode::SlowDown | ErrorCode::ServiceUnavailable
            ) {
                Some(5)
            } else {
                None
            },
        };
        if code == ErrorCode::ProtocolIncompatible {
            write(
                "ws-error",
                &ServerFrame::Error {
                    error: error.clone(),
                },
            );
        }
        write(&format!("error-{name}"), &ErrorResponse { error });
    }
}
