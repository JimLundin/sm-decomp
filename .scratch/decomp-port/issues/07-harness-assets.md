# Extend family harness for gameplay assets and captures
Status: resolved
Blocked by: 01
Triage: ready-for-agent

Support 32-bit routine values, validated immutable ROM data and captured full-WRAM
fixtures needed by Samus/enemy families. Validate canonical case domains before
unchecked C runs; hash original ROM/captured states and source identities. Retain
failure evidence and independent original expected values where available.

## Completion evidence

Implemented in `f9cb780` (`tools/port_family.py`, `tools/test_port_family.py`,
`docs/DECOMP_PORT.md`). Explicit uint8/uint16/int32/uint32 argument prototypes,
int32 raw return bits, pinned actual C support sources, authenticated immutable
ROM bytes, and captured full-WRAM files with mandatory hashes. Rust guarded
candidate admission executes before unchecked C and stops rejected domains.
`verify_family_evidence` authenticates retained artifact inventories, actual
full-WRAM/return equality, current source implementation identities and complete
nonempty coverage before live use. Shared memory module is unchanged by harness.

TDD: red missing captured identity validation, red C execution despite candidate
rejection, red missing typed arguments, red missing evidence verifier; each now
green. Seven public-interface tests passed after merging integration registration,
including actual signed Samus movement, actual sm_rtl ROM-support execution,
negative index rejection before C, and retained-output tampering rejection.
Retained rejected domain evidence: `/tmp/sm-port-07-evidence/domain`, with no C
output after candidate domain error. Existing family evidence requires fresh
regeneration to include artifact inventory and updated shared module identity.
