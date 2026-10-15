# 04 — Does Playwright work inside a Docker sandbox on Windows?

**Type:** research
**Status:** closed
**Blocked by:** —
**Claimed by:** research subagent (cancelled mid-run, completed inline)

**Findings:** [../research/04-playwright-in-sandbox.md](../research/04-playwright-in-sandbox.md)

## Question

Can the Scout team drive a real browser inside our Docker sandbox, on **Windows** Docker Desktop,
and if not, what is the cheapest way to make it work?

## Why this matters

The Scout is the most important role, and for any web UI its only real harness is a browser.
"Click, type, screenshot, assert" is not something an LLM can do from a shell — it needs
Chromium actually running.

The Scout's UX probe (does a human actually *succeed* at the task?) and crash probe for web
apps are both dead without this.

## The specific problem

Playwright's browsers need Linux system libraries that **do not exist on Windows hosts**:
`libnss3`, `libatk`, `libgbm`, `libasound2`, and a pile of X11/GTK deps.

Two possible worlds:

**A. We run the sandbox in Linux container mode.** Our worktree is a Windows path
(`C:\Users\...\worktrees\myapp`) bind-mounted into a Linux container. Docker Desktop translates
it via VirtioFS. Playwright's Linux deps are satisfied — but:
- Does Playwright's Chromium work under VirtioFS mount performance?
- Can we `docker run` a Linux image on this box at all (Windows containers vs Linux containers)?
- Extra disk for a second Playwright browser cache.

**B. We run Playwright on the Windows host**, pointing at the app in the container via
`http://host.docker.internal:PORT`. No Linux deps needed — Windows Chromium works natively.
The container just runs the app; the browser lives outside.

## What to establish

1. **Is this box Linux-container-capable?** `docker info` → `OSType`. Windows containers mode
   vs Linux containers mode is a Docker Desktop setting and it is *mutually exclusive*.
2. **If Linux mode:** does Playwright install cleanly in a `node:22-bookworm` image? What are the
   exact `apt-get` deps? What is the image size cost?
3. **If Windows mode only:** can a Windows container run Chromium? What is the Playwright
   Windows-container story — supported or a nightmare?
4. **Option B in detail** — host browser against `host.docker.internal`. Does the sandbox port
   publishing work? Who owns the browser lifecycle — the Rust binary, or a sidecar?
5. **Browser binary caching.** Playwright downloads ~400 MB. Per sandbox? Shared volume? Build
   time on every cold start is unacceptable.
6. **Screenshots/evidence** — where do they land, and how does the Builder/Reviewer see them?
7. **Concurrency.** How many browser contexts can one machine run before it dies? This feeds
   ticket 05's calibration numbers.

## Deliverable

A **yes/no on Linux containers working**, and if not, a recommended Option-B shape with the
concrete cost (image size, startup time, browser cache strategy).