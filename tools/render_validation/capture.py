"""Deterministic screenshot capture of real scenes.

    python tools/render_validation/capture.py <backend> [scene...]
    python tools/render_validation/capture.py --baseline [scene...]

Writes target/render_validation/captures/<backend>/<scene>.png and .json
(stats parsed from the CAPTURE log line printed by Application:captureTick).
--baseline copies the gl captures into tools/render_validation/baseline/.
"""
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

SCENES = ["PlanetTest", "Benchmark", "SolarSystemPlayable", "MoonTest", "PlanetTestRing", "WeaponSystem"]
# Capture name -> (state to launch, extra env). The scenes read these variables only under LTHEORY_CAPTURE.
VARIANTS = {
    # PlanetTest with a fixed seed that rolls a planet ring (seed 27: ring chance is 20%).
    "PlanetTestRing": ("PlanetTest", {"LTHEORY_CAPTURE_SEED": "27"}),
    "WeaponSystem": ("Testbeds/WeaponSystem", {}),
}
FRAME = os.environ.get("LTHEORY_CAPTURE_FRAME", "120")
TIMEOUT = float(os.environ.get("CAPTURE_TIMEOUT_SECONDS", "180"))

workdir = Path(__file__).resolve().parents[2]
artifact_dir = Path(
    os.environ.get("LTHEORY_VALIDATION_ARTIFACT_DIR", workdir / "target" / "render_validation")
)
baseline_dir = Path(__file__).resolve().parent / "baseline"
target_dir = Path(os.environ.get("CARGO_TARGET_DIR", workdir / "target"))
exe = Path(os.environ.get("LTHEORY_EXE", target_dir / "debug" / "ltr.exe"))
STATS_RE = re.compile(r"CAPTURE (frame=.*)")


def capture(backend, scene):
    out_dir = artifact_dir / "captures" / backend
    out_dir.mkdir(parents=True, exist_ok=True)
    png = out_dir / f"{scene}.png"
    log_path = out_dir / f"{scene}.log"
    png.unlink(missing_ok=True)
    env = os.environ.copy()
    env["LTHEORY_CAPTURE"] = str(png)
    env["LTHEORY_CAPTURE_FRAME"] = FRAME
    state, extra_env = VARIANTS.get(scene, (scene, {}))
    env.update(extra_env)
    if backend == "wgpu":
        env["LTHEORY_WGPU"] = "1"
        # Fatal mode: the first wgpu validation error panics instead of being logged.
        env.setdefault("LTHEORY_WGPU_FATAL", "1")
    else:
        env.pop("LTHEORY_WGPU", None)
    with log_path.open("w", encoding="utf-8", newline="") as log:
        proc = subprocess.Popen(
            [str(exe), "-e", "./script/Main.lua", state],
            cwd=workdir, env=env, stdout=log, stderr=subprocess.STDOUT,
            creationflags=getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0),
        )
        try:
            proc.wait(timeout=TIMEOUT)
        except subprocess.TimeoutExpired:
            subprocess.run(["taskkill.exe", "/PID", str(proc.pid), "/T", "/F"], check=False)
            proc.wait(timeout=5.0)
            print(f"{scene}: TIMEOUT after {TIMEOUT}s (log {log_path})")
            return False
    text = re.sub(r"\x1b\[[0-9;]*m", "", log_path.read_text(encoding="utf-8", errors="replace"))
    m = STATS_RE.search(text)
    if not png.exists() or not m:
        print(f"{scene}: FAILED (no capture, log {log_path})")
        return False
    stats = {}
    for kv in m.group(1).split():
        k, _, v = kv.partition("=")
        if k in ("path", "bound"):
            stats[k] = v if k == "bound" else None
        else:
            stats[k] = float(v) if "." in v else int(v)
    stats.pop("path", None)
    stats.update(scene=scene, backend=backend)
    (out_dir / f"{scene}.json").write_text(json.dumps(stats, indent=2))
    print(f"{scene}: ok fps={stats['fps']} frametime_ms={stats['frametime_ms']} "
          f"draw_calls={stats['draw_calls']} vertices={stats['vertices']} "
          f"render_idle={stats['render_idle_pct']}% bound={stats['bound']}")
    return True


def main():
    args = sys.argv[1:]
    if args[:1] == ["--baseline"]:
        src = artifact_dir / "captures" / "gl"
        baseline_dir.mkdir(parents=True, exist_ok=True)
        only = set(args[1:])  # optional scene names: bless just these
        for f in sorted(src.glob("*.png")) + sorted(src.glob("*.json")):
            if only and f.stem not in only:
                continue
            shutil.copy2(f, baseline_dir / f.name)
            print(f"baseline <- {f.name}")
        return 0
    if not args:
        print(__doc__)
        return 2
    backend, scenes = args[0], args[1:] or SCENES
    ok = [capture(backend, s) for s in scenes]
    return 0 if all(ok) else 1


if __name__ == "__main__":
    sys.exit(main())
