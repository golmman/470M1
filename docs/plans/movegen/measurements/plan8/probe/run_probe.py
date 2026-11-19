#!/usr/bin/env python3
"""Plan 8 per-call screen: 3 perf stat runs per op, instructions/call."""
import json, statistics, subprocess
from pathlib import Path
HERE = Path(__file__).resolve().parent
BIN = HERE / "target/release/plan8-probe"
OPS = ["v0", "vb", "v1", "v3"]
REPEATS = 2000
out = {}
for op in OPS:
    text = subprocess.run([BIN, op, str(REPEATS)], capture_output=True, text=True, check=True).stdout
    ops = int(text.split("ops=")[1].split()[0])
    per = []
    for _ in range(3):
        stat = subprocess.run(["perf","stat","-e","instructions:u","-x,",BIN,op,str(REPEATS)], capture_output=True, text=True, check=True).stderr
        inst = next(int(line.split(",")[0]) for line in stat.splitlines() if line.split(",")[2:3] == ["instructions:u"])
        per.append(inst/ops)
    out[op] = {"ops": ops, "instructions_per_call": round(statistics.median(per),1), "runs": per}
    print(op, out[op]["instructions_per_call"], flush=True)
(HERE/"results.json").write_text(json.dumps(out, indent=2)+"\n")
