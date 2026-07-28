#!/usr/bin/env bash
set -euo pipefail

cache_dir="${1:?cache directory required}"
cache_name="${2:?cache filename required}"
source_url="${3:?source URL required}"
expected_sha256="${4:?expected SHA-256 required}"
destination="${5:?destination required}"

if [[ ! "$cache_name" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ || "$cache_name" == *..* ]]; then
  echo "sandbox source cache name is unsafe" >&2
  exit 1
fi
if [[ ! "$expected_sha256" =~ ^[0-9a-f]{64}$ ]]; then
  echo "sandbox source cache checksum is invalid" >&2
  exit 1
fi

umask 077
if [[ -L "$cache_dir" ]]; then
  echo "sandbox source cache directory must not be a symlink" >&2
  exit 1
fi
install -d -m 0700 "$cache_dir"
if [[ -L "$cache_dir" || ! -d "$cache_dir" ]]; then
  echo "sandbox source cache directory is invalid" >&2
  exit 1
fi
cache_path="$cache_dir/$cache_name"

verify() {
  printf '%s  %s\n' "$expected_sha256" "$1" | sha256sum --check --status
}

if [[ -L "$cache_path" ]]; then
  rm -f -- "$cache_path"
fi

if [[ -f "$cache_path" ]] && ! verify "$cache_path"; then
  rm -f -- "$cache_path"
fi

if [[ ! -f "$cache_path" ]]; then
  temporary="$(mktemp "$cache_dir/.${cache_name}.tmp.XXXXXX")"
  trap 'rm -f -- "$temporary"' EXIT
  curl --fail --location --retry 3 --output "$temporary" "$source_url"
  verify "$temporary"
  mv -f -- "$temporary" "$cache_path"
  trap - EXIT
fi

mkdir -p "$(dirname "$destination")"
cp --reflink=auto -- "$cache_path" "$destination"
verify "$destination"
