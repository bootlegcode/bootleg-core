use crate::InvalidValue;
use schemars::{JsonSchema, Schema, json_schema};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

macro_rules! text {
    ($name:ident, $max:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);
        impl $name { pub fn as_str(&self) -> &str { &self.0 } }
        impl FromStr for $name {
            type Err = InvalidValue;
            fn from_str(s: &str) -> Result<Self, InvalidValue> {
                if s.is_empty() || s.trim() != s || s.chars().count() > $max || s.chars().any(char::is_control) {
                    return Err(InvalidValue);
                }
                Ok(Self(s.into()))
            }
        }
        impl TryFrom<String> for $name {
            type Error = InvalidValue;
            fn try_from(s: String) -> Result<Self, InvalidValue> { s.parse() }
        }
        impl From<$name> for String { fn from(v: $name) -> Self { v.0 } }
        impl JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> { stringify!($name).into() }
            fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
                json_schema!({"type":"string", "minLength":1, "maxLength":$max,
                    "description":"No Unicode control characters or leading/trailing Unicode whitespace."})
            }
        }
    };
}
text!(DisplayName, 128);
text!(MetadataText, 256);
text!(MessageText, 512);
text!(NoticeId, 128);
text!(AgentVersion, 64);
text!(PlatformName, 32);
text!(RequestId, 128);
text!(Cursor, 512);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct BuildCommit(String);
impl BuildCommit {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for BuildCommit {
    type Err = InvalidValue;
    fn from_str(s: &str) -> Result<Self, InvalidValue> {
        if !matches!(s.len(), 40 | 64)
            || !s
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(InvalidValue);
        }
        Ok(Self(s.into()))
    }
}
impl TryFrom<String> for BuildCommit {
    type Error = InvalidValue;
    fn try_from(s: String) -> Result<Self, InvalidValue> {
        s.parse()
    }
}
impl From<BuildCommit> for String {
    fn from(v: BuildCommit) -> Self {
        v.0
    }
}
impl JsonSchema for BuildCommit {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "BuildCommit".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({"type":"string", "pattern":"^([0-9a-f]{40}|[0-9a-f]{64})$"})
    }
}

/// Transport-only secret. Debug is redacted; Serialize deliberately exposes the
/// value for protocol delivery. Never serialize secret-bearing DTOs as CLI output
/// or logs. No Display implementation, no bearer/ID conversions.
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct SecretString(String);
impl SecretString {
    pub fn new(value: String) -> Self {
        Self(value)
    }
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}
