---
name: drift
description: Record and query the long-running drift log that tracks the gap between documented intent and code reality. Use whenever a code-vs-doc conflict is found, when a cross-pack assumption is challenged, or before any doc edit that touches a previously-logged drift.
metadata:
  version: "0.1.0"
  author: spanexx
  tags: documentation, drift, audit, cross-pack
---

# Drift Skill

## Purpose

Three artifacts: **Docs** (intent), **Code** (reality), **Drift log** (`docs/drift.md`) — the gap between them. Drift is not "docs wrong, fix them." It's "here's the breadcrumb, here's who reconciles." Without tracking, divergences pile up as silent `git blame` time-bombs.

## `drift write` — record a drift entry

Open `docs/drift.md` (create with header template below if missing). Append one entry at bottom of **## Open**:

```
- **<ID>** (<severity>, <YYYY-MM-DD>, reporter: <session>) — <summary>.
  - Doc claim: `<quote>` (`<doc path:line>`)
  - Code reality: `<quote>` (`<code path:line>`)
  - Why matters: <operational impact>
  - Owner: <pack or "cross-pack audit">
  - To fix: <fix doc, fix code, or surface to grill>
  - Related: <other drift IDs, ADR IDs>
```

**Trigger when:** code-vs-doc conflict, cross-pack inconsistency, glossary drift, behavioral change without doc update, external assumption invalidated, audit contradiction, decision locked without ADR, or a grill artifact references `[DRIFT-LINK <id>]` not yet in log.

**Severity:**
- **Critical** — blocks next phase. Resolve before continuing.
- **High** — resolve before affected feature ships. Doc in IMPL §"Open Drift".
- **Medium** — resolve this session. Doc same as High.
- **Low** — note, resolve opportunistically. Never block a phase.

**ID convention:** `D-<sequence>`. Read highest ID, increment. Never renumber on resolution. Also add a breadcrumb to the affected pack's IMPL §"Open Drift" block citing the drift ID.

## `drift read` — query the drift log

```bash
sed -n '/^## Open$/,/^## Resolved/p' docs/drift.md        # all open
grep -E 'Critical|High' docs/drift.md | head -20           # top-severity
grep -B1 -A1 'Owner: <pack>' docs/drift.md                 # by pack
```

**Run before:** any doc edit, any pack-doc authoring (PRD/TRD/FLOW/IMPL), session start, after scout produces a report, when grill-with-docs captures a decision touching existing drift, before Phase 5a decision-stability check, before any commit/merge/handoff touching a doc, before grilling a new pack.

## Reconcile by severity

| Severity | Procedure |
|---|---|
| **Critical** | STOP. Open issue in IMPL §"Critical Drift". Fix code (doc right) or fix docs (code right) or escalate to grill. Move entry from Open to Resolved with date + verifier. |
| **High** | Same as Critical but doc in IMPL §"Open Drift" inline. May defer to follow-up pack. |
| **Medium** | Doc in IMPL §"Open Drift". Resolve this session or schedule in grill. |
| **Low** | Note in drift.md. Fix opportunistically. Never block. |

## Header template for `docs/drift.md`

```
# Drift Log
**Last updated:** <YYYY-MM-DD>  **Open:** <N>  **Resolved:** <N>  **Critical/High:** <N>

## Open

- **D-1** — ...

---

## Resolved
```

## Rules

- Append-only for ## Resolved (never delete). Update counters on section moves. Never split/merge IDs.
- Every entry must have doc path:line AND code path:line — no citations = no drift.
- Resolved entries must have a `Verified by:` trail.

## Integration

- **`situation-awareness`**: prints last 10 Open/Resolved entries.
- **`scout`**: parent runs `drift read` on scout findings; Critical/High → drift entries, Low/Medium → inline doc patches.
- **`grill-with-docs`**: grill artifacts may reference `[DRIFT-LINK <id>]` — `drift write` promotes to full entry.
- **`feature-pipeline`**: Phase 5a includes `drift read` — Critical/High drifts blocking the phase must resolve before Phase 5b.

## Anti-patterns

- Logging every misread as drift (typo ≠ drift).
- Logging without file:line citations.
- Resolving doc without resolving code (or vice versa) when both are wrong.
- Mass-closing drifts without per-entry `Verified by:`.
- Skipping the IMPL breadcrumb (drift log is the index; IMPL breadcrumb is the local reminder — both needed).
