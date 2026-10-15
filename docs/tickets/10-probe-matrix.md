# 10 — Which probes are valid for which app types?

**Type:** grilling
**Status:** open
**Blocked by:** 09
**Claimed by:** —

## Question

Scout is a team of five specialised probes. Which subset applies to a web app, which to a CLI,
which to a library — and what does each probe need in order to run?

## Blocked by 09 because

We cannot enumerate valid probes per app type until we can classify app types.

## The team (from the destination session)

```
① SCOUT
     ├── ①a CRASH HUNTER     wrong input · null/empty · crashes · deadlocks
     ├── ①b STRESS ENGINE    volume · concurrency · duration · leaks
     ├── ①c SECURITY PROBE   authz holes · injection · secrets
     ├── ①d UX READER        can a human actually do the thing?
     └── ①e REGRESSION       did this break what worked?
```

**Why a team, not one agent:** not workload — *search strategy*. An LLM told "find bugs" does
surface fuzzing and stops. It will not run a 40-minute soak to find a 200 MB leak, and it will
not notice a save button that silently does nothing while looking like it worked. One agent
averaging all five searches = shallow on all five.

## What to decide

### 1. The validity matrix

| Probe | CLI | HTTP API | Web UI | Library | Worker |
|---|---|---|---|---|---|
| ①a Crash | ✅ | ✅ | ✅ | ✅ | ✅ |
| ①b Stress | ⚠️ spawn cost | ✅ | ⚠️ expensive | ✅ | ✅ |
| ①c Security | ⚠️ | ✅ | ✅ | ⚠️ | ⚠️ |
| ①d UX | ❌ n/a | ⚠️ minimal | ✅ **primary** | ❌ n/a | ⚠️ |
| ①e Regression | ✅ | ✅ | ✅ | ✅ | ✅ |

Fill this in properly. Which cells are genuinely n/a vs just expensive?

### 2. What each probe needs

Per probe: what must exist before it can run? Playwright installed? A seed dataset? Auth
credentials? Fixtures? A running app? **A probe that needs setup it cannot get is worse than no
probe** — it produces false confidence.

### 3. Tiering (already implied, needs numbers)

```
every cycle:            ①a crash hunter          ~2 min
on schedule (nightly):  + ①b stress              ~30 min
on new deps:            + ①c security            ~15 min
when UX changes:        + ①d ux reader           ~10 min
after every merge:      + ①e regression          ~5 min
```

What decides "on new deps"? A lockfile hash? Which probes are the daily baseline?

### 4. Parallelism and budget

Do probes run concurrently or serially? Per-cycle token budget? What does the Scout do when it
is running out — degrade to fewer probes, or skip the cycle?

### 5. UX Reader — the hardest one

It needs a *goal*, not a script. Does it get a task list ("sign up, then change your email")?
How does it know it succeeded? What does it file when a flow is merely *confusing* rather than
broken — and can the Builder "fix" a UX complaint and have the Repro verify it?

## Deliverable

The filled matrix, per-probe prerequisites, the tier schedule with triggers, and a clear answer
on how many probes v1 ships with.