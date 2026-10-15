# Gap Analysis — `agent-worktree` → full Sandcastle-equivalent

**Date:** 2026-10-13
**You have:** `agent-worktree` (190-line Python CLI, stdlib only)
**Reference:** `sandcastle` (1,500 lines TS + Effect, 53 test files, 20 ADRs, 1,209 commits since 2026-03-17)

---

## 0. Executive summary

`agent-worktree` covers **Layer 1 (worktree)** — and does it *better* than Sandcastle in one
respect: it refuses to run on a dirty tree, which Sandcastle never does.

Everything above Layer 1 is **zero lines** in your repo. But the *total* gap is smaller than
it looks, because Sandcastle's 1,500 lines are ~80% **TypeScript-and-Effect ceremony**, not
difficulty. The actual logic is roughly:

| Layer | Real logic | Verdict |
|---|---|---|
| 1. Worktree | `git worktree add` + guards | ✅ **you have this** |
| 2. Sandbox | `docker run -v` / Vercel POST | ❌ ~30 lines bash |
| 3. Agent | `docker exec` + parse stream | ❌ ~40 lines bash |
| 4. Prompt | `sed` + `eval` | ❌ ~20 lines bash |
| 5. Orchestration | loop + `timeout` | ❌ ~30 lines bash |
| 6. Branch strategies | `git merge` | ❌ ~15 lines bash |
| 7. Session resume | `docker cp` JSONL | ❌ ~30 lines bash (claude/codex only) |
| 8. Scaffold / init | write files | ❌ ~60 lines bash |
| 9. Structured output | `jq` + validate | ❌ ~15 lines bash |

**≈240 lines of bash** gets you a working equivalent. Not a Rust rewrite.

---

## 1. What you already have

`C:\Users\shuga\Desktop\shared\.agents\skills\using-git-worktrees\bin\agent-worktree`

| Capability | Line | Status |
|---|---|---|
| Repo-root detection | 35 | ✅ `git rev-parse --show-toplevel` |
| **Dirty-tree refusal** | 106–114 | ✅ refuses unless `--allow-dirty` — **Sandcastle lacks this** |
| Slug normalization | 44–52 | ✅ lowercase-hyphens, ≤80 chars |
| Branch kind prefixes | 18 | ✅ `feature/ fix/ chore/ docs/` |
| Base branch resolution | 54–68 | ✅ current HEAD, or `--base`, verified to resolve to a commit |
| Branch collision refusal | 121–123 | ✅ |
| Worktree-path collision refusal | 127–130 | ✅ |
| Cleanup on failure | 137–139 | ✅ rmdir if empty |
| Machine-readable output | 141–148 | ✅ `KEY=value` + shell-quoted `NEXT=` |
| Tests | — | ✅ `tests/test_agent_worktree.py` |

**Entry point now fixed:** `C:\Users\shuga\.local\bin\agent-worktree.bat`
(same pattern as your existing `agent-notify.bat` / `opencode-action.bat`)

**Evidence it has run:** `~/.config/superpowers/worktrees/speechspark/capacity-100k-closeout`
and `speechSpark/_removed-worktrees-backup/{docker-no-spa, feature-kubernetes, ...}`

### Gaps in `agent-worktree` itself (fix before building on it)

| Missing | Sandcastle equivalent | Why it matters |
|---|---|---|
| **Reuse existing worktree** | `WorktreeManager.create` + ADR 0003 | Sandcastle makes reuse the **default** — re-running accumulates progress on the same branch |
| **Stale pruning** | `WorktreeManager.pruneStale` (`:461`) | `git worktree prune` + orphan dir sweep. Crashes leave junk that blocks re-runs |
| **Dirty check / remove** | `hasUncommittedChanges` (`:434`), `remove` (`:447`) | Your SKILL.md defers cleanup to a `finishing-a-development-branch` skill. If that's missing, worktrees leak |
| **Worktree locking** | ADR 0007 | Two concurrent agents can grab the same path |

> ⚠️ **Verify:** does `finishing-a-development-branch` exist in `.agents/skills/`? Your
> SKILL.md says cleanup is **REQUIRED** and pairs with it. If it's gone, that's the same
> class of problem as the missing shim.

---

## 2. The gap table

Legend: ✅ have · 🟡 partial · ❌ missing · ⏭️ intentionally skip

### Layer 1 — Worktree

| # | Capability | You | Effort | How |
|---|---|---|---|---|
| 1.1 | Create worktree + guards | ✅ | — | `agent-worktree <kind> <slug>` |
| 1.2 | Reuse worktree by name | ❌ | 10 min | if dir exists → `git -C $WT fetch && reset --hard origin/$B` |
| 1.3 | Prune stale | ❌ | 10 min | `git worktree prune` + `rm -rf` orphans under root |
| 1.4 | Dirty check / safe remove | ❌ | 10 min | `git -C $WT status --porcelain` → preserve if dirty |
| 1.5 | Worktree lock | ❌ | 20 min | `mkdir $WT/.lock` atomic; trap EXIT to release |

### Layer 2 — Sandbox providers

| # | Capability | You | Effort | How |
|---|---|---|---|---|
| 2.1 | **Docker** (bind-mount) | ❌ | 5 min | `docker run -d -v "$WT:/workspace" -v "$REPO/.git:$REPO/.git"` |
| 2.2 | **Podman** | ❌ | 5 min | same, `podman` |
| 2.3 | **Vercel** (isolated) | ❌ | 15 min | `POST api.vercel.com/v1/sandbox` — has `source.type:"git"`, clones repo itself, no sync needed |
| 2.4 | Daytona (isolated) | ❌ | 15 min | REST, similar shape |
| 2.5 | **no-sandbox** (host) | 🟡 | 5 min | ⏭️ **you already are this** — just run the agent in the worktree |
| 2.6 | Image build / remove | ❌ | 15 min | `sandcastle docker build-image` → `docker build -t sandcastle:local -f .sandcastle/Dockerfile .` |
| 2.7 | UID/GID alignment | ❌ | 20 min | ADR 0014: `--build-arg AGENT_UID=$(id -u)`. **On Windows this is a no-op** — see §5 |
| 2.8 | Windows `.git` mount fix | ❌ | 20 min | ADR 0006: in a worktree `.git` is a *file* → must also mount the parent `.git` dir |
| 2.9 | SELinux labels / mounts / cpus | ❌ | 15 min | Linux-only. ⏭️ skip on Windows |

### Layer 3 — Agent providers

| # | Capability | You | Effort | How |
|---|---|---|---|---|
| 3.1 | **Claude Code** | ❌ | 5 min | `claude -p --dangerously-skip-permissions` (prompt on stdin) |
| 3.2 | **OpenCode** | ❌ | 5 min | `opencode run --format json --model <m>` — **your own agent** |
| 3.3 | **Codex** | ❌ | 5 min | `codex exec` |
| 3.4 | Pi / Cursor / Copilot | ❌ | 5 min each | copy 3.1, change argv |
| 3.5 | **Unified event contract** | ❌ | 20 min | `text \| result \| tool_call \| session_id \| usage` — the whole abstraction |
| 3.6 | Interactive TTY | ❌ | 20 min | `docker exec -it`; PTY alloc if `stdin.isTTY` |
| 3.7 | Windows agent CLIs | ❌ | 5 min | agents install as `.cmd` → need `shell: true` / `cmd.exe /c` |

> 🔑 **3.5 is the only non-trivial piece.** Every provider's output (Claude stream-json,
> Cursor `tool_call`, Pi `message_update`, Copilot `assistant.message_delta`) gets squashed
> into those 5 events. In bash: `jq -c` + a `case` statement. ~20 lines.

### Layer 4 — Prompt pipeline

| # | Capability | You | Effort | How |
|---|---|---|---|---|
| 4.1 | `{{KEY}}` substitution | ❌ | 5 min | `sed` / `jq` replace |
| 4.2 | `` !`command` `` shell expansion | ❌ | 20 min | extract with `grep -oP`, `eval`, splice back. Run **parallel**, in sandbox |
| 4.3 | Fails fast on non-zero | ❌ | 5 min | ADR 0020: never retry, never degrade |
| 4.4 | `{{SOURCE_BRANCH}}` / `{{TARGET_BRANCH}}` | ❌ | 5 min | auto-inject both |
| 4.5 | Inline prompts bypass processing | ❌ | 5 min | ADR 0008: `prompt:` is literal, no substitution |

### Layer 5 — Orchestration

| # | Capability | You | Effort | How |
|---|---|---|---|---|
| 5.1 | Iteration loop | ❌ | 10 min | `for i in $(seq $MAX)` |
| 5.2 | Completion signal | ❌ | 10 min | grep output for `<promise>COMPLETE</promise>` → break |
| 5.3 | Idle timeout | ❌ | 10 min | `timeout 600` wrapper |
| 5.4 | **Completion timeout** | ❌ | 20 min | ADR 0019: signal seen but process won't exit → 60s grace, then **succeed with warning**, keep commits |
| 5.5 | AbortSignal / Ctrl-C | ❌ | 10 min | ADR 0004: `trap` → kill agent, **preserve worktree** |
| 5.6 | Hooks (host/sandbox × 3 points) | ❌ | 20 min | ADR: `copyToWorktree` → `host.onWorktreeReady` (sequential) → container → `host+sandbox.onSandboxReady` (parallel) |
| 5.7 | Per-step timeouts | ❌ | 15 min | ADR 0001: `timeout` per lifecycle step, typed error each |

> 🔑 **5.4 is the one people miss.** Without it: agent emits COMPLETE, a spawned `gh` child
> holds stdout open, you wait the full 10-min idle timeout, then **fail and throw away
> commits the agent already made**. With it: succeed at 60s with a warning.

### Layer 6 — Branch strategies

| # | Capability | You | Effort | How |
|---|---|---|---|---|
| 6.1 | `branch` (named) | 🟡 | 5 min | worktree is already on the branch — you're 90% there |
| 6.2 | `merge-to-head` | ❌ | 10 min | `git -C $REPO checkout --detach && git merge $BRANCH && git branch -D $BRANCH` |
| 6.3 | `head` (direct) | ⏭️ | — | skip; defeats the purpose |
| 6.4 | **Commit collection** | ❌ | 5 min | `git rev-list "$BASE".."$BRANCH" --reverse` |
| 6.5 | Preserve dirty worktree | ❌ | 10 min | check dirty → keep on disk, print `cd` instructions |

> ⚠️ **Merge conflicts are NOT resolved by code.** `SandboxLifecycle.ts:443` is a bare
> `git merge`; on conflict it throws and preserves the branch. Conflict resolution is the
> **merger agent** in `templates/parallel-planner/` (`merge-prompt.md:10`). See §4.

### Layer 7 — Session capture / resume / fork

| # | Capability | You | Effort | How |
|---|---|---|---|---|
| 7.1 | Capture session JSONL | ❌ | 20 min | `docker cp` from sandbox → host, rewrite `cwd` fields |
| 7.2 | Resume | ❌ | 15 min | copy back + `--resume <id>`. ADR 0011: exactly **one** iteration |
| 7.3 | Fork | ❌ | 10 min | `--resume <id> --fork-session`. ADR 0018: **session only**, not branch |
| 7.4 | Token usage | ❌ | 10 min | parse last assistant msg from JSONL |
| 7.5 | Subagent transcripts | ❌ | 15 min | Claude only. Best-effort, never fatal |

> ⚠️ Each agent stores sessions differently (Claude `~/.claude/projects/<enc>/`, Codex
> `~/.codex/sessions/YYYY/MM/DD/`, Pi `~/.pi/agent/sessions/`). **Only if you need
> multi-turn agents.** Ship without it first.

### Layer 8 — Scaffold / CLI

| # | Capability | You | Effort | How |
|---|---|---|---|---|
| 8.1 | `init` — write Dockerfile + prompt + env | ❌ | 30 min | heredocs |
| 8.2 | 5 workflow templates | ❌ | 2 hrs | copy `sandcastle/src/templates/*` — they're standalone by ADR 0009 |
| 8.3 | `build-image` / `remove-image` | ❌ | 10 min | `docker build` / `docker rmi` |
| 8.4 | `--agent` / `--sandbox` flags | ❌ | 20 min | `getopts` or `case` |

### Layer 9 — DX / observability

| # | Capability | You | Effort | How |
|---|---|---|---|---|
| 9.1 | Typed errors per step | ❌ | 20 min | Sandcastle has ~15 distinct error classes |
| 9.2 | Log file per run | ❌ | 10 min | `.sandcastle/logs/<branch>.log` |
| 9.3 | Shutdown registry | ❌ | 10 min | cleanup containers on SIGINT/SIGTERM |
| 9.4 | Context window report | ❌ | 5 min | sum input+cache tokens, `ceil(k)` |

---

## 3. Build order

### Phase 1 — Local loop, no container (~2 hrs)
**Goal: proves the concept, zero deps.**

```bash
run.sh <slug> --agent claude|opencode|codex [--prompt file]
  1. agent-worktree feature <slug>          ← reuse YOUR tool
  2. <agent> -p "$(cat prompt.md)"          ← run in worktree, on host
  3. git rev-list "$BASE".."$BRANCH"        ← collect commits
  4. print commits, leave branch parked      ← NO merge yet
```

Covers: 1.1, 3.1–3.5, 4.1, 5.1–5.3, 6.1, 6.4

**This alone is more useful than Sandcastle for local work** — it uses `agent-worktree`'s
dirty-tree safety, which Sandcastle doesn't have.

### Phase 2 — Container (~1 hr)
**Goal: isolation + prebaked toolchain.**

```bash
docker run -d --name "agent-$SLUG" \
  -v "$WT:/workspace" -v "$REPO/.git:$REPO/.git" \
  -w /workspace -e ANTHROPIC_API_KEY sandcastle:local sleep infinity

docker exec -i "agent-$SLUG" sh -c 'claude -p' < prompt.md
docker rm -f "agent-$SLUG"
```

Covers: 2.1, 2.6, 2.8, 5.5, 9.3

### Phase 3 — Merge + multi-agent (~2 hrs)
**Goal: the actual product.**

```bash
 5. git -C $REPO checkout --detach
 6. git -C $REPO merge "$BRANCH" || echo "CONFLICT → run merger agent"
 7. git -C $REPO branch -D "$BRANCH"
 8. parallel loop over issues (cap 4), then one merger agent
```

Covers: 6.2, 6.5, and the planner/executor/merger template

### Phase 4 — Cloud (~1 hr)
Covers: 2.3, 2.4. Only if you need to run away from your laptop.

### Phase 5 — Polish (optional)
Session resume (7.x), structured output (below), completion timeout (5.4).

---

## 4. Architecture — how it connects

```
                    ┌─────────────────────────────┐
   run.sh <slug> ──▶│  1. agent-worktree          │  ← YOUR existing tool
                    │     (guards + path naming)  │
                    └──────────┬──────────────────┘
                               │ WORKTREE=...  BRANCH=...
                               ▼
                    ┌─────────────────────────────┐
                    │  2. sandbox provider       │  ◀── THE ONE SEAM
                    │     docker | vercel | none │
                    └──────────┬──────────────────┘
                               │ exec(cmd, onLine)  copyOut()  close()
                               ▼
                    ┌─────────────────────────────┐
                    │  3. agent provider          │
                    │  claude|opencode|codex|... │
                    │  → 5 events: text, result,  │
                    │    tool_call, session_id,   │
                    │    usage                    │
                    └──────────┬──────────────────┘
                               ▼
                    ┌─────────────────────────────┐
                    │  4. orchestrator            │
                    │  loop · completion signal   │
                    │  idle timeout (fail)        │
                    │  completion timeout (warn) │
                    └──────────┬──────────────────┘
                               ▼
                    ┌─────────────────────────────┐
                    │  5. branch strategy         │
                    │  head | merge-to-head |     │
                    │  branch                     │
                    └─────────────────────────────┘
```

**Two seams make this swappable.** Everything else is plumbing.

- **Seam A — sandbox provider:** `{ exec, copyIn, copyFileOut, close }`
- **Seam B — agent provider:** `{ buildCommand, parseStreamLine, sessionStorage? }`

Add Vercel = implement Seam A once. Add Crush = implement Seam B in 30 lines.

### The 3-phase workflow (this is the actual product)

```
PLAN     opus    reads issues, picks non-overlapping ones, names branches
                 emits <plan>{issues:[{id,title,branch}]}</plan>
                   ↓
EXECUTE  sonnet × N parallel, cap 4, each on its OWN branch
                   ↓
MERGE    sonnet  git merge one at a time
                 "resolve conflicts intelligently"
                 npm run typecheck && npm test
                 fix failures before the next branch
                   ↓
         outer loop ×10 — newly unblocked issues picked next round
```

Copy from `sandcastle/src/templates/parallel-planner/`. It's the single most valuable
artifact in that repo. ~200 lines of `.mts` + 3 prompt files.

---

## 5. What to skip entirely

| Feature | Why skip |
|---|---|
| **Podman** | You have Docker Desktop (it's on PATH). Port when you need rootless. |
| **Daytona** | Vercel covers cloud. |
| **Rust rewrite** | You need agent-agnostic *dispatch*. Rust helps there, but bash proves the loop first. Ship Phase 1 in an afternoon. |
| **Effect / typed error taxonomy** | Bash has exit codes. 15 error classes → 15 distinct exit codes. |
| **SELinux, cpus, devices, networks** | Linux-only. |
| **`head` branch strategy** | Defeats isolation. Sandcastle's Docker default, and a footgun. |
| **Subagent transcript capture** | Claude-only, best-effort. |
| **8 of 20 ADRs on this box** | See below. |

### Windows reality check

You're on Windows. These Sandcastle problems **don't apply to you**:

- ADR 0005/0014 — UID/GID alignment (Linux-only)
- ADR 0006 — Windows git mount fix (**you need this**)
- ADR 0007 — worktree locking (still applies)
- SELinux labels (Linux-only)
- `/dev/kvm`, `--device` (Linux-only)

**Real Windows costs:** agents install as `.cmd` → need `cmd.exe /c`; `.bat`/`sh` divergence
in every script; Docker Desktop VirtioFS is slower than Linux bind mounts for `node_modules`.

---

## 6. Risk register — the 20 ADRs

Every one is a bug already paid for. Cheap to avoid, expensive to rediscover.

| ADR | Risk | Cost to you on Windows |
|---|---|---|
| 0001 | Per-step timeouts | Medium |
| 0002 | `cwd` option | Low |
| 0003 | Reuse worktree by default | **High** — without it, re-runs lose progress |
| 0004 | AbortSignal | Medium |
| 0005×2 | UID alignment / token counts | **None** |
| 0006 | Windows git mounts | **High — applies to you** |
| 0007 | Worktree locking | High |
| 0008 | Inline prompts bypass processing | Low |
| 0009 | Templates don't share code | Free |
| 0010 | Structured output | Medium |
| 0011 | Resume = 1 iteration | Low |
| 0012 | Provider owns session storage | Medium |
| 0014 | Docker UID build-arg | **None** |
| 0015 | `noSandbox()` in run/createSandbox | Free — you already have this |
| 0016 | Resume needs filesystem sessions | Low |
| 0017 | Sandbox-owned sync base ref | Low (isolated only) |
| 0018 | Fork is session-only | Medium |
| 0019 | **Completion timeout** | **High** — silent commit loss |
| 0020 | Prompt expansion fails fast | Low |

---

## 7. Bottom line

| Question | Answer |
|---|---|
| Can bash do this? | **Yes.** ~240 lines. |
| Can Rust do this? | Yes — but only earns its keep for **agent-agnostic dispatch** (HTTP providers have no CLI to shell out to). |
| Do you need Sandcastle? | **No.** It's a good reference. It's not a dependency. |
| Where should you start? | **Phase 1.** Worktree + agent + collect commits. No container. Two hours. |
| What's the highest-value thing to copy? | `src/templates/parallel-planner/` — planner/executor/merger. |
| What's the highest-value doc? | `CONTEXT.md` — the ubiquitous language. Steal it verbatim. |

---

## Appendix — file map

**Yours**
```
~\Desktop\shared\.agents\skills\using-git-worktrees\
├── SKILL.md
├── bin\agent-worktree                    ← 190 lines, stdlib only
└── tests\test_agent_worktree.py
~\.local\bin\agent-worktree.bat           ← shim (created 2026-10-13)
```

**Sandcastle (reference)**
```
sandcastle\
├── CONTEXT.md                            ← vocabulary, steal this
├── docs\adr\0001..0020                   ← 20 decisions, read before building
├── src\
│   ├── run.ts                (894)       ← one-shot
│   ├── createSandbox.ts      (~1000)     ← reusable warm sandbox
│   ├── createWorktree.ts     (~900)      ← worktree as first-class
│   ├── Orchestrator.ts       (604)       ← the agent loop
│   ├── SandboxFactory.ts     (691)       ← worktree × provider × strategy
│   ├── SandboxLifecycle.ts   (544)       ← hooks, merge, commit collect
│   ├── AgentProvider.ts      (1267)      ← 6 agents, the seam
│   ├── SandboxProvider.ts    (330)       ← the other seam
│   ├── WorktreeManager.ts                 ← create/prune/dirty/remove
│   └── templates\                        ← 5 workflow templates
└── .sandcastle\run.ts                    ← dogfoods itself
```