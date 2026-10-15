# 20 — Agent Session Formats: OpenCode, Cline, Kilo Code, Cursor CLI, Copilot CLI

Research date: 2026-10-14. Companion to `20-*` covering Claude Code, Codex, Pi.
Everything marked **UNKNOWN** was not verified against source. Do not guess past it.

---

## 1. VERDICT

**Yes — all five can be normalised into one JSONL message shape, and the natural canonical shape is the Anthropic one: `{role, content: [blocks]}` with tool calls as `tool_use` / `tool_result` blocks inside `content`.** Four of the five carry `role`+`content` natively or one trivial hop away (Cline's `api_conversation_history.json` *is* `Anthropic.MessageParam[]`; Copilot's `user.message`/`assistant.message` event types map 1:1 onto `role`; Cursor's JSONL transcripts use `role`/`message` envelopes). **OpenCode is the outlier only in storage medium, not in message shape — and it is arguably the easiest of the five to normalise**, because its `session_message.data` column is a JSON blob with an explicitly tagged union (`type` + typed parts) that is already one-to-one with `role` + content blocks. The genuine outlier is **Cursor**, whose resumable store is a SQLite of content-addressed BLOBs whose root is a **protobuf** message; you must decode protobuf to walk it, and a clean-room writer only works against one pinned Linux CLI release. **The one real blocker for the "inject a message into a running agent" requirement is not format at all — it is concurrency.** Every one of these stores assumes a single owning process. Cursor's forum staff explicitly confirm non-destructive resume is broken today; Cline and OpenCode both have live file watchers (`chokidar`) that will overwrite your injected message on the next debounced flush.

**Read+write ranking:** Cursor's `agent-transcripts/*.jsonl` (read) and `events.jsonl` (write) are the two cheapest files to touch. **Inject safely = append-only to a file nobody re-writes** — that means Copilot `events.jsonl` and Cline `api_conversation_history.json`, and *not* Cline's `taskHistory.json` (watched) and *not* Cursor's transcript (clobberable).

---

## 2. Comparison table

| Agent | Session location (pattern) | File or DB | Resume flag | Read | Write |
|---|---|---|---|---|---|
| **OpenCode** | `$XDG_DATA_HOME/opencode/opencode.db` (Linux `~/.local/share/opencode/`; Windows `%LOCALAPPDATA%/opencode/` or `%APPDATA%/opencode/`). macOS **UNKNOWN**. Plus `opencode.db-wal` / `-shm`. | **SQLite** (Drizzle, effect-sqlite driver) | `opencode --continue` / `-c`; `/sessions` in TUI; `opencode export <id>` / `opencode import <file>` | Yes — `SELECT … FROM session_message ORDER BY seq`, or `opencode export <id>` for JSON | Yes — `session_inbox` insert (canonical) or `opencode import`. **WAL/concurrency caution** |
| **Cline** | `CLINE_DIR` else `~/.cline/`. CLI: `~/.cline/data/tasks/<taskId>/{api_conversation_history.json, ui_messages.json, task_metadata.json, settings.json}` + `~/.cline/data/state/taskHistory.json`. Legacy VS Code ext: VS Code `globalStorage/saoudrizwan.claude-dev/tasks/<id>/` | **Files** (JSON) | CLI: `cline --task <id>` / `-T <id>`; `cline h` (history) | Yes — plain JSON | Yes — plain JSON. **But `taskHistory.json` has a chokidar watcher + debounced writer** |
| **Kilo Code** | `KILO_DB` else `~/.local/share/kilo/kilo.db` (Windows `%USERPROFILE%\.local\share\kilo\kilo.db`). Config `~/.config/kilo/kilo.json` (legacy `opencode.json`) | **SQLite** | `kilo --continue` / `-c`; `--session <id>`; `/sessions` \| `/resume` \| `/continue` | Yes — `kilo db path`, `kilo db query "SELECT …"` | Yes. **Exact schema UNKNOWN** |
| **Cursor (cursor-agent)** | Root: `$CURSOR_CONFIG_DIR` → `$XDG_CONFIG_HOME/cursor` → `~/.cursor`.<br>• JSONL transcripts: `~/.cursor/projects/**/agent-transcripts/**/*.jsonl`<br>• Native store: `~/.cursor/chats/<md5(cwd)>/<chat-uuid>/{store.db, meta.json}`<br>• ACP: `~/.cursor/acp-sessions/**/store.db`<br>• IDE Composer: `state.vscdb` (`%APPDATA%/Cursor/User/`, `workspaceStorage/*/`) | **Both.** `agent-transcripts` = JSONL; `store.db` = **SQLite of BLOBs with a protobuf root** | `cursor-agent --resume=<chatId>`, `--continue` (= `--resume=-1`), `cursor-agent ls`, `agent resume` | JSONL: easy. store.db: protobuf decode required | JSONL: risky (forum reports `--resume` clobbers it). store.db: clean-room writer exists but pinned to one Linux release |
| **GitHub Copilot CLI** | `~/.copilot/session-state/<session-uuid>/{events.jsonl, workspace.yaml, checkpoints/NNN.json, plan.md, files/, index.md, session.db}`. Legacy `~/.copilot/history-session-state/`. Index: `~/.copilot/session-store.db`. Metrics: `~/.copilot/events.jsonl` | **Both.** `events.jsonl` = JSONL (canonical for resume). `session-store.db` = derived SQLite index (+FTS5) | `copilot --resume [ID]`, `--continue`, `-r`; in-session `/resume`, `/continue`; `/chronicle reindex` | Yes — `events.jsonl` is the replay source of truth | Yes — **append-only JSONL, the cleanest injection target of all five**. Keep `parentId` chain + `enqueued_seq` consistent |

---

## 3. Per-agent detail

### 3.1 OpenCode (sst/opencode) — full detail in §4

One SQLite DB. A conversation is **many rows**, not one. Resume works. `opencode export` / `opencode import` give a JSON round-trip.

### 3.2 Cline

Location resolved in `cli/src/vscode-context.ts`:
```ts
const CLINE_DIR = config.clineDir || process.env.CLINE_DIR || path.join(os.homedir(), ".cline")
```
```
~/.cline/
├── data/
│   ├── globalState.json
│   ├── secrets.json
│   ├── workspace/
│   └── tasks/<taskId>/
│       ├── api_conversation_history.json   <-- the real transcript
│       ├── ui_messages.json                 <-- UI-only view model
│       ├── task_metadata.json
│       └── settings.json
├── state/taskHistory.json                   <-- index of tasks
└── log/
```
(from `cli/man/cline.1.md`; `src/core/storage/disk.ts` implements `getGlobalStorageDir`, `getSavedApiConversationHistory`, `getTaskMetadata`.)

**Message shape** — `getSavedApiConversationHistory()` returns `Promise<Anthropic.MessageParam[]>`. That is literally the Anthropic wire format:
```json
[
  { "role": "user", "content": [ { "type": "text", "text": "fix the failing test" } ] },
  { "role": "assistant", "content": [
      { "type": "text", "text": "Let me look." },
      { "type": "tool_use", "id": "toolu_01", "name": "read_file", "input": { "path": "a.ts" } }
  ]},
  { "role": "user", "content": [
      { "type": "tool_result", "tool_use_id": "toolu_01", "content": "…" }
  ]}
]
```
**This is the easiest agent of the five to normalise — zero mapping required.**

**Resume** — `cline --task <id>` / `-T <id>`: *"Resume an existing task by ID. The prompt argument becomes an optional follow-up message."* That flag alone gives you steer-into-a-running-session semantics.

**Watch out** — `StateManager` runs `chokidar.watch(taskHistory.json)` and, on any external change, reloads into cache and calls `onSyncExternalChange()`. Debounced persistence (`scheduleDebouncedPersistence`) can clobber concurrent writes. Write `api_conversation_history.json`, not `taskHistory.json`, and expect a live-file race in headless use.

### 3.3 Kilo Code

**Kilo Code is a fork of OpenCode.** Its config file is `~/.config/kilo/kilo.json` with legacy name `opencode.json[c]`; legacy dirs `./.kilo/` and `./.kilocode/` are still read. This is the single most important fact about Kilo — whatever OpenCode adapter you build has a decent chance of applying.

Local store is **SQLite**, embedded (docs explicitly warn *"Local sessions use SQLite, not PostgreSQL"*):
| Platform | Path |
|---|---|
| Windows | `%USERPROFILE%\.local\share\kilo\kilo.db` |
| macOS / Linux | `~/.local/share/kilo/kilo.db` |
| VS Code Remote SSH | `~/.local/share/kilo/kilo.db` on the **remote** host |

Overridden by `KILO_DB`, `XDG_DATA_HOME`, dev channel, or isolated env — **always resolve with `kilo db path`, never hardcode.** Also `kilo.db-wal` / `kilo.db-shm`.

Inspect with `kilo db query "SELECT …"` or `sqlite3 -readonly`. Stop Kilo before copying/replacing.

**Message shape: UNKNOWN.** Because it is an OpenCode fork, the schema is very likely OpenCode's — but *which* generation is UNKNOWN: OpenCode's own schema has migrated from `session`/`message` to `session_v2`/`session_message` (see §4), and Kilo's fork point is unknown. **Verify empirically before writing an adapter.**

**Resume** — `kilo --continue` / `-c` (most recent session in current workspace); `--session <id>`; in-session `/sessions` (aliases `/resume`, `/continue`), plus `/new`, `/fork`, `/undo`/`/redo`, `/timeline`, `/export`, `/copy-session`.

**Bonus for you** — Kilo ships `/resume-claude [uuid]` and `/resume-codex [uuid]`: *"Kilo discovers Claude Code transcripts under `~/.claude/projects/` and Codex CLI rollouts under `~/.codex/sessions/`."* So the Claude Code and Codex JSONL parsers are already implemented inside Kilo as a reference implementation.

**Complexity flag** — Kilo's `--session` / `--fork` / `/share` path calls a **cloud tRPC backend** (`sessionClient.ts`, requires `kiloToken`) with a `SessionSyncService` flushing on a 1s interval. Local-only operation and authenticated sync are different code paths. If you drive Kilo headless with no token, confirm `--continue` still works against the local DB (the code does fall back to local task history, but the precedence differs).

### 3.4 Cursor (`cursor-agent` CLI)

**Two separate stores. Staff-confirmed, do not assume they unify.**

**(a) Readable JSONL transcripts** — `~/.cursor/projects/**/agent-transcripts/**/*.jsonl`, one file named after the chat id that `--resume` takes. Described as *"`role`/`message` envelopes"* (Claude-Code-like). Contains every turn **including tool calls, but NOT tool results**. Exact field names for tool-call blocks: **UNKNOWN** (not pinned from source in this pass). Folder name encoding blanks dots/spaces/non-ASCII and truncates long paths with a hash — **that encoding is not reversible**, so do not use the folder name to reconstruct `cwd`; read `meta.json` instead.

**(b) Native resumable store (the real one)** —
```
~/.cursor/chats/<md5-of-cwd>/<chat-uuid>/meta.json
~/.cursor/chats/<md5-of-cwd>/<chat-uuid>/store.db
```
`store.db` is SQLite with `blobs(id TEXT, data BLOB)` and `meta(key, value)`. Walk it like this:
1. `meta` holds a single hex-encoded JSON value naming `latestRootBlobId`.
2. That root blob is **protobuf**. Its **repeated field 1** is the ordered list of 32-byte child digests (message order).
3. Each child blob is **plain JSON, Vercel AI SDK shape**: `{"role", "content"}` where content parts are typed `text`, `reasoning`, `tool-call`, `tool-result`; tool calls keyed `toolName` / `args`.
4. `meta.json` beside it holds the `cwd` (and the parent dir name is md5 of that path).

So Cursor's message shape *is* `role` + `content` blocks — it just sits behind a protobuf manifest. Writable: a clean-room `session-migrate` adapter exists and produces a resumable `store.db`, but it is **pinned to one Linux x86_64 cursor-agent release** and refuses to overwrite an existing conversation. Treat Cursor write as high-friction.

**(c)** `~/.cursor/acp-sessions/**/store.db` — same shape, ACP surface.
**(d)** IDE Composer — `state.vscdb` under `workspaceStorage/*/` and `globalStorage/`, keyed by **composer id**. **The CLI's `--resume` does not accept these**, and a Cursor staff member on the forum confirmed IDE ↔ CLI session stores are separate and unsynced, with no common stable session id today.

**Resume** — `cursor-agent --resume [chatId]`, `--continue` (alias `--resume=-1`), `cursor-agent ls` (list, per-workspace), `agent resume`, `agent create-chat`. Headless: `-p/--print` with `--output-format text|json|stream-json` (+ `--stream-partial-output` for per-delta text).

**Warning (forum, confirmed by staff):** *"In practice it can also rewrite/clobber the observe transcript jsonl for that id."* Watch `cursor-agent` headless by reading the transcript, not by shelling out and re-entering.

### 3.5 GitHub Copilot CLI

```
~/.copilot/
├── config.json
├── session-state/<session-uuid>/
│   ├── events.jsonl            <-- canonical replay source of truth
│   ├── workspace.yaml          <-- id, cwd, summary_count, created_at, updated_at
│   ├── checkpoints/001.json    <-- {title, overview, history, work_done, technical_details, important_files, next_steps}
│   ├── plan.md
│   ├── files/                  <-- session artifacts
│   ├── index.md                <-- summary written at session end
│   ├── session.db              <-- per-session SQLite, TODOS ONLY
│   └── research/
├── history-session-state/      <-- legacy, migrated on resume
├── session-store.db            <-- global SQLite index (derived)
├── events.jsonl                <-- per-session usage metrics only
└── mcp-config.json
```
Authority order (per Copilot CLI internals): **`events.jsonl` is canonical; workspace sidecars are human-readable state; `session-store.db` is a *derived* searchable index and can be rebuilt with `/chronicle reindex`; remote/cloud is optional sync.** Writing to the SQLite index directly is pointless — it will be regenerated.

**Message shape — the real event envelope** (verified example):
```json
{"type":"session.start",
 "data":{"sessionId":"09371a50-…","version":1,"producer":"copilot-agent","copilotVersion":"0.0.420",
         "startTime":"2026-03-02T15:10:04.678Z",
         "context":{"cwd":"C:\\…\\my-project","gitRoot":"…","branch":"master"}},
 "id":"<event-uuid>","timestamp":"2026-03-02T15:10:04.817Z","parentId":null}
```
```json
{"type":"user.message",
 "data":{"content":"review my staged but uncommitted changes for issues",
         "transformedContent":"<current_datetime>…</current_datetime>\n\nreview my staged…",
         "attachments":[],"interactionId":"9352571e-…"},
 "id":"…","timestamp":"…","parentId":"<parent-event-uuid>"}
```
```json
{"type":"assistant.message",
 "data":{"messageId":"<uuid>",
         "content":"I'll review the staged changes in those three files.",
         "toolRequests":[
           {"toolCallId":"tooluse_…","name":"report_intent",
            "arguments":{"intent":"Reviewing staged changes"},"type":"function"},
           {"toolCallId":"tooluse_…","name":"powershell",
            "arguments":{"command":"git --no-pager diff --cached --stat",
                         "description":"Show staged diff"},"type":"function"}],
         "interactionId":"…",
         "reasoningOpaque":"<base64-encrypted-reasoning>",
         "reasoningText":"The user wants me to review staged git changes…"},
 "id":"…","timestamp":"…","parentId":"…"}
```
Other event types: `tool.execution_start` (`{toolCallId, toolName, arguments}`), `tool.execution_complete` (`{success:true}`), `assistant.turn_start` (`{turnId}`), `session.shutdown`, `session.compaction_start` / `_complete`, `session.snapshot_rewind`.

**There is no `role` field — the `type` IS the role.** Tool calls are a **sibling array `data.toolRequests`**, not content blocks. Use `data.content`, never `data.transformedContent` (that one has system context injected).

**Resume** — `copilot --resume` (picker) / `--resume SESSION-ID` / `--resume=<id or name>` / `-r`; `copilot --continue` (prefers current working directory); in-session `/resume [ID]`, `/continue`, `/session`, `/rename`, `/compact`, `/undo`. `/chronicle reindex` rebuilds the index **and syncs to your GitHub account**.

**Read/write verdict:** best injection target of all five. `events.jsonl` is append-only with an explicit `parentId` chain and a monotonic per-session ordering via `parentId`. Append a `user.message` event with a fresh `id`/`timestamp`, `parentId` = previous event, and reuse the session's `interactionId` if you want it merged into the current turn.

---

## 4. OpenCode's SQLite schema in detail (the priority)

### Location
`packages/core/drizzle.config.ts`:
```ts
export default defineConfig({
  dialect: "sqlite",
  schema: ["./src/**/*.sql.ts", "./src/**/sql.ts"],
  out: "./migration",
  dbCredentials: { url: "/home/thdxr/.local/share/opencode/opencode.db" },
})
```
Resolution chain reported by third-party tooling: `XDG_DATA_HOME/opencode/` → Windows `%LOCALAPPDATA%/opencode/` or `%APPDATA%/opencode/` → `~/.local/share/opencode/`. **macOS path: UNKNOWN** (many builds still use `~/.local/share/opencode/` via the `global` npm package; not confirmed). Override env: **UNKNOWN**, but `OPENCODE_DATA` / `XDG_DATA_HOME` are the likely knobs.
Live file also produces `opencode.db-wal` and `opencode.db-shm` — **a raw `cp` of only `opencode.db` will lose committed WAL state.** Use `sqlite3 .backup` or the SQLite backup API.

### Table: `session_v2` (one row per conversation)

```sql
CREATE TABLE `session_v2` (
  `id` text PRIMARY KEY,
  `project_id` text NOT NULL,
  `workspace_id` text,
  `parent_id` text,                    -- subagent sessions
  `fork_session_id` text,
  `fork_boundary` text,
  `slug` text NOT NULL,                -- URL-safe session slug
  `directory` text NOT NULL,           -- cwd the session ran in
  `path` text,
  `title` text,
  `version` text NOT NULL,             -- opencode version that created it
  `share_url` text,
  `summary_additions` integer,
  `summary_deletions` integer,
  `summary_files` integer,
  `summary_diffs` text,
  `metadata` text,
  `cost` real DEFAULT 0 NOT NULL,
  `tokens_input` integer DEFAULT 0 NOT NULL,
  `tokens_output` integer DEFAULT 0 NOT NULL,
  `tokens_reasoning` integer DEFAULT 0 NOT NULL,
  `tokens_cache_read` integer DEFAULT 0 NOT NULL,
  `tokens_cache_write` integer DEFAULT 0 NOT NULL,
  `revert` text,
  `permission` text,
  `agent` text,
  `model` text,
  `time_created` integer NOT NULL,
  `time_updated` integer NOT NULL,
  `time_idle` integer,
  `time_viewed` integer,
  `idle_outcome` text,                 -- succeeded | failed | interrupted
  `time_compacting` integer,
  `time_archived` integer,
  `time_suspended` integer,
  `resume_attempts` integer DEFAULT 0 NOT NULL,
  CONSTRAINT `fk_session_v2_project_id_project_id_fk`
    FOREIGN KEY (`project_id`) REFERENCES `project`(`id`) ON DELETE CASCADE
);
```
> Note the table is **`session_v2`, not `session`** — a migration `20260622170816_reset_v2_session_state` reset session state, and a `20260910120000_clear_v1_session_permission` exists. If you write queries, target `session_v2`.

### Table: `session_message` (a conversation is MANY rows)

```sql
CREATE TABLE `session_message` (
  `id` text PRIMARY KEY,          -- 'msg_' + monotonic ascending id
  `session_id` text NOT NULL,
  `type` text NOT NULL,           -- see union below
  `seq` integer NOT NULL,         -- per-session ordering, UNIQUE
  `time_created` integer NOT NULL,
  `time_updated` integer NOT NULL,
  `data` text NOT NULL,           -- <-- THE WHOLE MESSAGE, JSON-encoded
  CONSTRAINT `fk_session_message_session_id_session_v2_id_fk`
    FOREIGN KEY (`session_id`) REFERENCES `session_v2`(`id`) ON DELETE CASCADE
);
CREATE UNIQUE INDEX `session_message_session_seq_idx`             ON `session_message` (`session_id`,`seq`);
CREATE        INDEX `session_message_session_type_seq_idx`        ON `session_message` (`session_id`,`type`,`seq`);
CREATE        INDEX `session_message_session_time_created_id_idx` ON `session_message` (`session_id`,`time_created`,`id`);
CREATE        INDEX `session_message_time_created_idx`             ON `session_message` (`time_created`);
```
**Read a session:**
```sql
SELECT id, seq, type, time_created, json_extract(data,'$.text') AS text, data
FROM   session_message
WHERE  session_id = ?
ORDER  BY seq;
```

### Table: `session_inbox` — **this is your injection point**
```sql
CREATE TABLE `session_inbox` (
  `id` text PRIMARY KEY,
  `session_id` text NOT NULL REFERENCES `session_v2`(id) ON DELETE CASCADE,
  `type` text NOT NULL,
  `payload` text NOT NULL,   -- JSON
  `delivery` text NOT NULL,
  `enqueued_seq` integer NOT NULL,
  `time_created` integer NOT NULL
);
CREATE UNIQUE INDEX `session_inbox_session_enqueued_seq_idx`     ON `session_inbox` (`session_id`,`enqueued_seq`);
CREATE        INDEX `session_inbox_session_delivery_seq_idx`     ON `session_inbox` (`session_id`,`delivery`,`enqueued_seq`);
```
Migrations `20260603141458_session_input_inbox` → `20260604172448_event_sourced_session_input` → `20260622202450_simplify_session_input` → `20260812181746_session_inbox` show this was deliberately built as the steer-while-busy queue (OpenCode's own docs describe "prompts steered in while it was busy" — see the `Idle` message type below).
**Exact allowed values for `type`, `payload`, `delivery`: UNKNOWN.** Read `packages/core/src/session/inbox.ts` and `packages/schema/src/session-inbox.ts` before writing a row.

### Table: `event` / `event_sequence` — event-sourced log (second source of truth)
```sql
CREATE TABLE `event_sequence` (`aggregate_id` text PRIMARY KEY, `seq` integer NOT NULL, `owner_id` text);
CREATE TABLE `event` (
  `id` text PRIMARY KEY,
  `aggregate_id` text NOT NULL REFERENCES `event_sequence`(`aggregate_id`) ON DELETE CASCADE,
  `seq` integer NOT NULL,
  `created` integer DEFAULT 0 NOT NULL,
  `type` text NOT NULL,
  `data` text NOT NULL
);
CREATE UNIQUE INDEX `event_aggregate_seq_idx`      ON `event` (`aggregate_id`,`seq`);
CREATE        INDEX `event_aggregate_type_seq_idx` ON `event` (`aggregate_id`,`type`,`seq`);
```
`packages/core/src/session/projector.ts` projects events → `session_message`. **Both stores exist and must agree.** Writing `session_message` without a matching `event` will likely be overwritten or cause a projector replay to disagree. This is the biggest correctness risk in the whole OpenCode adapter. **UNVERIFIED which is authoritative for resume.**

### Other tables (not transcript, listed for completeness)
`project`, `project_directory`, `worktree`, `permission`, `workspace`, `instruction_blob`, `instruction_entry`, `instruction_state`, `session_pending`, `kv`, `account`, `account_state`, `control_account`, `credential`.

### Message union (from `packages/schema/src/session-message.ts`)

Discriminated on **`type`**, not `role`. Every variant carries `{ id: "msg_…", metadata?, time: { created } }` as `Base`.

```ts
type = "user" | "assistant" | "system" | "synthetic" | "skill" | "shell"
     | "compaction" | "idle" | "agent-switched" | "model-switched" | "location-switched"
```

**`user`** — `{ type:"user", text, files, agents, skills }`
**`synthetic`** — `{ type:"synthetic", text, description? }`
**`system`** — `{ type:"system", text, description? }`  (*"the model-facing update text, frozen at emit time"*)
**`skill`** — `{ type:"skill", skill, name, text }`
**`shell`** — `{ type:"shell", shellID, command, status, exit?, output?, time:{created, completed?} }`
**`assistant`** — `{ type:"assistant", agent, model, content: AssistantContent[], snapshot?, finish?, rawFinish?, providerState?, cost?, tokens?, error?, retry?, time:{created, streamed?, completed?} }`

`AssistantContent[]` is the direct analogue of Anthropic content blocks:
```ts
{ type:"text",      text: string, state?: ProviderState }
{ type:"reasoning", text: string, state?: ProviderState, time?: {created, completed?} }
{ type:"tool",      id: string, name: string, executed?: boolean,
                    providerState?, providerResultState?,
                    state: ToolState,
                    time: { created, ran?, completed? } }
```
`ToolState` is itself a 4-way tagged union on `status`:
```ts
{ status:"streaming", input: string }
{ status:"running",   input: Record<string,unknown>, metadata: Record<string,Json> }
{ status:"completed", input: Record<string,unknown>, content: NonEmptyArray<Content>, metadata? }
{ status:"error",     input: Record<string,unknown>, error: SessionError.Error, content?, metadata? }
```
**`idle`** — `{ type:"idle", outcome:"succeeded"|"failed"|"interrupted", error? }` — *"every step since the previous marker belongs to one turn, **including prompts steered in while it was busy**."* This is your turn-boundary signal, and it is also the marker that proves OpenCode's inbox supports mid-turn steering.

**`compaction`** — `{ type:"compaction", status:"running"|"completed"|"failed", reason:"auto"|"manual", summary, recent, … }`

### Normalisation mapping (OpenCode → Anthropic)
```
type:"user"        → role:"user",    content:[{type:"text", text}]
type:"synthetic"   → role:"user"    (tag: synthetic)
type:"system"      → role:"system"
type:"assistant" content[]:
   {type:"text"}                      → {type:"text"}
   {type:"reasoning"}                 → {type:"thinking"}
   {type:"tool", state.status=…}     → {type:"tool_use",  id, name, input}  (+ tool_result on completion)
type:"shell"      → role:"assistant" content:[{type:"tool_use", name:"bash", input:{command}}]
type:"idle"       → turn boundary marker, not a message
type:"compaction" → meta event
```
No information loss on tool calls. **This is the cheapest mapping in the set.**

### Read/write verdict for OpenCode
- **Read: yes.** `opencode export <sessionID>` (JSON to stdout; also `packages/cli/src/commands/handlers/session/export.ts`) is the safe path — it runs the projector for you. Direct SQL is faster but you own the WAL snapshot.
- **Write: yes, two routes.** (1) `opencode import <file>` / `packages/cli/src/commands/handlers/session/import.ts` for full session rewrite. (2) `INSERT INTO session_inbox` for mid-run steering.
- **Do not** hand-insert `session_message` rows unless you also write matching `event` rows and bump `session_message.seq` correctly. `session_inbox.enqueued_seq` is `UNIQUE` per session — you must read the current max.

---

## 5. What does NOT normalise cleanly

1. **Cursor's resumable store is protobuf-rooted.** `store.db`'s root is a protobuf message whose repeated field 1 is child digests. You must protobuf-decode to walk a conversation; the leaves are nice `{"role","content"}` JSON but the spine is not. No stable IDL published. Writing is worse — a working clean-room adapter is pinned to one Linux x86_64 `cursor-agent` build.
2. **Cursor has two non-interchangeable session identities.** CLI chat UUID (`~/.cursor/chats/…`) vs IDE composer id (`state.vscdb`). Staff-confirmed no bridge; `--resume` does not accept a composer id and may clobber the transcript. Any factory that must "watch an IDE chat and continue it headlessly" is not solvable today.
3. **Cursor's transcript has tool calls but no tool results.** Results live only in `store.db`. You cannot build a complete replayable session from the JSONL alone — you must join the two by (tool name, arguments, order).
4. **Cursor's project folder encoding is lossy.** Dots, spaces, non-ASCII are blanked and long paths truncated with a hash — not reversible. Use `meta.json`'s `cwd`.
5. **Copilot tool calls are a sibling array, not content blocks.** `assistant.message.data.toolRequests[]` + separate `tool.execution_start`/`tool.execution_complete` events. Normalising means synthesising `tool_use`/`tool_result` blocks and pairing them on `toolCallId`. Also Copilot has **no `role` field at all** — event `type` carries it, and `user.message` vs `assistant.message` vs `tool.*` is a wider type space than `role` allows.
6. **Copilot has a `parentId` chain, so a naive append corrupts replay.** Any injected event needs a fresh `id`, a `timestamp`, and `parentId` = the true tail. Also `session-store.db` is *derived* — you don't update it; `/chronicle reindex` (which also pushes to GitHub) does.
7. **Concurrency is the real blocker, not format.** All five stores assume a single owning process. Cline watches `taskHistory.json` with chokidar and debounces writes. Cursor's resume has been reported to rewrite the observe transcript. Copilot states plainly: *"No session locking — concurrent access to same session is undefined."* A factory that injects into a *running* agent needs an out-of-band queue + the agent's own steering API, not a file write. **OpenCode's `session_inbox` is the only first-class injection queue found in this whole set.**
8. **Kilo Code's schema is UNVERIFIED.** OpenCode-derived but fork-point unknown; `session`/`message` vs `session_v2`/`session_message` unresolved.
9. **Cloud coupling.** Kilo `--session`/`--fork`/`/share` hit a tRPC backend with a 1s sync loop when `kiloToken` is set; Copilot syncs sessions to your GitHub account by default and `/chronicle reindex` uploads. Headless factory runs should disable these.
10. **Legacy tree everywhere.** Copilot has `history-session-state/` (migrated on resume); Cline has pre-SDK `globalStorage/…/tasks/` and `state/taskHistory.json`; OpenCode has V1→V2 session migrations. Any adapter needs a "which generation am I looking at" probe.

---

## 6. Recommended canonical shape

Anthropic-native, one JSON object per line:
```jsonc
{"v":1,"sessionId":"…","agent":"opencode","seq":12,"id":"…","parentId":null,
 "ts":"2026-10-14T12:00:00.000Z","role":"assistant","model":"…","cwd":"…",
 "content":[{"type":"text","text":"…"},
            {"type":"tool_use","id":"toolu_…","name":"bash","input":{…}},
            {"type":"tool_result","tool_use_id":"toolu_…","content":[{"type":"text","text":"…"}],"is_error":false}]}
```
Per-agent extraction cost: **Cline 0**, **OpenCode ~0**, **Claude Code/Codex/Pi 0**, **Copilot ~1 hop** (sibling array → blocks, pair on `toolCallId`), **Cursor ~1 hop + protobuf spine**, **Cursor IDE Composer UNKNOWN/blocked**.

## 7. Open questions to close next
1. Kilo Code: run `kilo db path` then `SELECT name, sql FROM sqlite_master` — is it `session`/`message` or `session_v2`/`session_message`?
2. OpenCode: read `packages/schema/src/session-inbox.ts` + `packages/core/src/session/inbox.ts` for allowed `type`/`delivery` values and the exact payload shape for a steered user message.
3. OpenCode: is `event` or `session_message` authoritative for `--continue` resume?
4. OpenCode: macOS data path (does `global` resolve to `~/Library/Application Support/opencode/` or `~/.local/share/opencode/`?).
5. Cursor: dump one real `agent-transcripts/*.jsonl` line to pin the exact envelope and tool-call block field names.