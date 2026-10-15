# 13 — What lives on disk under `.factory/`?

**Type:** grilling
**Status:** open
**Blocked by:** 01, 02
**Claimed by:** —

## Question

What is the on-disk shape of a repository after the factory has run — where do issues, evidence,
repros, harnesses, known failures, and run history live, and what is gitignored?

## Blocked by 01 and 02 because

Evidence layout depends on the Finding format (ticket 01). What gets committed and cleaned up
depends on the branch strategy (ticket 02).

## Why this matters more than it looks

The Scout needs a **memory**, or the loop repeats itself forever:

```
merge v2.1 → scout finds "flaky timestamp" → issue #41
merge v2.2 → scout finds "flaky timestamp" → issue #67   ← same bug
merge v2.3 → scout finds "flaky timestamp" → issue #88   ← still!
```

Without a record of known failures, the scout rediscovers the same bug every cycle and burns
budget indefinitely. This is not a nice-to-have; it is what makes the loop converge instead of
diverge.

## The shape we sketched

```yaml
.factory/
├── harness.yaml          # auto-detected app cache. Human edits once.
├── issues/               # open + closed, in whatever format ticket 01 picks
├── evidence/             # repro scripts, screenshots, terminal captures
├── known-failures/       # "we know about this, stop telling us"
└── runs/                 # per-cycle logs, agent transcripts
```

## What to decide

### 1. Commit or ignore?

| Path | In git? | Why it matters |
|---|---|---|
| `harness.yaml` | ✅ commit | shared config, reviewable in a PR |
| `issues/` | ✅ commit | the work queue should be reviewable |
| `evidence/` | ? | screenshots are binary; repro scripts are text |
| `known-failures/` | ✅ commit | shared memory across machines |
| `runs/` | ❌ ignore | per-machine, huge, noisy |

**The line matters.** If evidence is committed, repos fill with PNGs. If it is ignored, the
Reviewer on another machine cannot replay the repro. (Fog: does review happen on the same
machine? If yes, ignore evidence and it is fine.)

### 2. Issue format

Markdown files? YAML front-matter? JSON? Has to be both human-readable *and* machine-parseable,
and ideally round-trips to GitHub Issues later. (Fog: two-way sync.)

### 3. Evidence retention

Screenshots and captures are big. Do they expire? Do they get pruned after the issue closes?
After N cycles?

### 4. Sandbox artifacts

Where does the sandbox's own state live — a Docker volume, a container name, a `gh` session?
What is cleaned up when a cycle dies halfway? (Fog, blocked on 02.)

### 5. Worktree location

`agent-worktree` puts worktrees at
`~/.config/superpowers/worktrees/<project>/<branch>` — **outside the repo**, shared across all
projects. Keep that, or move under `.sandcastle/worktrees/` like Sandcastle does?

Trade-off: global location means one `git worktree prune` spot and no risk of committing it.
Per-repo means everything about a repo is in one place, at the cost of polluting the repo tree.

### 6. Naming and collisions

`.factory/` vs `.sandcastle/` vs something else. Does the factory ever operate on a repo that
already has a `.sandcastle/` directory (i.e., a Sandcastle user)?

## Deliverable

A directory tree with a per-path in-git / ignored decision, the issue file format, and the
retention rule. Plus the worktree location decision.

## Known

- `.sandcastle/` already exists as a convention (Dockerfile, prompts, logs, worktrees, env). We
  are not using Sandcastle, but users may have one. Coexist or ignore?
- Sandcastle gitignores `.env`, `logs/`, `worktrees/` inside `.sandcastle/`.