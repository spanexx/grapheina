# 10 — Which probes are valid for which app types?

**Type:** grilling
**Status:** closed
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
## Resolution

### Two kinds of output, not one

Ticket 01's rule - *a Finding exists when its script failed twice* - holds for most Probes. But some
things cannot be replayed at all.

| | |
|---|---|
| **crash** | reproducible - the script **is** the repro |
| **stress** | reproducible - a fixed Seed and clock (ticket 18) give the same result twice |
| **security** | reproducible - a request is accepted or it isn't |
| **regression** | reproducible - a test passes or it doesn't |
| **usability** | **cannot** - nobody has to be confused twice |

**One Probe out of five.** The rule survives almost entirely, which is a much smaller hole than it
first looked.

`
xx   a Finding
xv   a Finding of unreliability
vv   a ghost - dropped
--   ------------------------------------------------------------
?    an Observation - real, worth keeping, not provable by replay
`

> **An Observation never becomes an Issue.** It goes in the report. A human promotes one by writing
> a Repro for it - and that is the gate.

### The matrix

| | crash | stress | security | usability | regression |
|---|---|---|---|---|---|
| **web page** | yes | yes | yes | yes | yes |
| **service** | yes | yes | yes | ~ | yes |
| **command line** | yes | yes | ~ | no | yes |
| **library** | yes | yes | ~ | no | yes |
| **worker** | yes | yes | ~ | no | yes |

`
yes  runs, and is meaningful here
~    runs, but is WEAK here - and must say so
no   cannot run. Not a gap, not a failure. The wrong question for this thing.
`

### A weak Probe produces notes, never Findings

> Security on a command line program works, but asks a much smaller question than security on a web
> app.

**If a weak Probe were allowed to produce ordinary Findings, the Builder would act on something
half-answered and the Reviewer would have no way of knowing.**

> **A strong Probe produces Findings**, which can send work to a Builder unattended.
> **A weak Probe produces notes only.** Notes go in the report for a human. **They never become
> work by themselves.**

Same failure as everywhere else in this factory: **something running and looking successful when
it should not have been.**

### v1 runs three Probes

**crash · stress · regression.** The two that either need nothing beyond a running app, or need a
Seed that ticket 01 already settled.

### What each Probe needs before it can run

> A Probe that needs setup it cannot get is **worse than no Probe** - it produces false
> confidence.

| Probe | Needs |
|---|---|
| crash | nothing beyond a running app |
| stress | a Seed, and a measurable target |
| security | credentials it can use, and a way to know what *should* be rejected |
| usability | **a goal, not a script** - "sign up, then change your email" |
| regression | a green baseline to compare against |

**Known gap:** nobody has defined how usability Probe *goals* are written or where they live. Not
blocking v1, which does not run usability.

### What usability would need when it arrives

It needs a **goal**, and a Builder must be able to tell whether it was met - because otherwise an
Observation about a confusing flow can never be promoted into an Issue with a Repro. That is
recorded as fog rather than a ticket, since the Probe itself is out of v1 scope.

## Original notes

### Blocked by 09 (closed) because

We cannot enumerate valid Probes per app type until we can classify app types. Ticket 09 fixed the
five kinds.