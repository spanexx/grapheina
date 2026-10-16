## What this is

<!-- feature pipeline or surgical change? new capability -> feature-pipeline. closing a gap against a locked decision -> surgical-change. -->

- [ ] **Origin:** `feature-pipeline` · `surgical-change` — and the pack or drift entry is named below
- [ ] **Drift entry** (`D-`__) linked, or *none — this is new capability*

## The gates

- [ ] Every line of logic commented, every function/class/enum commented above — [`commenting.md`](../docs/ethics/commenting.md)
- [ ] One job per module, no file grown past the cap — [`modular.md`](../docs/ethics/modular.md)
- [ ] Tests green, and the count stated in the description
- [ ] No secrets. Credentials come from the environment, never the file
- [ ] No CRLF. No diff that rewrites every line for no reason
- [ ] Feature docs updated in the same change if behaviour moved — [`docs/drift.md`](../docs/drift.md)

## Evidence

- What broke, and how you saw it
- How you verified the fix — the command and its output
- What you deliberately did **not** do, and why

<!-- A human says yes before anything lands. -->