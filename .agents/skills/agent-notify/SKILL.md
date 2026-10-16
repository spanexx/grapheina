---
name: agent-notify
description: "Session start: drain hub outbox, sweep, raise pending pokes."
version: 1.0.0
author: hermes-curator
license: MIT
metadata:
  hermes:
    tags: [agents, notifications, hub, events]
    related_skills: [cline-one, wayfinder]
---

# agent-notify — local event hub for agents

## When to use
- Publishing or subscribing to job/event notifications between agents
  (Hermes, cline-one, OpenCode, dsh, Victor).
- Waking an agent with full context when a job finishes (the original itch:
  don't keep an agent resident while another works).
- Continuing the hub build (steps 4-5 below).
- **EVERY session start** — see Session-start protocol below.

## Session-start protocol (run these, in order, EVERY session start)
```bash
agent-notify drain --dry-run     # inspect queued pokes first (rate-limit safe)
agent-notify drain               # deliver pokes (real mouths; skip if dry-run showed nothing worth it)
agent-notify sweep               # delete events idle >24h
```
Then read `~/.agent-notify/pending-victor.md`. If it has lines, the oldest
untouched entry is an **eminent item**: raise it to Victor at the top of the
conversation ("still waiting on you: event X (asked <time>)") until he
replies (a reply naming the event id = engagement and clears the nag;
`agent-notify dismiss <id>` = explicit clear). Entries older than 24h get a
louder re-poke (`hermes send -t telegram:<channel>` with a priority marker)
before the conversation raise.

## What it is
A file-based event hub: **no daemon, no cron, no resident LLM**. Event =
id + a growing markdown note. Agents subscribe to a **specific event by id**
(not by type). Subscribers get only the new note lines (delta), delivered
through each agent's own CLI mouth with full context.

Design authority: wayfinder map `~/wayfinder/agent-notify/` — `map.md`,
`tickets/`, `map.html`, and `GRILL-agent-notify.txt` (13 verbatim Q&As =
the locked contract). The GRILL wins if anything contradicts this file.

## CLI (built, v1) — `~/.local/bin/agent-notify`
```bash
agent-notify create <id> --title T                 # start event (creator auto-subbed)
agent-notify update <id> "new line" --as <agent>   # append note + announce + deliver
agent-notify done <id> --outcome success|fail|waiting [--result P]
agent-notify reply <id> "text"                     # message on the event (dismisses pending)
agent-notify sub <id> | unsub <id>                 # event guestbook, --as <name>
agent-notify show <id>                             # full note
agent-notify ls [--mine]                           # live events (+your subs)
agent-notify watch <id>                            # tail updates
agent-notify sweep                                 # delete events idle >24h (subscriber-driven)
agent-notify drain [--target NAME]                 # deliver queued outbox pokes
agent-notify dismiss <id>                          # clear pending-victor entry
```
`--as` works before OR after the subcommand (parent-parser SUPPRESS trick).

## Data layout (~/.agent-notify/)
- `events.jsonl` — append-only announcement stream (one JSON line per update)
- `events/<id>.md` — the note (append-only body; the memory)
- `events/<id>.guest.json` — subscriber guestbook (event-side, option B)
- `outbox/<target>.jsonl` — pending deliveries (event, target, context, retries)
- `subscribers/<name>.json` — registry (name + filters; missing file = react to all)
- `pending-victor.md` — unanswered human pokes (eminent reminders)

## Delivery contract (locked)
- **Poke = full context in the message**: header (id, title, creator, time) +
  trigger (delta lines) + intent (the ask) + note path
  (`~/.agent-notify/events/<id>.md`). Fresh sessions must know *why* they
  were woken without remembering anything.
- **Victor always gets voice AND text — enforced by the CLI, not by agent
  memory.** Every `create`/`update`/`done`/`reply` that reaches Victor's
  guestbook entry automatically triggers `deliver_victor_voice()` alongside
  the normal text poke. No skill instruction or agent habit is required;
  the enforcement lives inside `agent-notify` itself (`queue_delivery()`).
  The spoken summary:
  - includes the event id, spoken with hyphens replaced by spaces (never
    skipped, never read as raw file-path text);
  - includes the title, the publisher, and the ask;
  - replaces the raw ISO timestamp with a voice-friendly "just now" (pokes
    are always delivered immediately after the underlying action);
  - never speaks the note file path.
  Pipeline: `speechspark say <summary> -o tmp.wav` → `ffmpeg` to Opus →
  `hermes send --to <telegram target> "[[audio_as_voice]] MEDIA:<opus>"`
  (native Telegram voice bubble, not a file attachment). If `speechspark` or
  `ffmpeg` is missing or the pipeline fails, voice delivery is skipped and
  logged — text delivery is independent and always still happens.
- **Fire-and-forget with retry**: outbox per target, <=3 retries, then
  DROP + `delivery/failed` line on the JSONL. Drain piggybacks on the
  subscriber's own sweep — no daemon.
- **Catch-up**: ONE coalesced poke per event ("missed N updates since
  <time>") — never one poke per missed update.
- **Enforced delivery, not filtering**: non-subscribers never see updates
  (user rejected the lazy read-all-filter approach outright).
- **TTL**: event idle 24h -> `event/expired` announced, note file DELETED
  from disk (no archive — user's explicit call). No cron: first subscriber
  to notice sweeps.

## Agent mouths (delivery targets)
| target | mouth |
|---|---|
| hermes | `hermes chat -q "<ctx>"` (slow — real session) |
| cline | `cline-one --timeout 600 "<ctx>"` (one-shot, no resume) |
| opencode | `opencode run "<ctx>"` (binary at ~/.opencode/bin, NOT on PATH) |
| victor | `hermes send -t telegram[:<channel>] "<ctx>"` |
| dsh | plugin (step 5 — not built yet) |

## Build order (remaining after CLI v1)
1. ✅ `agent-notify` CLI (done, tested end-to-end)
2. ✅ cline-one hook (done 2026-08-16) — auto-publish job/start+done
   **only when a report file was produced** (report-file = substance test;
   trivial runs stay silent). Every substantive run: create <slug> --title
   (first 60 chars of prompt) + sub hermes + sub victor + done --outcome
   success|fail --result <report path>, all `--as cline-one`. Slug = first
   3 prompt words. Preview with `cline-one --dry-run "<prompt>"` (prints the
   publish steps WITHOUT running cline — use when Cline is rate-limited).
3. ✅ Hermes session-start skill + pending-victor (done 2026-08-16) — this
   skill IS the session-start protocol (drain/sweep/raise); pending-victor.md
   mechanics live in the CLI (`dismiss`, `reply` auto-clears); 24h escalation
   and any-reply-engagement per GRILL Q13
4. ✅ Gateway watcher (done 2026-08-16) — NOT a separate plugin: the
   gateway's built-in webhook platform IS the watcher. `agent-notify`
   delivers to hermes by POSTing the poke (HMAC-signed, GitHub-style
   X-Hub-Signature-256) to http://localhost:8644/webhooks/agent-notify-pokes
   (config in `~/.agent-notify/webhook.json` — URL + subscription secret).
   The subscription spawns an actionable agent turn with {payload.context}.
   Dies with the gateway; session-start drain catches the gap. Enablement
   needed BOTH `.env` vars (WEBHOOK_ENABLED/PORT/SECRET) AND config.yaml
   (`hermes config set platforms.webhook.enabled true` etc.) — the CLI
   checks config.yaml only. Fallback: hermes chat -q if webhook.json missing
   or POST fails.
5. Telegram: dedicated private channel for pokes + dsh plugin
   (`~/.dsh/plugins/agent-notify.mjs`, Cordis named exports, mounts via
   profile cordis.patch.yml)

## Rate limits & dry-run
Real `agent-notify drain` invokes LLM mouths — `hermes chat -q` spawns a
full session (minutes), `cline-one` burns Cline quota. ALWAYS inspect
first: `agent-notify drain --dry-run` prints every queued poke
(target + full context) without invoking anything. This session hit Cline
rate limits — dry-run is the safe default; real drain only when pokes are
confirmed worth the quota.

## Pitfalls
1. `agent-notify drain` with no `--target` delivers to EVERY outbox —
   including hermes, which spawns a real `hermes chat -q` (minutes). For
   tests, always `--target` a scratch name.
2. Test events pollute the real hub — the test event `ui-audit` may still
   hold queued pokes for hermes/victor; sweep or drain deliberately.
3. `sweep` deletes note files permanently (by design). No archive exists.
4. Delivery failure of an unknown mouth = retry 3x then silent drop (logged
   as delivery/failed in JSONL) — check the log if a poke never arrives.
5. The old cline HTTP shim (8788) is dead — `cline-one` is the Cline path.
   OpenCode free shim (8787) still serves Hermes's free model.

## Verification
After publishing, check: note file grew (`show <id>`), JSONL has the new
line, outbox files exist for each subscriber, `ls --mine` reflects subs.

## References

- `references/wayfinder-map-ops.md` — re-baking this project's wayfinder map
  (bake-map.mjs format contract: `## Open Tickets` casing, title-vs-filename
  matching, ticket frontmatter).
- `references/delivery-poke-format.md` — the poke context assembly contract.