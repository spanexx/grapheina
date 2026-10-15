# 07 — Multi-Agent Sandbox Image

**Ticket:** `tickets/07-multi-agent-image.md` · **Type:** research · **Status:** answered
**Evidence base:** `sandcastle/src/AgentProvider.ts` (stream parsers, print commands),
`sandcastle/src/InitService.ts:207-407` (per-agent Dockerfile templates), plus vendor docs.

---

## EXECUTIVE VERDICT

**Strategy: B — one image per agent.** This is not a new decision; it is what Sandcastle already
does. `InitService.ts` holds six separate `*_DOCKERFILE` templates (claude/pi/codex/cursor/
opencode/copilot), each `FROM node:22-bookworm` with exactly one agent install line. Option B is
the shipped, tested design. Re-deriving A or C would be re-litigating a solved problem.

**v1 agent set: `claude-code` + `codex`.** Two agents, opposite vendors (Anthropic + OpenAI),
which is enough to prove the seam is real without proving the *registry*. Everything else is a
registry entry, not architecture.

**Biggest risk: credential sprawl, not image size.** Six agents means six differently-named
secrets in `.sandcastle/.env` (`ANTHROPIC_API_KEY`, `OPENAI_KEY`, `PI_KEY`, `CURSOR_API_KEY`,
`OPENCODE_API_KEY`, `GITHUB_TOKEN`) and a per-agent "did auth hang?" failure mode. Two of the six
(Cursor, Copilot) have an interactive-login path that will **hang a container forever** if the
env var is missing or misnamed. A fat image would ship that hang to every run; a per-agent image
contains the blast radius to the one agent whose key is absent.

**Why B beats A and C specifically:**
- **A (fat image):** every cold start pays six Node/npm installs. B's per-agent images are all
  the same base — build time is dominated by the base layer, so A saves *nothing* on pull and
  loses on pull size. A's only real gain is "swap needs no build", which is already handled:
  the image is chosen at boot, not baked into the job.
- **C (runtime install):** adds a network dependency and 10-60s of install to *every* sandbox
  boot, and turns any npm registry blip into a failed build. For a factory that runs Builders
  many times a day, per-boot cost is the wrong place to economise.

---

## Comparison Table

| Agent | Install | Size (approx) | Auth (env, no login?) | Print mode | Prompt via | Stream | 5-event contract |
|---|---|---|---|---|---|---|---|
| **Claude Code** | `curl claude.ai/install.sh \| bash` → `/home/agent/.local/bin` | ~200-300 MB (bundled runtime, UNKNOWN exact) | `ANTHROPIC_API_KEY` — **yes**, fully env | `claude --print --output-format stream-json` | **stdin** (`-p -`) | JSONL | ✅ full incl. usage |
| **Codex** | `npm i -g @openai/codex` (ships native rust binary) | ~150-250 MB (UNKNOWN exact) | `OPENAI_KEY` — **yes**, fully env | `codex exec --json` | **stdin** | JSONL | ✅ full incl. usage |
| **Pi** | `npm i -g @mariozechner/pi-coding-agent` | ~100-150 MB (UNKNOWN exact) | `PI_KEY` — **yes** per sandcastle | `pi -p --mode json` | **stdin** | JSONL | ⚠️ partial — no `usage` |
| **OpenCode** | `npm i -g opencode-ai@latest` (postinstall pulls platform binary) | ~100-200 MB (UNKNOWN exact) | `OPENCODE_API_KEY` + provider keys | `opencode run --format json` | **argv** | JSONL | ⚠️ partial — no `usage` |
| **Cursor** | `curl https://cursor.com/install \| bash` | ~150-250 MB (UNKNOWN exact) | `CURSOR_API_KEY` — env works **but see risk** | `agent --print --output-format stream-json --force` | **argv only** (120 KB cap) | JSONL | ⚠️ partial — no `usage` |
| **Copilot** | `npm i -g @github/copilot` | ~80-150 MB (UNKNOWN exact) | `GITHUB_TOKEN`/`GH_TOKEN` — **yes**, documented headless | `copilot -p --output-format json` | **argv** (120 KB cap) | JSONL | ✅ session_id + result |

Sizes are **estimates, not measured**. Every one of them is UNKNOWN to the byte — none of the
vendors publish an installed-size figure, and this research did not build the images. Treat the
*ordering* (Claude/Codex largest, Copilot smallest) as reliable and the absolute numbers as
noise. **Do not put these numbers in a size budget until someone actually runs `docker images`.**

---

## 1. Install method + installed size

- **Claude Code** — official curl installer (`InitService.ts:229`), lands in `/home/agent/.local/bin`,
  needs a `PATH` ENV line. Native single binary. ~200-300 MB estimated.
- **Codex** — `npm install -g @openai/codex` as root (`InitService.ts:296`). Ships a per-platform
  native binary. ~150-250 MB estimated.
- **Pi** — `npm install -g @mariozechner/pi-coding-agent` as root (`InitService.ts:263`).
  Pinned in the parser comments to 0.73.1. ~100-150 MB estimated.
- **Cursor** — `curl https://cursor.com/install -fsS | bash` (`InitService.ts:330`), needs
  `PATH` ENV. ~150-250 MB estimated.
- **OpenCode** — `npm install -g opencode-ai@latest` as root (`InitService.ts:364`). The npm
  package's postinstall fetches a platform binary, so the build needs network twice. ~100-200 MB.
- **Copilot** — `npm install -g @github/copilot` as root (`InitService.ts:397`). Smallest of the
  six. ~80-150 MB estimated.

## 2. Auth — can each be fully authenticated from env?

- **Claude Code** — `ANTHROPIC_API_KEY` (`EnvResolver.test.ts:124`). Fully env-based, no login.
  An OAuth token is also possible via `claude setup-token`, but the API key is the container path.
- **Codex** — `OPENAI_KEY` (`InitService.ts:440`). Fully env-based. Codex *also* offers ChatGPT
  OAuth (`codex login`) — **do not use it in a container**; it is a browser flow.
- **Pi** — `PI_KEY` (`sandboxes/daytona.ts:29`). Env-based, no login.
- **OpenCode** — `OPENCODE_API_KEY` plus per-provider keys (`InitService.ts:461`). Env-based.
- **Cursor** — `CURSOR_API_KEY` (`InitService.ts:451`). Env-based **when present**. Cursor's
  historical default is an interactive browser login; if the env var is missing or the CLI
  version predates API-key support, it blocks on a browser prompt. **Risky.**
- **Copilot** — `GITHUB_TOKEN` / `GH_TOKEN` (`InitService.ts:472-473`), documented headless path.
  Without a token, `/login` is a browser device-flow → hangs.

**Answer: five of six are safe, two of those five are only safe if the env var name is exactly
right.** Every one of them needs no *interactive* step *provided* its key is present. The failure
mode when it is absent is identical for all six: the process waits on stdin forever. That is a
missing-key problem, not an agent problem, and it argues for failing fast on missing secrets
before spawning — not for a different image strategy.

## 3. Non-interactive print mode

| Agent | Command (from `AgentProvider.ts`) |
|---|---|
| Claude Code | `claude --print --verbose --output-format stream-json --model M [-p -]` (`:1213`) |
| Codex | `codex exec --json --dangerously-bypass-approvals-and-sandbox -m M` (`:811`) |
| Pi | `pi -p --mode json --model M` (`:651`) |
| Cursor | `agent --print --output-format stream-json --model M --force <prompt>` (`:856`) |
| OpenCode | `opencode run --format json --model M <prompt>` (`:981`) |
| Copilot | `copilot -p <prompt> --output-format json --model M` (`:1131`) |

**Prompt delivery splits 4/2.** Claude, Codex, and Pi read the prompt on **stdin**. OpenCode and
Cursor take it as **argv**. Copilot *can* read stdin but Sandcastle uses `-p` argv for parity with
the tested path.

**The ARG_MAX problem is real and specific.** Cursor and Copilot are capped at 120 KB
(`CURSOR_PRINT_PROMPT_MAX_BYTES`, `:128`; `COPILOT_PRINT_PROMPT_MAX_BYTES`, `:1011`) — under
Linux's ~128 KiB per-argument stack limit, so users get a clear error instead of a spawn `E2BIG`.
This is a genuine ergonomic cost of those two agents and a strong argument to defer them.

## 4. Stream format on stdout

All six emit **JSONL on stdout**, one JSON object per line. Parsers skip any line not starting with
`{` (`parseStreamJsonLine`, `:68`) — a uniform defensive posture across all providers.

- **Claude Code** — `system`(subtype `init`) → `assistant` (message.content blocks of
  `text`/`tool_use`) → `result`. (`:71-116`)
- **Codex** — `thread.started`, `item.started`/`item.completed` (item types
  `command_execution`, `agent_message`), `turn.completed`, `error`. (`:704-742`)
- **Pi** — `session` (first line, carries the UUID), `message_update` →
  `assistantMessageEvent{type:"text_delta",delta}`, `tool_execution_start`, `agent_end`,
  `agent_error`. (`:554-606`) Verified against `@mariozechner/pi-coding-agent` 0.73.1.
- **Cursor** — reuses Claude's `assistant`/`result` shape, plus top-level
  `tool_call{type,subtype:"started"}` with nested `readToolCall`/`writeToolCall`/`function`.
  (`:140-199`)
- **OpenCode** — `step_start`(carries `sessionID`), `text`(part.type `text`), `tool_use`
  (part.type `tool`, `part.state.status==="completed"`, `part.state.input`), `error`. (`:895-936`)
- **Copilot** — `assistant.message_delta`{data.deltaContent}, `assistant.message`{data.content},
  `tool.execution_start`{data.toolCallId,toolName,arguments}, `result`{sessionId,exitCode,usage},
  `error`/`agent_error`. (`:1027-1094`)

All six also route **errors onto stdout**, not stderr (`:581`, `:733`, `:933`, `:1091`) — surfaced
as `result` events so the Orchestrator's stderr-empty fallback can show them. That is a consistent,
deliberate pattern across the whole registry.

## 5. The 5-event contract

Contract: `text | result | tool_call | session_id | usage` (`AgentProvider.ts:34`).

| Agent | text | result | tool_call | session_id | usage | Verdict |
|---|---|---|---|---|---|---|
| Claude Code | ✅ `assistant` text blocks | ✅ `result` | ✅ `tool_use` (allowlisted) | ✅ `system/init.session_id` | ⚠️ not parsed | **4/5** |
| Codex | ✅ `item.completed/agent_message` | ✅ same line | ✅ `item.started/command_execution` | ✅ `thread.started.thread_id` | ✅ `turn.completed.usage` | **5/5** |
| Pi | ✅ `text_delta` | ✅ `agent_end` | ✅ `tool_execution_start` | ✅ `session.id` | ❌ | **4/5** |
| OpenCode | ✅ `text` | ✅ `text` (same event) | ✅ `tool_use` | ✅ `step_start.sessionID` | ❌ | **4/5** |
| Cursor | ✅ `assistant` | ✅ `result` | ✅ `tool_call/started` | ✅ via `system/init` | ❌ | **4/5** |
| Copilot | ✅ `message_delta` | ✅ `assistant.message` | ✅ `tool.execution_start` | ✅ `result.sessionId` | ⚠️ `result.usage` present, unparsed | **4/5** |

**Nobody is fully blocked.** Codex is the only provider whose `usage` is actually wired into
`parseCodexUsage` (`:681`). The others can *emit* usage in several cases but the parser doesn't
map it yet — that is a small parser addition, not a capability gap.

**Two real gaps to flag:**
1. **Tool allowlist is strict.** `TOOL_ARG_FIELDS` (`:44`) only maps `Bash`, `WebSearch`,
  `WebFetch`, `Agent`; unlisted tools are silently dropped (`:88`, `:571`). OpenCode uses a
   *different* map (`OPENCODE_TOOL_ARG_FIELDS`, `:882`). Agents whose tools fall outside these
   names show **no tool calls at all** — a silent hole, not an error.
2. **No session resume** for OpenCode, Cursor, or Copilot (`captureSessions: false` on
   `opencode`, `:965`). Only Claude, Codex, and Pi support capture/resume/fork. If v1 needs
   session continuity, that alone rules out three of the six.

## 6. A / B / C — recommendation

**B. One image per agent.** Already implemented in `InitService.ts`; no new architecture.

- *Size:* per-agent images share an identical `node:22-bookworm` base and differ by one install
  line. The fat image is roughly the **sum** of six installs; a per-agent image is the **max** of
  one. Swapping agents costs nothing because the image is selected at boot.
- *Cold start:* A pays ~6x the pull and layer-cache pressure on every run. B pays 1x. C pays
  1x pull *plus* 10-60s of network install, every single boot, and adds a registry outage mode.
- *The decisive point:* in B, a broken or missing credential for Cursor **cannot** affect a
  Claude run. In A, all six credentials live in the same container forever. Blast radius scales
  with image strategy, and B is the only option where it doesn't.

## 7. v1 agent set

**Ship `claude-code` + `codex`.**

- **Claude Code** — the factory is already pointed here; it's the reference implementation of the
  seam. Full session capture/resume/fork. Prompt on stdin (no 120 KB cap).
- **Codex** — the only provider that satisfies the **5/5** contract (`turn.completed.usage` is
  parsed, `:739`). Different vendor, different OAuth/API-key path, different session format. If
  the seam survives two vendors it survives six.

**v1 = two agents proves the seam. Six agents proves a registry, which is a v2 problem.**

**Defer to v2:**
- **Pi** — closest to ready (4/5, full session support, prompt on stdin). Low-risk second wave.
- **OpenCode** — no session capture, prompt via argv. Fine, but nothing urgent pulls it forward.
- **Cursor** — argv-only prompt, 120 KB cap, and a login path that can hang a container. Defer.
- **Copilot** — argv-only prompt, 120 KB cap, and `GITHUB_TOKEN` is usually scoped to repos not
  billed to a model. Defer furthest.

**Out of scope for v1 entirely:** the fat image (A) and runtime install (C). Both are strictly
worse and one of them is already solved.

---

## Open questions (UNKNOWN, not guessed)

1. **Exact installed sizes** — all six are estimates. Needs `docker images` on a real build.
2. **Cursor API-key auth** — does the current `cursor.com/install` CLI honour `CURSOR_API_KEY`
   headlessly, or does it still want a browser login? *Verify before shipping Cursor.*
3. **Claude `usage` mapping** — the stream carries it; the parser doesn't read it. Confirm the
   field shape before wiring.
4. **Tool-name coverage per agent** — the allowlist drops unlisted tools silently. Each new
   agent needs its own map reviewed, not just a print command.