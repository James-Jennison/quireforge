#!/usr/bin/env python3
"""Fail-closed validation for the reviewed Node development-tooling exception."""
from __future__ import annotations
import argparse, datetime as dt, hashlib, json, subprocess, sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
EXCEPTIONS_PATH = ROOT / "security" / "node-audit-exceptions.json"
RAW_AUDIT_COMMAND = ("pnpm", "audit", "--audit-level", "high", "--json")
REQUIRED_FIELDS = {"githubAdvisoryId", "severity", "package", "version", "pathCount", "pathSetSha256", "developmentOnly", "owner", "reviewDate", "expiresOn", "rationale", "compensatingControls", "removalTrigger"}

def path_set_sha256(paths: list[str]) -> str:
    return hashlib.sha256(json.dumps(sorted(paths), separators=(",", ":")).encode()).hexdigest()

def load_exceptions(path: Path = EXCEPTIONS_PATH) -> list[dict[str, Any]]:
    record = json.loads(path.read_text(encoding="utf-8"))
    exceptions = record.get("exceptions")
    if record.get("schemaVersion") != 1 or not isinstance(exceptions, list) or not exceptions:
        raise ValueError("node audit exception record must use non-empty schemaVersion 1 exceptions")
    for item in exceptions:
        if set(item) != REQUIRED_FIELDS or not isinstance(item["pathCount"], int) or item["pathCount"] < 1 or item["developmentOnly"] is not True:
            raise ValueError("node audit exception fields must match the reviewed schema")
        if not all(isinstance(item[field], str) and item[field] for field in REQUIRED_FIELDS - {"pathCount", "developmentOnly"}):
            raise ValueError("node audit exception text fields must be non-empty")
        if len(item["pathSetSha256"]) != 64 or any(char not in "0123456789abcdef" for char in item["pathSetSha256"]):
            raise ValueError("node audit exception pathSetSha256 must be a SHA-256 hex digest")
        review, expiry = dt.date.fromisoformat(item["reviewDate"]), dt.date.fromisoformat(item["expiresOn"])
        if review >= expiry or expiry < dt.date.today():
            raise ValueError("node audit exception review/expiry dates are invalid or expired")
    return exceptions

def validate_report(report: dict[str, Any], exceptions: list[dict[str, Any]]) -> list[str]:
    advisories = report.get("advisories")
    if not isinstance(advisories, dict): return ["raw pnpm audit JSON is missing advisories"]
    actual = {item.get("github_advisory_id"): item for item in advisories.values()}
    expected = {item["githubAdvisoryId"]: item for item in exceptions}
    errors: list[str] = []
    if None in actual or len(actual) != len(advisories) or set(actual) != set(expected): errors.append("raw audit advisories do not exactly match the reviewed exception IDs")
    for advisory_id, item in expected.items():
        advisory = actual.get(advisory_id)
        if not advisory: continue
        if advisory.get("severity") != item["severity"]: errors.append(f"{advisory_id}: severity does not match")
        if advisory.get("module_name") != item["package"]: errors.append(f"{advisory_id}: package does not match")
        findings = advisory.get("findings")
        if not isinstance(findings, list) or len(findings) != 1: errors.append(f"{advisory_id}: expected exactly one finding"); continue
        finding = findings[0]
        if finding.get("version") != item["version"]: errors.append(f"{advisory_id}: affected version does not match")
        if finding.get("dev") is not item["developmentOnly"]: errors.append(f"{advisory_id}: development-only status does not match")
        paths = finding.get("paths")
        if not isinstance(paths, list) or not all(isinstance(value, str) for value in paths) or len(paths) != item["pathCount"] or path_set_sha256(paths) != item["pathSetSha256"]:
            errors.append(f"{advisory_id}: dependency paths do not exactly match")
    return errors

def raw_audit_report() -> dict[str, Any]:
    completed = subprocess.run(RAW_AUDIT_COMMAND, cwd=ROOT, capture_output=True, text=True, check=False)
    if completed.returncode not in {0, 1}: raise RuntimeError(f"raw pnpm audit failed unexpectedly with exit {completed.returncode}: {completed.stderr.strip()}")
    try: return json.loads(completed.stdout)
    except json.JSONDecodeError as error: raise RuntimeError(f"raw pnpm audit did not return JSON: {completed.stderr.strip()}") from error

def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("--check-record", action="store_true"); arguments = parser.parse_args()
    try:
        exceptions = load_exceptions()
        if arguments.check_record: print("node audit exception record is current and structurally valid"); return 0
        errors = validate_report(raw_audit_report(), exceptions)
    except (OSError, RuntimeError, ValueError, json.JSONDecodeError) as error:
        print(f"node audit exception validation failed: {error}", file=sys.stderr); return 1
    if errors:
        for error in errors: print(f"node audit exception validation failed: {error}", file=sys.stderr)
        return 1
    print("node audit exception matched the raw audit exactly"); return 0

if __name__ == "__main__": raise SystemExit(main())
