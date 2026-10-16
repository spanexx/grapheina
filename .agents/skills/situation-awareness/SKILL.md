---
name: situation-awareness
description: Run the situ CLI (globally installed) to analyze the git repository state — git status, recent commits, handoffs, session documents. Repo-aware, works in any codebase.
metadata:
Tags: git, rlm, context, chunk
---

## When to Use
- At the start of a work session
- After completing a task
- When you need context about what is in progress
- Before deciding what to work on next
- When you need to quickly recover context from previous sessions

## Invocation

`situ` is a GLOBAL CLI (installed at `~/.local/bin/situ`) — it is repo-aware and
works in ANY codebase. It replaced the old repo-tied `main.sh`.

```bash
situ                # project report (in a repo) or workspace overview (2+ repos)
situ status         # git + handoff report of the current repo
situ status <path>  # git + handoff report of a named repo
situ handoffs       # every repo under the workspace + its latest handoff
situ handoff        # full handoff of the current repo
situ handoff <path> # full handoff of a named repo
situ workspace-root # print the detected workspace root (used by the handoff skill)
```

## Two Modes

### Mode 1 — Workspace Overview (no project focused)

**Triggered when:** `situ` is run from a workspace root (not inside a git repo)
with 2+ child repos, or explicitly via `situ handoffs`.

**What it does:**
- Scans every child repo found under the workspace
- For each repo shows: path, branch, last commit, handoff status + summary
- Ends with an explicit **Action Required** block

**Agent behaviour in this mode:**

1. Read the output — note which repos have active handoffs
2. **Ask the user:** _"Which project would you like to work on?"_ — list the repo names
3. Wait for the user to reply
4. `cd <project>` and re-run `situ` for the focused report

The agent must not proceed with any work until the user has confirmed a project.

### Mode 2 — Project-Scoped Report (project focused)

**Triggered when:** `situ` is run from inside a project directory.

**What it outputs:**
- Workspace root + project root
- Git snapshot: branch, staged/unstaged/untracked counts, last commit
- Recent commits (last 10)
- Git status (short) + full diffs (staged + unstaged, capped at 200 lines each)
- Last handoff (summary + 20-line preview) from the repo's sessions dir
- Suggested next actions

## Repo Awareness (how it detects)

`situ` auto-detects:
- **Current repo** = `git rev-parse --show-toplevel` from cwd (or explicit arg)
- **Workspace** = walks up until the parent is no longer a git repo; override
  with `AGENT_WORKSPACE_ROOT` env var
- **Child repos** = git repos found at depth ≤ 3 under the workspace

Handoff lookup is **repo-local only**: `<repo>/sessions/.last-handoff`. There is no workspace-level or parent-of-repo fallback — a repo shows its own handoff, or "(no handoff)".

**Lesson (never repeat):** the old container fallback caused sessions to read stale handoffs from the wrong project, and a subcommand bug (`find_repo` receiving the subcommand as its arg) silently resolved the repo to `"status"`. Both are fixed. If `situ` ever shows a handoff path that is not the repo's own `sessions/.last-handoff`, the binary is stale or misconfigured — fix it, never trust the wrong handoff. A second bug was found later: running `situ` with no subcommand from a non-repo directory errored with `local: can only be used in a function` (a top-level `local count=0` in the workspace-overview branch) — fixed by dropping `local`. If `situ` ever errors with `local:`/`unbound variable`, the binary is stale — it should print help or the workspace overview from any directory.

## What It Analyzes

### 1. Git State
- Current branch, staged/unstaged/untracked counts, last commit, last 10 commits
- Short status + diffs (capped at 200 lines each)

### 2. Handoff Documents
- Reads `<repo>/sessions/.last-handoff` (new format: content starts with `#`;
  legacy: single-line pointer to a /tmp file)
- `situ handoffs` shows one-line summaries across all repos

### 3. Session Documents
- Session docs live in the repo's `sessions/` dir (`.md` files, newest-first)
- The handoff skill writes them; situation-awareness surfaces the latest

## Integration

### Uses
- git commands for version control state
- `situ` for everything above — no repo-specific paths, no local scripts
- `$PROJECT_ROOT/sessions/.last-handoff` written by the `handoff` skill

## Constraints

- Non-invasive: Read-only operations
- Fast: completes in seconds for a focused repo; workspace scans are a bit slower
- Actionable: always suggests next steps or asks for user input
