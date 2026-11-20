#!/usr/bin/env python3
"""Plan 9 driver: evasion mask (ev) and unsafe-optimizations (uo) vs base.

Builds three scratch trees from `git archive HEAD` (base, ev = ev.patch,
uo = uo.patch), runs the ignored rules differential in ev and uo, then
`--rounds` interleaved rounds of `perf stat -e instructions:u` over core,
easy-win and deep-win (order base, ev, uo per cell), one `.pvr` export per
variant and cell, our checker (and `provari check` if present) on each proof,
and writes perf.txt, results.json, env.json and decision.json to --out.
Read-only for the repository; every build lives under --scratch.
"""
import argparse, hashlib, json, platform, socket, statistics, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}
VARIANTS = {"base": None, "ev": "ev.patch", "uo": "uo.patch"}
PROVE = ["--seconds", "10", "--memory", "192", "--seed", "0", "--json"]
THRESHOLD = 0.31e9
BENCH_HOST = "c90face26537"


def run(cmd, **kw):
    return subprocess.run(cmd, check=True, capture_output=True, text=True, **kw)


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def build(scratch, name, patch, skip):
    tree = scratch / name
    binary = tree / "target/release/470MI"
    if skip and binary.exists():
        return tree, binary
    if tree.exists():
        run(["rm", "-rf", str(tree)])
    tree.mkdir(parents=True)
    subprocess.run(f"git -C '{ROOT}' archive --format=tar HEAD | tar -x -C '{tree}'",
                   shell=True, check=True)
    if patch:
        run(["git", "apply", str(HERE / patch)], cwd=tree)
    print(f"build {name}", flush=True)
    subprocess.run(["cargo", "build", "--release", "--locked", "--quiet"], cwd=tree, check=True)
    return tree, binary


def differential(tree):
    out = run(["cargo", "test", "--release", "--locked", "--quiet", "--test", "differential",
               "--", "--ignored", "--exact", "rules_differential_full", "--nocapture"],
              cwd=tree).stdout
    line = next(l for l in out.splitlines() if l.startswith("DIFFERENTIAL {"))
    return json.loads(line.removeprefix("DIFFERENTIAL "))


def prove(binary, fen, perf):
    cmd = [str(binary), "prove", fen, *PROVE]
    if perf:
        cmd = ["perf", "stat", "-e", "instructions:u", "-x,", "--", *cmd]
    p = run(cmd)
    js = p.stdout.strip().splitlines()[-1]
    inst = None
    if perf:
        inst = next(int(l.split(",")[0]) for l in p.stderr.splitlines()
                    if l.split(",")[2:3] == ["instructions:u"])
    return js, json.loads(js), inst


def counters(j):
    return [j["child_evals"], j["expansions"], j["nodes"], j["mate"], j["found"]]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scratch", default="/tmp/plan9")
    ap.add_argument("--out", default=str(HERE))
    ap.add_argument("--rounds", type=int, default=3)
    ap.add_argument("--no-perf", action="store_true", help="plumbing check without a PMU")
    ap.add_argument("--skip-build", action="store_true")
    ap.add_argument("--provari", default="/tmp/470M1-oracle/provari/target/release/provari")
    a = ap.parse_args()
    scratch, out = Path(a.scratch), Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    dirty = run(["git", "-C", str(ROOT), "status", "--porcelain", "--", "src", "tests",
                 "Cargo.toml", "Cargo.lock"]).stdout.strip()
    if dirty:
        sys.exit(f"product tree has uncommitted changes:\n{dirty}")
    head = run(["git", "-C", str(ROOT), "rev-parse", "--short", "HEAD"]).stdout.strip()
    trees = {v: build(scratch, v, p, a.skip_build) for v, p in VARIANTS.items()}
    diff = {}
    for v in ("ev", "uo"):
        print(f"differential {v}", flush=True)
        diff[v] = differential(trees[v][0])

    perf = not a.no_perf
    log = [f"# Plan 9 — perf stat -e instructions:u, {head}, prove FEN {' '.join(PROVE)}"]
    runs = {c: {v: [] for v in VARIANTS} for c in CELLS}
    for r in range(1, a.rounds + 1):
        for c, fen in CELLS.items():
            for v in VARIANTS:
                js, j, inst = prove(trees[v][1], fen, perf)
                runs[c][v].append({"instructions": inst, "ms": j["ms"], "counters": counters(j)})
                log += [f"# round {r} {c} {v}", js, f"{inst},,instructions:u"]
                print(f"round {r} {c:8} {v:4} {inst} {j['ms']} ms", flush=True)

    pvr_dir = scratch / "pvr"
    pvr_dir.mkdir(parents=True, exist_ok=True)
    oracle = Path(a.provari) if Path(a.provari).exists() else None
    proofs = {c: {} for c in CELLS}
    for c, fen in CELLS.items():
        for v in VARIANTS:
            path = pvr_dir / f"{v}-{c}.pvr"
            run([str(trees[v][1]), "prove", fen, *PROVE, "--proofs", str(path)])
            ours = json.loads(subprocess.run([str(trees["base"][1]), "check", str(path)],
                                             capture_output=True, text=True).stdout)["valid"]
            theirs = None
            if oracle:
                theirs = json.loads(subprocess.run([str(oracle), "check", str(path)],
                                                   capture_output=True, text=True).stdout)["valid"]
            proofs[c][v] = {"sha256": sha256(path), "ours": ours, "provari": theirs}

    summary = {}
    for c in CELLS:
        base = runs[c]["base"]
        s = {}
        for v in VARIANTS:
            rs = runs[c][v]
            e = {"counters_identical": all(x["counters"] == base[0]["counters"] for x in rs),
                 "pvr_identical": proofs[c][v]["sha256"] == proofs[c]["base"]["sha256"],
                 "median_ms": statistics.median(x["ms"] for x in rs)}
            if perf:
                inst = [x["instructions"] for x in rs]
                e |= {"median": statistics.median(inst), "spread": max(inst) - min(inst)}
                if v != "base":
                    pairs = [b["instructions"] - x["instructions"] for b, x in zip(base, rs)]
                    e |= {"fall": s["base"]["median"] - e["median"], "pair_falls": pairs,
                          "fall_pct": round(100 * (s["base"]["median"] - e["median"])
                                            / s["base"]["median"], 3)}
            s[v] = e
        summary[c] = s

    host = socket.gethostname()
    on_bench = host == BENCH_HOST and platform.machine() == "x86_64"
    correct = {v: all(summary[c][v]["counters_identical"] and summary[c][v]["pvr_identical"]
                      and proofs[c][v]["ours"] and proofs[c][v]["provari"] is not False
                      for c in CELLS) and diff[v]["mismatches"] == 0 for v in ("ev", "uo")}
    ev_fall = summary["core"]["ev"].get("fall")
    release = bool(perf and correct["ev"] and ev_fall is not None and ev_fall >= THRESHOLD)
    decision = {
        "rule": "release (W10 + MD8) iff core ev fall >= 0.31e9 instructions:u with identical "
                "counters/mate and byte-identical .pvr on all three cells and differential 0",
        "bench_host": on_bench, "provisional": not on_bench or not perf,
        "ev": {"core_fall": ev_fall, "correct": correct["ev"],
               "branch": "release" if release else "acceptance"},
        "uo": {"falls": {c: summary[c]["uo"].get("fall") for c in CELLS},
               "fall_pct": {c: summary[c]["uo"].get("fall_pct") for c in CELLS},
               "correct": correct["uo"], "branch": "owner decision"},
    }
    env = {"host": host, "platform": platform.platform(), "machine": platform.machine(),
           "rustc": run(["rustc", "-V"]).stdout.strip(),
           "cargo": run(["cargo", "-V"]).stdout.strip(), "git_head": head,
           "perf_counter": "instructions:u" if perf else None, "rounds": a.rounds,
           "order": list(VARIANTS), "args": PROVE, "scratch": str(scratch),
           "binaries": {v: sha256(b) for v, (_, b) in trees.items()},
           "provari": str(oracle) if oracle else None}
    (out / "perf.txt").write_text("\n".join(log) + "\n")
    (out / "results.json").write_text(json.dumps(
        {"summary": summary, "runs": runs, "proofs": proofs, "differential": diff}, indent=1) + "\n")
    (out / "env.json").write_text(json.dumps(env, indent=2) + "\n")
    (out / "decision.json").write_text(json.dumps(decision, indent=2) + "\n")
    print(json.dumps(decision, indent=2))


if __name__ == "__main__":
    main()
