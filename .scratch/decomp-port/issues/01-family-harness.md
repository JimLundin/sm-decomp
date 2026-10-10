# Reusable family comparison and source inventory
Status: resolved
Blocked by: None
Triage: ready-for-agent

Build reusable compiled-C versus Rust family execution on canonical WRAM, source
identity checks, return/full-memory comparison and retained failing evidence.
Inventory actual C definitions with original addresses, dependencies and source
identity; do not label unverified routines accepted. Expose a CLI used by tickets
02 and 03, with wrong-output rejection. Own tools/port* and docs/DECOMP_PORT.md.

## Completion evidence

Implemented `tools/port_family.py` CLI, full-WRAM/return comparisons against
compiled actual pinned C, complete archive/source identity verification and
persistent build/mismatch evidence. `tools/port/source-inventory.json` records
5,354 actual definitions with original addresses where provided and dependency
pointers, all explicitly unverified. API/domain/verification scope and substantive
next gameplay frontier documented in `docs/DECOMP_PORT.md`.

TDD at the previously approved canonical WRAM routine seam: missing CLI failed,
then actual-bank known seed273 passed; added source inventory test failed before
implementation and now passes. Public CLI checks incorrect return and unrelated
upper-WRAM mutation rejection. `python3 -m unittest discover -s tools -p test_port_family.py`: 2 tests passed. Retained positive/negative evidence under
`/tmp/sm-port-harness-evidence/{correct,wrong-return,wrong-memory}`. System family
consumed this harness: 17 routines/475 cases offline-verified at
`/tmp/system-diff-final/20261010T194836.788691Z`. No live/original-ROM acceptance
claim is inferred from these offline comparisons.
