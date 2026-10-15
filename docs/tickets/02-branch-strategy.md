# 02 — What happens to a branch when the builder finishes?

**Type:** grilling
**Status:** closed
**Blocked by**: —
**Claimed by:** opencode

**Findings:** [../docs/features/branch-strategy/GRILL-branch-strategy.txt](../docs/features/branch-strategy/GRILL-branch-strategy.txt)

## Resolution

### The branch chain

```
  agents (each on its own branch)
        │
        ▼
  ┌──────────────────────────────────────────┐
  │  main    the machine branch              │
  │  merges automatically — D: only when    │
  │  main has not moved since the branch     │
  │  CI runs, must be green                  │
  └───────────────────┬──────────────────────┘
                      │ factory opens this by itself
                      ▼
              pull request  main ──▶ master
                      │
                      ▼
  ┌──────────────────────────────────────────┐
  │  master  the human branch                │
  │  nothing touches it but the human        │
  │  install it where it belongs, use it,    │
  │  approve, merge, bump the version        │
  │  per-repo, not a shared release line     │
  └──────────────────────────────────────────┘
```

**D** was chosen over merging autonomously because two agents branching from the same snapshot
and merging at different times is the collision that actually happens. Refusing is not failure —
it is correct behaviour, and it says why.

### When D refuses

**Rebase onto the new `main` and retry — once.** Rebases often dissolve a conflict on their own.
Only if it still fails does the **merge agent** step in. Starting the whole task over throws
away work that was usually fine.

### Conflicts are predicted, not discovered

The feature pack **already declares the files** — `IMPL-<slug>.md` is a phase checklist, and
`surgical-change` step 2a requires a file inventory in the handoff. Today a human reads those
to resume work after a cut-off session. The factory reads the same lists to know **which two
agents are about to touch the same lines.**

> The feature pack stops being documentation and becomes a booking.

No overlap → full speed. Overlap → take turns. And the declared list is the **first** filter, not
the last: after the agent finishes, what it *actually* changed is compared against what it *said*
it would change. A file outside the declaration is itself a signal.

### Short files are a throughput rule

The 350-line cap is not style. Long files mean large overlap sets, which means fewer things can
run at once. **Long files serialise the factory.** The Scout should treat an oversized file as a
Finding.

### Roles are small teams, and they size themselves

Every role is several focused agents with small briefs, not one agent with a large brief. When
one cannot finish, it **recruits**, with a written request naming how many and where:

```yaml
recruit:
  from: crash-hunter
  count: 3
  reason: "2 modules beyond the brief, each with its own test surface"
  places: [packages/auth/src/session.ts, packages/billing/src/invoice.ts, ...]
```

**Automated — no human picks the answer.** The Builder is several agents in one sandbox,
sequenced, each starting where the last stopped. That number is also a measurement: tasks that
always need splitting are telling you about the shape of the code.

### Rules are law, not documentation

Two rules, both checkable: **every feature has a pack**, and **every surgical change updates the
docs it affects.** Neither survives living only in a file — one agent skips it and nothing
notices. A **rules reviewer** checks them. It does not judge code quality; process compliance and
code quality are different checks and get different agents.

### Graduated

| Moved to | What |
|---|---|
| **11** reviewer rules | The rules reviewer · many single-purpose reviewers · the orchestrator recruiting |
| **12** loop controller | Small-team roles · self-sizing recruitment · the Builder's internal sequence |
| **17** release contract | The deployment half — approval is not shipping |

## Question

When a Builder finishes work in a sandbox on a worktree, what happens to the branch? Who merges
it, when, and what happens when the merge conflicts?

**Victor asked for this one explicitly** — "I still don't understand branch strategy, git merge."

## The problem in plain terms

```
BUILDER finishes in .sandcastle/worktrees/agent-issue-47/
        ↓
   the commits live on branch  agent/issue-47
        ↓
   ??? ← this ticket
        ↓
   main
```

Somewhere between those two arrows, a decision happens. Get it wrong and you either block
progress forever or corrupt main.

## The three shapes we know exist

| Strategy | What happens | Who resolves conflicts |
|---|---|---|
| **Park** | Branch sits. A human opens a PR later. | The human |
| **Merge-to-head** | Tool merges into your current branch on success | Nobody — `git merge` **throws** on conflict |
| **Always-PR** | Tool always opens a PR, never merges | The Reviewer on the PR, then a human merges |

## What Sandcastle does (and why it is a dead end for us)

`SandboxLifecycle.ts:443` is a bare `git merge`. On conflict (`:446`) it throws, preserves the
branch, and tells you to run it yourself. There is **no automatic conflict resolution**.

Its answer is a *prompt*: `templates/parallel-planner/merge-prompt.md:10` —

> *"If there are merge conflicts, resolve them intelligently by reading both sides"*

…sent to a whole separate **Merger agent** that then runs `npm run typecheck && npm test`.

So Sandcastle does not solve conflicts. It pays an agent to solve them.

## What we know (locked)

- Conflicts are **not solved by code**. The interlock is an agent, if it is an agent at all.
- Our Reviewer already replays the Repro. That is a *stronger* pre-merge check than
  "tests pass", because it proves the specific reported bug is actually fixed.
- v1 runs one cycle on a human trigger. A human is watching.

## What is still fog

- Does a Builder ever merge, or only ever park? (Separation of duties says the thing that writes
  code should not be the thing that lands it.)
- Is the Reviewer on the branch *before* merge, or on a PR after?
- How does main get protected from a bad approve? Branch protection, or a human gate?
- What happens on conflict — a merge agent, a human, or drop the issue back to the queue?
- Do we push branches anywhere, or do they live only as local worktrees?

## Related fog in the map

Deadlock between Builder and Reviewer; two-way GitHub sync; the on-disk layout of `.factory/`
(which depends on where branches and evidence actually live).
---

## Correction - ticket 23

> **"Packs are law" is true inside a process that uses packs.**

The implementation process is **pluggable**. Feature-pipeline and surgical-change are *one*
implementation of it; plain pull requests, or a project's existing CI/CD, are others.

**What changes:**

- The rules Reviewer checks for a pack **only exists where packs exist**
- Conflict prediction from declared file lists (this ticket) reads IMPL where there is one, and
  falls back to whatever the project's process actually declares
- main / master and rule D are unaffected - they are git, not process

**What does not change:** the merge chain, D, rebase-and-retry-once, the merge agent, and the
requirement that the declared file list be compared against what actually changed.