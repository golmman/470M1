# guard — row

- Set: `guard.json` sha256 `199c03ab144958e59a35669e4b59f55807e3d683493d15bdfd6439abbcf60bfb`
- Cap 10 s, memory 192 MB, `N=1`, repeats 5
- Reference: `provari 0.5.0, with its network and model built in` sha256 `3cd1a4ecf4aed82ca3b37f1e4a5af55d9457a0b1f4b44dfb8cb5abebe7118842`
- Candidate: sha256 `fcb39e6b36afa2e6dc9aa84df531412fd68059c059518645f32720af65229b01`
- Guide: reference `hand`, candidate `hand`
- Coverage: `3/3`; geomean: `1.0078`

| Cell | Reference [s] | Candidate [s] | Factor | Status | Ref mate | Cand mate |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| `1eb80b92f92c` | 0.0785 | 0.082 | 1.0446 | claimed | 18 | 18 |
| `core` | 0.3643 | 0.3589 | 0.9852 | claimed | 14 | 14 |
| `db6a741c723f` | 3.8214 | 3.8007 | 0.9946 | claimed | 20 | 20 |
