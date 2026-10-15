# 16 — What exactly must a Harness be able to do?

**Type:** grilling
**Status:** open
**Blocked by**: 09
**Claimed by:** —

## Question

Ticket 01 locked that every Probe needs an app booted and reset to a known Seed. Ticket 09
covers *detecting* what kind of app this is. Neither covers the contract itself: **what must a
Harness actually be able to do before any Probe can run?**

## Why this is separate from detection

Detection answers "what is this?" The contract answers "what must be true for anything to run
against it?" Those are different questions, and the second one is what actually breaks in
practice.

## What we already know

From ticket 01:
- The app must **boot** to a known state and be **resettable** to it
- Baseline reset is throwaway (delete the data container); upgrade is a repo-provided script
- No reset → the Scout degrades to read-only and skips state-changing Probes

From ticket 09 (still open):
- Detection reads signals, decides the app kind, and caches the answer in `.factory/harness.yaml`

From ticket 04:
- Browser Probes need a Linux container with Playwright baked in
- `--init` and `--ipc=host` are mandatory flags

## What to decide

### 1. The Harness contract

What must a Harness provide? A working list so far:

```
boot        start the app, wait until it answers
reset       put it back to the Seed          ← ticket 01
target      where Probes connect (URL, socket, command)
drive       how a Probe acts (HTTP, browser, terminal)
observe     how results come back (stdout, screenshot, response)
```

**What is missing from that list?** Candidates: timeouts, logging, health checking, teardown,
multi-service dependencies.

### 2. How long does boot get, and what if it never comes up?

An app that takes 40 seconds to start. An app that hangs on a missing service. An app waiting
on input. What is the boot timeout, and what does a failed boot produce — an error, or a
Finding ("this app cannot start cleanly")?

> **That second one is interesting.** A factory that cannot boot an app should be able to say
> so as a Finding rather than silently skipping the repo.

### 3. Dependencies beyond the app

Real apps need a database, sometimes a cache, sometimes a queue. Who starts them?

- The Harness starts them (a compose file the repo provides)
- The Harness discovers them and fails loudly
- v1 supports exactly one service and nothing more

### 4. Who writes the Harness when detection is unsure?

Ticket 09 resolves "detection is unsure" by asking a human once. But the Harness is more than
detection — it is the working arrangement. **Should the Scout be able to write a Harness
proposal for a repo that has none**, the same way it may write a Reset script as a proposal?

If yes, that is the factory teaching itself how to work with a codebase. If no, every new repo
costs a human a session before anything happens.

### 5. Does a Harness survive app changes?

An app gets rebuilt. Routes move. The port changes. When does `harness.yaml` become stale —
and does anything detect that, or does the Scout fail confusingly until a human intervenes?

## Deliverable

The Harness contract as a numbered list of capabilities, the boot timeout and failure
behaviour, the dependency policy, and an answer on whether the Scout may write one.