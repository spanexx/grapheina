---
name: execute-pack
description: Execute the next feature pack in the build backlog. Builds a pre-impl simulation, writes PRD-TRD + IMPL, implements all phases, builds a post-impl simulation (different from pre-impl), runs drift check via sub-agent, then reconciles both sims into one canonical script. Use when the user says "execute", "implement the next pack", "continue the backlog".
---

# Execute Pack

> **⛔ ISOLATION GATE — READ THIS FIRST. NON-NEGOTIABLE.**
> **Main is never the workbench.** All feature work happens in a git **worktree**
> on a dedicated branch, created ONLY by the sanctioned CLI — never
> `git branch` / `git switch` / `git worktree add` by hand.
>
> **You may perform NO step of this pipeline — no reading of implementation
> files, no writing docs, no building, no testing, no implementing — until you
> have created the worktree and are INSIDE it.** Grilling, pre-sim, PRD-TRD/IMPL,
> implementation, post-sim, and drift check ALL live in the worktree.
>
> **The gate (tick all before anything else):**
> - [ ] `agent-worktree feature <slug>` (or `fix`/`chore`/`docs` as appropriate)
>       ran from the repo root
> - [ ] ALL subsequent commands run from INSIDE the printed `WORKTREE` path
> - [ ] `git branch --show-current` reports the worktree branch, **NOT** `main`
>
> **Why:** a missed isolation step pollutes shared `main` with unreviewed,
> unverified work. Dirty-tree / name-collision reasons to "stay on main" are NOT
> valid — resolve them via the CLI's report (stash the unrelated work, pick a
> different slug). Never bypass.
>
> **If you already started on main:** STOP. Make the tree clean (commit/stash/
> resolve unrelated changes per the CLI's report), create the worktree, re-apply
> your work inside it, and reset `main` to its last clean commit.

Build a feature pack from scratch: pre-impl simulation → PRD-TRD + IMPL → implement → post-impl simulation (different script) → drift check.

## Step 0 — Boot

0. Invoke the `agent-notify` skill's session-start protocol before other work:
   inspect queued deliveries with `agent-notify drain --dry-run`, deliver only
   worthwhile pokes, run `agent-notify sweep`, and read `pending-victor.md`.
1. Invoke `caveman` skill (compact mode)
2. Invoke `situation-awareness` skill (git state, recent commits)
3. Invoke `tdd` skill
4. Read `AGENTS.md`, `CONTEXT.md`
5. Read backlog (`docs/Feature_Backlog.md`)

## Step 1 — Find next pack

Scan backlog top-to-bottom. First pack whose deps are all SHIPPED is the target. If all blocked, report and stop.

Once the target slug is known, isolate the work before FURTHER inspection — and
before ANY implementation-file read, doc write, build, or test (see the
ISOLATION GATE at the top; all pipeline steps live in the worktree):

```bash
agent-worktree feature <slug>
```

Use the printed absolute `WORKTREE` path. The CLI rejects dirty repositories
and existing branch/worktree collisions — resolve them, do NOT stay on main.
Do not manually create a branch or worktree. `feature-pipeline` owns the shared
isolation and human-gate contract; execute-pack must not duplicate it.

## Step 2 — Grill (if no GRILL exists)

If `docs/features/<slug>/GRILL-<slug>.txt` is missing, invoke `grill-with-docs` skill and produce it. The GRILL is the gate before any simulation work.

If GRILL exists, skip this step. Read it before continuing.

## Step 3 — Build the pre-impl simulation

**Deliverable:** `docs/features/<slug>/simulate-pre.sh` (or `.html`)

This runs **before** the PRD-TRD is written. Its purpose is to surface design issues the GRILL didn't catch — wrong operator workflow, missing concepts, surprises in the data flow. The user runs it and gives feedback.

Since no code exists yet:

- Hardcoded state is fine
- Stubbed behavior is fine
- What matters: the user can interact with the *shape* of the feature

Use `interconnected-simulation` skill conventions. CLI vs UI by asking the user.

**Gate:** user runs the sim, gives feedback. Loop until the design direction is confirmed. Only then move on.

## Step 4 — Write PRD-TRD + IMPL

If `docs/features/<slug>/PRD-TRD-<slug>.md` is missing, write it now (using `feature-pipeline` skill). If it exists, just read it.

Same for `IMPL-<slug>.md`.

Both docs reference the pre-impl sim — that's how the user knows what was tested.

## Step 5 — Orient

- Read project structure docs
- Run `drift read` for open divergences
- Look at sibling packages for patterns
- Read `CONTEXT.md` for canonical terms

## Step 6 — Implement phases bottom-up with strict TDD

Follow IMPL phase plan. Each phase:

1. **🔴 Red** — write one test for the phase's behavior via its public interface. Test fails.
2. **🟢 Green** — minimal code to pass. All existing tests still pass.
3. **🫸 Refactor** — clean up, deepen modules, remove duplication.
4. Repeat until phase complete.

Rules:
- Never write code before its test
- Never write more than one test ahead
- Never refactor while red
- Build, lint, and run the full suite after each green step
- If blocked, leave `NOTE[agent]` inline and continue

## Step 7 — Verify locally

Run the full suite and fix every failure before you commit. CI is the gate that decides whether
this merges — but never open a pull request against a red local suite either.

## Step 8 — Commit

Stage on purpose, path by path. Commit with Conventional Commits. **No `--no-verify`.**

## Step 9 — Update the backlog

Mark the pack SHIPPED and update `docs/Feature_Backlog.md` so the feature graph stays true.
Commit the backlog and graph changes together.

## Step 10 — Build the post-impl simulation

**Different from the pre-impl sim.** The pre-impl sim mirrored the design with hardcoded state. The post-impl sim mirrors the actual implementation. They live side-by-side so the user can compare them.

Use the `interconnected-simulation` skill conventions. Read the real source files for:

- Event names + payload shapes (`events.ts` or equivalent)
- Direct function calls between components
- State ownership (which component writes which key)
- Operator-facing commands

Build `docs/features/<slug>/simulate.sh` (CLI) or `simulate.html` (UI). Decide CLI vs UI by asking the user:

- **CLI** if the operator types commands and reads text output
- **UI** if state is visual (lists, badges, status indicators)

The sim must demonstrate every scenario from PRD-TRD's Behavioral Spec section. After building, run it alongside `simulate-pre.sh` — no surprises.

## Step 11 — Run drift check

Spawn a sub-agent with `feature-pipeline-review` skill. It compares:

- PRD-TRD contract vs implementation behavior
- PRD-TRD simulation contract vs post-impl sim output
- IMPL plan vs what got built

Wait for the sub-agent's report. Decide with the user: fix code, update PRD-TRD/IMPL, or accept drift (log to `docs/drift-issue-log.md`).

## Step 12 — Reconcile simulations

After gaps are settled, collapse the two simulations into one canonical script.

The reconciled sim reflects the locked design intent, the actual implementation behavior, and any drift items the user accepted. Process:

1. Read both `simulate-pre.sh` and `simulate.sh`
2. For each Behavioral Spec scenario, pick the most accurate version between them
3. Where they agree — keep either, simplify
4. Where they disagree — implementation wins (post-impl); mark accepted divergences as "intentional"
5. Delete `simulate-pre.sh` / `.html` (or move to `docs/features/<slug>/archive/`)
6. Keep `simulate.sh` / `.html` as the canonical simulation

The single remaining simulation is the **source of truth** for how this pack behaves. Future agents run this one file to see what the system does.

## Human Gates — inherited from feature-pipeline

Execute-pack uses the same mandatory gates as `feature-pipeline` and does not
create parallel notification events:

- Before merge, create one human-review event with `agent-notify create
  --ask "...evidence and approval request..."`; it auto-subscribes and notifies
  Victor, then stop.
- Before moving `stable`, create one version-blessing event with `agent-notify
  create --ask "...release evidence and blessing request..."`; notify Victor,
  then stop.
- Never add a manual Victor `sub` step; `agent-notify create` owns that invariant.
- Never merge or move `stable` in the same turn as a gate notification.

See the `feature-pipeline` **Human Gates** section for the exact event protocol.

## Step 13 — Handoff

Run `handoff` skill. Include: what was built, the canonical sim path, drift check verdict, what the next pack is.

## Reminders

- Files over 350 lines? Split.
- New domain term? Add to `CONTEXT.md` first.
- No PRD-TRD + IMPL? Run `feature-pipeline` skill.
- Unsure about design? `NOTE[agent]`, simpler path, mention in handoff.
- TDD: red-green-refactor per cycle.
- Two simulations during the pipeline; one after reconciliation. Pre-impl = design feedback, post-impl = reality check, reconciled = canonical.