#!/usr/bin/env python3
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]

VALID_FIXTURE_SPECS = [
    "behavior/fixtures/lint/lint-R00-clean-golden/repo/spec.json",
    "behavior/fixtures/verify/verify-R00-clean-golden/repo/spec.json",
    "behavior/fixtures/verify/verify-R10-requirement-failures/repo/spec.json",
    "behavior/fixtures/verify/verify-R20-verifier-nonzero/repo/spec.json",
    "behavior/fixtures/verify/verify-R21-protocol-violation/repo/spec.json",
    "behavior/fixtures/verify/verify-R22-coverage-miss/repo/spec.json",
    "behavior/fixtures/verify/verify-R24-verifier-missing/repo/spec.json",
    "behavior/fixtures/verify/verify-R25-custom-silent/repo/spec.json",
    "behavior/fixtures/verify/verify-R26-custom-broken/repo/spec.json",
]

NEW_LINT_FIXTURES = [
    "behavior/fixtures/lint/lint-R22-verifier-missing/repo/spec.json",
    "behavior/fixtures/lint/lint-R23-unknown-builtin/repo/spec.json",
    "behavior/fixtures/lint/lint-R24-builtin-category-mismatch/repo/spec.json",
    "behavior/fixtures/lint/lint-R25-top-level-verifiers/repo/spec.json",
]


def emit(entry, status, message=None):
    line = {
        "check": entry.get("check", "block-level-verifiers"),
        "status": status,
    }
    if message:
        line["message"] = message
    print(json.dumps(line, sort_keys=True))


def read_json(rel_path):
    return json.loads((ROOT / rel_path).read_text())


def requirement_blocks(spec):
    requirements = spec.get("requirements", {})
    blocks = []
    tree = requirements.get("tree")
    if isinstance(tree, dict) and any(tree.get(key) for key in ("required", "exists", "forbidden")):
        blocks.append(("tree", tree))
    for category in ("content", "dependencies", "exports", "enumerations", "custom"):
        entries = requirements.get(category, [])
        if isinstance(entries, list):
            for entry in entries:
                if isinstance(entry, dict):
                    blocks.append((category, entry))
    return blocks


def verifier_ok(value):
    return (
        isinstance(value, list)
        and bool(value)
        and all(isinstance(item, str) and item for item in value)
    )


def check_valid_fixture_specs_v2(_entry):
    failures = []
    for rel_path in VALID_FIXTURE_SPECS:
        spec = read_json(rel_path)
        if spec.get("version") != 3:
            failures.append(f"{rel_path}: version is {spec.get('version')!r}, expected 3")
        if "verifiers" in spec:
            failures.append(f"{rel_path}: top-level verifiers remains")
        for category, block in requirement_blocks(spec):
            if not verifier_ok(block.get("verifier")):
                failures.append(f"{rel_path}: {category} block has no valid verifier argv")
    return failures


def check_new_lint_fixtures(_entry):
    return [
        f"missing {rel_path}"
        for rel_path in NEW_LINT_FIXTURES
        if not (ROOT / rel_path).is_file()
    ]


def evidence_objects(value):
    if isinstance(value, dict):
        if isinstance(value.get("evidence"), list):
            yield from value["evidence"]
        for child in value.values():
            yield from evidence_objects(child)
    elif isinstance(value, list):
        for child in value:
            yield from evidence_objects(child)


def check_golden_reports(_entry):
    failures = []
    for path in sorted((ROOT / "behavior/golden").glob("**/*.json")):
        text = path.read_text()
        if '"source": "custom"' in text:
            failures.append(f"{path.relative_to(ROOT)}: source custom remains")
        data = json.loads(text)
        for index, evidence in enumerate(evidence_objects(data)):
            if "verifier" not in evidence:
                failures.append(f"{path.relative_to(ROOT)}: evidence[{index}] has no verifier")
    return failures


def check_protocol_docs(_entry):
    failures = []
    for rel_path in ("HELP.txt", "README.md"):
        text = (ROOT / rel_path).read_text()
        required = [
            '"version": 3',
            '"verifier"',
            "one command",
            "argv",
            "builtin:tree",
            "builtin:content",
        ]
        missing = [item for item in required if item not in text]
        if missing:
            failures.append(f"{rel_path}: missing {missing}")
        forbidden = ['"verifiers": {', "builtin when no verifier is declared", "builtin:text"]
        present = [item for item in forbidden if item in text]
        if present:
            failures.append(f"{rel_path}: forbidden {present}")
    return failures


def check_repo_plan_specs_v2(_entry):
    failures = []
    for path in sorted((ROOT / ".plans").glob("*.spec.json")):
        rel_path = path.relative_to(ROOT)
        spec = json.loads(path.read_text())
        if spec.get("version") != 3:
            failures.append(f"{rel_path}: version is {spec.get('version')!r}, expected 3")
        if "verifiers" in spec:
            failures.append(f"{rel_path}: top-level verifiers remains")
    return failures


CHECKS = {
    "valid-fixture-specs-v2": check_valid_fixture_specs_v2,
    "new-lint-fixtures": check_new_lint_fixtures,
    "golden-reports": check_golden_reports,
    "protocol-docs": check_protocol_docs,
    "repo-plan-specs-v2": check_repo_plan_specs_v2,
}


def main():
    if len(sys.argv) != 4 or sys.argv[2] != "custom":
        raise SystemExit("usage: verify-block-level-verifiers.py <spec.json> custom <blockIndex>")

    spec = read_json(sys.argv[1])
    block_index = int(sys.argv[3])
    entry = spec["requirements"].get("custom", [])[block_index]
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
