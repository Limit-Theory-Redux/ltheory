"""Hot-reload probe: edit a material shader while PlanetTest runs.

    python tools/render_validation/hot_reload_probe.py [gl]

Four captures of PlanetTest at the same frame (the engine uses a fixed time
step, so equal scenes give equal pixels). While the later ones run, the script
edits `res/shader/fragment/material/planet.glsl` after the scene is up:

  C  no edit                          the reference
  A  albedo tinted red                differs from C, only where the planet is
  B  tinted, then reverted            identical to C
  D  new first member of the block    identical to C: every offset of
     `MaterialParams` (shifts all      `MaterialParams` moved by 16 bytes, so the
     others by 16 bytes)               parameters had to be copied by name into
                                       the regenerated type

and the logs must show the reloads and no `ERROR`/panic lines (the
GL_INVALID_OPERATION of the first pass is old and ignored). The shader file is
restored at the end, also on failure. Needs Pillow. Override the binary with
`LTHEORY_EXE`. Exit code 0 means pass.
"""
import math
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

from PIL import Image, ImageChops

workdir = Path(__file__).resolve().parents[2]
artifact_dir = Path(os.environ.get("LTHEORY_VALIDATION_ARTIFACT_DIR", workdir / "target" / "render_validation")) / "hot_reload"
target_dir = Path(os.environ.get("CARGO_TARGET_DIR", workdir / "target"))
exe = Path(os.environ.get("LTHEORY_EXE", target_dir / "debug" / "ltr.exe"))
shader = workdir / "res" / "shader" / "fragment" / "material" / "planet.glsl"
FRAME = os.environ.get("HOT_RELOAD_FRAME", "9000")  # ~20 s: after every edit below
ANSI = re.compile(r"\x1b\[[0-9;]*m")


def edit(text, mode):
    nl = "\r\n" if "\r\n" in text else "\n"
    if mode in ("A", "B"):
        return text.replace("setAlbedo(color);", "setAlbedo(color * vec3(1.0, 0.15, 0.15));")
    if mode == "D":
        text = text.replace(
            "layout(std140) uniform MaterialParams {",
            "layout(std140) uniform MaterialParams {" + nl + "  vec4 hotProbe;",
        )
        return text.replace("setAlbedo(color);", "setAlbedo(color + hotProbe.xyz);")
    return text


def capture(mode, original):
    png = artifact_dir / f"{mode}.png"
    log_path = artifact_dir / f"{mode}.log"
    png.unlink(missing_ok=True)
    env = os.environ.copy()
    env.update(LTHEORY_CAPTURE=str(png), LTHEORY_CAPTURE_FRAME=FRAME)
    env.pop("LTHEORY_WGPU", None)
    env["LTHEORY_GL_CHECK"] = "1"
    with log_path.open("w", encoding="utf-8", newline="") as log:
        proc = subprocess.Popen([str(exe), "-e", "./script/Main.lua", "PlanetTest"],
                                cwd=workdir, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            time.sleep(8)  # the scene is running
            if mode != "C":
                shader.write_bytes(edit(original, mode).encode("utf-8"))
            if mode == "B":
                time.sleep(4)
                shader.write_bytes(original.encode("utf-8"))
            proc.wait(timeout=180)
        finally:
            shader.write_bytes(original.encode("utf-8"))
            if proc.poll() is None:
                subprocess.run(["taskkill.exe", "/PID", str(proc.pid), "/T", "/F"], check=False)
    text = ANSI.sub("", log_path.read_text(encoding="utf-8", errors="replace"))
    return png, text


def diff(a, b):
    d = ImageChops.difference(Image.open(a).convert("RGB"), Image.open(b).convert("RGB"))
    hist = d.histogram()
    n = d.size[0] * d.size[1] * 3
    rmse = math.sqrt(sum((i % 256) ** 2 * c for i, c in enumerate(hist)) / n)
    return rmse, d.getbbox()


def main():
    artifact_dir.mkdir(parents=True, exist_ok=True)
    original = shader.read_bytes().decode("utf-8")
    ok = True
    results = {}
    try:
        for mode in ("C", "A", "B", "D"):
            results[mode] = capture(mode, original)
    finally:
        shader.write_bytes(original.encode("utf-8"))
    ref = results["C"][0]
    for mode in ("A", "B", "D"):
        png, text = results[mode]
        reloads = len(re.findall(r"Reloaded shader", text))
        errors = [l for l in text.splitlines()
                  if re.search(r"\bERROR\b|panicked", l) and "commands before this PassCommands" not in l
                  and "Failed to signal main thread" not in l]
        rmse, bbox = diff(ref, png)
        expect_same = mode != "A"
        good = (rmse == 0.0) if expect_same else (rmse > 0.5)
        good = good and reloads >= (2 if mode == "B" else 1) and not errors
        ok &= good
        print(f"{mode}: reloads={reloads} rmse_vs_unedited={rmse:.4f} bbox={bbox} errors={len(errors)} "
              f"{'ok' if good else 'FAIL'}")
        for l in errors[:5]:
            print("   ", l[:200])
    layout = re.search(r"MaterialParams layout changed, ctype regenerated", results["D"][1])
    print(f"D: ctype regenerated: {'yes' if layout else 'NO'}")
    ok &= layout is not None
    print(f"\nSUMMARY hot reload: {'ok' if ok else 'FAIL'}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
