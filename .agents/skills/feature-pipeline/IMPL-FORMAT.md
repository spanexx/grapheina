# IMPL Format

Implementation plan template. Separate from PRD-TRD because:

- PRD-TRD is the **contract** (what + how it should work)
- IMPL is the **execution** (when, in what order, what proves each phase done)

## File Naming

`docs/features/<slug>/IMPL-<slug>.md`

## Sections

```markdown
# IMPL: <Feature Name>

**Slug:** <slug>
**Status:** [Draft | Approved | In Progress | Complete]
**Date:** YYYY-MM-DD

## Phase Plan

<Ordered phases. Each builds on previous. Mark blockers.>

**Before Phase 1:** if the pack adds any new external dep, run `opensrc` skill first and record findings in the "Dependency Analysis" section below.

### Phase 1: <name>

**Build:**
- <file or module to create>
- <contract to satisfy (cite PRD-TRD section)>

**Verify:**
- [ ] <test or check — concrete, runnable>
- [ ] <another check>

**Blocked by:** <previous phase or external dep, or "nothing">

### Phase 2: <name>
...

## Phase Dependencies

<Diagram or list of inter-phase deps. Optional — useful when phases have non-obvious ordering.>

## Test Strategy

<What kind of tests, where they live, how to run them. Reference packages/<slug>/src/__tests__/.>

## Dependency Analysis (opensrc)

**Required before any phase that adds a new external dependency.** For every dep:

- **Name + version:** `<package>@<version>`
- **License:** <MIT, Apache-2.0, etc.>
- **Maintenance status:** <active | maintenance | abandoned>
- **Why this dep:** <what it solves, what we'd write ourselves without it>
- **Alternatives considered:** <other libs evaluated>
- **Call pattern:** <we use it via X, not the way their README shows>

Skip this section if the pack has no new deps.

## Rollout

<How this ships. Migration strategy if replacing existing behavior. Flag flips if applicable.>

## Risk Notes

<Anything that could derail a phase. Things the next agent should know.>
```

## Anti-Patterns

- **Don't describe the design** — that's PRD-TRD. Cite PRD-TRD section instead.
- **Don't list behavioral scenarios** — those are in PRD-TRD.
- **Don't put vague ACs** like "code is clean" — every check must be verifiable.
- **Don't skip the test strategy** — even if it's "follow TDD per package convention."

## Length Target

Aim for 60-120 lines. A pack with 4-5 phases fits comfortably. If you exceed 350 lines, you probably need to split the pack.

## Status Updates

As phases complete, mark them inline:

```markdown
### Phase 1: types — ✅ Complete 2026-07-28
### Phase 2: store — 🔄 In Progress
### Phase 3: engine — ⏳ Pending
```

Then update `docs/Feature_Backlog.md` after each phase completes.