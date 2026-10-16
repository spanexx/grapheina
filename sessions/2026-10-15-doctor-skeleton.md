# Session — 2026-10-15 — Doctor skeleton built (first code)

**Repo:** `C:\Users\shuga\Desktop\shared\factory\GRAPHEINA`
**Outcome:** first code landed. Doctor skeleton implemented via TDD, 16 tests green, demoed on
this machine (prepared → exit 0; cargo hidden → honest ✗ + exit 2). Nothing committed yet.

---

## 1. Dev lane: WSL Debian (decided with Victor)

Windows-native Rust needs VS Build Tools (multi-GB). WSL Debian had none of that weight, is the
first rung of the sandbox ladder, and Linux is the safest distribution target. Victor approved.

- `rustup` stable **1.99.1** installed (`cargo`, `rustc`, `rustfmt`, `clippy`), PATH wired in `~/.profile`.
- The repo stays in **one place** — `C:\Users\shuga\Desktop\shared\factory\GRAPHEINA`
  (WSL reaches it at `/mnt/c/...`; builds are fast because the crate has zero dependencies).
- **Windows host deliberately stays Rust-free** — when a Windows binary is eventually built
  (cross-compiled from WSL), its first run on this host will honestly find missing cargo. The
  day-one Finding from ticket 24 keeps its test case.

## 2. What was built (ticket 24 → code)

`cargo init --name grapheina`, edition 2024, **no dependencies** (std only), single binary.

- **Registry, not a list** — `Check` trait (`id` / `section` / `run(env)`), `Registry::register`;
  `builtin()` seeds the two real checks: **git** and **cargo** (both System section).
- **Exit codes** — `0` fine | `2` broken | `4` accepted-as-degraded | `6` gave up (takes
  precedence); never `1`. Usage errors exit `64` (outside the Doctor's contract on purpose).
- **Ask/act boundary in the model** — `Fix { action, touches: BlastRadius }`; `OutsideProject`
  renders as "outside the project, always asks first". Nothing executes fixes; the Doctor
  recommends (ticket 24 Q2).
- **Rendering** — `grapheina doctor` prints System then Code, `✓ / ⚠ / ✗`, version details,
  indented `fix:` lines. Matches the ticket 24 example shape.
- **Probe seam** — `Environment` (fake closure for tests, `real()` spawns `<program> --version`
  and extracts the first digit-leading token as the version).

```
src/lib.rs                   crate root: pub mod doctor (integration tests import it)
src/main.rs                  CLI: doctor | usage (64)
src/doctor/types.rs          Section, Status, Fix, BlastRadius, Outcome
src/doctor/check.rs          Check trait
src/doctor/checks.rs         CommandCheck + builtin() (git, cargo)
src/doctor/environment.rs    probe seam + version extraction (+3 unit tests)
src/doctor/registry.rs       Registry::register / survey
src/doctor/report.rs         Report + exit_code()
src/doctor/render.rs         Report::render()
tests/doctor_survey.rs       9 tests (registry, exit codes, rendering, builtin checks)
tests/cli.rs                 4 tests (process-level: exit 0 / 2 / 64, usage text)
```

## 3. TDD trail (vertical slices, one behavior per cycle)

fine→0 · broken→2 · degraded→4 · precedence 2>4 · gave-up→6 · rendering golden · builtin checks
(fake env) · version extraction · CLI process-level (exit 0 on prepared machine, 2 when broken,
64 for usage). Refactor pass afterwards: `exit_code` collapsed from three scans to one loop.

## 4. Demo runs (the design testing itself)

- Prepared machine (WSL, Rust installed): `✓ git 2.47.3`, `✓ cargo 1.99.1`, **exit 0**.
- Cargo hidden (`env PATH=/usr/bin:/bin`): `✗ cargo — not installed`, one bounded fix line
  ("install Rust (rustup) — outside the project, always asks first"), **exit 2**. The day-one
  Finding works, honestly.

## 5. Observations worth keeping

- Through `wsl.exe` from PowerShell, `$?` gets mangled (even `false` reported 0) — use
  `&& echo OK || echo BAD` to read exit codes.
- POSIX default-path fallback: with PATH unset, `/usr/bin/git` is still found (`env -i` is not
  "no programs"). The hermetic process-level test uses `PATH=/nonexistent`.
- `std::process::Command` with `--version` is enough to answer "Installed? Which version?" —
  no shell needed (ticket 08 holds).

## 6. Deliberately not built yet

- `--json` machine-readable report (ticket 24 Q5 — same run, second rendering). Next.
- Acceptance cache for Degraded (`accept as-is` → exit 4). Status + exit code exist; the store does not.
- Worktree / container / agent probes (ticket 14 list) — will arrive as their parts exist.
- Windows binary (mingw cross-compile from WSL) — later.

## 7. Blocker found for the NEXT milestone

**Sandcastle is not a git repo.** `factory/sandcastle` has no `.git` — it is a copy, not a clone,
so the factory cannot branch/worktree/commit against it. Victor's decision needed: `git init`
there, or clone from `mattpocock/sandcastle`. Handoff step 4 (first real Cycle target) depends
on it.

## 8. Follow-up (2026-10-16) — the skipped process, reconstructed

Victor flagged that the pipeline was skipped for this scaffolding. Reconstructed in place (no
worktree): `docs/features/scaffolding/` gained `GRILL-scaffolding-bootstrap.txt`,
`PRD-TRD-scaffolding.md`, `IMPL-scaffolding.md`, `simulate-pre.sh` (now archived in
`archive/`) and the canonical `simulate.sh`. The `docs/features/doctor/` folder stays empty —
reserved until the full Doctor is built through the pipeline.

Drift check by a fresh sub-agent: `.reports/2026-10-16-drift-scaffolding.md` — verdict **Minor
Drift**: 4 wording-level gaps (missing `src/lib.rs` in the tree; overstated test hermeticity;
"Windows Rust-free" wrong — an off-PATH rustup toolchain exists; sim merged stdout+stderr).
All four fixed the same day, no code changes needed. Sim: 21/21 green.
