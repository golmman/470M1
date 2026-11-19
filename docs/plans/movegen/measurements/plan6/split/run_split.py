"""Plan 6 Phase A: turn the perf symbol tables and the counted build into
`split.json`.

Inputs (all parsed, no raw `perf.data`):
  symbols-2100.json   perf report --sort symbol --show-total-period, 2.10.0
  symbols-plan2.json  the same for the plan 2 binary (2f629fd)
  counts-2100.json    the counted build's `--counts` JSON line on `core`

Output: split.json

Run: python3 run_split.py
"""

import json
import os

from bucket_map import aligned_group, group

HERE = os.path.dirname(os.path.abspath(__file__))

# perf stat -e instructions:u on `core`, seed 0, 10 s, no export, this session
STAT_2100 = 8_004_086_093
STAT_PLAN2 = 6_569_377_210
EVALS_2100 = 3_896_430
EVALS_PLAN2 = 3_823_720

# counted sites -> the symbols their cost lands in, in the 2.10.0 binary
SITE_SYMBOLS = {
    "legal_in": ["legal_in::"],
    "effect": ["child_effect_with_occupancy", "blast_casualties", "castling_rights_after"],
    "pseudo_legal": ["Position>::pseudo_legal"],
    "play": ["Position>::play"],
    "null": ["Dfpn>::replay_threat"],
    "mover_new": ["Mover>::new", "populate_state"],
    "plain": ["Mover>::plain"],
    "exposes": ["Mover>::exposes"],
    "gives_check": ["Mover>::gives_check"],
    "threats": ["Mover>::threats"],
    "terms": ["eval::terms"],
    "replay_threat": ["Dfpn>::replay_threat"],
    "mover_legal": None,
    "attackers_to": ["Board>::attackers_to"],
}
COUNT_NAMES = {
    "legal_in": "context_legal",
    "effect": "effect",
    "pseudo_legal": "pseudo_legal",
    "play": "play",
    "null": "null",
    "mover_new": "mover_new",
    "plain": "plain",
    "exposes": "exposes",
    "gives_check": "gives_check",
    "threats": "threats",
    "terms": "terms",
    "replay_threat": "replay_threat",
    "mover_legal": "mover_legal",
    "attackers_to": "attackers_to",
}


def load(name):
    with open(os.path.join(HERE, name)) as f:
        return json.load(f)


def rollup(symbols, classifier):
    buckets = {}
    for row in symbols:
        name = classifier(row["symbol"])
        bucket = buckets.setdefault(name, {"instructions": 0, "symbols": 0})
        bucket["instructions"] += row["period"]
        bucket["symbols"] += 1
    return buckets


def symbols_matching(symbols, pattern):
    return sum(r["period"] for r in symbols if pattern in r["symbol"])


def main():
    sym2100 = load("symbols-2100.json")
    symplan2 = load("symbols-plan2.json")
    deep = load("counts-2100.json")
    counts = deep["counts"]

    primary = {
        "2100": rollup(sym2100, group),
        "plan2": rollup(symplan2, group),
    }
    aligned = {
        "2100": rollup(sym2100, aligned_group),
        "plan2": rollup(symplan2, aligned_group),
    }

    names = sorted(set(primary["2100"]) | set(primary["plan2"]))
    table = {}
    for name in names:
        a = primary["2100"].get(name, {}).get("instructions", 0)
        b = primary["plan2"].get(name, {}).get("instructions", 0)
        aa = aligned["2100"].get(name, {}).get("instructions", 0)
        ab = aligned["plan2"].get(name, {}).get("instructions", 0)
        table[name] = {
            "2100": a,
            "plan2": b,
            "delta": a - b,
            "aligned_2100": aa,
            "aligned_plan2": ab,
            "aligned_delta": aa - ab,
        }

    sites = {}
    for site, patterns in SITE_SYMBOLS.items():
        calls = counts[COUNT_NAMES[site]]
        ins = None
        if patterns:
            ins = sum(symbols_matching(sym2100, p) for p in patterns)
        sites[site] = {
            "calls": calls,
            "instructions": ins,
            "per_call": (ins / calls) if ins is not None and calls else None,
            "share": (ins / STAT_2100) if ins is not None else None,
        }

    total_delta = STAT_2100 - STAT_PLAN2
    tree_delta = (EVALS_2100 / EVALS_PLAN2 - 1) * STAT_PLAN2
    sum2100 = sum(r["period"] for r in sym2100)
    sumplan2 = sum(r["period"] for r in symplan2)

    readings = {}
    for label, buckets in (("primary", primary), ("aligned", aligned)):
        rules = buckets["2100"]["rules"]["instructions"] - buckets["plan2"]["rules"]["instructions"]
        geom = (
            buckets["2100"]["geometry"]["instructions"]
            - buckets["plan2"]["geometry"]["instructions"]
        )
        residual = total_delta - tree_delta - rules
        readings[label] = {
            "rules_delta": rules,
            "geometry_delta": geom,
            "tree_delta": tree_delta,
            "non_rules_residual": residual,
            "geometry_share": geom / residual,
            "geometry_at_least_300m": geom >= 300_000_000,
            "geometry_share_at_least_half": geom / residual >= 0.5,
        }

    out = {
        "note": "perf record -e instructions:u -c 500000 on core, seed 0, 10 s, no export",
        "counters": {k: deep[k] for k in ("child_evals", "expansions", "nodes", "mate")},
        "perf_stat": {"2100": STAT_2100, "plan2": STAT_PLAN2},
        "symbol_totals": {"2100": sum2100, "plan2": sumplan2},
        "reconciliation": {
            "symbol_sum_vs_perf_stat_2100": sum2100 / STAT_2100 - 1,
            "symbol_sum_vs_perf_stat_plan2": sumplan2 / STAT_PLAN2 - 1,
            "total_delta": total_delta,
            "tree_delta": tree_delta,
            "bucket_delta_sum": sum(v["delta"] for v in table.values()),
        },
        "sites": sites,
        "buckets": table,
        "readings": readings,
    }
    with open(os.path.join(HERE, "split.json"), "w") as f:
        json.dump(out, f, indent=1)
        f.write("\n")

    print(f"counters {out['counters']}")
    print(f"symbol totals {sum2100/1e9:.4f}e9 {sumplan2/1e9:.4f}e9 "
          f"({100*(sum2100/STAT_2100-1):+.2f}% / {100*(sumplan2/STAT_PLAN2-1):+.2f}%)")
    print(f"total delta {total_delta/1e9:.4f}e9 tree {tree_delta/1e9:.4f}e9")
    for label, r in readings.items():
        print(f"{label:8s} rules {r['rules_delta']/1e9:+.4f} geom {r['geometry_delta']/1e9:+.4f} "
              f"residual {r['non_rules_residual']/1e9:.4f} share {100*r['geometry_share']:.1f}% "
              f"door {'open' if r['geometry_at_least_300m'] and r['geometry_share_at_least_half'] else 'shut'}")
    print()
    for name in names:
        t = table[name]
        print(f"{name:11s} {t['2100']/1e9:8.4f} {t['plan2']/1e9:8.4f} {t['delta']/1e9:+8.4f}  "
              f"aligned {t['aligned_delta']/1e9:+8.4f}")
    print()
    for site, s in sites.items():
        per = "-" if s["per_call"] is None else f"{s['per_call']:.1f}"
        ins = "-" if s["instructions"] is None else f"{s['instructions']/1e9:.4f}e9"
        print(f"{site:14s} calls {s['calls']:>12,} ins {ins:>9s} per call {per:>7s}")


if __name__ == "__main__":
    main()
