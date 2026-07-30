#!/usr/bin/env python3
"""Restricted installed-host validator; installed as /usr/local/sbin/quireforge-validate-deb."""
from __future__ import annotations

import hashlib
import json
import os
import re
import stat
import subprocess
import sys
import uuid
from pathlib import Path
from typing import Callable

MAX_REQUEST_BYTES = 4096
PACKAGE = "quireforge"
DPKG_QUERY = "/usr/bin/dpkg-query"
DPKG = "/usr/bin/dpkg"
PROTECTED = ("/usr/bin/quireforge", "/usr/share/applications/io.github.codeframe78.QuireForge.desktop", "/usr/share/metainfo/io.github.codeframe78.QuireForge.metainfo.xml")
VERSION = re.compile(r"^[0-9][0-9A-Za-z.+-]{0,63}$")
DEBIAN_VERSION = re.compile(r"^[0-9][0-9A-Za-z.+:~-]{0,63}$")
UUID = re.compile(r"^[0-9a-f-]{36}$")
SHA = re.compile(r"^[0-9a-f]{64}$")

class Unavailable(Exception): pass
class Failed(Exception): pass

def canonical(value: dict[str, object]) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")

def read_request(source: object) -> dict[str, str]:
    raw = source.read(MAX_REQUEST_BYTES + 1)
    if not isinstance(raw, bytes) or len(raw) > MAX_REQUEST_BYTES:
        raise Failed()
    try:
        text = raw.decode("utf-8")
        decoder = json.JSONDecoder()
        value, index = decoder.raw_decode(text)
    except (UnicodeDecodeError, json.JSONDecodeError):
        raise Failed() from None
    if text[index:].strip() or not isinstance(value, dict) or set(value) != {"schema_version", "session_id", "nonce", "expected_application_version", "expected_debian_version"}:
        raise Failed()
    if value.get("schema_version") != 1 or not all(isinstance(value.get(key), str) for key in value if key != "schema_version"):
        raise Failed()
    try:
        session = uuid.UUID(value["session_id"])
    except ValueError:
        raise Failed() from None
    if not UUID.fullmatch(value["session_id"]) or session.version != 7 or not SHA.fullmatch(value["nonce"]) or not VERSION.fullmatch(value["expected_application_version"]) or not DEBIAN_VERSION.fullmatch(value["expected_debian_version"]):
        raise Failed()
    return value

def fixed_run(arguments: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(arguments, check=True, text=True, capture_output=True, shell=False)

def application_version_from_debian(value: str) -> str:
    return value.replace("~", "-", 1)

def validate(request: dict[str, str], run: Callable[[list[str]], object] = fixed_run, root: Path = Path("/"), stat_result: Callable[[Path], os.stat_result] = os.stat) -> dict[str, object]:
    try:
        status = run([DPKG_QUERY, "--showformat=${db:Status-Status}\\n${Version}\\n", "--show", PACKAGE]).stdout.splitlines()
        if status != ["installed", request["expected_debian_version"]]: raise Failed()
        if application_version_from_debian(request["expected_debian_version"]) != request["expected_application_version"]: raise Failed()
        for fixed in PROTECTED:
            path = root / fixed.lstrip("/")
            data = stat_result(path)
            if path.is_symlink() or not stat.S_ISREG(data.st_mode) or data.st_uid != 0 or data.st_mode & 0o022: raise Failed()
            owner = run([DPKG_QUERY, "--search", fixed]).stdout.strip()
            if owner != f"{PACKAGE}: {fixed}": raise Failed()
        integrity = run([DPKG, "--verify", PACKAGE]).stdout.strip()
        if integrity: raise Failed()
    except FileNotFoundError:
        raise Unavailable() from None
    except subprocess.CalledProcessError:
        raise Failed() from None
    facts: dict[str, object] = {"kind":"installed-host", "schema_version":1, "package_state":"installed", "version_match":True, "ownership_verified":True, "permissions_safe":True, "package_integrity_verified":True}
    return facts

def result(request: dict[str, str], outcome: str, facts: dict[str, object] | None = None) -> dict[str, object]:
    value: dict[str, object] = {"schema_version":1, "session_id":request["session_id"], "nonce":request["nonce"], "outcome":outcome, "facts":facts}
    value["result_sha256"] = hashlib.sha256(canonical(value)).hexdigest()
    return value

def main(argv: list[str] | None = None, source: object = sys.stdin.buffer, output: object = sys.stdout, error: object = sys.stderr) -> int:
    if (sys.argv[1:] if argv is None else argv):
        error.write("invalid request\n"); return 2
    try:
        request = read_request(source)
    except Failed:
        error.write("invalid request\n"); return 2
    try:
        value = result(request, "passed", validate(request))
    except Unavailable:
        value = result(request, "unavailable")
    except Failed:
        value = result(request, "failed")
    output.write(canonical(value).decode("utf-8") + "\n")
    return 0

if __name__ == "__main__": raise SystemExit(main())
