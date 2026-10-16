# Drift Log
**Last updated:** 2026-10-16  **Open:** 1  **Resolved:** 0  **Critical/High:** 0

## Open

- **D-1** (Medium, 2026-10-16, reporter: opencode) — The Doctor skeleton violates the commenting ethics on every line: 0 comments in 578 lines.
  - Doc claim: "**DO NOT LEAVE ANYTHING UNCOMMENTED** - not one line, not one small helper." (`docs/ethics/commenting.md:27`)
  - Doc claim: "**Above the thing** - every function, class, enum and enum variant." (`docs/ethics/commenting.md:9`)
  - Code reality: `pub enum Section {` carries no comment above it and no comment beside it (`src/doctor/types.rs:2`)
  - Code reality: all 12 files under `src/` and `tests/` total 578 lines and 0 comment lines; 63 declarations, 0 preceded by a comment
  - Why matters: comments are the agent-to-agent handover. Code with no trail cannot be justified, audited, or safely edited by the next agent. This is also a direct breach of `docs/ethics/origin.md` — the code cannot name the pack it came from.
  - Owner: `scaffolding` (retrofit, comments only — no behaviour changes)
  - To fix: fix code. Add WHY/PACK/edit-log comments per `docs/ethics/commenting.md`. Behaviour must not change; the 16 tests must stay green untouched.
  - Related: none

---

## Resolved