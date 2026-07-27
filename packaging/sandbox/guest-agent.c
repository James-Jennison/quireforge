/* Minimal non-interactive M39 initramfs agent. It accepts one raw virtio block
 * input with a QFELF001 header, runs it as uid/gid 65534 with stdout/stderr
 * discarded, and emits only a fixed result line on the root-only serial fd. */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <grp.h>
#include <linux/reboot.h>
#include <linux/memfd.h>
#include <sched.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/reboot.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#define MAX_SAMPLE_BYTES (32 * 1024 * 1024)

static void result(const char *status) {
  int console = open("/dev/console", O_WRONLY | O_NOCTTY | O_CLOEXEC);
  if (console >= 0) {
    dprintf(console, "QF_DYNAMIC_RESULT_V1:%s\n", status);
    close(console);
  }
}

int main(void) {
  /* The sealed sample is executed only through this process's anonymous FD.
   * Mount a private procfs so /proc/self/fd resolves inside the disposable
   * guest; no host filesystem is mounted. */
  if (mount("proc", "/proc", "proc", MS_NOSUID | MS_NODEV | MS_NOEXEC, NULL) != 0) {
    result("setup-failed");
    goto done;
  }
  unsigned char header[16];
  int input = open("/dev/vda", O_RDONLY | O_CLOEXEC | O_NOFOLLOW);
  if (input < 0 || read(input, header, sizeof(header)) != sizeof(header) ||
      memcmp(header, "QFELF001", 8) != 0) { result("setup-failed"); goto done; }
  uint64_t size = 0; memcpy(&size, header + 8, sizeof(size));
  if (!size || size > MAX_SAMPLE_BYTES) { result("policy-denied"); goto done; }
  int sample = syscall(SYS_memfd_create, "sample", MFD_CLOEXEC);
  if (sample < 0) { result("setup-failed"); goto done; }
  unsigned char buffer[65536]; uint64_t remaining = size;
  while (remaining) { size_t want = remaining > sizeof(buffer) ? sizeof(buffer) : (size_t)remaining; ssize_t got = read(input, buffer, want); if (got <= 0 || write(sample, buffer, (size_t)got) != got) { result("setup-failed"); close(sample); goto done; } remaining -= (uint64_t)got; }
  fchmod(sample, 0500);
  pid_t child = fork();
  if (child == 0) {
    int nullfd = open("/dev/null", O_RDWR | O_CLOEXEC); if (nullfd >= 0) { dup2(nullfd, STDIN_FILENO); dup2(nullfd, STDOUT_FILENO); dup2(nullfd, STDERR_FILENO); }
    if (setgroups(0, NULL) != 0 || setgid(65534) != 0 || setuid(65534) != 0) _exit(126);
    char fdpath[32]; snprintf(fdpath, sizeof(fdpath), "/proc/self/fd/%d", sample);
    char *const argv[] = { "sample", NULL }; char *const envp[] = { "PATH=/usr/bin:/bin", NULL };
    execve(fdpath, argv, envp); _exit(127);
  }
  if (child < 0) { result("setup-failed"); close(sample); goto done; }
  int status = 0; if (waitpid(child, &status, 0) < 0) result("setup-failed");
  else if (WIFEXITED(status) && WEXITSTATUS(status) == 0) result("completed");
  else if (WIFEXITED(status)) result("nonzero-exit");
  else if (WIFSIGNALED(status)) result("signal"); else result("setup-failed");
  close(sample);
done:
  if (input >= 0) close(input);
  sync(); reboot(LINUX_REBOOT_CMD_POWER_OFF); for (;;) pause();
}
