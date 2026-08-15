#!/usr/bin/env python3
"""Root-owned local Debian installer for root-owned QuireForge staging only."""
from __future__ import annotations

import grp
import json
import logging
import os
import signal
import socket
import stat
import subprocess
import sys
from pathlib import Path

SOCKET_PATH = Path("/run/quireforge-installd.sock")
STAGING_ROOT = Path("/opt/quireforge/packages")
GROUP = "quireforge-install"
MAX_REQUEST_BYTES = 4096


def response(ok: bool, code: int, message: str) -> bytes:
    return (json.dumps({"schema_version": 1, "ok": ok, "code": code, "message": message},
                       separators=(",", ":")) + "\n").encode()


def safe_root_owned(path: Path) -> bool:
    info = path.stat(follow_symlinks=False)
    return stat.S_ISREG(info.st_mode) and info.st_uid == 0 and not (info.st_mode & 0o022)


def safe_staging_root(path: Path) -> bool:
    info = path.stat(follow_symlinks=False)
    return stat.S_ISDIR(info.st_mode) and info.st_uid == 0 and not (info.st_mode & 0o022)


def validated_package(request_path: object) -> Path:
    if not isinstance(request_path, str) or not request_path.startswith("/"):
        raise ValueError("path must be absolute")
    original = Path(request_path)
    resolved_root = STAGING_ROOT.resolve(strict=True)
    resolved_package = original.resolve(strict=True)
    if original != resolved_package or resolved_package.parent != resolved_root:
        raise ValueError("path is outside the trusted staging directory")
    if resolved_package.suffix != ".deb" or not safe_root_owned(resolved_package):
        raise ValueError("package is not a safe root-owned Debian artifact")
    if not safe_staging_root(resolved_root):
        raise ValueError("trusted staging directory is unsafe")
    return resolved_package


def install(package: Path) -> tuple[bool, int]:
    first = subprocess.run(["/usr/bin/dpkg", "--install", str(package)], stdout=subprocess.DEVNULL,
                           stderr=subprocess.DEVNULL, check=False)
    repair = subprocess.run(["/usr/bin/apt-get", "-f", "install", "-y", "--no-download"],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
    return first.returncode == 0 and repair.returncode == 0, first.returncode or repair.returncode


def peer_uid(connection: socket.socket) -> int:
    raw = connection.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, 12)
    return int.from_bytes(raw[4:8], byteorder=sys.byteorder)


def handle(connection: socket.socket) -> None:
    uid = peer_uid(connection)
    raw = connection.recv(MAX_REQUEST_BYTES + 1)
    try:
        if len(raw) > MAX_REQUEST_BYTES:
            raise ValueError("request too large")
        request = json.loads(raw.decode("utf-8"))
        if not isinstance(request, dict) or set(request) != {"schema_version", "path"} or request["schema_version"] != 1:
            raise ValueError("invalid request")
        package = validated_package(request["path"])
        ok, code = install(package)
        logging.info("uid=%d path=%s result=%s exit=%d", uid, package, "installed" if ok else "failed", code)
        connection.sendall(response(ok, code, "installed" if ok else "installation failed"))
    except (OSError, UnicodeDecodeError, ValueError, json.JSONDecodeError) as error:
        logging.info("uid=%d path=%s result=rejected", uid, "<invalid>")
        connection.sendall(response(False, 64, "request rejected"))


def main() -> int:
    if os.geteuid() != 0:
        print("quireforge-installd must run as root", file=sys.stderr)
        return 77
    group = grp.getgrnam(GROUP)
    if SOCKET_PATH.exists() or SOCKET_PATH.is_symlink():
        if not stat.S_ISSOCK(SOCKET_PATH.lstat().st_mode):
            print("refusing to replace a non-socket path", file=sys.stderr)
            return 78
        SOCKET_PATH.unlink()
    logging.basicConfig(filename="/var/log/quireforge-installd.log", level=logging.INFO,
                        format="%(asctime)s %(message)s")
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server:
        server.bind(str(SOCKET_PATH))
        os.chown(SOCKET_PATH, 0, group.gr_gid)
        os.chmod(SOCKET_PATH, 0o660)
        server.listen(16)
        signal.signal(signal.SIGTERM, lambda *_: server.close())
        while True:
            try:
                connection, _ = server.accept()
            except OSError:
                return 0
            with connection:
                handle(connection)


if __name__ == "__main__":
    raise SystemExit(main())
