# PRD-TRD: Scaffolding — the factory's first working code

**Slug:** scaffolding
**Status:** Reconstructed 2026-10-15 (retroactive) — reviewed 2026-10-16, minor drift fixed
**Date:** 2026-10-15

## Why This Exists

Planning is complete: 24 wayfinder tickets, a full glossary, a map. Nothing has ever run. Every
decision so far has been a promise made on paper; the first working code is what turns the first
promises into facts. Per the planning handoff, the first code is the **Doctor skeleton** — the
registry, because every later part of the factory registers into it.

The machine was honestly unprepared (no cargo on PATH anywhere). Per ticket 21 the technical side
is confirmed before behaviour is enforced, so the scaffolding includes the dev lane decision
itself: development happens in WSL Debian, and the Windows host keeps cargo off its PATH (a
stale off-PATH rustup toolchain in the user profile plays no part; Windows-native builds are
rejected), so the day-one Finding from ticket 24 (missing cargo, printed fix, exit 2) stays real.

This pack is a **retroactive reconstruction**: the code was built first, the process was skipped,
and this document set records what the process would have required. The build session is
documented in `sessions/2026-10-15-doctor-skeleton.md`.

## Behavioral Spec

### Scenario S0: the bootstrap

**Given** a machine without Rust
**When** the dev lane is set up (Rust stable in WSL Debian, approved by the human)
**Then** `cargo --version` answers from the login shell, the repo remains a single copy, and the
crate exists with the locked shape: one crate `grapheina`, edition 2024, `[dependencies]` empty.

### Scenario S1: the project is healthy

**Given** the crate as built
**When** `cargo test` and `cargo build` run in the dev lane
**Then** all tests pass and `target/debug/grapheina` exists.

### Scenario S2: doctor on a prepared machine

**Given** git and cargo installed
**When** `grapheina doctor`
**Then** the report shows System then Code, `✓ git <version>`, `✓ cargo <version>`, and the exit
code is **0**.

### Scenario S3: a machine without Rust

**Given** git installed, cargo missing
**When** `grapheina doctor`
**Then** `✗ cargo not installed`, an indented fix line reading
`fix: install Rust (rustup) — outside the project, always asks first`, and the exit code is **2**.
Nothing is executed; the Doctor only recommends.

### Scenario S4: no programs at all (hermetic)

**Given** nothing on `PATH`
**When** `grapheina doctor`
**Then** both checks are `✗`, each with its bounded fix line, and the exit code is **2**.

### Scenario S5: usage errors

**Given** no command, or an unknown command
**When** `grapheina` / `grapheina dance`
**Then** `usage: grapheina doctor` is printed to stderr and the exit code is **64** — a usage
error, deliberately outside the Doctor's contract; never 1.

## Simulation Contract

```bash
# S0/S1 - the crate as built (run in WSL:  wsl -d Debian -- bash -lc 'bash docs/features/scaffolding/simulate.sh')
cargo build                                   # → target/debug/grapheina exists
awk '/^\[dependencies\]/{d=1;next} /^\[/{d=0} d&&/=/{print}' Cargo.toml
                                              # → no lines (zero dependencies)
cargo test -q                                 # → exit 0

# S2 - prepared machine
grapheina doctor                              # → contains "✓ git", "✓ cargo"; exit 0

# S3 - cargo hidden
env PATH=/usr/bin:/bin grapheina doctor       # → contains "✗ cargo", the fix line; exit 2

# S4 - hermetic
env PATH=/nonexistent grapheina doctor        # → contains "✗ git", "✗ cargo"; exit 2

# S5 - usage
grapheina                                     # → stderr has usage; exit 64
grapheina dance                               # → stderr has usage; exit 64
```

Exit code 4 (accepted degradation) and 6 (gave up) are **not** in the sim: no acceptance store
and no live gave-up path exist yet — see Non-Goals.

## Technical Design

### Data Models

- `Section` — `System`, `Code` (report order fixed).
- `Status` — `Fine | Degraded | Broken`.
- `BlastRadius` — `Read | InsideProject | OutsideProject` (ask/act boundary lives in the model).
- `Fix { action, touches }` — `OutsideProject` renders "outside the project, always asks first".
- `Outcome { status, detail, fix }` — one check's result.
- `Report { gave_up: Option<String>, results: Vec<CheckResult> }`.

### API Contracts

- `Check` trait — `id()`, `section()`, `run(&Environment) -> Outcome`.
- `Registry::register(Box<dyn Check>)`, `Registry::survey(&Environment) -> Report`.
- `Report::exit_code() -> u8` — 6 if gave up; 2 if any Broken; else 4 if any Degraded; else 0.
- `Report::render() -> String` — the operator-visible report.
- `CommandCheck` — probes a program's `--version`, extracts the version; `builtin()` seeds git
  and cargo.
- `Environment::from_probe(closure)` (hermetic tests) and `Environment::real()` (spawns
  `<program> --version`).

### Dependencies

**None.** std only, by decision (tickets 08/14; handoff). Nothing to audit, nothing to update,
no supply chain. The first dependency added later will need an opensrc analysis before it enters.

### Architecture Notes

```
src/lib.rs                   crate root (pub mod doctor)
src/main.rs                  CLI: doctor | usage (64)
src/doctor/types.rs          the model
src/doctor/check.rs          Check trait
src/doctor/checks.rs         CommandCheck + builtin() (git, cargo)
src/doctor/environment.rs    probe seam + version extraction
src/doctor/registry.rs       register / survey
src/doctor/report.rs         Report + exit_code()
src/doctor/render.rs         Report::render()
```

One job per module, small files (repo ethics). Tests are hermetic through the probe seam: the
not-installed cases pin `PATH` (`/nonexistent`). `tests/cli.rs` is the only place the real host
is touched — one deliberate smoke test asserts the prepared-machine report (exit 0) and fails if
git or cargo is missing.

## Non-Goals

- `--json` machine-readable report (ticket 24 Q5: same run, second rendering) — next phase.
- Acceptance store for Degraded / live exit 4 — model exists, store does not.
- Live gave-up path / exit 6 — model and rendering exist, nothing sets them at runtime yet.
- Worktree / container / agent checks (ticket 14 list) — arrive as their parts exist.
- Auto-fix, classifier, anything that acts — ticket 24 Q2: the Doctor recommends.
- Windows-native build (VS Build Tools) — rejected; cross-compile from WSL later.

## Out of Scope (Future)

- Sandcastle prep (first real Cycle target) — blocked: `factory/sandcastle` is not a git repo;
  Victor's decision needed.
- Full Doctor (all probes, JSON, acceptance) — own pack later.

## References

- `GRILL-scaffolding.txt` — ticket 21: scaffolding is a conversation; the output states runtime
  behaviour.
- `GRILL-scaffolding-bootstrap.txt` — the reconstructed decision record for this build.
- `docs/tickets/24-doctor.md` — the Doctor contract (authoritative behaviour source).
- `CONTEXT.md` — glossary.
- `IMPL-scaffolding.md` — execution plan (as-built + future).
- `sessions/2026-10-15-doctor-skeleton.md` — the build session.
