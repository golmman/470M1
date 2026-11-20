#!/usr/bin/env python3
"""Kill review 1 probe: does the bench row charge work the instruction
witness never measured?

The 10 s row (protocol `a`, `docs/bench.md`) runs the candidate with
`--proofs`, so its wall includes the proof-export walk. Every instruction
witness since plan 5 ran the binary *without* `--proofs`. This probe measures
the same three cells four ways -- {plan 12 product, plan 2 reference} x
{with, without a proof path} -- paired and interleaved, then attributes the
with/without instruction delta on `core` by symbol.

Read-only: it runs existing binaries, writes only under this directory.
It is a review probe, not a ledger row (`docs/bench.md` rule 4/6).

Usage: python3 run_killreview1.py [--rounds 5] [--skip-symbols]
"""
import argparse, hashlib, json, platform, statistics, subprocess, tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
CAND = Path("/workspace/470M1/target/release/470MI")
OLD = Path("/tmp/plan12/plan2/target/release/470m1")
CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}
FLAGS = ["--seconds", "10", "--memory", "192", "--seed", "0", "--json"]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def command(binary, fen, proof):
    return [str(binary), "prove", fen, *FLAGS] + (["--proofs", str(proof)] if proof else [])


def wall(binary, fen, proof):
    import time
    started = time.perf_counter()
    p = subprocess.run(command(binary, fen, proof), capture_output=True, text=True)
    elapsed = time.perf_counter() - started
    assert p.returncode == 0, p.stderr[-200:]
    return elapsed, json.loads(p.stdout.strip().splitlines()[-1])


def instructions(binary, fen, proof):
    p = subprocess.run(["perf", "stat", "-e", "instructions:u", "-x,", "--",
                        *command(binary, fen, proof)], capture_output=True, text=True)
    count = next(int(line.split(",")[0]) for line in p.stderr.splitlines()
                 if line.split(",")[2:3] == ["instructions:u"])
    return count, json.loads(p.stdout.strip().splitlines()[-1])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rounds", type=int, default=5)
    ap.add_argument("--perf-rounds", type=int, default=3)
    ap.add_argument("--skip-symbols", action="store_true")
    a = ap.parse_args()
    tmp = Path(tempfile.mkdtemp(prefix="killreview1-"))
    results = {"walls_s": {}, "instructions": {}, "search_ms": {}}

    for cell, fen in CELLS.items():
        per = {(side, proof): [] for side in ("cand", "old") for proof in (True, False)}
        for i in range(a.rounds):
            order = [("cand", CAND), ("old", OLD)]
            if i % 2:
                order.reverse()
            for side, binary in order:
                for proof in (True, False):
                    p = tmp / f"{side}-{int(proof)}.pvr" if proof else None
                    w, d = wall(binary, fen, p)
                    per[(side, proof)].append(w)
                    results["search_ms"][f"{cell}/{side}/{int(proof)}"] = d["ms"]
        for key, walls in per.items():
            results["walls_s"][f"{cell}/{key[0]}/{'proof' if key[1] else 'search'}"] = {
                "median": round(statistics.median(walls), 4),
                "runs": [round(w, 4) for w in walls],
            }
        for proof in (True, False):
            results.setdefault("ratios_wall", {})[f"{cell}/{'with' if proof else 'without'}"] = round(
                statistics.median(per[("cand", proof)]) / statistics.median(per[("old", proof)]), 4)

    for cell, fen in CELLS.items():
        for side, binary in (("cand", CAND), ("old", OLD)):
            for proof in (True, False):
                p = tmp / f"perf-{side}-{int(proof)}.pvr" if proof else None
                runs = [instructions(binary, fen, p) for _ in range(a.perf_rounds)]
                assert all(r[1]["child_evals"] == runs[0][1]["child_evals"] for r in runs)
                results["instructions"][f"{cell}/{side}/{'proof' if proof else 'search'}"] = {
                    "median": int(statistics.median(r[0] for r in runs)),
                    "runs": [int(r[0]) for r in runs],
                    "child_evals": runs[0][1]["child_evals"],
                    "mate": runs[0][1]["mate"],
                }

    if not a.skip_symbols:
        fen = CELLS["core"]
        deltas = {}
        for proof in (True, False):
            data = tmp / f"core-{int(proof)}.data"
            p = tmp / "sym.pvr" if proof else None
            subprocess.run(["perf", "record", "-q", "-o", str(data), "-e", "instructions:u",
                            "-c", "100000", "--", *command(CAND, fen, p)],
                           capture_output=True)
            out = subprocess.run(["perf", "report", "-i", str(data), "--stdio", "--sort", "symbol"],
                                 capture_output=True, text=True).stdout
            deltas[int(proof)] = out
        total = {proof: results["instructions"][f"core/cand/{'proof' if proof else 'search'}"]["median"]
                 for proof in (True, False)}

        import re
        tables = {}
        for proof, text in deltas.items():
            table = {}
            for line in text.splitlines():
                m = re.match(r"\s*([\d.]+)%\s+\[\S+\]\s+(\S+)", line)
                if m:
                    table[m.group(2)] = float(m.group(1)) * total[proof] / 100
            tables[proof] = table
        rows = []
        for name in set(tables[True]) | set(tables[False]):
            d = tables[True].get(name, 0) - tables[False].get(name, 0)
            rows.append((d, name))
        rows.sort(reverse=True)
        results["symbol_delta_core"] = [
            {"delta_instructions": int(d), "symbol": name} for d, name in rows[:14]
        ]

    results["env"] = {
        "date": "2026-11-20",
        "host": platform.node(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "candidate": str(CAND), "candidate_sha256": sha(CAND),
        "reference": str(OLD), "reference_sha256": sha(OLD),
        "reference_source": "470m1@2f629fd (plan 2 tree, rebuilt in /tmp/plan12)",
        "candidate_source": "470MI@e3c237e working tree (plan 12 product)",
        "flags": FLAGS, "rounds": a.rounds, "perf_rounds": a.perf_rounds,
        "perf_counter": "instructions:u", "interleaved": True,
        "note": "wall includes process start; the row's driver measures the same way",
    }
    (HERE / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
