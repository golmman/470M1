#!/usr/bin/env python3
"""Plan 12 driver: `M8` removal, instruction parity and the `MD6` row.

Builds the plan 11 product (`HEAD`) and the plan 2 reference (`2f629fd`) under
`--scratch`, measures the working tree's release binary against the plan 11
product with three interleaved `perf stat -e instructions:u` rounds, and runs
the plan 3 bench driver for the same-session `MD6` row. The working tree is
read-only to this script. Reproduces `perf.txt`, `env.json` and `bench/`.
"""
import argparse, hashlib, json, platform, shutil, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}
PROVE = ["--seconds", "10", "--memory", "192", "--seed", "0", "--json"]
DRIVER = ROOT / "docs/plans/parity/measurements/plan3/run_bench.py"


def run(cmd, **kw):
    return subprocess.run(cmd, check=True, capture_output=True, text=True, **kw)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def build(scratch, name, rev):
    tree = scratch / name
    if tree.exists():
        shutil.rmtree(tree)
    tree.mkdir(parents=True)
    run(f"git -C '{ROOT}' archive --format=tar {rev} | tar -x -C '{tree}'", shell=True)
    run(["cargo", "build", "--release", "--quiet"], cwd=tree)
    for name in ("470MI", "470m1"):
        binary = tree / "target/release" / name
        if binary.exists():
            return binary
    sys.exit(f"no binary in {tree}")


def instructions(binary, fen):
    p = run(["perf", "stat", "-e", "instructions:u", "-x,", "--", str(binary),
             "prove", fen, *PROVE])
    count = next(int(line.split(",")[0]) for line in p.stderr.splitlines()
                 if line.split(",")[2:3] == ["instructions:u"])
    payload = json.loads(p.stdout.strip().splitlines()[-1])
    return count, [payload["child_evals"], payload["expansions"], payload["nodes"],
                   payload["mate"], payload["found"]]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scratch", default="/tmp/plan12")
    ap.add_argument("--rounds", type=int, default=3)
    ap.add_argument("--provari", default="/tmp/470M1-oracle/provari/target/release/provari")
    ap.add_argument("--skip-perf", action="store_true")
    ap.add_argument("--skip-bench", action="store_true")
    a = ap.parse_args()
    scratch = Path(a.scratch)
    scratch.mkdir(parents=True, exist_ok=True)
    head = build(scratch, "head", "HEAD")
    old = build(scratch, "plan2", "2f629fd")
    candidate = ROOT / "target/release/470MI"
    run(["cargo", "build", "--release", "--quiet"], cwd=ROOT)

    if not a.skip_perf:
        rounds = {name: {"head": [], "cand": []} for name in CELLS}
        for _ in range(a.rounds):
            for name, fen in CELLS.items():
                rounds[name]["head"].append(instructions(head, fen))
                rounds[name]["cand"].append(instructions(candidate, fen))
        lines = []
        for name in CELLS:
            h = [x[0] for x in rounds[name]["head"]]
            c = [x[0] for x in rounds[name]["cand"]]
            assert all(x[1] == rounds[name]["head"][0][1] for x in rounds[name]["head"])
            assert all(x[1] == rounds[name]["cand"][0][1] for x in rounds[name]["cand"])
            assert rounds[name]["head"][0][1] == rounds[name]["cand"][0][1]
            hm, cm = sorted(h)[len(h) // 2], sorted(c)[len(c) // 2]
            lines += [f"{name}: head median {hm} (spread {max(h) - min(h)}), "
                      f"cand median {cm} (spread {max(c) - min(c)}), "
                      f"fall {hm - cm} ({(hm - cm) / hm * 100:.3f}%)",
                      f"  head rounds {h}", f"  cand rounds {c}"]
        (HERE / "perf.txt").write_text("\n".join(lines) + "\n")
        print("\n".join(lines))

    if not a.skip_bench:
        bench = HERE / "bench"
        bench.mkdir(parents=True, exist_ok=True)
        sides = {}
        for name, binary in (("results.json", candidate), ("old.json", old)):
            run([sys.executable, str(DRIVER), "--provari", a.provari,
                 "--candidate", str(binary), "--out", str(bench / name),
                 "--seconds", "10", "--memory", "192", "--repeats", "3",
                 "--seeds", "0", "--protocols", "a", "--d14"])
            sides[name] = json.loads((bench / "env.json").read_text())
        combined = dict(sides["results.json"])
        combined.update({
            "old": str(old), "old_sha256": sha(old),
            "old_source": "470m1@2f629fd",
            "candidate_sha256": sha(candidate),
            "note": "candidate and old ran back to back in one session; "
                    "the old pass overwrote the driver's env.json",
        })
        (bench / "env.json").write_text(json.dumps(combined, indent=2) + "\n")
        (HERE / "env.json").write_text(json.dumps({
            "date": "2026-11-20", "host": platform.node(),
            "platform": platform.platform(), "machine": platform.machine(),
            "perf_counter": "instructions:u", "rounds": a.rounds,
            "order": ["head", "cand"], "args": PROVE,
            "head_sha256": sha(head), "cand_sha256": sha(candidate),
            "old_sha256": sha(old), "old_source": "470m1@2f629fd",
            "head_source": "470MI@HEAD (plan 11 product)",
        }, indent=2) + "\n")


if __name__ == "__main__":
    main()
