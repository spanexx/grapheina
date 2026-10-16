# Wayfinder local-markdown tracker ops (agent-notify map)

The agent-notify wayfinder map lives at `~/wayfinder/agent-notify/`:
`map.md`, `tickets/*.md`, baked `map.html` via
`~/.agents/skills/wayfinder/scripts/bake-map.mjs`.

## bake-map.mjs exact format contract (debugged 2026-08-16)

The parser is picky — these were all hit when baking this map:

1. **Heading must be exactly `## Open Tickets`** (capital T). `## Open
   tickets` → parseOpenTicketsTable finds nothing → "0 tickets; 0 edges".
2. **Table Ticket column = the ticket TITLE, not the filename.** The script
   matches `t.title === tableTitle` (or an 8-char prefix fallback). A
   filename like `01-event-vocabulary.md` never matches the title
   "What event vocabulary does the hub speak?" — use the title text.
3. **Ticket frontmatter uses bold-key lines at the very top of the file:**
   ```
   **Type:** wayfinder:research (AFK)
   **Status:** closed
   **Blocks:** T03, T04, T06
   # Ticket: <title — must match the map table>
   ```
   Type string must be `wayfinder:<research|grilling|task|prototype>`.
   `**Blocks:**` uses `T<nn>` ticket numbers; `T<nn>` in the table's
   Blocked-by column names them.
4. **Blocked-by column**: `—` (em dash) for unblocked, comma-list for
   blocked. The script derives edges from Blocks/Blocked-by pairs.
5. Table header must start `| # |` — the `#` column is required.
6. After editing: `node ~/.agents/skills/wayfinder/scripts/bake-map.mjs .`
   from the map dir. Expect `baked → map.html (N tickets; M edges)`.

## Ticket lifecycle conventions used on this map

- Close a ticket: set `**Status:** closed`, append `## Resolution
  (YYYY-MM-DD)` with the decision + why + delivery tag, remove the row
  from Open Tickets, add a one-line gist to `## Decisions so far`.
- Research tickets are closed by the agent (AFK). Grilling tickets are
  HITL — Victor's verbatim answers go in `GRILL-agent-notify.txt` (one
  numbered entry per question: verbatim question, locked answer, why).
- Delivery tags: `decision-only` (no build), `small-change` (build
  directly), `feature-pipeline` (full GRILL→PRD→IMPL).
