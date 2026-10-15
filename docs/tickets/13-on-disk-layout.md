# 13 — What lives on disk under `.factory/`?

**Type:** grilling
**Status:** closed
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
## Resolution

### The shape

Most of this was already decided across other tickets; the layout just gathers it:

`
<repo>/
  CONTEXT.md                        the ubiquitous language (ticket 09)
  grapheina.yaml                    the release contract (ticket 17)
  docs/
    features/                       packs - GRILL, PRD-TRD, IMPL
    ethics/
      base/                          what the factory needs (ticket 22)
      this-repo/                     what this project adds
      overrides.yaml                 disagreements, with reasons
    drift.md                         Open / Resolved, append-only (ticket 18)
    findings/<n>/
      repro                          the script              COMMITTED
      scenario.yaml                  the scenario            COMMITTED
      meta.yaml                      machine, probe, seed    COMMITTED
      expected-failure.txt           what it failed with     COMMITTED
      failure.txt                    the output             ignored
      screenshot-*.png               the pictures           ignored
      demonstration.webm / trace.zip                         ignored
    sessions/
      .last-handoff                  the handoff (ticket 19 - already built, works)
      .handoff-log.md
    reports/                         cycle reports, agent transcripts

~/.grapheina/
  worktrees/<project>/<branch>/     SCRATCH - outside the repo
  logs/
  cache/
`

### Worktrees sit outside the repository - C

> **A worktree is not part of the project. It is a scratch copy of it.** It belongs beside the
> project, not inside it.

**Why not inside:** a worktree inside the repository is one .gitignore slip from disaster.
Sandcastle puts them under .sandcastle/ and then has to gitignore it - which works until someone
doesn't.

**Why not purely outside:** it becomes invisible. With several repos you hunt through a global
folder wondering which worktree belongs to what.

So: **outside by default, inside only when the project asks** - `worktrees: local` in its own
config, where it is a decision rather than an accident.

### Text commits, pictures do not

| Committed | Ignored |
|---|---|
| `repro` - the script | ailure.txt |
| `scenario.yaml` | screenshot-*.png |
| `meta.yaml` - machine, probe, seed | demonstration.webm |
| `expected-failure.txt` | 	race.zip |
| | udit/ - raw agent logs |

**Both arguments are real.** The Reviewer must be able to replay - that is the interlock (ticket 11),
not a nicety. And screenshots are big. Splitting by kind rather than by size resolves it.

### The check that makes it work

> **If a Reviewer cannot replay a Finding, it is not allowed to approve it.**

`
   evidence is on disk only, another machine
        |
   Reviewer cannot replay
        |
   Reviewer ESCALATES. It does not approve.
`

**That fails safe.** It cannot accidentally approve something it could not verify - it stops and
asks.

The pictures are **regenerable**: the Repro *is* the recipe for the screenshot.

## Still open - deliberately not decided here

Victor: **"agreed for now"** - recorded as provisional, not settled.

- **Retention.** Pictures and ailure.txt have no expiry rule yet. They regenerate, but nothing
  sweeps them.
- **Whether text-only evidence is enough forever.** The "Reviewer escalates if it cannot replay"
  path is correct but *expensive* - it turns a routine cycle into a human interruption. If that
  turns out to happen often, the answer may be to commit more, not less.

**Worth revisiting after the first real Cycle.** The gap analysis flagged this as the cheapest
thing to get wrong late, because it degrades quietly.

## Original notes

### Blocked by 17 and 18 (both closed)

Evidence layout depends on the Finding format (ticket 01). What gets committed and cleaned up
depends on the branch strategy (ticket 02). Both are now closed.

### Known

- `.sandcastle/` already exists as a convention (Dockerfile, prompts, logs, worktrees, env). We
  are not using Sandcastle, but users may have one. **Coexist, do not collide.**
- Sandcastle gitignores `.env`, `logs/`, `worktrees/` inside `.sandcastle/`.