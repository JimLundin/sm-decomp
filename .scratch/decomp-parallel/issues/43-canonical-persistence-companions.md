# Implement the same-owner canonical persistence companions

Type: task
Status: open
Blocked by: 41, 42
Triage: ready-for-agent

Use [the finished companion contract](../run-20261013/research-19/proposed-children/04-canonical-companions.md)
for NewSaveFile, LoadMirrorOfExploredMapTiles, SaveExploredMapTilesToSaved and
LoadFromLoadStation. Continue the codec owner's exact canonical state triplet;
no competing persistence file ownership. Preserve partial clears/defaults,
overlapping map ORs and station/room/position semantics. Establish supported
full-state fixture gates before edits. Integration tests are tasks; no broad
room/UI inventory research is assigned. Independent stages and cleanup required.

## Completion evidence

Pending same-owner source and fixtures.
