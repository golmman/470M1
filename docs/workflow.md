# Workflow

How work moves from the owner's intent to merged code. Goals may change; every
change is written down where it belongs.

```
product.md goal ── milestones ── initiatives ── plans ── reports
                                     │
                                  audits (code/architecture, docs, workflow)
```

## Where each fact lives

| Fact | One home |
| --- | --- |
| purpose, goal, milestones, requirements, goal changes | [`product.md`](product.md) |
| product-wide decisions | [`decisions.md`](decisions.md) |
| structure and seams | [`architecture.md`](architecture.md) |
| code rules | [`guidelines.md`](guidelines.md) |
| correctness traps | [`gotchas.md`](gotchas.md) |
| measurement rules and results | [`bench.md`](bench.md) |
| initiative status (one line each) and milestone map | [`plans/README.md`](plans/README.md) |
| an initiative's goal, items, decisions, status | `plans/<initiative>/initiative.md` (+ `decisions.md`) |
| what a plan intended / what happened | `planN.md` / `reportN.md` |
| owner's unfiltered notes | `notes/` (not normative) |

Other documents link to the home instead of repeating it. `README.md` and
`AGENTS.md` carry no figures.

## Initiatives

- **Open** with `initiative.md`: the milestone it serves, one goal, a done
  definition with measurable gates, non-goals, items, risks. Add a status
  line to `plans/README.md`.
- **Items** are stable semantic ids (`F1`, `G2`, ...); plans are numbered
  sequentially and independently; a plan header names its item.
- **Close** when the done definition is met or the owner stops it: a closing
  report, the three audits, the status line, and a goal-change row if the
  product goal moved.

## Sessions

| Session | Does | Writes |
| --- | --- | --- |
| plan | reads the initiative and the latest report; read-only probes only | `planN.md` (one sitting) |
| execution | implements the plan, runs its gates | code, tests, `reportN.md` |
| audit | runs one checklist below; implements nothing | `docs/audits/auditN.md` |
| documentation / ideation | aligns docs or explores options; no code | the touched docs |

**A plan states:** item and goal; scope and non-goals; the steps; the gates
(correctness first, then tests, then measurements) with pre-registered
pass/fail rules; the report as its last task.

**A report states:** the plan's goal and whether it was met; results with
evidence paths; scope changes; problems; open parts; missing tests; next
steps.

## Audits

Audits replace kill reviews (`PD7`). Each produces findings with a severity
and an owner action: fix in the next plan, new backlog item, or decision.

**Triggers:** every initiative closure (all three); a report that finds a
guideline breach (code); the owner, at any time.

| Audit | Checklist |
| --- | --- |
| code / architecture | layer and crate dependencies follow `architecture.md`; file sizes; `pub` surface; `unwrap`/`expect`/`panic!` count; `String` errors; duplicated logic; seams have contract tests; `unsafe` sites justified; dependency licences, ages and pins |
| documentation | one home per fact (no repeated figures); budgets; stale status; dead links; superseded decisions marked; `AGENTS.md` still an entry point |
| workflow | did plans fit one sitting; did gates catch real defects; what cost the most time; which rule was ignored or useless; propose at most one process change |

## Change control

- Goal or milestone change: a dated row in `product.md`.
- Decision: a new dated entry (`docs/decisions.md` or the initiative's); never
  edit an old one silently.
- Scope change during execution: in the report, then in `initiative.md`.

## Measurements

Rules are in [`bench.md`](bench.md): one process at a time, same-session
ratios. Commit drivers, parsed results, `env.json` and READMEs under
`plans/<initiative>/measurements/planN/`; never raw transcripts, oracle
artifacts or `__pycache__`.

## Upstream `atomic-movegen`

We own it. A needed rules-layer change is a numbered `wishlistN.md` request;
a release is integrated only by an owner-triggered plan and report that
update the pin and `PROVENANCE.md` (`M7`). No private fork or path
dependency.

## Budgets

| Document | Limit |
| --- | --- |
| `AGENTS.md` | 8 KiB |
| standing docs in `docs/` (not `bench.md`'s ledger) | 8 KB each |
| `planN.md`, `auditN.md` | 8 KB |
| `reportN.md` | 12 KB |
| status entries | one line |

If docs outgrow the code, cut docs, not tests.

## Closing a session

End with `SESSION COMPLETE`, the session's goal and whether it was met, the
deliverables, and follow-up options; option 1 is a copy-pasteable kickoff
prompt. Agents use read-only `git`; the owner commits (`PD8`).
