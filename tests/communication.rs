use bootleg_core::{MessageText, api::ErrorResponse, communication::*};
use serde_json::json;

#[test]
fn every_product_has_independent_version_attribution() {
    for (name, kind) in [("cli", ClientKind::Cli), ("smuggler", ClientKind::Smuggler)] {
        let client = ClientMetadata::from_headers(name, "3.0.0-rc.1+build123", "1").unwrap();
        assert_eq!(client.client, kind);
        assert_eq!(kind.header_value(), name);
        assert_eq!(client.version.as_str(), "3.0.0-rc.1+build123");
    }
    for name in ["", "browser", "CLI", "cli,smuggler"] {
        assert!(ClientMetadata::from_headers(name, "1.0.0", "1").is_err());
    }
    for version in [
        "",
        "1.0\r\ninjected:yes",
        " version",
        "a b",
        "é",
        &"a".repeat(65),
    ] {
        assert!(ClientMetadata::from_headers("cli", version, "1").is_err());
    }
    assert!(ClientMetadata::from_headers("cli", &"a".repeat(64), "1").is_ok());
    for protocol in ["", "01", "2", "1 ", "v1"] {
        assert_eq!(
            ClientMetadata::from_headers("cli", "1.0.0", protocol).unwrap_err(),
            ClientMetadataError::UnsupportedProtocol
        );
    }
}
#[test]
fn server_can_change_wording_without_changing_machine_behavior() {
    for level in ["info", "warning", "error"] {
        for text in ["Please retry shortly.", "Maintenance ends at 03:00 UTC."] {
            let original = json!({"error": {
                "code":"service_unavailable", "level":level, "message":text,
                "request_id":"req-1", "retry_after_seconds":30
            }});
            let dto: ErrorResponse = serde_json::from_value(original.clone()).unwrap();
            assert_eq!(dto.error.code.http_status(), 503);
            assert_eq!(dto.error.message.as_str(), text);
            assert_eq!(serde_json::to_value(dto).unwrap(), original);
        }
    }
}
#[test]
fn announcements_are_bounded_unique_and_plain_text() {
    assert!(serde_json::from_value::<Notices>(json!([])).is_ok());
    let notice = |id| json!({"id":format!("notice-{id}"),"level":"warning","message":"Maintenance tomorrow."});
    let four: Vec<_> = (0..4).map(notice).collect();
    let notices: Notices = serde_json::from_value(json!(four)).unwrap();
    assert_eq!(notices.as_slice().len(), MAX_NOTICES);
    assert!(
        serde_json::from_value::<Notices>(json!((0..5).map(notice).collect::<Vec<_>>())).is_err()
    );
    assert!(serde_json::from_value::<Notices>(json!([notice(1), notice(1)])).is_err());
    assert!("é".repeat(512).parse::<MessageText>().is_ok());
    assert!("é".repeat(513).parse::<MessageText>().is_err());
    for bad in ["", "\u{1b}[31mred", "two\nlines", " leading", "trailing "] {
        assert!(bad.parse::<MessageText>().is_err());
    }
    let unknown = json!([{"id":"n1", "level":"fatal", "message":"No."}]);
    assert!(serde_json::from_value::<Notices>(unknown).is_err());
}
