//! Evidence and the verify report.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::Category;

/// The judged outcome of one requirement. No soft states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// The requirement holds.
    Pass,
    /// The requirement does not hold.
    Fail,
}

/// A declared custom verifier's ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VerifierId(pub String);

/// Which verifier produced a piece of evidence. Recorded, never judged.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VerifierSource {
    /// A verifier shipped with spec3, routed by category.
    Builtin(Category),
    /// A verifier command declared in the spec.
    Custom(VerifierId),
}

impl core::fmt::Display for VerifierSource {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Builtin(category) => write!(f, "builtin:{}", category.as_str()),
            Self::Custom(id) => write!(f, "custom:{}", id.0),
        }
    }
}

/// One evidence line as a custom verifier emits it on stdout.
#[derive(Debug, Deserialize)]
pub struct WireEvidence {
    /// The requirement ID this line judges.
    pub id: String,
    /// The judged outcome.
    pub status: Status,
    /// Concrete failure description.
    #[serde(default)]
    pub message: Option<String>,
    /// What was found.
    #[serde(default)]
    pub observed: Option<serde_json::Value>,
    /// What the contract demands.
    #[serde(default)]
    pub expected: Option<serde_json::Value>,
    /// The repository path involved.
    #[serde(default)]
    pub path: Option<String>,
}

/// One judged requirement in the report.
#[derive(Debug, Serialize)]
pub struct Evidence {
    /// The requirement ID.
    pub id: String,
    /// Which verifier judged it.
    pub source: VerifierSource,
    /// The outcome.
    pub status: Status,
    /// Concrete failure description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// What was found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed: Option<serde_json::Value>,
    /// What the contract demands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<serde_json::Value>,
    /// The repository path involved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// A raw-byte hash of one input-closure file.
#[derive(Debug, Serialize)]
pub struct FileStamp {
    /// The file path as given.
    pub path: String,
    /// Hex SHA-256 of the raw bytes.
    pub sha256: String,
}

/// Git worktree state of one closure file. Diagnostic only, never enforced.
#[derive(Debug, Serialize)]
pub struct GitDiagnostic {
    /// The file path.
    pub path: String,
    /// `clean`, `modified`, `untracked`, `conflicted`, `not-a-repository`,
    /// or `unavailable`.
    pub state: String,
}

/// The verify report: input-closure stamps plus per-requirement evidence.
#[derive(Debug, Serialize)]
pub struct Report {
    /// The spec3 version that produced this report.
    pub spec3_version: String,
    /// Stamp of the spec file.
    pub spec: FileStamp,
    /// Stamps of every declared custom verifier file.
    pub verifier_files: Vec<FileStamp>,
    /// Git state of the closure files.
    pub git: Vec<GitDiagnostic>,
    /// Per-requirement evidence.
    pub evidence: Vec<Evidence>,
}

impl Report {
    /// True when every requirement passed.
    #[must_use]
    pub fn conforms(&self) -> bool {
        self.evidence.iter().all(|e| e.status == Status::Pass)
    }

    /// `(builtin, custom)` evidence counts.
    #[must_use]
    pub fn source_counts(&self) -> (usize, usize) {
        let builtin = self
            .evidence
            .iter()
            .filter(|e| matches!(e.source, VerifierSource::Builtin(_)))
            .count();
        (builtin, self.evidence.len() - builtin)
    }
}
