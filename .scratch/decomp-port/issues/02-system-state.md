# Port system state family
Status: resolved
Blocked by: None
Triage: ready-for-agent

Port ReleaseButtonsFilter, PrepareBitAccess, boss/event set/clear/check,
Multiply16x16, HandleFadeIn/Out, ClearUnusedOam, ClearOamExt and three map-fill
routines plus WriteLotsOf0x1c2f. Own crates/port/src/system.rs and related tests
and tools/port/system.json. Differential verification depends on ticket 01.
Declare valid domains and preserve scratch writes, signed comparisons and aliasing.

## Completion evidence

Implementation eb0f175 merged in 509d78a: 17 routines, eight independent Rust
behavioral tests, 475 actual-C fixtures. Exact-source differential evidence:
/tmp/system-diff-final/20261010T194836.788691Z/verdict.json.
Live coverage is separately recorded under ticket04.
