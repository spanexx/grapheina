---
name: grill-with-docs
description: Grilling session that locks requirements via Q&A. Output is plain text (.txt), records every question the user asked (verbatim), the locked answer, the why, and the source. Use before any PRD work — this is the gate that decides what gets built.
---

# grill-with-docs

Interview the user relentlessly until scope, non-goals, and edge cases are stable. Output a **plain text file** (not Markdown) that captures **both sides of every decision**: the original question the user asked (verbatim), and the answer with its reasoning.

Plain English first. Real-world scenarios, not abstractions. One question at a time — wait for the answer before asking the next.

## When to Use

- Before writing any PRD for a new feature
- When scope reverses (terms shift, non-goals change, dependencies resurface)
- When grilling produces decisions that affect other packs

## When NOT to Use

- The decision is already locked in `CONTEXT.md` or a previous GRILL
- The user already gave a complete spec (no questions to ask)
- Implementation is mid-flight — finish it first

## Output Format

Plain text file: `docs/features/<slug>/GRILL-<slug>.txt`

The doc is the **decision record**, not the conversation transcript. Every question answered during the session gets a numbered entry below. Matches the existing convention used across all sibling packs.

```
<Feature Name> - Grill Notes

Status: <Phase 0 grilling in progress | Locked>
Date: YYYY-MM-DD
Source: <path to feature backlog or upstream doc>

What this feature is about

- <bullet: what the feature does>
- <bullet: what it does NOT do>
- <bullet: scope clarifications>

Questions and answers locked so far

1. <Question — verbatim, as the user phrased it>
Answer:
- <the locked answer, in plain English>
- <supporting bullet if needed>

Why:
- <rationale, trade-offs, what this rules out>
- <cite source: file:line | "from CONTEXT.md" | "from sibling pack <slug>">

2. <Next question, verbatim>
Answer:
- ...

Why:
- ...

No remaining open questions

<When all questions resolve, add this section verbatim.>
```

## What to Document

For every question that gets answered during the session, record **four things**:

1. **The question verbatim** — the user's exact wording. Don't paraphrase, don't rephrase to sound technical. If they said "what about permissions?", that's the question — even if it's informal.
2. **Your recommended answer** — concrete, plain English. State it up front.
3. **The why** — the rationale, the trade-off, what this answer rules out.
4. **Source citations** (when relevant) — file:line, URL, or "inferred" / "from CONTEXT.md" / "from sibling pack <slug>". Don't guess — cite or admit you don't have one.

If the user pushes back or revises, record the **final** answer and note the previous version in passing. Don't keep dead branches — the doc reflects the locked state, not the conversation.

## Process

1. **Read CONTEXT.md first.** Don't ask about terms already defined. Reference them by name.
2. **Read sibling packs in `docs/features/`** for established patterns. Skip asking things the codebase already answers.
3. **One question at a time.** Group only when questions are independent and the user can answer in any order.
4. **For each question:**
   - State your recommended answer up front
   - Explain with a real scenario (operator, plugin author, runtime — whichever fits)
5. **Capture the original wording verbatim.** Use the user's exact phrasing for the question heading. Their language tells you what they actually meant.
6. **When stable:** write the .txt file, append the `No remaining open questions` section at the end, hand off to PRD+TRD.

## Drift

If a GRILL question's answer contradicts existing code or CONTEXT.md, surface it. "Your CONTEXT.md says X, but the code does Y — which is right?" Update CONTEXT.md if the answer is the code, or note the drift for later.

## What Goes In vs What Stays Out

| Include | Exclude |
|---------|---------|
| Every question the user actually asked, **verbatim** | Paraphrased or "cleaned up" questions |
| The locked answer + the why | Conversation history (debates, tangents) |
| Source citations for every claim | Implementation details (those go in IMPL) |
| Non-goals stated explicitly | Code structure or file layout |
| Edge cases the user named | Speculative future requirements |

## Why "Verbatim" Matters

The user's original phrasing is the most reliable signal of what they meant. When someone writes "what about permissions?" six months from now and reads the GRILL, they see the same informal question — and know it was a real concern, not an invented one. Rewriting it to "How are permissions scoped?" strips that signal and makes the doc feel like it was written by an engineer, not a human conversation.

If the user phrased it imprecisely, record the imprecise version and add a clarifying parenthetical — don't replace their words.

## After Lock

Run `feature-pipeline` skill, starting at Phase 1 (PRD+TRD). The GRILL becomes an appendix reference in the PRD-TRD.