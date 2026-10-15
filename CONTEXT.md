# Software Factory — Context

The ubiquitous language for the software factory. A glossary, not a spec.

Terms crystallise during grilling; this file is updated inline as they do.

---

## The loop

**Cycle**:
One full pass through the loop — Scout → Writer → Builder → Reviewer → Merger.
_Avoid_: run, iteration, round, job.

**Trigger**:
What starts a Cycle. In v1, a human starting it manually.
_Avoid_: schedule, cron, dispatcher.

**Factory**:
The whole system. Runs Cycles until stopped.
_Avoid_: the tool, the CLI, the orchestrator, the pipeline.

---

## Roles

**Scout**:
The role that uses the application and looks for problems. The only role that runs real code.
_Avoid_: tester, QA, hunter, explorer, agent.

**Writer**:
The role that turns a Finding into an Issue.
_Avoid_: reporter, triage, scribe.

**Builder**:
The role that works an Issue and produces commits.
_Avoid_: implementer, coder, fixer, agent.

**Reviewer**:
The role that verifies a Builder's work before it is allowed near main. A different role from
Builder, by design. _Avoid_: approver, gatekeeper, critic.

**Merger**:
The role that lands approved work onto main.
_Avoid_: integrator, lander.

---

## The Scout's team

**Probe**:
One specialised kind of looking — crash-hunting, stress, security, usability, regression.
The Scout runs several at once; each searches differently.
_Avoid_: test type, check, scanner.

**Probe team**:
The set of Probes valid for a given kind of application.
_Avoid_: matrix, suite.

**Harness**:
What lets a Probe run — what kind of app this is, how to start it, how to reach it, how to poke
it. The Scout produces one; a human rarely writes one.
_Avoid_: manifest, adapter, driver, config.

**Detector**:
A small program that reads one language ecosystem and answers the Harness questions for it. One
per ecosystem, loaded as plugins — **not** one big program that knows everything. A project the
factory has no Detector for is a true negative, not a guess.
_Avoid_: detection script, language support, parser.

**Blind spot**:
An ecosystem the factory has no Detector for. The Scout names it rather than guessing:
*"No detector for Ruby. I know: JS/TS, Python, Rust, Go."* A Blind spot is honest output, not a
failure.
_Avoid_: unsupported, unknown, missing support.

**Capacity ceiling**:
The measured number of concurrent users this machine reached before the application degraded.
Always measured, never estimated. _Avoid_: max users, limit, headroom.

---

## Findings

**Scenario**:
A situation a real person would genuinely hit — written down in `scenario.yaml` with its
starting state, its steps, and what was expected to happen. The Scenario is the **contract**;
the Repro is one implementation of it and may be rewritten freely.
_Avoid_: test case, flow, journey (implying it was designed rather than observed).

**Simulation**:
The world a Scenario runs in. **The code is real; the surroundings are not.** A real database,
real services, real code paths — fed simulated-but-realistic data, seeded state, and simulated
third-party responses.
_Avoid_: mock, stub, fake, sandbox (see **Sandbox** — different thing entirely).

**Seed**:
The known starting state a Scenario requires. Every Simulation must be reset back to it before
a run. Two ways in: **throwaway** (delete the container holding the data, start a new one — the
baseline, because it works on any app) or **scripted** (the repo provides a reset script — faster,
but only apps that have one). Absent a Seed, the Scout degrades to looking without touching, and
skips any Probe that changes data.
_Avoid_: fixture, setup — those describe the *contents*; a Seed is the reset point.

**Reset**:
Getting the world back to the Seed. The Scout may write a Reset script for a repo that lacks one,
but **only as a proposed change a human approves** — never in place.
_Avoid_: teardown, cleanup (both imply removing rather than restoring).

**Finding**:
Something the Scout discovered, backed by a **Repro** that fails against the current code.
The unit that becomes an Issue.
_Avoid_: bug (implyes it is real before verification), report, observation, issue.

**Repro**:
A script the Scout wrote that re-enacts the **Scenario**. It must fail against the code as it
stands today. This is the entire trust mechanism.
_Avoid_: test, spec, proof, regression test (a regression test is what the Repro becomes later).

**Evidence**:
What gets saved alongside a Finding — the failing output, and a **Demonstration** of the
Scenario being re-enacted. A boolean `failed` proves something broke; Evidence shows where and
what a person was doing.
_Avoid_: logs, artefacts, attachments.

**Demonstration**:
The Scenario re-enacted and recorded — for web, a video of the real run; for a CLI, a replayable
terminal recording; for an API, the request and response side by side.
_Avoid_: screenshot (implies a picture of a broken state rather than a re-enactment), video only.

**Ghost**:
A candidate Finding whose Repro passes against current code — it never reproduced. Dropped.
This is the normal fate of most Scout output, not a failure.
_Avoid_: false positive (it may not be false — it may just be unreproducible), bad finding.

**Issue**:
A Finding, written up so a Builder can act on it.
_Avoid_: ticket, task, bug, work item.

---

## Trust

**Gate**:
A check a proposed test must pass before it can be offered for approval. Fails loudly and stops
the process. Two kinds: **mechanical** (a pattern match, a path check — always certain) and
**judgement** (an agent's opinion — sometimes wrong). Only Gates that name a line and a reason
may block; "this looks odd" must not.
_Avoid_: check, validation, linter (a linter cannot block a human).

**Sanitiser**:
The mechanical Gate that runs on a Repro before it is ever shown for approval. Refuses and says
which line matched. Never warns-and-proceeds — a warning beside a secret gets clicked past.
_Avoid_: scanner, filter, secret scanner (it is more than that: it also enforces the no-inline-
credentials rule).

**Inlined credential**:
A key, token or password written directly into a Repro. **Forbidden outright**, not scanned for.
A test that needs to sign in reads it from the environment instead — so there is nothing in the
file to leak.
_Avoid_: hardcoded secret (that is the symptom; this is the thing we refuse to accept).

---

## Open terms

These are still fog — do not use them as if defined:

- **Known-failures** — the Scout's memory of what it has already reported
- **Ceiling report** — the honest capacity statement shown to a human
- **Interlock** — the Reviewer's objective gate that cannot be argued with
- **Degradation** — what the factory does when a host cannot do what a Cycle needs
