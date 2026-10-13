# Initial setup compatibility failure

Before any actual Codex session, claim or authority policy was created,
`decomp-drill setup` rejected `git init -b drill-main` because the installed
Git is version 2.25.1. The isolated path held only authored fixture files.
The compatible recipe uses `git init` followed by `git checkout -b drill-main`.
The frozen engine source and binary remain unchanged. The harness is rebuilt
and its new digest is recorded before the actual session launch.
