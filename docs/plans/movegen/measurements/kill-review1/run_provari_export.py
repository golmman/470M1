#!/usr/bin/env python3
"""Does the reference pay for a proof under protocol `a`?

`docs/bench.md` protocol `a` runs the candidate with `--proofs` and the
reference without. This probe adds `--proofs` to the pinned provari oracle and
measures the same three cells, so the export asymmetry has a number.
Read-only; writes `provari_export.json` next to this file.
"""
import json, statistics, subprocess, tempfile, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
PROVARI = Path("/tmp/470M1-oracle/provari/target/release/provari")
CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}


def run(fen, proof):
    with tempfile.TemporaryDirectory() as tmp:
        cmd = [str(PROVARI), "prove", fen, "--seconds", "10", "--memory", "192",
               "--guide", "hand", "--json"]
        if proof:
            cmd += ["--proofs", str(Path(tmp) / "p.pvr")]
        started = time.perf_counter()
        p = subprocess.run(cmd, capture_output=True, text=True)
        wall = time.perf_counter() - started
        assert p.returncode == 0, p.stderr[-200:]
        return wall, json.loads(p.stdout.strip().splitlines()[-1])


results = {"cells": {}, "env": {"date": "2026-11-20", "oracle": str(PROVARI),
                                "source": "provari v0.4.0 bceb1ee", "rounds": 5}}
for cell, fen in CELLS.items():
    bare, exported = [], []
    for _ in range(5):
        bare.append(run(fen, False)[0])
        exported.append(run(fen, True)[0])
    payload = run(fen, True)[1]
    results["cells"][cell] = {
        "bare_median_s": round(statistics.median(bare), 4),
        "proof_median_s": round(statistics.median(exported), 4),
        "export_ms": round((statistics.median(exported) - statistics.median(bare)) * 1000, 1),
        "runs_bare": [round(w, 4) for w in bare],
        "runs_proof": [round(w, 4) for w in exported],
        "mate": payload.get("mate"),
    }
(HERE / "provari_export.json").write_text(json.dumps(results, indent=2) + "\n")
print(json.dumps(results, indent=2))
