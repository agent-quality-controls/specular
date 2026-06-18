use std::collections::{BTreeMap, BTreeSet};

use aqc_filetree::{FileEntry, FileKind, FileTree};
use aqc_fs_utils::{ReadTextOptions, read_text};
use aqc_rust_syntax::{RustEnumDecl, parse_rust_syntax};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

use crate::error::VerifyError;
use crate::evidence::{Evidence, Status, VerifierSource};
use crate::model::{Category, EnumerationRequirement};

pub(crate) fn check_rust_enumerations(
    block: &EnumerationRequirement,
    tree: &FileTree,
    verifier: &str,
    out: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    let files = matched_rust_files(&block.files, tree)?;
    let target = target(block);
    if files.is_empty() {
        emit_all_values(
            block,
            verifier,
            out,
            Status::Fail,
            Some(format!("no files matched {}", block.files.join(", "))),
            None,
        );
        return Ok(());
    }

    let mut parsed_files = Vec::new();
    let mut parse_failures = Vec::new();
    for entry in &files {
        match parse_file(entry) {
            Ok(enums) => parsed_files.push(ParsedFile {
                path: entry.rel_path.clone(),
                enums,
            }),
            Err(message) => {
                parse_failures.push(format!("{} invalid Rust: {message}", entry.rel_path))
            }
        }
    }
    if !parse_failures.is_empty() {
        emit_all_values(
            block,
            verifier,
            out,
            Status::Fail,
            Some(parse_failures.join("; ")),
            None,
        );
        return Ok(());
    }

    let matches = matching_enums(&parsed_files, &block.name);
    if matches.is_empty() {
        emit_all_values(
            block,
            verifier,
            out,
            Status::Fail,
            Some(format!("enum {} not found in matched files", block.name)),
            None,
        );
        return Ok(());
    }

    let variant_sets = distinct_variant_sets(&matches);
    if variant_sets.len() > 1 {
        emit_all_values(
            block,
            verifier,
            out,
            Status::Fail,
            Some(format!(
                "ambiguous enum {}: matched declarations have different variant sets",
                block.name
            )),
            Some(observed_matches(&matches)),
        );
        return Ok(());
    }

    let observed = matches
        .first()
        .map(|matched| matched.variants.clone())
        .unwrap_or_default();
    let observed_set = observed.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for value in &block.values {
        let status = if observed_set.contains(value.as_str()) {
            Status::Pass
        } else {
            Status::Fail
        };
        out.push(evidence(
            target.clone(),
            value,
            verifier,
            status,
            fail_message(status, format!("missing enum variant: {value}")),
            Some(serde_json::json!(observed)),
            Some(serde_json::json!(block.values)),
        ));
    }
    for extra in observed
        .iter()
        .filter(|value| !block.values.iter().any(|expected| expected == *value))
    {
        out.push(evidence(
            target.clone(),
            extra,
            verifier,
            Status::Fail,
            Some(format!("unexpected enum variant: {extra}")),
            Some(serde_json::json!(observed)),
            Some(serde_json::json!(block.values)),
        ));
    }
    Ok(())
}

struct ParsedFile {
    path: String,
    enums: Vec<RustEnumDecl>,
}

#[derive(Clone)]
struct MatchedEnum {
    path: String,
    name: String,
    variants: Vec<String>,
    line: usize,
}

fn matched_rust_files<'a>(
    files: &[String],
    tree: &'a FileTree,
) -> Result<Vec<&'a FileEntry>, VerifyError> {
    let scope = glob_set(files)?;
    Ok(tree
        .entries
        .iter()
        .filter(|entry| entry.kind == FileKind::File && scope.is_match(&entry.rel_path))
        .collect())
}

fn glob_set(patterns: &[String]) -> Result<GlobSet, VerifyError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .map_err(|error| VerifyError::Walk(format!("glob '{pattern}': {error}")))?;
        let _ = builder.add(glob);
    }
    builder
        .build()
        .map_err(|error| VerifyError::Walk(error.to_string()))
}

fn parse_file(entry: &FileEntry) -> Result<Vec<RustEnumDecl>, String> {
    let text = read_text(&entry.abs_path, &ReadTextOptions::default())
        .map_err(|error| format!("unreadable: {error}"))?;
    parse_rust_syntax(&text)
        .map(|syntax| syntax.enums)
        .map_err(|error| error.message)
}

fn matching_enums(files: &[ParsedFile], name: &str) -> Vec<MatchedEnum> {
    let qualified = name.contains("::");
    let mut matches = Vec::new();
    for file in files {
        for enum_decl in &file.enums {
            let full_name = full_name(enum_decl);
            let is_match = if qualified {
                full_name == name
            } else {
                enum_decl.name == name
            };
            if is_match {
                matches.push(MatchedEnum {
                    path: file.path.clone(),
                    name: full_name,
                    variants: enum_decl.variants.clone(),
                    line: enum_decl.line,
                });
            }
        }
    }
    matches
}

fn full_name(enum_decl: &RustEnumDecl) -> String {
    if enum_decl.module_path.is_empty() {
        enum_decl.name.clone()
    } else {
        format!("{}::{}", enum_decl.module_path.join("::"), enum_decl.name)
    }
}

fn distinct_variant_sets(matches: &[MatchedEnum]) -> BTreeSet<Vec<String>> {
    matches
        .iter()
        .map(|matched| {
            let mut variants = matched.variants.clone();
            variants.sort();
            variants
        })
        .collect()
}

fn observed_matches(matches: &[MatchedEnum]) -> serde_json::Value {
    serde_json::json!(
        matches
            .iter()
            .map(|matched| {
                serde_json::json!({
                    "path": matched.path,
                    "name": matched.name,
                    "line": matched.line,
                    "variants": matched.variants
                })
            })
            .collect::<Vec<_>>()
    )
}

fn emit_all_values(
    block: &EnumerationRequirement,
    verifier: &str,
    out: &mut Vec<Evidence>,
    status: Status,
    message: Option<String>,
    observed: Option<serde_json::Value>,
) {
    let target = target(block);
    for value in &block.values {
        out.push(evidence(
            target.clone(),
            value,
            verifier,
            status,
            message.clone(),
            observed.clone(),
            Some(serde_json::json!(block.values)),
        ));
    }
}

fn target(block: &EnumerationRequirement) -> Option<serde_json::Value> {
    Some(serde_json::json!({
        "files": block.files,
        "name": block.name
    }))
}

fn evidence(
    target: Option<serde_json::Value>,
    item: &str,
    verifier: &str,
    status: Status,
    message: Option<String>,
    observed: Option<serde_json::Value>,
    expected: Option<serde_json::Value>,
) -> Evidence {
    Evidence {
        category: Category::Enumerations,
        target,
        polarity: None,
        item: Some(item.to_owned()),
        source: VerifierSource::Builtin,
        verifier: verifier.to_owned(),
        status,
        message,
        observed,
        expected,
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
