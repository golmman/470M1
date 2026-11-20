# Plan 9 measurements — evasion mask (`ev`) and `unsafe-optimizations` (`uo`)

**Status:** driver ready, bench-host run pending. Plan: [`../../plan9.md`](../../plan9.md).

## Files

- `run_plan9.py` — the driver. It builds `base`/`ev`/`uo` from `git archive
  HEAD` under `--scratch` (default `/tmp/plan9`) and runs the ignored
  differential in `ev`/`uo`. It then runs three interleaved rounds of `perf stat
  -e instructions:u` on `core`/`easy-win`/`deep-win`, in the order `base`,
  `ev`, `uo`. Each variant exports one `.pvr` per cell, which is checked by our
  checker and by `provari check` if `/tmp/470M1-oracle` exists. The driver
  writes `perf.txt`, `results.json`, `env.json` and `decision.json` here.
  `--no-perf` checks the plumbing only.
- `ev.patch` — the `/tmp/mg-ev` shape on `src/moves.rs`. It is a diagnostic
  only, because `MD5` forbids prover-side legality.
- `uo.patch` — `Cargo.toml` with `features = ["unsafe-optimizations"]`. The
  lockfile is unchanged.

## Plan-session census (seed 0, instrumented `Mover::legal`, `/tmp/evasion-probe`)

The counters below are `PROBE[0..20]`. Call counts do not depend on the
architecture.

| Cell | in-check oracle | quiet non-king | mask rejects (legal) | castle/EP (legal) | `Mover::new` in check | single-slider / double |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 8,220,516 | 6,128,196 | 5,882,026 (0) | 116,128 (0) | 337,911 | 307,515 / 1,135 |
| `easy-win` | 1,038,152 | 778,190 | 761,648 (0) | 444 (0) | 50,380 | 37,675 / 90 |
| `deep-win` | 61,347,947 | 47,679,180 | 45,072,205 (0) | 470,483 (21) | 2,704,490 | 2,351,864 / 2,136 |

## Plumbing check (aarch64 session host `44ef11c9218f`, no PMU, `--no-perf --rounds 1`)

This run produces no instruction numbers. Its correctness result:

- The differential reports `mismatches: 0` and `key_mismatches: 0` for both
  `ev` and `uo` (27,088 positions, 540,383 moves).
- Counters and mate are identical to `base` on all three cells.
- The `.pvr` is byte-identical on all three cells. `core` is `714723211c7a0246…`,
  the same as plan 8. Our checker accepts all 9 proofs; provari is absent on
  that host.
