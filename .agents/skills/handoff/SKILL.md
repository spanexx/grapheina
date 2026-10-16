---
name: handoff
description: Compact the current conversation into a handoff document for another agent to pick up.
argument-hint: What will the next session be used for?
---

# Handoff

Write a handoff document summarizing the current conversation so a fresh agent can continue the work.

## Process

1. Compute the sessions directory — **always the repo's own `sessions/`** (`$PROJECT_ROOT/sessions`). No container pattern: `situ` reads handoffs repo-local only, so a handoff written anywhere else (workspace-level or parent-of-repo) is invisible to `situation-awareness`. Uses the GLOBAL `situ` CLI (installed at ~/.local/bin/situ) for repo awareness — works in any codebase:

   ```bash
   PROJECT_ROOT="$(git rev-parse --show-toplevel)"
   SESSIONS_DIR="$PROJECT_ROOT/sessions"
   mkdir -p "$SESSIONS_DIR"
   ```

2. **Write the handoff content to `$SESSIONS_DIR/.last-handoff`** (not `/tmp/`). The file lives in the project tree, so it survives reboots and `/tmp/` clears. Each new handoff overwrites the previous one — `.last-handoff` is always the latest.

3. **Leave a breadcrumb** by appending a short line to `$SESSIONS_DIR/.handoff-log.md` (append-only, capped at 50 lines). One line per handoff: `YYYY-MM-DD HH:MM | <first heading>`. This is the trail the next agent reads when the latest handoff has been overwritten by an even newer one.

   ```bash
   HEADING="$(head -n 1 "$SESSIONS_DIR/.last-handoff" | sed 's/^#\+ *//')"
   TS="$(date '+%Y-%m-%d %H:%M')"
   printf -- "- %s | %s\n" "$TS" "$HEADING" >> "$SESSIONS_DIR/.handoff-log.md"
   # Keep at most 50 lines.
   if [ "$(wc -l < "$SESSIONS_DIR/.handoff-log.md")" -gt 50 ]; then
       tmp="$(mktemp)"
       tail -50 "$SESSIONS_DIR/.handoff-log.md" > "$tmp" && mv "$tmp" "$SESSIONS_DIR/.handoff-log.md"
   fi
   ```

4. (Optional) keep a `/tmp/handoff-XXXX.md` copy for current-process conveniences like quick `cat` or attaching to logs. The canonical copy is always the one in `sessions/`.

5. Suggest relevant skills for the next session based on the work ahead.

6. **Avoid duplication** — don't rewrite content already captured elsewhere (PRDs, plans, ADRs, issues, commits, diffs). Reference them by path or URL instead.

7. If the user passed arguments describing the next session's focus, tailor the handoff accordingly — emphasize what's relevant to that focus.

8. **Checkpoint handoffs are valid.** A mid-task handoff (e.g. `surgical-change` step 2a — before a multi-surface/subtle implementation starts) is NOT a failure mode — it is the recovery path if the session is cut off before the final handoff. Checkpoint handoffs must end with explicit **Remaining steps (in order)** + the command that proves each, + **user-only follow-ups** (things the agent must NOT do). A fresh agent resumes by reading `.last-handoff` via `situation-awareness` and continuing the steps verbatim.

## For the next agent

`situation-awareness` reads `$SESSIONS_DIR/.last-handoff` directly. New format = the file contains the handoff content (starts with `#`). Legacy format = a single-line pointer to `/tmp/...`; the script still falls back to that for older handoffs.

If you need the breadcrumb trail, read `$SESSIONS_DIR/.handoff-log.md` (last 10 lines are usually enough).

```bash
# sessions dir = always <repo>/sessions — check situation-awareness output
cat <sessions-dir>/.last-handoff          # read latest handoff (full content)
tail -10 <sessions-dir>/.handoff-log.md   # read recent handoff trail
```

## Output

The handoff document should be:
- **Self-contained** — readable by a fresh agent with minimal context-switching
- **Linked** — external references instead of copied text
- **Actionable** — clear next steps and decision points
- **Scoped** — focused on what the next session will do, not the full history
