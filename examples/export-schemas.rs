//! Run from the crate root: cargo run --example export-schemas
use bootleg_core::{api::*, protocol::*};
use schemars::JsonSchema;
use std::{fs, path::Path};
fn export<T: JsonSchema>(name: &str) {
    let text = serde_json::to_string_pretty(&schemars::schema_for!(T)).unwrap() + "\n";
    fs::write(Path::new("schemas/v1").join(format!("{name}.json")), text).unwrap();
}
fn main() {
    fs::create_dir_all("schemas/v1").unwrap();
    export::<bootleg_core::communication::BootstrapResponse>("bootstrap-response");
    export::<AgentFrame>("agent-frame");
    export::<ServerFrame>("server-frame");
    export::<ErrorResponse>("error");
    export::<DeviceCodeRequest>("device-code-request");
    export::<DeviceCodeResponse>("device-code-response");
    export::<DeviceTokenRequest>("device-token-request");
    export::<DeviceTokenResponse>("device-token-response");
    export::<IdentityResponse>("identity-response");
    export::<Page<Organization>>("orgs-response");
    export::<Page<Project>>("projects-response");
    export::<CreateProjectRequest>("create-project-request");
    export::<RotateProjectTokenRequest>("rotate-project-token-request");
    export::<ProjectTokenResponse>("project-token-response");
    export::<PageQuery>("page-query");
    export::<InstancesResponse>("instances-response");
}
