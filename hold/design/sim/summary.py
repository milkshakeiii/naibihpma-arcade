#!/usr/bin/env python3
"""Ten seeds of econ.py's first-scenario start, as medians and ranges."""
import statistics
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import econ  # noqa: E402

res = []
for s in range(1, 11):
    econ.BUILD_ORDER = list(econ.FULL_ORDER)
    w = econ.World(s, scenario=True)
    res.append((w, w.run()))
ms = [m for m in econ.MILESTONES if m not in ("Burrow built", "Chorus Log built", "Toad Hall built", "first toad")]
ms += ["first iron shovel made", "first peytral made", "first star pearl made"]
for m in ms:
    ts = sorted(w.first[m] for w, _ in res if m in w.first)
    if not ts:
        print(f"{m:32s} never")
        continue
    print(f"{m:32s} {econ.clock(int(statistics.median(ts)))}   {econ.clock(ts[0])} to {econ.clock(ts[-1])}   {len(ts)}/10")
inc = [statistics.mean(w.hall_income[m] for w, _ in res) for m in range(45)]
print("dew into the Hall per minute, by 5-minute block:", [round(sum(inc[i:i + 5]) / 5) for i in range(0, 45, 5)])
print("dew held in the Hall every 5 minutes (median):", [int(statistics.median(sn[i]["dew"] for _, sn in res)) for i in range(0, 45, 5)])
idle = {}
for w, _ in res:
    for k in w.idle:
        idle.setdefault(k, []).append(w.idle[k] / (w.idle[k] + w.busy[k]))
print("share of time idle:", {k: f"{statistics.mean(v):.0%}" for k, v in idle.items()})
