# 23 — Adopting the factory into an existing project

**Type:** grilling
**Status:** closed
**Blocked by**: 22 (closed)
**Claimed by**: opencode

**Findings:** [../docs/features/adoption/GRILL-adoption.txt](../docs/features/adoption/GRILL-adoption.txt)

## Question

The factory arrives at a codebase that is **already active** — four years old, four hundred
thousand lines, two hundred features, no packs. **What does the factory do, in what order, and
what does it refuse to touch?**

## Checked: nothing covers this

Searched every skill for adoption, onboarding, brownfield, legacy, grandfathering. **Nothing.**

Three pieces are adjacent but none sequences the arrival:

| Piece | What it covers |
|---|---|
| the `drift` skill | the gap between docs and code - can describe existing reality |
| observed baseline (tickets 09, 18) | a repo with no pack gets one, marked *observed* |
| `surgical-change` escalation | "more than ~200 lines, or 3+ packages" -> escalate to feature-pipeline |

**None of them answers: what do you do first?**

## Why adoption is a different problem from scaffolding

Scaffolding starts from nothing. **Adoption starts from everything**, and the rules cannot apply
retroactively:

```
you cannot demand a pack for 200 features that have never had one
you cannot apply a 350-line cap to code that already breaks it
you cannot rewrite four years of documentation
you cannot re-run a Scout across a codebase with no Harness (ticket 16)
```

## The core tension

> Every rule must apply to **new** work and **grandfather** the old — or the factory is refused
> on day one.

And then: **something has to record what is being adopted and what is being left behind**, or in
a year nobody knows which is which.

## What to decide

### 1. What is the first thing the factory does when it arrives?

Options worth weighing:

- **Run `drift read` across the whole codebase** and produce a picture of reality before touching
  anything
- **Start on one directory** — pick the newest code, bring *that* under the rules, prove it works
- **Do nothing until a human says which part to start with**

**My view: drift read first, then one directory.** A full picture is cheap and it prevents the
factory guessing. But acting on all of it at once is how you get refused.

### 2. What is grandfathered, and how is that recorded?

Every existing violation of the six base ethics is grandfathered. But where does that live?

- A file listing every grandfathered violation
- A rule: **ethics apply to files the factory touches**, everything else is exempt until touched
- An explicit adoption pack recording what was waived

**My view: the second, plus a count.** The rule is simple enough to explain in one sentence. The
count matters because **"2,400 grandfathered files" and "12" are very different conversations**,
and nobody should discover which one you have by scrolling.

### 3. Does the Scout run across everything, or only on Harness-able parts?

Ticket 16 established a Scout needs a Harness to probe anything. A four-year-old codebase may have
no Harness anywhere.

**Does the factory build Harnesses for existing parts?** That is a lot of work and possibly the
largest single cost of adoption.

### 4. Who decides the order?

A human has to say which parts matter. Is there a principled default — newest first, most-changed
first, most-bugged first?

**My view: a human decides, with the factory proposing.** Most-changed-first is defensible (that
code will change again anyway) but it's the human's call.

### 5. What does the factory refuse to do, ever?

- Rewrite history
- Bulk-reformat existing code
- Retrofit packs for old features "while we're here"
- Refactor to satisfy an ethic nobody asked for

**My view: all four, stated explicitly**, because each is something an eager agent will happily do.

### 6. How does the human know adoption is going well?

What would tell them it is working — and what would tell them it is not?

**My view: the same signal as everywhere else.** A rising count of grandfathered files means the
factory is touching things it shouldn't. Drift severity distribution tells you whether new work
is actually clean.

## Deliverable

The first thing the factory does on arrival, what is grandfathered and where that is recorded,
whether existing parts get Harnesses, who decides the order, the hard refusals, and how progress
is judged.
## Resolution

### Adoption inverts: the factory adapts itself, not the code

> **The code is not the problem.** Four years of work by people who knew what they were doing - and
> it is the evidence of how this project actually thinks.

The factory **does not touch the code**. It reads it, then writes the ethics down: *"this is the
ethic here, this is how we do it, this is why."*

### The Doctor

A kind of Reviewer whose only job is **to notice when the code and the written ethics have drifted
apart.** Not to fix the code - to notice. Once proven, the state is documented.

The ethics are what people learn from. **If they go out of date, every new agent learns the wrong
thing.**

### Two kinds of ethics, pulling opposite ways

| | Pull |
|---|---|
| **Project ethics** - naming from CONTEXT.md, local conventions | the **system realigns to them** |
| **Base ethics** - file cap, no secrets, one job per module | the **system adapts to them** |

> *It should not change the way the system has been operating. It should get work that does not
> conflict.*

### The implementation process is pluggable

**The same pattern as Detectors (per language) and ethics (per project).** Feature-pipeline and
surgical-change are *one* implementation. Plain PRs, or an existing CI/CD, are others.

`
docs/ethics/
├── base/          what the factory itself needs
├── process/       only if the project HAS one of these
└── this-repo/     detected from how the project actually works
`

**The Doctor does not impose a process.** It notices which one is there and applies the matching
ethics. A project that does not comment today gets advised, not failed.

### Enforce only what is detected

Silently ignore the rest. Offering the process was rejected - the failure mode is the factory
selling its house to a project that did not ask.

### Corrections to closed tickets

| Ticket | Was | Now |
|---|---|---|
| [22](22-repo-ethics.md) | comment-cites-the-pack as a **base** ethic | conditional on the project using packs |
| [02](02-branch-strategy.md) | "packs are law" | true inside a process that uses packs |

**The six base ethics are not sacred.** They are a starting set a project can change.