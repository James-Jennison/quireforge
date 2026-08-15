#!/usr/bin/env python3
"""Unprivileged client for the root-owned QuireForge installer daemon."""
from __future__ import annotations

import json
import socket
import sys

SOCKET_PATH = "/run/quireforge-installd.sock"


def main(arguments: list[str]) -> int:
    if len(arguments) != 1 or not arguments[0].startswith("/"):
        print("usage: quireforge-install /absolute/path/to/package.deb", file=sys.stderr)
        return 64
    request = json.dumps({"schema_version": 1, "path": arguments[0]}, separators=(",", ":")).encode()
    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.connect(SOCKET_PATH)
            client.sendall(request)
            raw = client.recv(4096)
        result = json.loads(raw.decode("utf-8"))
        if not isinstance(result, dict) or result.get("schema_version") != 1 or not isinstance(result.get("code"), int):
            raise ValueError("invalid daemon response")
    except (OSError, UnicodeDecodeError, ValueError, json.JSONDecodeError) as error:
        print(json.dumps({"ok": False, "code": 69, "message": "installer unavailable"}, separators=(",", ":")))
        return 69
    print(json.dumps(result, separators=(",", ":")))
    return 0 if result.get("ok") else result["code"] or 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
