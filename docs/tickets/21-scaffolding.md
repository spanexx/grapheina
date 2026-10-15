# 21 — Scaffolding: what does the factory do with an empty directory?

**Type:** grilling
**Status:** closed
**Blocked by**: 18
**Claimed by**: opencode

**Findings:** [../docs/features/scaffolding/GRILL-scaffolding.txt](../docs/features/scaffolding/GRILL-scaffolding.txt)

## Question

Everything else in this map starts from an existing repository. What happens when the factory is
pointed at **nothing** — and what must exist before the loop can run for the first time?

## Why this is a different job

| | Existing repo | Empty directory |
|---|---|---|
| The Scout finds | a bug | **nothing to find yet** |
| Behaviour to preserve | a pack, or the code | **none** |
| The Reviewer checks | against a spec | **against what?** |
| Intent | already locked | **has to be created first** |

Ticket 18 assumes an intent exists. Scaffolding is the case where **it doesn't yet**, and the
factory has to bring one into being before any of the loop makes sense.

## Raised by

> Victor, during ticket 18: *"we should also look at scaffolding. We should create it as a
> ticket."*

## What to decide

### 1. Who decides what to build first?

There is no bug, no issue, no Finding. So what starts the loop?

- Victor states an intent and the factory builds it
- The factory observes something and proposes what to build
- A template catalogue — pick a starting point, then diverge

**My view: Victor states it.** The Scout is superb at finding problems in something that exists
and useless at inventing a first draft. That division of labour should be explicit, not blurred.

### 2. What is the minimum that must exist before the loop can run?

Everything else assumes these. Which does scaffolding have to produce, and which can the factory
make itself?

```
CONTEXT.md          the ubiquitous language — what the words mean
docs/features/...   the pack — why, what it does, how
harness.yaml        how to start it, how to reach it, how to poke it
a Detector          how we work out what this is
a Seed / Reset      how we get it back to a known state
```

### 3. Does scaffolding use wayfinder?

The existing workflow says yes — you cannot build until you can say what you want. Does the
factory run wayfinder unattended, or does it stop and ask Victor questions?

**My view: it stops and asks, then works from the answer.** An agent asked to invent its own
requirements produces confident nonsense.

### 4. Which Detector reads a brand-new project?

Ticket 09 locked one Detector per ecosystem, each reading that ecosystem's manifests. A project
written by the factory has whatever manifests the factory decided to create — so it would always
detect as whatever it scaffolded.

**Is that a problem?** A Rust project scaffolded by the factory detects as Rust, correctly. But
the Detector is then confirming something the factory chose, not observing something real.

### 5. What language?

Ticket 09 fixed v1 Detectors at JS/TS, Python, Rust, Go. Does scaffolding support all four from
day one, or one — and does the choice come from the intent or from the factory's default?

### 6. What is the smallest thing worth building?

The first thing the factory makes should be small enough that a human can read all of it. **What
is that?** A single command that does one thing, with no configuration surface?

**My view: yes.** The first output exists to prove the loop works end to end, not to be useful.
A useful first output will be neither reviewed properly nor used.

## Deliverable

What starts the loop, what scaffolding must produce, whether wayfinder runs unattended, how a
brand-new project is detected, which languages are supported, and what the smallest useful first
output is.
## Resolution

### The process is a conversation, not a form

There is **no rigid rule on scaffolding.** It runs through the **wayfinder** skill: Victor talks
with an agent until they both know what the thing is supposed to be. **No particular way to
scaffold a particular software** - it depends on the agent and the human deciding together.

The first framing offered here - *"Victor states it and the factory builds it"* - was a form, and a
form is the wrong shape for this. Every application is different, and the right questions only
appear once you start asking.

### The process is not fixed. The output is.

> **Every decision states how the thing should behave at runtime.**

`
1. wayfinder — talk it through with an agent      a conversation, not a form
2. confirm the technical side                      what it is, which language, which Detector
3. THEN enforce: every decision states its runtime behaviour
`

The order matters: you cannot tell someone what it should do until you both know what it is. So
the technical side is confirmed first and the behaviour is held second.

### Repo ethics are scaffolded too

Files under a length, packs exist, secrets are forbidden - these are **not questions**, they are
rules. They live in the repo, AGENTS.md points at them, and a Reviewer refuses work that breaks
them.

**Six of these were already decided across this map and none had a name or a home.**

Graduated: **[22 - Repo ethics](../tickets/22-repo-ethics.md)** - where they live, the two tiers,
and crucially what the ethics Reviewer may refuse on.