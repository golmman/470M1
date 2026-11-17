#!/usr/bin/env python3
"""Reference-side bench driver for the `parity` initiative.

Adapted from atomi `a4802e7cbe7ea6338e99dd21d3b84dbd6f871f76`
(`bench/run_bench.py`): only the pinned provari reference side, the corpus
sha256 gate, the three fixed cells and parsed JSON output remain.

Usage:
    python3 bench/run_bench.py --provari /tmp/470M1-oracle/provari/target/release/provari \
        --out docs/plans/parity/measurements/plan1/results.json
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
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
CORPUS = HERE / "corpus.json"
CORPUS_SHA256 = "9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e"
CELLS = {"core": "core", "easy-win": "1eb80b92f92c", "deep-win": "db6a741c723f"}


def args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--provari", required=True)
    parser.add_argument("--corpus", default=str(CORPUS))
    parser.add_argument("--out", required=True)
    parser.add_argument("--seconds", type=float, default=10.0)
    parser.add_argument("--memory", type=int, default=192)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--guide", default="hand")
    parser.add_argument("--dry-run", action="store_true")
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
    missing = [cell_id for cell_id in CELLS.values() if cell_id not in by_id]
    if missing:
        raise SystemExit(f"corpus lacks cells: {missing}")
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


def main() -> None:
    options = args()
    digest = verify_corpus(Path(options.corpus))
    cells = load_cells(Path(options.corpus))
    rows = []
    for cell in cells:
        for run in range(1, options.repeats + 1):
            command = [
                options.provari, "prove", cell["fen"],
                "--seconds", str(int(options.seconds)),
                "--memory", str(options.memory),
                "--guide", options.guide, "--json",
            ]
            if options.dry_run:
                print(" ".join(command))
                continue
            started = time.perf_counter()
            proc = subprocess.run(
                command, capture_output=True, text=True,
                timeout=options.seconds + 60, check=False,
            )
            wall = time.perf_counter() - started
            payload = last_json(proc.stdout + proc.stderr)
            rows.append({
                "cell": cell["name"], "id": cell["id"], "run": run,
                "wall_s": round(wall, 3), "returncode": proc.returncode,
                "win": bool(payload.get("found")),
                "mate": payload.get("mate"), "nodes": payload.get("nodes"),
                "ms": payload.get("ms"),
            })
    if options.dry_run:
        return
    summary = {}
    for cell in cells:
        mine = [row for row in rows if row["cell"] == cell["name"]]
        summary[cell["name"]] = {
            "id": cell["id"],
            "walls_s": [row["wall_s"] for row in mine],
            "median_wall_s": round(statistics.median(row["wall_s"] for row in mine), 3),
            "wins": sum(row["win"] for row in mine),
            "mate": next((row["mate"] for row in mine if row["win"]), None),
            "nodes": next((row["nodes"] for row in mine if row["win"]), None),
        }
    out = Path(options.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps({
        "side": f"provari-{options.guide}",
        "corpus_sha256": digest,
        "seconds": options.seconds,
        "memory_mb": options.memory,
        "repeats": options.repeats,
        "rows": rows,
        "summary": summary,
    }, indent=2) + "\n")
    env = {
        "date": datetime.now(timezone.utc).isoformat(),
        "host": platform.node(),
        "platform": platform.platform(),
        "python": platform.python_version(),
        "provari": options.provari,
        "provari_version": subprocess.run(
            [options.provari, "--version"], capture_output=True, text=True,
            check=False,
        ).stdout.strip(),
        "corpus_sha256": digest,
        "seconds": options.seconds,
        "memory_mb": options.memory,
        "repeats": options.repeats,
        "guide": options.guide,
        "cwd": os.getcwd(),
    }
    (out.parent / "env.json").write_text(json.dumps(env, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
