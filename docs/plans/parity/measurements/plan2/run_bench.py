#!/usr/bin/env python3
"""Plan2 bench driver: reference and candidate, same session.

Runs the three fixed cells at the plan's cap, three seeds for the candidate,
three repeats per seed, and checks every candidate proof with our checker and
the pinned provari oracle. Writes parsed results and env.json; no raw
transcripts.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import statistics
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[4]
CORPUS = REPO / "bench/corpus.json"
CORPUS_SHA256 = "9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e"
CELLS = {"core": "core", "easy-win": "1eb80b92f92c", "deep-win": "db6a741c723f"}


def args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--provari", required=True)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--corpus", default=str(CORPUS))
    parser.add_argument("--out", required=True)
    parser.add_argument("--seconds", type=float, default=120.0)
    parser.add_argument("--reference-seconds", type=float, default=10.0)
    parser.add_argument("--memory", type=int, default=192)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--seeds", default="0,1,2")
    return parser.parse_args()


def verify_corpus(path: Path) -> str:
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != CORPUS_SHA256:
        raise SystemExit(f"corpus sha256 mismatch: {digest} != {CORPUS_SHA256}")
    return digest


def load_cells(path: Path) -> list[dict]:
    data = json.loads(path.read_text())
    by_id = {data["core"]["id"]: data["core"]}
    by_id.update({cell["id"]: cell for cell in data["sample20"]})
    return [
        {"name": name, "id": cell_id, "fen": by_id[cell_id]["fen"]}
        for name, cell_id in CELLS.items()
    ]


def last_json(text: str) -> dict:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    return {}


def run(command: list[str], timeout: float) -> tuple[dict, float, int]:
    started = time.perf_counter()
    proc = subprocess.run(command, capture_output=True, text=True, timeout=timeout, check=False)
    wall = time.perf_counter() - started
    return last_json(proc.stdout + proc.stderr), wall, proc.returncode


def check(proof: Path, checker: list[str]) -> dict:
    proc = subprocess.run(checker + ["check", str(proof)], capture_output=True, text=True, check=False)
    return last_json(proc.stdout + proc.stderr)


def main() -> None:
    options = args()
    digest = verify_corpus(Path(options.corpus))
    cells = load_cells(Path(options.corpus))
    seeds = [int(seed) for seed in options.seeds.split(",") if seed]
    rows = []
    for cell in cells:
        for run_no in range(1, options.repeats + 1):
            payload, wall, code = run([
                options.provari, "prove", cell["fen"],
                "--seconds", str(int(options.reference_seconds)),
                "--memory", str(options.memory), "--guide", "hand", "--json",
            ], options.reference_seconds + 60)
            rows.append({
                "side": "provari-hand", "cell": cell["name"], "id": cell["id"],
                "seed": None, "run": run_no, "wall_s": round(wall, 3),
                "returncode": code, "win": bool(payload.get("found")),
                "mate": payload.get("mate"), "nodes": payload.get("nodes"),
                "child_evals": None, "expansions": None,
                "verdict": "accepted" if payload.get("found") else "censored",
            })
        for seed in seeds:
            for run_no in range(1, options.repeats + 1):
                with tempfile.TemporaryDirectory() as tmp:
                    proof = Path(tmp) / f"{cell['name']}-{seed}-{run_no}.pvr"
                    payload, wall, code = run([
                        options.candidate, "prove", cell["fen"],
                        "--seconds", str(int(options.seconds)),
                        "--memory", str(options.memory), "--guide", "hand",
                        "--seed", str(seed), "--proofs", str(proof), "--json",
                    ], options.seconds + 60)
                    verdict = "censored"
                    if payload.get("found"):
                        ours = check(proof, [options.candidate])
                        theirs = check(proof, [options.provari])
                        if ours.get("valid") is True and theirs.get("valid") is True:
                            verdict = "accepted"
                        elif ours.get("valid") is False or theirs.get("valid") is False:
                            verdict = "checker-reject"
                        else:
                            verdict = "unchecked"
                    rows.append({
                        "side": "candidate-hand", "cell": cell["name"], "id": cell["id"],
                        "seed": seed, "run": run_no, "wall_s": round(wall, 3),
                        "returncode": code, "win": bool(payload.get("found")),
                        "mate": payload.get("mate"), "nodes": payload.get("nodes"),
                        "child_evals": payload.get("child_evals"),
                        "expansions": payload.get("expansions"),
                        "verdict": verdict,
                    })
    summary = {}
    for cell in cells:
        ref = [row for row in rows if row["cell"] == cell["name"] and row["side"] == "provari-hand"]
        cand = [row for row in rows if row["cell"] == cell["name"] and row["side"] == "candidate-hand"]
        ref_wall = statistics.median(row["wall_s"] for row in ref)
        cand_wall = statistics.median(row["wall_s"] for row in cand)
        accepted = [row for row in cand if row["verdict"] == "accepted"]
        summary[cell["name"]] = {
            "id": cell["id"],
            "reference_wall_s": round(ref_wall, 3),
            "candidate_wall_s": round(cand_wall, 3),
            "factor": round(cand_wall / ref_wall, 3) if accepted else None,
            "reference_mate": next((row["mate"] for row in ref if row["win"]), None),
            "candidate_mate": next((row["mate"] for row in accepted), None),
            "candidate_child_evals": next((row["child_evals"] for row in accepted), None),
            "candidate_expansions": next((row["expansions"] for row in accepted), None),
            "accepted": len(accepted),
            "runs": len(cand),
            "verdicts": sorted({row["verdict"] for row in cand}),
        }
    factors = [cell["factor"] for cell in summary.values() if cell["factor"] is not None]
    geomean = math.exp(sum(math.log(f) for f in factors) / len(factors)) if factors else None
    out = Path(options.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps({
        "side": "candidate-hand vs provari-hand",
        "corpus_sha256": digest,
        "seconds": options.seconds,
        "reference_seconds": options.reference_seconds,
        "memory_mb": options.memory,
        "repeats": options.repeats,
        "seeds": seeds,
        "rows": rows,
        "summary": summary,
        "geomean": round(geomean, 3) if geomean is not None else None,
    }, indent=2) + "\n")
    env = {
        "date": datetime.now(timezone.utc).isoformat(),
        "host": platform.node(),
        "platform": platform.platform(),
        "python": platform.python_version(),
        "provari": options.provari,
        "provari_version": subprocess.run([options.provari, "--version"], capture_output=True, text=True, check=False).stdout.strip(),
        "candidate": options.candidate,
        "candidate_version": subprocess.run([options.candidate, "--version"], capture_output=True, text=True, check=False).stdout.strip(),
        "corpus_sha256": digest,
        "seconds": options.seconds,
        "reference_seconds": options.reference_seconds,
        "memory_mb": options.memory,
        "repeats": options.repeats,
        "seeds": seeds,
        "cwd": os.getcwd(),
    }
    (out.parent / "env.json").write_text(json.dumps(env, indent=2) + "\n")
    print(json.dumps({"summary": summary, "geomean": geomean}, indent=2))


if __name__ == "__main__":
    main()
