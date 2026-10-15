# 04 — Does Playwright work inside a Docker sandbox on Windows?

**Status:** RESOLVED · 2026-10-13 · Sources: `playwright.dev/docs/docker`, local `docker version`

---

## EXECUTIVE VERDICT

# ✅ YES — and via Linux containers, which this host is already in.

No workaround needed. Docker here reports **`Server.Os = linux`, version `29.7.2`** — Linux
container mode. Playwright's Linux dependencies are satisfiable. The original ticket's worst
case (Option B, browser-on-host) is **not required**.

**The one real trap is version pinning**, not the platform. See "The trap" below.

---

## 1. Observed host state

```
$ docker version --format '{{.Server.Os}} {{.Server.Version}}'
linux 29.7.2
```

| Fact | Value |
|---|---|
| Container OS | **linux** ✅ |
| Engine | Docker Desktop 29.7.2 |
| Implication | Playwright image (`noble`/`jammy`/`resolute`) pulls and runs natively |

---

## 2. Two ways to get a browser

### Option 1 (recommended) — bake it into our own image

```dockerfile
FROM node:22-bookworm            # matches Sandcastle's base
RUN npx -y playwright@1.64.0 install --with-deps
```

`--with-deps` installs the system libraries (`libnss3`, `libatk`, `libgbm`, `libasound2`, X11/GTK
deps) **at build time**. Nothing downloaded at sandbox boot. One image serves agent + app + browser.

### Option 2 — Microsoft's prebuilt image

```bash
docker pull mcr.microsoft.com/playwright:v1.64.0-noble
```

⚠️ **It does not contain the Playwright package** — only browsers + system deps. You still
`npm i -D @playwright/test`. Convenient for a quick spike; Option 1 is better for shipping.

Available tags: `-noble` (24.04), `-jammy` (22.04), `-resolute` (26.04).
**Alpine is not supported** — Firefox/WebKit are glibc-only, musl breaks them.

---

## 3. 🔴 The trap: version pinning

From the docs, verbatim:

> *"If the Playwright version in your Docker image doesn't match the version in your project/tests,
> Playwright will be unable to locate browser executables."*

**This will bite us.** The scenario:

```
Image built weeks ago:  browsers for playwright 1.64.0
Repo's package.json:    "@playwright/test": "1.71.2"     ← bumped by the agent
Agent installs deps  →  gets 1.71.2
Agent runs playwright →  cannot find the 1.64.0 browsers → fails
```

**Options:** pin `@playwright/test` exactly (no `^`) in every repo we test, or run
`npx playwright install` at sandbox boot (slow), or rebuild the image on version drift. **Ticket
for later** — this is not a v1 blocker if we pin.

---

## 4. Required container flags

Playwright's docs call out three:

| Flag | Why | Needed? |
|---|---|---|
| `--init` | Avoids zombie-process special-casing for PID 1. *"A common reason for zombie processes."* | ✅ **always** |
| `--ipc=host` | *"Without it, Chromium can run out of memory and crash."* | ✅ **always** for any browser probe |
| `--cap-add=SYS_ADMIN` | Workaround for weird Chromium launch errors, local dev only | ⚠️ only if we see them |

**`--ipc=host` must become a first-class option in our sandbox provider.** It is not optional
for browser work, and our provider currently knows nothing about it.

Other relevant flags from the docs:

- `--user pwuser` + a seccomp profile, **only for untrusted sites**. Our apps are trusted, so
  root is acceptable — Playwright themselves say so.
- `--add-host=hostmachine:host-gateway` — makes the host's `localhost` reachable from inside the
  container. Replaces the Windows-specific `host.docker.internal`.
- `--workdir` — we mount the worktree here anyway.

---

## 5. Remote-server option (probably not for us)

Playwright can run a **browser server** in Docker with tests on the host:

```bash
docker run -p 3000:3000 --rm --init --user pwuser \
  mcr.microsoft.com/playwright:v1.64.0-noble \
  /bin/sh -c "npx -y playwright@1.64.0 run-server --port 3000 --host 0.0.0.0"
```
```bash
PW_TEST_CONNECT_WS_ENDPOINT=ws://127.0.0.1:3000/ npx playwright test
```

**We don't want this** — our whole model is "everything happens inside the sandbox." Keeping the
test runner co-located with the browser is simpler. Noting it because it is the escape hatch if
browser memory becomes a bottleneck.

---

## 6. Browser caching — settled

Not a problem, once we use Option 1: browsers are baked into the **image layer**. Every sandbox
from that image already has them. **No volume, no per-sandbox download.** This is a genuine
argument for the container over `noSandbox()` for browser probes.

---

## 7. Concurrency — unresolved, feeds ticket 05

Not measured. We know `--ipc=host` exists *because* Chromium OOMs without it, which implies
**shared memory is the first bottleneck**. The realistic concurrent-context count on this machine
is unknown and belongs to [05-k6-local-capacity](05-k6-local-capacity.md).

---

## 8. What this changes elsewhere

| Ticket | Impact |
|---|---|
| **06** Windows worktree + Docker | Our provider needs `--ipc=host` and `--init` on Windows too; `hostmachine:host-gateway` vs `host.docker.internal` |
| **07** Multi-agent image | One fat image (Option A) becomes more attractive — browsers are already a big layer, so add agents to it |
| **10** Probe matrix | UX Reader is now feasible on this platform ✅ |

---

## Summary

| Question | Answer |
|---|---|
| Linux containers work here? | **YES** — `docker version` → `linux 29.7.2` |
| Playwright deps satisfiable? | **YES** — `npx playwright install --with-deps` at build |
| Need host-browser workaround? | **NO** |
| Image caching solved? | **YES** — bake into the image layer |
| Biggest trap | **Version pinning** between baked browsers and `@playwright/test` |
| Flags we must add | `--init`, `--ipc=host` (mandatory), `--cap-add=SYS_ADMIN` (only if needed) |
| Unknown | Concurrent browser contexts per machine → ticket 05 |