#!/usr/bin/env python3
"""Plan 3 gate 4 instruments: cachegrind `Ir` and startup walls, base vs cand.

`Ir` is valgrind cachegrind with `--cache-sim=no`, one run per binary and
cell, no `--seconds` (a ~38x slowdown would hit a 10 s cap). `core` is the
gate (cand `<= +0.5%` of base); mate-in-1 is a diagnostic. Startup is the
median of 20 interleaved walls of the mate-in-1 prove, a diagnostic only.
Raw cachegrind files stay under /tmp; this writes `cachegrind.json` and
`startup.json` beside itself.
"""
import json
import re
import statistics
import subprocess
import sys
import time
from pathlib import Path

CORE = "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1"
M1 = "6rk/8/8/8/8/8/8/6QK w - - 0 1"
CELLS = {"core": CORE, "m1_explosion": M1}
COUNTERS = ("found", "mate", "nodes", "child_evals", "expansions")
RAW = Path("/tmp/470M1-plan3-cachegrind")
HERE = Path(__file__).resolve().parent


def prove(binary: str, fen: str) -> list[str]:
    return [binary, "prove", fen, "--memory", "192", "--guide", "hand",
            "--seed", "0", "--json"]


def last_json(text: str) -> dict:
    for line in reversed(text.splitlines()):
        try:
            return json.loads(line)
        except json.JSONDecodeError:
            continue
    return {}


def cachegrind(side: str, binary: str, cell: str) -> dict:
    out = RAW / f"{side}-{cell}.out"
    proc = subprocess.run(
        ["valgrind", "--tool=cachegrind", "--cache-sim=no",
         f"--cachegrind-out-file={out}", *prove(binary, CELLS[cell])],
        capture_output=True, text=True, check=True)
    refs = re.search(r"I\s+refs:\s+([\d,]+)", proc.stderr)
    if refs is None:
        sys.exit(f"no I refs for {side} {cell}:\n{proc.stderr}")
    payload = last_json(proc.stdout)
    return {"side": side, "cell": cell, "Ir": int(refs.group(1).replace(",", "")),
            **{key: payload.get(key) for key in COUNTERS}}


def main() -> None:
    base, cand = sys.argv[1], sys.argv[2]
    RAW.mkdir(parents=True, exist_ok=True)
    runs = [cachegrind(side, binary, cell)
            for cell in CELLS for side, binary in (("base", base), ("cand", cand))]
    by = {(r["side"], r["cell"]): r for r in runs}
    delta = {cell: by[("cand", cell)]["Ir"] / by[("base", cell)]["Ir"] - 1
             for cell in CELLS}
    same = {cell: all(by[("base", cell)][k] == by[("cand", cell)][k] for k in COUNTERS)
            for cell in CELLS}
    (HERE / "cachegrind.json").write_text(json.dumps({
        "runs": runs, "delta": delta, "counters_identical": same,
        "gate_core_le_0.005": delta["core"] <= 0.005}, indent=2) + "\n")

    walls: dict[str, list[float]] = {"base": [], "cand": []}
    for _ in range(20):
        for side, binary in (("base", base), ("cand", cand)):
            started = time.perf_counter()
            subprocess.run(prove(binary, M1), capture_output=True, check=True)
            walls[side].append(round(time.perf_counter() - started, 5))
    medians = {side: statistics.median(w) for side, w in walls.items()}
    (HERE / "startup.json").write_text(json.dumps({
        "cell": "m1_explosion", "walls_s": walls, "median_s": medians,
        "ratio": medians["cand"] / medians["base"]}, indent=2) + "\n")
    print(json.dumps({"delta": delta, "counters_identical": same,
                      "startup_median_s": medians}, indent=2))


if __name__ == "__main__":
    main()
