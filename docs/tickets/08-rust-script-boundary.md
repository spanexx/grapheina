# 08 — Where exactly does the Rust/script line fall?

**Type:** grilling
**Status:** open
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