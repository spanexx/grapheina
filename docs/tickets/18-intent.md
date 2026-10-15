# 18 — Intent: how does the factory know what the application is supposed to do?

**Type:** grilling
**Status:** closed
**Blocked by**: —
**Claimed by**: opencode

**Findings:** [../docs/features/intent/GRILL-intent.txt](../docs/features/intent/GRILL-intent.txt)

## Question

Where does the application's **intent** live, and what makes sure a change serves it rather than
merely being correct?

## Why this is the largest hole found so far

Everything designed so far checks **correctness**:

  - the Repro fails twice  (ticket 01)
  - the sanitiser passes    (ticket 15)
  - the test suite is green
  - the code is clean

**None of it asks whether the code is doing the right thing.**

> Victor: *"There is something more important than getting the code right. And that is the
> following intent of the application."*

### The cup

> Somebody asks for an app that holds a cup. The agent builds something that *technically* holds
> a cup. Every test passes. Reviews clean.
>
> But maybe the cup should sit in the **bottom of the hand**, not the fingers. Maybe it should be
> the **right** hand.
>
> The agent did not get that wrong because it is careless. It got it wrong because **nobody ever
> wrote down which one was wanted.**

So every gate we built says *yes to the wrong thing*. This is the silent degradation identified in
ticket 11, arriving through a different door — not wrong code, but **right code implementing an
unstated intention**.

## What this changes about the loop

Today, for a bug:

```
  Scout finds a symptom  →  Builder fixes the symptom  →  Reviewer checks it works
```

**There is no step where anyone establishes what the correct behaviour actually was.** The Scout
reports what broke. Nobody writes down what should have happened instead. The Builder guesses.

> Victor: *"if the intent is clean, pure, [an agent] can do it."* — so the work is making the
> intent clean, before anything is built. **That is why wayfinder belongs earlier.**

```
  Scout finds a symptom
        ↓
  Wayfinder — what is the intended behaviour here?      ← MISSING TODAY
        ↓  GRILL, locked
  PRD-TRD — the behavioural spec
        ↓
  IMPL — the plan  (already gives us the file list ticket 02 needs)
        ↓
  Builder implements
        ↓
  Reviewer — is the code right?  AND is the behaviour what was specified?
```

## Where intent already lives

The existing workflow already produces it. It is just not wired into the factory.

| Artifact | What it holds |
|---|---|
| `GRILL-<slug>.txt` | the locked decisions, with the reasoning |
| `PRD-TRD-<slug>.md` | "Why this exists" and the **behavioral spec** — what it does, operator-visible |
| `CONTEXT.md` | the ubiquitous language — what the words *mean* |
| `simulate.sh` | the canonical demonstration of how it behaves |
| `docs/drift.md` | where code and docs have diverged |

**The Scout's Finding has none of these.** It is a repro and a scenario. That is enough to fix a
crash and not enough to know you fixed the right thing.

## What to decide

### 1. Does every Fix need a spec, or only new things?

Every change goes through wayfinder + PRD-TRD (very slow, and absurd for a typo).
Every *behaviour* goes through it; a crash fix does not.
The Scout decides, and says which it chose.

**My view:** the second. A one-character fix does not need a GRILL. A change in what the software
*does* does. And the Scout has to say which kind it is, every time — because "I fixed a typo" and
"I decided what correct looks like" are very different claims and only one of them is safe.

### 2. Which changes always need a human to specify behaviour?

Adding a feature. Changing what something means. Anything touching auth, data, or money.
Everything else can be derived from the existing spec.

**My view:** anything that changes a documented behaviour. If `PRD-TRD` says X and the change
makes it Y, a human confirms Y is what they meant. That is a narrow, checkable rule.

### 3. Where does behaviour live for software that has no pack yet?

The Scout finds a bug in a repo nobody ever wrote a GRILL for. There is no spec to check against.
The Scout **invents one** and asks.

**My view: yes, and it proposes rather than assumes.** The same rule as the Reset script and the
Harness: the factory may draft it, a human approves it.

### 4. What about scaffolding — building something from nothing?

> Victor: *"we should also look at scaffolding"*

There is no bug to find and no existing behaviour to preserve. The Scout has nothing to work
from. What does the factory do with an empty directory?

This is likely the real first use — and it may deserve more than a section in this ticket.

### 5. The behaviour gate

Code quality is machine-decidable → **reject** when unsure (ticket 11).
Behaviour is a human value judgement → **escalate** when unsure.

Concretely, what does the Reviewer actually do?

- Re-read the PRD-TRD's behavioural spec and the diff side by side, and answer
  "does this change do what the spec says?"
- If yes → approve.
- If it can tell it does *not* → reject, name the line.
- If it cannot tell → **escalate to a human**, do not approve and do not reject.

**My view: escalating must not be the same as rejecting.** Rejection sends work back to the
Builder, which will try the same thing again. Escalation asks a person, who can say *"no, that's
right"* — which unblocks the work rather than looping it.

### 6. Language-agnostic by construction

> Victor: *"it shouldn't really matter what language... the procedure of stuff holding the
> application should always remain the same."*

The procedure is identical whatever the Detector said: **intent → spec → plan → build → verify
behaviour.** A Rust binary and a Python service go through the same chain. If the procedure ever
diverges by language, that is a bug in the procedure.

## Deliverable

Where intent lives for a repo that has never had a pack, which changes need a human to specify
behaviour, how the behaviour gate differs from the code gate, what happens to scaffolding from
nothing, and how the escalation differs from a rejection.
## Resolution

### Intent is not a wall

A locked decision is locked **for now**. When it changes, that is not a mistake - it is something
that happened for a reason, and **the reason is the valuable part.**

The mechanism already exists: the **drift** skill. Three artifacts - **Docs** (intent), **Code**
(reality), **Drift log** (docs/drift.md). Its own framing: *"Drift is not 'docs wrong, fix them.'
It's 'here's the breadcrumb, here's who reconciles.'"*

Two rules carry the weight:

| Rule | Why |
|---|---|
| **Nothing is deleted.** A drifted pack is **appended to** - *"we decided this, then we ended up here, and this is why"* | the history of a decision changing is worth more than the decision |
| **Every entry needs doc path:line AND code path:line** | no citations, no entry - it is a fact, not an opinion |

### Where drift gets caught

- **The Reviewer** - the moment it is created, with the diff already in front of it
- **A dedicated drift Probe** - what accumulated while nobody was watching

**The factory proposes doc edits and never applies them.** Same rule as the reset script and the
harness: it may draft, a human approves. Docs are what everything else trusts.

### When the factory stops and asks

**Everything proceeds. Drift catches the ones it got wrong.**

Without drift, guessing wrong is permanent and invisible. With drift, guessing wrong is recorded
with a reason, an owner and a way to fix it. This is the trade Victor named: *drift on some things
and still land in the same product.*

The cost is real - some changes will be unwanted. The mitigation is not to guess less but that
**High-severity drift becoming common is itself a visible signal** that the Scout's judgement
needs tuning.

### A repository with no pack at all

The Scout writes a **proposed baseline from the code's actual behaviour**, marked **observed**
rather than **decided**, and a human approves it once.

> **Observed and decided must never look the same in the file.** An observed baseline is a
> description; a decided one is an instruction. Otherwise a snapshot of today's behaviour, bugs
> included, silently becomes the specification.

### The two gates, from ticket 11

Code is machine-decidable, so **reject** when unsure. Behaviour is a value judgement, so
**escalate**. Rejection loops the work; escalation unblocks it.

### Graduated

**[21 - Scaffolding](../tickets/21-scaffolding.md)** - an empty directory is a different job: no
bug to find, no behaviour to preserve, no spec to check against.