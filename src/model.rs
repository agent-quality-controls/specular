//! The typed spec model: the contract `lint` produces and `verify` consumes.

use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A human explanation: one citation or several.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Reason {
    /// A single citation.
    One(String),
    /// Several citations, kept when rules merge.
    Many(Vec<String>),
}

/// The closed set of requirement categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// Required and forbidden repository paths.
    Tree,
    /// Required or forbidden fixed substrings in scoped files.
    Content,
    /// Required and forbidden crates in manifests.
    Dependencies,
    /// Public types and functions.
    Exports,
    /// Closed variant sets.
    Enumerations,
    /// Durable format artifacts.
    Schemas,
}

impl Category {
    /// The wire name (same as the JSON key).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tree => "tree",
            Self::Content => "content",
            Self::Dependencies => "dependencies",
            Self::Exports => "exports",
            Self::Enumerations => "enumerations",
            Self::Schemas => "schemas",
        }
    }
}

/// The spec file: the source contract.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    /// Spec format version; only 1 is supported.
    #[garde(range(min = 1, max = 1))]
    pub version: u32,
    /// Requirements by category. All six categories are present; unused ones
    /// stay empty arrays.
    #[garde(dive)]
    pub requirements: Requirements,
    /// Declared custom verifiers.
    #[serde(default)]
    #[garde(dive)]
    pub verifiers: Vec<VerifierDecl>,
}

/// Requirements by category.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct Requirements {
    /// Required and forbidden repository paths.
    #[garde(dive)]
    pub tree: Vec<TreeRequirement>,
    /// Required or forbidden fixed substrings in scoped files.
    #[garde(dive)]
    pub content: Vec<ContentRequirement>,
    /// Required and forbidden crates in manifests.
    #[garde(dive)]
    pub dependencies: Vec<DependencyRequirement>,
    /// Public types and functions.
    #[garde(dive)]
    pub exports: Vec<ExportRequirement>,
    /// Closed variant sets.
    #[garde(dive)]
    pub enumerations: Vec<EnumerationRequirement>,
    /// Durable format artifacts.
    #[garde(dive)]
    pub schemas: Vec<SchemaRequirement>,
}

/// Required and forbidden repository paths.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TreeRequirement {
    /// Unique requirement ID.
    #[garde(length(min = 1))]
    pub id: String,
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub reason: Option<Reason>,
    /// Paths that must exist.
    #[garde(skip)]
    pub required_paths: Vec<String>,
    /// Globs no repository path may match.
    #[garde(skip)]
    pub forbidden_globs: Vec<String>,
}

/// Required or forbidden fixed substrings in scoped files.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContentRequirement {
    /// Unique requirement ID.
    #[garde(length(min = 1))]
    pub id: String,
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub reason: Option<Reason>,
    /// Globs scoping which files are read.
    #[garde(length(min = 1))]
    pub files: Vec<String>,
    /// Substrings no scoped file may contain.
    #[garde(skip)]
    pub forbidden_substrings: Vec<String>,
    /// Substrings at least one scoped file must contain.
    #[garde(skip)]
    pub required_substrings: Vec<String>,
}

/// Required and forbidden crates in manifests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DependencyRequirement {
    /// Unique requirement ID.
    #[garde(length(min = 1))]
    pub id: String,
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub reason: Option<Reason>,
    /// Globs selecting the manifests to inspect.
    #[garde(length(min = 1))]
    pub manifest_globs: Vec<String>,
    /// Crates that must be declared.
    #[garde(skip)]
    pub required_crates: Vec<String>,
    /// Crates that must not be declared.
    #[garde(skip)]
    pub forbidden_crates: Vec<String>,
    /// Name prefixes no declared crate may start with.
    #[serde(default)]
    #[garde(skip)]
    pub forbidden_crate_prefixes: Vec<String>,
}

/// Public types and functions that must be exposed.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ExportRequirement {
    /// Unique requirement ID.
    #[garde(length(min = 1))]
    pub id: String,
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub reason: Option<Reason>,
    /// The package whose public surface is checked.
    #[garde(length(min = 1))]
    pub package: String,
    /// Public type names that must exist.
    #[garde(skip)]
    pub types: Vec<String>,
    /// Public function names that must exist.
    #[garde(skip)]
    pub functions: Vec<String>,
}

/// A closed variant set; drift in either direction fails.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EnumerationRequirement {
    /// Unique requirement ID.
    #[garde(length(min = 1))]
    pub id: String,
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub reason: Option<Reason>,
    /// The enum type name.
    #[serde(rename = "type")]
    #[garde(length(min = 1))]
    pub type_name: String,
    /// The exact variant set.
    #[garde(length(min = 1))]
    pub variants: Vec<String>,
}

/// A durable format artifact committed to the repository.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SchemaRequirement {
    /// Unique requirement ID.
    #[garde(length(min = 1))]
    pub id: String,
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub reason: Option<Reason>,
    /// The committed artifact path.
    #[garde(length(min = 1))]
    pub file: String,
}

/// A declared custom verifier.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VerifierDecl {
    /// Unique verifier ID.
    #[garde(length(min = 1))]
    pub id: String,
    /// The command argv; the first element is the executable file.
    #[garde(length(min = 1))]
    pub command: Vec<String>,
    /// Requirement IDs this verifier owns.
    #[garde(length(min = 1))]
    pub requirement_ids: Vec<String>,
    /// Why a custom verifier is needed. Required.
    #[garde(skip)]
    pub reason: Reason,
}
