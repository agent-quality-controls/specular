#!/usr/bin/env python3
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PINNED_ACTION = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)?@[0-9a-f]{40}$")


def emit(entry, status, message=None):
    result = {"check": entry.get("check", "repo-quality"), "status": status}
    if message:
        result["message"] = message
    print(json.dumps(result, sort_keys=True))


def gh_json(args):
    completed = subprocess.run(
        ["gh", *args],
        cwd=ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return json.loads(completed.stdout)


def repo_view(repo):
    return gh_json(
        [
            "repo",
            "view",
            repo,
            "--json",
            "isPrivate,hasIssuesEnabled,hasDiscussionsEnabled,repositoryTopics",
        ]
    )


def security_state(repo):
    data = gh_json(["api", f"repos/{repo}", "--jq", ".security_and_analysis"])
    return data or {}


def check_repo_field(entry):
    data = repo_view(entry["repo"])
    actual = data.get(entry["field"])
    expected = entry["equals"]
    if actual == expected:
        emit(entry, "pass")
        return
    emit(entry, "fail", f"{entry['repo']} {entry['field']} is {actual!r}, expected {expected!r}")


def check_topics(entry):
    data = repo_view(entry["repo"])
    actual = {topic["name"] for topic in data.get("repositoryTopics", [])}
    expected = set(entry["topics"])
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if not missing and not extra:
        emit(entry, "pass")
        return
    emit(entry, "fail", f"missing topics={missing}; extra topics={extra}")


def check_security(entry):
    state = security_state(entry["repo"])
    failures = []
    for key, expected in entry["security"].items():
        actual = state.get(key, {}).get("status")
        if actual != expected:
            failures.append(f"{key}={actual!r}, expected {expected!r}")
    emit(entry, "pass" if not failures else "fail", "; ".join(failures) or None)


def cargo_toml(entry):
    return tomllib.loads((ROOT / entry["file"]).read_text())


def check_cargo_metadata(entry):
    package = cargo_toml(entry)["package"]
    required = {
        "description": "Deterministic spec-driven development CLI",
        "license": "MIT",
        "repository": "https://github.com/agent-quality-controls/specular",
        "readme": "README.md",
        "rust-version": "1.85",
    }
    failures = [
        f"{key}={package.get(key)!r}, expected {expected!r}"
        for key, expected in required.items()
        if package.get(key) != expected
    ]
    if package.get("keywords") != ["cli", "spec", "agents", "verification", "json"]:
        failures.append("keywords do not match the approved list")
    if package.get("categories") != ["command-line-utilities", "development-tools"]:
        failures.append("categories do not match the approved list")
    emit(entry, "pass" if not failures else "fail", "; ".join(failures) or None)


def check_no_local_path_deps(entry):
    text = (ROOT / entry["file"]).read_text()
    forbidden = ['path = "../aqc-shared', "path = '../aqc-shared"]
    hits = [item for item in forbidden if item in text]
    emit(entry, "pass" if not hits else "fail", f"local AQC path dependencies remain: {hits}" if hits else None)


def check_workflow_actions_pinned(entry):
    failures = []
    for rel_path in entry["files"]:
        for line_no, line in enumerate((ROOT / rel_path).read_text().splitlines(), start=1):
            stripped = line.strip()
            if not stripped.startswith("uses:"):
                continue
            value = stripped.removeprefix("uses:").strip().split("#", 1)[0].strip()
            if value.startswith("./"):
                continue
            if not PINNED_ACTION.fullmatch(value):
                failures.append(f"{rel_path}:{line_no}: {value}")
    emit(entry, "pass" if not failures else "fail", "; ".join(failures) or None)


def check_release_trusted_publishing(entry):
    text = (ROOT / entry["file"]).read_text()
    required = [
        "id-token: write",
        "rust-lang/crates-io-auth-action",
        "CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}",
        "cargo publish",
    ]
    forbidden = ["CARGO_REGISTRY_TOKEN: ${{ secrets."]
    failures = [f"missing {item}" for item in required if item not in text]
    failures.extend(f"forbidden {item}" for item in forbidden if item in text)
    emit(entry, "pass" if not failures else "fail", "; ".join(failures) or None)


CHECKS = {
    "github-specular-public": check_repo_field,
    "github-aqc-shared-public": check_repo_field,
    "github-specular-issues": check_repo_field,
    "github-specular-discussions": check_repo_field,
    "github-specular-topics": check_topics,
    "github-security-settings": check_security,
    "cargo-metadata-complete": check_cargo_metadata,
    "no-local-aqc-path-deps": check_no_local_path_deps,
    "workflow-actions-pinned": check_workflow_actions_pinned,
    "release-trusted-publishing": check_release_trusted_publishing,
}


def main():
    if len(sys.argv) != 3 or sys.argv[2] != "custom":
        raise SystemExit("usage: verify-repo-quality.py <spec.json> custom")

    spec = json.loads(Path(sys.argv[1]).read_text())
    for entry in spec["requirements"].get("custom", []):
        check = CHECKS.get(entry.get("check"))
        if check is None:
            emit(entry, "fail", f"unknown check {entry.get('check')!r}")
            continue
        try:
            check(entry)
        except Exception as error:
            emit(entry, "fail", str(error))


if __name__ == "__main__":
    main()
