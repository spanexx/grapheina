# 03 — Can a Vercel Sandbox host a long-running agent?

**Type:** research
**Status:** closed
**Blocked by:** —
**Claimed by:** research subagent

**Findings:** [../research/03-vercel-sandbox-api.md](../research/03-vercel-sandbox-api.md)

## Question

Can a Vercel Sandbox run a full coding agent for 10–60 minutes — installing dependencies,
running tests, making commits — and is it reachable and scriptable without the Node SDK?

## Why this matters

Vercel is one of two sandbox providers in scope (the other is Docker). It matters because it
solves a problem Docker cannot: **load testing needs somewhere to aim the traffic.**

`src/sandboxes/vercel.ts` wraps `@vercel/sandbox`, an SDK we do not want (it is Node, and we are
Rust). Everything must be reachable over plain HTTPS with a bearer token, or Vercel is not
usable from our binary.

## Specifics to establish

1. **The REST surface.** Endpoints for create / exec / read-output / kill / destroy. Auth header
   shape. Token from `VERCEL_TOKEN` or `VERCEL_OIDC_TOKEN`.
2. **Does `exec` stream?** Our whole product is line-by-line stdout with idle timeouts. A
   request/response `exec` that buffers until completion is **unusable** — we would lose the
   idle timeout, the completion signal, and live display.
3. **Long-running processes.** Agents run for tens of minutes. Is there a per-command timeout?
   Is there a sandbox-level max lifetime? What happens on timeout — is the sandbox killed with
   the work half-done?
4. **`source: { type: "git", url, revision }`.** Sandcastle passes this. Does the sandbox clone
   the repo itself? Can it take a **branch** or a **PR head SHA**, or only a branch name? This
   determines whether a Builder can be pointed at its own branch in one call.
5. **Pushing commits out.** If the agent commits inside the sandbox, how do those commits reach
   the host repo? Sandcastle's `syncOut` uses `format-patch`/`am`, which rewrites SHAs. Is there
   a cleaner path — a token the sandbox can push with, or do we always patch out?
6. **Runtimes.** Can it run `node22`, `python3.13`? What about images with system packages
   (`gh`, `git`, browsers)? Is there a custom-image path?
7. **Cost and quotas.** Free tier. Rate limits. What a 30-minute agent run costs.
8. **Concurrency.** How many sandboxes at once? Sandcastle's planner template wants 4 parallel.

## What is already known

- Sandcastle's provider is `tag: "isolated"` — own filesystem, sync in and out required.
- Worktree path inside the sandbox is `/vercel/sandbox/workspace` (`VERCEL_REPO_PATH`).
- Sandcastle's `syncIn` / `syncOut` exist because there is no shared filesystem. If we go Vercel,
  we inherit that whole sync problem — this ticket should say whether it is worth it.

## Deliverable

A written answer with **curl-able request/response examples** for each operation, plus an
explicit **yes/no on streaming exec** — that one answer decides whether Vercel is viable at all.