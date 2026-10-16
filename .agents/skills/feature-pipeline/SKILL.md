---
name: feature-pipeline
description: Lean feature delivery. GRILL → Pre-impl Sim → PRD-TRD → IMPL → Implement → Post-impl Sim (different script) → Drift Check → Reconcile (collapse to one canonical sim). Use when the user says "build this", "ship X", "implement the plan", or after grill-with-docs locks scope.
---

# Feature Pipeline

> **⛔ ISOLATION GATE — READ THIS FIRST. NON-NEGOTIABLE.**
> **Main is never the workbench.** All feature work happens in a git **worktree**
> on a dedicated branch, created ONLY by the sanctioned CLI — never
> `git branch` / `git switch` / `git worktree add` by hand.
>
> **You may perform NO step of this pipeline — no reading of implementation
> files, no writing docs, no building, no testing, no implementing — until you
> have created the worktree and are INSIDE it.** That means the grilling,
> pre-sim, PRD-TRD, IMPL, implementation, and post-sim ALL live in the worktree.
>
> **The gate (tick all before anything else):**
> - [ ] `agent-worktree feature <slug>` ran from the repo root
> - [ ] ALL subsequent commands run from INSIDE the printed `WORKTREE` path
> - [ ] `git branch --show-current` reports the worktree branch, **NOT** `main`
>
> **Why:** a missed isolation step pollutes shared `main` with unreviewed,
> unverified work and corrupts the protected-merge model. The dirty-tree or
> name-collision reasons you might reach for to "stay on main" are NOT valid —
> the CLI reports them; you resolve them (stash the unrelated work, pick
> another slug). You do NOT bypass them.
>
> **If you already started on main:** STOP immediately. Make the tree clean
> (commit/stash/resolve unrelated changes per the CLI's report), run
> `agent-worktree feature <slug>`, re-apply your work inside the worktree, and
> reset `main` to its last clean commit. Never leave your edits on main.

```
Bootstrap + Audit → Targeted Exploration → GRILL
                                            ↓
                              Pre-impl Simulation (user runs, finds issues)
                                            ↓
                                       PRD-TRD (one doc)
                                            ↓
                                            IMPL
                                            ↓
                                      Implement (TypeScript)
                                            ↓
                                  Post-impl Simulation (different script)
                                            ↓
                                      Drift Check (sub-agent)
                                            ↓
                              Reconcile (collapse to one canonical sim)
```

Each phase has a deliverable. No phase starts before the previous one is approved.

## Entry

| Situation | Start at |
|---|---|
| Raw idea, not grilled | GRILL |
| Grilled, no pre-impl sim | Pre-impl Simulation |
| Pre-impl sim done, no docs | PRD-TRD |
| PRD-TRD exists, no IMPL | IMPL |
| Mid-implementation | Implement (resume) |
| Code exists, no post-impl sim | Post-impl Simulation |

## Bootstrap + Audit (once)

0. **Start notification handling.** Invoke the `agent-notify` skill's session-start
   protocol (`drain --dry-run`, decide whether any queued delivery is worth
   spending a live mouth on, then `sweep`, and read `pending-victor.md`).
1. **Isolate first — THE GATE ABOVE.** From the current repository, run
   `agent-worktree feature <slug>` BEFORE any other step (before inspecting OR
   writing anything). Use the printed absolute `WORKTREE` path for the rest of
   the pipeline. If already inside the intended worktree, continue there; never
   manually run `git branch` or `git worktree add`. The CLI rejects dirty trees
   and existing branch/worktree collisions — resolve them, do NOT stay on main.
   All of: the targeted exploration, GRILL, pre-sim, PRD-TRD, IMPL, implement,
   post-sim, and drift check run INSIDE this worktree.
2. Discover: where `docs/` lives, what `CONTEXT.md` says, what `AGENTS.md` mandates.
3. Scan `docs/features/` for sibling packs already shipped. Copy their file structure.
4. If ambiguous, ask the user.

## Targeted Exploration

Inspect the specific feature's source code **before** writing the PRD. Don't generalize across the repo — read only the files relevant to this feature.

- The package directory: `packages/<slug>/src/`
- Tests: `packages/<slug>/src/__tests__/`
- Any caller or callee that this feature depends on

Output: a short bullet list of "what exists today" and "what the contract looks like." This goes into the PRD's Context section.

## Phase 0: GRILL

Invoke `grill-with-docs` skill. Output: `docs/features/<slug>/GRILL-<slug>.txt`.

**Gate:** Status = Locked, no open questions.

## Phase 0.5: Pre-impl Simulation

**Deliverable:** `docs/features/<slug>/simulate-pre.sh` (or `.html`)

**This runs BEFORE the PRD-TRD.** Its purpose is to surface issues the grilling didn't catch — design flaws, missing concepts, wrong assumptions about operator workflow. The user runs it and gives feedback. Only after the user signs off does the PRD-TRD get written.

The pre-impl sim:

- **Mirrors the design**, not code (no code exists yet at this point)
- Uses placeholder data and stubbed behavior
- Shows the operator flow concretely: what they'd type, what they'd see
- Exposes the user to the *shape* of the feature before the contract is locked

Use `interconnected-simulation` skill conventions. Since the sim is design-time:

- Hardcoded state is fine (no real events yet)
- Real commands are fine (the user types them)
- Output should make the design *visible* — what changes, what the operator observes

**Gate:** user runs the sim, gives feedback, confirms the design direction is right. Loop until stable.

## Phase 1: PRD + TRD (one document)

**Deliverable:** `docs/features/<slug>/PRD-TRD-<slug>.md`

This is the merged document. It contains:

1. **Why this exists** — problem it solves (lifted from GRILL)
2. **What it does** — behavioral spec (operator-visible actions, not internals)
3. **Simulation contract** — what the post-impl sim must demonstrate
4. **Technical design** — data models, API contracts, dependency analysis, opensrc findings
5. **Non-goals** — what this does NOT do

See `PRD-TRD-FORMAT.md` (in this directory) for the exact template.

**Gate:** user approves the document. Don't proceed until approval.

## Phase 2: IMPL

**Deliverable:** `docs/features/<slug>/IMPL-<slug>.md`

This is the implementation phase checklist. Separate from PRD+TRD because:

- PRD+TRD is the *contract* (what + how)
- IMPL is the *execution* (when, in what order, what tests prove each phase)

See `IMPL-FORMAT.md` (in this directory) for the template.

**Gate:** user approves phase order + scope.

## Phase 3: Implement

For each phase in IMPL:

1. Read the phase
2. Verify no existing behavior replaced without migration
3. Implement — minimal diff, follow the contract exactly
4. Validate: build && test && lint
5. Move to next phase

**Anti-drift:** every phase check it serves PRD goals and behaves per the simulation contract. No additions beyond IMPL plan without doc update.

## Human Gates — mandatory Victor notification and stop

These two gates are non-negotiable. Run them in the same turn as the step that
precedes them, then **stop**. No subsequent step runs until Victor replies.
`agent-notify create` automatically subscribes Victor, notifies him by text AND
a spoken voice message (event id, title, publisher, ask — no raw timestamps),
and queues the event for `pending-victor.md` follow-up. Agents never add a
separate `sub` step.

### Human review + approval

Run after implementation, post-impl sim, drift decisions, and full verification
are complete — before any merge or release action. **The PR comes first:** push
the worktree branch and open the pull request — the PR is the review artifact
Victor approves:

```bash
# 1. Push the feature branch and open the PR.
git push -u origin <branch>
gh pr create --base main --head <branch> \
  --title "feat(<scope>): <one-liner>" \
  --body-file <pr-body.md>   # PRD-TRD summary, IMPL phases, test/sim evidence, docs touched

# 2. The review event must carry the PR URL.
agent-notify create <project>-<slug>-human-review \
  --title "Human review required: <project> <slug>" \
  --ask "PR: <url>. Evidence: PRD-TRD, IMPL, simulation, drift report, and verify output are ready. Review the pull request and approve or request changes." \
  --as <agent-name>
```

Do not merge, close the worktree, or continue to release preparation until
Victor replies. On the next turn, read `agent-notify show <event-id>` and
honor the reply.

**Merging is PR-only.** After Victor approves, merge through the PR
(`gh pr merge --squash`, or the UI) — never a local merge/push to `main`.
Subsequent review-fix commits go to the same branch and update the PR.
Closing the worktree happens after the PR merge lands.

### Victor blesses stable?

Run after the annotated version tag and install/release smoke evidence exist —
before moving `stable`:

```bash
agent-notify create <project>-<version>-stable \
  --title "Victor decision required: bless <version> as stable" \
  --ask "Evidence: annotated tag, full verification, install gate, health check, and release smoke are complete. Move stable only if you explicitly bless this version." \
  --as <agent-name>
```

Never move `stable` in the same turn as the notification. Resume only after
Victor explicitly blesses that exact version.

## Phase 4: Post-impl Simulation

**Deliverable:** `docs/features/<slug>/simulate.sh` (and/or `simulate.html`)

**Different from the pre-impl simulation.** Where the pre-impl sim mirrored the design with hardcoded state, the post-impl sim mirrors the actual implementation. The user can run both side-by-side to spot drift between what was designed and what was built.

How to build it:

1. Inspect `packages/<slug>/src/` for the real public API
2. List events the component publishes (event names + payload shapes)
3. List direct calls the component makes to other components
4. Identify what state the sim needs to track
5. Build the sim using `interconnected-simulation` skill conventions

UI vs CLI: ask the user. If operator-facing (CLI commands, dashboards), prefer CLI. If state is visual (plugin list with badges, event log timeline), prefer HTML.

The post-impl sim must demonstrate every scenario from PRD-TRD's Behavioral Spec section.

**Gate:** sim runs, all expected behaviors are demonstrated, and running it alongside `simulate-pre.sh` doesn't reveal any unexpected divergences.

## Phase 5: Drift Check

Spawn a sub-agent (via `feature-pipeline-review` skill). It compares:

- PRD-TRD contract vs real implementation behavior
- PRD-TRD simulation contract vs post-impl sim output
- IMPL plan vs what got built

Output: `.reports/<timestamp>-drift.md` with gap list.

**Gate:** user reviews gaps. Decide: fix code, update PRD-TRD/IMPL, or accept drift (logged).

## Phase 6: Reconcile Simulations

After gaps are settled (code fixed, docs updated, or drift accepted), the two simulations collapse into one. The reconciled sim reflects:

- The locked design intent (from pre-impl)
- The actual implementation behavior (from post-impl)
- Any drift items that were accepted (now part of the canonical behavior)

**Process:**

1. Read both `simulate-pre.sh` and `simulate.sh`
2. For every scenario in PRD-TRD's Behavioral Spec, pick the most accurate version between the two
3. Where they agree — keep either, simplify
4. Where they disagree — the implementation wins (post-impl), but document the divergence as "intentional" if the user accepted it
5. Delete `simulate-pre.sh` and `simulate-pre.html` (or archive them in `docs/features/<slug>/archive/`)
6. Keep `simulate.sh` (or `.html`) as the canonical simulation

The reconciled sim is the **single source of truth** for how this pack behaves. Future agents reading the docs run this one file to see what the system does.

**Gate:** `simulate-pre.sh` removed, `simulate.sh` reflects both design intent and shipped behavior, no scenario contradicts the Behavioral Spec.

## Deliverable Checklist

- [ ] Bootstrap done (CONTEXT.md, AGENTS.md read; sibling packs scanned)
- [ ] Targeted exploration recorded
- [ ] GRILL exists + locked
- [ ] Pre-impl simulation exists, run by user, design confirmed
- [ ] PRD-TRD exists + approved
- [ ] IMPL exists + approved
- [ ] Implementation phases complete + validated
- [ ] Tests pass
- [ ] Post-impl simulation runs + demonstrates contract
- [ ] Post-impl sim compared against pre-impl sim (no surprises)
- [ ] Drift check ran + gaps resolved or logged
- [ ] Simulations reconciled into one canonical script; pre-impl archived/removed
- [ ] CONTEXT.md updated if new terms emerged
- [ ] Branch pushed + PR opened (gh pr create); PR URL in the human-review event
- [ ] Human review gate: `agent-notify create ... --ask` sent, pipeline stopped, Victor replied
- [ ] Merge via the PR after approval (never local merge to main); worktree closed after merge
- [ ] Stable gate (release only): `agent-notify create ... --ask` sent, stopped, Victor blessed

## What This Skill Drops

Compared to the prior pipeline, this version removes:

- **FLOW as separate doc** — absorbed into PRD+TRD (Behavioral Spec section)
- **EXPLAINED as separate doc** — operator-facing scenarios live in PRD+TRD now
- **TRD as separate doc** — merged with PRD into PRD-TRD

IMPL stays separate because:

- PRD+TRD is the contract (what + how it should work)
- IMPL is the execution (when, in what order, what tests prove each phase)

If you need a deep technical-design doc with full module diagrams, ADRs, and dependency analysis, write it inline in PRD+TRD as subsections. Don't make it a separate file unless PRD+TRD exceeds 350 lines.

## Naming Convention

```
docs/features/<slug>/
  GRILL-<slug>.txt         (Phase 0)
  simulate-pre.sh          (Phase 0.5, design-time)
  simulate-pre.html        (Phase 0.5, design-time, optional)
  PRD-TRD-<slug>.md        (Phase 1, single doc)
  IMPL-<slug>.md           (Phase 2)
  simulate.sh              (Phase 4 → Phase 6 reconciled, canonical)
  simulate.html            (Phase 4, post-impl, optional)
  archive/                 (Phase 6, optional — superseded artifacts)
```

The two simulations live side-by-side during the pipeline so the user can compare them when looking for drift between what was designed (pre-impl) and what was built (post-impl). They are not interchangeable.

After drift is settled (Phase 6), the two collapse into one. `simulate-pre.sh` is removed (or moved to `archive/`); `simulate.sh` becomes the canonical simulation.

Lowercase-hyphenated topic slugs.

**Before drafting**, scan `docs/features/` for sibling packs. Copy their structure. The skill's reference list may miss artifacts — siblings are the source of truth.