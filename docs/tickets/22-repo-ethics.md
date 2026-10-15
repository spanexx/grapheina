# 22 — Repo ethics: how the code is written

**Type:** grilling
**Status:** closed
**Blocked by**: 21
**Claimed by**: opencode

**Findings:** [../docs/features/repo-ethics/GRILL-repo-ethics.txt](../docs/features/repo-ethics/GRILL-repo-ethics.txt)

## Question

Where do a repository's ethics live, what makes one, and **how does a machine tell a checkable
ethic from an opinion?**

## Why this is its own thing

Behaviour is what the software *does* (ticket 18). **Ethics are how the code is written.**

They are separate because they are separate in effect:

> A file being 340 lines instead of 200 changes **nothing for the user**. It changes everything for
> the next agent - and for us, because long files are exactly what make agents collide.

And ticket 02 already proved the point: *"the 350-line cap is throughput, not style."*

## This is a generalisation, not a new idea

We have been inventing ethics piecemeal all through this map, without naming them:

| Ethic | Where it was decided | Checkable? |
|---|---|---|
| Files under 350 lines | ticket 02 | ✅ measurable |
| Every feature has a pack | ticket 02 | ✅ file exists |
| Surgical changes update the docs they affect | ticket 02 | ✅ file exists |
| No inline credentials in a repro | ticket 15 | ✅ pattern |
| Approved tests run in CI | ticket 02 | ✅ pipeline |
| Ambiguity is named, never guessed | ticket 09 | ⚠️ judgement |
| Behaviour is right, not just correct | ticket 18 | ⚠️ judgement |

**Six of them are already enforced. None of them has a name or a home.**

## What Victor asked for

> *"We should have something like an ethic that we follow and this should be a scaffold too."*
>
> *"If a user or a human prefer modular structure, then this doesn't have to be a question. It
> should be like an ethic that the agents should follow."*
>
> *"If we use a preferred each file to be 200 lines long, then that's an ethic."*
>
> *"The repo should have a folder inside the docs... maybe we should enforce this, so whenever the
> agent wants to work, the AGENTS.md can refer them to these ethics that they must follow, and we
> should have a reviewer that refused this."*

## The finding that shapes everything

> Some of these can be checked by a machine and some cannot.

```
a file is 340 lines, the rule says 200    ← a fact.  ✅
"this code should be readable"            ← an opinion. ❌
```

**If both live in one list and the Reviewer may refuse either, the Reviewer learns to wave through
the opinions to get to the facts — and then it stops refusing anything.**

That is the same failure mode ticket 15 named: a gate that blocks on vague suspicion gets disabled,
and a gate that gets clicked past is worse than no gate.

## What to decide

### 1. Two tiers, or one list?

```
docs/ethics/
├── rules.yaml     ← data.  A machine reads it and refuses. HARD.
└── *.md           ← prose.  Guidance and aspiration.  Never blocks.
```

**My view: two tiers, and only the data tier blocks.** A file-length rule in prose is a suggestion.
The same rule in data is a rule.

The naming matters: if the blocking tier is called `ethics` too, the distinction is lost. Maybe
`ethics/rules.yaml` blocks, `ethics/*.md` advises.

### 2. Who writes them?

Scaffolded at project creation (ticket 21), then amended — who amends, and does it need approval?

**My view: the same rule as everywhere else in this map.** The factory may **draft**, a human
**approves**. Ethics are rules the Reviewer enforces, so the factory inventing them means the
factory enforces rules it wrote for itself.

### 3. What is the minimum viable set?

Every project has the same starting rules regardless of language. Which?

- file length cap
- naming conventions
- no secrets, ever
- every feature has a pack
- modules have one job each

### 4. Does every repo get its own, or is there a shared base?

A shared base every repo starts from, extended per repo. Or entirely per-repo, chosen from
scratch each time?

**My view: shared base.** The factory's own rules are language-agnostic — that was locked in
ticket 18. Duplicating them into every repo means they drift apart silently.

### 5. Can a repo relax a rule?

Some projects genuinely need longer files, or no packs.

**My view: yes, explicitly and visibly.** A project that relaxes a base rule says so in
`rules.yaml`, and the reason is right there. A project that relaxes it *quietly* is the failure
mode.

### 6. What does the ethics Reviewer actually refuse on?

Only the data tier. But: does one violation block the whole change, or is there a severity?

A long file is probably **Medium** — fix it, don't halt. An inline credential is **Critical** —
refuse outright. That maps onto the drift severity scale already in use.

## Deliverable

Where ethics live, the two tiers and their names, who writes them, the minimum set, whether there
is a shared base, whether a repo may relax a rule, and what the ethics Reviewer refuses on.
## Resolution

### Two layers

| | |
|---|---|
| **Base** | ships with the factory, applies everywhere. **The human changes whatever they disagree with** |
| **Repo** | what the project adds for itself |

### Six base ethics - CORRECTED by ticket 23

> The comment rule was written as **base**, but it only means anything if the project uses packs.
> **Base now means "what the factory itself needs to function."** Process-dependent ethics live in
> their own tier. The six are a starting set, not sacred.

| # | Ethic | Tier |
|---|---|---|
| 1 | **Comment** - why, referencing the pack or surgical change | **process** - only if the project has one |
| 2 | **File length cap** - and it is *throughput*, not style | base |
| 3 | **Every feature has a pack** - packs are law | **process** |
| 4 | **Surgical changes update the docs they affect** | **process** |
| 5 | **No secrets, ever** | base |
| 6 | **One job per module** | base |

**Naming is repo-level and reads from CONTEXT.md.** If the glossary calls it a *session*, the code
calls it a session - naming follows the project's language.

The test for base vs process vs repo: **would this still make sense for a project the factory has
never seen, working in a completely different way?** Yes -> base. Needs the project to already have
something -> process. Depends on this project's preferences -> repo.
### A project may disagree - loudly

`
# overrides.yaml
file-length:
  rule: 350
  changed_to: 800
  why: "A compiler pass is one cohesive unit; splitting it
        produces abstractions nobody wants."
  agreed: 2026-10-13
`

**The failure mode is not that somebody disagrees** - a compiler and a CRUD app have different
shapes. **The failure mode is disagreeing quietly**, so the default later looks arbitrary and
nobody remembers why.

> **An override without a reason is a bug. An override with one is a decision.**

Works both ways: a project that *tightens* a rule records it the same way, so effort is visible too.

### The Reviewer has three responses, not two

`
  something small is wrong   ->  FIX IT ITSELF, same session
  something real is wrong    ->  HAND BACK to the Builder
  something about intent     ->  ASK A PERSON
`

This **revises ticket 11**, which had modelled rejection as the only response. It is the largest
single thing in this ticket:

- **Low** - missing comment, formatting, stray import -> fix in place, seconds
- **High** - wrong behaviour, missing pack -> hand back, a cycle
- **Critical** - intent unclear, behaviour contradicts the spec -> a person

The comment ethic is the worked example. Sending a missing comment back to the Builder costs a
whole cycle for something that takes ten seconds.

> **The Builder always corrects. It does not stop.** Rejection is a hand-back with the context
> still attached, not a dead end.

### Checkable versus opinion

`
a file is 340 lines against a rule of 200    <- a FACT.  mechanically checked.
"this code should be readable"               <- an OPINION. never blocks.
`

If a Reviewer may refuse either, it learns to wave through opinions to reach facts - and then
stops refusing anything. **Only the fact tier blocks.** A file-length rule in prose is a
suggestion; the same rule in data is a rule.

Graduated

- **Ticket 23 - Adopting the factory into an existing project.** Raised by Victor. Scaffolding
  starts from nothing; adoption starts from everything, and no existing mechanism covers it.