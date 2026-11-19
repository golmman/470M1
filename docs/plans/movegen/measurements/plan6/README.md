# Plan 6 measurements — the non-rules residual

**Date:** 2026-11-19. **Host:** `c90face26537` (x86_64, Linux 7.2.13, 4 vCPU).
**Pin:** `atomic-movegen` 2.10.0 (`5083647`), shakmaty 0.30.1. **Tree:**
`470MI@1aba046` (`atomic470mi` 0.1.0), unchanged at the end of the session.
**Outcome:** the split is resolved; `NodeFacts` (C1) passed the adoption rule
and then failed its falsifier, so nothing was adopted. No wall-clock bench and
no ledger row: the tree did not change.

`split/` is Phase A: the bucket map, the counted instrument, the two parsed
`perf report` symbol tables, and `split.json`. `c1/attempt.patch` is the
falsified formulation change. `core-perf.txt` holds every `perf stat` line
this session read from.

## Binaries

| Side | Path | sha256 | Note |
| --- | --- | --- | --- |
| 2.10.0 | `target/release/470MI` | `14093d41…` | HEAD `1aba046`, rebuilt after the revert |
| plan 2 | `target/release/470m1` | `71fff646…` | `2f629fd`, verified bit-identical in plan 5 |

The 2.10.0 binary was rebuilt from the reverted tree and is **bit-identical**
to the plan 5 binary, so plan 5's wall row still stands for this tree.

## Phase A — instruments

1. **Counted build** (temporary, `split/counts.patch`): 12 pre-registered
   sites plus three supplementary entry points (`mover_legal`,
   `attackers_to`, `context_legal`) counted with relaxed `AtomicU64`s, and a
   `--counts` field on `prove --json`. Built in a scratch copy of the tree,
   never committed. Counters on `core` are identical with and without it:
   `child_evals` 3,896,430, `expansions` 191,557, `nodes` 350,430, mate 14.
2. **`perf record -e instructions:u -c 500000`** on the unchanged binaries,
   `perf report --sort symbol --show-total-period`, bucketed by
   `split/bucket_map.py`. Symbol totals 7.9965e9 / 6.5640e9, i.e. −0.09% and
   −0.08% of `perf stat` (8.0041e9 / 6.5694e9). **Reconciliation passes** at
   10× tighter than the plan's ±3% / ±5% budgets.

```
cd docs/plans/movegen/measurements/plan6/split
cargo build --release                      # in the scratch counted tree
./target/release/470MI prove "$CORE" --seconds 10 --memory 192 --seed 0 --json --counts
perf record -q -o 2100.data  -e instructions:u -c 500000 target/release/470MI prove ...
perf record -q -o plan2.data -e instructions:u -c 500000 target/release/470m1 prove ...
perf report -i 2100.data --stdio -g none --sort symbol --show-total-period
python3 run_split.py                       # writes split.json
```

## Phase A — the split

Buckets, instructions per `core` run (2.10.0 / plan 2 / delta):

| Bucket | 2.10.0 | plan 2 | Δ | aligned Δ |
| --- | ---: | ---: | ---: | ---: |
| rules | 1.9830e9 | 1.3660e9 | **+0.6170e9** | +0.8290e9 |
| geometry | 2.2305e9 | 1.8720e9 | **+0.3585e9** | +0.1465e9 |
| generation | 0.6620e9 | 0.5365e9 | +0.1255e9 | +0.1255e9 |
| play + memmove | 0.3370e9 | 0.2795e9 | +0.0575e9 | +0.0575e9 |
| search/table | 2.6070e9 | 2.3565e9 | +0.2505e9 | +0.2505e9 |
| null | 0.0655e9 | 0.0615e9 | +0.0040e9 | +0.0040e9 |
| adapter screens | 0.1115e9 | 0.0920e9 | +0.0195e9 | +0.0195e9 |

Per site, from the counted build and the symbol table:

| Site | Calls | Instructions | Per call |
| --- | ---: | ---: | ---: |
| `context_legal` (rules) | 10,012,184 | 1.2100e9 | **120.9** |
| `Mover::legal` (entry) | 15,171,649 | — | — |
| effect (crate + blast + rights) | 4,779,358 | 0.6325e9 | **132.3** |
| `Mover::new` (+ `populate_state`) | 578,396 | 0.3230e9 | **558.4** |
| `pseudo_legal` (generation) | 578,396 | 0.6620e9 | **1144.5** |
| `plain` | 6,630,358 | 0.2820e9 | 42.5 |
| `exposes` | 4,745,979 | 0.3205e9 | 67.5 |
| `gives_check` | 3,358,938 | 0.1645e9 | 49.0 |
| `threats` | 987,717 | 0.0595e9 | 60.2 |
| `terms` | 383,114 | 0.3550e9 | 926.6 |
| `play` | 949,651 | 0.1585e9 | 166.9 |
| `replay_threat` (null) | 44,734 | 0.0655e9 | — (child scan included) |
| `attackers_to` | 2,933,157 | 0.2120e9 | 72.3 |

`120.9` per legality call reproduces plan 5's `W7` (121.9) from the whole
binary, which is the cross-check that the counted sites and the symbol table
describe the same work. Plan 5's `W8` (215.2) is a probe-harness figure: the
binary splits the effect path over `child_effect_with_occupancy` (98.1) plus
`blast_casualties` and `castling_rights_after` (132.3 together).

**Decision input.** Total gap 1.4347e9; tree volume +1.9% `child_evals` =
0.1249e9; rules +0.6170e9; **non-rules residual 0.6928e9**. Geometry is
+0.3585e9 = **51.7%**, so the plan's adoption rule opens. Under the inlining
aligned map — `Board::attackers_to` is its own symbol only in 2.10.0, in plan 2
its counterpart sits inside `checked_after` — geometry is +0.1465e9 = 30.5%
and the rule would shut. Σ bucket deltas 1.4325e9 against the 1.4347e9 gap
(−0.15%). Both readings are in `split.json`; C1 was attempted
under the primary one and settled by measurement rather than by the map.

## Phase B — C1 `NodeFacts`, falsified

`c1/attempt.patch`: one `NodeFacts` (occupied, two colour masks, two kings,
six role masks) built per expanded node from `Board`'s public bitboards, passed
by reference to `Mover` and `eval::terms`; the candidate queries read it.
Legality and effect stay direct crate calls (`MD5`). Built clean, tests clean,
counters unchanged — and **+0.0382e9 instructions** (three interleaved runs
each side, `core-perf.txt`): the accessors it removes are `#[inline(always)]`
field reads, while building the 88-byte value once per `Mover::new` (578,396
runs) costs more than they save. The falsifier wanted −0.15e9. C1 dropped and
reverted; `target/release/470MI` is bit-identical to plan 5's.

## What was not run, and why

No `bench/` directory: the tree did not change, so there is no candidate to
bench and no `movegen plan6` row (plan 4 precedent). Rules layer, `.pvr`
format, `check.rs`, search/eval/threat ordering, seeds and counters were not
touched. Gates re-run on the reverted tree: `cargo fmt --check`, clippy, test,
doc, `make test-full` and the full differential (plan 5 counts to the digit).
No raw transcripts, `perf.data`, oracle artifacts or `__pycache__` are
committed.
