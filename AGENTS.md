# Software Factory - Agent Guide

## IMPORTANT!!!
  ### DOs 
    - Use the `say` skill to communicate with me.
    - Use plain English; refer to `plain-english-first`.
    - Answer in chunks; refer to `answer-question`.
    - Run every git and build command through WSL Debian. Windows shell is for looking, not doing.
    - **The Dev Lane** - `docs/ethics/dev-lane.md` - two environments, one split. WSL does, Windows looks.
  
  ### BEFORE YOU COMMIT
    - Read `docs/ethics/commit.md`. It is the gate.
  
  
  ### BEFORE YOU WRITE ANY PIECE OF CODE
    - **I Only Accept modular Ethics** - `docs/ethics/modular.md` 
    - **Must Follow Implementation Ethics** - `docs/ethics/implementation.md`
    - **Commenting Is Not Optional** - `docs/ethics/commenting.md`
    - **Every Piece of Code Has An Origin** - `docs/ethics/origin.md` - it arrives through `feature-pipeline` or `surgical-change`. There is no third way in.
    - **ENVIONMENT AGNOSTIC** - Ensure the platform works across all environments, not just Windows/WSL. Reassess your previous decisions and revise any that limit cross-environment compatibility.


  ## DO NOTs
    - **DO NOT KISS MY ASS** - Don't be a yes-man. Challenge my bad decisions, flag conflicts with existing decisions, and defend established principles. Prioritize logic over pleasing me. Protect the project, not my ego.
    - **DO NOT USE WINDOWS GIT** - staging, diffing and committing go through WSL Debian. Windows git carries its own line-ending rules and will not agree with WSL git about what changed.