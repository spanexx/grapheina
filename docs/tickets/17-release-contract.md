# 17 — The release contract: what every piece of software must provide

**Type:** grilling
**Status:** open
**Blocked by**: 02
**Claimed by**: —

## Question

What must a piece of software hand over so that deploying it is **automatic** — and what is the
one shape that is the same across every kind of software the factory builds?

## Why this is its own ticket, and why it is big

Ticket 02 established the branch chain: work lands on `main`, CI runs, a pull request to `master`
opens automatically, a human tests and approves. **But approval is not deployment.**

Without a release contract, the loop ends at "a human said yes" and every deployment becomes a
human remembering how they did it last time. That is the opposite of the factory.

> Victor: *"a factory that cannot ship what it builds is just a very organised way of generating
> code."*

## What is already locked (from ticket 02)

- `main` is the machine branch; `master` is human-controlled and holds the released version
- The factory opens `main → master` automatically once CI is green
- **The human's role is to install and test it where it belongs**, then approve
- **After approval, deployment is fully automatic**
- Deployment shape is **universal**; the implementation is per-software

## What to decide

### 1. What is the human's exact role?

Locked in spirit: install somewhere real, use it, approve. Still to settle:

- Does the human install it, or does the factory install it to a place the human then uses?
- Which environments? Victor mentioned Windows. Several machines? A physical device?
- What does "it works" mean concretely — a checklist, a scripted smoke test, or a judgement?

> My view: the factory **installs**, the human **uses**. Installing is mechanical and should not
> be a human task. The human's value is entirely in "does this actually work when I use it."

### 2. What is the universal shape?

Every software ships somehow. The shapes worth deciding between:

| Shape | Example | Automatable? |
|---|---|---|
| **Single file** | a script, a binary | ✅ trivial |
| **Archive** | a zip with a documented entry point | ✅ |
| **Package** | npm, PyPI, crates.io | ✅ registries |
| **Container** | a published image | ✅ |
| **Running service** | deployed somewhere reachable | ⚠️ needs a target |
| **Source only** | a repo | ⚠️ nothing to install |

**Is there one shape, or one shape per kind?** My view: one *contract*, several implementations.
Every software declares which shape it uses, and the factory has an automated path for each
declared shape. A zip download and a published container image are different mechanisms but the
same contract: **the software says how to obtain itself and how to prove it works.**

### 3. What must every software declare?

This is the heart of it. The minimum a factory needs to deploy and verify anything:

```
name          what it is called
shape         file | archive | package | container | service | source
build         how to produce it
obtain        where it lands once built
install       how to put it where it belongs
verify        how to prove it works
entry         the one command or URL a human should touch first
```

**Is that the right list?** What is missing that the factory would need?

### 4. Where does the declaration live?

The pack already exists — `docs/features/<slug>/`. Does the release declaration go in the pack,
or somewhere machine-readable alongside it?

My view: **alongside, in a single file** (`factory.yaml` or similar) that points at the pack for
the design and carries the mechanical facts. Packs are written for humans and are allowed to be
verbose. This file is read by a machine on every deploy, and should stay short enough that its
size is not a problem.

### 5. Who checks it?

Ticket 02 locked that rules are enforced by a reviewer, not remembered. This is another one:

- A release reviewer that refuses a merge to `master` if the declaration is missing or wrong?
- Or the deploy step simply fails, loudly, with a clear message?

My view: **the deploy step failing is the check.** A dedicated reviewer for this duplicates the
mechanism. But the failure message has to name exactly what is missing.

### 6. What about software that cannot be deployed automatically?

Some things genuinely cannot — hardware, a thing that needs a physical setup. What happens?

- Refuse to build it (honest, but limits what the factory can make)
- Build it and stop at "ready to install" (a manual finish line)
- Let it declare itself non-deployable and mark it

My view: **a declared non-deployed state, and the factory is honest about it** rather than
pretending. Some software simply ends at "here it is, a person finishes."

### 7. Rollback

If `master` holds a released version and the next one is bad — who notices, and what happens?

- A human notices (current model)
- A monitoring probe notices and the factory rolls back
- Rollback is one command, run by a human when they decide

My view: **v1 is one command, run by a human.** Automatic rollback needs a health signal we do
not have yet, and a wrong automatic rollback is worse than a slow one.

## Deliverable

The universal release contract, the declaration format and where it lives, the automatic paths
for each declared shape, the human's exact role, and what happens to software that cannot be
deployed automatically.