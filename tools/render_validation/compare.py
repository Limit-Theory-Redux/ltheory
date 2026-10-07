"""Diff captures against tools/render_validation/baseline.

    python tools/render_validation/compare.py <backend> [scene...]
        [--max-rmse 2.0] [--pixel-threshold 8] [--max-bad-pct 1.0]

Per image: RMSE (0-255, RGB), max abs diff, % of pixels whose max channel diff
exceeds --pixel-threshold. Exit 1 if RMSE > --max-rmse or bad pixels >
--max-bad-pct, or an image is missing/mismatched. Diff heatmaps are written to
target/render_validation/captures/<backend>/diff/<scene>.png. Needs Pillow.
"""
import argparse
import math
import os
import sys
from pathlib import Path

from PIL import Image, ImageChops

workdir = Path(__file__).resolve().parents[2]
artifact_dir = Path(
    os.environ.get("LTHEORY_VALIDATION_ARTIFACT_DIR", workdir / "target" / "render_validation")
)
baseline_dir = Path(__file__).resolve().parent / "baseline"
sys.path.insert(0, str(Path(__file__).resolve().parent))
from capture import SCENES, outputs  # noqa: E402  (scene list and multi-frame names)


def heat(v):
    # black -> blue -> red -> yellow as the diff grows (saturates at 64)
    t = min(v / 64.0, 1.0)
    return (int(255 * min(t * 2, 1)), int(255 * max(t * 2 - 1, 0)), int(255 * (1 - t) * t * 2))


def diff_one(ref_path, cur_path, heat_path, px_thr):
    ref = Image.open(ref_path).convert("RGB")
    cur = Image.open(cur_path).convert("RGB")
    if ref.size != cur.size:
        return None
    px = list(ImageChops.difference(ref, cur).getdata())
    n = len(px)
    per_px = [max(p) for p in px]
    sq = sum(v * v for p in px for v in p) / (3 * n)
    lut = [heat(v) for v in range(256)]
    h = Image.new("RGB", ref.size)
    h.putdata([lut[v] for v in per_px])
    heat_path.parent.mkdir(parents=True, exist_ok=True)
    h.save(heat_path)
    return math.sqrt(sq), max(per_px), 100.0 * sum(v > px_thr for v in per_px) / n


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("backend")
    ap.add_argument("scenes", nargs="*")
    ap.add_argument("--max-rmse", type=float, default=2.0)
    ap.add_argument("--pixel-threshold", type=int, default=8)
    ap.add_argument("--max-bad-pct", type=float, default=1.0)
    a = ap.parse_args()
    cap_dir = artifact_dir / "captures" / a.backend
    print(f"{'scene':<22}{'rmse':>8}{'max':>6}{'%>thr':>9}  result")
    failed = False
    for scene in [n for s in (a.scenes or SCENES) for n in outputs(s)]:
        ref, cur = baseline_dir / f"{scene}.png", cap_dir / f"{scene}.png"
        if not ref.exists() or not cur.exists():
            print(f"{scene:<22}{'-':>8}{'-':>6}{'-':>9}  FAIL (missing {'baseline' if not ref.exists() else 'capture'})")
            failed = True
            continue
        r = diff_one(ref, cur, cap_dir / "diff" / f"{scene}.png", a.pixel_threshold)
        if r is None:
            print(f"{scene:<22}{'-':>8}{'-':>6}{'-':>9}  FAIL (size mismatch)")
            failed = True
            continue
        rmse, mx, bad = r
        ok = rmse <= a.max_rmse and bad <= a.max_bad_pct
        failed |= not ok
        print(f"{scene:<22}{rmse:>8.3f}{mx:>6}{bad:>9.3f}  {'ok' if ok else 'FAIL'}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
