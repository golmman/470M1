# Measurements — `parity`

One directory per executed plan: `planN/`. The committed record is the
driver, the parsed results, `env.json` and a short `README.md` with
provenance. Raw transcripts, oracle sources/binaries/proofs and
`__pycache__` are never committed.

## Layout

```
planN/
  README.md      provenance: what ran, on which host, with which pins
  env.json       host, CPU, memory, OS, binary sha256, oracle version
  *.py, *.rs     drivers and probes
  *.json, *.csv  parsed results
```

## Rules

- One process at a time for wall measurements.
- Same-session reference and candidate runs; report medians of three repeats.
- Record the cap, coverage and geomean together.
- A censored cell is `>= cap / reference`, never a factor.
- Record proof length beside the same-session reference `mate`.
- The ledger is [`../../../bench.md`](../../../bench.md); a measurement
  directory is the evidence, not a second ledger.
