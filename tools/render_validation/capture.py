"""Deterministic screenshot capture of real scenes.

    python tools/render_validation/capture.py <backend> [scene...]
    python tools/render_validation/capture.py --baseline [scene...]

Writes target/render_validation/captures/<backend>/<scene>.png and .json
(stats parsed from the CAPTURE log line printed by Application:captureTick).
--baseline copies the gl captures into tools/render_validation/baseline/.

Every capture runs a full Lua GC halfway to its (first) capture frame
(LTHEORY_CAPTURE_GC_FRAME): a GPU resource whose only strong reference was a
collected Lua object then shows up in the image, and the scene fails if the
log reports a texture that was bound after it was destroyed.
"""
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

SCENES = ["PlanetTest", "Benchmark", "SolarSystemPlayable", "MoonTest", "PlanetTestRing", "WeaponSystem",
          "BenchmarkPhases", "Benchmark1080"]
# Capture name -> (state to launch, extra env). The scenes read these variables only under LTHEORY_CAPTURE.
VARIANTS = {
    # PlanetTest with a fixed seed that rolls a planet ring (seed 27: ring chance is 20%).
    "PlanetTestRing": ("PlanetTest", {"LTHEORY_CAPTURE_SEED": "27"}),
    "WeaponSystem": ("Testbeds/WeaponSystem", {}),
    # Benchmark's camera phases in one run (fixed 60 Hz dt): ring fly-through
    # (600), asteroid zoom (1000), asteroid close-up (1300), return (1600),
    # moon zoom (1850), moon close-up (2100).
    "BenchmarkPhases": ("Benchmark", {
        "LTHEORY_CAPTURE_FRAME": "2100",
        "LTHEORY_CAPTURE_EXTRA_FRAMES": "600,1000,1300,1600,1850",
    }),
    # Benchmark at its real (non-capture) resolution.
    "Benchmark1080": ("Benchmark", {"LTHEORY_CAPTURE_SIZE": "1920x1080"}),
}
FRAME = os.environ.get("LTHEORY_CAPTURE_FRAME", "120")
TIMEOUT = float(os.environ.get("CAPTURE_TIMEOUT_SECONDS", "300"))
# A texture bound after its GPU object was destroyed (GL and wgpu word it the same).
MISSING_TEXTURE_RE = re.compile(r"texture ResourceId\(\d+\) not found")


def outputs(scene):
    """Image names a capture writes: <scene>_f<n> per extra frame, then the scene."""
    extra = VARIANTS.get(scene, (scene, {}))[1].get("LTHEORY_CAPTURE_EXTRA_FRAMES", "")
    return [f"{scene}_f{f}" for f in extra.split(",") if f] + [scene]

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
    for name in outputs(scene):
        (out_dir / f"{name}.png").unlink(missing_ok=True)
    env = os.environ.copy()
    env["LTHEORY_CAPTURE"] = str(png)
    env["LTHEORY_CAPTURE_FRAME"] = FRAME
    state, extra_env = VARIANTS.get(scene, (scene, {}))
    env.update(extra_env)
    # Full GC halfway to the first captured frame.
    frames = [env["LTHEORY_CAPTURE_FRAME"], *env.get("LTHEORY_CAPTURE_EXTRA_FRAMES", "").split(",")]
    env.setdefault("LTHEORY_CAPTURE_GC_FRAME", str(max(1, min(int(f) for f in frames if f) // 2)))
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
    missing = [n for n in outputs(scene) if not (out_dir / f"{n}.png").exists()]
    if missing or not m:
        print(f"{scene}: FAILED (no capture {missing}, log {log_path})")
        return False
    lost = sorted(set(MISSING_TEXTURE_RE.findall(text)))
    if lost:
        print(f"{scene}: FAILED (bound after destruction: {', '.join(lost)}; log {log_path})")
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
        names = {n for s in only for n in outputs(s)}
        for f in sorted(src.glob("*.png")) + sorted(src.glob("*.json")):
            if only and f.stem not in names:
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
