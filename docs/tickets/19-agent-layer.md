# 19 — The agent layer: who owns the session, and who picks the model?

**Type:** grilling
**Status:** closed
**Blocked by**: —
**Claimed by:** opencode

**Findings:** [../docs/features/agent-layer/GRILL-agent-layer.txt](../docs/features/agent-layer/GRILL-agent-layer.txt)

## Resolution

### Both kinds, one interface. CLI first, then API.


### Q1 — both kinds, one interface

Adding the API layer is **alongside** the CLI agents, not instead of them. Behind **one**
interface, so no other ticket has to know which kind it is talking to. The five-event contract
from ticket 07 does most of that work: `text | result | tool_call | session_id | usage`.

### Q2 — canonical shape, decided on evidence

**JSONL, `{role, content:[blocks]}`.** Not invented — it is the shape these tools converged on.

- **Cline** stores `Anthropic.MessageParam[]` verbatim — zero work
- **OpenCode** is SQLite-only, but its schema is pinned in-file and its payload is already in a
  shape we understand. It also has **first-class injection** (`session_inbox`)
- **Kilo Code** ships `/resume-claude` and `/resume-codex` — parsers for two of our known
  formats already exist, in someone else's repo
- **Cursor** — neglected. Protobuf blobs behind two non-interchangeable ids with no bridge
- **Copilot** — no session locking

Research: [../research/20-agent-session-formats.md](../research/20-agent-session-formats.md)

**Injection landmines:** Cline's file watcher clobbers injected writes; Cursor's `--resume`
clobbers the observed transcript.

### Q3 — how an API agent resumes, and who writes the summary

An API agent holds a list of messages **we own**, so it resumes exactly. When the list is too long
to send, it is summarised.

**The summary is what travels.** The full history stays with the agent that made it; the summary
belongs to nobody, because it is just words. **That is what makes cross-agent resume possible**,
and it is lossy at the edges by nature.

**The summary mechanism already exists — the `handoff` skill.**

| | |
|---|---|
| Where | `sessions/.last-handoff` per repo, with a `.handoff-log.md` breadcrumb trail |
| What | self-contained · linked · actionable · scoped — what was done, the work done, and **the prompt the human passed** |
| Read by | `situation-awareness`, and now any agent |

**What the factory adds — event-driven rather than agent-invoked:**

```
agent stops normally    →  the agent writes its own handoff     best quality
agent is killed/crashes →  the factory triggers a cheap model   fallback
```

The second path is the gap: an agent killed mid-turn **cannot write its own summary**, which is
exactly when a summary matters most.

**The handoff already captures the human's steering note** — so steering crosses agents for free.
That is the mechanism behind "even a different agent can resume."

### Now blocks

## Question

The factory needs every agent to be resumable and steerable, and needs to choose the right model
per task. Does that mean launching agent programs, talking to model APIs, or both — and what does
each buy?

## Progress

**Q1 — locked: both, not either/or.**

> *"I was going to say that just because I'm introducing API doesn't mean we should eliminate the
> agent that already have harnesses."*

Adding the API layer is **alongside** the CLI agents, not instead of them. A CLI agent brings
tools and a permission model the factory would otherwise have to reproduce; the API layer brings
ownership of the session. Neither dominates, so both stay — behind **one** interface, so no other
ticket has to know which kind it is talking to.

The five-event contract from ticket 07 already does most of that work:
`text | result | tool_call | session_id | usage`.


- **[11](11-reviewer-rules.md)** — "is the Reviewer a different model from the Builder?" is now
  trivially answerable here, and should be settled as part of this ticket
- [08](08-rust-script-boundary.md), [12](12-loop-controller.md), [13](13-on-disk-layout.md) —
  all need "where does the session live" and "who picks the model" answered first

### Also answers ticket 11

Separation of duties is now **cheap** - a different model is a different request target. So the Reviewer **should** use a different model than the Builder, because same-model-different-context inherits the Builder's blind spots, which is the exact failure ticket 11 exists to prevent.

---

## Background

### Where this came from

Raised during ticket 11, while working out how a human steers a running agent.

> Victor: *"I don't think any agents should not have the capability of starting from where it
> stopped... I know that Claude has this. Is Codex, and so is Pi. We already have a session.
> However, those sessions are mostly... probably some are local and all that. But what I'm
> looking at is a session that we can start from where we stopped."*

And then the turn that opened this ticket:

> Victor: *"even if we are to be working with agents like OpenCode, we should also have a room for
> a human to work with agents as an API. So you can just add Claude, Anthropic's API, and you can
> select any model you want."*

## A correction to the record

Earlier in this session it was stated that `opencode`, `cursor` and `copilot` do not support
resume, and that steering is therefore unavailable for them.

**That was wrong.** Sandcastle sets `captureSessions: false` on those providers — meaning *the
capture code was never written*, not that the session cannot be persisted. The session is a file
or a database either way (OpenCode's is SQLite). **If the factory holds it, it can resume it.**

Steering being unavailable to three agents was never a real constraint. It was an artefact of
inheriting one implementation's unfinished work.

## The two kinds of agent

### A. Launch a program (what Sandcastle has)

```
you  ──spawn──▶  claude / codex / opencode ...
agent ──JSONL──▶  you
agent exits. It owned the history while it ran.
```

- Easy to start with — wrapping a binary is less work than writing an agent loop
- Brings a whole agent CLI into the sandbox image
- Session handling is whatever that CLI happens to do
- **Routing per model** means swapping binaries

### B. Talk to a model API (missing)

```
you  ──▶  POST api.anthropic.com
you  ◀──  text + tool calls
you  run the tools itself
you  append the results
you  ──▶  POST again
```

- **The history is yours** — a list of messages you store, add to, and replay
- **No agent binary in the image at all** — the sandbox shrinks by roughly a gigabyte
- **Every model is resumable by construction**, because you own the record
- **The model is chosen per request**, so a cheap call can go to a cheap model mid-conversation

## What each requirement needs

| Requirement | A: launch a program | B: model API |
|---|---|---|
| Resume | whatever the CLI offers | **trivial — you have the messages** |
| Human steering | kill and resume with a note | **append one message** |
| Attempts must differ | pass the previous reason forward | **automatic — it is already in the history** |
| Router (ticket 20) | swap binaries | **choose per message** |
| Cheap models for cheap probes | separate runs | **mid-conversation** |
| No agent binary in the image | ❌ | ✅ |
| Set-up cost | low | **an agent loop — a few hundred lines** |

> **Sandcastle only built A because wrapping a binary is easier than writing an agent loop.** That
> is a reasonable place to start and a poor place to finish.

## What to decide

### 1. Both, or one?

**My view: both, behind one interface.** A and B are genuinely different — B cannot do
everything A does (a CLI agent may have its own tools, its own sandboxing, its own session
formatting that people rely on). Neither is strictly better.

But they must present **one** interface to the rest of the factory, or every other ticket has to
know which kind it is talking to. The five-event contract from ticket 07 already does most of
this work — `text | result | tool_call | session_id | usage`.

**Is B the default and A the exception?** A sensible split: API agents for probes and cheap
work; CLI agents where a specific one is wanted for quality.

### 2. What does the factory store?

A session is currently the agent's own format. If the factory owns some of them, what is the
**common** representation?

- Store the raw API message list (simple, exact, only for API agents)
- Normalise everything into one shape on the way in and out (more work, uniform)
- Keep each provider's native format and write adapters (what Sandcastle does now)

**My view: normalise.** The alternative means every consumer — the reviewer, the router, the
human reading a session — has to know which kind of agent produced it. That spreads complexity
out instead of containing it.

### 3. Cost, and whether the router is in scope here

With B, routing is *choosing a model per request* rather than a separate system. That dissolves
most of what a separate router ticket would decide.

**Is this ticket the router?** My view: **yes** — the router is a few lines on top of owning the
request. Keep them together and split later only if it turns out to be more.

### 4. What does B *not* do that A does?

Honest gaps to name, not to paper over:

- A CLI agent may bring tools the factory has not implemented
- It may have its own permission model the factory would have to reproduce
- Its session format may already be understood by other tools the user runs
- **The 5-event contract may not be reachable** for some APIs without reimplementing their
  tool-call semantics

### 5. Order of work

Does B come first (build the foundation, then everything else sits on it), or A first (something
working end to end, then improve)? **My view: A first for exactly one cycle**, so there is a
working factory to observe. Then B, because everything it makes possible is better built on it.

## Deliverable

One interface, both implementations behind it, the common session representation, whether the
router folds into this ticket, and the order of work.
