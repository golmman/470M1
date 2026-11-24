# Product — `470MI`

The normative statement of purpose, goal, requirements and milestones. It is
the owner's PRD (2026-11-24) in checkable form. It changes only through a
dated row in [Goal changes](#goal-changes); the reasoning lives in
[`decisions.md`](decisions.md).

## Purpose

Build the fastest Atomic chess win prover. Every claimed win carries a proof
that two independent checkers accept.

## Goal (moving)

**Now: geomean wall factor `<= 0.5` against the reference** — twice as fast
as provari ([`PD1`](decisions.md#pd1--reference-and-goal-metric)).

| Term | Definition |
| --- | --- |
| Reference | the latest pinned provari release with every feature on: learned guide, tablebases, `--threads N` |
| Fairness | the same `N`, host, session, memory and tablebases on both sides; `N` is part of a row, like the cap |
| Bench set | `strong` (cells the reference proves at the canonical cap) plus a `held-out` set that is never tuned on; defined in [`bench.md`](bench.md) |
| Factor | candidate wall / reference wall, per cell; geomean over claimed cells; coverage stated beside it |
| Guard column | provari `--guide hand`, 1 thread, the historical ledger column; a regression there is a finding |

Correctness is an invariant, not a goal: a faster false win is a failure.

## Milestones

Milestones are the stops on the way to the goal. Initiatives complete them
([`workflow.md`](workflow.md)).

| Milestone | Done when |
| --- | --- |
| `beat-provari` | `strong` geomean `< 1.0` against the reference, coverage `>=` the reference's, `held-out` not worse, every claim accepted |
| `proof-db` | a durable proof database à la `atomic_solver`: runs add to it, it survives restarts and crashes, it exports checkable `.pvr`; warm and cold runs are reported apart |
| `nf3-d6` | a `.pvr` for the position after `1. Nf3 d6` (g1f3 d7d6), White to move, accepted by both checkers |
| `start` | a `.pvr` for the starting position, White to move, accepted by both checkers |

The goal factor is tightened only by a goal-change row, typically after
`beat-provari`.

## Functional requirements

| # | Requirement | Acceptance |
| --- | --- | --- |
| FR1 | **Configuration** by CLI and config file | every option has a flag and a TOML key; precedence default < file < flag; unknown keys and bad values are errors; the effective configuration is printed with every result |
| FR2 | **Robust error handling** | no user-triggerable panic; typed errors with cause and remedy; documented exit codes; a failure never corrupts a written proof or the proof database |
| FR3 | **Human-readable logging** | errors and progress on stderr, the result on stdout ([`PD3`](decisions.md#pd3--output-channels)); progress at a fixed wall cadence (elapsed, nodes, nodes/s, root numbers, table fill, iteration where the search has one); levels via flags/config; logging never changes the search |
| FR4 | **Commands** | `prove`, `check`, `line`, `bench`; the proof database adds its own commands |
| FR5 | **Claims** | a win is claimed only after our checker and `provari check` accept its `.pvr` |

## Non-functional requirements

| Quality | How it is met and measured |
| --- | --- |
| Performance | wall time (product metric); search tree size (nodes, child evaluations at 1 thread); nodes/s (secondary). Rules in [`bench.md`](bench.md) |
| Maintainability | SOLID, low coupling, high cohesion ([`architecture.md`](architecture.md), [`guidelines.md`](guidelines.md)); enforced by audits |
| Testability | a test pyramid run by `cargo test`/`make`; every behaviour reachable through a flag or option |
| Documentation | one home per fact, short documents, budgets in [`workflow.md`](workflow.md) |
| Workflow | agile: goals may change, every change is recorded; milestones -> initiatives -> plans -> reports |

## Interaction model

- **AI** writes all code and documentation, runs all tests and reviews,
  proposes changes and consults.
- **Owner** provides guides (this document, priorities, decisions) and
  sensors (bench hosts, upstream releases, report review), and commits
  ([`PD8`](decisions.md#pd8--roles-and-git-authority)).

## Goal changes

| Date | Change | Source |
| --- | --- | --- |
| 2026-11-17 | `parity`: geomean `<= 1.0` vs provari `--guide hand`, 3 cells, 10 s | owner, `parity` |
| 2026-11-18 | `movegen`: shakmaty out of the prover, no bench regression | owner, `MD1` |
| 2026-11-24 | fastest prover; goal `<= 0.5` vs full-feature provari at equal threads; milestones `beat-provari`, `proof-db`, `nf3-d6`, `start` | owner PRD, `PD1` |
