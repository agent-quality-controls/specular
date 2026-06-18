#!/usr/bin/env python3
import json
import tomllib
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def emit(entry, status, message=None):
    out = {"check": entry.get("check", "cargo-dependencies-plan"), "status": status}
    if message:
        out["message"] = message
    print(json.dumps(out, sort_keys=True))


def read(rel_path):
    return (ROOT / rel_path).read_text()


def require_contains(failures, rel_path, needles):
    text = read(rel_path)
    for needle in needles:
        if needle not in text:
            failures.append(f"{rel_path}: missing {needle!r}")


def require_absent(failures, rel_path, needles):
    text = read(rel_path)
    for needle in needles:
        if needle in text:
            failures.append(f"{rel_path}: forbidden {needle!r}")


def require_contains_casefold(failures, rel_path, needle):
    if needle.casefold() not in read(rel_path).casefold():
        failures.append(f"{rel_path}: missing {needle!r}")


def check_model(_entry):
    failures = []
    require_contains(
        failures,
        "src/model.rs",
        [
            "pub files: Vec<String>",
            "pub forbidden_globs: Vec<String>",
            "Cargo package-name globs that must not be declared.",
        ],
    )
    require_absent(failures, "src/model.rs", ["pub manifests: Vec<String>"])
    return failures


def check_lint(_entry):
    failures = []
    require_contains(
        failures,
        "src/lint.rs",
        [
            '"builtin:cargo-dependencies"',
            "Category::Dependencies",
            "forbidden_globs",
            "contains_glob_meta",
            "check_dependency_items",
            "forbiddenGlobs",
        ],
    )
    require_absent(failures, "src/lint.rs", ["x.manifests", "block.manifests"])
    return failures


def check_verify(_entry):
    failures = []
    require_contains(
        failures,
        "src/verify.rs",
        [
            "check_cargo_dependencies",
            '"builtin:cargo-dependencies"',
            "x.files",
            "forbidden_globs",
        ],
    )
    require_contains(failures, "src/lib.rs", ["mod cargo_dependencies;"])
    require_absent(failures, "src/verify.rs", ["x.manifests"])
    require_contains(
        failures,
        "src/cargo_dependencies.rs",
        [
            "CargoTomlEngine",
            "CargoTomlRequirements",
            "DependencyPackageGlob",
            "ForbiddenGlobRequirements",
            "ItemRequirements",
            "workspace_dependencies",
            "forbidden_workspace_dependency_package_globs",
            "DependencyKind::Normal",
            "DependencyKind::Dev",
            "DependencyKind::Build",
        ],
    )
    return failures


def check_docs(_entry):
    failures = []
    for rel_path in ("HELP.txt", "README.md"):
        require_contains(
            failures,
            rel_path,
            [
                "builtin:cargo-dependencies",
                "files",
                "forbiddenGlobs",
            ],
        )
        require_absent(failures, rel_path, ['"manifests"'])
        require_contains_casefold(failures, rel_path, "renamed")
    return failures


def check_dependencies(_entry):
    failures = []
    cargo = tomllib.loads(read("Cargo.toml"))
    deps = cargo.get("dependencies", {})
    for name in ("aqc-cargo-toml-engine", "aqc-file-engine-core", "toml_edit"):
        if name not in deps:
            failures.append(f"Cargo.toml: missing dependency {name}")
    if "g3rs-cargo-adapter" in deps:
        failures.append("Cargo.toml: g3rs-cargo-adapter must not be a dependency")
    if cargo.get("package", {}).get("version") == "0.3.0":
        failures.append("Cargo.toml: package version was not bumped")
    require_contains(
        failures,
        "Cargo.lock",
        ['name = "aqc-cargo-toml-engine"', 'name = "aqc-file-engine-core"'],
    )
    return failures


def check_fixtures(_entry):
    failures = []
    fixture_specs = [
        "behavior/fixtures/lint/lint-R00-clean-golden/repo/spec.json",
        "behavior/fixtures/lint/lint-R20-semantic/repo/spec.json",
        "behavior/fixtures/lint/lint-R24-builtin-category-mismatch/repo/spec.json",
        "behavior/fixtures/lint/lint-R26-cargo-dependencies-clean/repo/spec.json",
        "behavior/fixtures/lint/lint-R27-cargo-dependencies-invalid/repo/spec.json",
        "behavior/fixtures/verify/verify-R00-clean-golden/repo/spec.json",
        "behavior/fixtures/verify/verify-R10-requirement-failures/repo/spec.json",
        "behavior/fixtures/verify/verify-R27-cargo-dependencies-clean/repo/spec.json",
        "behavior/fixtures/verify/verify-R28-cargo-dependencies-failures/repo/spec.json",
        "behavior/fixtures/verify/verify-R29-cargo-dependencies-invalid/repo/spec.json",
    ]
    for rel_path in fixture_specs:
        path = ROOT / rel_path
        if not path.is_file():
            failures.append(f"missing {rel_path}")
            continue
        spec = json.loads(path.read_text())
        if spec.get("version") != 3:
            failures.append(f"{rel_path}: version is not 3")
        text = path.read_text()
        if '"manifests"' in text:
            failures.append(f"{rel_path}: manifests remains")
    required_fixture_text = {
        "behavior/fixtures/verify/verify-R27-cargo-dependencies-clean/repo/spec.json": [
            "builtin:cargo-dependencies",
            "forbiddenGlobs",
        ],
        "behavior/fixtures/verify/verify-R27-cargo-dependencies-clean/repo/Cargo.toml": [
            'package = "renamed-package"',
            "[workspace.dependencies]",
        ],
        "behavior/fixtures/verify/verify-R28-cargo-dependencies-failures/repo/Cargo.toml": [
            'package = "renamed-package"',
            "[target.'cfg(unix)'.dependencies]",
        ],
        "behavior/fixtures/verify/verify-R29-cargo-dependencies-invalid/repo/Cargo.toml": [
            "not valid toml",
        ],
    }
    for rel_path, needles in required_fixture_text.items():
        if (ROOT / rel_path).is_file():
            require_contains(failures, rel_path, needles)
    return failures


def check_golden(_entry):
    failures = []
    for rel_path in (
        "behavior/golden/lint/approved.normalized.json",
        "behavior/golden/verify/approved.normalized.json",
    ):
        require_contains(failures, rel_path, ["cargo-dependencies"])
        require_absent(failures, rel_path, ['"manifests"'])
    return failures


def check_dogfood(_entry):
    failures = []
    spec = json.loads(read(".plans/2026-06-17-134859-builtin-cargo-dependencies.md.spec.json"))
    if spec.get("version") != 3:
        failures.append("dogfood spec version is not 3")
    coverage = read(".plans/2026-06-17-134859-builtin-cargo-dependencies.md.spec.coverage.md")
    required_headings = [
        "Goal",
        "Current Dependency State",
        "Semantics",
        "Approach",
        "Key Decisions",
        "Files To Modify",
    ]
    for heading in required_headings:
        if f"## {heading}" not in coverage:
            failures.append(f"coverage missing {heading}")
    if "UNCOVERED" in coverage:
        failures.append("coverage contains UNCOVERED")
    return failures


CHECKS = {
    "model": check_model,
    "lint": check_lint,
    "verify": check_verify,
    "docs": check_docs,
    "dependencies": check_dependencies,
    "fixtures": check_fixtures,
    "golden": check_golden,
    "dogfood": check_dogfood,
}


def main():
    if len(sys.argv) != 4 or sys.argv[2] != "custom":
        raise SystemExit("usage: verify-cargo-dependencies-plan.py <spec.json> custom <blockIndex>")
    spec = json.loads(Path(sys.argv[1]).read_text())
    block_index = int(sys.argv[3])
    entry = spec["requirements"]["custom"][block_index]
    check = CHECKS.get(entry.get("check"))
    if check is None:
        emit(entry, "fail", f"unknown check {entry.get('check')!r}")
        return
    try:
        failures = check(entry)
    except Exception as error:
        failures = [str(error)]
    emit(entry, "pass" if not failures else "fail", "; ".join(failures) or None)


if __name__ == "__main__":
    main()
