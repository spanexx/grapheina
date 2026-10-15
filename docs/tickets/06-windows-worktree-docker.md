# 06 — What breaks with git worktrees on Windows + Docker Desktop?

**Type:** research
**Status:** closed
**Blocked by:** —
**Claimed by:** research subagent (cancelled mid-run, completed inline)

**Findings:** [../research/06-windows-worktree-docker.md](../research/06-windows-worktree-docker.md)

## Question

What specifically fails when a Windows-host git worktree is bind-mounted into a Docker container?
Get a working mount set, or find the workaround.

## Why this matters

The Builder runs in a container, on a worktree, and commits. If the mount is wrong, **every
git operation in the sandbox fails** — and `node_modules` alone can turn a 2-second worktree
into a 3-minute one.

This is not speculative. Sandcastle spent **ADR 0006** on exactly this.

## The core problem

In a normal checkout, `.git` is a **directory** — mount it, done.

In a **worktree**, `.git` is a **one-line file**:

```
gitdir: C:/Users/shuga/myapp/.git/worktrees/feature-login
```

So mounting the worktree gives the container a `.git` file pointing at a path **that does not
exist inside the container**. Every git command then fails.

Sandcastle's fix (`src/SandboxFactory.ts:264` `resolveGitMounts`): detect the file, parse the
`gitdir:`, and mount **both** the `.git` file *and* the parent `.git` directory.

## Specifics to establish

1. **Does Sandcastle's two-mount fix work on this host?** Reproduce it. Report the exact mount
   set that works.
2. **Path translation.** The `gitdir:` path inside the file is a **Windows** path
   (`C:/Users/...`). Inside a Linux container that is meaningless. Does the mount at the same
   absolute path fix it, or does the file need rewriting?
3. **VirtioFS performance.** Docker Desktop's file sharing. Measure: `npm install` in the
   worktree, `npm test`, a `git status` over a large repo. Compare against a copy inside the
   container filesystem. **Is bind-mounting the worktree even worth it vs just copying?**
4. **File watching / inode churn.** Do watchers (vitest, next dev, nodemon) work through the
   mount? Windows file-watching is notoriously bad.
5. **Line endings and permissions.** Does `core.autocrlf` in the host repo cause spurious diffs
   inside the container? Does `safe.directory` need setting (Sandcastle does this at
   `SandboxLifecycle.ts:236`)?
6. **The `node_modules` question.** Bind-mounted (slow), copied in (fast but GBs), or symlinked
   to the main repo (instant, but `npm install` writes back to the main repo). We said this
   doesn't matter *for correctness* because worktrees are throwaway — but it matters for
   *speed*, and speed is the Scout's whole game.
7. **Worktree locking.** Sandcastle has ADR 0007 — two agents grabbing the same path. Does that
   reproduce here?

## Deliverable

A **known-good mount list** for Windows host → Linux container → git worktree, with measured
timings, and a clear statement of whether bind-mount beats copy-in on this platform.

## Already known

- Sandcastle mounts at `/home/agent/workspace` (`SANDBOX_REPO_DIR`).
- Its Windows mount test files exist: `SandboxFactory-windowsMounts.test.ts`,
  `WorktreeManager.windowsPath.test.ts`, `createWorktree-windowsMounts.test.ts`,
  `interactive-windowsMounts.test.ts`. Read them before experimenting.
- `git worktree list --porcelain` reports **forward slashes on Windows** (`WorktreeManager.ts:114`).