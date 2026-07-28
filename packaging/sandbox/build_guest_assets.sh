#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
source "$root/packaging/sandbox/sources.lock"
output="${1:?output directory required}"
work="${2:?work directory required}"
mkdir -p "$output" "$work"

kernel_tar="$work/linux-${LINUX_VERSION}.tar.xz"
curl --fail --location --retry 3 --output "$kernel_tar" "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-${LINUX_VERSION}.tar.xz"
echo "${LINUX_TAR_XZ_SHA256}  ${kernel_tar}" | sha256sum --check --status
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
(cd "$init" && find . -print0 | cpio --null --create --format=newc | gzip -9) > "$output/initramfs.cpio.gz"
curl --fail --location --retry 3 --output "$work/firecracker.tgz" "https://github.com/firecracker-microvm/firecracker/releases/download/v${FIRECRACKER_VERSION}/firecracker-v${FIRECRACKER_VERSION}-x86_64.tgz"
echo "${FIRECRACKER_X86_64_SHA256}  ${work}/firecracker.tgz" | sha256sum --check --status
mkdir -p "$work/firecracker"
tar --extract --gzip --file "$work/firecracker.tgz" --directory "$work/firecracker"
release_dir="$work/firecracker/release-v${FIRECRACKER_VERSION}-x86_64"
install -m 0755 "$release_dir/firecracker-v${FIRECRACKER_VERSION}-x86_64" "$output/firecracker"
install -m 0755 "$release_dir/jailer-v${FIRECRACKER_VERSION}-x86_64" "$output/jailer"
(cd "$output" && sha256sum firecracker jailer vmlinux initramfs.cpio.gz) > "$output/SHA256SUMS"
