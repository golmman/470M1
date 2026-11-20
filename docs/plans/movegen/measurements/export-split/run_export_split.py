#!/usr/bin/env python3
"""One counted split of the proof-export walk on `core`.

Measures, one process at a time and interleaved by round, the export delta
(with `--proofs` minus without) in user instructions for three binaries:
the plan 12 product, the plan 2 product (shakmaty) and the pinned provari
oracle. Also measures the oracle with `SKIP_WALK` set, which is the pinned
source plus a short-circuit in `format::walk`, so the oracle's own walk has a
number: without `--proofs` it still walks, so its with/without delta is only
the file write.

The instrumented counter dump and the byte-identical child-reuse variant are
built from scratch copies with `instrument.patch` and `reuse.patch`; see the
README. This driver only reproduces `clean_instructions` in `results.json`.

Usage:
    python3 run_export_split.py \
        --candidate /workspace/470M1/target/release/470MI \
        --plan2 /tmp/plan12/plan2/target/release/470m1 \
        --provari /tmp/470M1-oracle/provari/target/release/provari \
        --provari-skip /tmp/expsplit/provari-clean/target/release/provari \
        --out raw.json
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import statistics
import subprocess
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True

FEN = "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1"


def args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--plan2", required=True)
    parser.add_argument("--provari", required=True)
    parser.add_argument("--provari-skip", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--rounds", type=int, default=3)
    return parser.parse_args()


def once(binary: str, proofs: bool, provari: bool, skip: bool = False) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        command = [
            binary, "prove", FEN,
            "--seconds", "10", "--memory", "192", "--json",
        ]
        if provari:
            command += ["--guide", "hand"]
        else:
            command += ["--seed", "0"]
        proof = Path(tmp) / "p.pvr"
        if proofs:
            command += ["--proofs", str(proof)]
        env = dict(os.environ)
        if skip:
            env["SKIP_WALK"] = "1"
        measured = ["perf", "stat", "-e", "instructions:u", "-x,", "--", *command]
        proc = subprocess.run(measured, capture_output=True, text=True, env=env)
        instructions = next(
            int(line.split(",")[0])
            for line in proc.stderr.splitlines()
            if line.split(",")[2:3] == ["instructions:u"]
        )
        payload = json.loads(proc.stdout.strip().splitlines()[-1])
        digest = hashlib.sha256(proof.read_bytes()).hexdigest() if proofs else None
    return {
        "instructions": instructions,
        "mate": payload.get("mate"),
        "nodes": payload.get("nodes"),
        "child_evals": payload.get("child_evals"),
        "proof_moves": payload.get("proof_moves"),
        "proof_sha256": digest,
    }


def arm(binary: str, provari: bool, skip: bool, rounds: int) -> dict:
    bare = [once(binary, False, provari, skip) for _ in range(rounds)]
    proof = [once(binary, True, provari, skip) for _ in range(rounds)]
    without = int(statistics.median(row["instructions"] for row in bare))
    with_proof = int(statistics.median(row["instructions"] for row in proof))
    return {
        "without": [row["instructions"] for row in bare],
        "with": [row["instructions"] for row in proof],
        "median_without": without,
        "median_with": with_proof,
        "delta": with_proof - without,
        "shape": proof[0],
    }


def main() -> None:
    options = args()
    rounds = options.rounds
    out = {
        "cell": "core",
        "fen": FEN,
        "candidate": arm(options.candidate, False, False, rounds),
        "plan2": arm(options.plan2, False, False, rounds),
        "provari": arm(options.provari, True, False, rounds),
        "provari_full_walk": arm(options.provari_skip, True, False, rounds),
        "provari_skip_walk": arm(options.provari_skip, True, True, rounds),
        "env": {
            "host": platform.node(),
            "platform": platform.platform(),
            "machine": platform.machine(),
            "commands": {
                "candidate": "470MI prove FEN --seconds 10 --memory 192 --seed 0 [--proofs P] --json",
                "provari": "provari prove FEN --seconds 10 --memory 192 --guide hand [--proofs P] --json",
            },
            "perf": "perf stat -e instructions:u -x,",
        },
    }
    # The oracle's own walk: the same SKIP_WALK binary, full walk minus skip.
    full = out["provari_full_walk"]["median_with"]
    skip = out["provari_skip_walk"]["median_with"]
    out["provari_walk"] = {"full": full, "skip": skip, "walk": full - skip}
    out["ratios"] = {
        "candidate_over_plan2_walk": round(
            out["candidate"]["delta"] / out["plan2"]["delta"], 3
        ),
        "candidate_over_provari_walk": round(
            out["candidate"]["delta"] / out["provari_walk"]["walk"], 3
        ),
    }
    Path(options.out).write_text(json.dumps(out, indent=2) + "\n")
    print(json.dumps({
        "candidate_delta": out["candidate"]["delta"],
        "plan2_delta": out["plan2"]["delta"],
        "provari_without_proof_delta": out["provari"]["delta"],
        "provari_walk": out["provari_walk"]["walk"],
        "ratios": out["ratios"],
    }, indent=2))


if __name__ == "__main__":
    main()
