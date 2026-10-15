# 06 — git worktrees on Windows + Docker Desktop

**Status:** RESOLVED (with a landmine) · 2026-10-13 · Hands-on experiment, Linux container mode

---

## EXECUTIVE VERDICT

# ✅ It works — with a two-step fix, and ⚠️ one of those steps corrupts the host.

The Builder can commit from inside a container on a Windows-host worktree. Proven end to end:
container-made commit visible on the host branch.

**But the obvious fix silently breaks the host-side worktree.** See [The landmine](#-the-landmine).

---

## The `.git` file — observed verbatim

```git
gitdir: C:/Users/shuga/AppData/Local/Temp/awtest/.git/worktrees/wt
```

A **Windows absolute path**. Inside a Linux container that path does not exist. Confirmed by
experiment, not assumed.

---

## Experiments

### T1 — worktree mounted alone

```powershell
docker run --rm --entrypoint sh -v "${T}\wt:/w" -w /w alpine/git -c "git status"
```
```
fatal: not a git repository: (null)
```

✅ Exactly as predicted. The worktree is inert without its `.git` metadata.

### T1b — 🔴 Docker rejects Windows paths as *container* paths

```powershell
docker run --rm -v "${T}\wt:/w" -v "${T}\.git:${T}/.git" ...
```
```
docker: Error response from daemon: invalid mode: \Users\shuga\AppData\Local\Temp/awtest
```

**Docker strips the `C:` and produces `\Users\...`, then rejects it.** You **cannot** satisfy the
`gitdir:` line as written — the container side of a `-v` must be a POSIX path. Sandcastle's
`resolveGitMounts` mounts both paths *as-is*, which cannot work on Windows.

### T2 — POSIX mount + rewritten `gitdir:`

```sh
sed -i 's|^gitdir:.*|gitdir: /repo-git/worktrees/wt|' /w/.git
git config --global --add safe.directory '*'
git status --short --branch && git log --oneline
```
```
gitdir: /repo-git/worktrees/wt
## feature-x
 M README.md
--- log ---
c331f10 init
```

✅ **Works.** Branch detected, log readable.

### T3 — commit from inside the container

```sh
echo x > new.txt && git add -A && git commit -qm 'made in container'
```
```
926cac5 made in container
c331f10 init
```

✅ **Works.** And on the host:

```powershell
git -C $T branch -v
  feature-x 926cac5 made in container
* main      c331f10 init
```

**The Builder's core loop is proven on this platform.**

---

## 🔴 The landmine

After T2/T3, run git against the worktree **on the host**:

```powershell
git -C "$T\wt" log --oneline
fatal: not a git repository: (NULL)
```

**The worktree is now broken on the host.** The `sed` wrote *through the bind mount* onto the real
host file, replacing the host-valid path with a container-only path. One sandbox run permanently
corrupts the worktree.

### The fix — set env vars, never rewrite the file

```sh
export GIT_DIR=/repo-git/worktrees/wt
export GIT_COMMON_DIR=/repo-git
git config --global --add safe.directory '*'
git status
```

No file mutation, nothing to restore, no trap. **This must be how our sandbox provider does it.**

If rewriting is ever unavoidable (e.g. a tool that hardcodes reading `.git`), wrap it:

```sh
cp /w/.git /tmp/gitfile.bak
trap 'cp /tmp/gitfile.bak /w/.git' EXIT     # always restore
sed -i 's|^gitdir:.*|gitdir: /repo-git/worktrees/wt|' /w/.git
```

---

## ⚠️ Second finding — line-ending noise

`git status` inside the container reported ` M README.md` — a file the agent never touched.
Cause: Windows `Out-File` wrote CRLF + BOM; Linux git reads it as modified.

**Every sandbox will see phantom modifications.** Mitigations to evaluate:

- Set `core.autocrlf=input` (or `false`) in the container's git config
- Match the host repo's existing `core.autocrlf` rather than imposing one
- Filter out `M` files that are byte-identical after normalisation

**Untested** — flagging as a known hazard, not a resolved issue.

---

## Does Sandcastle agree?

**No.** `SandboxFactory.ts:264` `resolveGitMounts` mounts the `.git` file and the parent `.git`
directory **at their host absolute paths**. Per T1b, **that cannot work on Windows** — Docker
rejects the container-side path outright. Their own test
`SandboxFactory-windowsMounts.test.ts` asserts the mount set without running a container, so it
passes while the real thing fails.

**Lesson: read their intent, verify their code.** We are not using Sandcastle, so this costs us
nothing — but it's exactly the bug we would have inherited by copying.

---

## What our Rust sandbox provider must implement

```rust
// Windows host → Linux container, worktree case
mounts.push((worktree_path,       "/workspace"));
mounts.push((repo_git_dir,        "/repo-git"));   // POSIX path, NEVER a Windows path

env.insert("GIT_DIR",            "/repo-git/worktrees/<name>");
env.insert("GIT_COMMON_DIR",     "/repo-git");
exec("git config --global --add safe.directory '*'");
```

| Rule | Why |
|---|---|
| Container-side mount paths are **always POSIX** | Docker rejects `C:\...` / `\Users\...` |
| **Never rewrite `.git`** | The rewrite writes through to the host and corrupts it |
| Always set `GIT_DIR` + `GIT_COMMON_DIR` | Environment beats mutation — nothing to restore |
| Always set `safe.directory` | Host uid ≠ container uid → "dubious ownership" |
| Propagate host `user.name`/`user.email` | Otherwise commits are attributed to `agent` |

---

## Still unknown

| | |
|---|---|
| **VirtioFS performance** | This test repo was ~2 files. Real repos with `node_modules` may be unusable bind-mounted. **Needs a real repo measurement.** |
| **File watching** | Does vitest / `next dev` see changes through the mount? |
| **Worktree locking** | ADR 0007 — two agents grabbing one path. Untested here. |
| **Line endings** | The `core.autocrlf` mitigation is proposed, not verified. |

These are performance/robustness items, not blockers. The Builder loop **works**.

---

## Reproduce

```powershell
$T="$env:TEMP\awtest"; New-Item -ItemType Directory $T | Out-Null; cd $T
git init -q -b main; "hello" | Out-File README.md; git add -A
git -c user.name=t -c user.email=t@e.co commit -qm init
git worktree add -q "$T\wt" -b feature-x

# FAILS
docker run --rm --entrypoint sh -v "${T}\wt:/w" -w /w alpine/git -c "git status"

# WORKS
docker run --rm --entrypoint sh -v "${T}\wt:/w" -v "${T}\.git:/repo-git" -w /w alpine/git `
  -c 'export GIT_DIR=/repo-git/worktrees/wt GIT_COMMON_DIR=/repo-git; git config --global --add safe.directory "*"; git status'
```