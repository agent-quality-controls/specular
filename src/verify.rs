//! `verify`: judge the repository against a linted [`Spec`].

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
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
use crate::model::{
    Category, ContentRequirement, CustomRequirement, DependencyRequirement, Spec, TreeRequirement,
    VerifierCommand,
};

use crate::cargo_dependencies;

const VERIFIER_TIMEOUT: Duration = Duration::from_secs(60);

struct VerifyContext<'a> {
    spec: &'a Spec,
    spec_path: &'a Path,
    root: &'a Path,
    tree: &'a FileTree,
}

/// Check the repository at `root` against the spec loaded from `spec_path`.
///
/// # Errors
///
/// Every [`VerifyError`] variant: stamping, walking, verifier runtime or
/// protocol failures, and broken evidence coverage. Item outcomes are never
/// errors; they are evidence.
pub fn verify(spec: &Spec, root: &Path, spec_path: &Path) -> Result<Report, VerifyError> {
    let spec_stamp = stamp(spec_path)?;
    let verifier_files = verifier_file_stamps(spec, root)?;
    let git = git_diagnostics(root, &spec_stamp, &verifier_files);

    let tree = build_file_tree(root, &WalkOptions::default())
        .map_err(|e| VerifyError::Walk(e.to_string()))?;
    let ctx = VerifyContext {
        spec,
        spec_path,
        root,
        tree: &tree,
    };

    let mut evidence = Vec::new();
    for category in Category::ALL {
        if category_is_empty(spec, category) {
            continue;
        }
        match category {
            Category::Tree => run_tree_block(&ctx, &mut evidence)?,
            Category::Content => {
                for (index, block) in spec.requirements.content.iter().enumerate() {
                    run_typed_block(
                        &ctx,
                        Category::Content,
                        index,
                        &block.verifier,
                        &mut evidence,
                    )?;
                }
            }
            Category::Dependencies => {
                for (index, block) in spec.requirements.dependencies.iter().enumerate() {
                    run_typed_block(
                        &ctx,
                        Category::Dependencies,
                        index,
                        &block.verifier,
                        &mut evidence,
                    )?;
                }
            }
            Category::Exports => {
                for (index, block) in spec.requirements.exports.iter().enumerate() {
                    run_typed_block(
                        &ctx,
                        Category::Exports,
                        index,
                        &block.verifier,
                        &mut evidence,
                    )?;
                }
            }
            Category::Enumerations => {
                for (index, block) in spec.requirements.enumerations.iter().enumerate() {
                    run_typed_block(
                        &ctx,
                        Category::Enumerations,
                        index,
                        &block.verifier,
                        &mut evidence,
                    )?;
                }
            }
            Category::Custom => {
                for (index, block) in spec.requirements.custom.iter().enumerate() {
                    run_custom_block(spec_path, root, index, block, &mut evidence)?;
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

fn verifier_file_stamps(spec: &Spec, root: &Path) -> Result<Vec<FileStamp>, VerifyError> {
    let mut paths = BTreeSet::new();
    for command in verifier_commands(spec) {
        if is_builtin(command) {
            continue;
        }
        for part in command.as_slice() {
            let path = Path::new(part);
            if path.is_absolute() {
                continue;
            }
            if root.join(path).is_file() {
                paths.insert(part.clone());
            }
        }
    }
    paths
        .into_iter()
        .map(|path| {
            let display = Path::new(&path);
            stamp_with_display(&root.join(display), display)
        })
        .collect()
}

fn verifier_commands(spec: &Spec) -> Vec<&VerifierCommand> {
    let mut commands = Vec::new();
    if !category_is_empty(spec, Category::Tree) {
        commands.push(&spec.requirements.tree.verifier);
    }
    commands.extend(
        spec.requirements
            .content
            .iter()
            .map(|block| &block.verifier),
    );
    commands.extend(
        spec.requirements
            .dependencies
            .iter()
            .map(|block| &block.verifier),
    );
    commands.extend(
        spec.requirements
            .exports
            .iter()
            .map(|block| &block.verifier),
    );
    commands.extend(
        spec.requirements
            .enumerations
            .iter()
            .map(|block| &block.verifier),
    );
    commands.extend(spec.requirements.custom.iter().map(|block| &block.verifier));
    commands
}

fn is_builtin(command: &VerifierCommand) -> bool {
    command
        .first()
        .is_some_and(|selector| selector.starts_with("builtin:"))
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
    verifier: &str,
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
            verifier,
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
            verifier,
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
            verifier,
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
    verifier: &str,
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
            verifier,
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
            verifier,
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
            verifier,
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
    verifier: &str,
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
        verifier: verifier.to_owned(),
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

fn run_tree_block(
    ctx: &VerifyContext<'_>,
    evidence: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    run_typed_block(
        ctx,
        Category::Tree,
        0,
        &ctx.spec.requirements.tree.verifier,
        evidence,
    )
}

fn run_typed_block(
    ctx: &VerifyContext<'_>,
    category: Category,
    block_index: usize,
    verifier: &VerifierCommand,
    evidence: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    match verifier.first() {
        Some("builtin:tree" | "builtin:content" | "builtin:cargo-dependencies") => run_builtin(
            ctx.spec,
            ctx.tree,
            category,
            block_index,
            verifier,
            evidence,
        ),
        Some(selector) if selector.starts_with("builtin:") => Err(VerifyError::Verifier {
            id: format!("{}[{block_index}]", category.as_str()),
            message: format!("builtin verifier '{selector}' cannot run for this block"),
        }),
        Some(_) => {
            let block = typed_block(ctx.spec, category, block_index)?;
            let lines = run_script(
                verifier.as_slice(),
                ctx.spec_path,
                category,
                block_index,
                ctx.root,
            )?;
            check_script_block(
                verifier_label(verifier.as_slice()),
                category,
                block_index,
                &block,
                lines,
                evidence,
            )
        }
        None => Err(VerifyError::Verifier {
            id: format!("{}[{block_index}]", category.as_str()),
            message: "missing verifier command".to_owned(),
        }),
    }
}

fn run_custom_block(
    spec_path: &Path,
    root: &Path,
    block_index: usize,
    block: &CustomRequirement,
    evidence: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    if is_builtin(&block.verifier) {
        return Err(VerifyError::Verifier {
            id: format!("custom[{block_index}]"),
            message: "custom blocks cannot use builtin verifiers".to_owned(),
        });
    }
    let lines = run_script(
        block.verifier.as_slice(),
        spec_path,
        Category::Custom,
        block_index,
        root,
    )?;
    if lines.len() != 1 {
        return Err(VerifyError::Verifier {
            id: format!("custom[{block_index}]"),
            message: format!(
                "custom verifier must emit exactly one evidence line, got {}",
                lines.len()
            ),
        });
    }
    let Some(line) = lines.into_iter().next() else {
        return Err(VerifyError::Verifier {
            id: format!("custom[{block_index}]"),
            message: "custom verifier must emit exactly one evidence line, got 0".to_owned(),
        });
    };
    evidence.push(custom_evidence(
        verifier_label(block.verifier.as_slice()),
        Category::Custom,
        line,
    ));
    Ok(())
}

fn run_builtin(
    spec: &Spec,
    tree: &FileTree,
    category: Category,
    block_index: usize,
    verifier: &VerifierCommand,
    evidence: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    match verifier.first() {
        Some("builtin:tree") if category == Category::Tree => {
            check_tree(&spec.requirements.tree, tree, "builtin:tree", evidence)
        }
        Some("builtin:content") if category == Category::Content => {
            let Some(block) = spec.requirements.content.get(block_index) else {
                return Err(VerifyError::Coverage(format!(
                    "content[{block_index}] does not exist"
                )));
            };
            check_content(block, tree, "builtin:content", evidence)
        }
        Some("builtin:cargo-dependencies") if category == Category::Dependencies => {
            let Some(block) = spec.requirements.dependencies.get(block_index) else {
                return Err(VerifyError::Coverage(format!(
                    "dependencies[{block_index}] does not exist"
                )));
            };
            cargo_dependencies::check_cargo_dependencies(
                block,
                tree,
                "builtin:cargo-dependencies",
                evidence,
            )
        }
        Some(selector) => Err(VerifyError::Verifier {
            id: format!("{}[{block_index}]", category.as_str()),
            message: format!("builtin verifier '{selector}' cannot run for this block"),
        }),
        None => Err(VerifyError::Verifier {
            id: format!("{}[{block_index}]", category.as_str()),
            message: "missing builtin verifier command".to_owned(),
        }),
    }
}

fn run_script(
    command: &[String],
    spec_path: &Path,
    category: Category,
    block_index: usize,
    root: &Path,
) -> Result<Vec<WireEvidence>, VerifyError> {
    let label = format!("{}[{block_index}]", category.as_str());
    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..])
        .arg(spec_path)
        .arg(category.as_str())
        .arg(block_index.to_string())
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
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

fn typed_block(
    spec: &Spec,
    category: Category,
    block_index: usize,
) -> Result<ScriptBlock, VerifyError> {
    let r = &spec.requirements;
    match category {
        Category::Tree => Ok(ScriptBlock {
            target: None,
            items: quantified_items(&r.tree.required, &r.tree.exists, &r.tree.forbidden),
        }),
        Category::Content => r.content.get(block_index).map_or_else(
            || Err(missing_block(category, block_index)),
            |x| {
                Ok(ScriptBlock {
                    target: Some(serde_json::json!(x.files)),
                    items: quantified_items(&x.required, &x.exists, &x.forbidden),
                })
            },
        ),
        Category::Dependencies => r.dependencies.get(block_index).map_or_else(
            || Err(missing_block(category, block_index)),
            |x| {
                Ok(ScriptBlock {
                    target: Some(serde_json::json!(x.files)),
                    items: dependency_script_items(x),
                })
            },
        ),
        Category::Exports => r.exports.get(block_index).map_or_else(
            || Err(missing_block(category, block_index)),
            |x| {
                Ok(ScriptBlock {
                    target: Some(serde_json::json!(x.package)),
                    items: quantified_items(&x.required, &x.exists, &x.forbidden),
                })
            },
        ),
        Category::Enumerations => r.enumerations.get(block_index).map_or_else(
            || Err(missing_block(category, block_index)),
            |x| {
                Ok(ScriptBlock {
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
            },
        ),
        Category::Custom => Err(missing_block(category, block_index)),
    }
}

fn missing_block(category: Category, block_index: usize) -> VerifyError {
    VerifyError::Coverage(format!(
        "{}[{block_index}] does not exist",
        category.as_str()
    ))
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

fn dependency_script_items(block: &DependencyRequirement) -> Vec<ScriptItem> {
    quantified_items(&block.required, &block.exists, &block.forbidden)
        .into_iter()
        .chain(block.forbidden_globs.iter().map(|value| ScriptItem {
            value: value.clone(),
            polarity: Some(Polarity::Forbidden),
        }))
        .collect()
}

fn check_script_block(
    verifier: String,
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
            source: VerifierSource::Script,
            verifier: verifier.clone(),
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

fn custom_evidence(verifier: String, category: Category, line: WireEvidence) -> Evidence {
    Evidence {
        category,
        target: None,
        polarity: None,
        item: line.item,
        source: VerifierSource::Script,
        verifier,
        status: line.status,
        message: line.message,
        observed: line.observed,
        expected: line.expected,
        path: line.path,
        extra: line.extra,
    }
}

fn verifier_label(command: &[String]) -> String {
    command.join(" ")
}
