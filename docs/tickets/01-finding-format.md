# 01 — What does a factory finding actually look like?

**Type:** grilling
**Status:** closed
**Blocked by:** —
**Claimed by:** opencode

**Findings:** [../docs/features/finding-format/GRILL-finding-format.txt](../docs/features/finding-format/GRILL-finding-format.txt)

## Resolution

> **A Finding exists when the Scout has written a script that re-enacts a real person's scenario
> against the real application, and that script has failed twice in a row on the code as it
> stands today.**

| The two runs showed | What it is |
|---|---|
| ✗ ✗ | A **Finding** — the Builder gets it |
| ✗ ✓ or ✓ ✗ | A **Finding of unreliability** — "this only breaks sometimes" |
| ✓ ✓ | Not a Finding. Dropped. |

**Locked alongside it:**

- **Real code, simulated world.** A real database, real services, real code paths — fed a
  seeded world. Fully synthetic tests pass in the factory and fail for users, which is worse
  than finding nothing.
- **Seed & Reset.** Baseline is throwaway (delete the data container, start a new one — works on
  any app). Upgrade is a reset script the repo provides. No Seed → Scout degrades to read-only
  and skips state-changing probes. The Scout may write a Reset script **as a proposed change a
  human approves**, never in place.
- **Evidence is generated from the run**, not written by the Scout — a narration plus a
  Demonstration (video for web, terminal recording for a CLI, request/response for an API). It
  cannot drift from what happened.
- **The Repro ends as a proposal**, not an automatic edit — the Builder offers it as a permanent
  test in the same pull request, a human decides.
- **Definition stays one sentence.** Who writes it → ticket 09. Where evidence lives → ticket 13.
  What happens to the branch → ticket 02.

**Graduated from fog:** ticket 15 (executable repros from untrusted output), ticket 16 (boot +
reset as part of the Harness contract).

## Question

What is the exact, runnable artifact that a Scout produces when it finds something — and what
makes the difference between a real Finding and a hallucinated one?

## Why this is the keystone ticket

Scout is the only role in the loop that touches running code. Everything downstream — Writer,
Builder, Reviewer — reasons about text it cannot execute. If Scout is wrong, the entire loop
builds on sand.

Worse: **an LLM asked to "find bugs" will happily invent one.** It will report "login fails on
Safari" having never launched Safari. That flows: issue → Builder "fixes" it → Reviewer approves
→ main has a random change to login for no reason. Repeat every cycle and main rots.

The fix is structural, and it hangs entirely on this ticket's answer:

> **A Finding that cannot be replayed is not a Finding. It is a ghost, and it is dropped.**

Everything else in the loop is downstream of deciding what a replayable artifact *is*.

## What we already know (locked)

- Findings must be **executable**, not prose. A Reviewer must be able to run the thing and see
  it pass or fail — before it reads a single line of the diff.
- A Scout finding is confirmed only if it **reproduces twice**. Once is noise; twice is signal.
- Stress findings include a **capacity ceiling** — the number of concurrent users the machine
  reached before degrading — plus a priced cloud escape hatch.
- Ghost findings cost money every cycle if not caught. Dropping them is the whole point.

## What is still fog

- Exact artifact format: a test file? a shell script? a recorded trace?
- Who owns writing the repro — the Scout agent, or a wrapper that captures whatever it ran?
- What "twice" means: same process twice, or two independent Scout runs?
- How do repros survive the app being *fixed*? A repro that stops failing is how you know the
  fix worked — but it must not look like a broken test after merge.

## Related fog in the map

Prompt injection (a finding built from untrusted app output), regression baselines ("what is
green?"), and whether the repro is the reviewer's only gate or just the first one.