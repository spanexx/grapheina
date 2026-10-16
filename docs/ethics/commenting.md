# Software Factory - Commenting Ethics

## IMPORTANT!!!
  Commenting is not decoration. It is the handover between agents.
  A comment that says nothing is a comment that loses the work.

  ### The Two Comments
    - **Beside the code** - every line of logic, no matter how small.
    - **Above the thing** - every function, class, enum and enum variant.

  ### What A Comment Says
    - **Why it is there.** Not what it does. The code already says what.
    - **Which work it belongs to.** The feature-pack slug, or the drift entry D-N.
    - **Who changed it, and when.** The edit log, so no agent is left guessing.

  ### Shape
      // WHY:  <why it exists>
      // PACK: <feature slug | D-N>
      // 2026-10-16 opencode: <what changed, and why> [pack: <slug>] 

  ### DOs
    - Plain English. Short sentences. A term the reader does not already have gets explained.
    - Write the comment as you write the code, not as an afterthought.
    - Keep them current. A stale comment is worse than no comment.

  ## DO NOTs
    - **DO NOT LEAVE ANYTHING UNCOMMENTED** - not one line, not one small helper.
    - **DO NOT RESTATE THE CODE** - `i += 1` does not need "increment i". Say why it increments.
    - **DO NOT COMMENT OUT CODE** - delete it, or raise a drift entry.
    - **DO NOT EDIT WITHOUT LOGGING IT** - an unexplained edit breaks the trail between agents.
    - **DO NOT WRITE JARGON** - plain English, always.
