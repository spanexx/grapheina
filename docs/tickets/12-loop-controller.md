# 12 — What triggers a cycle, and what happens when one fails?

**Type:** grilling
**Status:** open
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