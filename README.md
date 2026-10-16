# GRAPHEINA

A self-running software factory. One Rust binary that drives a closed loop: it uses your
application, finds real problems, fixes them on a worktree, and has a **separate** reviewer verify
the fix before anything is allowed to land.

> *grapheina*, the tablet, from *graphein* — to write. With *fabrica*, the forge.
> Factory is a coinage. **This is a forge.**

---

## Status — read this first

**Pre-alpha. Decisions ahead of code.**

- **23 of 24** design tickets are closed. The one open ticket needs a real measurement, not another
  opinion. See the [map](docs/WAYFINDER.md).
- The only code so far is the **Doctor** skeleton — a survey of the machine the factory runs on.
  **16 tests green.**
- Nothing here runs a full Cycle yet. The loop is designed and locked, not built.

Do not expect a working factory from this repository yet. Expect a documented one.

---

## The loop

```
  SCOUT      use the app, run Probes, write a Repro
     |
     |   Repro passed twice? -> GHOST, dropped. Most output dies here.
     v
  WRITER     Finding -> a written Issue
     |
     v
  BUILDER    worktree + sandbox -> make the Repro pass
     |
     v
  REVIEWER   1. replay the Repro   <- the Interlock, the thing that says no
            2. then read the diff
     |   reject -> back to BUILDER
     v
  MERGER     land it on main
```

The Reviewer is a different model on purpose. Same model, different context, inherits the Builder's
blind spots — which is exactly the failure the Reviewer exists to prevent.

A **Finding** is not a guess. It exists only when a script re-enacts a real person's scenario
against the real application and fails twice in a row. Anything else is a **Ghost**, dropped without
ceremony.

---

## Running it

Development happens in **WSL Debian**. Windows is for reading, not building.

```bash
# build and test
cargo test

# run the Doctor — exit 0|2|4|6, never 1
./target/debug/grapheina doctor
```

The Doctor **reports and recommends. It never acts.** It will tell you a tool is missing and what to
install; it will not install it. Its blast radius decides what needs asking, not certainty.

---

## How this repository holds itself to account

Most projects have conventions. These are gates, and they are written down in `docs/ethics/` so
that **any agent** can be held to them the same way.

| Rule | Where | What it means |
|---|---|---|
| **Every piece of code has an origin** | [`origin.md`](docs/ethics/origin.md) | Code arrives through the feature pipeline or a surgical change. There is no third way in, and no "just make this change". |
| **Commenting is not optional** | [`commenting.md`](docs/ethics/commenting.md) | Every line carries a comment saying **why**, and naming the pack it came from. Comments are the handover between agents. |
| **Modular by construction** | [`modular.md`](docs/ethics/modular.md) | One job per module. Small focused files. Clear boundaries. |
| **Implementation ethics** | [`implementation.md`](docs/ethics/implementation.md) | Test-driven, real tests, regression coverage. |
| **Before you commit** | [`commit.md`](docs/ethics/commit.md) | Tests green, diff read line by line, no secrets, no CRLF, and Victor's explicit go. |

When code and the written decisions disagree, that gap is not silently fixed. It is recorded in
[`docs/drift.md`](docs/drift.md) with **both** a document reference and a code reference — no
citations, no entry.

---

## Layout

```
CONTEXT.md            the ubiquitous language. read it before anything else.
docs/WAYFINDER.md     the map — destination, decisions, open questions, out of scope
docs/tickets/         one file per decision, the way it was settled, and its resolution
docs/features/        the implementation packs: GRILL -> PRD-TRD -> IMPL
docs/research/        the five findings behind the risky decisions
docs/ethics/          the gates above
docs/drift.md         where decisions and code disagree, with citations
sessions/             handoffs between agents. how the last session ended, and why
src/doctor/           the only code: the survey of this machine
```

A decision lives in exactly one place — its ticket. The map is an index, never a store.

---

## Two rules that shape everything

**The product is the deliverable, not this machine.** The factory must run anywhere: one binary, no
Node, no Python, no Docker needed to run *it*. Only the sandbox varies, and it takes the best rung it
finds — WSL, then Docker/Podman, then Vercel, then the bare host. Degraded means *less isolation*,
never less capability.

**No `bash` in the contract.** Rust owns every process it starts. Scripts are optional convenience,
never the interface.

---

## Reference implementation

`sandcastle/` (from `mattpocock/sandcastle`) is a **reference, not a dependency**. Its architecture
decision records are a risk register: each one is a bug someone already paid for. It is also the
honest first Cycle target — the only codebase here whose Harness is already understood.

---

## Out of scope

Load-testing production. A GUI — everything here is a terminal tool. Distributed browser fleets.
Rewriting Sandcastle. Migrating existing projects.