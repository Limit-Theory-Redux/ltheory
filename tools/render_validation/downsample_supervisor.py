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
log_path = artifact_dir / f"downsample-{backend}-latest.log"
env = os.environ.copy()
if backend == "wgpu":
    env["LTHEORY_WGPU"] = "1"
else:
    env.pop("LTHEORY_WGPU", None)
args = [str(exe), "-e", "./script/Main.lua", "Rendering/Downsample"]
timeout_seconds = float(os.environ.get("DOWNSAMPLE_TIMEOUT_SECONDS", "20"))

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
    r"\[DownsampleProbe\].*sourceCenter=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)\s+"
    r"linearCenter=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)\s+"
    r"nearestCenter=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)\s+"
    r"upscaledCenter=\(([-+0-9.eE]+),([-+0-9.eE]+),([-+0-9.eE]+)\)",
    text,
)
print(f"backend={backend} returncode={code} normal_exit={code == 0}", flush=True)
print("--- probe/log markers ---")
for line in text.splitlines():
    if any(key in line for key in (
        "Application name:", "DownsampleProbe", "Render thread stopped",
        "All Lua workers", "GL context", "WARN", "ERROR", "panic",
        "validation", "wgpu adapter", "surface configured",
    )):
        print(line)

required = (
    "[DownsampleProbe]",
    "Render thread stopped",
    "All Lua workers were stopped",
)
has_error = re.search(r"\b(?:ERROR|panic)\b", text) is not None
if code != 0 or marker is None or has_error or not all(item in text for item in required):
    print(f"downsample_acceptance=false error={has_error}", flush=True)
    sys.exit(code if code else 2)

values = [float(item) for item in marker.groups()]
source, linear, nearest, upscaled = values[0:3], values[3:6], values[6:9], values[9:12]
print(
    f"downsample_samples source={tuple(source)} linear={tuple(linear)} "
    f"nearest={tuple(nearest)} upscaled={tuple(upscaled)}",
    flush=True,
)
linear_blended = (
    0.10 < linear[0] < 0.90
    and 0.10 < linear[1] < 0.90
    and abs(linear[0] + linear[1] - 1.0) < 0.12
)
nearest_binary = (
    (nearest[0] < 0.10 or nearest[0] > 0.90)
    and (nearest[1] < 0.10 or nearest[1] > 0.90)
    and abs(nearest[0] + nearest[1] - 1.0) < 0.12
)
source_known = (
    ((source[0] < 0.10 and source[1] > 0.90)
     or (source[0] > 0.90 and source[1] < 0.10))
    and source[2] < 0.08
)
upscale_matches = (
    abs(upscaled[0] - linear[0]) < 0.12
    and abs(upscaled[1] - linear[1]) < 0.12
    and upscaled[2] < 0.08
)
accepted = source_known and linear_blended and nearest_binary and upscale_matches
print(
    f"downsample_acceptance={accepted} source_known={source_known} "
    f"linear_blended={linear_blended} nearest_binary={nearest_binary} "
    f"upscale_matches={upscale_matches}",
    flush=True,
)
sys.exit(0 if accepted else 3)
