# IMPL: Scaffolding — the factory's first working code

**Slug:** scaffolding
**Status:** Reconstructed 2026-10-15 (retroactive) — phases P1–P6 were executed before this
document existed; written from the session record and the diffs. Reviewed 2026-10-16 (minor
drift fixed). Future phases pending.
**Date:** 2026-10-15

## Phase Plan

**Before Phase 1:** no new external dependency is added by this pack (zero-dependency decision) —
no opensrc analysis required.

### Phase 1: dev lane — ✅ Complete 2026-10-15

**Build:**
- Rust stable in WSL Debian (`rustup`: cargo, rustc, rustfmt, clippy), PATH wired in `~/.profile`.
- One repo copy; WSL reaches it at `/mnt/c/...`.

**Verify:**
- [x] `wsl -d Debian -- bash -lc 'cargo --version'` → `cargo 1.99.1` (PRD-TRD S0)
- [x] No Windows cargo on PATH — a stale off-PATH rustup toolchain exists in the user profile;
      Windows-native builds rejected

**Blocked by:** nothing

### Phase 2: crate bootstrap — ✅ Complete 2026-10-15

**Build:**
- `cargo init --name grapheina`, edition 2024, `[dependencies]` empty (PRD-TRD Technical Design).
- `.gitignore` (`/target`).

**Verify:**
- [x] `cargo build` succeeds with zero dependencies (PRD-TRD S0/S1)
- [x] `Cargo.toml` has no dependency entries (simulated by `simulate.sh`)

**Blocked by:** Phase 1

### Phase 3: exit-code model — ✅ Complete 2026-10-15

**Build:**
- `src/doctor/types.rs` (`Section`, `Status`, `BlastRadius`, `Fix`, `Outcome`),
  `src/doctor/check.rs` (`Check` trait), `src/doctor/registry.rs` (`register` / `survey`),
  `src/doctor/report.rs` (`exit_code()`).

**Verify:**
- [x] `a_fine_check_reports_fine_and_exit_code_zero`, `a_broken_check_makes_exit_code_two`
- [x] `an_accepted_degradation_makes_exit_code_four`, `an_unaccepted_break_outweighs_an_accepted_degradation`
- [x] `a_given_up_doctor_makes_exit_code_six`

**Blocked by:** Phase 2

### Phase 4: rendering — ✅ Complete 2026-10-15

**Build:**
- `src/doctor/render.rs` — System → Code, `✓ / ⚠ / ✗`, version details, indented `fix:` lines
  with the blast-radius boundary, gave-up line.

**Verify:**
- [x] `a_report_renders_statuses_sections_and_the_fix_boundary` (golden string)
- [x] `a_given_up_report_renders_the_reason`

**Blocked by:** Phase 3

### Phase 5: checks and the probe seam — ✅ Complete 2026-10-15

**Build:**
- `src/doctor/checks.rs` (`CommandCheck`, `builtin()`: git, cargo),
  `src/doctor/environment.rs` (`Environment::from_probe` / `real()`, version extraction).

**Verify:**
- [x] `the_builtin_registry_checks_git_and_cargo_on_a_prepared_machine`
- [x] `a_missing_program_is_broken_with_a_system_fix`
- [x] 3 unit tests in `environment.rs` (version extraction)

**Blocked by:** Phase 3

### Phase 6: CLI — ✅ Complete 2026-10-15

**Build:**
- `src/main.rs` — `doctor` runs the real survey and exits with its code; every other path prints
  usage to stderr and exits 64 (PRD-TRD S5).

**Verify:**
- [x] `doctor_on_this_machine_prints_the_report_and_exits_zero`
- [x] `doctor_exits_two_when_a_check_is_broken` (pinned `PATH=/nonexistent`)
- [x] `no_command_prints_usage_and_exits_sixty_four`, `an_unknown_command_prints_usage_and_exits_sixty_four`
- [x] Full run: `cargo test` → 16 green; `cargo fmt --check` clean; clippy clean

**Blocked by:** Phase 4, Phase 5

### Phase 7: `--json` second rendering — ⏳ Pending

**Build:** machine-readable report (ticket 24 Q5 — same run, second rendering).
**Verify:** same run produces both renderings; golden JSON in tests.
**Blocked by:** nothing (next phase agreed).

### Phase 8: acceptance store (live exit 4) — ⏳ Pending

**Build:** persist `accept as-is` decisions; Degraded + accepted → 4 at runtime.
**Verify:** acceptance round-trip test; rejected acceptance → 2.
**Blocked by:** a decision on where acceptance lives (repo file vs user config).

### Phase 9: live gave-up path (exit 6) — ⏳ Pending

**Build:** wire a real failure (e.g. probe infrastructure cannot start) to `Report.gave_up`.
**Verify:** forced-failure test → exit 6, precedence over 2/4.
**Blocked by:** nothing hard; ranked below 7–8.

### Phase 10: further probes — ⏳ Pending

**Build:** worktree / container / agent probes (ticket 14 list) as their parts exist.
**Blocked by:** the parts they probe.

## Phase Dependencies

```
P1 → P2 → P3 → { P4, P5 } → P6 → [ P7, P8, P9, P10 ]
```

## Test Strategy

TDD, vertical slices — one behaviour per red-green-refactor cycle (fine→0, broken→2,
degraded→4, precedence 2>4, gave-up→6, rendering golden, builtin checks, version extraction,
CLI process-level). Unit/integration tests in `tests/` and inline `#[cfg(test)]`; run with
`cargo test` in WSL. Real-host tests only in `tests/cli.rs`: the not-installed cases pin `PATH`
(`/nonexistent`); the prepared-machine smoke test deliberately uses the real host.

## Dependency Analysis (opensrc)

Skipped — the pack adds no external dependency, by decision. Next dependency requires opensrc
analysis before it enters.

## Rollout

Not applicable — first code in a repo with no shipped product. Nothing to migrate; nothing to
flag. The binary is built and run from `target/debug/`.

## Risk Notes

- `wsl.exe` from PowerShell mangles `$?` — read exit codes with `&&` / `||`, or run scripts
  through `bash -lc`.
- POSIX default-path fallback: with `PATH` unset, `/usr/bin/git` is still found — hermetic
  tests must set `PATH=/nonexistent`, not unset it.
- Builds on `/mnt/c` are fast only because there are zero dependencies; revisit if the tree
  grows (consider a WSL-side clone later).
- Line endings: repo files are LF (written from WSL); keep it that way.
- Windows binary is deferred; the day-one Finding demo currently runs in WSL only.
- This host carries an off-PATH Windows rustup toolchain (cargo 1.98.1); if it ever lands on
  PATH, the day-one demo stops reproducing.
