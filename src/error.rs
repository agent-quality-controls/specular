//! Error contracts for `lint` and `verify`.

use serde::Serialize;

/// One spec defect found by `lint`.
#[derive(Debug, Clone, Serialize)]
pub struct SpecViolation {
    /// Stable violation code, e.g. `DUPLICATE_TARGET`.
    pub code: String,
    /// What is wrong, concretely.
    pub message: String,
}

/// Why `lint` could not produce a valid `Spec`.
#[derive(Debug)]
pub enum LintError {
    /// The spec file could not be read.
    Read {
        /// The path given.
        path: String,
        /// The I/O failure.
        message: String,
    },
    /// The file is not valid JSON, or does not deserialize.
    Parse {
        /// The parser failure.
        message: String,
    },
    /// The spec parsed but violates the contract; the full list.
    InvalidSpec(Vec<SpecViolation>),
}

impl core::fmt::Display for LintError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Read { path, message } => write!(f, "cannot read {path}: {message}"),
            Self::Parse { message } => write!(f, "parse: {message}"),
            Self::InvalidSpec(violations) => {
                write!(f, "invalid spec ({} violations)", violations.len())
            }
        }
    }
}

impl core::error::Error for LintError {}

/// Why `verify` could not complete. Distinct from a failed requirement:
/// requirement outcomes live in evidence; this is the run breaking.
#[derive(Debug)]
pub enum VerifyError {
    /// An input-closure file could not be read for stamping.
    Stamp {
        /// The file.
        path: String,
        /// The failure.
        message: String,
    },
    /// The repository walk failed.
    Walk(String),
    /// A custom verifier failed to run, exited nonzero, or broke the protocol.
    Verifier {
        /// The verifier ID.
        id: String,
        /// The failure.
        message: String,
    },
    /// Evidence coverage is broken: missing, duplicate, or unknown IDs.
    Coverage(String),
}

impl core::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Stamp { path, message } => write!(f, "cannot stamp {path}: {message}"),
            Self::Walk(message) => write!(f, "walk: {message}"),
            Self::Verifier { id, message } => write!(f, "verifier {id}: {message}"),
            Self::Coverage(message) => write!(f, "coverage: {message}"),
        }
    }
}

impl core::error::Error for VerifyError {}
