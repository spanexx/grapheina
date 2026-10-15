# 24 — The Doctor: one command for the health of the whole system

**Type:** grilling
**Status:** closed
**Blocked by**: —
**Claimed by**: opencode

**Resolution:** see below

## Question

What is `grapheina doctor`, what does it check, what may it fix without asking, and where does a
classifier fit?

## Where this came from

The Doctor started in [ticket 23](23-adoption.md) as one role — a Reviewer whose job was noticing
when code and written ethics drift apart. Raised during ticket 16:

> *"We have to have a doctor. The doctor is a command that checks everything is OK in the system. So
> we need to start early to respect the doctor and connect everything to it. User can easily check
> what is wrong with the system. The doctor should have diagnoses for everything. Everything should
> be connected to the doctor, so the doctor can easily fix every problem... And the installation
> also should go through the doctor, because the doctor would know what to install, especially now
> that we're trying to use a classifier."*

## It is a promotion, not an addition

The Doctor is **not new work**. It is the runner and the registry for checks that already exist:

| Check | Decided in |
|---|---|
| Is git present, working, right version? | ticket 14 |
| Is there a container? can it stream? | ticket 14 |
| Is the Harness stale? can the app start? | tickets 09, 16 |
| Are the base ethics satisfied? which apply here? | ticket 22 |
| Does the code match the written ethics? | ticket 23 |
| Is there drift, and at what severity? | ticket 18 |
| Is the agent installed, authenticated, resumable? | tickets 07, 19 |

**Every part registers its own check. The Doctor is the registry, not a list** — which is why it
should be designed in from the start rather than added later. It is automatically complete the
moment everything else exists.

## What it does

```
grapheina doctor

  ✓ git                    2.45.0
  ✓ worktree               D: reaches a commit in 0.3s
  ⚠ sandbox                no container -> noSandbox (accepted 2026-10-13)
      fix: [install docker] [install WSL] [accept as-is]
  ✗ agent: claude-code      not installed
      fix: [install] [use codex instead]
  ✗ detector: rust          missing -> Scout will report a blind spot
      fix: [install]
```

## Installation goes through the Doctor

**The Doctor is the one place that already knows what the system needs**, so it owns installation:

| | |
|---|---|
| WSL | needs a **reboot** — the Doctor warns before, never during |
| an agent | needs a **key** — the Doctor checks before it tries |
| a Detector | needs the language — already known from detection |

**And it concentrates one rule in one place.** *It may draft; it never touches your machine without
asking.* Everywhere else in the factory this rule is repeated. Here it is enforced once, at the
single component that actually modifies the machine.

## Rules first, classifier for what is unmatched - **LOCKED**

Answer: **B.**

```
   something is wrong
        |
        ├── known failure?  -->  the rule. exact, testable, may FIX,
        |                          because the fix is known and bounded
        |
        └── unrecognised    -->  the classifier: "this looks like X,
                                here's what I'd check, here's what I'd
                                fix, shall I?"   a SUGGESTION
```

Why:
- A Doctor that only knows fixed failures meets the first one it does not know and has nothing to
  say. **"I don't know, here are my best guesses, shall I?" is a useful answer. Silence isn't.**
- **The split is the whole safety story.** A wrong *fix* is damage; a wrong *diagnosis* is
  information. The classifier stays on the harmless side — talking — and the deterministic rules
  stay on the dangerous one — acting.
- Rejected: a classifier driving everything, because it will eventually be confidently wrong with
  a wrench in its hand.

## What to decide

### 1. What may it fix without asking?

Nothing, probably - but it should be a list, not a policy.

- Read-only checks: **always run, never ask**
- Anything that modifies the machine, installs software, or reaches the network: **always asks**
- Is there anything in between? *Something with a dry-run and a safe default?*

**My view:** ask for anything that writes outside `.factory/` and the config. Never ask for reading,
and never for a dry run.

### 2. Does it have one output, or per-context output?

A terminal report for you. Structured output for other agents and for CI. Both, from one run.

**My view: both.** Same as ticket 05's local-and-cloud shape - the *answer* is one thing, the
*rendering* depends on who is reading.

### 3. Does it fail, or report and exit zero?

CI wants a non-zero exit on a problem. You want a readable report. **Both?**

**My view:** non-zero exit when something is broken, always zero when everything is merely
*degraded* - because degraded is a decision someone already made.

### 4. Can the fix be automated later?

If the Doctor fixes the same thing every cycle, should it stop asking?

**My view: yes, but only with a written record** - the same rule as `overrides.yaml` in ticket 22.
An automation nobody approved is not automation, it is a surprise.

### 5. What is it called, given "Doctor" already means the ethics checker?

Same command, two scopes? `grapheina doctor` (the system) and something else (the code)? Or one
command that reports both?

**My view: one command, two sections.** The code checks are the system checks applied to a
specific repository. One thing to remember beats two.

### 6. Does the classifier run offline?

If it is Laya, it runs locally. If it is an API, it needs a key and a network - and a Doctor that
needs a key to tell you what's wrong is no help on a broken machine.

**My view: rules must work with no classifier at all.** The classifier is an upgrade, never a
dependency.

## Deliverable

What `grapheina doctor` checks, what it may fix without asking, its output shapes, its exit
codes, what may be automated later, and how the code-level ethics check relates to the system-level
one.
## Resolution

### Q1 - The blast radius, not the certainty - C

`
 READS                             -> never ask
 WRITES inside the project          -> never ask   (cache, logs, worktrees - its own, gitignored)
 WRITES outside the project         -> always ask  (installs, PATH, registry, system)
`

> **The rule is where the damage stops, not how confident it feels.**

C's danger is that *"inside the project"* quietly expands. So every boundary the Doctor crosses
gets **written down**, by the same rule ticket 22 uses for `overrides.yaml`: a silent assumption
is a bug.

This makes routine runs free (`grapheina doctor` before every Cycle) while guaranteeing that
**nothing touches your machine without you having said yes.**

### Q2 - It recommends, it does not fix

`fix: run docker install` - the command is printed, not run. **Nothing invisible ever touched
your machine.** This concentrates the rule *\"it may draft; it never touches your machine without
asking\"* in the single component that can modify it.

### Q3 - One command, two sections

`grapheina doctor` reports **System**, then **Code**. The code checks are the system checks
applied to a particular repository - one thing to remember beats two.

### Q4 - The exit code carries meaning

| Code | Meaning |
|---|---|
| `0` | all fine |
| `2` | something is broken |
| `4` | broken **and you told me to ignore it** |
| `6` | gave up early - could not finish checking |

**Not `1`** - `1` usually means *crash*, and a crashed Doctor must never look like a broken
check. `4` is the degraded-but-accepted state from ticket 14: **degraded is a decision someone
already made**, so it does not fail CI.

### Q5 - The rendering depends on who is reading

The terminal report and the machine-readable report (for agents and CI) are **the same run**. The
answer is one thing; the rendering is not. Same shape as ticket 05's local-and-cloud.

### Q6 - The classifier is an upgrade, never a dependency

> **The rules must work with nothing installed. A Doctor that needs a key to tell you what is wrong
is no help on a broken machine.**

Combined with the locked rules-first decision (B), the whole safety story is: **deterministic rules
act, the classifier talks.**

### Q7 - Automatic fixing is allowed later, never silently

The same written-record rule as `overrides.yaml` (ticket 22). **An automation nobody approved is
not automation, it is a surprise.**

## The whole design in one picture

`
   something is wrong
        |
        +-- known failure?     -> the rule. exact, testable, recommends a bounded fix
        +-- unrecognised       -> the classifier: "this looks like X, shall I?"  a SUGGESTION
        |
        +-- recommended fix touches only inside the project   -> may run
        +-- recommended fix touches anything outside          -> ALWAYS asks

   exit:  0 fine | 2 broken | 4 accepted-as-degraded | 6 could not finish
   (never 1 - 1 means the Doctor itself broke, and that must be loud)
`

**Every part of the factory registers its own check; the Doctor is the registry, not a list.** It is
automatically complete the moment everything else exists - which is why it belongs in the design
from the start and not as an afterthought.

## Original notes

### Known

- Ticket 05 is the last open item and is a **run**, not a decision - it needs real measurement and a
  plugin slot, neither of which can be settled on paper.
- `cargo` and `rustc` are not installed on this machine. The Doctor would be the thing that
  notices and offers to install them - the first real use of its own contract.