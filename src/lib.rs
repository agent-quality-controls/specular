//! driftless: deterministic spec-driven development.
//!
//! The library starts at the JSON spec: [`lint`] is the only constructor of a
//! valid [`Spec`]; [`verify`] judges a repository against it and returns a
//! [`Report`] of per-item [`Evidence`]. The library records facts and
//! never judges callers: no roles, no approval, no trust grades.

mod error;
mod evidence;
mod lint;
mod model;
mod verify;

pub use error::{LintError, SpecViolation, VerifyError};
pub use evidence::{
    Evidence, FileStamp, GitDiagnostic, Polarity, Report, Status, VerifierSource, WireEvidence,
};
pub use lint::lint;
pub use model::{
    Category, ContentRequirement, DependencyRequirement, EnumerationRequirement, ExportRequirement,
    Reason, Requirements, Spec, TreeRequirement,
};
pub use verify::verify;
