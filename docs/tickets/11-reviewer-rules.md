# 11 — How does the Reviewer judge, and what makes it say no?

**Type:** grilling
**Status:** closed
**Blocked by:** 01 (closed)
**Claimed by:** opencode

## Question

What are the Reviewer's gates, in what order, and what is its **default verdict** when the
evidence is thin?

## Blocked by 01 because

The Reviewer's primary gate is replaying the Finding's repro. We cannot design its gates until
ticket 01 fixes what a repro *is*.

## Why this ticket decides whether the whole thing survives

> **The loop has no brake.** A Builder ships a subtly wrong fix → Reviewer approves → merged →
> Scout uses the app → finds the breakage → files it → Builder "fixes" it → Reviewer approves
> again. One pass at a time, main degrades, nobody notices for months.

LLMs are agreeable by default. Asked "does this look right?", a model says yes. An autonomous
loop with an agreeable reviewer is a machine for making confident, unreviewed changes to main.

**The Reviewer is the interlock. Its default verdict is the most important single decision in
this project.**

## The shape we sketched

```
④ REVIEWER
   1. REPLAY THE REPRO        ← objective, binary, cannot be argued with
   2. run the test suite      ← objective
   3. READ THE DIFF           ← subjective, last
   4. VERDICT
```

Order matters. **Objective gates first.** If the repro fails, the reviewer has nothing to
discuss — the fix did not work, full stop. Only after both objective gates pass does it read
code, and by then it is confirming a fix rather than deciding whether one exists.

## What to decide

### 1. The gate order, precisely

Is it repro → tests → diff? Or diff → tests → repro? (We argue repro-first. Counter-argument:
a diff that obviously deletes the test suite should be caught before anything runs.)

### 2. Default verdict

| When… | Verdict? |
|---|---|
| Repro passes, tests pass, diff looks fine | approve |
| Repro passes, tests pass, diff looks wrong | ??? |
| Repro **fails** | reject — no discussion |
| Tests fail | reject — or "preexisting failure"? |
| Cannot tell | **??? ← this row is the ticket** |

**Argument for reject-by-default:** a reviewer that must argue *for* approval is far safer than
one that must argue *against* it. Cost: blocked issues. Benefit: main doesn't rot.

**Argument for approve-on-uncertainty:** a reject-default factory stalls, and a stalled factory
is an abandoned factory. Which failure mode do we prefer?

### 3. Separation of duties

Should the Reviewer be a **different model** from the Builder? Same model, fresh context?
Different agent entirely? Same-model-different-prompt has the correlated-blindspot problem —
it cannot catch its own mistake.

### 4. What is explicitly out of the Reviewer's reach

Does it review security? Style? Architecture? Or only "does the reported bug now work and do
tests still pass"? (Scope creep here is how a factory becomes unreviewable.)

### 5. Escalation

When does a human get pulled in? Third rejection? Any reviewer rejecting on a "big" issue?
Touching auth, migrations, or dependencies? How does the factory *know* to ask?

### 6. The rejection loop

Reject → back to Builder. How many attempts? Does the Builder see the previous attempt and the
reason, or start blind? What if the third Builder tries the same thing?

## Deliverable

The ordered gate list, the default-verdict rule stated as a single sentence, the attempt limit,
and the escalation trigger.
## Resolution

### Two gates, two defaults

| Question | The machine can decide it? | When unsure |
|---|---|---|
| Is the **code** right? | Yes - checkable | **Reject** |
| Is the **behaviour** right? | No - a value judgement | **Escalate** |

**Rejection and escalation are different.** Rejection sends work back to the Builder. Escalation
asks a person, who can say "no, that's right" - which unblocks rather than loops.

The argument for reject-by-default: **a blocked Finding is one you notice. A bad merge is one
nobody notices.** One is a delay; the other is permanent.

### The loop terminates

- **D - limit, escalate, and every attempt must differ.** Limit alone burns three rounds doing
  the same thing; escalate alone produces an infinite loop with paperwork.
- **A rejected issue goes to the back of the queue.** It stops being urgent the moment it failed
  once.
- **Attempts must differ, and that is now automatic.** A rejected attempt's session carries what
  was tried and why it failed. Attempt two resumes the same session, so it *cannot* repeat itself.

### Separation of duties - settled by ticket 19

Choosing a different model is choosing a different request target, which is free once the factory
owns the request. **So the Reviewer uses a different model than the Builder.** Same-model-different-
context inherits the Builder's blind spots - the exact failure this ticket exists to prevent.

### Human supervision, always on

Not escalation - supervision. **Every running agent is visible and steerable at any time.**

| | |
|---|---|
| **Visible** | every agent streams what it is doing, and **states its intent before acting** |
| **Redirect** | the human's note goes into the session; the agent picks it up still in that session |
| **Stop** | kills it - the session survives |

Stated intent is the cheap moment: reading a plan costs a second, reading a diff means the wrong
path already happened and has to be undone - a whole cycle.

**Cheaper than escalating.** A watching human sees a bad thing at cycle one; a reading human sees
it at cycle three, after two wasted rounds.

Escalation then arrives with the session intact - which *is* the record. A human picking up an
issue after three attempts knows what was tried, what the Reviewer said, and what was already
steered toward, **without reading a single log file**. The path should end in "take it if you want
it", not "here it is, I'm done."

### Gates already locked elsewhere

- **Repro first**, before the diff (ticket 01) - objective, cannot be argued with
- **Sanitiser and PR agent are hard gates**, blocking only by naming a line and a reason
  (ticket 15)
- **The rules reviewer** checks process compliance - packs exist, docs updated - separate from
  code quality (ticket 02)