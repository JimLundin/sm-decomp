# Freeze four canonical persistence companion routines

Type: research
Status: proposed; coordinator review/publication required
Origin: exact research19 claim; see [Answer](../Answer.md) and [children.toml](../children.toml).
No writable ownership, supported native scope or gameplay acceptance is granted.

The bounded companions are NewSaveFile81B2CB, LoadMirrorOfExploredMapTiles80858C,
SaveExploredMapTilesToSaved8085C6 and LoadFromLoadStation80C437. Their source
identities, partial-clear semantics and ROM extents are already recorded. Review
a sequential continuation for the same canonical persistence triplet/owner after
codec integration; do not create a parallel writable claim or expand room/UI/
gunship inventories. Public room/system hooks remain integrator-owned.

NewSaveFile preserves events/boss/loading/unnamed payload while setting99health/
resource/controller/options/debug defaults and clearing selected bitsets and
1792map bytes. Preserve duplicate item clears. LoadMirror copies256bytes for
full area and reads mapstation flag; SaveMirror copies256 and conditionally
performs overlapping16bit OR00FF atD908+area. LoadFromLoadStation consumes one
14byte table record, playerx=screenx+128+xoffset, playery=screeny+yoffset,
previous positions match, scroll0, low-byte area from room8F header and low-byte
minimap clear; door_bts is ignored. Refuse malformed full-area/record/header
bounds before any effects.

Freeze exactly these four routines, accepted room initialization dependencies,
same-owner files and genuine observed roots before implementation admission.
Conditional unrun gates reuse `cargo test -p port --locked --offline --test persistence`
(60s) and the adapter-registered persistence family-verify command in the manifest
(3600s). All targets remain proposed. Boot VerifySRAM stays13; signature stays20;
file/copy/clear/wrapper remains research18 successor. Cleanup follows the same
two-pass/source-retention/worktree/authentication contract.
