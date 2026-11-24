#!/usr/bin/env python3
"""Bench driver v2: reference and candidate sides, sets, rows and claims.

Inherited from atomi `a4802e7c...` (`bench/run_bench.py`) through the parity
plan1 driver; the reference command, the corpus gate and the parsed JSON
survive. v2 (`foundation` plan1) adds the candidate side, pre-registered
sets, calibration, selection, interleaved repeats and claim checking.

Modes
  row        measure a set's `strong` (or `held-out`) cells, both sides
  select     reference-only selection: build a strong/held-out set file
  calibrate  choose `N` per host from the calibration cells

Everything is one process at a time. The driver refuses a corpus, a high200
file, a set file or a tablebase manifest whose digest does not match, and a
reference whose `--version` is not the pinned full-feature build.

Usage examples (see bench/README.md):
  python3 bench/run_bench.py --mode row --set bench/sets/matched-v1.json \
      --provari /tmp/470M1-oracle/provari/target/release/provari \
      --bin target/release/470MI --guide model --threads 1 \
      --out docs/plans/foundation/measurements/plan1/matched
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
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
CORPUS = HERE / "corpus.json"
HIGH200 = HERE / "high200.jsonl"
SETS = HERE / "sets"

CORPUS_SHA256 = "9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e"
HIGH200_SHA256 = "857496451612904ef4eddc77a93ae4328f00d603b7755c49b8ae114d89682af6"
TB_MANIFEST_SHA256 = "a0c7a69bc720dae8c77a776810d2f33486c153655cebdef229e27a40fdeefc37"
NETWORK_SHA256 = "3d76258c556ce216d088f89ff094dadab4310e3838110c64f66049b6ab5c40b1"
MODEL_SHA256 = "51be8f472304ce535a385ad88906c75a976e5c8d2b267162bd9818a59231882b"
REF_VERSION = "provari 0.5.0, with its network and model built in"
REF_COMMIT = "b87d179c01ef8eae92a8ae93e4ca8b3df924dd71"

# high200 1-based lines 3, 23, ..., 143: calibration cells, excluded forever.
CALIBRATION_LINES = tuple(range(3, 144, 20))
SPREAD_UNSTABLE = 0.20
SLOW_PROVE_S = 9.0
GUARD_CELLS = ("core", "easy-win", "deep-win")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fail(message: str):
    raise SystemExit(f"bench: {message}")


def gate(path: Path, expected: str, label: str) -> str:
    got = sha256(path)
    if got != expected:
        fail(f"{label} sha256 mismatch: {got} != {expected}")
    return got


def last_json(text: str) -> dict:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    return {}


def load_jsonl(path: Path) -> list[dict]:
    cells = []
    for line in path.read_text().splitlines():
        if line.strip():
            cells.append(json.loads(line))
    return cells


def universe(corpus_path: Path, high200_path: Path) -> tuple[dict[str, str], set[str]]:
    gate(corpus_path, CORPUS_SHA256, "corpus-v1")
    gate(high200_path, HIGH200_SHA256, "high200")
    corpus = json.loads(corpus_path.read_text())
    cells = {corpus["core"]["id"]: corpus["core"]["fen"]}
    lines = load_jsonl(high200_path)
    if len(lines) != 200:
        fail(f"high200 has {len(lines)} lines, expected 200")
    calibration = set()
    for index, cell in enumerate(lines, start=1):
        cells[cell["id"]] = cell["fen"]
        if index in CALIBRATION_LINES:
            calibration.add(cell["id"])
    for cell in corpus["sample20"]:
        if cells.get(cell["id"]) != cell["fen"]:
            fail(f"corpus sample20 disagrees with high200 at {cell['id']}")
    return cells, calibration


def geomean(values: list[float]) -> float | None:
    values = [v for v in values if v > 0]
    if not values:
        return None
    return math.exp(sum(math.log(v) for v in values) / len(values))


def spread(walls: list[float]) -> float:
    median = statistics.median(walls)
    if median <= 0:
        return 0.0
    return (max(walls) - min(walls)) / median


class Runner:
    def __init__(self, options: argparse.Namespace) -> None:
        self.o = options
        self.rows: list[dict] = []

    def reference(self, fen: str, proofs: str | None = None) -> list[str]:
        command = [
            self.o.provari, "prove", fen,
            "--seconds", str(int(self.o.seconds)),
            "--memory", str(self.o.memory),
            "--guide", self.o.guide,
            "--seed", str(self.o.seed),
            "--json",
        ]
        if self.o.threads > 1:
            command += ["--threads", str(self.o.threads)]
        if self.o.tablebases:
            command += ["--tablebases", self.o.tablebases]
        if self.o.network:
            command += ["--network", self.o.network]
        if self.o.model:
            command += ["--model", self.o.model]
        if proofs:
            command += ["--proofs", proofs]
        return command

    def candidate(self, fen: str, proofs: str | None = None) -> list[str]:
        command = [
            self.o.bin, "prove", fen,
            "--seconds", str(int(self.o.seconds)),
            "--memory", str(self.o.memory),
            "--guide", self.o.candidate_guide,
            "--seed", str(self.o.candidate_seed),
            "--json",
        ]
        if self.o.candidate_evals:
            command += ["--evals", str(self.o.candidate_evals)]
        if proofs:
            command += ["--proofs", proofs]
        return command

    def run(self, command: list[str]) -> dict:
        started = time.perf_counter()
        proc = subprocess.run(
            command, capture_output=True, text=True,
            timeout=self.o.seconds + 120, check=False,
        )
        wall = time.perf_counter() - started
        payload = last_json(proc.stdout + proc.stderr)
        return {
            "wall_s": round(wall, 4),
            "returncode": proc.returncode,
            "found": bool(payload.get("found")),
            "mate": payload.get("mate"),
            "nodes": payload.get("nodes"),
            "child_evals": payload.get("child_evals"),
            "ms": payload.get("ms"),
            "error": payload.get("error"),
            "command": command,
        }


def factor_rows(rows: list[dict], repeats: int, cap_s: float,
                cells: dict[str, str]) -> dict:
    """Per-cell reference and candidate medians and the factor."""
    summary: dict = {}
    for row in rows:
        cell = summary.setdefault(row["id"], {
            "id": row["id"],
            "fen": cells[row["id"]],
            "cap_s": cap_s,
            "ref": {"walls_s": [], "found": 0, "mates": [], "nodes": []},
            "cand": {"walls_s": [], "found": 0, "mates": [], "child_evals": [],
                     "nodes": []},
        })
        side = cell["ref"] if row["side"] == "ref" else cell["cand"]
        side["walls_s"].append(row["wall_s"])
        if row["found"]:
            side["found"] += 1
            if row["mate"] is not None:
                side["mates"].append(row["mate"])
            if side is cell["ref"] and row["nodes"] is not None:
                side["nodes"].append(row["nodes"])
            if side is cell["cand"]:
                if row["child_evals"] is not None:
                    side["child_evals"].append(row["child_evals"])
                if row["nodes"] is not None:
                    side["nodes"].append(row["nodes"])

    for cell in summary.values():
        for side in ("ref", "cand"):
            walls = cell[side]["walls_s"]
            cell[side]["median_wall_s"] = round(statistics.median(walls), 4)
            cell[side]["spread"] = round(spread(walls), 4)
            mates = cell[side].pop("mates")
            cell[side]["mate"] = mates[-1] if mates else None
            nodes = cell[side].pop("nodes")
            cell[side]["nodes"] = nodes[-1] if nodes else None
            evals = cell[side].pop("child_evals", None)
            if evals is not None:
                cell[side]["child_evals"] = evals[-1] if evals else None
        ref = cell["ref"]
        cand = cell["cand"]
        cell["status"] = "censored"
        cell["factor"] = None
        cell["bound"] = None
        if ref["found"] == repeats and ref["spread"] < SPREAD_UNSTABLE:
            if cand["found"] == repeats:
                cell["status"] = "claimed"
                cell["factor"] = round(
                    cand["median_wall_s"] / ref["median_wall_s"], 4)
            else:
                cell["bound"] = round(cap_s / ref["median_wall_s"], 4)
        elif ref["found"] == repeats:
            cell["status"] = "unstable"
            cell["bound"] = round(cap_s / ref["median_wall_s"], 4)
    return summary


def summarize(summary: dict, cap_s: float) -> dict:
    claimed = []
    censored = []
    unstable = []
    bounds = {}
    for cell in summary.values():
        if cell["status"] == "claimed":
            claimed.append(cell["id"])
        elif cell["status"] == "unstable":
            unstable.append(cell["id"])
        else:
            censored.append(cell["id"])
            if cell["ref"]["found"] and cell["bound"]:
                bounds[cell["id"]] = cell["bound"]
    factors = [c["factor"] for c in summary.values() if c["factor"] is not None]
    ordered = sorted(summary.values(), key=lambda c: c["id"])
    return {
        "cells": ordered,
        "claimed": sorted(claimed),
        "censored": sorted(censored),
        "unstable": sorted(unstable),
        "coverage": f"{len(claimed)}/{len(summary)}",
        "geomean": round(geomean(factors), 4) if factors else None,
        "bounds": bounds,
    }


def nproc() -> int:
    """`P = nproc`: affinity and container CPU quota aware.

    Coreutils' `nproc` honours a cgroup CPU quota, which
    `sched_getaffinity` in Python does not, so ask it directly.
    """
    try:
        value = subprocess.run(
            ["nproc"], capture_output=True, text=True, check=True,
        ).stdout.strip()
        if value.isdigit() and int(value) > 0:
            return int(value)
    except (OSError, subprocess.SubprocessError):
        pass
    if hasattr(os, "sched_getaffinity"):
        return len(os.sched_getaffinity(0))
    return os.cpu_count() or 1


def env_record(o: argparse.Namespace, digests: dict) -> dict:
    rustc = subprocess.run(
        ["rustc", "--version"], capture_output=True, text=True, check=False,
    ).stdout.strip()
    record = {
        "date": datetime.now(timezone.utc).isoformat(),
        "host": platform.node(),
        "platform": platform.platform(),
        "nproc": nproc(),
        "python": platform.python_version(),
        "rustc": rustc,
        "provari": o.provari,
        "provari_version": subprocess.run(
            [o.provari, "--version"], capture_output=True, text=True,
            check=False,
        ).stdout.strip(),
        "provari_sha256": sha256(Path(o.provari)),
        "oracle_commit": REF_COMMIT,
        "candidate": o.bin,
        "candidate_sha256": sha256(Path(o.bin)) if o.bin else None,
        "cap_s": o.seconds,
        "memory_mb": o.memory,
        "repeats": o.repeats,
        "guide": o.guide,
        "candidate_guide": o.candidate_guide,
        "seed": o.seed,
        "threads": o.threads,
        "tablebases": o.tablebases,
        "network": o.network,
        "model": o.model,
        "digests": digests,
        "cwd": os.getcwd(),
    }
    return record


def data_digests(o: argparse.Namespace) -> dict:
    digests = {"network": NETWORK_SHA256, "model": MODEL_SHA256}
    if o.network:
        digests["network_used"] = sha256(Path(o.network))
        if digests["network_used"] != NETWORK_SHA256:
            fail(f"network sha256 mismatch: {digests['network_used']}")
    if o.model:
        digests["model_used"] = sha256(Path(o.model))
        if digests["model_used"] != MODEL_SHA256:
            fail(f"model sha256 mismatch: {digests['model_used']}")
    if o.tablebases:
        root = Path(o.tablebases)
        manifest = root / "3-4-5.atbw.md5"
        if not manifest.is_file():
            manifest = root / "3-4-5" / "3-4-5.atbw.md5"
        if not manifest.is_file():
            fail(f"tablebase manifest missing under {root}")
        digests["tablebase_manifest"] = gate(
            manifest, TB_MANIFEST_SHA256, "tablebase manifest")
    return digests


def check_gates(o: argparse.Namespace) -> None:
    if not Path(o.provari).is_file():
        fail(f"missing reference: {o.provari}")
    version = subprocess.run(
        [o.provari, "--version"], capture_output=True, text=True, check=False,
    ).stdout.strip()
    if version != REF_VERSION:
        fail(f"reference version {version!r} is not {REF_VERSION!r}")
    if o.bin and not Path(o.bin).is_file():
        fail(f"missing candidate: {o.bin}")
    if o.expect_provari_sha256:
        got = sha256(Path(o.provari))
        if got != o.expect_provari_sha256:
            fail(f"reference sha256 mismatch: {got} != {o.expect_provari_sha256}")
    if o.expect_candidate_sha256:
        got = sha256(Path(o.bin))
        if got != o.expect_candidate_sha256:
            fail(f"candidate sha256 mismatch: {got} != {o.expect_candidate_sha256}")


def write_set(path: Path, column: str, members: list[str], o: argparse.Namespace,
              cells: dict[str, str], kind: str = "strong", split: str = "strong") -> dict:
    ordered = sorted(members)
    if kind == "guard":
        held_out: list[str] = []
        strong = ordered
    else:
        held_out = [cell for index, cell in enumerate(ordered, start=1)
                    if index % 5 == 0]
        strong = [cell for cell in ordered if cell not in set(held_out)]
    data = {
        "column": column,
        "kind": kind,
        "cap_s": o.seconds,
        "memory_mb": o.memory,
        "guide": o.guide,
        "seed": o.seed,
        "threads": o.threads,
        "tablebases": o.tablebases or None,
        "corpus_sha256": CORPUS_SHA256,
        "high200_sha256": HIGH200_SHA256,
        "members": ordered,
        "strong": strong,
        "held_out": held_out,
        "cells": [{"id": cell, "fen": cells[cell]} for cell in ordered],
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        fail(f"{path} already exists: sets are never edited; a changed "
             f"command, N, cap or tablebase set makes a new -v2")
    path.write_text(json.dumps(data, indent=2) + "\n")
    return data


def load_set(path: Path, cells: dict[str, str], calibration: set[str],
             o: argparse.Namespace) -> dict:
    data = json.loads(path.read_text())
    if data.get("corpus_sha256") != CORPUS_SHA256:
        fail(f"{path}: corpus digest mismatch")
    if data.get("high200_sha256") != HIGH200_SHA256:
        fail(f"{path}: high200 digest mismatch")
    members = data.get("members", [])
    strong = data.get("strong", [])
    held_out = data.get("held_out", [])
    if sorted(strong + held_out) != sorted(members):
        fail(f"{path}: strong and held-out do not partition the members")
    unknown = [cell for cell in members if cell not in cells]
    if unknown:
        fail(f"{path}: cells outside the universe: {unknown[:3]}")
    excluded = [cell for cell in members if cell in calibration]
    if excluded:
        fail(f"{path}: calibration cells are never in a set: {excluded}")
    if data.get("kind") != "guard":
        expected = [cell for index, cell in enumerate(sorted(members), start=1)
                    if index % 5 == 0]
        if held_out != expected:
            fail(f"{path}: held-out is not every 5th member by cell id")
    for key, attr in (("cap_s", "seconds"), ("memory_mb", "memory"),
                     ("guide", "guide"), ("threads", "threads")):
        if data.get(key) != getattr(o, attr):
            fail(f"{path}: {key} {data.get(key)!r} != {getattr(o, attr)!r}")
    # The tablebase *set* is pinned by the manifest gate; the directory is
    # host-specific and may move between hosts.
    if bool(data.get("tablebases")) != bool(o.tablebases):
        fail(f"{path}: tablebases {data.get('tablebases')!r} != {o.tablebases!r}")
    return data


def cells_for(data: dict, cells: dict[str, str], split: str) -> list[dict]:
    ids = data["strong"] if split == "strong" else (
        data["held_out"] if split == "held-out" else data["members"])
    return [{"id": cell, "fen": cells[cell]} for cell in ids]


def mode_row(o: argparse.Namespace, runner: Runner, cells: dict[str, str],
             calibration: set[str], digests: dict) -> None:
    data = load_set(Path(o.set), cells, calibration, o)
    if o.expect_set_sha256:
        got = sha256(Path(o.set))
        if got != o.expect_set_sha256:
            fail(f"set sha256 mismatch: {got} != {o.expect_set_sha256}")
    selected = cells_for(data, cells, o.split)
    side = o.side
    out = Path(o.out)
    out.mkdir(parents=True, exist_ok=True)
    rows = []
    for cell in selected:
        for repeat in range(1, o.repeats + 1):
            sides = ["ref", "cand"] if side == "both" else [side]
            for which in sides:
                if which == "ref":
                    row = runner.run(runner.reference(cell["fen"]))
                else:
                    row = runner.run(runner.candidate(cell["fen"]))
                row.update({"id": cell["id"], "side": which, "repeat": repeat})
                rows.append(row)
                print(f"{cell['id']} {which} {repeat}/{o.repeats} "
                      f"{row['wall_s']:.3f}s found={row['found']}", flush=True)
    if side == "both":
        summary = summarize(
            factor_rows(rows, o.repeats, o.seconds, cells), o.seconds)
    else:
        summary = summarize_single(rows, side, o.seconds, cells)
    env = env_record(o, digests)
    env["cell_count"] = len(selected)
    env["split"] = o.split
    env["set_sha256"] = sha256(Path(o.set))
    env["set"] = str(Path(o.set).resolve())
    write_outputs(out, o, rows, summary, env, data)
    claims = claim_cells(o, runner, summary, out) if o.claim else None
    if claims is not None:
        (out / "claims.json").write_text(json.dumps(claims, indent=2) + "\n")
        print(json.dumps({"claims": {k: [c["id"] for c in v] for k, v in claims.items()}},
                         indent=2))


def summarize_single(rows: list[dict], side: str, cap_s: float,
                     universe: dict[str, str]) -> dict:
    per: dict = {}
    for row in rows:
        cell = per.setdefault(row["id"], {"walls_s": [], "found": 0, "mates": []})
        cell["walls_s"].append(row["wall_s"])
        if row["found"]:
            cell["found"] += 1
            if row["mate"] is not None:
                cell["mates"].append(row["mate"])
    cells = []
    for cell_id in sorted(per):
        cell = per[cell_id]
        claimed = cell["found"] == len(cell["walls_s"])
        cells.append({
            "id": cell_id,
            "fen": universe[cell_id],
            "side": side,
            "status": "claimed" if claimed else "censored",
            "walls_s": cell["walls_s"],
            "median_wall_s": round(statistics.median(cell["walls_s"]), 4),
            "spread": round(spread(cell["walls_s"]), 4),
            "found": cell["found"],
            "mate": cell["mates"][-1] if cell["mates"] else None,
            "cap_s": cap_s,
        })
    claimed = [c["id"] for c in cells if c["status"] == "claimed"]
    return {
        "side": side,
        "cells": cells,
        "claimed": claimed,
        "coverage": f"{len(claimed)}/{len(cells)}",
        "geomean": None,
        "bounds": {},
    }


def write_outputs(out: Path, o: argparse.Namespace, rows: list[dict],
                  summary: dict, env: dict, data: dict) -> None:
    (out / "results.json").write_text(json.dumps({
        "column": data.get("column"),
        "set": str(Path(o.set).resolve()) if o.set else None,
        "cap_s": o.seconds,
        "memory_mb": o.memory,
        "repeats": o.repeats,
        "rows": rows,
    }, indent=2) + "\n")
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    (out / "env.json").write_text(json.dumps(env, indent=2) + "\n")
    lines = [
        f"# {data.get('column')} — {o.mode}",
        "",
        f"- Set: `{Path(o.set).name if o.set else '(none)'}` "
        f"sha256 `{env.get('set_sha256', '-')}`",
        f"- Cap {o.seconds:g} s, memory {o.memory} MB, `N={o.threads}`, "
        f"repeats {o.repeats}",
        f"- Reference: `{env['provari_version']}` sha256 `{env['provari_sha256']}`",
        f"- Candidate: sha256 `{env['candidate_sha256']}`",
        f"- Guide: reference `{o.guide}`, candidate `{o.candidate_guide}`",
        f"- Coverage: `{summary.get('coverage')}`; "
        f"geomean: `{summary.get('geomean')}`",
        "",
        "| Cell | Reference [s] | Candidate [s] | Factor | Status | Ref mate | Cand mate |",
        "| --- | ---: | ---: | ---: | --- | ---: | ---: |",
    ]
    for cell in summary.get("cells", []):
        if "ref" in cell and "cand" in cell:
            ref = cell.get("ref", {})
            cand = cell.get("cand", {})
            lines.append(
                f"| `{cell['id']}` | {ref.get('median_wall_s', '-')} | "
                f"{cand.get('median_wall_s', '-')} | "
                f"{cell.get('factor') if cell.get('factor') is not None else '-'} | "
                f"{cell.get('status', '-')} | {ref.get('mate', '-')} | "
                f"{cand.get('mate', '-')} |")
        else:
            lines.append(
                f"| `{cell['id']}` | {cell.get('median_wall_s', '-')} | - | - | "
                f"found {cell.get('found')} | {cell.get('mate', '-')} | - |")
    (out / "README.md").write_text("\n".join(lines) + "\n")
    print(json.dumps({k: summary.get(k) for k in ("coverage", "geomean", "bounds")},
                     indent=2))


def claim_cells(o: argparse.Namespace, runner: Runner, summary: dict,
                out: Path) -> dict:
    """One proof run per claimed cell per side; both checkers on ours."""
    proofs = out / "proofs"
    (proofs / "reference").mkdir(parents=True, exist_ok=True)
    (proofs / "candidate").mkdir(parents=True, exist_ok=True)
    result: dict = {"reference": [], "candidate": []}
    only = summary.get("side")
    for cell in summary["cells"]:
        if cell["status"] != "claimed":
            continue
        if only in (None, "ref"):
            claim_reference(o, runner, cell, proofs, result)
        if only in (None, "cand"):
            claim_candidate(o, runner, cell, proofs, result)
    return result


def claim_reference(o: argparse.Namespace, runner: Runner, cell: dict,
                    proofs: Path, result: dict) -> None:
    fen = cell["fen"]
    ref_proof = proofs / "reference" / f"{cell['id']}.pvr"
    run = runner.run(runner.reference(fen, str(ref_proof)))
    verdict = None
    checked = None
    if ref_proof.is_file():
        check_cmd = [o.provari, "check", str(ref_proof)]
        if o.tablebases:
            check_cmd += ["--tablebases", o.tablebases]
        checked = subprocess.run(check_cmd, capture_output=True, text=True,
                                 check=False)
        verdict = last_json(checked.stdout + checked.stderr).get("valid")
    result["reference"].append({
        "id": cell["id"], "run": run, "proof": str(ref_proof),
        "verdict": verdict,
        "exit": checked.returncode if checked else None,
    })
    print(f"claim ref {cell['id']}: provari={verdict}", flush=True)


def claim_candidate(o: argparse.Namespace, runner: Runner, cell: dict,
                    proofs: Path, result: dict) -> None:
    fen = cell["fen"]
    cand_proof = proofs / "candidate" / f"{cell['id']}.pvr"
    run = runner.run(runner.candidate(fen, str(cand_proof)))
    ours = theirs = None
    if cand_proof.is_file():
        ours = subprocess.run([o.bin, "check", str(cand_proof)],
                              capture_output=True, text=True, check=False)
        theirs = subprocess.run([o.provari, "check", str(cand_proof)],
                                capture_output=True, text=True, check=False)
    result["candidate"].append({
        "id": cell["id"], "run": run, "proof": str(cand_proof),
        "ours": last_json(ours.stdout + ours.stderr).get("valid") if ours else None,
        "ours_exit": ours.returncode if ours else None,
        "theirs": last_json(theirs.stdout + theirs.stderr).get("valid") if theirs else None,
        "theirs_exit": theirs.returncode if theirs else None,
    })
    print(f"claim cand {cell['id']}: ours={result['candidate'][-1]['ours']} "
          f"provari={result['candidate'][-1]['theirs']}", flush=True)


def mode_select(o: argparse.Namespace, runner: Runner, cells: dict[str, str],
                calibration: set[str], digests: dict) -> None:
    eligible = [cell for cell in cells if cell not in calibration]
    eligible.sort()
    out = Path(o.out)
    out.mkdir(parents=True, exist_ok=True)
    members = []
    rows = []
    for cell_id in eligible:
        attempts = []
        joined = False
        for attempt in range(1, 4):
            row = runner.run(runner.reference(cells[cell_id]))
            row.update({"id": cell_id, "attempt": attempt})
            rows.append(row)
            attempts.append(row)
            print(f"select {cell_id} {attempt}/3 {row['wall_s']:.3f}s "
                  f"found={row['found']}", flush=True)
            found = [r for r in attempts if r["found"]]
            if attempt == 1 and row["found"] and row["wall_s"] < SLOW_PROVE_S:
                joined = True
                break
            if len(found) >= 2:
                joined = True
                break
            # Two failures already make 2-of-3 impossible; stop early.
            if len(attempts) - len(found) >= 2:
                break
        if joined or len([r for r in attempts if r["found"]]) >= 2:
            members.append(cell_id)
    data = write_set(Path(o.set_out), o.column, members, o, cells,
                     kind="strong")
    env = env_record(o, digests)
    env["mode"] = "select"
    env["column"] = o.column
    env["eligible"] = len(eligible)
    env["members"] = len(members)
    env["set_sha256"] = sha256(Path(o.set_out))
    (out / "env.json").write_text(json.dumps(env, indent=2) + "\n")
    (out / "selection.json").write_text(json.dumps({
        "column": o.column,
        "eligible": eligible,
        "members": sorted(members),
        "strong": data["strong"],
        "held_out": data["held_out"],
        "attempts": rows,
    }, indent=2) + "\n")
    print(json.dumps({"column": o.column, "eligible": len(eligible),
                      "members": len(members),
                      "strong": len(data["strong"]),
                      "held_out": len(data["held_out"])}, indent=2))


def mode_calibrate(o: argparse.Namespace, runner: Runner, cells: dict[str, str],
                   calibration: set[str], digests: dict) -> None:
    out = Path(o.out)
    out.mkdir(parents=True, exist_ok=True)
    nmax = min(8, nproc())
    choices = [n for n in (1, 2, 4, 8) if 1 <= n <= nmax]
    walls: dict[int, dict[str, float]] = {}
    rows = []
    for n in choices:
        o.threads = n
        walls[n] = {}
        for cell_id in sorted(calibration):
            runs = []
            for repeat in range(1, o.repeats + 1):
                row = runner.run(runner.reference(cells[cell_id]))
                row.update({"id": cell_id, "threads": n, "repeat": repeat})
                rows.append(row)
                runs.append(row["wall_s"])
                print(f"calibrate N={n} {cell_id} {repeat}/{o.repeats} "
                      f"{row['wall_s']:.3f}s found={row['found']}", flush=True)
            walls[n][cell_id] = statistics.median(runs)
    r = {n: geomean(list(walls[n].values())) for n in choices}
    chosen = 1
    previous = choices[0]
    for n in choices:
        if n == choices[0]:
            continue
        if r[n] is not None and r[1] and r[previous] is not None:
            if r[n] / r[1] <= 0.8 and r[n] / r[previous] <= 0.9:
                chosen = n
        previous = n
    # The row's N is the choice, not the last probe.
    o.threads = chosen
    env = env_record(o, digests)
    env.update({
        "mode": "calibrate",
        "nmax": nmax,
        "threads_list": choices,
        "r": {str(n): (round(r[n], 4) if r[n] else None) for n in choices},
        "median_walls": {str(n): {k: round(v, 4) for k, v in walls[n].items()}
                         for n in choices},
        "chosen_n": chosen,
    })
    (out / "env.json").write_text(json.dumps(env, indent=2) + "\n")
    (out / "calibration.json").write_text(json.dumps(env, indent=2) + "\n")
    (out / "runs.json").write_text(json.dumps(rows, indent=2) + "\n")
    print(json.dumps({"nproc": env["nproc"], "nmax": nmax,
                      "r": env["r"], "chosen_n": chosen}, indent=2))


def args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("row", "select", "calibrate"),
                        default="row")
    parser.add_argument("--side", choices=("ref", "cand", "both"), default="both")
    parser.add_argument("--provari", required=False,
                        default="/tmp/470M1-oracle/provari/target/release/provari")
    parser.add_argument("--bin", default="target/release/470MI")
    parser.add_argument("--corpus", default=str(CORPUS))
    parser.add_argument("--high200", default=str(HIGH200))
    parser.add_argument("--set", default=None)
    parser.add_argument("--set-out", default=None)
    parser.add_argument("--column", default=None)
    parser.add_argument("--split", choices=("strong", "held-out", "members"),
                        default="strong")
    parser.add_argument("--out", required=True)
    parser.add_argument("--seconds", type=float, default=10.0)
    parser.add_argument("--memory", type=int, default=192)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--guide", default="model")
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--tablebases", default=None)
    parser.add_argument("--network", default=None)
    parser.add_argument("--model", default=None)
    parser.add_argument("--seed", type=int, default=0)
    parser.add_argument("--candidate-guide", default="hand")
    parser.add_argument("--candidate-seed", type=int, default=0)
    parser.add_argument("--candidate-evals", type=int, default=None)
    parser.add_argument("--expect-set-sha256", default=None)
    parser.add_argument("--expect-provari-sha256", default=None)
    parser.add_argument("--expect-candidate-sha256", default=None)
    parser.add_argument("--claim", action="store_true")
    return parser.parse_args()


def main() -> None:
    o = args()
    check_gates(o)
    cells, calibration = universe(Path(o.corpus), Path(o.high200))
    if o.mode == "select":
        if not o.column or not o.set_out:
            fail("--mode select needs --column and --set-out")
        digests = data_digests(o)
        mode_select(o, Runner(o), cells, calibration, digests)
        return
    if o.mode == "calibrate":
        if not o.tablebases and o.threads == 1:
            pass
        digests = data_digests(o)
        mode_calibrate(o, Runner(o), cells, calibration, digests)
        return
    if not o.set:
        fail("--mode row needs --set")
    digests = data_digests(o)
    mode_row(o, Runner(o), cells, calibration, digests)


if __name__ == "__main__":
    main()
