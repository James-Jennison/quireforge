#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
source "$root/packaging/sandbox/sources.lock"
output="${1:?output directory required}"
work="${2:?work directory required}"
cache_root="${QUIRE_FORGE_SANDBOX_SOURCE_CACHE:?authoritative sandbox source cache required}"
source_date_epoch="${SOURCE_DATE_EPOCH:?SOURCE_DATE_EPOCH is required}"
if [[ ! "$source_date_epoch" =~ ^[0-9]+$ ]]; then
  echo "SOURCE_DATE_EPOCH must be a non-negative integer" >&2
  exit 1
fi
export KBUILD_BUILD_TIMESTAMP
KBUILD_BUILD_TIMESTAMP="$(date --utc --date="@${source_date_epoch}" '+%a %b %e %T %Y')"
export KBUILD_BUILD_USER=quireforge
export KBUILD_BUILD_HOST=ubuntu-22.04
export KBUILD_BUILD_VERSION=1
mkdir -p "$output" "$work"

kernel_tar="$work/linux-${LINUX_VERSION}.tar.xz"
bash "$root/packaging/sandbox/cache_verified_source.sh" \
  "$cache_root" \
  "linux-${LINUX_VERSION}-${LINUX_TAR_XZ_SHA256}.tar.xz" \
  "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-${LINUX_VERSION}.tar.xz" \
  "$LINUX_TAR_XZ_SHA256" \
  "$kernel_tar"
tar --extract --file "$kernel_tar" --directory "$work"
kernel="$work/linux-${LINUX_VERSION}"
make -C "$kernel" ARCH=x86_64 tinyconfig
"$kernel/scripts/config" --file "$kernel/.config" --enable BLK_DEV --enable VIRTIO --enable VIRTIO_PCI --enable VIRTIO_BLK --enable DEVTMPFS --enable DEVTMPFS_MOUNT --enable PROC_FS --enable BINFMT_ELF --enable ELFCORE --enable SERIAL_8250 --enable SERIAL_8250_CONSOLE --enable TTY --enable VT --enable UNIX --enable TMPFS --enable INET --disable NET --disable MODULES
make -C "$kernel" ARCH=x86_64 olddefconfig vmlinux -j"${CARGO_BUILD_JOBS:-4}"
install -m 0644 "$kernel/vmlinux" "$output/vmlinux"

agent="$work/quireforge-guest-agent"
gcc -static -Os -s -Wall -Wextra -Werror "$root/packaging/sandbox/guest-agent.c" -o "$agent"
init="$work/initramfs"; mkdir -p "$init/proc" "$init/dev"
install -m 0755 "$agent" "$init/init"
find "$init" -exec touch --no-dereference --date="@${source_date_epoch}" {} +
(
  cd "$init"
  find . -print0 | LC_ALL=C sort --zero-terminated | cpio --null --create --format=newc --owner=0:0 --reproducible | gzip -n -9
) > "$output/initramfs.cpio.gz"
bash "$root/packaging/sandbox/cache_verified_source.sh" \
  "$cache_root" \
  "firecracker-v${FIRECRACKER_VERSION}-${FIRECRACKER_X86_64_SHA256}.tgz" \
  "https://github.com/firecracker-microvm/firecracker/releases/download/v${FIRECRACKER_VERSION}/firecracker-v${FIRECRACKER_VERSION}-x86_64.tgz" \
  "$FIRECRACKER_X86_64_SHA256" \
  "$work/firecracker.tgz"
mkdir -p "$work/firecracker"
tar --extract --gzip --file "$work/firecracker.tgz" --directory "$work/firecracker"
release_dir="$work/firecracker/release-v${FIRECRACKER_VERSION}-x86_64"
install -m 0755 "$release_dir/firecracker-v${FIRECRACKER_VERSION}-x86_64" "$output/firecracker"
install -m 0755 "$release_dir/jailer-v${FIRECRACKER_VERSION}-x86_64" "$output/jailer"
(cd "$output" && sha256sum firecracker jailer vmlinux initramfs.cpio.gz) > "$output/SHA256SUMS"
