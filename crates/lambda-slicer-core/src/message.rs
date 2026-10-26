//! Language-neutral messages for users
//!
//! Anything the core reports to a user carries a stable code and named
//! arguments, which the GUI looks up in its catalogs. The English text stays
//! alongside for the CLI, logs and JSON, and as the detail a bug report
//! quotes. Codes are kebab-case and double as catalog message ids, so a code
//! never changes meaning once released

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::Serialize;

/// An argument value. Paths, names and times are text, formatted as the
/// CLI shows them
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Arg {
    Integer(i64),
    Number(f64),
    Text(String),
}

impl From<&str> for Arg {
    fn from(v: &str) -> Self {
        Arg::Text(v.to_string())
    }
}

impl From<String> for Arg {
    fn from(v: String) -> Self {
        Arg::Text(v)
    }
}

impl From<&Path> for Arg {
    fn from(v: &Path) -> Self {
        Arg::Text(v.display().to_string())
    }
}

impl From<f64> for Arg {
    fn from(v: f64) -> Self {
        Arg::Number(v)
    }
}

macro_rules! integer_arg {
    ($($t:ty),*) => {$(
        impl From<$t> for Arg {
            fn from(v: $t) -> Self {
                Arg::Integer(v as i64)
            }
        }
    )*};
}
integer_arg!(u8, u16, u32, i32, i64, u64, usize);

/// Every code the core emits, so catalogs can be checked for completeness.
/// Building a message with any other code fails in debug builds
pub const CODES: &[&str] = &[
    "analysis-panicked",
    "cache-cancelled",
    "cache-database",
    "cache-directory",
    "container-bad-magic",
    "container-frame-bad-time",
    "container-frame-missing-terminal",
    "container-frame-negative-length",
    "container-frame-over-limit",
    "container-frame-truncated",
    "container-frame-unknown-tag",
    "container-io",
    "container-no-directory",
    "container-range",
    "container-too-short",
    "coverage-hltv",
    "coverage-mostly-spectating",
    "coverage-no-local-events",
    "coverage-singleplayer",
    "coverage-unknown",
    "coverage-visible-only",
    "decode-failed",
    "export-active-effect",
    "export-client-dropped-packets",
    "export-clock-starts-over",
    "export-destination-exists",
    "export-empty-range",
    "export-io",
    "export-missing-delta-base",
    "export-missing-parent",
    "export-range-beyond-end",
    "export-same-as-source",
    "export-source-changed",
    "export-source-decode-failed",
    "export-source-failed-crosscheck",
    "export-start-during-loading",
    "export-start-first-group",
    "export-stopped",
    "export-unknown-persistent-message",
    "export-unproven-state",
    "export-unrepresentable-time-field",
    "export-unsupported",
    "export-validation-failed",
    "io-not-found",
    "io-other",
    "io-permission-denied",
    "io-unexpected-end",
    "protocol-unsupported",
    "protocol-unsupported-network",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Message {
    pub code: &'static str,
    pub args: BTreeMap<&'static str, Arg>,
    /// The English text
    pub text: String,
}

impl Message {
    pub fn new(code: &'static str, text: impl Into<String>) -> Self {
        debug_assert!(
            CODES.contains(&code) || code.starts_with("test-"),
            "message code {code} is not in CODES"
        );
        Self {
            code,
            args: BTreeMap::new(),
            text: text.into(),
        }
    }

    pub fn arg(mut self, name: &'static str, value: impl Into<Arg>) -> Self {
        self.args.insert(name, value.into());
        self
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Something with a stable message code, usually an error
pub trait Coded: fmt::Display {
    fn code(&self) -> &'static str;

    fn args(&self) -> Vec<(&'static str, Arg)> {
        Vec::new()
    }

    fn message(&self) -> Message {
        let mut m = Message::new(self.code(), self.to_string());
        for (name, value) in self.args() {
            m.args.insert(name, value);
        }
        m
    }
}

/// A file system error, coded by its kind with the system's text as detail
pub fn io_message(e: &std::io::Error) -> Message {
    use std::io::ErrorKind;

    let code = match e.kind() {
        ErrorKind::NotFound => "io-not-found",
        ErrorKind::PermissionDenied => "io-permission-denied",
        ErrorKind::UnexpectedEof => "io-unexpected-end",
        _ => "io-other",
    };
    Message::new(code, e.to_string()).arg("detail", e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_serialize_with_code_args_and_text() {
        let m = Message::new("container-too-short", "file is 3 bytes").arg("len", 3u64);
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            serde_json::json!({
                "code": "container-too-short",
                "args": {"len": 3},
                "text": "file is 3 bytes"
            })
        );
        assert_eq!(m.to_string(), "file is 3 bytes");
    }

    #[test]
    fn codes_are_sorted_unique_kebab_case() {
        assert!(CODES.windows(2).all(|w| w[0] < w[1]));
        for c in CODES {
            assert!(c.chars().all(|c| c.is_ascii_lowercase() || c == '-'), "{c}");
        }
    }

    #[test]
    fn io_errors_are_coded_by_kind() {
        let e = std::io::Error::from(std::io::ErrorKind::NotFound);
        assert_eq!(io_message(&e).code, "io-not-found");
    }
}
