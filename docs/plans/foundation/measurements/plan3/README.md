# Plan 3 measurements — library errors and the invariant audit

Same session, host container `6f38b7cbff1e`,
`Linux-7.3.4-401.asahi.fc44.aarch64+16k-aarch64-with-glibc2.43`, `nproc` 4,
rustc 1.100.0. One process at a time.

| Binary | sha256 | Location |
| --- | --- | --- |
| base: 96c40da release (`git archive` into `/tmp/470M1-base`) | `fcb39e6b…5229b01` | `/tmp/470M1-plan3-bin/base` |
| cand: plan 3 tree release | `64bf0dee…8821c6e` | `/tmp/470M1-plan3-bin/cand` |

Both binaries were copied into the fresh directory `/tmp/470M1-plan3-bin` on
one filesystem before the row (plan 2's location finding), and the row used
those copies. `binaries.sha256` records them.

| Path | What |
| --- | --- |
| `guard/` | gate-4 row as pre-registered: `run_bench.py --mode row --set bench/sets/guard.json --side cand --baseline-bin /tmp/470M1-plan3-bin/base --bin /tmp/470M1-plan3-bin/cand --repeats 5 --seconds 10 --memory 192 --guide hand --candidate-guide hand --candidate-seed 0 --threads 1 --claim`; geomean **0.9782** (pass, `<= 1.01`) |
| `guard/cmp.txt` | the three claimed `.pvr` vs plan 2's `guard/proofs/candidate/`: byte-identical |
| `guard/claims.json` | ours and `provari check` accept all three; no control here |
| `cachegrind.json` | `diag.py /tmp/470M1-plan3-bin/base /tmp/470M1-plan3-bin/cand`: `core` `Ir` Δ **−0.000001%** (`<= +0.5%`); mate-in-1 Δ −0.0005% |
| `startup.json` | median of 20 interleaved mate-in-1 walls: base 12.825 ms, cand 11.810 ms (ratio 0.921), diagnostic only |
| `diag.py` | the `Ir` and startup instrument, copied from plan 2 with its raw path changed |
| `test-full.txt`, `test.txt`, `doc.txt` | gate output tails; `make test-full` passed, including the veto (9/9 accepted, control rejected) |
| `scope.txt`, `binaries.sha256` | identity and scope |

## Gate 2 — counters and determinism

Every completed search in the row (5 base + 5 cand per cell) has identical
`mate` / `nodes` / `child_evals` / `expansions`, the plan 2 product:

| Cell | `mate` | `nodes` | `child_evals` | `expansions` |
| --- | ---: | ---: | ---: | ---: |
| `core` | 14 | 350,430 | 3,896,430 | 191,557 |
| `1eb80b92f92c` | 18 | 68,692 | 904,368 | 53,237 |
| `db6a741c723f` | 20 | 3,553,009 | 44,828,946 | 2,487,550 |

`counters_identical` is true for all three cells in `summary.json`; every run
in `results.json` shows the same numbers.

## Gate 4 — medians

| Cell | base [s] | cand [s] | Factor |
| --- | ---: | ---: | ---: |
| `1eb80b92f92c` | 0.1254 | 0.1157 | 0.9226 |
| `core` | 0.5128 | 0.5144 | 1.0031 |
| `db6a741c723f` | 5.2058 | 5.2646 | 1.0113 |
| **Geomean** | | | **0.9782** |

Spreads are 19.5% / 3.5% / 7.4% on base and 3.5% / 4.5% / 3.1% on cand; the
`easy-win` base spread comes from the first repeat and is the only one near
the driver's stability threshold. The gate passed on the pre-registered row,
so there was no rerun.
