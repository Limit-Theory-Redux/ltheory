import os
import re
import subprocess
import sys
from pathlib import Path

backend = sys.argv[1]
stage = sys.argv[2] if len(sys.argv) > 2 else "baseline"
workdir = Path(__file__).resolve().parents[2]
artifact_dir = Path(
    os.environ.get("LTHEORY_VALIDATION_ARTIFACT_DIR", workdir / "target" / "render_validation")
)
artifact_dir.mkdir(parents=True, exist_ok=True)
target_dir = Path(os.environ.get("CARGO_TARGET_DIR", workdir / "target"))
exe = Path(os.environ.get("LTHEORY_EXE", target_dir / "debug" / "ltr.exe"))
log_path = artifact_dir / f"indexed-geometry-{stage}-{backend}-latest.log"
env = os.environ.copy()
env["INDEXED_GEOMETRY_STAGE"] = stage
if backend == "wgpu":
    env["LTHEORY_WGPU"] = "1"
else:
    env.pop("LTHEORY_WGPU", None)
args = [str(exe), "-e", "./script/Main.lua", "Rendering/IndexedGeometry"]
timeout_seconds = float(os.environ.get("INDEXED_GEOMETRY_TIMEOUT_SECONDS", "20"))

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
    r"\[IndexedGeometryProbe\].*samples tl=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)\s+"
    r"center=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)\s+"
    r"br=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)",
    text,
)
print(f"stage={stage} backend={backend} returncode={code} normal_exit={code == 0}", flush=True)
print("--- probe/log markers ---")
for line in text.splitlines():
    if any(key in line for key in ("Application name:", "IndexedGeometryProbe", "Render thread stopped", "All Lua workers", "GL context", "WARN", "ERROR", "panic", "validation")):
        print(line)

required = ("[IndexedGeometryProbe]", "Render thread stopped", "All Lua workers were stopped")
if code != 0 or marker is None or not all(item in text for item in required):
    print(f"indexed_geometry_acceptance=false stage={stage}", flush=True)
    sys.exit(code if code else 2)

values = [float(item) for item in marker.groups()]
if len(values) != 9:
    print(f"indexed_geometry_acceptance=false stage={stage} reason=sample-shape", flush=True)
    sys.exit(2)
tl, center, br = values[0:3], values[3:6], values[6:9]
accepted = center[0] > 0.5 and center[1] > 0.05 and tl[0] < 0.05 and br[0] < 0.05
print(
    f"indexed_geometry_samples tl={tuple(tl)} center={tuple(center)} br={tuple(br)}",
    flush=True,
)
print(f"indexed_geometry_acceptance={accepted} stage={stage}", flush=True)
sys.exit(0 if accepted else 3)
