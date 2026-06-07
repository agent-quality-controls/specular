//! `lint`: the only constructor of a valid [`Spec`].

use std::collections::{HashMap, HashSet};
use std::path::Path;

use camino::Utf8Path;
use garde::Validate as _;
use globset::GlobBuilder;

use crate::error::{LintError, SpecViolation};
use crate::model::Spec;

/// Validate the spec file and produce the typed contract.
///
/// # Errors
///
/// [`LintError::Read`] / [`LintError::Parse`] when the file cannot be read or
/// parsed at all; [`LintError::InvalidSpec`] with the full violation list when
/// the contract is violated.
pub fn lint(path: &Path) -> Result<Spec, LintError> {
    let text = std::fs::read_to_string(path).map_err(|e| LintError::Read {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| LintError::Parse {
        message: e.to_string(),
    })?;

    let mut violations = Vec::new();
    schema_violations(&value, &mut violations);
    if !violations.is_empty() {
        return Err(LintError::InvalidSpec(violations));
    }

    let spec: Spec = serde_json::from_value(value).map_err(|e| LintError::Parse {
        message: e.to_string(),
    })?;

    if let Err(report) = spec.validate() {
        for (field, error) in report.iter() {
            violations.push(SpecViolation {
                code: "FIELD_RULE".to_owned(),
                message: format!("{field}: {error}"),
            });
        }
    }
    semantic_violations(&spec, &mut violations);

    if violations.is_empty() {
        Ok(spec)
    } else {
        Err(LintError::InvalidSpec(violations))
    }
}

fn schema_violations(value: &serde_json::Value, out: &mut Vec<SpecViolation>) {
    let schema = schemars::schema_for!(Spec);
    let Ok(schema_value) = serde_json::to_value(&schema) else {
        out.push(SpecViolation {
            code: "INTERNAL".to_owned(),
            message: "generated schema does not serialize".to_owned(),
        });
        return;
    };
    match jsonschema::validator_for(&schema_value) {
        Ok(validator) => {
            for error in validator.iter_errors(value) {
                out.push(SpecViolation {
                    code: "JSON_SCHEMA".to_owned(),
                    message: format!("{} (at {})", error, error.instance_path()),
                });
            }
        }
        Err(error) => out.push(SpecViolation {
            code: "INTERNAL".to_owned(),
            message: format!("generated schema does not compile: {error}"),
        }),
    }
}

fn semantic_violations(spec: &Spec, out: &mut Vec<SpecViolation>) {
    check_ids(spec, out);
    check_paths_and_globs(spec, out);
    check_mergeable(spec, out);
    check_vacuous(spec, out);
    check_ownership(spec, out);
}

/// Every `(id, category)` pair in the spec, in document order.
fn all_ids(spec: &Spec) -> Vec<(&str, &'static str)> {
    let r = &spec.requirements;
    let mut ids: Vec<(&str, &'static str)> = Vec::new();
    ids.extend(r.tree.iter().map(|x| (x.id.as_str(), "tree")));
    ids.extend(r.content.iter().map(|x| (x.id.as_str(), "content")));
    ids.extend(
        r.dependencies
            .iter()
            .map(|x| (x.id.as_str(), "dependencies")),
    );
    ids.extend(r.exports.iter().map(|x| (x.id.as_str(), "exports")));
    ids.extend(
        r.enumerations
            .iter()
            .map(|x| (x.id.as_str(), "enumerations")),
    );
    ids.extend(r.schemas.iter().map(|x| (x.id.as_str(), "schemas")));
    ids
}

fn check_ids(spec: &Spec, out: &mut Vec<SpecViolation>) {
    let mut seen: HashSet<&str> = HashSet::new();
    for (id, _) in all_ids(spec) {
        if !seen.insert(id) {
            out.push(SpecViolation {
                code: "DUPLICATE_ID".to_owned(),
                message: format!("requirement id '{id}' appears more than once"),
            });
        }
        let mut chars = id.chars();
        let head_ok = chars.next().is_some_and(|c| c.is_ascii_uppercase());
        let tail_ok = chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if !(head_ok && tail_ok) {
            out.push(SpecViolation {
                code: "ID_FORMAT".to_owned(),
                message: format!("requirement id '{id}' is not SCREAMING_SNAKE_CASE"),
            });
        }
    }
}

fn check_path_text(id: &str, value: &str, out: &mut Vec<SpecViolation>) {
    let path = Utf8Path::new(value);
    let escapes = path.components().any(|c| {
        matches!(
            c,
            camino::Utf8Component::ParentDir
                | camino::Utf8Component::RootDir
                | camino::Utf8Component::Prefix(_)
        )
    });
    if value.is_empty() || value.starts_with('/') || value.contains("//") || escapes {
        out.push(SpecViolation {
            code: "PATH_RULE".to_owned(),
            message: format!(
                "{id}: '{value}' must be repo-root-relative with '/', no '..', no empty components"
            ),
        });
    }
}

fn check_glob(id: &str, pattern: &str, out: &mut Vec<SpecViolation>) {
    check_path_text(id, pattern, out);
    if let Err(error) = GlobBuilder::new(pattern).literal_separator(true).build() {
        out.push(SpecViolation {
            code: "GLOB".to_owned(),
            message: format!("{id}: glob '{pattern}' does not compile: {error}"),
        });
    }
}

fn check_paths_and_globs(spec: &Spec, out: &mut Vec<SpecViolation>) {
    let r = &spec.requirements;
    for x in &r.tree {
        for p in &x.required_paths {
            check_path_text(&x.id, p, out);
        }
        for g in &x.forbidden_globs {
            check_glob(&x.id, g, out);
        }
    }
    for x in &r.content {
        for g in &x.files {
            check_glob(&x.id, g, out);
        }
    }
    for x in &r.dependencies {
        for g in &x.manifest_globs {
            check_glob(&x.id, g, out);
        }
    }
    for x in &r.schemas {
        check_path_text(&x.id, &x.file, out);
    }
}

/// Granularity is derived: identical scope and polarity means one rule.
fn check_mergeable(spec: &Spec, out: &mut Vec<SpecViolation>) {
    let r = &spec.requirements;
    let mut groups: HashMap<String, Vec<&str>> = HashMap::new();
    for x in &r.tree {
        if !x.required_paths.is_empty() {
            groups
                .entry("tree/required".to_owned())
                .or_default()
                .push(&x.id);
        }
        if !x.forbidden_globs.is_empty() {
            groups
                .entry("tree/forbidden".to_owned())
                .or_default()
                .push(&x.id);
        }
    }
    for x in &r.content {
        let scope = sorted_key(&x.files);
        if !x.forbidden_substrings.is_empty() {
            groups
                .entry(format!("content/forbidden/{scope}"))
                .or_default()
                .push(&x.id);
        }
        if !x.required_substrings.is_empty() {
            groups
                .entry(format!("content/required/{scope}"))
                .or_default()
                .push(&x.id);
        }
    }
    for x in &r.dependencies {
        let scope = sorted_key(&x.manifest_globs);
        if !x.required_crates.is_empty() {
            groups
                .entry(format!("dependencies/required/{scope}"))
                .or_default()
                .push(&x.id);
        }
        if !(x.forbidden_crates.is_empty() && x.forbidden_crate_prefixes.is_empty()) {
            groups
                .entry(format!("dependencies/forbidden/{scope}"))
                .or_default()
                .push(&x.id);
        }
    }
    for x in &r.exports {
        groups
            .entry(format!("exports/{}", x.package))
            .or_default()
            .push(&x.id);
    }
    for x in &r.enumerations {
        groups
            .entry(format!("enumerations/{}", x.type_name))
            .or_default()
            .push(&x.id);
    }
    for x in &r.schemas {
        groups
            .entry(format!("schemas/{}", x.file))
            .or_default()
            .push(&x.id);
    }
    let mut offending: Vec<String> = groups
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|(_, ids)| ids.join("+"))
        .collect();
    offending.sort();
    for ids in offending {
        out.push(SpecViolation {
            code: "MERGEABLE_REQUIREMENTS".to_owned(),
            message: format!("same scope and polarity; merge into one row: {ids}"),
        });
    }
}

fn sorted_key(values: &[String]) -> String {
    let mut sorted: Vec<&str> = values.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    sorted.join(",")
}

/// A spec of pure prohibitions passes on an empty repository.
fn check_vacuous(spec: &Spec, out: &mut Vec<SpecViolation>) {
    let r = &spec.requirements;
    let positive = r.tree.iter().any(|x| !x.required_paths.is_empty())
        || r.content.iter().any(|x| !x.required_substrings.is_empty())
        || r.dependencies.iter().any(|x| !x.required_crates.is_empty())
        || !r.exports.is_empty()
        || !r.enumerations.is_empty()
        || !r.schemas.is_empty();
    if !positive {
        out.push(SpecViolation {
            code: "VACUOUS_SPEC".to_owned(),
            message: "no positive assertion; this spec passes on an empty repository".to_owned(),
        });
    }
}

/// Every requirement has exactly one owner; the escape hatch must not erode
/// the builtin core.
fn check_ownership(spec: &Spec, out: &mut Vec<SpecViolation>) {
    let r = &spec.requirements;
    let builtin_ids: HashSet<&str> = r
        .tree
        .iter()
        .map(|x| x.id.as_str())
        .chain(r.content.iter().map(|x| x.id.as_str()))
        .collect();
    let known_ids: HashSet<&str> = all_ids(spec).into_iter().map(|(id, _)| id).collect();

    let mut claimed: HashMap<&str, Vec<&str>> = HashMap::new();
    for verifier in &spec.verifiers {
        for claim in &verifier.requirement_ids {
            if !known_ids.contains(claim.as_str()) {
                out.push(SpecViolation {
                    code: "UNKNOWN_CLAIM".to_owned(),
                    message: format!(
                        "verifier '{}' claims unknown requirement '{claim}'",
                        verifier.id
                    ),
                });
                continue;
            }
            if builtin_ids.contains(claim.as_str()) {
                out.push(SpecViolation {
                    code: "BUILTIN_COVERED_CLAIM".to_owned(),
                    message: format!(
                        "verifier '{}' claims '{claim}', which a builtin verifier covers",
                        verifier.id
                    ),
                });
                continue;
            }
            claimed
                .entry(claim.as_str())
                .or_default()
                .push(&verifier.id);
        }
        if !Path::new(&verifier.command[0]).exists() {
            out.push(SpecViolation {
                code: "VERIFIER_COMMAND_MISSING".to_owned(),
                message: format!(
                    "verifier '{}' command file '{}' does not exist",
                    verifier.id, verifier.command[0]
                ),
            });
        }
    }
    for (claim, owners) in &claimed {
        if owners.len() > 1 {
            out.push(SpecViolation {
                code: "OVERLAPPING_CLAIM".to_owned(),
                message: format!("requirement '{claim}' claimed by: {}", owners.join(", ")),
            });
        }
    }
    for (id, category) in all_ids(spec) {
        let is_builtin = category == "tree" || category == "content";
        if !is_builtin && !claimed.contains_key(id) {
            out.push(SpecViolation {
                code: "UNCLAIMED_REQUIREMENT".to_owned(),
                message: format!(
                    "'{id}' ({category}) has no builtin verifier in this version and no custom claim"
                ),
            });
        }
    }
}
