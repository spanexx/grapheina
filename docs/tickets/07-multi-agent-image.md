# 07 — Can one sandbox image carry Claude, OpenCode and Codex together?

**Type:** research
**Status:** closed
**Blocked by:** —
**Claimed by:** research subagent

**Findings:** [../research/07-multi-agent-image.md](../research/07-multi-agent-image.md)

## Question

Should the sandbox image ship every agent CLI we support, and if not, how does the Builder get
the agent it was assigned?

## Why this matters

Agent-agnostic was a goal from the first conversation. Right now the factory is pointed at
Claude — but the whole point is that the Builder is swappable.

The naive answer is "install all of them in one Dockerfile." That has costs:

| Concern | Cost of shipping all |
|---|---|
| Image size | Claude + Codex + OpenCode + Pi + Cursor + Copilot |
| Cold start | every run pays the whole image |
| Auth | each needs a different token in `.sandcastle/.env` |
| Supply chain | six third-party installers in one image |
| Attack surface | six CLIs, each with file/network access |

## The three options

**A. One fat image.** All agents baked in. Swap costs nothing. Image is huge and slow.

**B. One image per agent.** `sandcastle/claude:latest`, `sandcastle/opencode:latest`. Pick at
run time. N images to build and maintain, but each is small and fast.

**C. One thin image + runtime install.** Base image has git, node, gh. The agent is installed by
a setup hook when the sandbox boots. Slow every time, and network-dependent at boot.

## What to establish

1. **Install method + size for each.** Claude Code (curl script), Codex (npm), OpenCode (npm
   binary), Pi (npm), Cursor, Copilot. Size and install time each.
2. **Credentials.** Which need an OAuth token vs an API key? Which can be passed as an env var
   at run time vs which need a login step?
3. **Headless / non-interactive auth.** Can each be authenticated purely from env vars, or does
   any need an interactive login that would hang in a container?
4. **What "print mode" does each support?** Our whole product assumes: pipe prompt in on stdin,
   get JSON events out on stdout. Which agents support a true non-interactive print/JSON mode,
   and which force argv (Sandcastle had to cap Cursor and Copilot at 120 KB because of the Linux
   `ARG_MAX` limit)?
5. **The 5-event contract.** Sandcastle normalises every agent's stream into
   `text | result | tool_call | session_id | usage` (`AgentProvider.ts:34`). Confirm each agent
   can emit enough to satisfy it. Which cannot — and what do we do for those?
6. **Recommended default.** For v1, how many agents must work? The map's destination says
   "agent-agnostic" but v1 could legitimately ship one and prove the seam.

## Deliverable

A recommendation (A / B / C) with per-agent install size, auth method, print-mode support, and
stream-parser feasibility. Plus a **named v1 agent set** and the list of agents that are
out of scope until later.

## Reference — what Sandcastle supports today

Six providers already parsed and tested: `claudeCode()`, `codex()`, `pi()`, `cursor()`,
`opencode()`, `copilot()` (`src/AgentProvider.ts`). Session capture/resume/fork works for
claude-code, codex, and pi only. Copy the parsers rather than rediscovering the stream shapes.