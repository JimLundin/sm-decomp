# Translate and verify all four SRAM codec routines

Type: task
Status: open
Blocked by: 41
Triage: ready-for-agent

Use [the completed codec proposal](../run-20261013/research-19/proposed-children/03-complete-codec.md)
for SaveToSram, LoadFromSram, PackMapToSave and UnpackMapFromSave. One owner
claims persistence.rs/test/persistence.toml after full-state actual-C adapter41
exists. Preserve checksum-pair fallback, invalid-load mutation order, aliases,
unnamed tails/map holes and exactly one post-save disk request. Refuse unsafe
slot3, zero-count and bounds inputs without invoking unsafe C.

Use reference-C full WRAM/SRAM/raw-return/order fixtures and independent
controls. Live provenance and original-ROM route remain separate acceptance
requirements. Tests60s/merged3600s; commit source and fixture declarations,
publish translated/verified stages and complete retained evidence cleanup.

## Completion evidence

Pending source and actual-C fixture submission.
