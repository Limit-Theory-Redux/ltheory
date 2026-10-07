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
log_path = artifact_dir / f"ui-composite-{backend}-latest.log"
env = os.environ.copy()
if backend == "wgpu":
    env["LTHEORY_WGPU"] = "1"
    # Fatal mode: the first wgpu validation error panics instead of being logged.
    env.setdefault("LTHEORY_WGPU_FATAL", "1")
else:
    env.pop("LTHEORY_WGPU", None)
args = [str(exe), "-e", "./script/Main.lua", "Rendering/UiComposite"]
timeout_seconds = float(os.environ.get("UI_COMPOSITE_TIMEOUT_SECONDS", "20"))

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
    r"\[UiCompositeProbe\].*outside=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)\s+"
    r"inside=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)",
    text,
)
print(f"backend={backend} returncode={code} normal_exit={code == 0}", flush=True)
print("--- probe/log markers ---")
for line in text.splitlines():
    if any(key in line for key in (
        "Application name:", "UiCompositeProbe", "Render thread stopped",
        "All Lua workers", "GL context", "WARN", "ERROR", "panic",
        "validation",
    )):
        print(line)

required = (
    "[UiCompositeProbe]",
    "All Lua workers were stopped",
)
has_error = re.search(r"\b(?:ERROR|panic)\b", text) is not None
if code != 0 or marker is None or has_error or not all(item in text for item in required):
    print(f"ui_composite_acceptance=false error={has_error}", flush=True)
    sys.exit(code if code else 2)

values = [float(item) for item in marker.groups()]
outside, inside = values[0:3], values[3:6]
print(f"ui_composite_samples outside={tuple(outside)} inside={tuple(inside)}", flush=True)
accepted = (
    max(abs(outside[0]), abs(outside[1]), abs(outside[2] - 1.0)) < 0.06
    and abs(inside[0] - 0.5) < 0.08
    and inside[1] < 0.06
    and abs(inside[2] - 0.5) < 0.08
)
print(f"ui_composite_acceptance={accepted}", flush=True)
sys.exit(0 if accepted else 3)
