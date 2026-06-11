//! The typed spec model: the contract `lint` produces and `verify` consumes.

use std::collections::BTreeMap;

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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// Required, existing, and forbidden repository paths.
    Tree,
    /// Required, existing, or forbidden fixed substrings in scoped files.
    Content,
    /// Required, existing, and forbidden packages in manifests.
    Dependencies,
    /// Public items exposed by a package.
    Exports,
    /// Closed named value sets.
    Enumerations,
    /// Opaque author-defined checks.
    Custom,
}

impl Category {
    /// Every category, in document order.
    pub const ALL: [Self; 6] = [
        Self::Tree,
        Self::Content,
        Self::Dependencies,
        Self::Exports,
        Self::Enumerations,
        Self::Custom,
    ];

    /// The wire name (same as the JSON key).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tree => "tree",
            Self::Content => "content",
            Self::Dependencies => "dependencies",
            Self::Exports => "exports",
            Self::Enumerations => "enumerations",
            Self::Custom => "custom",
        }
    }

    /// Parse a category from its wire name.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str() == name)
    }

    /// Whether driftless ships a builtin verifier for this category.
    #[must_use]
    pub const fn has_builtin(self) -> bool {
        matches!(self, Self::Tree | Self::Content)
    }
}

/// The spec file: the source contract.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    /// Spec format version; only 1 is supported.
    pub version: u32,
    /// Verifier overrides: category name -> command argv. A category absent here
    /// uses its builtin verifier (tree, content) or fails lint.
    #[serde(default)]
    pub verifiers: BTreeMap<String, Vec<String>>,
    /// Requirements by category. Unused categories may be omitted.
    #[serde(default)]
    pub requirements: Requirements,
}

/// Requirements by category.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Requirements {
    /// Required, existing, and forbidden repository paths.
    #[serde(default)]
    pub tree: TreeRequirement,
    /// Required, existing, or forbidden fixed substrings in scoped files.
    #[serde(default)]
    pub content: Vec<ContentRequirement>,
    /// Required, existing, and forbidden packages in manifests.
    #[serde(default)]
    pub dependencies: Vec<DependencyRequirement>,
    /// Public items exposed by a package.
    #[serde(default)]
    pub exports: Vec<ExportRequirement>,
    /// Closed named value sets.
    #[serde(default)]
    pub enumerations: Vec<EnumerationRequirement>,
    /// Opaque author-defined checks.
    #[serde(default)]
    pub custom: Vec<serde_json::Value>,
}

/// Required, existing, and forbidden repository paths.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TreeRequirement {
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
    /// Paths that must exist.
    #[serde(default)]
    pub required: Vec<String>,
    /// Paths where at least one must exist. Rejected by lint.
    #[serde(default)]
    pub exists: Vec<String>,
    /// Globs no repository path may match.
    #[serde(default)]
    pub forbidden: Vec<String>,
}

/// Required, existing, or forbidden fixed substrings in scoped files.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContentRequirement {
    /// Globs scoping which files are read.
    #[serde(default)]
    pub files: Vec<String>,
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
    /// Substrings that must exist in every scoped file.
    #[serde(default)]
    pub required: Vec<String>,
    /// Substrings that must exist in at least one scoped file.
    #[serde(default)]
    pub exists: Vec<String>,
    /// Substrings no scoped file may contain.
    #[serde(default)]
    pub forbidden: Vec<String>,
}

/// Required, existing, and forbidden packages in manifests.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DependencyRequirement {
    /// Globs selecting the manifests to inspect.
    #[serde(default)]
    pub manifests: Vec<String>,
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
    /// Packages that must be declared in every matched manifest.
    #[serde(default)]
    pub required: Vec<String>,
    /// Packages that must be declared in at least one matched manifest.
    #[serde(default)]
    pub exists: Vec<String>,
    /// Packages or package globs that must not be declared.
    #[serde(default)]
    pub forbidden: Vec<String>,
}

/// Public items exposed by a package.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ExportRequirement {
    /// The package whose public surface is checked.
    #[serde(default)]
    pub package: String,
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
    /// Public item names that must exist.
    #[serde(default)]
    pub required: Vec<String>,
    /// Public item names where at least one must exist. Rejected by lint.
    #[serde(default)]
    pub exists: Vec<String>,
    /// Public item names that must not exist.
    #[serde(default)]
    pub forbidden: Vec<String>,
}

/// A closed value set; drift in either direction fails.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EnumerationRequirement {
    /// The set name.
    #[serde(default)]
    pub name: String,
    /// Plan citations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
    /// The exact value set.
    #[serde(default)]
    pub values: Vec<String>,
}
