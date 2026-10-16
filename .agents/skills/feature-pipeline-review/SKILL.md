---
name: feature-pipeline-review
description: Spawn a sub-agent to compare a feature's PRD-TRD + IMPL against the actual implementation. Outputs a gap report in plain English. Use after implementing a feature, before committing or marking it SHIPPED.
---

# Feature Pipeline Review

Run a fresh sub-agent to compare the **plan** (PRD-TRD + IMPL) against the **reality** (TypeScript code + post-impl simulation). Optionally also compares the post-impl simulation against the pre-impl simulation to surface design drift. Outputs a plain-English gap report.

The agent doing the review is **not** the agent that wrote the code. Fresh eyes catch drift the author rationalized away.

## When to Use

- After implementing all phases of a pack
- Before marking a pack as SHIPPED in the backlog
- When you suspect drift between docs and code
- Before a release that bundles multiple packs

## When NOT to Use

- Mid-implementation (review each phase as it lands, not the whole pack)
- When the only changes are tests or docs (no behavior change to review)

## How to Invoke

This skill **spawns a sub-agent** — it does not run inline. Use `delegate_task` with the orchestrator pattern, or run it manually with the steps below.

```python
delegate_task(
    goal="Compare docs/features/<slug>/ against packages/<slug>/src/ for drift",
    context="""
      Doc pack:
      - docs/features/<slug>/PRD-TRD-<slug>.md
      - docs/features/<slug>/IMPL-<slug>.md

      Implementation:
      - packages/<slug>/src/

      Simulation:
      - docs/features/<slug>/simulate-pre.sh   (design-time, may be .html)
      - docs/features/<slug>/simulate.sh       (post-impl, may be .html)
      - docs/features/<slug>/simulate.html     (only if .sh not used)

      Output: .reports/<timestamp>-drift-<slug>.md
      Format: see "Report Structure" below.
    """,
    role="leaf"
)
```

The sub-agent must:
1. Read the doc pack and the source code
2. Run the simulation if it exists
3. Produce the gap report
4. **Not modify any code or docs** — review only

## What the Sub-Agent Compares

Three or four dimensions:

| Dimension | Compares |
|---|---|
| **Contract** | PRD-TRD's Behavioral Spec + Technical Design vs. actual code behavior |
| **Execution** | IMPL's phase plan vs. what got built (all phases done? in order? tests exist?) |
| **Simulation** | PRD-TRD's Simulation Contract vs. what the post-impl sim actually demonstrates |
| **Design drift** *(if pre-impl sim exists)* | Pre-impl sim behavior vs. post-impl sim behavior — surfaces drift between original design and shipped reality |

## Report Structure

The sub-agent writes to `.reports/<timestamp>-drift-<slug>.md`:

```markdown
# Drift Review: <slug>

**Date:** YYYY-MM-DD
**Reviewer:** sub-agent
**Verdict:** [Aligned | Minor Drift | Major Drift]

## Summary

- Contract gaps: N
- Execution gaps: N
- Simulation gaps: N
- Recommended action: [ship | fix code | update docs | log drift]

## Contract Gaps

### Gap 1: <title>

**Expected (from PRD-TRD):**
> <quote or AC from doc>

**Found (in code):**
```typescript
<code snippet or "not implemented">
```

**Plain English:** <what this means for the operator/user>

**Recommendation:** [fix code | update PRD-TRD | log drift]

---

## Execution Gaps

### Phase 1: <title>
**Status:** [complete | partial | missing]
**Notes:** <what's missing or drifted>

---

## Simulation Gaps

### Scenario 1: <name>
**Expected:** <from PRD-TRD Simulation Contract>
**Demonstrated:** <what post-impl sim actually shows>
**Match:** [yes | no]

---

## Design Drift (pre-impl sim vs post-impl sim)

<Only included if both sims exist.>

### Drift 1: <title>
**Designed (pre-impl sim):** <what the user originally saw>
**Shipped (post-impl sim):** <what was actually built>
**Plain English:** <what this means for the operator/user>

---

## Drift Items to Log

<Bulleted list of accepted gaps. Each gets logged to docs/drift-issue-log.md.>

## Recommendation

<One paragraph: ship as-is, fix code, update docs, or accept drift.>
```

## After the Sub-Agent Reports

The parent agent (you) reviews the report with the user. Three actions:

1. **No gaps:** mark pack SHIPPED, update `docs/Feature_Backlog.md`, commit with Conventional Commits
2. **Minor gaps:** decide per gap — fix code or update docs
3. **Major gaps:** stop. Re-enter `feature-pipeline` at the right phase to fix.

## Anti-Patterns

- **Don't run this inline** — fresh eyes matter. Always spawn a sub-agent.
- **Don't have the sub-agent fix anything** — review only. Fixing is the parent's job.
- **Don't skip the simulation check** — a sim that doesn't demonstrate the contract is itself drift.
- **Don't compare old-format docs** (PRD/TRD/FLOW/IMPL as separate files) — that flow is deprecated. Use only PRD-TRD + IMPL.

## Limitations

- The sub-agent can only see static code. Runtime behavior must be checked via the simulation or tests.
- Drift detection depends on doc quality. A vague PRD-TRD produces vague gap reports.