# 05 — How does the Scout report load capacity honestly?

**Type:** grilling
**Status:** open
**Blocked by**: 10 (probe matrix)
**Claimed by**: —

## Question

When someone asks "how many people can this handle?", how does the Scout answer — on whatever
machine it happens to be running, at whatever quality, **without ever spending money without
asking first?**

## Why this ticket was rewritten

It opened as: *"how many concurrent users can this machine actually push with k6?"* — a
measurement task.

That was the wrong shape. It made a local measurement look like a requirement, when in fact:

- **It depends on the person running it.** Some machines can push thousands, some hundreds.
  A number that is right for one host is wrong for another.
- **The person decides.** Local is free and limited; cloud costs money and is not.
  Neither is the default.
- **It is not needed early.** The Scout's crash and usability probes work without it.
  It belongs after deployment, not before.
- **It is a capability, not a feature.** The same way the sandbox is a capability.

> Victor: *"this kind of capability should be a plugin to the system, just in case they need it."*

## The design it actually is

**Load capacity is a provider**, with the same shape as the sandbox provider:

```
   LoadForge (cloud)  ─┐
   cloud k6         ───┼──▶  ceiling { max_users, where_it_broke,
   local k6         ───┤              what_more_would_cost }
   nothing          ───┘
```

Three questions worth settling, none of which is "what is the number":

### 1. The fallback order

The Scout tries the cheapest first, because that is the one that is always available.

| | |
|---|---|
| **Local** | Free. Always available. Hits a ceiling fast. |
| **Cloud** | Costs money. Arrives at a true number. **Requires permission.** |
| **None** | "I cannot tell you from here." A valid answer, not a failure. |

**Is local-first right?** My view: yes. A free, honest ceiling beats a paid, precise one.

### 2. The rule about money

> **The Scout never spends money without asking.** Always.

This is the only rule here that is not negotiable, and it should be checked the same way ticket
15 checks credentials — mechanically, not by good intentions. An autonomous loop with a payment
method attached and no gate is a different project entirely.

### 3. What "honest" means

- The number is **measured**, never estimated. If nothing measured it, say nothing.
- The number comes with **where it broke**, not just a total — "1600 users, 4 second waits" is
  useful; "1600" alone is not.
- The number comes with **what more would cost**, so the person can decide whether it matters.
- **Local numbers are about the machine and the app together.** The Scout should say which of the
  two ran out first, because that changes the advice entirely: if the crowd simulator was the
  wall, the app may handle more than reported.

### 4. What does the Scout do with a ceiling it cannot reach?

Options: report it and stop · report it and continue with what it got · skip the probe entirely
and say why.

My view: **report and stop.** A stress result below the ceiling is not a stress result.

## Costs, for the report (verified September 2026)

| Service | Cost | Notes |
|---|---|---|
| **k6 free** | **$0** | 500 virtual-user-hours/month — 1,000 users for 30 min |
| **k6 paid** | $19/month + **$0.15**/user-hour | volume discounts |
| **LoadForge** | **$67 / $242 / $417** per month | **per-minute, no user cap** · 10k / 50k / **250k** concurrent |
| Artillery | $0 / $199 / $499 per month | load runs in your own AWS/Azure |
| Gatling | €89 / €356 per month | 60k / 180k virtual users |

**LoadForge's per-minute billing is the answer to "millions of users"** — a million costs the
same as ten.

**The free tier is enough for a real test.** 1,000 users for half an hour, every month, costs
nothing.

## Still worth knowing, when we get to it

Not needed for the design, and not needed now:

- What this machine's actual ceiling is
- Whether the rule of thumb is cores or memory
- How many concurrent browser contexts fit (feeds Playwright, ticket 04)

These are calibration numbers. They improve the report. They do not decide the design.

## Deliverable

The `LoadProvider` shape, the fallback order, the money gate, and what counts as an honest
number. **Not** a measured value — that is a run, not a decision.