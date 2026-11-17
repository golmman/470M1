# plan1 measurements — reference side

Plan1 has no candidate search; these are the pinned provari reference values
measured in one session, one process at a time. They are not a ledger row.

| Item | Value |
| --- | --- |
| Binary | provari `v0.4.0`, `bceb1ee1faf7d9cf2422eb89c931be69ae30619d` |
| Command | `provari prove FEN --seconds 10 --memory 192 --guide hand --json` |
| Repeats | 3 per cell |
| Corpus | `corpus-v1`, sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e` |
| Driver | `run_bench.py` (adapted from atomi, reference side only) |

| Cell | id | median wall | mate | nodes | wins |
| --- | --- | ---: | ---: | ---: | ---: |
| `core` | `core` | 0.342 s | 17 | 342,289 | 3/3 |
| `easy-win` | `1eb80b92f92c` | 0.075 s | 15 | 72,523 | 3/3 |
| `deep-win` | `db6a741c723f` | 4.721 s | 22 | 4,574,912 | 3/3 |

Raw parsed rows are in `results.json`; host and tool versions in `env.json`.
No ledger row is appended: there is no candidate search yet.
