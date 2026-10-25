"""Shared process handling for the validation scripts: which `ltr` to run,
its environment, and stopping it.

The renderer has one backend, wgpu. Scripts still take a backend name
because it labels their artifacts (captures/<backend>/, probe file names);
anything but "wgpu" is rejected instead of being run under a wrong label.
"""
import os
import subprocess
import sys
from pathlib import Path

BACKENDS = ("wgpu",)


def check_backend(backend):
    if backend not in BACKENDS:
        sys.exit(f"unknown backend {backend!r}: the renderer runs on wgpu only "
                 f"(the OpenGL backend was removed); pass one of {BACKENDS}")
    return backend


def exe(target_dir):
    """`LTHEORY_EXE`, or the debug build of `ltr` in `target_dir`."""
    name = "ltr.exe" if os.name == "nt" else "ltr"
    return Path(os.environ.get("LTHEORY_EXE", Path(target_dir) / "debug" / name))


def prepare_env(env, backend):
    """Adjust a copy of the environment for a validation run of `backend`."""
    check_backend(backend)
    # Fatal mode: the first wgpu validation error panics instead of being logged.
    env.setdefault("LTHEORY_WGPU_FATAL", "1")
    return env


# Start `ltr` in its own process group, so a timed-out run can be stopped
# together with anything it spawned.
POPEN_GROUP = (
    {"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP}
    if os.name == "nt"
    else {"start_new_session": True}
)


def kill_tree(proc):
    """Stop `proc` and its process group, and reap it."""
    if proc.poll() is not None:
        return
    if os.name == "nt":
        subprocess.run(["taskkill.exe", "/PID", str(proc.pid), "/T", "/F"], check=False)
    else:
        import signal
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    try:
        proc.wait(timeout=5.0)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
