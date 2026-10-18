//! Validate the published cross-language examples, not only serde roundtrips.
use serde_json::Value;
use std::fs;
#[test]
fn every_fixture_validates_against_its_published_schema() {
    for entry in fs::read_dir("fixtures/v1").unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_str().unwrap();
        let schema_name = match name {
            "bootstrap" => "bootstrap-response",
            "register" | "heartbeat" => "agent-frame",
            "registered" | "heartbeat-ack" | "ws-error" => "server-frame",
            "instances" => "instances-response",
            "device-code-request" => "device-code-request",
            "device-code" => "device-code-response",
            "device-token-request" => "device-token-request",
            "device-token" => "device-token-response",
            "identity" => "identity-response",
            "orgs" => "orgs-response",
            "projects" => "projects-response",
            "create-project" => "create-project-request",
            "rotate-project-token" => "rotate-project-token-request",
            "project-token" => "project-token-response",
            "page-query" => "page-query",
            name if name.starts_with("error-") => "error",
            _ => panic!("unmapped fixture: {name}"),
        };
        let schema: Value = serde_json::from_str(
            &fs::read_to_string(format!("schemas/v1/{schema_name}.json")).unwrap(),
        )
        .unwrap();
        let example: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let errors: Vec<_> = validator
            .iter_errors(&example)
            .map(|e| e.to_string())
            .collect();
        assert!(errors.is_empty(), "{name}: {errors:?}");
    }
}
