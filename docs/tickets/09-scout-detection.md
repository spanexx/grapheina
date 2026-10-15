# 09 — How does Scout detect what kind of app it is looking at?

**Type:** grilling
**Status:** closed
**Blocked by:** 01 (closed)
**Claimed by:** opencode

**Findings:** [../docs/features/scout-detection/GRILL-scout-detection.txt](../docs/features/scout-detection/GRILL-scout-detection.txt)

## Question

What signals does Scout read, how confident must it be before acting, and what does it write
down so it never has to ask twice?

## Resolution

The Scout needs four answers before any Probe can run — **what is this, how do I start it, how do
I reach it, how do I poke it.** Answering those four is the **Harness**.

### Five kinds

web page · service · command line program · library · worker

The smallest set that separates the Probes cleanly. Usability probing is only real for a web page;
security probing is thin for command line programs and libraries — named rather than hidden.

### One Detector per ecosystem

Not one big detector with a table of languages. Each is small, knows one world — its manifests,
its layout, its start commands — and is loaded as a plugin.

```
python   pyproject.toml · requirements.txt · manage.py · uvicorn / flask / django
rust     Cargo.toml · src/main.rs -> program · src/lib.rs -> library
```

`npm run dev`, `uvicorn app.main:app`, `cargo run`, `go run .` — **start commands do not translate
between languages.**

**This produced the answer to a question the ticket had not asked:** the Scout can name its own
**blind spots**. Given a Ruby project with no Ruby Detector, the true answer is *"no Detector for
Ruby — I know JS/TS, Python, Rust, Go"* — not a guess, and not silence.

### Unsure → ask once, remember

Stop, show what was read, offer the most likely guesses, the human chooses, the answer is written
down and never asked again. Guessing-and-labelling was rejected: **a guess that gets tested is
indistinguishable from a guess that got it right**, and a disclaimer nobody reads is not
transparency.

A Scout does not need the whole repo understood, only the app under test. It probes what it
understands and says what it skipped.

### Staleness

Each Detector records **which files it read**. The Scout hashes them; if they changed, that
Detector runs again. Cheap, and precise.

**Backstop:** if the boot fails, try once with fresh detection. If that fails too, **the boot
failure is the Finding** — because "my saved answer is out of date" and "your app does not start"
look identical from the outside and mean completely different things.

### Reporting

Every report says what the Scout did **and what it skipped**. A report listing only successes is
indistinguishable from one where everything failed quietly.

### v1 Detectors

**JavaScript/TypeScript · Python · Rust · Go**

Chosen by Victor after the ecosystem-plugin decision. Each is a small, self-contained Detector:

| Ecosystem | Decides from | Kind signal | Starts with |
|---|---|---|---|
| **JS/TS** | `package.json`, `next.config`, `vite.config` | `bin` field → command line; server framework → service; `app/`+`pages/` → web page | `npm run dev` |
| **Python** | `pyproject.toml`, `requirements.txt`, `manage.py`, `uvicorn`/`flask`/`django` in deps | `manage.py` → service; console entry point → command line; `src/<pkg>` only → library | `uvicorn app.main:app`, `flask run`, `manage.py runserver`, `python -m app` |
| **Rust** | `Cargo.toml`, `src/main.rs`, `src/lib.rs` | `main.rs` → program; `lib.rs` alone → library; `[[bin]]` names the binary | `cargo run` |
| **Go** | `go.mod`, `cmd/`, `main.go` | `cmd/<name>/main.go` → program; `package <name>` with no `main` → library | `go run .`, or `go build -o bin/<name> && ./bin/<name>` |

Rust and Go both answer the kind question from structure alone — `main.rs` versus `lib.rs`,
`cmd/` versus no `main` — before a line of code is read.

**Graceful extension:** a fifth Detector is a new file. The Scout reports what it has, so a
missing ecosystem names itself rather than guessing.

### Graduated

[16 — Harness contract](16-harness-contract.md) inherits these four answers as its starting
contract.

---

## Original notes

### Blocked by 01 because

The Finding format determines what evidence the Scout must collect while exploring. You cannot
design the detection if you do not yet know what detection is *for*. (A web UI finding needs a
screenshot; a CLI finding needs terminal output; a stress finding needs a machine spec.)

### Known failure cases to design around

- **Monorepo** — 6 packages, 2 libs, 1 CLI, 1 web app. Detection picks one; humans want another.
- **Polyglot** — genuinely two harnesses in one repo.
- **Obfuscated** — custom build, no recognizable markers. Must ask.

### Deliverable

A written detection algorithm with a confidence rubric, plus the on-disk shape of `harness.yaml`.
