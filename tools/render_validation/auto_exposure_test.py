"""Auto-exposure convergence test (render API v2, S8).

    python tools/render_validation/auto_exposure_test.py [backend]

Runs the `AutoExposure` scene (RenderCoreSystem:tonemap with auto-exposure on over a black and then a mid-gray
frame, 300 frames at a fixed dt) and compares the exposure it converges to with `baseline/auto_exposure.json`,
which was recorded with the synchronous implementation before S8: the targets must match exactly (they sit on
the config clamps) and the adapted exposure within `current_tolerance` (the asynchronous version lags by a few
frames). Prints the main-thread cost of `tonemap` per frame. Exit code 0 means pass.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

backend = sys.argv[1] if len(sys.argv) > 1 else "gl"
workdir = Path(__file__).resolve().parents[2]
artifact_dir = Path(
    os.environ.get("LTHEORY_VALIDATION_ARTIFACT_DIR", workdir / "target" / "render_validation")
)
artifact_dir.mkdir(parents=True, exist_ok=True)
target_dir = Path(os.environ.get("CARGO_TARGET_DIR", workdir / "target"))
exe = Path(os.environ.get("LTHEORY_EXE", target_dir / "debug" / "ltr.exe"))
reference = json.loads((Path(__file__).resolve().parent / "baseline" / "auto_exposure.json").read_text())
log_path = artifact_dir / f"auto-exposure-{backend}.log"

env = os.environ.copy()
if backend == "wgpu":
    env["LTHEORY_WGPU"] = "1"
    # Fatal mode: the first wgpu validation error panics instead of being logged.
    env.setdefault("LTHEORY_WGPU_FATAL", "1")
else:
    env.pop("LTHEORY_WGPU", None)
with log_path.open("w", encoding="utf-8", newline="") as log:
    proc = subprocess.Popen(
        [str(exe), "-e", "./script/Main.lua", "AutoExposure"],
        cwd=workdir, env=env, stdout=log, stderr=subprocess.STDOUT,
        creationflags=getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0),
    )
    try:
        proc.wait(timeout=float(os.environ.get("AUTO_EXPOSURE_TIMEOUT_SECONDS", "90")))
    except subprocess.TimeoutExpired:
        subprocess.run(["taskkill.exe", "/PID", str(proc.pid), "/T", "/F"], check=False)
        print("timeout")
        sys.exit(124)

text = re.sub(r"\x1b\[[0-9;]*m", "", log_path.read_text(encoding="utf-8", errors="replace"))
m = re.search(
    r"RESULT dark_target=([\d.]+) dark_current=([\d.]+) bright_target=([\d.]+) bright_current=([\d.]+)", text
)
if not m:
    print(f"no RESULT line, see {log_path}")
    sys.exit(1)
got = dict(zip(["dark_target", "dark_current", "bright_target", "bright_current"], map(float, m.groups())))
ok = True
for key, value in got.items():
    tol = reference["target_tolerance" if key.endswith("target") else "current_tolerance"]
    good = abs(value - reference[key]) <= tol
    ok &= good
    print(f"{key:<15} {value:9.5f}  reference {reference[key]:9.5f}  tol {tol:g}  {'ok' if good else 'FAIL'}")
cost = re.search(r"tonemap_main_thread_ms=([\d.]+)", text)
if cost:
    print(f"tonemap main-thread cost: {float(cost.group(1)):.3f} ms/frame")
ok &= "[AutoExposure] PASS" in text
print("PASS" if ok else "FAIL")
sys.exit(0 if ok else 1)
