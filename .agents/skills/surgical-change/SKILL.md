---
name: surgical-change
description: Perform a surgical change — a small, targeted code fix driven by a drift-log entry (D-N) or a discovered issue — without the full feature pipeline. No new GRILL, no PRD-TRD/IMPL written from scratch, no pre/post-impl sims. The design already exists and is locked; you fix the gap between doc and code (or a bug) in the smallest surface, then update the affected feature docs under docs/features/<slug>/ so the docs stay the truth. Use when the user says "fix the drift", "surgical change", "this is broken, just fix it", or points at a D-N entry or a behavior that contradicts its feature docs.
---

# Surgical Change

> **⛔ ISOLATION GATE — READ THIS FIRST. NON-NEGOTIABLE.**
> **Main is never the workbench.** All fix work happens in a git **worktree**
> on a dedicated branch, created ONLY by the sanctioned CLI — never
> `git branch` / `git switch` / `git worktree add` by hand.
>
> **You may perform NO step of this workflow — no reading of implementation
> files, no writing docs, no building, no testing — until you have created the
> worktree and are INSIDE it.** Triage/doc-reading gives you the slug; the
> moment that slug is known you create the worktree before touching any code
> or doc file.
>
> **The gate (tick all before any edit):**
> - [ ] `agent-worktree fix <slug>` ran from the repo root
> - [ ] ALL subsequent commands run from INSIDE the printed `WORKTREE` path
> - [ ] `git branch --show-current` reports the worktree branch, **NOT** `main`
>
> **Why:** a missed isolation step pollutes shared `main` with an unreviewed,
> unverified fix. Dirty-tree / name-collision reasons to "stay on main" are NOT
> valid — resolve them via the CLI's report (stash the unrelated work, pick a
> different slug). Never bypass.
>
> **If you already started on main:** STOP. Make the tree clean (commit/stash/
> resolve unrelated changes per the CLI's report), create the worktree, re-apply
> your work inside it, and reset `main` to its last clean commit.

Small fix, locked design. The feature pipeline exists for NEW behavior; this is
for bringing reality and docs back together (drift) or fixing a discovered bug —
while keeping the feature's docs current.

## When to use

- Fixing a **drift-log entry** (`docs/drift.md`, D-N) — code or docs must move.
- A **discovered issue** that contradicts an existing feature's docs (this pack's
  post-impl reviews, release validation, operator complaints).
- A **small behavior change** whose design is already locked in a PRD-TRD.
- Examples from this workspace: the `.agentide` data-dir unification, init
  idempotency (D-118), the group-form live-dispatch fix (review Gap 1).

## When NOT to use — escalate to feature-pipeline

- **New surface**: new commands/flags/capabilities, new config keys, cross-package
  contract changes, new GRILL decisions.
- **No feature docs exist for the behavior** (`docs/features/<slug>/` has no
  GRILL/PRD-TRD/IMPL for it) — the design is NOT locked; escalation to
  feature-pipeline.
- The fix needs its own design discussion (ambiguous intent, multiple valid
  shapes). If the user can't point at a locked decision, it's a feature.
- More than ~200 lines across packages, or touching 3+ packages.

Escalation rule: if triage shows new surface or new decisions, stop and say
"this needs feature-pipeline" — do NOT half-build a feature here.

## Workflow

### 1. Boot (mandatory practices — same as any task)

0. `agent-notify` skill session-start protocol: inspect queued deliveries with
   `agent-notify drain --dry-run`, deliver only worthwhile pokes, run
   `agent-notify sweep`, and read `pending-victor.md`.
1. `caveman` skill (token-efficient mode)
2. `situation-awareness` skill — git state, recent commits, handoff
3. `drift read` (the drift skill) — the entry you're fixing + anything adjacent

### 2. Triage — START from the feature docs (they ARE the design)

The feature is ALREADY decided, documented, and (usually) implemented. The
pack in `docs/features/<slug>/` is the source of truth — investigate it FIRST,
not the code:

1. **Read the feature pack first**: `docs/features/<slug>/` — GRILL (locked
   decisions), PRD-TRD (the contract), IMPL (the execution plan). The behavior
   under fix is defined THERE, in the words the user approved. Read them in
   that order before touching any code.
2. **Map the doc to code**: find the implementation the docs describe — search
   `packages/*/src/` for the command/capability/term the docs name. The docs
   say what it SHOULD do; the code shows what it DOES. The gap between them
   is the drift.
3. **Classify the fix**: **code-only** (code drifted from the docs — fix
   code), **doc-only** (code is the locked intent — amend the doc), or
   **both** (most common).
4. **Decide scope WITH the user**: surgical vs feature-pipeline (criteria
   above).

5. **Isolate before editing — THE GATE ABOVE.** Once the target feature pack and
   fix slug are known, run `agent-worktree fix <slug>` from the current repository
   and use its printed absolute `WORKTREE` path — BEFORE reading implementation
   files, writing docs, building, or testing. If already inside the intended
   worktree, continue there. Never manually run `git branch` or `git worktree add`;
   the CLI rejects dirty trees and collisions — resolve them, do NOT stay on main.

A change can touch several packs (e.g. a config default touches
`cli-restructure` AND `agentide-cli-consumer`) — read all affected feature
packs the same way, docs first.

### 2a. Continuity checkpoint — write the RESUME handoff BEFORE the work

**MANDATORY for any change that spans more than one surface** (code + docs,
features docs + drift log, sim + release, …) **or whose fix is subtle**
(precedence chains, new config defaults, opt-in switches, cross-pack
semantics). Write the handoff (`handoff` skill → `sessions/.last-handoff`)
the moment the design is locked, BEFORE the first code edit:

1. Locked design in 3-5 lines — decision + why + what is deliberately NOT done
   (e.g. "no migration code — operator deletes old dirs by hand").
2. Ordered remaining steps, each with the command that PROVES it (e.g.
   `bash docs/features/<slug>/simulate.sh` → expect 21/21; `npm run precommit`).
3. File inventory: every file the fix touches (code / tests / docs / sim) +
   the drift entry (D-N) it resolves or writes.
4. User-only follow-ups (do NOT do them: cleanup, dir deletions, external
   sign-offs).

Refresh this handoff at every phase boundary (code green → docs amended →
 drift resolved → committed). **Why:** a finished change gets its final
 handoff (step 7), but a cut-off session leaves none — and a subtle fix's
 intent (precedence, defaults, deliberate non-goes) is unrecoverable from
 git history alone. The checkpoint + the D-N entry + the amended PRD-TRD/
 IMPL notes are the fresh agent's resume path (situation-awareness reads
 `.last-handoff` on boot).

### 3. Implement — TDD, smallest surface

- One test → minimal fix → repeat (see `tdd` skill; vertical red-green).
- Fix the ROOT cause, not the symptom. If the root is a doc that lied, the
  "fix" is a doc edit and the test pins the REAL behavior.
- Respect existing seams (`home`, `pidFile`, `stdin`/`stdout`, `env`, `fs` in
  CliOptions) — add a seam only if the test genuinely needs determinism.
- Keep files under the 350-line cap (AGENTS.md rule 9) — if the fix pushes a
  file over, split it in the same change (a surgical split, not a refactor).
- Do NOT touch unrelated drift entries or tests — mention them, leave them.

### 4. Update the affected feature docs — MANDATORY

The docs are the truth. After the fix:

- `docs/features/<slug>/PRD-TRD-<slug>.md` — amend the clause that diverged;
  if the shipped reality differs deliberately, add a short note citing the
  reason (tooling gate, deferred scope, GRILL lock) so the doc isn't stale.
- `docs/features/<slug>/IMPL-<slug>.md` — append a **delivery note** entry
  (date + what changed + why + verification); flip phase statuses the fix
  touches. Never rewrite history — add notes.
- Post-impl sims (`simulate.sh`, `simulate.html` or equivalent) are canonical:
  update them ONLY if the fix changes what they demonstrate (e.g. a new error
  string the sim greps).
- `CONTEXT.md` — if the fix introduces or renames a canonical term, update the
  glossary FIRST (rule 3).
- `drift read` BEFORE every doc edit; if the fix reveals a NEW divergence,
  `drift write` it before editing (rule 4).

### 5. Drift log ledger

- Fixing a D-N: move it from Open to Resolved with a dated note — what changed,
  commit ref, verification. Update the header counts (Open/Resolved).
- Discovered issue with no entry: write a new D-N (open) BEFORE the fix, then
  resolve it in the same session, so the log tells the story.
- Accepted as-is (doc-only, deliberate): annotate the entry with the decision.

### 6. Validate

- The full suite is green locally, and CI is green on the branch.
- Full suite: `pnpm exec vitest run` from the agentide repo root. Pre-existing
  failures are NOT yours — name them in the final message.
- If the change affects a published surface, offer `post-release-validation`.

### 7. Human gates

Before merge, open a pull request, then create a Victor review event and stop:

```bash
# 1. Push the fix branch and open the PR — the PR is the review artifact.
git push -u origin <branch>
gh pr create --base main --head <branch> \
  --title "fix(<scope>): <one-liner>" \
  --body-file <pr-body.md>   # problem, fix, test evidence, docs touched

# 2. The review event must carry the PR URL.
agent-notify create <project>-<slug>-human-review \
  --title "Human review required: <project> <slug>" \
  --ask "PR: <url>. Evidence: affected feature docs, tests, drift resolution, and verification are ready. Review and approve or request changes." \
  --as <agent-name>
```

`agent-notify create` automatically subscribes and immediately notifies
Victor with the complete ask. Do not merge, close the worktree, or continue in
the same turn. On the next turn, read `agent-notify show <event-id>` and honor
Victor's response.

**Merging is PR-only.** After Victor approves, merge through the PR
(`gh pr merge --squash`, or the UI) — never a local merge/push to `main`.
Closing the worktree happens after the PR merge lands.

If this change is part of a release, use the same version-blessing event and
stop before moving `stable`. Stable remains Victor-only.

### 8. Commit + close out

- Stage on purpose, path by path, then commit with a Conventional Commit.
- Type by content: `fix(...)` for behavior, `test(...)` for pins, `docs(...)`
  for spec amendments, `chore(...)` for syncs.
- Push the branch and open the PR (step 7) if not already done — every
  subsequent fix commit goes to the same branch and updates the PR.
- Update `docs/Feature_Backlog.md` ONLY if a follow-up pack is now justified.
- `handoff` skill — refresh the step 2a checkpoint into the final handoff
  (sessions/.last-handoff + breadcrumb): add commit hash, verified-by trail,
  the PR URL, and the D-N referenced; keep the remaining-steps section (now
  empty / done) + user-only follow-ups so the next session knows what stays
  open.

## Checklist

```
[ ] Trigger classified: drift D-N / discovered issue / locked-design fix
[ ] Scope agreed with user (surgical vs doc-only vs feature-pipeline)
[ ] Feature pack read FIRST (docs/features/<slug>/ — GRILL → PRD-TRD → IMPL)
[ ] Code mapped to the docs (the drift is the doc-vs-code gap)
[ ] STEP 2a: resume handoff written BEFORE implementation (multi-surface / subtle fixes)
[ ] TDD: test → fix → test (no code before its test)
[ ] Root cause fixed, not symptom
[ ] Feature docs updated (PRD-TRD clause + IMPL delivery note) — drift read first
[ ] CONTEXT.md glossary updated if a term changed
[ ] Drift log: D-N resolved/annotated, counts updated
[ ] Precommit chain + full suite green (pre-existing fails named)
[ ] committed; branch pushed + PR opened (gh pr create)
[ ] PR URL linked in the Victor review event
[ ] Merge via the PR after approval (never local merge to main)
[ ] Handoff written (refresh of the step 2a checkpoint)
```

## Example (the archetype)

`.agentide/` data-dir unification (Option C, global default): a discovered
operator issue ("the CLI creates `.agentide/` in every repo") with a locked design
already discussed. No GRILL, no PRD-TRD from scratch — instead: one small resolver
helper in the CLI + a config key + tests, then amend the `cli-restructure` PRD-TRD
(S1/S8 context clauses) + IMPL delivery note, update the `agentide-cli-consumer`
config docs, resolve/annotate the drift entry, commit `fix(agentide): ...`.

## See also

- `feature-pipeline` / `execute-pack` — when triage escalates (new surface)
- `drift` — read before doc edits, write new divergences
- `feature-pipeline-review` — fresh-eyes check after the change, if behavior shifted
- `tdd`, `handoff` — same tools, surgical scale