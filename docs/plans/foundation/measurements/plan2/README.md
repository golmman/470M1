# Plan 2 measurements — base vs cand, guard set

Same session, host container `0f28967c792a`,
`Linux-7.2.7-200.fc44.aarch64-aarch64-with-glibc2.43`, `nproc` 4,
rustc 1.100.0. One process at a time.

| Binary | sha256 | Location |
| --- | --- | --- |
| base: `f7ba48a` release (`git archive` into `/tmp/470M1-base`) | `25cc08d8…427fee` (= plan1 candidate) | tmpfs |
| cand: plan 2 tree release | `fcb39e6b…5229b01` | overlay (`target/release`); tmpfs copy `/tmp/470M1-cand-copy` |

| Path | What |
| --- | --- |
| `guard/` | gate-4 row as pre-registered: `run_bench.py --mode row --set bench/sets/guard.json --side cand --baseline-bin BASE --repeats 5 --seconds 10 --memory 192 --guide hand --candidate-guide hand --candidate-seed 0 --threads 1 --claim`; geomean 1.0114 (miss) |
| `guard-rerun/` | the one same-session rerun, identical command; geomean 1.0078 (decides: pass) |
| `guard/cmp.txt` | the three claimed `.pvr` vs plan1's `guard/proofs/candidate/`: identical |
| `guard-tmpfs/` | diagnostic, not a gate: same command without `--claim`, `--bin /tmp/470M1-cand-copy` (both binaries on tmpfs); geomean 0.9926 |
| `diag.py` | the `Ir` and startup instrument |
| `cachegrind.json`, `startup.json` | `diag.py BASE target/release/470MI` (cand on overlay) |
| `tmpfs/` | `diag.py BASE /tmp/470M1-cand-copy` outputs (both on tmpfs) |
| `binaries.sha256`, `scope.txt`, `test-full.txt` | identity, scope diff, `make test-full` tail |

Location finding: `perf stat -r 30 BIN --version` gives 10 major faults and
~1.48 ms for cand on overlay against 0 and ~0.47 ms for the same bytes on
tmpfs (base on tmpfs 0.42 ms; base copied to overlay 1.07 ms, 4 faults). The
pre-registered rows therefore charge cand a filesystem cost that base does
not pay; `guard-tmpfs/` and `tmpfs/` remove it.
