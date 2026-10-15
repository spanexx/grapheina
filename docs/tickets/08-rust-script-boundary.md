# 08 — Where exactly does the Rust/script line fall?

**Type:** grilling
**Status:** closed
**Blocked by:** 02
**Claimed by:** —

## Question

Which responsibilities live in the Rust binary, which live in leaf scripts, and what is the
interface between them?

## Blocked by 02 because

The branch strategy decides the exact set of git operations the system must perform. Until we
know whether a Builder merges, parks, or opens a PR, we cannot say which of those belong in
Rust and which are a five-line script.

## The shape already agreed (informal)

```
┌──────────────────────────────────────┐
│  factory (Rust, one binary)          │
│  ── lifecycle, concurrency, abort    │
│  ── timeouts, streaming, structured  │
│    errors, the loop controller       │
└───────────────┬──────────────────────┘
                │ spawns
  ┌─────────────┼─────────────┬──────────────┐
  ▼             ▼             ▼              ▼
worktree.sh  docker.sh     agent.sh      vercel.sh
```

Rust owns **time and process tree**. Scripts own **git, docker, and the cloud APIs**.

## What to decide

1. **Which is a trait, which is a script?** Sandcastle has two seams — SandboxProvider and
   AgentProvider. Should both be Rust traits, or should agent invocation stay a script so a new
   agent is one shell file rather than a recompile?
2. **The script interface.** argv? stdin JSON? env vars? Exit codes as a result type? How does
   a script report a *stream* of events back to Rust — stdout lines, or a side channel?
3. **Where does the "5-event contract" live?** `text | result | tool_call | session_id | usage`.
   In Rust (typed enum) or in each script (raw JSON lines Rust parses)?
4. **Is Rust even worth it?** Honest question. It buys: timeouts with process-group kill,
   Ctrl-C that stops N agents cleanly, a concurrency semaphore, `Drop`-based cleanup ordering,
   one binary to ship. Bash buys: everything readable in 5 seconds. Where is the line where
   bash genuinely breaks?
5. **Language for scripts.** Bash? Python (stdlib, like `agent-worktree`)? PowerShell, since
   this is Windows? A Rust tool that shells out to a .bat is a smell.
6. **Does the binary embed the scripts?** One binary that writes its own scripts to a temp dir,
   or scripts shipped alongside in `~/.local/bin`?

## Recommendation to argue against

Rust binary owns the **loop, lifecycle, and streaming**. Scripts stay **git and Docker only** —
roughly ten files of five lines each. Agent invocation sits on the fence: a Rust trait if we
want type safety, a script if we want one-file-per-agent.
## Resolution

### Everything in Rust. There is no line.

Answer: **A.** Zero script dependency, one binary, one code path.

### How this decision changed

It started the session as Victor's own idea - *"use Rust to connect all the scripts"* - on the
reason that scripts are readable and easy to drop in.

**Ticket 14 overturned the premise.** If the factory must run everywhere, and bash is not on a
default Windows install, then a binary depending on shell scripts **inherits the dependency it was
supposed to remove.**

And the performance argument for scripts never existed: Rust calling git and a shell calling
git are the same system call running the same program.

### What it gains

> **One code path, identical on every machine.**

With scripts allowed, a machine without bash behaves differently, and *"why did it work on
mine?"* becomes a question that can never be answered once and for all. **That is a whole class of
bug that only exists on some machines.** This decision makes it impossible.

### What it costs - named, not buried

- **Adding a new sandbox provider or agent means recompiling.** No more dropping a file in.
- Iteration is slower than editing a five-line script.

That was the part of the original idea that was genuinely good, and it is genuinely gone.

### The way back, if it is wanted later

A **plugin** - a separate small program talking to the factory over a pipe. Same extensibility,
**no bash**. It does not need to exist in v1; the seam (the SandboxProvider and AgentProvider
interfaces) should still be a real interface so that adding one is a crate, not a refactor.

## Original notes

### Recommendation to argue against

Rust binary owns the **loop, lifecycle, and streaming**. Scripts stay **git and Docker only** -
roughly ten files of five lines each. Agent invocation sits on the fence.