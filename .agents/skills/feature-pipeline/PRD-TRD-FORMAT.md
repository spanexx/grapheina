# PRD-TRD Format

Merged PRD + TRD template. Single document holding the **why**, **what**, and **technical design** for a feature. IMPL is a separate doc.

## File Naming

`docs/features/<slug>/PRD-TRD-<slug>.md`

## Sections

```markdown
# PRD-TRD: <Feature Name>

**Slug:** <slug>
**Status:** [Draft | Approved]
**Date:** YYYY-MM-DD

## Why This Exists

<2-3 paragraphs: the problem, who hits it, what it costs to leave unsolved.
Plain English. Real scenarios. No jargon unless in CONTEXT.md.>

## Behavioral Spec

<What the operator or consumer sees. Concrete actions, expected outcomes.
Numbered scenarios (not prose). Each scenario is testable — you can run the
post-impl simulation against it.>

### Scenario 1: <name>

**Given** <preconditions>
**When** <action>
**Then** <observable result>

### Scenario 2: <name>
...

## Simulation Contract

<What the post-impl sim MUST demonstrate. This is the test harness for the
behavioral spec above. Each scenario maps to one or more sim commands.>

```bash
# Example: install a plugin should produce these state changes
install <manifest-yaml>
# → plugins[] has new entry
# → capabilities[] has new entries (called registry.register)
# → events[] has plugin.installed + capability.registered
```

## Technical Design

### Data Models

<Schemas, types, key shapes. Reference TypeScript interfaces in source when they exist.>

### API Contracts

<Public functions, event names, payload shapes.>

### Dependencies

<Every package this feature depends on. For each, opensrc findings: version, license, maintenance status, alternatives considered.>

### Architecture Notes

<Module boundaries, data flow, integration points. Diagrams welcome but not required.>

## Non-Goals

<What this feature does NOT do. Bullet list. Each item names the deferral reason.>

## Out of Scope (Future)

<Items the user might expect but are deferred to other packs.>

## References

- GRILL-<slug>.txt — locked decisions
- CONTEXT.md — glossary
- IMPL-<slug>.md — execution plan (separate doc)
- packages/<slug>/src/ — implementation
- docs/drift-issue-log.md — project-wide drift
```

## Anti-Patterns

- **Don't restate CONTEXT.md** — link to it instead
- **Don't describe execution order** in this doc — that's IMPL's job
- **Don't skip Simulation Contract** — it's the bridge between doc and sim
- **Don't list future enhancements** as goals — that's what Non-Goals is for

## Length Target

Aim for 80-150 lines for typical packs. Hard cap: 350 lines per the file-size rule. If you exceed it, extract architectural decisions to an ADR — but only if those decisions are genuinely hard to reverse.