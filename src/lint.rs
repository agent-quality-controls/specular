//! `lint`: the only constructor of a valid [`Spec`].

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use camino::Utf8Path;
use globset::GlobBuilder;

use crate::error::{LintError, SpecViolation};
use crate::model::{Category, ExportRequirement, Spec};

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
    check_version(spec, out);
    check_paths_and_globs(spec, out);
    check_targets(spec, out);
    check_items(spec, out);
    check_vacuous(spec, out);
    check_verifiers(spec, out);
    check_custom_shape(spec, out);
}

fn check_version(spec: &Spec, out: &mut Vec<SpecViolation>) {
    if spec.version != 1 {
        out.push(SpecViolation {
            code: "JSON_SCHEMA".to_owned(),
            message: format!("version must be 1, got {}", spec.version),
        });
    }
}

fn check_path_text(label: &str, value: &str, out: &mut Vec<SpecViolation>) {
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
                "{label}: '{value}' must be repo-root-relative with '/', no '..', no empty components"
            ),
        });
    }
}

fn check_glob(label: &str, pattern: &str, out: &mut Vec<SpecViolation>) {
    check_path_text(label, pattern, out);
    if let Err(error) = GlobBuilder::new(pattern).literal_separator(true).build() {
        out.push(SpecViolation {
            code: "GLOB".to_owned(),
            message: format!("{label}: glob '{pattern}' does not compile: {error}"),
        });
    }
}

fn check_paths_and_globs(spec: &Spec, out: &mut Vec<SpecViolation>) {
    let tree = &spec.requirements.tree;
    for p in tree.required.iter().chain(&tree.exists) {
        check_path_text("tree", p, out);
    }
    for g in &tree.forbidden {
        check_glob("tree", g, out);
    }
    for block in &spec.requirements.content {
        for g in &block.files {
            check_glob("content", g, out);
        }
    }
    for block in &spec.requirements.dependencies {
        for g in &block.manifests {
            check_glob("dependencies", g, out);
        }
    }
}

fn check_targets(spec: &Spec, out: &mut Vec<SpecViolation>) {
    duplicate_targets(
        "content",
        spec.requirements
            .content
            .iter()
            .map(|x| sorted_key(&x.files)),
        out,
    );
    duplicate_targets(
        "dependencies",
        spec.requirements
            .dependencies
            .iter()
            .map(|x| sorted_key(&x.manifests)),
        out,
    );
    duplicate_targets(
        "exports",
        spec.requirements.exports.iter().map(|x| x.package.clone()),
        out,
    );
    duplicate_targets(
        "enumerations",
        spec.requirements
            .enumerations
            .iter()
            .map(|x| x.name.clone()),
        out,
    );
}

fn duplicate_targets(
    category: &str,
    targets: impl Iterator<Item = String>,
    out: &mut Vec<SpecViolation>,
) {
    let mut seen = HashSet::new();
    for target in targets {
        if !seen.insert(target.clone()) {
            out.push(SpecViolation {
                code: "DUPLICATE_TARGET".to_owned(),
                message: format!("{category}: target '{target}' appears more than once"),
            });
        }
    }
}

fn sorted_key(values: &[String]) -> String {
    let mut sorted: Vec<&str> = values.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    sorted.join(",")
}

fn check_items(spec: &Spec, out: &mut Vec<SpecViolation>) {
    let tree = &spec.requirements.tree;
    check_item_lists(
        "tree",
        "",
        &tree.required,
        &tree.exists,
        &tree.forbidden,
        out,
    );
    if !tree.exists.is_empty() {
        out.push(SpecViolation {
            code: "EXISTS_SINGLE_PLACE".to_owned(),
            message: "tree: exists is not allowed because tree has one place".to_owned(),
        });
    }
    for block in &spec.requirements.content {
        let target = sorted_key(&block.files);
        check_item_lists(
            "content",
            &target,
            &block.required,
            &block.exists,
            &block.forbidden,
            out,
        );
    }
    for block in &spec.requirements.dependencies {
        let target = sorted_key(&block.manifests);
        check_item_lists(
            "dependencies",
            &target,
            &block.required,
            &block.exists,
            &block.forbidden,
            out,
        );
    }
    for block in &spec.requirements.exports {
        check_export_items(block, out);
    }
    for block in &spec.requirements.enumerations {
        let mut seen = HashSet::new();
        for value in &block.values {
            check_plain_item("enumerations", &block.name, value, false, out);
            if !seen.insert(value) {
                out.push(SpecViolation {
                    code: "DUPLICATE_ITEM".to_owned(),
                    message: format!(
                        "enumerations/{}: item '{value}' appears more than once",
                        block.name
                    ),
                });
            }
        }
    }
}

fn check_export_items(block: &ExportRequirement, out: &mut Vec<SpecViolation>) {
    check_item_lists(
        "exports",
        &block.package,
        &block.required,
        &block.exists,
        &block.forbidden,
        out,
    );
    if !block.exists.is_empty() {
        out.push(SpecViolation {
            code: "EXISTS_SINGLE_PLACE".to_owned(),
            message: format!(
                "exports/{}: exists is not allowed because exports has one place",
                block.package
            ),
        });
    }
}

fn check_item_lists(
    category: &str,
    target: &str,
    required: &[String],
    exists: &[String],
    forbidden: &[String],
    out: &mut Vec<SpecViolation>,
) {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for item in required.iter().chain(exists).chain(forbidden) {
        *counts.entry(item.as_str()).or_insert(0) += 1;
    }
    let label = target_label(category, target);
    for (item, count) in counts {
        if count > 1 {
            out.push(SpecViolation {
                code: "DUPLICATE_ITEM".to_owned(),
                message: format!("{label}: item '{item}' appears more than once"),
            });
        }
    }
    for item in required {
        check_plain_item(category, target, item, category == "tree", out);
    }
    for item in exists {
        check_plain_item(category, target, item, category == "tree", out);
    }
    let required_set: HashSet<&str> = required.iter().map(String::as_str).collect();
    let exists_set: HashSet<&str> = exists.iter().map(String::as_str).collect();
    let forbidden_set: HashSet<&str> = forbidden.iter().map(String::as_str).collect();
    for item in required_set.intersection(&forbidden_set) {
        out.push(SpecViolation {
            code: "CONTRADICTION".to_owned(),
            message: format!("{label}: item '{item}' is both required and forbidden"),
        });
    }
    for item in required_set.intersection(&exists_set) {
        out.push(SpecViolation {
            code: "REDUNDANT".to_owned(),
            message: format!("{label}: item '{item}' is both required and exists"),
        });
    }
}

fn check_plain_item(
    category: &str,
    target: &str,
    item: &str,
    also_check_path: bool,
    out: &mut Vec<SpecViolation>,
) {
    let bad_glob = item.contains('*') || item.contains('?') || item.contains('[');
    if item.is_empty() || item.trim() != item || bad_glob {
        out.push(SpecViolation {
            code: "ITEM_FORMAT".to_owned(),
            message: format!(
                "{}: required/exists item '{item}' must be non-empty, trimmed, and non-glob",
                target_label(category, target)
            ),
        });
    }
    if also_check_path {
        check_path_text(category, item, out);
    }
}

fn target_label(category: &str, target: &str) -> String {
    if target.is_empty() {
        category.to_owned()
    } else {
        format!("{category}/{target}")
    }
}

fn check_vacuous(spec: &Spec, out: &mut Vec<SpecViolation>) {
    let r = &spec.requirements;
    let positive = !r.tree.required.is_empty()
        || !r.tree.exists.is_empty()
        || r.content
            .iter()
            .any(|x| !x.required.is_empty() || !x.exists.is_empty())
        || r.dependencies
            .iter()
            .any(|x| !x.required.is_empty() || !x.exists.is_empty())
        || r.exports
            .iter()
            .any(|x| !x.required.is_empty() || !x.exists.is_empty())
        || r.enumerations.iter().any(|x| !x.values.is_empty());
    if !positive {
        out.push(SpecViolation {
            code: "VACUOUS_SPEC".to_owned(),
            message: "no positive assertion; this spec passes on an empty repository".to_owned(),
        });
    }
}

fn check_verifiers(spec: &Spec, out: &mut Vec<SpecViolation>) {
    for (key, command) in &spec.verifiers {
        let Some(category) = Category::parse(key) else {
            out.push(SpecViolation {
                code: "UNKNOWN_CATEGORY".to_owned(),
                message: format!("verifiers map names '{key}', which is not a category"),
            });
            continue;
        };
        if command.is_empty() {
            out.push(SpecViolation {
                code: "VERIFIER_COMMAND_EMPTY".to_owned(),
                message: format!("verifier for '{key}' has an empty command array"),
            });
        }
        if category_is_empty(spec, category) {
            out.push(SpecViolation {
                code: "DEAD_VERIFIER".to_owned(),
                message: format!("verifier for '{key}' is declared but the category is empty"),
            });
        }
    }
    for category in Category::ALL {
        if category_is_empty(spec, category) {
            continue;
        }
        let overridden = spec.verifiers.contains_key(category.as_str());
        if !category.has_builtin() && !overridden {
            out.push(SpecViolation {
                code: "CATEGORY_HAS_NO_VERIFIER".to_owned(),
                message: format!(
                    "category '{}' has requirements but no builtin verifier and no verifier entry",
                    category.as_str()
                ),
            });
        }
    }
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

fn check_custom_shape(spec: &Spec, out: &mut Vec<SpecViolation>) {
    for (index, entry) in spec.requirements.custom.iter().enumerate() {
        if !entry.is_object() {
            out.push(SpecViolation {
                code: "CUSTOM_SHAPE".to_owned(),
                message: format!("custom[{index}] must be an object"),
            });
        }
    }
}
