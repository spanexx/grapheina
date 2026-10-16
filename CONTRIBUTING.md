# Contributing to GRAPHEINA

GRAPHEINA is a forge, and the rules here are gates rather than suggestions. They are written down in
[`docs/ethics/`](docs/ethics/) so that **any agent** can be held to the same standard as any human.
Read [`CONTEXT.md`](CONTEXT.md) first — it holds the language this project speaks.

---

## There are only two ways in

Every piece of code arrives through one of these. There is no third way, and "just make this change"
is not one of them.

| | Use it for | The shape of the work |
|---|---|---|
| **`feature-pipeline`** | New capability | GRILL → pre-implementation simulation → PRD-TRD → IMPL → build → post-implementation simulation → drift check → reconcile |
| **`surgical-change`** | Closing a gap between a decision already locked and the code that exists | Driven by a drift entry (`D-N`) or a bug, in the smallest surface, with the feature's docs updated in the same change |

**The test:** is there a locked decision behind this?

- **Yes** → surgical change. You are closing a gap.
- **No** → feature pipeline. There is nothing to be surgical about.
- **Cannot tell** → it is a feature. Size never picks the process; small means *surgical*, not
  *nothing*.

Full rule: [`docs/ethics/origin.md`](docs/ethics/origin.md).

---

## Before you commit

The gate is [`docs/ethics/commit.md`](docs/ethics/commit.md). In short:

- Tests green. Say how many.
- Diff read line by line before it is staged, not after.
- No secrets, ever. Credentials are **forbidden outright**, not scanned for — a test that needs to
  sign in reads from the environment, so there is nothing in the file to leak.
- No CRLF. A diff that rewrites every line for no reason is line-ending noise, not a change.
- A human says yes before anything lands.

---

## Comments are not optional

Comments are the handover between agents. The next one may be you, with no memory of this session.

- **Every line of logic** gets a comment beside it, no matter how small.
- **Every function, class and enum** gets a comment above it.
- A comment says **why**, not what. The code already says what.
- A comment names the **pack or drift entry** it belongs to.
- Plain English. A term the reader does not already have gets explained.
- Never restate the code. Never comment code out.

Full rule: [`docs/ethics/commenting.md`](docs/ethics/commenting.md).

---

## When code and decisions disagree, write it down

Do not quietly fix it and do not quietly edit the doc.

Record it in [`docs/drift.md`](docs/drift.md) with **both** a document reference (`path:line`) and a
code reference (`path:line`). No citations, no entry. Nothing is deleted; a drifted pack is appended
to. A decision lives in exactly one place — its ticket.

---

## What "done" means here

- One job per module. Small focused files. Clear boundaries.
- Tests are real tests. Regression and edge cases, not mocks of the thing under test.
- The Doctor **reports and recommends. It never acts.** If your change makes it install something,
  that is a bug in the change.
- No `bash` in the contract. Rust owns every process it starts. Scripts are optional convenience,
  never the interface.
- The factory runs anywhere. Windows-native is the worst case. If your change only works on this
  machine, it is not finished.

---

## Reporting problems

Bugs and proposals: [GitHub issues](https://github.com/spanexx/grapheina/issues).
Security: see [`SECURITY.md`](SECURITY.md).
Behaviour that falls short of this file: [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).