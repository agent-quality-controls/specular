//! Evidence and the verify report.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::Category;

/// The judged outcome of one item. No soft states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// The item holds.
    Pass,
    /// The item does not hold.
    Fail,
}

/// Which verifier produced a piece of evidence. Recorded, never judged.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum VerifierSource {
    /// The builtin verifier for this category.
    Builtin,
    /// A script verifier.
    Custom,
}

/// The typed quantifier an item belongs to.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Polarity {
    /// Present in every matched place.
    Required,
    /// Present in at least one matched place.
    Exists,
    /// Present in no matched place.
    Forbidden,
}

/// One evidence line as a verifier emits it on stdout.
#[derive(Debug, Deserialize)]
pub struct WireEvidence {
    /// The judged outcome.
    pub status: Status,
    /// The typed item this line judges.
    #[serde(default)]
    pub item: Option<String>,
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
    /// Custom verifier fields, included verbatim for custom evidence.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// One judged item in the report.
#[derive(Debug, Serialize)]
pub struct Evidence {
    /// The category this evidence belongs to.
    pub category: Category,
    /// The block target, if the category has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<serde_json::Value>,
    /// The item's quantifier, for typed categories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polarity: Option<Polarity>,
    /// The typed item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
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
    /// Custom verifier fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
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

/// The verify report: input-closure stamps plus per-item evidence.
#[derive(Debug, Serialize)]
pub struct Report {
    /// The specular version that produced this report.
    pub specular_version: String,
    /// Stamp of the spec file.
    pub spec: FileStamp,
    /// Stamps of every declared verifier file.
    pub verifier_files: Vec<FileStamp>,
    /// Git state of the closure files.
    pub git: Vec<GitDiagnostic>,
    /// Per-item evidence.
    pub evidence: Vec<Evidence>,
    /// True when every item passed.
    pub conforms: bool,
}

impl Report {
    /// Build a report and derive conformance from its evidence.
    #[must_use]
    pub fn new(
        specular_version: String,
        spec: FileStamp,
        verifier_files: Vec<FileStamp>,
        git: Vec<GitDiagnostic>,
        evidence: Vec<Evidence>,
    ) -> Self {
        let conforms = evidence.iter().all(|e| e.status == Status::Pass);
        Self {
            specular_version,
            spec,
            verifier_files,
            git,
            evidence,
            conforms,
        }
    }
}
