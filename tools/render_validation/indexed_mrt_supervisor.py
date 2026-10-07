import os
import re
import subprocess
import sys
from pathlib import Path

backend = sys.argv[1]
workdir = Path(__file__).resolve().parents[2]
artifact_dir = Path(
    os.environ.get("LTHEORY_VALIDATION_ARTIFACT_DIR", workdir / "target" / "render_validation")
)
artifact_dir.mkdir(parents=True, exist_ok=True)
target_dir = Path(os.environ.get("CARGO_TARGET_DIR", workdir / "target"))
exe = Path(os.environ.get("LTHEORY_EXE", target_dir / "debug" / "ltr.exe"))
log_path = artifact_dir / f"indexed-mrt-{backend}-latest.log"
env = os.environ.copy()
if backend == "wgpu":
    env["LTHEORY_WGPU"] = "1"
    # Fatal mode: the first wgpu validation error panics instead of being logged.
    env.setdefault("LTHEORY_WGPU_FATAL", "1")
else:
    env.pop("LTHEORY_WGPU", None)
args = [str(exe), "-e", "./script/Main.lua", "Rendering/IndexedMrt"]
timeout_seconds = float(os.environ.get("INDEXED_MRT_TIMEOUT_SECONDS", "20"))

with log_path.open("w", encoding="utf-8", newline="") as log:
    proc = subprocess.Popen(
        args,
        cwd=workdir,
        env=env,
        stdout=log,
        stderr=subprocess.STDOUT,
        creationflags=getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0),
    )
    print(f"backend={backend} pid={proc.pid} log={log_path}", flush=True)
    try:
        code = proc.wait(timeout=timeout_seconds)
    except subprocess.TimeoutExpired:
        print(f"backend={backend} timeout=true; terminating process tree", flush=True)
        subprocess.run(["taskkill.exe", "/PID", str(proc.pid), "/T", "/F"], check=False)
        code = proc.wait(timeout=5.0)
        print(f"backend={backend} returncode={code} normal_exit=false", flush=True)
        sys.exit(124)

text = log_path.read_text(encoding="utf-8", errors="replace")
marker = re.search(
    r"\[IndexedMrtProbe\].*buffer0=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)\s+"
    r"buffer1=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)\s+"
    r"zBufferL=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)",
    text,
)
print(f"backend={backend} returncode={code} normal_exit={code == 0}", flush=True)
print("--- probe/log markers ---")
for line in text.splitlines():
    if any(key in line for key in (
        "Application name:", "IndexedMrtProbe", "Render thread stopped",
        "All Lua workers", "GL context", "WARN", "ERROR", "panic",
        "validation",
    )):
        print(line)

required = (
    "[IndexedMrtProbe]",
    "All Lua workers were stopped",
)
has_error = re.search(r"\b(?:ERROR|panic)\b", text) is not None
if code != 0 or marker is None or has_error or not all(item in text for item in required):
    print(f"indexed_mrt_acceptance=false error={has_error}", flush=True)
    sys.exit(code if code else 2)

values = [float(item) for item in marker.groups()]
buffer0, buffer1, zbufferl = values[0:3], values[3:6], values[6:9]
print(
    f"indexed_mrt_samples buffer0={tuple(buffer0)} buffer1={tuple(buffer1)} zBufferL={tuple(zbufferl)}",
    flush=True,
)
accepted = (
    buffer0[0] > 0.9 and buffer0[1] < 0.05 and buffer0[2] < 0.05
    and buffer1[0] < 0.05 and buffer1[1] > 0.9 and buffer1[2] < 0.05
    and abs(zbufferl[0] - 0.75) < 0.08 and zbufferl[1] < 0.05 and zbufferl[2] < 0.05
)
print(f"indexed_mrt_acceptance={accepted}", flush=True)
sys.exit(0 if accepted else 3)
