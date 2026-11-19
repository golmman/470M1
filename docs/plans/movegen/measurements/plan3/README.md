# Plan 3 measurements — rename and the `M1` adapter

**Date:** 2026-11-19. **Host:** `c90face26537` (x86_64, Linux 7.2.13).
**Harness:** `tests/differential.rs`, pin `atomic-movegen` `2.9.0`
(`af3faf0`), shakmaty `0.30.1`. **Bench:** product default (seed 0), 10 s,
192 MB, protocol `a`, three repeats. **Outcome:** full sweep green, 3/3 cells,
9/9 accepted, `D14` counters identical; geomean **1.206**, a regression
against the plan 2 row's **0.972**.

`results.json` is the measured adapter row. `old.json` is the same-session
diagnostic of the plan 2 binary (`target/release/470m1`, never committed),
measured after the claim with the same driver and protocol. It isolates the
host and the adapter: the old binary reads 0.998 / 1.007 / 0.958, geomean
0.987, while the adapter reads 1.352 / 1.250 / 1.038.

## Commands

```
cargo test --test differential fen_acceptance_and_rules_smoke -- --nocapture
cargo test --test differential rules_differential_full -- --ignored --nocapture
cargo build --release
```

```
python3 docs/plans/movegen/measurements/plan3/run_bench.py \
  --provari /tmp/470M1-oracle/provari/target/release/provari \
  --candidate target/release/470MI \
  --out docs/plans/movegen/measurements/plan3/results.json \
  --seconds 10 --reference-seconds 10 --memory 192 \
  --repeats 3 --seeds 0 --protocols a --d14
```

The diagnostic was the same command with
`--candidate target/release/470m1 --out /tmp/plan3-old-diagnostic.json`,
copied to `old.json`; it does not carry its own `env.json`, so the host and
build settings are those in `env.json` (the binary predates this tree). No raw
transcripts or oracle artifacts are committed.

## Full sweep

Fixture/corpus FENs, generated acceptance mutations, the five en-passant
fixtures, 3 seeds x 128 games x <= 200 plies, and the EP play-out sweep.

| Count | Plan 3 |
| --- | ---: |
| FEN accepted / rejected | 84 / 11 |
| Positions compared | 27,088 |
| Legal moves / child keys compared | 540,383 |
| Play-out games / plies | 392 / 26,612 |
| Immediate explosions present | 72,131 |
| Decided children skipped | 6,684 |
| Material exemptions | 642 |
| Immediate-explosion mismatches | **0** |
| `Mover::exposes` mismatches | **0** |
| Node-level explosion mismatches | **0** |
| Key mismatches (native key vs movegen `Legal`) | **0** |
| `MD3` `ep_child` / `ep_threats_verdicts` / `ep_threats_difference` | 87 / **0** / **0** |
| Other rules mismatches | **0** |

Two full runs gave identical counts. The three EP counters are all zero.

## Bench — protocol `a`, same session

| Cell | Reference wall | Candidate wall | Factor | Mate / ref mate | `child_evals` | Accepted |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.528 s | 0.714 s | **1.352** | 14 / 17 | 3,896,430 | 3/3 |
| `easy-win` | 0.136 s | 0.170 s | **1.250** | 18 / 15 | 904,368 | 3/3 |
| `deep-win` | 6.618 s | 6.868 s | **1.038** | 20 / 22 | 44,828,946 | 3/3 |
| **Geomean** | | | **1.206** | | | **9/9** |

Coverage 3/3. `D14`: the untimed export repeated the timed counters for all
three cells, and both checkers accepted every proof. Per-cell delta to the
plan 2 row (cross-session, not a factor): +0.371 / +0.272 / +0.080; geomean
+0.234.

## Same-session old-binary diagnostic

| Cell | Reference wall | Old candidate wall | Old factor | Old mate / ref | Old `child_evals` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 0.539 s | 0.538 s | **0.998** | 17 / 17 | 3,823,720 |
| `easy-win` | 0.135 s | 0.136 s | **1.007** | 15 / 15 | 897,937 |
| `deep-win` | 6.683 s | 6.402 s | **0.958** | 19 / 22 | 54,633,842 |
| **Geomean** | | | **0.987** | | |

The adapter's `deep-win` tree is 18% cheaper in `child_evals` yet 3.8% slower
in wall time; `core` and `easy-win` are 25–35% slower. The regression is in
the adapter's per-node cost, not in host drift or in proof export (the
reported `ms` stops before export). A profile points at the native board's
movegen/legality/hash primitives: `pseudo_legal`, `checked_after`, and the
effect key. The risk fallback (`hash_and_rule50_after` once, plus a Legal
correction only for a double push) is already in `Position::effect`; it did not
close the gap.
