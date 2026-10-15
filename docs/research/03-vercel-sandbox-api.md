# 03 — Can a Vercel Sandbox host a long-running agent?

**Ticket:** `tickets/03-vercel-sandbox-api.md`
**Research date:** 2026-10-13
**Primary sources:** Vercel REST API OpenAPI spec (`https://vercel.com/openapi.json`, 11 MB, retrieved 2026-10-13) and the `@vercel/sandbox` SDK source at `github.com/vercel/sandbox` (main branch). Docs pages are dated 2026-08-11 → 2026-10-13.

---

## EXECUTIVE VERDICT

# ✅ YES — Vercel is viable as a Rust-callable sandbox for a long-running coding agent.

**Streaming exec works. It is not buffered.** The API returns `application/x-ndjson` and the SDK consumes it with an incremental `ReadableStream` reader that yields each JSON record the instant it arrives. This is the single question the ticket said would decide viability, and it is a clean yes.

**We do not need the Node SDK.** Every operation Sandcastle needs — create, exec (streaming), read file, write files, mkdir, kill, stdin, stop, delete, extend timeout — is a plain HTTPS call with `Authorization: Bearer <token>`. A Rust client with `reqwest` needs a streaming NDJSON line reader and nothing else.

### Single biggest caveat

**Billing is wall-clock on memory, and Hobby caps sessions at 45 minutes.** A 30-minute agent run bills *full provisioned memory for the whole 30 minutes* even while the agent is blocked waiting on an LLM — only CPU is "Active CPU" metered. A 4 vCPU / 8 GB run costs ≈ **$0.11 idle-ish, ≈ $0.21 at 50% CPU, ≈ $0.34 at full tilt**. And the 10–60 minute range in the ticket straddles a plan wall: **Hobby max session duration is 45 minutes**, so 60-minute runs require the paid Pro plan ($20/mo base). Vercel is cheap per-run but it is a *paid, metered, vendor-locked* dependency, and our idle timeout cannot be allowed to drift past the session deadline.

### Second-order risk (state honestly)

The wire protocol is a bespoke Vercel NDJSON dialect, not SSE. It is well-defined and the SDK's recovery logic (reconnect + byte-offset skip) is the reference for what a correct client must do — but it is **underdocumented outside the SDK source**. See "Top unknowns" below.

---

## 1. REST surface — ✅ ANSWERED

Base URL: the REST docs use `https://api.vercel.com`. **The SDK actually uses `https://vercel.com/api`** (`packages/vercel-sandbox/src/api-client/api-client.ts:165` → `baseUrl: params.baseUrl ?? "https://vercel.com/api"`, and its own tests assert paths like `/api/v2/sandboxes/drives/workspace`). Both are live; treat `https://api.vercel.com` as canonical and *verify the other in a spike*.

Auth header shape (from `api-client/base-client.ts:60-62`):

```
Authorization: Bearer <VERCEL_TOKEN>
Content-Type: application/json
```

Scope via query params on every request: `?teamId=team_xxx` or `?slug=my-team`. No project scope needed on sub-resources — project comes from the token+team binding.

### Endpoints (all verified present in the OpenAPI spec)

| Operation | Method + Path |
|---|---|
| Create sandbox | `POST /v4/sandboxes` (also `POST /v3/sandboxes`, `POST /v2/sandboxes`) |
| Get sandbox | `GET /v2/sandboxes/{name}` |
| Update sandbox | `PATCH /v2/sandboxes/{name}` |
| Delete (destroy) | `DELETE /v2/sandboxes/{name}` |
| **Exec (fire & forget)** | `POST /v2/sandboxes/sessions/{sessionId}/cmd` |
| **Exec (streaming)** | `POST /v2/sandboxes/sessions/{sessionId}/cmd` with `wait:true, logs:true` → NDJSON |
| **Read output (stream)** | `GET /v2/sandboxes/sessions/{sessionId}/cmd/{cmdId}/logs` → NDJSON |
| Get command / long-poll wait | `GET /v2/sandboxes/sessions/{sessionId}/cmd/{cmdId}[?wait=true]` |
| Kill command | `POST /v2/sandboxes/sessions/{sessionId}/cmd/{cmdId}/kill` |
| Write stdin | `POST /v2/sandboxes/sessions/{sessionId}/cmd/{cmdId}/stdin` |
| Read file | `POST /v2/sandboxes/sessions/{sessionId}/fs/read` |
| Write files | `POST /v2/sandboxes/sessions/{sessionId}/fs/write` (gzip tar body, `x-cwd` header) |
| Make directory | `POST /v2/sandboxes/sessions/{sessionId}/fs/mkdir` |
| Stop session (kill VM) | `POST /v2/sandboxes/sessions/{sessionId}/stop` |
| Extend timeout | `POST /v2/sandboxes/sessions/{sessionId}/extend-timeout` |
| List commands | `GET /v2/sandboxes/sessions/{sessionId}/cmd` |
| Snapshot / fork | `POST /v2/sandboxes/sessions/{sessionId}/snapshot`, `POST /v2/sandboxes/{name}/fork` |

### Create

```bash
curl -sS -X POST "https://api.vercel.com/v4/sandboxes?teamId=$VERCEL_TEAM_ID" \
  -H "Authorization: Bearer $VERCEL_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-sandbox",
    "source": {
      "type": "git",
      "url": "https://github.com/acme/widgets.git",
      "revision": "feature/my-branch",
      "depth": 50
    },
    "image": "vercel/sandbox/universal",
    "resources": { "vcpus": 4 },
    "timeout": 3600000,
    "persistent": false,
    "env": { "GITHUB_TOKEN": "ghp_xxx" }
  }'
```

Response (`200`), fields you need:

```json
{
  "routes": [],
  "sandbox": { "name": "my-sandbox", "currentSessionId": "sbx_...", "cwd": "/vercel/sandbox", "status": "running", "timeout": 3600000, "persistent": false },
  "session": { "id": "sbx_123a6c5209bc3778245d011443644c8d27dc2c50", "status": "running", "cwd": "/vercel/sandbox", "timeout": 3600000, "vcpus": 4, "memory": 8192, "region": "iad1" }
}
```

`session.id` is the `sessionId` for every subsequent call. `cwd` defaults to **`/vercel/sandbox`**.

### Kill a command

```bash
curl -sS -X POST "https://api.vercel.com/v2/sandboxes/sessions/$SID/cmd/$CID/kill?teamId=$VERCEL_TEAM_ID" \
  -H "Authorization: Bearer $VERCEL_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"signal": 15}'      # 15 = SIGTERM, 9 = SIGKILL. Required field, numeric POSIX signal.
```

### Stop + destroy

```bash
# stop the VM (terminates all processes in the session; cannot be restarted)
curl -sS -X POST "https://api.vercel.com/v2/sandboxes/sessions/$SID/stop?teamId=$VERCEL_TEAM_ID" \
  -H "Authorization: Bearer $VERCEL_TOKEN"

# destroy the named sandbox + all its sessions (keeps snapshots)
curl -sS -X DELETE "https://api.vercel.com/v2/sandboxes/my-sandbox?teamId=$VERCEL_TEAM_ID" \
  -H "Authorization: Bearer $VERCEL_TOKEN"
```

---

## 2. Does `exec` STREAM stdout line-by-line? — ✅ ANSWERED: **YES, DECISIVELY**

**This is documented in the OpenAPI spec, not inferred.** From `POST /v2/sandboxes/sessions/{sessionId}/cmd`:

> `logs` — *"If true, stream the logs of the command execution in real-time via ND-JSON. This is only applicable if `wait` is also true."*
> `wait` — *"If true, returns an ND-JSON stream that emits the command status when started and again when finished."*

Response content type: `application/x-ndjson`.

### Two streaming shapes

**A. Single-connection streaming (best fit for us).** `POST .../cmd` with `wait:true, logs:true`.

```bash
curl -N -X POST "https://api.vercel.com/v2/sandboxes/sessions/$SID/cmd?teamId=$VERCEL_TEAM_ID" \
  -H "Authorization: Bearer $VERCEL_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
        "command": "sh",
        "args": ["-c", "claude -p \"fix the bug\""],
        "cwd": "/vercel/sandbox/widgets",
        "env": {},
        "sudo": false,
        "wait": true,
        "logs": true,
        "timeout": 1800000,
        "attachStdin": false
      }'
```

Frame sequence (verified against `api-client.ts:465-534`):

1. `{"command":{...}}` — command started, gives you `command.id`
2. `{"stream":"stdout","data":"<chunk>"}` / `{"stream":"stderr","data":"<chunk>"}` — repeated, live
3. `{"stream":"error","data":{"code":"...","message":"..."}}` — stream-level failure, **terminates the stream**
4. `{"command":{...,"exitCode":0,"durationMs":1234}}` — terminal frame, command finished

**B. Detached + separate log stream.** `POST .../cmd` (no `wait`) returns JSON `{command:{...}}` immediately; then `GET .../cmd/{cmdId}/logs` streams NDJSON indefinitely; `GET .../cmd/{cmdId}?wait=true` long-polls for the exit code.

### ⚠️ Critical caveat: chunks are NOT line-delimited

The transport is NDJSON (one JSON object per line) but **the `data` string inside is an arbitrary-size raw byte chunk, not one line.** Proof: the SDK's reconnect logic tracks `delivered = { stdout: 0, stderr: 0 }` as **character counts** and slices `parsed.data.slice(skipped)` to de-duplicate across a resumed stream (`api-client.ts:1098-1116`). You can only do that if chunks are arbitrary-length.

**Consequence for us:** we must buffer and split on `\n` ourselves, keeping a partial-line tail — exactly what `sandcastle/src/sandboxes/vercel.ts:191-210` already does with its `partial` variable. Sandcastle's `onLine` design is correct for this API. Do not assume `data === one line`.

### Idle timeout: fully under our control

Because the stream is a live long-lived HTTP response, an idle timeout is just "no NDJSON frame received for N seconds" in our own read loop. No server-side idle concept interferes. The SDK even disables undici's `bodyTimeout: 0` with the comment *"disable body timeout to allow long logs streaming"* (`base-client.ts:16-18`) — confirming these connections stay open indefinitely. **In Rust, `reqwest` streams response bodies incrementally by default, so this works with no special setup. Just do not set a read timeout.**

**Sources:** [stream-command-logs](https://vercel.com/docs/rest-api/sandboxes/stream-command-logs) · [execute-a-command](https://vercel.com/docs/rest-api/sandboxes/execute-a-command) · SDK `api-client.ts`

---

## 3. Long-running processes — ✅ ANSWERED

### Per-command timeout
From the OpenAPI schema for `POST .../cmd`:

> `timeout` — *"Maximum duration in milliseconds the command may run before it is killed with **SIGKILL**, up to **5 hours**. Enforced at exec time, independently of `wait`."*
> `type: integer`, `minimum: 100`, `maximum: 18000000`

So: **max 5 hours per command**, killed with **SIGKILL** (not SIGTERM — no cleanup hook). Set it explicitly on every exec; there is no documented default for this field.

### Sandbox/session timeout
- `timeout` on create: ms, min 1000. **Default 5 minutes.**
- Max session duration: **Hobby 45 min · Pro 24 h · Enterprise 24 h.**
- Extensible mid-run: `POST /v2/sandboxes/sessions/{sessionId}/extend-timeout` with `{"duration": <ms to ADD>}` (min 1000). Documented as "Extends the maximum execution time of a running session."
- The limit applies **per session, not per sandbox** — a persistent sandbox that stops and resumes gets a fresh window, so total lifetime is effectively unbounded.

### What happens on timeout — ⚠️ PARTIALLY UNKNOWN

- **Command timeout** → SIGKILL. Clear.
- **Session timeout** → the VM stops automatically. **Whether in-flight commands get any grace period or a distinct exit code is UNKNOWN.** If the session dies mid-agent, the log stream just closes; there is no documented "session_expired" terminal frame in the NDJSON schema (the only documented stream-error code is `sandbox_stream_closed`). A robust client must treat *any* stream end without a terminal `{"command":{...,"exitCode":...}}` frame as an unknown-fate failure and reconcile by re-fetching the sandbox state.
- If the sandbox is `persistent: true` (the default!) the filesystem is snapshotted on stop, so work is not lost. If `persistent: false`, it is gone.

**Practical guardrail:** create with `persistent: true` (cheap insurance), set `timeout` to ~1.5× the expected agent duration, and heartbeat `extend-timeout` on every streamed output frame. Never let the deadline be the thing that ends a run.

---

## 4. `source: { type: "git", url, revision }` — ✅ ANSWERED

**Yes, the sandbox clones the repo itself.** From the v4 create schema:

```json
{
  "type": "object",
  "description": "Clone a Git repository into the sandbox.",
  "required": ["type", "url"],
  "additionalProperties": false,
  "properties": {
    "type":   { "description": "Indicates the source is a Git repository." },
    "url":    { "format": "uri", "example": "https://github.com/vercel/next.js.git" },
    "username": { "description": "Username for Git authentication. Required together with password for private repositories." },
    "password": { "description": "Password or personal access token for Git authentication. Required together with password for private repositories." },
    "depth":    { "type": "integer", "minimum": 1, "description": "Create a shallow clone with history truncated to the specified number of commits." },
    "revision": { "type": "string", "description": "The specific commit SHA, branch name, or tag to checkout after cloning.", "example": "main" }
  }
}
```

**`revision` explicitly accepts a commit SHA, a branch name, or a tag.** A Builder can be pointed at a PR head SHA in the same call that creates the sandbox. Sandcastle's pass-through of `source` is fully supported.

Private repos: `username` + `password` (a GitHub fine-grained PAT, classic PAT, or GitHub App installation token — the SDK's own `examples/private-repo/private-repo.ts` uses `username: 'x-access-token'`).

### ⚠️ Where does the clone land? — **UNKNOWN (inferred)**

The schema does not say. Evidence points to **a subdirectory of `/vercel/sandbox` named after the repo**: the SDK's `examples/private-repo/private-repo.ts` clones `https://github.com/vercel/some-private-repo.git` and then runs `cwd: 'some-private-repo'` (a *relative* path, and relative paths resolve against `/vercel/sandbox`).

**This means Sandcastle's hardcoded `VERCEL_REPO_PATH = "/vercel/sandbox/workspace"` (`sandcastle/src/sandboxes/vercel.ts:23`) is probably wrong** — it looks like a leftover from the old deprecated `runtime: "node22"` images where a fixed `workspace` dir existed. Under today's image-based sandboxes the path is repo-name-derived. **Verify with a one-line spike (`ls /vercel/sandbox` after create) before trusting any path.** Note `create` does return `session.cwd`, so read it rather than assuming.

---

## 5. Getting commits back out — ✅ ANSWERED (with a caveat)

**There is no Vercel-side push primitive.** Vercel is not a git host; it has no opinion about your commits. Two workable paths:

**Option 1 — give the sandbox a push token (recommended).** We control `env` at create time and per-command. The universal image ships `git`, `gh`, and `git-lfs` and has passwordless `sudo`. So:

```bash
# at create time
-d '{"env": {"GITHUB_TOKEN": "ghp_xxx"}}'
```

The agent commits and `git push`s with that token. Commits keep their real SHAs — **no `format-patch`/`am` rewrite**, which sidesteps the entire `syncOut` class of problem. Network egress defaults to `allow-all`, so the push goes out. This is strictly better than patch extraction.

**Option 2 — patch extraction.** Keep Sandcastle's `syncOut` (format-patch + `am --3way` + diff + untracked). Works, but rewrites SHAs and is fiddly. Per-file extraction is available via `POST .../fs/read`.

**Neither requires the SDK.** Option 1 is a single `git push` exec.

**Sandcastle's `syncIn` (git bundle over `copyIn`) becomes largely unnecessary** — see §4. If the repo is public or we supply `username`/`password`, Vercel clones it for us. `syncIn` is only worth keeping for air-gapped/private-without-credential cases.

**Security caveat:** injecting a long-lived PAT into a sandbox running untrusted agent-generated code is a real exposure. Prefer a short-lived, repo-scoped, single-branch-push token. Vercel does have `networkPolicy` with **credential brokering** (inject auth headers on matching requests via `transform`) as a more locked-down alternative, but that is aimed at outbound API keys, not git pushes.

---

## 6. Runtimes, images, system packages — ✅ ANSWERED

`runtime: "node22" | "node24" | "node26" | "python3.13"` is **deprecated**. Use `image`. The SDK constant `RUNTIMES` still lists the old names; the create schema's `runtime` field is marked deprecated in favour of `image`.

### Vercel Managed Images

| Image | Base | Contents |
|---|---|---|
| `vercel/sandbox/universal:latest` *(default)* | `vercel/sandbox/ubuntu` | Node LTS (24), Python 3.14, coding agents, utilities |
| `vercel/sandbox/node:22\|24\|26` | `vercel/sandbox/ubuntu` | Node (major pinned), pnpm |
| `vercel/sandbox/python:3.14` | `vercel/sandbox/ubuntu` | Python 3.14, pip, venv, uv |
| `vercel/sandbox/arch:latest` | `archlinux:latest` | Arch, yay/AUR, base-devel, git |
| `vercel/sandbox/ubuntu:latest` | `ubuntu:26.04` | Ubuntu + sudo |

All roll nightly; pin by tag or `@sha256:` digest. Dockerfiles are public at `github.com/vercel/sandbox/tree/main/images`.

### System packages in `universal` — ✅ confirmed by reading `images/universal/Dockerfile`

`gh` (GitHub CLI) · `git` · `git-lfs` (installed `--system`) · `ripgrep` · `jq` · `curl` · `wget` · `openssl` · `gnupg` · `rsync` · `sudo` (**NOPASSWD**, `Defaults !env_reset`) · `tmux` · `fzf` · `sqlite3` · `netcat-openbsd` · `procps` · `iproute2` · `nano`/`vim`/`less`/`tree` · `zip`/`unzip`/`xz-utils`/`bzip2` · `file`/`dnsutils`/`lsof`/`iputils-ping`

Languages: Node 24, pnpm 11, Python 3.14 + pip + uv, bun.

**Pre-installed coding agents:** `opencode-ai`, `@anthropic-ai/claude-code`, `@openai/codex`, `@earendil-works/pi-coding-agent` — all global. We could literally skip our own agent install step.

**Browsers: NOT in the universal image.** No Chrome/Chromium/Playwright. **UNKNOWN** whether an exposed port can be proxied to an externally-hosted browser, or whether a custom image with Playwright is required. Flag this if load-testing needs a real browser — it likely means building a custom image.

### Custom images — ✅ supported
Push any OCI image to Vercel Container Registry, reference it as `my-repo:v1`, `my-repo@sha256:...`, `team-slug/project-slug/my-repo:v1`, or `vcr.vercel.com/team-slug/project-slug/my-repo:v1`. VCR implements the Docker Registry API (works with docker/podman/buildah). Must be `linux/amd64` — VCR reports a `Preparing` → `Ready` state and create fails with `image_not_ready` until ready. **`ENTRYPOINT`/`CMD` are ignored.** If the Dockerfile sets `WORKDIR`, commands start there; otherwise `/`. `sudo` works, so `apt-get install` at runtime is also viable as a one-off.

---

## 7. Cost and quotas — ✅ ANSWERED

|  | **Hobby** (included) | **Pro** | **Enterprise** |
|---|---|---|---|
| Sandbox Active CPU | 5 h/month | $0.128/vCPU-hour | $0.128/vCPU-hour |
| Provisioned Memory | 420 GB-h/month | $0.0212/GB-hour | $0.0212/GB-hour |
| Sandbox Creations | 5,000/month | $0.60/1M | $0.60/1M |
| Data Transfer (egress) | 20 GB/month | included in flat-rate CDN | $0.15/GB |
| Snapshot Storage | 15 GB lifetime | $0.08/GB-month | $0.08/GB-month |
| **Concurrent Sandboxes** | **10** | 10,000 | 10,000 |
| **Max Session Duration** | **45 minutes** | **24 hours** | 24 hours |
| Max vCPUs / memory / disk | 4 / 8 GB / 64 GB | 8 / 16 GB / 64 GB | 32 / 64 GB / 64 GB |
| vCPU allocation rate | 20–40/min | 150 → 5,000/min | 150 → 5,000/min |
| Control-plane quota | 1,000 req/min | 10,000 req/min | 100,000 req/min |
| Deletion quota | 20 req/s (all plans) | | |

Rates are for `iad1`; **regional rates vary** for CPU/memory/network, identical for creations and snapshot storage.

### Cost of a 30-minute agent run

Using the docs' own 10%-CPU assumption (agent mostly blocked on LLM calls), 4 vCPU / 8 GB:

| Utilization | Active CPU | Memory (wall-clock) | Total |
|---|---|---|---|
| 10% (idle-ish agent) | $0.0256 | $0.0848 | **≈ $0.11** |
| 50% | $0.128 | $0.0848 | **≈ $0.21** |
| 100% (full test suite) | $0.256 | $0.0848 | **≈ $0.34** |

Plus $0.0000006 per creation (negligible). Plus snapshot storage if we leave `persistent: true` — a 64 GB NVMe disk is provisioned, but only the **logical used size** is billed at $0.08/GB-month.

**Key economic insight:** memory is billed for *wall-clock*, CPU only when *active*. The 10% CPU figure is the right default for an LLM-bound coding agent. At 30 min × 4 vCPU that's ~11 cents. At 60 min ~22 cents. Vercel is affordable; the plan gate and the metered-surprise risk are the real concerns, not unit economics.

**Hobby behaviour:** free within quota; exceeding it **pauses sandbox creation** until the next billing cycle — it does not bill you. That is a hard stop, not a soft one, so a Hobby-account prototype that runs 10+ sandboxes can silently break.

**Downloads are free.** "Data your sandbox downloads from the internet, such as packages, Git repositories, artifacts, and datasets, is free." So `npm install` / `pip install` / repo clone cost nothing. Egress is billable — relevant to the load-testing use case, where traffic *into* an exposed port is charged.

---

## 8. Concurrency — ✅ ANSWERED

- **Hobby: 10 concurrent sandboxes. Pro/Enterprise: 10,000.**
- Sandcastle's planner template wants **4 parallel** → comfortably within Hobby.
- Allocation rate limit is **dynamic**, not fixed: it ramps with sustained usage (+500 vCPUs/min per minute on Pro) and decays back after 10 minutes idle. Hobby starts at 20 vCPUs/min and maxes at 40 — with 4 vCPU sandboxes that is **5–10 concurrent creations per minute** on Hobby. A burst of 4 is fine.
- Control-plane ops (exec, read, write) are a separate fixed quota: 1,000 req/min on Hobby. Our exec pattern is ~1 POST + 1 long-lived log GET per command, so this is a non-issue even at high command counts.
- Deletions are capped at 20 req/s per team.

---

## Top unknowns (must be resolved by a spike, not by more doc-reading)

1. **The clone destination path.** The spec is silent; the SDK example implies `/vercel/sandbox/<repo-name>`. **Sandcastle's hardcoded `/vercel/sandbox/workspace` is probably stale and likely broken** under image-based sandboxes. *Resolve: create a sandbox from a git source, `ls /vercel/sandbox`.*
2. **Which base URL accepts a raw access token.** Docs say `https://api.vercel.com`; the SDK ships `https://vercel.com/api`. Both work for the SDK. Unverified that `api.vercel.com` accepts a plain `VERCEL_TOKEN` bearer for sandbox endpoints specifically. *Resolve: one authenticated curl.*
3. **Session-timeout behaviour mid-stream.** No documented terminal frame for session expiry; only `sandbox_stream_closed`. A stream that ends without an `exitCode` frame is ambiguous — killed, expired, or network-dropped? *Resolve: create with a 30 s timeout, sleep 60 s, observe.*

**Secondary unknowns:**
- Browser availability (no Chrome in any managed image) — matters for load testing.
- Per-chunk framing guarantees: are chunks bounded in size? Can a single `data` be megabytes? Affects our line-buffer sizing and memory.
- Whether the NDJSON stream survives a proxy/load-balancer idle gap. The SDK reconnects with byte-offset de-dup — we should replicate that, but the failure thresholds are tuned constants in their code, not documented.
- Whether `fs/read` has a size limit for large patch files (no documented cap found).

---

## Mapping to Sandcastle's `vercel.ts` — what a Rust port needs

| Sandcastle call | REST equivalent | Notes |
|---|---|---|
| `Sandbox.create(...)` | `POST /v4/sandboxes` | `persistent: false` for ephemeral runs; set `timeout` |
| `sandbox.mkDir(p)` | `POST .../fs/mkdir` `{"path": p}` | relative to `/vercel/sandbox` |
| `runCommand({stdout, stderr, cwd, sudo})` | `POST .../cmd` `wait:true, logs:true` | NDJSON; **buffer + split on `\n` yourself** |
| `result.exitCode` | final `{"command":{...,"exitCode":N}}` frame | |
| `sandbox.writeFiles([...])` | `POST .../fs/write` (gzip tar, `x-cwd`) | chunked; 1 MB request limit for stdin endpoint |
| `sandbox.readFileToBuffer({path})` | `POST .../fs/read` `{"path": p}` | returns null-ish if missing |
| `sandbox.stop()` | `POST .../sessions/{sid}/stop` then `DELETE /v2/sandboxes/{name}` | |
| — | `POST .../extend-timeout` `{"duration": ms}` | **not used by Sandcastle — we should add it** |
| — | `POST .../cmd/{cmdId}/kill` `{"signal": 15}` | **not used by Sandcastle — needed for idle timeout** |

**Two gaps in the Sandcastle wrapper worth noting:** it never sets the per-command `timeout` (so the 5-hour max applies, but there is no explicit bound), and it has no `extendTimeout` or `kill` equivalent. A Rust port should have both — `kill` is what makes a real idle timeout possible.

## Sources

- OpenAPI spec (authoritative, machine-readable): https://vercel.com/openapi.json
- Sandboxes REST index: https://vercel.com/docs/rest-api/sandboxes
- Stream command logs: https://vercel.com/docs/rest-api/sandboxes/stream-command-logs
- Pricing and quotas: https://vercel.com/docs/sandbox/pricing
- JS SDK reference: https://vercel.com/docs/sandbox/sdk-reference
- Working with Sandbox: https://vercel.com/docs/sandbox/working-with-sandbox
- Images: https://vercel.com/docs/sandbox/concepts/images
- Authentication: https://vercel.com/docs/sandbox/concepts/authentication
- Product overview: https://vercel.com/docs/sandbox
- SDK source: https://github.com/vercel/sandbox — `packages/vercel-sandbox/src/api-client/api-client.ts`, `packages/vercel-sandbox/src/api-client/base-client.ts`, `images/universal/Dockerfile`