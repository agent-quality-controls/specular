use std::collections::BTreeMap;

use aqc_cargo_toml_engine::{
    CargoTomlEngine, CargoTomlRequirements, DependencyKind, DependencyPackageGlob,
    DependencyRequirement as CargoDependencyRequirement, DependencyScope, DependencySpec,
};
use aqc_file_engine_core::{
    Engine, EngineRequirement, Finding, ForbiddenGlobRequirements, ItemRequirements, Provenance,
};
use aqc_filetree::{FileEntry, FileKind, FileTree};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use toml_edit::{DocumentMut, Item};

use crate::error::VerifyError;
use crate::evidence::{Evidence, Polarity, Status, VerifierSource};
use crate::model::{Category, DependencyRequirement};

enum DependencyTableTarget {
    Scope(DependencyScope),
    Workspace,
}

pub(crate) fn check_cargo_dependencies(
    block: &DependencyRequirement,
    tree: &FileTree,
    verifier: &str,
    out: &mut Vec<Evidence>,
) -> Result<(), VerifyError> {
    let files = matched_cargo_files(&block.files, tree)?;
    let target = Some(serde_json::json!(block.files));
    let mut states = Vec::new();
    for entry in &files {
        let bytes = read_cargo_file(entry)?;
        let state = match discover_dependency_tables(&bytes) {
            Ok(tables) => FileState {
                path: entry.rel_path.clone(),
                bytes,
                tables,
                parse_error: None,
            },
            Err(message) => FileState {
                path: entry.rel_path.clone(),
                bytes,
                tables: Vec::new(),
                parse_error: Some(message),
            },
        };
        states.push(state);
    }

    for package in &block.required {
        let failures = required_failures(&states, package, &block.files);
        let status = status_for_failures(&failures);
        out.push(evidence(
            target.clone(),
            Polarity::Required,
            package,
            verifier,
            status,
            fail_message(status, failures.join("; ")),
        ));
    }
    for package in &block.exists {
        let (status, message) = exists_status(&states, package, &block.files);
        out.push(evidence(
            target.clone(),
            Polarity::Exists,
            package,
            verifier,
            status,
            message,
        ));
    }
    for package in &block.forbidden {
        let failures = exact_forbidden_failures(&states, package)?;
        let status = status_for_failures(&failures);
        out.push(evidence(
            target.clone(),
            Polarity::Forbidden,
            package,
            verifier,
            status,
            fail_message(status, failures.join("; ")),
        ));
    }
    for glob in &block.forbidden_globs {
        let failures = glob_forbidden_failures(&states, glob)?;
        let status = status_for_failures(&failures);
        out.push(evidence(
            target.clone(),
            Polarity::Forbidden,
            glob,
            verifier,
            status,
            fail_message(status, failures.join("; ")),
        ));
    }
    Ok(())
}

struct FileState {
    path: String,
    bytes: Vec<u8>,
    tables: Vec<DependencyTableTarget>,
    parse_error: Option<String>,
}

fn matched_cargo_files<'a>(
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

fn read_cargo_file(entry: &FileEntry) -> Result<Vec<u8>, VerifyError> {
    std::fs::read(&entry.abs_path)
        .map_err(|error| VerifyError::Walk(format!("unreadable {}: {error}", entry.rel_path)))
}

fn discover_dependency_tables(bytes: &[u8]) -> Result<Vec<DependencyTableTarget>, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    let doc = text
        .parse::<DocumentMut>()
        .map_err(|error| error.to_string())?;
    let mut targets = Vec::new();
    for kind in [
        DependencyKind::Normal,
        DependencyKind::Dev,
        DependencyKind::Build,
    ] {
        if table_exists(&doc, kind) {
            targets.push(DependencyTableTarget::Scope(DependencyScope {
                kind,
                target: None,
            }));
        }
    }
    collect_target_tables(&doc, &mut targets);
    if workspace_dependencies_exists(&doc) {
        targets.push(DependencyTableTarget::Workspace);
    }
    Ok(targets)
}

fn table_exists(doc: &DocumentMut, kind: DependencyKind) -> bool {
    doc.get(kind_key(kind)).and_then(Item::as_table).is_some()
}

fn collect_target_tables(doc: &DocumentMut, out: &mut Vec<DependencyTableTarget>) {
    let Some(targets) = doc.get("target").and_then(Item::as_table) else {
        return;
    };
    for (target, item) in targets {
        let Some(table) = item.as_table() else {
            continue;
        };
        for kind in [
            DependencyKind::Normal,
            DependencyKind::Dev,
            DependencyKind::Build,
        ] {
            if table.get(kind_key(kind)).and_then(Item::as_table).is_some() {
                out.push(DependencyTableTarget::Scope(DependencyScope {
                    kind,
                    target: Some(target.to_owned()),
                }));
            }
        }
    }
}

fn workspace_dependencies_exists(doc: &DocumentMut) -> bool {
    doc.get("workspace")
        .and_then(Item::as_table)
        .and_then(|table| table.get("dependencies"))
        .and_then(Item::as_table)
        .is_some()
}

fn kind_key(kind: DependencyKind) -> &'static str {
    match kind {
        DependencyKind::Normal => "dependencies",
        DependencyKind::Dev => "dev-dependencies",
        DependencyKind::Build => "build-dependencies",
    }
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

fn required_failures(states: &[FileState], package: &str, files: &[String]) -> Vec<String> {
    if states.is_empty() {
        return vec![format!("no files matched {}", files.join(", "))];
    }
    states
        .iter()
        .filter_map(|state| {
            if let Some(message) = &state.parse_error {
                return Some(format!("{} invalid TOML: {message}", state.path));
            }
            if file_has_package(&state.bytes, &state.tables, package) {
                None
            } else {
                Some(format!("missing in {}", state.path))
            }
        })
        .collect()
}

fn exists_status(
    states: &[FileState],
    package: &str,
    files: &[String],
) -> (Status, Option<String>) {
    if states.is_empty() {
        return (
            Status::Fail,
            Some(format!("no files matched {}", files.join(", "))),
        );
    }
    let mut invalid = Vec::new();
    for state in states {
        if let Some(message) = &state.parse_error {
            invalid.push(format!("{} invalid TOML: {message}", state.path));
            continue;
        }
        if file_has_package(&state.bytes, &state.tables, package) {
            return (Status::Pass, None);
        }
    }
    if invalid.is_empty() {
        (
            Status::Fail,
            Some(format!("found in no matched Cargo file: {package}")),
        )
    } else {
        (Status::Fail, Some(invalid.join("; ")))
    }
}

fn exact_forbidden_failures(
    states: &[FileState],
    package: &str,
) -> Result<Vec<String>, VerifyError> {
    let mut failures = Vec::new();
    for state in states {
        if let Some(message) = &state.parse_error {
            failures.push(format!("{} invalid TOML: {message}", state.path));
            continue;
        }
        for hit in file_exact_forbidden_hits(&state.bytes, &state.tables, package)? {
            failures.push(format!("{} {hit}", state.path));
        }
    }
    Ok(failures)
}

fn glob_forbidden_failures(states: &[FileState], glob: &str) -> Result<Vec<String>, VerifyError> {
    let mut failures = Vec::new();
    for state in states {
        if let Some(message) = &state.parse_error {
            failures.push(format!("{} invalid TOML: {message}", state.path));
            continue;
        }
        for hit in file_glob_forbidden_hits(&state.bytes, &state.tables, glob)? {
            failures.push(format!("{} {hit}", state.path));
        }
    }
    Ok(failures)
}

fn file_has_package(bytes: &[u8], tables: &[DependencyTableTarget], package: &str) -> bool {
    tables.iter().any(|table| {
        let findings = run_cargo_engine(bytes, exact_requirement_for_table(table, package, true));
        findings.is_empty()
    })
}

fn file_exact_forbidden_hits(
    bytes: &[u8],
    tables: &[DependencyTableTarget],
    package: &str,
) -> Result<Vec<String>, VerifyError> {
    let mut hits = Vec::new();
    for table in tables {
        let findings = run_cargo_engine(bytes, exact_requirement_for_table(table, package, false));
        hits.extend(
            findings
                .into_iter()
                .map(|finding| finding_summary(table, finding)),
        );
    }
    Ok(hits)
}

fn file_glob_forbidden_hits(
    bytes: &[u8],
    tables: &[DependencyTableTarget],
    glob: &str,
) -> Result<Vec<String>, VerifyError> {
    let mut hits = Vec::new();
    for table in tables {
        let findings = run_cargo_engine(bytes, glob_requirement_for_table(table, glob));
        hits.extend(
            findings
                .into_iter()
                .map(|finding| finding_summary(table, finding)),
        );
    }
    Ok(hits)
}

fn exact_requirement_for_table(
    table: &DependencyTableTarget,
    package: &str,
    required: bool,
) -> CargoTomlRequirements {
    let items = exact_items(package, required);
    let mut req = CargoTomlRequirements::default();
    match table {
        DependencyTableTarget::Scope(scope) => {
            let _ = req.dependencies.insert(scope.clone(), items);
        }
        DependencyTableTarget::Workspace => {
            req.workspace_dependencies = Some(items);
        }
    }
    req
}

fn glob_requirement_for_table(table: &DependencyTableTarget, glob: &str) -> CargoTomlRequirements {
    let globs = glob_items(glob);
    let mut req = CargoTomlRequirements::default();
    match table {
        DependencyTableTarget::Scope(scope) => {
            let _ = req
                .forbidden_dependency_package_globs
                .insert(scope.clone(), globs);
        }
        DependencyTableTarget::Workspace => {
            req.forbidden_workspace_dependency_package_globs = Some(globs);
        }
    }
    req
}

fn exact_items(package: &str, required: bool) -> ItemRequirements<CargoDependencyRequirement> {
    let item = exact_requirement(package);
    if required {
        ItemRequirements {
            required: vec![(item, format!("{package} must be declared"))],
            ..ItemRequirements::default()
        }
    } else {
        ItemRequirements {
            banned: vec![(item, format!("{package} must not be declared"))],
            ..ItemRequirements::default()
        }
    }
}

fn exact_requirement(package: &str) -> CargoDependencyRequirement {
    CargoDependencyRequirement {
        file_key: None,
        value: DependencySpec {
            package: Some(package.to_owned()),
            ..DependencySpec::default()
        },
    }
}

fn glob_items(glob: &str) -> ForbiddenGlobRequirements<DependencyPackageGlob> {
    ForbiddenGlobRequirements {
        globs: vec![(
            DependencyPackageGlob {
                glob: glob.to_owned(),
            },
            format!("{glob} package family must not be declared"),
        )],
    }
}

fn run_cargo_engine(bytes: &[u8], requirement: CargoTomlRequirements) -> Vec<Finding> {
    let engine = CargoTomlEngine;
    let reqs: Vec<(Provenance, Box<dyn EngineRequirement>)> = vec![(
        Provenance {
            policy: "specular".to_owned(),
        },
        Box::new(requirement),
    )];
    engine.reconcile(Some(bytes), &reqs).findings
}

fn finding_summary(table: &DependencyTableTarget, finding: Finding) -> String {
    let table = table_label(table);
    match finding {
        Finding::Mismatch { key, current, .. } => current.map_or_else(
            || format!("{table}: {key}"),
            |current| format!("{table}: {key} = {current}"),
        ),
        Finding::InvalidRequirements { key, message, .. } => {
            format!("{table}: {key}: {message}")
        }
        Finding::ParseError { message, .. } => format!("{table}: invalid TOML: {message}"),
        Finding::ConflictingRequirements { key, reason, .. } => {
            format!("{table}: {key}: {reason}")
        }
        Finding::UnwritableRequiredKey { key, expected, .. } => {
            format!("{table}: {key}: {expected}")
        }
        Finding::InternalError { message } => format!("{table}: {message}"),
    }
}

fn table_label(table: &DependencyTableTarget) -> String {
    match table {
        DependencyTableTarget::Scope(scope) => scope.table_path(),
        DependencyTableTarget::Workspace => "[workspace.dependencies]".to_owned(),
    }
}

fn status_for_failures(failures: &[String]) -> Status {
    if failures.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    }
}

fn fail_message(status: Status, message: String) -> Option<String> {
    if status == Status::Fail {
        Some(message)
    } else {
        None
    }
}

fn evidence(
    target: Option<serde_json::Value>,
    polarity: Polarity,
    item: &str,
    verifier: &str,
    status: Status,
    message: Option<String>,
) -> Evidence {
    Evidence {
        category: Category::Dependencies,
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
