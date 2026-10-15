# 15 — The Repro is executable code built from untrusted output

**Type:** grilling
**Status:** closed
**Blocked by**: 01 ✅
**Claimed by:** opencode

**Findings:** [../docs/features/untrusted-repro/GRILL-untrusted-repro.txt](../docs/features/untrusted-repro/GRILL-untrusted-repro.txt)

## Resolution

### The threat model, corrected

The Scout already has a shell in the sandbox and already ran commands to find the bug. **Running
a Repro adds no new capability.** What is new is its *lifetime* — it outlives the cycle.

The frequent case is **careless, not evil**: a repro that deletes data to demonstrate a data-loss
bug, one that needs your keys, one that inlines a secret into a file that then gets committed.

### Approved tests run in CI

Accepting the privilege escalation explicitly: a test proposed by the Builder runs in CI
automatically. A committed-but-never-run test is documentation, not a guard. **Approving does
not damage the repo — the file is visible and revertable — it moves a script somewhere with more
power,** since CI holds real deploy keys the cycle never had.

### Gates, not warnings

| Layer | What it catches |
|---|---|
| **Sanitiser** (mechanical, blocking) | Inlined credentials · paths off this machine · unknown network destinations · tools not on the allowlist |
| **PR agent** (required stop) | Everything else — **may block only by naming a line and a reason** |
| **Human** | Approves |

### Credentials are forbidden, not scanned for

> A Repro that needs to sign in **reads it from the environment**. There is then nothing in the
> file to leak.

Prevention beats detection. A pattern match can be evaded and manufactures false confidence; a
rule the file cannot violate cannot be evaded. **"This looks suspicious" must not block** — a gate
that blocks on vague suspicion gets disabled, and a gate that gets clicked past is worse than no
gate.

**Carry into ticket 11:** a Repro refused by the Sanitiser is itself a Finding about the Scout's
behaviour, and a test the reviewing agent blocks three times running is a signal about the Scout,
not the test.

## Question

The Scout writes a script. The script is built from what the application said and did while
running. **How does the factory stop that script from doing something other than reproducing a
bug?**

## Why this is now a real ticket

Ticket 01 locked the Repro as *executable code written by an LLM*. Before that, a Finding was
prose. Prose can lie. **A repro can execute.**

This was fog on the original map ("prompt injection"). It became sharp the moment we chose
"script, not recording" — and it got sharper again when the Builder proposes that script into
the project's permanent test suite, where it will run forever in CI.

## The exposure

```
     untrusted                        LLM writes                    the repro
  ─────────────                     ────────────                  ─────────
  app under test                 reads the output,               gets executed
  database contents              decides "this is a bug"          in a sandbox
  third-party API replies
  error messages                 ...then emits code...
  file names, form values
```

Anything the app printed can end up inside the script. A hostile or compromised dependency
does not need to break your app — it just needs to **say** the right thing.

## What to decide

### 1. What does the repro get to do?

| Allowed | Denied |
|---|---|
| Send HTTP requests to the app under test | Reach the public internet |
| Read/write files inside the worktree | Read anything outside it |
| Use the app's own API | Read `.env`, `~/.ssh`, cloud credentials |
| | Shell out to arbitrary binaries |

**Is that the right list?** What else belongs on the deny side?

### 2. The sandbox is not automatically a boundary

Per the map's portability rules, the **default provider is `noSandbox`** — the repro runs on the
host with the developer's own credentials.

> **If we ship `noSandbox` as the default, a repro from untrusted output runs with the developer's
> full access.** That is a live risk, not a theoretical one.

Options:
- Repro execution always requires a container, even if the rest of the cycle does not
- Repro runs with a scrubbed environment (no secrets in the environment at all)
- Repro runs read-only, write access opt-in per Finding
- Accept it, and make the Scout's output trust the same way we trust a PR author

### 3. Static checks before execution

Should a repro be inspected before it runs? Cheap catches:
- Only the languages the Scout is allowed to emit
- No network calls to anything but the app under test
- No reads outside the worktree
- Size limit, execution time limit

**Or is review-by-the-Scout itself the check?** (A model reviewing code from another model is
weak, but not nothing.)

### 4. The permission-escalation path

When the Builder proposes a Repro as a permanent test, that test runs in **the project's own
CI**, forever, with whatever access CI has. Is a Factory-authored test the same as a
human-authored one?

### 5. What the Scout is allowed to read in the first place

The narrowest fix is upstream: if the Scout never reads untrusted text into its context, it
cannot be injected. Does the Scout see raw application output at all, or a structured,
sanitised view of it?

## Deliverable

The allow/deny list, the answer to whether repro execution requires a container, and the static
checks (if any) that run before a repro is trusted.