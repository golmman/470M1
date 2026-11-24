# guard — row

- Set: `guard.json` sha256 `199c03ab144958e59a35669e4b59f55807e3d683493d15bdfd6439abbcf60bfb`
- Cap 10 s, memory 192 MB, `N=1`, repeats 3
- Reference: `provari 0.5.0, with its network and model built in` sha256 `3cd1a4ecf4aed82ca3b37f1e4a5af55d9457a0b1f4b44dfb8cb5abebe7118842`
- Candidate: sha256 `25cc08d869699f2ec061afabc8a3539c37dce53f3e8fe8e959f1500bcf427fee`
- Guide: reference `hand`, candidate `hand`
- Coverage: `3/3`; geomean: `1.0228`

| Cell | Reference [s] | Candidate [s] | Factor | Status | Ref mate | Cand mate |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| `1eb80b92f92c` | 0.11 | 0.1356 | 1.2327 | claimed | 15 | 18 |
| `core` | 0.4858 | 0.5113 | 1.0525 | claimed | 17 | 14 |
| `db6a741c723f` | 6.2357 | 5.1417 | 0.8246 | claimed | 22 | 20 |
