# Software Factory - Dev Lane

## IMPORTANT!!!
  Two environments. One is for looking. One is for doing.

  ### The Split
    - **Windows** - reading, planning, looking. Where `AGENTS.md` lives.
    - **WSL Debian** - editing, building, testing, and every git command.

  ### Rules
    - No build on Windows. Cargo is not on the Windows PATH, deliberately.
    - Every git command runs in WSL: staging, diffing, committing, pushing.
    - Worktrees live in WSL, outside the repository, created only by `agent-worktree`.
    - Edit over SSH, not over the Windows mount. Same files, native speed.
    - Skills live at `~/.agents` on both sides. In WSL it is a link to the Windows
      copy, so there is one truth and neither side drifts.
    - Write WSL files through `\\wsl.localhost\Debian\...`. Reach the filesystem
      directly. A temp script per file is noise.

  ### The cargo trap
    - `cargo` is missing from a plain `bash script.sh` and present in `bash -lc`.
      Use the login shell. Cargo is not broken. Install nothing.

  ### Why the split
    - The Windows mount is slow. The map already calls it the worst case.
    - Two gits disagree about line endings. One lane removes the question.
    - A thing that only runs on this machine is not finished. See `origin.md`.

  ## DO NOTs
    - **DO NOT BUILD ON WINDOWS** - if cargo is missing there, that is the design, not a fault.
    - **DO NOT WRAP COMMANDS YOU DO NOT NEED TO WRAP** - inside WSL, run the command.
    - **DO NOT HAND-CREATE A WORKTREE** - `agent-worktree` only, and it refuses a dirty tree.
