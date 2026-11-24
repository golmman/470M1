# Decisions — product level

Decisions that bind the whole product, not one initiative. A decision changes
only through a new dated entry; an initiative's `decisions.md` holds what
binds only that initiative.

| # | Decision | Resolution |
| --- | --- | --- |
| PD1 | Reference and goal metric | latest provari release, all features, equal threads; `strong` + `held-out` sets; hand column kept as guard |
| PD2 | Determinism | counters reproducible at `--threads 1`; threaded runs are claimed by checkers and repeat statistics |
| PD3 | Output channels | result on stdout, human-readable logs and errors on stderr |
| PD4 | Sequencing | `foundation` first, then `guided` -> `parallel` -> `proof-db` -> openings |
| PD5 | Scope | learned guide, tablebases, threads and a proof database are in scope; web, UCI and draw/loss proofs are not |
| PD6 | Claims and formats | both checkers, `.pvr` v1 is the claim format; the proof database is internal |
| PD7 | Audits | code/architecture, documentation and workflow audits replace kill reviews |
| PD8 | Roles and git authority | AI builds, tests and reviews; the owner guides, senses and commits |
| PD9 | Configuration and CLI | `clap` + `serde` + `toml`; one typed config per command |
| PD10 | Error model | typed errors per layer; exit codes mapped only in the binary |
| PD11 | Documentation structure | standing docs in `docs/`; initiatives hold only initiative state |

## PD1 — Reference and goal metric

**Resolution (2026-11-24).** The reference is the latest provari release,
built from source at a pinned commit, with the learned guide, tablebases and
`--threads N`. Both sides run with the same `N`, host, session, memory and
tablebases. The goal factor is the geomean of candidate/reference wall over
the claimed `strong` cells; a `held-out` set is never tuned on and must not
get worse. The provari `--guide hand`, 1-thread column stays as a guard.
Supersedes `D2` (pin) and `D6` (set and metric).
**Why.** The PRD's milestone is full-feature provari; equal threads makes the
claim algorithmic, not a core count. Three hand-selected cells cannot tell a
general lever from a lucky one (atomi's held-out lesson).
**Consequences.** `foundation` re-pins the oracle and defines `strong`,
`held-out` and `N` in `bench.md`. Earlier rows are not rewritten.

## PD2 — Determinism

**Resolution (2026-11-24).** At `--threads 1` the same binary, position and
configuration give identical counters, and proof collection, logging and
configuration plumbing never change the trajectory. At `--threads N > 1`
counters may vary; a claim needs both checkers, and a performance reading
needs repeats with spread. Narrows `D8`.
**Why.** Deterministic parallel search would cost the speed threads buy.
**Consequences.** Every mechanism measurement and regression fixture runs at
1 thread; threaded code keeps the 1-thread path byte-identical.

## PD3 — Output channels

**Resolution (2026-11-24).** stdout carries only the result (text, or one
JSON line with `--json`). stderr carries human-readable logs: errors,
warnings, progress. Verbosity via `--log LEVEL`, `-q`, `-v` and the config.
**Why.** Drivers and pipes parse stdout; progress there would break them.

## PD4 — Sequencing

**Resolution (2026-11-24).** Open `foundation` first: configuration, logging,
typed errors, module split, checker isolation and bench v2, with no search
change. Then `guided`, `parallel`, `proof-db`, and the opening milestones.
`docs/notes/ideas.md` is the input, not the plan.
**Why.** Every later initiative rewrites the large search modules; doing it
once, under identical counters, is cheaper than per initiative.

## PD5 — Scope

**Resolution (2026-11-24).** Product scope adds a learned guide, tablebases,
parallel search and a proof database. Out: web frontend, UCI, draw/loss
proofs, unless a milestone needs them. Supersedes `D7` and the `movegen`
non-goals.

## PD6 — Claims and formats

**Resolution (2026-11-24).** `D4` and `D9` carry forward unchanged. provari's
compact `.pvr` v1 stays the claim and exchange format (`D5`). The proof
database is an internal store that exports `.pvr`; a store entry is never a
claim. A milestone proof too large for either checker is a blocker to solve,
not a reason to skip a checker.

## PD7 — Audits

**Resolution (2026-11-24).** Kill reviews end. Three audits replace them:
code/architecture, documentation, workflow; triggers and checklists are in
[`workflow.md`](workflow.md#audits). Findings become backlog items or
decisions.
**Why.** Owner: kill reviews brought no benefit; the PRD asks for regular
audits of these three subjects.

## PD8 — Roles and git authority

**Resolution (2026-11-24).** AI writes all code and docs, tests, reviews,
proposes and consults. The owner sets goals and priorities, provides sensors
(hosts, releases, review) and commits; agents use read-only `git`.

## PD9 — Configuration and CLI

**Resolution (2026-11-24).** `clap` (derive) parses flags; `serde` + `toml`
read `--config FILE`. Precedence default < file < flag. Each command has one
typed, validated config built at the edge; the library never reads flags,
files or the environment. Unknown keys are errors.
**Why.** FR1, with GPL-compatible, widely used crates instead of a growing
hand parser.

## PD10 — Error model

**Resolution (2026-11-24).** Each layer has its own error enum
(`std::error::Error` + `Display`); `String` errors are retired. Only the
binary maps errors to exit codes and messages. Writes are atomic (temporary
file + rename).

## PD11 — Documentation structure

**Resolution (2026-11-24).** Standing documents live in `docs/`:
`product.md`, `decisions.md`, `architecture.md`, `guidelines.md`,
`workflow.md`, `gotchas.md`, `bench.md`, `proof-format.md`. Status lives only
in `docs/plans/README.md` and each `initiative.md`. Supersedes `D12`.

## Carried forward from closed initiatives

| Decision | Still binding |
| --- | --- |
| `D1`, `D10` | licence, copying and provenance |
| `D4`, `D9` | checker independence (shakmaty only) and claim protocol |
| `D5` | `.pvr` v1 and JSONL; move order is part of the format |
| `MD5` | the rules hot path calls `atomic-movegen` directly; conversions only at edges |
| `MD9` | `unsafe-optimizations` stays guarded by `make unsafe-feature` |
| `M7` | upstream `atomic-movegen` asks go in `wishlistN.md`; releases are integrated by owner-triggered plans |
