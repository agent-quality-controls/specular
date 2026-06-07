//! `verify`: judge the repository against a linted [`Spec`].

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use aqc_filetree::{FileKind, FileTree, WalkOptions, build_file_tree};
use aqc_fs_utils::{ReadTextOptions, read_text};
use aqc_git_helpers::{ChangeStatus, PorcelainOptions, worktree_changes};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use sha2::{Digest as _, Sha256};

use crate::error::VerifyError;
use crate::evidence::{
    Evidence, FileStamp, GitDiagnostic, Report, Status, VerifierSource, WireEvidence,
};
use crate::model::{Category, ContentRequirement, Spec, TreeRequirement};

/// Check the repository at `root` against the spec loaded from `spec_path`.
///
/// # Errors
///
/// Every [`VerifyError`] variant: stamping, walking, custom verifier runtime
/// or protocol failures, and broken evidence coverage. Requirement outcomes
/// are never errors; they are evidence.
pub fn verify(spec: &Spec, root: &Path, spec_path: &Path) -> Result<Report, VerifyError> {
    let spec_stamp = stamp(spec_path)?;
    let mut verifier_files = Vec::new();
    for command in spec.verifiers.values() {
        verifier_files.push(stamp(Path::new(&command[0]))?);
    }
    let git = git_diagnostics(root, &spec_stamp, &verifier_files);

    let tree = build_file_tree(root, &WalkOptions::default())
        .map_err(|e| VerifyError::Walk(e.to_string()))?;

    let mut evidence = Vec::new();
    for category in Category::ALL {
        let ids = category_ids(spec, category);
        if ids.is_empty() {
            continue;
        }
        if let Some(command) = spec.verifiers.get(category.as_str()) {
            run_override(command, spec_path, category, &ids, root, &mut evidence)?;
        } else {
            match category {
                Category::Tree => {
                    for requirement in &spec.requirements.tree {
                        evidence.push(check_tree(requirement, &tree)?);
                    }
                }
                Category::Content => {
                    for requirement in &spec.requirements.content {
                        evidence.push(check_content(requirement, &tree)?);
                    }
                }
                _ => {
                    return Err(VerifyError::Coverage(format!(
                        "category '{}' has no verifier (lint should have caught this)",
                        category.as_str()
                    )));
                }
            }
        }
    }
    check_coverage(spec, &evidence)?;

    Ok(Report {
        spec3_version: env!("CARGO_PKG_VERSION").to_owned(),
        spec: spec_stamp,
        verifier_files,
        git,
        evidence,
    })
}

fn stamp(path: &Path) -> Result<FileStamp, VerifyError> {
    let bytes = std::fs::read(path).map_err(|e| VerifyError::Stamp {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    let digest = Sha256::digest(&bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    Ok(FileStamp {
        path: path.display().to_string(),
        sha256: hex,
    })
}

fn git_diagnostics(
    root: &Path,
    spec_stamp: &FileStamp,
    verifier_files: &[FileStamp],
) -> Vec<GitDiagnostic> {
    let paths: Vec<&str> = std::iter::once(spec_stamp.path.as_str())
        .chain(verifier_files.iter().map(|s| s.path.as_str()))
        .collect();
    let changes = match worktree_changes(root, PorcelainOptions::default()) {
        Ok(changes) => changes,
        Err(error) => {
            let state = match error {
                aqc_git_helpers::GitError::NotARepository => "not-a-repository",
                _ => "unavailable",
            };
            return paths
                .iter()
                .map(|p| GitDiagnostic {
                    path: (*p).to_owned(),
                    state: state.to_owned(),
                })
                .collect();
        }
    };
    let by_path: HashMap<&str, &ChangeStatus> = changes
        .iter()
        .map(|c| (c.path.as_str(), &c.status))
        .collect();
    // Porcelain v1 collapses an untracked directory into one `?? dir/` record;
    // a file inside it is untracked even though no record names it exactly.
    let untracked_dirs: Vec<&str> = changes
        .iter()
        .filter(|c| matches!(c.status, ChangeStatus::Untracked) && c.path.ends_with('/'))
        .map(|c| c.path.as_str())
        .collect();
    paths
        .iter()
        .map(|p| {
            let state = match by_path.get(p) {
                None if untracked_dirs.iter().any(|d| p.starts_with(d)) => "untracked",
                None => "clean",
                Some(ChangeStatus::Untracked) => "untracked",
                Some(ChangeStatus::Conflicted) => "conflicted",
                Some(_) => "modified",
            };
            GitDiagnostic {
                path: (*p).to_owned(),
                state: state.to_owned(),
            }
        })
        .collect()
}

/// Spec glob semantics: `*` does not cross `/`; `**` does.
fn glob_set(patterns: &[String]) -> Result<GlobSet, VerifyError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .map_err(|e| VerifyError::Walk(format!("glob '{pattern}': {e}")))?;
        let _ = builder.add(glob);
    }
    builder
        .build()
        .map_err(|e| VerifyError::Walk(e.to_string()))
}

fn check_tree(requirement: &TreeRequirement, tree: &FileTree) -> Result<Evidence, VerifyError> {
    let mut problems = Vec::new();
    for path in &requirement.required_paths {
        if tree.entry(path).is_none() {
            problems.push(format!("missing required path: {path}"));
        }
    }
    if !requirement.forbidden_globs.is_empty() {
        let set = glob_set(&requirement.forbidden_globs)?;
        let hits: Vec<&str> = tree
            .entries
            .iter()
            .filter(|e| set.is_match(&e.rel_path))
            .map(|e| e.rel_path.as_str())
            .take(5)
            .collect();
        if !hits.is_empty() {
            problems.push(format!("forbidden paths present: {}", hits.join(", ")));
        }
    }
    Ok(to_evidence(&requirement.id, Category::Tree, problems))
}

fn check_content(
    requirement: &ContentRequirement,
    tree: &FileTree,
) -> Result<Evidence, VerifyError> {
    let scope = glob_set(&requirement.files)?;
    let mut problems = Vec::new();
    let mut required_found: Vec<bool> = vec![false; requirement.required_substrings.len()];
    for entry in &tree.entries {
        if entry.kind != FileKind::File || !scope.is_match(&entry.rel_path) {
            continue;
        }
        let text = match read_text(&entry.abs_path, &ReadTextOptions::default()) {
            Ok(text) => text,
            Err(error) => {
                problems.push(format!("unreadable {}: {error}", entry.rel_path));
                continue;
            }
        };
        for needle in &requirement.forbidden_substrings {
            if text.contains(needle) {
                problems.push(format!("forbidden '{needle}' found in {}", entry.rel_path));
            }
        }
        for (index, needle) in requirement.required_substrings.iter().enumerate() {
            if text.contains(needle) {
                required_found[index] = true;
            }
        }
    }
    for (index, found) in required_found.iter().enumerate() {
        if !found {
            problems.push(format!(
                "required '{}' found in no scoped file",
                requirement.required_substrings[index]
            ));
        }
    }
    Ok(to_evidence(&requirement.id, Category::Content, problems))
}

fn to_evidence(id: &str, category: Category, problems: Vec<String>) -> Evidence {
    let status = if problems.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    };
    Evidence {
        id: id.to_owned(),
        source: VerifierSource::Builtin(category),
        status,
        message: if problems.is_empty() {
            None
        } else {
            Some(problems.join("; "))
        },
        observed: None,
        expected: None,
        path: None,
    }
}

/// Requirement IDs in one category, in document order.
fn category_ids(spec: &Spec, category: Category) -> Vec<String> {
    let r = &spec.requirements;
    match category {
        Category::Tree => r.tree.iter().map(|x| x.id.clone()).collect(),
        Category::Content => r.content.iter().map(|x| x.id.clone()).collect(),
        Category::Dependencies => r.dependencies.iter().map(|x| x.id.clone()).collect(),
        Category::Exports => r.exports.iter().map(|x| x.id.clone()).collect(),
        Category::Enumerations => r.enumerations.iter().map(|x| x.id.clone()).collect(),
        Category::Schemas => r.schemas.iter().map(|x| x.id.clone()).collect(),
    }
}

/// Run a category's override command: `<command...> <spec> <category>`. Each
/// emitted line must judge a requirement of that category; missing or duplicate
/// ids are caught by the global coverage check.
fn run_override(
    command: &[String],
    spec_path: &Path,
    category: Category,
    ids: &[String],
    root: &Path,
    evidence: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    let label = category.as_str().to_owned();
    let output = Command::new(&command[0])
        .args(&command[1..])
        .arg(spec_path)
        .arg(category.as_str())
        .current_dir(root)
        .output()
        .map_err(|e| VerifyError::Verifier {
            id: label.clone(),
            message: e.to_string(),
        })?;
    if !output.status.success() {
        return Err(VerifyError::Verifier {
            id: label,
            message: format!(
                "exit {:?}; stderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        let wire: WireEvidence = serde_json::from_str(line).map_err(|e| VerifyError::Verifier {
            id: label.clone(),
            message: format!("protocol violation in line '{line}': {e}"),
        })?;
        if !ids.contains(&wire.id) {
            return Err(VerifyError::Verifier {
                id: label.clone(),
                message: format!(
                    "reported '{}', which is not a requirement in category '{label}'",
                    wire.id
                ),
            });
        }
        evidence.push(Evidence {
            id: wire.id,
            source: VerifierSource::Custom(category),
            status: wire.status,
            message: wire.message,
            observed: wire.observed,
            expected: wire.expected,
            path: wire.path,
        });
    }
    Ok(())
}

fn check_coverage(spec: &Spec, evidence: &[Evidence]) -> Result<(), VerifyError> {
    let r = &spec.requirements;
    let expected: Vec<&str> = r
        .tree
        .iter()
        .map(|x| x.id.as_str())
        .chain(r.content.iter().map(|x| x.id.as_str()))
        .chain(r.dependencies.iter().map(|x| x.id.as_str()))
        .chain(r.exports.iter().map(|x| x.id.as_str()))
        .chain(r.enumerations.iter().map(|x| x.id.as_str()))
        .chain(r.schemas.iter().map(|x| x.id.as_str()))
        .collect();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for item in evidence {
        *counts.entry(item.id.as_str()).or_insert(0) += 1;
    }
    for id in &expected {
        match counts.get(id) {
            None => {
                return Err(VerifyError::Coverage(format!(
                    "'{id}' produced no evidence"
                )));
            }
            Some(1) => {}
            Some(n) => {
                return Err(VerifyError::Coverage(format!(
                    "'{id}' reported {n} times; exactly once required"
                )));
            }
        }
    }
    Ok(())
}
