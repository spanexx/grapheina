# 12 — What triggers a cycle, and what happens when one fails?

**Type:** grilling
**Status:** closed
**Blocked by:** 02
**Claimed by:** —

## Question

What starts a cycle, in what order does the loop do things, and what is the failure behaviour
when any step dies?

## Blocked by 02 because

The loop's per-cycle bookkeeping is tied to the branch strategy — what gets cleaned up, what
survives a failure, and when a branch is deleted all depend on ticket 02.

## The v1 constraint, already locked

> *"For the first version, it's okay to make it run and stop for now, but we should also create a
> room where it can be automated."*

This has a precise reading, and it is the most important architectural constraint in the map:

> **v1 is the real loop with exactly one iteration, triggered by a human pressing Enter.**
> Automation is a change of *trigger*. Nothing else moves.

Not "a simpler version of automation." The same code path. If we build a special manual mode,
we will build it wrong and the automation will be a rewrite.

## What to decide

### 1. The trigger interface

```rust
enum Trigger {
    Once,                    // v1 — human presses Enter
    Interval(Duration),      // later
    OnSchedule(Vec<Schedule>),// later — nightly stress
    Webhook,                 // later
}
```

Does v1 ship all four with only `Once` wired? Or ship `Once` and add variants as fog clears?

### 2. Cycle order

```
trigger → scout → [issues?] → for each issue: builder → reviewer
                     ↓
              nothing found? ──→ stop or loop?
```

One issue per cycle, or all of them? Sandcastle's `parallel-planner` does **all unblocked issues
per cycle**, capped at 4 concurrent, then a merge step, then repeats up to 10 times. Copy that
shape or simplify?

### 3. Failure taxonomy

Every step can fail. For each, what does the factory do?

| Step | Failure | Options |
|---|---|---|
| Scout | harness won't boot | skip cycle · fall back to fewer probes · alert |
| Scout | probe crashes mid-run | keep partial · discard |
| Scout | finding won't reproduce | **drop as ghost** (already locked) |
| Writer | issue malformed | retry · drop |
| Builder | agent errors | retry · next issue · stop |
| Builder | agent hangs | timeout → kill, keep work? |
| Reviewer | replay fails | reject → Builder (attempts left?) |
| Reviewer | reviewer itself errors | retry · escalate to human |
| Merger | merge conflict | merger agent · human · drop |

**Which of these stop the cycle vs skip and continue?** That distinction is the difference
between a factory that survives a bad afternoon and one that dies at 3am.

### 4. Budget and backpressure

An autonomous loop spends forever. Per-cycle token cap? Daily cap? When budget is spent, does
it degrade probes or stop?

### 5. Loop safety

- **Max cycles** before stopping and reporting (Sandcastle uses 10).
- **No-op detection** — Scout finds nothing, N cycles running. Stop, or keep looking?
- **Repeat-issue detection** — the same bug filed 5 times means the fix isn't sticking. Detect
  and escalate rather than loop forever.

### 6. Idempotency

If the factory is killed mid-cycle, what state exists? Can it resume, or does it start clean?
Does a half-finished Builder's worktree just sit there? (Fog: worktree cleanup.)

## Deliverable

The trigger enum, the cycle pseudocode, and the failure table filled in — every cell with a
chosen behaviour, not an "it depends".
## Resolution

### v1 is not a manual mode

> *\"For the first version, it's okay to make it run and stop for now, but we should also create a
> room where it can be automated.\"*

That sentence decided the architecture before any ticket existed:

`
   v1 = a special manual mode     ->  built wrong, automation is a rewrite
   v1 = the real loop, once       ->  automation is one change
`

> **The test:** if you catch yourself writing `if manual`, the shape is already wrong. Stop and
> fix it; do not carry it.

### Q1 - Trigger: A - an enum, one variant real

`ust
enum Trigger { Once, Interval(Duration), OnSchedule(Vec<Schedule>) }
`

**The list is the promise; one variant is the whole of v1.**

- A **function** is easiest and promises nothing - nothing stops `if manual` branches appearing
  later.
- A **trait** is a rule that never mentions how many triggers exist, so it cannot break - but it
  makes you write the loop against an abstraction before you know what you want.
- An **enum** commits now to several and ships one. It is the smallest thing that decides the shape.

### Q2 - Failures are classified - B

`
   the cycle cannot BEGIN       ->  STOP, loudly. Name what is missing.
   one ITEM fails                ->  CARRY ON. Record it. Keep looking.
   the loop DISAGREED (rejected) ->  NOT A FAILURE. The interlock working.
`

**Scout failures route to the Doctor** (ticket 24). **The loop does not diagnose its own inability
to run.**

### The same failure three times is a different kind of failure

> Eleven Builders failing for the same reason, reported as *"1 worked, 10 failed"*, is **a total
> failure dressed as a partial success** - and nobody reads that report at 3am.

`
   1 failure, one reason  ->  an issue failed. carry on.
   3 failures, same reason -> one system fault, three times. STOP.
`

**Three is a starting point, not a measurement** - and it should be configurable, with the number
of stops counted. **A threshold that never needs raising is a guess that got lucky.**

### Q3 - One Cycle takes every unblocked Finding

`
  Scout finds everything it can
        |
  Builder x N, in parallel, up to a cap you choose
        |
  Reviewer x N    -> reject: back to Builder, attempt must differ (ticket 11)
        |
  merge to main, per branch -> D refuses: rebase once, then merge agent (ticket 02)
        |
  cycle ends
`

The cap is a **starting number, not a ceiling** - ticket 02 locked that every role recruits for
itself.

### A Cycle that finds nothing

> **Not a failure. Not a success. The loop telling you something true.**

It must say so plainly, because a Cycle that found nothing and a Cycle that never ran are identical
in a log file. **That ambiguity is exactly what this project has been removing.**

### Q4 - The Scout's memory expires when the code changes - C

> **The memory should expire when the reason it existed stops being true.**

The Scout stopped reporting a bug because it was *already reported and not yet fixed*. If someone
changes that code, the question is open again.

`
   reported -> memory: "known, #47, still unfixed"
        |
        +-- someone touches src/auth/session.ts
        |       -> fingerprint changed -> forget it, look again
        |
        +-- nobody touches it
                -> keep suppressing. correct. still unfixed.
`

**Bounded by reality, not by time.** It shrinks when code moves and holds when code does not. A bug
fixed in an untouched file stays suppressed forever - which is fine, because it *is* fixed.

### Why this mattered

> **A memory that only grows eventually stops the Scout finding anything at all - and it looks like
> the software got perfect.**

That is the most dangerous quiet failure in the design: a loop reporting *"0 findings"* because its
memory is full looks identical to one reporting *"0 findings"* because the software is fine.

## Original notes

### Known gap carried forward

The **repeated-failure threshold** is a starting value. Counting the stops is what tells you whether
it is right - not intuition now.

## New questions this raised

None sharp enough to ticket. The usability Probe goal format (ticket 10) remains fog and is out of
v1 scope.