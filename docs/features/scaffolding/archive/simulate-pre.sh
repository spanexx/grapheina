#!/usr/bin/env bash
# scaffolding - design-time simulation (reconstructed 2026-10-15) - ARCHIVED 2026-10-16
#
# Mirrors the LOCKED DESIGN (GRILL-scaffolding.txt + bootstrap addendum, tickets 14 / 21 / 24)
# with hardcoded state - no code required. It was compared against the post-impl simulation
# during the drift check (.reports/2026-10-16-drift-scaffolding.md): no design drift found.
# The canonical sim is ../simulate.sh (runs the real binary); this file is kept for comparison.
#
#   bash docs/features/scaffolding/archive/simulate-pre.sh
set -u

say() { printf '%s\n' "$@"; }
scenario() { say ""; say "=== $1 ==="; say ""; }

say "scaffolding - design-time simulation (the design as locked, not the implementation)"

scenario "S0 - the bootstrap"
say "  step 1  the dev lane exists: Rust stable in WSL Debian (approved install)"
say "            \$ cargo --version"
say "            cargo 1.99.1"
say "  step 2  the crate exists, exactly one copy of the repo:"
say "            Cargo.toml  -> name grapheina, edition 2024"
say "            [dependencies] -> empty (std only)"

scenario "S1 - the project is healthy"
say "  \$ cargo test"
say "            -> all tests green"
say "  \$ cargo build"
say "            -> target/debug/grapheina exists"

scenario "S2 - doctor on a prepared machine (exit 0)"
say "  \$ grapheina doctor"
say ""
say "  grapheina doctor"
say ""
say "    System"
say "      ✓ git    2.47.3"
say "      ✓ cargo  1.99.1"
say ""
say "    Code"
say "      (no checks registered)"
say ""
say "  # exit code 0 - all fine"

scenario "S3 - a machine without Rust (exit 2)"
say "  \$ grapheina doctor"
say ""
say "  grapheina doctor"
say ""
say "    System"
say "      ✓ git    2.47.3"
say "      ✗ cargo  not installed"
say "          fix: install Rust (rustup) — outside the project, always asks first"
say ""
say "    Code"
say "      (no checks registered)"
say ""
say "  # exit code 2 - something is broken; the fix is PRINTED, never run"

scenario "S4 - no programs at all, hermetic (exit 2)"
say "  \$ grapheina doctor        (with nothing on PATH)"
say ""
say "  grapheina doctor"
say ""
say "    System"
say "      ✗ git    not installed"
say "          fix: install git — outside the project, always asks first"
say "      ✗ cargo  not installed"
say "          fix: install Rust (rustup) — outside the project, always asks first"
say ""
say "    Code"
say "      (no checks registered)"
say ""
say "  # exit code 2"

scenario "S5 - usage errors (exit 64)"
say "  \$ grapheina"
say "  grapheina: no command given"
say "  usage: grapheina doctor"
say ""
say "  \$ grapheina dance"
say "  grapheina: unknown command: dance"
say "  usage: grapheina doctor"
say ""
say "  # exit code 64 - a usage error, outside the 0|2|4|6 contract; never 1"

say ""
say "Design notes"
say "- Exit codes: 0 fine | 2 broken | 4 broken-and-accepted | 6 gave up early. Never 1."
say "- Codes 4 and 6 exist in the design; this build demonstrates 0, 2 and 64 only"
say "  (the acceptance store and the gave-up path are deferred - see PRD-TRD Non-Goals)."
say "- The Doctor recommends; nothing it prints is ever executed."
say "- Section order is fixed: System, then Code."
exit 0
