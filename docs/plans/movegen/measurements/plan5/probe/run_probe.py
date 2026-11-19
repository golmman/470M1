#!/usr/bin/env python3
"""Run the per-call probe: three `perf stat -e instructions` runs per op.

Writes `instructions / ops` per run and the median, the shape plan 4 reported.
No raw transcripts; `target/` and the binary stay untracked.
"""

from __future__ import annotations

import json
import statistics
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
BINARY = HERE / "target/release/hotpath-probe"
OPS = [
    "state", "counts", "populate_counts", "legal_full", "legal_context",
    "effect_old", "effect_child", "null", "native_roundtrip",
]
REPEATS = 2000
RUNS = 3


def main() -> None:
    results = {}
    for op in OPS:
        out = subprocess.run([BINARY, op, str(REPEATS)], capture_output=True,
                             text=True, check=True).stdout
        ops = int(out.split("ops=")[1].split()[0])
        per_call = []
        for _ in range(RUNS):
            stat = subprocess.run(
                ["perf", "stat", "-e", "instructions", "-x,", BINARY, op,
                 str(REPEATS)],
                capture_output=True, text=True, check=True).stderr
            instructions = next(
                int(line.split(",")[0]) for line in stat.splitlines()
                if line.split(",")[2:3] == ["instructions:u"])
            per_call.append(instructions / ops)
        results[op] = {
            "ops": ops,
            "instructions_per_call": round(statistics.median(per_call), 1),
            "runs": per_call,
        }
        print(op, results[op]["instructions_per_call"], flush=True)
    (HERE / "results.json").write_text(json.dumps(results, indent=2) + "\n")


if __name__ == "__main__":
    main()
