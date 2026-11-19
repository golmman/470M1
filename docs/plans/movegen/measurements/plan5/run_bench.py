#!/usr/bin/env python3
"""Movegen plan5 driver: reference and candidate, same session, interleaved.

Plan 5's bar is the same-session plan 2 binary, so the driver is plan 3's
(copied from plan2's), unchanged apart from this header. Plan 5 measures the
adopted 2.10.0 tree and the plan 2 reference (`target/release/470m1`, built
from `2f629fd`, never committed) with the same interleaved pass: the adopted
prover at the product default (seed 0) against the same-session provari hand
reference. Every cell runs for the chosen protocol in one interleaved pass
over the same session, so that warm-up and drift hit both sides alike:

  a  candidate writes the proof, reference does not  (bench.md protocol)
  b  both write a proof                             (symmetric end to end)
  c  neither writes a proof                         (search wall only)

Each variant is measured with the repeats given on the command line; the
candidate side runs the seeds given there, the reference has no seed. Every
candidate proof is checked by our checker and by the pinned provari oracle.
`--d14` also runs the untimed proof export for every claimed win and compares
its counters with the timed run.

Writes parsed results and env.json; no raw transcripts.

Size note: one measurement driver, kept whole; the three protocol variants
share one interleaved pass so that a warm-up effect cannot land on one side.
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
    parser.add_argument("--seconds", type=float, default=10.0)
    parser.add_argument("--reference-seconds", type=float, default=10.0)
    parser.add_argument("--memory", type=int, default=192)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--seeds", default="0,1,2")
    parser.add_argument("--protocols", default="a,b,c")
    parser.add_argument("--guide", default="hand")
    parser.add_argument("--d14", action="store_true",
                        help="untimed export and counter comparison")
    parser.add_argument("--candidate-flags", default="")
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
    proc = subprocess.run(command, capture_output=True, text=True,
                          timeout=timeout, check=False)
    wall = time.perf_counter() - started
    return last_json(proc.stdout + proc.stderr), wall, proc.returncode


def check(proof: Path, checker: list[str]) -> dict:
    proc = subprocess.run(checker + ["check", str(proof)],
                          capture_output=True, text=True, check=False)
    return last_json(proc.stdout + proc.stderr)


def median(values):
    return statistics.median(values) if values else None


def main() -> None:
    options = args()
    digest = verify_corpus(Path(options.corpus))
    cells = load_cells(Path(options.corpus))
    seeds = [int(seed) for seed in options.seeds.split(",") if seed]
    protocols = [p for p in options.protocols.split(",") if p]
    extra = [flag for flag in options.candidate_flags.split() if flag]
    rows: list[dict] = []

    def reference(cell: dict, proofs: bool, run_no: int) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "ref.pvr"
            command = [
                options.provari, "prove", cell["fen"],
                "--seconds", str(int(options.reference_seconds)),
                "--memory", str(options.memory), "--guide", "hand", "--json",
            ] + (["--proofs", str(path)] if proofs else [])
            payload, wall, code = run(command, options.reference_seconds + 60)
            verdict = "censored"
            if payload.get("found"):
                verdict = "found"
                if proofs:
                    ours = check(path, [options.candidate])
                    theirs = check(path, [options.provari])
                    verdict = ("accepted" if ours.get("valid") is True
                               and theirs.get("valid") is True else "checker-reject")
            rows.append({
                "side": "provari-hand", "cell": cell["name"], "id": cell["id"],
                "protocol": "b" if proofs else "a/c", "proofs": proofs,
                "seed": None, "run": run_no, "wall_s": round(wall, 3),
                "returncode": code, "win": bool(payload.get("found")),
                "mate": payload.get("mate"), "nodes": payload.get("nodes"),
                "child_evals": None, "expansions": None, "verdict": verdict,
            })

    def candidate(cell: dict, seed: int, proofs: bool, run_no: int) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "cand.pvr"
            command = [
                options.candidate, "prove", cell["fen"],
                "--seconds", str(int(options.seconds)),
                "--memory", str(options.memory), "--guide", options.guide,
                "--seed", str(seed), "--json",
            ] + (["--proofs", str(path)] if proofs else []) + extra
            payload, wall, code = run(command, options.seconds + 60)
            verdict = "censored"
            if payload.get("found"):
                verdict = "found"
                if proofs:
                    ours = check(path, [options.candidate])
                    theirs = check(path, [options.provari])
                    if ours.get("valid") is True and theirs.get("valid") is True:
                        verdict = "accepted"
                    elif ours.get("valid") is False or theirs.get("valid") is False:
                        verdict = "checker-reject"
                    else:
                        verdict = "unchecked"
            rows.append({
                "side": "candidate-hand", "cell": cell["name"], "id": cell["id"],
                "protocol": "b" if proofs else "a/c", "proofs": proofs,
                "seed": seed, "run": run_no, "wall_s": round(wall, 3),
                "returncode": code, "win": bool(payload.get("found")),
                "mate": payload.get("mate"), "nodes": payload.get("nodes"),
                "child_evals": payload.get("child_evals"),
                "expansions": payload.get("expansions"), "verdict": verdict,
            })

    for cell in cells:
        # Every run of one repeat, rotated by one place each repeat, so that
        # the first-run warm-up lands on a different task each time.
        tasks: list[tuple] = []
        if "a" in protocols or "c" in protocols:
            tasks.append(("reference", False))
        if "b" in protocols:
            tasks.append(("reference", True))
        for seed in seeds:
            if "a" in protocols or "b" in protocols:
                tasks.append(("candidate", seed, True))
            if "c" in protocols:
                tasks.append(("candidate", seed, False))
        for run_no in range(1, options.repeats + 1):
            shift = (run_no - 1) % len(tasks)
            for kind, *rest in tasks[shift:] + tasks[:shift]:
                if kind == "reference":
                    reference(cell, rest[0], run_no)
                else:
                    candidate(cell, rest[0], rest[1], run_no)

    # D14: the untimed export must repeat the timed run's counters.
    d14 = []
    if options.d14:
        for cell in cells:
            for seed in seeds:
                with tempfile.TemporaryDirectory() as tmp:
                    path = Path(tmp) / "untimed.pvr"
                    payload, wall, code = run([
                        options.candidate, "prove", cell["fen"],
                        "--seconds", str(int(options.seconds)),
                        "--memory", str(options.memory), "--guide", options.guide,
                        "--seed", str(seed), "--proofs", str(path), "--json",
                    ] + extra, 3600)
                    ours = check(path, [options.candidate])
                    theirs = check(path, [options.provari])
                    timed = [row for row in rows
                             if row["side"] == "candidate-hand"
                             and row["cell"] == cell["name"] and row["seed"] == seed
                             and row["proofs"]]
                    d14.append({
                        "cell": cell["name"], "seed": seed,
                        "wall_s": round(wall, 3), "returncode": code,
                        "found": bool(payload.get("found")),
                        "mate": payload.get("mate"),
                        "nodes": payload.get("nodes"),
                        "child_evals": payload.get("child_evals"),
                        "expansions": payload.get("expansions"),
                        "timed_counters": [
                            [r["mate"], r["nodes"], r["child_evals"], r["expansions"]]
                            for r in timed
                        ],
                        "counters_match": all(
                            [r["mate"], r["nodes"], r["child_evals"], r["expansions"]]
                            == [payload.get("mate"), payload.get("nodes"),
                                payload.get("child_evals"), payload.get("expansions")]
                            for r in timed
                        ),
                        "ours": ours.get("valid"), "provari": theirs.get("valid"),
                    })

    # A timing run that writes no proof is still a claim when the identical
    # run (same cell, seed and repeat) with the proof was accepted: the
    # search is deterministic, so the counters are the same.
    twins = {(row["cell"], row["seed"], row["run"]): row["verdict"]
             for row in rows
             if row["side"] == "candidate-hand" and row["proofs"]}
    for row in rows:
        if row["side"] == "candidate-hand" and not row["proofs"]:
            twin = twins.get((row["cell"], row["seed"], row["run"]))
            if twin == "accepted" and row["verdict"] == "found":
                row["verdict"] = "accepted"
                row["twin"] = True

    # Summaries: protocol a (candidate proofs, bare reference), b (both
    # proof), c (neither); per-seed medians for the candidate.
    def ref_rows(cell, proofs):
        return [row for row in rows
                if row["cell"] == cell and row["side"] == "provari-hand"
                and row["proofs"] == proofs]

    def cand_rows(cell, proofs):
        return [row for row in rows
                if row["cell"] == cell and row["side"] == "candidate-hand"
                and row["proofs"] == proofs]

    summary = {}
    for protocol, candidate_proofs, reference_proofs in [
        ("a", True, False), ("b", True, True), ("c", False, False)
    ]:
        if protocol not in protocols:
            continue
        cells_out = {}
        for cell in cells:
            ref = ref_rows(cell["name"], reference_proofs)
            cand = cand_rows(cell["name"], candidate_proofs)
            if not ref or not cand:
                continue
            ref_wall = median([row["wall_s"] for row in ref])
            cand_wall = median([row["wall_s"] for row in cand])
            accepted = [row for row in cand if row["verdict"] == "accepted"]
            per_seed = {}
            for seed in seeds:
                seed_rows = [row for row in cand if row["seed"] == seed]
                seed_accepted = [row for row in seed_rows
                                 if row["verdict"] == "accepted"]
                seed_wall = median([row["wall_s"] for row in seed_rows])
                per_seed[str(seed)] = {
                    "wall_s": round(seed_wall, 3) if seed_wall else None,
                    "factor": round(seed_wall / ref_wall, 3) if seed_wall else None,
                    "mate": next((r["mate"] for r in seed_accepted), None),
                    "nodes": next((r["nodes"] for r in seed_accepted), None),
                    "child_evals": next((r["child_evals"] for r in seed_accepted), None),
                    "expansions": next((r["expansions"] for r in seed_accepted), None),
                    "accepted": len(seed_accepted), "runs": len(seed_rows),
                }
            at_median = min(cand, key=lambda row: abs(row["wall_s"] - cand_wall))
            cells_out[cell["name"]] = {
                "id": cell["id"],
                "reference_wall_s": round(ref_wall, 3),
                "reference_wall_spread_s": [
                    min(row["wall_s"] for row in ref),
                    max(row["wall_s"] for row in ref),
                ],
                "candidate_wall_s": round(cand_wall, 3),
                "factor": round(cand_wall / ref_wall, 3) if accepted else None,
                "reference_mate": next((r["mate"] for r in ref if r["win"]), None),
                "candidate_mate": next((r["mate"] for r in accepted), None),
                "candidate_at_median": {
                    "seed": at_median["seed"], "mate": at_median["mate"],
                    "nodes": at_median["nodes"],
                    "child_evals": at_median["child_evals"],
                    "expansions": at_median["expansions"],
                },
                "per_seed": per_seed,
                "accepted": len(accepted), "runs": len(cand),
                "verdicts": sorted({row["verdict"] for row in cand}),
            }
        factors = [c["factor"] for c in cells_out.values() if c["factor"]]
        summary[protocol] = {
            "cells": cells_out,
            "geomean": round(math.exp(sum(math.log(f) for f in factors)
                                      / len(factors)), 3) if factors else None,
            "coverage": f"{len(factors)}/{len(cells_out)}",
        }

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
        "protocols": protocols,
        "candidate_flags": options.candidate_flags,
        "rows": rows,
        "summary": summary,
        "d14": d14,
    }, indent=2) + "\n")
    env = {
        "date": datetime.now(timezone.utc).isoformat(),
        "host": platform.node(),
        "platform": platform.platform(),
        "python": platform.python_version(),
        "provari": options.provari,
        "provari_version": subprocess.run(
            [options.provari, "--version"], capture_output=True, text=True,
            check=False).stdout.strip(),
        "candidate": options.candidate,
        "candidate_version": subprocess.run(
            [options.candidate, "--version"], capture_output=True, text=True,
            check=False).stdout.strip(),
        "corpus_sha256": digest,
        "seconds": options.seconds,
        "reference_seconds": options.reference_seconds,
        "memory_mb": options.memory,
        "repeats": options.repeats,
        "seeds": seeds,
        "protocols": protocols,
        "candidate_flags": options.candidate_flags,
        "cwd": os.getcwd(),
    }
    env_path = (out.parent / "env.json" if out.stem == "results"
                else out.parent / f"env-{out.stem}.json")
    env_path.write_text(json.dumps(env, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
