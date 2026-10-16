# Delivery poke — observed format (verified 2026-08-16)

Real outbox entry produced by `agent-notify update ui-audit "..." --as cline-one`
with hermes in the guestbook. This is the **full-context poke** contract in
action — copy this shape when building new mouths (step 4/5).

```json
{
  "event": "ui-audit",
  "target": "hermes",
  "context": "agent-notify: event ui-audit (UI reuse audit)\n"
             "publisher: cline-one  time: 2026-08-16T13:33:18Z\n"
             "new lines (1):\n"
             "  found 4 duplicate badges across post-card and home\n"
             "full note: /home/spanexx/.agent-notify/events/ui-audit.md "
             "(read it for the whole story)",
  "retries": 0,
  "queuedAt": 1786887198
}
```

## Poke context template (4 layers — GRILL Q9)
```
agent-notify: event <id> (<title>)
publisher: <who>  time: <when>
ask: <intent — what the receiver should DO>        (optional, done/reply only)
new lines (N):
  <delta line 1>
  <...>
full note: ~/.agent-notify/events/<id>.md (read it for the whole story)
```

## Lifecycle events on the JSONL stream
- `job/start` — create; deltaCount 0; noteOffset = note file size
- `job/progress` — update; deltaCount = lines added
- `job/done` — done; carries outcome + result path
- `message` — reply; carries target ("*" = all subscribers)
- `event/expired` — sweep after 24h idle (then note file deleted)
- `delivery/failed` — drained outbox entry dropped after 3 retries

## CLI quirks learned in testing
- `--as` accepted before OR after the subcommand (parent-parser with
  `default=argparse.SUPPRESS`).
- `drain` without `--target` = deliver to EVERY outbox — hermes's mouth is
  a real `hermes chat -q` (slow). Test with `--target scratch-unknown` to
  exercise retry/drop: retry 2/3 → DROPPED → file emptied (0 bytes).
- Sweep test technique: `touch -d "30 hours ago" events/<id>.md` to fake
  an idle event.
- Announced `delivery/failed` carries `publisher: agent-notify`.