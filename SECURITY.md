# Security Policy

## Supported versions

GRAPHEINA is pre-alpha. There is no released version. Everything is fixed on `master`.

| Version | Supported |
|---|---|
| `master` | yes |
| anything older | no |

There is nothing to backport a fix to, and pretending otherwise would be a lie in a security notice.

---

## The threat model you should know about

This project plans to run **agents that hold credentials and can merge code**. That is the whole
point of it, and it is also the sharp edge. The design answers it as follows:

- **A Repro is executable code built from untrusted output.** The frequent case is *careless*, not
  *evil* — an agent that wrote a bad script. Approved tests run in CI, which is an accepted
  privilege escalation: the repository is not damaged, the script simply moves somewhere more
  powerful.
- **Credentials are forbidden outright, not scanned for.** A test that needs to sign in reads from
  the environment. There is nothing in the file to leak.
- **The Reviewer is the interlock.** It replays the Repro before it reads the diff, and it runs on a
  *different model* on purpose — the same model with different context inherits the Builder's blind
  spots, which is the exact failure the Reviewer exists to prevent.
- **An agent may block only by naming a line and a reason.** A vague block gets disabled. A gate that
  gets clicked past is worse than no gate.
- **The Doctor never acts.** It reports a missing tool and recommends the install. It does not
  install it. Blast radius decides what needs asking a person, not certainty.

If you find a hole in any of that, that is a serious report, not a nitpick.

---

## Reporting a vulnerability

**Do not open a public issue.**

Use GitHub's private reporting — Security → Report a vulnerability on
[this repository](https://github.com/spanexx/grapheina/security/advisories/new).

If advisories are unavailable to you, email `spanexx@users.noreply.github.com` instead.

Please include: what you found, what an attacker gains, how to reproduce it, and which version or
commit. A working proof beats a theory.

### What to expect

- An acknowledgement within a few days.
- An assessment, and an honest severity, before a fix exists.
- Credit in the advisory if you want it.

There is no bug bounty and no SLA. Saying so now is better than pretending later.

### What is not a vulnerability

- The Scout running arbitrary code. That is the design, bounded by the sandbox and the Reviewer.
- Sandcastle's architecture decision records. They are a published risk register of bugs already
  paid for.
- Anything reachable only from a developer's own machine.

---

## Secrets

If you believe a credential has been committed — yours or anyone's — **say so immediately**, through
the private channel above, before opening a pull request. Commit history is the part you cannot
clean up by deleting a file.