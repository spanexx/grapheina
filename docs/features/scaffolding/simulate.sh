#!/usr/bin/env bash
# scaffolding - post-impl simulation: runs the REAL binary and asserts the contract.
#
# Run in the dev lane (WSL Debian):
#   wsl -d Debian -- bash -lc 'bash docs/features/scaffolding/simulate.sh'
#
# Each scenario maps to PRD-TRD-scaffolding.md Behavioral Spec S0-S5. Exits 0 when every
# scenario matches the design, 1 otherwise.
set -u
export PATH="$HOME/.cargo/bin:$PATH"

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BIN="$ROOT/target/debug/grapheina"
TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

pass=0
fail=0
check() {
  if [ "$1" -eq 0 ]; then
    pass=$((pass + 1))
    printf '  ok    %s\n' "$2"
  else
    fail=$((fail + 1))
    printf '  FAIL  %s\n' "$2"
  fi
}
contains() { printf '%s' "$1" | grep -Fq -- "$2"; }
scenario() { printf '\n=== %s ===\n\n' "$1"; }

cd "$ROOT"
printf 'scaffolding - post-impl simulation (the real binary in %s)\n' "$ROOT"

scenario "S0 - the bootstrap"
command -v cargo >/dev/null 2>&1; check $? "cargo present in the dev lane"
test -f Cargo.toml; check $? "Cargo.toml exists"
deps="$(awk '/^\[dependencies\]/{d=1;next} /^\[/{d=0} d&&/=/{print}' Cargo.toml)"
[ -z "$deps" ]; check $? "zero dependencies in Cargo.toml"

scenario "S1 - the project is healthy"
test_out="$(cargo test -q 2>&1)"; code=$?
[ "$code" -eq 0 ]; check $? "cargo test green"
if [ "$code" -ne 0 ]; then printf '%s\n' "$test_out" | tail -n 15; fi
test -x "$BIN"; check $? "binary built at target/debug/grapheina"

scenario "S2 - doctor on a prepared machine (exit 0)"
out="$("$BIN" doctor 2>&1)"; code=$?
[ "$code" -eq 0 ]; check $? "exit code is 0 (got $code)"
contains "$out" "✓ git"; check $? "report shows ✓ git"
contains "$out" "✓ cargo"; check $? "report shows ✓ cargo"

scenario "S3 - a machine without Rust (exit 2)"
out="$(env PATH=/usr/bin:/bin "$BIN" doctor 2>&1)"; code=$?
[ "$code" -eq 2 ]; check $? "exit code is 2 (got $code)"
contains "$out" "✗ cargo"; check $? "report shows ✗ cargo"
contains "$out" "fix: install Rust (rustup)"; check $? "fix line names the bounded action"
contains "$out" "outside the project, always asks first"; check $? "fix carries its blast radius"

scenario "S4 - no programs at all, hermetic (exit 2)"
out="$(env PATH=/nonexistent "$BIN" doctor 2>&1)"; code=$?
[ "$code" -eq 2 ]; check $? "exit code is 2 (got $code)"
contains "$out" "✗ git"; check $? "report shows ✗ git"
contains "$out" "✗ cargo"; check $? "report shows ✗ cargo"
contains "$out" "fix: install git"; check $? "git fix line present"
contains "$out" "fix: install Rust (rustup)"; check $? "cargo fix line present"

scenario "S5 - usage errors (exit 64)"
out="$("$BIN" 2>"$TMP")"; code=$?
[ "$code" -eq 64 ]; check $? "no command -> exit 64 (got $code)"
err="$(cat "$TMP")"
contains "$err" "usage: grapheina doctor"; check $? "usage printed to stderr"
[ -z "$out" ]; check $? "nothing printed to stdout"

out="$("$BIN" dance 2>"$TMP")"; code=$?
[ "$code" -eq 64 ]; check $? "unknown command -> exit 64 (got $code)"
err="$(cat "$TMP")"
contains "$err" "dance"; check $? "the unknown command is named on stderr"
[ -z "$out" ]; check $? "nothing printed to stdout"

scenario "summary"
printf '  %d passed, %d failed\n' "$pass" "$fail"
if [ "$fail" -eq 0 ]; then
  printf '\nall scenarios passed - the build matches the design\n'
  exit 0
else
  printf '\nscenarios FAILED - the build does not match the design\n'
  exit 1
fi
