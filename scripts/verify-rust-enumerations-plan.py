#!/usr/bin/env python3
import json
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
AQC = ROOT.parent / "aqc-shared"
AQC_CRATE = AQC / "packages/source/rust/aqc-rust-syntax"


def emit(entry, status, message=None):
    out = {"check": entry.get("check", "rust-enumerations-plan"), "status": status}
    if message:
        out["message"] = message
    print(json.dumps(out, sort_keys=True))


def read(root, rel_path):
    return (root / rel_path).read_text()


def read_if_exists(path):
    if path.is_file():
        return path.read_text()
    return ""


def require_file(failures, path):
    if not path.is_file():
        failures.append(f"missing {path}")


def require_contains(failures, root, rel_path, needles):
    path = root / rel_path
    if not path.is_file():
        failures.append(f"missing {rel_path}")
        return
    text = path.read_text()
    for needle in needles:
        if needle not in text:
            failures.append(f"{rel_path}: missing {needle!r}")


def require_absent(failures, root, rel_path, needles):
    path = root / rel_path
    if not path.is_file():
        return
    text = path.read_text()
    for needle in needles:
        if needle in text:
            failures.append(f"{rel_path}: forbidden {needle!r}")


def check_aqc_crate(_entry):
    failures = []
    require_file(failures, AQC_CRATE / "Cargo.toml")
    require_file(failures, AQC_CRATE / "src/lib.rs")
    require_file(failures, AQC_CRATE / "tests/enums.rs")
    require_contains(
        failures,
        AQC,
        "Cargo.toml",
        ['"packages/source/rust/aqc-rust-syntax"'],
    )
    require_contains(
        failures,
        AQC_CRATE,
        "Cargo.toml",
        [
            'name = "aqc-rust-syntax"',
            'syn = { version = "2"',
            'proc-macro2 = { version = "1"',
            "span-locations",
        ],
    )
    require_contains(
        failures,
        AQC_CRATE,
        "src/lib.rs",
        [
            "pub fn parse_rust_syntax",
            "pub struct RustFileSyntax",
            "pub struct RustEnumDecl",
            "pub enum RustVisibility",
            "pub struct RustSyntaxError",
            "module_path",
            "syn::visit::Visit",
        ],
    )
    require_absent(
        failures,
        AQC_CRATE,
        "src/lib.rs",
        [
            "std::fs",
            "Cargo",
            "Guardrail",
            "g3",
            "Specular",
            "regex",
        ],
    )
    return failures


def check_aqc_tests(_entry):
    failures = []
    tests = read_if_exists(AQC_CRATE / "tests/enums.rs")
    for needle in [
        "unit",
        "tuple",
        "struct",
        "public",
        "crate",
        "restricted",
        "inline",
        "duplicate",
        "bom",
        "malformed",
        "attribute",
    ]:
        if needle not in tests.casefold():
            failures.append(f"aqc-rust-syntax tests missing {needle}")
    return failures


def check_model(_entry):
    failures = []
    require_contains(
        failures,
        ROOT,
        "src/model.rs",
        [
            "pub files: Vec<String>",
            "EnumerationRequirement",
            "Closed named value sets.",
        ],
    )
    return failures


def check_lint(_entry):
    failures = []
    require_contains(
        failures,
        ROOT,
        "src/lint.rs",
        [
            '"builtin:rust-enumerations"',
            "Category::Enumerations",
            "check_enumeration",
            "files",
            "ENUMERATION_FILES_REQUIRED",
        ],
    )
    return failures


def check_verify(_entry):
    failures = []
    require_contains(
        failures,
        ROOT,
        "src/lib.rs",
        ["mod rust_enumerations;"],
    )
    require_contains(
        failures,
        ROOT,
        "src/verify.rs",
        [
            "rust_enumerations::check_rust_enumerations",
            '"builtin:rust-enumerations"',
        ],
    )
    require_contains(
        failures,
        ROOT,
        "src/rust_enumerations.rs",
        [
            "parse_rust_syntax",
            "aqc_rust_syntax",
            "module_path",
            "ambiguous",
            "no files matched",
            "invalid Rust",
        ],
    )
    return failures


def check_docs(_entry):
    failures = []
    for rel_path in ("HELP.txt", "README.md"):
        require_contains(
            failures,
            ROOT,
            rel_path,
            [
                "builtin:rust-enumerations",
                "enumerations",
                "files",
                "Use built-in verifiers when they exist",
            ],
        )
    return failures


def check_fixtures(_entry):
    failures = []
    required = [
        "behavior/fixtures/lint/lint-R28-rust-enumerations-clean/repo/spec.json",
        "behavior/fixtures/lint/lint-R29-rust-enumerations-invalid/repo/spec.json",
        "behavior/fixtures/verify/verify-R30-rust-enumerations-clean/repo/spec.json",
        "behavior/fixtures/verify/verify-R30-rust-enumerations-clean/repo/src/model.rs",
        "behavior/fixtures/verify/verify-R31-rust-enumerations-failures/repo/spec.json",
        "behavior/fixtures/verify/verify-R31-rust-enumerations-failures/repo/src/model.rs",
        "behavior/fixtures/verify/verify-R32-rust-enumerations-invalid-rust/repo/spec.json",
        "behavior/fixtures/verify/verify-R32-rust-enumerations-invalid-rust/repo/src/model.rs",
    ]
    for rel_path in required:
        require_file(failures, ROOT / rel_path)
    for rel_path in required:
        if rel_path.endswith("spec.json"):
            needles = ["builtin:rust-enumerations", '"values"']
            if "lint-R29-rust-enumerations-invalid" not in rel_path:
                needles.append('"files"')
            require_contains(
                failures,
                ROOT,
                rel_path,
                needles,
            )
    fixture_text = "\n".join(
        path.read_text()
        for path in (ROOT / "behavior/fixtures").glob("**/*")
        if path.is_file()
    )
    for needle in ["wire::Status", "ambiguous", "no files matched", "not valid rust"]:
        if needle not in fixture_text:
            failures.append(f"fixtures missing {needle!r}")
    return failures


def check_version(_entry):
    failures = []
    cargo = tomllib.loads(read(ROOT, "Cargo.toml"))
    if cargo.get("package", {}).get("version") == "0.4.0":
        failures.append("Cargo.toml: package version was not bumped")
    require_contains(failures, ROOT, "src/lint.rs", ["version must be 4"])
    require_contains(failures, ROOT, "HELP.txt", ["Version 4"])
    return failures


def check_dogfood(_entry):
    failures = []
    spec = json.loads(read(ROOT, ".plans/2026-06-18-201059-rust-enumerations-builtin.md.spec.json"))
    if spec.get("version") != 4:
        failures.append("dogfood spec version must be 4")
    coverage = read(ROOT, ".plans/2026-06-18-201059-rust-enumerations-builtin.md.spec.coverage.md")
    required_headings = [
        "Goal",
        "End State",
        "Boundary Decision",
        "Why This Is Shared But Not Overbuilt",
        "New AQC Crate",
        "AQC Implementation Details",
        "Specular Format Change",
        "Explicit Non-Goals",
        "Key Decisions",
        "Files To Modify",
        "Work Order",
    ]
    for heading in required_headings:
        if f"## {heading}" not in coverage:
            failures.append(f"coverage missing {heading}")
    if "UNCOVERED" in coverage:
        failures.append("coverage contains UNCOVERED")
    plan = read(ROOT, ".plans/2026-06-18-201059-rust-enumerations-builtin.md")
    if "Guardrail3 Reuse Requirement" in plan or "Migrate one Guardrail3 enum consumer" in plan:
        failures.append("plan still contains g3 migration requirement")
    return failures


CHECKS = {
    "aqc-crate": check_aqc_crate,
    "aqc-tests": check_aqc_tests,
    "model": check_model,
    "lint": check_lint,
    "verify": check_verify,
    "docs": check_docs,
    "fixtures": check_fixtures,
    "version": check_version,
    "dogfood": check_dogfood,
}


def main():
    if len(sys.argv) != 4:
        raise SystemExit("usage: verifier <spec> <category> <index>")
    spec_path = Path(sys.argv[1])
    category = sys.argv[2]
    index = int(sys.argv[3])
    spec = json.loads(spec_path.read_text())
    entry = spec["requirements"][category][index]
    check = entry.get("check")
    failures = CHECKS[check](entry)
    emit(entry, "fail" if failures else "pass", "; ".join(failures) if failures else None)


if __name__ == "__main__":
    main()
