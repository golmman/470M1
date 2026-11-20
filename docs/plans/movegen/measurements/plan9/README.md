# Plan 9 measurements — evasion mask (`ev`) and `unsafe-optimizations` (`uo`)

**Status:** bench-host run complete, **release** branch for `ev`
(`wishlist3` `W10`, `MD8`); `uo` is a material win recorded for an owner
decision. Plan: [`../../plan9.md`](../../plan9.md); report:
[`../../report9.md`](../../report9.md).

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

## Bench-host run (2026-11-20, `c90face26537`, x86_64)

`make oracle` built pinned provari `bceb1ee` (0.4.0); `make test` was green
before and after. Three `perf stat -e instructions:u` rounds per cell, order
`base`, `ev`, `uo`, `--seconds 10 --memory 192 --seed 0 --json`, no export.
The `base` binary is sha256 `b7c16c94…` — the plan 8 bench-host binary — and
`core` reads 7,816,219,508 (spread 672), reproducing plan 8's 7.8162193e9.
The driver's `bench_host`/`provisional` flags are a `socket.gethostname()`
equality and read `false`/`true` inside this container (UTS name
`36f35886a6b3`); host identity rests on the binary hash, the platform string
and the base reading.

| Cell | base | `ev` | `ev` fall | `uo` | `uo` fall | `uo` fall % | spreads (base/`ev`/`uo`) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `core` | 7,816,219,508 | 7,091,614,362 | **724,605,146** (9.271%) | 7,713,283,724 | 102,935,784 | 1.317 | 672 / 296 / 650 |
| `easy-win` | 1,640,875,322 | 1,556,631,861 | 84,243,461 (5.134%) | 1,619,316,966 | 21,558,356 | 1.314 | 251 / 652 / 636 |
| `deep-win` | 82,738,904,410 | 77,443,772,287 | 5,295,132,123 (6.400%) | 81,680,942,381 | 1,057,962,029 | 1.279 | 635 / 456 / 359 |

`core` `ev` pair falls are 724,605,146 / 724,605,024 / 724,604,648. `uo` is
material on every cell (>3× the larger spread, ≥0.5%). Both variants:
differential `mismatches: 0`, `key_mismatches: 0`, EP 87/0/0; counters/mate
identical in every round; `.pvr` byte-identical on all three cells; our
checker and `provari check` accept 9/9. `decision.json` branches `ev` to
**release**; `uo` is `owner decision` (enable in `Cargo.toml` with a
`PROVENANCE.md` note and a feature-test gate, or keep defaults).

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
