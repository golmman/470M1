# Ideas — post-`parity`/`movegen` initiatives

Brainstorm assessment, 2026-11-20. Read-only probes only; no repo source
changes. Scratch clones/probes lived under `/tmp`.

The original goal is met: the ledger row is **1.001** vs `provari --guide
hand`, 3/3 at 10 s, 9/9 accepted. The first question is therefore not “which
optimisation?” but **what is the next fixed point?** The candidates below
imply different goals:

- **absolute wall time** on the existing 3 cells,
- **parity against provari’s strongest guide** (not hand),
- **coverage** on a wider corpus,
- **batch/incremental proving** rather than one-shot latency.

## What was measured

Same host, same session, 10 s, 192 MB, one process at a time, provari
`--threads 1` unless stated.

| Lever | `core` | `easy-win` | `deep-win` | Geomean effect |
| --- | ---: | ---: | ---: | ---: |
| provari hand (reference) | 0.50 s | 0.107 s | 6.55 s | 1.00 |
| provari **model** guide | 0.325 s | 0.034 s | 1.61 s | **~0.37 (2.7× faster)** |
| provari **network-only** guide | 0.89 s | 0.050 s | 3.87 s | ~0.79 |
| provari hand, **4 threads** | 0.206 s | 0.032 s | 2.47 s | **~0.36 (2.75× faster)** |
| provari hand + 3-4-5 tablebases | 0.49 s | 0.11 s | 6.60 s | **no change, identical nodes** |

Coverage census on `core + sample20` at 10 s:

| Guide | Proved |
| --- | ---: |
| our prover, hand | 3/21 |
| provari, hand | 3/21 |
| provari, model | **5/21** (`b9a7ed7eb368`, `dd3f89766a40` newly proved) |

Other probes:

- Seed portfolio is weak: 8 seeds on `deep-win` gave min 5.37 s, median
  7.04 s. A 4-way independent portfolio buys ~1.2× for 4× the cores — worse
  than Lazy SMP.
- Memory split matters more than TT structure: `deep-win` at 10 s was 4.92 s
  at 64 MB, 5.37 s at 192 MB, **7.3 s at 128 MB**. That is a config/tree
  effect, not evidence for a TT rewrite.
- Zeroing parts of provari’s model showed the network is load-bearing on
  `deep-win`: full model 1.61 s, no layer weights 2.12 s, no `net` feature
  weight 2.21 s. A tiny hand-feature-only model is not obviously equivalent.

---

## Ranked initiatives

### 1. `guided` — learned guide (NNUE + provability model) — **recommended first**

**Goal:** adopt a learned guide as an opt-in, then as the product default if
it passes gates: existing 3 cells at 10 s, plus the two model-only corpus
cells, with every proof accepted by both checkers.

**Why yes:** this is the only lever with a measured ~2.7× wall reduction *and*
a coverage increase on this host. It changes the tree, not just per-node cost,
and it does not touch soundness or proof checking. It also gives the project a
new goal that is not “shave the last 0.1% against hand”.

**Scope:** port provari’s `model.rs` and the first-layer part of `nnue.rs`
(GPL, provenance row), plus the network/model data with SHA-256 and a licence
review. Start model-only as a flag; `--guide hand` must remain byte-identical.
Optimise inference after the first gate; the model is only useful if its
per-node cost stays below the tree gain.

**Estimated gain:** ~2.5–3× on shared cells, coverage 3/21 → 5/21. Caveat: this
is provari’s model on provari’s search; our port must re-measure.

**Cheapest decisive experiment:** a scratch `--guide model` on `deep-win`, one
seed, 10 s. Pre-register: if `deep-win` is not below ~2.5 s with 9/9 accepted
and identical counters on repeat, stop before a full integration.

**Fairness note:** compare candidate model vs provari model, and keep
hand-vs-hand as a separate ledger column. Adding model-only cells to the
*existing* 3-cell set would silently change the goal.

---

### 2. `parallel` — Lazy SMP / shared-memory DF-PN — **yes, second, but only under an absolute-wall goal**

**Goal:** absolute wall time with `--threads N`, not equal-thread parity. Keep
`--threads 1` deterministic and unchanged as the product default until the
threaded mode passes its own gates.

**Why yes:** the upper bound is measured on our exact bench: 2.6–3.3× on 4
vCPU. Our table and proofs are still provari-v0.1-style single-threaded;
provari v0.4’s shared table/proofs/pause machinery is the reference
implementation and is GPL-copyable with provenance.

**Why not first:** it is the highest-risk change. It touches the table,
proofs, filter, mate cache and determinism contract. Equal-thread comparison
against provari will roughly cancel the gain; the honest claim is “faster on
the same host with N cores”, not “algorithmically faster”. On this container
we only have 4 vCPU, so the scaling result is host-limited.

**Estimated gain:** ~2.6–3.3× at 4 threads; potentially more on a 16-core
owner host. Combined with `guided`, plausibly 4–6×, not 7×.

**Cheapest decisive experiment:** already done for the upper bound. The next
one is a one-plan port of only the shared table/proofs and `--threads 2`,
gated on identical proofs and no regression at `--threads 1`. If 2-thread
wall does not beat 0.75× on `deep-win`, stop.

**Rejected alternative (scope):** process-level seed portfolio for median
speed on cells that already prove. The seed spread only supports ~1.2×; not
worth the cores. This rejection does **not** cover the coverage/tail case in
§5, where a portfolio can turn a censored cell into a proof.

---

### 3. `proofdb` — incremental proof store / batch proving — **the formulation change; probe before building**

**Goal:** shift from “prove one FEN” to “maintain a durable proof DAG and
solve a corpus incrementally”. Store full-key `(position, counter, proof)`
entries, finalise a canonical proof subtree across transpositions, serve exact
queries, and measure cold vs warm workloads separately.

**Why consider it:** `atomic_solver` already has the shape — proof events,
`finalize()`, validation, compact binary export, TT snapshot. For us the value
is not the 3-cell latency (a prepopulated DB would be cache gaming); it is
batch/corpus throughput, proof reuse, a public proof corpus, and training data
for `guided`.

**Expected gain:** unknown. If proof trees across corpus positions share many
positions, a proof DAG can cut repeated work; if overlap is low, it is a
product feature, not a speed lever.

**Cheapest decisive experiment:** read-only overlap census. Generate proofs
for the corpus wins, count total proof positions vs distinct positions across
all proofs. Pre-register: if shared positions are <20% of the total, do not
build a proof DB for speed; keep it as a product/data initiative.

**Contract:** a warm-cache run is its own row. Never compare a warm DB run
against a cold provari run.

---

### 4. `coverage` — widen the bench and add a held-out gate — **fold into `guided`**

**Goal:** stop tuning on three cells. Add a fixed `strong-v1` set = cells
where provari’s strongest guide wins at 10 s (currently the 3 plus `b9a7` and
`dd3f`), and a small held-out set that is never tuned on.

**Why yes:** atomi’s experience is the warning: cell-specific levers (their
`w = 2`) looked good on one cell and failed held-out. Our hand-guide census
says 3/21; the model says 5/21. A 3-cell bench cannot distinguish a general
guide from a lucky one.

**Cost:** one measurement plan, no engine change. It should be the first plan
of `guided`, not a standalone initiative.

---

### 5. `tt` / `perf` — **not standalone initiatives now**

*Amended 2026-11-20 after the owner’s note: “TT salts were a game changer for
`atomic_solver`.” The priority below is unchanged; the word “salt” was doing
two different jobs.*

- **TT, two different “salts”:**
  - **Bucket-index salt as trajectory diversification** is what
    `atomic_solver` actually introduced (`--salt`: `0` keeps shipped indexing,
    `s > 0` changes the bucket index). It does not make one run faster; it
    gives independent basins, and its payoff was a **portfolio**:
    `atomic_solver` plan8/plan9 shipped salt-seeded racers, rescuing
    `m20_white` from a 2B-child-eval timeout to a proof at 791,401,553 evals
    (~96 s wall) in 3/3 races, with dec10 0.768× wall / 0.685× evals and no
    regression, at ~3.9× CPU (~747 core-s per hard answer). That is a
    **coverage/tail** lever and belongs to `parallel`/`proofdb`, not to a TT
    rewrite. We have no bucket salt; our `--seed` perturbs tie-breaks. The
    relevant lesson is the portfolio shape, not the exact knob — our own
    8-seed `deep-win` probe measured a different question (median speed on a
    cell that already proves): min 5.37 s, median 7.04 s, ~1.2× for 4× the
    cores. Re-test a salt/seed portfolio only under a coverage or
    absolute-wall goal, on a cell solo **cannot** prove.
  - **Salt as a TT collision/structure device** is not a measured win for us:
    atomi’s census puts probe at ~0.04 pp and store at ~1.18 pp, and our table
    is already provari’s 4-slot/filter/least-work design. The measured 8–36%
    memory-split swing is a configuration/tree effect, not a case for
    bucketing or salts. A single “memory policy” plan (adaptive proof share,
    avoid the 128 MB anomaly) is worth maybe 5–10%; fold it into `perf` if we
    ever need it.
- **Perf:** the movegen audit closed with product parity and the export walk
  at 1.58× plan 2. The walk is ~9.8% of `core` instructions with `--proofs`; a
  perfect recovery is ~2.4 wall points, inside the instrument’s noise. The
  remaining per-node work is not a route to another 2×. If `guided` lands,
  NNUE inference optimisation belongs inside `guided`.

**Verdict:** no standalone `tt` or `perf` initiative until `guided`/`parallel`
are exhausted. A salt-shaped portfolio is a `parallel` plan, and only for
coverage or absolute wall.

---

### 5b. `tune` — external optimizer interface — **defer; probe sensitivity first**

**What `atomic_solver` did:** `tune` was not an in-house optimizer. It shipped
a narrow contract (`benchmark --json`, `--config` TOML, `quick`/`thorough`
suites) and left baselines, the parameter space, the loss and the search to an
external optimizer. The compiled `ScorerParams::default()` stayed hand-tuned;
the optimizer’s `config.toml` is an opt-in artifact, not a defaults copy, and
the initiative closed on the interface — its gain is not recorded as a
feature-ledger win. The neighbouring constant-retuning items that were
measured went no-go (history/killer constants: the shipped set is the local
optimum).

**For us:** the analogous surface is `src/eval.rs`’s six-entry `VALUES` table
plus `TARGET`, `TARGETS`, `FUSE`, `PRESSURE`, `CHECKER`, and `dfpn.rs`’s
`HAND_DECISIVE` band and `EF_B`. The eval is a copied upstream module whose
constants were inherited from provari, never fitted to our tree. `prove --json`
and `bench` already emit machine-readable rows, so a tune-style harness is
mostly a config surface plus a multi-cell suite.

**Why maybe:** cheap (interface and harness, no new algorithm), deterministic
`child_evals` makes evaluation stable, and the same surface serves post-
`guided` tuning (model inference constants, guide thresholds, move order).
Caveat: the hand eval only sets a 3-way band `{0,1,2}` at ±`HAND_DECISIVE`, so
a weight change matters only when it moves a position across the band — the
surface may be flat.

**Why not first:** `guided` is the measured ~2.7×; tuning a coarse hand eval
is not a route to 2×. With three ledger cells, a tuned set is likely overfit;
`atomic_solver`’s own lesson is that constant sweeps can sit at a local
optimum, and that its tuned set came from an external ordering-parameter
search, not a hand sweep.

**Cheapest decisive experiment:** a read-only sensitivity sweep, not an
interface. In a scratch tree, expose the eval terms, `HAND_DECISIVE` and
`EF_B` via env overrides; run each at roughly ±25–50% on the three cells with
3 seeds (or `strong-v1` once it exists), reading deterministic `child_evals`
and accepted proofs. Pre-register: if no single perturbation moves a cell by
≥10%, and no cell wins without a paired loss, the surface is flat — no `tune`
initiative; fold the harness into `guided` only once the model has parameters
worth fitting. If it does move, write the interface plan with the observed
axes and a held-out cell.

**Verdict:** not a standalone initiative before `guided`. Keep it as a
`guided` follow-on (tune model/guide constants) or a `coverage`-first plan;
never tune against the three ledger cells alone.

---

### 6. `tb` — tablebases — **no, not now**

The 3-4-5 Atomic tables changed **zero nodes** on all three bench cells with
provari hand. The current proofs do not reach the tablebase boundary at 10 s.
Tablebases remain a horizon item for endgame/coverage work, and a 6-piece
probe should only be run if a coverage cell actually needs it. Do not start a
TB initiative for the current bench.

---

## Recommended sequence

1. **Open `guided`** with a plan that does three things in one sitting:
   - licence/provenance/source take-list for `model.rs` + first-layer `nnue.rs`
     + data SHA,
   - a `strong-v1` measurement column (3 shared + 2 model-only cells),
   - a scratch `--guide model` integration and one 10 s `deep-win` A/B.
2. If the gate passes, make `guided` the product default and append a ledger
   row with both hand-vs-hand and model-vs-model columns.
3. Then decide between `parallel` (absolute wall) and `proofdb`
   (batch/incremental), based on the owner’s next goal.
4. Run the §5b sensitivity sweep before opening `tune`; if the hand-eval
   surface is flat, tune only inside `guided` and only with a held-out cell.

### Suggested kickoff prompt (option 1)

> Open a new initiative `guided` in `docs/plans/guided/`. Read `AGENTS.md`,
> `docs/bench.md`, `docs/plans/movegen/report13.md`, and the read-only probes
> recorded in `docs/notes/ideas.md`. Goal: adopt provari’s provability model +
> Atomic-Stockfish network as an opt-in guide, then as the product default only
> if the gate passes. First plan: (a) source take-list and provenance/licence
> rows for `model.rs`, the first-layer part of `nnue.rs`, `model.json` and
> `network.nnue` with SHA-256; (b) a `strong-v1` bench column = the 3 existing
> cells plus the model-only cells `b9a7ed7eb368` and `dd3f89766a40`, kept
> separate from the hand ledger rows; (c) a scratch `--guide model` integration
> and one-seed 10 s `deep-win` A/B. Gate: 3/3 at 10 s, `deep-win` < 2.5 s, 9/9
> accepted by both checkers, identical counters on repeat, `--guide hand`
> byte-identical. Do not touch the checker, the proof format, or the hand-guide
> defaults before the gate.

---

**SESSION COMPLETE**

Deliverables: ranked initiative list, measured probe evidence, recommended
first initiative, and a kickoff prompt. No repo source files were modified.
