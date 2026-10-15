# 14 — What environments must the factory actually run in?

**Type:** grilling
**Status:** closed
**Blocked by:** —
**Claimed by:** —

## Question

Which hosts do we claim to support, what does "supported" mean, and what does the factory do when
the host can't do what a cycle needs?

## Why this ticket exists

Victor, mid-charting:

> *"I understand this is a Windows/WSL environment, you should know this is supposed to work in
> **all environments**. Do not bottle this platform up to work only in this kinda environments."*

Then, clarifying:

> *"Well, this is Windows, but for ease AI uses WSL sometimes. I was referring to **the finished
> product**."*

So: **the dev box is incidental. The binary is the deliverable.** Without a defined support
matrix, every portability decision is guesswork — and we already made one wrong (06's
over-generalisation of a Windows failure).

---

## What's already decided (locked in the map)

| Rule | Consequence |
|---|---|
| Provider **detected by capability at runtime** | No config file telling the user what their host supports |
| **Default = lowest-prerequisite provider** (`noSandbox`) | Docker/Podman/Vercel are upgrades, not requirements |
| **No `bash` in the contract** | Rust owns every `process::Command` |
| Windows-native is the **worst case** | If it works there, the rest follows |

## What to decide

### 1. The support matrix

Which hosts do we claim? Fill in and commit:

| Host | git | agent CLI | container runtime | our provider | tier |
|---|---|---|---|---|---|
| Linux CI (GitHub Actions) | ✅ | ✅ | docker | docker | 1st |
| Linux server / VPS | ✅ | ✅ | docker / podman | docker / podman | 1st |
| macOS (dev) | ✅ | ✅ | Docker Desktop / Podman | docker | 1st |
| **Windows-native** | ✅ | ⚠️ `.cmd` | Docker Desktop (VM, slow) | docker | 1st |
| **WSL2** | ✅ | ✅ | **native, fast** | docker | 1st |
| Restricted / no container | ✅ | ✅ | ❌ | **noSandbox** | 2nd |
| Minimal container, no agent installed | ✅ | ⚠️ | ✅ | docker + runtime install | 3rd |

**Which tiers are in scope for v1?** Is "no container at all" a supported path or a degraded one?

### 2. What does "supported" mean?

- Do we **test** each host, or just not crash on it?
- Which matrix cell runs in CI, and which is documented-and-best-effort?
- Is there a `factory doctor` command that reports what the current host can do?

### 3. Capability detection

Not "is Docker installed" — **"can this host run what a cycle needs?"** What does the factory
probe, and how long is it allowed to spend finding out?

```
probe: git present?            → yes/no
probe: worktree creatable?     → try in tmp, clean up
probe: agent CLI on PATH?      → which one, what version
probe: container runtime?      → docker / podman / none
probe: runtime supports exec?  → can it stream stdout line-by-line?
```

**Rule:** detection must be fast and silent. A user running `factory` should not watch a 30-second
capability scan.

### 4. Degradation

When a cycle needs something the host can't do, what happens?

| Situation | Options |
|---|---|
| No container, but cycle asked for Docker | auto-fall-back to `noSandbox` · hard error · warn + continue |
| Agent CLI missing for the requested agent | error naming what to install · fall back to an available one |
| Browser probes impossible (no Playwright) | skip ①d UX · run the rest · fail the cycle |
| Load ceiling hit | report honestly, skip escalation (already locked) |

**Which failures are silent, which warn, which abort?** Silent degradation is how a factory
quietly tests nothing.

### 5. Explicit override

Auto-detection is the default, but a user must be able to force a provider. CLI flag, env var, or
`.factory/harness.yaml`? Which wins?

### 6. What is explicitly unsupported

Name it so it can be refused cleanly:

- Windows containers (not Linux containers) for our Playwright use case
- Sandcastle's `format-patch` sync (Vercel pushes real SHAs per ticket 03)
- Alpine/musl sandboxes — Playwright's WebKit is glibc-only

---

## Deliverable

A support matrix table, a capability-probe list, and a degradation table where every row has a
chosen behaviour. Plus the explicit non-goals.
## Resolution

### The reframe

**Two separate places where things run.** The ticket had been conflating them.

| | |
|---|---|
| **The factory** | runs anywhere, as a single binary. No Node, Python or Docker needed to run it |
| **A sandbox** | where the agent works. Depends entirely on where the factory landed |

Only the second is a matrix.

### The ladder

`
   WSL                    Windows' escape hatch - near-Linux, fast native containers
   Docker / Podman        the usual
   Vercel (cloud)         works from ANY computer, needs only network
   noSandbox              genuinely degraded
`

**The factory takes the best one it finds.** No computer lacks a real sandbox unless it has no WSL,
no Docker, **and** no network.

WSL is **offered, never installed unattended** - and honest about needing a reboot.

### Degraded means less isolation, not less capability

| On a bare Windows box | |
|---|---|
| Git worktree, agent, sessions, merge chain | all work |
| **Browser probes** | work - Playwright is a native Windows program |
| **Load probes** | work - k6 ships a Windows build |
| **Isolation** | **none** - the agent runs as you, with your keys |

> **Every machine can do everything. What changes is how much damage an agent could do if it went
> wrong.**

The container provides a wall; it is not what makes the browser work. The one genuine gap: results
are **runnable** cross-platform, not **reproducible** - a test that passes on Windows may fail on
Linux.

### When unprotected

- **It says and carries on**, and remembers the acceptance.
- **Carve-out: a machine it does not recognise asks.** Your acceptance on your laptop says nothing
  about that machine.

### The name: **GRAPHEINA**

`
  fabrica      the forge      makes the thing          <- Latin, from faber, a smith
  grapheina    the tablet     writes down what it
                               should do               <- Greek, from graphein, to write
`

**Factory is not Latin for a factory** - it descends from *fabrica*, which meant a **forge**. And
*graphein* gave us *program*, *programma* (a public notice, posted, never hidden), *graph*,
*progress*, *paragraph*.

Chosen over the coinage FABRICYCLUS: a coinage is judged by whether you like it; a real word
carries its own meaning.

> **Factory means assembly line. This is a forge.**