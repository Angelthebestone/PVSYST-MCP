//! Crate-wide error type.
//!
//! The `Display` text of every variant is what the MCP client sees, so messages
//! stay human-readable; the variant is what logs and callers can match on.

use std::io;

/// `Result` defaulting to this crate's [`Error`].
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

/// `Error::kind()` of a missing argument.
pub(crate) const KIND_MISSING_ARG: &str = "missing_arg";

/// Why a tool call could not be completed.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    /// A required argument is absent (`hint` may be empty).
    #[error("{}", missing_text(.key, .hint))]
    MissingArg { key: String, hint: String },
    /// Several required arguments are absent; `ejemplo` is a call that supplies them all.
    #[error("Faltan: {}. Ejemplo: {}", .claves.join(", "), .ejemplo)]
    MissingArgs {
        claves: Vec<String>,
        ejemplo: String,
    },
    /// A file, object, project or component does not exist; the text lists candidates.
    #[error("{0}")]
    NotFound(String),
    /// A filesystem operation failed; `context` says what was attempted.
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: io::Error,
    },
    /// PVsyst file content that cannot be interpreted or would be corrupted.
    #[error("{0}")]
    Format(String),
    /// PVsystCLI is unavailable or its run failed.
    #[error("{0}")]
    Cli(String),
    /// Win32 / UI Automation failure while driving the PVsyst GUI.
    #[error("{0}")]
    Gui(String),
    /// A network request (PVGIS) failed.
    #[error("{0}")]
    Network(String),
    /// The request cannot be satisfied as given: bad value, wrong state or precondition.
    #[error("{0}")]
    Invalid(String),
}

fn missing_text(key: &str, hint: &str) -> String {
    if hint.is_empty() {
        format!("Missing required argument '{key}'")
    } else {
        format!("Missing required argument '{key}' ({hint})")
    }
}

impl Error {
    /// Filesystem failure with the operation it interrupted.
    pub(crate) fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    /// Short stable name of the variant, for logs.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::MissingArg { .. } | Self::MissingArgs { .. } => KIND_MISSING_ARG,
            Self::NotFound(_) => "not_found",
            Self::Io { .. } => "io",
            Self::Format(_) => "format",
            Self::Cli(_) => "cli",
            Self::Gui(_) => "gui",
            Self::Network(_) => "network",
            Self::Invalid(_) => "invalid",
        }
    }
}

/// Serialised as its message, so `json!({ "error": e })` shows what the client would read.
impl serde::Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
#[path = "../tests/unit/error.rs"]
mod tests;
