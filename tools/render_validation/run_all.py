"""Run all validation: 13 supervisors + captures + baseline compare.

    python tools/render_validation/run_all.py <backend>
"""
import subprocess
import sys
from pathlib import Path

here = Path(__file__).resolve().parent
backend = sys.argv[1] if len(sys.argv) > 1 else "gl"


def run(*args):
    return subprocess.run([sys.executable, *map(str, args)], capture_output=True, text=True)


results = []
for sup in sorted(here.glob("*_supervisor.py")):
    r = run(sup, backend)
    results.append((sup.stem.replace("_supervisor", ""), r.returncode == 0))
    print(f"supervisor {results[-1][0]:<22}{'PASS' if r.returncode == 0 else 'FAIL'}", flush=True)

cap = run(here / "capture.py", backend)
print(cap.stdout.rstrip())
cmp = run(here / "compare.py", backend)
print(cmp.stdout.rstrip(), cmp.stderr.rstrip())

passed = sum(ok for _, ok in results)
print(f"\nSUMMARY backend={backend}: supervisors {passed}/{len(results)}, "
      f"capture {'ok' if cap.returncode == 0 else 'FAIL'}, "
      f"compare {'ok' if cmp.returncode == 0 else 'FAIL'}")
sys.exit(0 if passed == len(results) and cap.returncode == 0 and cmp.returncode == 0 else 1)
