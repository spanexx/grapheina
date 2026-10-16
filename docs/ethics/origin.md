# Software Factory - Origin Ethics

## IMPORTANT!!!
  Every piece of code must account for why it is there.
  There are exactly two ways in. There is no third.

  ### The Two Ways In
    - **`feature-pipeline`** - new work. GRILL, pre-sim, PRD-TRD, IMPL, build, post-sim, drift check.
    - **`surgical-change`** - closing a gap between a locked decision and the code. Driven by a drift entry D-N, or a bug.

  ### Pick One Before You Write A Line
    - New capability? `feature-pipeline`.
    - Closing a gap against something already decided? `surgical-change`.
    - Cannot tell? It is a feature. Surgery needs a locked decision behind it. No decision, nothing to fix against.

  ### The Trail
    - The comment on the code carries the pack slug or the drift D-N. See `commenting.md`.
    - Code with no trail is code nobody can justify. Treat it as a Finding and raise it.

  ## DO NOTs
    - **DO NOT JUST MAKE A CHANGE** - there is no in-between and no informal pass.
    - **DO NOT ADD CODE NO DECISION SITS BEHIND** - no drive-by improvements, no opportunistic refactors, no "while I was in there".
    - **DO NOT CALL IT A FIX WHEN NO DECISION EXISTS** - a drift entry needs a locked decision behind it.
    - **DO NOT SKIP THE PROCESS BECAUSE THE CHANGE IS SMALL** - small is not a category that skips. Small means surgical, not nothing.