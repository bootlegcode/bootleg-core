use bootleg_core::{api::*, protocol::*, *};
use mti::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{fs, path::Path};

fn fixture(name: &str) -> String {
    fs::read_to_string(format!("fixtures/v1/{name}.json")).unwrap()
}
fn roundtrip<T: Serialize + DeserializeOwned>(name: &str) {
    let original: Value = serde_json::from_str(&fixture(name)).unwrap();
    let parsed: T = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), original, "{name}");
}
#[test]
fn golden_fixtures_roundtrip() {
    for name in ["register", "heartbeat"] {
        roundtrip::<AgentFrame>(name);
        decode_agent_frame(&fixture(name)).unwrap();
    }
    for name in ["registered", "heartbeat-ack", "ws-error"] {
        roundtrip::<ServerFrame>(name);
        decode_server_frame(&fixture(name)).unwrap();
    }
    roundtrip::<bootleg_core::communication::BootstrapResponse>("bootstrap");
    roundtrip::<InstancesResponse>("instances");
    roundtrip::<DeviceCodeRequest>("device-code-request");
    roundtrip::<DeviceCodeResponse>("device-code");
    roundtrip::<DeviceTokenRequest>("device-token-request");
    roundtrip::<DeviceTokenResponse>("device-token");
    roundtrip::<IdentityResponse>("identity");
    roundtrip::<Page<Organization>>("orgs");
    roundtrip::<Page<Project>>("projects");
    roundtrip::<CreateProjectRequest>("create-project");
    roundtrip::<RotateProjectTokenRequest>("rotate-project-token");
    roundtrip::<ProjectTokenResponse>("project-token");
    roundtrip::<PageQuery>("page-query");
    for entry in fs::read_dir("fixtures/v1").unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_str().unwrap();
        if name.starts_with("error-") {
            roundtrip::<ErrorResponse>(name);
        }
    }
}
#[test]
fn identities_are_canonical_typed_v4() {
    macro_rules! check {
        ($ty:ty, $prefix:literal) => {{
            let id = <$ty>::new();
            let s = id.to_string();
            assert_eq!(s.len(), $prefix.len() + 27);
            assert_eq!(s.parse::<$ty>().unwrap(), id);
            assert_eq!(serde_json::from_value::<$ty>(json!(s)).unwrap(), id);
            assert_ne!(<$ty>::new(), id);
            for bad in [
                s.to_uppercase(),
                format!(" {s}"),
                format!("{s} "),
                s.replacen($prefix, "wrong", 1),
                format!("{}_00000000000000000000000000", $prefix),
                $prefix.create_type_id::<V7>().to_string(),
                format!("{}_80000000008008000000000000", $prefix),
                format!("{}_00000000008000000000000000", $prefix),
            ] {
                assert!(bad.parse::<$ty>().is_err(), "accepted {bad}");
            }
        }};
    }
    check!(ContainerId, "container");
    check!(SmugglerSessionId, "smuggler_session");
    check!(DeviceGrantId, "device_grant");
    check!(CliCredentialId, "cli_credential");
    check!(OrgId, "org");
    check!(UserId, "user");
    check!(ProjectId, "project");
    assert!(
        ContainerId::new()
            .to_string()
            .parse::<SmugglerSessionId>()
            .is_err()
    );
}
#[test]
fn project_identity_is_a_type_id_not_a_json_number() {
    let example = "project_6m8x91b0349vbrq30fxn997q5s";
    let parsed = example.parse::<ProjectId>().unwrap();
    assert_eq!(parsed.to_string(), example);
    assert_eq!(serde_json::to_value(&parsed).unwrap(), json!(example));
    assert_eq!(
        serde_json::from_value::<ProjectId>(json!(example)).unwrap(),
        parsed
    );
    for s in [
        "0",
        "-1",
        "+1",
        "01",
        " 1",
        "1 ",
        "1.0",
        "9223372036854775808",
        "42",
    ] {
        assert!(s.parse::<ProjectId>().is_err(), "{s}");
    }
    assert!(serde_json::from_value::<ProjectId>(json!(42)).is_err());
    let id = ProjectId::new();
    assert_eq!(serde_json::to_value(&id).unwrap(), json!(id.to_string()));
    assert_eq!(id.to_string().parse::<ProjectId>().unwrap(), id);
}
#[test]
fn wire_values_enforce_boundaries_without_normalizing() {
    for bad in ["", " ", " a", "a ", "a\nb", "a\u{7f}b"] {
        assert!(bad.parse::<DisplayName>().is_err());
    }
    assert!("é".repeat(128).parse::<DisplayName>().is_ok());
    assert!("é".repeat(129).parse::<DisplayName>().is_err());
    assert!("<script>".parse::<DisplayName>().is_ok()); // UI must escape, not sanitize identities.
    for n in [40, 64] {
        assert!("a".repeat(n).parse::<BuildCommit>().is_ok());
    }
    for bad in [
        "a".repeat(7),
        "a".repeat(41),
        "A".repeat(40),
        "g".repeat(64),
    ] {
        assert!(bad.parse::<BuildCommit>().is_err());
    }
    for n in [0, 201, u16::MAX] {
        assert!(
            PageQuery {
                limit: n,
                cursor: None
            }
            .validate()
            .is_err()
        );
    }
    for n in [1, 50, 200] {
        PageQuery {
            limit: n,
            cursor: None,
        }
        .validate()
        .unwrap();
    }
}
#[test]
fn protocol_rejects_unknown_commands_versions_fields_and_large_frames() {
    assert!(decode_agent_frame(r#"{"type":"exec","command":"id"}"#).is_err());
    let mut value: Value = serde_json::from_str(&fixture("register")).unwrap();
    value["protocol_version"] = json!(2);
    assert!(matches!(
        decode_agent_frame(&value.to_string()),
        Err(FrameError::UnsupportedVersion)
    ));
    value["protocol_version"] = json!(1);
    value["project_token"] = json!("do-not-echo-me");
    let err = decode_agent_frame(&value.to_string()).unwrap_err();
    assert!(!format!("{err:?} {err}").contains("do-not-echo-me"));
    assert!(decode_agent_frame(&value.to_string()).is_err());
    let text = fixture("heartbeat");
    let padded = format!("{text}{}", " ".repeat(MAX_FRAME_BYTES - text.len()));
    decode_agent_frame(&padded).unwrap();
    assert!(matches!(
        decode_agent_frame(&(padded + " ")),
        Err(FrameError::TooLarge)
    ));
    assert!(decode_agent_frame("not json").is_err());
}
#[test]
fn secret_debug_is_redacted_but_wire_delivery_is_explicitly_possible() {
    let dto: ProjectTokenResponse = serde_json::from_str(&fixture("project-token")).unwrap();
    assert!(!format!("{dto:?}").contains(dto.project_token.expose_secret()));
    assert!(
        serde_json::to_string(&dto)
            .unwrap()
            .contains(dto.project_token.expose_secret())
    );
    let grant: DeviceCodeResponse = serde_json::from_str(&fixture("device-code")).unwrap();
    assert!(!format!("{grant:?}").contains(grant.device_code.expose_secret()));
    assert!(!format!("{grant:?}").contains(grant.user_code.expose_secret()));
    let token: DeviceTokenResponse = serde_json::from_str(&fixture("device-token")).unwrap();
    assert!(!format!("{token:?}").contains(token.access_token.expose_secret()));
}
#[test]
fn published_schemas_match_rust() {
    fn check<T: schemars::JsonSchema>(name: &str) {
        let expected: Value = serde_json::from_str(
            &fs::read_to_string(Path::new("schemas/v1").join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(schemars::schema_for!(T)).unwrap(),
            expected,
            "{name}"
        );
    }
    check::<bootleg_core::communication::BootstrapResponse>("bootstrap-response");
    check::<AgentFrame>("agent-frame");
    check::<ServerFrame>("server-frame");
    check::<ErrorResponse>("error");
    check::<DeviceCodeRequest>("device-code-request");
    check::<DeviceCodeResponse>("device-code-response");
    check::<DeviceTokenRequest>("device-token-request");
    check::<DeviceTokenResponse>("device-token-response");
    check::<IdentityResponse>("identity-response");
    check::<Page<Organization>>("orgs-response");
    check::<Page<Project>>("projects-response");
    check::<CreateProjectRequest>("create-project-request");
    check::<RotateProjectTokenRequest>("rotate-project-token-request");
    check::<ProjectTokenResponse>("project-token-response");
    check::<PageQuery>("page-query");
    check::<InstancesResponse>("instances-response");
}
#[test]
fn stable_error_statuses() {
    for (code, status) in [
        (ErrorCode::Unauthorized, 401),
        (ErrorCode::Forbidden, 403),
        (ErrorCode::NotFound, 404),
        (ErrorCode::Validation, 400),
        (ErrorCode::RateLimited, 429),
        (ErrorCode::ServiceUnavailable, 503),
        (ErrorCode::ProtocolIncompatible, 400),
        (ErrorCode::Conflict, 409),
        (ErrorCode::QuotaExceeded, 429),
        (ErrorCode::AuthorizationPending, 400),
        (ErrorCode::SlowDown, 400),
        (ErrorCode::AccessDenied, 400),
        (ErrorCode::ExpiredToken, 400),
        (ErrorCode::InvalidGrant, 400),
    ] {
        assert_eq!(code.http_status(), status);
    }
}
