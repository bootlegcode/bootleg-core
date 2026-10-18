//! Public identities only. Project keys are never identities.
use crate::InvalidValue;
use mti::prelude::*;
use schemars::{JsonSchema, Schema, json_schema};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

macro_rules! id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(MagicTypeId);
        impl $name {
            pub fn new() -> Self { Self($prefix.create_type_id::<V4>()) }
        }
        impl Default for $name { fn default() -> Self { Self::new() } }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
        }
        impl FromStr for $name {
            type Err = InvalidValue;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let id = MagicTypeId::from_str(value).map_err(|_| InvalidValue)?;
                let uuid = id.suffix().to_uuid();
                if id.prefix().as_str() != $prefix || uuid.get_version_num() != 4
                    || !matches!(uuid.as_bytes()[8] >> 6, 2) || id.as_str() != value
                    || value.len() != $prefix.len() + 27 {
                    return Err(InvalidValue);
                }
                Ok(Self(id))
            }
        }
        impl TryFrom<String> for $name {
            type Error = InvalidValue;
            fn try_from(s: String) -> Result<Self, Self::Error> { s.parse() }
        }
        impl From<$name> for String { fn from(id: $name) -> Self { id.to_string() } }
        impl JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> { stringify!($name).into() }
            fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
                json_schema!({"type":"string", "pattern":concat!("^", $prefix, "_[0-7][0-9a-hjkmnp-tv-z]{25}$"),
                    "description":"Canonical mti TypeID with RFC 4122 variant UUIDv4; prefix is type-specific."})
            }
        }
    };
}
id!(ContainerId, "container");
id!(SmugglerSessionId, "smuggler_session");
id!(DeviceGrantId, "device_grant");
id!(CliCredentialId, "cli_credential");
// Wire counterparts of existing bounded-context identities; convert at adapters.
id!(OrgId, "org");
id!(UserId, "user");

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ProjectId(i64);
impl ProjectId {
    pub fn new(value: i64) -> Result<Self, InvalidValue> {
        if value > 0 {
            Ok(Self(value))
        } else {
            Err(InvalidValue)
        }
    }
    pub fn get(self) -> i64 {
        self.0
    }
}
impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl FromStr for ProjectId {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, InvalidValue> {
        let id = Self::new(s.parse().map_err(|_| InvalidValue)?)?;
        if id.to_string() != s {
            return Err(InvalidValue);
        }
        Ok(id)
    }
}
impl TryFrom<String> for ProjectId {
    type Error = InvalidValue;
    fn try_from(s: String) -> Result<Self, InvalidValue> {
        s.parse()
    }
}
impl From<ProjectId> for String {
    fn from(id: ProjectId) -> Self {
        id.to_string()
    }
}
impl JsonSchema for ProjectId {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "ProjectId".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({"type":"string", "pattern":"^[1-9][0-9]{0,18}$", "description":"Positive decimal i64, at most 9223372036854775807."})
    }
}
