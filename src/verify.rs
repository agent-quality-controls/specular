//! `verify`: judge the repository against a linted [`Spec`].

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use aqc_filetree::{FileKind, FileTree, WalkOptions, build_file_tree};
use aqc_fs_utils::{ReadTextOptions, read_text};
use aqc_git_helpers::{ChangeStatus, PorcelainOptions, worktree_changes};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use sha2::{Digest as _, Sha256};

use crate::error::VerifyError;
use crate::evidence::{
    Evidence, FileStamp, GitDiagnostic, Polarity, Report, Status, VerifierSource, WireEvidence,
};
use crate::model::{Category, ContentRequirement, Spec, TreeRequirement};

const VERIFIER_TIMEOUT: Duration = Duration::from_secs(60);

/// Check the repository at `root` against the spec loaded from `spec_path`.
///
/// # Errors
///
/// Every [`VerifyError`] variant: stamping, walking, verifier runtime or
/// protocol failures, and broken evidence coverage. Item outcomes are never
/// errors; they are evidence.
pub fn verify(spec: &Spec, root: &Path, spec_path: &Path) -> Result<Report, VerifyError> {
    let spec_stamp = stamp(spec_path)?;
    let mut verifier_files = Vec::new();
    for command in spec.verifiers.values() {
        if let Some(path) = command.first() {
            let path = Path::new(path);
            verifier_files.push(stamp_with_display(&root.join(path), path)?);
        }
    }
    let git = git_diagnostics(root, &spec_stamp, &verifier_files);

    let tree = build_file_tree(root, &WalkOptions::default())
        .map_err(|e| VerifyError::Walk(e.to_string()))?;

    let mut evidence = Vec::new();
    for category in Category::ALL {
        if category_is_empty(spec, category) {
            continue;
        }
        if let Some(command) = spec.verifiers.get(category.as_str()) {
            run_script(command, spec, spec_path, category, root, &mut evidence)?;
        } else {
            match category {
                Category::Tree => check_tree(&spec.requirements.tree, &tree, &mut evidence)?,
                Category::Content => {
                    for block in &spec.requirements.content {
                        check_content(block, &tree, &mut evidence)?;
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

    Ok(Report::new(
        env!("CARGO_PKG_VERSION").to_owned(),
        spec_stamp,
        verifier_files,
        git,
        evidence,
    ))
}

fn stamp(path: &Path) -> Result<FileStamp, VerifyError> {
    stamp_with_display(path, path)
}

fn stamp_with_display(read_path: &Path, display_path: &Path) -> Result<FileStamp, VerifyError> {
    let bytes = std::fs::read(read_path).map_err(|e| VerifyError::Stamp {
        path: display_path.display().to_string(),
        message: e.to_string(),
    })?;
    let digest = Sha256::digest(&bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    Ok(FileStamp {
        path: display_path.display().to_string(),
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

fn category_is_empty(spec: &Spec, category: Category) -> bool {
    let r = &spec.requirements;
    match category {
        Category::Tree => {
            r.tree.required.is_empty() && r.tree.exists.is_empty() && r.tree.forbidden.is_empty()
        }
        Category::Content => r.content.is_empty(),
        Category::Dependencies => r.dependencies.is_empty(),
        Category::Exports => r.exports.is_empty(),
        Category::Enumerations => r.enumerations.is_empty(),
        Category::Custom => r.custom.is_empty(),
    }
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

fn check_tree(
    requirement: &TreeRequirement,
    tree: &FileTree,
    out: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    for path in &requirement.required {
        let status = if tree.entry(path).is_some() {
            Status::Pass
        } else {
            Status::Fail
        };
        out.push(typed_evidence(
            Category::Tree,
            None,
            Polarity::Required,
            path,
            status,
            fail_message(status, format!("missing required path: {path}")),
        ));
    }
    for path in &requirement.exists {
        let status = if tree.entry(path).is_some() {
            Status::Pass
        } else {
            Status::Fail
        };
        out.push(typed_evidence(
            Category::Tree,
            None,
            Polarity::Exists,
            path,
            status,
            fail_message(status, format!("path does not exist: {path}")),
        ));
    }
    for pattern in &requirement.forbidden {
        let set = glob_set(std::slice::from_ref(pattern))?;
        let hits: Vec<&str> = tree
            .entries
            .iter()
            .filter(|e| set.is_match(&e.rel_path))
            .map(|e| e.rel_path.as_str())
            .take(5)
            .collect();
        let status = if hits.is_empty() {
            Status::Pass
        } else {
            Status::Fail
        };
        out.push(typed_evidence(
            Category::Tree,
            None,
            Polarity::Forbidden,
            pattern,
            status,
            fail_message(
                status,
                format!("forbidden paths present: {}", hits.join(", ")),
            ),
        ));
    }
    Ok(())
}

fn check_content(
    block: &ContentRequirement,
    tree: &FileTree,
    out: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    let scope = glob_set(&block.files)?;
    let mut files = Vec::new();
    for entry in &tree.entries {
        if entry.kind == FileKind::File && scope.is_match(&entry.rel_path) {
            files.push(entry);
        }
    }
    let target = Some(serde_json::json!(block.files));
    for needle in &block.required {
        let mut failures = Vec::new();
        if files.is_empty() {
            failures.push(format!("no files matched {}", block.files.join(", ")));
        }
        for entry in &files {
            let text = read_text(&entry.abs_path, &ReadTextOptions::default())
                .map_err(|e| VerifyError::Walk(format!("unreadable {}: {e}", entry.rel_path)))?;
            if !text.contains(needle) {
                failures.push(format!("missing in {}", entry.rel_path));
            }
        }
        let status = if failures.is_empty() {
            Status::Pass
        } else {
            Status::Fail
        };
        out.push(typed_evidence(
            Category::Content,
            target.clone(),
            Polarity::Required,
            needle,
            status,
            fail_message(status, failures.join("; ")),
        ));
    }
    for needle in &block.exists {
        let mut found = false;
        let mut unreadable = Vec::new();
        for entry in &files {
            match read_text(&entry.abs_path, &ReadTextOptions::default()) {
                Ok(text) if text.contains(needle) => found = true,
                Ok(_) => {}
                Err(error) => unreadable.push(format!("unreadable {}: {error}", entry.rel_path)),
            }
        }
        let status = if found { Status::Pass } else { Status::Fail };
        let message = if status == Status::Fail {
            if files.is_empty() {
                Some(format!("no files matched {}", block.files.join(", ")))
            } else if unreadable.is_empty() {
                Some(format!("found in no scoped file: {needle}"))
            } else {
                Some(unreadable.join("; "))
            }
        } else {
            None
        };
        out.push(typed_evidence(
            Category::Content,
            target.clone(),
            Polarity::Exists,
            needle,
            status,
            message,
        ));
    }
    for needle in &block.forbidden {
        let mut hits = Vec::new();
        for entry in &files {
            let text = read_text(&entry.abs_path, &ReadTextOptions::default())
                .map_err(|e| VerifyError::Walk(format!("unreadable {}: {e}", entry.rel_path)))?;
            if text.contains(needle) {
                hits.push(entry.rel_path.as_str());
            }
        }
        let status = if hits.is_empty() {
            Status::Pass
        } else {
            Status::Fail
        };
        out.push(typed_evidence(
            Category::Content,
            target.clone(),
            Polarity::Forbidden,
            needle,
            status,
            fail_message(
                status,
                format!("forbidden substring found in {}", hits.join(", ")),
            ),
        ));
    }
    Ok(())
}

fn typed_evidence(
    category: Category,
    target: Option<serde_json::Value>,
    polarity: Polarity,
    item: &str,
    status: Status,
    message: Option<String>,
) -> Evidence {
    Evidence {
        category,
        target,
        polarity: Some(polarity),
        item: Some(item.to_owned()),
        source: VerifierSource::Builtin,
        status,
        message,
        observed: None,
        expected: None,
        path: None,
        extra: BTreeMap::new(),
    }
}

fn fail_message(status: Status, message: String) -> Option<String> {
    if status == Status::Fail {
        Some(message)
    } else {
        None
    }
}

fn run_script(
    command: &[String],
    spec: &Spec,
    spec_path: &Path,
    category: Category,
    root: &Path,
    evidence: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    if category == Category::Custom {
        let lines = run_command(command, spec_path, category, None, root)?;
        if lines.is_empty() {
            return Err(VerifyError::Verifier {
                id: category.as_str().to_owned(),
                message: "custom verifier emitted zero evidence lines".to_owned(),
            });
        }
        for line in lines {
            evidence.push(custom_evidence(category, line));
        }
        return Ok(());
    }

    let blocks = typed_blocks(spec, category);
    for (block_index, block) in blocks.into_iter().enumerate() {
        let lines = run_command(command, spec_path, category, Some(block_index), root)?;
        check_script_block(category, block_index, &block, lines, evidence)?;
    }
    Ok(())
}

fn run_command(
    command: &[String],
    spec_path: &Path,
    category: Category,
    block_index: Option<usize>,
    root: &Path,
) -> Result<Vec<WireEvidence>, VerifyError> {
    let label = match block_index {
        Some(index) => format!("{}[{index}]", category.as_str()),
        None => category.as_str().to_owned(),
    };
    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..])
        .arg(spec_path)
        .arg(category.as_str())
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(index) = block_index {
        cmd.arg(index.to_string());
    }
    let output = run_with_timeout(cmd, &label)?;
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
    let mut lines = Vec::new();
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        let wire: WireEvidence = serde_json::from_str(line).map_err(|e| VerifyError::Verifier {
            id: label.clone(),
            message: format!("protocol violation in line '{line}': {e}"),
        })?;
        lines.push(wire);
    }
    Ok(lines)
}

fn run_with_timeout(mut command: Command, label: &str) -> Result<Output, VerifyError> {
    let mut child = command.spawn().map_err(|e| VerifyError::Verifier {
        id: label.to_owned(),
        message: e.to_string(),
    })?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child.wait_with_output().map_err(|e| VerifyError::Verifier {
                    id: label.to_owned(),
                    message: e.to_string(),
                });
            }
            Ok(None) if started.elapsed() >= VERIFIER_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(VerifyError::Verifier {
                    id: label.to_owned(),
                    message: "timed out after 60 seconds".to_owned(),
                });
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => {
                return Err(VerifyError::Verifier {
                    id: label.to_owned(),
                    message: error.to_string(),
                });
            }
        }
    }
}

#[derive(Debug)]
struct ScriptBlock {
    target: Option<serde_json::Value>,
    items: Vec<ScriptItem>,
}

#[derive(Debug)]
struct ScriptItem {
    value: String,
    polarity: Option<Polarity>,
}

fn typed_blocks(spec: &Spec, category: Category) -> Vec<ScriptBlock> {
    let r = &spec.requirements;
    match category {
        Category::Tree => vec![ScriptBlock {
            target: None,
            items: quantified_items(&r.tree.required, &r.tree.exists, &r.tree.forbidden),
        }],
        Category::Content => r
            .content
            .iter()
            .map(|x| ScriptBlock {
                target: Some(serde_json::json!(x.files)),
                items: quantified_items(&x.required, &x.exists, &x.forbidden),
            })
            .collect(),
        Category::Dependencies => r
            .dependencies
            .iter()
            .map(|x| ScriptBlock {
                target: Some(serde_json::json!(x.manifests)),
                items: quantified_items(&x.required, &x.exists, &x.forbidden),
            })
            .collect(),
        Category::Exports => r
            .exports
            .iter()
            .map(|x| ScriptBlock {
                target: Some(serde_json::json!(x.package)),
                items: quantified_items(&x.required, &x.exists, &x.forbidden),
            })
            .collect(),
        Category::Enumerations => r
            .enumerations
            .iter()
            .map(|x| ScriptBlock {
                target: Some(serde_json::json!(x.name)),
                items: x
                    .values
                    .iter()
                    .map(|value| ScriptItem {
                        value: value.clone(),
                        polarity: None,
                    })
                    .collect(),
            })
            .collect(),
        Category::Custom => Vec::new(),
    }
}

fn quantified_items(
    required: &[String],
    exists: &[String],
    forbidden: &[String],
) -> Vec<ScriptItem> {
    required
        .iter()
        .map(|value| ScriptItem {
            value: value.clone(),
            polarity: Some(Polarity::Required),
        })
        .chain(exists.iter().map(|value| ScriptItem {
            value: value.clone(),
            polarity: Some(Polarity::Exists),
        }))
        .chain(forbidden.iter().map(|value| ScriptItem {
            value: value.clone(),
            polarity: Some(Polarity::Forbidden),
        }))
        .collect()
}

fn check_script_block(
    category: Category,
    block_index: usize,
    block: &ScriptBlock,
    lines: Vec<WireEvidence>,
    evidence: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    let expected: HashMap<&str, Option<Polarity>> = block
        .items
        .iter()
        .map(|item| (item.value.as_str(), item.polarity))
        .collect();
    let mut seen: HashSet<String> = HashSet::new();
    for line in lines {
        let Some(item) = line.item.clone() else {
            return Err(VerifyError::Verifier {
                id: format!("{}[{block_index}]", category.as_str()),
                message: "typed evidence line is missing item".to_owned(),
            });
        };
        if !line.extra.is_empty() {
            return Err(VerifyError::Verifier {
                id: format!("{}[{block_index}]", category.as_str()),
                message: format!("typed evidence item '{item}' contains unsupported extra fields"),
            });
        }
        let Some(polarity) = expected.get(item.as_str()).copied() else {
            return Err(VerifyError::Verifier {
                id: format!("{}[{block_index}]", category.as_str()),
                message: format!(
                    "reported item '{item}', expected one of [{}]",
                    expected_items(block).join(", ")
                ),
            });
        };
        if !seen.insert(item.clone()) {
            return Err(VerifyError::Verifier {
                id: format!("{}[{block_index}]", category.as_str()),
                message: format!("reported item '{item}' more than once"),
            });
        }
        evidence.push(Evidence {
            category,
            target: block.target.clone(),
            polarity,
            item: Some(item),
            source: VerifierSource::Custom,
            status: line.status,
            message: line.message,
            observed: line.observed,
            expected: line.expected,
            path: line.path,
            extra: BTreeMap::new(),
        });
    }
    for item in &block.items {
        if !seen.contains(&item.value) {
            return Err(VerifyError::Coverage(format!(
                "{}[{block_index}] item '{}' produced no evidence",
                category.as_str(),
                item.value
            )));
        }
    }
    Ok(())
}

fn expected_items(block: &ScriptBlock) -> Vec<String> {
    block.items.iter().map(|item| item.value.clone()).collect()
}

fn custom_evidence(category: Category, line: WireEvidence) -> Evidence {
    Evidence {
        category,
        target: None,
        polarity: None,
        item: line.item,
        source: VerifierSource::Custom,
        status: line.status,
        message: line.message,
        observed: line.observed,
        expected: line.expected,
        path: line.path,
        extra: line.extra,
    }
}
