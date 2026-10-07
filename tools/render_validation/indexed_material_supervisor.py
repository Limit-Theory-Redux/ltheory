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
log_path = artifact_dir / f"indexed-material-{backend}-latest.log"
env = os.environ.copy()
if backend == "wgpu":
    env["LTHEORY_WGPU"] = "1"
    # Fatal mode: the first wgpu validation error panics instead of being logged.
    env.setdefault("LTHEORY_WGPU_FATAL", "1")
else:
    env.pop("LTHEORY_WGPU", None)
args = [str(exe), "-e", "./script/Main.lua", "Rendering/IndexedMaterial"]
timeout_seconds = float(os.environ.get("INDEXED_MATERIAL_TIMEOUT_SECONDS", "20"))

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
    r"\[IndexedMaterialProbe\].*samples tl=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)\s+"
    r"center=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)\s+"
    r"br=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)",
    text,
)
print(f"backend={backend} returncode={code} normal_exit={code == 0}", flush=True)
print("--- probe/log markers ---")
for line in text.splitlines():
    if any(key in line for key in (
        "Application name:", "IndexedMaterialProbe", "Render thread stopped",
        "All Lua workers", "GL context", "WARN", "ERROR", "panic",
        "validation",
    )):
        print(line)

required = (
    "[IndexedMaterialProbe]",
    "All Lua workers were stopped",
)
if code != 0 or marker is None or not all(item in text for item in required):
    print("indexed_material_acceptance=false", flush=True)
    sys.exit(code if code else 2)

values = [float(item) for item in marker.groups()]
tl, center, br = values[0:3], values[3:6], values[6:9]
print(
    f"indexed_material_samples tl={tuple(tl)} center={tuple(center)} br={tuple(br)}",
    flush=True,
)
accepted = (
    abs(center[0] - 0.25) < 0.05
    and abs(center[1] - 0.75) < 0.05
    and abs(center[2] - 0.125) < 0.05
    and tl[0] < 0.05
    and br[0] < 0.05
)
print(f"indexed_material_acceptance={accepted}", flush=True)
sys.exit(0 if accepted else 3)
