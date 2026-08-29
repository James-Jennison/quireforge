#!/usr/bin/env bash
set -euo pipefail

# Keep the release build's high-write paths off /tmp.  This host mounts /tmp
# as tmpfs, where a large linker output can fail with SIGBUS once RAM and swap
# backing are exhausted.
umask 077

repository_root="$(git rev-parse --show-toplevel)"
if [[ "$(pwd -P)" != "$(cd "$repository_root" && pwd -P)" ]]; then
  echo "Run this script from the QuireForge repository root." >&2
  exit 1
fi

builder_image="quireforge-packaging:ubuntu-22.04"
builder_source="ubuntu:22.04@sha256:0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982"
build_root="/mnt/faststorage/quireforge-build"
build_marker="$build_root/.quireforge-linux-packaging-root"
cache_root="$build_root/cache"
target_root="$build_root/target"
temporary_root="$build_root/tmp"
minimum_free_bytes=$((30 * 1024 * 1024 * 1024))
minimum_free_inodes=200000
source_revision="$(git rev-parse HEAD)"
if [[ ! "$source_revision" =~ ^[0-9a-f]{40}$ ]]; then
  echo "Could not resolve a full lowercase source revision." >&2
  exit 1
fi
if [[ -n "$(git status --short --untracked-files=all)" ]]; then
  echo "Authoritative Linux release artifacts require a clean working tree." >&2
  exit 1
fi
source_epoch="$(git show -s --format=%ct "$source_revision")"
if [[ ! "$source_epoch" =~ ^[0-9]+$ ]]; then
  echo "Could not resolve SOURCE_DATE_EPOCH from the source revision." >&2
  exit 1
fi

mkdir -p "$build_root" "$cache_root" "$target_root" "$temporary_root"
if [[ ! -e "$build_marker" ]]; then
  printf '%s\n' 'quireforge-linux-packaging-build-root-v1' > "$build_marker"
elif [[ "$(<"$build_marker")" != 'quireforge-linux-packaging-build-root-v1' ]]; then
  echo "Refusing unrecognized QuireForge packaging build root: $build_root" >&2
  exit 1
fi

build_filesystem="$(findmnt -no FSTYPE -T "$build_root")"
case "$build_filesystem" in
  tmpfs|ramfs)
    echo "QuireForge packaging build root must be disk-backed, not $build_filesystem: $build_root" >&2
    exit 1
    ;;
esac

# A single target/cache namespace keeps Cargo incremental artifacts reusable
# and makes the free-space guarantee meaningful.  A second release build must
# wait rather than racing a point-in-time df check.
exec 9>"$build_root/.release.lock"
if ! flock -n 9; then
  echo "Another QuireForge Linux package build is already using $build_root" >&2
  exit 1
fi

QUIRE_FORGE_BUILD_LOCK_HELD=1 \
  "$repository_root/scripts/cleanup_linux_package_build_cache.sh"

available_bytes="$(df -PB1 "$build_root" | awk 'NR == 2 {print $4}')"
available_inodes="$(df -Pi "$build_root" | awk 'NR == 2 {print $4}')"
if [[ ! "$available_bytes" =~ ^[0-9]+$ || "$available_bytes" -lt "$minimum_free_bytes" ]]; then
  echo "QuireForge packaging requires at least 30 GiB free at $build_root" >&2
  exit 1
fi
if [[ ! "$available_inodes" =~ ^[0-9]+$ || "$available_inodes" -lt "$minimum_free_inodes" ]]; then
  echo "QuireForge packaging requires at least $minimum_free_inodes free inodes at $build_root" >&2
  exit 1
fi

temporary_build="$(mktemp -d "$temporary_root/run.XXXXXXXX")"
printf '%s\n' 'quireforge-linux-packaging-run-v1' > "$temporary_build/.quireforge-linux-packaging-run"
cleanup_temporary_build() {
  local status=$?
  if [[ $status -eq 0 && -d "$temporary_build" \
    && -f "$temporary_build/.quireforge-linux-packaging-run" ]]; then
    rm -rf -- "$temporary_build"
  elif [[ -d "$temporary_build" && -f "$temporary_build/.quireforge-linux-packaging-run" ]]; then
    : > "$temporary_build/.failed"
  fi
  exit "$status"
}
trap cleanup_temporary_build EXIT

mkdir -p \
  "$cache_root/cargo" \
  "$cache_root/home" \
  "$cache_root/node_modules/desktop" \
  "$cache_root/node_modules/root" \
  "$cache_root/node_modules/website" \
  "$cache_root/pnpm-store" \
  "$cache_root/sandbox-sources"

docker build \
  --file packaging/linux/Dockerfile \
  --tag "$builder_image" \
  packaging/linux

docker run \
  --rm \
  --init \
  --user "$(id -u):$(id -g)" \
  --volume "$repository_root:/workspace" \
  --volume "$cache_root:/cache" \
  --volume "$target_root:/workspace/target" \
  --volume "$temporary_build:/build-tmp" \
  --volume "$cache_root/node_modules/root:/workspace/node_modules" \
  --volume "$cache_root/node_modules/desktop:/workspace/apps/desktop/node_modules" \
  --volume "$cache_root/node_modules/website:/workspace/apps/website/node_modules" \
  --env CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}" \
  --env CARGO_HOME=/cache/cargo \
  --env CARGO_TARGET_DIR=/workspace/target/ubuntu-22.04 \
  --env CI=true \
  --env HOME=/cache/home \
  --env QUIRE_FORGE_BUILD_DISTRIBUTION=ubuntu \
  --env QUIRE_FORGE_BUILD_IMAGE="$builder_source" \
  --env QUIRE_FORGE_BUILD_VERSION=22.04 \
  --env QUIRE_FORGE_RELEASE_BUILDER=pinned-ubuntu-22.04 \
  --env QUIRE_FORGE_SOURCE_REVISION="$source_revision" \
  --env QUIRE_FORGE_SANDBOX_SOURCE_CACHE=/cache/sandbox-sources \
  --env QUIRE_FORGE_TAURI_CACHE_DIR=/cache/home/.cache/tauri \
  --env SOURCE_DATE_EPOCH="$source_epoch" \
  --env TEMP=/build-tmp \
  --env TMP=/build-tmp \
  --env TMPDIR=/build-tmp \
  --env XDG_CACHE_HOME=/cache/home/.cache \
  --workdir /workspace \
  "$builder_image" \
  /bin/bash -c \
  "pnpm install --frozen-lockfile --store-dir /cache/pnpm-store \
    && pnpm package:linux:release \
    && python3 scripts/validate_release_artifacts.py \
      --artifact-dir target/ubuntu-22.04/release/packages/.candidate-$(node -p 'require("./package.json").version') \
      --lifecycle \
      --smoke \
    && python3 scripts/package_linux.py --finalize \
    && python3 scripts/validate_release_artifacts.py \
      --artifact-dir target/ubuntu-22.04/release/packages"
